//! Test game 1900–1930 (Architektur §4, M15): a player with a simple strategy plays
//! through the session, as the interface does, against 100 AI companies. Long; run with
//! `cargo test --release -p wsim-session --test testpartie -- --ignored --nocapture`.

use std::path::Path;

use serde_json::json;
use wsim_session::{NewGameRequest, Session};

fn data_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

/// The simple strategy: wire bought for 20 days, another nail machine when the cash
/// allows it and the nails sell; quarterly rounds until `until`. Returns the machines
/// built.
fn play_nails(session: &mut Session, until: &str) -> u32 {
    let site = session.production().unwrap().sites[0].index;
    let machine = session
        .production()
        .unwrap()
        .facilities
        .iter()
        .find(|f| f.key == "nagelmaschine")
        .map(|f| f.investment_usd)
        .unwrap();

    let mut built = 0;
    while session.overview().unwrap().date.as_str() < until {
        // Wire for 20 days at a price somewhat above the market.
        let production = session.production().unwrap();
        let s = production.sites.iter().find(|s| s.index == site).unwrap();
        let market = session.market("DEU").unwrap();
        let price = |p: &str| {
            market
                .lines
                .iter()
                .find(|l| l.product == p)
                .map_or(0.0, |l| l.price_usd)
        };
        let need: f64 = s
            .inputs
            .iter()
            .filter(|i| i.product == "draht")
            .map(|i| i.need_per_day)
            .sum();
        let wire = (price("draht") * 1.3).max(1.0);
        session
            .command(json!({"SetPurchase": {
                "site": site, "product": "draht", "target": (need * 20.0).max(10.0),
                "max_price": (wire * 10_000.0).round() as i64, "min_quality": 0.0
            }}))
            .unwrap();
        // Another nail machine when the cash allows it and the nails sell.
        let finance = session.finance().unwrap();
        let offer = s.offers.iter().find(|o| o.product == "naegel");
        let selling = offer.is_some_and(|o| o.sold_last_month > 0.0);
        if finance.assets[0].1 > 3.0 * machine && selling && built < 20 {
            session
                .command(json!({"BuildFacility": {"site": site, "facility": "nagelmaschine", "count": 1}}))
                .unwrap();
            let slot = session.production().unwrap().sites[0].slots.len() - 1;
            session
                .command(json!({"SetProduction": {
                    "site": site, "slot": slot, "recipe": "naegel_maschine", "utilization": 1.0
                }}))
                .unwrap();
            built += 1;
        }
        let report = session.end_round("quartal", |_| {}).unwrap();
        assert!(!report.game_over, "Partie beendet am {}", report.to);
    }

    built
}

#[test]
#[ignore = "plays 30 years, several minutes in release"]
fn test_game_1900_to_1930() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::open(&data_dir(), dir.path().to_path_buf()).unwrap();
    let options = session.options();
    session
        .new_game(&NewGameRequest {
            seed: 1900,
            start_year: 1900,
            country: "DEU".into(),
            capital_usd: 250_000.0,
            start_form: "werkstatt".into(),
            company_name: "Testpartie AG".into(),
            companies: options.companies.default,
            difficulty: options.default_difficulty.clone(),
            research_factor: 1.0,
            startups: None,
            tariffs: None,
            event_effects: true,
            found_at_start: true,
            person_name: String::new(),
            birth_year: None,
            married: true,
            children: 0,
        })
        .unwrap();
    let built = play_nails(&mut session, "1930-01-01");

    let overview = session.overview().unwrap();
    let finance = session.finance().unwrap();
    eprintln!(
        "1930: Eigenkapital {:.0} USD, Kasse {:.0} USD, {} Nagelmaschinen gebaut, {} aktive KI-Firmen, {} ausgeschieden",
        overview.company.as_ref().unwrap().equity_usd,
        finance.assets[0].1,
        built,
        overview.competitors_active,
        overview.competitors_bankrupt,
    );
    assert!(overview.company.as_ref().unwrap().equity_usd > 0.0);
    assert!(overview.competitors_active >= 50, "{overview:?}");
}

/// PE3: the same strategy as a person who founds the workshop with the suggested capital
/// and lives from its salary; the private account must not run dry.
#[test]
#[ignore = "plays 15 years, a few minutes in release"]
fn a_founder_lives_from_the_workshop_1900_to_1915() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::open(&data_dir(), dir.path().to_path_buf()).unwrap();
    let options = session.options();
    session
        .new_game(&NewGameRequest {
            seed: 1900,
            start_year: 1900,
            country: "DEU".into(),
            capital_usd: 250_000.0,
            start_form: String::new(),
            company_name: String::new(),
            companies: options.companies.default,
            difficulty: options.default_difficulty.clone(),
            research_factor: 1.0,
            startups: None,
            tariffs: None,
            event_effects: true,
            found_at_start: false,
            person_name: String::new(),
            birth_year: None,
            married: true,
            children: 0,
        })
        .unwrap();
    let founding = session.person().unwrap().founding.unwrap();
    session
        .command(json!({"FoundCompany": {
            "name": "Gründer AG", "form": "Workshop", "country": "DEU",
            "capital": (founding.capital_suggestion_usd * 10_000.0).round() as i64
        }}))
        .unwrap();
    let built = play_nails(&mut session, "1915-01-01");

    let person = session.person().unwrap();
    let overview = session.overview().unwrap();
    let money = person.money.unwrap();
    eprintln!(
        "1915: Privatkonto {:.0} USD, Vermögen {:.0} USD, Gehalt {:.0} USD/Jahr, Eigenkapital {:.0} USD, {} Nagelmaschinen",
        money.cash_usd,
        money.wealth_usd,
        money.salary_usd,
        overview.company.as_ref().unwrap().equity_usd,
        built,
    );
    for (key, usd) in &money.flows {
        eprintln!("  {key}: {usd:.0} USD");
    }
    assert!(overview.company.as_ref().unwrap().equity_usd > 0.0);
    assert!(money.cash_usd > 0.0, "{money:?}");
}
