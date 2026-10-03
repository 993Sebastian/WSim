//! A running game: catalog, state, journal of decisions, and the round loop
//! (Lastenheft §13).

use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::EARLIEST_START_YEAR;
use crate::calendar::{Date, GAME_END, RoundLength};
use crate::catalog::Catalog;
use crate::command::{self, Command, CommandError, NameError};
use crate::ids::Id;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{Company, CompanyId, CompanyKind, CountryState, GameSettings, GameState, PerId};

/// Latest selectable start year; technology freezes in 2026 (Lastenheft §3.1).
pub const LATEST_START_YEAR: i32 = 2026;
/// Highest start capital in USD.
pub const MAX_START_CAPITAL_USD: f64 = 1.0e12;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NewGameError {
    StartYear { year: i32 },
    StartCapital,
    StartCountry,
    Name(NameError),
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
        let name = command::check_company_name(None, &settings.company_name, None)
            .map_err(NewGameError::Name)?;

        let date = Date::first_of_year(settings.start_year);
        let countries = PerId::from_fn(catalog.countries.len(), |id| {
            CountryState::at(&catalog, id, date)
        });
        let player = Company {
            name,
            kind: CompanyKind::Player,
            headquarters: settings.start_country,
            cash: settings.start_capital,
            founded: date,
            rng: SimRng::for_stream(settings.seed, Stream::Company(0)),
        };
        let state = GameState {
            world_rng: SimRng::for_stream(settings.seed, Stream::World),
            settings,
            date,
            countries,
            companies: vec![player],
            player: CompanyId(0),
            game_over: false,
        };
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
        let next = self.state.date.next_day();
        self.state.date = next;
        for (id, country) in self.state.countries.iter_mut() {
            *country = CountryState::at(&self.catalog, id, next);
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

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xCBF2_9CE4_8422_2325, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3)
    })
}
