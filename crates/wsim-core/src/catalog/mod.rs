//! The catalog: immutable game content compiled from the data files.
//!
//! The catalog never changes during a game. Everything that changes lives in the
//! game state. Field semantics are documented in the data format (`docs/DATENFORMAT.md`).

mod table;
#[cfg(test)]
pub(crate) mod test_support;

pub use table::Table;

use serde::{Deserialize, Serialize};

use crate::ids::{Id, IdKind, KeyTable};

use crate::ids::{
    BranchId, ContinentId, CountryId, DepositId, FacilityId, GoodsGroupId, LaborGroupId,
    MilestoneId, ProductId, QualificationId, RecipeId, SpecializationId, TechnologyId,
    TransportClassId, UnitId, VehicleId,
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
    /// Goals for the player after the introduction, in display order (M23).
    pub milestones: Table<MilestoneId, Milestone>,
    pub country_model: CountryModel,
    pub production_model: ProductionModel,
    pub finance_model: FinanceModel,
    pub market_model: MarketModel,
    pub transport_model: TransportModel,
    pub research_model: ResearchModel,
    pub ai_model: AiModel,
    /// Offers between companies for sites and licences (M30).
    pub deal_model: DealModel,
    /// Plots of land for the sites (M35); without size classes there are none.
    pub plot_model: PlotModel,
    /// Historical companies of the start population and later foundings.
    pub real_companies: Vec<RealCompany>,
    /// Historical events, sorted by date (Lastenheft §4.1), with their effects (H1).
    pub events: Vec<HistoricalEvent>,
    /// Parts for the names of generated companies.
    pub name_groups: Vec<NameGroup>,
    /// Parts for the names companies give their end products (M42).
    pub product_naming: ProductNaming,
    /// Currencies of the countries, for display only (M21).
    pub currencies: crate::currency::CurrencyModel,
    /// Positions, managers and their market (MA1); without functions there are none.
    pub management: ManagementModel,
    /// Headquarters and central departments (ZA1–ZA3).
    pub central: CentralModel,
    /// Start-ups (SU1–SU3).
    pub ventures: VentureModel,
    /// Import tariffs, trade zones and embargoes (W3); without a series there are none.
    pub tariffs: TariffModel,
    /// Supply contracts between companies (W4); without the section there are none.
    pub contracts: ContractModel,
    /// Fleets, state transport and freight risk (W5); without the section there are none.
    pub logistics: LogisticsModel,
    /// Subsidiaries (W6); without the section the player cannot found any.
    pub subsidiaries: SubsidiaryModel,
    /// Stock market (K1); without the section no company is listed.
    pub stock: StockModel,
    /// Corporate bonds (K2); without the section no company issues any.
    pub bonds: BondModel,
    /// The player's banks (K4); without the section there are none.
    pub bank: BankModel,
    /// Parameters of the events' effects (H1).
    pub event_model: EventModel,
    /// Age, retirement and death of the managers (PE1).
    pub life: LifeModel,
    /// The player as a person (PE2).
    pub person: PersonModel,
    /// Emissions, retrofitting, CO2 price, antitrust, image (H2); without the section
    /// none of it applies.
    pub environment: EnvironmentModel,
    /// Regulations by country and date, sorted by date (H2).
    pub regulations: Vec<Regulation>,
}

/// Environment and regulation (H2).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnvironmentModel {
    /// Levels of retrofitting, the first first.
    pub retrofit: Vec<RetrofitLevel>,
    /// CO2 price in USD per t by country and year.
    pub co2_price: CountrySeries,
    /// Highest combined share of a market a takeover may reach where antitrust applies.
    pub antitrust_share_max: f64,
    /// How strongly the pollution intensity bends the gain of brand awareness.
    pub image_weight: f64,
    pub provenance: Provenance,
}

/// A level of retrofitting: available from a year, cuts pollutants by a share, costs a
/// share of the unit's investment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RetrofitLevel {
    pub from_year: i32,
    pub reduction: f64,
    pub cost_share: f64,
}

/// A regulation of countries from a day on (H2).
#[derive(Clone, Debug, PartialEq)]
pub struct Regulation {
    pub key: String,
    pub date: crate::calendar::Date,
    pub countries: Vec<CountryId>,
    pub kind: RegulationKind,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RegulationKind {
    /// Units with pollutants need at least this retrofit level after so many months.
    Retrofit { level: u32, months: u32 },
    /// Hourly wages rise by this share.
    Safety { wage_surcharge: f64 },
    /// The products may not be made and/or sold.
    Ban {
        products: Vec<ProductId>,
        production: bool,
        sales: bool,
    },
    /// Takeovers are checked for market shares.
    Antitrust,
}

/// The management of companies (MA1, docs/MANAGER.md).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ManagementModel {
    pub functions: Vec<ManagementFunction>,
    /// Site, country, continent, board (in this order).
    pub levels: Vec<ManagementLevel>,
    /// Specialist functions per site type (indices into `functions`).
    pub specialists: Vec<(SiteType, Vec<usize>)>,
    /// Topics of the running routine: positions take the rules' option (MA2).
    pub routine_topics: Vec<crate::decision::Topic>,
    /// Topics whose effect counts only costs (loans, advertising): positions take the
    /// rules' option, but concerns about them are important (MA5).
    pub rule_topics: Vec<crate::decision::Topic>,
    /// Least budget in yearly salaries of the manager: per decision, per year.
    pub budget_floor: (f64, f64),
    pub concerns: ConcernModel,
    pub strategy: StrategyModel,
    pub mandate: MandateModel,
    pub market: ManagerMarketModel,
    /// Less expertise for a head doing the work of a missing specialist.
    pub head_discount: f64,
    /// Chance that a position without any diligence notices a situation (0–1).
    pub notice_base: f64,
    /// Labor group whose wage the salaries follow.
    pub salary_group: Option<LaborGroupId>,
    pub severance_months: f64,
    pub pool: ManagerPoolModel,
    pub skills: SkillModel,
    pub provenance: Provenance,
}

impl ManagementModel {
    pub fn enabled(&self) -> bool {
        !self.functions.is_empty() && !self.levels.is_empty()
    }

    /// Specialist functions of a site type.
    pub fn specialists_of(&self, kind: SiteType) -> &[usize] {
        self.specialists
            .iter()
            .find(|(k, _)| *k == kind)
            .map_or(&[], |(_, f)| f.as_slice())
    }

    pub fn function(&self, key: &str) -> Option<usize> {
        self.functions.iter().position(|f| f.key == key)
    }
}

/// A function of a company (production, purchasing …) and the topics of decisions it
/// takes care of.
#[derive(Clone, Debug, PartialEq)]
pub struct ManagementFunction {
    pub key: String,
    pub topics: Vec<crate::decision::Topic>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ManagementLevel {
    pub key: String,
    /// Days between the checks of a position.
    pub check_days: u32,
    /// Salary factors on the yearly wage of the salary group.
    pub salary_specialist: f64,
    pub salary_head: f64,
    /// Budget as shares of the unit's revenue: per decision, per year (MA2).
    pub budget_specialist: (f64, f64),
    pub budget_head: (f64, f64),
    /// Specialist functions of a country or continent (indices into `functions`, MA3);
    /// sites have theirs per site type.
    pub specialists: Vec<usize>,
    /// Topics the positions of the level take up themselves (MA3).
    pub topics: Vec<crate::decision::Topic>,
}

/// Concerns of positions (MA2).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConcernModel {
    pub deadline_days: u32,
    /// Rest of a topic at a position after the player declined it.
    pub block_days: u32,
    pub open_per_position: u32,
    /// The position reports the effect of an executed option after these days.
    pub followup_days: u32,
    /// Largest relative error of a forecast without expertise.
    pub estimate_error: f64,
    /// Chance that a recommendation without any judgment is the best option.
    pub recommend_base: f64,
    /// From this many alike concerns of different sites on a day the position above
    /// asks once (MA3).
    pub bundle_from: u32,
}

/// Strategies the player sets for the managers (MA4, docs/FORMELN.md).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StrategyModel {
    /// Price floor on the full unit cost and markup an offer starts at on the market
    /// price: premium and fighting price.
    pub premium: (f64, f64),
    pub fight: (f64, f64),
    /// Largest settings: margin on the full cost, stock days, reserve in months.
    pub min_margin_max: f64,
    pub stock_days_max: f64,
    pub reserve_months_max: f64,
}

/// Mandate and strategy review (MA5, docs/FORMELN.md).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MandateModel {
    /// Aggressiveness of the rules per guideline: growth, profit, safety, leadership.
    pub guidelines: [f64; 4],
    pub proposals_max: u32,
    /// Chances and risks in a report, each.
    pub chances_risks: u32,
    pub reviews_kept: u32,
    /// How much a personnel board member with full expertise sharpens the impression.
    pub personnel_sharpness: f64,
}

/// Headquarters and central departments (ZA1–ZA3, docs/FORMELN.md).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CentralModel {
    pub headquarters: HeadquartersModel,
    /// The central departments, in the order of the data (ZA2).
    pub departments: Vec<Department>,
    /// How much a fully covered department narrows its head's errors (ZA2).
    pub accuracy: f64,
    pub refinance: RefinanceModel,
    pub hit_rate: HitRateModel,
    pub ai: CentralAiModel,
    /// The city of the headquarters (W2); `None`: no limit by academics, offices at the
    /// price of the data.
    pub city: Option<HqCityModel>,
    pub provenance: Provenance,
}

/// How the city of the headquarters limits and prices the central departments (W2).
#[derive(Clone, Debug, PartialEq)]
pub struct HqCityModel {
    /// Academics of a country in a city: concentration × the city's share of the
    /// population, at most all.
    pub academics_concentration: f64,
    /// Share of a city's academics open to the central departments of all companies.
    pub hq_share: f64,
    /// Inhabitants at which the offices cost what the data says.
    pub office_reference_population: f64,
    /// Elasticity of the office costs to the inhabitants of the city.
    pub office_elasticity: f64,
    /// Share of the cost of a move within the country (a different city).
    pub move_within_country: f64,
    /// The year whose population the inhabitants of the cities belong to.
    pub population_year: i32,
}

/// How AI companies set up central departments and move their headquarters (ZA4).
/// The default sets up none and never moves.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CentralAiModel {
    /// Share of the revenue of twelve months the departments may cost a year, by
    /// competence.
    pub revenue_share: Span,
    /// The departments in the order they are set up, each from its least workload.
    pub order: Vec<(DepartmentKind, f64)>,
    /// Least share of the revenue a country must bring to become the seat.
    pub seat_revenue_share: f64,
    /// Least GDP per capita of the new seat as a share of today's.
    pub seat_gdp_share: f64,
    /// Years in which the savings must pay for a move.
    pub payback_years: f64,
    /// Years the seat stays after a move.
    pub lock_years: u32,
}

impl CentralModel {
    pub fn department(&self, kind: DepartmentKind) -> Option<&Department> {
        self.departments.iter().find(|d| d.kind == kind)
    }
}

/// What a central department does; its work is part of the core (ZA2, ZA3).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum DepartmentKind {
    Strategy,
    Finance,
    Personnel,
    Legal,
    Marketing,
}

impl DepartmentKind {
    pub const ALL: [DepartmentKind; 5] = [
        DepartmentKind::Strategy,
        DepartmentKind::Finance,
        DepartmentKind::Personnel,
        DepartmentKind::Legal,
        DepartmentKind::Marketing,
    ];

    /// Key of the data and the texts (`abteilung.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            DepartmentKind::Strategy => "strategie",
            DepartmentKind::Finance => "finanzen",
            DepartmentKind::Personnel => "personal",
            DepartmentKind::Legal => "recht",
            DepartmentKind::Marketing => "marketing",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.key() == key)
    }
}

/// A central department (ZA2).
#[derive(Clone, Debug, PartialEq)]
pub struct Department {
    pub kind: DepartmentKind,
    /// The function (index into the management's functions) whose member of the board
    /// heads it.
    pub function: usize,
    /// The group of its employees, paid in the country of the headquarters.
    pub labor_group: LaborGroupId,
    /// Office costs per employee and year.
    pub office: Money,
    /// Cases an employee works on in a month.
    pub cases: f64,
    /// Its effect at full quality and coverage.
    pub effect: f64,
}

/// Loans at a lower rate (ZA3).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RefinanceModel {
    /// Least advantage of the rate (absolute).
    pub min_advantage: f64,
    /// Fee as a share of the outstanding amount.
    pub fee: f64,
}

/// The hit rate of heads of departments (ZA3).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HitRateModel {
    /// Months after which a recommendation is judged.
    pub months: u32,
    pub mean: f64,
    /// Weight of the mean in judged cases.
    pub prior: f64,
    /// Exponent of the salary demand.
    pub k: f64,
}

/// Start-ups (SU1–SU3, docs/FORMELN.md). Without phases or with none a year there are
/// none.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VentureModel {
    /// What the game calls them from a year on (`startup.bezeichnung.<key>`), in order.
    pub labels: Vec<(String, i32)>,
    pub per_year: f64,
    /// Choices of the new game: key (`startup.haeufigkeit.<key>`) and factor on
    /// `per_year`, and the default one.
    pub frequencies: Vec<(String, f64)>,
    pub default_frequency: usize,
    /// Share working on a new technology.
    pub new_share: f64,
    pub lead_years_max: u32,
    pub phases: Vec<VenturePhase>,
    /// GDP per capita at which the capital of the phases applies.
    pub reference_gdp_usd: f64,
    /// Bounds of the income factor on the capital.
    pub capital_factor: (f64, f64),
    /// Per year ahead of history: more capital, less chance.
    pub lead_capital: f64,
    pub lead_chance: f64,
    pub chance_min: f64,
    /// Chance per month that investors outside the game fund an open round.
    pub investor_chance: f64,
    pub deadline_months: u32,
    /// Stakes of companies (SU2).
    pub stakes: VentureStakeModel,
    /// Blur of the shown chance without a strategy department.
    pub blur: f64,
    /// From which shown chance the level is medium, and high.
    pub chance_levels: (f64, f64),
    /// Years closed start-ups stay in the list.
    pub keep_years: u32,
    pub inventors: Vec<Inventor>,
    pub provenance: Provenance,
}

/// A credit grade of bonds (K2): the worst debt ratio and the least interest coverage it
/// allows, and its spread over the base rate.
#[derive(Clone, Debug, PartialEq)]
pub struct BondGrade {
    /// Key of the text `bonitaet.<key>`.
    pub key: String,
    pub debt_ratio_max: f64,
    pub coverage_min: f64,
    pub spread: f64,
}

/// The player's banks (K4, docs/FORMELN.md). Disabled without the section.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BankModel {
    pub enabled: bool,
    /// Deposits at most this multiple of the equity.
    pub leverage_max: f64,
    /// The deposit spread over the base rate that draws half the capacity.
    pub neutral_spread: f64,
    /// How fast the deposits' target grows with the spread.
    pub elasticity: f64,
    /// Share of the gap to the target closed each month.
    pub adjustment: f64,
    /// Share of the deposits kept as cash.
    pub reserve: f64,
    pub start_deposit_spread: f64,
    pub start_loan_discount: f64,
    pub start_max_debt_ratio: f64,
    pub provenance: Provenance,
}

impl BondGrade {
    /// Keys a grade may have, best first (texts `bonitaet.<key>` in the interface).
    pub const KEYS: [&'static str; 7] = ["aaa", "aa", "a", "bbb", "bb", "b", "ccc"];
}

/// Corporate bonds (K2, docs/FORMELN.md). Disabled without the section.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BondModel {
    pub enabled: bool,
    pub equity_min: Money,
    pub volume_min: Money,
    pub term_min_years: u32,
    pub term_max_years: u32,
    pub cost_share: f64,
    pub redeem_premium: f64,
    /// Closed months the interest coverage is measured over.
    pub earnings_months: u32,
    /// Best grade first.
    pub grades: Vec<BondGrade>,
    pub ai_term_years: u32,
    /// The AI issues a bond when its coupon is at least this far below the loan rate.
    pub ai_advantage_min: f64,
    pub provenance: Provenance,
}

/// Stock market (K1, docs/FORMELN.md). Disabled without the section.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StockModel {
    pub enabled: bool,
    pub book_weight: f64,
    pub pe: f64,
    pub book_floor: f64,
    /// Yearly profit per equity assumed for months without figures.
    pub assumed_return: f64,
    /// Closed months the earnings are averaged over (at most `ledger::MONTHS_KEPT`).
    pub earnings_months: u32,
    pub sentiment_volatility: f64,
    pub sentiment_reversion: f64,
    pub inertia: f64,
    pub noise: f64,
    pub ipo_equity_min: Money,
    pub ipo_share_max: f64,
    pub ipo_discount: f64,
    pub ipo_cost_share: f64,
    pub trade_premium: f64,
    pub trade_discount: f64,
    pub trade_impact: f64,
    pub trade_share_max: f64,
    pub start_equity_min: Money,
    pub start_free_float: f64,
    pub ai_ipo_chance: f64,
    pub ai_ipo_share: f64,
    /// Takeover premium over the market value and its costs (K3).
    pub takeover_premium: f64,
    pub takeover_cost_share: f64,
    pub buyback_share_max: f64,
    /// AI investors: share of the surplus cash put into shares, the largest stake, the
    /// band around the fair value, the monthly takeover chance and the cash it may use.
    pub ai_portfolio_cash_share: f64,
    pub ai_portfolio_stake_max: f64,
    pub ai_undervaluation: f64,
    pub ai_takeover_chance: f64,
    pub ai_takeover_cash_share: f64,
    pub provenance: Provenance,
}

/// Subsidiaries (W6, docs/FORMELN.md). Disabled without the section.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubsidiaryModel {
    pub enabled: bool,
    pub min_capital: Money,
    pub founding_cost: Money,
    /// Character of the subsidiaries' own management (the AI rules).
    pub competence: f64,
    pub aggressiveness: f64,
    /// Logistics subsidiaries: share of cash for vehicles a month, least yearly return.
    pub logistics_cash_share: f64,
    pub logistics_min_return: f64,
    pub provenance: Provenance,
}

/// What a fleet needs to know of a vehicle (W5).
#[derive(Clone, Debug, PartialEq)]
pub struct FleetVehicle {
    pub payload_t: TimeSeries,
    pub price_usd: TimeSeries,
}

/// Logistics (W5, docs/FORMELN.md). Disabled without the section.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LogisticsModel {
    pub enabled: bool,
    /// State transport: surcharge on the route cost and factor on the risk.
    pub state_surcharge: f64,
    pub state_risk_factor: f64,
    /// Margin of the logistics companies in the market freight: what a fully used fleet
    /// saves.
    pub market_margin: f64,
    /// Share of the capacity a fleet really uses (loading, empty runs back).
    pub load: f64,
    /// Upkeep per year as a share of the purchase value.
    pub upkeep_share: f64,
    pub life_years: f64,
    /// Sale at this share of the book value.
    pub sale_share: f64,
    /// Share of free capacity that finds cargo of others.
    pub rental_share: f64,
    /// Share of the freight market company fleets may take over (W6).
    pub rental_market_share: f64,
    /// Chance of losing a load by land and by sea, by year.
    pub risk_land: Option<TimeSeries>,
    pub risk_sea: Option<TimeSeries>,
    pub ai_share: f64,
    pub ai_cash_share: f64,
    pub provenance: Provenance,
}

/// Supply contracts (W4, docs/FORMELN.md). With `months_max` 0 there are none.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContractModel {
    pub months_max: u32,
    pub months_default: u32,
    /// Penalty as a share of the value of the missing quantity.
    pub penalty_max: f64,
    pub penalty_default: f64,
    /// Whoever cancels pays the penalty on this many monthly quantities.
    pub cancel_months: u32,
    /// Days a proposal of an AI company waits for the player's answer.
    pub proposal_days: u32,
    /// Months closed contracts stay in the list.
    pub keep_months: u32,
    pub ai: ContractAiModel,
    pub provenance: Provenance,
}

/// How AI companies answer and propose contracts.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContractAiModel {
    /// An AI seller takes up to this much less than its own price for a sure buyer.
    pub sale_discount: f64,
    /// An AI buyer pays up to this much more than the market price for sure supplies.
    pub purchase_premium: f64,
    /// At most this share of its output or need goes into contracts.
    pub share: f64,
    pub penalty_max: f64,
    /// Chance per month and product of the player that an AI company proposes one.
    pub proposal_chance: f64,
}

impl ContractModel {
    pub fn enabled(&self) -> bool {
        self.months_max > 0
    }
}

/// Import tariffs (W3, docs/FORMELN.md). Without `default` there are no tariffs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TariffModel {
    /// Average import tariff of countries without their own series (share of the value).
    pub default: Option<TimeSeries>,
    /// Own series per country (index = `CountryId`).
    pub countries: Vec<Option<TimeSeries>>,
    /// Factor per goods group (index = `GoodsGroupId`), 1 without an entry.
    pub groups: Vec<f64>,
    /// Factor of single products instead of their group's (index = `ProductId`), e.g. 0
    /// for duty-free fertilizer.
    pub products: Vec<Option<f64>>,
    pub zones: Vec<TariffZone>,
    pub embargoes: Vec<Embargo>,
    pub dynamics: TariffDynamics,
    pub provenance: Provenance,
}

/// Countries trading at a fraction of the tariff among themselves.
#[derive(Clone, Debug, PartialEq)]
pub struct TariffZone {
    pub key: String,
    pub factor: f64,
    /// Member, year of entry and year of exit (member before it).
    pub members: Vec<(CountryId, i32, Option<i32>)>,
}

/// No trade between two countries from a year until before another.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Embargo {
    pub countries: (CountryId, CountryId),
    pub from: i32,
    pub until: Option<i32>,
}

/// Random change of the tariffs after the data ends.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TariffDynamics {
    /// Standard deviation of the yearly change.
    pub deviation: f64,
    pub min: f64,
    pub max: f64,
    /// Choices of the new game: key (`zoll.dynamik.<key>`) and factor on `deviation`.
    pub levels: Vec<(String, f64)>,
    pub default_level: usize,
}

impl TariffModel {
    pub fn enabled(&self) -> bool {
        self.default.is_some()
    }

    /// The factor of the default dynamics; 1 without choices.
    pub fn default_factor(&self) -> f64 {
        self.dynamics
            .levels
            .get(self.dynamics.default_level)
            .map_or(1.0, |l| l.1)
    }

    /// Last year with data: the dynamics start after it.
    pub fn last_year(&self) -> i32 {
        self.default
            .iter()
            .chain(self.countries.iter().flatten())
            .filter_map(|s| s.points().last().map(|p| p.0))
            .max()
            .unwrap_or(i32::MAX)
    }
}

impl VentureModel {
    pub fn enabled(&self) -> bool {
        self.per_year > 0.0 && !self.phases.is_empty()
    }

    /// The factor of the default choice; 1 without choices.
    pub fn default_factor(&self) -> f64 {
        self.frequencies
            .get(self.default_frequency)
            .map_or(1.0, |f| f.1)
    }

    /// The label of a year: the last whose year is reached.
    pub fn label(&self, year: i32) -> Option<&str> {
        self.labels
            .iter()
            .rev()
            .find(|(_, from)| *from <= year)
            .or_else(|| self.labels.first())
            .map(|(key, _)| key.as_str())
    }
}

/// Stakes of companies in start-ups (SU2, docs/FORMELN.md).
#[derive(Clone, Debug, PartialEq)]
pub struct VentureStakeModel {
    /// Value at success as a multiple of the value after the last round.
    pub success_factor: f64,
    /// Above the value when buying between rounds or buying out; below it when selling.
    pub buy_premium: f64,
    pub sale_discount: f64,
    /// Share of the gap to a sure phase a grant of the phase's capital closes.
    pub grant_effect: f64,
    /// Shares for the blocking minority and the majority.
    pub blocking: f64,
    pub majority: f64,
    /// Share of the research effort the majority gets when it fails.
    pub research_bonus: f64,
    pub chance_max: f64,
    /// Factors on months and chance of the phases: fast and thorough.
    pub fast: (f64, f64),
    pub thorough: (f64, f64),
    /// Return per dollar above 1 the strategy department wants at a readiness for risks
    /// of 0, and the most of the cash it puts in.
    pub min_return: f64,
    pub cash_share: f64,
    /// Progress of a research project from which it may be spun off (SU3).
    pub spin_off_progress_min: f64,
    pub ai: VentureAiModel,
}

impl Default for VentureStakeModel {
    fn default() -> Self {
        Self {
            success_factor: 1.0,
            buy_premium: 0.0,
            sale_discount: 0.0,
            grant_effect: 0.0,
            blocking: 0.25,
            majority: 0.5,
            research_bonus: 0.0,
            chance_max: 1.0,
            fast: (1.0, 1.0),
            thorough: (1.0, 1.0),
            min_return: 0.0,
            cash_share: 0.0,
            spin_off_progress_min: 1.0,
            ai: VentureAiModel::default(),
        }
    }
}

/// How AI companies take part in start-ups (SU3, docs/FORMELN.md).
#[derive(Clone, Debug, PartialEq)]
pub struct VentureAiModel {
    /// Chance a month that a company with at least `cash_min` looks at the start-ups.
    pub check_chance: f64,
    pub cash_min: Money,
    /// The most of its cash it pledges in a month.
    pub cash_share: f64,
    /// Return per dollar above 1 it wants at an aggressiveness of 0.
    pub min_return: f64,
    /// It takes over useful start-ups from this shown chance, if it already holds this
    /// share, for up to this share of its cash.
    pub takeover_chance_min: f64,
    pub takeover_share_min: f64,
    pub takeover_cash: f64,
    /// Chance at the start of a year that it spins off a research project, and the share
    /// it sells to investors then.
    pub spin_off_chance: f64,
    pub spin_off_sale: f64,
}

impl Default for VentureAiModel {
    fn default() -> Self {
        Self {
            check_chance: 0.0,
            cash_min: Money::ZERO,
            cash_share: 0.0,
            min_return: 0.0,
            takeover_chance_min: 1.0,
            takeover_share_min: 1.0,
            takeover_cash: 0.0,
            spin_off_chance: 0.0,
            spin_off_sale: 0.0,
        }
    }
}

/// A phase of a start-up (SU1).
#[derive(Clone, Debug, PartialEq)]
pub struct VenturePhase {
    /// Text `startup.phase.<key>`.
    pub key: String,
    pub months: u32,
    pub capital: Money,
    pub chance: f64,
    /// Value before the round as a multiple of the capital raised.
    pub valuation: f64,
}

/// A historical inventor of a technology (SU1).
#[derive(Clone, Debug, PartialEq)]
pub struct Inventor {
    pub technology: TechnologyId,
    pub name: String,
    pub country: CountryId,
    pub provenance: Provenance,
}

/// Moving the headquarters (ZA1).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HeadquartersModel {
    pub months: u32,
    pub cost_base: Money,
    pub cost_per_employee: Money,
    /// Share of the central departments' employees who move along.
    pub moving_share: f64,
}

/// The living market of managers (MA6, docs/FORMELN.md). All zero: nothing moves.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ManagerMarketModel {
    /// Chance per month of a point of expertise, and the room above the focus at the
    /// draw that a manager can grow into.
    pub experience_chance: f64,
    pub experience_room: u8,
    pub satisfaction: SatisfactionModel,
    /// Below this satisfaction a manager may resign, at most with this chance a month.
    pub resignation_threshold: u8,
    pub resignation_chance: f64,
    pub poaching: PoachingModel,
    pub ai: AiHiringModel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SatisfactionModel {
    pub start: u8,
    pub base: f64,
    /// Points per percent of salary above (below) the market value.
    pub salary_weight: f64,
    pub loss_penalty: f64,
    pub overruled_penalty: f64,
    /// Share of the way to the target a month (0–1).
    pub adjust: f64,
    /// Upper bounds of the levels "unzufrieden" and "gemischt".
    pub bands: [u8; 2],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PoachingModel {
    pub min_strength: f64,
    /// How much stronger than the best free candidate.
    pub lead: f64,
    pub markup: f64,
    pub ignored_penalty: u8,
    /// AI companies keep a manager up to this factor of his salary.
    pub ai_counter_max: f64,
    /// Months after an offer before the next one to the same manager.
    pub pause_months: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AiHiringModel {
    pub per_month: u32,
    pub ceo_revenue: Money,
    pub site_revenue: Money,
    /// Salary at most this share of the unit's revenue in twelve months.
    pub salary_share: f64,
    pub competence_ceo: f64,
    pub competence_heads: f64,
}

/// Candidates per continent (MA1).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ManagerPoolModel {
    pub per_million_academics: f64,
    pub min: u32,
    pub max: u32,
    /// Chance that a free candidate leaves the market at the start of a month.
    pub leave_per_month: f64,
}

/// Distributions of the skills of new managers (0–100).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkillModel {
    /// Mean and spread of the expertise in the focus function.
    pub focus: (f64, f64),
    pub other: (f64, f64),
    /// Detection, judgment and leadership.
    pub general: (f64, f64),
    /// Largest offset of the shown impression from the true skill.
    pub impression_blur: f64,
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

    /// Workable reserve of a deposit in a year (before the market scale), `None` for
    /// renewable deposits: the deposit's value times the output index of its raw
    /// material, as exploration and technology let the reserves grow with the output
    /// (M41).
    pub fn reserve(&self, deposit: DepositId, year: i32) -> Option<f64> {
        let d = self.deposits.get(deposit);
        let index = self
            .products
            .get(d.resource)
            .output_index
            .as_ref()
            .map_or(1.0, |i| i.value_at(f64::from(year)));
        d.reserve.map(|r| r * index)
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
                IdKind::Milestone => self.milestones.keys().to_vec(),
            })
            .collect();
        let mut table = KeyTable::new(keys);
        for (id, country) in self.countries.iter() {
            for member in &country.members {
                table.add_alias(IdKind::Country, member, crate::ids::Id::index(id));
            }
        }
        table
    }
}

/// A goal for the player (M23). It has no effect on the simulation.
#[derive(Clone, Debug, PartialEq)]
pub struct Milestone {
    pub condition: MilestoneCondition,
}

/// When a milestone is reached (docs/FORMELN.md, M23).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MilestoneCondition {
    FirstSale,
    ProfitMonth,
    /// Finished units of all facilities.
    Facilities(u32),
    /// An own facility makes what another one uses.
    OwnInput,
    /// Sites in this many countries.
    Countries(u32),
    /// A technology researched by the company itself.
    Research,
    /// Sold most of a product in a country last month, with at least this share.
    MarketLeader(f64),
    /// Equity of at least this multiple of the start capital.
    Equity(f64),
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
    /// Payload of one vehicle in tonnes and its price in USD (W5); `None`: not for
    /// fleets.
    pub fleet: Option<FleetVehicle>,
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
    /// Development of researched products (M37).
    pub development: DevelopmentModel,
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
            development: DevelopmentModel::default(),
        }
    }
}

/// Development of researched products level by level (M37, docs/FORMELN.md).
#[derive(Clone, Debug, PartialEq)]
pub struct DevelopmentModel {
    /// Highest level; 0 turns development off.
    pub levels: u8,
    /// Per level in all recipes of the product: quality points, and the shares of the
    /// labor hours and of the inputs saved per run.
    pub quality_per_level: f64,
    pub labor_per_level: f64,
    pub inputs_per_level: f64,
    /// Effort of level n: base · share · growth^(n − 1); the base is the largest research
    /// effort of the product's technologies, at least `base_effort`.
    pub effort_share: f64,
    pub effort_growth: f64,
    pub base_effort: f64,
    /// Years after the first company reached a level when every company has it.
    pub public_domain_years: f64,
    /// Research field of products without a technology, indexed by `BranchId`.
    pub fields: Vec<Option<SpecializationId>>,
}

impl Default for DevelopmentModel {
    fn default() -> Self {
        Self {
            levels: 0,
            quality_per_level: 4.0,
            labor_per_level: 0.03,
            inputs_per_level: 0.02,
            effort_share: 0.2,
            effort_growth: 1.6,
            base_effort: 10_000.0,
            public_domain_years: 15.0,
            fields: Vec::new(),
        }
    }
}

/// A value that depends on a company trait (competence or aggressiveness, 0–1).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

/// Where a plot lies (M35).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Location {
    #[default]
    City,
    Port,
    Rural,
}

impl Location {
    pub const ALL: [Location; 3] = [Location::City, Location::Port, Location::Rural];

    /// Key in the data and texts (`lage.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            Location::City => "stadt",
            Location::Port => "hafen",
            Location::Rural => "land",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

/// Size of a facility (M36); the data values of a facility are those of `Medium`.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum FacilitySize {
    VerySmall,
    Small,
    #[default]
    Medium,
    Large,
    VeryLarge,
}

impl FacilitySize {
    pub const ALL: [FacilitySize; 5] = [
        FacilitySize::VerySmall,
        FacilitySize::Small,
        FacilitySize::Medium,
        FacilitySize::Large,
        FacilitySize::VeryLarge,
    ];

    /// Key in the data and texts (`anlagengroesse.<key>`).
    pub fn key(self) -> &'static str {
        match self {
            FacilitySize::VerySmall => "sehr_klein",
            FacilitySize::Small => "klein",
            FacilitySize::Medium => "mittel",
            FacilitySize::Large => "gross",
            FacilitySize::VeryLarge => "sehr_gross",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    /// The next smaller size, if any.
    pub fn smaller(self) -> Option<FacilitySize> {
        self.index().checked_sub(1).map(|i| FacilitySize::ALL[i])
    }

    /// Steps away from `Medium`.
    fn steps_from_medium(self) -> usize {
        self.index().abs_diff(FacilitySize::Medium.index())
    }
}

/// What the size of a facility changes (M36): capacity by its factor `k`, investment,
/// area and construction time by `k` to a power, labor per run by `k` to a power.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SizeModel {
    /// `k` per size (`FacilitySize::index`); 1 for `Medium`.
    pub capacity: [f64; 5],
    pub investment_exponent: f64,
    pub labor_exponent: f64,
    pub area_exponent: f64,
    pub build_exponent: f64,
}

impl Default for SizeModel {
    /// Every size like `Medium` (catalogs without sizes, tests).
    fn default() -> Self {
        Self {
            capacity: [1.0; 5],
            investment_exponent: 0.0,
            labor_exponent: 0.0,
            area_exponent: 0.0,
            build_exponent: 0.0,
        }
    }
}

impl SizeModel {
    pub fn capacity(&self, size: FacilitySize) -> f64 {
        self.capacity[size.index()]
    }

    fn power(&self, size: FacilitySize, exponent: f64) -> f64 {
        if size == FacilitySize::Medium {
            1.0
        } else {
            libm::pow(self.capacity(size), exponent)
        }
    }

    /// Factor on the investment of the data size.
    pub fn investment(&self, size: FacilitySize) -> f64 {
        self.power(size, self.investment_exponent)
    }

    /// Factor on the labor hours per run of the data size.
    pub fn labor(&self, size: FacilitySize) -> f64 {
        self.power(size, self.labor_exponent)
    }

    /// Factor on the land per unit of the data size (M35).
    pub fn area(&self, size: FacilitySize) -> f64 {
        self.power(size, self.area_exponent)
    }

    /// Construction time of a size for the days of the data size (at least one day).
    pub fn build_days(&self, size: FacilitySize, days: u32) -> u32 {
        if size == FacilitySize::Medium {
            return days;
        }
        // Construction times are a few hundred days; the cast cannot overflow.
        (f64::from(days) * self.power(size, self.build_exponent))
            .round()
            .max(1.0) as u32
    }

    /// Size and number of units for `wanted` capacity in units of the data size: the
    /// largest size with `k <= wanted` (else the smallest), as many units as round
    /// `wanted / k` (at least one). Among sizes of equal capacity the one nearest to
    /// `Medium` wins.
    pub fn units_for(&self, wanted: f64) -> (FacilitySize, u32) {
        let better = |a: FacilitySize, b: FacilitySize, larger: bool| {
            let (ka, kb) = (self.capacity(a), self.capacity(b));
            if ka == kb {
                a.steps_from_medium() < b.steps_from_medium()
            } else {
                (ka > kb) == larger
            }
        };
        let mut best: Option<FacilitySize> = None;
        for size in FacilitySize::ALL {
            if self.capacity(size) <= wanted + 1e-9 && best.is_none_or(|b| better(size, b, true)) {
                best = Some(size);
            }
        }
        let size = best.unwrap_or_else(|| {
            FacilitySize::ALL
                .into_iter()
                .reduce(|b, s| if better(s, b, false) { s } else { b })
                .unwrap_or_default()
        });
        // Small counts; the cast cannot overflow.
        let count = (wanted / self.capacity(size)).round().max(1.0) as u32;
        (size, count)
    }
}

/// What a location means for plots and sites (M35).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocationModel {
    /// Share of the new plots (ports only where there is a coast).
    pub share: f64,
    pub area_factor: f64,
    pub price_factor: f64,
    /// Added to the wage premium when sites compete for workers (M18).
    pub hiring: f64,
    /// Factor on freight by sea to and from sites here.
    pub sea_freight: f64,
    /// Share of the revenue on the home market paid for deliveries.
    pub delivery_cost: f64,
}

impl Default for LocationModel {
    fn default() -> Self {
        Self {
            share: 0.0,
            area_factor: 1.0,
            price_factor: 1.0,
            hiring: 0.0,
            sea_freight: 1.0,
            delivery_cost: 0.0,
        }
    }
}

/// A size class of new plots (M35).
#[derive(Clone, Debug, PartialEq)]
pub struct PlotClass {
    pub key: String,
    /// Smallest and largest area in ha in the reference year.
    pub area_ha: (f64, f64),
    /// Share of the new plots in rich, middle and poor countries.
    pub shares: [f64; 3],
}

/// Plots of land (`parameter/grundstuecksmodell.yaml`, formulas in docs/FORMELN.md, M35).
#[derive(Clone, Debug, PartialEq)]
pub struct PlotModel {
    /// Commercial land per bn USD of GDP (times the market scale).
    pub area_per_gdp_bn_ha: f64,
    /// New plots grow by 1 + (year − `growth_from_year`) / `growth_years`.
    pub growth_from_year: i32,
    pub growth_years: f64,
    pub rich_from_usd: f64,
    pub poor_below_usd: f64,
    /// Size classes; none means the game has no plots (small test catalogs).
    pub classes: Vec<PlotClass>,
    /// Indexed by `Location::index`.
    pub locations: [LocationModel; 3],
    pub land_price_usd_per_ha: f64,
    /// Land prices rise by this times the occupied share of a country's land.
    pub scarcity: f64,
    /// Yearly rent as share of the land value.
    pub rent_share: f64,
    /// Area of facilities without their own value: investment per ha.
    pub investment_per_ha_usd: f64,
    /// Added to the facilities' area for ways, stores and offices.
    pub overhead: f64,
    pub min_site_area_ha: f64,
    /// Room the AI keeps for growth when it chooses a plot.
    pub ai_reserve: f64,
}

impl PlotModel {
    pub fn enabled(&self) -> bool {
        !self.classes.is_empty()
    }

    pub fn location(&self, location: Location) -> &LocationModel {
        &self.locations[location.index()]
    }
}

impl Default for PlotModel {
    fn default() -> Self {
        Self {
            area_per_gdp_bn_ha: 25.0,
            growth_from_year: 1900,
            growth_years: 30.0,
            rich_from_usd: 15_000.0,
            poor_below_usd: 5_000.0,
            classes: Vec::new(),
            locations: [LocationModel::default(); 3],
            land_price_usd_per_ha: 200_000.0,
            scarcity: 2.0,
            rent_share: 0.05,
            investment_per_ha_usd: 10_000_000.0,
            overhead: 0.2,
            min_site_area_ha: 0.2,
            ai_reserve: 0.5,
        }
    }
}

/// Offers between companies (`data/parameter/kaufmodell.yaml`, docs/FORMELN.md M30).
#[derive(Clone, Debug, PartialEq)]
pub struct DealModel {
    /// Months an offer waits for an answer.
    pub valid_months: u32,
    /// Months a buyer waits after a declined or expired offer for the same object.
    pub block_months: u32,
    /// Months a site must exist before it can be bought.
    pub min_age_months: u32,
    /// Earnings value = yearly result × this many years.
    pub earnings_years: f64,
    /// Sites with fewer closed months have no earnings value.
    pub earnings_min_months: u32,
    /// Years over which bought goodwill is written off.
    pub goodwill_years: f64,
    /// Workers from this qualification rank on count as qualified.
    pub qualified_rank: u8,
    pub ai: DealAi,
    /// Days the sites of an insolvent AI company are auctioned (M38; 0: given up at once).
    pub insolvency_days: u32,
    /// Lowest bid in such an auction, as share of a site's base value.
    pub insolvency_min_share: f64,
    /// Shares of companies (PE5).
    pub stakes: StakeModel,
}

/// Prices of shares of companies (PE5, docs/FORMELN.md).
#[derive(Clone, Debug, PartialEq)]
pub struct StakeModel {
    /// Premium over the company's value a holder asks: with poor and with good earnings.
    pub premium_min: f64,
    pub premium_max: f64,
    /// Net profit per equity from which the earnings count as good.
    pub good_return: f64,
    /// Discount of investors buying at once: with good and with poor earnings.
    pub discount_min: f64,
    pub discount_max: f64,
}

impl Default for StakeModel {
    fn default() -> Self {
        Self {
            premium_min: 0.2,
            premium_max: 0.5,
            good_return: 0.15,
            discount_min: 0.05,
            discount_max: 0.2,
        }
    }
}

/// How AI companies buy and sell (spans over aggressiveness).
#[derive(Clone, Debug, PartialEq)]
pub struct DealAi {
    /// Chance per month to look for a deal.
    pub chance: Span,
    pub open_max: u32,
    /// New offers to the player per month, from all AI companies together.
    pub player_offers_per_month: u32,
    /// Highest price at least this share above the base value.
    pub min_advantage: f64,
    pub bid_markup: Span,
    pub min_price_usd: f64,
    /// A price at most this share of the cash.
    pub cash_share_max: f64,
    pub competition_markup: Span,
    pub staff_markup: f64,
    pub build_time_markup: f64,
    pub new_build_share: f64,
    pub license_bid: Span,
    pub license_max: f64,
    pub sale_markup: Span,
    /// A site with this share of the revenue (or the only one with facilities) is core.
    pub core_share: f64,
    pub core_markup: f64,
    pub license_min: f64,
    pub license_competition: f64,
    /// Prices from this share of the minimum get a counter-offer.
    pub counter_threshold: f64,
}

impl Default for DealModel {
    fn default() -> Self {
        Self {
            valid_months: 2,
            block_months: 12,
            min_age_months: 12,
            earnings_years: 5.0,
            earnings_min_months: 3,
            goodwill_years: 10.0,
            qualified_rank: 3,
            ai: DealAi {
                chance: Span {
                    at_0: 0.02,
                    at_1: 0.06,
                },
                open_max: 2,
                player_offers_per_month: 1,
                min_advantage: 0.1,
                bid_markup: Span {
                    at_0: 0.05,
                    at_1: 0.2,
                },
                min_price_usd: 10_000.0,
                cash_share_max: 0.5,
                competition_markup: Span {
                    at_0: 0.1,
                    at_1: 0.6,
                },
                staff_markup: 0.3,
                build_time_markup: 0.15,
                new_build_share: 0.5,
                license_bid: Span {
                    at_0: 0.3,
                    at_1: 0.6,
                },
                license_max: 0.9,
                sale_markup: Span {
                    at_0: 0.1,
                    at_1: 0.3,
                },
                core_share: 0.4,
                core_markup: 0.5,
                license_min: 0.3,
                license_competition: 1.0,
                counter_threshold: 0.7,
            },
            // Catalogs without auctions give the sites up at once.
            insolvency_days: 0,
            insolvency_min_share: 0.5,
            stakes: StakeModel::default(),
        }
    }
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
    /// Error of an AI company's estimates of cost and margin (competence; B2).
    pub estimate_error: Span,
    /// Training target of an AI company's sites (competence; W1).
    pub training: Span,
    /// Monthly advertising as share of the revenue of a goods group in a country.
    pub advertising_share: Span,
    pub purchase_markup: f64,
    /// Expansion when the utilization and the margin reach these (aggressiveness).
    pub expand_utilization: Span,
    pub expand_margin: Span,
    /// Least planned utilization of all running units of a product worldwide for any of
    /// them to grow (C4); 0: no limit.
    pub expand_market_load: f64,
    /// No expansion while an input costs more than this multiple of its reference price
    /// in the country: it is scarce (M16).
    pub expand_input_price_max: f64,
    pub invest_share_max: f64,
    /// Expansions a company starts at most per check, the best first (C2).
    pub expansions_max: u32,
    /// Wage premium of a site: raised by the step while a facility waits for workers,
    /// lowered otherwise, up to the maximum (M18).
    pub wage_premium_step: f64,
    pub wage_premium_max: f64,
    /// Research on technologies up to this many years before their invention (competence).
    pub research_lookahead_years: Span,
    pub research_min_revenue_usd: f64,
    pub research_competence_min: f64,
    /// Companies researching the same technology for a market gap at once (M32).
    pub research_gap_companies: u32,
    /// Products that can be made historically within this many years count as market
    /// gaps for research already (M39); 0 = off.
    pub research_lead_years: u32,
    /// Development (M37): yearly benefit of one level as share of the product's revenue,
    /// and the years in which the next level must pay for itself.
    pub development_benefit_per_level: f64,
    pub development_payback_years: f64,
    pub cash_min_months: f64,
    pub cash_max_months: f64,
    pub loan_years: u32,
    pub foundings_per_month: u32,
    /// Rich companies that build in another company's bottleneck per quarter.
    pub diversifications_per_quarter: u32,
    /// Dear or short markets with few makers that get one more maker per quarter (C3).
    pub entries_per_quarter: u32,
    /// Markets that pay at least this multiple of the reference price draw newcomers
    /// (M33) while fewer than `entry_companies_max` companies make the product (0: never);
    /// a newcomer plans for `entry_share` of what sells.
    pub entry_price_factor: f64,
    pub entry_companies_max: u32,
    pub entry_share: f64,
    /// A new concession only where the remaining reserve lasts this many years of its
    /// full output (M33).
    pub reserve_years_min: f64,
    pub founding_capital_factor: f64,
    /// Below this planned utilization of a product at a site the AI shuts down the
    /// units it does not need at the target utilization (M22).
    pub mothball_utilization: f64,
    pub mothball_target_utilization: f64,
    /// Only below this price (× reference price in the country) (M22).
    pub mothball_price_max: f64,
    /// A shut down facility starts up again when more than this share of the running
    /// facilities' full output is taken (M22).
    pub restart_utilization: f64,
    /// Units shut down for longer than this are sold (M22).
    pub sell_after_months: u32,
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
                estimate_error: Span::default(),
                training: Span::default(),
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
                expand_market_load: 0.0,
                expand_input_price_max: 1.5,
                invest_share_max: 0.3,
                expansions_max: 1,
                wage_premium_step: 0.05,
                wage_premium_max: 0.3,
                research_lookahead_years: Span {
                    at_0: 0.0,
                    at_1: 2.0,
                },
                research_min_revenue_usd: 5_000_000.0,
                research_competence_min: 0.5,
                research_gap_companies: 2,
                research_lead_years: 0,
                development_benefit_per_level: 0.03,
                development_payback_years: 5.0,
                cash_min_months: 2.0,
                cash_max_months: 6.0,
                loan_years: 10,
                foundings_per_month: 2,
                diversifications_per_quarter: 4,
                entries_per_quarter: 0,
                entry_price_factor: 1.3,
                entry_companies_max: 0,
                entry_share: 0.25,
                reserve_years_min: 10.0,
                founding_capital_factor: 1.5,
                mothball_utilization: 0.5,
                mothball_target_utilization: 0.8,
                mothball_price_max: 1.0,
                restart_utilization: 0.95,
                sell_after_months: 24,
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

/// A historical event shown as world news (Lastenheft §4.1, §13.2). The country values
/// contain its economic slump; its effects on markets, trade and companies follow
/// (H1, docs/FORMELN.md).
#[derive(Clone, Debug, PartialEq)]
pub struct HistoricalEvent {
    pub key: String,
    pub date: crate::calendar::Date,
    /// Kind of event, text `ereignisart.<kind>`.
    pub kind: String,
    pub countries: Vec<CountryId>,
    pub effects: Vec<EventEffect>,
    pub provenance: Provenance,
}

/// An effect of a historical event (H1).
#[derive(Clone, Debug, PartialEq)]
pub struct EventEffect {
    /// The countries it acts in: its own, else those of the event.
    pub countries: Vec<CountryId>,
    pub kind: EffectKind,
    /// End of an effect with a duration; `None` until the end of the game.
    pub until: Option<crate::calendar::Date>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectKind {
    /// Consumer and state demand for goods of the groups (all without groups).
    Demand {
        groups: Vec<GoodsGroupId>,
        consumer: f64,
        state: f64,
    },
    /// No trade between the countries and `against`.
    Embargo { against: Vec<CountryId> },
    /// Extra tariff on imports into the countries from `against` (all others if empty).
    Tariff {
        against: Vec<CountryId>,
        surcharge: f64,
    },
    /// Available workers of all groups.
    Labor { factor: f64 },
    /// Runs of the facilities making goods of the groups (all without groups).
    Production {
        groups: Vec<GoodsGroupId>,
        factor: f64,
    },
    /// Companies from other countries open no sites there; with `all` none but the
    /// state company.
    Closure { all: bool },
    /// Once: a share of the facility units and stocks destroyed.
    Destruction { share: f64 },
    /// Once: sites handed to the country's state company.
    Expropriation {
        foreign_only: bool,
        compensation: f64,
    },
    /// Once: drop of the stock market's sentiment (0–1).
    StockCrash { drop: f64 },
}

impl EffectKind {
    /// Effects that act once, on the event's day, rather than for a time.
    pub fn once(&self) -> bool {
        matches!(
            self,
            Self::Destruction { .. } | Self::Expropriation { .. } | Self::StockCrash { .. }
        )
    }
}

/// A value per country and year (PE1): a series for all and own series for some countries.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CountrySeries {
    /// For countries without their own; `None`: 0.
    pub default: Option<crate::time_series::TimeSeries>,
    /// Indexed by country; `None` uses the default.
    pub countries: Vec<Option<crate::time_series::TimeSeries>>,
}

impl CountrySeries {
    pub fn value(&self, country: CountryId, year: f64) -> f64 {
        self.countries
            .get(country.index())
            .and_then(Option::as_ref)
            .or(self.default.as_ref())
            .map_or(0.0, |s| s.value_at(year))
    }
}

/// Age at entering the pool of managers: a truncated normal distribution (PE1).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AgeSpan {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub spread: f64,
}

/// Age, retirement and death of the managers (`parameter/lebenslauf.yaml`, PE1).
/// Disabled without the section: managers then neither age nor retire.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LifeModel {
    pub enabled: bool,
    /// Per level: site, country, continent, board.
    pub entry_age: [AgeSpan; 4],
    /// Strength from which a candidate counts for country, continent, board.
    pub level_strength: [f64; 3],
    /// Experience chance factors: below `young_until` and from `old_from` on.
    pub young_until: f64,
    pub young_factor: f64,
    pub old_from: f64,
    pub old_factor: f64,
    pub risk_from: f64,
    pub risk_per_year: f64,
    pub risk_max: f64,
    pub decline_from: f64,
    pub decline_chance: f64,
    /// Personal deviation of the retirement age (± years).
    pub retirement_spread: f64,
    pub warning_months: u32,
    pub extension_years_max: u32,
    pub extension_raise: f64,
    /// The retirement age of the year in which he reaches this age applies to a manager.
    pub reference_age: f64,
    /// The chance to agree to stay longer falls from the first age to nothing at the second.
    pub acceptance_ages: [f64; 2],
    pub mortality_from: f64,
    /// Monthly chance of death at the life expectancy.
    pub mortality_chance: f64,
    pub doubling_years: f64,
    pub retirement_age: CountrySeries,
    pub life_expectancy: CountrySeries,
    pub provenance: Provenance,
}

/// The player as a person (`parameter/person.yaml`, PE2). Without the section the
/// person has no children.
#[derive(Clone, Debug, PartialEq)]
pub struct PersonModel {
    pub enabled: bool,
    /// Age at the start: default, least and most.
    pub start_age: [u32; 3],
    /// Children come from the first to below the second age, at most so many.
    pub children_ages: [u32; 2],
    pub children_max: u32,
    /// Chance of a child per year.
    pub child_chance: f64,
    /// Age from which a child has a manager card.
    pub card_age: u32,
    /// Founding a company (PE3).
    pub founding: FoundingModel,
    /// With co-owners the person's salary is at most this multiple of the suggestion.
    pub salary_co_owner_max: f64,
    /// Loans of the person to its companies: highest yearly rate, longest term in years.
    pub loan_max_rate: f64,
    pub loan_max_years: u32,
    /// The levels of lifestyle, indexed by `Lifestyle`.
    pub lifestyles: [LifestyleLevel; 4],
    pub default_lifestyle: crate::state::Lifestyle,
    /// The lifestyle changes at most once in so many months.
    pub lifestyle_months: u32,
    /// Income tax rate and savings rate by country and year.
    pub income_tax: CountrySeries,
    pub savings_rate: CountrySeries,
    /// Without a child a nephew or niece of this age inherits, from–to years (PE6).
    pub nephew_age: [u32; 2],
    /// From this age a yearly hint to settle the succession while no heir is set.
    pub succession_hint_from: u32,
    /// Inheritance and gift tax rate by country and year.
    pub inheritance_tax: CountrySeries,
    pub provenance: Provenance,
}

impl PersonModel {
    pub fn lifestyle(&self, level: crate::state::Lifestyle) -> &LifestyleLevel {
        &self.lifestyles[level as usize]
    }
}

impl Default for PersonModel {
    /// Without the section: no costs of founding or living, no taxes, no interest.
    fn default() -> Self {
        PersonModel {
            enabled: false,
            start_age: [30, 18, 60],
            children_ages: [22, 45],
            children_max: 0,
            child_chance: 0.0,
            card_age: 25,
            founding: FoundingModel::default(),
            salary_co_owner_max: 2.0,
            loan_max_rate: 0.0,
            loan_max_years: 1,
            lifestyles: [LifestyleLevel::default(); 4],
            default_lifestyle: crate::state::Lifestyle::Middle,
            lifestyle_months: 12,
            income_tax: CountrySeries::default(),
            savings_rate: CountrySeries::default(),
            nephew_age: [25, 40],
            succession_hint_from: 70,
            inheritance_tax: CountrySeries::default(),
            provenance: Provenance::default(),
        }
    }
}

/// Costs of founding a company (PE3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoundingModel {
    /// Share of the capital paid in.
    pub cost_share: f64,
    /// At least so many monthly wages of an academic in the seat's country.
    pub cost_min_months: f64,
    /// Share of the private account the dialog suggests as capital.
    pub capital_suggestion: f64,
}

impl Default for FoundingModel {
    fn default() -> Self {
        Self {
            cost_share: 0.0,
            cost_min_months: 0.0,
            capital_suggestion: 1.0,
        }
    }
}

/// What a level of lifestyle costs and does (PE3).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LifestyleLevel {
    /// Monthly wages of an academic in the home country per month.
    pub cost: f64,
    /// Added to the rate of new bank loans of the controlled companies.
    pub interest: f64,
    /// Share more (or less) that candidates for the board ask.
    pub salary_demand: f64,
    /// Points on every skill of a child's manager card.
    pub education: f64,
    /// Factor on the person's chance of death.
    pub mortality: f64,
}

/// Parameters of the events' effects (`parameter/ereignisse.yaml`, H1).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EventModel {
    /// Working capital the state gives its company, as a share of the seized book value.
    pub working_capital_share: f64,
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
    /// First names of managers (MA1).
    pub first_names: Vec<String>,
    /// Managers' names with the family name first.
    pub surname_first: bool,
    pub places: Vec<String>,
    pub legal_forms: Vec<String>,
    /// Patterns with `{familienname}`, `{ort}`, `{rechtsform}`, `{branche}`.
    pub patterns: Vec<String>,
    /// Word for the business per branch, indexed by `BranchId`.
    pub branch_words: Vec<Option<String>>,
}

/// Parts for the invented names companies give their end products (M42,
/// `data/ki/produktnamen.yaml`).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ProductNaming {
    /// Chance that a further product gets a stem the company already uses in the style.
    pub house_brand: f64,
    /// Real product and brand names: no word of a product name may be one of them.
    pub excluded: Vec<String>,
    pub styles: Vec<NamingStyle>,
    /// Style per goods group, indexed by `GoodsGroupId`.
    pub style_of_group: Vec<Option<usize>>,
}

impl ProductNaming {
    /// The naming style of a product: end products of a goods group with a style.
    pub fn style(&self, catalog: &Catalog, product: ProductId) -> Option<&NamingStyle> {
        let p = catalog.products.get(product);
        if p.kind != ProductKind::EndProduct {
            return None;
        }
        let index = self
            .style_of_group
            .get(p.goods_group.index())
            .copied()
            .flatten()?;
        self.styles.get(index)
    }

    /// Whether a word of `name` is a real product or brand name.
    pub fn is_excluded(&self, name: &str) -> bool {
        name.split_whitespace().any(|word| {
            self.excluded
                .iter()
                .any(|e| crate::product_names::same_name(e, word))
        })
    }
}

/// How the products of some goods groups are named (M42).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NamingStyle {
    pub key: String,
    /// Invented words a name starts with.
    pub stems: Vec<String>,
    pub patterns: Vec<NamePattern>,
    pub numbers: Vec<u32>,
    pub letters: Vec<String>,
    pub additions: Vec<String>,
    /// Generation marks of successor models (B1) in order; empty: no successor models.
    pub successors: Vec<String>,
}

/// A name pattern with `{stamm}`, `{zahl}`, `{buchstabe}` and `{zusatz}`, used in the
/// years from `from` to `until` (both optional).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NamePattern {
    pub text: String,
    pub from: Option<i32>,
    pub until: Option<i32>,
}

impl NamePattern {
    pub fn applies(&self, year: i32) -> bool {
        self.from.is_none_or(|a| year >= a) && self.until.is_none_or(|b| year <= b)
    }
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
    /// ISO codes of the countries merged into this entry (M34), the leading one first;
    /// empty for a single country. Old saves find a merged country under its region.
    pub members: Vec<String>,
    /// The largest cities, the most populous first (W2); may be empty.
    pub cities: Vec<City>,
    pub values: CountryValues,
    pub profile: CountryProfile,
    pub provenance: Provenance,
}

/// A city where a company can have its headquarters (W2).
#[derive(Clone, Debug, PartialEq)]
pub struct City {
    pub key: String,
    /// Inhabitants of the agglomeration today.
    pub population: f64,
    pub location: GeoPoint,
    pub capital: bool,
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
    pub dividends: DividendModel,
}

/// Dividends of all companies (PE4, docs/FORMELN.md).
#[derive(Clone, Debug, PartialEq)]
pub struct DividendModel {
    /// The cash keeps at least so many months of running costs.
    pub reserve_months_min: f64,
    /// Above this share a holding company gets dividends free of tax into its reserves.
    pub group_share: f64,
    /// Share of the profit an aggressive AI company or a risk-loving CEO pays out.
    pub payout_min: f64,
    /// Share of the profit a cautious one pays out.
    pub payout_max: f64,
    /// Below so many months of running costs in cash AI and CEO propose nothing.
    pub cash_months: f64,
}

impl Default for DividendModel {
    fn default() -> Self {
        Self {
            reserve_months_min: 3.0,
            group_share: 0.5,
            payout_min: 0.2,
            payout_max: 0.6,
            cash_months: 4.0,
        }
    }
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
            dividends: DividendModel::default(),
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
    /// A scarce good far below its reference price rises faster: the upward step times
    /// the gap (reference / price), at most this factor (M22).
    pub catch_up_max: f64,
    /// Governments pay at most this multiple of the reference price.
    pub state_price_cap: f64,
    /// Years over which a successor with state demand displaces the state demand of the
    /// product it replaces (M33).
    pub state_displacement_years: f64,
    /// Closed months kept per market for the charts (M24).
    pub history_months: u32,
    /// A competitor's price cut since its last high (or last report) that the round
    /// report names (M24).
    pub price_cut_report: f64,
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
    /// Price islands (C1): traders also bring this share of a market's sales when its
    /// price exceeds landed cost and margin from abroad by `arbitrage_gap`.
    pub arbitrage_gap: f64,
    pub arbitrage_share: f64,
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
            catch_up_max: 20.0,
            state_price_cap: 1.5,
            state_displacement_years: 15.0,
            history_months: 24,
            price_cut_report: 0.1,
            price_level_share: [1.0; 5],
            index_smoothing: 0.1,
            trader_margin: 0.05,
            trader_cover_days: 30.0,
            demand_smoothing_days: 30.0,
            arbitrage_gap: 0.15,
            arbitrage_share: 0.0,
            brand: BrandModel::default(),
        }
    }
}

/// Parameters of production (`data/parameter/produktionsmodell.yaml`, formulas in
/// docs/FORMELN.md).
/// Training of the sites (`produktionsmodell.schulung`, W1, docs/FORMELN.md).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrainingModel {
    /// Daily cost as share of the site's wage bill at the target 1.
    pub cost_share: f64,
    pub gain_per_month: f64,
    pub loss_per_month: f64,
    /// Share of labor hours saved at the full level.
    pub labor_saving: f64,
    /// Quality points at the full level.
    pub quality_points: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProductionModel {
    /// Cost of founding a site (land, buildings) per site type.
    pub site_cost: Vec<(SiteType, Money)>,
    pub building_lifetime_years: f64,
    pub development_lifetime_years: f64,
    /// Remaining share of a deposit's reserve below which its allowed output falls in
    /// proportion (C2); 0: full output to the end.
    pub decline_from: f64,
    /// Training of the sites (W1).
    pub training: TrainingModel,
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
    /// Shut down facilities: share of the maintenance they still cost, days and cost
    /// (share of the investment) of a restart (M22).
    pub mothball_maintenance_share: f64,
    pub restart_days: u32,
    pub restart_cost_share: f64,
    /// Sold facilities: proceeds as share of the book value, at least the scrap value
    /// as share of the investment (M22).
    pub sale_proceeds_share: f64,
    pub scrap_share: f64,
    /// What a new company owns at the start, per start form (Lastenheft §15).
    pub start_setups: Vec<(StartForm, StartSetup)>,
    /// Facility sizes (M36).
    pub sizes: SizeModel,
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
            decline_from: 0.0,
            training: TrainingModel::default(),
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
            mothball_maintenance_share: 0.25,
            restart_days: 30,
            restart_cost_share: 0.02,
            sale_proceeds_share: 0.5,
            scrap_share: 0.03,
            start_setups: Vec::new(),
            sizes: SizeModel::default(),
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
#[derive(Clone, Debug, PartialEq)]
pub struct StateDemand {
    /// Units per year and million USD of GDP.
    pub per_million_gdp: f64,
    /// Multiplier in times of war (effective from stage 4).
    pub war_factor: f64,
    /// Factor over time on the demand per GDP (M39); none means 1.
    pub index: Option<TimeSeries>,
}

impl StateDemand {
    /// Units per year and million USD of GDP on a date.
    pub fn per_million_gdp_at(&self, date: crate::calendar::Date) -> f64 {
        self.per_million_gdp
            * self
                .index
                .as_ref()
                .map_or(1.0, |i| i.value_at(date.year_fraction()))
    }
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
    /// Land per unit in ha where the rule of the plot model does not fit (M35).
    pub area_ha: Option<f64>,
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
    /// Emissions per run (H2): t CO2 and kg of pollutants.
    pub co2_t: f64,
    pub pollutant_kg: f64,
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
