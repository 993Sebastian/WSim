//! Milestones of the player after the introduction (M23; docs/FORMELN.md). They give
//! the player a direction and have no effect on the simulation.

use std::collections::BTreeMap;

use crate::calendar::Date;
use crate::catalog::{Catalog, MilestoneCondition, SiteType};
use crate::ids::{CountryId, ProductId};
use crate::ledger::{Account, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, GameState};

/// How far the player is on the way to a milestone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Progress {
    pub current: f64,
    pub target: f64,
}

/// Marks the milestones the player reached by `date` and reports each once.
pub(crate) fn check(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    for (id, m) in catalog.milestones.iter() {
        if state.milestones.get(id).is_some() || !reached(state, catalog, m.condition, date) {
            continue;
        }
        *state.milestones.get_mut(id) = Some(date);
        news.push(Message::new(MessageKind::Success, keys::MILESTONE).with(
            "etappe",
            Param::TextKey(format!("etappe.{}", catalog.milestones.key(id))),
        ));
    }
    news
}

/// Whether the player meets a condition on `date` (the day just simulated, or today).
pub fn reached(
    state: &GameState,
    catalog: &Catalog,
    condition: MilestoneCondition,
    date: Date,
) -> bool {
    let player = state.player;
    let ledger = &state.companies[player.index()].ledger;
    match condition {
        MilestoneCondition::FirstSale => {
            std::iter::once(&ledger.year).chain(&ledger.years).any(|y| {
                y.by_type
                    .get(&CostType::Revenue)
                    .is_some_and(|&r| r > Money::ZERO)
            })
        }
        MilestoneCondition::ProfitMonth => ledger.months.iter().any(|m| m.total() > Money::ZERO),
        MilestoneCondition::OwnInput => own_input(state, catalog, player, date),
        MilestoneCondition::Research => !state.companies[player.index()].technologies.is_empty(),
        MilestoneCondition::Facilities(_)
        | MilestoneCondition::Countries(_)
        | MilestoneCondition::MarketLeader(_)
        | MilestoneCondition::Equity(_) => {
            progress(state, condition, date).is_some_and(|p| p.current >= p.target)
        }
    }
}

/// Progress towards a condition that has a measure; `None` for yes-or-no conditions.
pub fn progress(state: &GameState, condition: MilestoneCondition, date: Date) -> Option<Progress> {
    let player = state.player;
    let own = || state.sites.iter().filter(move |s| s.owner == player);
    let (current, target) = match condition {
        MilestoneCondition::Facilities(n) => {
            let units: u32 = own()
                .flat_map(|s| &s.slots)
                .filter(|sl| sl.ready <= date)
                .map(|sl| sl.count)
                .sum();
            (f64::from(units), f64::from(n))
        }
        MilestoneCondition::Countries(n) => {
            let mut countries: Vec<CountryId> = own().map(|s| s.country).collect();
            countries.sort_unstable();
            countries.dedup();
            // Few countries; the cast cannot overflow.
            (countries.len() as f64, f64::from(n))
        }
        MilestoneCondition::MarketLeader(share) => (best_lead(state, player), share),
        MilestoneCondition::Equity(factor) => {
            let ledger = &state.companies[player.index()].ledger;
            let equity = ledger.total_assets() - ledger.balance(Account::Loans);
            (
                equity.to_usd(),
                state.settings.start_capital.to_usd() * factor,
            )
        }
        MilestoneCondition::FirstSale
        | MilestoneCondition::ProfitMonth
        | MilestoneCondition::OwnInput
        | MilestoneCondition::Research => return None,
    };
    Some(Progress { current, target })
}

/// The player makes, with a running facility, what another running facility of its own
/// uses.
fn own_input(state: &GameState, catalog: &Catalog, player: CompanyId, date: Date) -> bool {
    let running: Vec<_> = state
        .sites
        .iter()
        .filter(|s| s.owner == player && s.kind != SiteType::ResearchCenter)
        .flat_map(|s| &s.slots)
        .filter(|sl| sl.operating(date))
        .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r)))
        .collect();
    running.iter().any(|made| {
        running
            .iter()
            .any(|user| user.inputs.iter().any(|&(input, _)| input == made.product))
    })
}

/// The player's best share of last month's sales of a product in a country where it
/// sold more than any other company (0 if it leads nowhere).
fn best_lead(state: &GameState, player: CompanyId) -> f64 {
    let mut best: f64 = 0.0;
    let markets: Vec<(ProductId, CountryId)> = state
        .sites
        .iter()
        .filter(|s| s.owner == player)
        .flat_map(|s| s.offers.keys().map(move |&p| (p, s.country)))
        .collect();
    for (product, country) in markets {
        let total = state.markets.get(product).get(country).last_month.sold;
        if total <= 1e-9 {
            continue;
        }
        let mut by_company: BTreeMap<CompanyId, f64> = BTreeMap::new();
        for s in state.sites.iter().filter(|s| s.country == country) {
            if state.companies[s.owner.index()].bankrupt {
                continue;
            }
            if let Some(o) = s.offers.get(&product) {
                *by_company.entry(s.owner).or_default() += o.sold_last_month;
            }
        }
        let own = by_company.get(&player).copied().unwrap_or(0.0);
        let leads = by_company
            .iter()
            .all(|(&c, &sold)| c == player || sold < own);
        if own > 1e-9 && leads {
            best = best.max(own / total);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::calendar::RoundLength;
    use crate::catalog::{Milestone, test_support};
    use crate::command::{self, Command};
    use crate::game::Game;
    use crate::save;
    use crate::state::{GameSettings, PriceMode, SaleOffer, SiteId, StartForm};

    fn usd(v: f64) -> Money {
        Money::from_usd(v).unwrap()
    }

    fn game_with(milestones: &[(&str, MilestoneCondition)]) -> Game {
        let mut catalog = test_support::production();
        for &(key, condition) in milestones {
            catalog
                .milestones
                .insert(key, Milestone { condition })
                .unwrap();
        }
        let catalog = Arc::new(catalog);
        let settings = GameSettings {
            seed: 3,
            start_year: 1900,
            start_country: catalog.countries.id("AAA").unwrap(),
            start_capital: usd(20_000_000.0),
            start_form: StartForm::Workshop,
            company_name: "Hütte AG".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: Default::default(),
        };
        Game::new(catalog, settings).unwrap()
    }

    fn found(game: &mut Game, country: &str, kind: SiteType) -> SiteId {
        let country = game.catalog().countries.id(country).unwrap();
        game.apply(Command::FoundSite { country, kind }).unwrap();
        SiteId(u32::try_from(game.state().sites.len() - 1).unwrap())
    }

    fn milestone_messages(messages: &[Message]) -> Vec<&Message> {
        messages
            .iter()
            .filter(|m| m.key == keys::MILESTONE)
            .collect()
    }

    #[test]
    fn a_milestone_is_reported_once_and_keeps_its_day() {
        let mut game = game_with(&[("zwei_anlagen", MilestoneCondition::Facilities(2))]);
        let id = game.catalog().milestones.id("zwei_anlagen").unwrap();
        let site = found(&mut game, "AAA", SiteType::Factory);
        let furnace = game.catalog().facilities.id("ofen").unwrap();
        game.apply(Command::BuildFacility {
            site,
            facility: furnace,
            count: 2,
            size: crate::catalog::FacilitySize::Medium,
        })
        .unwrap();
        let ready = game.state().sites[site.index()].slots[0].ready;
        assert_eq!(
            progress(game.state(), MilestoneCondition::Facilities(2), ready),
            Some(Progress {
                current: 2.0,
                target: 2.0
            })
        );

        let report = game.advance(RoundLength::Month, |_| {});
        let news = milestone_messages(&report.messages);
        assert_eq!(news.len(), 1);
        assert_eq!(news[0].kind, MessageKind::Success);
        assert_eq!(
            news[0].params,
            vec![(
                "etappe".to_owned(),
                Param::TextKey("etappe.zwei_anlagen".into())
            )]
        );
        assert_eq!(*game.state().milestones.get(id), Some(ready));
        let view = &crate::views::overview(&game).milestones[0];
        assert_eq!(view.key, "zwei_anlagen");
        assert_eq!(view.reached.as_deref(), Some(ready.to_string().as_str()));
        let p = view.progress.as_ref().unwrap();
        assert_eq!((p.current, p.target, p.unit.as_str()), (2.0, 2.0, "anzahl"));

        // Selling the furnaces does not undo it, and it is not reported again.
        game.apply(Command::SellFacility {
            site,
            slot: 0,
            count: 2,
        })
        .unwrap();
        let report = game.advance(RoundLength::Month, |_| {});
        assert!(milestone_messages(&report.messages).is_empty());
        assert_eq!(*game.state().milestones.get(id), Some(ready));

        // Saved by key, replayed from the journal.
        let loaded = save::decode(&save::encode(&game), game.catalog().clone()).unwrap();
        assert_eq!(loaded.game.state(), game.state());
        let replayed = Game::replay(
            game.catalog().clone(),
            game.state().settings.clone(),
            game.journal(),
        )
        .unwrap();
        assert_eq!(replayed.state(), game.state());
    }

    #[test]
    fn equity_counts_against_the_start_capital() {
        let mut game = game_with(&[("reich", MilestoneCondition::Equity(2.0))]);
        let date = game.state().date;
        let p = progress(game.state(), MilestoneCondition::Equity(2.0), date).unwrap();
        assert!((p.target - 40_000_000.0).abs() < 1e-6);
        assert!(p.current <= 20_000_000.0 + 1e-6);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        assert!(check(state, &catalog, date).is_empty());
        state.companies[0]
            .ledger
            .transfer(Account::Cash, Account::Equity, usd(25_000_000.0));
        assert_eq!(check(state, &catalog, date).len(), 1);
    }

    #[test]
    fn market_leadership_needs_the_largest_share() {
        let condition = MilestoneCondition::MarketLeader(0.3);
        let mut game = game_with(&[("fuehrung", condition)]);
        let own = found(&mut game, "AAA", SiteType::Factory);
        let catalog = game.catalog().clone();
        let iron = catalog.products.id("eisen").unwrap();
        let aaa = catalog.countries.id("AAA").unwrap();
        let date = game.state().date;
        // A competitor with its own works in the same country.
        let state = game.state_mut();
        let mut rival = state.companies[0].clone();
        rival.name = "Konkurrenz AG".into();
        state.companies.push(rival);
        let other = CompanyId(1);
        command::execute(
            state,
            &catalog,
            other,
            &Command::FoundSite {
                country: aaa,
                kind: SiteType::Factory,
            },
        )
        .unwrap();
        let theirs = SiteId(u32::try_from(state.sites.len() - 1).unwrap());
        let offer = |sold: f64| SaleOffer {
            mode: PriceMode::Market {
                markup: 0.0,
                floor: Money::ZERO,
            },
            price: usd(100.0),
            keep: 0.0,
            sold_today: 0.0,
            sold_month: 0.0,
            sold_last_month: sold,
            to_traders_month: 0.0,
            to_companies_month: 0.0,
        };
        state.markets.get_mut(iron).get_mut(aaa).last_month.sold = 100.0;
        state.sites[own.index()].offers.insert(iron, offer(40.0));
        state.sites[theirs.index()].offers.insert(iron, offer(50.0));
        assert!(!reached(state, &catalog, condition, date));
        assert_eq!(progress(state, condition, date).unwrap().current, 0.0);

        state.sites[theirs.index()].offers.insert(iron, offer(20.0));
        assert!(reached(state, &catalog, condition, date));
        assert!((progress(state, condition, date).unwrap().current - 0.4).abs() < 1e-9);

        // Leading with too small a share is not enough.
        state.markets.get_mut(iron).get_mut(aaa).last_month.sold = 1000.0;
        assert!(!reached(state, &catalog, condition, date));
    }

    #[test]
    fn countries_and_own_inputs() {
        let mut game = game_with(&[
            ("zwei_laender", MilestoneCondition::Countries(2)),
            ("vorprodukt", MilestoneCondition::OwnInput),
        ]);
        let catalog = game.catalog().clone();
        let date = game.state().date;
        let countries = MilestoneCondition::Countries(2);
        found(&mut game, "AAA", SiteType::Factory);
        found(&mut game, "AAA", SiteType::Factory);
        assert!(!reached(game.state(), &catalog, countries, date));
        found(&mut game, "BBB", SiteType::Factory);
        assert!(reached(game.state(), &catalog, countries, date));

        // Ore from an own mine for an own furnace.
        assert!(!reached(
            game.state(),
            &catalog,
            MilestoneCondition::OwnInput,
            date
        ));
        let state = game.state_mut();
        let mine = catalog.facilities.id("mine").unwrap();
        let furnace = catalog.facilities.id("ofen").unwrap();
        let slot = |facility, recipe: &str| crate::state::Slot {
            facility,
            ready: date,
            count: 1,
            cost: Money::ZERO,
            recipe: catalog.recipes.id(recipe),
            utilization: 1.0,
            automation: 0.0,
            condition: 1.0,
            batches: Vec::new(),
            last_runs: 0.0,
            limit: None,
            operation: crate::state::Operation::Running,
            size: crate::catalog::FacilitySize::Medium,
        };
        let site = state.sites.len() - 1;
        state.sites[site].slots.push(slot(mine, "erz_abbau"));
        assert!(!reached(
            state,
            &catalog,
            MilestoneCondition::OwnInput,
            date
        ));
        state.sites[site]
            .slots
            .push(slot(furnace, "eisen_schmelzen"));
        assert!(reached(state, &catalog, MilestoneCondition::OwnInput, date));
    }
}
