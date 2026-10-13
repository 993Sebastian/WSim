//! Views of the play screens (Lastenheft §14.1): production with its causes, the
//! options to build, markets, research and finances. Like all views they hold keys
//! and finished numbers only.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::catalog::{Catalog, FacilitySize, SiteType};
use crate::command::site_type_key;
use crate::finance;
use crate::game::Game;
use crate::ids::{CountryId, GoodsGroupId, Id, ProductId};
use crate::market;
use crate::money::Money;
use crate::reports;
use crate::research;
use crate::state::{CompanyId, Limit, Market, Operation, PriceMode};

/// Why a facility made less than planned on the last day, as text key and product.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cause {
    /// `ursache.vorprodukt`, `ursache.arbeitskraefte`, `ursache.strom`,
    /// `ursache.lagerstaette`, `ursache.im_bau`, `ursache.erschliessung`,
    /// `ursache.kein_rezept`, `ursache.ruht`.
    pub key: String,
    /// Missing input or labor group.
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlotDetail {
    pub index: usize,
    pub facility: String,
    pub count: u32,
    /// Size of the units (M36; text `anlagengroesse.<key>`) and their capacity as a
    /// multiple of the data size.
    #[serde(default = "medium_key")]
    pub size: String,
    #[serde(default = "one")]
    pub capacity: f64,
    pub recipe: Option<String>,
    pub product: Option<String>,
    pub utilization: f64,
    pub automation: f64,
    pub automation_max: f64,
    pub ready: String,
    /// Output per day at the planned utilization.
    pub planned_per_day: f64,
    /// Output on the last day.
    pub made_per_day: f64,
    pub cause: Option<Cause>,
    /// Condition of the facilities (1 = new).
    pub condition: f64,
    /// Inputs per day at the planned utilization.
    pub inputs_per_day: Vec<(String, f64)>,
    /// The company's development level of the product (M37).
    #[serde(default)]
    pub development_level: u8,
    /// The owner's research level of the product against the leader (Q1, 0–1).
    #[serde(default)]
    pub development_share: f64,
    /// `laeuft`, `stillgelegt` or `wiederanlauf` (M22).
    pub operation: String,
    /// Shut down since, or producing again from (`stillgelegt`, `wiederanlauf`).
    pub operation_date: Option<String>,
    /// Book value of all units and what selling them would bring now (M22).
    pub book_value_usd: f64,
    pub sale_value_usd: f64,
    /// Starting the facility up again: one-off cost and days.
    pub restart_cost_usd: f64,
    pub restart_days: u32,
    /// Maintenance per month while running and while shut down.
    pub maintenance_month_usd: f64,
    pub maintenance_mothballed_month_usd: f64,
    /// Emissions per day at the planned utilization (H2): t CO2, kg of pollutants.
    #[serde(default)]
    pub co2_t_per_day: f64,
    #[serde(default)]
    pub pollutant_kg_per_day: f64,
    /// Retrofit level, the level the country requires, and what the next level costs.
    #[serde(default)]
    pub retrofit: u32,
    #[serde(default)]
    pub retrofit_required: u32,
    #[serde(default)]
    pub retrofit_cost_usd: Option<f64>,
    /// How the quality of the next batch comes about with today's stock of inputs.
    #[serde(default)]
    pub quality: Option<QualityView>,
    /// Standing after a breakdown or for an overhaul until the day (BP2).
    #[serde(default)]
    pub down_until: Option<String>,
    #[serde(default)]
    pub down_overhaul: bool,
    /// Cost of an overhaul; none without the maintenance model.
    #[serde(default)]
    pub overhaul_cost_usd: Option<f64>,
    /// Expected breakdowns per year at today's condition.
    #[serde(default)]
    pub failures_year: f64,
    /// Share of the output lost as scrap (BP4).
    #[serde(default)]
    pub scrap: f64,
}

/// Quality of a product in points (docs/FORMELN.md): recipe, inputs, automation,
/// condition, research level and training; `total` is the sum, 0–100.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QualityView {
    /// Average quality of the inputs in stock, weighted by the recipe (`None` without
    /// inputs or stock).
    pub input_quality: Option<f64>,
    pub base: f64,
    pub inputs: f64,
    pub automation: f64,
    pub condition: f64,
    pub research: f64,
    pub training: f64,
    /// Points from the trim level (MN2).
    #[serde(default)]
    pub trim: f64,
    pub total: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfferView {
    pub product: String,
    /// `markt`, `fest` or `aufschlag` (cost plus `markup` as the margin)
    pub mode: String,
    pub price_usd: f64,
    pub floor_usd: f64,
    pub markup: f64,
    pub keep: f64,
    /// Sold in the running month.
    pub sold_month: f64,
    pub sold_last_month: f64,
    /// In stock at the site.
    pub stock: f64,
    /// Price index of the product in the site's country.
    pub market_price_usd: f64,
    /// Reference price in the site's country.
    pub reference_usd: f64,
    /// Expected cost per unit: of making it here, else of what is in stock.
    pub unit_cost_usd: Option<f64>,
    /// (price − unit cost) / price.
    pub margin: Option<f64>,
    /// The site uses the product itself (an input of its facilities).
    pub used_here: bool,
    /// The company's own name for the product (M42).
    #[serde(default)]
    pub product_name: Option<String>,
    /// The company's sales offices that offer the product, and whether this site supplies
    /// them by need at cost.
    #[serde(default)]
    pub outlets: Vec<OutletView>,
    /// Trim level (`einfach`, `standard`, `premium`) of an end product made here (MN2);
    /// `None` where there is none to choose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trim: Option<String>,
}

/// A sales office of the same company that offers a product.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutletView {
    pub site: u32,
    pub name: Option<String>,
    pub country: String,
    /// It draws the product from this site.
    pub supplied_here: bool,
}

fn outlets(game: &Game, site: crate::state::SiteId, p: ProductId) -> Vec<OutletView> {
    let (state, catalog) = (game.state(), game.catalog());
    let Some(s) = state.site(site) else {
        return Vec::new();
    };
    state
        .sites
        .iter()
        .enumerate()
        .filter(|&(i, o)| {
            i != site.index()
                && o.kind == SiteType::SalesOffice
                && o.held_by(s.owner)
                && o.offers.contains_key(&p)
        })
        .map(|(i, o)| OutletView {
            site: u32::try_from(i).unwrap_or(u32::MAX),
            name: o.name.clone(),
            country: catalog.countries.key(o.country).to_owned(),
            supplied_here: o.supplies.get(&p).is_some_and(|x| x.source == site),
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrderView {
    pub product: String,
    pub target: f64,
    pub max_price_usd: f64,
    pub min_quality: f64,
    /// Bought in the running month.
    pub bought_month: f64,
    pub bought_last_month: f64,
    /// One-off purchase still to buy and its price limit (`BuyOnce`).
    #[serde(default)]
    pub once: f64,
    #[serde(default)]
    pub once_max_price_usd: f64,
}

/// An input of the site's production: daily need, stock and how long it lasts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputSupply {
    pub product: String,
    pub need_per_day: f64,
    pub stock: f64,
    /// Days the stock lasts at the planned production (`None` without need).
    pub days: Option<f64>,
    /// Made at the same site.
    pub own: bool,
    /// Bought by a purchase order of the site.
    pub ordered: bool,
    /// Price index in the site's country.
    pub market_price_usd: f64,
    /// Quality of the stock, if there is any.
    #[serde(default)]
    pub quality: Option<f64>,
}

/// A labor group at a site (M18): what production needs, who works there, who is free
/// in the country, and the wage the site pays.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StaffLine {
    pub group: String,
    pub needed: f64,
    pub employed: f64,
    /// Workers of the group in the country that no site employs.
    pub free_in_country: f64,
    /// Wage per hour at the site, with its premium.
    pub wage_usd: f64,
    /// Wage per hour in the country.
    pub country_wage_usd: f64,
}

/// Maintenance of a site (BP2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceView {
    pub level: f64,
    pub min: f64,
    pub max: f64,
    /// Breakdowns this year.
    pub failures: u32,
    /// Accidents this year and the rate per 100 000 working hours (BP4).
    #[serde(default)]
    pub accidents: u32,
    #[serde(default)]
    pub accident_rate: f64,
    /// Condition below which facilities are overhauled automatically: the site's own
    /// setting, and the one that holds (own or strategy's).
    #[serde(default)]
    pub overhaul_below: Option<f64>,
    #[serde(default)]
    pub overhaul_effective: Option<f64>,
}

/// Expected cost of one unit of a product made at the site, by kind (USD per unit).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitCostView {
    pub product: String,
    pub output_per_day: f64,
    pub material_usd: f64,
    pub labor_usd: f64,
    pub energy_usd: f64,
    pub overhead_usd: f64,
    pub rent_usd: f64,
    /// Depreciation and maintenance of the facilities.
    pub facility_usd: f64,
    pub total_usd: f64,
    /// Without the facilities: what one more unit costs.
    pub variable_usd: f64,
    /// Learning factor on the hours (1 = no experience; BP1).
    pub experience: f64,
    /// Units the site has made (experience).
    pub experience_units: f64,
    /// Labor cost per unit the experience saves.
    pub learning_saving_usd: f64,
}

/// An amount by cost type, e.g. `kostenart.material` (income positive).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResultLine {
    pub key: String,
    pub usd: f64,
}

/// Revenue and gross margin of a product at a site (M18): revenue less the production
/// cost of what was sold.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductResult {
    pub product: String,
    pub revenue_usd: f64,
    pub margin_usd: f64,
}

/// A site's result in the last closed month (M18).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteResult {
    /// First day of the month.
    pub month: String,
    pub revenue_usd: f64,
    /// Everything but revenue by cost type, in the order of the income statement.
    pub lines: Vec<ResultLine>,
    pub result_usd: f64,
    pub products: Vec<ProductResult>,
}

/// A deposit an extraction site without deposit could develop.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DepositOption {
    pub key: String,
    pub resource: String,
    pub cost_usd: f64,
    pub days: u32,
    /// Yearly output of the free field.
    pub output_per_year: f64,
}

/// What closing a site brings and costs today (docs/FORMELN.md, Standort schließen).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClosingView {
    pub facilities_usd: f64,
    pub building_usd: f64,
    pub land_usd: f64,
    pub stock_usd: f64,
    pub proceeds_usd: f64,
    /// Book values written off.
    pub book_usd: f64,
    pub severance_usd: f64,
    /// Proceeds less book values and severance.
    pub result_usd: f64,
    /// Why the site cannot be closed now.
    pub blocker: Option<crate::message::Message>,
}

/// The store of a site (docs/FORMELN.md, Lager): room in m³, the stock's value at
/// reference prices.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StorageView {
    pub capacity_m3: f64,
    pub fill_m3: f64,
    pub value_usd: f64,
    /// Fill as share of the capacity.
    pub share: f64,
    pub full: bool,
    /// The site's own products fill the store: the facilities stand.
    #[serde(default)]
    pub stops: bool,
    pub steps: u32,
    pub step_capacity_m3: f64,
    pub step_cost_usd: f64,
    pub cost_per_day_usd: f64,
    /// Goods lie in outside storage at a higher rate (BP3).
    #[serde(default)]
    pub external: bool,
    /// Room up to which outside storage takes goods.
    #[serde(default)]
    pub hard_capacity_m3: f64,
}

fn storage_view(game: &Game, site: &crate::state::Site) -> Option<StorageView> {
    let catalog = game.catalog();
    let capacity = crate::storage::capacity(catalog, site)?;
    let fill = crate::storage::fill(catalog, site);
    let m = &catalog.production_model.storage;
    let value = site
        .inventory
        .iter()
        .map(|(&p, s)| catalog.products.get(p).reference_price.to_usd() * s.quantity.max(0.0))
        .sum();
    Some(StorageView {
        capacity_m3: capacity,
        fill_m3: fill,
        value_usd: value,
        share: if capacity > 0.0 { fill / capacity } else { 0.0 },
        full: crate::storage::is_full(catalog, site),
        stops: crate::storage::stops_output(catalog, site),
        external: crate::storage::external(catalog, site),
        hard_capacity_m3: crate::storage::hard_capacity(catalog, site).unwrap_or(0.0),
        steps: site.storage_steps,
        step_capacity_m3: m.step_capacity,
        step_cost_usd: usd(m.step_cost),
        cost_per_day_usd: usd(crate::storage::daily_cost(catalog, site)),
    })
}

fn closing_view(game: &Game, site: crate::state::SiteId) -> ClosingView {
    let (state, catalog) = (game.state(), game.catalog());
    let c = crate::closing::value(state, catalog, site);
    ClosingView {
        facilities_usd: usd(c.facilities),
        building_usd: usd(c.building),
        land_usd: usd(c.land),
        stock_usd: usd(c.stock),
        proceeds_usd: usd(c.proceeds()),
        book_usd: usd(c.book),
        severance_usd: usd(c.severance),
        result_usd: usd(c.result()),
        blocker: crate::closing::blocker(state, catalog, site).map(|e| e.message()),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteDetail {
    pub index: u32,
    /// The works' branch under mandatory research (FW1; key), none before its first product.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Products the company may buy a unit for here (FW1): known products of the works'
    /// branch, or of any branch while the works has none. Empty without mandatory research.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub buy_products: Vec<String>,
    pub country: String,
    /// Key of the country's continent, to group the sites.
    #[serde(default)]
    pub continent: String,
    /// For commands, e.g. `Factory`.
    pub kind: SiteType,
    /// Text key, e.g. `standorttyp.werk`.
    pub kind_text: String,
    /// The player's own name for the site, if any.
    #[serde(default)]
    pub name: Option<String>,
    pub deposit: Option<String>,
    /// Day the development of the site's deposit field is finished.
    pub deposit_ready: Option<String>,
    /// Deposits to develop (extraction sites without deposit only).
    pub free_deposits: Vec<DepositOption>,
    pub workers: f64,
    pub slots: Vec<SlotDetail>,
    pub stock: Vec<super::StockView>,
    pub offers: Vec<OfferView>,
    pub orders: Vec<OrderView>,
    pub inputs: Vec<InputSupply>,
    pub research: Option<String>,
    /// Product the research center develops (M37).
    #[serde(default)]
    pub development: Option<String>,
    /// Premium over the country's wages (0.1 = 10 %) and its highest allowed value.
    pub wage_premium: f64,
    pub wage_premium_max: f64,
    /// Highest premium another company pays at a site in the same country.
    pub rival_premium_max: f64,
    pub staff: Vec<StaffLine>,
    /// Wages per day of the staff employed now.
    pub wage_cost_per_day_usd: f64,
    /// Training (W1): level reached and target, 0–1.
    #[serde(default)]
    pub training: f64,
    #[serde(default)]
    pub training_target: f64,
    /// Where the target comes from: `standort`, `strategie` or `keine`.
    #[serde(default)]
    pub training_source: String,
    /// Training costs per day at today's wages.
    #[serde(default)]
    pub training_cost_per_day_usd: f64,
    /// Effect of the level reached: share of labor hours saved, quality points.
    #[serde(default)]
    pub training_labor_saving: f64,
    #[serde(default)]
    pub training_quality: f64,
    /// Maintenance level and its range (BP2); none without the maintenance model.
    #[serde(default)]
    pub maintenance: Option<MaintenanceView>,
    pub unit_costs: Vec<UnitCostView>,
    /// Result of the last closed month, if the site existed then.
    pub last_month: Option<SiteResult>,
    /// Prices of all products in the site's country.
    pub prices: BTreeMap<String, PriceInfo>,
    /// The plot the site stands on (M35); `None` for extraction sites.
    #[serde(default)]
    pub plot: Option<SitePlotView>,
    /// Running and proposed supply contracts of the site.
    #[serde(default)]
    pub contracts: Vec<super::ContractView>,
    /// Products the site draws from other sites of the company or group, and the
    /// suggested reach for supply by need.
    #[serde(default)]
    pub supplies: Vec<super::SupplyView>,
    #[serde(default)]
    pub supply_days_default: f64,
    /// What closing the site would bring and cost today.
    #[serde(default)]
    pub closing: Option<ClosingView>,
    /// The site's store; `None` when stocks are unlimited.
    #[serde(default)]
    pub storage: Option<StorageView>,
    /// Market prices of the last twelve months in the site's country for every product
    /// the site buys, sells, stores or has under contract.
    #[serde(default)]
    pub price_trends: BTreeMap<String, PriceTrendView>,
    /// Sites of the same type of the company and its group a facility can be moved to.
    #[serde(default)]
    pub move_targets: Vec<MoveTargetView>,
}

/// A site a facility can be moved to: within the company free, to another company of
/// the group as a sale.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MoveTargetView {
    pub site: u32,
    pub name: Option<String>,
    pub country: String,
    pub company: String,
    /// The same company as the site's: the move is free.
    pub same_company: bool,
}

fn move_targets(game: &Game, site: crate::state::SiteId) -> Vec<MoveTargetView> {
    let (state, catalog) = (game.state(), game.catalog());
    let Some(s) = state.site(site) else {
        return Vec::new();
    };
    state
        .sites
        .iter()
        .enumerate()
        .filter(|&(i, t)| {
            i != site.index()
                && t.closed.is_none()
                && t.kind == s.kind
                && crate::group::same_group(state, s.owner, t.owner)
        })
        .map(|(i, t)| MoveTargetView {
            site: u32::try_from(i).unwrap_or(u32::MAX),
            name: t.name.clone(),
            country: catalog.countries.key(t.country).to_owned(),
            company: state.companies[t.owner.index()].name.clone(),
            same_company: t.owner == s.owner,
        })
        .collect()
}

/// Market price of a product over the last twelve closed months.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PriceTrendView {
    /// Oldest first; months without sales have no price.
    pub months: Vec<MarketMonthView>,
    /// Average price of the month twelve months before the last closed one.
    pub year_ago_usd: Option<f64>,
    /// Last closed month against `year_ago_usd` (0.1 = 10 % dearer).
    pub change: Option<f64>,
    pub low_usd: Option<f64>,
    pub high_usd: Option<f64>,
}

/// Twelve months shown, plus the month a year before the last for the comparison.
const TREND_MONTHS: usize = 12;

fn price_trend(state: &crate::state::GameState, m: &Market) -> PriceTrendView {
    let all = market_history(state, m);
    let year_ago = all
        .len()
        .checked_sub(TREND_MONTHS + 1)
        .and_then(|i| all[i].price_usd);
    let months: Vec<MarketMonthView> = all[all.len().saturating_sub(TREND_MONTHS)..].to_vec();
    let prices = || months.iter().filter_map(|v| v.price_usd);
    let last = months.last().and_then(|v| v.price_usd);
    PriceTrendView {
        change: last
            .zip(year_ago)
            .filter(|&(_, a)| a > 0.0)
            .map(|(l, a)| l / a - 1.0),
        year_ago_usd: year_ago,
        low_usd: prices().reduce(f64::min),
        high_usd: prices().reduce(f64::max),
        months,
    }
}

/// The plot of a site (M35).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SitePlotView {
    /// Key of the location (text `lage.<key>`).
    pub location: String,
    pub area_ha: f64,
    /// Land the facilities take with ways and offices.
    pub used_ha: f64,
    pub owned: bool,
    /// Today's value of the land.
    pub value_usd: f64,
    /// Rent per year while leased.
    pub rent_usd_year: Option<f64>,
    /// Units of each known facility of the site's type that still fit on the plot, by
    /// size (`FacilitySize::ALL` order, M36).
    #[serde(default)]
    pub fits: BTreeMap<String, Vec<u32>>,
}

/// Price index and reference price of a product in a country.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PriceInfo {
    pub market_usd: f64,
    pub reference_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteTypeOption {
    pub kind: SiteType,
    pub kind_text: String,
    pub cost_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FacilityOption {
    pub key: String,
    pub site_type: SiteType,
    pub investment_usd: f64,
    pub build_days: u32,
    pub runs_per_day: f64,
    pub automation_max: f64,
    /// Recipes of this facility the company knows.
    pub recipes: Vec<String>,
    /// Land per unit with ways and offices (M35; 0 without plots).
    #[serde(default)]
    pub area_ha: f64,
    /// The sizes to build (M36), the smallest first.
    #[serde(default)]
    pub sizes: Vec<SizeOption>,
}

/// A size of a facility to build (M36).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SizeOption {
    /// For `BuildFacility`, e.g. `Large`.
    pub size: FacilitySize,
    /// Text key `anlagengroesse.<key>`.
    pub key: String,
    /// Capacity as a multiple of the data size.
    pub capacity: f64,
    pub investment_usd: f64,
    pub build_days: u32,
    /// Land per unit with ways and offices (0 without plots).
    pub area_ha: f64,
    /// Labor per unit made as a multiple of the data size.
    pub labor_per_unit: f64,
}

fn medium_key() -> String {
    FacilitySize::Medium.key().to_owned()
}

fn one() -> f64 {
    1.0
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecipeOption {
    pub key: String,
    pub facility: String,
    pub product: String,
    pub output: f64,
    /// Output of one facility per day at full utilization, and the inputs it takes.
    pub output_per_day: f64,
    pub inputs_per_day: Vec<(String, f64)>,
    pub duration_days: u32,
    pub extraction: bool,
    pub inputs: Vec<(String, f64)>,
    pub labor_hours: Vec<(String, f64)>,
    pub energy_mwh: f64,
}

/// The player's sites and what the player could build (technologies known today).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductionView {
    pub date: String,
    pub cash_usd: f64,
    pub sites: Vec<SiteDetail>,
    pub site_types: Vec<SiteTypeOption>,
    pub facilities: Vec<FacilityOption>,
    pub recipes: Vec<RecipeOption>,
    /// All products (for new purchase orders and offers).
    pub products: Vec<String>,
    /// Unit of each product (key of `einheit.<key>`).
    pub units: BTreeMap<String, String>,
    /// The player's running machine tenders with their offers (MAA).
    #[serde(default)]
    pub tenders: Vec<MachineTenderView>,
    /// Facilities are asked for in tenders instead of bought directly (MAA).
    #[serde(default)]
    pub machine_tenders_enabled: bool,
}

/// A tender for machines: a build or a changeover, with the offers received so far.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MachineTenderView {
    pub id: u32,
    pub site: u32,
    /// `bauen` or `umbauen`.
    pub kind: String,
    pub product: String,
    /// Key of the size (`anlagengroesse.<key>`) and the number of units.
    pub size: String,
    pub count: u32,
    /// The unit changed over, for a changeover.
    pub slot: Option<usize>,
    pub start: String,
    /// Day of the tender today, the days it runs, and its last day.
    pub day: i32,
    pub days: u32,
    pub ends: String,
    pub offers: Vec<MachineOfferView>,
}

/// What an offer would make of one facility, as factors on the catalog values.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MachineFactorsView {
    pub output: f64,
    pub labor: f64,
    pub inputs: f64,
    pub scrap: f64,
    pub area: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaterialLineView {
    pub product: String,
    pub quantity: f64,
    pub price_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MachineOfferView {
    pub id: u32,
    /// Key of the builder (`maschinenbauer.<key>`) and of its country.
    pub builder: String,
    pub builder_country: String,
    pub received: String,
    pub valid_until: String,
    pub price_usd: f64,
    /// What the same units cost at the catalog.
    pub catalog_price_usd: f64,
    pub build_days: u32,
    /// Build time at the catalog, for the comparison.
    pub catalog_build_days: u32,
    /// Day the units are ready when the offer is taken today.
    pub ready: String,
    pub factors: MachineFactorsView,
    /// Land of one unit and of all, and the free land of the plot before and after (m²).
    pub area_unit_m2: f64,
    pub area_total_m2: f64,
    pub free_before_m2: Option<f64>,
    pub free_after_m2: Option<f64>,
    /// Material at the market prices of the day the offer came in, per unit made.
    pub materials: Vec<MaterialLineView>,
    pub material_usd: f64,
    /// All costs per unit made with the offer, and with the facility at the catalog.
    pub unit_cost_usd: f64,
    pub catalog_unit_cost_usd: f64,
}

fn machine_tenders(game: &Game) -> Vec<MachineTenderView> {
    let state = game.state();
    let catalog = game.catalog();
    let player = game.player();
    let model = &catalog.machine_offers;
    state
        .machine_tenders
        .iter()
        .filter(|t| t.company == player)
        .filter_map(|t| {
            let base = crate::procurement::base(state, catalog, t.site, t.kind).ok()?;
            let (kind, product, slot) = match t.kind {
                crate::procurement::TenderKind::Build { product, .. } => ("bauen", product, None),
                crate::procurement::TenderKind::Switch { slot, product } => {
                    ("umbauen", product, Some(slot))
                }
            };
            let m2 = |ha: f64| ha * 10_000.0;
            let offers = t
                .offers
                .iter()
                .filter_map(|o| {
                    let costs = crate::procurement::offer_costs(state, catalog, t, o)?;
                    let area = crate::procurement::offer_area(state, catalog, t, o)?;
                    Some(MachineOfferView {
                        id: o.id,
                        builder: catalog.machine_builders.key(o.builder).to_owned(),
                        builder_country: catalog
                            .countries
                            .key(catalog.machine_builders.get(o.builder).country)
                            .to_owned(),
                        received: iso(o.received),
                        valid_until: iso(o.valid_until),
                        price_usd: usd(o.price),
                        catalog_price_usd: usd(base.price()),
                        build_days: o.build_days,
                        catalog_build_days: base.days,
                        ready: iso(state
                            .date
                            .add_days(i32::try_from(o.build_days).unwrap_or(0))),
                        factors: MachineFactorsView {
                            output: o.factors.output,
                            labor: o.factors.labor,
                            inputs: o.factors.inputs,
                            scrap: o.factors.scrap,
                            area: o.factors.area,
                        },
                        area_unit_m2: m2(area.unit_ha),
                        area_total_m2: m2(area.total_ha),
                        free_before_m2: area.free_before_ha.map(m2),
                        free_after_m2: area.free_after_ha.map(m2),
                        materials: costs
                            .materials
                            .iter()
                            .map(|l| MaterialLineView {
                                product: catalog.products.key(l.product).to_owned(),
                                quantity: l.quantity,
                                price_usd: l.price_usd,
                            })
                            .collect(),
                        material_usd: costs.material_usd,
                        unit_cost_usd: costs.unit_cost_usd,
                        catalog_unit_cost_usd: costs.catalog_unit_cost_usd,
                    })
                })
                .collect();
            Some(MachineTenderView {
                id: t.id,
                site: t.site.0,
                kind: kind.to_owned(),
                product: catalog.products.key(product).to_owned(),
                size: base.size.key().to_owned(),
                count: base.count,
                slot,
                start: iso(t.start),
                day: t.day(state.date),
                days: model.tender_days,
                ends: iso(t
                    .start
                    .add_days(i32::try_from(model.tender_days).unwrap_or(0) - 1)),
                offers,
            })
        })
        .collect()
}

pub fn production(game: &Game) -> ProductionView {
    let state = game.state();
    let catalog = game.catalog();
    let player = game.player();
    let knows =
        |t: Option<crate::ids::TechnologyId>| t.is_none_or(|t| state.knows(catalog, player, t));
    let recipes: Vec<RecipeOption> = catalog
        .recipes
        .iter()
        .filter(|(_, r)| {
            knows(r.technology)
                && knows(catalog.facilities.get(r.facility).technology)
                && state.knows_product(catalog, player, r.product)
        })
        .map(|(id, r)| RecipeOption {
            key: catalog.recipes.key(id).to_owned(),
            facility: catalog.facilities.key(r.facility).to_owned(),
            product: catalog.products.key(r.product).to_owned(),
            output: r.output,
            output_per_day: r.output * catalog.facilities.get(r.facility).runs_per_day,
            inputs_per_day: r
                .inputs
                .iter()
                .map(|&(p, q)| {
                    (
                        catalog.products.key(p).to_owned(),
                        q * catalog.facilities.get(r.facility).runs_per_day,
                    )
                })
                .collect(),
            duration_days: r.duration_days,
            extraction: r.extraction,
            inputs: r
                .inputs
                .iter()
                .map(|&(p, q)| (catalog.products.key(p).to_owned(), q))
                .collect(),
            labor_hours: r
                .labor_hours
                .iter()
                .map(|&(g, h)| (catalog.labor_groups.key(g).to_owned(), h))
                .collect(),
            energy_mwh: r.energy_mwh,
        })
        .collect();
    let facilities = catalog
        .facilities
        .iter()
        .filter(|(_, f)| knows(f.technology))
        .map(|(id, f)| {
            let key = catalog.facilities.key(id).to_owned();
            FacilityOption {
                recipes: recipes
                    .iter()
                    .filter(|r| r.facility == key)
                    .map(|r| r.key.clone())
                    .collect(),
                key,
                site_type: f.site_type,
                investment_usd: usd(f.investment),
                build_days: f.build_days,
                runs_per_day: f.runs_per_day,
                automation_max: f.automation_max,
                area_ha: if catalog.plot_model.enabled() {
                    crate::plots::facility_area(catalog, id) * (1.0 + catalog.plot_model.overhead)
                } else {
                    0.0
                },
                sizes: FacilitySize::ALL
                    .iter()
                    .map(|&size| {
                        let sizes = &catalog.production_model.sizes;
                        SizeOption {
                            size,
                            key: size.key().to_owned(),
                            capacity: sizes.capacity(size),
                            investment_usd: usd(f.investment.scale(sizes.investment(size))),
                            build_days: sizes.build_days(size, f.build_days),
                            area_ha: if catalog.plot_model.enabled() {
                                crate::plots::unit_area(catalog, id, size)
                                    * (1.0 + catalog.plot_model.overhead)
                            } else {
                                0.0
                            },
                            labor_per_unit: sizes.labor(size),
                        }
                    })
                    .collect(),
            }
        })
        .collect();
    let site_types = [
        SiteType::Extraction,
        SiteType::Factory,
        SiteType::PowerPlant,
        SiteType::Warehouse,
        SiteType::SalesOffice,
        SiteType::ResearchCenter,
    ]
    .into_iter()
    .map(|kind| SiteTypeOption {
        kind,
        kind_text: site_type_key(kind),
        cost_usd: usd(catalog.production_model.site_cost(kind)),
    })
    .collect();

    let sites = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.held_by(player))
        .map(|(i, s)| {
            let site_id = crate::state::SiteId(u32::try_from(i).unwrap_or(u32::MAX));
            let concession = s.deposit.and_then(|d| {
                state
                    .deposits
                    .get(d)
                    .concessions
                    .iter()
                    .find(|c| c.site == Some(site_id))
            });
            let developing = concession.is_none_or(|c| c.ready.is_none_or(|r| r > state.date));
            let mut need: Vec<(crate::ids::ProductId, f64)> = Vec::new();
            let slots = s
                .slots
                .iter()
                .enumerate()
                .map(|(index, sl)| {
                    let f = catalog.facilities.get(sl.facility);
                    let recipe = sl.recipe.map(|r| catalog.recipes.get(r));
                    // A shut down facility needs nothing (M22).
                    let runs = if sl.mothballed() {
                        0.0
                    } else {
                        sl.full_runs(catalog) * sl.utilization
                    };
                    if let Some(r) = recipe {
                        // With what the owner's research saves in inputs.
                        let saved =
                            crate::production::site_effect(catalog, state, s, r.product).inputs;
                        for &(p, q) in &r.inputs {
                            match need.iter_mut().find(|(x, _)| *x == p) {
                                Some((_, n)) => *n += q * saved * runs,
                                None => need.push((p, q * saved * runs)),
                            }
                        }
                    }
                    let lab = f.site_type == SiteType::ResearchCenter;
                    let model = &catalog.production_model;
                    let sale = crate::production::sale_value(catalog, sl, sl.count, state.date);
                    let cause = if sl.ready > state.date {
                        Some(("ursache.im_bau", None))
                    } else if sl.mothballed() {
                        Some(("ursache.stillgelegt", None))
                    } else if !sl.operating(state.date) {
                        Some(("ursache.wiederanlauf", None))
                    } else if recipe.is_none() && !lab {
                        Some(("ursache.kein_rezept", None))
                    } else if sl.utilization <= 0.0 {
                        Some(("ursache.ruht", None))
                    } else if recipe.is_some_and(|r| r.extraction) && developing {
                        Some(("ursache.erschliessung", None))
                    } else {
                        sl.limit.map(|l| match l {
                            Limit::Input(p) => (
                                "ursache.vorprodukt",
                                Some(catalog.products.key(p).to_owned()),
                            ),
                            Limit::Labor(g) => (
                                "ursache.arbeitskraefte",
                                Some(catalog.labor_groups.key(g).to_owned()),
                            ),
                            Limit::Electricity => ("ursache.strom", None),
                            Limit::Deposit => ("ursache.lagerstaette", None),
                            Limit::Event => ("ursache.ereignis", None),
                            Limit::Regulation => ("ursache.regulierung", None),
                            Limit::Patent => ("ursache.patent", None),
                            Limit::Storage => ("ursache.lager_voll", None),
                            Limit::Breakdown => ("ursache.ausfall", None),
                            Limit::Overhaul => ("ursache.ueberholung", None),
                        })
                    };
                    SlotDetail {
                        index,
                        facility: catalog.facilities.key(sl.facility).to_owned(),
                        count: sl.count,
                        size: sl.size.key().to_owned(),
                        capacity: catalog.production_model.sizes.capacity(sl.size),
                        recipe: sl.recipe.map(|r| catalog.recipes.key(r).to_owned()),
                        product: recipe.map(|r| catalog.products.key(r.product).to_owned()),
                        utilization: sl.utilization,
                        automation: sl.automation,
                        automation_max: f.automation_max,
                        ready: iso(sl.ready),
                        // With the output research of the owner (manufacturing research).
                        planned_per_day: recipe.map_or(0.0, |r| {
                            runs * r.output
                                * crate::production::site_effect(catalog, state, s, r.product)
                                    .output
                        }),
                        made_per_day: recipe.map_or(0.0, |r| {
                            sl.last_runs
                                * r.output
                                * crate::production::site_effect(catalog, state, s, r.product)
                                    .output
                        }),
                        development_level: recipe.map_or(0, |r| {
                            crate::development::level(catalog, state, s.owner, r.product)
                        }),
                        development_share: recipe.map_or(0.0, |r| {
                            crate::development::share(state, s.owner, r.product)
                        }),
                        cause: cause.map(|(key, detail)| Cause {
                            key: key.to_owned(),
                            detail,
                        }),
                        condition: sl.condition,
                        inputs_per_day: recipe.map_or_else(Vec::new, |r| {
                            // The company's development level saves inputs (M37).
                            let saved =
                                crate::production::site_effect(catalog, state, s, r.product).inputs;
                            r.inputs
                                .iter()
                                .map(|&(p, q)| {
                                    (catalog.products.key(p).to_owned(), q * saved * runs)
                                })
                                .collect()
                        }),
                        operation: match sl.operation {
                            Operation::Running => "laeuft",
                            Operation::Mothballed { .. } => "stillgelegt",
                            Operation::Restarting { .. } => "wiederanlauf",
                        }
                        .to_owned(),
                        operation_date: match sl.operation {
                            Operation::Running => None,
                            Operation::Mothballed { since } => Some(iso(since)),
                            Operation::Restarting { until } => Some(iso(until)),
                        },
                        book_value_usd: usd(sale.0),
                        sale_value_usd: usd(sale.1),
                        restart_cost_usd: usd(sl.cost.scale(model.restart_cost_share)),
                        restart_days: model.restart_days,
                        maintenance_month_usd: usd(sl
                            .cost
                            .scale(f.maintenance_share * s.maintenance / 12.0)),
                        maintenance_mothballed_month_usd: usd(sl
                            .cost
                            .scale(f.maintenance_share * model.mothball_maintenance_share / 12.0)),
                        co2_t_per_day: sl.recipe.map_or(0.0, |r| {
                            let r = catalog.recipes.get(r);
                            r.co2_t * planned_runs_per_day(catalog, sl)
                        }),
                        pollutant_kg_per_day: sl.recipe.map_or(0.0, |r| {
                            let r = catalog.recipes.get(r);
                            crate::regulation::pollutant(catalog, r, sl.retrofit)
                                * planned_runs_per_day(catalog, sl)
                        }),
                        retrofit: sl.retrofit,
                        retrofit_required: state.regulation.required(s.country),
                        retrofit_cost_usd: crate::regulation::retrofit_cost(
                            catalog, state, site_id, index,
                        )
                        .map(usd),
                        down_until: crate::maintenance::down_on(sl, state.date)
                            .map(|d| iso(d.until)),
                        down_overhaul: crate::maintenance::down_on(sl, state.date)
                            .is_some_and(|d| d.overhaul),
                        overhaul_cost_usd: catalog
                            .production_model
                            .maintenance
                            .enabled
                            .then(|| usd(crate::maintenance::overhaul_cost(catalog, sl))),
                        failures_year: 365.0
                            * crate::maintenance::failure_chance(catalog, sl.condition),
                        scrap: recipe.map_or(0.0, |r| {
                            crate::operations::scrap_rate(catalog, game.state(), s, index, r)
                        }),
                        quality: recipe.map(|r| slot_quality(game, site_id, index, r)),
                    }
                })
                .collect();
            let made: Vec<crate::ids::ProductId> = s
                .slots
                .iter()
                .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r).product))
                .collect();
            let inputs = need
                .iter()
                .map(|&(p, per_day)| {
                    let stock = s.inventory.get(&p).map_or(0.0, |x| x.quantity);
                    InputSupply {
                        product: catalog.products.key(p).to_owned(),
                        need_per_day: per_day,
                        stock,
                        days: (per_day > 1e-12).then(|| stock / per_day),
                        own: made.contains(&p),
                        ordered: s.orders.contains_key(&p),
                        market_price_usd: usd(market::market_price(catalog, state, s.country, p)),
                        quality: s
                            .inventory
                            .get(&p)
                            .filter(|x| x.quantity > 1e-9)
                            .map(|x| x.quality),
                    }
                })
                .collect();
            let unit_costs = crate::production::unit_costs(catalog, state, site_id);
            let used_here: Vec<crate::ids::ProductId> = need.iter().map(|&(p, _)| p).collect();
            let (staff, rival_premium_max, wage_cost_per_day_usd) = site_staff(game, site_id);
            let (training_target, training_source) =
                crate::training::target(catalog, state, site_id);
            SiteDetail {
                index: u32::try_from(i).unwrap_or(u32::MAX),
                branch: s.branch.map(|b| catalog.branches.key(b).to_owned()),
                buy_products: buy_products(catalog, state, player, s.branch),
                country: catalog.countries.key(s.country).to_owned(),
                continent: catalog
                    .continents
                    .key(catalog.countries.get(s.country).continent)
                    .to_owned(),
                kind: s.kind,
                kind_text: site_type_key(s.kind),
                name: s.name.clone(),
                deposit: s.deposit.map(|d| catalog.deposits.key(d).to_owned()),
                deposit_ready: concession.and_then(|c| c.ready).map(iso),
                free_deposits: if s.kind == SiteType::Extraction && s.deposit.is_none() {
                    free_deposits(game, s.country)
                } else {
                    Vec::new()
                },
                workers: s.workforce.values().sum(),
                slots,
                stock: super::stock_views(state, catalog, site_id),
                offers: s
                    .offers
                    .iter()
                    .map(|(p, o)| {
                        let (mode, markup, floor) = match o.mode {
                            PriceMode::Fixed(_) => ("fest", 0.0, Money::ZERO),
                            PriceMode::CostPlus { margin } => ("aufschlag", margin, Money::ZERO),
                            PriceMode::Market { markup, floor } => ("markt", markup, floor),
                            PriceMode::Relative { deviation, floor } => {
                                ("marktbezug", deviation, floor)
                            }
                        };
                        let stock = s.inventory.get(p);
                        let unit_cost_usd = unit_costs
                            .iter()
                            .find(|u| u.product == *p)
                            .map(crate::production::UnitCost::total)
                            .or_else(|| {
                                stock
                                    .filter(|st| st.quantity > 1e-9)
                                    .map(|st| usd(st.value) / st.quantity)
                            });
                        OfferView {
                            product: catalog.products.key(*p).to_owned(),
                            mode: mode.to_owned(),
                            price_usd: usd(o.price),
                            floor_usd: usd(floor),
                            markup,
                            keep: o.keep,
                            sold_month: o.sold_month,
                            sold_last_month: o.sold_last_month,
                            stock: stock.map_or(0.0, |st| st.quantity),
                            market_price_usd: usd(market::market_price(
                                catalog, state, s.country, *p,
                            )),
                            reference_usd: usd(market::local_reference(
                                catalog, state, s.country, *p,
                            )),
                            margin: unit_cost_usd
                                .filter(|_| o.price > Money::ZERO)
                                .map(|c| 1.0 - c / usd(o.price)),
                            unit_cost_usd,
                            used_here: used_here.contains(p),
                            product_name: state.companies[s.owner.index()]
                                .product_names
                                .get(p)
                                .cloned(),
                            outlets: if s.kind == SiteType::SalesOffice {
                                Vec::new()
                            } else {
                                outlets(game, site_id, *p)
                            },
                            trim: (catalog.production_model.trims.enabled
                                && catalog.products.get(*p).kind
                                    == crate::catalog::ProductKind::EndProduct
                                && s.slots.iter().any(|sl| {
                                    sl.recipe
                                        .is_some_and(|r| catalog.recipes.get(r).product == *p)
                                }))
                            .then(|| crate::production::trim(s, *p).key().to_owned()),
                        }
                    })
                    .collect(),
                orders: s
                    .orders
                    .iter()
                    .map(|(p, o)| OrderView {
                        product: catalog.products.key(*p).to_owned(),
                        target: o.target,
                        max_price_usd: usd(o.max_price),
                        min_quality: o.min_quality,
                        bought_month: o.bought_month,
                        bought_last_month: o.bought_last_month,
                        once: o.once,
                        once_max_price_usd: usd(o.once_max_price),
                    })
                    .collect(),
                inputs,
                research: s.research.map(|t| catalog.technologies.key(t).to_owned()),
                development: crate::development::site_shares(s)
                    .first()
                    .map(|&(p, _)| catalog.products.key(p).to_owned()),
                wage_premium: s.wage_premium,
                wage_premium_max: catalog.production_model.wage_premium_max,
                rival_premium_max,
                staff,
                wage_cost_per_day_usd,
                training: s.training,
                training_target,
                training_source: match training_source {
                    crate::training::TargetSource::Site => "standort",
                    crate::training::TargetSource::Strategy => "strategie",
                    crate::training::TargetSource::None => "keine",
                }
                .into(),
                training_cost_per_day_usd: crate::training::daily_cost(
                    catalog,
                    state,
                    site_id,
                    wage_cost_per_day_usd,
                ),
                training_labor_saving: 1.0 - crate::training::labor_factor(catalog, s.training),
                training_quality: crate::training::quality(catalog, s.training),
                maintenance: {
                    let m = &catalog.production_model.maintenance;
                    m.enabled.then_some(MaintenanceView {
                        level: s.maintenance,
                        min: m.level_min,
                        max: m.level_max,
                        failures: s.failures,
                        accidents: s.accidents,
                        accident_rate: crate::operations::accident_rate(catalog, s, state.date),
                        overhaul_below: s.overhaul_below,
                        overhaul_effective: crate::maintenance::overhaul_threshold(
                            state, catalog, site_id,
                        ),
                    })
                },
                unit_costs: unit_costs
                    .iter()
                    .map(|u| {
                        let experience = s
                            .slots
                            .iter()
                            .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r)))
                            .find(|r| r.product == u.product)
                            .map_or(1.0, |r| crate::learning::factor(catalog, s, r));
                        UnitCostView {
                            product: catalog.products.key(u.product).to_owned(),
                            output_per_day: u.output_per_day,
                            material_usd: u.material,
                            labor_usd: u.labor,
                            energy_usd: u.energy,
                            overhead_usd: u.overhead,
                            rent_usd: u.rent,
                            facility_usd: u.facility,
                            total_usd: u.total(),
                            variable_usd: u.variable(),
                            experience,
                            experience_units: crate::learning::units(s, u.product),
                            learning_saving_usd: u.labor * (1.0 / experience - 1.0),
                        }
                    })
                    .collect(),
                last_month: site_result(game, site_id),
                // Every product: a branch picks what to trade by these prices.
                prices: catalog
                    .products
                    .iter()
                    .map(|(p, _)| {
                        (
                            catalog.products.key(p).to_owned(),
                            PriceInfo {
                                market_usd: usd(market::market_price(catalog, state, s.country, p)),
                                reference_usd: usd(market::local_reference(
                                    catalog, state, s.country, p,
                                )),
                            },
                        )
                    })
                    .collect(),
                plot: s.plot.map(|plot| {
                    let p = &state.plots[plot.index()];
                    let value = usd(crate::plots::value(catalog, state, plot));
                    let leased = p.tenure == crate::state::Tenure::Leased;
                    SitePlotView {
                        location: p.location.key().to_owned(),
                        area_ha: p.area_ha,
                        used_ha: crate::plots::site_area(catalog, s, None),
                        owned: !leased,
                        value_usd: value,
                        rent_usd_year: leased.then(|| value * catalog.plot_model.rent_share),
                        fits: catalog
                            .facilities
                            .iter()
                            .filter(|(_, f)| f.site_type == s.kind && knows(f.technology))
                            .map(|(f, _)| {
                                (
                                    catalog.facilities.key(f).to_owned(),
                                    FacilitySize::ALL
                                        .iter()
                                        .map(|&size| {
                                            crate::plots::units_that_fit(
                                                catalog,
                                                state,
                                                site_id,
                                                (f, size),
                                            )
                                        })
                                        .collect(),
                                )
                            })
                            .collect(),
                    }
                }),
                price_trends: {
                    let mut products: BTreeSet<ProductId> = s.offers.keys().copied().collect();
                    products.extend(s.orders.keys().copied());
                    products.extend(
                        s.inventory
                            .iter()
                            .filter(|(_, l)| l.quantity > 0.0)
                            .map(|(&p, _)| p),
                    );
                    products.extend(catalog.products.ids().filter(|&p| {
                        crate::contracts::daily_need(state, catalog, site_id, p) > 0.0
                    }));
                    products.extend(
                        state
                            .contracts
                            .iter()
                            .filter(|c| c.buyer == site_id || c.seller == site_id)
                            .map(|c| c.product),
                    );
                    products
                        .into_iter()
                        .map(|p| {
                            (
                                catalog.products.key(p).to_owned(),
                                price_trend(state, state.markets.get(p).get(s.country)),
                            )
                        })
                        .collect()
                },
                contracts: super::site_contracts(game, site_id),
                supplies: super::site_supplies(game, site_id),
                supply_days_default: catalog.logistics.supply_days_default,
                closing: Some(closing_view(game, site_id)),
                storage: storage_view(game, s),
                move_targets: move_targets(game, site_id),
            }
        })
        .collect();
    ProductionView {
        date: iso(state.date),
        cash_usd: usd(state.companies[player.index()].ledger.cash()),
        sites,
        site_types,
        facilities,
        recipes,
        products: catalog
            .products
            .iter()
            .map(|(p, _)| catalog.products.key(p).to_owned())
            .collect(),
        units: units(catalog),
        tenders: machine_tenders(game),
        machine_tenders_enabled: catalog.machine_offers.enabled,
    }
}

/// Unit of each product, by product key.
pub fn units(catalog: &crate::catalog::Catalog) -> BTreeMap<String, String> {
    catalog
        .products
        .iter()
        .map(|(p, x)| {
            (
                catalog.products.key(p).to_owned(),
                catalog.units.key(x.unit).to_owned(),
            )
        })
        .collect()
}

/// Staff of a site by labor group, the highest premium of other companies' sites in its
/// country, and the wages per day of the staff employed now.
fn site_staff(game: &Game, site: crate::state::SiteId) -> (Vec<StaffLine>, f64, f64) {
    let state = game.state();
    let catalog = game.catalog();
    let s = &state.sites[site.index()];
    let country = state.countries.get(s.country);
    let needed = crate::production::needed_workers(catalog, state, site, state.date);
    let hours = crate::production::hours_per_worker_day(catalog, state.date);
    let factor = 1.0 + s.wage_premium;
    let mut wage_cost = 0.0;
    let staff = catalog
        .labor_groups
        .ids()
        .map(|g| {
            let employed_in_country: f64 = state
                .sites
                .iter()
                .filter(|o| o.country == s.country)
                .map(|o| *o.workforce.get(g))
                .sum();
            let employed = *s.workforce.get(g);
            let country_wage = country
                .hourly_wage_usd
                .get(g.index())
                .copied()
                .unwrap_or(0.0);
            wage_cost += employed * hours * country_wage * factor;
            StaffLine {
                group: catalog.labor_groups.key(g).to_owned(),
                needed: needed.get(g.index()).copied().unwrap_or(0.0),
                employed,
                free_in_country: (country
                    .labor_available
                    .get(g.index())
                    .copied()
                    .unwrap_or(0.0)
                    - employed_in_country)
                    .max(0.0),
                wage_usd: country_wage * factor,
                country_wage_usd: country_wage,
            }
        })
        .filter(|l| l.needed > 1e-9 || l.employed > 1e-9)
        .collect();
    let rival_premium_max = state
        .sites
        .iter()
        .filter(|o| o.closed.is_none() && o.country == s.country && o.owner != s.owner)
        .map(|o| o.wage_premium)
        .fold(0.0, f64::max);
    (staff, rival_premium_max, wage_cost)
}

/// A site's result in the last closed month by cost type, and per product.
fn site_result(game: &Game, site: crate::state::SiteId) -> Option<SiteResult> {
    let state = game.state();
    let catalog = game.catalog();
    let owner = state.sites[site.index()].owner;
    let period = state.companies[owner.index()].ledger.months.last()?;
    if period.by_center.keys().all(|c| c.site != Some(site)) {
        return None;
    }
    let lines: Vec<ResultLine> = crate::ledger::CostType::ALL
        .iter()
        .filter(|&&t| t != crate::ledger::CostType::Revenue)
        .map(|&t| (t, period.site_type(site, t)))
        .filter(|(_, m)| *m != Money::ZERO)
        .map(|(t, m)| ResultLine {
            key: t.text_key().to_owned(),
            usd: usd(m),
        })
        .collect();
    let revenue = period.site_type(site, crate::ledger::CostType::Revenue);
    let products = period
        .by_center
        .iter()
        .filter(|(c, _)| c.site == Some(site))
        .filter_map(|(c, by_type)| {
            let product = c.product?;
            let revenue = by_type
                .get(&crate::ledger::CostType::Revenue)
                .copied()
                .unwrap_or(Money::ZERO);
            Some(ProductResult {
                product: catalog.products.key(product).to_owned(),
                revenue_usd: usd(revenue),
                margin_usd: usd(by_type.values().copied().sum()),
            })
        })
        .collect();
    Some(SiteResult {
        month: super::iso(period.start?),
        revenue_usd: usd(revenue),
        result_usd: usd(revenue) + lines.iter().map(|l| l.usd).sum::<f64>(),
        lines,
        products,
    })
}

/// Discovered deposits of a country with a free field, priced as `DevelopDeposit` does.
fn free_deposits(game: &Game, country: crate::ids::CountryId) -> Vec<DepositOption> {
    let state = game.state();
    let catalog = game.catalog();
    catalog
        .deposits
        .iter()
        .filter(|(_, d)| d.country == country)
        .filter(|(_, d)| d.discovered.is_none_or(|y| y <= state.date.year()))
        .filter_map(|(id, d)| {
            let field = state
                .deposits
                .get(id)
                .concessions
                .iter()
                .find(|c| c.site.is_none())?;
            Some(DepositOption {
                key: catalog.deposits.key(id).to_owned(),
                resource: catalog.products.key(d.resource).to_owned(),
                cost_usd: usd(d
                    .development_cost
                    .scale(state.settings.market_scale * field.share)),
                days: d.development_days,
                // What the concession allows this year, at the market scale like the
                // production itself.
                output_per_year: catalog.max_output(id, state.date.year())
                    * state.settings.market_scale
                    * field.share,
            })
        })
        .collect()
}

/// One product on the market of a country.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketLine {
    pub product: String,
    pub price_usd: f64,
    pub reference_usd: f64,
    /// Price of the state market, if it sells the product.
    pub state_price_usd: Option<f64>,
    pub demand_last_month: f64,
    /// Size of the market: demand of the last month at the market price (USD).
    pub market_size_usd: f64,
    pub sold_last_month: f64,
    pub imported_last_month: f64,
    pub exported_last_month: f64,
    pub sellers: u32,
    /// The player's asking price here, if the player offers the product.
    pub own_price_usd: Option<f64>,
    /// Own sales in the last closed month.
    pub own_sold_last_month: f64,
    /// The player really offers the product here: sold in the last or running month, or
    /// goods in stock to sell (an offer left without goods does not count).
    #[serde(default)]
    pub own_active: bool,
    /// Goods group; brands and advertising work per group (M16).
    pub group: String,
    /// Own share of the sales in the country in the last closed month (0–1).
    pub own_share: f64,
    /// Company that sold most here in the last closed month, and its share.
    pub leader: Option<String>,
    pub leader_share: f64,
    /// Average price of last month's sales (for the trend against the price now).
    pub price_last_month_usd: Option<f64>,
    /// Share of consumers' and government demand served last month.
    pub supply: Option<f64>,
    /// Demand last month that found no goods at its price.
    pub unmet_last_month: f64,
    /// Openings for a newcomer (display hints): `mangel`, `teuer`, `wenige_anbieter`.
    pub chances: Vec<String>,
}

/// Supply below this share of consumers' and government demand is a shortage.
const SHORTAGE_SUPPLY: f64 = 0.9;
/// A price this far above the reference price makes room for a cheaper newcomer.
const EXPENSIVE_FACTOR: f64 = 1.2;
/// With demand and at most this many sellers, a market is open to newcomers – if it is
/// not fully served or pays more than the reference price.
const FEW_SELLERS: u32 = 2;

fn chances(line: &MarketLine) -> Vec<String> {
    let mut found = Vec::new();
    let unmet_share = if line.demand_last_month > 1e-9 {
        line.unmet_last_month / line.demand_last_month
    } else {
        0.0
    };
    if line.supply.is_some_and(|s| s < SHORTAGE_SUPPLY) || unmet_share > 1.0 - SHORTAGE_SUPPLY {
        found.push("mangel");
    }
    if line.reference_usd > 0.0 && line.price_usd >= EXPENSIVE_FACTOR * line.reference_usd {
        found.push("teuer");
    }
    let open = line.supply.is_some_and(|s| s < 0.99)
        || unmet_share > 0.01
        || line.price_usd > line.reference_usd;
    if line.demand_last_month > 1e-9 && line.sellers <= FEW_SELLERS && open {
        found.push("wenige_anbieter");
    }
    found.into_iter().map(str::to_owned).collect()
}

/// Brands of a goods group in a country (M16).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BrandLine {
    pub group: String,
    /// Own awareness, 0–1.
    pub own_awareness: f64,
    /// Best known other company selling the group here, and its awareness.
    pub top: Option<String>,
    pub top_awareness: f64,
    /// Own advertising budget per month.
    pub budget_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketView {
    pub date: String,
    pub country: String,
    pub lines: Vec<MarketLine>,
    /// Goods groups that consumers buy here or the player advertises or is known for.
    pub brands: Vec<BrandLine>,
    /// Best advertising medium of the year (text `werbemittel.<key>`).
    pub medium: Option<String>,
    /// Budget per month that reaches the whole country once.
    pub reach_usd: f64,
    /// Unit of each product (key of `einheit.<key>`).
    pub units: BTreeMap<String, String>,
    /// The continents with their countries, for choosing a market.
    #[serde(default)]
    pub regions: Vec<RegionChoice>,
}

/// A continent and its countries, both by key.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegionChoice {
    pub continent: String,
    pub countries: Vec<String>,
}

fn region_choices(catalog: &Catalog) -> Vec<RegionChoice> {
    catalog
        .continents
        .ids()
        .map(|k| RegionChoice {
            continent: catalog.continents.key(k).to_owned(),
            countries: catalog
                .countries
                .iter()
                .filter(|(_, c)| c.continent == k)
                .map(|(id, _)| catalog.countries.key(id).to_owned())
                .collect(),
        })
        .filter(|r| !r.countries.is_empty())
        .collect()
}

/// The market of a country, or of several summed up: `welt` for all, `kontinent:<key>`
/// for a continent's countries. Quantities add up, prices are weighted by demand;
/// brands and advertising only show for a country.
pub fn market(game: &Game, scope: &str) -> Option<MarketView> {
    let catalog = game.catalog();
    if let Some(c) = catalog.countries.id(scope) {
        let mut v = country_market(game, c, scope)?;
        v.regions = region_choices(catalog);
        return Some(v);
    }
    let countries: Vec<CountryId> = if scope == "welt" {
        catalog.countries.ids().collect()
    } else {
        let k = catalog.continents.id(scope.strip_prefix("kontinent:")?)?;
        catalog
            .countries
            .iter()
            .filter(|(_, c)| c.continent == k)
            .map(|(id, _)| id)
            .collect()
    };
    let views: Vec<MarketView> = countries
        .iter()
        .filter_map(|&c| country_market(game, c, catalog.countries.key(c)))
        .collect();
    let first = views.first()?;
    let mut lines: Vec<MarketLine> = Vec::new();
    for (i, base) in first.lines.iter().enumerate() {
        let all: Vec<&MarketLine> = views.iter().filter_map(|v| v.lines.get(i)).collect();
        let sum = |f: &dyn Fn(&MarketLine) -> f64| all.iter().map(|l| f(l)).sum::<f64>();
        let demand = sum(&|l| l.demand_last_month);
        let weight = |l: &MarketLine| {
            if demand > 1e-9 {
                l.demand_last_month
            } else {
                1.0
            }
        };
        let weights = sum(&|l| weight(l)).max(1e-12);
        let mean = |f: &dyn Fn(&MarketLine) -> f64| sum(&|l| f(l) * weight(l)) / weights;
        let sold = sum(&|l| l.sold_last_month);
        let own_sold = sum(&|l| l.own_sold_last_month);
        let own_prices: Vec<f64> = all.iter().filter_map(|l| l.own_price_usd).collect();
        // The leader of the biggest single market stands for the region.
        let leader = all.iter().filter(|l| l.leader.is_some()).max_by(|a, b| {
            (a.leader_share * a.sold_last_month).total_cmp(&(b.leader_share * b.sold_last_month))
        });
        let served = all
            .iter()
            .filter_map(|l| l.supply.map(|s| (s, l.demand_last_month)))
            .fold((0.0, 0.0), |(a, b), (s, d)| (a + s * d, b + d));
        let mut line = MarketLine {
            product: base.product.clone(),
            price_usd: mean(&|l| l.price_usd),
            reference_usd: mean(&|l| l.reference_usd),
            state_price_usd: None,
            demand_last_month: demand,
            market_size_usd: sum(&|l| l.market_size_usd),
            sold_last_month: sold,
            imported_last_month: sum(&|l| l.imported_last_month),
            exported_last_month: sum(&|l| l.exported_last_month),
            sellers: all.iter().map(|l| l.sellers).sum(),
            own_price_usd: (!own_prices.is_empty())
                .then(|| own_prices.iter().sum::<f64>() / own_prices.len() as f64),
            own_sold_last_month: own_sold,
            own_active: all.iter().any(|l| l.own_active),
            group: base.group.clone(),
            own_share: if sold > 1e-9 { own_sold / sold } else { 0.0 },
            leader: leader.and_then(|l| l.leader.clone()),
            leader_share: leader.map_or(0.0, |l| {
                if sold > 1e-9 {
                    l.leader_share * l.sold_last_month / sold
                } else {
                    0.0
                }
            }),
            price_last_month_usd: None,
            supply: (served.1 > 1e-9).then(|| served.0 / served.1),
            unmet_last_month: sum(&|l| l.unmet_last_month),
            chances: Vec::new(),
        };
        line.chances = chances(&line);
        lines.push(line);
    }
    Some(MarketView {
        date: first.date.clone(),
        country: scope.to_owned(),
        lines,
        brands: Vec::new(),
        medium: first.medium.clone(),
        reach_usd: 0.0,
        units: first.units.clone(),
        regions: region_choices(catalog),
    })
}

fn country_market(game: &Game, c: CountryId, country: &str) -> Option<MarketView> {
    let state = game.state();
    let catalog = game.catalog();
    let lines = catalog
        .products
        .ids()
        .filter(|&p| catalog.products.get(p).kind != crate::catalog::ProductKind::Energy)
        // What nobody can make yet has no market (M32).
        .filter(|&p| market::available(state, catalog, p, state.date.year()))
        .map(|p| {
            let m = state.markets.get(p).get(c);
            let t = &m.last_month;
            let mut sellers = 0;
            let mut own_price = None;
            let mut own_sold = 0.0;
            let mut own_active = false;
            let mut by_company: BTreeMap<CompanyId, f64> = BTreeMap::new();
            for s in state.sites.iter().filter(|s| s.country == c) {
                if let Some(o) = s.offers.get(&p) {
                    if state.companies[s.owner.index()].bankrupt {
                        continue;
                    }
                    sellers += 1;
                    *by_company.entry(s.owner).or_default() += o.sold_last_month;
                    if state.is_main(s.owner) {
                        own_price = Some(usd(o.price));
                        own_sold += o.sold_last_month;
                        let stock = s.inventory.get(&p).map_or(0.0, |x| x.quantity) - o.keep;
                        own_active |=
                            o.sold_last_month > 1e-9 || o.sold_month > 1e-9 || stock > 1e-9;
                    }
                }
            }
            let share = |sold: f64| if t.sold > 1e-9 { sold / t.sold } else { 0.0 };
            let leader = by_company
                .iter()
                .filter(|&(_, &sold)| sold > 1e-9)
                .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(a.0)));
            let mut line = MarketLine {
                product: catalog.products.key(p).to_owned(),
                price_usd: usd(market::market_price(catalog, state, c, p)),
                reference_usd: usd(market::local_reference(catalog, state, c, p)),
                state_price_usd: market::state_price(catalog, state, c, p).map(usd),
                demand_last_month: t.demand,
                market_size_usd: t.demand * usd(market::market_price(catalog, state, c, p)),
                sold_last_month: t.sold,
                imported_last_month: t.imported,
                exported_last_month: t.exported,
                sellers,
                own_price_usd: own_price,
                own_sold_last_month: own_sold,
                own_active,
                group: catalog
                    .goods_groups
                    .key(catalog.products.get(p).goods_group)
                    .to_owned(),
                own_share: share(own_sold),
                leader: leader.map(|(&id, _)| state.companies[id.index()].name.clone()),
                leader_share: leader.map_or(0.0, |(_, &sold)| share(sold)),
                price_last_month_usd: (t.sold > 1e-9).then(|| usd(t.revenue) / t.sold),
                supply: (t.outside_demand > 1e-9).then(|| t.outside_sold / t.outside_demand),
                unmet_last_month: t.unmet(),
                chances: Vec::new(),
            };
            line.chances = chances(&line);
            line
        })
        .collect();
    Some(MarketView {
        date: iso(state.date),
        country: country.to_owned(),
        lines,
        brands: brands(game, c),
        medium: catalog
            .market_model
            .brand
            .medium(state.date.year())
            .map(|m| m.key.clone()),
        reach_usd: crate::brand::reach_usd(state, catalog, c),
        units: units(catalog),
        regions: Vec::new(),
    })
}

/// A seller on a product market.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SellerLine {
    pub company: String,
    pub own: bool,
    /// A historical company.
    pub real: bool,
    pub price_usd: f64,
    pub sold_last_month: f64,
    /// Share of the market's sales last month (0–1).
    pub share: f64,
    /// The company's development generation of the product (M37).
    #[serde(default)]
    pub level: u8,
    /// Its research level of the product against the leader (Q1, 0–1).
    #[serde(default)]
    pub research_share: f64,
    /// The company's own name for the product (M42).
    #[serde(default)]
    pub product_name: Option<String>,
    /// Quality of what it offers here: its stock at its selling sites, weighted by
    /// quantity (`None` without stock).
    #[serde(default)]
    pub quality: Option<f64>,
}

/// One product on the market of a country (M18): prices, demand by buyer, sellers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductMarketView {
    pub date: String,
    pub country: String,
    pub product: String,
    pub unit: String,
    pub group: String,
    pub price_usd: f64,
    pub reference_usd: f64,
    pub price_last_month_usd: Option<f64>,
    pub state_price_usd: Option<f64>,
    /// Asking price of the traders' imports, if they hold any.
    pub import_price_usd: Option<f64>,
    /// Consumer demand per month by income fifth (poorest first), at today's rates.
    pub consumers_per_month: [f64; 5],
    /// Government demand per month at today's rate.
    pub state_per_month: f64,
    pub demand_last_month: f64,
    /// Of last month's demand: consumers and government; the rest is companies.
    pub outside_demand_last_month: f64,
    pub sold_last_month: f64,
    pub unmet_last_month: f64,
    pub imported_last_month: f64,
    pub exported_last_month: f64,
    pub supply: Option<f64>,
    /// Companies selling here, by last month's sales.
    pub sellers: Vec<SellerLine>,
    pub own_awareness: f64,
    pub chances: Vec<String>,
    /// The last closed months, oldest first (M24).
    #[serde(default)]
    pub history: Vec<MarketMonthView>,
    /// How the market price comes about (M27).
    #[serde(default)]
    pub price_parts: Option<PriceParts>,
    /// How the consumer demand comes about (M27); `None` without consumer demand.
    #[serde(default)]
    pub demand_parts: Option<DemandParts>,
    /// Whether companies give the product their own names (M42), the player's name and
    /// three free suggestions for it.
    #[serde(default)]
    pub nameable: bool,
    #[serde(default)]
    pub own_name: Option<String>,
    #[serde(default)]
    pub name_suggestions: Vec<String>,
    /// Raw materials: the commodity cycle's swing of the world's output (0.1 = 10 % more).
    #[serde(default)]
    pub supply_cycle: Option<f64>,
}

/// The market price as reference price × price level factor × market situation (M27).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PriceParts {
    /// Reference price of the product at price level 1.
    pub world_reference_usd: f64,
    /// Price level of the country (1: reference country) and the share of it that
    /// carries into this kind of product.
    pub price_level: f64,
    pub level_share: f64,
    /// price level ^ share; reference price × factor = reference price in the country.
    pub level_factor: f64,
    pub reference_usd: f64,
    pub price_usd: f64,
    /// Market price / reference price in the country.
    pub situation: f64,
    /// Last month: supply of consumers and government, unserved share of all demand,
    /// share of the sales from imports, sellers here.
    pub supply: Option<f64>,
    pub unmet_share: f64,
    pub import_share: f64,
    pub sellers: u32,
    /// Ceiling of automatic prices (multiple of the reference price in the country).
    pub price_max_factor: f64,
}

/// The consumer demand of a market from its parts (M27; docs/FORMELN.md, M7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DemandParts {
    /// `verbrauch`, `gebrauch` or `ergaenzung`.
    pub kind: String,
    /// The durable a complement is used with.
    pub complement_of: Option<String>,
    /// Inhabitants the market counts (the market scale applied) and GDP per head.
    pub population: f64,
    pub gdp_per_capita_usd: f64,
    /// Price the households compare with: the reference price at the country's price
    /// level; and today's market price.
    pub reference_usd: f64,
    pub price_usd: f64,
    /// Share of households with electricity (for goods that need it).
    pub grid: Option<f64>,
    /// Seasonal factor of this month.
    pub season: f64,
    pub fifths: Vec<FifthParts>,
}

/// One income fifth (poorest first).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FifthParts {
    /// Income per head and year.
    pub income_usd: f64,
    /// Share that buys at today's price (from income and price).
    pub propensity: f64,
    /// Consumables: units per head and year at saturation; complements: durables owned
    /// per head; durables: ownership per head the fifth aims at.
    pub base: f64,
    /// Durables: units owned per head.
    pub owned: Option<f64>,
    /// Demand per head and year this month.
    pub per_head_year: f64,
}

/// One closed month of a market (M24).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketMonthView {
    /// First day of the month.
    pub month: String,
    /// Average price paid; `None` when nothing was sold.
    pub price_usd: Option<f64>,
    pub sold: f64,
    /// The player's share of the units sold; `None` while the player never sold here.
    pub own_share: Option<f64>,
}

fn price_parts(game: &Game, country: CountryId, product: crate::ids::ProductId) -> PriceParts {
    let state = game.state();
    let catalog = game.catalog();
    let p = catalog.products.get(product);
    let level = state.countries.get(country).price_level;
    let factor = market::price_factor(catalog, state, country, product);
    let reference = market::local_reference(catalog, state, country, product).to_usd();
    let price = market::market_price(catalog, state, country, product).to_usd();
    let m = state.markets.get(product).get(country);
    let t = &m.last_month;
    let sellers = state
        .sites
        .iter()
        .filter(|s| s.country == country && s.offers.contains_key(&product))
        .filter(|s| !state.companies[s.owner.index()].bankrupt)
        .count();
    PriceParts {
        world_reference_usd: usd(p.reference_price),
        price_level: level,
        level_share: catalog.market_model.price_level_share[p.kind.index()],
        level_factor: factor,
        reference_usd: reference,
        price_usd: price,
        situation: if reference > 0.0 {
            price / reference
        } else {
            1.0
        },
        supply: (t.outside_demand > 1e-9).then(|| t.outside_sold / t.outside_demand),
        unmet_share: if t.demand > 1e-9 {
            t.unmet() / t.demand
        } else {
            0.0
        },
        import_share: if t.sold > 1e-9 {
            t.imported / t.sold
        } else {
            0.0
        },
        sellers: u32::try_from(sellers).unwrap_or(u32::MAX),
        price_max_factor: catalog.market_model.price_max_factor,
    }
}

/// The parts of the consumer demand, from the same formulas as `market::update_demand`.
fn demand_parts(
    game: &Game,
    country: CountryId,
    product: crate::ids::ProductId,
) -> Option<DemandParts> {
    use crate::catalog::ConsumptionType;
    let state = game.state();
    let catalog = game.catalog();
    let p = catalog.products.get(product);
    let d = p.consumer_demand.as_ref()?;
    let cs = state.countries.get(country);
    let m = state.markets.get(product).get(country);
    let reference = p.reference_price.to_usd() * cs.price_level;
    let price = m.price.to_usd();
    let per_layer = cs.market_population / 5.0;
    let month = usize::try_from(state.date.month() - 1).unwrap_or(0);
    let season = d.seasonality.map_or(1.0, |s| s[month]);
    let grid = d.needs_grid.then_some(cs.grid_share);
    let (kind, complement_of) = match d.consumption {
        ConsumptionType::Consumable { .. } => ("verbrauch", None),
        ConsumptionType::Durable { .. } => ("gebrauch", None),
        ConsumptionType::Complement { of, .. } => {
            ("ergaenzung", Some(catalog.products.key(of).to_owned()))
        }
    };
    let displaced = market::displaced(state, country, &market::successors(catalog, product));
    let fifths = (0..5)
        .map(|q| {
            let income = cs.income_quintiles_usd[q];
            let share = market::propensity(d, income, price, reference);
            let (base, owned) = match d.consumption {
                ConsumptionType::Consumable {
                    per_capita_per_year,
                } => (per_capita_per_year, None),
                ConsumptionType::Durable { max_ownership, .. } => {
                    let target = max_ownership * share * displaced[q] * grid.unwrap_or(1.0);
                    (target, Some(m.ownership[q]))
                }
                ConsumptionType::Complement { of, .. } => {
                    (state.markets.get(of).get(country).ownership[q], None)
                }
            };
            FifthParts {
                income_usd: income,
                propensity: share,
                base,
                owned,
                per_head_year: if per_layer > 0.0 {
                    m.consumer_rate[q] * 365.0 / per_layer
                } else {
                    0.0
                },
            }
        })
        .collect();
    Some(DemandParts {
        kind: kind.to_owned(),
        complement_of,
        population: cs.market_population,
        gdp_per_capita_usd: cs.gdp_per_capita_usd,
        reference_usd: reference,
        price_usd: price,
        grid,
        season,
        fifths,
    })
}

/// The series of a market with the first day of each month.
fn market_history(state: &crate::state::GameState, m: &Market) -> Vec<MarketMonthView> {
    let h = &m.history;
    let mut month = state.date.first_of_month();
    let mut months = Vec::with_capacity(h.price.len());
    for _ in 0..h.price.len() {
        month = month.add_days(-1).first_of_month();
        months.push(month);
    }
    months.reverse();
    months
        .into_iter()
        .enumerate()
        .map(|(i, month)| {
            let sold = f64::from(h.sold[i]);
            MarketMonthView {
                month: iso(month),
                price_usd: (h.price[i] > Money::ZERO).then(|| usd(h.price[i])),
                sold,
                own_share: h.own.get(i).map(|&own| {
                    if sold > 0.0 {
                        f64::from(own) / sold
                    } else {
                        0.0
                    }
                }),
            }
        })
        .collect()
}

/// Days of a month for demand per month at daily rates.
const DAYS_PER_MONTH: f64 = 30.0;

pub fn product_market(game: &Game, country: &str, product: &str) -> Option<ProductMarketView> {
    let state = game.state();
    let catalog = game.catalog();
    let c = catalog.countries.id(country)?;
    let p = catalog.products.id(product)?;
    let line = market(game, country)?
        .lines
        .into_iter()
        .find(|l| l.product == product);
    let m = state.markets.get(p).get(c);
    let t = &m.last_month;
    let mut sellers: Vec<SellerLine> = Vec::new();
    // Quality × quantity and quantity of the stock, by seller; without stock (sold out)
    // the quality its facilities make now.
    let mut stocks: Vec<(f64, f64)> = Vec::new();
    let mut made: Vec<(f64, f64)> = Vec::new();
    for (index, s) in state.sites.iter().enumerate() {
        if s.country != c {
            continue;
        }
        let Some(o) = s.offers.get(&p) else { continue };
        let company = &state.companies[s.owner.index()];
        if company.bankrupt {
            continue;
        }
        let stock = s
            .inventory
            .get(&p)
            .filter(|st| st.quantity > 1e-9)
            .map_or((0.0, 0.0), |st| (st.quality * st.quantity, st.quantity));
        let site_id = crate::state::SiteId(u32::try_from(index).unwrap_or(u32::MAX));
        let making = s
            .slots
            .iter()
            .enumerate()
            .filter_map(|(i, sl)| {
                let r = catalog.recipes.get(sl.recipe?);
                (r.product == p).then(|| slot_quality(game, site_id, i, r).total)
            })
            .fold((0.0, 0.0), |(sum, n), q| (sum + q, n + 1.0));
        match sellers.iter().position(|x| x.company == company.name) {
            Some(i) => {
                let x = &mut sellers[i];
                x.sold_last_month += o.sold_last_month;
                x.price_usd = x.price_usd.min(usd(o.price));
                stocks[i].0 += stock.0;
                stocks[i].1 += stock.1;
                made[i].0 += making.0;
                made[i].1 += making.1;
            }
            None => sellers.push(SellerLine {
                company: company.name.clone(),
                own: state.is_main(s.owner),
                real: company.ai.as_ref().is_some_and(|a| a.real.is_some()),
                price_usd: usd(o.price),
                sold_last_month: o.sold_last_month,
                share: 0.0,
                level: crate::development::level(catalog, state, s.owner, p),
                research_share: crate::development::share(state, s.owner, p),
                product_name: company.product_names.get(&p).cloned(),
                quality: None,
            }),
        }
        if stocks.len() < sellers.len() {
            stocks.push(stock);
            made.push(making);
        }
    }
    for ((x, &(sum, quantity)), &(made_sum, n)) in sellers.iter_mut().zip(&stocks).zip(&made) {
        x.quality = Some(if quantity > 1e-9 {
            sum / quantity
        } else if n > 0.0 {
            made_sum / n
        } else {
            // A trading site, sold out and making nothing: the base quality.
            50.0
        });
    }
    for x in &mut sellers {
        x.share = if t.sold > 1e-9 {
            x.sold_last_month / t.sold
        } else {
            0.0
        };
    }
    sellers.sort_by(|a, b| {
        b.sold_last_month
            .total_cmp(&a.sold_last_month)
            .then(a.price_usd.total_cmp(&b.price_usd))
            .then(a.company.cmp(&b.company))
    });
    let group = catalog.products.get(p).goods_group;
    Some(ProductMarketView {
        date: iso(state.date),
        country: country.to_owned(),
        product: product.to_owned(),
        unit: catalog.units.key(catalog.products.get(p).unit).to_owned(),
        group: catalog.goods_groups.key(group).to_owned(),
        price_usd: usd(market::market_price(catalog, state, c, p)),
        reference_usd: usd(market::local_reference(catalog, state, c, p)),
        price_last_month_usd: (t.sold > 1e-9).then(|| usd(t.revenue) / t.sold),
        state_price_usd: market::state_price(catalog, state, c, p).map(usd),
        import_price_usd: (m.imports.quantity > 1e-9).then(|| usd(m.import_price)),
        consumers_per_month: m.consumer_rate.map(|r| r * DAYS_PER_MONTH),
        state_per_month: m.state_rate * DAYS_PER_MONTH,
        demand_last_month: t.demand,
        outside_demand_last_month: t.outside_demand,
        sold_last_month: t.sold,
        unmet_last_month: t.unmet(),
        imported_last_month: t.imported,
        exported_last_month: t.exported,
        supply: (t.outside_demand > 1e-9).then(|| t.outside_sold / t.outside_demand),
        sellers,
        own_awareness: state.companies[game.player().index()].awareness(c, group),
        chances: line.map(|l| l.chances).unwrap_or_default(),
        supply_cycle: state
            .commodity_cycle
            .get(&p)
            .map(|_| crate::production::cycle_factor(state, p) - 1.0),
        history: market_history(state, m),
        price_parts: Some(price_parts(game, c, p)),
        demand_parts: demand_parts(game, c, p),
        nameable: catalog.product_naming.style(catalog, p).is_some(),
        own_name: state.companies[game.player().index()]
            .product_names
            .get(&p)
            .cloned(),
        name_suggestions: {
            // A named product gets its successor model first (B1).
            let successor = crate::product_names::successor(catalog, state, game.player(), p);
            let count = if successor.is_some() { 2 } else { 3 };
            successor
                .into_iter()
                .chain(crate::product_names::suggestions(
                    catalog,
                    state,
                    game.player(),
                    p,
                    count,
                ))
                .collect()
        },
    })
}

/// A product on the market of one country, for the world map.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldMarketLine {
    pub country: String,
    pub price_usd: f64,
    pub reference_usd: f64,
    pub demand_last_month: f64,
    pub unmet_last_month: f64,
    pub supply: Option<f64>,
    pub sellers: u32,
    pub own_sellers: u32,
}

/// One product in every country with demand or sellers (map layer "Absatzchancen").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldMarketView {
    pub date: String,
    pub product: String,
    pub unit: String,
    pub countries: Vec<WorldMarketLine>,
}

pub fn world_market(game: &Game, product: &str) -> Option<WorldMarketView> {
    let state = game.state();
    let catalog = game.catalog();
    let p = catalog.products.id(product)?;
    let mut sellers: BTreeMap<CountryId, (u32, u32)> = BTreeMap::new();
    for s in &state.sites {
        if s.offers.contains_key(&p) && !state.companies[s.owner.index()].bankrupt {
            let e = sellers.entry(s.country).or_default();
            e.0 += 1;
            if state.is_main(s.owner) {
                e.1 += 1;
            }
        }
    }
    let countries = catalog
        .countries
        .ids()
        .filter_map(|c| {
            let t = &state.markets.get(p).get(c).last_month;
            let (count, own) = sellers.get(&c).copied().unwrap_or_default();
            (t.demand > 1e-9 || count > 0).then(|| WorldMarketLine {
                country: catalog.countries.key(c).to_owned(),
                price_usd: usd(market::market_price(catalog, state, c, p)),
                reference_usd: usd(market::local_reference(catalog, state, c, p)),
                demand_last_month: t.demand,
                unmet_last_month: t.unmet(),
                supply: (t.outside_demand > 1e-9).then(|| t.outside_sold / t.outside_demand),
                sellers: count,
                own_sellers: own,
            })
        })
        .collect();
    Some(WorldMarketView {
        date: iso(state.date),
        product: product.to_owned(),
        unit: catalog.units.key(catalog.products.get(p).unit).to_owned(),
        countries,
    })
}

/// Awareness and advertising per goods group in a country: groups with consumer demand
/// here and groups the player advertises or is known for.
fn brands(game: &Game, country: CountryId) -> Vec<BrandLine> {
    let state = game.state();
    let catalog = game.catalog();
    let player = &state.companies[game.player().index()];
    let mut groups: BTreeSet<GoodsGroupId> = catalog
        .products
        .iter()
        .filter(|&(p, _)| {
            state
                .markets
                .get(p)
                .get(country)
                .consumer_rate
                .iter()
                .sum::<f64>()
                > 0.0
        })
        .map(|(_, product)| product.goods_group)
        .collect();
    groups.extend(
        player
            .brands
            .iter()
            .filter(|b| b.country == country)
            .map(|b| b.group),
    );
    groups.extend(
        player
            .advertising
            .iter()
            .filter(|a| a.country == country)
            .map(|a| a.group),
    );
    groups
        .into_iter()
        .map(|group| {
            let top = state
                .sites
                .iter()
                .filter(|s| s.country == country && !state.is_main(s.owner))
                .filter(|s| !state.companies[s.owner.index()].bankrupt)
                .filter(|s| {
                    s.offers
                        .keys()
                        .any(|&p| catalog.products.get(p).goods_group == group)
                })
                .map(|s| {
                    let company = &state.companies[s.owner.index()];
                    (company.awareness(country, group), s.owner)
                })
                .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
            BrandLine {
                group: catalog.goods_groups.key(group).to_owned(),
                own_awareness: player.awareness(country, group),
                top: top.map(|(_, id)| state.companies[id.index()].name.clone()),
                top_awareness: top.map_or(0.0, |(a, _)| a),
                budget_usd: player
                    .advertising
                    .iter()
                    .find(|a| a.country == country && a.group == group)
                    .map_or(0.0, |a| usd(a.budget)),
            }
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TechnologyView {
    pub key: String,
    pub field: String,
    pub invention_year: i32,
    pub prerequisites: Vec<String>,
    pub known: bool,
    pub researchable: bool,
    /// Points needed today (`None` for technologies known from the start).
    pub needed: Option<f64>,
    /// Factor against the historical effort (above 1 ahead of history).
    pub factor: Option<f64>,
    pub points: f64,
    /// Sites working on it.
    pub sites: Vec<u32>,
    /// Facilities and recipes it opens (text keys).
    pub opens: Vec<String>,
    /// `bekannt`, `in_arbeit`, `erforschbar` or `gesperrt` (prerequisites missing).
    pub status: String,
    /// Technologies that need this one.
    pub leads_to: Vec<String>,
    /// Points still missing today.
    pub remaining: Option<f64>,
    /// Points per day of the own research centers working on it now.
    pub points_per_day: f64,
    /// Days to go at that pace.
    pub days: Option<f64>,
    /// What one fully used laboratory in the company's home country would take.
    pub one_lab: Option<LabEstimate>,
    /// Facilities it opens, with what they cost and make.
    pub facilities: Vec<FacilityUnlock>,
    /// Recipes it opens and the facility each needs.
    pub recipes: Vec<RecipeUnlock>,
    /// Products the opened recipes make.
    pub products: Vec<String>,
    /// Patent or patent claim on it (P7).
    #[serde(default)]
    pub patent: Option<PatentView>,
}

/// A patent or an open claim on a technology (P7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatentView {
    pub holder: String,
    /// Held by the player's company.
    pub own: bool,
    /// Last day to file further countries (own claims only).
    pub deadline: Option<String>,
    pub until: Option<String>,
    /// ISO codes of the countries filed.
    pub countries: Vec<String>,
    /// The player may use it everywhere (holder, licensee or prior user).
    pub free_for_player: bool,
    /// Player's countries with sites where it is blocked.
    pub blocks_player_in: Vec<String>,
}

/// A country to file a patent in, with its cost (P7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatentCountryView {
    pub country: String,
    pub cost_usd: f64,
    /// Among the player's site countries or the largest markets.
    pub suggested: bool,
}

/// Time and cost of a technology with one fully used laboratory (M19, estimate).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LabEstimate {
    pub country: String,
    pub researchers: f64,
    pub points_per_day: f64,
    pub days: f64,
    /// Researchers' wages and material, without building the laboratory.
    pub cost_usd: f64,
}

/// A facility a technology opens.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FacilityUnlock {
    pub key: String,
    pub site_type: SiteType,
    pub kind_text: String,
    pub investment_usd: f64,
    pub build_days: u32,
    /// Recipes that run on it.
    pub recipes: Vec<String>,
}

/// A recipe a technology opens.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecipeUnlock {
    pub key: String,
    pub facility: String,
    /// Technology the facility needs, if the player does not know it yet.
    pub facility_missing: Option<String>,
    pub product: String,
    /// Output and inputs of one facility per day at full utilization.
    pub output_per_day: f64,
    pub inputs_per_day: Vec<(String, f64)>,
}

/// A product the player may develop or has developed (Q1, before M37).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DevelopmentView {
    pub product: String,
    /// The player makes it in a facility of its own.
    pub own: bool,
    /// Generation reached (point thresholds for successor models and start-ups).
    pub level: u8,
    pub own_level: u8,
    /// Highest generation of another company.
    pub best_rival: u8,
    /// All points of the player and of the leader; the player's level (0–1 of the
    /// leader's points), the best rival's, and how far the product is developed (0–1).
    #[serde(default)]
    pub total_points: f64,
    #[serde(default)]
    pub leader_points: f64,
    #[serde(default)]
    pub share: f64,
    #[serde(default)]
    pub best_rival_share: f64,
    #[serde(default)]
    pub maturity: f64,
    /// The next level, the points it takes today and the points collected.
    pub next: Option<u8>,
    pub needed: Option<f64>,
    pub points: f64,
    /// Points per day of the own research centers working on it, and days to go.
    pub points_per_day: f64,
    pub days: Option<f64>,
    /// What one fully used laboratory would take for the next level.
    pub one_lab: Option<LabEstimate>,
    /// Field of its researchers.
    pub field: String,
    /// What the level changes now, and at the lead (100 %) with today's maturity.
    pub effect: EffectView,
    pub lead_effect: EffectView,
    /// Research centers developing it, with their shares of researchers.
    pub sites: Vec<u32>,
    /// Manufacturing research (Q6): own points, the player's share of the leader's,
    /// how far the manufacturing is researched at all (0–1), and the centers on it.
    #[serde(default)]
    pub process_points: f64,
    #[serde(default)]
    pub process_share: f64,
    #[serde(default)]
    pub process_maturity: f64,
    #[serde(default)]
    pub process_sites: Vec<u32>,
}

/// A product to research (Q6): research works on products, the technologies stay behind.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductResearchView {
    pub product: String,
    /// `rohstoff`, `halbzeug`, `komponente`, `endprodukt`, `energie`.
    pub kind: String,
    pub branch: String,
    /// `bekannt`, `in_arbeit`, `erforschbar` or `gesperrt`.
    pub status: String,
    /// The product exists in the world (it can be bought).
    pub in_world: bool,
    /// Points it still takes in all, and those collected for it.
    pub needed: f64,
    pub collected: f64,
    /// Inputs that do not exist in the world yet (they block the research).
    pub missing_inputs: Vec<String>,
    /// Inputs of its main recipe.
    pub inputs: Vec<String>,
    /// Latest historical year of its missing technologies.
    pub historical_year: Option<i32>,
    /// Field of the researchers it needs next.
    pub field: Option<String>,
    /// Research centers working on it, their points per day and the days to go.
    pub sites: Vec<u32>,
    pub points_per_day: f64,
    pub days: Option<f64>,
    /// What one fully used laboratory would take.
    pub one_lab: Option<LabEstimate>,
}

/// What a development level changes (M37): quality points, shares of labor and inputs
/// saved per run.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct EffectView {
    pub quality: f64,
    pub labor_saved: f64,
    pub inputs_saved: f64,
    /// Share of output gained per run (manufacturing research, Q6).
    #[serde(default)]
    pub output_gained: f64,
}

impl From<crate::development::Effect> for EffectView {
    fn from(e: crate::development::Effect) -> Self {
        Self {
            quality: e.quality,
            labor_saved: 1.0 - e.labor,
            inputs_saved: 1.0 - e.inputs,
            output_gained: e.output - 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResearchCenterView {
    pub site: u32,
    /// The owner's name for the site.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub country: String,
    pub researchers: f64,
    pub project: Option<String>,
    /// Product it develops with all researchers (M37).
    pub development: Option<String>,
    /// Shares of the researchers per developed product (Q1); the rest works on `project`.
    #[serde(default)]
    pub shares: Vec<(String, f64)>,
    /// Shares of the researchers on manufacturing research per product (Q6).
    #[serde(default)]
    pub process_shares: Vec<(String, f64)>,
    /// Product it researches (Q6); `project` is the technology it works on for it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub research_product: Option<String>,
    /// A laboratory is ready.
    pub ready: bool,
    /// Day the first laboratory is ready while all are still being built.
    pub building_until: Option<String>,
    pub labs: Vec<LabView>,
}

/// A laboratory slot of a research center; its utilization sets the researcher posts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LabView {
    pub slot: usize,
    pub count: u32,
    pub utilization: f64,
    pub ready: String,
    /// The laboratory's own project (LB1); none: it works on the center's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work: Option<LabWorkView>,
    /// Share of the center's researchers working in this laboratory.
    #[serde(default)]
    pub share: f64,
}

/// The own project of a laboratory (LB1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LabWorkView {
    /// `technologie`, `produkt` (researches a product) or `entwicklung`.
    pub kind: String,
    /// Key of the technology or product.
    pub key: String,
    /// The technology a product research works on now.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResearchOverview {
    pub date: String,
    pub technologies: Vec<TechnologyView>,
    /// All products with their research state (Q6).
    #[serde(default)]
    pub products: Vec<ProductResearchView>,
    /// Products to develop (M37), the highest level and what one level brings.
    pub developments: Vec<DevelopmentView>,
    pub development_levels: u8,
    /// What the lead brings at full maturity (Q1).
    pub development_max: EffectView,
    pub centers: Vec<ResearchCenterView>,
    /// Kind of every product (`rohstoff`, `halbzeug`, `komponente`, `endprodukt`,
    /// `energie`), to tell inputs from end products.
    #[serde(default)]
    pub product_kinds: BTreeMap<String, String>,
    /// Facility for research centers (to build a laboratory).
    pub laboratory: Option<String>,
    /// Its price and researcher posts.
    pub laboratory_usd: f64,
    pub laboratory_posts: f64,
    /// Unit of each product (key of `einheit.<key>`).
    pub units: BTreeMap<String, String>,
    /// Countries to file patents in, largest markets first (P7; empty without patents).
    #[serde(default)]
    pub patent_countries: Vec<PatentCountryView>,
}

fn lab_work_view(catalog: &Catalog, work: crate::state::LabWork) -> LabWorkView {
    use crate::state::LabWork;
    match work {
        LabWork::Technology(t) => LabWorkView {
            kind: "technologie".into(),
            key: catalog.technologies.key(t).to_owned(),
            technology: None,
        },
        LabWork::Product {
            product,
            technology,
        } => LabWorkView {
            kind: "produkt".into(),
            key: catalog.products.key(product).to_owned(),
            technology: technology.map(|t| catalog.technologies.key(t).to_owned()),
        },
        LabWork::Development(p) => LabWorkView {
            kind: "entwicklung".into(),
            key: catalog.products.key(p).to_owned(),
            technology: None,
        },
    }
}

/// The patent on a technology as the player sees it (P7).
fn patent_view(game: &Game, id: crate::ids::TechnologyId) -> Option<PatentView> {
    let state = game.state();
    let catalog = game.catalog();
    let model = catalog.research_model.patents.as_ref()?;
    let p = state.patents.get(id).as_ref()?;
    let player = game.player();
    let own = p.holder == player;
    let filed = crate::patents::in_force(state, catalog, id).is_some();
    if !own && !filed {
        return None;
    }
    let blocks: Vec<String> = state
        .sites
        .iter()
        .filter(|s| s.held_by(player))
        .map(|s| s.country)
        .filter(|&c| crate::patents::blocks(state, catalog, player, id, c))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|c| catalog.countries.key(c).to_owned())
        .collect();
    Some(PatentView {
        holder: state
            .company(p.holder)
            .map_or_else(String::new, |c| c.name.clone()),
        own,
        deadline: (own && state.date <= p.deadline(model)).then(|| iso(p.deadline(model))),
        until: p.until(model).map(iso),
        countries: p
            .countries
            .iter()
            .map(|&c| catalog.countries.key(c).to_owned())
            .collect(),
        free_for_player: own || p.licensees.contains(&player),
        blocks_player_in: blocks,
    })
}

fn patent_countries(game: &Game) -> Vec<PatentCountryView> {
    let state = game.state();
    let catalog = game.catalog();
    if catalog.research_model.patents.is_none() {
        return Vec::new();
    }
    let (_, suggested) = crate::patents::ai_countries(state, catalog, game.player());
    let mut list: Vec<(f64, CountryId)> = catalog
        .countries
        .ids()
        .map(|c| {
            let v = state.countries.get(c);
            (v.population * v.gdp_per_capita_usd, c)
        })
        .collect();
    list.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    list.into_iter()
        .map(|(_, c)| PatentCountryView {
            country: catalog.countries.key(c).to_owned(),
            cost_usd: usd(crate::patents::filing_cost(state, catalog, c)),
            suggested: suggested.contains(&c),
        })
        .collect()
}

/// One fully used laboratory in a country working on a technology: points, days and
/// cost (researchers' wages and material) for the points still missing.
fn lab_estimate(
    catalog: &Catalog,
    state: &crate::state::GameState,
    country: CountryId,
    field: crate::ids::SpecializationId,
    remaining: f64,
) -> Option<LabEstimate> {
    let lab = catalog
        .facilities
        .values()
        .find(|f| f.site_type == SiteType::ResearchCenter)?;
    let group = catalog
        .research_model
        .researchers
        .get(field.index())
        .copied()
        .flatten()?;
    let c = state.countries.get(country);
    let researchers = lab.runs_per_day;
    let points_per_day = researchers
        * c.research_efficiency
            .get(field.index())
            .copied()
            .unwrap_or(1.0);
    if points_per_day <= 0.0 {
        return None;
    }
    let hours = crate::production::hours_per_worker_day(catalog, state.date);
    let wage = c.hourly_wage_usd.get(group.index()).copied().unwrap_or(0.0);
    let per_day =
        researchers * (wage * hours + catalog.research_model.material_usd_per_day * c.price_level);
    let days = remaining / points_per_day;
    Some(LabEstimate {
        country: catalog.countries.key(country).to_owned(),
        researchers,
        points_per_day,
        days,
        cost_usd: days * per_day,
    })
}

/// Products the player may develop or has developed, with points, level against the
/// leader, the effort of the next generation and what the level brings (Q1).
fn developments(game: &Game) -> Vec<DevelopmentView> {
    use crate::development;
    let state = game.state();
    let catalog = game.catalog();
    let player = game.player();
    let company = &state.companies[player.index()];
    let lab_country = state
        .sites
        .iter()
        .find(|s| s.held_by(player) && s.kind == SiteType::ResearchCenter)
        .map_or(company.headquarters, |s| s.country);
    let made: BTreeSet<ProductId> = state
        .sites
        .iter()
        .filter(|s| s.held_by(player))
        .flat_map(|s| &s.slots)
        .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r).product))
        .collect();
    catalog
        .products
        .ids()
        .filter_map(|p| {
            let (field, _) = development::basis(catalog, p)?;
            let total = development::points(state, player, p);
            if total <= 0.0 && !development::can_develop(catalog, state, player, p) {
                return None;
            }
            let level = development::level(catalog, state, player, p);
            let next = development::next_effort(catalog, state, player, p, state.date);
            let points = company.development.points.get(&p).copied().unwrap_or(0.0);
            let remaining = next.map(|(_, needed)| (needed - points).max(0.0));
            let working: Vec<(usize, f64)> = state
                .sites
                .iter()
                .enumerate()
                .filter(|(_, s)| s.held_by(player))
                .filter_map(|(i, s)| {
                    development::site_shares(s)
                        .into_iter()
                        .find(|&(q, _)| q == p)
                        .map(|(_, share)| (i, share))
                })
                .collect();
            let points_per_day: f64 = working
                .iter()
                .map(|&(i, share)| {
                    let s = &state.sites[i];
                    let researchers = catalog
                        .research_model
                        .researchers
                        .get(field.index())
                        .copied()
                        .flatten()
                        .map_or(0.0, |g| *s.workforce.get(g));
                    researchers
                        * share
                        * state
                            .countries
                            .get(s.country)
                            .research_efficiency
                            .get(field.index())
                            .copied()
                            .unwrap_or(1.0)
                })
                .sum();
            let rivals = || {
                state
                    .companies
                    .iter()
                    .enumerate()
                    .filter(|&(i, c)| i != player.index() && !c.bankrupt)
                    .map(|(i, _)| crate::state::CompanyId(u32::try_from(i).unwrap_or(u32::MAX)))
            };
            let best_rival = rivals()
                .map(|c| development::level(catalog, state, c, p))
                .max()
                .unwrap_or(0);
            let best_rival_share = rivals()
                .map(|c| development::share(state, c, p))
                .fold(0.0, f64::max);
            let maturity = development::maturity(catalog, state, p);
            Some(DevelopmentView {
                product: catalog.products.key(p).to_owned(),
                // Made now, or learned, licensed or taken over (FW1): researched products
                // belong to the player before the first works makes them.
                own: made.contains(&p) || company.known_products.contains(&p),
                level,
                own_level: level,
                best_rival,
                total_points: total,
                leader_points: development::leader_points(state, p),
                share: development::share(state, player, p),
                best_rival_share,
                maturity,
                next: next.map(|(n, _)| n),
                needed: next.map(|(_, needed)| needed),
                points,
                points_per_day,
                days: remaining
                    .filter(|_| points_per_day > 0.0)
                    .map(|r| r / points_per_day),
                one_lab: remaining
                    .and_then(|r| lab_estimate(catalog, state, lab_country, field, r)),
                field: catalog.specializations.key(field).to_owned(),
                effect: development::effect(catalog, state, player, p).into(),
                lead_effect: development::Effect::of(catalog, 1.0, maturity)
                    .with_process(
                        catalog,
                        1.0,
                        development::process_maturity(catalog, state, p),
                    )
                    .into(),
                sites: working
                    .iter()
                    .map(|&(i, _)| u32::try_from(i).unwrap_or(u32::MAX))
                    .collect(),
                process_points: development::process_points(state, player, p),
                process_share: development::process_share(state, player, p),
                process_maturity: development::process_maturity(catalog, state, p),
                process_sites: state
                    .sites
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| {
                        s.held_by(player)
                            && development::site_process_shares(s)
                                .iter()
                                .any(|&(q, share)| q == p && share > 0.0)
                    })
                    .map(|(i, _)| u32::try_from(i).unwrap_or(u32::MAX))
                    .collect(),
            })
        })
        .collect()
}

/// Research state of every product for the player (Q6).
fn product_research_views(game: &Game) -> Vec<ProductResearchView> {
    use crate::product_research as pr;
    let state = game.state();
    let catalog = game.catalog();
    let player = game.player();
    let company = &state.companies[player.index()];
    let lab_country = state
        .sites
        .iter()
        .find(|s| s.held_by(player) && s.kind == SiteType::ResearchCenter)
        .map_or(company.headquarters, |s| s.country);
    pr::all(catalog)
        .filter_map(|p| {
            let recipe = pr::main_recipe(catalog, p)?;
            let x = catalog.products.get(p);
            let known = pr::known(catalog, state, player, p);
            let missing = if known {
                Vec::new()
            } else {
                pr::missing(catalog, state, player, p)
            };
            let (needed, collected) = if known {
                (0.0, 0.0)
            } else {
                pr::effort(catalog, state, player, p, state.date)
            };
            let working: Vec<usize> = state
                .sites
                .iter()
                .enumerate()
                .filter(|(_, s)| s.held_by(player) && s.research_product == Some(p))
                .map(|(i, _)| i)
                .collect();
            let next = (!known)
                .then(|| pr::next(catalog, state, player, p))
                .flatten()
                .or_else(|| missing.first().copied());
            let field = next.map(|t| catalog.technologies.get(t).field);
            let points_per_day: f64 = field.map_or(0.0, |field| {
                working
                    .iter()
                    .map(|&i| {
                        let s = &state.sites[i];
                        let researchers = catalog
                            .research_model
                            .researchers
                            .get(field.index())
                            .copied()
                            .flatten()
                            .map_or(0.0, |g| *s.workforce.get(g));
                        researchers
                            * crate::development::technology_share(s)
                            * state
                                .countries
                                .get(s.country)
                                .research_efficiency
                                .get(field.index())
                                .copied()
                                .unwrap_or(1.0)
                    })
                    .sum()
            });
            let remaining = (!known).then(|| (needed - collected).max(0.0));
            let status = if known {
                "bekannt"
            } else if !working.is_empty() {
                "in_arbeit"
            } else if pr::can_research(catalog, state, player, p) {
                "erforschbar"
            } else {
                "gesperrt"
            };
            let key = |q: ProductId| catalog.products.key(q).to_owned();
            Some(ProductResearchView {
                product: key(p),
                kind: super::chains::kind_key(x.kind).to_owned(),
                branch: catalog.branches.key(x.branch).to_owned(),
                status: status.to_owned(),
                in_world: pr::known_in_world(catalog, state, p),
                needed,
                collected,
                missing_inputs: if known {
                    Vec::new()
                } else {
                    pr::missing_inputs(catalog, state, p)
                        .into_iter()
                        .map(key)
                        .collect()
                },
                inputs: catalog
                    .recipes
                    .get(recipe)
                    .inputs
                    .iter()
                    .map(|&(q, _)| key(q))
                    .collect(),
                historical_year: missing
                    .iter()
                    .map(|&t| catalog.technologies.get(t).invention_year)
                    .max(),
                field: field.map(|f| catalog.specializations.key(f).to_owned()),
                sites: working
                    .iter()
                    .map(|&i| u32::try_from(i).unwrap_or(u32::MAX))
                    .collect(),
                points_per_day,
                days: remaining
                    .filter(|_| points_per_day > 0.0)
                    .map(|r| r / points_per_day),
                one_lab: remaining.zip(field).and_then(|(r, f)| {
                    (r > 0.0)
                        .then(|| lab_estimate(catalog, state, lab_country, f, r))
                        .flatten()
                }),
            })
        })
        .collect()
}

pub fn research_overview(game: &Game) -> ResearchOverview {
    let state = game.state();
    let catalog = game.catalog();
    let player = game.player();
    let company = &state.companies[player.index()];
    let technologies = catalog
        .technologies
        .iter()
        .map(|(id, t)| {
            let effort = research::effort(catalog, state, id, state.date);
            let mut opens: Vec<String> = catalog
                .facilities
                .iter()
                .filter(|(_, f)| f.technology == Some(id))
                .map(|(f, _)| format!("anlage.{}", catalog.facilities.key(f)))
                .collect();
            opens.extend(
                catalog
                    .recipes
                    .iter()
                    .filter(|(_, r)| r.technology == Some(id))
                    .map(|(r, _)| format!("rezept.{}", catalog.recipes.key(r))),
            );
            let known = state.knows(catalog, player, id);
            let researchable = research::can_research(catalog, state, player, id);
            let points = company.research.get(&id).copied().unwrap_or(0.0);
            let remaining = effort
                .map(|e| (e.points - points).max(0.0))
                .filter(|_| !known);
            let working: Vec<usize> = state
                .sites
                .iter()
                .enumerate()
                .filter(|(_, s)| s.held_by(player) && s.research == Some(id))
                .map(|(i, _)| i)
                .collect();
            let points_per_day: f64 = working
                .iter()
                .map(|&i| {
                    let s = &state.sites[i];
                    let researchers = catalog
                        .research_model
                        .researchers
                        .get(t.field.index())
                        .copied()
                        .flatten()
                        .map_or(0.0, |g| *s.workforce.get(g));
                    researchers
                        * state
                            .countries
                            .get(s.country)
                            .research_efficiency
                            .get(t.field.index())
                            .copied()
                            .unwrap_or(1.0)
                })
                .sum();
            let status = if known {
                "bekannt"
            } else if !working.is_empty() {
                "in_arbeit"
            } else if researchable {
                "erforschbar"
            } else {
                "gesperrt"
            };
            let lab_country = state
                .sites
                .iter()
                .find(|s| s.held_by(player) && s.kind == SiteType::ResearchCenter)
                .map_or(company.headquarters, |s| s.country);
            let knows = |t: Option<crate::ids::TechnologyId>| {
                t.is_none_or(|t| state.knows(catalog, player, t))
            };
            let facilities: Vec<FacilityUnlock> = catalog
                .facilities
                .iter()
                .filter(|(_, f)| f.technology == Some(id))
                .map(|(f, x)| FacilityUnlock {
                    key: catalog.facilities.key(f).to_owned(),
                    site_type: x.site_type,
                    kind_text: site_type_key(x.site_type),
                    investment_usd: usd(x.investment),
                    build_days: x.build_days,
                    recipes: catalog
                        .recipes
                        .iter()
                        .filter(|(_, r)| r.facility == f)
                        .map(|(r, _)| catalog.recipes.key(r).to_owned())
                        .collect(),
                })
                .collect();
            // Recipes it opens: those that need it, and those on its facilities.
            let recipes: Vec<RecipeUnlock> = catalog
                .recipes
                .iter()
                .filter(|(_, r)| {
                    r.technology == Some(id)
                        || catalog.facilities.get(r.facility).technology == Some(id)
                })
                .map(|(r, x)| {
                    let f = catalog.facilities.get(x.facility);
                    RecipeUnlock {
                        key: catalog.recipes.key(r).to_owned(),
                        facility: catalog.facilities.key(x.facility).to_owned(),
                        facility_missing: f
                            .technology
                            .filter(|&ft| ft != id && !knows(Some(ft)))
                            .map(|ft| catalog.technologies.key(ft).to_owned()),
                        product: catalog.products.key(x.product).to_owned(),
                        output_per_day: x.output * f.runs_per_day,
                        inputs_per_day: x
                            .inputs
                            .iter()
                            .map(|&(p, q)| (catalog.products.key(p).to_owned(), q * f.runs_per_day))
                            .collect(),
                    }
                })
                .collect();
            let mut products: Vec<String> = Vec::new();
            for r in &recipes {
                if !products.contains(&r.product) {
                    products.push(r.product.clone());
                }
            }
            TechnologyView {
                status: status.to_owned(),
                leads_to: catalog
                    .technologies
                    .iter()
                    .filter(|(_, other)| other.prerequisites.contains(&id))
                    .map(|(o, _)| catalog.technologies.key(o).to_owned())
                    .collect(),
                remaining,
                points_per_day,
                days: remaining
                    .filter(|_| points_per_day > 0.0)
                    .map(|r| r / points_per_day),
                one_lab: remaining
                    .and_then(|r| lab_estimate(catalog, state, lab_country, t.field, r)),
                facilities,
                recipes,
                products,
                patent: patent_view(game, id),
                key: catalog.technologies.key(id).to_owned(),
                field: catalog.specializations.key(t.field).to_owned(),
                invention_year: t.invention_year,
                prerequisites: t
                    .prerequisites
                    .iter()
                    .map(|&p| catalog.technologies.key(p).to_owned())
                    .collect(),
                known,
                researchable,
                needed: effort.map(|e| e.points),
                factor: effort.map(|e| e.factor),
                points,
                sites: working
                    .iter()
                    .map(|&i| u32::try_from(i).unwrap_or(u32::MAX))
                    .collect(),
                opens,
            }
        })
        .collect();
    let centers = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.held_by(player) && s.kind == SiteType::ResearchCenter)
        .map(|(i, s)| ResearchCenterView {
            site: u32::try_from(i).unwrap_or(u32::MAX),
            name: s.name.clone(),
            country: catalog.countries.key(s.country).to_owned(),
            researchers: s.workforce.values().sum(),
            project: s.research.map(|t| catalog.technologies.key(t).to_owned()),
            development: s.development.map(|p| catalog.products.key(p).to_owned()),
            shares: crate::development::site_shares(s)
                .into_iter()
                .map(|(p, share)| (catalog.products.key(p).to_owned(), share))
                .collect(),
            process_shares: crate::development::site_process_shares(s)
                .into_iter()
                .map(|(p, share)| (catalog.products.key(p).to_owned(), share))
                .collect(),
            research_product: s
                .research_product
                .map(|p| catalog.products.key(p).to_owned()),
            ready: s.slots.iter().any(|sl| sl.operating(state.date)),
            building_until: s
                .slots
                .iter()
                .map(|sl| sl.ready)
                .min()
                .filter(|&r| r > state.date)
                .map(iso),
            labs: s
                .slots
                .iter()
                .enumerate()
                .map(|(slot, sl)| LabView {
                    slot,
                    count: sl.count,
                    utilization: sl.utilization,
                    ready: iso(sl.ready),
                    work: sl.work.map(|w| lab_work_view(catalog, w)),
                    share: crate::labs::lab_share(catalog, s, slot, state.date),
                })
                .collect(),
        })
        .collect();
    ResearchOverview {
        date: iso(state.date),
        technologies,
        products: product_research_views(game),
        developments: developments(game),
        development_levels: catalog.research_model.development.levels,
        development_max: crate::development::Effect::of(catalog, 1.0, 1.0)
            .with_process(catalog, 1.0, 1.0)
            .into(),
        centers,
        product_kinds: catalog
            .products
            .iter()
            .map(|(id, p)| {
                (
                    catalog.products.key(id).to_owned(),
                    super::chains::kind_key(p.kind).to_owned(),
                )
            })
            .collect(),
        laboratory: catalog
            .facilities
            .iter()
            .find(|(_, f)| f.site_type == SiteType::ResearchCenter)
            .map(|(f, _)| catalog.facilities.key(f).to_owned()),
        laboratory_usd: catalog
            .facilities
            .values()
            .find(|f| f.site_type == SiteType::ResearchCenter)
            .map_or(0.0, |f| usd(f.investment)),
        laboratory_posts: catalog
            .facilities
            .values()
            .find(|f| f.site_type == SiteType::ResearchCenter)
            .map_or(0.0, |f| f.runs_per_day),
        units: units(catalog),
        patent_countries: patent_countries(game),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoanView {
    pub index: usize,
    pub principal_usd: f64,
    pub balance_usd: f64,
    pub rate: f64,
    pub start: String,
    pub months: u32,
    pub instalment_usd: f64,
}

/// A bond of the player's company (K2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BondView {
    pub index: usize,
    pub principal_usd: f64,
    pub coupon: f64,
    pub issued: String,
    pub maturity: String,
    /// Grade at issue (`bonitaet.<key>`).
    pub grade: String,
    /// What buying it back now costs.
    pub redeem_usd: f64,
}

/// What a bond of an amount would get.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BondQuoteView {
    pub amount_usd: f64,
    pub grade: String,
    pub coupon: f64,
}

/// Bonds of the player's company: its standing, what new bonds would cost, its bonds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BondsView {
    /// Without bonds in the data the rest is empty.
    pub enabled: bool,
    /// Grade without a new bond; `None`: no investor would buy.
    pub grade: Option<String>,
    pub debt_ratio: f64,
    /// EBIT over interest per year; `None` without a closed month or interest.
    pub coverage: Option<f64>,
    pub has_figures: bool,
    pub equity_usd: f64,
    pub equity_min_usd: f64,
    pub volume_min_usd: f64,
    pub term_min_years: u32,
    pub term_max_years: u32,
    pub cost_share: f64,
    pub redeem_premium: f64,
    /// The largest bond that sells today (0: none).
    pub max_usd: f64,
    /// Amounts up to the largest with grade and coupon.
    pub quotes: Vec<BondQuoteView>,
    pub bonds: Vec<BondView>,
}

/// Shares of the largest bond offered as amounts.
const BOND_STEPS: [f64; 4] = [0.25, 0.5, 0.75, 1.0];

fn bonds_view(game: &Game) -> BondsView {
    let (state, catalog) = (game.state(), game.catalog());
    let m = &catalog.bonds;
    let me = game.player();
    let company = &state.companies[me.index()];
    let cut = crate::central::premium_cut(catalog, state, me);
    let s = crate::bonds::standing(catalog, company);
    let now = crate::bonds::grade_for(catalog, company, Money::ZERO, state.date, cut);
    let equity = crate::ranking::equity(company);
    let max = if m.enabled && equity >= m.equity_min {
        crate::bonds::max_amount(catalog, company, state.date, cut)
    } else {
        Money::ZERO
    };
    let mut quotes: Vec<BondQuoteView> = Vec::new();
    if max > Money::ZERO {
        for share in BOND_STEPS {
            let amount = max.scale(share).max(m.volume_min);
            if quotes.last().is_some_and(|q| q.amount_usd >= usd(amount)) {
                continue;
            }
            if let Some((g, coupon)) =
                crate::bonds::grade_for(catalog, company, amount, state.date, cut)
            {
                quotes.push(BondQuoteView {
                    amount_usd: usd(amount),
                    grade: m.grades[g].key.clone(),
                    coupon,
                });
            }
        }
    }
    BondsView {
        enabled: m.enabled,
        grade: now.map(|(g, _)| m.grades[g].key.clone()),
        debt_ratio: s.debt_ratio(Money::ZERO).unwrap_or(0.0),
        coverage: s.coverage(Money::ZERO, 0.0).filter(|c| c.is_finite()),
        has_figures: s.ebit.is_some(),
        equity_usd: usd(equity),
        equity_min_usd: usd(m.equity_min),
        volume_min_usd: usd(m.volume_min),
        term_min_years: m.term_min_years,
        term_max_years: m.term_max_years,
        cost_share: m.cost_share,
        redeem_premium: m.redeem_premium,
        max_usd: usd(max),
        quotes,
        bonds: company
            .bonds
            .iter()
            .enumerate()
            .map(|(index, b)| BondView {
                index,
                principal_usd: usd(b.principal),
                coupon: b.coupon,
                issued: iso(b.issued),
                maturity: iso(b.maturity),
                grade: b.grade.clone(),
                redeem_usd: usd(crate::bonds::redeem_price(catalog, b)),
            })
            .collect(),
    }
}

/// Income statement lines by cost type (text key, amount; costs negative).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Statement {
    pub lines: Vec<(String, f64)>,
    pub result_usd: f64,
    /// Cash flow from operations, investments and financing.
    pub cash_flow_usd: [f64; 3],
}

fn statement(period: &crate::ledger::PeriodResult) -> Statement {
    let s = reports::income_statement(period);
    // Every cost type, so that the periods line up in one table.
    Statement {
        lines: crate::ledger::CostType::ALL
            .iter()
            .map(|t| {
                let amount = s
                    .lines
                    .iter()
                    .find(|(x, _)| x == t)
                    .map_or(Money::ZERO, |l| l.1);
                (t.text_key().to_owned(), usd(amount))
            })
            .collect(),
        result_usd: usd(s.result),
        cash_flow_usd: [
            usd(s.cash_flow.operating),
            usd(s.cash_flow.investing),
            usd(s.cash_flow.financing),
        ],
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FinanceView {
    pub date: String,
    /// Balance sheet: (text key of the account, amount).
    pub assets: Vec<(String, f64)>,
    pub claims: Vec<(String, f64)>,
    pub total_usd: f64,
    pub year: Statement,
    pub last_year: Option<Statement>,
    pub last_month: Option<Statement>,
    pub loans: Vec<LoanView>,
    /// Bonds (K2).
    pub bonds: BondsView,
    pub credit_limit_usd: f64,
    pub overdraft_limit_usd: f64,
    /// Interest rate a loan of 10 % of the credit limit would get today.
    pub loan_rate: f64,
    pub max_term_years: u32,
    pub loss_carryforward_usd: f64,
    /// Closed months, oldest first (M18).
    pub history: Vec<super::MonthView>,
    /// Where the money was made: by site and product, last month and this year.
    pub centers_last_month: Option<CenterResults>,
    pub centers_year: Option<CenterResults>,
}

/// Revenue and result of a site in a period.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteLine {
    pub site: u32,
    pub kind_text: String,
    /// The player's own name for the site, if any.
    #[serde(default)]
    pub site_name: Option<String>,
    pub country: String,
    pub revenue_usd: f64,
    pub result_usd: f64,
}

/// Results of a period by site and by product (all sites), and what belongs to the
/// company as a whole (interest, taxes, advertising).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CenterResults {
    pub sites: Vec<SiteLine>,
    /// Revenue and gross margin (revenue less the production cost of what was sold).
    pub products: Vec<ProductResult>,
    pub company_usd: f64,
}

fn center_results(game: &Game, period: &crate::ledger::PeriodResult) -> Option<CenterResults> {
    use crate::ledger::CostType;
    if period.by_center.is_empty() {
        return None;
    }
    let state = game.state();
    let catalog = game.catalog();
    let sum = |filter: &dyn Fn(&crate::ledger::CostCenter) -> bool, revenue_only: bool| {
        period
            .by_center
            .iter()
            .filter(|(c, _)| filter(c))
            .flat_map(|(_, t)| t.iter())
            .filter(|(t, _)| !revenue_only || **t == CostType::Revenue)
            .map(|(_, &m)| m)
            .sum::<Money>()
    };
    let sites = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| state.is_main(s.owner))
        .map(|(i, s)| {
            let id = crate::state::SiteId(u32::try_from(i).unwrap_or(u32::MAX));
            SiteLine {
                site: id.0,
                kind_text: site_type_key(s.kind),
                site_name: s.name.clone(),
                country: catalog.countries.key(s.country).to_owned(),
                revenue_usd: usd(sum(&|c| c.site == Some(id), true)),
                result_usd: usd(sum(&|c| c.site == Some(id), false)),
            }
        })
        .collect();
    let products: BTreeSet<crate::ids::ProductId> =
        period.by_center.keys().filter_map(|c| c.product).collect();
    Some(CenterResults {
        sites,
        products: products
            .into_iter()
            .map(|p| ProductResult {
                product: catalog.products.key(p).to_owned(),
                revenue_usd: usd(sum(&|c| c.product == Some(p), true)),
                margin_usd: usd(sum(&|c| c.product == Some(p), false)),
            })
            .collect(),
        company_usd: usd(sum(&|c| c.site.is_none(), false)),
    })
}

pub fn finance_overview(game: &Game) -> FinanceView {
    let state = game.state();
    let catalog = game.catalog();
    let company = &state.companies[game.player().index()];
    let ledger = &company.ledger;
    let sheet = reports::balance_sheet(ledger);
    let limit = finance::credit_limit(catalog, company);
    FinanceView {
        date: iso(state.date),
        assets: sheet
            .assets
            .iter()
            .map(|(a, m)| (a.text_key().to_owned(), usd(*m)))
            .collect(),
        claims: sheet
            .claims
            .iter()
            .map(|(a, m)| (a.text_key().to_owned(), usd(*m)))
            .collect(),
        total_usd: usd(sheet.total),
        year: statement(&ledger.year),
        last_year: ledger.years.last().map(statement),
        last_month: ledger.months.last().map(statement),
        loans: company
            .loans
            .iter()
            .enumerate()
            .map(|(index, l)| LoanView {
                index,
                principal_usd: usd(l.principal),
                balance_usd: usd(l.balance),
                rate: l.rate,
                start: iso(l.start),
                months: l.months,
                instalment_usd: usd(l.instalment),
            })
            .collect(),
        bonds: bonds_view(game),
        credit_limit_usd: usd(limit),
        overdraft_limit_usd: usd(finance::overdraft_limit(catalog, ledger)),
        loan_rate: finance::loan_rate(
            catalog,
            company,
            limit.scale(0.1),
            state.date,
            crate::central::premium_cut(catalog, state, game.player()),
        ),
        max_term_years: catalog.finance_model.max_term_years,
        history: super::history(ledger),
        centers_last_month: ledger.months.last().and_then(|m| center_results(game, m)),
        centers_year: center_results(game, &ledger.year),
        loss_carryforward_usd: usd(company.loss_carryforward),
    }
}

/// Runs per day at the planned utilization (shut down facilities none).
fn planned_runs_per_day(catalog: &Catalog, sl: &crate::state::Slot) -> f64 {
    if sl.mothballed() {
        return 0.0;
    }
    sl.full_runs(catalog) * sl.utilization
}

/// Quality of the next batch of a slot with the inputs the site holds now.
fn slot_quality(
    game: &Game,
    site: crate::state::SiteId,
    slot: usize,
    recipe: &crate::catalog::Recipe,
) -> QualityView {
    let (state, catalog) = (game.state(), game.catalog());
    let s = &state.sites[site.index()];
    let (mut sum, mut weight) = (0.0, 0.0);
    for &(p, q) in &recipe.inputs {
        if let Some(st) = s.inventory.get(&p).filter(|st| st.quantity > 1e-9) {
            sum += st.quality * q;
            weight += q;
        }
    }
    let input_quality = (weight > 1e-12).then(|| sum / weight);
    let q = crate::production::quality_parts(
        catalog,
        state,
        (site, slot),
        recipe,
        input_quality.unwrap_or(50.0),
    );
    QualityView {
        input_quality,
        base: q.base,
        inputs: q.inputs,
        automation: q.automation,
        condition: q.condition,
        research: q.research,
        training: q.training,
        trim: q.trim,
        total: q.total,
    }
}

/// Products a company may buy units for at a works (FW1): with mandatory research the
/// products it knows in the works' branch, else in any branch.
fn buy_products(
    catalog: &Catalog,
    state: &crate::state::GameState,
    company: crate::state::CompanyId,
    branch: Option<crate::ids::BranchId>,
) -> Vec<String> {
    if !state.settings.research_mandatory {
        return Vec::new();
    }
    catalog
        .products
        .ids()
        .filter(|&p| {
            branch.is_none_or(|b| catalog.products.get(p).branch == b)
                && crate::product_research::main_recipe(catalog, p).is_some()
                && state.knows_product(catalog, company, p)
        })
        .map(|p| catalog.products.key(p).to_owned())
        .collect()
}

#[cfg(test)]
mod explain_tests {
    use std::sync::Arc;

    use super::*;
    use crate::catalog::test_support;
    use crate::state::{GameSettings, StartForm};

    fn game() -> Game {
        let catalog = Arc::new(test_support::production());
        let settings = GameSettings {
            seed: 3,
            start_year: 1900,
            start_country: catalog.countries.id("AAA").unwrap(),
            start_capital: Money::from_usd(1_000_000.0).unwrap(),
            start_form: StartForm::Workshop,
            company_name: "Test AG".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: Default::default(),
            ventures: 1.0,
            tariff_dynamics: 1.0,
            event_effects: true,
            research_mandatory: false,
            found_at_start: true,
            person: Default::default(),
        };
        Game::new(catalog, settings).unwrap()
    }

    /// Twelve months shown; the comparison takes the month a year before the last.
    #[test]
    fn the_price_trend_compares_with_a_year_before() {
        let game = game();
        let mut m = Market::default();
        // 14 closed months at 10, 11, … USD; the second last month sold nothing.
        for i in 0..14 {
            let price = if i == 12 { 0.0 } else { 10.0 + f64::from(i) };
            m.history.price.push(Money::from_usd(price).unwrap());
            m.history.sold.push(5.0);
        }
        let v = price_trend(game.state(), &m);
        assert_eq!(v.months.len(), 12);
        assert_eq!(v.months[10].price_usd, None);
        assert_eq!(v.year_ago_usd, Some(11.0));
        assert!((v.change.unwrap() - (23.0 / 11.0 - 1.0)).abs() < 1e-9);
        assert_eq!((v.low_usd, v.high_usd), (Some(12.0), Some(23.0)));
        // Without a year of history there is nothing to compare with.
        m.history.price.truncate(5);
        m.history.sold.truncate(5);
        let short = price_trend(game.state(), &m);
        assert_eq!(
            (short.months.len(), short.year_ago_usd, short.change),
            (5, None, None)
        );
    }

    /// M32: goods that nobody can make yet do not appear in the market.
    #[test]
    fn the_market_lists_only_goods_that_can_be_made() {
        let mut catalog = test_support::production();
        let later = catalog.technologies.id("hochofen_2000").unwrap();
        let wheel = catalog.products.id("rad").unwrap();
        let smelting = catalog.recipes.id("eisen_schmelzen").unwrap();
        let mut building = catalog.recipes.get(smelting).clone();
        building.product = wheel;
        building.technology = Some(later);
        catalog.recipes.insert("rad_bauen", building).unwrap();
        let settings = game().state().settings.clone();
        let game = Game::new(Arc::new(catalog), settings).unwrap();
        let view = market(&game, "AAA").unwrap();
        let listed: Vec<&str> = view.lines.iter().map(|l| l.product.as_str()).collect();
        assert!(listed.contains(&"brot"), "{listed:?}");
        assert!(!listed.contains(&"rad"), "{listed:?}");
    }

    #[test]
    fn the_parts_of_the_price_make_the_price() {
        let game = game();
        for (country, product) in [("AAA", "brot"), ("BBB", "rad"), ("BBB", "eisen")] {
            let m = product_market(&game, country, product).unwrap();
            let p = m.price_parts.unwrap();
            assert!(
                (p.world_reference_usd * p.level_factor - p.reference_usd).abs() < 1e-3,
                "{product}"
            );
            assert!(
                (p.reference_usd * p.situation - p.price_usd).abs() < 1e-3,
                "{product}"
            );
            assert!((libm::pow(p.price_level, p.level_share) - p.level_factor).abs() < 1e-12);
            assert_eq!(p.price_usd, m.price_usd);
        }
    }

    #[test]
    fn the_parts_of_the_demand_make_the_demand() {
        // At the start the demand was set with today's prices: the parts give it exactly.
        let game = game();
        let bread = product_market(&game, "AAA", "brot").unwrap();
        let d = bread.demand_parts.unwrap();
        assert_eq!(d.kind, "verbrauch");
        assert_eq!(d.fifths.len(), 5);
        for (q, f) in d.fifths.iter().enumerate() {
            let expected = f.base * f.propensity * d.grid.unwrap_or(1.0) * d.season;
            assert!((f.per_head_year - expected).abs() < 1e-9, "fifth {q}");
            // Demand per month as the market view shows it.
            let monthly = f.per_head_year * d.population / 5.0 / 365.0 * 30.0;
            assert!((monthly - bread.consumers_per_month[q]).abs() < 1e-6);
        }
        // Richer fifths buy more readily.
        assert!(d.fifths[4].propensity > d.fifths[0].propensity);

        // A durable shows its ownership and the ownership the fifth aims at.
        let wheel = product_market(&game, "AAA", "rad").unwrap();
        let d = wheel.demand_parts.unwrap();
        assert_eq!(d.kind, "gebrauch");
        assert!(
            d.fifths
                .iter()
                .all(|f| f.owned.is_some() && f.base <= 0.5 + 1e-12)
        );
        assert!(
            product_market(&game, "AAA", "eisen")
                .unwrap()
                .demand_parts
                .is_none()
        );
    }
}
