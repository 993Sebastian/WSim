//! Desktop shell of WSim. Thin adapter between the simulation core and the UI:
//! it forwards requests to the game session and returns views, but contains no game
//! logic.

// Hide the console window in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use wsim_core::CoreInfo;
use wsim_core::views::{
    BankView, ChainsView, CompaniesView, CompanyDetailView, ConcernsView, ContractPartnersView,
    ContractsView, ControllingView, CountryDetail, FinanceView, GroupView, LogisticsView,
    ManagerMarketView, MarketView, MessageView, NewGameOptions, OffersView, OrganisationView,
    Overview, PersonView, ProductMarketView, ProductionView, ResearchOverview, ReviewsView,
    RoundReportView, StockMarketView, StrategyView, VenturesView, WorldMap, WorldMarketView,
};
use wsim_session::{NewGameRequest, SaveEntry, Session};

/// The session, or why the data could not be loaded.
type Shared = Arc<Mutex<Result<Session, String>>>;

/// The player's stop button: outside the session, which a running round holds.
#[derive(Default)]
struct Anhalten(Arc<AtomicBool>);

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

/// Where panics are written; the release build has no console (Windows).
static FEHLERPROTOKOLL: OnceLock<PathBuf> = OnceLock::new();

fn protokolliere(text: &str) {
    let Some(pfad) = FEHLERPROTOKOLL.get() else {
        return;
    };
    if let Some(ordner) = pfad.parent() {
        let _ = std::fs::create_dir_all(ordner);
    }
    if let Ok(mut datei) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(pfad)
    {
        let _ = writeln!(datei, "{text}\n");
    }
}

fn mit_sitzung<T>(
    state: &Shared,
    f: impl FnOnce(&mut Session) -> Result<T, MessageView>,
) -> Result<T, Fehler> {
    // A panic in an earlier request must not lock the game for good.
    let mut guard = state
        .lock()
        .unwrap_or_else(|vergiftet| vergiftet.into_inner());
    match guard.as_mut() {
        Ok(session) => match catch_unwind(AssertUnwindSafe(|| f(session))) {
            Ok(antwort) => antwort.map_err(Fehler::from),
            Err(_) => {
                let ort = FEHLERPROTOKOLL
                    .get()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                Err(Fehler::Text(format!(
                    "Interner Fehler im Simulationskern. Bitte speichern und das Spiel neu \
                     starten; Einzelheiten stehen in {ort}."
                )))
            }
        },
        Err(text) => Err(Fehler::Text(text.clone())),
    }
}

/// Runs a request off the window's main thread: a long round must not freeze the window
/// while another view waits for the session.
async fn im_hintergrund<T: Send + 'static>(
    state: &Shared,
    f: impl FnOnce(&mut Session) -> Result<T, MessageView> + Send + 'static,
) -> Result<T, Fehler> {
    let shared = state.clone();
    tauri::async_runtime::spawn_blocking(move || mit_sitzung(&shared, f))
        .await
        .map_err(|e| Fehler::Text(e.to_string()))?
}

#[tauri::command]
fn kern_info() -> CoreInfo {
    wsim_core::core_info()
}

#[tauri::command]
async fn optionen(state: State<'_, Shared>) -> Result<NewGameOptions, Fehler> {
    im_hintergrund(state.inner(), move |s| Ok(s.options())).await
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
async fn uebersicht(state: State<'_, Shared>) -> Result<Overview, Fehler> {
    im_hintergrund(state.inner(), move |s| s.overview()).await
}

#[tauri::command]
async fn runde_beenden(
    app: AppHandle,
    state: State<'_, Shared>,
    anhalten_signal: State<'_, Anhalten>,
    laenge: String,
    bis: Option<String>,
    anhalten: Option<String>,
) -> Result<RoundReportView, Fehler> {
    let shared = state.inner().clone();
    let signal = anhalten_signal.0.clone();
    signal.store(false, Ordering::SeqCst);
    tauri::async_runtime::spawn_blocking(move || {
        mit_sitzung(&shared, |s| {
            let bis = bis.as_deref().unwrap_or("runde");
            let anhalten = anhalten.as_deref().unwrap_or("wichtige");
            s.end_rounds_until(
                &laenge,
                bis,
                anhalten,
                |p| {
                    // A lost progress event only delays the bar.
                    let _ = app.emit(
                        "fortschritt",
                        Fortschritt {
                            done: p.done,
                            total: p.total,
                        },
                    );
                },
                || signal.load(Ordering::SeqCst),
            )
        })
    })
    .await
    .map_err(|e| Fehler::Text(e.to_string()))?
}

/// Stops the running round after the current day.
#[tauri::command]
fn runde_anhalten(anhalten_signal: State<'_, Anhalten>) {
    anhalten_signal.0.store(true, Ordering::SeqCst);
}

#[tauri::command]
async fn weltkarte(state: State<'_, Shared>) -> Result<WorldMap, Fehler> {
    im_hintergrund(state.inner(), move |s| s.world_map()).await
}

#[tauri::command]
async fn land(state: State<'_, Shared>, schluessel: String) -> Result<CountryDetail, Fehler> {
    im_hintergrund(state.inner(), move |s| s.country(&schluessel)).await
}

#[tauri::command]
async fn produktion(state: State<'_, Shared>) -> Result<ProductionView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.production()).await
}

#[tauri::command]
async fn markt(state: State<'_, Shared>, land: String) -> Result<MarketView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.market(&land)).await
}

#[tauri::command]
async fn produktmarkt(
    state: State<'_, Shared>,
    land: String,
    produkt: String,
) -> Result<ProductMarketView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.product_market(&land, &produkt)).await
}

#[tauri::command]
async fn ketten(state: State<'_, Shared>) -> Result<ChainsView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.chains()).await
}

#[tauri::command]
async fn angebote(state: State<'_, Shared>) -> Result<OffersView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.offers()).await
}

#[tauri::command]
async fn firmen(state: State<'_, Shared>) -> Result<CompaniesView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.companies()).await
}

#[tauri::command]
async fn firma(state: State<'_, Shared>, index: u32) -> Result<CompanyDetailView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.company(index)).await
}

#[tauri::command]
async fn weltmarkt(state: State<'_, Shared>, produkt: String) -> Result<WorldMarketView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.world_market(&produkt)).await
}

#[tauri::command]
async fn forschung(state: State<'_, Shared>) -> Result<ResearchOverview, Fehler> {
    im_hintergrund(state.inner(), move |s| s.research()).await
}

#[tauri::command]
async fn finanzen(state: State<'_, Shared>) -> Result<FinanceView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.finance()).await
}

#[tauri::command]
async fn organisation(state: State<'_, Shared>) -> Result<OrganisationView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.organisation()).await
}

#[tauri::command]
async fn anliegen(state: State<'_, Shared>) -> Result<ConcernsView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.concerns()).await
}

#[tauri::command]
async fn strategie(state: State<'_, Shared>) -> Result<StrategyView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.strategy()).await
}

#[tauri::command]
async fn ruecksprache(state: State<'_, Shared>) -> Result<ReviewsView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.reviews()).await
}

#[tauri::command]
async fn startups(state: State<'_, Shared>) -> Result<VenturesView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.ventures()).await
}

#[tauri::command]
async fn controlling(
    state: State<'_, Shared>,
    zeitraum: String,
) -> Result<ControllingView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.controlling(&zeitraum)).await
}

#[tauri::command]
async fn bank(state: State<'_, Shared>) -> Result<BankView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.bank()).await
}

#[tauri::command]
async fn person(state: State<'_, Shared>) -> Result<PersonView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.person()).await
}

#[tauri::command]
async fn boerse(state: State<'_, Shared>) -> Result<StockMarketView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.stock()).await
}

#[tauri::command]
async fn konzern(state: State<'_, Shared>) -> Result<GroupView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.group()).await
}

#[tauri::command]
async fn logistik(state: State<'_, Shared>) -> Result<LogisticsView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.logistics()).await
}

#[tauri::command]
async fn vertraege(state: State<'_, Shared>) -> Result<ContractsView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.contracts()).await
}

#[tauri::command]
async fn vertragspartner(
    state: State<'_, Shared>,
    standort: u32,
    produkt: String,
) -> Result<ContractPartnersView, Fehler> {
    im_hintergrund(state.inner(), move |s| {
        s.contract_partners(standort, &produkt)
    })
    .await
}

#[tauri::command]
async fn managermarkt(
    state: State<'_, Shared>,
    einheit: String,
    stelle: String,
) -> Result<ManagerMarketView, Fehler> {
    im_hintergrund(state.inner(), move |s| s.manager_market(&einheit, &stelle)).await
}

/// A decision of the player, as JSON command with keys (see `Session::command`).
#[tauri::command]
async fn befehl(state: State<'_, Shared>, befehl: serde_json::Value) -> Result<Overview, Fehler> {
    im_hintergrund(state.inner(), move |s| s.command(befehl)).await
}

#[tauri::command]
async fn speichern(state: State<'_, Shared>, name: String) -> Result<SaveEntry, Fehler> {
    im_hintergrund(state.inner(), move |s| s.save(&name)).await
}

#[tauri::command]
async fn spielstaende(state: State<'_, Shared>) -> Result<Vec<SaveEntry>, Fehler> {
    im_hintergrund(state.inner(), move |s| Ok(s.saves())).await
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
            if let Ok(ordner) = handle.path().app_local_data_dir() {
                let _ = FEHLERPROTOKOLL.set(ordner.join("wsim-fehler.log"));
            }
            let standard = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                let rueckverfolgung = std::backtrace::Backtrace::force_capture();
                protokolliere(&format!("{info}\n{rueckverfolgung}"));
                standard(info);
            }));
            let saves = handle
                .path()
                .app_local_data_dir()
                .map(|d| d.join("spielstaende"))
                .unwrap_or_else(|_| PathBuf::from("spielstaende"));
            let session = Session::open(&datenverzeichnis(handle), saves);
            let shared: Shared = Arc::new(Mutex::new(session));
            app.manage(shared);
            app.manage(Anhalten::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            kern_info,
            optionen,
            neues_spiel,
            uebersicht,
            runde_beenden,
            runde_anhalten,
            weltkarte,
            land,
            produktion,
            markt,
            produktmarkt,
            ketten,
            angebote,
            firmen,
            firma,
            weltmarkt,
            forschung,
            finanzen,
            organisation,
            anliegen,
            strategie,
            ruecksprache,
            startups,
            vertraege,
            vertragspartner,
            logistik,
            konzern,
            controlling,
            boerse,
            bank,
            person,
            managermarkt,
            befehl,
            speichern,
            spielstaende,
            laden
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Start von WSim");
}
