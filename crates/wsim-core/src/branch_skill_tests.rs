//! Industry experience of managers (MG5; docs/FORMELN.md MG5).

use crate::ids::{BranchId, Id};
use crate::management_tests::{cheap_furnaces, days, hire_sharp, specialist, weak_works};
use crate::production;
use crate::rng::{SimRng, Stream};

fn first_branch() -> BranchId {
    BranchId::from_index(0)
}

#[test]
fn a_new_candidate_has_a_home_industry_and_little_else() {
    let catalog = crate::catalog::test_support::management();
    let n = catalog.branches.keys().len();
    assert!(n >= 1, "the test catalog has branches");
    let b = catalog.management.branch;
    let aaa = catalog.countries.id("AAA").unwrap();
    let mut names = std::collections::BTreeSet::new();
    for i in 0..200 {
        let mut rng = SimRng::for_stream(1, Stream::Manager { id: i, day: 0 });
        let mut m = crate::management::draw(&catalog, &mut rng, aaa, &mut names);
        crate::management::assign_branches(&catalog, 1, crate::state::ManagerId(i), &mut m);
        assert_eq!(m.branch.len(), n, "a value for every branch");
        let strong = m.branch.values().filter(|&&v| v > b.other_max).count();
        assert!(
            strong <= 2,
            "a home industry and a second at most: {strong}"
        );
    }
}

#[test]
fn the_production_manager_lends_his_strength_to_the_works() {
    let (mut game, works, head) = weak_works(cheap_furnaces());
    let branch = first_branch();
    game.state_mut().sites[works.index()].branch = Some(branch);
    game.state_mut()
        .managers
        .get_mut(&head)
        .unwrap()
        .branch
        .insert(branch, 100);
    days(&mut game, 35);
    let s = game.state().sites[works.index()].branch_strength;
    assert!((s - 0.5).abs() < 1e-9, "the head has half the effect: {s}");

    let spec = hire_sharp(&mut game, specialist(works, "produktion"));
    game.state_mut()
        .managers
        .get_mut(&spec)
        .unwrap()
        .branch
        .insert(branch, 80);
    days(&mut game, 35);
    let s = game.state().sites[works.index()].branch_strength;
    assert!((s - 0.6).abs() < 1e-9, "the specialist counts: {s}");

    // The effect: fewer hours and less material than without.
    let state = game.state();
    let site = &state.sites[works.index()];
    let c = game.catalog();
    let recipe = c.recipes.get(
        site.slots[0]
            .recipe
            .unwrap_or_else(|| c.recipes.id("stahl").unwrap()),
    );
    let with = production::site_effect_unfloored(c, state, site, recipe.product);
    let mut plain = site.clone();
    plain.branch_strength = 0.0;
    let without = production::site_effect_unfloored(c, state, &plain, recipe.product);
    assert!((with.labor / without.labor - (1.0 - 0.10 * 0.6)).abs() < 1e-9);
    assert!((with.inputs / without.inputs - (1.0 - 0.03 * 0.6)).abs() < 1e-9);
}

#[test]
fn the_floors_hold_whatever_the_factors_say() {
    let mut catalog = cheap_furnaces();
    catalog.management.branch.floor_material = 0.999;
    catalog.management.branch.floor_scrap = 1.0;
    let (mut game, works, _) = weak_works(catalog);
    game.state_mut().sites[works.index()].branch_strength = 1.0;
    let state = game.state();
    let c = game.catalog();
    let site = &state.sites[works.index()];
    for slot in 0..site.slots.len() {
        let Some(r) = site.slots[slot].recipe else {
            continue;
        };
        let recipe = c.recipes.get(r);
        let e = production::slot_effect(c, state, site, &site.slots[slot], recipe.product);
        assert!(e.inputs >= 0.999 - 1e-12, "material floor: {}", e.inputs);
        let base = crate::operations::scrap_rate_for(c, state, site, &site.slots[slot], recipe);
        let raw = crate::operations::scrap_factors(c, state, site, &site.slots[slot], recipe);
        assert!(
            raw < 1.0 || base == 0.0,
            "the factors alone are below the base"
        );
    }
}
