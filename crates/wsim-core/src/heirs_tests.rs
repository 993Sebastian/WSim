//! Scenario tests for death, heir and hand-over of the person (PE6) with
//! `test_support::aging` and the person's money of `test_support::private`.

use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{Catalog, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::heirs;
use crate::management_tests::usd;
use crate::message::keys;
use crate::money::Money;
use crate::save;
use crate::state::{
    Child, GameSettings, Holder, LifeEventKind, PersonSettings, PrivateFlow, StartForm,
};

fn catalog() -> Catalog {
    let mut c = test_support::aging();
    c.person = test_support::private();
    c
}

/// A person with `money` and an investment firm founded with `capital` of it.
fn game(money: f64, capital: f64) -> Game {
    let catalog = Arc::new(catalog());
    let settings = GameSettings {
        seed: 11,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(money),
        start_form: StartForm::Investor,
        company_name: "Haus AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: false,
        person: PersonSettings {
            married: false,
            ..PersonSettings::default()
        },
    };
    let mut game = Game::new(catalog, settings).unwrap();
    if capital > 0.0 {
        let country = game.catalog().countries.id("AAA").unwrap();
        game.apply(Command::FoundCompany {
            name: "Haus AG".into(),
            form: StartForm::Investor,
            country,
            capital: usd(capital),
        })
        .unwrap();
        game.apply(Command::SetPersonSalary {
            amount: Money::ZERO,
        })
        .unwrap();
    }
    game
}

/// The person is very old: months pass until it dies; the message keys on the way.
fn die(game: &mut Game) -> Vec<String> {
    game.state_mut().person.born = Date::new(1780, 1, 1).unwrap();
    let before = game.state().person.ancestors.len();
    let mut keys = Vec::new();
    for _ in 0..120 {
        keys.extend(to_next_month(game));
        if game.state().person.ancestors.len() > before {
            return keys;
        }
    }
    panic!("the person did not die");
}

fn child(name: &str, born: (i32, u32, u32)) -> Child {
    Child {
        name: name.into(),
        born: Date::new(born.0, born.1, born.2).unwrap(),
        manager: None,
        died: None,
    }
}

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

#[test]
fn without_a_child_a_nephew_or_niece_inherits_and_pays_the_tax() {
    let mut game = game(1_000_000.0, 500_000.0);
    let main = game.player();
    let catalog = game.catalog().clone();
    let old = game.state().person.name.clone();
    let estate = heirs::estate(&catalog, game.state());
    let before = game.state().person.account.balance;
    let keys = die(&mut game);
    assert!(keys.iter().any(|k| k == keys::PERSON_DIED), "{keys:?}");
    let p = &game.state().person;
    assert_ne!(p.name, old);
    // Same family name, age 25 to 40.
    assert_eq!(
        p.name.split_whitespace().last(),
        old.split_whitespace().last()
    );
    let age = crate::person::age(game.state(), game.state().date);
    assert!((25..=40).contains(&age), "{age}");
    assert_eq!(p.ancestors.len(), 1);
    assert!(p.ancestors[0].died);
    assert!(p.history.iter().any(|e| matches!(
        &e.kind,
        LifeEventKind::Succession { from, died: true, .. } if *from == old
    )));
    // A fifth of the estate as tax, paid from the account; shares and company stay.
    let tax = p
        .history
        .iter()
        .find_map(|e| match e.kind {
            LifeEventKind::Succession { tax, .. } => Some(tax),
            _ => None,
        })
        .unwrap();
    assert!(tax > Money::ZERO);
    assert!((tax.to_usd() - estate.to_usd() * 0.2).abs() < estate.to_usd() * 0.05);
    assert!(p.account.balance < before);
    assert_eq!(game.main_company(), Some(main));
    assert_eq!(crate::person::share(game.state(), main), 1.0);
    assert!(!game.is_over());
}

#[test]
fn the_chosen_child_inherits_and_leaves_its_post() {
    let mut game = game(1_000_000.0, 500_000.0);
    {
        let p = &mut game.state_mut().person;
        p.children = vec![
            child("Ältere Erbin", (1870, 3, 1)),
            child("Jüngerer Erbe", (1872, 5, 1)),
        ];
    }
    // Both children get their manager cards at the next month start.
    to_next_month(&mut game);
    let younger = game.state().person.children[1].manager.expect("card");
    assert_eq!(
        game.apply(Command::SetHeir { child: Some(7) }),
        Err(CommandError::NoSuchChild)
    );
    game.apply(Command::SetHeir { child: Some(1) }).unwrap();
    assert_eq!(heirs::heir(game.state()), Some(1));
    die(&mut game);
    let s = game.state();
    assert_eq!(s.person.name, "Jüngerer Erbe");
    assert_eq!(s.person.born, Date::new(1872, 5, 1).unwrap());
    // The heir's card ended; the sister stays a manager of the family.
    assert!(!s.managers.contains_key(&younger));
    assert_eq!(s.person.heir, None);
    assert!(
        s.managers
            .values()
            .any(|m| m.name == "Ältere Erbin" && m.family)
    );
}

#[test]
fn the_eldest_child_inherits_without_a_choice() {
    let mut game = game(200_000.0, 0.0);
    {
        let p = &mut game.state_mut().person;
        p.children = vec![child("Bert", (1880, 1, 1)), child("Anna", (1878, 1, 1))];
        p.children.push(Child {
            died: Some(Date::new(1899, 1, 1).unwrap()),
            ..child("Carl", (1870, 1, 1))
        });
    }
    assert_eq!(heirs::heir(game.state()), Some(1));
    die(&mut game);
    assert_eq!(game.state().person.name, "Anna");
}

#[test]
fn without_money_shares_are_sold_for_the_tax() {
    let mut game = game(1_000_000.0, 990_000.0);
    let main = game.player();
    let catalog = game.catalog().clone();
    let estate = heirs::estate(&catalog, game.state());
    let cash = game.state().person.account.balance;
    assert!(cash.to_usd() < estate.to_usd() * 0.2);
    game.apply(Command::HandOver {}).unwrap();
    let s = game.state();
    // Part of the company went to investors; the person keeps control.
    let held = crate::person::share(s, main);
    assert!(held < 1.0 && held > 0.5, "{held}");
    assert!(
        s.companies[main.index()]
            .owners
            .iter()
            .any(|o| o.holder == Holder::Investors)
    );
    assert!(!s.person.account.balance.is_negative());
    // The revaluation leaves no gain to tax on the forced sale.
    assert_eq!(
        s.person
            .account
            .month
            .get(&PrivateFlow::GainTax)
            .copied()
            .unwrap_or_default(),
        Money::ZERO
    );
    let p = &s.person;
    assert_eq!(p.ancestors.len(), 1);
    assert!(!p.ancestors[0].died);
    assert!(s.companies[main.index()].ledger.is_balanced());
}

#[test]
fn the_hint_comes_each_january_from_seventy_without_an_heir() {
    let mut game = game(1_000_000.0, 0.0);
    game.state_mut().person.born = Date::new(1829, 6, 1).unwrap();
    // Low mortality for the test: no death in the next months.
    let mut keys = Vec::new();
    while game.state().date < Date::new(1901, 1, 2).unwrap() {
        let report = game.advance(RoundLength::Day, |_| {});
        keys.extend(report.messages.into_iter().map(|m| m.key));
        if !game.state().person.ancestors.is_empty() {
            return;
        }
    }
    assert!(
        keys.iter().any(|k| k == keys::PERSON_SUCCESSION_HINT),
        "{keys:?}"
    );
}

#[test]
fn successions_survive_saving() {
    let mut game = game(1_000_000.0, 500_000.0);
    die(&mut game);
    let mut loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(game.state_hash(), loaded.state_hash());
    to_next_month(&mut game);
    to_next_month(&mut loaded);
    assert_eq!(game.state_hash(), loaded.state_hash());
}

#[test]
fn a_family_lasts_two_hundred_years() {
    let mut game = game(5_000_000.0, 0.0);
    game.apply(Command::SetLifestyle {
        level: crate::state::Lifestyle::Modest,
    })
    .unwrap();
    let end = Date::new(2100, 12, 1).unwrap();
    while game.state().date < end && !game.is_over() {
        game.advance(RoundLength::Month, |_| {});
    }
    assert!(!game.is_over());
    let generations = game.state().person.ancestors.len() + 1;
    assert!(generations >= 3, "{generations}");
}
