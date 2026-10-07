//! Scenario tests for the headquarters (ZA1) with `test_support::management`.

use crate::calendar::{Date, RoundLength};
use crate::catalog::test_support;
use crate::command::{Command, CommandError};
use crate::ledger::CostType;
use crate::management;
use crate::management_tests::{days, mine_and_works, new_game, usd};
use crate::message::keys;
use crate::money::Money;
use crate::state::{CompanyId, Unit};

#[test]
fn moving_the_headquarters_costs_and_takes_months() {
    let mut c = test_support::management();
    c.central.headquarters.months = 2;
    c.central.headquarters.cost_base = usd(250_000.0);
    let mut game = new_game(c);
    mine_and_works(&mut game);
    let catalog = game.catalog().clone();
    let aaa = catalog.countries.id("AAA").unwrap();
    let bbb = catalog.countries.id("BBB").unwrap();
    let player = CompanyId(0);
    assert_eq!(game.state().companies[0].headquarters, aaa);
    assert_eq!(
        game.apply(Command::SetHeadquarters { country: aaa }),
        Err(CommandError::SameHeadquarters)
    );
    let cash = game.state().companies[0].ledger.cash();
    game.apply(Command::SetHeadquarters { country: bbb })
        .unwrap();
    let state = game.state();
    assert_eq!(state.companies[0].ledger.cash(), cash - usd(250_000.0));
    let other = state.companies[0]
        .ledger
        .month
        .by_type
        .get(&CostType::Other)
        .copied()
        .unwrap_or(Money::ZERO);
    assert!(other <= -usd(250_000.0), "booked as other costs");
    assert!(state.companies[0].ledger.is_balanced());
    let until = state.companies[0].relocation.unwrap().until;
    assert_eq!(until, Date::new(1900, 3, 1).unwrap());
    // Only one move at a time.
    assert_eq!(
        game.apply(Command::SetHeadquarters { country: aaa }),
        Err(CommandError::RelocationUnderWay { until })
    );
    // Still the old seat until the move is done.
    let mut keys_seen = Vec::new();
    while game.state().date < until {
        assert_eq!(game.state().companies[0].headquarters, aaa);
        let report = game.advance(RoundLength::Day, |_| {});
        keys_seen.extend(report.messages.into_iter().map(|m| m.key));
    }
    let state = game.state();
    assert_eq!(state.companies[0].headquarters, bbb);
    assert!(state.companies[0].relocation.is_none());
    assert!(keys_seen.iter().any(|k| k == keys::HEADQUARTERS_MOVED));
    // The board's salaries follow the new seat.
    assert_eq!(
        management::seat_country(&catalog, state, player, Unit::Board),
        Some(bbb)
    );
    days(&mut game, 1);
}

#[test]
fn a_move_needs_the_cash() {
    let mut c = test_support::management();
    c.central.headquarters.cost_base = usd(1e12);
    let mut game = new_game(c);
    let bbb = game.catalog().countries.id("BBB").unwrap();
    assert!(matches!(
        game.apply(Command::SetHeadquarters { country: bbb }),
        Err(CommandError::NotEnoughCash { .. })
    ));
    assert!(game.state().companies[0].relocation.is_none());
}
