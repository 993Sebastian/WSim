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
    game_with(catalog(), money, capital)
}

fn game_with(catalog: Catalog, money: f64, capital: f64) -> Game {
    let catalog = Arc::new(catalog);
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
        research_mandatory: false,
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
            company: None,
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
        private_school: false,
        school_days: 0,
        ..Child::default()
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
            private_school: false,
            school_days: 0,
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

fn legacy_model() -> crate::catalog::LegacyModel {
    crate::catalog::LegacyModel {
        education_from: 16,
        education_cost: 3.0,
        education_points: 12.0,
        ambition_points: 8.0,
        talent_points: 15.0,
        traits: [20.0, 100.0],
        interest_per_month: 1.0,
        interest_base: 15.0,
        educations: [vec![0], vec![0], vec![0], vec![0]],
        partner_share: 0.5,
        firm_relief: 0.5,
        dispute_chance: 1.0,
        dispute_sale: 0.5,
    }
}

fn legacy_game(married: bool, kids: usize) -> Game {
    let mut c = catalog();
    c.person.legacy = Some(legacy_model());
    let mut game = game_with(c, 1_000_000.0, 500_000.0);
    let p = &mut game.state_mut().person;
    p.married = married;
    p.children = (0..kids)
        .map(|i| {
            child(
                &format!("Kind {i}"),
                (1895 + i32::try_from(i).unwrap(), 3, 1),
            )
        })
        .collect();
    game
}

#[test]
fn the_statutory_succession_gives_the_partner_half_and_the_children_the_rest() {
    use crate::state::{Heir, WillShare};
    let mut game = legacy_game(true, 2);
    let c = game.catalog().clone();
    let shares = heirs::shares(&c, game.state());
    assert_eq!(shares.len(), 3);
    assert_eq!(shares[0], (Heir::Partner, 0.5));
    assert!((shares[1].1 - 0.25).abs() < 1e-12);
    // A will must add up to 100 percent and name people who can inherit.
    let bad = |g: &mut Game, shares: Vec<WillShare>| g.apply(Command::SetWill { shares });
    assert_eq!(
        bad(
            &mut game,
            vec![WillShare {
                heir: Heir::Child(0),
                percent: 90
            }]
        ),
        Err(CommandError::WillInvalid)
    );
    assert_eq!(
        bad(
            &mut game,
            vec![WillShare {
                heir: Heir::Child(7),
                percent: 100
            }]
        ),
        Err(CommandError::WillInvalid)
    );
    bad(
        &mut game,
        vec![
            WillShare {
                heir: Heir::Child(0),
                percent: 70,
            },
            WillShare {
                heir: Heir::Child(1),
                percent: 30,
            },
        ],
    )
    .unwrap();
    let shares = heirs::shares(&c, game.state());
    assert_eq!(shares, vec![(Heir::Child(0), 0.7), (Heir::Child(1), 0.3)]);
    game.apply(Command::SetHeir { child: Some(0) }).unwrap();
    assert!((heirs::successor_share(&c, game.state()) - 0.7).abs() < 1e-12);
}

#[test]
fn the_co_heirs_take_their_part_in_shares_and_money() {
    use crate::state::{Heir, WillShare};
    let play = |will: bool| {
        let mut game = legacy_game(false, 2);
        game.apply(Command::SetHeir { child: Some(0) }).unwrap();
        if will {
            game.apply(Command::SetWill {
                shares: vec![
                    WillShare {
                        heir: Heir::Child(0),
                        percent: 50,
                    },
                    WillShare {
                        heir: Heir::Child(1),
                        percent: 50,
                    },
                ],
            })
            .unwrap();
        }
        let main = game.player();
        die(&mut game);
        let held = crate::person::share(game.state(), main);
        (
            game.state().person.account.balance,
            held,
            game.state().person.heir_stakes.clone(),
        )
    };
    // Without a will the two children share equally as well: the half goes to the sibling.
    let (cash_a, held_a, stakes_a) = play(false);
    let (cash_b, held_b, stakes_b) = play(true);
    assert_eq!(cash_a, cash_b);
    assert!((held_a - held_b).abs() < 1e-12);
    assert!(held_a < 0.6 && held_a > 0.0, "{held_a}");
    assert!(!stakes_a.is_empty() && stakes_a == stakes_b);
    // A sole heir keeps everything.
    let mut solo = legacy_game(false, 1);
    let main = solo.player();
    die(&mut solo);
    assert!(crate::person::share(solo.state(), main) > 0.99);
    assert!(solo.state().person.heir_stakes.is_empty());
}

#[test]
fn company_shares_ease_the_inheritance_tax() {
    let game = legacy_game(false, 1);
    let c = game.catalog().clone();
    let with = heirs::tax_due(&c, game.state());
    let mut plain = catalog();
    plain.person.legacy = None;
    let g2 = game_with(plain, 1_000_000.0, 500_000.0);
    let without = heirs::tax_due(g2.catalog(), g2.state());
    assert!(with < without, "{with:?} {without:?}");
}

#[test]
fn the_co_heirs_quarrel_and_sell_part_of_their_shares() {
    let mut game = legacy_game(false, 2);
    let main = game.player();
    game.apply(Command::SetHeir { child: Some(0) }).unwrap();
    die(&mut game);
    let before = game.state().person.heir_stakes[&main];
    let investors_before =
        crate::stock::stake(&game.state().companies[main.index()], Holder::Investors);
    // The next January the quarrel comes (the chance is 1 in this test).
    let mut quarrel = false;
    for _ in 0..13 {
        quarrel |= to_next_month(&mut game)
            .iter()
            .any(|k| k == keys::HEIRS_DISPUTE);
    }
    assert!(quarrel);
    assert!(game.state().person.heir_stakes[&main] < before);
    let investors = crate::stock::stake(&game.state().companies[main.index()], Holder::Investors);
    assert!(investors > investors_before);
}

#[test]
fn education_is_chosen_from_sixteen_costs_money_and_shapes_the_card() {
    use crate::state::Education;
    let mut game = legacy_game(false, 0);
    let p = &mut game.state_mut().person;
    // A child of 17 and one of 10.
    p.children.push(child("Grosse", (1883, 1, 1)));
    p.children.push(child("Kleine", (1890, 1, 1)));
    assert_eq!(
        game.apply(Command::SetEducation {
            child: 1,
            education: Education::Engineer
        }),
        Err(CommandError::NotPossible)
    );
    game.apply(Command::SetEducation {
        child: 0,
        education: Education::Engineer,
    })
    .unwrap();
    let cash = game.state().person.account.balance;
    to_next_month(&mut game);
    to_next_month(&mut game);
    assert!(game.state().person.account.balance < cash);
    assert!(game.state().person.children[0].education_months >= 1);
}

#[test]
fn a_holding_keeps_the_shares_together_and_gives_the_co_heirs_a_claim() {
    let mut game = legacy_game(false, 2);
    game.apply(Command::SetHeir { child: Some(0) }).unwrap();
    game.state_mut().person.trust.holding = true;
    let main = game.player();
    let before = crate::person::share(game.state(), main);
    die(&mut game);
    let after = crate::person::share(game.state(), main);
    assert!((before - after).abs() < 1e-12, "{before} {after}");
    let claim = crate::trust::claim(game.state(), main);
    assert!(claim > 0.0 && claim < 1.0, "{claim}");
    // Without the holding the shares move and no claim arises.
    let mut free = legacy_game(false, 2);
    free.apply(Command::SetHeir { child: Some(0) }).unwrap();
    die(&mut free);
    assert!(crate::person::share(free.state(), main) < before);
    assert_eq!(crate::trust::claim(free.state(), main), 0.0);
}
