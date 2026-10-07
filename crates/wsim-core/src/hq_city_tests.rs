//! Scenario tests of the headquarters in a city (W2, docs/FORMELN.md).

use crate::calendar::RoundLength;
use crate::catalog::{Catalog, City, DepartmentKind, GeoPoint, HqCityModel, test_support};
use crate::central;
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::management_tests::{new_game, usd};
use crate::message::keys;
use crate::state::CompanyId;

fn city(key: &str, population: f64, capital: bool) -> City {
    City {
        key: key.into(),
        population,
        location: GeoPoint {
            lat: 50.0,
            lon: 10.0,
        },
        capital,
    }
}

/// The management catalog with two cities in AAA: a large capital and a small town.
fn with_cities(hq_share: f64) -> Catalog {
    let mut c = test_support::management();
    let aaa = c.countries.id("AAA").unwrap();
    c.countries.get_mut(aaa).cities = vec![
        city("hauptstadt", 2_000_000.0, true),
        city("kleinstadt", 20_000.0, false),
    ];
    c.central.city = Some(HqCityModel {
        academics_concentration: 4.0,
        hq_share,
        office_reference_population: 1_000_000.0,
        office_elasticity: 0.15,
        move_within_country: 0.5,
        population_year: 1900,
    });
    c.central.headquarters.months = 4;
    c.central.headquarters.cost_base = usd(100_000.0);
    c.central.headquarters.moving_share = 0.6;
    c
}

fn staff(game: &mut Game, n: u32) {
    game.apply(Command::StaffDepartment {
        department: DepartmentKind::Finance,
        staff: n,
    })
    .unwrap();
}

fn academics(game: &Game, key: &str) -> f64 {
    let c = game.catalog();
    let aaa = c.countries.id("AAA").unwrap();
    let group = c
        .central
        .department(DepartmentKind::Finance)
        .unwrap()
        .labor_group;
    central::city_academics(
        c,
        game.state(),
        aaa,
        central::city_in(c, aaa, Some(key)),
        group,
    )
}

#[test]
fn the_city_limits_the_staff_and_sets_the_rent() {
    let mut game = new_game(with_cities(0.05));
    let player = game.player();
    let catalog = game.catalog().clone();
    let aaa = catalog.countries.id("AAA").unwrap();
    // The capital is the default seat.
    assert_eq!(
        central::hq_city(&catalog, game.state(), player).map(|c| c.key.as_str()),
        Some("hauptstadt")
    );
    let big = academics(&game, "hauptstadt");
    let small = academics(&game, "kleinstadt");
    assert!(big > small && small > 0.0, "{big} {small}");
    // Many more posts than the small town has academics, fewer than the capital.
    let want = (small * 4.0).ceil() as u32 + 2;
    assert!(f64::from(want) < big);
    staff(&mut game, want);
    assert_eq!(
        central::staffed(game.state(), player, DepartmentKind::Finance),
        want
    );

    // Rent: the large city costs more per employee.
    let office = catalog
        .central
        .department(DepartmentKind::Finance)
        .unwrap()
        .office;
    let state = game.state();
    let rent = |key| {
        central::office_per_employee(
            &catalog,
            state,
            aaa,
            central::city_in(&catalog, aaa, Some(key)),
            office,
        )
    };
    assert!(rent("hauptstadt") > rent("kleinstadt"));

    // A move within the country: half the cost, half the time.
    assert_eq!(
        game.apply(Command::SetHeadquarters {
            country: aaa,
            city: Some("hauptstadt".into()),
        }),
        Err(CommandError::SameHeadquarters)
    );
    assert_eq!(
        game.apply(Command::SetHeadquarters {
            country: aaa,
            city: Some("atlantis".into()),
        }),
        Err(CommandError::UnknownCity)
    );
    let cash = game.state().companies[0].ledger.cash();
    let (cost, months) = central::move_terms(&catalog, game.state(), player, aaa);
    assert_eq!(months, 2);
    assert_eq!(
        cost,
        central::relocation_cost(&catalog, game.state(), player).scale(0.5)
    );
    game.apply(Command::SetHeadquarters {
        country: aaa,
        city: Some("kleinstadt".into()),
    })
    .unwrap();
    assert_eq!(game.state().companies[0].ledger.cash(), cash - cost);
    let mut seen = Vec::new();
    while game.state().companies[0].relocation.is_some() {
        let report = game.advance(RoundLength::Day, |_| {});
        seen.extend(report.messages.into_iter().map(|m| m.key));
    }
    assert!(seen.iter().any(|k| k == keys::HEADQUARTERS_MOVED_CITY));
    let state = game.state();
    assert_eq!(state.companies[0].hq_city.as_deref(), Some("kleinstadt"));
    // Part of the staff stayed behind; of the rest the town fills only what it has.
    let wanted = central::staff(state, player, DepartmentKind::Finance);
    let filled = central::staffed(state, player, DepartmentKind::Finance);
    assert!(filled < wanted, "{filled} of {wanted}, town {small}");
    assert!(f64::from(filled) <= academics(&game, "kleinstadt") + 1e-9);
    // Costs follow the filled posts.
    let (personnel, _) = central::monthly_cost(&catalog, game.state(), player);
    let d = catalog.central.department(DepartmentKind::Finance).unwrap();
    let wage = crate::management::group_yearly_wage(&catalog, game.state(), aaa, d.labor_group);
    assert!((personnel.to_usd() - f64::from(filled) * wage / 12.0).abs() < 0.01);
}

#[test]
fn companies_in_one_city_share_its_academics() {
    let mut game = new_game(with_cities(0.05));
    let catalog = game.catalog().clone();
    let pool = academics(&game, "kleinstadt");
    // A second company with its seat in the same town.
    let other = {
        let state = game.state_mut();
        let mut c = state.companies[0].clone();
        c.name = "Nachbar AG".into();
        c.hq_city = Some("kleinstadt".into());
        state.companies.push(c);
        CompanyId(1)
    };
    game.state_mut().companies[0].hq_city = Some("kleinstadt".into());
    let three = (pool * 1.5).ceil() as u32;
    let one = (pool * 0.5).ceil() as u32;
    staff(&mut game, three);
    game.apply_as(
        other,
        Command::StaffDepartment {
            department: DepartmentKind::Finance,
            staff: one,
        },
    )
    .unwrap();
    let state = game.state();
    let mine = central::staffed(state, CompanyId(0), DepartmentKind::Finance);
    let theirs = central::staffed(state, other, DepartmentKind::Finance);
    let all = f64::from(three + one);
    assert_eq!(mine, (f64::from(three) * pool / all).floor() as u32);
    assert_eq!(theirs, (f64::from(one) * pool / all).floor() as u32);
    assert!(f64::from(mine + theirs) <= pool + 1e-9);
    // In the capital the same posts are all filled.
    let aaa = catalog.countries.id("AAA").unwrap();
    assert_eq!(
        central::city_wanted(&catalog, state, aaa, "kleinstadt"),
        three + one
    );
}

#[test]
fn ai_companies_move_to_the_city_with_more_academics() {
    let mut game = new_game(with_cities(0.05));
    let catalog = game.catalog().clone();
    let pool = academics(&game, "kleinstadt");
    game.state_mut().companies[0].hq_city = Some("kleinstadt".into());
    staff(&mut game, (pool * 2.0).ceil() as u32);
    let player = game.player();
    assert_eq!(
        central::ai_city(&catalog, game.state(), player),
        Some("hauptstadt".to_owned())
    );
    // Fully staffed: no reason to move.
    staff(&mut game, (pool * 0.5).floor().max(1.0) as u32);
    assert_eq!(central::ai_city(&catalog, game.state(), player), None);
}
