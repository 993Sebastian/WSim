//! Tests of the learning curve (docs/FORMELN.md „Lernkurve“, BP1).

use crate::catalog::{Catalog, LearningModel, test_support};
use crate::command::Command;
use crate::facility_tests::{mine, new_game, works};
use crate::trade_tests::days;

fn with_learning(rate: f64) -> Catalog {
    let mut catalog = test_support::production();
    catalog.production_model.learning = LearningModel {
        enabled: true,
        rates: Vec::new(),
        default_rate: rate,
        minimum: 0.5,
        forget_per_month: 0.03,
        layoff_loss: 0.1,
        transfer: 0.5,
        established_years: 10.0,
    };
    catalog
}

#[test]
fn eight_times_the_quantity_cuts_the_hours_by_the_rate_cubed() {
    let catalog = with_learning(0.85);
    let recipe = catalog
        .recipes
        .get(catalog.recipes.id("eisen_schmelzen").unwrap());
    let n0 = crate::learning::reference_units(&catalog, recipe);
    assert!(n0 > 0.0);
    let f = |n| crate::learning::factor_of(&catalog, recipe, n);
    assert_eq!(f(0.0), 1.0, "a new site works at the data hours");
    assert_eq!(f(n0), 1.0);
    assert!((f(8.0 * n0) - 0.85f64.powi(3)).abs() < 1e-9);
    assert_eq!(f(1e9 * n0), 0.5, "never below the minimum");
    let off = test_support::production();
    let recipe = off.recipes.get(off.recipes.id("eisen_schmelzen").unwrap());
    assert_eq!(crate::learning::factor_of(&off, recipe, 8.0 * n0), 1.0);
}

#[test]
fn an_experienced_works_makes_cheaper_than_a_new_one() {
    let mut game = new_game(with_learning(0.85));
    let c = game.catalog().clone();
    let iron = c.recipes.id("eisen_schmelzen");
    let sites = [works(&mut game, 1), works(&mut game, 1)];
    for site in sites {
        game.apply(Command::SetProduction {
            site,
            slot: 0,
            recipe: iron,
            utilization: 1.0,
        })
        .unwrap();
    }
    // Five years ahead: eight times a year's output.
    let recipe = c.recipes.get(iron.unwrap());
    let n0 = crate::learning::reference_units(&c, recipe);
    game.state_mut().sites[sites[0].index()]
        .experience
        .entry(recipe.product)
        .or_default()
        .units = 8.0 * n0;
    let cost = |game: &crate::game::Game, site: crate::state::SiteId| {
        crate::production::unit_costs(game.catalog(), game.state(), site)
            .into_iter()
            .find(|u| u.product == recipe.product)
            .unwrap()
    };
    let (old, new) = (cost(&game, sites[0]), cost(&game, sites[1]));
    assert!((old.labor / new.labor - 0.85f64.powi(3)).abs() < 1e-6);
    assert!(old.total() < new.total());
    assert!(old.material == new.material, "material does not learn");

    // Making counts; a month without output forgets.
    let ore = c.products.id("erz").unwrap();
    game.state_mut().sites[sites[1].index()]
        .inventory
        .entry(ore)
        .or_default()
        .add(
            10_000.0,
            crate::money::Money::from_usd(10_000.0).unwrap(),
            50.0,
        );
    days(&mut game, 40);
    let made = crate::learning::units(&game.state().sites[sites[1].index()], recipe.product);
    assert!(made > 0.0, "the new works gains experience");
    game.apply(Command::SetProduction {
        site: sites[1],
        slot: 0,
        recipe: None,
        utilization: 0.0,
    })
    .unwrap();
    days(&mut game, 62);
    let left = crate::learning::units(&game.state().sites[sites[1].index()], recipe.product);
    assert!(left < made, "idle months forget: {left} < {made}");
}

#[test]
fn layoffs_of_more_than_a_third_cost_experience_once_a_month() {
    let catalog = with_learning(0.85);
    let mut game = new_game(catalog.clone());
    let site = mine(&mut game);
    let product = catalog.products.id("erz").unwrap();
    let s = &mut game.state_mut().sites[site.index()];
    s.experience.entry(product).or_default().units = 1000.0;
    crate::learning::laid_off(&catalog, s, 30.0, 25.0);
    assert_eq!(crate::learning::units(s, product), 1000.0);
    crate::learning::laid_off(&catalog, s, 30.0, 10.0);
    assert!((crate::learning::units(s, product) - 900.0).abs() < 1e-9);
    crate::learning::laid_off(&catalog, s, 30.0, 10.0);
    assert!((crate::learning::units(s, product) - 900.0).abs() < 1e-9);
}
