//! Scenario tests for the start-ups (SU1) with `test_support::research` and
//! `test_support::management`.

use std::sync::Arc;

use crate::board_tests::member;
use crate::calendar::{Date, RoundLength};
use crate::catalog::{
    Catalog, DepartmentKind, Inventor, Provenance, Technology, VentureModel, VenturePhase,
    test_support,
};
use crate::command::Command;
use crate::game::Game;
use crate::management_tests::{hire_sharp, mine_and_works, usd};
use crate::message::keys;
use crate::research;
use crate::state::{
    GameSettings, Holder, StartForm, Venture, VentureFailure, VentureStatus, VentureTarget,
};
use crate::ventures;

/// Two short sure phases, investors at once, one start-up a month.
fn model() -> VentureModel {
    let phase = |key: &str, months, capital, valuation| VenturePhase {
        key: key.to_owned(),
        months,
        capital: usd(capital),
        chance: 1.0,
        valuation,
    };
    VentureModel {
        labels: vec![("erfinder".into(), 1900), ("startups".into(), 1990)],
        per_year: 12.0,
        frequencies: vec![("keine".into(), 0.0), ("normal".into(), 1.0)],
        default_frequency: 1,
        new_share: 1.0,
        lead_years_max: 15,
        phases: vec![
            phase("idee", 2, 100_000.0, 1.0),
            phase("prototyp", 3, 200_000.0, 2.0),
        ],
        reference_gdp_usd: 5_000.0,
        capital_factor: (0.05, 2.0),
        lead_capital: 0.1,
        lead_chance: 0.0,
        chance_min: 0.05,
        investor_chance: 1.0,
        deadline_months: 3,
        blur: 0.5,
        chance_levels: (0.2, 0.4),
        keep_years: 1,
        inventors: Vec::new(),
        provenance: Provenance::default(),
    }
}

fn catalog_with(model: VentureModel) -> Catalog {
    let mut c = test_support::research();
    c.ventures = model;
    c
}

fn game_with(catalog: Catalog, seed: u64, ventures: f64) -> Game {
    let catalog = Arc::new(catalog);
    let settings = GameSettings {
        seed,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(1_000_000.0),
        start_form: StartForm::Workshop,
        company_name: "Labor AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures,
    };
    Game::new(catalog, settings).unwrap()
}

/// Days until `date`, with the keys of all messages.
fn until(game: &mut Game, date: Date) -> Vec<String> {
    let mut keys = Vec::new();
    while game.state().date < date {
        let report = game.advance(RoundLength::Day, |_| {});
        keys.extend(report.messages.into_iter().map(|m| m.key));
    }
    keys
}

fn date(y: i32, m: u32, d: u32) -> Date {
    Date::new(y, m, d).unwrap()
}

fn venture(game: &Game, id: u32) -> &Venture {
    game.state().ventures.iter().find(|v| v.id == id).unwrap()
}

/// More technologies a few years ahead, all needing only smelting.
fn add_technologies(c: &mut Catalog, n: usize, year: i32) {
    let smelting = c.technologies.id("schmelzen").unwrap();
    let field = c.technologies.get(smelting).field;
    for i in 0..n {
        c.technologies
            .insert(
                &format!("neu_{i}"),
                Technology {
                    field,
                    invention_year: year,
                    prerequisites: vec![smelting],
                    research_effort: Some(500.0),
                    provenance: Provenance::default(),
                },
            )
            .unwrap();
    }
}

#[test]
fn a_sure_start_up_invents_its_technology_early() {
    let mut game = game_with(catalog_with(model()), 3, 1.0);
    let catalog = game.catalog().clone();
    let turbine = catalog.technologies.id("turbine").unwrap();
    // No start-up before the first month start.
    assert!(game.state().ventures.is_empty());
    until(&mut game, date(1900, 2, 1));
    // The only technology within reach: turbine, two years ahead of history.
    let v = venture(&game, 0).clone();
    assert_eq!(v.target, VentureTarget::Technology(turbine));
    assert_eq!(v.lead, 2.0);
    assert_eq!(v.phase, 0);
    assert_eq!(v.owners.len(), 1);
    assert_eq!(v.owners[0].holder, Holder::Private);
    // Capital at the reference income, 10 % more per year ahead.
    assert_eq!(v.capital, usd(120_000.0));
    assert_eq!(v.round_until, Some(date(1900, 5, 1)));
    // A month later investors fund the round: half the shares at a valuation of 1.
    until(&mut game, date(1900, 3, 1));
    let v = venture(&game, 0).clone();
    assert_eq!(v.round_until, None);
    assert_eq!(v.raised, usd(120_000.0));
    assert_eq!(v.phase_until, Some(date(1900, 5, 1)));
    let share = |v: &Venture, h: Holder| {
        v.owners
            .iter()
            .filter(|s| s.holder == h)
            .map(|s| s.share)
            .sum::<f64>()
    };
    assert!((share(&v, Holder::Private) - 0.5).abs() < 1e-12);
    assert!((share(&v, Holder::Investors) - 0.5).abs() < 1e-12);
    let before = research::effort(&catalog, game.state(), turbine, date(1900, 6, 1))
        .unwrap()
        .factor;
    assert!(before > 1.0, "ahead of history: {before}");
    // Phase two: its own round with a valuation of two dilutes everyone by a third.
    let keys = until(&mut game, date(1900, 9, 2));
    let v = venture(&game, 0).clone();
    assert_eq!(v.status, VentureStatus::Succeeded(date(1900, 9, 1)));
    assert!((share(&v, Holder::Private) - 1.0 / 3.0).abs() < 1e-12);
    assert!((share(&v, Holder::Investors) - 2.0 / 3.0).abs() < 1e-12);
    // The technology counts as invented: researching it gets cheaper for everyone.
    assert_eq!(
        *game.state().inventions.get(turbine),
        Some(date(1900, 9, 1))
    );
    assert!(keys.iter().any(|k| k == keys::VENTURE_INVENTION));
    let after = research::effort(&catalog, game.state(), turbine, date(1901, 1, 1))
        .unwrap()
        .factor;
    assert!(after <= 1.0 && after < before, "{after} after {before}");
    // Nothing left to found a start-up for.
    until(&mut game, date(1901, 1, 1));
    assert_eq!(game.state().ventures.len(), 1);
}

#[test]
fn an_unfunded_round_fails_and_closed_ones_leave_the_list() {
    let mut m = model();
    m.investor_chance = 0.0;
    let mut game = game_with(catalog_with(m), 3, 1.0);
    until(&mut game, date(1900, 5, 2));
    assert_eq!(
        venture(&game, 0).status,
        VentureStatus::Failed(date(1900, 5, 1), VentureFailure::Funding)
    );
    // The technology is free again: a new start-up takes it up.
    until(&mut game, date(1900, 6, 2));
    let next = game.state().ventures.iter().find(|v| v.id == 1).unwrap();
    assert_eq!(next.status, VentureStatus::Active);
    // A year after failing, the first one leaves the list.
    until(&mut game, date(1901, 5, 2));
    assert!(game.state().ventures.iter().all(|v| v.id != 0));
    assert!(
        game.state()
            .ventures
            .iter()
            .all(|v| v.id > 0 && v.owners == crate::state::Stake::sole(Holder::Private))
    );
}

#[test]
fn most_start_ups_fail_by_their_chances() {
    // Thirty technologies, halves at each of two phases: a quarter succeeds.
    let mut m = model();
    m.phases[0].chance = 0.5;
    m.phases[1].chance = 0.5;
    m.per_year = 24.0;
    m.keep_years = 50;
    let mut c = catalog_with(m);
    add_technologies(&mut c, 30, 1914);
    let mut game = game_with(c, 11, 1.0);
    until(&mut game, date(1912, 1, 1));
    let closed: Vec<&Venture> = game
        .state()
        .ventures
        .iter()
        .filter(|v| v.status != VentureStatus::Active)
        .collect();
    let won = closed
        .iter()
        .filter(|v| matches!(v.status, VentureStatus::Succeeded(_)))
        .count();
    // Successes take their technology off the list; at most one each.
    assert!(won <= 31, "{won}");
    let tried = closed.len();
    assert!(tried > 60, "{tried}");
    let failed = (tried - won) as f64 / tried as f64;
    assert!((0.65..0.85).contains(&failed), "{failed}");
}

#[test]
fn the_setting_and_the_mean_set_how_many_are_founded() {
    let mut m = model();
    m.per_year = 6.0;
    m.investor_chance = 0.0;
    let mut c = catalog_with(m);
    add_technologies(&mut c, 10, 1910);
    // Half a start-up a month: about sixty in ten years.
    let mut game = game_with(c.clone(), 5, 1.0);
    until(&mut game, date(1909, 12, 2));
    let founded = game.state().next_venture;
    assert!((40..=80).contains(&founded), "{founded}");
    // Twice the setting, twice as many.
    let mut many = game_with(c.clone(), 5, 2.0);
    until(&mut many, date(1909, 12, 2));
    let more = many.state().next_venture;
    assert!(more > founded + 30, "{more} against {founded}");
    // None with the setting none.
    let mut none = game_with(c, 5, 0.0);
    until(&mut none, date(1903, 1, 1));
    assert_eq!(none.state().next_venture, 0);
    assert!(none.state().ventures.is_empty());
}

#[test]
fn start_ups_are_deterministic() {
    let mut m = model();
    m.phases[0].chance = 0.6;
    m.investor_chance = 0.5;
    m.new_share = 0.5;
    m.keep_years = 10;
    let mut c = catalog_with(m);
    add_technologies(&mut c, 6, 1908);
    c.research_model.development.levels = 3;
    let run = |seed| {
        let mut game = game_with(c.clone(), seed, 1.0);
        until(&mut game, date(1903, 1, 1));
        game.state().ventures.clone()
    };
    let a = run(21);
    assert!(a.len() > 5);
    assert_eq!(a, run(21));
    assert_ne!(a, run(22));
}

#[test]
fn an_improvement_takes_the_next_level_into_the_world() {
    // Without technologies within reach: improvements only, of what the world makes.
    let mut c = test_support::management();
    let mut m = model();
    m.new_share = 0.0;
    m.lead_years_max = 1;
    c.ventures = m;
    c.research_model.development.levels = 3;
    let iron = c.products.id("eisen").unwrap();
    let smelting = c.recipes.id("eisen_schmelzen");
    let mut game = crate::management_tests::new_game(c);
    until(&mut game, date(1900, 2, 2));
    assert!(game.state().ventures.is_empty(), "nobody makes iron yet");
    let (_, works) = mine_and_works(&mut game);
    game.apply(Command::SetProduction {
        site: works,
        slot: 0,
        recipe: smelting,
        utilization: 1.0,
    })
    .unwrap();
    let keys = until(&mut game, date(1901, 3, 1));
    let v = game.state().ventures[0].clone();
    assert_eq!(
        v.target,
        VentureTarget::Development {
            product: iron,
            level: 1
        }
    );
    // No lead for improvements: the capital of the reference.
    assert_eq!(v.lead, 0.0);
    let VentureStatus::Succeeded(when) = v.status else {
        panic!("{:?}", v.status);
    };
    assert_eq!(game.state().developments.get(iron).first(), Some(&when));
    // The player hears of it when it sells iron.
    assert_eq!(
        keys.iter().any(|k| k == keys::VENTURE_DEVELOPMENT),
        research::player_offers(game.state(), iron)
    );
    // The next start-up works on the level after it.
    let next = game
        .state()
        .ventures
        .iter()
        .find(|x| x.id == v.id + 1)
        .unwrap();
    assert_eq!(
        next.target,
        VentureTarget::Development {
            product: iron,
            level: 2
        }
    );
}

#[test]
fn the_world_can_overtake_a_start_up() {
    // Turbine is invented in history in 1902: a slow start-up from 1900 is too late.
    let mut m = model();
    m.phases[1].months = 30;
    let mut game = game_with(catalog_with(m), 3, 1.0);
    until(&mut game, date(1902, 1, 2));
    assert_eq!(
        venture(&game, 0).status,
        VentureStatus::Failed(date(1902, 1, 1), VentureFailure::Overtaken)
    );
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    assert_eq!(*game.state().inventions.get(turbine), None);
}

#[test]
fn a_historical_inventor_founds_once_in_his_country() {
    let mut m = model();
    m.investor_chance = 0.0;
    let mut c = catalog_with(m);
    let turbine = c.technologies.id("turbine").unwrap();
    let bbb = c.countries.id("BBB").unwrap();
    c.ventures.inventors.push(Inventor {
        technology: turbine,
        name: "Ada Muster".into(),
        country: bbb,
        provenance: Provenance::default(),
    });
    let mut game = game_with(c, 3, 1.0);
    until(&mut game, date(1902, 1, 1));
    let first = venture(&game, 0);
    assert_eq!(first.name, "Ada Muster");
    assert_eq!(first.country, bbb);
    assert!(first.inventor);
    // The others after the failure have founders from the name lists.
    let others: Vec<&Venture> = game.state().ventures.iter().filter(|v| v.id > 0).collect();
    assert!(!others.is_empty());
    assert!(others.iter().all(|v| !v.inventor && v.name != "Ada Muster"));
    // The inventor stays in the list after the years of the data.
    assert!(game.state().ventures.iter().any(|v| v.id == 0));
}

#[test]
fn the_strategy_department_sharpens_the_shown_chance() {
    let mut c = test_support::management();
    let mut m = model();
    m.phases[0].chance = 0.5;
    m.phases[1].chance = 0.8;
    m.investor_chance = 0.0;
    m.blur = 0.5;
    c.ventures = m;
    let mut game = crate::management_tests::new_game(c);
    mine_and_works(&mut game);
    until(&mut game, date(1900, 2, 2));
    let catalog = game.catalog().clone();
    let player = game.player();
    let v = game.state().ventures[0].clone();
    let chance = ventures::success_chance(&catalog.ventures, &v);
    assert!((chance - 0.4).abs() < 1e-12);
    // Without a department the draw blurs it by up to half.
    let (shown, works) = ventures::shown_chance(&catalog, game.state(), player, &v);
    assert!(!works);
    assert!((shown - 0.4 * (1.0 + v.blur * 0.5)).abs() < 1e-12);
    // A department with a perfect head sees it as it is.
    game.apply(Command::StaffDepartment {
        department: DepartmentKind::Strategy,
        staff: 1,
    })
    .unwrap();
    hire_sharp(&mut game, member("strategie"));
    let (shown, works) = ventures::shown_chance(&catalog, game.state(), player, &v);
    assert!(works);
    assert!((shown - 0.4).abs() < 1e-12, "{shown}");
}
