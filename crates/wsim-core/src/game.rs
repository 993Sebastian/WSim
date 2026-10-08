//! A running game: catalog, state, journal of decisions, and the round loop
//! (Lastenheft §13).

use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::EARLIEST_START_YEAR;
use crate::calendar::{Date, GAME_END, RoundLength};
use crate::catalog::Catalog;
use crate::command::{self, Command, CommandError, NameError};
use crate::finance;
use crate::ids::Id;
use crate::ledger::Account;
use crate::ledger::Ledger;
use crate::market;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::production;
use crate::research;
use crate::rng::{SimRng, Stream};
use crate::state::{
    Company, CompanyId, CompanyKind, GameSettings, GameState, PerId, PriceMode, PurchaseOrder,
    SaleOffer, Site, Slot,
};
use crate::trade;

/// Latest selectable start year; technology freezes in 2026 (Lastenheft §3.1).
pub const LATEST_START_YEAR: i32 = 2026;
/// Highest start capital in USD.
pub const MAX_START_CAPITAL_USD: f64 = 1.0e12;
/// Range of the research cost setting (Lastenheft §15).
pub const MIN_RESEARCH_FACTOR: f64 = 0.25;
pub const MAX_RESEARCH_FACTOR: f64 = 4.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NewGameError {
    StartYear {
        year: i32,
    },
    StartCapital,
    StartCountry,
    Name(NameError),
    ResearchFactor,
    /// The start capital does not cover the workshop or office of the start form.
    StartFormTooExpensive {
        needed: Money,
    },
    TooManyCompanies {
        max: u32,
    },
    /// The person's start money does not found the cheapest company (PE3).
    StartMoneyTooLow {
        needed: Money,
    },
}

impl NewGameError {
    pub fn message(&self) -> Message {
        match self {
            NewGameError::StartYear { year } => Message::error(keys::NEW_GAME_START_YEAR)
                .with("jahr", Param::Integer(i64::from(*year)))
                .with("von", Param::Integer(i64::from(EARLIEST_START_YEAR)))
                .with("bis", Param::Integer(i64::from(LATEST_START_YEAR))),
            NewGameError::StartCapital => Message::error(keys::NEW_GAME_START_CAPITAL).with(
                "max",
                Param::Money(Money::from_usd(MAX_START_CAPITAL_USD).expect("in range")),
            ),
            NewGameError::StartCountry => Message::error(keys::NEW_GAME_START_COUNTRY),
            NewGameError::Name(e) => e.message(),
            NewGameError::StartFormTooExpensive { needed } => {
                Message::error(keys::NEW_GAME_START_FORM).with("betrag", Param::Money(*needed))
            }
            NewGameError::StartMoneyTooLow { needed } => {
                Message::error(keys::NEW_GAME_START_MONEY).with("betrag", Param::Money(*needed))
            }
            NewGameError::TooManyCompanies { max } => {
                Message::error(keys::NEW_GAME_TOO_MANY_COMPANIES)
                    .with("max", Param::Integer(i64::from(*max)))
            }
            NewGameError::ResearchFactor => Message::error(keys::NEW_GAME_RESEARCH_FACTOR)
                .with("von", Param::Number(MIN_RESEARCH_FACTOR))
                .with("bis", Param::Number(MAX_RESEARCH_FACTOR)),
        }
    }
}

/// Everything that happened in a game, in order. Replaying it on the same catalog
/// reproduces the game exactly.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum JournalEntry {
    Command {
        date: Date,
        actor: CompanyId,
        command: Command,
    },
    Round {
        from: Date,
        length: RoundLength,
    },
    /// A command of the person (PE3).
    Person {
        date: Date,
        command: Command,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    /// Days simulated so far in this round.
    pub done: u32,
    pub total: u32,
    pub date: Date,
}

/// Result of one round. Extended to the full round report in M11.
#[derive(Clone, Debug, PartialEq)]
pub struct RoundReport {
    pub from: Date,
    /// First day after the round.
    pub to: Date,
    pub days: u32,
    pub messages: Vec<Message>,
}

/// Fingerprint of the complete game state, for reproducibility checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StateHash(pub u64);

impl fmt::Display for StateHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReplayError {
    NewGame(NewGameError),
    Command {
        index: usize,
        error: CommandError,
    },
    /// The journal does not fit the game (entry at the wrong date).
    OutOfOrder {
        index: usize,
    },
}

#[derive(Clone, Debug)]
pub struct Game {
    catalog: Arc<Catalog>,
    state: GameState,
    journal: Vec<JournalEntry>,
}

impl Game {
    pub fn new(catalog: Arc<Catalog>, settings: GameSettings) -> Result<Self, NewGameError> {
        if !(EARLIEST_START_YEAR..=LATEST_START_YEAR).contains(&settings.start_year) {
            return Err(NewGameError::StartYear {
                year: settings.start_year,
            });
        }
        let max_capital = Money::from_usd(MAX_START_CAPITAL_USD).expect("in range");
        if settings.start_capital <= Money::ZERO || settings.start_capital > max_capital {
            return Err(NewGameError::StartCapital);
        }
        if settings.start_country.index() >= catalog.countries.len() {
            return Err(NewGameError::StartCountry);
        }
        if !(MIN_RESEARCH_FACTOR..=MAX_RESEARCH_FACTOR).contains(&settings.research_ahead_factor) {
            return Err(NewGameError::ResearchFactor);
        }
        let name = if settings.found_at_start {
            command::check_company_name(None, &settings.company_name, None)
                .map_err(NewGameError::Name)?
        } else {
            String::new()
        };
        if settings.ai.companies > catalog.ai_model.max_companies {
            return Err(NewGameError::TooManyCompanies {
                max: catalog.ai_model.max_companies,
            });
        }
        let mut settings = settings;
        if settings.ai.companies > 0 {
            settings.market_scale = catalog.ai_model.market_scale(settings.ai.companies);
        }

        let date = Date::first_of_year(settings.start_year);
        let mut state = GameState {
            world_rng: SimRng::for_stream(settings.seed, Stream::World),
            settings,
            date,
            countries: PerId::default(),
            companies: Vec::new(),
            sites: Vec::new(),
            markets: PerId::default(),
            shipments: Vec::new(),
            plots: Vec::new(),
            routes: Default::default(),
            import_markets: Default::default(),
            deposits: PerId::default(),
            inventions: PerId::default(),
            patents: PerId::default(),
            developments: PerId::default(),
            milestones: PerId::default(),
            watched_markets: Vec::new(),
            standings: Vec::new(),
            offers: Vec::new(),
            next_offer: 0,
            managers: Default::default(),
            next_manager: 0,
            concerns: Vec::new(),
            next_concern: 0,
            followups: Vec::new(),
            poach_offers: Vec::new(),
            judgments: Vec::new(),
            ventures: Vec::new(),
            next_venture: 0,
            tariff_offsets: PerId::default(),
            tariffs: Default::default(),
            events: Default::default(),
            regulation: Default::default(),
            contracts: Vec::new(),
            next_contract: 0,
            freight_market: Default::default(),
            stock: Default::default(),
            main_company: None,
            game_over: false,
            person: Default::default(),
        };
        state.refresh_countries(&catalog);
        let found_at_start = state.settings.found_at_start;
        if found_at_start {
            let (country, capital) = (state.settings.start_country, state.settings.start_capital);
            let id = push_player_company(&mut state, name, country, capital);
            state.main_company = Some(id);
        } else {
            let needed =
                crate::private::start_minimum(&catalog, &state, state.settings.start_country);
            if state.settings.start_capital < needed {
                return Err(NewGameError::StartMoneyTooLow { needed });
            }
        }
        state.fit_to_catalog(&catalog);
        market::initial_demand(&mut state, &catalog, date);
        crate::plots::supply(&mut state, &catalog, date.year());
        // The start set is there before the closures of the events (H1).
        let events = std::mem::take(&mut state.events);
        if let Some(id) = state.main_company {
            let s = &state.settings;
            let (form, country, capital) = (s.start_form, s.start_country, s.start_capital);
            start_setup(&mut state, &catalog, id, (form, country, capital))?;
        }
        crate::population::populate(&mut state, &catalog);
        state.events = events;
        crate::events::start(&mut state);
        crate::stock::list_at_start(&mut state, &catalog);
        crate::management::month_start(&mut state, &catalog, date);
        crate::person::start(&mut state, &catalog, date);
        crate::private::start(&mut state, &catalog, date);
        crate::ranking::record(&mut state);
        Ok(Self {
            catalog,
            state,
            journal: Vec::new(),
        })
    }

    /// Rebuilds a game from its settings and journal.
    pub fn replay(
        catalog: Arc<Catalog>,
        settings: GameSettings,
        journal: &[JournalEntry],
    ) -> Result<Self, ReplayError> {
        let mut game = Self::new(catalog, settings).map_err(ReplayError::NewGame)?;
        for (index, entry) in journal.iter().enumerate() {
            match entry {
                JournalEntry::Command {
                    date,
                    actor,
                    command,
                } => {
                    if *date != game.date() {
                        return Err(ReplayError::OutOfOrder { index });
                    }
                    game.apply_as(*actor, command.clone())
                        .map_err(|error| ReplayError::Command { index, error })?;
                }
                JournalEntry::Round { from, length } => {
                    if *from != game.date() {
                        return Err(ReplayError::OutOfOrder { index });
                    }
                    game.advance(*length, |_| {});
                }
                JournalEntry::Person { date, command } => {
                    if *date != game.date() {
                        return Err(ReplayError::OutOfOrder { index });
                    }
                    game.apply(command.clone())
                        .map_err(|error| ReplayError::Command { index, error })?;
                }
            }
        }
        Ok(game)
    }

    pub(crate) fn from_parts(
        catalog: Arc<Catalog>,
        state: GameState,
        journal: Vec<JournalEntry>,
    ) -> Self {
        Self {
            catalog,
            state,
            journal,
        }
    }

    pub fn catalog(&self) -> &Arc<Catalog> {
        &self.catalog
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Direct access for tests that set up special situations.
    #[cfg(test)]
    pub(crate) fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    pub fn journal(&self) -> &[JournalEntry] {
        &self.journal
    }

    pub fn date(&self) -> Date {
        self.state.date
    }

    /// The main company. Panics before the person founded one: callers that may run
    /// without a company ask `main_company` first.
    pub fn player(&self) -> CompanyId {
        self.state.main_company.expect("the person has a company")
    }

    /// The person's main company (PE3); `None` before the first founding.
    pub fn main_company(&self) -> Option<CompanyId> {
        self.state.main_company
    }

    pub fn is_over(&self) -> bool {
        self.state.game_over
    }

    /// Executes a decision of the player.
    pub fn apply(&mut self, command: Command) -> Result<(), CommandError> {
        if command.is_personal() {
            crate::private::execute(&mut self.state, &self.catalog, &command)?;
            self.journal.push(JournalEntry::Person {
                date: self.state.date,
                command,
            });
            return Ok(());
        }
        let actor = self.state.main_company.ok_or(CommandError::NoCompany)?;
        self.apply_as(actor, command)
    }

    /// Executes a decision of any company. Successful commands go into the journal.
    pub fn apply_as(&mut self, actor: CompanyId, command: Command) -> Result<(), CommandError> {
        command::execute(&mut self.state, &self.catalog, actor, &command)?;
        self.journal.push(JournalEntry::Command {
            date: self.state.date,
            actor,
            command,
        });
        Ok(())
    }

    /// Simulates one round day by day (Lastenheft §13.1). `progress` is called after
    /// every day.
    pub fn advance(
        &mut self,
        length: RoundLength,
        mut progress: impl FnMut(Progress),
    ) -> RoundReport {
        let from = self.state.date;
        let mut report = RoundReport {
            from,
            to: from,
            days: 0,
            messages: Vec::new(),
        };
        if self.state.game_over {
            return report;
        }
        self.journal.push(JournalEntry::Round { from, length });
        let to = length.end(from);
        let total = u32::try_from(from.days_until(to)).expect("rounds move forward");
        for done in 1..=total {
            self.simulate_day(&mut report);
            progress(Progress {
                done,
                total,
                date: self.state.date,
            });
        }
        report.to = to;
        report.days = total;
        if let Some(warning) = finance::overdraft_warning(&self.state, &self.catalog) {
            report.messages.push(warning);
        }
        report
            .messages
            .extend(production::input_warnings(&self.state, &self.catalog));
        if to >= GAME_END {
            self.state.game_over = true;
            report
                .messages
                .push(Message::new(MessageKind::Info, keys::GAME_END));
        }
        report
    }

    /// One day. Systems run in a fixed order (docs/ARCHITEKTUR.md §2.2); production,
    /// markets and finance join from M5 on.
    fn simulate_day(&mut self, report: &mut RoundReport) {
        let today = self.state.date;
        report.messages.extend(crate::management::simulate_day(
            &mut self.state,
            &self.catalog,
            today,
        ));
        report.messages.extend(crate::staffing::simulate_day(
            &mut self.state,
            &self.catalog,
            today,
        ));
        report
            .messages
            .extend(crate::ai::decide(&mut self.state, &self.catalog, today));
        report
            .messages
            .extend(trade::deliver(&mut self.state, &self.catalog, today));
        production::simulate_day(&mut self.state, &self.catalog, today);
        report.messages.extend(crate::contracts::deliver(
            &mut self.state,
            &self.catalog,
            today,
        ));
        market::clear(&mut self.state, &self.catalog, today);
        report.messages.extend(research::simulate_day(
            &mut self.state,
            &self.catalog,
            today,
        ));
        report
            .messages
            .extend(world_events(&self.state, &self.catalog, today));
        report.messages.extend(crate::events::simulate_day(
            &mut self.state,
            &self.catalog,
            today,
        ));
        report.messages.extend(crate::deals::simulate_day(
            &mut self.state,
            &self.catalog,
            today,
        ));
        if today.day() == 1 {
            report
                .messages
                .extend(currency_reforms(&self.state, &self.catalog, today));
        }

        let next = today.next_day();
        if next.day() == 1 {
            market::settle_all_idle(&mut self.state, &self.catalog, next);
        }
        self.state.date = next;
        if next.day() == 1 {
            report.messages.extend(crate::contracts::month_start(
                &mut self.state,
                &self.catalog,
                next,
            ));
            crate::management::month_end(&mut self.state, today);
            crate::private::pay_salary(&mut self.state, today);
            crate::central::month_end(&mut self.state, &self.catalog);
            crate::logistics::month_end(&mut self.state, &self.catalog, today);
            report
                .messages
                .extend(crate::bonds::month_end(&mut self.state, today));
            crate::bank::month_end(&mut self.state, &self.catalog, today);
            finance::month_end(&mut self.state, &self.catalog, today);
            report.messages.extend(crate::dividends::pay_year(
                &mut self.state,
                &self.catalog,
                today,
            ));
            report.messages.extend(crate::private::month_end(
                &mut self.state,
                &self.catalog,
                today,
            ));
            report.messages.extend(crate::holdings::month_end(
                &mut self.state,
                &self.catalog,
                today,
            ));
            for company in &mut self.state.companies {
                company.ledger.close_month(next);
            }
            report
                .messages
                .extend(finance::check_insolvency(&mut self.state, &self.catalog));
            report
                .messages
                .extend(crate::bank::write_off_failures(&mut self.state));
            report
                .messages
                .extend(crate::group::settle_failures(&mut self.state));
            report.messages.extend(crate::stock::month_start(
                &mut self.state,
                &self.catalog,
                next,
            ));
            report.messages.extend(crate::dividends::year_start(
                &mut self.state,
                &self.catalog,
                next,
            ));
            if next.ordinal() == 1 {
                crate::tariffs::new_year(&mut self.state, &self.catalog, next.year());
            }
            self.state.refresh_countries(&self.catalog);
            report.messages.extend(crate::management::month_start(
                &mut self.state,
                &self.catalog,
                next,
            ));
            report.messages.extend(crate::central::month_start(
                &mut self.state,
                &self.catalog,
                next,
            ));
            report.messages.extend(crate::central::ai_month_start(
                &mut self.state,
                &self.catalog,
                next,
            ));
            report.messages.extend(crate::staffing::month_start(
                &mut self.state,
                &self.catalog,
                next,
            ));
            report.messages.extend(crate::heirs::month_start(
                &mut self.state,
                &self.catalog,
                next,
            ));
            report.messages.extend(crate::person::month_start(
                &mut self.state,
                &self.catalog,
                next,
            ));
            crate::private::month_start(&mut self.state, &self.catalog, next);
            report.messages.extend(crate::regulation::month_start(
                &mut self.state,
                &self.catalog,
                next,
            ));
            report
                .messages
                .extend(crate::patents::month_start(&mut self.state, &self.catalog));
            report.messages.extend(crate::ventures::month_start(
                &mut self.state,
                &self.catalog,
                next,
            ));
            crate::plots::month_start(&mut self.state, &self.catalog, next);
            crate::training::ai_month_start(&mut self.state, &self.catalog, next);
            crate::training::month_start(&mut self.state, &self.catalog);
            production::new_month(&mut self.state);
            market::month_start(&mut self.state, &self.catalog, next);
            market::reset_site_months(&mut self.state);
            report.messages.extend(crate::competition::month_end(
                &mut self.state,
                &self.catalog,
            ));
            report
                .messages
                .extend(crate::ranking::month_end(&mut self.state));
            report.messages.extend(crate::review::month_end(
                &mut self.state,
                &self.catalog,
                next,
            ));
            crate::brand::month_start(&mut self.state, &self.catalog, next);
        }
        if next.ordinal() == 1 {
            production::new_year(&mut self.state);
        }
        report.messages.extend(crate::milestones::check(
            &mut self.state,
            &self.catalog,
            today,
        ));
        if next.ordinal() == 1 && next < GAME_END {
            report.messages.push(
                Message::new(MessageKind::Info, keys::NEW_YEAR)
                    .with("jahr", Param::Integer(i64::from(next.year()))),
            );
        }
    }

    pub fn state_hash(&self) -> StateHash {
        hash_of(&self.state)
    }
}

/// Fingerprint of a game state.
pub fn hash_of(state: &GameState) -> StateHash {
    let bytes = rmp_serde::to_vec_named(state).expect("state is serializable");
    StateHash(fnv1a(&bytes))
}

/// Historical events of the day as world news (Lastenheft §4.1, §13.2).
fn world_events(state: &GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let first = catalog.events.partition_point(|e| e.date < date);
    catalog.events[first..]
        .iter()
        .take_while(|e| e.date == date)
        .flat_map(|e| {
            let news = Message::new(MessageKind::WorldEvent, keys::WORLD_EVENT)
                .with("ereignis", Param::TextKey(format!("ereignis.{}", e.key)))
                .with(
                    "beschreibung",
                    Param::TextKey(format!("ereignis.{}.text", e.key)),
                )
                .with("art", Param::TextKey(format!("ereignisart.{}", e.kind)))
                .with("datum", Param::Date(e.date))
                .with(
                    "laender",
                    Param::Countries(
                        e.countries
                            .iter()
                            .map(|&c| catalog.countries.key(c).to_owned())
                            .collect(),
                    ),
                );
            // Its effects follow as lines of their own (H1).
            let effects = if state.settings.event_effects {
                crate::events::effect_messages(catalog, e)
            } else {
                Vec::new()
            };
            std::iter::once(news).chain(effects)
        })
        .collect()
}

/// Changes of currency in the countries of the player's headquarters and sites (M28):
/// history told along the way. Amounts in the game do not change; they are shown in
/// the new currency from now on.
fn currency_reforms(state: &GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let model = &catalog.currencies;
    // Periods start on the first day of a month: year + (month − 1)/12.
    let t = f64::from(date.year()) + f64::from(date.month() - 1) / 12.0;
    // Without a company the person's home.
    let home = state.main_company.map_or(state.person.home, |c| {
        state.companies[c.index()].headquarters
    });
    let mut countries = vec![home];
    for s in state.sites.iter().filter(|s| state.is_main(s.owner)) {
        if !countries.contains(&s.country) {
            countries.push(s.country);
        }
    }
    countries
        .into_iter()
        .filter_map(|country| {
            let reform = model.reform_at(country, t)?;
            let old = &model.currencies[reform.old];
            let new = &model.currencies[reform.new];
            let key = catalog.countries.key(country).to_owned();
            Some(
                Message::new(MessageKind::WorldEvent, keys::CURRENCY_REFORM)
                    .with(
                        "ereignis",
                        Param::TextKey(keys::CURRENCY_REFORM_TITLE.to_owned()),
                    )
                    .with("art", Param::TextKey(keys::EVENT_KIND_CURRENCY.to_owned()))
                    .with("land", Param::Country(key.clone()))
                    .with("monat", Param::TextKey(format!("monat.{}", date.month())))
                    .with("jahr", Param::Integer(i64::from(date.year())))
                    .with("alt", Param::TextKey(format!("waehrung.{}", old.key)))
                    .with("symbol_alt", Param::Text(old.symbol.clone()))
                    .with("neu", Param::TextKey(format!("waehrung.{}", new.key)))
                    .with("symbol_neu", Param::Text(new.symbol.clone()))
                    .with("faktor", Param::Number(reform.factor))
                    .with("laender", Param::Countries(vec![key])),
            )
        })
        .collect()
}

/// Gives the new company the site of its start form (Lastenheft §2, §15), paid from
/// the start capital: a small workshop that makes simple parts, or a trading office.
/// A company of the person with nothing but `capital` in cash (PE3).
pub(crate) fn push_player_company(
    state: &mut GameState,
    name: String,
    country: crate::ids::CountryId,
    capital: Money,
) -> CompanyId {
    let index = u32::try_from(state.companies.len()).expect("company count fits u32");
    let date = state.date;
    state.companies.push(Company {
        brands: Vec::new(),
        emissions: Default::default(),
        emissions_last: Default::default(),
        advertising: Vec::new(),
        auction_until: None,
        development: Default::default(),
        product_names: Default::default(),
        positions: Vec::new(),
        budget_rules: Vec::new(),
        strategies: Vec::new(),
        mandate: crate::mandate::Mandate::default(),
        reviews: Vec::new(),
        relocation: None,
        relocated: None,
        departments: Default::default(),
        departments_staffed: Default::default(),
        hq_city: None,
        participations: Default::default(),
        logistics: Default::default(),
        subsidiary_of: None,
        listing: None,
        dividend_payout: None,
        dividend: Default::default(),
        stock_cost: Default::default(),
        bonds: Vec::new(),
        bank: None,
        state_owned: None,
        former_managers: Vec::new(),
        owners: crate::state::Stake::sole(crate::state::Holder::Player),
        name,
        kind: CompanyKind::Player,
        headquarters: country,
        founded: date,
        rng: SimRng::for_stream(state.settings.seed, Stream::Company(index)),
        ledger: Ledger::new(date, capital),
        technologies: BTreeSet::new(),
        bankrupt: false,
        loans: Vec::new(),
        loss_carryforward: Money::ZERO,
        sales_policies: Vec::new(),
        research: Default::default(),
        ai: None,
    });
    CompanyId(index)
}

/// Builds the start form of the person's new company in a country from its capital
/// (FORMELN „Startformen“); a bank gets its settings (K4).
pub(crate) fn start_setup(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    (form, country, capital): (crate::state::StartForm, crate::ids::CountryId, Money),
) -> Result<(), NewGameError> {
    if form == crate::state::StartForm::Bank && catalog.bank.enabled {
        state.companies[company.index()].bank = Some(crate::bank::BankSettings::start(catalog));
    }
    let Some(setup) = catalog.production_model.start_setup(form) else {
        return Ok(());
    };
    let cost = setup.cost(catalog);
    if cost > capital {
        return Err(NewGameError::StartFormTooExpensive { needed: cost });
    }
    let date = state.date;
    // The facilities are completed on the first day (`complete_constructions`).
    let facilities: Money = setup
        .facilities
        .iter()
        .map(|&(f, _, _)| catalog.facilities.get(f).investment)
        .sum();
    let ledger = &mut state.companies[company.index()].ledger;
    ledger.transfer(Account::FixedAssets, Account::Cash, cost - facilities);
    ledger.transfer(Account::AssetsUnderConstruction, Account::Cash, facilities);
    let slots = setup
        .facilities
        .iter()
        .map(|&(facility, recipe, utilization)| Slot {
            facility,
            ready: date,
            count: 1,
            cost: catalog.facilities.get(facility).investment,
            recipe,
            utilization,
            automation: 0.0,
            condition: 1.0,
            batches: Vec::new(),
            last_runs: 0.0,
            limit: None,
            operation: crate::state::Operation::Running,
            size: crate::catalog::FacilitySize::Medium,
            retrofit: 0,
        })
        .collect();
    let offers = setup
        .sales
        .iter()
        .map(|&product| {
            let price = market::market_price(catalog, state, country, product);
            let offer = SaleOffer {
                mode: PriceMode::Market {
                    markup: 0.0,
                    floor: Money::ZERO,
                },
                price,
                keep: 0.0,
                sold_today: 0.0,
                sold_month: 0.0,
                sold_last_month: 0.0,
                to_traders_month: 0.0,
                to_companies_month: 0.0,
            };
            (product, offer)
        })
        .collect();
    let orders = setup
        .purchases
        .iter()
        .map(|&(product, target, max_price)| {
            let order = PurchaseOrder {
                target,
                max_price,
                min_quality: 0.0,
                bought_month: 0.0,
                bought_last_month: 0.0,
            };
            (product, order)
        })
        .collect();
    let site = crate::state::SiteId(u32::try_from(state.sites.len()).expect("site count fits u32"));
    state.sites.push(Site {
        owner: company,
        country,
        kind: setup.site_type,
        founded: date,
        building_cost: setup.building,
        deposit: None,
        slots,
        inventory: Default::default(),
        workforce: PerId::from_fn(catalog.labor_groups.len(), |_| 0.0),
        staffing_due: true,
        offers,
        orders,
        research: None,
        development: None,
        wage_premium: 0.0,
        training: 0.0,
        training_target: None,
        acquired: None,
        goodwill: None,
        plot: None,
    });
    // The workshop leases its plot: the start capital pays only building and facilities.
    if crate::plots::needs_plot(catalog, setup.site_type) {
        let s = &state.sites[site.index()];
        let need =
            crate::plots::site_area(catalog, s, None) * (1.0 + catalog.plot_model.ai_reserve);
        let revenue = crate::plots::planned_revenue(state, catalog, s);
        let plot = crate::plots::for_existing(state, catalog, country, (need, revenue));
        crate::plots::occupy(state, plot, site, crate::state::Tenure::Leased);
    }
    Ok(())
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xCBF2_9CE4_8422_2325, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3)
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::catalog::{Provenance, SiteType, test_support};
    use crate::currency::{Currency, CurrencyModel, Period, Rate};
    use crate::state::StartForm;

    /// AAA changes its currency in March 1901 at the ratio of the rates, BBB in May 1901
    /// at a rate fixed by law.
    fn game(site_in_bbb: bool) -> Game {
        let mut catalog = test_support::production();
        let currency = |key: &str, rate: f64| Currency {
            key: key.to_owned(),
            symbol: key.to_uppercase(),
            rate: Rate::Points(vec![(1900.5, rate)]),
            provenance: Provenance::default(),
        };
        let reform = |month: f64, conversion| Period {
            from: 1901.0 + (month - 1.0) / 12.0,
            currency: 2,
            conversion,
        };
        catalog.currencies = CurrencyModel {
            currencies: vec![
                currency("usd", 1.0),
                currency("mark", 4.0),
                currency("neumark", 0.04),
            ],
            periods: vec![
                vec![Period::new(1900.0, 1), reform(3.0, None)],
                vec![Period::new(1900.0, 1), reform(5.0, Some(99.0))],
            ],
            lead: 0,
            us_prices: vec![(1900.5, 10.0)],
            base_year: 2026,
            inflation_after: 0.0,
            prices_provenance: Provenance::default(),
        };
        let catalog = Arc::new(catalog);
        let settings = GameSettings {
            seed: 1,
            start_year: 1901,
            start_country: catalog.countries.id("AAA").unwrap(),
            start_capital: Money::from_usd(20_000_000.0).unwrap(),
            start_form: StartForm::Workshop,
            company_name: "Umsteller".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: Default::default(),
            ventures: 1.0,
            tariff_dynamics: 1.0,
            event_effects: true,
            found_at_start: true,
            person: Default::default(),
        };
        let mut game = Game::new(catalog.clone(), settings).unwrap();
        if site_in_bbb {
            game.apply(Command::FoundSite {
                country: catalog.countries.id("BBB").unwrap(),
                kind: SiteType::Factory,
            })
            .unwrap();
        }
        game
    }

    fn reforms_until_june(mut game: Game) -> Vec<(u32, Message)> {
        let mut found = Vec::new();
        for month in 1..=5 {
            let report = game.advance(RoundLength::Month, |_| {});
            found.extend(
                report
                    .messages
                    .into_iter()
                    .filter(|m| m.key == keys::CURRENCY_REFORM)
                    .map(|m| (month, m)),
            );
        }
        found
    }

    fn param(m: &Message, name: &str) -> Param {
        m.params
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, p)| p.clone())
            .expect("parameter")
    }

    #[test]
    fn changes_of_currency_are_told_where_the_player_is() {
        // Only the headquarters' country: AAA in March, at 4 / 0.04 marks per new mark.
        let only_home = reforms_until_june(game(false));
        assert_eq!(only_home.len(), 1);
        let (month, m) = &only_home[0];
        assert_eq!(*month, 3);
        assert_eq!(m.kind, MessageKind::WorldEvent);
        assert_eq!(param(m, "land"), Param::Country("AAA".into()));
        assert_eq!(param(m, "monat"), Param::TextKey("monat.3".into()));
        assert_eq!(param(m, "jahr"), Param::Integer(1901));
        assert_eq!(param(m, "alt"), Param::TextKey("waehrung.mark".into()));
        assert_eq!(param(m, "symbol_neu"), Param::Text("NEUMARK".into()));
        assert_eq!(param(m, "faktor"), Param::Number(100.0));

        // With a site in BBB its change in May is told too, at the rate fixed by law.
        let both = reforms_until_june(game(true));
        assert_eq!(both.len(), 2);
        let (month, m) = &both[1];
        assert_eq!(*month, 5);
        assert_eq!(param(m, "land"), Param::Country("BBB".into()));
        assert_eq!(param(m, "faktor"), Param::Number(99.0));
    }
}
