//! Browser version of WSim: the game session compiled to WebAssembly, with the game
//! data built in. JavaScript sends requests as JSON (`{"op": "...", "args": {...}}`)
//! under the same names as the desktop app's commands and gets `{"ok": ...}` or
//! `{"err": ...}` back; saves travel as bytes and are kept in the browser's storage by
//! the caller. No game logic lives here.

#[cfg(target_arch = "wasm32")]
mod bridge;

use std::cell::RefCell;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use wsim_core::views::MessageView;
use wsim_session::{MemoryStore, NewGameRequest, Session};

include!(concat!(env!("OUT_DIR"), "/daten.rs"));

type WebSession = Session<MemoryStore>;

thread_local! {
    /// The session, or why the data could not be loaded; opened by the first request.
    static SITZUNG: RefCell<Option<Result<WebSession, String>>> = const { RefCell::new(None) };
}

/// Error answer in the shape of the desktop app: a message with text key, or a plain
/// text (data errors, invalid requests).
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

#[derive(Deserialize)]
struct Anfrage {
    op: String,
    #[serde(default)]
    args: Value,
}

fn oeffnen() -> Result<WebSession, String> {
    let sources = DATEN
        .iter()
        .map(|&(path, text)| wsim_data::Source::new(path, text))
        .collect();
    let outcome = wsim_data::load_sources(sources);
    let Some(data) = outcome.data else {
        let text: Vec<String> = outcome.report.errors().map(ToString::to_string).collect();
        return Err(text.join("\n\n"));
    };
    Ok(Session::with_store(
        Arc::new(data.catalog),
        MemoryStore::default(),
    ))
}

/// Runs `f` with the session, opening it on first use.
fn mit_sitzung<T>(f: impl FnOnce(&mut WebSession) -> T) -> Result<T, String> {
    SITZUNG.with(|cell| {
        let mut slot = cell.borrow_mut();
        let session = slot.get_or_insert_with(oeffnen);
        session.as_mut().map(f).map_err(|e| e.clone())
    })
}

fn argument<T: DeserializeOwned>(args: &Value, name: &str) -> Result<T, Fehler> {
    let value = args.get(name).cloned().unwrap_or(Value::Null);
    serde_json::from_value(value).map_err(|e| Fehler::Text(format!("„{name}“: {e}")))
}

fn wert<T: Serialize>(value: T) -> Result<Value, Fehler> {
    serde_json::to_value(value).map_err(|e| Fehler::Text(e.to_string()))
}

fn ausfuehren(anfrage: &Anfrage, fortschritt: &mut dyn FnMut(u32, u32)) -> Result<Value, Fehler> {
    let args = &anfrage.args;
    match anfrage.op.as_str() {
        "kern_info" => wert(wsim_core::core_info()),
        op => {
            let ergebnis = mit_sitzung(|s| -> Result<Value, Fehler> {
                match op {
                    "optionen" => wert(s.options()),
                    "neues_spiel" => {
                        let einstellungen: NewGameRequest = argument(args, "einstellungen")?;
                        wert(s.new_game(&einstellungen)?)
                    }
                    "uebersicht" => wert(s.overview()?),
                    "runde_beenden" => {
                        let laenge: String = argument(args, "laenge")?;
                        let bis: Option<String> = argument(args, "bis")?;
                        let bis = bis.as_deref().unwrap_or("runde");
                        let anhalten: Option<String> = argument(args, "anhalten")?;
                        let anhalten = anhalten.as_deref().unwrap_or("wichtige");
                        wert(s.end_rounds(&laenge, bis, anhalten, |p| {
                            fortschritt(p.done, p.total);
                        })?)
                    }
                    "weltkarte" => wert(s.world_map()?),
                    "land" => {
                        let schluessel: String = argument(args, "schluessel")?;
                        wert(s.country(&schluessel)?)
                    }
                    "produktion" => wert(s.production()?),
                    "markt" => {
                        let land: String = argument(args, "land")?;
                        wert(s.market(&land)?)
                    }
                    "produktmarkt" => {
                        let land: String = argument(args, "land")?;
                        let produkt: String = argument(args, "produkt")?;
                        wert(s.product_market(&land, &produkt)?)
                    }
                    "ketten" => wert(s.chains()?),
                    "angebote" => wert(s.offers()?),
                    "firmen" => wert(s.companies()?),
                    "firma" => {
                        let index: u32 = argument(args, "index")?;
                        wert(s.company(index)?)
                    }
                    "weltmarkt" => {
                        let produkt: String = argument(args, "produkt")?;
                        wert(s.world_market(&produkt)?)
                    }
                    "forschung" => wert(s.research()?),
                    "finanzen" => wert(s.finance()?),
                    "organisation" => wert(s.organisation()?),
                    "anliegen" => wert(s.concerns()?),
                    "strategie" => wert(s.strategy()?),
                    "ruecksprache" => wert(s.reviews()?),
                    "startups" => wert(s.ventures()?),
                    "vertraege" => wert(s.contracts()?),
                    "logistik" => wert(s.logistics()?),
                    "konzern" => wert(s.group()?),
                    "boerse" => wert(s.stock()?),
                    "bank" => wert(s.bank()?),
                    "controlling" => {
                        let zeitraum: String = argument(args, "zeitraum")?;
                        wert(s.controlling(&zeitraum)?)
                    }
                    "vertragspartner" => {
                        let standort: u32 = argument(args, "standort")?;
                        let produkt: String = argument(args, "produkt")?;
                        wert(s.contract_partners(standort, &produkt)?)
                    }
                    "managermarkt" => {
                        let einheit: String = argument(args, "einheit")?;
                        let stelle: String = argument(args, "stelle")?;
                        wert(s.manager_market(&einheit, &stelle)?)
                    }
                    "befehl" => {
                        let befehl: Value = argument(args, "befehl")?;
                        wert(s.command(befehl)?)
                    }
                    "speichern" => {
                        let name: String = argument(args, "name")?;
                        wert(s.save(&name)?)
                    }
                    "spielstaende" => wert(s.saves()),
                    "laden" => {
                        let name: String = argument(args, "name")?;
                        wert(s.load(&name)?)
                    }
                    "geschrieben" => wert(s.store_mut().take_written()),
                    // For the check that browser and desktop compute the same game.
                    "zustands_hash" => wert(s.game().map(|g| g.state_hash().to_string())),
                    other => Err(Fehler::Text(format!("Unbekannte Anfrage „{other}“"))),
                }
            });
            ergebnis.map_err(Fehler::Text)?
        }
    }
}

/// Answers a JSON request with `{"ok": …}` or `{"err": …}`; `fortschritt` hears the
/// progress of a round (done, total days).
pub fn anfrage(json: &str, fortschritt: &mut dyn FnMut(u32, u32)) -> String {
    let antwort = serde_json::from_str::<Anfrage>(json)
        .map_err(|e| Fehler::Text(format!("Ungültige Anfrage: {e}")))
        .and_then(|a| ausfuehren(&a, fortschritt));
    match antwort {
        Ok(value) => json!({ "ok": value }),
        Err(fehler) => json!({ "err": fehler }),
    }
    .to_string()
}

/// The bytes of a save, if it exists.
pub fn spielstand(name: &str) -> Option<Vec<u8>> {
    mit_sitzung(|s| s.store_mut().get(name).map(<[u8]>::to_vec))
        .ok()
        .flatten()
}

/// Adds a save kept in the browser's storage.
pub fn spielstand_einlegen(name: &str, bytes: Vec<u8>) {
    // Without data there is no session to hold saves; the next request reports why.
    let _ = mit_sitzung(|s| s.store_mut().put(name, bytes));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(json: &str) -> Value {
        let antwort: Value = serde_json::from_str(&anfrage(json, &mut |_, _| {})).unwrap();
        assert!(antwort.get("err").is_none(), "{antwort}");
        antwort["ok"].clone()
    }

    #[test]
    fn plays_saves_and_loads_like_the_desktop_app() {
        let optionen = ok(r#"{"op": "optionen"}"#);
        assert!(
            optionen["countries"]
                .as_array()
                .is_some_and(|l| l.len() > 100)
        );
        let spiel = json!({
            "op": "neues_spiel",
            "args": { "einstellungen": {
                "seed": 3, "start_year": 1900, "country": "DEU", "capital_usd": 100000.0,
                "start_form": "werkstatt", "company_name": "Browser AG", "companies": 5,
                "difficulty": "mittel", "research_factor": 1.0
            }}
        });
        ok(&spiel.to_string());
        let mut tage = 0;
        let bericht = anfrage(
            r#"{"op": "runde_beenden", "args": {"laenge": "woche"}}"#,
            &mut |done, _| tage = done,
        );
        assert!(bericht.starts_with(r#"{"ok""#), "{bericht}");
        assert_eq!(tage, 7);
        ok(r#"{"op": "speichern", "args": {"name": "Probe"}}"#);
        let geschrieben = ok(r#"{"op": "geschrieben"}"#);
        assert_eq!(geschrieben, json!(["Automatisch", "Probe"]));
        let bytes = spielstand("Probe").expect("saved");
        spielstand_einlegen("Kopie", bytes);
        let liste = ok(r#"{"op": "spielstaende"}"#);
        assert_eq!(liste.as_array().map(Vec::len), Some(3));
        let uebersicht = ok(r#"{"op": "laden", "args": {"name": "Kopie"}}"#);
        assert_eq!(uebersicht["company"]["name"], "Browser AG");
    }

    #[test]
    fn reports_errors_in_the_shape_of_the_desktop_app() {
        let antwort: Value =
            serde_json::from_str(&anfrage(r#"{"op": "zaubern"}"#, &mut |_, _| {})).unwrap();
        assert!(
            antwort["err"]
                .as_str()
                .is_some_and(|t| t.contains("zaubern"))
        );
        let antwort: Value = serde_json::from_str(&anfrage(
            r#"{"op": "markt", "args": {"land": "XXX"}}"#,
            &mut |_, _| {},
        ))
        .unwrap();
        assert!(antwort["err"]["key"].is_string(), "{antwort}");
    }
}
