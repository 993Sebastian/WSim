//! Tests of maintenance, breakdowns and overhauls (docs/FORMELN.md „BP2“).

use crate::catalog::{Catalog, MaintenanceModel, test_support};
use crate::command::{Command, CommandError};
use crate::facility_tests::{mine, new_game, works};
use crate::money::Money;
use crate::state::Limit;
use crate::trade_tests::days;

fn with_maintenance() -> Catalog {
    let mut catalog = test_support::production();
    catalog.production_model.maintenance = MaintenanceModel {
        enabled: true,
        level_min: 0.5,
        level_max: 1.5,
        exponent: 1.5,
        capacity_weight: 0.3,
        failure_chance: 0.02,
        failure_days: (3, 14),
        failure_cost_share: 0.02,
        failures_warning: 3,
        overhaul_cost_share: 0.3,
        overhaul_days: 30,
        ai_overhaul_below: 0.35,
    };
    catalog
}

#[test]
fn little_maintenance_wears_and_breaks_down_more_often() {
    let mut game = new_game(with_maintenance());
    let sites = [works(&mut game, 1), works(&mut game, 1)];
    game.apply(Command::SetMaintenance {
        site: sites[0],
        level: 0.5,
    })
    .unwrap();
    game.apply(Command::SetMaintenance {
        site: sites[1],
        level: 1.5,
    })
    .unwrap();
    let mut breakdowns = [0u32; 2];
    let mut last = [None; 2];
    for _ in 0..(10 * 365) {
        days(&mut game, 1);
        for (i, &site) in sites.iter().enumerate() {
            let down = game.state().sites[site.index()].slots[0].down;
            if down.is_some() && down != last[i] {
                breakdowns[i] += 1;
            }
            last[i] = down;
        }
    }
    let condition = |i: usize| game.state().sites[sites[i].index()].slots[0].condition;
    assert!(condition(0) < condition(1), "less maintenance wears faster");
    assert!(
        breakdowns[0] > breakdowns[1],
        "breakdowns at 0.5: {}, at 1.5: {}",
        breakdowns[0],
        breakdowns[1]
    );
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    assert!(ledger.is_balanced());
}

#[test]
fn an_overhaul_stands_a_month_and_restores_the_condition() {
    let mut game = new_game(with_maintenance());
    let me = game.player();
    let site = mine(&mut game);
    days(&mut game, 40);
    game.state_mut().sites[site.index()].slots[0].condition = 0.3;
    let worn = game.state().sites[site.index()].slots[0].full_runs(game.catalog());
    let cash = game.state().company(me).unwrap().ledger.cash();
    let cost = crate::maintenance::overhaul_cost(
        game.catalog(),
        &game.state().sites[site.index()].slots[0],
    );
    assert!(cost > Money::ZERO);
    game.apply(Command::Overhaul { site, slot: 0 }).unwrap();
    let s = &game.state().sites[site.index()];
    assert_eq!(s.slots[0].condition, 1.0);
    assert!(s.slots[0].full_runs(game.catalog()) > worn * 1.2);
    assert_eq!(cash - game.state().company(me).unwrap().ledger.cash(), cost);
    assert_eq!(
        game.apply(Command::Overhaul { site, slot: 0 }),
        Err(CommandError::NotPossible),
        "not while it stands"
    );
    days(&mut game, 2);
    let s = &game.state().sites[site.index()];
    assert_eq!(s.slots[0].limit, Some(Limit::Overhaul));
    assert_eq!(s.slots[0].last_runs, 0.0);
    days(&mut game, 30);
    assert!(game.state().sites[site.index()].slots[0].last_runs > 0.0);
    assert!(game.state().company(me).unwrap().ledger.is_balanced());
}

#[test]
fn the_level_stays_in_range_and_needs_the_model() {
    let mut game = new_game(with_maintenance());
    let site = mine(&mut game);
    assert_eq!(
        game.apply(Command::SetMaintenance { site, level: 2.0 }),
        Err(CommandError::MaintenanceOutOfRange { min: 0.5, max: 1.5 })
    );
    let mut off = new_game(test_support::production());
    let site = mine(&mut off);
    assert_eq!(
        off.apply(Command::SetMaintenance { site, level: 1.0 }),
        Err(CommandError::NotPossible)
    );
    assert_eq!(
        off.apply(Command::Overhaul { site, slot: 0 }),
        Err(CommandError::NotPossible)
    );
}
