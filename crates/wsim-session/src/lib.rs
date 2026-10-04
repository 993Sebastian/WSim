//! A game session for user interfaces: the loaded data, the running game and the
//! saves on disk. It translates requests of the interface into settings and commands
//! of the core and answers with views (`wsim_core::views`); errors are messages with
//! text keys. No game logic lives here.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wsim_core::calendar::RoundLength;
use wsim_core::catalog::Catalog;
use wsim_core::command::Command;
use wsim_core::game::{Game, Progress};
use wsim_core::message::{Message, Param};
use wsim_core::money::Money;
use wsim_core::save;
use wsim_core::state::{AiSettings, GameSettings};
use wsim_core::views::{
    self, CountryDetail, FinanceView, MarketView, MessageView, NewGameOptions, Overview,
    ProductionView, ResearchOverview, RoundReportView, WorldMap,
};

/// File extension of saves.
pub const SAVE_EXTENSION: &str = "wsim";
/// Longest name of a save.
pub const MAX_SAVE_NAME: usize = 60;

/// What the new-game dialog sends (Lastenheft §15).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewGameRequest {
    pub seed: u64,
    pub start_year: i32,
    pub country: String,
    pub capital_usd: f64,
    pub start_form: String,
    pub company_name: String,
    pub companies: u32,
    pub difficulty: String,
    pub research_factor: f64,
}

/// One save in the list of the load dialog.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaveEntry {
    pub name: String,
    pub date: String,
    pub company: String,
}

/// Name of the save written after every round (Lastenheft §13.1).
pub const AUTOSAVE_NAME: &str = "Automatisch";

pub struct Session {
    catalog: Arc<Catalog>,
    saves: PathBuf,
    game: Option<Game>,
    /// Result of the last round, for the comparison in the next report.
    last_period: Option<views::PeriodView>,
}

fn error(key: &str) -> MessageView {
    views::message_view(&Message::error(key))
}

fn error_with(key: &str, name: &str, value: String) -> MessageView {
    views::message_view(&Message::error(key).with(name, Param::Text(value)))
}

impl Session {
    /// Loads and checks the data; on errors returns the report as text.
    pub fn open(data: &Path, saves: PathBuf) -> Result<Self, String> {
        let outcome = wsim_data::load_dir(data);
        let Some(loaded) = outcome.data else {
            let text: Vec<String> = outcome.report.errors().map(ToString::to_string).collect();
            return Err(text.join("\n\n"));
        };
        Ok(Self::with_catalog(Arc::new(loaded.catalog), saves))
    }

    pub fn with_catalog(catalog: Arc<Catalog>, saves: PathBuf) -> Self {
        Self {
            catalog,
            saves,
            game: None,
            last_period: None,
        }
    }

    pub fn options(&self) -> NewGameOptions {
        views::new_game_options(&self.catalog)
    }

    pub fn game(&self) -> Option<&Game> {
        self.game.as_ref()
    }

    pub fn new_game(&mut self, request: &NewGameRequest) -> Result<Overview, MessageView> {
        let c = &self.catalog;
        let start_country = c
            .countries
            .id(&request.country)
            .ok_or_else(|| error(keys::UNKNOWN_COUNTRY))?;
        let start_form = views::start_form_from_key(&request.start_form)
            .ok_or_else(|| error(keys::UNKNOWN_START_FORM))?;
        let difficulty = c
            .ai_model
            .difficulties
            .iter()
            .find(|d| d.key == request.difficulty)
            .ok_or_else(|| error(keys::UNKNOWN_DIFFICULTY))?;
        let settings = GameSettings {
            seed: request.seed,
            start_year: request.start_year,
            start_country,
            start_capital: Money::from_usd(request.capital_usd).unwrap_or(Money::ZERO),
            start_form,
            company_name: request.company_name.clone(),
            research_ahead_factor: request.research_factor,
            market_scale: 1.0,
            ai: AiSettings {
                companies: request.companies,
                competence: difficulty.competence,
                aggressiveness: difficulty.aggressiveness,
            },
        };
        let game = Game::new(c.clone(), settings).map_err(|e| views::message_view(&e.message()))?;
        let overview = views::overview(&game);
        self.game = Some(game);
        self.last_period = None;
        Ok(overview)
    }

    pub fn overview(&self) -> Result<Overview, MessageView> {
        self.game
            .as_ref()
            .map(views::overview)
            .ok_or_else(|| error(keys::NO_GAME))
    }

    pub fn world_map(&self) -> Result<WorldMap, MessageView> {
        self.game
            .as_ref()
            .map(views::world_map)
            .ok_or_else(|| error(keys::NO_GAME))
    }

    pub fn country(&self, key: &str) -> Result<CountryDetail, MessageView> {
        let game = self.game.as_ref().ok_or_else(|| error(keys::NO_GAME))?;
        views::country_detail(game, key).ok_or_else(|| error(keys::UNKNOWN_COUNTRY))
    }

    fn view<T>(&self, f: impl FnOnce(&Game) -> T) -> Result<T, MessageView> {
        self.game
            .as_ref()
            .map(f)
            .ok_or_else(|| error(keys::NO_GAME))
    }

    pub fn production(&self) -> Result<ProductionView, MessageView> {
        self.view(views::production)
    }

    pub fn research(&self) -> Result<ResearchOverview, MessageView> {
        self.view(views::research_overview)
    }

    pub fn finance(&self) -> Result<FinanceView, MessageView> {
        self.view(views::finance_overview)
    }

    pub fn market(&self, country: &str) -> Result<MarketView, MessageView> {
        self.view(|g| views::market(g, country))?
            .ok_or_else(|| error(keys::UNKNOWN_COUNTRY))
    }

    /// Carries out a decision of the player. The command comes as JSON with keys for
    /// content (`{"FoundSite": {"country": "DEU", "kind": "Factory"}}`); it passes the
    /// same checks as every command and goes into the journal.
    pub fn command(&mut self, json: serde_json::Value) -> Result<Overview, MessageView> {
        let game = self.game.as_mut().ok_or_else(|| error(keys::NO_GAME))?;
        let table = game.catalog().key_table();
        let (parsed, missing) =
            wsim_core::ids::with_keys(&table, || serde_json::from_value::<Command>(json));
        let command = match (parsed, missing) {
            (Ok(c), None) => c,
            (Err(e), _) => return Err(error_with(keys::INVALID_COMMAND, "fehler", e.to_string())),
            (_, Some((_, key))) => {
                return Err(error_with(keys::INVALID_COMMAND, "fehler", key));
            }
        };
        game.apply(command)
            .map_err(|e| views::message_view(&e.message()))?;
        Ok(views::overview(game))
    }

    /// Simulates one round of `length` (`tag`, `woche`, `monat`, `quartal`).
    pub fn end_round(
        &mut self,
        length: &str,
        progress: impl FnMut(Progress),
    ) -> Result<RoundReportView, MessageView> {
        let length = match length {
            "tag" => RoundLength::Day,
            "woche" => RoundLength::Week,
            "monat" => RoundLength::Month,
            "quartal" => RoundLength::Quarter,
            _ => return Err(error(keys::UNKNOWN_ROUND_LENGTH)),
        };
        let game = self.game.as_mut().ok_or_else(|| error(keys::NO_GAME))?;
        let before = views::snapshot(game);
        let report = game.advance(length, progress);
        let mut view = views::round_report(game, &report, &before);
        view.previous = self.last_period.replace(view.period.clone());
        if let Err(e) = self.save(AUTOSAVE_NAME) {
            view.messages.push(e);
        }
        Ok(view)
    }

    fn path(&self, name: &str) -> Result<PathBuf, MessageView> {
        let name = name.trim();
        let valid = !name.is_empty()
            && name.chars().count() <= MAX_SAVE_NAME
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'))
            && !name.starts_with('.');
        if !valid {
            return Err(error(keys::INVALID_SAVE_NAME));
        }
        Ok(self.saves.join(format!("{name}.{SAVE_EXTENSION}")))
    }

    pub fn save(&self, name: &str) -> Result<SaveEntry, MessageView> {
        let game = self.game.as_ref().ok_or_else(|| error(keys::NO_GAME))?;
        let path = self.path(name)?;
        fs::create_dir_all(&self.saves)
            .and_then(|()| fs::write(&path, save::encode(game)))
            .map_err(|e| error_with(keys::SAVE_FAILED, "fehler", e.to_string()))?;
        let o = views::overview(game);
        Ok(SaveEntry {
            name: name.trim().to_owned(),
            date: o.date,
            company: o.company.name,
        })
    }

    /// All saves, newest game date first.
    pub fn saves(&self) -> Vec<SaveEntry> {
        let Ok(dir) = fs::read_dir(&self.saves) else {
            return Vec::new();
        };
        let mut list: Vec<SaveEntry> = dir
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                if path.extension()? != SAVE_EXTENSION {
                    return None;
                }
                let name = path.file_stem()?.to_string_lossy().into_owned();
                let bytes = fs::read(&path).ok()?;
                let header = save::read_header(&bytes).ok()?;
                let d = header.date;
                Some(SaveEntry {
                    name,
                    date: format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day()),
                    company: header.company_name,
                })
            })
            .collect();
        list.sort_by(|a, b| b.date.cmp(&a.date).then(a.name.cmp(&b.name)));
        list
    }

    pub fn load(&mut self, name: &str) -> Result<Overview, MessageView> {
        let path = self.path(name)?;
        let bytes =
            fs::read(&path).map_err(|e| error_with(keys::LOAD_FAILED, "fehler", e.to_string()))?;
        let loaded = save::decode(&bytes, self.catalog.clone())
            .map_err(|e| views::message_view(&e.message()))?;
        let overview = views::overview(&loaded.game);
        self.game = Some(loaded.game);
        self.last_period = None;
        Ok(overview)
    }
}

/// Text keys of the session's messages (`data/texte/de/meldungen.yaml`).
pub mod keys {
    pub const NO_GAME: &str = "fehler.sitzung.kein_spiel";
    pub const UNKNOWN_COUNTRY: &str = "fehler.sitzung.unbekanntes_land";
    pub const UNKNOWN_START_FORM: &str = "fehler.sitzung.unbekannte_startform";
    pub const UNKNOWN_DIFFICULTY: &str = "fehler.sitzung.unbekannte_schwierigkeit";
    pub const UNKNOWN_ROUND_LENGTH: &str = "fehler.sitzung.unbekannte_rundenlaenge";
    pub const INVALID_SAVE_NAME: &str = "fehler.sitzung.name_ungueltig";
    pub const SAVE_FAILED: &str = "fehler.sitzung.speichern";
    pub const LOAD_FAILED: &str = "fehler.sitzung.laden";
    pub const INVALID_COMMAND: &str = "fehler.sitzung.befehl_ungueltig";

    pub const ALL: &[&str] = &[
        NO_GAME,
        UNKNOWN_COUNTRY,
        UNKNOWN_START_FORM,
        UNKNOWN_DIFFICULTY,
        UNKNOWN_ROUND_LENGTH,
        INVALID_SAVE_NAME,
        SAVE_FAILED,
        LOAD_FAILED,
        INVALID_COMMAND,
    ];
}
