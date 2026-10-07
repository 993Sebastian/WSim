//! Strategies (Vorgaben) of a company for its managers (MA4, docs/FORMELN.md): set for
//! the company, a continent, a country or a site and inherited downwards. The rules of
//! the positions read them; AI companies have none, and the defaults are exactly the
//! behaviour of their rules.

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::ids::{ContinentId, CountryId};
use crate::money::Money;
use crate::state::{Company, CompanyId, GameState, SiteId, Unit};

/// Where a strategy holds; narrower scopes override wider ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StrategyScope {
    Company,
    Continent(ContinentId),
    Country(CountryId),
    Site(SiteId),
}

impl StrategyScope {
    /// Higher is narrower.
    pub fn rank(self) -> u8 {
        match self {
            StrategyScope::Company => 0,
            StrategyScope::Continent(_) => 1,
            StrategyScope::Country(_) => 2,
            StrategyScope::Site(_) => 3,
        }
    }

    /// The scope of a unit.
    pub fn of(unit: Unit) -> Self {
        match unit {
            Unit::Site(s) => StrategyScope::Site(s),
            Unit::Country(c) => StrategyScope::Country(c),
            Unit::Continent(k) => StrategyScope::Continent(k),
            Unit::Board => StrategyScope::Company,
        }
    }

    /// Whether the scope holds for a unit: its own and those above it.
    pub fn contains(self, catalog: &Catalog, state: &GameState, unit: Unit) -> bool {
        match self {
            StrategyScope::Company => true,
            StrategyScope::Continent(k) => continent_of(catalog, state, unit) == Some(k),
            StrategyScope::Country(c) => country_of(state, unit) == Some(c),
            StrategyScope::Site(s) => unit == Unit::Site(s),
        }
    }
}

fn country_of(state: &GameState, unit: Unit) -> Option<CountryId> {
    match unit {
        Unit::Site(s) => state.sites.get(s.index()).map(|x| x.country),
        Unit::Country(c) => Some(c),
        Unit::Continent(_) | Unit::Board => None,
    }
}

fn continent_of(catalog: &Catalog, state: &GameState, unit: Unit) -> Option<ContinentId> {
    match unit {
        Unit::Continent(k) => Some(k),
        _ => country_of(state, unit).map(|c| catalog.countries.get(c).continent),
    }
}

/// The fields of the strategy (Lastenheft §5.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StrategyField {
    Price,
    Stock,
    Wages,
    Supply,
    Investment,
    Reserve,
    /// Training target of the sites (W1).
    Training,
}

impl StrategyField {
    pub const ALL: [StrategyField; 7] = [
        StrategyField::Price,
        StrategyField::Stock,
        StrategyField::Wages,
        StrategyField::Supply,
        StrategyField::Investment,
        StrategyField::Reserve,
        StrategyField::Training,
    ];

    /// Key of the texts (`strategie.feld.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            StrategyField::Price => "preis",
            StrategyField::Stock => "lager",
            StrategyField::Wages => "personal",
            StrategyField::Supply => "eigenfertigung",
            StrategyField::Investment => "investition",
            StrategyField::Reserve => "reserve",
            StrategyField::Training => "schulung",
        }
    }
}

/// How the positions price their offers in the market price mode.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PriceStrategy {
    /// The price floor of the rules.
    Market,
    Premium,
    Fight,
    /// A floor at the full unit cost plus this share.
    MinMargin(f64),
}

/// Stock reach in days of use or sales.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StockStrategy {
    /// Below it purchases bid more.
    pub input_min_days: f64,
    /// Purchases fill the inputs' stock up to it.
    pub input_max_days: f64,
    /// Production steers the stock of its goods to it.
    pub output_days: f64,
}

/// Bounds of the wage premium over the market wage.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WageStrategy {
    pub min: f64,
    pub max: f64,
}

/// Whether inputs come from the company's own sites or from the market.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupplyStrategy {
    OwnFirst,
    /// From own sites only where their price plus freight does not exceed the market.
    ByPrice,
    /// No deliveries between own sites.
    Buy,
}

/// The setting of one field.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum StrategyValue {
    Price(PriceStrategy),
    Stock(StockStrategy),
    Wages(WageStrategy),
    Supply(SupplyStrategy),
    /// Investments the positions may decide per calendar year.
    Investment(Money),
    /// Cash the positions keep, in months of running costs.
    Reserve(f64),
    /// Training target of the sites, 0–1 (W1).
    Training(f64),
}

impl StrategyValue {
    pub fn field(&self) -> StrategyField {
        match self {
            StrategyValue::Price(_) => StrategyField::Price,
            StrategyValue::Stock(_) => StrategyField::Stock,
            StrategyValue::Wages(_) => StrategyField::Wages,
            StrategyValue::Supply(_) => StrategyField::Supply,
            StrategyValue::Investment(_) => StrategyField::Investment,
            StrategyValue::Reserve(_) => StrategyField::Reserve,
            StrategyValue::Training(_) => StrategyField::Training,
        }
    }
}

fn is_zero(m: &Money) -> bool {
    *m == Money::ZERO
}

fn is_zero_year(year: &i32) -> bool {
    *year == 0
}

/// A strategy a company set for a scope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrategySetting {
    pub scope: StrategyScope,
    pub value: StrategyValue,
    /// Investments the positions decided against this budget in `year`.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub spent: Money,
    #[serde(default, skip_serializing_if = "is_zero_year")]
    pub year: i32,
}

/// What the rules of a site's positions follow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SiteStrategy {
    pub price: PriceStrategy,
    pub stock: StockStrategy,
    pub wages: WageStrategy,
    pub supply: SupplyStrategy,
}

impl SiteStrategy {
    /// The defaults: the behaviour of the AI's rules.
    pub fn defaults(catalog: &Catalog) -> Self {
        let ai = &catalog.ai_model;
        SiteStrategy {
            price: PriceStrategy::Market,
            stock: StockStrategy {
                input_min_days: ai.behavior.stock_low_days,
                input_max_days: ai.start.input_stock_days,
                output_days: ai.behavior.stock_target_days,
            },
            wages: WageStrategy {
                min: 0.0,
                max: ai
                    .behavior
                    .wage_premium_max
                    .min(catalog.production_model.wage_premium_max),
            },
            supply: SupplyStrategy::OwnFirst,
        }
    }
}

/// The default of a field; `None` for the investment budget, which has none.
pub fn default_value(catalog: &Catalog, field: StrategyField) -> Option<StrategyValue> {
    let d = SiteStrategy::defaults(catalog);
    match field {
        StrategyField::Price => Some(StrategyValue::Price(d.price)),
        StrategyField::Stock => Some(StrategyValue::Stock(d.stock)),
        StrategyField::Wages => Some(StrategyValue::Wages(d.wages)),
        StrategyField::Supply => Some(StrategyValue::Supply(d.supply)),
        StrategyField::Investment => None,
        StrategyField::Reserve => Some(StrategyValue::Reserve(0.0)),
        StrategyField::Training => Some(StrategyValue::Training(0.0)),
    }
}

/// The training target the company's strategy sets for a site (W1); `None` without a
/// setting.
pub fn training_for_site(
    state: &GameState,
    company: CompanyId,
    site: SiteId,
    catalog: &Catalog,
) -> Option<f64> {
    let c = state.companies.get(company.index())?;
    if c.strategies.is_empty() {
        return None;
    }
    match setting(catalog, state, c, Unit::Site(site), StrategyField::Training)?.value {
        StrategyValue::Training(t) => Some(t),
        _ => None,
    }
}

/// The setting of a field that holds for a unit: that of the narrowest scope containing
/// it; `None` if only the default holds.
pub fn setting<'a>(
    catalog: &Catalog,
    state: &GameState,
    company: &'a Company,
    unit: Unit,
    field: StrategyField,
) -> Option<&'a StrategySetting> {
    company
        .strategies
        .iter()
        .filter(|s| s.value.field() == field && s.scope.contains(catalog, state, unit))
        .max_by_key(|s| s.scope.rank())
}

/// The value of a field that holds for a unit and the scope it comes from (`None`: the
/// default).
pub fn effective(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    unit: Unit,
    field: StrategyField,
) -> (Option<StrategyValue>, Option<StrategyScope>) {
    let c = &state.companies[company.index()];
    match setting(catalog, state, c, unit, field) {
        Some(s) => (Some(s.value), Some(s.scope)),
        None => (default_value(catalog, field), None),
    }
}

/// What the rules of a site follow.
pub fn for_site(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    site: SiteId,
) -> SiteStrategy {
    let mut out = SiteStrategy::defaults(catalog);
    let c = &state.companies[company.index()];
    if c.strategies.is_empty() {
        return out;
    }
    let unit = Unit::Site(site);
    for field in [
        StrategyField::Price,
        StrategyField::Stock,
        StrategyField::Wages,
        StrategyField::Supply,
    ] {
        match setting(catalog, state, c, unit, field).map(|s| s.value) {
            Some(StrategyValue::Price(p)) => out.price = p,
            Some(StrategyValue::Stock(s)) => out.stock = s,
            Some(StrategyValue::Wages(w)) => out.wages = w,
            Some(StrategyValue::Supply(s)) => out.supply = s,
            _ => {}
        }
    }
    out
}

/// Price floor on the full unit cost and markup an offer starts at. `rules_floor` is the
/// floor factor of the rules for the company.
pub fn price_terms(catalog: &Catalog, price: PriceStrategy, rules_floor: f64) -> (f64, f64) {
    let m = &catalog.management.strategy;
    match price {
        PriceStrategy::Market => (rules_floor, 0.0),
        PriceStrategy::Premium => m.premium,
        PriceStrategy::Fight => m.fight,
        PriceStrategy::MinMargin(margin) => (1.0 + margin, 0.0),
    }
}

/// An investment budget that holds for a unit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InvestmentBudget {
    pub scope: StrategyScope,
    pub per_year: Money,
    /// What is left of it in the year.
    pub left: Money,
}

/// The investment budget of a setting in a year, if it is one.
pub fn budget_of(setting: &StrategySetting, year: i32) -> Option<InvestmentBudget> {
    match setting.value {
        StrategyValue::Investment(per_year) => {
            let spent = if setting.year == year {
                setting.spent
            } else {
                Money::ZERO
            };
            Some(InvestmentBudget {
                scope: setting.scope,
                per_year,
                left: (per_year - spent).max(Money::ZERO),
            })
        }
        _ => None,
    }
}

/// The investment budgets that hold for a unit in a year: all of the scopes containing it.
pub fn investment_budgets(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    unit: Unit,
    year: i32,
) -> Vec<InvestmentBudget> {
    state.companies[company.index()]
        .strategies
        .iter()
        .filter(|s| s.scope.contains(catalog, state, unit))
        .filter_map(|s| budget_of(s, year))
        .collect()
}

/// The budget that binds first at a unit: the one with the least left.
pub fn binding_budget(budgets: &[InvestmentBudget]) -> Option<InvestmentBudget> {
    budgets.iter().copied().min_by(|a, b| {
        a.left
            .cmp(&b.left)
            .then(b.scope.rank().cmp(&a.scope.rank()))
    })
}

/// Counts investments the positions decided for a unit against its budgets.
pub(crate) fn count_investment(
    catalog: &Catalog,
    state: &mut GameState,
    company: CompanyId,
    unit: Unit,
    (amount, year): (Money, i32),
) {
    let hits: Vec<usize> = state.companies[company.index()]
        .strategies
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            matches!(s.value, StrategyValue::Investment(_))
                && s.scope.contains(catalog, state, unit)
        })
        .map(|(i, _)| i)
        .collect();
    let strategies = &mut state.companies[company.index()].strategies;
    for i in hits {
        let s = &mut strategies[i];
        if s.year != year {
            s.year = year;
            s.spent = Money::ZERO;
        }
        s.spent += amount;
    }
}

/// Months of running costs the positions keep at a unit.
pub fn reserve_months(catalog: &Catalog, state: &GameState, company: CompanyId, unit: Unit) -> f64 {
    match effective(catalog, state, company, unit, StrategyField::Reserve).0 {
        Some(StrategyValue::Reserve(months)) => months,
        _ => 0.0,
    }
}

/// Running costs of a month of all the company's sites at their planned production.
pub fn monthly_cost(catalog: &Catalog, state: &GameState, company: CompanyId) -> Money {
    let own: Vec<SiteId> = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.owner == company)
        // Few sites; the cast is exact.
        .map(|(i, _)| SiteId(i as u32))
        .collect();
    crate::ai::daily_cost(catalog, state, &own).scale(30.0)
}

/// `SetStrategy`: sets the strategy of a field for a scope; `None` removes it. The
/// investments counted against a budget stay when its amount changes.
pub(crate) fn set(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    (scope, field): (StrategyScope, StrategyField),
    value: Option<StrategyValue>,
) -> Result<(), CommandError> {
    if !catalog.management.enabled() {
        return Err(CommandError::InvalidStrategy);
    }
    if let StrategyScope::Site(site) = scope {
        let s = state
            .sites
            .get(site.index())
            .ok_or(CommandError::UnknownSite)?;
        if s.owner != company {
            return Err(CommandError::NotOwner);
        }
    }
    if let Some(v) = value
        && (v.field() != field || !valid(catalog, &v))
    {
        return Err(CommandError::InvalidStrategy);
    }
    let strategies = &mut state.companies[company.index()].strategies;
    let at = strategies
        .iter()
        .position(|s| s.scope == scope && s.value.field() == field);
    match (value, at) {
        (None, Some(i)) => {
            strategies.remove(i);
        }
        (None, None) => {}
        (Some(v), Some(i)) => strategies[i].value = v,
        (Some(v), None) => {
            strategies.push(StrategySetting {
                scope,
                value: v,
                spent: Money::ZERO,
                year: 0,
            });
            strategies.sort_by(|a, b| {
                a.value
                    .field()
                    .cmp(&b.value.field())
                    .then(a.scope.cmp(&b.scope))
            });
        }
    }
    Ok(())
}

/// Whether a value lies within the bounds of its field (docs/FORMELN.md, MA4).
fn valid(catalog: &Catalog, value: &StrategyValue) -> bool {
    let m = &catalog.management.strategy;
    let between = |x: f64, lo: f64, hi: f64| x.is_finite() && (lo..=hi).contains(&x);
    match *value {
        StrategyValue::Price(PriceStrategy::MinMargin(margin)) => {
            between(margin, 0.0, m.min_margin_max)
        }
        StrategyValue::Price(_) | StrategyValue::Supply(_) => true,
        StrategyValue::Stock(s) => {
            s.input_min_days > 0.0
                && between(s.input_min_days, 0.0, s.input_max_days)
                && between(s.input_max_days, 0.0, m.stock_days_max)
                && between(s.output_days, 0.0, m.stock_days_max)
        }
        StrategyValue::Wages(w) => {
            between(w.min, 0.0, w.max)
                && between(w.max, 0.0, catalog.production_model.wage_premium_max)
        }
        StrategyValue::Investment(amount) => amount >= Money::ZERO,
        StrategyValue::Reserve(months) => between(months, 0.0, m.reserve_months_max),
        StrategyValue::Training(t) => between(t, 0.0, 1.0),
    }
}
