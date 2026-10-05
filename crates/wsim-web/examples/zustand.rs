//! Plays the requests of `ui/scripts/wasm-pruefen.mjs` natively and prints the state
//! hash; CI compares it with the hash of the WebAssembly module (same game in browser
//! and desktop app).

fn main() {
    let einstellungen = r#"{"op": "neues_spiel", "args": {"einstellungen": {
        "seed": 7, "start_year": 1900, "country": "DEU", "capital_usd": 100000.0,
        "start_form": "werkstatt", "company_name": "Probe", "companies": 20,
        "difficulty": "mittel", "research_factor": 1.0}}}"#;
    let mut leise = |_, _| {};
    for request in [
        einstellungen,
        r#"{"op": "runde_beenden", "args": {"laenge": "monat"}}"#,
        r#"{"op": "runde_beenden", "args": {"laenge": "monat"}}"#,
    ] {
        let answer = wsim_web::anfrage(request, &mut leise);
        assert!(answer.starts_with(r#"{"ok""#), "{answer}");
    }
    println!(
        "{}",
        wsim_web::anfrage(r#"{"op": "zustands_hash"}"#, &mut leise)
    );
}
