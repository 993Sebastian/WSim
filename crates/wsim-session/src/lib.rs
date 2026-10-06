//! A game session for user interfaces: the loaded data, the running game and the
//! saves (files on disk, or the browser's storage in the web version). It translates
//! requests of the interface into settings and commands of the core and answers with
//! views (`wsim_core::views`); errors are messages with text keys. No game logic lives
//! here.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wsim_core::calendar::Date;
use wsim_core::calendar::RoundLength;
use wsim_core::catalog::Catalog;
use wsim_core::command::Command;
use wsim_core::game::{Game, Progress, RoundReport};
use wsim_core::message::{Message, MessageKind, Param};
use wsim_core::money::Money;
use wsim_core::save;
use wsim_core::state::{AiSettings, GameSettings};
use wsim_core::views::{
    self, ChainsView, CompaniesView, CompanyDetailView, CountryDetail, FinanceView,
    ManagerMarketView, MarketView, MessageView, NewGameOptions, OffersView, OrganisationView,
    Overview, ProductMarketView, ProductionView, ResearchOverview, RoundReportView, WorldMap,
    WorldMarketView,
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

/// Where the saves live, by name: files on disk (desktop) or memory that the browser
/// persists (web version).
pub trait SaveStore {
    fn write(&mut self, name: &str, bytes: Vec<u8>) -> Result<(), String>;
    fn read(&self, name: &str) -> Result<Vec<u8>, String>;
    /// Names of all saves.
    fn names(&self) -> Vec<String>;
}

/// Saves as files `<name>.wsim` in a directory.
pub struct DirStore {
    dir: PathBuf,
}

impl DirStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn file(&self, name: &str) -> PathBuf {
        self.dir.join(format!("{name}.{SAVE_EXTENSION}"))
    }
}

impl SaveStore for DirStore {
    fn write(&mut self, name: &str, bytes: Vec<u8>) -> Result<(), String> {
        fs::create_dir_all(&self.dir)
            .and_then(|()| fs::write(self.file(name), bytes))
            .map_err(|e| e.to_string())
    }

    fn read(&self, name: &str) -> Result<Vec<u8>, String> {
        fs::read(self.file(name)).map_err(|e| e.to_string())
    }

    fn names(&self) -> Vec<String> {
        let Ok(dir) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        dir.filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                (path.extension()? == SAVE_EXTENSION)
                    .then(|| path.file_stem().map(|s| s.to_string_lossy().into_owned()))?
            })
            .collect()
    }
}

/// Saves in memory. The web version fills it from the browser's storage at the start
/// and writes back what `take_written` hands out.
#[derive(Default)]
pub struct MemoryStore {
    saves: BTreeMap<String, Vec<u8>>,
    written: Vec<String>,
}

impl MemoryStore {
    /// Names of the saves written since the last call, for persisting them elsewhere.
    pub fn take_written(&mut self) -> Vec<String> {
        std::mem::take(&mut self.written)
    }

    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.saves.get(name).map(Vec::as_slice)
    }

    /// Adds a save kept elsewhere (without marking it as written).
    pub fn put(&mut self, name: &str, bytes: Vec<u8>) {
        self.saves.insert(name.to_owned(), bytes);
    }
}

impl SaveStore for MemoryStore {
    fn write(&mut self, name: &str, bytes: Vec<u8>) -> Result<(), String> {
        self.saves.insert(name.to_owned(), bytes);
        if !self.written.iter().any(|n| n == name) {
            self.written.push(name.to_owned());
        }
        Ok(())
    }

    fn read(&self, name: &str) -> Result<Vec<u8>, String> {
        self.saves
            .get(name)
            .cloned()
            .ok_or_else(|| format!("„{name}“ fehlt"))
    }

    fn names(&self) -> Vec<String> {
        self.saves.keys().cloned().collect()
    }
}

pub struct Session<S: SaveStore = DirStore> {
    catalog: Arc<Catalog>,
    store: S,
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

impl Session<DirStore> {
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
        Session::with_store(catalog, DirStore::new(saves))
    }
}

impl<S: SaveStore> Session<S> {
    pub fn with_store(catalog: Arc<Catalog>, store: S) -> Self {
        Self {
            catalog,
            store,
            game: None,
            last_period: None,
        }
    }

    pub fn store_mut(&mut self) -> &mut S {
        &mut self.store
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

    /// One product on the market of a country.
    pub fn product_market(
        &self,
        country: &str,
        product: &str,
    ) -> Result<ProductMarketView, MessageView> {
        let game = self.game.as_ref().ok_or_else(|| error(keys::NO_GAME))?;
        if game.catalog().countries.id(country).is_none() {
            return Err(error(keys::UNKNOWN_COUNTRY));
        }
        views::product_market(game, country, product).ok_or_else(|| error(keys::UNKNOWN_PRODUCT))
    }

    /// Production chains of the end products (M25).
    pub fn chains(&self) -> Result<ChainsView, MessageView> {
        self.view(views::chains)
    }

    /// The player's offers to buy and sell (M30).
    pub fn offers(&self) -> Result<OffersView, MessageView> {
        self.view(views::offers)
    }

    /// The active companies, largest equity first (M30).
    pub fn companies(&self) -> Result<CompaniesView, MessageView> {
        self.view(views::companies)
    }

    /// A company with its sites and the licences the player could buy (M30).
    pub fn company(&self, index: u32) -> Result<CompanyDetailView, MessageView> {
        self.view(|g| views::company_detail(g, index))?
            .ok_or_else(|| error(keys::UNKNOWN_COMPANY))
    }

    /// The player's positions and their managers (MA1).
    pub fn organisation(&self) -> Result<OrganisationView, MessageView> {
        self.view(views::organisation)
    }

    /// Candidates for a position of the player: `role` is `leitung` or a function (MA1).
    pub fn manager_market(&self, site: u32, role: &str) -> Result<ManagerMarketView, MessageView> {
        self.view(|g| views::manager_market(g, site, role))?
            .ok_or_else(|| error(keys::UNKNOWN_POSITION))
    }

    /// One product in all countries (world map).
    pub fn world_market(&self, product: &str) -> Result<WorldMarketView, MessageView> {
        self.view(|g| views::world_market(g, product))?
            .ok_or_else(|| error(keys::UNKNOWN_PRODUCT))
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
        self.end_round_until(length, "runde", progress)
    }

    /// Simulates rounds of `length` one after the other (M26): one (`runde`), up to the
    /// end of the year (`jahresende`), or up to the next warning or world event, at most
    /// a year (`meldung`). The report covers all of them.
    pub fn end_round_until(
        &mut self,
        length: &str,
        until: &str,
        mut progress: impl FnMut(Progress),
    ) -> Result<RoundReportView, MessageView> {
        let length = match length {
            "tag" => RoundLength::Day,
            "woche" => RoundLength::Week,
            "monat" => RoundLength::Month,
            "quartal" => RoundLength::Quarter,
            _ => return Err(error(keys::UNKNOWN_ROUND_LENGTH)),
        };
        if !["runde", "jahresende", "meldung"].contains(&until) {
            return Err(error(keys::UNKNOWN_UNTIL));
        }
        let game = self.game.as_mut().ok_or_else(|| error(keys::NO_GAME))?;
        let before = views::snapshot(game);
        let start = game.date();
        let year_later = Date::new(start.year() + 1, start.month(), 1).unwrap_or(start);
        let mut all = RoundReport {
            from: start,
            to: start,
            days: 0,
            messages: Vec::new(),
        };
        let mut rounds = 0;
        let stop = loop {
            let report = game.advance(length, &mut progress);
            rounds += 1;
            let news = report.messages.iter().find_map(|m| match m.kind {
                MessageKind::WorldEvent => Some("weltereignis"),
                MessageKind::Warning | MessageKind::Crisis => Some("warnung"),
                // An offer waits for the player's answer (M30).
                _ if m.key.starts_with("meldung.angebot.erhalten")
                    || m.key.starts_with("meldung.angebot.gegenangebot") =>
                {
                    Some("angebot")
                }
                _ => None,
            });
            all.to = report.to;
            all.days += report.days;
            all.messages.extend(report.messages);
            let next_year = game.date().year() > start.year();
            let stop = if game.is_over() || report.days == 0 {
                Some("spielende")
            } else {
                match until {
                    "jahresende" => next_year.then_some("jahresende"),
                    "meldung" => news.or((game.date() >= year_later).then_some("ein_jahr")),
                    _ => Some("runde"),
                }
            };
            if let Some(stop) = stop {
                break stop;
            }
        };
        let mut view = views::round_report(game, &all, &before);
        view.rounds = rounds;
        view.stop = (rounds > 1 || stop != "runde").then(|| stop.to_owned());
        view.previous = self.last_period.replace(view.period.clone());
        if let Err(e) = self.save(AUTOSAVE_NAME) {
            view.messages.push(e);
        }
        Ok(view)
    }

    /// The checked name of a save (it becomes a file name on the desktop).
    fn checked_name(name: &str) -> Result<&str, MessageView> {
        let name = name.trim();
        let valid = !name.is_empty()
            && name.chars().count() <= MAX_SAVE_NAME
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'))
            && !name.starts_with('.');
        if valid {
            Ok(name)
        } else {
            Err(error(keys::INVALID_SAVE_NAME))
        }
    }

    pub fn save(&mut self, name: &str) -> Result<SaveEntry, MessageView> {
        let game = self.game.as_ref().ok_or_else(|| error(keys::NO_GAME))?;
        let name = Self::checked_name(name)?;
        let bytes = save::encode(game);
        let o = views::overview(game);
        self.store
            .write(name, bytes)
            .map_err(|e| error_with(keys::SAVE_FAILED, "fehler", e))?;
        Ok(SaveEntry {
            name: name.to_owned(),
            date: o.date,
            company: o.company.name,
        })
    }

    /// All saves, newest game date first.
    pub fn saves(&self) -> Vec<SaveEntry> {
        let mut list: Vec<SaveEntry> = self
            .store
            .names()
            .into_iter()
            .filter_map(|name| {
                let bytes = self.store.read(&name).ok()?;
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
        let name = Self::checked_name(name)?;
        let bytes = self
            .store
            .read(name)
            .map_err(|e| error_with(keys::LOAD_FAILED, "fehler", e))?;
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
    pub const UNKNOWN_PRODUCT: &str = "fehler.sitzung.unbekanntes_produkt";
    pub const UNKNOWN_UNTIL: &str = "fehler.sitzung.unbekanntes_ziel";
    pub const UNKNOWN_COMPANY: &str = "fehler.sitzung.unbekannte_firma";
    pub const UNKNOWN_POSITION: &str = "fehler.sitzung.unbekannte_stelle";

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
        UNKNOWN_PRODUCT,
        UNKNOWN_UNTIL,
        UNKNOWN_COMPANY,
        UNKNOWN_POSITION,
    ];
}
