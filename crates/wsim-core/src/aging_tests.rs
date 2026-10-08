//! Scenario tests for the managers' age, retirement and death (PE1) with
//! `test_support::aging`.

use std::sync::Arc;

use crate::aging;
use crate::calendar::{Date, RoundLength};
use crate::catalog::{Catalog, test_support};
use crate::command::{Command, CommandError};
use crate::decision::{ChoiceKind, Topic};
use crate::game::Game;
use crate::management::{self, ConcernAnswer};
use crate::management_tests::{free, head, mine_and_works, new_game, specialist};
use crate::message::keys;
use crate::save;
use crate::state::{ConcernReason, ConcernStatus, Departure, ManagerId, Position, SiteId};

fn date(y: i32, m: u32, d: u32) -> Date {
    Date::new(y, m, d).unwrap()
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

/// A free candidate takes a position; his day of birth and personal deviation are set.
fn hire(game: &mut Game, position: Position, born: Date, offset: i8) -> ManagerId {
    let id = free(game)
        .into_iter()
        .find(|&id| {
            let m = &game.state().managers[&id];
            aging::retirement(game.catalog(), m).is_some_and(|d| d > date(1905, 1, 1))
        })
        .expect("a candidate");
    game.apply(Command::HireManager {
        manager: id,
        position,
    })
    .unwrap();
    let m = game.state_mut().managers.get_mut(&id).unwrap();
    m.born = Some(born);
    m.retire_offset = offset;
    id
}

fn succession(game: &Game) -> Option<crate::state::Concern> {
    game.state()
        .concerns
        .iter()
        .find(|c| c.status == ConcernStatus::Open && c.decision.topic == Topic::Succession)
        .cloned()
}

fn option(c: &crate::state::Concern, kind: ChoiceKind) -> Option<usize> {
    c.decision.choices.iter().position(|o| o.kind == kind)
}

fn works_game(catalog: Catalog) -> (Game, SiteId) {
    let mut game = new_game(catalog);
    let (_, works) = mine_and_works(&mut game);
    (game, works)
}

#[test]
fn ages_count_full_years() {
    let born = date(1850, 3, 15);
    assert_eq!(aging::age(born, date(1900, 3, 14)), 49);
    assert_eq!(aging::age(born, date(1900, 3, 15)), 50);
    assert_eq!(aging::age(date(1852, 2, 29), date(1900, 2, 28)), 47);
    assert_eq!(aging::age(date(1852, 2, 29), date(1900, 3, 1)), 48);
}

#[test]
fn candidates_are_born_by_their_level() {
    let game = new_game(test_support::aging());
    let c = game.catalog();
    let today = game.state().date;
    assert!(!game.state().managers.is_empty());
    for m in game.state().managers.values() {
        let age = f64::from(aging::age_of(m, today).expect("born"));
        let strength = management::strength(m);
        let span = c.life.entry_age[c
            .life
            .level_strength
            .iter()
            .filter(|&&s| strength >= s)
            .count()];
        // Drawn in days: a candidate of just 26 may be a day short of it.
        assert!(
            age + 1.0 >= span.min && age <= span.max,
            "{age} outside {span:?} at strength {strength}"
        );
        assert!((-5..=5).contains(&m.retire_offset));
    }
    // Without the data nobody ages.
    let plain = new_game(test_support::management());
    assert!(plain.state().managers.values().all(|m| m.born.is_none()));
}

#[test]
fn the_retirement_is_the_first_month_start_at_the_retirement_age() {
    let game = new_game(test_support::aging());
    let mut m = game.state().managers.values().next().unwrap().clone();
    m.born = Some(date(1850, 3, 15));
    m.retire_offset = -2;
    let c = game.catalog();
    assert_eq!(aging::retirement(c, &m), Some(date(1913, 4, 1)));
    m.born = Some(date(1850, 3, 1));
    assert_eq!(aging::retirement(c, &m), Some(date(1913, 3, 1)));
    m.extended = 3;
    assert_eq!(aging::retirement(c, &m), Some(date(1916, 3, 1)));
    assert_eq!(aging::retirement_age(c, &m), Some(66));
}

#[test]
fn age_lowers_the_readiness_for_risks_and_the_experience() {
    let (mut game, works) = works_game(test_support::aging());
    // Turns 44 in January: the first full point of 0,3 a year from 40.
    let id = hire(&mut game, head(works), date(1856, 1, 15), 5);
    game.state_mut().managers.get_mut(&id).unwrap().risk = 50;
    to_next_month(&mut game);
    assert_eq!(game.state().managers[&id].risk, 49);
    let c = game.catalog();
    let today = game.state().date;
    let mut m = game.state().managers[&id].clone();
    assert!((aging::experience_factor(c, &m, today) - 1.0).abs() < 1e-12);
    m.born = Some(date(1870, 1, 1));
    assert!((aging::experience_factor(c, &m, today) - 1.5).abs() < 1e-12);
    m.born = Some(date(1840, 1, 1));
    assert!((aging::experience_factor(c, &m, today) - 0.5).abs() < 1e-12);
}

#[test]
fn the_succession_is_asked_a_year_ahead_and_the_successor_takes_over() {
    let (mut game, works) = works_game(test_support::aging());
    let boss = hire(&mut game, head(works), date(1870, 1, 1), 0);
    // Retires at 60 on 1 August 1900.
    let old = hire(
        &mut game,
        specialist(works, "produktion"),
        date(1840, 7, 20),
        -5,
    );
    assert_eq!(
        aging::retirement(game.catalog(), &game.state().managers[&old]),
        Some(date(1900, 8, 1))
    );
    let keys = to_next_month(&mut game);
    assert!(keys.iter().any(|k| k == keys::MANAGER_RETIRING), "{keys:?}");
    let c = succession(&game).expect("asked");
    assert_eq!(c.reason, ConcernReason::Retirement);
    assert_eq!(c.manager, boss, "the head of the site asks");
    assert_eq!(c.deadline, date(1900, 7, 31));
    assert!(option(&c, ChoiceKind::Extend).is_some());
    assert!(option(&c, ChoiceKind::Vacant).is_some());
    let pick = option(&c, ChoiceKind::Successor).expect("a candidate");
    assert_eq!(c.recommended, pick, "the only candidate is recommended");
    game.apply(Command::AnswerConcern {
        concern: c.id,
        answer: ConcernAnswer::Choose(pick),
    })
    .unwrap();
    let position = specialist(works, "produktion");
    let next = aging::successor_of(game.state(), game.player(), &position).expect("waits");
    assert_eq!(
        management::holder(game.state(), game.player(), &position),
        Some(old),
        "the old one holds the position until he retires"
    );
    // Asked once only.
    to_next_month(&mut game);
    assert!(succession(&game).is_none());
    let mut seen = Vec::new();
    while game.state().date < date(1900, 8, 1) {
        seen.extend(to_next_month(&mut game));
    }
    assert!(
        seen.iter().any(|k| k == keys::MANAGER_RETIRED_SUCCESSOR),
        "{seen:?}"
    );
    let state = game.state();
    assert!(!state.managers.contains_key(&old));
    assert_eq!(
        management::holder(state, game.player(), &position),
        Some(next)
    );
    let former = &state.company(game.player()).unwrap().former_managers;
    assert_eq!(former.len(), 1);
    assert_eq!(former[0].reason, Departure::Retired);
    assert_eq!(former[0].until, date(1900, 8, 1));
    assert!(state.company(game.player()).unwrap().ledger.is_balanced());
}

#[test]
fn without_an_answer_the_position_is_free() {
    let (mut game, works) = works_game(test_support::aging());
    hire(&mut game, head(works), date(1870, 1, 1), 0);
    let old = hire(
        &mut game,
        specialist(works, "produktion"),
        date(1840, 4, 10),
        -5,
    );
    to_next_month(&mut game);
    assert!(succession(&game).is_some());
    let mut seen = Vec::new();
    while game.state().date < date(1900, 5, 1) {
        seen.extend(to_next_month(&mut game));
    }
    assert!(seen.iter().any(|k| k == keys::MANAGER_RETIRED), "{seen:?}");
    assert!(!game.state().managers.contains_key(&old));
    let position = specialist(works, "produktion");
    assert_eq!(
        management::holder(game.state(), game.player(), &position),
        None
    );
    assert!(succession(&game).is_none(), "no concern after a retirement");
}

#[test]
fn a_content_manager_stays_longer() {
    let (mut game, works) = works_game(test_support::aging());
    hire(&mut game, head(works), date(1870, 1, 1), 0);
    let old = hire(
        &mut game,
        specialist(works, "produktion"),
        date(1840, 9, 10),
        -5,
    );
    to_next_month(&mut game);
    let c = succession(&game).expect("asked");
    game.state_mut()
        .managers
        .get_mut(&old)
        .unwrap()
        .job
        .as_mut()
        .unwrap()
        .satisfaction = Some(100);
    let salary = game.state().managers[&old].job.as_ref().unwrap().salary;
    let extend = option(&c, ChoiceKind::Extend).unwrap();
    game.apply(Command::AnswerConcern {
        concern: c.id,
        answer: ConcernAnswer::Choose(extend),
    })
    .unwrap();
    let m = &game.state().managers[&old];
    assert_eq!(m.extended, 3);
    assert_eq!(m.job.as_ref().unwrap().salary, salary.scale(1.2));
    assert_eq!(
        aging::retirement(game.catalog(), m),
        Some(date(1903, 10, 1))
    );
    // Not again within the period, and at most the years left.
    assert_eq!(
        game.apply(Command::ExtendContract {
            manager: old,
            years: 1
        }),
        Err(CommandError::NotRetiring)
    );
    to_next_month(&mut game);
    assert!(succession(&game).is_none());
}

#[test]
fn an_unhappy_manager_refuses_and_the_succession_is_asked_again() {
    let (mut game, works) = works_game(test_support::aging());
    hire(&mut game, head(works), date(1870, 1, 1), 0);
    let old = hire(
        &mut game,
        specialist(works, "produktion"),
        date(1840, 9, 10),
        -5,
    );
    to_next_month(&mut game);
    let c = succession(&game).expect("asked");
    game.state_mut()
        .managers
        .get_mut(&old)
        .unwrap()
        .job
        .as_mut()
        .unwrap()
        .satisfaction = Some(0);
    assert_eq!(
        game.apply(Command::ExtendContract {
            manager: old,
            years: 4
        }),
        Err(CommandError::ExtensionTooLong { max: 3 })
    );
    let extend = option(&c, ChoiceKind::Extend).unwrap();
    game.apply(Command::AnswerConcern {
        concern: c.id,
        answer: ConcernAnswer::Choose(extend),
    })
    .unwrap();
    assert!(game.state().managers[&old].extension_refused);
    let keys = to_next_month(&mut game);
    assert!(
        keys.iter().any(|k| k == keys::MANAGER_RETIRING_REFUSED),
        "{keys:?}"
    );
    let again = succession(&game).expect("asked again");
    assert!(option(&again, ChoiceKind::Extend).is_none());
}

#[test]
fn a_death_frees_the_position_and_asks_for_a_new_holder() {
    let mut catalog = test_support::aging();
    // Everyone from 50 dies within the month.
    catalog.life.mortality_chance = 1.0;
    catalog.life.life_expectancy.default =
        Some(crate::time_series::TimeSeries::new(vec![(1900, 30.0), (2100, 30.0)]).unwrap());
    let (mut game, works) = works_game(catalog);
    let boss = hire(&mut game, head(works), date(1870, 1, 1), 0);
    let old = hire(
        &mut game,
        specialist(works, "produktion"),
        date(1845, 6, 1),
        5,
    );
    let keys = to_next_month(&mut game);
    assert!(keys.iter().any(|k| k == keys::MANAGER_DIED), "{keys:?}");
    let state = game.state();
    assert!(!state.managers.contains_key(&old));
    assert!(state.managers.contains_key(&boss));
    let former = &state.company(game.player()).unwrap().former_managers;
    assert_eq!(former.last().unwrap().reason, Departure::Died);
    let c = succession(&game).expect("a new holder");
    assert_eq!(c.reason, ConcernReason::Vacancy);
    assert_eq!(c.manager, boss);
    assert!(option(&c, ChoiceKind::Extend).is_none());
    // Hiring settles the concern at the next month start.
    let pick = option(&c, ChoiceKind::Successor).expect("a candidate");
    game.apply(Command::AnswerConcern {
        concern: c.id,
        answer: ConcernAnswer::Choose(pick),
    })
    .unwrap();
    assert!(
        management::holder(
            game.state(),
            game.player(),
            &specialist(works, "produktion")
        )
        .is_some(),
        "a free position is taken at once"
    );
}

#[test]
fn candidates_leave_the_market_when_they_retire() {
    let mut game = new_game(test_support::aging());
    let id = free(&game)[0];
    let m = game.state_mut().managers.get_mut(&id).unwrap();
    m.born = Some(date(1835, 1, 10));
    m.retire_offset = 0;
    to_next_month(&mut game);
    assert!(!game.state().managers.contains_key(&id));
}

#[test]
fn old_saves_get_days_of_birth_when_loaded() {
    let catalog = Arc::new(test_support::aging());
    let (mut game, works) = works_game((*catalog).clone());
    let boss = hire(&mut game, head(works), date(1870, 1, 1), 0);
    for m in game.state_mut().managers.values_mut() {
        m.born = None;
        m.retire_offset = 0;
    }
    let loaded = save::decode(&save::encode(&game), catalog.clone())
        .unwrap()
        .game;
    let again = save::decode(&save::encode(&game), catalog).unwrap().game;
    let state = loaded.state();
    let today = state.date;
    assert!(state.managers.values().all(|m| m.born.is_some()));
    assert_eq!(state.managers, again.state().managers, "drawn alike");
    // The head of a site is drawn as one of a site.
    let age = f64::from(aging::age_of(&state.managers[&boss], today).unwrap());
    let span = loaded.catalog().life.entry_age[0];
    assert!(age + 1.0 >= span.min && age <= span.max);
}
