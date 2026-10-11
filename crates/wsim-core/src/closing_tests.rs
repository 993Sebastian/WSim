//! Tests of closing a site (docs/FORMELN.md, Standort schließen).

use crate::catalog::test_support;
use crate::command::{Command, CommandError};
use crate::facility_tests::{mine, new_game, works};
use crate::ledger::Account;
use crate::money::Money;
use crate::supply::SupplyMode;
use crate::trade_tests::{days, inventory_matches, iron, stock, warehouse};

#[test]
fn closing_sells_everything_pays_the_staff_and_leaves_an_empty_site() {
    let mut game = new_game(test_support::production());
    let me = game.player();
    let site = mine(&mut game);
    days(&mut game, 400);
    let s = &game.state().sites[site.index()];
    assert!(
        s.workforce.values().sum::<f64>() > 0.0,
        "the mine has staff"
    );
    let deposit = s.deposit.unwrap();

    let c = crate::closing::value(game.state(), game.catalog(), site);
    assert!(c.facilities > Money::ZERO && c.building > Money::ZERO);
    assert!(c.severance > Money::ZERO);
    let ledger = &game.state().company(me).unwrap().ledger;
    let (cash, fixed) = (ledger.cash(), ledger.balance(Account::FixedAssets));

    game.apply(Command::CloseSite { site }).unwrap();

    let ledger = &game.state().company(me).unwrap().ledger;
    assert_eq!(ledger.cash() - cash, c.proceeds() - c.severance);
    assert!(ledger.balance(Account::FixedAssets) < fixed);
    assert!(ledger.is_balanced());
    assert!(inventory_matches(&game, me));
    let s = &game.state().sites[site.index()];
    assert!(s.closed.is_some() && s.slots.is_empty() && s.inventory.is_empty());
    assert!(s.workforce.values().sum::<f64>() < 1e-9);
    assert!(s.deposit.is_none());
    assert!(
        game.state()
            .deposits
            .get(deposit)
            .concessions
            .iter()
            .all(|f| f.site != Some(site)),
        "the concession goes back"
    );
    // The site is gone for commands; running on costs nothing more.
    assert_eq!(
        game.apply(Command::CloseSite { site }),
        Err(CommandError::UnknownSite)
    );
    let cash = game.state().company(me).unwrap().ledger.cash();
    let before = game
        .state()
        .company(me)
        .unwrap()
        .ledger
        .month
        .by_site
        .get(&site)
        .copied();
    days(&mut game, 3);
    let after = game
        .state()
        .company(me)
        .unwrap()
        .ledger
        .month
        .by_site
        .get(&site)
        .copied();
    assert_eq!(before, after, "no costs at the closed site");
    assert!(game.state().company(me).unwrap().ledger.cash() <= cash);
}

#[test]
fn a_site_under_construction_cannot_be_closed() {
    let mut game = new_game(test_support::production());
    let site = works(&mut game, 1);
    assert_eq!(
        game.apply(Command::CloseSite { site }),
        Err(CommandError::UnderConstruction)
    );
    days(&mut game, 25);
    game.apply(Command::CloseSite { site }).unwrap();
}

#[test]
fn stocks_are_sold_off_and_supplies_from_the_site_end() {
    let mut game = crate::trade_tests::new_game(test_support::trading());
    let me = game.player();
    let plant = warehouse(&mut game, me, "AAA", 1000.0);
    let branch = warehouse(&mut game, me, "AAA", 0.0);
    let product = iron(&game);
    game.apply(Command::SetInternalSupply {
        site: branch,
        product,
        source: Some(plant),
        mode: SupplyMode::Fixed { per_day: 10.0 },
    })
    .unwrap();
    days(&mut game, 1);
    let value = game.state().sites[plant.index()].inventory[&product].value;
    let c = crate::closing::value(game.state(), game.catalog(), plant);
    let share = game.catalog().production_model.close_stock_share;
    assert_eq!(c.stock, value.scale(share));

    game.apply(Command::CloseSite { site: plant }).unwrap();
    assert!(game.state().sites[branch.index()].supplies.is_empty());
    let held = stock(&game, branch);
    days(&mut game, 2);
    assert!((stock(&game, branch) - held).abs() < 1e-9);
    assert!(game.state().company(me).unwrap().ledger.is_balanced());
    assert!(inventory_matches(&game, me));
}
