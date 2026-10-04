//! Tests for markets and prices (M7) with the consumer goods of the test catalog.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{SiteType, test_support};
use crate::command::Command;
use crate::game::Game;
use crate::ids::{CountryId, ProductId};
use crate::ledger::{CostType, Ledger};
use crate::market::{self, propensity};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{Company, CompanyId, CompanyKind, GameSettings, PriceMode, SiteId, StartForm};

fn usd(v: f64) -> Money {
    Money::from_usd(v).unwrap()
}

fn new_game() -> Game {
    let catalog = Arc::new(test_support::production());
    let settings = GameSettings {
        seed: 9,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(1_000_000.0),
        start_form: StartForm::Trading,
        company_name: "Händler".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
    };
    Game::new(catalog, settings).unwrap()
}

fn aaa(game: &Game) -> CountryId {
    game.catalog().countries.id("AAA").unwrap()
}

fn product(game: &Game, key: &str) -> ProductId {
    game.catalog().products.id(key).unwrap()
}

/// A warehouse of `owner` in AAA holding `quantity` of `key` (quality 50, value 1 USD each).
fn warehouse(game: &mut Game, owner: CompanyId, key: &str, quantity: f64, quality: f64) -> SiteId {
    let country = aaa(game);
    let p = product(game, key);
    let groups = game.catalog().labor_groups.len();
    let state = game.state_mut();
    state.sites.push(crate::state::Site {
        owner,
        country,
        kind: SiteType::Warehouse,
        founded: state.date,
        building_cost: Money::ZERO,
        deposit: None,
        slots: Vec::new(),
        inventory: Default::default(),
        workforce: crate::state::PerId::from_fn(groups, |_| 0.0),
        staffing_due: false,
        offers: Default::default(),
        orders: Default::default(),
        research: None,
    });
    let site = SiteId(u32::try_from(state.sites.len() - 1).unwrap());
    state.sites[site.index()]
        .inventory
        .entry(p)
        .or_default()
        .add(quantity, usd(quantity), quality);
    // Keep the books right: the goods came in as equity.
    let ledger = &mut state.companies[owner.index()].ledger;
    ledger.transfer(
        crate::ledger::Account::Inventory,
        crate::ledger::Account::Equity,
        usd(quantity),
    );
    site
}

fn competitor(game: &mut Game) -> CompanyId {
    let state = game.state_mut();
    let id = CompanyId(u32::try_from(state.companies.len()).unwrap());
    let date = state.date;
    state.companies.push(Company {
        name: "Konkurrenz".into(),
        kind: CompanyKind::Ai,
        headquarters: state.settings.start_country,
        founded: date,
        rng: SimRng::for_stream(1, Stream::Company(id.0)),
        ledger: Ledger::new(date, usd(1_000_000.0)),
        technologies: BTreeSet::new(),
        bankrupt: false,
        loans: Vec::new(),
        loss_carryforward: Money::ZERO,
        sales_policies: Vec::new(),
        research: Default::default(),
        ai: None,
    });
    id
}

fn sell(game: &mut Game, site: SiteId, key: &str, mode: PriceMode) {
    let p = product(game, key);
    let owner = game.state().sites[site.index()].owner;
    game.apply_as(
        owner,
        Command::SetSale {
            site,
            product: p,
            mode: Some(mode),
            keep: 0.0,
        },
    )
    .unwrap();
}

fn stock(game: &Game, site: SiteId, key: &str) -> f64 {
    let p = product(game, key);
    game.state().sites[site.index()]
        .inventory
        .get(&p)
        .map_or(0.0, |s| s.quantity)
}

fn day(game: &mut Game) {
    game.advance(RoundLength::Day, |_| {});
}

#[test]
fn propensity_follows_income_and_price() {
    let game = new_game();
    let d = game
        .catalog()
        .products
        .get(product(&game, "rad"))
        .consumer_demand
        .clone()
        .unwrap();
    // Income equal to threshold × price → half the layer buys.
    assert!((propensity(&d, 100.0, 100.0, 100.0) - 0.5).abs() < 1e-12);
    assert!(propensity(&d, 50.0, 100.0, 100.0) < 0.5);
    // A price cut opens the market to poorer layers.
    assert!(propensity(&d, 50.0, 60.0, 100.0) > propensity(&d, 50.0, 100.0, 100.0));
    assert_eq!(propensity(&d, 0.0, 100.0, 100.0), 0.0);
}

#[test]
fn consumer_demand_per_layer() {
    let game = new_game();
    let state = game.state();
    let c = aaa(&game);
    let m = state.markets.get(product(&game, "brot")).get(c);
    let cs = state.countries.get(c);
    let d = game
        .catalog()
        .products
        .get(product(&game, "brot"))
        .consumer_demand
        .clone()
        .unwrap();
    for q in 0..5 {
        let expected = 200.0 * propensity(&d, cs.income_quintiles_usd[q], 2.0, 2.0) * cs.population
            / 5.0
            / 365.0;
        assert!((m.consumer_rate[q] - expected).abs() < 1e-9);
    }
    // Richer layers buy more.
    assert!(m.consumer_rate.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn consumers_buy_and_revenue_is_booked() {
    let mut game = new_game();
    let player = game.player();
    let site = warehouse(&mut game, player, "brot", 1_000_000.0, 50.0);
    sell(&mut game, site, "brot", PriceMode::Fixed(usd(2.0)));
    let demand: f64 = game
        .state()
        .markets
        .get(product(&game, "brot"))
        .get(aaa(&game))
        .consumer_rate
        .iter()
        .sum();
    day(&mut game);
    let sold = 1_000_000.0 - stock(&game, site, "brot");
    assert!((sold - demand).abs() < 1e-6, "{sold} vs {demand}");
    let ledger = &game.state().company(player).unwrap().ledger;
    let revenue = ledger.month.by_type[&CostType::Revenue].to_usd();
    assert!(
        (revenue - 2.0 * sold).abs() < 0.01 * sold.max(1.0),
        "{revenue}"
    );
    assert!(ledger.is_balanced());
}

#[test]
fn cheaper_and_better_offers_win_more_customers() {
    let mut game = new_game();
    let player = game.player();
    let rival = competitor(&mut game);
    let cheap = warehouse(&mut game, player, "brot", 1e9, 50.0);
    let dear = warehouse(&mut game, rival, "brot", 1e9, 50.0);
    sell(&mut game, cheap, "brot", PriceMode::Fixed(usd(1.8)));
    sell(&mut game, dear, "brot", PriceMode::Fixed(usd(2.2)));
    day(&mut game);
    let sold = |s| 1e9 - stock(&game, s, "brot");
    assert!(
        sold(cheap) > 1.2 * sold(dear),
        "{} {}",
        sold(cheap),
        sold(dear)
    );

    // Same price, higher quality wins.
    let mut game = new_game();
    let player = game.player();
    let rival = competitor(&mut game);
    let good = warehouse(&mut game, player, "brot", 1e9, 80.0);
    let poor = warehouse(&mut game, rival, "brot", 1e9, 40.0);
    sell(&mut game, good, "brot", PriceMode::Fixed(usd(2.0)));
    sell(&mut game, poor, "brot", PriceMode::Fixed(usd(2.0)));
    day(&mut game);
    let sold = |s| 1e9 - stock(&game, s, "brot");
    assert!(sold(good) > sold(poor));
}

#[test]
fn automatic_prices_follow_supply_and_demand() {
    // Scarce supply: sold out every day with demand left → the price rises.
    let mut game = new_game();
    let player = game.player();
    let site = warehouse(&mut game, player, "brot", 10.0, 50.0);
    sell(
        &mut game,
        site,
        "brot",
        PriceMode::Market {
            markup: 0.0,
            floor: Money::ZERO,
        },
    );
    let p = product(&game, "brot");
    for _ in 0..5 {
        game.state_mut().sites[site.index()]
            .inventory
            .get_mut(&p)
            .unwrap()
            .add(10.0, usd(10.0), 50.0);
        day(&mut game);
    }
    let price = game.state().sites[site.index()].offers[&p].price;
    assert_eq!(
        price,
        usd(2.0)
            .scale(1.02)
            .scale(1.02)
            .scale(1.02)
            .scale(1.02)
            .scale(1.02)
    );

    // Glut at a high price: the price falls, but not below the floor.
    let mut game = new_game();
    let player = game.player();
    let site = warehouse(&mut game, player, "brot", 1e12, 50.0);
    sell(
        &mut game,
        site,
        "brot",
        PriceMode::Market {
            markup: 1.0,
            floor: usd(3.9),
        },
    );
    for _ in 0..10 {
        day(&mut game);
    }
    let price = game.state().sites[site.index()].offers[&product(&game, "brot")].price;
    assert_eq!(price, usd(3.9));
}

#[test]
fn the_market_price_follows_sales() {
    let mut game = new_game();
    let player = game.player();
    let site = warehouse(&mut game, player, "brot", 1e9, 50.0);
    sell(&mut game, site, "brot", PriceMode::Fixed(usd(3.0)));
    for _ in 0..30 {
        day(&mut game);
    }
    let price = game
        .state()
        .markets
        .get(product(&game, "brot"))
        .get(aaa(&game))
        .price
        .to_usd();
    // Smoothed towards 3.0 from the reference price 2.0.
    assert!(price > 2.9 && price < 3.0, "{price}");
}

#[test]
fn sites_buy_from_other_companies() {
    let mut game = new_game();
    let player = game.player();
    let rival = competitor(&mut game);
    let seller = warehouse(&mut game, rival, "erz", 5_000.0, 60.0);
    sell(&mut game, seller, "erz", PriceMode::Fixed(usd(12.0)));
    let buyer = warehouse(&mut game, player, "eisen", 0.0, 50.0);
    let ore = product(&game, "erz");
    // Too low a limit: nothing happens.
    game.apply(Command::SetPurchase {
        site: buyer,
        product: ore,
        target: 1_000.0,
        max_price: usd(10.0),
        min_quality: 0.0,
    })
    .unwrap();
    day(&mut game);
    assert_eq!(stock(&game, buyer, "erz"), 0.0);
    game.apply(Command::SetPurchase {
        site: buyer,
        product: ore,
        target: 1_000.0,
        max_price: usd(15.0),
        min_quality: 0.0,
    })
    .unwrap();
    day(&mut game);
    assert_eq!(stock(&game, buyer, "erz"), 1_000.0);
    let s = &game.state().sites[buyer.index()].inventory[&ore];
    assert_eq!(s.value, usd(12_000.0));
    assert_eq!(s.quality, 60.0);
    // The rival booked the revenue.
    let rival_ledger = &game.state().company(rival).unwrap().ledger;
    assert_eq!(
        rival_ledger.month.by_type[&CostType::Revenue],
        usd(12_000.0)
    );
    assert!(
        rival_ledger.is_balanced() && game.state().company(player).unwrap().ledger.is_balanced()
    );
}

#[test]
fn governments_buy_up_to_a_price_cap() {
    let mut game = new_game();
    let player = game.player();
    let site = warehouse(&mut game, player, "eisen", 1e6, 50.0);
    sell(&mut game, site, "eisen", PriceMode::Fixed(usd(160.0)));
    day(&mut game);
    assert_eq!(
        stock(&game, site, "eisen"),
        1e6,
        "above 1.5 × reference price 100"
    );
    sell(&mut game, site, "eisen", PriceMode::Fixed(usd(140.0)));
    day(&mut game);
    let rate = game
        .state()
        .markets
        .get(product(&game, "eisen"))
        .get(aaa(&game))
        .state_rate;
    assert!(rate > 0.0);
    assert!((1e6 - stock(&game, site, "eisen") - rate).abs() < 1e-6);
}

#[test]
fn the_state_market_supplies_goods_without_a_chain() {
    let mut game = new_game();
    for _ in 0..3 {
        game.advance(RoundLength::Month, |_| {});
    }
    let m = game
        .state()
        .markets
        .get(product(&game, "kutsche"))
        .get(aaa(&game));
    assert!(m.last_month.sold > 0.0);
    assert!(m.ownership.iter().any(|&o| o > 0.0), "{:?}", m.ownership);
}

#[test]
fn bicycles_displace_carriages() {
    let mut game = new_game();
    let player = game.player();
    for _ in 0..2 {
        game.advance(RoundLength::Month, |_| {});
    }
    let carriage = product(&game, "kutsche");
    let rate_before: f64 = game
        .state()
        .markets
        .get(carriage)
        .get(aaa(&game))
        .consumer_rate
        .iter()
        .sum();
    let site = warehouse(&mut game, player, "rad", 1e9, 50.0);
    sell(&mut game, site, "rad", PriceMode::Fixed(usd(60.0)));
    for _ in 0..36 {
        game.advance(RoundLength::Month, |_| {});
    }
    let bikes = game
        .state()
        .markets
        .get(product(&game, "rad"))
        .get(aaa(&game));
    assert!(bikes.ownership[4] > 0.1, "{:?}", bikes.ownership);
    let rate_after: f64 = game
        .state()
        .markets
        .get(carriage)
        .get(aaa(&game))
        .consumer_rate
        .iter()
        .sum();
    assert!(rate_after < rate_before, "{rate_after} {rate_before}");
}

#[test]
fn markets_survive_save_and_replay() {
    let mut game = new_game();
    let player = game.player();
    let site = warehouse(&mut game, player, "brot", 1e7, 50.0);
    sell(
        &mut game,
        site,
        "brot",
        PriceMode::Market {
            markup: 0.1,
            floor: usd(1.0),
        },
    );
    game.advance(RoundLength::Month, |_| {});
    let catalog = game.catalog().clone();
    let mut loaded = crate::save::decode(&crate::save::encode(&game), catalog)
        .unwrap()
        .game;
    game.advance(RoundLength::Quarter, |_| {});
    loaded.advance(RoundLength::Quarter, |_| {});
    assert_eq!(game.state_hash(), loaded.state_hash());
    assert!(game.date() >= Date::new(1900, 4, 1).unwrap());
    let _ = market::market_price(
        game.catalog(),
        game.state(),
        aaa(&game),
        product(&game, "brot"),
    );
}

#[test]
fn displacement_curve_is_gradual() {
    let mut game = new_game();
    let player = game.player();
    game.advance(RoundLength::Month, |_| {});
    let (carriage, bike, country) = (product(&game, "kutsche"), product(&game, "rad"), aaa(&game));
    let site = warehouse(&mut game, player, "rad", 1e9, 50.0);
    sell(&mut game, site, "rad", PriceMode::Fixed(usd(60.0)));
    let mut curve = Vec::new();
    for _ in 0..8 {
        let s = game.state();
        let owned: f64 = s.markets.get(bike).get(country).ownership.iter().sum();
        let carriages: f64 = s
            .markets
            .get(carriage)
            .get(country)
            .consumer_rate
            .iter()
            .sum();
        curve.push((owned, carriages));
        for _ in 0..6 {
            game.advance(RoundLength::Month, |_| {});
        }
    }
    // Bicycles spread, carriage demand falls step by step, not all at once.
    for pair in curve.windows(2) {
        assert!(pair[1].0 > pair[0].0, "{curve:?}");
        assert!(pair[1].1 < pair[0].1, "{curve:?}");
    }
    let (start, half_year, end) = (curve[0].1, curve[1].1, curve[7].1);
    assert!(half_year > 0.5 * start, "{curve:?}");
    assert!(end < 0.7 * start, "{curve:?}");
}
