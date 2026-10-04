//! A played game 1900–1905 (Architektur §4, M14): the player expands the workshop
//! through commands, as the interface sends them, and the company survives.

use std::path::Path;

use serde_json::json;
use wsim_session::{NewGameRequest, Session, keys};

fn data_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

#[test]
fn commands_from_the_interface() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::open(&data_dir(), dir.path().to_path_buf()).unwrap();
    session
        .new_game(&NewGameRequest {
            seed: 3,
            start_year: 1900,
            country: "DEU".into(),
            capital_usd: 100_000.0,
            start_form: "werkstatt".into(),
            company_name: "Befehlsprobe".into(),
            companies: 20,
            difficulty: "mittel".into(),
            research_factor: 1.0,
        })
        .unwrap();
    // Unknown keys and refused commands answer with messages.
    let unknown = json!({"FoundSite": {"country": "XXX", "kind": "Factory"}});
    assert_eq!(
        session.command(unknown).unwrap_err().key,
        keys::INVALID_COMMAND
    );
    let site = session.production().unwrap().sites[0].index;
    let wrong = json!({"BuildFacility": {"site": site, "facility": "erzbergwerk", "count": 1}});
    assert_eq!(
        session.command(wrong).unwrap_err().key,
        "fehler.befehl.falscher_standorttyp"
    );
    // A valid command changes the game and is answered with the overview.
    let rename = json!({"RenameCompany": {"name": "Befehlsprobe AG"}});
    let overview = session.command(rename).unwrap();
    assert_eq!(overview.company.name, "Befehlsprobe AG");
}

#[test]
fn played_game_1900_to_1905() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::open(&data_dir(), dir.path().to_path_buf()).unwrap();
    session
        .new_game(&NewGameRequest {
            seed: 9,
            start_year: 1900,
            country: "DEU".into(),
            capital_usd: 250_000.0,
            start_form: "werkstatt".into(),
            company_name: "Nagelwerk Probe".into(),
            companies: 20,
            difficulty: "mittel".into(),
            research_factor: 1.0,
        })
        .unwrap();
    let production = session.production().unwrap();
    let site = production.sites[0].index;
    assert!(
        production
            .facilities
            .iter()
            .any(|f| f.key == "nagelmaschine")
    );

    // Two more nail machines, run at full load, with more wire bought.
    session
        .command(json!({"BuildFacility": {"site": site, "facility": "nagelmaschine", "count": 2}}))
        .unwrap();
    session
        .command(json!({"SetProduction": {
            "site": site, "slot": 1, "recipe": "naegel_maschine", "utilization": 1.0
        }}))
        .unwrap();
    session
        .command(json!({"SetPurchase": {
            "site": site, "product": "draht", "target": 30.0,
            "max_price": 25_000_000, "min_quality": 0.0
        }}))
        .unwrap();
    session
        .command(json!({"SetSale": {
            "site": site, "product": "naegel",
            "mode": {"Market": {"markup": 0.0, "floor": 15_000_000}}, "keep": 0.0
        }}))
        .unwrap();
    session
        .command(json!({"TakeLoan": {"amount": 500_000_000, "years": 10}}))
        .unwrap();

    let mut months = 0;
    while session.overview().unwrap().date.as_str() < "1905-01-01" {
        let report = session.end_round("monat", |_| {}).unwrap();
        assert!(!report.game_over, "insolvent in {}", report.from);
        months += 1;
    }
    assert_eq!(months, 60);
    let overview = session.overview().unwrap();
    assert!(overview.company.equity_usd > 0.0, "{:?}", overview.company);
    let production = session.production().unwrap();
    let slot = &production.sites[0].slots[1];
    assert_eq!(slot.count, 2);
    assert!(slot.made_per_day > 0.0, "{slot:?}");
    let finance = session.finance().unwrap();
    assert_eq!(finance.loans.len(), 1);
    assert!(finance.loans[0].balance_usd < finance.loans[0].principal_usd);
    assert!(finance.last_year.is_some());
    let market = session.market("DEU").unwrap();
    let nails = market.lines.iter().find(|l| l.product == "naegel").unwrap();
    assert!(nails.own_price_usd.is_some());
    let research = session.research().unwrap();
    assert!(research.technologies.iter().any(|t| t.researchable));
    // The game was played through commands only: replaying the journal gives it again.
    let game = session.game().unwrap();
    let again = wsim_core::game::Game::replay(
        game.catalog().clone(),
        game.state().settings.clone(),
        game.journal(),
    )
    .unwrap();
    assert_eq!(again.state_hash(), game.state_hash());
}
