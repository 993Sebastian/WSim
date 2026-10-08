//! Tests of logistics: fleets, state transport, freight risk (W5).

use crate::calendar::RoundLength;
use crate::catalog::{Catalog, FleetVehicle, LogisticsModel, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::ids::VehicleId;
use crate::ledger::{Account, CostType};
use crate::logistics::{self, FreightMode, Load};
use crate::message::keys;
use crate::money::Money;
use crate::save;
use crate::time_series::TimeSeries;
use crate::trade_tests::{country, days, inventory_matches, iron, new_game, stock, warehouse};

fn series(v: f64) -> Option<TimeSeries> {
    Some(TimeSeries::new(vec![(1900, v)]).expect("valid"))
}

fn fleet_vehicle(c: &mut Catalog, key: &str, payload: f64) {
    let id = c.vehicles.id(key).expect("exists");
    c.vehicles.get_mut(id).fleet = Some(FleetVehicle {
        payload_t: series(payload).unwrap(),
        price_usd: series(100_000.0).unwrap(),
    });
}

/// Share of the market freight a run of the vehicle costs for iron.
fn running(game: &Game, vehicle: VehicleId) -> f64 {
    let c = game.catalog();
    let class = c.products.get(iron(game)).transport_class;
    logistics::running_share(c, vehicle, class, game.state().date.year())
}

/// The trading catalog with logistics; a train carries 300 t, a steamer 4000 t, each for
/// 100 000 USD.
fn with_logistics(risk: f64) -> Catalog {
    let mut c = test_support::trading();
    for (key, payload) in [("bahn", 300.0), ("dampfer", 4000.0)] {
        fleet_vehicle(&mut c, key, payload);
    }
    c.logistics = LogisticsModel {
        enabled: true,
        state_surcharge: 0.5,
        state_risk_factor: 0.5,
        market_margin: 0.15,
        load: 0.6,
        upkeep_share: 0.06,
        life_years: 20.0,
        sale_share: 0.6,
        rental_share: 0.5,
        rental_market_share: 1.0,
        risk_land: series(risk),
        risk_sea: series(risk),
        ai_share: 0.5,
        ai_cash_share: 1.0,
        provenance: Default::default(),
    };
    c
}

/// The vehicle that carries iron from AAA to BBB: steamer if the route goes by sea.
fn carrier(game: &Game) -> VehicleId {
    let c = game.catalog();
    let class = c.products.get(iron(game)).transport_class;
    let route = game
        .state()
        .routes
        .get(class, country(game, "AAA"), country(game, "BBB"))
        .unwrap();
    let key = if route.by_sea { "dampfer" } else { "bahn" };
    c.vehicles.id(key).unwrap()
}

fn market_freight(game: &Game, quantity: f64) -> Money {
    let (per_unit, _) = game
        .state()
        .routes
        .for_product(
            game.catalog(),
            iron(game),
            country(game, "AAA"),
            country(game, "BBB"),
        )
        .unwrap();
    Money::times(per_unit, quantity)
}

fn transport(game: &Game) -> Money {
    let ledger = &game.state().companies[game.player().index()].ledger;
    -ledger
        .month
        .by_type
        .get(&CostType::Transport)
        .copied()
        .unwrap_or_default()
}

fn transfer(game: &mut Game, quantity: f64) -> Result<(), CommandError> {
    let player = game.player();
    let home = warehouse(game, player, "AAA", quantity);
    let abroad = warehouse(game, player, "BBB", 0.0);
    let product = iron(game);
    game.apply(Command::TransferGoods {
        from: home,
        to: abroad,
        product,
        quantity,
    })
}

fn set_mode(game: &mut Game, mode: FreightMode, carry_for_others: bool) {
    game.apply(Command::SetLogistics {
        mode,
        carry_for_others,
    })
    .unwrap();
}

#[test]
fn a_fleet_carries_cheaper_and_costs_upkeep() {
    let mut game = new_game(with_logistics(0.0));
    let player = game.player();
    let vehicle = carrier(&game);
    let year = game.state().date.year();
    let price = logistics::price(game.catalog(), vehicle, year).unwrap();
    let cash = game.state().companies[player.index()].ledger.cash();
    game.apply(Command::BuyVehicles { vehicle, count: 1 })
        .unwrap();
    let ledger = &game.state().companies[player.index()].ledger;
    assert_eq!(ledger.cash(), cash - price);
    assert_eq!(ledger.balance(Account::FixedAssets), price);
    set_mode(&mut game, FreightMode::Fleet, false);

    transfer(&mut game, 100.0).unwrap();
    let market = market_freight(&game, 100.0);
    let share = running(&game, vehicle);
    assert!(share > 0.0 && share < 0.85, "{share}");
    assert_eq!(transport(&game), market.scale(share));
    let l = &game.state().companies[player.index()].logistics;
    assert!(l.fleet[0].used > 0.0);
    assert!(l.month.market_tkm.abs() < 1e-9);

    // Month end: upkeep and depreciation of one month.
    days(&mut game, 31);
    let c = &game.state().companies[player.index()];
    let last = &c.logistics.last_month;
    assert_eq!(last.upkeep, price.scale(0.06 / 12.0));
    assert_eq!(last.depreciation, price.scale(1.0 / 240.0));
    assert_eq!(c.logistics.fleet[0].value, price - last.depreciation);
    assert!(c.logistics.fleet[0].used.abs() < 1e-9);
    assert!(c.logistics.fleet[0].used_last > 0.0);
    assert!(c.ledger.is_balanced());
    assert!(inventory_matches(&game, player));
}

#[test]
fn a_full_fleet_leaves_the_rest_to_the_market() {
    // Vehicles of one tonne fill quickly.
    let mut catalog = with_logistics(0.0);
    for key in ["bahn", "dampfer"] {
        fleet_vehicle(&mut catalog, key, 1.0);
    }
    let mut game = new_game(catalog);
    let player = game.player();
    let vehicle = carrier(&game);
    game.apply(Command::BuyVehicles { vehicle, count: 1 })
        .unwrap();
    set_mode(&mut game, FreightMode::Fleet, false);
    let date = game.state().date;
    let capacity = logistics::capacity_per_vehicle(game.catalog(), vehicle, date);
    // Far more than one vehicle carries in a month.
    let km = 1.3
        * game
            .state()
            .routes
            .distance_km(country(&game, "AAA"), country(&game, "BBB"))
            .unwrap();
    let quantity = 4.0 * capacity / km;
    transfer(&mut game, quantity).unwrap();
    let market = market_freight(&game, quantity);
    let paid = transport(&game);
    let share = running(&game, vehicle);
    assert!(
        paid > market.scale(share) && paid < market,
        "{paid:?} {market:?}"
    );
    let l = &game.state().companies[player.index()].logistics;
    assert!((l.fleet[0].used - capacity).abs() < 1e-6 * capacity);
    assert!(l.month.market_tkm > 0.0);
}

#[test]
fn state_transport_costs_more() {
    let mut game = new_game(with_logistics(0.0));
    set_mode(&mut game, FreightMode::State, false);
    transfer(&mut game, 100.0).unwrap();
    assert_eq!(transport(&game), market_freight(&game, 100.0).scale(1.5));
    let l = &game.state().companies[game.player().index()].logistics;
    assert!(l.month.state_tkm > 0.0);
}

#[test]
fn lost_loads_are_written_off_on_arrival() {
    let mut game = new_game(with_logistics(1.0));
    let player = game.player();
    transfer(&mut game, 100.0).unwrap();
    assert!(game.state().shipments[0].lost);
    let mut warned = false;
    for _ in 0..30 {
        let report = game.advance(RoundLength::Day, |_| {});
        warned |= report.messages.iter().any(|m| m.key == keys::FREIGHT_LOST);
    }
    assert!(warned);
    assert!(game.state().shipments.is_empty());
    let abroad = crate::state::SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    assert_eq!(stock(&game, abroad), 0.0);
    let c = &game.state().companies[player.index()];
    assert_eq!(c.logistics.month.losses, 1);
    assert!(c.logistics.month.lost_value > Money::ZERO);
    assert!(c.ledger.is_balanced());
    assert!(inventory_matches(&game, player));
}

#[test]
fn selling_vehicles_returns_a_share_of_the_book_value() {
    let mut game = new_game(with_logistics(0.0));
    let player = game.player();
    let vehicle = carrier(&game);
    game.apply(Command::BuyVehicles { vehicle, count: 2 })
        .unwrap();
    let cash = game.state().companies[player.index()].ledger.cash();
    let book = game.state().companies[player.index()].logistics.fleet[0].value;
    game.apply(Command::SellVehicles { vehicle, count: 1 })
        .unwrap();
    let c = &game.state().companies[player.index()];
    assert_eq!(c.ledger.cash(), cash + book.scale(0.5).scale(0.6));
    assert_eq!(c.logistics.fleet[0].count, 1);
    assert_eq!(
        game.apply(Command::SellVehicles { vehicle, count: 2 }),
        Err(CommandError::TooManyVehicles { count: 1 })
    );
    game.apply(Command::SellVehicles { vehicle, count: 1 })
        .unwrap();
    let c = &game.state().companies[player.index()];
    assert!(c.logistics.fleet.is_empty());
    assert_eq!(c.ledger.balance(Account::FixedAssets), Money::ZERO);
    assert!(c.ledger.is_balanced());
}

#[test]
fn vehicles_without_payload_or_data_are_refused() {
    let mut game = new_game(with_logistics(0.0));
    let cart = game.catalog().vehicles.id("karren").unwrap();
    assert_eq!(
        game.apply(Command::BuyVehicles {
            vehicle: cart,
            count: 1
        }),
        Err(CommandError::VehicleNotForFleet)
    );
    let mut plain = new_game(test_support::trading());
    let ship = plain.catalog().vehicles.id("dampfer").unwrap();
    assert_eq!(
        plain.apply(Command::BuyVehicles {
            vehicle: ship,
            count: 1
        }),
        Err(CommandError::NoLogistics)
    );
}

#[test]
fn free_capacity_earns_freight_of_others() {
    // Rental for a month of freight on the market: `market` tkm by land and by sea.
    let rental = |market: f64| {
        let mut game = new_game(with_logistics(0.0));
        let player = game.player();
        let vehicle = carrier(&game);
        game.apply(Command::BuyVehicles { vehicle, count: 1 })
            .unwrap();
        set_mode(&mut game, FreightMode::Fleet, true);
        days(&mut game, 30);
        let fm = &mut game.state_mut().freight_market;
        (fm.land, fm.sea) = (market, market);
        days(&mut game, 1);
        let c = &game.state().companies[player.index()];
        assert!(c.ledger.is_balanced());
        assert_eq!(game.state().freight_market.land_last, market);
        c.logistics.last_month.rental
    };
    let ample = rental(1e12);
    assert!(ample > Money::ZERO);
    // A small market takes less; none, nothing.
    let small = rental(1e5);
    assert!(small > Money::ZERO && small < ample, "{small:?} {ample:?}");
    assert_eq!(rental(0.0), Money::ZERO);
}

#[test]
fn ai_buys_vehicles_for_its_loads() {
    let mut game = new_game(with_logistics(0.0));
    let player = game.player();
    let vehicle = carrier(&game);
    let sea = game.catalog().vehicles.get(vehicle).way == crate::catalog::Way::Sea;
    let class = game.catalog().products.get(iron(&game)).transport_class;
    let date = game.state().date;
    let per = logistics::capacity_per_vehicle(game.catalog(), vehicle, date);
    let catalog = game.catalog().clone();
    game.state_mut().companies[player.index()]
        .logistics
        .loads_last = vec![Load {
        sea,
        class,
        tkm: 10.0 * per,
        market: 10.0 * per,
    }];
    let buys = logistics::ai_purchases(game.state(), &catalog, player);
    assert_eq!(buys, vec![(vehicle, 5)]);
    // Too little freight for one vehicle: none.
    game.state_mut().companies[player.index()]
        .logistics
        .loads_last[0]
        .tkm = per;
    assert!(logistics::ai_purchases(game.state(), &catalog, player).is_empty());
}

#[test]
fn fleets_survive_saving() {
    let mut game = new_game(with_logistics(0.01));
    let vehicle = carrier(&game);
    game.apply(Command::BuyVehicles { vehicle, count: 1 })
        .unwrap();
    set_mode(&mut game, FreightMode::Fleet, true);
    transfer(&mut game, 100.0).unwrap();
    let mut loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    days(&mut game, 40);
    days(&mut loaded, 40);
    assert_eq!(game.state_hash(), loaded.state_hash());
}

#[test]
fn lost_contract_deliveries_cost_the_seller() {
    use crate::contracts_tests::{agreed, cash, with_contracts};
    let mut game = new_game(with_contracts(with_logistics(1.0), 0.0));
    let player = game.player();
    let (seller, buyer, other) = agreed(&mut game, "BBB", 300.0, 2);
    let paid = cash(&game, other);
    days(&mut game, 31 + 10);
    let c = &game.state().contracts[0];
    assert_eq!(c.delivered_total, 0.0);
    assert!(stock(&game, seller) < 1000.0);
    assert_eq!(stock(&game, buyer), 0.0);
    assert!(game.state().shipments.is_empty());
    assert_eq!(cash(&game, other), paid);
    assert!(
        game.state().companies[player.index()]
            .logistics
            .month
            .losses
            > 0
    );
    for company in [player, other] {
        assert!(game.state().companies[company.index()].ledger.is_balanced());
        assert!(inventory_matches(&game, company));
    }
}

#[test]
fn vehicles_dearer_than_the_market_stay_idle() {
    let mut catalog = with_logistics(0.0);
    fleet_vehicle(&mut catalog, "karren", 2.0);
    let mut game = new_game(catalog);
    let player = game.player();
    let cart = game.catalog().vehicles.id("karren").unwrap();
    let class = game.catalog().products.get(iron(&game)).transport_class;
    assert!(logistics::running_share(game.catalog(), cart, class, 1900) > 1.0);
    game.apply(Command::BuyVehicles {
        vehicle: cart,
        count: 1,
    })
    .unwrap();
    set_mode(&mut game, FreightMode::Fleet, true);
    transfer(&mut game, 10.0).unwrap();
    assert_eq!(transport(&game), market_freight(&game, 10.0));
    days(&mut game, 31);
    let c = &game.state().companies[player.index()];
    assert!(c.logistics.fleet[0].used_last.abs() < 1e-9);
    // Nobody hires it either.
    assert_eq!(c.logistics.last_month.rental, Money::ZERO);
}
