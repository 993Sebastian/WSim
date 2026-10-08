//! Brand awareness and advertising (Lastenheft §9.4, formulas in docs/FORMELN.md, M16).
//!
//! Awareness lies between 0 and 1 per company, country and goods group. It grows with
//! advertising and with the company's own sales of end products (word of mouth) and
//! fades without either. Consumers prefer known brands (`market`).

use std::collections::BTreeMap;

use crate::calendar::Date;
use crate::catalog::{Catalog, ProductKind};
use crate::ids::{CountryId, GoodsGroupId, Id};
use crate::ledger::{Account, CostCenter, CostType};
use crate::math;
use crate::money::Money;
use crate::state::{Brand, GameState};

/// Entries below this awareness without a budget are dropped.
const NEGLIGIBLE: f64 = 0.001;

/// Monthly advertising budget in USD that reaches the whole country once (K): it closes
/// 1 − e^(−w) of the awareness gap with a medium of effect w. Shrinks with the market
/// scale like the markets.
pub fn reach_usd(state: &GameState, catalog: &Catalog, country: CountryId) -> f64 {
    let values = state.countries.get(country);
    catalog.market_model.brand.cost_per_inhabitant_usd
        * values.market_population
        * values.price_level
}

/// Monthly step at the start of `date`'s month, after the sales counters of the closed
/// month moved to `sold_last_month`: advertising of the new month is paid and the
/// awareness follows advertising, sales and forgetting.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) {
    let model = &catalog.market_model.brand;
    let effect = model.medium(date.year()).map_or(0.0, |m| m.effect);
    // Sales of end products last month by country and goods group: in total and per
    // company.
    let mut total: BTreeMap<(CountryId, GoodsGroupId), f64> = BTreeMap::new();
    let mut own: BTreeMap<(usize, CountryId, GoodsGroupId), f64> = BTreeMap::new();
    for site in &state.sites {
        for (&product, offer) in &site.offers {
            let p = catalog.products.get(product);
            if p.kind != ProductKind::EndProduct || offer.sold_last_month <= 0.0 {
                continue;
            }
            let value = offer.sold_last_month * offer.price.to_usd();
            *total.entry((site.country, p.goods_group)).or_default() += value;
            *own.entry((site.owner.index(), site.country, p.goods_group))
                .or_default() += value;
        }
    }
    let reach: Vec<f64> = catalog
        .countries
        .ids()
        .map(|c| reach_usd(state, catalog, c))
        .collect();
    // A marketing department strengthens advertising (ZA2): the effect × (1 + W).
    let boost: Vec<f64> = (0..state.companies.len())
        // Few companies; the cast is exact.
        .map(|i| {
            crate::central::strength(
                catalog,
                state,
                crate::state::CompanyId(i as u32),
                crate::catalog::DepartmentKind::Marketing,
            )
        })
        .collect();
    for (index, company) in state.companies.iter_mut().enumerate() {
        let effect = effect * (1.0 + boost[index]);
        if company.bankrupt {
            company.advertising.clear();
            continue;
        }
        for ad in &company.advertising {
            if ad.budget > Money::ZERO {
                company.ledger.expense(
                    CostType::Marketing,
                    CostCenter::default(),
                    Account::Cash,
                    ad.budget,
                );
            }
        }
        let mut keys: Vec<(CountryId, GoodsGroupId)> = company
            .brands
            .iter()
            .map(|b| (b.country, b.group))
            .chain(company.advertising.iter().map(|a| (a.country, a.group)))
            .chain(
                own.range((index, CountryId::from_index(0), GoodsGroupId::from_index(0))..)
                    .take_while(|((i, _, _), _)| *i == index)
                    .map(|((_, c, g), _)| (*c, *g)),
            )
            .collect();
        keys.sort();
        keys.dedup();
        let mut brands = Vec::with_capacity(keys.len());
        for (country, group) in keys {
            let b = company.awareness(country, group);
            let budget = company
                .advertising
                .iter()
                .find(|a| a.country == country && a.group == group)
                .map_or(0.0, |a| a.budget.to_usd());
            let reach = reach[country.index()];
            let advertising = if reach > 0.0 {
                (1.0 - b) * (1.0 - math::exp(-effect * budget / reach))
            } else {
                0.0
            };
            let share = match total.get(&(country, group)) {
                Some(&t) if t > 0.0 => {
                    own.get(&(index, country, group)).copied().unwrap_or(0.0) / t
                }
                _ => 0.0,
            };
            let word_of_mouth = (1.0 - b) * model.word_of_mouth * share;
            let next = (b * (1.0 - model.forgetting_per_month) + advertising + word_of_mouth)
                .clamp(0.0, 1.0);
            if next >= NEGLIGIBLE || budget > 0.0 {
                brands.push(Brand {
                    country,
                    group,
                    awareness: next,
                });
            }
        }
        company.brands = brands;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::test_support;
    use crate::command::Command;
    use crate::game::Game;
    use crate::state::{AiSettings, GameSettings, StartForm};
    use std::sync::Arc;

    fn game() -> Game {
        let catalog = Arc::new(test_support::production());
        let settings = GameSettings {
            seed: 1,
            start_year: 1900,
            start_country: catalog.countries.id("AAA").unwrap(),
            start_capital: Money::from_usd(10_000_000.0).unwrap(),
            start_form: StartForm::Trading,
            company_name: "Werber".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: AiSettings::default(),
            ventures: 1.0,
            tariff_dynamics: 1.0,
            event_effects: true,
            person: Default::default(),
        };
        Game::new(catalog, settings).unwrap()
    }

    #[test]
    fn advertising_builds_awareness_that_fades_without_it() {
        let mut game = game();
        let country = game.state().settings.start_country;
        let group = game.catalog().products.values().next().unwrap().goods_group;
        let reach = reach_usd(game.state(), game.catalog(), country);
        let budget = Money::from_usd(reach).unwrap();
        game.apply(Command::SetAdvertising {
            country,
            group,
            budget,
        })
        .unwrap();
        let player = game.player();
        let cash = game.state().company(player).unwrap().ledger.cash();
        game.advance(crate::calendar::RoundLength::Month, |_| {});
        let company = game.state().company(player).unwrap();
        let first = company.awareness(country, group);
        // One month at the cost of reaching the country once: 1 − e^−1 of the gap.
        assert!((first - (1.0 - math::exp(-1.0))).abs() < 0.01, "{first}");
        assert!(company.ledger.cash() <= cash - budget);
        assert!(company.ledger.is_balanced());
        // Stopping the advertising: the awareness fades slowly.
        game.apply(Command::SetAdvertising {
            country,
            group,
            budget: Money::ZERO,
        })
        .unwrap();
        for _ in 0..12 {
            game.advance(crate::calendar::RoundLength::Month, |_| {});
        }
        let later = game
            .state()
            .company(player)
            .unwrap()
            .awareness(country, group);
        assert!(later < first && later > 0.4 * first, "{later}");
        // Negative budgets are refused.
        assert!(
            game.apply(Command::SetAdvertising {
                country,
                group,
                budget: Money::from_usd(-1.0).unwrap(),
            })
            .is_err()
        );
    }
}
