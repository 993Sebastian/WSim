//! Tests of the sites' storage (docs/FORMELN.md, Lager: Kapazität, Kosten, Verderb).

use crate::catalog::{Catalog, SiteType, StorageModel, test_support};
use crate::command::{Command, CommandError};
use crate::facility_tests::{mine, new_game};
use crate::ledger::CostType;
use crate::money::Money;
use crate::state::Limit;
use crate::trade_tests::{days, inventory_matches};

fn with_storage(spoilage: f64) -> Catalog {
    let mut catalog = test_support::production();
    let groups: Vec<_> = (0..catalog.goods_groups.len())
        .map(|i| (crate::ids::Id::from_index(i), spoilage))
        .collect();
    catalog.production_model.storage = StorageModel {
        enabled: true,
        minimum: vec![(SiteType::Extraction, Money::from_usd(1.0).unwrap())],
        days: 5.0,
        step_capacity: Money::from_usd(1_000_000.0).unwrap(),
        step_cost: Money::from_usd(10_000.0).unwrap(),
        cost_share_year: 0.1,
        spoilage: if spoilage > 0.0 { groups } else { Vec::new() },
        ai_expand_at: 0.9,
    };
    catalog
}

#[test]
fn a_full_store_stops_the_site_until_it_is_expanded() {
    let mut game = new_game(with_storage(0.0));
    let me = game.player();
    let site = mine(&mut game);
    days(&mut game, 400);
    let s = &game.state().sites[site.index()];
    let catalog = game.catalog();
    assert!(
        crate::storage::is_full(catalog, s),
        "the mine's store fills up"
    );
    assert!(s.storage_full);
    assert_eq!(s.slots[0].limit, Some(Limit::Storage));
    assert_eq!(s.slots[0].last_runs, 0.0);
    let ledger = &game.state().company(me).unwrap().ledger;
    assert!(
        ledger
            .year
            .by_type
            .get(&CostType::Storage)
            .copied()
            .unwrap_or_default()
            < Money::ZERO
    );
    assert!(ledger.is_balanced());

    let cash = ledger.cash();
    let building = s.building_cost;
    game.apply(Command::BuildStorage { site, steps: 2 })
        .unwrap();
    let s = &game.state().sites[site.index()];
    assert_eq!(s.storage_steps, 2);
    assert_eq!(
        s.building_cost - building,
        Money::from_usd(20_000.0).unwrap()
    );
    let ledger = &game.state().company(me).unwrap().ledger;
    assert_eq!(cash - ledger.cash(), Money::from_usd(20_000.0).unwrap());
    assert!(ledger.is_balanced());
    days(&mut game, 1);
    let s = &game.state().sites[site.index()];
    assert!(!s.storage_full);
    assert!(s.slots[0].last_runs > 0.0, "the mine runs again");
    assert_eq!(
        game.apply(Command::BuildStorage { site, steps: 0 }),
        Err(CommandError::NotPossible)
    );
}

#[test]
fn perishable_goods_spoil_at_the_end_of_the_month() {
    let mut game = new_game(with_storage(0.5));
    let me = game.player();
    let site = mine(&mut game);
    game.apply(Command::BuildStorage { site, steps: 50 })
        .unwrap();
    days(&mut game, 400);
    let ledger = &game.state().company(me).unwrap().ledger;
    let lost = ledger
        .year
        .by_type
        .get(&CostType::Storage)
        .copied()
        .unwrap_or_default();
    assert!(lost < Money::ZERO, "spoiled goods are a cost");
    assert!(ledger.is_balanced());
    assert!(inventory_matches(&game, me));
}

#[test]
fn without_data_stocks_are_unlimited() {
    let mut game = new_game(test_support::production());
    let site = mine(&mut game);
    days(&mut game, 400);
    let s = &game.state().sites[site.index()];
    assert!(crate::storage::capacity(game.catalog(), s).is_none());
    assert!(!s.storage_full);
    assert_eq!(
        game.apply(Command::BuildStorage { site, steps: 1 }),
        Err(CommandError::NotPossible)
    );
}
