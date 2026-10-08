//! Tests for transport and trade between countries (M8).

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::calendar::RoundLength;
use crate::catalog::{Catalog, SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::ids::{CountryId, ProductId};
use crate::ledger::{Account, CostType, Ledger};
use crate::market;
use crate::money::Money;
use crate::policy::{BuyerGroup, SalesRule, Scope};
use crate::rng::{SimRng, Stream};
use crate::save;
use crate::state::{
    Company, CompanyId, CompanyKind, GameSettings, PerId, PriceMode, Site, SiteId, StartForm,
};
use crate::transport::Routes;

pub(super) fn usd(v: f64) -> Money {
    Money::from_usd(v).unwrap()
}

pub(super) fn new_game(catalog: Catalog) -> Game {
    let catalog = Arc::new(catalog);
    let settings = GameSettings {
        seed: 3,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(1_000_000.0),
        start_form: StartForm::Trading,
        company_name: "Exporteur".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        person: Default::default(),
    };
    Game::new(catalog, settings).unwrap()
}

pub(super) fn country(game: &Game, key: &str) -> CountryId {
    game.catalog().countries.id(key).unwrap()
}

pub(super) fn iron(game: &Game) -> ProductId {
    game.catalog().products.id("eisen").unwrap()
}

/// A warehouse of `owner` in `key` holding `quantity` iron (value 1 USD per tonne).
pub(super) fn warehouse(game: &mut Game, owner: CompanyId, key: &str, quantity: f64) -> SiteId {
    let country = country(game, key);
    let p = iron(game);
    let groups = game.catalog().labor_groups.len();
    let state = game.state_mut();
    state.sites.push(Site {
        owner,
        country,
        kind: SiteType::Warehouse,
        founded: state.date,
        building_cost: Money::ZERO,
        deposit: None,
        slots: Vec::new(),
        inventory: Default::default(),
        workforce: PerId::from_fn(groups, |_| 0.0),
        staffing_due: false,
        offers: Default::default(),
        orders: Default::default(),
        research: None,
        development: None,
        wage_premium: 0.0,
        training: 0.0,
        training_target: None,
        acquired: None,
        goodwill: None,
        plot: None,
    });
    let site = SiteId(u32::try_from(state.sites.len() - 1).unwrap());
    if quantity > 0.0 {
        state.sites[site.index()]
            .inventory
            .entry(p)
            .or_default()
            .add(quantity, usd(quantity), 50.0);
        state.companies[owner.index()].ledger.transfer(
            Account::Inventory,
            Account::Equity,
            usd(quantity),
        );
    }
    site
}

pub(super) fn competitor(game: &mut Game) -> CompanyId {
    let state = game.state_mut();
    let id = CompanyId(u32::try_from(state.companies.len()).unwrap());
    let date = state.date;
    state.companies.push(Company {
        brands: Vec::new(),
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
        stock_cost: Default::default(),
        bonds: Vec::new(),
        bank: None,
        state_owned: None,
        former_managers: Vec::new(),
        owners: crate::state::Stake::sole(crate::state::Holder::Private),
        name: "Käufer".into(),
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

fn sell_fixed(game: &mut Game, site: SiteId, price: f64) {
    let product = iron(game);
    game.apply(Command::SetSale {
        site,
        product,
        mode: Some(PriceMode::Fixed(usd(price))),
        keep: 0.0,
    })
    .unwrap();
}

pub(super) fn stock(game: &Game, site: SiteId) -> f64 {
    game.state().sites[site.index()]
        .inventory
        .get(&iron(game))
        .map_or(0.0, |s| s.quantity)
}

pub(super) fn days(game: &mut Game, n: u32) {
    for _ in 0..n {
        game.advance(RoundLength::Day, |_| {});
    }
}

/// Quantity bought by traders in AAA over `n` days.
fn exported_over(game: &mut Game, n: u32) -> f64 {
    let (product, a) = (iron(game), country(game, "AAA"));
    let mut total = 0.0;
    for _ in 0..n {
        days(game, 1);
        total += game.state().markets.get(product).get(a).today.exported;
    }
    total
}

/// Inventory account equals the goods in warehouses and on the way.
pub(super) fn inventory_matches(game: &Game, company: CompanyId) -> bool {
    let state = game.state();
    let in_sites: Money = state
        .sites
        .iter()
        .filter(|s| s.owner == company)
        .flat_map(|s| s.inventory.values())
        .map(|s| s.value)
        .sum();
    let underway: Money = state
        .shipments
        .iter()
        .filter(|s| match s.to {
            crate::state::Consignee::Site(site) => state.sites[site.index()].owner == company,
            crate::state::Consignee::Importer(_) => false,
        })
        .map(|s| s.value)
        .sum();
    let books = state.companies[company.index()]
        .ledger
        .balance(Account::Inventory);
    (books - in_sites - underway).to_usd().abs() < 0.01
}

#[test]
fn routes_take_the_cheapest_way() {
    let catalog = test_support::trading();
    let routes = Routes::new(&catalog, 1900, None);
    let bulk = catalog.transport_classes.id("schuettgut").unwrap();
    let (a, b) = (
        catalog.countries.id("AAA").unwrap(),
        catalog.countries.id("BBB").unwrap(),
    );
    let route = routes.get(bulk, a, b).unwrap();
    // Both countries are coastal: the steamship beats the railway.
    assert!(route.by_sea);
    assert!(
        route.cost_per_t > 15.0 && route.cost_per_t < 40.0,
        "{route:?}"
    );
    assert_eq!(routes.get(bulk, b, a).unwrap(), route);
    assert_eq!(routes.get(bulk, a, a).unwrap().cost_per_t, 0.0);

    // Without a coast only the land way is left; rail beats the cart.
    let mut inland = test_support::trading();
    let id = inland.countries.id("BBB").unwrap();
    inland.countries.get_mut(id).landlocked = true;
    let land = Routes::new(&inland, 1900, None).get(bulk, a, b).unwrap();
    assert!(!land.by_sea);
    assert!(land.cost_per_t > route.cost_per_t);
    let rail_km = 714.0 * inland.transport_model.detour_land;
    assert!(land.cost_per_t < rail_km * 2.0, "{land:?}");

    // Vehicles count only from their first year.
    let mut later = test_support::trading();
    let ids: Vec<_> = later.vehicles.ids().collect();
    for id in ids {
        later.vehicles.get_mut(id).available_from = 1950;
    }
    assert!(Routes::new(&later, 1900, None).get(bulk, a, b).is_none());
    assert!(Routes::new(&later, 1950, None).get(bulk, a, b).is_some());
}

#[test]
fn transfer_abroad_costs_freight_and_time() {
    let mut game = new_game(test_support::trading());
    let player = game.player();
    let home = warehouse(&mut game, player, "AAA", 100.0);
    let abroad = warehouse(&mut game, player, "BBB", 0.0);
    let product = iron(&game);
    let (per_unit, travel) = game
        .state()
        .routes
        .for_product(
            game.catalog(),
            product,
            country(&game, "AAA"),
            country(&game, "BBB"),
        )
        .unwrap();
    game.apply(Command::TransferGoods {
        from: home,
        to: abroad,
        product,
        quantity: 100.0,
    })
    .unwrap();
    let ledger = &game.state().companies[player.index()].ledger;
    assert_eq!(
        ledger.month.by_type[&CostType::Transport],
        -Money::times(per_unit, 100.0)
    );
    assert_eq!(stock(&game, home), 0.0);
    assert_eq!(game.state().shipments.len(), 1);
    assert!(inventory_matches(&game, player));

    days(&mut game, travel - 1);
    assert_eq!(
        stock(&game, abroad),
        0.0,
        "not there before the arrival day"
    );
    days(&mut game, 2);
    assert_eq!(stock(&game, abroad), 100.0);
    assert!(game.state().shipments.is_empty());
    let ledger = &game.state().companies[player.index()].ledger;
    assert!(ledger.is_balanced());
    assert!(inventory_matches(&game, player));
}

#[test]
fn transfer_without_route_is_refused() {
    let mut game = new_game(test_support::production());
    let player = game.player();
    let home = warehouse(&mut game, player, "AAA", 10.0);
    let abroad = warehouse(&mut game, player, "BBB", 0.0);
    let product = iron(&game);
    let err = game
        .apply(Command::TransferGoods {
            from: home,
            to: abroad,
            product,
            quantity: 5.0,
        })
        .unwrap_err();
    assert!(matches!(err, CommandError::NoRoute { .. }), "{err:?}");
    assert_eq!(stock(&game, home), 10.0);
}

/// Player sells cheap iron in AAA; BBB has government demand and no supplier.
fn export_setup() -> (Game, SiteId) {
    let mut game = new_game(test_support::trading());
    let player = game.player();
    let site = warehouse(&mut game, player, "AAA", 100_000.0);
    sell_fixed(&mut game, site, 50.0);
    (game, site)
}

#[test]
fn traders_link_prices_up_to_transport_cost() {
    let (mut game, site) = export_setup();
    let product = iron(&game);
    let (a, b) = (country(&game, "AAA"), country(&game, "BBB"));
    let before = market::market_price(game.catalog(), game.state(), b, product);
    days(&mut game, 240);
    let state = game.state();
    let (transport, _) = state
        .routes
        .for_product(game.catalog(), product, a, b)
        .unwrap();
    let margin = game.catalog().market_model.trader_margin;
    let ceiling = (usd(50.0) + transport).scale(1.0 + margin);
    let price_b = market::market_price(game.catalog(), state, b, product);
    assert!(stock(&game, site) < 100_000.0, "traders bought");
    assert!(state.markets.get(product).get(b).last_month.imported > 0.0);
    assert!(state.markets.get(product).get(a).last_month.exported > 0.0);
    assert!(
        price_b < before,
        "imports lowered the price: {price_b:?} vs {before:?}"
    );
    // Price difference at most transport cost plus the traders' margin.
    assert!(price_b <= ceiling.scale(1.01), "{price_b:?} > {ceiling:?}");
    assert!(price_b >= usd(50.0) + transport);
    let ledger = &state.companies[game.player().index()].ledger;
    assert!(ledger.is_balanced());
    assert!(inventory_matches(&game, game.player()));
}

#[test]
fn sales_ban_stops_traders() {
    let (mut game, site) = export_setup();
    game.apply(Command::SetSalesPolicy {
        buyer: BuyerGroup::Traders,
        scope: Scope::Company,
        rule: Some(SalesRule {
            allowed: false,
            ..SalesRule::default()
        }),
    })
    .unwrap();
    assert_eq!(exported_over(&mut game, 90), 0.0);
    assert!(game.state().shipments.is_empty());

    // A product policy overrides the company one.
    let product = iron(&game);
    game.apply(Command::SetSalesPolicy {
        buyer: BuyerGroup::Traders,
        scope: Scope::Product(product),
        rule: Some(SalesRule {
            max_per_month: Some(50.0),
            ..SalesRule::default()
        }),
    })
    .unwrap();
    // Two months: at most 50 each.
    let date = game.date();
    let n = u32::try_from(date.days_until(date.add_days(59))).unwrap();
    let sold = exported_over(&mut game, n);
    assert!(sold > 0.0 && sold <= 100.0 + 1e-6, "{sold}");
    let offer = &game.state().sites[site.index()].offers[&product];
    assert!(offer.to_traders_month <= 50.0 + 1e-6);
}

#[test]
fn minimum_price_applies_to_traders() {
    let (mut game, site) = export_setup();
    game.apply(Command::SetSalesPolicy {
        buyer: BuyerGroup::Traders,
        scope: Scope::Company,
        rule: Some(SalesRule {
            min_price: Some(usd(60.0)),
            ..SalesRule::default()
        }),
    })
    .unwrap();
    assert_eq!(
        exported_over(&mut game, 60),
        0.0,
        "offer is below the minimum"
    );
    sell_fixed(&mut game, site, 60.0);
    assert!(exported_over(&mut game, 30) > 0.0);
}

#[test]
fn sales_ban_stops_other_companies() {
    let mut game = new_game(test_support::trading());
    let player = game.player();
    let site = warehouse(&mut game, player, "AAA", 1_000.0);
    sell_fixed(&mut game, site, 50.0);
    game.apply(Command::SetSalesPolicy {
        buyer: BuyerGroup::Traders,
        scope: Scope::Company,
        rule: Some(SalesRule {
            allowed: false,
            ..SalesRule::default()
        }),
    })
    .unwrap();
    let product = iron(&game);
    let aaa = country(&game, "AAA");
    game.apply(Command::SetSalesPolicy {
        buyer: BuyerGroup::Companies,
        scope: Scope::ProductInCountry(product, aaa),
        rule: Some(SalesRule {
            allowed: false,
            ..SalesRule::default()
        }),
    })
    .unwrap();
    let buyer = competitor(&mut game);
    let factory = warehouse(&mut game, buyer, "AAA", 0.0);
    game.apply_as(
        buyer,
        Command::SetPurchase {
            site: factory,
            product,
            target: 100.0,
            max_price: usd(80.0),
            min_quality: 0.0,
        },
    )
    .unwrap();
    days(&mut game, 5);
    assert_eq!(stock(&game, factory), 0.0);

    // Lifting the ban (back to the company default) lets the order through.
    game.apply(Command::SetSalesPolicy {
        buyer: BuyerGroup::Companies,
        scope: Scope::ProductInCountry(product, aaa),
        rule: None,
    })
    .unwrap();
    days(&mut game, 1);
    assert_eq!(stock(&game, factory), 100.0);
}

#[test]
fn shipments_survive_saving() {
    let (mut game, _) = export_setup();
    days(&mut game, 20);
    assert!(!game.state().shipments.is_empty());
    let mut loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    days(&mut game, 30);
    days(&mut loaded, 30);
    assert_eq!(game.state_hash(), loaded.state_hash());
}

/// BBB is served at home at a high fixed price; AAA sells cheaply (C1).
fn price_island(share: f64) -> (Game, SiteId) {
    let mut catalog = test_support::trading();
    catalog.market_model.arbitrage_share = share;
    let mut game = new_game(catalog);
    let player = game.player();
    let cheap = warehouse(&mut game, player, "AAA", 200_000.0);
    sell_fixed(&mut game, cheap, 50.0);
    let rival = competitor(&mut game);
    let dear = warehouse(&mut game, rival, "BBB", 200_000.0);
    let product = iron(&game);
    game.apply_as(
        rival,
        Command::SetSale {
            site: dear,
            product,
            mode: Some(PriceMode::Fixed(usd(150.0))),
            keep: 0.0,
        },
    )
    .unwrap();
    (game, dear)
}

#[test]
fn traders_bring_cheap_goods_to_a_price_island() {
    let imported = |share| {
        let (mut game, dear) = price_island(share);
        days(&mut game, 150);
        let product = iron(&game);
        let b = country(&game, "BBB");
        let state = game.state();
        let m = state.markets.get(product).get(b);
        (m.last_month.imported, m.last_month.sold, stock(&game, dear))
    };
    // Without arbitrage the local seller covers BBB and nothing comes in.
    let (none, sold, kept) = imported(0.0);
    assert!(sold > 0.0);
    assert!(none < 0.01 * sold, "{none} of {sold}");
    // With it, traders bring a share of the sales and the local seller sells less.
    let (some, sold_with, kept_with) = imported(0.25);
    assert!(some > 0.1 * sold_with, "{some} of {sold_with}");
    assert!(kept_with > kept, "the dear seller keeps more stock");
}

/// The trading catalog with one flat import tariff for all countries (W3), and an
/// embargo between AAA and BBB if wanted.
pub(super) fn with_tariff(rate: f64, embargo: bool) -> Catalog {
    use crate::catalog::{Embargo, TariffModel};
    use crate::time_series::TimeSeries;
    let mut catalog = test_support::trading();
    let (a, b) = (
        catalog.countries.id("AAA").unwrap(),
        catalog.countries.id("BBB").unwrap(),
    );
    catalog.tariffs = TariffModel {
        default: Some(TimeSeries::new(vec![(1900, rate)]).unwrap()),
        countries: vec![None; catalog.countries.len()],
        groups: vec![1.0; catalog.goods_groups.len()],
        embargoes: if embargo {
            vec![Embargo {
                countries: (a, b),
                from: 1900,
                until: None,
            }]
        } else {
            Vec::new()
        },
        ..TariffModel::default()
    };
    catalog
}

#[test]
fn transfer_abroad_pays_customs() {
    let mut game = new_game(with_tariff(0.2, false));
    let player = game.player();
    let home = warehouse(&mut game, player, "AAA", 100.0);
    let abroad = warehouse(&mut game, player, "BBB", 0.0);
    let product = iron(&game);
    game.apply(Command::TransferGoods {
        from: home,
        to: abroad,
        product,
        quantity: 100.0,
    })
    .unwrap();
    // 100 t worth 1 USD each, 20 % customs.
    let ledger = &game.state().companies[player.index()].ledger;
    assert_eq!(ledger.month.by_type[&CostType::Customs], -usd(20.0));
    assert!(ledger.is_balanced());
    assert!(inventory_matches(&game, player));
}

#[test]
fn an_embargo_refuses_transfers_and_stops_traders() {
    let mut game = new_game(with_tariff(0.0, true));
    let player = game.player();
    let home = warehouse(&mut game, player, "AAA", 100_000.0);
    let abroad = warehouse(&mut game, player, "BBB", 0.0);
    let product = iron(&game);
    let err = game
        .apply(Command::TransferGoods {
            from: home,
            to: abroad,
            product,
            quantity: 5.0,
        })
        .unwrap_err();
    assert!(matches!(err, CommandError::Embargo { .. }), "{err:?}");
    sell_fixed(&mut game, home, 50.0);
    days(&mut game, 120);
    let b = country(&game, "BBB");
    assert_eq!(
        game.state().markets.get(product).get(b).last_month.imported,
        0.0
    );
}

#[test]
fn tariffs_raise_the_price_of_imports() {
    let price_in_b = |rate: f64| {
        let mut game = new_game(with_tariff(rate, false));
        let player = game.player();
        let site = warehouse(&mut game, player, "AAA", 100_000.0);
        sell_fixed(&mut game, site, 50.0);
        days(&mut game, 240);
        let product = iron(&game);
        let (a, b) = (country(&game, "AAA"), country(&game, "BBB"));
        let state = game.state();
        assert!(state.markets.get(product).get(b).last_month.imported > 0.0);
        let (transport, _) = state
            .routes
            .for_product(game.catalog(), product, a, b)
            .unwrap();
        (
            market::market_price(game.catalog(), state, b, product),
            usd(50.0) + transport,
        )
    };
    let (free, landed) = price_in_b(0.0);
    let (taxed, _) = price_in_b(0.5);
    assert!(taxed >= landed.scale(1.5), "{taxed:?} < {landed:?} · 1.5");
    assert!(taxed > free, "{taxed:?} vs {free:?}");
}
