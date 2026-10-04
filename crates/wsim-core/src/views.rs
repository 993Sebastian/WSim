//! Views for the user interface (Architektur §2.4): plain data derived from the game,
//! with keys instead of display texts and every number already computed, so that the
//! interface shows them without game logic of its own. Amounts are USD as floating
//! point numbers (display only), dates are `YYYY-MM-DD`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::EARLIEST_START_YEAR;
use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::site_type_key;
use crate::game::{
    Game, MAX_RESEARCH_FACTOR, MAX_START_CAPITAL_USD, MIN_RESEARCH_FACTOR, RoundReport,
};
use crate::ids::Id;
use crate::ledger::{Account, CostType};
use crate::message::{Message, MessageKind, Param};
use crate::money::Money;
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
}

fn equity(company: &Company) -> Money {
    company.ledger.total_assets() - company.ledger.balance(Account::Loans)
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
    let ai: Vec<&Company> = state.companies.iter().filter(|c| c.ai.is_some()).collect();
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
    // Messages about the own money lead to the overview with the finances; the other
    // views follow with M13 and M14.
    let target = match message.kind {
        MessageKind::Warning | MessageKind::Crisis | MessageKind::Success => {
            Some("uebersicht".to_owned())
        }
        _ => None,
    };
    let group = if message.kind == MessageKind::WorldEvent {
        "welt"
    } else if message.key.starts_with("meldung.ki.")
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
    years_closed: usize,
}

pub fn snapshot(game: &Game) -> Snapshot {
    let state = game.state();
    let company = &state.companies[state.player.index()];
    Snapshot {
        cash: company.ledger.cash(),
        equity: equity(company),
        year: company.ledger.year.by_type.clone(),
        years_closed: company.ledger.years.len(),
    }
}

/// Revenue, costs and result of a period, with the amounts by cost type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeriodView {
    pub revenue_usd: f64,
    /// Costs as a positive amount (everything but revenue and inventory change).
    pub costs_usd: f64,
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
                max_output_per_year: d.max_output_per_year * state.settings.market_scale,
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
    Some(CountryDetail {
        key: key.to_owned(),
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
        let catalog = Arc::new(test_support::production());
        let settings = GameSettings {
            seed: 3,
            start_year: 1900,
            start_country: catalog.countries.ids().next().expect("one country"),
            start_capital: Money::from_usd(1_000_000.0).expect("valid"),
            start_form: StartForm::Workshop,
            company_name: "Test AG".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: AiSettings::default(),
        };
        Game::new(catalog, settings).expect("valid settings")
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
        assert_eq!(v.target.as_deref(), Some("uebersicht"));
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

    #[test]
    fn start_form_keys_round_trip() {
        for f in [StartForm::Workshop, StartForm::Trading] {
            assert_eq!(start_form_from_key(start_form_key(f)), Some(f));
        }
    }
}
