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
        let name = command::check_company_name(None, &settings.company_name, None)
            .map_err(NewGameError::Name)?;
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
        let player = Company {
            brands: Vec::new(),
            advertising: Vec::new(),
            owners: crate::state::Stake::sole(crate::state::Holder::Player),
            name,
            kind: CompanyKind::Player,
            headquarters: settings.start_country,
            founded: date,
            rng: SimRng::for_stream(settings.seed, Stream::Company(0)),
            ledger: Ledger::new(date, settings.start_capital),
            technologies: BTreeSet::new(),
            bankrupt: false,
            loans: Vec::new(),
            loss_carryforward: Money::ZERO,
            sales_policies: Vec::new(),
            research: Default::default(),
            ai: None,
        };
        let mut state = GameState {
            world_rng: SimRng::for_stream(settings.seed, Stream::World),
            settings,
            date,
            countries: PerId::default(),
            companies: vec![player],
            sites: Vec::new(),
            markets: PerId::default(),
            shipments: Vec::new(),
            routes: Default::default(),
            import_markets: Default::default(),
            deposits: PerId::default(),
            inventions: PerId::default(),
            player: CompanyId(0),
            game_over: false,
        };
        state.refresh_countries(&catalog);
        state.fit_to_catalog(&catalog);
        market::initial_demand(&mut state, &catalog, date);
        apply_start_setup(&mut state, &catalog)?;
        crate::population::populate(&mut state, &catalog);
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

    pub fn player(&self) -> CompanyId {
        self.state.player
    }

    pub fn is_over(&self) -> bool {
        self.state.game_over
    }

    /// Executes a decision of the player.
    pub fn apply(&mut self, command: Command) -> Result<(), CommandError> {
        self.apply_as(self.state.player, command)
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
        report
            .messages
            .extend(crate::ai::decide(&mut self.state, &self.catalog, today));
        trade::deliver(&mut self.state, today);
        production::simulate_day(&mut self.state, &self.catalog, today);
        market::clear(&mut self.state, &self.catalog, today);
        report.messages.extend(research::simulate_day(
            &mut self.state,
            &self.catalog,
            today,
        ));
        report.messages.extend(world_events(&self.catalog, today));

        let next = today.next_day();
        if next.day() == 1 {
            market::settle_all_idle(&mut self.state, &self.catalog, next);
        }
        self.state.date = next;
        if next.day() == 1 {
            finance::month_end(&mut self.state, &self.catalog, today);
            for company in &mut self.state.companies {
                company.ledger.close_month(next);
            }
            report
                .messages
                .extend(finance::check_insolvency(&mut self.state, &self.catalog));
            self.state.refresh_countries(&self.catalog);
            production::new_month(&mut self.state);
            market::month_start(&mut self.state, &self.catalog, next);
            market::reset_site_months(&mut self.state);
            crate::brand::month_start(&mut self.state, &self.catalog, next);
        }
        if next.ordinal() == 1 {
            production::new_year(&mut self.state);
        }
        if next.ordinal() == 1 && next < GAME_END {
            report.messages.push(
                Message::new(MessageKind::Info, keys::NEW_YEAR)
                    .with("jahr", Param::Integer(i64::from(next.year()))),
            );
        }
    }

    pub fn state_hash(&self) -> StateHash {
        let bytes = rmp_serde::to_vec_named(&self.state).expect("state is serializable");
        StateHash(fnv1a(&bytes))
    }
}

/// Historical events of the day as world news (Lastenheft §4.1, §13.2).
fn world_events(catalog: &Catalog, date: Date) -> Vec<Message> {
    let first = catalog.events.partition_point(|e| e.date < date);
    catalog.events[first..]
        .iter()
        .take_while(|e| e.date == date)
        .map(|e| {
            Message::new(MessageKind::WorldEvent, keys::WORLD_EVENT)
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
                )
        })
        .collect()
}

/// Gives the new company the site of its start form (Lastenheft §2, §15), paid from
/// the start capital: a small workshop that makes simple parts, or a trading office.
fn apply_start_setup(state: &mut GameState, catalog: &Catalog) -> Result<(), NewGameError> {
    let Some(setup) = catalog
        .production_model
        .start_setup(state.settings.start_form)
    else {
        return Ok(());
    };
    let cost = setup.cost(catalog);
    if cost > state.settings.start_capital {
        return Err(NewGameError::StartFormTooExpensive { needed: cost });
    }
    let date = state.date;
    let country = state.settings.start_country;
    // The facilities are completed on the first day (`complete_constructions`).
    let facilities: Money = setup
        .facilities
        .iter()
        .map(|&(f, _, _)| catalog.facilities.get(f).investment)
        .sum();
    let ledger = &mut state.companies[state.player.index()].ledger;
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
    state.sites.push(Site {
        owner: state.player,
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
        wage_premium: 0.0,
    });
    Ok(())
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xCBF2_9CE4_8422_2325, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3)
    })
}
