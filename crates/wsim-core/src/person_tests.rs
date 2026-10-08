//! Scenario tests for the player as a person (PE2) with `test_support::aging` and the
//! person of the data.

use std::sync::Arc;

use crate::aging;
use crate::calendar::{Date, RoundLength};
use crate::catalog::{Catalog, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::management_tests::{free, mine_and_works, specialist, usd};
use crate::message::keys;
use crate::person;
use crate::save;
use crate::state::{
    CompanyId, GameSettings, LifeEventKind, Person, PersonSettings, Position, Role, StartForm, Unit,
};
use crate::views;

fn catalog() -> Catalog {
    let mut c = test_support::aging();
    c.person = test_support::person();
    c
}

fn game_with(catalog: Catalog, person: PersonSettings) -> Game {
    let catalog = Arc::new(catalog);
    let settings = GameSettings {
        seed: 7,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(10_000_000.0),
        start_form: StartForm::Workshop,
        company_name: "Hütte AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        person,
    };
    Game::new(catalog, settings).unwrap()
}

/// Days up to and including the next month start; the message keys on the way.
fn to_next_month(game: &mut Game) -> Vec<String> {
    let mut keys = Vec::new();
    loop {
        let report = game.advance(RoundLength::Day, |_| {});
        keys.extend(report.messages.into_iter().map(|m| m.key));
        if game.state().date.day() == 1 {
            return keys;
        }
    }
}

fn date(y: i32, m: u32, d: u32) -> Date {
    Date::new(y, m, d).unwrap()
}

#[test]
fn a_new_game_starts_with_the_person_as_set() {
    let game = game_with(
        catalog(),
        PersonSettings {
            name: "Clara Weber".into(),
            birth_year: Some(1860),
            married: true,
            children: 2,
        },
    );
    let p = &game.state().person;
    assert_eq!(p.name, "Clara Weber");
    assert_eq!(p.born.year(), 1860);
    assert_eq!(p.home, game.catalog().countries.id("AAA").unwrap());
    assert_eq!(p.ceo, Some(game.player()));
    assert_eq!(p.children.len(), 2);
    for c in &p.children {
        assert!(
            c.born >= date(1882, 1, 1) && c.born <= date(1900, 1, 1),
            "{:?}",
            c.born
        );
        assert!(c.name.ends_with(" Weber"), "{}", c.name);
    }
    assert!(matches!(p.history[0].kind, LifeEventKind::ChildBorn { .. }));
    assert!(
        p.history
            .iter()
            .any(|e| matches!(e.kind, LifeEventKind::Start { .. }))
    );
    // An age outside 18–60 is brought within.
    let old = game_with(
        catalog(),
        PersonSettings {
            birth_year: Some(1800),
            ..PersonSettings::default()
        },
    );
    assert_eq!(old.state().person.born.year(), 1840);
}

#[test]
fn by_default_the_person_is_thirty_and_named_after_the_country() {
    let game = game_with(catalog(), PersonSettings::default());
    let p = &game.state().person;
    assert!(!p.name.is_empty());
    let age = person::age(game.state(), game.state().date);
    assert!(age == 29 || age == 30, "{age}");
    assert_eq!(p.born.year(), 1870);
    assert!(p.married);
    assert!(p.children.is_empty());
    let view = views::overview(&game);
    assert_eq!(view.person.unwrap().name, p.name);
}

#[test]
fn children_come_while_married_and_of_age() {
    let mut c = catalog();
    c.person.child_chance = 1.0;
    let mut game = game_with(
        c.clone(),
        PersonSettings {
            birth_year: Some(1870),
            ..PersonSettings::default()
        },
    );
    let keys = to_next_month(&mut game);
    assert!(keys.iter().any(|k| k == keys::PERSON_CHILD), "{keys:?}");
    assert_eq!(game.state().person.children.len(), 1);
    for _ in 0..6 {
        to_next_month(&mut game);
    }
    assert_eq!(game.state().person.children.len(), 4, "at most four");
    // Unmarried: none.
    let mut single = game_with(
        c.clone(),
        PersonSettings {
            birth_year: Some(1870),
            married: false,
            ..PersonSettings::default()
        },
    );
    to_next_month(&mut single);
    assert!(single.state().person.children.is_empty());
    // Too old: none.
    let mut old = game_with(
        c,
        PersonSettings {
            birth_year: Some(1850),
            ..PersonSettings::default()
        },
    );
    to_next_month(&mut old);
    assert!(old.state().person.children.is_empty());
}

#[test]
fn a_child_of_age_gets_a_manager_card_only_for_the_person() {
    let mut game = game_with(
        catalog(),
        PersonSettings {
            birth_year: Some(1870),
            children: 1,
            ..PersonSettings::default()
        },
    );
    let (_, works) = mine_and_works(&mut game);
    assert!(game.state().person.children[0].manager.is_none());
    {
        let child = &mut game.state_mut().person.children[0];
        child.born = date(1874, 12, 15);
        child.manager = None;
    }
    let keys = to_next_month(&mut game);
    assert!(keys.iter().any(|k| k == keys::PERSON_CAREER), "{keys:?}");
    let state = game.state();
    let child = &state.person.children[0];
    let id = child.manager.expect("a card at 25");
    let m = &state.managers[&id];
    assert!(m.family && m.job.is_none());
    assert_eq!(m.name, child.name);
    assert_eq!(m.born, Some(child.born));
    // Only the person's companies may employ the child.
    assert_eq!(
        person::check_family(state, CompanyId(7), m),
        Err(CommandError::FamilyOnly)
    );
    assert!(person::check_family(state, game.player(), m).is_ok());
    // The market lists the child first.
    let market = views::manager_market(&game, &format!("standort:{}", works.0), "produktion")
        .expect("a market");
    assert_eq!(market.candidates[0].manager.id, id.0);
    assert!(market.candidates[0].manager.family);
    // It stays in the market, also beyond the monthly leaving.
    game.apply(Command::HireManager {
        manager: id,
        position: specialist(works, "produktion"),
    })
    .unwrap();
    let job = game
        .state_mut()
        .managers
        .get_mut(&id)
        .unwrap()
        .job
        .as_mut()
        .unwrap();
    job.satisfaction = Some(0);
    for _ in 0..3 {
        to_next_month(&mut game);
    }
    assert!(
        game.state().managers[&id].job.is_some(),
        "a child does not resign"
    );
    let view = views::person(&game);
    assert!(view.children[0].position.is_some());
}

#[test]
fn the_person_hands_over_the_lead_and_takes_it_back() {
    let mut game = game_with(catalog(), PersonSettings::default());
    mine_and_works(&mut game);
    let ceo = Position::new(Unit::Board, Role::Head);
    let id = free(&game)[0];
    game.apply(Command::HireManager {
        manager: id,
        position: ceo.clone(),
    })
    .unwrap();
    let keys = to_next_month(&mut game);
    assert!(
        keys.iter().any(|k| k == keys::PERSON_CEO_HANDED_OVER),
        "{keys:?}"
    );
    assert_eq!(game.state().person.ceo, None);
    game.apply(Command::DismissManager { manager: id }).unwrap();
    let keys = to_next_month(&mut game);
    assert!(
        keys.iter().any(|k| k == keys::PERSON_CEO_TAKEN_BACK),
        "{keys:?}"
    );
    assert_eq!(game.state().person.ceo, Some(game.player()));
    let history = &game.state().person.history;
    assert!(matches!(
        history[history.len() - 2].kind,
        LifeEventKind::CeoHandedOver { .. }
    ));
    let view = views::person(&game);
    assert!(view.holdings[0].person_ceo);
    assert!((view.holdings[0].share - 1.0).abs() < 1e-12);
}

#[test]
fn a_child_manager_who_dies_is_mourned() {
    let mut c = catalog();
    c.life.mortality_chance = 1.0;
    c.life.life_expectancy.default =
        Some(crate::time_series::TimeSeries::new(vec![(1900, 30.0), (2100, 30.0)]).unwrap());
    let mut game = game_with(
        c,
        PersonSettings {
            birth_year: Some(1870),
            children: 1,
            ..PersonSettings::default()
        },
    );
    game.state_mut().person.children[0].born = date(1845, 1, 1);
    // The card first, then the life table of the managers.
    to_next_month(&mut game);
    let id = game.state().person.children[0].manager.expect("a card");
    assert_eq!(
        aging::age_of(&game.state().managers[&id], game.state().date),
        Some(55)
    );
    let keys = to_next_month(&mut game);
    let p = &game.state().person;
    assert!(p.children[0].died.is_some(), "{keys:?}");
    assert!(
        p.history
            .iter()
            .any(|e| matches!(e.kind, LifeEventKind::ChildDied { .. }))
    );
}

#[test]
fn older_saves_get_a_standard_person() {
    let catalog = Arc::new(catalog());
    let mut game = game_with((*catalog).clone(), PersonSettings::default());
    game.state_mut().person = Person::default();
    let loaded = save::decode(&save::encode(&game), catalog).unwrap().game;
    let p = &loaded.state().person;
    assert!(!p.name.is_empty());
    assert_eq!(person::age(loaded.state(), loaded.state().date), 30);
    assert_eq!(p.ceo, Some(loaded.player()));
    assert!(matches!(p.history[0].kind, LifeEventKind::Takeover { .. }));
}
