//! The catalog: immutable game content compiled from the data files.
//!
//! The catalog never changes during a game. Everything that changes lives in the
//! game state. Field semantics are documented in the data format (`docs/DATENFORMAT.md`).

mod table;
#[cfg(test)]
pub(crate) mod test_support;

pub use table::Table;

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
    pub values: CountryValues,
    pub provenance: Provenance,
}

/// Yearly country values, interpolated in between (Lastenheft §3.2).
#[derive(Clone, Debug, PartialEq)]
pub struct CountryValues {
    /// Inhabitants.
    pub population: TimeSeries,
    /// GDP per capita in USD (purchasing power 2026).
    pub gdp_per_capita_usd: TimeSeries,
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

/// Consumer demand characteristics (Lastenheft §9.1). Formulas follow in M7.
#[derive(Clone, Debug, PartialEq)]
pub struct ConsumerDemand {
    pub need_class: NeedClass,
    pub consumption: ConsumptionType,
    /// Yearly income per capita (USD) from which a household layer starts buying.
    pub income_threshold_usd: f64,
    pub price_sensitivity: f64,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
