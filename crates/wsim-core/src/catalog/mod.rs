//! The catalog: immutable game content compiled from the data files.
//!
//! The catalog never changes during a game. Everything that changes lives in the
//! game state. Field semantics are documented in the data format (`docs/DATENFORMAT.md`).

mod table;
#[cfg(test)]
pub(crate) mod test_support;

pub use table::Table;

use crate::ids::{IdKind, KeyTable};

use crate::ids::{
    BranchId, ContinentId, CountryId, DepositId, FacilityId, GoodsGroupId, LaborGroupId, ProductId,
    QualificationId, RecipeId, SpecializationId, TechnologyId, TransportClassId, UnitId,
};
use crate::money::Money;
use crate::time_series::TimeSeries;

#[derive(Clone, Debug, Default)]
pub struct Catalog {
    pub data_version: u32,
    pub units: Table<UnitId, Unit>,
    pub continents: Table<ContinentId, Continent>,
    pub branches: Table<BranchId, Branch>,
    pub goods_groups: Table<GoodsGroupId, GoodsGroup>,
    pub transport_classes: Table<TransportClassId, TransportClass>,
    pub qualifications: Table<QualificationId, Qualification>,
    pub specializations: Table<SpecializationId, Specialization>,
    pub labor_groups: Table<LaborGroupId, LaborGroup>,
    pub countries: Table<CountryId, Country>,
    pub products: Table<ProductId, Product>,
    pub facilities: Table<FacilityId, Facility>,
    pub recipes: Table<RecipeId, Recipe>,
    pub technologies: Table<TechnologyId, Technology>,
    pub deposits: Table<DepositId, Deposit>,
    pub country_model: CountryModel,
    pub production_model: ProductionModel,
    pub finance_model: FinanceModel,
    pub market_model: MarketModel,
}

impl Catalog {
    /// Keys of all entries by kind, for saving and loading.
    pub fn key_table(&self) -> KeyTable {
        let keys = IdKind::ALL
            .iter()
            .map(|kind| match kind {
                IdKind::Unit => self.units.keys().to_vec(),
                IdKind::Continent => self.continents.keys().to_vec(),
                IdKind::Branch => self.branches.keys().to_vec(),
                IdKind::GoodsGroup => self.goods_groups.keys().to_vec(),
                IdKind::TransportClass => self.transport_classes.keys().to_vec(),
                IdKind::Qualification => self.qualifications.keys().to_vec(),
                IdKind::Specialization => self.specializations.keys().to_vec(),
                IdKind::LaborGroup => self.labor_groups.keys().to_vec(),
                IdKind::Country => self.countries.keys().to_vec(),
                IdKind::Product => self.products.keys().to_vec(),
                IdKind::Facility => self.facilities.keys().to_vec(),
                IdKind::Recipe => self.recipes.keys().to_vec(),
                IdKind::Technology => self.technologies.keys().to_vec(),
                IdKind::Deposit => self.deposits.keys().to_vec(),
            })
            .collect();
        KeyTable::new(keys)
    }
}

/// Where a value comes from (Lastenheft §16.2: approximations are marked).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Provenance {
    pub approximation: bool,
    pub source: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    /// Weight of one unit in kg, if the unit implies one (t = 1000).
    pub weight_kg: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Continent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branch;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoodsGroup;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportClass;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Qualification {
    /// Rank, higher means more qualified. Higher ranks can stand in for lower ones.
    pub rank: u8,
    /// Whether workers of this qualification are split by specialization.
    pub has_specialization: bool,
}

/// A specialization of skilled workers and academics; also the research fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Specialization;

/// A labor pool group: qualification, plus specialization where the qualification has one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaborGroup {
    pub qualification: QualificationId,
    pub specialization: Option<SpecializationId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Country {
    pub continent: ContinentId,
    pub area_km2: f64,
    pub capital: GeoPoint,
    pub landlocked: bool,
    /// Countries with a land border.
    pub neighbors: Vec<CountryId>,
    pub values: CountryValues,
    pub profile: CountryProfile,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoPoint {
    pub lat: f64,
    pub lon: f64,
}

/// Yearly country values, interpolated in between (Lastenheft §3.2). All in today's
/// borders and with the real course of history, including wars and crises.
#[derive(Clone, Debug, PartialEq)]
pub struct CountryValues {
    /// Inhabitants.
    pub population: TimeSeries,
    /// GDP per capita in USD at purchasing power parity (purchasing power 2026).
    pub gdp_per_capita_usd: TimeSeries,
    /// Gini coefficient of income (0–1).
    pub gini: TimeSeries,
    /// Own values; otherwise the defaults of the country model apply.
    pub stability: Option<TimeSeries>,
    pub corporate_tax: Option<TimeSeries>,
    pub dividend_tax: Option<TimeSeries>,
}

/// Country-specific deviations from the country model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CountryProfile {
    /// Added to the automation affinity.
    pub automation_bonus: f64,
    /// Weight per specialization (1 = average), indexed by `SpecializationId`.
    pub specialization_weights: Vec<f64>,
    /// Research strength per field (1 = average), indexed by `SpecializationId`.
    pub research_weights: Vec<f64>,
}

/// Parameters of banking and taxes (`data/parameter/finanzmodell.yaml`, formulas in
/// docs/FORMELN.md). Interest rates are real rates: the game has no inflation.
#[derive(Clone, Debug, PartialEq)]
pub struct FinanceModel {
    pub real_rate: TimeSeries,
    pub premium_min: f64,
    pub premium_per_debt_ratio: f64,
    /// Loans up to this share of fixed assets and inventory.
    pub loan_to_value: f64,
    /// Overdraft up to this share of total assets.
    pub overdraft_share: f64,
    pub overdraft_premium: f64,
    pub max_term_years: u32,
}

impl Default for FinanceModel {
    fn default() -> Self {
        Self {
            real_rate: TimeSeries::new(vec![(1900, 0.03)]).expect("valid"),
            premium_min: 0.01,
            premium_per_debt_ratio: 0.08,
            loan_to_value: 0.6,
            overdraft_share: 0.1,
            overdraft_premium: 0.06,
            max_term_years: 30,
        }
    }
}

/// Parameters of markets (`data/parameter/marktmodell.yaml`, formulas in docs/FORMELN.md).
#[derive(Clone, Debug, PartialEq)]
pub struct MarketModel {
    /// Weight of the price when choosing a seller, per income fifth (poorest first).
    pub price_weight: [f64; 5],
    /// Weight of the quality when choosing a seller, per income fifth.
    pub quality_weight: [f64; 5],
    /// Share of the gap to the target ownership that is bought per year (durables).
    pub adoption_per_year: f64,
    pub price_step_up: f64,
    pub price_step_down: f64,
    /// Unsold stock worth more than this many days of sales lowers the price.
    pub stock_days: f64,
    /// Governments pay at most this multiple of the reference price.
    pub state_price_cap: f64,
    pub index_smoothing: f64,
}

impl Default for MarketModel {
    fn default() -> Self {
        Self {
            price_weight: [2.0, 1.6, 1.2, 0.9, 0.6],
            quality_weight: [0.3, 0.5, 0.8, 1.1, 1.5],
            adoption_per_year: 0.25,
            price_step_up: 0.02,
            price_step_down: 0.01,
            stock_days: 30.0,
            state_price_cap: 1.5,
            index_smoothing: 0.1,
        }
    }
}

/// Parameters of production (`data/parameter/produktionsmodell.yaml`, formulas in
/// docs/FORMELN.md).
#[derive(Clone, Debug, PartialEq)]
pub struct ProductionModel {
    /// Cost of founding a site (land, buildings) per site type.
    pub site_cost: Vec<(SiteType, Money)>,
    pub building_lifetime_years: f64,
    pub development_lifetime_years: f64,
    /// Share of labor that full automation saves at full automation affinity.
    pub automation_labor_saving: f64,
    /// Cost of raising automation from 0 to 1, as share of the facility investment.
    pub automation_cost_share: f64,
    pub quality_inputs: f64,
    pub quality_automation: f64,
    pub quality_condition: f64,
    pub condition_min: f64,
}

impl Default for ProductionModel {
    fn default() -> Self {
        Self {
            site_cost: Vec::new(),
            building_lifetime_years: 50.0,
            development_lifetime_years: 30.0,
            automation_labor_saving: 0.8,
            automation_cost_share: 0.5,
            quality_inputs: 0.3,
            quality_automation: 10.0,
            quality_condition: 20.0,
            condition_min: 0.2,
        }
    }
}

impl ProductionModel {
    pub fn site_cost(&self, kind: SiteType) -> Money {
        self.site_cost
            .iter()
            .find(|(k, _)| *k == kind)
            .map_or(Money::ZERO, |(_, c)| *c)
    }
}

/// Parameters of the country model (`data/parameter/laendermodell.yaml`,
/// formulas in docs/FORMELN.md).
#[derive(Clone, Debug, PartialEq)]
pub struct CountryModel {
    pub price_reference: Option<CountryId>,
    pub price_elasticity: f64,
    pub price_min: f64,
    pub price_max: f64,
    pub participation_rate: f64,
    pub labor_share: f64,
    pub annual_hours: TimeSeries,
    /// Rows `(GDP per capita, share per qualification)`, ascending by GDP.
    pub qualification_shares: Vec<(f64, Vec<f64>)>,
    /// Rows `(GDP per capita, wage factor per qualification)`, ascending by GDP.
    pub wage_factors: Vec<(f64, Vec<f64>)>,
    /// Per qualification: share per specialization (empty for qualifications without).
    pub specialization_shares: Vec<Vec<f64>>,
    pub electricity_price_usd_mwh: TimeSeries,
    pub grid_reach: TimeSeries,
    pub grid_reference_usd: f64,
    pub corporate_tax: TimeSeries,
    pub dividend_tax: TimeSeries,
    pub development_from_usd: f64,
    pub development_to_usd: f64,
    pub rail: TimeSeries,
    pub road: TimeSeries,
    pub air: TimeSeries,
    pub port: TimeSeries,
    pub stability: f64,
    pub research_reference_usd: f64,
    pub research_elasticity: f64,
    pub research_min: f64,
    pub research_max: f64,
    pub automation_base: f64,
    pub automation_per_doubling: f64,
    pub automation_reference_usd: f64,
}

impl Default for CountryModel {
    fn default() -> Self {
        let constant = |v: f64| TimeSeries::new(vec![(1900, v)]).expect("valid");
        Self {
            price_reference: None,
            price_elasticity: 0.0,
            price_min: 1.0,
            price_max: 1.0,
            participation_rate: 0.42,
            labor_share: 0.6,
            annual_hours: constant(2000.0),
            qualification_shares: Vec::new(),
            wage_factors: Vec::new(),
            specialization_shares: Vec::new(),
            electricity_price_usd_mwh: constant(100.0),
            grid_reach: constant(1.0),
            grid_reference_usd: 13_000.0,
            corporate_tax: constant(0.2),
            dividend_tax: constant(0.2),
            development_from_usd: 1_300.0,
            development_to_usd: 65_000.0,
            rail: constant(1.0),
            road: constant(1.0),
            air: constant(1.0),
            port: constant(1.0),
            stability: 0.8,
            research_reference_usd: 26_000.0,
            research_elasticity: 0.3,
            research_min: 0.3,
            research_max: 1.5,
            automation_base: 0.3,
            automation_per_doubling: 0.12,
            automation_reference_usd: 13_000.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductKind {
    RawMaterial,
    SemiFinished,
    Component,
    EndProduct,
    Energy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Usage {
    Industry,
    Consumer,
    Both,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Product {
    pub kind: ProductKind,
    pub branch: BranchId,
    pub unit: UnitId,
    pub usage: Usage,
    pub goods_group: GoodsGroupId,
    pub transport_class: TransportClassId,
    /// Typical price around 1900 at price level 1; starting point of the markets.
    pub reference_price: Money,
    /// Weight of one unit in kg (from the product or its unit).
    pub weight_kg: f64,
    /// Energy content per unit in MWh, if the product can serve as fuel.
    pub heating_value_mwh: Option<f64>,
    pub consumer_demand: Option<ConsumerDemand>,
    pub state_demand: Option<StateDemand>,
    pub state_market: Option<StateMarketOffer>,
    /// Products this one displaces over time (Lastenheft §6.4).
    pub replaces: Vec<ProductId>,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeedClass {
    Basic,
    Durable,
    Luxury,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConsumptionType {
    /// Used up continuously (food, clothing): demand per capita and year.
    Consumable { per_capita_per_year: f64 },
    /// Owned and replaced after its service life (furniture, bicycle, car).
    Durable {
        service_life_years: f64,
        max_ownership: f64,
    },
}

/// Consumer demand characteristics (Lastenheft §9.1, formulas in docs/FORMELN.md M7).
#[derive(Clone, Debug, PartialEq)]
pub struct ConsumerDemand {
    pub need_class: NeedClass,
    pub consumption: ConsumptionType,
    /// Yearly income per capita relative to the price at which half of a household
    /// layer buys (e.g. 2 = an income of twice the price).
    pub purchase_threshold: f64,
    /// Exponent of the price in the purchase propensity.
    pub price_sensitivity: f64,
    /// Exponent of the income in the purchase propensity.
    pub income_sensitivity: f64,
    /// Twelve monthly factors, if demand is seasonal.
    pub seasonality: Option<[f64; 12]>,
}

/// Government demand (Lastenheft §9.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateDemand {
    /// Units per year and million USD of GDP.
    pub per_million_gdp: f64,
    /// Multiplier in times of war (effective from stage 4).
    pub war_factor: f64,
}

/// Goods without their own production chain can be bought from the state market in
/// every country (decision on open points 8–10).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateMarketOffer {
    pub price: Money,
    pub available_from: Option<i32>,
    pub available_until: Option<i32>,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum SiteType {
    /// Mine, oil field, plantation, forest.
    Extraction,
    Factory,
    PowerPlant,
    Warehouse,
    SalesOffice,
    ResearchCenter,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Facility {
    pub site_type: SiteType,
    pub investment: Money,
    pub build_days: u32,
    /// Recipe runs per day at full utilisation.
    pub runs_per_day: f64,
    pub lifetime_years: u32,
    /// Maintenance per year as share of the investment.
    pub maintenance_share: f64,
    /// Highest possible degree of automation (0–1).
    pub automation_max: f64,
    pub technology: Option<TechnologyId>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Recipe {
    pub product: ProductId,
    /// Output of the main product per run.
    pub output: f64,
    pub by_products: Vec<(ProductId, f64)>,
    pub duration_days: u32,
    pub facility: FacilityId,
    pub technology: Option<TechnologyId>,
    /// Takes the main product out of a deposit at the site instead of from inputs.
    pub extraction: bool,
    pub inputs: Vec<(ProductId, f64)>,
    /// Person-hours per run.
    pub labor_hours: Vec<(LaborGroupId, f64)>,
    /// Electricity per run in MWh.
    pub energy_mwh: f64,
    /// Quality (0–100) before inputs, training, automation and equipment condition.
    pub base_quality: f64,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Technology {
    pub field: SpecializationId,
    pub invention_year: i32,
    pub prerequisites: Vec<TechnologyId>,
    /// Research points needed at the historical invention year. `None` for
    /// technologies known before the earliest start year.
    pub research_effort: Option<f64>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Deposit {
    pub country: CountryId,
    pub resource: ProductId,
    /// Remaining reserve in product units; `None` for renewable deposits.
    pub reserve: Option<f64>,
    pub discovered: Option<i32>,
    pub development_cost: Money,
    pub development_days: u32,
    pub max_output_per_year: f64,
    /// Relative extraction cost (1 = typical).
    pub cost_factor: f64,
    pub provenance: Provenance,
}
