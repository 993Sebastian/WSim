//! A session from new game to save and load, with the shipped data.

use std::path::Path;

use wsim_session::{NewGameRequest, Session, keys};

fn data_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn request() -> NewGameRequest {
    NewGameRequest {
        seed: 5,
        start_year: 1900,
        country: "DEU".into(),
        capital_usd: 100_000.0,
        start_form: "werkstatt".into(),
        company_name: "Sitzung AG".into(),
        companies: 10,
        difficulty: "mittel".into(),
        research_factor: 1.0,
    }
}

#[test]
fn new_game_round_save_and_load() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::open(&data_dir(), dir.path().join("spielstaende")).unwrap();
    let options = session.options();
    assert!(options.countries.contains(&"DEU".to_owned()));
    assert_eq!(options.default_difficulty, "mittel");

    assert_eq!(session.overview().unwrap_err().key, keys::NO_GAME);
    let overview = session.new_game(&request()).unwrap();
    assert_eq!(overview.company.name, "Sitzung AG");
    assert_eq!(overview.competitors_active, 10);

    let map = session.world_map().unwrap();
    assert!(
        map.countries
            .iter()
            .any(|c| c.key == "DEU" && c.own_sites == 1)
    );
    let deu = session.country("DEU").unwrap();
    assert!(
        deu.companies
            .iter()
            .any(|c| c.own && c.name == "Sitzung AG")
    );
    assert_eq!(
        session.country("XXX").unwrap_err().key,
        keys::UNKNOWN_COUNTRY
    );

    let mut steps = 0;
    let report = session.end_round("woche", |_| steps += 1).unwrap();
    assert_eq!(report.days, 7);
    assert!(steps > 0);
    assert_eq!(
        session.end_round("jahrzehnt", |_| {}).unwrap_err().key,
        keys::UNKNOWN_ROUND_LENGTH
    );

    let entry = session.save("Erster Stand").unwrap();
    assert_eq!(entry.date, "1900-01-08");
    assert_eq!(
        session.save("../böse").unwrap_err().key,
        keys::INVALID_SAVE_NAME
    );
    let list = session.saves();
    // The round also wrote the automatic save.
    assert_eq!(list.len(), 2);
    assert!(list.iter().any(|s| s.name == wsim_session::AUTOSAVE_NAME));
    assert_eq!(list[0].company, "Sitzung AG");
    let second = session.end_round("woche", |_| {}).unwrap();
    assert_eq!(second.previous.as_ref(), Some(&report.period));

    session.load("Erster Stand").unwrap();
    let hash = session.game().unwrap().state_hash();
    session.end_round("tag", |_| {}).unwrap();
    let loaded = session.load("Erster Stand").unwrap();
    assert_eq!(loaded.date, "1900-01-08");
    assert_eq!(session.game().unwrap().state_hash(), hash);
}

#[test]
fn invalid_requests_answer_with_messages() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::open(&data_dir(), dir.path().to_path_buf()).unwrap();
    let mut r = request();
    r.country = "XXX".into();
    assert_eq!(session.new_game(&r).unwrap_err().key, keys::UNKNOWN_COUNTRY);
    let mut r = request();
    r.start_year = 1850;
    assert_eq!(
        session.new_game(&r).unwrap_err().key,
        "fehler.spielstart.startjahr"
    );
}

#[test]
fn every_session_message_has_a_text() {
    let data = wsim_data::load_dir(&data_dir()).data.expect("data loads");
    for key in keys::ALL {
        assert!(data.texts.get(key).is_some(), "Text „{key}“ fehlt");
    }
}

/// Several rounds at a stretch (M26): up to the end of the year or the next warning.
#[test]
fn rounds_up_to_the_year_end_or_the_next_news() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::open(&data_dir(), dir.path().join("spielstaende")).unwrap();
    session.new_game(&request()).unwrap();
    assert_eq!(
        session
            .end_round_until("monat", "jahrhundert", |_| {})
            .unwrap_err()
            .key,
        keys::UNKNOWN_UNTIL
    );
    let one = session.end_round_until("monat", "runde", |_| {}).unwrap();
    assert_eq!((one.rounds, one.stop.as_deref()), (1, None));

    // From February to the end of the year: eleven months in one report.
    let year = session
        .end_round_until("monat", "jahresende", |_| {})
        .unwrap();
    assert_eq!(year.rounds, 11);
    assert_eq!(year.stop.as_deref(), Some("jahresende"));
    assert_eq!(
        (year.from.as_str(), year.to.as_str()),
        ("1900-02-01", "1900-12-31")
    );
    assert_eq!(year.days, 334);
    assert_eq!(session.overview().unwrap().date, "1901-01-01");

    // Up to the next warning or world event, at most a year.
    let news = session.end_round_until("monat", "meldung", |_| {}).unwrap();
    let stop = news.stop.clone().unwrap();
    match stop.as_str() {
        "warnung" => assert!(
            news.messages
                .iter()
                .any(|m| m.kind == "warning" || m.kind == "crisis")
        ),
        "weltereignis" => assert!(news.messages.iter().any(|m| m.kind == "world_event")),
        "ein_jahr" => assert_eq!(news.rounds, 12),
        other => panic!("unexpected stop {other}"),
    }
    assert!(news.rounds <= 12);
}

/// The competition tab (M30): companies, one in detail, and the player's offers.
#[test]
fn companies_and_offers_for_the_competition_tab() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::open(&data_dir(), dir.path().join("spielstaende")).unwrap();
    session.new_game(&request()).unwrap();
    let companies = session.companies().unwrap();
    assert_eq!(companies.companies.len(), 11);
    assert!(companies.companies.iter().any(|c| c.player));
    let rival = companies.companies.iter().find(|c| !c.player).unwrap();
    let detail = session.company(rival.index).unwrap();
    assert_eq!(detail.company.name, rival.name);
    assert_eq!(detail.sites.len(), rival.sites as usize);
    // At the start every site is too young to buy.
    assert!(
        detail
            .sites
            .iter()
            .all(|s| s.blocked.as_deref() == Some("zu_jung"))
    );
    assert!(session.offers().unwrap().offers.is_empty());
    assert_eq!(
        session.company(9_999).unwrap_err().key,
        keys::UNKNOWN_COMPANY
    );
    // An offer for a site that is too young is refused with the reason.
    let site = detail.sites[0].site;
    let err = session
        .command(serde_json::json!({
            "MakeOffer": {"seller": rival.index, "object": {"Site": site}, "price": 10_000_000}
        }))
        .unwrap_err();
    assert_eq!(err.key, "fehler.befehl.standort_zu_jung");
}

/// A run of rounds halts for new concerns of the player's positions as the player set it
/// (MA2): all, the important ones, or none.
#[test]
fn runs_halt_for_concerns() {
    use serde_json::json;
    let start = |halt: &str| {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::open(&data_dir(), dir.path().join("spielstaende")).unwrap();
        session.new_game(&request()).unwrap();
        // A head for the workshop who asks about every expense.
        let site = session.organisation().unwrap().continents[0].countries[0].sites[0]
            .site
            .unwrap();
        let manager = session
            .manager_market(&format!("standort:{site}"), "leitung")
            .unwrap()
            .candidates[0]
            .manager
            .id;
        let head = json!({"site": site, "role": "Head"});
        session
            .command(json!({"HireManager": {"manager": manager, "position": head}}))
            .unwrap();
        session
            .command(json!({"SetBudget": {"position": head, "shares": [0.0, 0.0]}}))
            .unwrap();
        let run = session.end_rounds("monat", "jahresende", halt, |_| {});
        (session, run, dir)
    };
    let (_, run, _dir) = start("manchmal");
    assert_eq!(run.unwrap_err().key, keys::UNKNOWN_HALT);

    let (session, run, _dir) = start("alle");
    let run = run.unwrap();
    assert_eq!(run.stop.as_deref(), Some("anliegen"));
    assert!(run.rounds < 12);
    let concerns = session.concerns().unwrap();
    assert!(!concerns.open.is_empty());
    assert!(
        run.messages
            .iter()
            .any(|m| m.key == "meldung.anliegen.neu" && m.target.as_deref() == Some("organisation"))
    );

    // Never: up to the end of the year, the concerns wait (or expire).
    let (_, run, _dir) = start("nie");
    let run = run.unwrap();
    assert_eq!(run.stop.as_deref(), Some("jahresende"));
    assert_eq!(run.rounds, 12);
}
