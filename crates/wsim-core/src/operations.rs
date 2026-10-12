//! Scrap, accidents, corruption and the rule of law at the sites (docs/FORMELN.md „BP4“).

use crate::calendar::Date;
use crate::catalog::{Catalog, Recipe, SiteType};
use crate::ids::Id as _;
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{Downtime, GameState, Site, SiteId};

/// Share of a run's output lost as scrap: high for a new product and untrained staff,
/// lower with training, the company's development of the product, good condition,
/// experience and automation.
pub fn scrap_rate(
    catalog: &Catalog,
    state: &GameState,
    site: &Site,
    slot: usize,
    recipe: &Recipe,
) -> f64 {
    scrap_rate_for(catalog, state, site, &site.slots[slot], recipe)
}

/// `scrap_rate` for a facility that need not stand at the site yet (an offer's values).
pub fn scrap_rate_for(
    catalog: &Catalog,
    state: &GameState,
    site: &Site,
    sl: &crate::state::Slot,
    recipe: &Recipe,
) -> f64 {
    let m = &catalog.production_model.operations;
    // Extraction yields what it yields; scrap is a matter of processing.
    let base = scrap_base(catalog, recipe);
    if base <= 0.0 {
        return 0.0;
    }
    let learning = crate::learning::factor(catalog, site, recipe);
    (base
        * (1.0 - m.scrap_training * site.training.clamp(0.0, 1.0))
        * (1.0 - m.scrap_development * developed(catalog, state, site, recipe))
        * (1.5 - 0.5 * sl.condition.clamp(0.0, 1.0))
        * libm::sqrt(learning)
        * (1.0 - 0.3 * sl.automation.clamp(0.0, 1.0))
        * sl.machine.map_or(1.0, |m| m.scrap))
    .clamp(m.scrap_min, m.scrap_max)
}

/// How far the site's owner developed the product (0–1): its level against the leader
/// times the product's maturity, as the development's other effects (M37).
fn developed(catalog: &Catalog, state: &GameState, site: &Site, recipe: &Recipe) -> f64 {
    if catalog.research_model.development.levels == 0 {
        return 0.0;
    }
    crate::development::share(state, site.owner, recipe.product)
        * crate::development::maturity(catalog, state, recipe.product)
}

/// Scrap of a new facility of an undeveloped product with untrained staff and the start
/// experience: the start population and the AI plan capacity with it.
pub(crate) fn start_scrap(catalog: &Catalog, recipe: &Recipe) -> f64 {
    let m = &catalog.production_model.operations;
    let base = scrap_base(catalog, recipe);
    if base <= 0.0 {
        return 0.0;
    }
    let lm = &catalog.production_model.learning;
    let units = lm.established_years.max(0.0) * crate::learning::reference_units(catalog, recipe);
    let learning = crate::learning::factor_of(catalog, recipe, units);
    (base * libm::sqrt(learning)).clamp(m.scrap_min, m.scrap_max)
}

/// Base scrap of a recipe by the kind of its product; extraction yields what it yields.
fn scrap_base(catalog: &Catalog, recipe: &Recipe) -> f64 {
    let m = &catalog.production_model.operations;
    if !m.enabled || recipe.extraction {
        return 0.0;
    }
    m.scrap_base[catalog.products.get(recipe.product).kind.index()]
}

/// Accidents per 100 000 working hours at a site.
pub fn accident_rate(catalog: &Catalog, site: &Site, date: Date) -> f64 {
    let m = &catalog.production_model.operations;
    let Some(series) = m.accidents.as_ref().filter(|_| m.enabled) else {
        return 0.0;
    };
    let condition = if site.slots.is_empty() {
        1.0
    } else {
        site.slots.iter().map(|sl| sl.condition).sum::<f64>() / site.slots.len() as f64
    };
    series.value_at(date.year_fraction()).max(0.0)
        * (1.0 - 0.4 * site.training.clamp(0.0, 1.0))
        * (1.5 - 0.5 * condition.clamp(0.0, 1.0))
}

/// Accidents of the day: the chance grows with the hours worked; an accident costs a
/// share of a year's wages, stops a facility for a day and is counted for the year.
pub(crate) fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let m = &catalog.production_model.operations;
    let mut news = Vec::new();
    if !m.enabled || m.accidents.is_none() {
        return news;
    }
    let hours_per_worker = crate::production::hours_per_worker_day(catalog, date);
    let day = u32::try_from(Date::first_of_year(1800).days_until(date)).unwrap_or(0);
    for index in 0..state.sites.len() {
        let s = &state.sites[index];
        if s.closed.is_some() || s.slots.is_empty() {
            continue;
        }
        let workers: f64 = s.workforce.values().sum();
        if workers <= 0.0 {
            continue;
        }
        let expected = workers * hours_per_worker / 100_000.0 * accident_rate(catalog, s, date);
        let site = SiteId(u32::try_from(index).unwrap_or(u32::MAX));
        let mut rng =
            SimRng::for_stream(state.settings.seed, Stream::Accidents { site: site.0, day });
        if rng.next_f64() >= expected.min(1.0) {
            continue;
        }
        let serious = rng.next_f64() < m.accident_stop_share;
        let slot = ((rng.next_f64() * s.slots.len() as f64) as usize).min(s.slots.len() - 1);
        let country = s.country;
        // The average wage of the site's workforce.
        let wages = &state.countries.get(country).hourly_wage_usd;
        let wage = s
            .workforce
            .iter()
            .map(|(g, &w)| w * wages.get(g.index()).copied().unwrap_or(0.0))
            .sum::<f64>()
            / workers
            * (1.0 + s.wage_premium);
        let cost = Money::from_usd(wage * m.accident_hours).unwrap_or(Money::ZERO);
        let owner = s.owner;
        crate::reputation::accident(state, catalog, owner, country);
        let s = &mut state.sites[index];
        s.accidents += 1;
        if serious && s.slots[slot].down.is_none_or(|d| d.until <= date) {
            s.slots[slot].down = Some(Downtime {
                until: date.add_days(1),
                overhaul: false,
            });
        }
        if cost > Money::ZERO {
            state.companies[owner.index()].ledger.expense(
                CostType::Personnel,
                CostCenter::site(site),
                Account::Cash,
                cost,
            );
        }
        if !serious {
            continue;
        }
        if let Some(company) = state.news_of(owner) {
            news.push(
                Message::new(MessageKind::Warning, keys::ACCIDENT)
                    .with(
                        "land",
                        Param::Country(catalog.countries.key(country).to_owned()),
                    )
                    .with("kosten", Param::Money(cost))
                    .of(company),
            );
        }
    }
    news
}

/// Construction time of a facility in a country: longer where corruption is high.
pub fn build_days(catalog: &Catalog, state: &GameState, site: SiteId, days: u32) -> u32 {
    let m = &catalog.production_model.operations;
    let corruption = state
        .site(site)
        .map_or(0.0, |s| state.countries.get(s.country).corruption);
    let factor = 1.0 + corruption * m.corruption_build_time;
    (f64::from(days) * factor).round() as u32
}

/// Fee for a permit on a new facility: corruption times a share of the investment.
pub fn permit_fee(catalog: &Catalog, state: &GameState, site: SiteId, investment: Money) -> Money {
    let m = &catalog.production_model.operations;
    let corruption = state
        .site(site)
        .map_or(0.0, |s| state.countries.get(s.country).corruption);
    investment.scale(corruption * m.corruption_fee)
}

/// At a month start: officials demand payments from some sites (corruption), paid from
/// the cash as other levies; on 1 January foreign sites in lawless, unstable countries
/// may be expropriated.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let m = &catalog.production_model.operations;
    let mut news = Vec::new();
    if !m.enabled {
        return news;
    }
    let month = u32::try_from(date.year() * 12).unwrap_or(0) + date.month();
    let mut rng = SimRng::for_stream(state.settings.seed, Stream::Officials { month });
    for index in 0..state.sites.len() {
        let s = &state.sites[index];
        if s.closed.is_some() || s.kind == SiteType::SalesOffice {
            continue;
        }
        let corruption = state.countries.get(s.country).corruption;
        let chance = corruption * m.demand_chance / 12.0;
        if chance <= 0.0 || rng.next_f64() >= chance {
            continue;
        }
        let investment = s
            .slots
            .iter()
            .map(|sl| sl.cost)
            .fold(s.building_cost, |a, b| a + b);
        let demand = investment.scale(m.demand_share);
        let owner = s.owner;
        let country = s.country;
        let site = SiteId(u32::try_from(index).unwrap_or(u32::MAX));
        let ledger = &mut state.companies[owner.index()].ledger;
        if demand <= Money::ZERO || ledger.cash() < demand {
            continue;
        }
        ledger.expense(
            CostType::Other,
            CostCenter::site(site),
            Account::Cash,
            demand,
        );
        if let Some(company) = state.news_of(owner) {
            news.push(
                Message::new(MessageKind::Warning, keys::BRIBE_DEMAND)
                    .with(
                        "land",
                        Param::Country(catalog.countries.key(country).to_owned()),
                    )
                    .with("betrag", Param::Money(demand))
                    .of(company),
            );
        }
    }
    if date.month() == 1 && m.expropriation > 0.0 {
        let mut rng = SimRng::for_stream(
            state.settings.seed,
            Stream::Officials {
                month: month + 1_000_000,
            },
        );
        for country in catalog.countries.ids() {
            let c = state.countries.get(country);
            let chance = (1.0 - c.rule_of_law) * (1.0 - c.stability) * m.expropriation;
            if chance <= 0.0 || rng.next_f64() >= chance {
                continue;
            }
            let compensation = c.rule_of_law;
            news.extend(crate::events::expropriate(
                state,
                catalog,
                (
                    Param::TextKey("meldung.grund.enteignung_risiko".to_owned()),
                    &[country],
                ),
                true,
                compensation,
            ));
        }
    }
    news
}

/// On 1 January the yearly counts start anew.
pub(crate) fn new_year(state: &mut GameState) {
    for s in &mut state.sites {
        s.accidents = 0;
    }
}
