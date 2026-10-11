//! Tests of the sites' storage (docs/FORMELN.md, Lager: Kapazität, Kosten, Verderb).

use crate::catalog::{Catalog, SiteType, StorageClass, StorageModel, test_support};
use crate::command::{Command, CommandError};
use crate::facility_tests::{mine, new_game, works};
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
        minimum: vec![(SiteType::Extraction, 1.0)],
        days: 5.0,
        step_capacity: 1_000_000.0,
        step_cost: Money::from_usd(10_000.0).unwrap(),
        cost_share_year: 0.1,
        spoilage: if spoilage > 0.0 { groups } else { Vec::new() },
        ai_expand_at: 0.9,
        ..StorageModel::default()
    };
    // One m³ per unit: the room counts units.
    catalog.production_model.storage.volume = vec![1.0; catalog.products.len()];
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
    assert_eq!(s.building_cost, building, "no lasting asset");
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

#[test]
fn a_store_full_of_inputs_keeps_the_facilities_running() {
    let mut game = new_game(with_storage(0.0));
    let site = works(&mut game, 1);
    let c = game.catalog().clone();
    game.apply(Command::SetProduction {
        site,
        slot: 0,
        recipe: c.recipes.id("eisen_schmelzen"),
        utilization: 1.0,
    })
    .unwrap();
    days(&mut game, 25);
    // Far more ore than the store holds, nothing made yet.
    let ore = c.products.id("erz").unwrap();
    game.state_mut().sites[site.index()]
        .inventory
        .entry(ore)
        .or_default()
        .add(1_000_000.0, Money::from_usd(1_000_000.0).unwrap(), 50.0);
    let s = &game.state().sites[site.index()];
    assert!(crate::storage::is_full(game.catalog(), s));
    assert!(!crate::storage::stops_output(game.catalog(), s));
    days(&mut game, 2);
    let s = &game.state().sites[site.index()];
    assert!(s.storage_full, "orders do not buy");
    assert!(
        s.slots[0].last_runs > 0.0,
        "the furnace turns ore into iron"
    );
    assert_ne!(s.slots[0].limit, Some(Limit::Storage));
}

/// Storage classes, outside storage and shelf life (BP3).
fn with_classes(shelf_life: Option<u32>) -> Catalog {
    let mut catalog = with_storage(0.0);
    let n = catalog.products.len();
    let m = &mut catalog.production_model.storage;
    m.classes = vec![
        (StorageClass::Bulk, 0.02),
        (StorageClass::Piece, 0.04),
        (StorageClass::Liquid, 0.03),
        (StorageClass::Chilled, 0.12),
        (StorageClass::Hazardous, 0.06),
    ];
    m.product_class = vec![StorageClass::Bulk; n];
    m.shelf_life = vec![shelf_life; n];
    m.external_capacity = 2.0;
    m.external_rate = 2.0;
    m.spoil_message_share = 0.05;
    catalog
}

#[test]
fn goods_without_buyers_spoil_after_their_shelf_life() {
    let mut game = new_game(with_classes(Some(30)));
    let me = game.player();
    let site = mine(&mut game);
    game.apply(Command::BuildStorage { site, steps: 50 })
        .unwrap();
    days(&mut game, 200);
    let ore = game.catalog().products.id("erz").unwrap();
    let made = |game: &crate::game::Game| {
        game.state().sites[site.index()]
            .inventory
            .get(&ore)
            .map_or(0.0, |s| s.quantity)
    };
    assert!(made(&game) > 0.0);
    days(&mut game, 60);
    let stock = &game.state().sites[site.index()].inventory[&ore];
    let oldest = stock.lots.first().unwrap().0;
    assert!(
        oldest.days_until(game.state().date) < 31,
        "nothing older than the shelf life stays"
    );
    let total: f64 = stock.lots.iter().map(|&(_, q)| q).sum();
    assert!(
        total <= stock.quantity + 1e-6,
        "today's output gets its lot tomorrow"
    );
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
    assert!(inventory_matches(&game, me));
}

#[test]
fn takings_leave_the_oldest_lots_first() {
    let d = |day| crate::calendar::Date::new(1900, 1, day).unwrap();
    let mut stock = crate::state::Stock::default();
    stock.add(10.0, Money::from_usd(10.0).unwrap(), 50.0);
    crate::storage::settle_lots(&mut stock, d(1));
    stock.add(5.0, Money::from_usd(5.0).unwrap(), 50.0);
    crate::storage::settle_lots(&mut stock, d(2));
    assert_eq!(stock.lots, vec![(d(1), 10.0), (d(2), 5.0)]);
    stock.take(12.0);
    crate::storage::settle_lots(&mut stock, d(3));
    assert_eq!(stock.lots, vec![(d(2), 3.0)]);
}

#[test]
fn outside_storage_costs_double_and_does_not_stop_the_site() {
    let catalog = with_classes(None);
    let mut game = new_game(catalog);
    let site = mine(&mut game);
    days(&mut game, 2);
    let c = game.catalog().clone();
    let ore = c.products.id("erz").unwrap();
    let cap = crate::storage::capacity(&c, &game.state().sites[site.index()]).unwrap();
    // Fill to one and a half times the own capacity (one m³ per unit).
    let units = cap * 1.5;
    let s = &mut game.state_mut().sites[site.index()];
    s.inventory = Default::default();
    s.inventory
        .entry(ore)
        .or_default()
        .add(units, Money::from_usd(1.0).unwrap(), 50.0);
    let s = &game.state().sites[site.index()];
    assert!(crate::storage::external(&c, s));
    assert!(
        !crate::storage::is_full(&c, s),
        "outside storage takes the rest"
    );
    let cost = crate::storage::daily_cost(&c, s).to_usd();
    // The cost follows the value of the stock.
    let own = units * c.products.get(ore).reference_price.to_usd() * 0.02 / 365.0;
    assert!(
        (cost - own * (1.0 + 1.0 / 3.0)).abs() < 1e-2 * own,
        "a third lies outside at double rate: {cost} vs {own}"
    );
}

#[test]
fn perishable_goods_are_stocked_for_half_their_shelf_life() {
    let catalog = with_classes(Some(14));
    let ore = catalog.products.id("erz").unwrap();
    assert!((crate::storage::stock_days(&catalog, ore, 30.0) - 7.0).abs() < 1e-9);
    assert!((crate::storage::stock_days(&catalog, ore, 5.0) - 5.0).abs() < 1e-9);
    let lasting = with_classes(None);
    assert!((crate::storage::stock_days(&lasting, ore, 30.0) - 30.0).abs() < 1e-9);
}

#[test]
fn a_stock_position_is_scrapped_with_its_book_value() {
    let mut game = new_game(with_storage(0.0));
    let me = game.player();
    let site = mine(&mut game);
    days(&mut game, 60);
    let ore = game.catalog().products.id("erz").unwrap();
    let value = game.state().sites[site.index()].inventory[&ore].value;
    assert!(value > Money::ZERO);
    game.apply(Command::ScrapStock { site, product: ore })
        .unwrap();
    assert_eq!(
        game.state().sites[site.index()].inventory[&ore].quantity,
        0.0
    );
    assert!(game.state().company(me).unwrap().ledger.is_balanced());
    assert!(inventory_matches(&game, me));
    assert_eq!(
        game.apply(Command::ScrapStock { site, product: ore }),
        Err(CommandError::InvalidQuantity)
    );
}
