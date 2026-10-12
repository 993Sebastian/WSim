//! Tests for markets and prices (M7) with the consumer goods of the test catalog.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::ids::{CountryId, ProductId};
use crate::ledger::{CostType, Ledger};
use crate::market::{self, propensity};
use crate::message::keys;
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{Company, CompanyId, CompanyKind, GameSettings, PriceMode, SiteId, StartForm};

fn usd(v: f64) -> Money {
    Money::from_usd(v).unwrap()
}

fn new_game() -> Game {
    game_with(test_support::production())
}

fn game_with(catalog: crate::catalog::Catalog) -> Game {
    let catalog = Arc::new(catalog);
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
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        research_mandatory: false,
        found_at_start: true,
        person: Default::default(),
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
        process_shares: Default::default(),
        branch: None,
        owner,
        name: None,
        totals: std::collections::BTreeMap::new(),
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
        research_product: None,
        development: None,
        development_shares: std::collections::BTreeMap::new(),
        wage_premium: 0.0,
        training: 0.0,
        training_target: None,
        overhaul_below: None,
        acquired: None,
        goodwill: None,
        plot: None,
        supplies: std::collections::BTreeMap::new(),
        closed: None,
        storage_steps: 0,
        storage_full: false,
        experience: Default::default(),
        maintenance: 1.0,
        failures: 0,
        accidents: 0,
        trims: Default::default(),
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
        process: Default::default(),
        brands: Vec::new(),
        reputations: Vec::new(),
        sales_areas: Vec::new(),
        emissions: Default::default(),
        emissions_last: Default::default(),
        advertising: Vec::new(),
        auction_until: None,
        development: Default::default(),
        product_names: Default::default(),
        positions: Vec::new(),
        budget_rules: Vec::new(),
        strategies: Vec::new(),
        mandate: crate::mandate::Mandate::default(),
        reviews: Vec::new(),
        relocation: None,
        relocated: None,
        departments: Default::default(),
        departments_staffed: Default::default(),
        hq_city: None,
        participations: Default::default(),
        logistics: Default::default(),
        subsidiary_of: None,
        listing: None,
        dividend_payout: None,
        dividend: Default::default(),
        stock_cost: Default::default(),
        bonds: Vec::new(),
        bank: None,
        state_owned: None,
        former_managers: Vec::new(),
        owners: crate::state::Stake::sole(crate::state::Holder::Private),
        name: "Konkurrenz".into(),
        kind: CompanyKind::Ai,
        headquarters: state.settings.start_country,
        founded: date,
        rng: SimRng::for_stream(1, Stream::Company(id.0)),
        ledger: Ledger::new(date, usd(1_000_000.0)),
        technologies: BTreeSet::new(),
        known_products: Default::default(),
        product_points: Default::default(),
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

/// M16: Customers find a seller in proportion to its presence (production and stock); a
/// small newcomer does not win a large share just by appearing.
#[test]
fn small_sellers_win_customers_in_proportion_to_their_presence() {
    let mut game = new_game();
    let player = game.player();
    let rival = competitor(&mut game);
    let small = warehouse(&mut game, player, "brot", 1e6, 50.0);
    let large = warehouse(&mut game, rival, "brot", 1e9, 50.0);
    sell(&mut game, small, "brot", PriceMode::Fixed(usd(2.0)));
    sell(&mut game, large, "brot", PriceMode::Fixed(usd(2.0)));
    day(&mut game);
    let small_sold = 1e6 - stock(&game, small, "brot");
    let large_sold = 1e9 - stock(&game, large, "brot");
    assert!(small_sold > 0.0);
    let ratio = small_sold / large_sold;
    assert!(
        (0.0005..0.002).contains(&ratio),
        "{small_sold} {large_sold}"
    );
}

/// M16: In a saturated market an unknown newcomer sells less than a known brand at the
/// same price and quality; a lower price or advertising wins customers back.
#[test]
fn known_brands_win_customers_and_newcomers_must_compete() {
    let setup = |newcomer_price: f64| {
        let mut game = new_game();
        let player = game.player();
        let rival = competitor(&mut game);
        let new = warehouse(&mut game, player, "brot", 1e9, 50.0);
        let known = warehouse(&mut game, rival, "brot", 1e9, 50.0);
        let country = aaa(&game);
        let group = game
            .catalog()
            .products
            .get(product(&game, "brot"))
            .goods_group;
        game.state_mut().companies[rival.index()].brands = vec![crate::state::Brand {
            country,
            group,
            awareness: 0.6,
        }];
        sell(
            &mut game,
            new,
            "brot",
            PriceMode::Fixed(usd(newcomer_price)),
        );
        sell(&mut game, known, "brot", PriceMode::Fixed(usd(2.0)));
        (game, new, known)
    };
    let (mut game, new, known) = setup(2.0);
    day(&mut game);
    let sold = |g: &Game, s| 1e9 - stock(g, s, "brot");
    let (n, k) = (sold(&game, new), sold(&game, known));
    assert!(k > 1.3 * n, "bekannt {k}, neu {n}");

    // A lower price wins customers back.
    let (mut game, new, known) = setup(1.6);
    day(&mut game);
    let (n2, k2) = (sold(&game, new), sold(&game, known));
    assert!(n2 / k2 > n / k, "{n2} {k2}");

    // Advertising makes the newcomer known.
    let (mut game, new, known) = setup(2.0);
    let country = aaa(&game);
    let group = game
        .catalog()
        .products
        .get(product(&game, "brot"))
        .goods_group;
    let reach = crate::brand::reach_usd(game.state(), game.catalog(), country);
    game.apply(Command::SetAdvertising {
        country,
        group,
        budget: usd(3.0 * reach),
    })
    .unwrap();
    game.advance(RoundLength::Month, |_| {});
    let player = game.player();
    let awareness = game.state().companies[player.index()].awareness(country, group);
    assert!(awareness > 0.9, "{awareness}");
    let before = (sold(&game, new), sold(&game, known));
    day(&mut game);
    let after = (sold(&game, new) - before.0, sold(&game, known) - before.1);
    assert!(after.0 > after.1, "{after:?}");
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
    // Markets of goods only companies buy close their month as well (before M16 their
    // month never closed and the views showed nothing).
    let country = aaa(&game);
    game.advance(RoundLength::Month, |_| {});
    let last = &game.state().markets.get(ore).get(country).last_month;
    assert!(last.sold >= 1_000.0, "{last:?}");
    game.advance(RoundLength::Month, |_| {});
    let last = &game.state().markets.get(ore).get(country).last_month;
    assert!(last.sold < 1_000.0, "{last:?}");
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
fn a_cost_plus_price_follows_the_production_cost() {
    let mut game = new_game();
    let player = game.player();
    // 1 USD a unit in stock, 50 % margin.
    let site = warehouse(&mut game, player, "eisen", 1e6, 50.0);
    sell(
        &mut game,
        site,
        "eisen",
        PriceMode::CostPlus { margin: 0.5 },
    );
    let eisen = product(&game, "eisen");
    let price = |game: &Game| game.state().sites[site.index()].offers[&eisen].price;
    assert_eq!(price(&game), usd(1.5));
    // Dearer goods come in: the average cost and with it the price rise.
    let s = &mut game.state_mut().sites[site.index()];
    s.inventory
        .get_mut(&eisen)
        .unwrap()
        .add(1e6, usd(3e6), 50.0);
    day(&mut game);
    let st = &game.state().sites[site.index()].inventory[&eisen];
    let expected = st.value.to_usd() / st.quantity * 1.5;
    assert!(
        (price(&game).to_usd() - expected).abs() < 1e-3,
        "{:?}",
        price(&game)
    );
    assert!(price(&game).to_usd() > 2.9);
    // An unreasonable margin is refused.
    let err = game.apply(Command::SetSale {
        site,
        product: eisen,
        mode: Some(PriceMode::CostPlus { margin: -0.9 }),
        keep: 0.0,
    });
    assert!(err.is_err());
}

#[test]
fn state_demand_follows_its_profile_over_time() {
    use crate::catalog::StateDemand;
    use crate::time_series::TimeSeries;
    let rate = |index: Option<TimeSeries>| {
        let mut catalog = test_support::production();
        let iron = catalog.products.id("eisen").unwrap();
        catalog
            .products
            .get_mut(iron)
            .state_demand
            .as_mut()
            .unwrap()
            .index = index;
        let mut game = game_with(catalog);
        day(&mut game);
        game.state().markets.get(iron).get(aaa(&game)).state_rate
    };
    let plain = rate(None);
    assert!(plain > 0.0);
    let doubled = rate(Some(TimeSeries::new(vec![(1900, 2.0)]).unwrap()));
    assert!((doubled / plain - 2.0).abs() < 1e-9, "{plain} {doubled}");
    // Between two years the factor follows the date.
    let demand = StateDemand {
        per_million_gdp: 0.5,
        war_factor: 1.0,
        armament: false,
        index: Some(TimeSeries::new(vec![(1900, 1.0), (1910, 3.0)]).unwrap()),
    };
    let mid = Date::new(1905, 1, 1).unwrap();
    assert!((demand.per_million_gdp_at(mid) - 1.0).abs() < 1e-12);
    let later = demand.per_million_gdp_at(Date::new(1905, 7, 2).unwrap());
    assert!(later > 1.04 && later < 1.06, "{later}");
    assert_eq!(
        demand.per_million_gdp_at(Date::new(1920, 1, 1).unwrap()),
        1.5
    );
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

/// M32: nobody asks for a product that cannot be made yet; the demand comes with the
/// technology – by its invention year or as soon as a company knows it.
#[test]
fn demand_waits_for_the_technology() {
    let mut catalog = test_support::production();
    let later = catalog.technologies.id("hochofen_2000").unwrap();
    let bread = catalog.products.id("brot").unwrap();
    let smelting = catalog.recipes.id("eisen_schmelzen").unwrap();
    let mut baking = catalog.recipes.get(smelting).clone();
    baking.product = bread;
    baking.technology = Some(later);
    catalog.recipes.insert("brot_backen", baking).unwrap();
    let settings = new_game().state().settings.clone();
    let mut game = Game::new(Arc::new(catalog), settings).unwrap();
    game.advance(RoundLength::Month, |_| {});
    let rate = |game: &Game| -> f64 {
        game.state()
            .markets
            .get(product(game, "brot"))
            .get(aaa(game))
            .consumer_rate
            .iter()
            .sum()
    };
    assert_eq!(rate(&game), 0.0);
    let player = game.player();
    game.state_mut().companies[player.index()]
        .technologies
        .insert(later);
    game.advance(RoundLength::Month, |_| {});
    assert!(rate(&game) > 0.0);
}

/// M33: a successor with state demand displaces the state demand of the good it
/// replaces evenly over `verdraengung_staat_jahre` from its invention – earlier if a
/// company invents it ahead of history.
#[test]
fn a_successor_displaces_the_state_demand() {
    let mut catalog = test_support::research();
    let turbine = catalog.technologies.id("turbine").unwrap();
    let iron = catalog.products.id("eisen").unwrap();
    let mut steel = catalog.products.get(iron).clone();
    steel.replaces = vec![iron];
    let steel = catalog.products.insert("stahl", steel).unwrap();
    let smelting = catalog.recipes.id("eisen_schmelzen").unwrap();
    let mut making = catalog.recipes.get(smelting).clone();
    making.product = steel;
    making.technology = Some(turbine);
    catalog.recipes.insert("stahl_schmelzen", making).unwrap();
    let years = catalog.market_model.state_displacement_years;
    let settings = new_game().state().settings.clone();
    let mut game = Game::new(Arc::new(catalog), settings).unwrap();
    let catalog = game.catalog().clone();
    let at = |year: f64| {
        let y = year.floor();
        // Whole days into the year; the cast cannot overflow.
        Date::new(y as i32, 1, 1)
            .unwrap()
            .add_days(((year - y) * 365.0).round() as i32)
    };
    // The turbine is invented in 1902: full demand before, half after half the years.
    assert_eq!(
        market::available_since(game.state(), &catalog, steel),
        Some(1902.0)
    );
    let left =
        |game: &Game, year: f64| market::state_demand_left(game.state(), &catalog, iron, at(year));
    assert_eq!(left(&game, 1901.0), 1.0);
    assert!((left(&game, 1902.0 + years / 2.0) - 0.5).abs() < 0.01);
    assert_eq!(left(&game, 1902.0 + years + 1.0), 0.0);
    // The successor itself keeps its demand.
    assert_eq!(
        market::state_demand_left(game.state(), &catalog, steel, at(1950.0)),
        1.0
    );
    // A company invents it in 1901: the displacement starts then.
    *game.state_mut().inventions.get_mut(turbine) = Some(Date::new(1901, 1, 1).unwrap());
    assert_eq!(
        market::available_since(game.state(), &catalog, steel),
        Some(1901.0)
    );
    assert!((left(&game, 1901.0 + years / 2.0) - 0.5).abs() < 0.01);
}

/// H3: armament goods follow the military expenditure, war-relevant goods rise with the
/// war intensity.
#[test]
fn state_demand_follows_military_expenditure_and_war() {
    use crate::catalog::{ArmamentModel, CountrySeries};
    use crate::time_series::TimeSeries;
    let rate = |share: Option<f64>, war_factor: f64, armament: bool| {
        let mut catalog = test_support::production();
        if let Some(share) = share {
            catalog.armament = ArmamentModel {
                enabled: true,
                military_share: CountrySeries {
                    default: Some(TimeSeries::new(vec![(1890, share), (2100, share)]).unwrap()),
                    countries: Vec::new(),
                },
                peace: 0.06,
                war: 0.30,
                reference: 0.03,
                provenance: Default::default(),
            };
        }
        let iron = catalog.products.id("eisen").unwrap();
        let demand = catalog
            .products
            .get_mut(iron)
            .state_demand
            .as_mut()
            .unwrap();
        demand.war_factor = war_factor;
        demand.armament = armament;
        let mut game = game_with(catalog);
        day(&mut game);
        let country = game.state().countries.get(aaa(&game)).clone();
        (
            game.state().markets.get(iron).get(aaa(&game)).state_rate,
            country.war_intensity,
        )
    };
    let (plain, _) = rate(None, 3.0, false);
    assert!(plain > 0.0);
    // Peace: no war factor.
    let (peace, w) = rate(Some(0.03), 3.0, false);
    assert_eq!(w, 0.0);
    assert!((peace / plain - 1.0).abs() < 1e-9);
    // Half way to full war: 1 + (3 − 1) · 0.5 = 2.
    let (half, w) = rate(Some(0.18), 3.0, false);
    assert!((w - 0.5).abs() < 1e-9, "{w}");
    assert!((half / plain - 2.0).abs() < 1e-9, "{half} {plain}");
    // Full war beyond the war level.
    let (full, w) = rate(Some(0.6), 3.0, false);
    assert_eq!(w, 1.0);
    assert!((full / plain - 3.0).abs() < 1e-9);
    // Armament goods: share / reference, no war factor.
    let (arms, _) = rate(Some(0.06), 1.0, true);
    assert!((arms / plain - 2.0).abs() < 1e-9, "{arms} {plain}");
}

/// A one-off purchase buys its quantity on top of the stock target and then ends; while
/// the market lacks goods it waits, and setting the stock target leaves it open.
#[test]
fn a_one_off_purchase_buys_its_quantity_once() {
    let mut game = new_game();
    let player = game.player();
    let rival = competitor(&mut game);
    let seller = warehouse(&mut game, rival, "erz", 600.0, 60.0);
    sell(&mut game, seller, "erz", PriceMode::Fixed(usd(12.0)));
    let buyer = warehouse(&mut game, player, "eisen", 0.0, 50.0);
    let ore = product(&game, "erz");
    let once = |quantity: f64, max_price| Command::BuyOnce {
        site: buyer,
        product: ore,
        quantity,
        max_price,
    };
    // Too low a limit: nothing happens, the order waits.
    game.apply(once(1_000.0, usd(10.0))).unwrap();
    day(&mut game);
    assert_eq!(stock(&game, buyer, "erz"), 0.0);
    game.apply(once(1_000.0, usd(15.0))).unwrap();
    // A stock target set meanwhile keeps the open quantity.
    game.apply(Command::SetPurchase {
        site: buyer,
        product: ore,
        target: 0.0,
        max_price: Money::ZERO,
        min_quality: 0.0,
    })
    .unwrap();
    day(&mut game);
    // Only 600 t on offer: 400 t stay open.
    assert_eq!(stock(&game, buyer, "erz"), 600.0);
    let order = &game.state().sites[buyer.index()].orders[&ore];
    assert!((order.once - 400.0).abs() < 1e-6, "{order:?}");
    assert_eq!(order.bought_month, 600.0);
    // Fresh goods: the rest is bought, then the order is gone.
    game.state_mut().sites[seller.index()]
        .inventory
        .get_mut(&ore)
        .unwrap()
        .quantity += 1_000.0;
    day(&mut game);
    assert_eq!(stock(&game, buyer, "erz"), 1_000.0);
    assert!(!game.state().sites[buyer.index()].orders.contains_key(&ore));
    assert!(game.state().company(player).unwrap().ledger.is_balanced());
    // Invalid quantities are refused.
    assert!(game.apply(once(-1.0, usd(15.0))).is_err());
    assert!(game.apply(once(10.0, Money::ZERO)).is_err());
}

/// The commodity cycle swings each raw material's output monthly, within its limit, the
/// same for the same seed.
#[test]
fn the_commodity_cycle_swings_output_within_its_limit() {
    let run = || {
        let mut catalog = test_support::production();
        catalog.production_model.commodity_cycle = crate::catalog::CommodityCycle {
            shock: 0.2,
            persistence: 0.9,
            limit: 0.3,
        };
        let settings = new_game().state().settings.clone();
        let mut game = Game::new(Arc::new(catalog), settings).unwrap();
        let mut seen = Vec::new();
        for _ in 0..24 {
            game.advance(RoundLength::Month, |_| {});
            seen.push(game.state().commodity_cycle.clone());
        }
        (game, seen)
    };
    let (game, seen) = run();
    let values: Vec<f64> = seen.iter().flat_map(|m| m.values().copied()).collect();
    assert!(!values.is_empty());
    assert!(values.iter().all(|x| x.abs() <= 0.3 + 1e-12));
    assert!(values.iter().any(|x| x.abs() > 0.05), "{values:?}");
    // The allowed output follows the factor.
    let (d, dep) = game.catalog().deposits.iter().next().unwrap();
    let factor = crate::production::cycle_factor(game.state(), dep.resource);
    let year = game.state().date.year();
    let full = game.catalog().max_output(d, year);
    let allowed = crate::production::deposit_output(game.catalog(), game.state(), d, year);
    assert!(allowed <= full * factor + 1e-9);
    assert_eq!(run().1, seen);
}

#[test]
fn a_relative_price_follows_the_other_sellers() {
    let mut game = new_game();
    let player = game.player();
    let rival = competitor(&mut game);
    let mine = warehouse(&mut game, player, "brot", 1e9, 50.0);
    let theirs = warehouse(&mut game, rival, "brot", 1e9, 50.0);
    sell(&mut game, theirs, "brot", PriceMode::Fixed(usd(3.0)));
    sell(
        &mut game,
        mine,
        "brot",
        PriceMode::Relative {
            deviation: -0.1,
            floor: Money::ZERO,
        },
    );
    let p = product(&game, "brot");
    let price = |game: &Game| game.state().sites[mine.index()].offers[&p].price.to_usd();
    day(&mut game);
    assert!((price(&game) - 2.7).abs() < 0.01, "{}", price(&game));
    // The other seller raises its price: the own one follows at the same deviation.
    sell(&mut game, theirs, "brot", PriceMode::Fixed(usd(4.0)));
    day(&mut game);
    assert!((price(&game) - 3.6).abs() < 0.01, "{}", price(&game));
}

#[test]
fn the_demand_of_an_offer_learns_only_on_days_with_goods() {
    let mut catalog = crate::catalog::test_support::trading();
    catalog.logistics.supply_learn_rate = 0.1;
    catalog.logistics.supply_sold_out_premium = 0.25;
    let mut offer = crate::state::SaleOffer {
        demand: 10.0,
        mode: crate::state::PriceMode::Fixed(crate::money::Money::from_units(1)),
        price: crate::money::Money::from_units(1),
        keep: 0.0,
        sold_today: 0.0,
        sold_month: 0.0,
        sold_last_month: 0.0,
        to_traders_month: 0.0,
        to_companies_month: 0.0,
    };
    let m = &catalog.logistics;
    // No goods: nothing to learn.
    crate::market::learn_demand(&catalog, &mut offer, 0.0, 0.0);
    assert!((offer.demand - 10.0).abs() < 1e-12);
    // Goods left over: follows the sales.
    crate::market::learn_demand(&catalog, &mut offer, 50.0, 20.0);
    assert!((offer.demand - (10.0 + m.supply_learn_rate * 10.0)).abs() < 1e-12);
    // Sold out: the day counts higher than what was sold.
    let before = offer.demand;
    crate::market::learn_demand(&catalog, &mut offer, 5.0, 5.0);
    let sample = 5.0 * (1.0 + m.supply_sold_out_premium);
    assert!((offer.demand - (before + m.supply_learn_rate * (sample - before))).abs() < 1e-12);
}

/// MN1: a cheaper consumable that replaces another takes over part of its demand where
/// it is sold, up to its highest share.
#[test]
fn a_cheaper_consumable_takes_over_part_of_the_demand() {
    let run = |substitute: bool| {
        let mut c = test_support::production();
        let bread = c.products.id("brot").unwrap();
        let mut margarine = c.products.get(bread).clone();
        margarine.replaces = vec![bread];
        margarine.replaces_max_share = 0.6;
        c.products.insert("margarine", margarine).unwrap();
        if substitute {
            c.market_model.substitution_steepness = 2.0;
            c.market_model.substitution_per_year = 3.0;
        }
        let mut game = game_with(c);
        let player = game.player();
        let site = warehouse(&mut game, player, "margarine", 1e12, 50.0);
        sell(&mut game, site, "margarine", PriceMode::Fixed(usd(1.0)));
        for _ in 0..24 {
            game.advance(RoundLength::Month, |_| {});
        }
        let rate = |key: &str| -> f64 {
            game.state()
                .markets
                .get(product(&game, key))
                .get(aaa(&game))
                .consumer_rate
                .iter()
                .sum()
        };
        let share = game
            .state()
            .markets
            .get(product(&game, "margarine"))
            .get(aaa(&game))
            .substitution
            .first()
            .copied()
            .unwrap_or(0.0);
        (rate("brot"), rate("margarine"), share)
    };
    let (bread_off, margarine_off, share_off) = run(false);
    let (bread_on, margarine_on, share_on) = run(true);
    assert_eq!(share_off, 0.0);
    assert!(share_on > 0.3 && share_on <= 0.6, "{share_on}");
    assert!(bread_on < 0.75 * bread_off, "{bread_on} {bread_off}");
    assert!(
        margarine_on > margarine_off,
        "{margarine_on} {margarine_off}"
    );
}

/// MN1: a trend multiplies the propensity to buy while its curve runs.
#[test]
fn a_trend_raises_the_demand_while_it_runs() {
    let mut c = test_support::production();
    let bread = c.products.id("brot").unwrap();
    let group = c.products.get(bread).goods_group;
    let trend = crate::catalog::Trend {
        key: "brotwelle".into(),
        groups: Vec::new(),
        products: vec![bread],
        countries: Vec::new(),
        curve: crate::time_series::TimeSeries::new(vec![(1899, 2.0), (1905, 2.0)]).unwrap(),
    };
    let country = c.countries.id("AAA").unwrap();
    assert_eq!(trend.factor(bread, group, country, 1902.0), 2.0);
    assert_eq!(trend.factor(bread, group, country, 1906.0), 1.0);
    c.market_model.trends.push(trend);
    let with = game_with(c);
    let without = new_game();
    let rate = |game: &Game| -> f64 {
        game.state()
            .markets
            .get(product(game, "brot"))
            .get(aaa(game))
            .consumer_rate
            .iter()
            .sum()
    };
    assert!((rate(&with) / rate(&without) - 2.0).abs() < 1e-9);
}

fn reputation_catalog(recall_chance: f64) -> crate::catalog::Catalog {
    let mut c = test_support::production();
    c.market_model.reputation = crate::catalog::ReputationModel {
        enabled: true,
        gain_per_month: 0.05,
        return_per_month: 0.02,
        accident_loss: 0.005,
        recall_below: 25.0,
        recall_chance,
        recall_loss: 0.1,
        weight: [0.2, 0.3, 0.5, 0.7, 1.0],
    };
    c
}

/// MN2: good quality raises the reputation, bad quality lowers it and is recalled; a
/// better reputation wins customers.
#[test]
fn reputation_follows_quality_and_recalls_cost() {
    let month = |chance: f64| {
        let mut game = game_with(reputation_catalog(chance));
        let player = game.player();
        let rival = competitor(&mut game);
        let good = warehouse(&mut game, rival, "brot", 1e9, 80.0);
        let bad = warehouse(&mut game, player, "brot", 1e9, 10.0);
        sell(&mut game, good, "brot", PriceMode::Fixed(usd(2.0)));
        sell(&mut game, bad, "brot", PriceMode::Fixed(usd(2.0)));
        let report = game.advance(RoundLength::Month, |_| {});
        (game, player, rival, report)
    };
    let (game, player, rival, report) = month(1.0);
    let country = aaa(&game);
    let s = game.state();
    let (mine, theirs) = (
        s.company(player).unwrap().reputation(country),
        s.company(rival).unwrap().reputation(country),
    );
    assert!(theirs > 0.5 && mine < 0.4, "{mine} {theirs}");
    assert!(
        report
            .messages
            .iter()
            .any(|m| m.key == crate::message::keys::RECALL),
        "recall reported"
    );
    let ledger = &s.company(player).unwrap().ledger;
    assert!(ledger.is_balanced());
    // Without the recall the month ends with more cash.
    let (calm, calm_player, _, calm_report) = month(0.0);
    assert!(
        !calm_report
            .messages
            .iter()
            .any(|m| m.key == crate::message::keys::RECALL)
    );
    let calm_cash = calm.state().company(calm_player).unwrap().ledger.cash();
    assert!(
        ledger.cash() < calm_cash,
        "{:?} {:?}",
        ledger.cash(),
        calm_cash
    );

    // Same price and quality: the better reputation sells more.
    let mut game = game_with(reputation_catalog(0.0));
    let player = game.player();
    let rival = competitor(&mut game);
    let a = warehouse(&mut game, player, "brot", 1e9, 50.0);
    let b = warehouse(&mut game, rival, "brot", 1e9, 50.0);
    sell(&mut game, a, "brot", PriceMode::Fixed(usd(2.0)));
    sell(&mut game, b, "brot", PriceMode::Fixed(usd(2.0)));
    let country = aaa(&game);
    let set = |game: &mut Game, company: CompanyId, value: f64| {
        game.state_mut().companies[company.index()]
            .reputations
            .push(crate::state::Reputation {
                country,
                value,
                sold: 0.0,
                quality_sum: 0.0,
                cost: Money::ZERO,
                low_quality: false,
            });
    };
    set(&mut game, player, 0.9);
    set(&mut game, rival, 0.1);
    day(&mut game);
    let sold = |s| 1e9 - stock(&game, s, "brot");
    assert!(sold(a) > 1.1 * sold(b), "{} {}", sold(a), sold(b));
}

/// MN2: a government's lot goes to the best bid and becomes a state order with daily
/// deliveries paid by the state.
#[test]
fn a_tender_awards_a_lot_as_state_order() {
    let mut c = test_support::production();
    let erze = c.goods_groups.id("erze").unwrap();
    c.market_model.tenders = crate::catalog::TenderModel {
        enabled: true,
        groups: vec![erze],
        lot_share: 0.5,
        deadline_days: 30,
        months: 12,
        price_weight: 2.0,
        quality_weight: 0.5,
        home_bonus: 0.3,
        war_factor: 3.0,
        reputation_weight: 1.0,
        ai_discount: 0.05,
        penalty: 0.2,
    };
    let mut game = game_with(c);
    let player = game.player();
    let rival = competitor(&mut game);
    let mine = warehouse(&mut game, player, "eisen", 1e6, 50.0);
    let theirs = warehouse(&mut game, rival, "eisen", 1e6, 50.0);
    let iron = product(&game, "eisen");
    let country = aaa(&game);
    // Only the lot: no sales to anyone else.
    for site in [mine, theirs] {
        let owner = game.state().sites[site.index()].owner;
        game.apply_as(
            owner,
            Command::SetSale {
                site,
                product: iron,
                mode: Some(PriceMode::Fixed(usd(1e6))),
                keep: 0.0,
            },
        )
        .unwrap();
    }
    let mut opened = None;
    for _ in 0..200 {
        let report = game.advance(RoundLength::Day, |_| {});
        if let Some(t) = game.state().tenders.iter().find(|t| t.product == iron) {
            assert!(
                report.messages.iter().any(|m| m.key == keys::TENDER_OPEN),
                "player told"
            );
            opened = Some(t.clone());
            break;
        }
    }
    let tender = opened.expect("a lot for iron");
    assert_eq!(tender.country, country);
    assert_eq!(game.state().date.month() % 3, 1);
    let reference = market::local_reference(game.catalog(), game.state(), country, iron);
    game.apply(Command::BidTender {
        tender: tender.id,
        site: mine,
        price: reference,
    })
    .unwrap();
    game.apply_as(
        rival,
        Command::BidTender {
            tender: tender.id,
            site: theirs,
            price: reference.scale(1.3),
        },
    )
    .unwrap();
    // A site abroad or without the product cannot bid.
    assert!(matches!(
        game.apply(Command::BidTender {
            tender: tender.id + 99,
            site: mine,
            price: reference,
        }),
        Err(CommandError::TenderClosed)
    ));
    let before = stock(&game, mine, "eisen");
    let mut won = false;
    for _ in 0..40 {
        let report = game.advance(RoundLength::Day, |_| {});
        won |= report.messages.iter().any(|m| m.key == keys::TENDER_WON);
    }
    assert!(won, "best bid wins");
    let s = game.state();
    let order = s.state_orders.iter().find(|o| o.id == tender.id).unwrap();
    assert_eq!(order.company, player);
    assert_eq!(order.price, reference);
    assert!(order.delivered_total > 0.0);
    let delivered = before - stock(&game, mine, "eisen");
    assert!((delivered - order.delivered_total).abs() < 1e-6);
    assert_eq!(stock(&game, theirs, "eisen"), 1e6);
    // The government buys the lot's quantity less on the market.
    let rest = s.markets.get(iron).get(country).state_rate;
    assert!(rest < 1.2 * tender.per_day, "{rest} {}", tender.per_day);
    assert!(s.company(player).unwrap().ledger.is_balanced());
}

fn channel_catalog() -> crate::catalog::Catalog {
    use crate::time_series::TimeSeries;
    let mut c = test_support::production();
    c.market_model.channels = crate::catalog::ChannelModel {
        base_reach: 0.5,
        ai_delay_years: 10.0,
        ai_margin: 0.25,
    };
    let channel = |from: i32, reach: f64| crate::catalog::Channel {
        available_from: from,
        groups: Vec::new(),
        reach: TimeSeries::new(vec![(1800, reach)]).unwrap(),
        layer_factor: [1.0; 5],
        fixed_cost_usd: TimeSeries::new(vec![(1800, 12_000.0)]).unwrap(),
        cost_per_sale: 0.1,
        reputation_factor: 1.0,
        provenance: Default::default(),
    };
    c.channels.insert("versand", channel(1890, 0.5)).unwrap();
    c.channels.insert("online", channel(1995, 0.9)).unwrap();
    c
}

/// MN3: a channel reaches more households – the seller with it sells more at the same
/// price and quality – and costs a share of the revenue and a fixed amount a month.
#[test]
fn a_sales_channel_wins_customers_and_costs() {
    let mut game = game_with(channel_catalog());
    let player = game.player();
    let rival = competitor(&mut game);
    let a = warehouse(&mut game, player, "brot", 1e9, 50.0);
    let b = warehouse(&mut game, rival, "brot", 1e9, 50.0);
    sell(&mut game, a, "brot", PriceMode::Fixed(usd(2.0)));
    sell(&mut game, b, "brot", PriceMode::Fixed(usd(2.0)));
    let country = aaa(&game);
    let group = game
        .catalog()
        .products
        .get(product(&game, "brot"))
        .goods_group;
    let (versand, online) = (
        game.catalog().channels.id("versand").unwrap(),
        game.catalog().channels.id("online").unwrap(),
    );
    // Not invented yet.
    assert!(matches!(
        game.apply(Command::SetChannels {
            country,
            group,
            channels: vec![online],
        }),
        Err(CommandError::NoChannel(_))
    ));
    game.apply(Command::SetChannels {
        country,
        group,
        channels: vec![versand],
    })
    .unwrap();
    let overhead = |game: &Game| {
        game.state()
            .company(player)
            .unwrap()
            .ledger
            .month
            .by_type
            .get(&CostType::Overhead)
            .map_or(Money::ZERO, |&m| Money::ZERO - m)
    };
    let before = overhead(&game);
    day(&mut game);
    let sold = |game: &Game, s| 1e9 - stock(game, s, "brot");
    // Reach 1.0 against 0.5: twice the attraction.
    let (mine, theirs) = (sold(&game, a), sold(&game, b));
    assert!(mine > 1.8 * theirs, "{mine} {theirs}");
    // Half of the sales run through the channel at 10 %.
    let cost = (overhead(&game) - before).to_usd();
    let expected = mine * 2.0 * 0.5 * 0.1;
    assert!(
        (cost - expected).abs() < 0.01 * expected + 0.01,
        "{cost} {expected}"
    );
    // The fixed cost at the month start: 12 000 USD a year at the price level.
    while game.state().date.day() != 1 {
        day(&mut game);
    }
    assert!(overhead(&game) > usd(500.0), "{:?}", overhead(&game));
    assert!(game.state().company(player).unwrap().ledger.is_balanced());
    // Saved and loaded with the channel.
    let loaded = crate::save::decode(&crate::save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(
        crate::channels::chosen(loaded.state().company(player).unwrap(), country, group),
        &[versand]
    );
}

/// MN3: AI companies take up a channel once it is established long enough for their
/// skill and the extra margin carries its cost.
#[test]
fn ai_channels_follow_skill_and_revenue() {
    let game = game_with(channel_catalog());
    let c = game.catalog();
    let country = aaa(&game);
    let group = c.products.get(product(&game, "brot")).goods_group;
    let versand = c.channels.id("versand").unwrap();
    let area = |revenue: f64| crate::channels::SalesArea {
        country,
        group,
        channels: Vec::new(),
        revenue: Money::ZERO,
        revenue_last_year: usd(revenue),
    };
    let s = game.state();
    // 1900: ten years after 1890 suffice for skill 0, five for skill 0.5.
    assert_eq!(
        crate::channels::ai_choice(c, s, &area(1e7), 0.5, 1900),
        vec![versand]
    );
    assert_eq!(
        crate::channels::ai_choice(c, s, &area(1e7), 0.0, 1899),
        Vec::new()
    );
    // Too little revenue for the fixed cost.
    assert_eq!(
        crate::channels::ai_choice(c, s, &area(1000.0), 1.0, 1900),
        Vec::new()
    );
    // A channel whose share of the revenue eats the margin is left out.
    let mut costly = channel_catalog();
    let id = costly.channels.id("versand").unwrap();
    costly.channels.get_mut(id).cost_per_sale = 0.6;
    let game = game_with(costly);
    assert_eq!(
        crate::channels::ai_choice(game.catalog(), game.state(), &area(1e7), 1.0, 1900),
        Vec::new()
    );
}
