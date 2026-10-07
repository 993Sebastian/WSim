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
        startups: None,
        tariffs: None,
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
        // In 1950 the workshop's market wants more works; in 1900 the nail works of the
        // world plan below the load at which anyone builds (C4).
        let later = NewGameRequest {
            start_year: 1950,
            capital_usd: 200_000.0,
            ..request()
        };
        session.new_game(&later).unwrap();
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

/// Strategies come as JSON with keys and show where they hold (MA4).
#[test]
fn strategies_with_keys_and_their_origin() {
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::open(&data_dir(), dir.path().join("spielstaende")).unwrap();
    session.new_game(&request()).unwrap();
    session
        .command(json!({"SetStrategy": {
            "scope": {"Continent": "europa"},
            "field": "Price",
            "value": {"Price": "Premium"}
        }}))
        .unwrap();
    session
        .command(json!({"SetStrategy": {
            "scope": "Company",
            "field": "Investment",
            "value": {"Investment": 50_000_000_000_i64}
        }}))
        .unwrap();
    let wrong = session.command(json!({"SetStrategy": {
        "scope": "Company",
        "field": "Reserve",
        "value": {"Reserve": 1000.0}
    }}));
    assert_eq!(wrong.unwrap_err().key, "fehler.befehl.vorgabe_ungueltig");
    let v = session.strategy().unwrap();
    assert!(v.enabled);
    let keys: Vec<&str> = v.units.iter().map(|u| u.key.as_str()).collect();
    assert_eq!(&keys[..3], ["firma", "kontinent:europa", "land:DEU"]);
    let site = &v.units[3];
    assert_eq!(site.level, "standort");
    let price = &site.entries[0];
    assert_eq!(price.field, "preis");
    assert_eq!(price.origin.as_deref(), Some("kontinent:europa"));
    assert_eq!(
        serde_json::to_value(price.value).unwrap(),
        json!({"Price": "Premium"})
    );
    let invest = site
        .entries
        .iter()
        .find(|e| e.field == "investition")
        .unwrap();
    assert_eq!(invest.budget_usd, Some(5_000_000.0));
    assert_eq!(invest.left_usd, Some(5_000_000.0));
    assert_eq!(invest.binding.as_deref(), Some("firma"));
}

/// The mandate comes as JSON with keys; the CEO reviews the strategy at the end of the
/// quarter, a run of rounds halts there, and the report is the round's (MA5).
#[test]
fn the_ceo_reviews_the_quarter() {
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::open(&data_dir(), dir.path().join("spielstaende")).unwrap();
    // A CEO costs more than a workshop earns.
    let request = NewGameRequest {
        capital_usd: 20_000_000.0,
        ..request()
    };
    session.new_game(&request).unwrap();
    let group = session.reviews().unwrap().groups[0].clone();
    session
        .command(json!({"SetMandate": {"mandate": {
            "guideline": {"Leadership": group},
            "goals": {"growth": 0.1, "rank": 3},
            "max_debt": 0.5,
            "blocked_countries": ["RUS"]
        }}}))
        .unwrap();
    let wrong = session.command(json!({"SetMandate": {"mandate": {"max_debt": 2.0}}}));
    assert_eq!(wrong.unwrap_err().key, "fehler.befehl.auftrag_ungueltig");
    let v = session.reviews().unwrap();
    assert!(v.enabled);
    assert_eq!(v.mandate.guideline, "marktfuehrung");
    assert_eq!(v.mandate.leading_group.as_deref(), Some(group.as_str()));
    assert_eq!(v.mandate.blocked_countries, ["RUS"]);
    assert_eq!(v.mandate.review, "quartalsweise");
    assert_eq!(v.guidelines.len(), 4);
    assert!(v.ceo.is_none() && v.next_review.is_none());

    // A CEO.
    let manager = session
        .manager_market("vorstand", "leitung")
        .unwrap()
        .candidates[0]
        .manager
        .id;
    session
        .command(json!({"HireManager": {
            "manager": manager,
            "position": {"unit": "Board", "role": "Head"}
        }}))
        .unwrap();
    let v = session.reviews().unwrap();
    assert_eq!(v.next_review.as_deref(), Some("1900-04-01"));

    // Month by month up to the end of the year: the review halts at the end of March,
    // whatever the concerns.
    let run = session
        .end_rounds("monat", "jahresende", "nie", |_| {})
        .unwrap();
    assert_eq!(run.stop.as_deref(), Some("ruecksprache"));
    assert_eq!(run.rounds, 3);
    assert!(run.messages.iter().any(|m| m.key == "meldung.ruecksprache"));
    let v = session.reviews().unwrap();
    let r = &v.reviews[0];
    assert_eq!(
        (r.from.as_str(), r.to.as_str()),
        ("1900-01-01", "1900-03-31")
    );
    assert!(
        (r.revenue_usd - run.period.revenue_usd).abs() < 0.01,
        "{} vs {}",
        r.revenue_usd,
        run.period.revenue_usd
    );
    let products: f64 = run.products.iter().map(|p| p.revenue_usd).sum();
    let groups: f64 = r.groups.iter().map(|g| g.revenue_usd).sum();
    assert!((products - groups).abs() < 0.01, "{products} vs {groups}");
    let margins: f64 = run.products.iter().map(|p| p.margin_usd).sum();
    let group_margins: f64 = r.groups.iter().map(|g| g.result_usd).sum();
    assert!(
        (margins - group_margins).abs() < 0.01,
        "{margins} vs {group_margins}"
    );
    assert_eq!(r.goals.len(), 2);
}

/// A raise and a head that fills its unit's positions, as JSON with keys (MA6).
#[test]
fn pay_and_a_head_that_hires() {
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::open(&data_dir(), dir.path().join("spielstaende")).unwrap();
    session.new_game(&request()).unwrap();
    let site = session.organisation().unwrap().continents[0].countries[0].sites[0]
        .site
        .unwrap();
    let candidate = session
        .manager_market(&format!("standort:{site}"), "leitung")
        .unwrap()
        .candidates[0]
        .manager
        .id;
    let head = json!({"unit": {"Site": site}, "role": "Head"});
    session
        .command(json!({"HireManager": {"manager": candidate, "position": head}}))
        .unwrap();
    let holder = |session: &Session| {
        let o = session.organisation().unwrap();
        o.continents[0].countries[0].sites[0].positions[0].clone()
    };
    let position = holder(&session);
    let h = position.holder.clone().unwrap();
    assert_eq!(h.satisfaction, 2, "content at the start");
    assert!(h.market_usd > 0.0 && h.offer.is_none());
    assert_eq!(position.hires, Some(false));
    // Raise by a tenth, in Money units (hundredths of a cent).
    let salary = (h.salary_usd * 1.1 * 10_000.0).round() as i64;
    session
        .command(json!({"RaiseSalary": {"manager": candidate, "salary": salary}}))
        .unwrap();
    let lower = session.command(json!({"RaiseSalary": {"manager": candidate, "salary": 1}}));
    assert_eq!(lower.unwrap_err().key, "fehler.befehl.gehalt_nicht_hoeher");
    session
        .command(json!({"SetHiringByHead": {"position": head, "enabled": true}}))
        .unwrap();
    let position = holder(&session);
    assert_eq!(position.hires, Some(true));
    assert!((position.holder.unwrap().salary_usd - h.salary_usd * 1.1).abs() < 0.01);
    // At the next month start the head hires for the first position with work.
    let run = session.end_round("monat", |_| {}).unwrap();
    let hired = run
        .messages
        .iter()
        .find(|m| m.key == "meldung.manager.eingestellt")
        .expect("a hire");
    assert_eq!(hired.target.as_deref(), Some("organisation"));
    let o = session.organisation().unwrap();
    let filled = o.continents[0].countries[0].sites[0]
        .positions
        .iter()
        .filter(|p| p.holder.is_some())
        .count();
    assert_eq!(filled, 2);
}

/// How many start-ups there are follows the choice of the new game (SU1).
#[test]
fn start_ups_follow_the_choice() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::open(&data_dir(), dir.path().join("spielstaende")).unwrap();
    let options = session.options();
    assert_eq!(options.default_startups.as_deref(), Some("normal"));
    let many = options.startups.iter().find(|f| f.key == "viele").unwrap();
    assert_eq!(many.per_year, 24.0);
    let unknown = NewGameRequest {
        startups: Some("unbekannt".into()),
        ..request()
    };
    assert_eq!(
        session.new_game(&unknown).unwrap_err().key,
        "fehler.sitzung.unbekannte_haeufigkeit"
    );
    let none = NewGameRequest {
        startups: Some("keine".into()),
        ..request()
    };
    session.new_game(&none).unwrap();
    for _ in 0..3 {
        session.end_round("monat", |_| {}).unwrap();
    }
    let view = session.ventures().unwrap();
    assert_eq!(view.per_year, 0.0);
    assert_eq!(view.founded, 0);
    assert!(view.active.is_empty());
    // The default: about one a month, named for the epoch, chances only as levels.
    session.new_game(&request()).unwrap();
    for _ in 0..3 {
        session.end_round("monat", |_| {}).unwrap();
    }
    let view = session.ventures().unwrap();
    assert_eq!(view.per_year, 12.0);
    assert_eq!(view.label.as_deref(), Some("erfinder"));
    assert!(view.founded >= 2, "{}", view.founded);
    assert!(!view.estimated);
    let first = &view.active[0];
    assert!(first.chance.is_none());
    assert!(first.chance_level.is_some());
    assert_eq!(first.phase.as_deref(), Some("idee"));
    assert_eq!(first.owners[0].holder, "gruender");
}

#[test]
fn supply_contracts_with_ai_companies() {
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::open(&data_dir(), dir.path().join("spielstaende")).unwrap();
    let later = NewGameRequest {
        start_year: 1950,
        capital_usd: 200_000.0,
        ..request()
    };
    session.new_game(&later).unwrap();
    let view = session.contracts().unwrap();
    assert!(view.enabled);
    assert!(view.contracts.is_empty());
    // The workshop buys wire; sellers are the wire works.
    let site = view
        .sites
        .iter()
        .find(|s| s.buys.iter().any(|p| p == "draht"))
        .expect("Werkstatt braucht Draht")
        .site;
    let partners = session.contract_partners(site, "draht").unwrap();
    assert_eq!(partners.role, "einkauf");
    let seller = partners.partners.first().expect("ein Lieferant").clone();
    let quantity = seller.free_per_month.min(0.5 * partners.own_per_month);
    assert!(quantity > 0.0);
    let price = (seller.suggested_price_usd * 10_000.0).round() as i64;
    session
        .command(json!({"ProposeContract": {
            "seller": seller.site, "buyer": site, "product": "draht",
            "per_month": quantity, "price": price, "months": 6,
            "min_quality": 0.0, "penalty": 0.2
        }}))
        .unwrap();
    let view = session.contracts().unwrap();
    let c = &view.contracts[0];
    assert_eq!((c.status.as_str(), c.role.as_str()), ("laufend", "einkauf"));
    assert_eq!(c.partner, seller.company);
    assert!(c.can_cancel && c.cancel_fee_usd > 0.0);
    // Too much is declined with the reason.
    let err = session
        .command(json!({"ProposeContract": {
            "seller": seller.site, "buyer": site, "product": "draht",
            "per_month": seller.free_per_month * 10.0, "price": price, "months": 6,
            "min_quality": 0.0, "penalty": 0.2
        }}))
        .unwrap_err();
    assert_eq!(err.key, "fehler.befehl.vertrag_abgelehnt");
    for _ in 0..2 {
        session.end_round("monat", |_| {}).unwrap();
    }
    let c = &session.contracts().unwrap().contracts[0];
    assert!(c.delivered_total > 0.0, "{c:?}");
}
