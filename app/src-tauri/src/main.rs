//! Desktop shell of WSim. Thin adapter between the simulation core and the UI:
//! it forwards requests to the game session and returns views, but contains no game
//! logic.

// Hide the console window in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use wsim_core::CoreInfo;
use wsim_core::views::{
    ChainsView, CountryDetail, FinanceView, MarketView, MessageView, NewGameOptions, Overview,
    ProductMarketView, ProductionView, ResearchOverview, RoundReportView, WorldMap,
    WorldMarketView,
};
use wsim_session::{NewGameRequest, SaveEntry, Session};

/// The session, or why the data could not be loaded.
type Shared = Arc<Mutex<Result<Session, String>>>;

/// Error answer for the UI: a message with text key, or a plain text (data errors).
#[derive(Serialize)]
#[serde(untagged)]
enum Fehler {
    Meldung(MessageView),
    Text(String),
}

impl From<MessageView> for Fehler {
    fn from(m: MessageView) -> Self {
        Fehler::Meldung(m)
    }
}

#[derive(Clone, Serialize)]
struct Fortschritt {
    done: u32,
    total: u32,
}

fn mit_sitzung<T>(
    state: &Shared,
    f: impl FnOnce(&mut Session) -> Result<T, MessageView>,
) -> Result<T, Fehler> {
    let mut guard = state.lock().map_err(|e| Fehler::Text(e.to_string()))?;
    match guard.as_mut() {
        Ok(session) => f(session).map_err(Fehler::from),
        Err(text) => Err(Fehler::Text(text.clone())),
    }
}

#[tauri::command]
fn kern_info() -> CoreInfo {
    wsim_core::core_info()
}

#[tauri::command]
fn optionen(state: State<'_, Shared>) -> Result<NewGameOptions, Fehler> {
    mit_sitzung(&state, |s| Ok(s.options()))
}

#[tauri::command]
async fn neues_spiel(
    state: State<'_, Shared>,
    einstellungen: NewGameRequest,
) -> Result<Overview, Fehler> {
    let shared = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        mit_sitzung(&shared, |s| s.new_game(&einstellungen))
    })
    .await
    .map_err(|e| Fehler::Text(e.to_string()))?
}

#[tauri::command]
fn uebersicht(state: State<'_, Shared>) -> Result<Overview, Fehler> {
    mit_sitzung(&state, |s| s.overview())
}

#[tauri::command]
async fn runde_beenden(
    app: AppHandle,
    state: State<'_, Shared>,
    laenge: String,
) -> Result<RoundReportView, Fehler> {
    let shared = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        mit_sitzung(&shared, |s| {
            s.end_round(&laenge, |p| {
                // A lost progress event only delays the bar.
                let _ = app.emit(
                    "fortschritt",
                    Fortschritt {
                        done: p.done,
                        total: p.total,
                    },
                );
            })
        })
    })
    .await
    .map_err(|e| Fehler::Text(e.to_string()))?
}

#[tauri::command]
fn weltkarte(state: State<'_, Shared>) -> Result<WorldMap, Fehler> {
    mit_sitzung(&state, |s| s.world_map())
}

#[tauri::command]
fn land(state: State<'_, Shared>, schluessel: String) -> Result<CountryDetail, Fehler> {
    mit_sitzung(&state, |s| s.country(&schluessel))
}

#[tauri::command]
fn produktion(state: State<'_, Shared>) -> Result<ProductionView, Fehler> {
    mit_sitzung(&state, |s| s.production())
}

#[tauri::command]
fn markt(state: State<'_, Shared>, land: String) -> Result<MarketView, Fehler> {
    mit_sitzung(&state, |s| s.market(&land))
}

#[tauri::command]
fn produktmarkt(
    state: State<'_, Shared>,
    land: String,
    produkt: String,
) -> Result<ProductMarketView, Fehler> {
    mit_sitzung(&state, |s| s.product_market(&land, &produkt))
}

#[tauri::command]
fn ketten(state: State<'_, Shared>) -> Result<ChainsView, Fehler> {
    mit_sitzung(&state, |s| s.chains())
}

#[tauri::command]
fn weltmarkt(state: State<'_, Shared>, produkt: String) -> Result<WorldMarketView, Fehler> {
    mit_sitzung(&state, |s| s.world_market(&produkt))
}

#[tauri::command]
fn forschung(state: State<'_, Shared>) -> Result<ResearchOverview, Fehler> {
    mit_sitzung(&state, |s| s.research())
}

#[tauri::command]
fn finanzen(state: State<'_, Shared>) -> Result<FinanceView, Fehler> {
    mit_sitzung(&state, |s| s.finance())
}

/// A decision of the player, as JSON command with keys (see `Session::command`).
#[tauri::command]
fn befehl(state: State<'_, Shared>, befehl: serde_json::Value) -> Result<Overview, Fehler> {
    mit_sitzung(&state, |s| s.command(befehl))
}

#[tauri::command]
fn speichern(state: State<'_, Shared>, name: String) -> Result<SaveEntry, Fehler> {
    mit_sitzung(&state, |s| s.save(&name))
}

#[tauri::command]
fn spielstaende(state: State<'_, Shared>) -> Result<Vec<SaveEntry>, Fehler> {
    mit_sitzung(&state, |s| Ok(s.saves()))
}

#[tauri::command]
async fn laden(state: State<'_, Shared>, name: String) -> Result<Overview, Fehler> {
    let shared = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || mit_sitzung(&shared, |s| s.load(&name)))
        .await
        .map_err(|e| Fehler::Text(e.to_string()))?
}

/// The game data: bundled with the installer, during development from the repository.
fn datenverzeichnis(app: &AppHandle) -> PathBuf {
    let bundled = app.path().resource_dir().map(|d| d.join("data"));
    match bundled {
        Ok(dir) if dir.join("meta.yaml").exists() => dir,
        _ => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data"),
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();
            let saves = handle
                .path()
                .app_local_data_dir()
                .map(|d| d.join("spielstaende"))
                .unwrap_or_else(|_| PathBuf::from("spielstaende"));
            let session = Session::open(&datenverzeichnis(handle), saves);
            let shared: Shared = Arc::new(Mutex::new(session));
            app.manage(shared);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            kern_info,
            optionen,
            neues_spiel,
            uebersicht,
            runde_beenden,
            weltkarte,
            land,
            produktion,
            markt,
            produktmarkt,
            ketten,
            weltmarkt,
            forschung,
            finanzen,
            befehl,
            speichern,
            spielstaende,
            laden
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Start von WSim");
}
