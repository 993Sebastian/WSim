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
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].company, "Sitzung AG");

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
