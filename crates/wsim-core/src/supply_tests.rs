//! Tests of the internal supply between sites of a company or group (W5).

use crate::catalog::test_support;
use crate::command::{Command, CommandError};
use crate::group::SubsidiaryOf;
use crate::state::{PriceMode, SaleOffer};
use crate::supply::SupplyMode;
use crate::trade_tests::{
    competitor, days, inventory_matches, iron, new_game, stock, usd, warehouse,
};

fn set(
    game: &mut crate::game::Game,
    site: crate::state::SiteId,
    source: Option<crate::state::SiteId>,
    mode: SupplyMode,
) -> Result<(), CommandError> {
    let product = iron(game);
    game.apply(Command::SetInternalSupply {
        site,
        product,
        source,
        mode,
    })
    .map(|_| ())
}

#[test]
fn a_fixed_quantity_arrives_every_day() {
    let mut game = new_game(test_support::trading());
    let me = game.player();
    let plant = warehouse(&mut game, me, "AAA", 1000.0);
    let branch = warehouse(&mut game, me, "AAA", 0.0);
    set(
        &mut game,
        branch,
        Some(plant),
        SupplyMode::Fixed { per_day: 10.0 },
    )
    .unwrap();
    days(&mut game, 5);
    assert!((stock(&game, branch) - 50.0).abs() < 1e-9);
    assert!((stock(&game, plant) - 950.0).abs() < 1e-9);
    let s = &game.state().sites[branch.index()].supplies[&iron(&game)];
    assert!((s.delivered_month - 50.0).abs() < 1e-9);
    assert!(inventory_matches(&game, me));
    // Stopped, nothing more comes.
    set(&mut game, branch, None, SupplyMode::Fixed { per_day: 10.0 }).unwrap();
    days(&mut game, 2);
    assert!((stock(&game, branch) - 50.0).abs() < 1e-9);
}

#[test]
fn by_need_the_stock_is_topped_up_counting_goods_on_the_way() {
    let mut game = new_game(test_support::trading());
    let me = game.player();
    // Parts made abroad for a site at home: 10 a day sold, 5 days of stock wanted.
    let plant = warehouse(&mut game, me, "BBB", 1000.0);
    let branch = warehouse(&mut game, me, "AAA", 0.0);
    let product = iron(&game);
    game.state_mut().sites[branch.index()].offers.insert(
        product,
        SaleOffer {
            demand: 0.0,
            mode: PriceMode::Fixed(usd(1_000_000.0)),
            price: usd(1_000_000.0),
            keep: 0.0,
            sold_today: 0.0,
            sold_month: 0.0,
            sold_last_month: 300.0,
            to_traders_month: 0.0,
            to_companies_month: 0.0,
        },
    );
    set(
        &mut game,
        branch,
        Some(plant),
        SupplyMode::Need { days: 5.0 },
    )
    .unwrap();
    days(&mut game, 1);
    // 50 left as freight; the next day nothing more, it is on the way.
    assert!((stock(&game, plant) - 950.0).abs() < 1e-9);
    assert!((crate::supply::in_transit(game.state(), branch, product) - 50.0).abs() < 1e-9);
    days(&mut game, 1);
    assert!((stock(&game, plant) - 950.0).abs() < 1e-9);
    assert!(game.state().companies[me.index()].ledger.is_balanced());
    assert!(inventory_matches(&game, me));
}

#[test]
fn a_subsidiary_buys_at_the_value_of_the_stock_and_outsiders_are_refused() {
    let mut game = new_game(test_support::trading());
    let me = game.player();
    let daughter = competitor(&mut game);
    game.state_mut().companies[daughter.index()].subsidiary_of = Some(SubsidiaryOf {
        parent: me,
        focus: Default::default(),
    });
    // The warehouse helper values iron at 1 USD a unit.
    let plant = warehouse(&mut game, me, "AAA", 100.0);
    let theirs = warehouse(&mut game, daughter, "AAA", 0.0);
    let before = (
        game.state().companies[me.index()].ledger.cash(),
        game.state().companies[daughter.index()].ledger.cash(),
    );
    let product = iron(&game);
    let (catalog, today) = (game.catalog().clone(), game.state().date);
    crate::supply::transfer(
        game.state_mut(),
        &catalog,
        (plant, theirs),
        product,
        40.0,
        today,
    )
    .unwrap();
    assert!((stock(&game, theirs) - 40.0).abs() < 1e-9);
    let after = (
        game.state().companies[me.index()].ledger.cash(),
        game.state().companies[daughter.index()].ledger.cash(),
    );
    assert_eq!(after.0 - before.0, usd(40.0));
    assert_eq!(before.1 - after.1, usd(40.0));
    for c in [me, daughter] {
        assert!(game.state().companies[c.index()].ledger.is_balanced());
        assert!(inventory_matches(&game, c));
    }
    // A company outside the group is no source.
    let other = competitor(&mut game);
    let stranger = warehouse(&mut game, other, "AAA", 100.0);
    let mine = warehouse(&mut game, me, "AAA", 0.0);
    assert_eq!(
        set(
            &mut game,
            mine,
            Some(stranger),
            SupplyMode::Fixed { per_day: 1.0 }
        ),
        Err(CommandError::NotOwner)
    );
    assert_eq!(
        set(
            &mut game,
            mine,
            Some(plant),
            SupplyMode::Fixed { per_day: 0.0 }
        ),
        Err(CommandError::InvalidQuantity)
    );
}

#[test]
fn an_empty_store_keeps_its_learned_demand_and_a_new_one_starts_from_the_market() {
    let mut catalog = test_support::trading();
    catalog.logistics.supply_start_share = 0.05;
    let mut game = new_game(catalog);
    let me = game.player();
    let branch = warehouse(&mut game, me, "AAA", 0.0);
    let product = iron(&game);
    let today = game.state().date;
    let offer = SaleOffer {
        demand: 12.0,
        mode: PriceMode::Fixed(usd(1_000_000.0)),
        price: usd(1_000_000.0),
        keep: 0.0,
        sold_today: 0.0,
        sold_month: 0.0,
        sold_last_month: 0.0,
        to_traders_month: 0.0,
        to_companies_month: 0.0,
    };
    game.state_mut().sites[branch.index()]
        .offers
        .insert(product, offer.clone());
    let need = |game: &crate::game::Game| {
        crate::supply::daily_demand(game.state(), game.catalog(), branch, product, today)
    };
    // Nothing sold for lack of goods: the learned demand holds.
    assert!((need(&game) - 12.0).abs() < 1e-9);
    // Without any history: a start share of the country's demand.
    game.state_mut().sites[branch.index()].offers.insert(
        product,
        SaleOffer {
            demand: 0.0,
            ..offer
        },
    );
    let country = game.state().sites[branch.index()].country;
    game.state_mut()
        .markets
        .get_mut(product)
        .get_mut(country)
        .last_month
        .demand = 3000.0;
    let share = game.catalog().logistics.supply_start_share;
    assert!((need(&game) - 100.0 * share).abs() < 1e-9);
}
