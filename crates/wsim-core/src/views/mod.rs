//! Views for the user interface (Architektur §2.4): plain data derived from the game,
//! with keys instead of display texts and every number already computed, so that the
//! interface shows them without game logic of its own. Amounts are USD as floating
//! point numbers (display only), dates are `YYYY-MM-DD`.

use std::collections::BTreeMap;

mod central;
mod chains;
mod concerns;
mod contracts;
mod controlling;
mod deals;
mod group;
mod hints;
mod logistics;
mod organisation;
mod play;
mod review;
mod stock;
mod strategy;
mod ventures;
pub use central::*;
pub use chains::*;
pub use concerns::*;
pub use contracts::*;
pub use controlling::*;
pub use deals::{
    AreaView, CompaniesView, CompanyDetailView, CompanyRowView, DealObjectView, ForeignSiteView,
    LicenseView, OfferView, OffersView, SiteValueView, companies, company_detail, offers,
};
pub use group::*;
pub use hints::*;
pub use logistics::*;
pub use organisation::*;
pub use play::*;
pub use review::*;
pub use stock::*;
pub use strategy::*;
pub use ventures::*;

use serde::{Deserialize, Serialize};

use crate::EARLIEST_START_YEAR;
use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::site_type_key;
use crate::currency::MoneyDisplay;
use crate::game::{
    Game, MAX_RESEARCH_FACTOR, MAX_START_CAPITAL_USD, MIN_RESEARCH_FACTOR, RoundReport,
};
use crate::ids::{CountryId, Id};
use crate::ledger::{Account, CostType};
use crate::message::{Message, MessageKind, Param};
use crate::money::Money;
use crate::ranking::equity;
use crate::state::{Company, StartForm};

/// Latest start year offered in stage 1 (docs/OFFENE_PUNKTE.md, point 7).
pub const LATEST_START_YEAR_STAGE_1: i32 = 1930;
/// Start capital proposed in the new-game dialog (USD).
pub const DEFAULT_START_CAPITAL_USD: f64 = 100_000.0;

fn usd(m: Money) -> f64 {
    m.to_usd()
}

fn iso(d: Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day())
}

/// Key of a start form as in the data and the texts (`startform.<key>`).
pub fn start_form_key(form: StartForm) -> &'static str {
    match form {
        StartForm::Workshop => "werkstatt",
        StartForm::Trading => "handel",
    }
}

pub fn start_form_from_key(key: &str) -> Option<StartForm> {
    [StartForm::Workshop, StartForm::Trading]
        .into_iter()
        .find(|&f| start_form_key(f) == key)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
    pub default: T,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StartFormOption {
    pub key: String,
    /// What the workshop or office costs from the start capital.
    pub cost_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DifficultyOption {
    pub key: String,
    pub competence: f64,
    pub aggressiveness: f64,
}

/// A choice of how many start-ups there are (SU1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrequencyOption {
    pub key: String,
    /// Start-ups a year with this choice.
    pub per_year: f64,
}

/// What the new-game dialog offers (Lastenheft §15, stage 1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewGameOptions {
    pub start_year: Range<i32>,
    /// Country keys in catalog order; the interface sorts them by name.
    pub countries: Vec<String>,
    pub default_country: String,
    pub start_forms: Vec<StartFormOption>,
    pub start_capital_usd: Range<f64>,
    pub companies: Range<u32>,
    pub difficulties: Vec<DifficultyOption>,
    pub default_difficulty: String,
    pub research_factor: Range<f64>,
    /// How many start-ups there are (SU1); empty without start-ups in the data.
    pub startups: Vec<FrequencyOption>,
    pub default_startups: Option<String>,
    /// How the tariffs change after the data (W3): keys of the choices; empty without
    /// tariffs in the data.
    #[serde(default)]
    pub tariffs: Vec<String>,
    #[serde(default)]
    pub default_tariffs: Option<String>,
}

pub fn new_game_options(catalog: &Catalog) -> NewGameOptions {
    let ai = &catalog.ai_model;
    let start_forms = [StartForm::Workshop, StartForm::Trading]
        .into_iter()
        .filter_map(|f| {
            let setup = catalog.production_model.start_setup(f)?;
            Some(StartFormOption {
                key: start_form_key(f).to_owned(),
                cost_usd: usd(setup.cost(catalog)),
            })
        })
        .collect();
    let default_country = catalog
        .countries
        .id("DEU")
        .or_else(|| catalog.countries.ids().next())
        .map(|c| catalog.countries.key(c).to_owned())
        .unwrap_or_default();
    NewGameOptions {
        start_year: Range {
            min: EARLIEST_START_YEAR,
            max: LATEST_START_YEAR_STAGE_1,
            default: EARLIEST_START_YEAR,
        },
        countries: catalog.countries.keys().to_vec(),
        default_country,
        start_forms,
        start_capital_usd: Range {
            min: 1.0,
            max: MAX_START_CAPITAL_USD,
            default: DEFAULT_START_CAPITAL_USD,
        },
        companies: Range {
            min: 0,
            max: ai.max_companies,
            default: ai.default_companies,
        },
        difficulties: ai
            .difficulties
            .iter()
            .map(|d| DifficultyOption {
                key: d.key.clone(),
                competence: d.competence,
                aggressiveness: d.aggressiveness,
            })
            .collect(),
        default_difficulty: ai
            .difficulties
            .get(ai.default_difficulty)
            .map(|d| d.key.clone())
            .unwrap_or_default(),
        research_factor: Range {
            min: MIN_RESEARCH_FACTOR,
            max: MAX_RESEARCH_FACTOR,
            default: 1.0,
        },
        startups: if catalog.ventures.enabled() {
            catalog
                .ventures
                .frequencies
                .iter()
                .map(|(key, factor)| FrequencyOption {
                    key: key.clone(),
                    per_year: catalog.ventures.per_year * factor,
                })
                .collect()
        } else {
            Vec::new()
        },
        default_startups: catalog
            .ventures
            .enabled()
            .then(|| {
                catalog
                    .ventures
                    .frequencies
                    .get(catalog.ventures.default_frequency)
            })
            .flatten()
            .map(|f| f.0.clone()),
        tariffs: if catalog.tariffs.enabled() {
            catalog
                .tariffs
                .dynamics
                .levels
                .iter()
                .map(|(key, _)| key.clone())
                .collect()
        } else {
            Vec::new()
        },
        default_tariffs: catalog
            .tariffs
            .enabled()
            .then(|| {
                catalog
                    .tariffs
                    .dynamics
                    .levels
                    .get(catalog.tariffs.dynamics.default_level)
            })
            .flatten()
            .map(|l| l.0.clone()),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FacilityView {
    pub facility: String,
    pub count: u32,
    pub recipe: Option<String>,
    pub product: Option<String>,
    pub utilization: f64,
    /// Day the construction is finished.
    pub ready: String,
    /// Units made on the last day.
    pub output_per_day: f64,
    /// Shut down (M22).
    #[serde(default)]
    pub mothballed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StockView {
    pub product: String,
    pub quantity: f64,
    pub value_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteView {
    pub index: u32,
    pub country: String,
    pub kind: String,
    pub deposit: Option<String>,
    pub workers: f64,
    pub facilities: Vec<FacilityView>,
    pub stock: Vec<StockView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompanyView {
    pub name: String,
    pub headquarters: String,
    pub cash_usd: f64,
    pub equity_usd: f64,
    pub loans_usd: f64,
    pub total_assets_usd: f64,
    /// Result of the running year so far.
    pub result_year_usd: f64,
    /// Result of the last closed month (`None` before the first month ends).
    pub result_last_month_usd: Option<f64>,
    pub revenue_last_month_usd: Option<f64>,
    pub sites: Vec<SiteView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompetitorView {
    pub name: String,
    pub headquarters: String,
    pub equity_usd: f64,
    /// A historical company.
    pub real: bool,
}

/// The main screen: the own company and the competition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Overview {
    pub date: String,
    pub game_over: bool,
    pub start_year: i32,
    pub market_scale: f64,
    pub company: CompanyView,
    pub competitors_active: u32,
    pub competitors_bankrupt: u32,
    /// The largest active AI companies by equity.
    pub competitors: Vec<CompetitorView>,
    /// What needs the player's attention now (M18).
    pub hints: Vec<HintView>,
    /// Closed months of the player's books, oldest first.
    pub history: Vec<MonthView>,
    /// Ways to show amounts (M21); `None` without currency data.
    pub money: Option<MoneyOptions>,
    /// Goals after the introduction, in data order (M23).
    #[serde(default)]
    pub milestones: Vec<MilestoneView>,
    /// The player's places among all active companies (M29); `None` without others.
    #[serde(default)]
    pub rank: Option<RankView>,
    /// Concerns of the player's positions waiting for an answer (MA2).
    #[serde(default)]
    pub concerns_open: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RankView {
    pub now: StandingView,
    /// The places a year before (start of the same month), if recorded.
    pub year_before: Option<StandingView>,
}

/// Places by equity and by revenue of the last twelve closed months.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StandingView {
    pub date: String,
    pub equity: u32,
    /// `None` without revenue in the last twelve months.
    pub revenue: Option<u32>,
    /// Active companies, the player included.
    pub companies: u32,
}

impl From<crate::ranking::Standing> for StandingView {
    fn from(s: crate::ranking::Standing) -> Self {
        Self {
            date: iso(s.date),
            equity: s.equity,
            revenue: s.revenue,
            companies: s.companies,
        }
    }
}

fn rank(game: &Game) -> Option<RankView> {
    let state = game.state();
    let now = crate::ranking::standing(state, state.player);
    (now.companies >= 2).then(|| RankView {
        now: now.into(),
        year_before: crate::ranking::year_before(state, state.date).map(Into::into),
    })
}

/// A goal of the player (texts `etappe.<key>` and `etappe.<key>.hinweis`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MilestoneView {
    pub key: String,
    /// Day it was reached.
    pub reached: Option<String>,
    /// Way there for measurable goals: current value and target.
    pub progress: Option<ProgressView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProgressView {
    pub current: f64,
    pub target: f64,
    /// `anzahl`, `anteil` or `geld` (USD).
    pub unit: String,
}

fn milestones(game: &Game) -> Vec<MilestoneView> {
    use crate::catalog::MilestoneCondition as C;
    let state = game.state();
    let catalog = game.catalog();
    catalog
        .milestones
        .iter()
        .map(|(id, m)| {
            let unit = match m.condition {
                C::MarketLeader(_) => "anteil",
                C::Equity(_) => "geld",
                _ => "anzahl",
            };
            MilestoneView {
                key: catalog.milestones.key(id).to_owned(),
                reached: state.milestones.get(id).map(iso),
                progress: crate::milestones::progress(state, m.condition, state.date).map(|p| {
                    ProgressView {
                        current: p.current,
                        target: p.target,
                        unit: unit.to_owned(),
                    }
                }),
            }
        })
        .collect()
}

/// Ways to show amounts (Lastenheft §3.6, §18.2): the headquarters' currency or the US
/// dollar, each at the purchasing power of the base year or at the prices of the game
/// date. The factor turns the game's dollars into the shown units.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MoneyOptions {
    pub home_base: MoneyDisplay,
    pub home_then: MoneyDisplay,
    pub lead_base: MoneyDisplay,
    pub lead_then: MoneyDisplay,
    pub base_year: i32,
}

fn money_options(game: &Game) -> Option<MoneyOptions> {
    let state = game.state();
    let m = &game.catalog().currencies;
    let home = state.companies[state.player.index()].headquarters;
    let t = state.date.year_fraction();
    Some(MoneyOptions {
        home_base: m.at_base(home)?,
        home_then: m.at_time(home, t)?,
        lead_base: m.lead(None)?,
        lead_then: m.lead(Some(t))?,
        base_year: m.base_year,
    })
}

fn company_view(game: &Game) -> CompanyView {
    let state = game.state();
    let catalog = game.catalog();
    let player = state.player;
    let company = &state.companies[player.index()];
    let ledger = &company.ledger;
    let last_month = ledger.months.last();
    let sites = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.owner == player)
        .map(|(i, s)| SiteView {
            index: u32::try_from(i).unwrap_or(u32::MAX),
            country: catalog.countries.key(s.country).to_owned(),
            kind: site_type_key(s.kind),
            deposit: s.deposit.map(|d| catalog.deposits.key(d).to_owned()),
            workers: s.workforce.values().sum(),
            facilities: s
                .slots
                .iter()
                .map(|sl| {
                    let recipe = sl.recipe.map(|r| catalog.recipes.get(r));
                    FacilityView {
                        facility: catalog.facilities.key(sl.facility).to_owned(),
                        count: sl.count,
                        recipe: sl.recipe.map(|r| catalog.recipes.key(r).to_owned()),
                        product: recipe.map(|r| catalog.products.key(r.product).to_owned()),
                        utilization: sl.utilization,
                        ready: iso(sl.ready),
                        output_per_day: recipe.map_or(0.0, |r| sl.last_runs * r.output),
                        mothballed: sl.mothballed(),
                    }
                })
                .collect(),
            stock: s
                .inventory
                .iter()
                .filter(|(_, st)| st.quantity > 1e-9)
                .map(|(p, st)| StockView {
                    product: catalog.products.key(*p).to_owned(),
                    quantity: st.quantity,
                    value_usd: usd(st.value),
                })
                .collect(),
        })
        .collect();
    CompanyView {
        name: company.name.clone(),
        headquarters: catalog.countries.key(company.headquarters).to_owned(),
        cash_usd: usd(ledger.cash()),
        equity_usd: usd(equity(company)),
        loans_usd: usd(ledger.balance(Account::Loans)),
        total_assets_usd: usd(ledger.total_assets()),
        result_year_usd: usd(ledger.year.total()),
        result_last_month_usd: last_month.map(|m| usd(m.total())),
        revenue_last_month_usd: last_month.map(|m| {
            usd(m
                .by_type
                .get(&crate::ledger::CostType::Revenue)
                .copied()
                .unwrap_or(Money::ZERO))
        }),
        sites,
    }
}

pub fn overview(game: &Game) -> Overview {
    let state = game.state();
    let catalog = game.catalog();
    // The player's subsidiaries are no competitors (W6).
    let ai: Vec<&Company> = state
        .companies
        .iter()
        .filter(|c| c.ai.is_some() && c.subsidiary_of.is_none())
        .collect();
    let mut active: Vec<&&Company> = ai.iter().filter(|c| !c.bankrupt).collect();
    active.sort_by(|a, b| equity(b).cmp(&equity(a)).then(a.name.cmp(&b.name)));
    Overview {
        date: iso(state.date),
        game_over: state.game_over,
        start_year: state.settings.start_year,
        market_scale: state.settings.market_scale,
        company: company_view(game),
        competitors_active: u32::try_from(active.len()).unwrap_or(u32::MAX),
        competitors_bankrupt: u32::try_from(ai.len() - active.len()).unwrap_or(u32::MAX),
        competitors: active
            .iter()
            .take(10)
            .map(|c| CompetitorView {
                name: c.name.clone(),
                headquarters: catalog.countries.key(c.headquarters).to_owned(),
                equity_usd: usd(equity(c)),
                real: c.ai.as_ref().is_some_and(|a| a.real.is_some()),
            })
            .collect(),
        hints: hints(game),
        history: history(&state.companies[state.player.index()].ledger),
        money: money_options(game),
        milestones: milestones(game),
        rank: rank(game),
        concerns_open: u32::try_from(
            state
                .concerns
                .iter()
                .filter(|c| {
                    c.company == state.player && c.status == crate::state::ConcernStatus::Open
                })
                .count(),
        )
        .unwrap_or(u32::MAX),
    }
}

/// A message parameter with its type, so the interface can format it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ParamView {
    Text(String),
    Integer(i64),
    Number(f64),
    Money(f64),
    Date(String),
    /// Country key, shown with its name.
    Country(String),
    /// Another text key.
    TextKey(String),
    /// Country keys, shown as a list of names.
    Countries(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MessageView {
    /// `info`, `success`, `warning`, `crisis`, `world_event`, `stock`, `error`.
    pub kind: String,
    /// Section of the round report (Lastenheft §13.2): `welt`, `wettbewerb`, `warnung`,
    /// `forschung` or `allgemein`.
    pub group: String,
    pub key: String,
    pub params: BTreeMap<String, ParamView>,
    /// View the message leads to when clicked (Lastenheft §13.3).
    pub target: Option<String>,
}

pub fn message_view(message: &Message) -> MessageView {
    let kind = match message.kind {
        MessageKind::Info => "info",
        MessageKind::Success => "success",
        MessageKind::Warning => "warning",
        MessageKind::Crisis => "crisis",
        MessageKind::WorldEvent => "world_event",
        MessageKind::Stock => "stock",
        MessageKind::Error => "error",
    };
    let params = message
        .params
        .iter()
        .map(|(name, p)| {
            let v = match p {
                Param::Text(t) => ParamView::Text(t.clone()),
                Param::Integer(i) => ParamView::Integer(*i),
                Param::Number(n) => ParamView::Number(*n),
                Param::Money(m) => ParamView::Money(usd(*m)),
                Param::Date(d) => ParamView::Date(iso(*d)),
                Param::Country(c) => ParamView::Country(c.clone()),
                Param::TextKey(k) => ParamView::TextKey(k.clone()),
                Param::Countries(c) => ParamView::Countries(c.clone()),
            };
            (name.clone(), v)
        })
        .collect();
    // The view where the player can act on the message.
    let target = if message.key == crate::message::keys::INPUT_MISSING {
        Some("produktion")
    } else if [
        crate::message::keys::MILESTONE,
        crate::message::keys::RANK_YEAR_END,
        crate::message::keys::RANK_YEAR_END_COMPARED,
    ]
    .contains(&message.key.as_str())
    {
        Some("uebersicht")
    } else if message.key.starts_with("meldung.angebot.")
        || message.key == crate::message::keys::AI_BUYS_SITE
    {
        Some("wettbewerb")
    } else if [
        crate::message::keys::AI_NEW_SELLER,
        crate::message::keys::AI_SELLER_GONE,
        crate::message::keys::AI_PRICE_CUT,
        crate::message::keys::AI_NEW_SELLER_NAMED,
        crate::message::keys::AI_SELLER_GONE_NAMED,
        crate::message::keys::AI_PRICE_CUT_NAMED,
    ]
    .contains(&message.key.as_str())
        || message.key.starts_with("meldung.vertrag.")
        || message.key.starts_with("meldung.logistik.")
    {
        Some("markt")
    } else if message.key.starts_with("meldung.forschung") {
        Some("forschung")
    } else if message.key.starts_with("meldung.boerse.") {
        Some("finanzen")
    } else if message.key.starts_with("meldung.anliegen.")
        || message.key.starts_with("meldung.manager.")
        || message.key.starts_with("meldung.tochter.")
    {
        Some("organisation")
    } else if matches!(
        message.kind,
        MessageKind::Warning | MessageKind::Crisis | MessageKind::Success
    ) {
        Some("finanzen")
    } else {
        None
    }
    .map(str::to_owned);
    let group = if message.kind == MessageKind::WorldEvent {
        "welt"
    } else if message.key == crate::message::keys::MILESTONE {
        "erfolg"
    } else if message.key.starts_with("meldung.ki.")
        || message.key.starts_with("meldung.rang")
        || message.key.starts_with("meldung.angebot.")
        || message.key == crate::message::keys::COMPANY_INSOLVENT
    {
        "wettbewerb"
    } else if matches!(message.kind, MessageKind::Warning | MessageKind::Crisis) {
        "warnung"
    } else if message.key.starts_with("meldung.forschung") {
        "forschung"
    } else {
        "allgemein"
    };
    MessageView {
        kind: kind.to_owned(),
        group: group.to_owned(),
        key: message.key.clone(),
        params,
        target,
    }
}

/// The player's books before a round, to compare with the books after it.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub cash: Money,
    pub equity: Money,
    year: BTreeMap<CostType, Money>,
    /// Revenue and result by product of the running year.
    products: BTreeMap<crate::ids::ProductId, (Money, Money)>,
    years_closed: usize,
}

/// Revenue and result (gross margin) by product of a period, from its cost centers.
fn product_totals(
    period: &crate::ledger::PeriodResult,
) -> BTreeMap<crate::ids::ProductId, (Money, Money)> {
    let mut totals: BTreeMap<crate::ids::ProductId, (Money, Money)> = BTreeMap::new();
    for (center, by_type) in &period.by_center {
        let Some(p) = center.product else { continue };
        let e = totals.entry(p).or_insert((Money::ZERO, Money::ZERO));
        e.0 += by_type
            .get(&CostType::Revenue)
            .copied()
            .unwrap_or(Money::ZERO);
        e.1 += by_type.values().copied().sum::<Money>();
    }
    totals
}

pub fn snapshot(game: &Game) -> Snapshot {
    let state = game.state();
    let company = &state.companies[state.player.index()];
    Snapshot {
        cash: company.ledger.cash(),
        equity: equity(company),
        year: company.ledger.year.by_type.clone(),
        products: product_totals(&company.ledger.year),
        years_closed: company.ledger.years.len(),
    }
}

/// Revenue, costs and result of a period, with the amounts by cost type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeriodView {
    pub revenue_usd: f64,
    /// Costs as a positive amount (everything but revenue and inventory change).
    pub costs_usd: f64,
    /// Goods made but not yet sold, at their production cost (positive: more in stock).
    /// Revenue − costs + inventory change = result.
    pub inventory_change_usd: f64,
    pub result_usd: f64,
    /// (text key of the cost type, amount; costs negative)
    pub lines: Vec<(String, f64)>,
}

fn period(game: &Game, before: &Snapshot) -> PeriodView {
    let state = game.state();
    let ledger = &state.companies[state.player.index()].ledger;
    let mut sums: BTreeMap<CostType, Money> = BTreeMap::new();
    let closed = ledger.years.get(before.years_closed..).unwrap_or_default();
    for year in closed
        .iter()
        .map(|y| &y.by_type)
        .chain([&ledger.year.by_type])
    {
        for (&t, &m) in year {
            *sums.entry(t).or_insert(Money::ZERO) += m;
        }
    }
    // The year running at the start of the round already held these amounts.
    for (&t, &m) in &before.year {
        *sums.entry(t).or_insert(Money::ZERO) -= m;
    }
    let get = |t: CostType| sums.get(&t).copied().unwrap_or(Money::ZERO);
    let result: Money = sums.values().copied().fold(Money::ZERO, |a, b| a + b);
    let revenue = get(CostType::Revenue);
    let costs = result - revenue - get(CostType::InventoryChange);
    PeriodView {
        revenue_usd: usd(revenue),
        costs_usd: -usd(costs),
        inventory_change_usd: usd(get(CostType::InventoryChange)),
        result_usd: usd(result),
        lines: sums
            .iter()
            .filter(|(_, m)| **m != Money::ZERO)
            .map(|(t, m)| (t.text_key().to_owned(), usd(*m)))
            .collect(),
    }
}

/// A research project of the player.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResearchView {
    pub technology: String,
    pub points: f64,
    pub needed: f64,
}

fn research_projects(game: &Game) -> Vec<ResearchView> {
    let state = game.state();
    let catalog = game.catalog();
    let company = &state.companies[state.player.index()];
    let mut projects: Vec<ResearchView> = Vec::new();
    for s in state.sites.iter().filter(|s| s.owner == state.player) {
        let Some(t) = s.research else { continue };
        if projects
            .iter()
            .any(|p| p.technology == catalog.technologies.key(t))
        {
            continue;
        }
        let Some(effort) = crate::research::effort(catalog, state, t, state.date) else {
            continue;
        };
        projects.push(ResearchView {
            technology: catalog.technologies.key(t).to_owned(),
            points: company.research.get(&t).copied().unwrap_or(0.0),
            needed: effort.points,
        });
    }
    projects
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoundReportView {
    pub from: String,
    /// Last day of the round.
    pub to: String,
    pub days: u32,
    pub cash_before_usd: f64,
    pub cash_after_usd: f64,
    pub equity_change_usd: f64,
    /// Result of this round.
    pub period: PeriodView,
    /// Result of the round before, if the session knows it.
    pub previous: Option<PeriodView>,
    pub research: Vec<ResearchView>,
    pub messages: Vec<MessageView>,
    pub game_over: bool,
    /// What the round brought by product: revenue and gross margin (M18).
    pub products: Vec<ProductResult>,
    /// What needs the player's attention after the round.
    pub hints: Vec<HintView>,
    /// Rounds the report covers (M26).
    #[serde(default = "one")]
    pub rounds: u32,
    /// Why several rounds stopped (M26): `jahresende`, `warnung`, `weltereignis`,
    /// `ein_jahr`, `spielende`; `None` for a single round.
    #[serde(default)]
    pub stop: Option<String>,
}

fn one() -> u32 {
    1
}

/// Revenue and gross margin by product in the round: the years closed during the
/// round (only the last one keeps its details) and the running year, less the running
/// year at its start.
fn round_products(game: &Game, before: &Snapshot) -> Vec<ProductResult> {
    let state = game.state();
    let catalog = game.catalog();
    let ledger = &state.companies[state.player.index()].ledger;
    let mut sums: BTreeMap<crate::ids::ProductId, (Money, Money)> = BTreeMap::new();
    let mut add = |totals: BTreeMap<crate::ids::ProductId, (Money, Money)>, sign: i64| {
        for (p, (revenue, result)) in totals {
            let e = sums.entry(p).or_insert((Money::ZERO, Money::ZERO));
            if sign > 0 {
                e.0 += revenue;
                e.1 += result;
            } else {
                e.0 -= revenue;
                e.1 -= result;
            }
        }
    };
    for year in ledger.years.get(before.years_closed..).unwrap_or_default() {
        add(product_totals(year), 1);
    }
    add(product_totals(&ledger.year), 1);
    add(before.products.clone(), -1);
    sums.into_iter()
        .filter(|(_, (revenue, result))| *revenue != Money::ZERO || *result != Money::ZERO)
        .map(|(p, (revenue, result))| ProductResult {
            product: catalog.products.key(p).to_owned(),
            revenue_usd: usd(revenue),
            margin_usd: usd(result),
        })
        .collect()
}

pub fn round_report(game: &Game, report: &RoundReport, before: &Snapshot) -> RoundReportView {
    let after = snapshot(game);
    RoundReportView {
        from: iso(report.from),
        to: iso(report.to.add_days(-1)),
        days: report.days,
        cash_before_usd: usd(before.cash),
        cash_after_usd: usd(after.cash),
        equity_change_usd: usd(after.equity - before.equity),
        period: period(game, before),
        previous: None,
        research: research_projects(game),
        messages: report.messages.iter().map(message_view).collect(),
        game_over: game.is_over(),
        products: round_products(game, before),
        hints: hints(game),
        rounds: 1,
        stop: None,
    }
}

/// A country on the world map with the values of its layers (Lastenheft §3.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapCountry {
    pub key: String,
    pub lat: f64,
    pub lon: f64,
    pub population: f64,
    pub gdp_per_capita_usd: f64,
    /// Hourly wage of the unskilled (first labor group).
    pub wage_usd: f64,
    pub development: f64,
    pub grid_share: f64,
    pub own_sites: u32,
    pub other_sites: u32,
    /// Average import tariff (share of the value, W3).
    #[serde(default)]
    pub tariff: f64,
}

/// A deposit on the world map, shown at its country's capital.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapDeposit {
    pub key: String,
    pub country: String,
    pub resource: String,
    /// Yearly output in game quantities (market scale applied).
    pub max_output_per_year: f64,
    pub concessions: u32,
    pub free_concessions: u32,
    /// Discovered later than the current year.
    pub undiscovered: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldMap {
    pub date: String,
    pub countries: Vec<MapCountry>,
    pub deposits: Vec<MapDeposit>,
    /// Raw materials with deposits, for the resource filter.
    pub resources: Vec<String>,
    /// Traded products (without energy), for the layer of sales chances.
    pub products: Vec<String>,
}

pub fn world_map(game: &Game) -> WorldMap {
    let state = game.state();
    let catalog = game.catalog();
    let mut own = vec![0u32; catalog.countries.len()];
    let mut other = vec![0u32; catalog.countries.len()];
    for s in &state.sites {
        if state.companies[s.owner.index()].bankrupt {
            continue;
        }
        let n = if s.owner == state.player {
            &mut own
        } else {
            &mut other
        };
        n[s.country.index()] += 1;
    }
    let countries = catalog
        .countries
        .iter()
        .map(|(id, c)| {
            let v = state.countries.get(id);
            MapCountry {
                key: catalog.countries.key(id).to_owned(),
                lat: c.capital.lat,
                lon: c.capital.lon,
                population: v.population,
                gdp_per_capita_usd: v.gdp_per_capita_usd,
                wage_usd: v.hourly_wage_usd.first().copied().unwrap_or(0.0),
                development: v.development,
                grid_share: v.grid_share,
                own_sites: own[id.index()],
                other_sites: other[id.index()],
                tariff: state.tariffs.average(id),
            }
        })
        .collect();
    let year = state.date.year();
    let mut resources: Vec<String> = Vec::new();
    let deposits = catalog
        .deposits
        .iter()
        .map(|(id, d)| {
            let resource = catalog.products.key(d.resource).to_owned();
            if !resources.contains(&resource) {
                resources.push(resource.clone());
            }
            let ds = state.deposits.get(id);
            MapDeposit {
                key: catalog.deposits.key(id).to_owned(),
                country: catalog.countries.key(d.country).to_owned(),
                resource,
                max_output_per_year: catalog.max_output(id, state.date.year())
                    * state.settings.market_scale,
                concessions: u32::try_from(ds.concessions.len()).unwrap_or(u32::MAX),
                free_concessions: u32::try_from(
                    ds.concessions.iter().filter(|c| c.site.is_none()).count(),
                )
                .unwrap_or(u32::MAX),
                undiscovered: d.discovered.is_some_and(|y| y > year),
            }
        })
        .collect();
    WorldMap {
        date: iso(state.date),
        countries,
        deposits,
        resources,
        products: catalog
            .products
            .iter()
            .filter(|(_, p)| p.kind != crate::catalog::ProductKind::Energy)
            .map(|(id, _)| catalog.products.key(id).to_owned())
            .collect(),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LaborView {
    /// Labor group key, e.g. `fachkraft.metall`.
    pub group: String,
    pub persons: f64,
    /// Workers the companies can hire in the game (market scale applied).
    pub available: f64,
    pub wage_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CountryCompany {
    pub name: String,
    pub sites: u32,
    pub own: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CountryMarket {
    pub product: String,
    pub price_usd: f64,
    pub demand_last_month: f64,
    pub sold_last_month: f64,
}

/// Everything about one country (Lastenheft §3.2, §14.1 Länderdetail).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CountryDetail {
    pub key: String,
    /// Countries merged into this region (ISO codes, texts `teilland.<ISO>`; M34),
    /// empty for a single country.
    pub members: Vec<String>,
    pub date: String,
    pub population: f64,
    pub gdp_per_capita_usd: f64,
    pub price_level: f64,
    pub gini: f64,
    pub income_quintiles_usd: [f64; 5],
    pub labor_force: f64,
    pub labor: Vec<LaborView>,
    pub electricity_price_usd_mwh: f64,
    pub grid_share: f64,
    pub corporate_tax: f64,
    pub dividend_tax: f64,
    pub development: f64,
    /// Rail, road, port, air (0–1).
    pub infrastructure: [f64; 4],
    pub stability: f64,
    pub deposits: Vec<MapDeposit>,
    /// Companies with sites in the country, most sites first.
    pub companies: Vec<CountryCompany>,
    /// Markets with demand or sales, largest turnover first.
    pub markets: Vec<CountryMarket>,
    /// The country's currencies, oldest first (M21; empty without currency data).
    pub currencies: Vec<CurrencyPeriodView>,
    /// Units of the current currency per US dollar of the game date; `None` for the
    /// lead currency itself or without currency data.
    pub currency_per_usd: Option<f64>,
    /// Commercial land and its free plots (M35); `None` without plots.
    #[serde(default)]
    pub land: Option<LandView>,
    /// Import tariffs, trade zones and embargoes (W3); `None` without tariffs.
    #[serde(default)]
    pub tariffs: Option<TariffView>,
}

/// Import tariffs of a country (W3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TariffView {
    /// Average import tariff before goods group and zones (share of the value).
    pub average: f64,
    /// Tariff per goods group (keys, texts `warengruppe.<key>`), highest first.
    pub groups: Vec<(String, f64)>,
    /// Trade zones the country belongs to (texts `zoll.zone.<key>`).
    pub zones: Vec<String>,
    /// Countries it has an embargo with.
    pub embargoes: Vec<String>,
}

/// Commercial land of a country (M35).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LandView {
    pub area_ha: f64,
    pub occupied_ha: f64,
    /// Land price per ha by location key (`stadt`, `hafen`, `land`).
    pub price_per_ha_usd: Vec<(String, f64)>,
    /// Keys of the size classes, the smallest first (texts `grundstuecksklasse.<key>`).
    #[serde(default)]
    pub classes: Vec<String>,
    /// Free plots, by location (city, port, country side), the largest first.
    pub free: Vec<PlotView>,
}

/// A free plot (M35).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlotView {
    /// For `FoundSiteOnPlot`.
    pub id: u32,
    /// Key of the location (text `lage.<key>`).
    pub location: String,
    /// Key of the size class (text `grundstuecksklasse.<key>`).
    pub class: String,
    pub area_ha: f64,
    /// Price when bought today.
    pub value_usd: f64,
    pub rent_usd_year: f64,
}

/// The commercial land of a country with its free plots.
pub fn land_view(game: &Game, country: CountryId) -> Option<LandView> {
    let (state, catalog) = (game.state(), game.catalog());
    let m = &catalog.plot_model;
    if !m.enabled() {
        return None;
    }
    let (area_ha, occupied_ha) = crate::plots::land(state, country);
    let mut free: Vec<PlotView> = state
        .plots
        .iter()
        .enumerate()
        .filter(|(_, p)| p.country == country && p.site.is_none())
        .map(|(i, p)| {
            let id = crate::state::PlotId(u32::try_from(i).unwrap_or(u32::MAX));
            let value = usd(crate::plots::value(catalog, state, id));
            PlotView {
                id: id.0,
                location: p.location.key().to_owned(),
                class: m
                    .classes
                    .get(usize::from(p.class))
                    .map_or_else(String::new, |c| c.key.clone()),
                area_ha: p.area_ha,
                value_usd: value,
                rent_usd_year: value * m.rent_share,
            }
        })
        .collect();
    let order = |key: &str| {
        crate::catalog::Location::ALL
            .iter()
            .position(|l| l.key() == key)
            .unwrap_or(0)
    };
    free.sort_by(|a, b| {
        order(&a.location)
            .cmp(&order(&b.location))
            .then(b.area_ha.total_cmp(&a.area_ha))
            .then(a.id.cmp(&b.id))
    });
    Some(LandView {
        area_ha,
        occupied_ha,
        price_per_ha_usd: crate::catalog::Location::ALL
            .iter()
            .map(|&l| {
                (
                    l.key().to_owned(),
                    usd(crate::plots::price_per_ha(catalog, state, country, l)),
                )
            })
            .collect(),
        classes: m.classes.iter().map(|c| c.key.clone()).collect(),
        free,
    })
}

/// A currency of a country from a month on (M21).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CurrencyPeriodView {
    /// First month of use, "1923-12".
    pub from: String,
    pub currency: String,
    pub symbol: String,
    /// In use at the game date.
    pub current: bool,
}

/// The currencies of a country and the rate of the one in use at the game date.
fn country_currencies(game: &Game, id: CountryId) -> (Vec<CurrencyPeriodView>, Option<f64>) {
    let m = &game.catalog().currencies;
    let t = game.state().date.year_fraction();
    let periods = m.periods_of(id);
    // As in `currency_at`: before the first period the first one applies.
    let now = periods.partition_point(|p| p.from <= t).saturating_sub(1);
    let views = periods
        .iter()
        .enumerate()
        .map(|(i, p)| {
            // Periods start on the first day of a month: year + (month − 1)/12.
            let year = p.from.floor();
            let month = (p.from - year) * 12.0 + 1.0;
            let c = p.currency;
            CurrencyPeriodView {
                from: format!("{year:04.0}-{month:02.0}"),
                currency: m.currencies[c].key.clone(),
                symbol: m.currencies[c].symbol.clone(),
                current: i == now,
            }
        })
        .collect();
    let per_usd = m
        .currency_at(id, t)
        .filter(|&c| c != m.lead)
        .map(|c| m.rate(c, t));
    (views, per_usd)
}

pub fn country_detail(game: &Game, key: &str) -> Option<CountryDetail> {
    let state = game.state();
    let catalog = game.catalog();
    let id = catalog.countries.id(key)?;
    let v = state.countries.get(id);
    let labor = catalog
        .labor_groups
        .iter()
        .map(|(g, _)| LaborView {
            group: catalog.labor_groups.key(g).to_owned(),
            persons: v.labor_pool.get(g.index()).copied().unwrap_or(0.0),
            available: v.labor_available.get(g.index()).copied().unwrap_or(0.0),
            wage_usd: v.hourly_wage_usd.get(g.index()).copied().unwrap_or(0.0),
        })
        .collect();
    let mut sites: BTreeMap<usize, u32> = BTreeMap::new();
    for s in state.sites.iter().filter(|s| s.country == id) {
        if !state.companies[s.owner.index()].bankrupt {
            *sites.entry(s.owner.index()).or_default() += 1;
        }
    }
    let mut companies: Vec<CountryCompany> = sites
        .into_iter()
        .map(|(c, n)| CountryCompany {
            name: state.companies[c].name.clone(),
            sites: n,
            own: c == state.player.index(),
        })
        .collect();
    companies.sort_by(|a, b| {
        b.own
            .cmp(&a.own)
            .then(b.sites.cmp(&a.sites))
            .then(a.name.cmp(&b.name))
    });
    let mut markets: Vec<CountryMarket> = catalog
        .products
        .ids()
        .filter_map(|p| {
            let m = state.markets.get(p).get(id);
            let t = &m.last_month;
            (t.demand > 0.0 || t.sold > 0.0).then(|| CountryMarket {
                product: catalog.products.key(p).to_owned(),
                price_usd: usd(crate::market::market_price(catalog, state, id, p)),
                demand_last_month: t.demand,
                sold_last_month: t.sold,
            })
        })
        .collect();
    markets.sort_by(|a, b| {
        (b.sold_last_month * b.price_usd).total_cmp(&(a.sold_last_month * a.price_usd))
    });
    let map = world_map(game);
    let (currencies, currency_per_usd) = country_currencies(game, id);
    let land = land_view(game, id);
    let year = state.date.year();
    let tariffs = catalog.tariffs.enabled().then(|| {
        let average = state.tariffs.average(id);
        let mut groups: Vec<(String, f64)> = catalog
            .goods_groups
            .ids()
            .map(|g| {
                let factor = catalog
                    .tariffs
                    .groups
                    .get(g.index())
                    .copied()
                    .unwrap_or(1.0);
                (catalog.goods_groups.key(g).to_owned(), average * factor)
            })
            .collect();
        groups.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        TariffView {
            average,
            groups,
            zones: crate::tariffs::zones_of(catalog, id, year)
                .into_iter()
                .map(str::to_owned)
                .collect(),
            embargoes: crate::tariffs::embargoes_of(catalog, id, year)
                .into_iter()
                .map(|c| catalog.countries.key(c).to_owned())
                .collect(),
        }
    });
    Some(CountryDetail {
        key: key.to_owned(),
        members: catalog.countries.get(id).members.clone(),
        date: iso(state.date),
        population: v.population,
        gdp_per_capita_usd: v.gdp_per_capita_usd,
        price_level: v.price_level,
        gini: v.gini,
        income_quintiles_usd: v.income_quintiles_usd,
        labor_force: v.labor_force,
        labor,
        electricity_price_usd_mwh: v.electricity_price_usd_mwh,
        grid_share: v.grid_share,
        corporate_tax: v.corporate_tax,
        dividend_tax: v.dividend_tax,
        development: v.development,
        infrastructure: [
            v.infrastructure.rail,
            v.infrastructure.road,
            v.infrastructure.port,
            v.infrastructure.air,
        ],
        stability: v.stability,
        deposits: map
            .deposits
            .into_iter()
            .filter(|d| d.country == key)
            .collect(),
        companies,
        markets,
        currencies,
        currency_per_usd,
        land,
        tariffs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::RoundLength;
    use crate::catalog::test_support;
    use crate::state::{AiSettings, GameSettings};
    use std::sync::Arc;

    fn game() -> Game {
        game_with(1_000_000.0)
    }

    fn game_with(capital_usd: f64) -> Game {
        let catalog = Arc::new(test_support::production());
        let settings = GameSettings {
            seed: 3,
            start_year: 1900,
            start_country: catalog.countries.ids().next().expect("one country"),
            start_capital: Money::from_usd(capital_usd).expect("valid"),
            start_form: StartForm::Workshop,
            company_name: "Test AG".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: AiSettings::default(),
            ventures: 1.0,
            tariff_dynamics: 1.0,
        };
        Game::new(catalog, settings).expect("valid settings")
    }

    #[test]
    fn the_rank_needs_competitors_and_compares_with_a_year_before() {
        let mut g = game();
        assert_eq!(overview(&g).rank, None);
        // A richer rival with the player's books.
        let state = g.state_mut();
        let mut rival = state.companies[0].clone();
        "Rivale AG".clone_into(&mut rival.name);
        rival.ledger.transfer(
            Account::Cash,
            Account::Equity,
            Money::from_usd(1.0).expect("valid"),
        );
        state.companies.push(rival);
        let rank = overview(&g).rank.expect("rank");
        assert_eq!(
            (rank.now.equity, rank.now.revenue, rank.now.companies),
            (2, None, 2)
        );
        assert_eq!(rank.year_before, None);
        // A year later the start is the year before.
        for _ in 0..12 {
            g.advance(RoundLength::Month, |_| {});
        }
        let rank = overview(&g).rank.expect("rank");
        let before = rank.year_before.expect("a year before");
        assert_eq!(before.date, "1900-01-01");
    }

    #[test]
    fn overview_shows_the_own_company() {
        let g = game();
        let o = overview(&g);
        assert_eq!(o.date, "1900-01-01");
        assert_eq!(o.company.name, "Test AG");
        assert!((o.company.cash_usd + 0.0).is_finite());
        assert_eq!(o.competitors_active, 0);
        let json = serde_json::to_string(&o).expect("serializable");
        assert!(json.contains("\"cash_usd\""));
    }

    /// The test game with currencies: a krone everywhere, from November 1923 a new one.
    fn game_with_currencies() -> Game {
        use crate::catalog::Provenance;
        use crate::currency::{Currency, CurrencyModel, Period, Rate};
        let mut catalog = test_support::production();
        let currency = |key: &str, points: Vec<(f64, f64)>| Currency {
            key: key.to_owned(),
            symbol: key.to_uppercase(),
            rate: Rate::Points(points),
            provenance: Provenance::default(),
        };
        catalog.currencies = CurrencyModel {
            currencies: vec![
                currency("usd", vec![(1900.5, 1.0)]),
                currency("krone", vec![(1900.5, 4.0), (2026.5, 9.0)]),
                currency("krone_neu", vec![(1924.5, 9.0)]),
            ],
            periods: vec![
                vec![Period::new(1900.0, 1), Period::new(1923.0 + 10.0 / 12.0, 2)];
                catalog.countries.len()
            ],
            lead: 0,
            us_prices: vec![(1900.5, 10.0), (2026.5, 300.0)],
            base_year: 2026,
            inflation_after: 0.02,
            prices_provenance: Provenance::default(),
        };
        let settings = game().state().settings.clone();
        Game::new(Arc::new(catalog), settings).expect("valid settings")
    }

    #[test]
    fn money_options_follow_the_headquarters_and_the_date() {
        // Without currency data the amounts stay in game dollars.
        assert_eq!(overview(&game()).money, None);
        let g = game_with_currencies();
        let money = overview(&g).money.expect("currency data");
        assert_eq!(money.home_base.currency, "krone_neu");
        assert!((money.home_base.factor - 9.0).abs() < 1e-9);
        assert_eq!(money.home_then.currency, "krone");
        // 1 January 1900: the rate and the prices of that day.
        assert!((money.home_then.factor - 4.0 * 10.0 / 300.0).abs() < 1e-9);
        assert_eq!(
            (money.lead_base.currency.as_str(), money.lead_base.factor),
            ("usd", 1.0)
        );
        assert!((money.lead_then.factor - 10.0 / 300.0).abs() < 1e-9);
        assert_eq!(money.base_year, 2026);
    }

    #[test]
    fn country_detail_names_the_currencies_and_the_rate() {
        let key = |g: &Game| {
            let c = g.catalog().countries.ids().next().expect("one country");
            g.catalog().countries.key(c).to_owned()
        };
        let plain = game();
        let d = country_detail(&plain, &key(&plain)).expect("country");
        assert!(d.currencies.is_empty());
        assert_eq!(d.currency_per_usd, None);

        let g = game_with_currencies();
        let d = country_detail(&g, &key(&g)).expect("country");
        let periods: Vec<(&str, &str, bool)> = d
            .currencies
            .iter()
            .map(|p| (p.from.as_str(), p.currency.as_str(), p.current))
            .collect();
        assert_eq!(
            periods,
            [("1900-01", "krone", true), ("1923-11", "krone_neu", false)]
        );
        // 1 January 1900: before the first rate the first one holds.
        assert!((d.currency_per_usd.expect("rate") - 4.0).abs() < 1e-9);
    }

    #[test]
    fn round_report_compares_cash_and_lists_messages() {
        let mut g = game();
        let before = snapshot(&g);
        let report = g.advance(RoundLength::Month, |_| {});
        let view = round_report(&g, &report, &before);
        assert_eq!(view.from, "1900-01-01");
        assert_eq!(view.to, "1900-01-31");
        assert_eq!(view.days, 31);
        assert!((view.cash_before_usd - usd(before.cash)).abs() < 1e-9);
        // The result of the round is the change of equity (no capital moves here).
        assert!((view.period.result_usd - view.equity_change_usd).abs() < 0.01);
        let json = serde_json::to_string(&view).expect("serializable");
        let back: RoundReportView = serde_json::from_str(&json).expect("round trip");
        assert_eq!(back, view);
    }

    #[test]
    fn round_result_spans_the_turn_of_the_year() {
        let mut g = game();
        for _ in 0..11 {
            g.advance(RoundLength::Month, |_| {});
        }
        let before = snapshot(&g);
        let report = g.advance(RoundLength::Quarter, |_| {});
        assert_eq!(report.from.year(), 1900);
        let view = round_report(&g, &report, &before);
        assert!((view.period.result_usd - view.equity_change_usd).abs() < 0.01);
    }

    #[test]
    fn message_parameters_keep_their_type() {
        let m = Message::new(MessageKind::Warning, "test")
            .with(
                "betrag",
                Param::Money(Money::from_usd(12.5).expect("valid")),
            )
            .with("land", Param::Country("DEU".into()));
        let v = message_view(&m);
        assert_eq!(v.kind, "warning");
        assert_eq!(v.params["betrag"], ParamView::Money(12.5));
        let json = serde_json::to_string(&v).expect("serializable");
        assert!(
            json.contains(r#""betrag":{"type":"money","value":12.5}"#),
            "{json}"
        );
        assert_eq!(v.target.as_deref(), Some("finanzen"));
    }

    #[test]
    fn world_map_and_country_detail() {
        let g = game();
        let map = world_map(&g);
        assert_eq!(map.countries.len(), g.catalog().countries.len());
        let sites = map
            .countries
            .iter()
            .map(|c| c.own_sites + c.other_sites)
            .sum::<u32>();
        assert_eq!(sites as usize, g.state().sites.len());
        let key = map.countries[0].key.clone();
        let detail = country_detail(&g, &key).expect("known country");
        assert_eq!(detail.labor.len(), g.catalog().labor_groups.len());
        assert!(country_detail(&g, "XXX").is_none());
    }

    /// Mine and furnace in AAA (test chain): the furnace has no ore and its iron is
    /// not offered.
    fn chain() -> (Game, u32, u32) {
        use crate::catalog::SiteType;
        use crate::command::Command;
        use crate::state::SiteId;
        let mut g = game_with(10_000_000.0);
        let c = g.catalog().clone();
        let aaa = c.countries.id("AAA").expect("exists");
        let found = |g: &mut Game, kind| {
            g.apply(Command::FoundSite { country: aaa, kind })
                .expect("valid");
            SiteId(u32::try_from(g.state().sites.len() - 1).expect("fits"))
        };
        let mine = found(&mut g, SiteType::Extraction);
        let works = found(&mut g, SiteType::Factory);
        g.apply(Command::DevelopDeposit {
            site: mine,
            deposit: c.deposits.id("grube").expect("exists"),
        })
        .expect("valid");
        for (site, facility, recipe) in [
            (mine, "mine", "erz_abbau"),
            (works, "ofen", "eisen_schmelzen"),
        ] {
            g.apply(Command::BuildFacility {
                site,
                facility: c.facilities.id(facility).expect("exists"),
                count: 1,
                size: crate::catalog::FacilitySize::Medium,
            })
            .expect("valid");
            g.apply(Command::SetProduction {
                site,
                slot: 0,
                recipe: c.recipes.id(recipe),
                utilization: 1.0,
            })
            .expect("valid");
        }
        (g, mine.0, works.0)
    }

    #[test]
    fn hints_point_to_the_place_to_act() {
        let (mut g, mine, works) = chain();
        g.advance(RoundLength::Month, |_| {});
        let hints = hints(&g);
        let find = |key: &str, site: u32| {
            hints
                .iter()
                .find(|h| h.message.key == key && h.site == Some(site))
        };
        // The furnace waits for ore and has no purchase for it.
        let input = find(crate::message::keys::HINT_INPUT, works).expect("ore missing");
        assert_eq!(input.area.as_deref(), Some("einkauf"));
        assert_eq!(input.message.target.as_deref(), Some("produktion"));
        assert_eq!(
            input.message.params["produkt"],
            ParamView::TextKey("produkt.erz".into())
        );
        assert_eq!(
            input.message.params["standort"],
            ParamView::TextKey("standorttyp.werk".into())
        );
        // The ore of the mine is made but not offered.
        let offer = find(crate::message::keys::HINT_NO_OFFER, mine).expect("no offer");
        assert_eq!(offer.area.as_deref(), Some("verkauf"));
        assert!(
            hints
                .iter()
                .all(|h| h.message.key != crate::message::keys::HINT_OVERDRAWN)
        );
        // Urgent first: no info before a warning.
        let ranks: Vec<bool> = hints.iter().map(|h| h.message.kind == "info").collect();
        assert!(ranks.windows(2).all(|w| !(w[0] && !w[1])), "{hints:?}");
    }

    #[test]
    fn history_works_the_cash_back_from_today() {
        let (mut g, _, works) = chain();
        for _ in 0..3 {
            g.advance(RoundLength::Month, |_| {});
        }
        g.advance(RoundLength::Week, |_| {});
        let ledger = &g.state().companies[g.state().player.index()].ledger;
        let months = history(ledger);
        assert_eq!(months.len(), 3);
        assert_eq!(months[0].month, "1900-01-01");
        let last = months.last().expect("three months");
        let today = usd(ledger.cash() - ledger.month.cash_flow.total());
        assert!((last.cash_usd - today).abs() < 1e-6);
        let before = last.cash_usd - usd(ledger.months[2].cash_flow.total());
        assert!((months[1].cash_usd - before).abs() < 1e-6);
        // The results by site add up to the company's result with its own items.
        let f = finance_overview(&g);
        let centers = f.centers_last_month.expect("details of the last month");
        let sites: f64 = centers.sites.iter().map(|s| s.result_usd).sum();
        assert!((sites + centers.company_usd - last.result_usd).abs() < 0.01);
        assert!(centers.sites.iter().any(|s| s.site == works));
        assert_eq!(f.history, months);
    }

    #[test]
    fn product_market_lists_sellers_and_demand() {
        use crate::command::Command;
        use crate::state::{PriceMode, SiteId};
        let (mut g, mine, _) = chain();
        g.advance(RoundLength::Month, |_| {});
        let c = g.catalog().clone();
        let ore = c.products.id("erz").expect("exists");
        g.apply(Command::SetSale {
            site: SiteId(mine),
            product: ore,
            mode: Some(PriceMode::Fixed(Money::from_usd(9.0).expect("valid"))),
            keep: 0.0,
        })
        .expect("valid");
        g.advance(RoundLength::Month, |_| {});
        let view = product_market(&g, "AAA", "erz").expect("known product");
        assert_eq!(view.unit, "t");
        let own = view.sellers.iter().find(|s| s.own).expect("own offer");
        assert!((own.price_usd - 9.0).abs() < 1e-9);
        let shares: f64 = view.sellers.iter().map(|s| s.share).sum();
        assert!(shares <= 1.0 + 1e-9);
        assert!(product_market(&g, "AAA", "gibts_nicht").is_none());
        let world = world_market(&g, "erz").expect("known product");
        let aaa = world
            .countries
            .iter()
            .find(|l| l.country == "AAA")
            .expect("own seller in AAA");
        assert_eq!(aaa.own_sellers, 1);
        let line = market(&g, "AAA")
            .expect("known country")
            .lines
            .into_iter()
            .find(|l| l.product == "erz")
            .expect("ore");
        assert_eq!(
            line.chances,
            product_market(&g, "AAA", "erz").expect("ore").chances
        );
    }

    #[test]
    fn round_report_shows_what_each_product_brought() {
        use crate::command::Command;
        use crate::state::{PriceMode, SiteId};
        let (mut g, mine, _) = chain();
        let c = g.catalog().clone();
        // Bread for the consumers of AAA (the test chain has no recipe for it).
        let bread = c.products.id("brot").expect("exists");
        g.state_mut().sites[mine as usize]
            .inventory
            .entry(bread)
            .or_default()
            .add(1e9, Money::from_usd(1e9).expect("valid"), 50.0);
        g.apply(Command::SetSale {
            site: SiteId(mine),
            product: bread,
            mode: Some(PriceMode::Fixed(Money::from_usd(2.0).expect("valid"))),
            keep: 0.0,
        })
        .expect("valid");
        // Over the turn of the year: the closed year still counts.
        for _ in 0..11 {
            g.advance(RoundLength::Month, |_| {});
        }
        let before = snapshot(&g);
        let report = g.advance(RoundLength::Quarter, |_| {});
        let view = round_report(&g, &report, &before);
        let sold = view
            .products
            .iter()
            .find(|p| p.product == "brot")
            .expect("bread was sold");
        assert!(sold.revenue_usd > 0.0);
        // Bought in at 1 USD, sold at 2 USD.
        assert!((sold.margin_usd - 0.5 * sold.revenue_usd).abs() < 0.01);
        let revenue = view
            .period
            .lines
            .iter()
            .find(|(k, _)| k == "kostenart.umsatz")
            .map_or(0.0, |l| l.1);
        let by_product: f64 = view.products.iter().map(|p| p.revenue_usd).sum();
        assert!(
            (revenue - by_product).abs() < 0.01,
            "{revenue} vs {by_product}"
        );
        // The summary adds up: revenue − costs + change of the stock = result.
        let p = &view.period;
        assert!(p.inventory_change_usd.abs() > 0.01, "the chain makes goods");
        assert!(
            (p.revenue_usd - p.costs_usd + p.inventory_change_usd - p.result_usd).abs() < 0.01,
            "{p:?}"
        );
    }

    #[test]
    fn start_form_keys_round_trip() {
        for f in [StartForm::Workshop, StartForm::Trading] {
            assert_eq!(start_form_from_key(start_form_key(f)), Some(f));
        }
    }
}
