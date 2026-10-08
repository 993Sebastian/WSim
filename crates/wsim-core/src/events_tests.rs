//! Tests of the effects of historical events (H1) with the small chain of
//! `test_support::production`.

use crate::calendar::Date;
use crate::catalog::{Catalog, EffectKind, EventEffect, HistoricalEvent, SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::events::{self, EventTable};
use crate::facility_tests::{mine, new_game, works};
use crate::game::Game;
use crate::ids::Id;
use crate::ledger::{Account, CostType};
use crate::message::keys;
use crate::money::Money;
use crate::save;
use crate::state::{Limit, SiteId};

fn date(y: i32, m: u32, d: u32) -> Date {
    Date::new(y, m, d).unwrap()
}

/// The catalog with one event in `countries` on `day` with the given effects.
fn with_event(
    mut catalog: Catalog,
    day: Date,
    countries: &[&str],
    effects: Vec<(EffectKind, Option<Date>)>,
) -> Catalog {
    let ids: Vec<_> = countries
        .iter()
        .map(|k| catalog.countries.id(k).unwrap())
        .collect();
    catalog.events.push(HistoricalEvent {
        key: "probe".into(),
        date: day,
        kind: "krieg".into(),
        countries: ids.clone(),
        effects: effects
            .into_iter()
            .map(|(kind, until)| EventEffect {
                countries: ids.clone(),
                kind,
                until,
            })
            .collect(),
        provenance: Default::default(),
    });
    catalog.event_model.working_capital_share = 0.1;
    catalog
}

fn days(game: &mut Game, n: u32) -> Vec<String> {
    let mut keys = Vec::new();
    for _ in 0..n {
        let report = game.advance(crate::calendar::RoundLength::Day, |_| {});
        keys.extend(report.messages.into_iter().map(|m| m.key));
    }
    keys
}

fn balanced(game: &Game) -> bool {
    game.state()
        .companies
        .iter()
        .all(|c| c.ledger.is_balanced())
}

#[test]
fn effects_act_from_the_next_month_until_their_end() {
    let catalog = with_event(
        test_support::production(),
        date(1900, 3, 15),
        &["AAA"],
        vec![(EffectKind::Labor { factor: 0.8 }, Some(date(1900, 6, 10)))],
    );
    let a = catalog.countries.id("AAA").unwrap();
    let b = catalog.countries.id("BBB").unwrap();
    let at = |m: u32, on: bool| EventTable::new(&catalog, date(1900, m, 1), on).labor(a);
    assert_eq!(at(3, true), 1.0);
    assert_eq!(at(4, true), 0.8);
    assert_eq!(at(6, true), 0.8);
    assert_eq!(at(7, true), 1.0);
    assert_eq!(at(5, false), 1.0);
    assert_eq!(
        EventTable::new(&catalog, date(1900, 5, 1), true).labor(b),
        1.0
    );
}

#[test]
fn demand_and_workers_follow_the_effects() {
    let plain = test_support::production();
    let shifted = with_event(
        test_support::production(),
        date(1900, 1, 1),
        &["AAA"],
        vec![
            (
                EffectKind::Demand {
                    groups: Vec::new(),
                    consumer: 0.5,
                    state: 2.0,
                },
                None,
            ),
            (EffectKind::Labor { factor: 0.75 }, None),
        ],
    );
    let (mut g0, mut g1) = (new_game(plain), new_game(shifted));
    days(&mut g0, 35);
    days(&mut g1, 35);
    let a = g0.catalog().countries.id("AAA").unwrap();
    let mut compared = 0;
    for (p, _) in g0.catalog().products.iter() {
        let (m0, m1) = (
            g0.state().markets.get(p).get(a),
            g1.state().markets.get(p).get(a),
        );
        let (c0, c1): (f64, f64) = (m0.consumer_rate.iter().sum(), m1.consumer_rate.iter().sum());
        if c0 > 0.0 {
            assert!((c1 / c0 - 0.5).abs() < 0.05, "{c1} / {c0}");
            compared += 1;
        }
        if m0.state_rate > 0.0 {
            assert!((m1.state_rate / m0.state_rate - 2.0).abs() < 0.1);
            compared += 1;
        }
    }
    assert!(compared >= 2);
    let (l0, l1) = (
        &g0.state().countries.get(a).labor_available,
        &g1.state().countries.get(a).labor_available,
    );
    for (x, y) in l0.iter().zip(l1) {
        assert!((y - 0.75 * x).abs() <= 1e-9 * x.max(1.0), "{y} vs {x}");
    }
}

#[test]
fn an_embargo_blocks_trade_for_its_time() {
    let catalog = with_event(
        test_support::trading(),
        date(1900, 1, 1),
        &["AAA"],
        vec![(
            EffectKind::Embargo {
                against: vec![crate::ids::CountryId::from_index(1)],
            },
            Some(date(1900, 3, 1)),
        )],
    );
    let mut game = crate::trade_tests::new_game(catalog);
    let (a, b) = (
        game.catalog().countries.id("AAA").unwrap(),
        game.catalog().countries.id("BBB").unwrap(),
    );
    let iron = game.catalog().products.id("eisen").unwrap();
    assert!(game.state().tariffs.blocked(a, b));
    assert!(game.state().tariffs.blocked(b, a));
    let catalog = game.catalog().clone();
    assert_eq!(game.state().tariffs.for_product(&catalog, a, b, iron), None);
    days(&mut game, 60);
    assert!(!game.state().tariffs.blocked(a, b));
    assert!(
        game.state()
            .tariffs
            .for_product(&catalog, a, b, iron)
            .is_some()
    );
}

#[test]
fn production_cuts_hold_the_runs_back() {
    let catalog = with_event(
        test_support::production(),
        date(1900, 2, 1),
        &["AAA"],
        vec![(
            EffectKind::Production {
                groups: Vec::new(),
                factor: 0.0,
            },
            None,
        )],
    );
    let mut game = new_game(catalog);
    let site = mine(&mut game);
    days(&mut game, 45);
    let sl = &game.state().site(site).unwrap().slots[0];
    assert_eq!(sl.limit, Some(Limit::Event));
    assert_eq!(sl.last_runs, 0.0);
}

/// A works with four furnaces ready before the event on 15 February 1900.
fn struck(kind: EffectKind, countries: &[&str]) -> (Game, SiteId, Vec<String>) {
    let catalog = with_event(
        test_support::production(),
        date(1900, 2, 15),
        countries,
        vec![(kind, None)],
    );
    let mut game = new_game(catalog);
    let site = works(&mut game, 4);
    let news = days(&mut game, 60);
    (game, site, news)
}

#[test]
fn destruction_writes_off_units_and_stocks() {
    let (game, site, news) = struck(EffectKind::Destruction { share: 0.5 }, &["AAA"]);
    let s = game.state().site(site).unwrap();
    assert_eq!(s.slots[0].count, 2);
    let me = &game.state().companies[game.player().index()];
    assert!(me.ledger.year.by_type[&CostType::Other] < Money::ZERO);
    assert!(news.iter().any(|k| k == keys::EVENT_DESTRUCTION));
    assert!(balanced(&game));
    // Elsewhere nothing happens.
    let (game, site, _) = struck(EffectKind::Destruction { share: 0.5 }, &["BBB"]);
    assert_eq!(game.state().site(site).unwrap().slots[0].count, 4);
}

#[test]
fn expropriation_hands_the_sites_to_the_state_company() {
    // Only foreign sites: the player's company is from AAA and keeps its works.
    let foreign = EffectKind::Expropriation {
        foreign_only: true,
        compensation: 0.0,
    };
    let (game, site, _) = struck(foreign, &["AAA"]);
    assert_eq!(game.state().site(site).unwrap().owner, game.player());
    // All sites: the works goes to the new state company, half paid for.
    let all = EffectKind::Expropriation {
        foreign_only: false,
        compensation: 0.5,
    };
    let (game, site, news) = struck(all, &["AAA"]);
    let owner = game.state().site(site).unwrap().owner;
    assert_ne!(owner, game.player());
    let state_company = &game.state().companies[owner.index()];
    assert_eq!(
        state_company.state_owned,
        game.catalog().countries.id("AAA")
    );
    assert!(state_company.ledger.balance(Account::FixedAssets) > Money::ZERO);
    assert!(state_company.ledger.cash() > Money::ZERO);
    assert!(news.iter().any(|k| k == keys::EVENT_EXPROPRIATION));
    assert!(news.iter().any(|k| k == keys::EVENT_STATE_COMPANY));
    assert!(balanced(&game));
    // Left on the books: at most the day's depreciation already booked (as in a sale).
    let me = &game.state().companies[game.player().index()];
    let rest = me.ledger.balance(Account::FixedAssets).to_usd().abs();
    assert!(rest < 5_000.0, "{rest}");
}

#[test]
fn closed_countries_admit_no_new_sites() {
    let catalog = with_event(
        test_support::production(),
        date(1900, 1, 1),
        &["BBB"],
        vec![(EffectKind::Closure { all: false }, None)],
    );
    let mut game = new_game(catalog);
    let b = game.catalog().countries.id("BBB").unwrap();
    let a = game.catalog().countries.id("AAA").unwrap();
    assert_eq!(
        game.apply(Command::FoundSite {
            country: b,
            kind: SiteType::Extraction,
        }),
        Err(CommandError::CountryClosed)
    );
    assert!(events::closed_to(game.state(), game.player(), b));
    assert!(!events::closed_to(game.state(), game.player(), a));
    game.apply(Command::FoundSite {
        country: a,
        kind: SiteType::Extraction,
    })
    .unwrap();
    // Closed to all: even the player's own country.
    let catalog = with_event(
        test_support::production(),
        date(1900, 1, 1),
        &["AAA"],
        vec![(EffectKind::Closure { all: true }, None)],
    );
    let mut game = new_game(catalog);
    assert_eq!(
        game.apply(Command::FoundSite {
            country: a,
            kind: SiteType::Extraction,
        }),
        Err(CommandError::CountryClosed)
    );
}

#[test]
fn games_without_effects_only_tell_the_news() {
    let catalog = with_event(
        test_support::production(),
        date(1900, 2, 15),
        &["AAA"],
        vec![(EffectKind::Destruction { share: 1.0 }, None)],
    );
    let mut catalog_off = catalog.clone();
    catalog_off.events[0].key = "probe".into();
    let mut game = new_game(catalog_off);
    game.state_mut().settings.event_effects = false;
    let site = works(&mut game, 4);
    let news = days(&mut game, 60);
    assert_eq!(game.state().site(site).unwrap().slots[0].count, 4);
    assert!(news.iter().any(|k| k == keys::WORLD_EVENT));
    assert!(!news.iter().any(|k| k.starts_with("meldung.folge.")));
    // With effects the news names them.
    let mut game = new_game(catalog);
    works(&mut game, 4);
    let news = days(&mut game, 60);
    assert!(news.iter().any(|k| k == "meldung.folge.zerstoerung"));
}

#[test]
fn effects_survive_saving() {
    let catalog = with_event(
        test_support::production(),
        date(1900, 2, 15),
        &["AAA"],
        vec![
            (
                EffectKind::Expropriation {
                    foreign_only: false,
                    compensation: 0.2,
                },
                None,
            ),
            (EffectKind::Labor { factor: 0.9 }, None),
        ],
    );
    let mut game = new_game(catalog);
    works(&mut game, 3);
    days(&mut game, 50);
    let mut loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(game.state_hash(), loaded.state_hash());
    days(&mut game, 40);
    days(&mut loaded, 40);
    assert_eq!(game.state_hash(), loaded.state_hash());
    assert_eq!(game.state().countries, loaded.state().countries);
}

#[test]
fn ai_companies_in_countries_closed_to_all_are_state_companies() {
    let catalog = with_event(
        test_support::production(),
        date(1899, 6, 1),
        &["AAA"],
        vec![(EffectKind::Closure { all: true }, None)],
    );
    let mut game = new_game(catalog);
    let a = game.catalog().countries.id("AAA").unwrap();
    let rival = crate::trade_tests::competitor(&mut game);
    game.state_mut().companies[rival.index()].ai = Some(crate::state::AiState {
        competence: 0.5,
        aggressiveness: 0.5,
        real: None,
        next_operations: date(1900, 1, 2),
        staff: 0.0,
    });
    assert!(events::closed_to(game.state(), rival, a));
    events::start(game.state_mut());
    assert_eq!(game.state().companies[rival.index()].state_owned, Some(a));
    assert!(!events::closed_to(game.state(), rival, a));
    // The player's company stays private and may not open more sites there.
    assert_eq!(
        game.state().companies[game.player().index()].state_owned,
        None
    );
    assert!(events::closed_to(game.state(), game.player(), a));
}
