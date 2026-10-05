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
    QualificationId, RecipeId, SpecializationId, TechnologyId, TransportClassId, UnitId, VehicleId,
};
use crate::money::Money;
use crate::state::StartForm;
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
    pub vehicles: Table<VehicleId, Vehicle>,
    pub country_model: CountryModel,
    pub production_model: ProductionModel,
    pub finance_model: FinanceModel,
    pub market_model: MarketModel,
    pub transport_model: TransportModel,
    pub research_model: ResearchModel,
    pub ai_model: AiModel,
    /// Historical companies of the start population and later foundings.
    pub real_companies: Vec<RealCompany>,
    /// Historical events, sorted by date (Lastenheft §4.1; effects follow in stage 4).
    pub events: Vec<HistoricalEvent>,
    /// Parts for the names of generated companies.
    pub name_groups: Vec<NameGroup>,
}

impl Catalog {
    /// Highest output of a deposit per year in a year (before the market scale): the
    /// deposit's value times the output index of its raw material (M16).
    pub fn max_output(&self, deposit: DepositId, year: i32) -> f64 {
        let d = self.deposits.get(deposit);
        let index = self
            .products
            .get(d.resource)
            .output_index
            .as_ref()
            .map_or(1.0, |i| i.value_at(f64::from(year)));
        d.max_output_per_year * index
    }

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
                IdKind::Vehicle => self.vehicles.keys().to_vec(),
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

#[derive(Clone, Debug, PartialEq)]
pub struct TransportClass {
    /// Transport costs relative to bulk goods (1 = bulk).
    pub cost_factor: f64,
}

/// Kind of way a vehicle uses (Lastenheft §3.5, §8.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Way {
    /// No built infrastructure needed (carts on tracks and dirt roads).
    Terrain,
    Road,
    Rail,
    Sea,
    Air,
}

/// A means of transport (Lastenheft §8.1). Stage 1 uses vehicles only through the
/// abstract freight service between countries (formulas in docs/FORMELN.md, M8).
#[derive(Clone, Debug, PartialEq)]
pub struct Vehicle {
    pub way: Way,
    pub available_from: i32,
    pub available_until: Option<i32>,
    /// Transport classes the vehicle carries.
    pub classes: Vec<TransportClassId>,
    /// Cost per tonne-kilometre of bulk goods in USD, by year.
    pub cost_per_tkm: TimeSeries,
    pub km_per_day: TimeSeries,
    pub provenance: Provenance,
}

impl Vehicle {
    pub fn available(&self, year: i32) -> bool {
        self.available_from <= year && self.available_until.is_none_or(|y| y >= year)
    }
}

/// Parameters of research (`data/parameter/forschungsmodell.yaml`, formulas in
/// docs/FORMELN.md, M9).
#[derive(Clone, Debug, PartialEq)]
pub struct ResearchModel {
    /// Cost factor per year of research ahead of the historical invention.
    pub ahead_base: f64,
    /// Yearly discount for latecomers after an invention, and the lowest share left.
    pub latecomer_discount: f64,
    pub latecomer_min: f64,
    /// Years after the historical invention when everyone may use a technology.
    pub public_domain_years: i32,
    /// Equipment and material per researcher and day (USD at price level 1).
    pub material_usd_per_day: f64,
    /// Labor group of the researchers per field, indexed by `SpecializationId`.
    pub researchers: Vec<Option<LaborGroupId>>,
}

impl Default for ResearchModel {
    fn default() -> Self {
        Self {
            ahead_base: 1.25,
            latecomer_discount: 0.1,
            latecomer_min: 0.2,
            public_domain_years: 25,
            material_usd_per_day: 40.0,
            researchers: Vec::new(),
        }
    }
}

/// A value that depends on a company trait (competence or aggressiveness, 0–1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub at_0: f64,
    pub at_1: f64,
}

impl Span {
    pub const fn fixed(v: f64) -> Self {
        Self { at_0: v, at_1: v }
    }

    pub fn at(&self, t: f64) -> f64 {
        self.at_0 + (self.at_1 - self.at_0) * t.clamp(0.0, 1.0)
    }
}

/// A preset of the AI difficulty (Lastenheft §15).
#[derive(Clone, Debug, PartialEq)]
pub struct Difficulty {
    pub key: String,
    pub competence: f64,
    pub aggressiveness: f64,
}

/// Parameters of the AI companies (`data/parameter/kimodell.yaml`, docs/FORMELN.md M10).
#[derive(Clone, Debug, PartialEq)]
pub struct AiModel {
    pub default_companies: u32,
    pub max_companies: u32,
    /// Market scale = companies / this number, within the limits below.
    pub companies_for_real_size: f64,
    pub scale_min: f64,
    pub scale_max: f64,
    /// Labor pools never shrink below this many persons per group.
    pub min_labor_pool: f64,
    /// Extraction facilities a concession is sized for, and the most concessions.
    pub plants_per_concession: f64,
    pub max_concessions: u32,
    pub difficulties: Vec<Difficulty>,
    pub default_difficulty: usize,
    /// Spread of competence and aggressiveness around the setting, per company.
    pub trait_spread: f64,
    pub start: AiStart,
    pub behavior: AiBehavior,
}

/// How the AI companies are placed at the start.
#[derive(Clone, Debug, PartialEq)]
pub struct AiStart {
    pub utilization: f64,
    /// Plants below this share of one facility are not built.
    pub min_plant_share: f64,
    pub input_stock_days: f64,
    pub output_stock_days: f64,
    pub cash_months: f64,
    /// Weight of the development level when placing plants, by product kind
    /// (raw material, semi-finished, component, end product, energy).
    pub development_weight: [f64; 5],
    /// Wage for comparing recipes (USD per hour).
    pub reference_wage_usd: f64,
    /// Plants are planned for this multiple of the demand, per product kind: saturated
    /// markets (M16).
    pub market_cover: [f64; 5],
}

/// How the AI companies decide; spans depend on competence or aggressiveness.
#[derive(Clone, Debug, PartialEq)]
pub struct AiBehavior {
    /// Days between operating decisions (competence).
    pub operations_days: Span,
    pub stock_high_days: f64,
    pub stock_low_days: f64,
    pub utilization_step: f64,
    pub utilization_min: f64,
    /// Largest change of the planned utilization per decision: one weak month must not
    /// stop a plant, nor one good month fill every warehouse at once.
    pub utilization_change_max: f64,
    /// Stock the production aims at, in days of sales and own use (M16).
    pub stock_target_days: f64,
    /// Days in which the production closes the gap to the stock target.
    pub stock_adjust_days: f64,
    /// Price floor = normal cost × this factor (aggressiveness).
    pub floor_factor: Span,
    /// Monthly advertising as share of the revenue of a goods group in a country.
    pub advertising_share: Span,
    pub purchase_markup: f64,
    /// Expansion when the utilization and the margin reach these (aggressiveness).
    pub expand_utilization: Span,
    pub expand_margin: Span,
    /// No expansion while an input costs more than this multiple of its reference price
    /// in the country: it is scarce (M16).
    pub expand_input_price_max: f64,
    pub invest_share_max: f64,
    /// Wage premium of a site: raised by the step while a facility waits for workers,
    /// lowered otherwise, up to the maximum (M18).
    pub wage_premium_step: f64,
    pub wage_premium_max: f64,
    /// Research on technologies up to this many years before their invention (competence).
    pub research_lookahead_years: Span,
    pub research_min_revenue_usd: f64,
    pub research_competence_min: f64,
    pub cash_min_months: f64,
    pub cash_max_months: f64,
    pub loan_years: u32,
    pub foundings_per_month: u32,
    /// Rich companies that build in another company's bottleneck per quarter.
    pub diversifications_per_quarter: u32,
    pub founding_capital_factor: f64,
}

impl Default for AiModel {
    fn default() -> Self {
        Self {
            default_companies: 100,
            max_companies: 10_000,
            companies_for_real_size: 1000.0,
            scale_min: 0.01,
            scale_max: 1.0,
            min_labor_pool: 2000.0,
            plants_per_concession: 2.0,
            max_concessions: 12,
            difficulties: Vec::new(),
            default_difficulty: 0,
            trait_spread: 0.15,
            start: AiStart {
                utilization: 0.85,
                min_plant_share: 0.15,
                input_stock_days: 20.0,
                output_stock_days: 10.0,
                cash_months: 3.0,
                development_weight: [0.0, 1.5, 2.0, 0.5, 1.0],
                reference_wage_usd: 4.0,
                market_cover: [1.15; 5],
            },
            behavior: AiBehavior {
                operations_days: Span {
                    at_0: 14.0,
                    at_1: 7.0,
                },
                stock_high_days: 20.0,
                stock_low_days: 7.0,
                utilization_step: 0.1,
                utilization_min: 0.2,
                utilization_change_max: 1.0,
                stock_target_days: 14.0,
                stock_adjust_days: 15.0,
                floor_factor: Span {
                    at_0: 1.05,
                    at_1: 0.9,
                },
                advertising_share: Span {
                    at_0: 0.01,
                    at_1: 0.03,
                },
                purchase_markup: 0.25,
                expand_utilization: Span {
                    at_0: 0.95,
                    at_1: 0.8,
                },
                expand_margin: Span {
                    at_0: 0.25,
                    at_1: 0.08,
                },
                expand_input_price_max: 1.5,
                invest_share_max: 0.3,
                wage_premium_step: 0.05,
                wage_premium_max: 0.3,
                research_lookahead_years: Span {
                    at_0: 0.0,
                    at_1: 2.0,
                },
                research_min_revenue_usd: 5_000_000.0,
                research_competence_min: 0.5,
                cash_min_months: 2.0,
                cash_max_months: 6.0,
                loan_years: 10,
                foundings_per_month: 2,
                diversifications_per_quarter: 4,
                founding_capital_factor: 1.5,
            },
        }
    }
}

impl AiModel {
    /// Market scale for a number of AI companies.
    pub fn market_scale(&self, companies: u32) -> f64 {
        (f64::from(companies) / self.companies_for_real_size).clamp(self.scale_min, self.scale_max)
    }
}

/// A historical event shown as world news (Lastenheft §4.1, §13.2). In stage 1 it has
/// no effects of its own: the country values already contain its economic slump.
#[derive(Clone, Debug, PartialEq)]
pub struct HistoricalEvent {
    pub key: String,
    pub date: crate::calendar::Date,
    /// Kind of event, text `ereignisart.<kind>`.
    pub kind: String,
    pub countries: Vec<CountryId>,
    pub provenance: Provenance,
}

/// A historical company (Lastenheft §10): its activities at `snapshot_year`.
#[derive(Clone, Debug, PartialEq)]
pub struct RealCompany {
    pub key: String,
    /// Proper name, not translated.
    pub name: String,
    pub headquarters: CountryId,
    pub founded: i32,
    pub sites: Vec<RealSite>,
    pub competence: Option<f64>,
    pub aggressiveness: Option<f64>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RealSite {
    pub country: CountryId,
    pub deposit: Option<DepositId>,
    /// Facility, real number of units (scaled in the game), recipe.
    pub facilities: Vec<(FacilityId, f64, Option<RecipeId>)>,
}

/// Name parts for generated companies of a language region.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NameGroup {
    pub key: String,
    pub countries: Vec<CountryId>,
    pub is_default: bool,
    pub surnames: Vec<String>,
    pub places: Vec<String>,
    pub legal_forms: Vec<String>,
    /// Patterns with `{familienname}`, `{ort}`, `{rechtsform}`, `{branche}`.
    pub patterns: Vec<String>,
    /// Word for the business per branch, indexed by `BranchId`.
    pub branch_words: Vec<Option<String>>,
}

/// Parameters of transport (`data/parameter/transportmodell.yaml`).
#[derive(Clone, Debug, PartialEq)]
pub struct TransportModel {
    /// Route length relative to the great-circle distance between capitals.
    pub detour_land: f64,
    pub detour_sea: f64,
    pub detour_air: f64,
    /// Loading or unloading at a port or airport, per tonne of bulk goods (USD).
    pub handling_cost_usd: f64,
    pub handling_days: f64,
    /// Below this infrastructure level a way cannot be used.
    pub min_infrastructure: f64,
}

impl Default for TransportModel {
    fn default() -> Self {
        Self {
            detour_land: 1.3,
            detour_sea: 1.4,
            detour_air: 1.05,
            handling_cost_usd: 4.0,
            handling_days: 2.0,
            min_infrastructure: 0.05,
        }
    }
}

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
    /// Automatic prices also fall while the seller's facilities for the product run
    /// below this share of their capacity: idle plants compete for customers (M16).
    pub normal_utilization: f64,
    /// Automatic prices stay below this multiple of the local reference price.
    pub price_max_factor: f64,
    /// Governments pay at most this multiple of the reference price.
    pub state_price_cap: f64,
    /// How far a country's price level carries into the prices of goods, per product
    /// kind (0: world price, 1: fully). Goods are traded; only the local share of
    /// wages, trade and distribution in their price follows the price level.
    pub price_level_share: [f64; 5],
    pub index_smoothing: f64,
    /// Traders import only if the market price exceeds their landed cost by this share.
    pub trader_margin: f64,
    /// Traders keep stock for this many days of open demand.
    pub trader_cover_days: f64,
    /// Days over which the open demand of a market is averaged.
    pub demand_smoothing_days: f64,
    /// Brands and advertising (M16).
    pub brand: BrandModel,
}

/// Brand awareness and advertising (`marktmodell.marke`, formulas in docs/FORMELN.md, M16).
#[derive(Clone, Debug, PartialEq)]
pub struct BrandModel {
    /// Weight of the brand awareness when choosing a seller, per income fifth.
    pub weight: [f64; 5],
    pub forgetting_per_month: f64,
    /// Share of the gap to full awareness closed per month at a market share of 1.
    pub word_of_mouth: f64,
    /// Advertising that reaches a country once, per inhabitant at price level 1.
    pub cost_per_inhabitant_usd: f64,
    /// Awareness of established companies where they sell at the start.
    pub start_awareness: f64,
    pub start_awareness_real: f64,
    /// Awareness of goods that traders import and of the state market.
    pub trade_awareness: f64,
    pub state_market_awareness: f64,
    pub media: Vec<AdvertisingMedium>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdvertisingMedium {
    pub key: String,
    pub from_year: i32,
    pub effect: f64,
}

impl BrandModel {
    /// The best medium available in `year`, if any.
    pub fn medium(&self, year: i32) -> Option<&AdvertisingMedium> {
        self.media
            .iter()
            .filter(|m| m.from_year <= year)
            .max_by(|a, b| a.effect.total_cmp(&b.effect))
    }
}

impl Default for BrandModel {
    fn default() -> Self {
        Self {
            weight: [0.4, 0.6, 0.9, 1.2, 1.5],
            forgetting_per_month: 0.03,
            word_of_mouth: 0.08,
            cost_per_inhabitant_usd: 0.05,
            start_awareness: 0.5,
            start_awareness_real: 0.7,
            trade_awareness: 0.2,
            state_market_awareness: 0.3,
            media: vec![AdvertisingMedium {
                key: "zeitung".into(),
                from_year: 1800,
                effect: 1.0,
            }],
        }
    }
}

impl Default for MarketModel {
    fn default() -> Self {
        Self {
            price_weight: [7.0, 6.0, 5.0, 4.0, 3.0],
            quality_weight: [0.3, 0.5, 0.8, 1.1, 1.5],
            adoption_per_year: 0.25,
            price_step_up: 0.02,
            price_step_down: 0.01,
            stock_days: 30.0,
            normal_utilization: 0.85,
            price_max_factor: 20.0,
            state_price_cap: 1.5,
            price_level_share: [1.0; 5],
            index_smoothing: 0.1,
            trader_margin: 0.05,
            trader_cover_days: 30.0,
            demand_smoothing_days: 30.0,
            brand: BrandModel::default(),
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
    /// The product that stands for electricity (own power plants, Lastenheft §6.1).
    pub electricity: Option<ProductId>,
    /// Own electricity fed into the grid earns this share of the industrial price.
    pub feed_in_share: f64,
    /// Administration, sales and logistics per run, as share of the value it adds at
    /// reference prices, per product kind (M16).
    pub overhead_share: [f64; 5],
    /// Plausible margin at reference prices of the best recipe of a product in the first
    /// year it can be made (checked when loading the data; extraction only from below).
    pub reference_margin: (f64, f64),
    /// By-products beyond this many days of their output in stock are disposed of.
    pub by_product_stock_days: f64,
    /// Highest wage premium of a site (M18).
    pub wage_premium_max: f64,
    /// What a new company owns at the start, per start form (Lastenheft §15).
    pub start_setups: Vec<(StartForm, StartSetup)>,
}

/// The first site of a new company, paid from the start capital.
#[derive(Clone, Debug, PartialEq)]
pub struct StartSetup {
    pub site_type: SiteType,
    /// Land and buildings (a workshop costs less than an industrial site).
    pub building: Money,
    /// Facilities ready at the start, with recipe and planned utilization.
    pub facilities: Vec<(FacilityId, Option<RecipeId>, f64)>,
    /// Purchase orders: product, target stock, highest price.
    pub purchases: Vec<(ProductId, f64, Money)>,
    /// Products offered at the market price.
    pub sales: Vec<ProductId>,
}

impl StartSetup {
    /// Building and facilities together.
    pub fn cost(&self, catalog: &Catalog) -> Money {
        self.building
            + self
                .facilities
                .iter()
                .map(|&(f, _, _)| catalog.facilities.get(f).investment)
                .sum::<Money>()
    }
}

impl ProductionModel {
    /// Overhead per unit made, as share of the reference price.
    pub fn overhead_share(&self, kind: ProductKind) -> f64 {
        self.overhead_share[kind.index()]
    }

    pub fn start_setup(&self, form: StartForm) -> Option<&StartSetup> {
        self.start_setups
            .iter()
            .find(|(f, _)| *f == form)
            .map(|(_, s)| s)
    }
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
            electricity: None,
            feed_in_share: 0.5,
            overhead_share: [0.0; 5],
            reference_margin: (0.05, 0.45),
            by_product_stock_days: 90.0,
            wage_premium_max: 1.0,
            start_setups: Vec::new(),
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
    /// Labor productivity = (GDP per capita / reference)^elasticity, bounded: the
    /// recipes' hours hold at the reference (M16).
    pub productivity_reference_usd: f64,
    pub productivity_elasticity: f64,
    pub productivity_min: f64,
    pub productivity_max: f64,
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
            productivity_reference_usd: 10_000.0,
            productivity_elasticity: 0.0,
            productivity_min: 0.1,
            productivity_max: 4.0,
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

impl ProductKind {
    /// Position in tables with one value per product kind (`[T; 5]`).
    pub fn index(self) -> usize {
        match self {
            ProductKind::RawMaterial => 0,
            ProductKind::SemiFinished => 1,
            ProductKind::Component => 2,
            ProductKind::EndProduct => 3,
            ProductKind::Energy => 4,
        }
    }
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
    /// Raw materials: how the highest output of their deposits develops over the years
    /// (more land and better yields), as factor on the deposit values (M16).
    pub output_index: Option<TimeSeries>,
    /// Raw materials: land rent and royalties per unit extracted, as share of the
    /// reference price in the country (M16).
    pub rent_share: f64,
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
    /// Used up with a durable the households own (petrol per car, kerosene per lamp).
    Complement {
        of: ProductId,
        per_unit_per_year: f64,
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
    /// Only households with electricity buy (demand times the grid share).
    pub needs_grid: bool,
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
