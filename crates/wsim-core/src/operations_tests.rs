//! Tests of scrap, accidents and corruption (docs/FORMELN.md „BP4“).

use crate::catalog::{Catalog, OperationsModel, test_support};
use crate::command::Command;
use crate::facility_tests::{mine, new_game};
use crate::time_series::TimeSeries;
use crate::trade_tests::days;

fn with_operations(scrap: f64, accidents: f64, corruption: f64) -> Catalog {
    let mut catalog = test_support::production();
    catalog.production_model.operations = OperationsModel {
        enabled: true,
        scrap_base: [scrap; 5],
        scrap_min: 0.0,
        scrap_max: 0.5,
        scrap_training: 0.6,
        scrap_development: 0.8,
        accidents: Some(TimeSeries::new(vec![(1900, accidents), (2100, accidents)]).unwrap()),
        accident_hours: 100.0,
        accident_stop_share: 1.0,
        corruption: (corruption, 0.0, 0.0),
        corruption_build_time: 0.5,
        corruption_fee: 0.05,
        demand_chance: 0.0,
        demand_share: 0.0,
        rule_of_law: (1.0, 0.0, 0.0),
        expropriation: 0.0,
        make_or_buy_tolerance: 0.1,
    };
    catalog
}

#[test]
fn scrap_falls_with_training_but_spares_extraction() {
    let catalog = with_operations(0.1, 0.0, 0.0);
    let mut game = new_game(catalog);
    let site = mine(&mut game);
    let c = game.catalog().clone();
    let mine_recipe = c
        .recipes
        .get(game.state().sites[site.index()].slots[0].recipe.unwrap());
    let state = game.state();
    let mut s = state.sites[site.index()].clone();
    assert_eq!(
        crate::operations::scrap_rate(&c, state, &s, 0, mine_recipe),
        0.0,
        "extraction has no scrap"
    );
    let smelting = c.recipes.get(c.recipes.id("eisen_schmelzen").unwrap());
    s.slots[0].condition = 1.0;
    s.slots[0].automation = 0.0;
    s.experience.clear();
    // New facility, untrained staff, undeveloped product: the base.
    let new = crate::operations::scrap_rate(&c, state, &s, 0, smelting);
    assert!((new - 0.1).abs() < 1e-9, "scrap {new}");
    // Fully trained staff take `schulung` of it away.
    s.training = 1.0;
    let trained = crate::operations::scrap_rate(&c, state, &s, 0, smelting);
    assert!((trained - 0.1 * 0.4).abs() < 1e-9, "scrap {trained}");
}

#[test]
fn accidents_cost_and_stop_a_facility() {
    let mut game = new_game(with_operations(0.0, 100_000.0, 0.0));
    let me = game.player();
    let site = mine(&mut game);
    days(&mut game, 120);
    let s = &game.state().sites[site.index()];
    assert!(s.accidents > 0, "a site with workers has accidents");
    assert!(game.state().company(me).unwrap().ledger.is_balanced());
}

#[test]
fn corruption_makes_building_slow_and_dear() {
    let build = |corruption: f64| {
        let mut game = new_game(with_operations(0.0, 0.0, corruption));
        let me = game.player();
        let c = game.catalog().clone();
        game.apply(Command::FoundSite {
            country: c.countries.id("AAA").unwrap(),
            kind: crate::catalog::SiteType::Factory,
        })
        .unwrap();
        let site = crate::state::SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
        let cash = game.state().company(me).unwrap().ledger.cash();
        game.apply(Command::BuildFacility {
            site,
            facility: c.facilities.id("ofen").unwrap(),
            count: 1,
            size: crate::catalog::FacilitySize::Medium,
        })
        .unwrap();
        let s = &game.state().sites[site.index()];
        let days = game.state().date.days_until(s.slots[0].ready);
        (days, cash - game.state().company(me).unwrap().ledger.cash())
    };
    let (clean_days, clean_cost) = build(0.0);
    let (corrupt_days, corrupt_cost) = build(1.0);
    assert!(corrupt_days > clean_days, "{corrupt_days} > {clean_days}");
    assert!(corrupt_cost > clean_cost, "a permit fee on top");
}

#[test]
fn development_and_stability_set_corruption_and_rule_of_law() {
    let m = OperationsModel {
        enabled: true,
        corruption: (0.6, 0.6, 0.4),
        rule_of_law: (0.1, 0.5, 0.4),
        ..OperationsModel::default()
    };
    assert!(m.corruption(1.0, 1.0) < 1e-9);
    assert!((m.corruption(0.0, 0.5) - 0.8).abs() < 1e-9);
    assert!((m.rule_of_law(1.0, 1.0) - 1.0).abs() < 1e-9);
    assert!(m.rule_of_law(0.0, 0.0) < m.rule_of_law(0.5, 0.5));
}
