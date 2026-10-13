//! Scenario tests for works and their branches (FW1): buying a unit for a product of the
//! works' branch, the branch of a works, and changing a unit to another product.

use std::sync::Arc;

use crate::calendar::RoundLength;
use crate::catalog::{
    Branch, Catalog, FacilitySize, Product, ProductKind, Recipe, SiteType, Usage, test_support,
};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::ledger::CostType;
use crate::money::Money;
use crate::product_research;
use crate::state::{GameSettings, SiteId, StartForm};

fn usd(v: f64) -> Money {
    Money::from_usd(v).unwrap()
}

/// The production catalog with a second branch (chemistry, soda) and a second product of
/// the mining branch (copper) on the same furnace; iron is the basic product.
fn catalog() -> Catalog {
    let mut c = test_support::production();
    let iron = c.products.id("eisen").unwrap();
    let ore = c.products.id("erz").unwrap();
    let furnace = c.facilities.id("ofen").unwrap();
    let bergbau = c.branches.id("bergbau").unwrap();
    let chemie = c.branches.insert("chemie", Branch).unwrap();
    let template = c.products.get(iron).clone();
    let copper = c
        .products
        .insert(
            "kupfer",
            Product {
                kind: ProductKind::SemiFinished,
                ..template.clone()
            },
        )
        .unwrap();
    let soda = c
        .products
        .insert(
            "soda",
            Product {
                kind: ProductKind::SemiFinished,
                branch: chemie,
                usage: Usage::Industry,
                ..template
            },
        )
        .unwrap();
    let base = c
        .recipes
        .get(c.recipes.id("eisen_schmelzen").unwrap())
        .clone();
    c.recipes
        .insert(
            "kupfer_machen",
            Recipe {
                product: copper,
                technology: None,
                inputs: vec![(ore, 1.0)],
                ..base.clone()
            },
        )
        .unwrap();
    c.recipes
        .insert(
            "soda_machen",
            Recipe {
                product: soda,
                technology: None,
                facility: furnace,
                inputs: vec![(ore, 1.0)],
                ..base
            },
        )
        .unwrap();
    assert_eq!(c.products.get(copper).branch, bergbau);
    c.basic_products = vec![iron];
    c.basic_technologies = product_research::basic_technologies(&c);
    c
}

fn game(research_mandatory: bool) -> Game {
    let catalog = Arc::new(catalog());
    let settings = GameSettings {
        seed: 5,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(20_000_000.0),
        start_form: StartForm::Workshop,
        company_name: "Werk AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        research_mandatory,
        found_at_start: true,
        person: Default::default(),
    };
    let mut game = Game::new(catalog, settings).unwrap();
    // Copper is learned by research in the scenarios below.
    let player = game.player();
    let copper = game.catalog().products.id("kupfer").unwrap();
    game.state_mut().companies[player.index()]
        .known_products
        .insert(copper);
    game
}

fn factory(game: &mut Game) -> SiteId {
    let c = game.catalog().clone();
    game.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: SiteType::Factory,
    })
    .unwrap();
    SiteId(u32::try_from(game.state().sites.len() - 1).unwrap())
}

fn days(game: &mut Game, n: u32) {
    for _ in 0..n {
        game.advance(RoundLength::Day, |_| {});
    }
}

fn product(game: &Game, key: &str) -> crate::ids::ProductId {
    game.catalog().products.id(key).unwrap()
}

#[test]
fn the_first_product_fixes_the_branch_of_the_works() {
    let mut game = game(true);
    let site = factory(&mut game);
    let (iron, soda, copper) = (
        product(&game, "eisen"),
        product(&game, "soda"),
        product(&game, "kupfer"),
    );
    let player = game.player();
    // Soda is not known yet.
    assert!(matches!(
        game.apply(Command::BuildForProduct {
            site,
            product: soda,
            count: 1,
            size: FacilitySize::Medium
        }),
        Err(CommandError::ProductUnknown(_))
    ));
    game.apply(Command::BuildForProduct {
        site,
        product: iron,
        count: 1,
        size: FacilitySize::Medium,
    })
    .unwrap();
    let c = game.catalog().clone();
    let branch = c.products.get(iron).branch;
    assert_eq!(game.state().sites[site.index()].branch, Some(branch));
    // The works keeps the recipe of the product it was bought for.
    let slot = game.state().sites[site.index()].slots.last().unwrap();
    assert_eq!(slot.recipe, product_research::main_recipe(&c, iron));
    // Copper is in the same branch; soda in chemistry is refused.
    game.state_mut().companies[player.index()]
        .known_products
        .insert(soda);
    assert!(matches!(
        game.apply(Command::BuildForProduct {
            site,
            product: soda,
            count: 1,
            size: FacilitySize::Medium
        }),
        Err(CommandError::OtherBranch(_))
    ));
    game.apply(Command::BuildForProduct {
        site,
        product: copper,
        count: 1,
        size: FacilitySize::Medium,
    })
    .unwrap();
}

#[test]
fn switching_keeps_the_size_and_costs_half_the_unit_at_half_the_build_time() {
    let mut game = game(true);
    let site = factory(&mut game);
    let (iron, copper) = (product(&game, "eisen"), product(&game, "kupfer"));
    game.apply(Command::BuildForProduct {
        site,
        product: iron,
        count: 2,
        size: FacilitySize::Large,
    })
    .unwrap();
    days(&mut game, 60);
    let slot = game.state().sites[site.index()].slots.len() - 1;
    let cash = game.state().companies[game.player().index()].ledger.cash();
    let today = game.state().date;
    let (assets, cost_before, expenses) = {
        let l = &game.state().companies[game.player().index()].ledger;
        (
            l.balance(crate::ledger::Account::FixedAssets),
            game.state().sites[site.index()].slots[slot].cost,
            l.year.by_type.get(&CostType::Overhead).copied(),
        )
    };
    game.apply(Command::SwitchProduct {
        site,
        slot,
        product: copper,
    })
    .unwrap();
    let s = game.state();
    let sl = &s.sites[site.index()].slots[slot];
    let c = game.catalog().clone();
    assert_eq!(sl.size, FacilitySize::Large, "size stays");
    assert_eq!(sl.count, 2, "count stays");
    assert_eq!(sl.recipe, Some(c.recipes.id("kupfer_machen").unwrap()));
    // Half the catalog investment of the size and count: 2_000_000 · 2 · 0.5 · size factor.
    let expected = usd(2_000_000.0)
        .scale(c.production_model.sizes.investment(FacilitySize::Large) * 2.0 * 0.5);
    let paid = cash - s.companies[game.player().index()].ledger.cash();
    assert_eq!(paid, expected);
    // The cost is capitalized (AKT): the book value and the depreciation basis rise by it,
    // and nothing is expensed.
    let l = &s.companies[game.player().index()].ledger;
    assert_eq!(
        l.balance(crate::ledger::Account::FixedAssets) - assets,
        expected
    );
    assert_eq!(sl.cost - cost_before, expected);
    assert_eq!(l.year.by_type.get(&CostType::Overhead).copied(), expenses);
    // Half the catalog build time of the size, counted from today.
    let full = c.production_model.sizes.build_days(FacilitySize::Large, 20);
    let until = sl.down.expect("stands still during the changeover").until;
    assert_eq!(
        until,
        today.add_days(((f64::from(full)) * 0.5).ceil() as i32)
    );
    assert!(s.companies[game.player().index()].ledger.is_balanced());
}

#[test]
fn a_unit_under_construction_cannot_be_changed() {
    let mut game = game(true);
    let site = factory(&mut game);
    let (iron, copper) = (product(&game, "eisen"), product(&game, "kupfer"));
    game.apply(Command::BuildForProduct {
        site,
        product: iron,
        count: 1,
        size: FacilitySize::Medium,
    })
    .unwrap();
    assert!(matches!(
        game.apply(Command::SwitchProduct {
            site,
            slot: 0,
            product: copper
        }),
        Err(CommandError::UnderConstruction)
    ));
}

#[test]
fn without_mandatory_research_every_branch_is_free() {
    let mut game = game(false);
    let site = factory(&mut game);
    let (iron, soda) = (product(&game, "eisen"), product(&game, "soda"));
    let player = game.player();
    game.state_mut().companies[player.index()]
        .known_products
        .insert(soda);
    game.apply(Command::BuildForProduct {
        site,
        product: iron,
        count: 1,
        size: FacilitySize::Medium,
    })
    .unwrap();
    game.apply(Command::BuildForProduct {
        site,
        product: soda,
        count: 1,
        size: FacilitySize::Medium,
    })
    .unwrap();
}
