//! Tests of moving facilities between sites.

use crate::command::{Command, CommandError};
use crate::facility_tests::{mine, new_game, works};
use crate::trade_tests::days;

#[test]
fn a_facility_moves_to_another_own_site_for_free() {
    let mut game = new_game(crate::catalog::test_support::production());
    let me = game.player();
    let from = works(&mut game, 3);
    let to = works(&mut game, 1);
    days(&mut game, 120);
    let cash = game.state().company(me).unwrap().ledger.cash();
    let cost = game.state().sites[from.index()].slots[0].cost;
    let ready = game.state().sites[from.index()].slots[0].ready;
    // Not into an extraction site, not more units than there are.
    let pit = mine(&mut game);
    assert!(matches!(
        game.apply(Command::MoveFacility {
            site: from,
            slot: 0,
            count: 1,
            to: pit,
            price: None,
        }),
        Err(CommandError::WrongSiteType { .. })
    ));
    assert!(matches!(
        game.apply(Command::MoveFacility {
            site: from,
            slot: 0,
            count: 4,
            to,
            price: None,
        }),
        Err(CommandError::TooManyUnits { .. })
    ));
    let cash = game.state().company(me).unwrap().ledger.cash().min(cash);
    game.apply(Command::MoveFacility {
        site: from,
        slot: 0,
        count: 2,
        to,
        price: None,
    })
    .unwrap();
    let s = &game.state().sites[from.index()];
    assert_eq!(s.slots[0].count, 1);
    let t = &game.state().sites[to.index()];
    let moved = t.slots.last().unwrap();
    assert_eq!(moved.count, 2);
    assert_eq!(moved.ready, ready, "it keeps its age");
    assert_eq!(moved.cost + s.slots[0].cost, cost, "and its value");
    assert!(moved.down.is_some(), "it stands while it is moved");
    assert!(game.state().company(me).unwrap().ledger.cash() >= cash);
    assert!(game.state().company(me).unwrap().ledger.is_balanced());
}
