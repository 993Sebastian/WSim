//! Tests of research per product and of manufacturing research (Q6) with the research
//! catalog: steel from iron needs the turbine (invented 1902, 300 points), cables from
//! steel are not researchable while steel does not exist in the world.

use std::sync::Arc;

use crate::calendar::RoundLength;
use crate::catalog::{Catalog, FacilitySize, Provenance, Recipe, SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::development;
use crate::game::Game;
use crate::ids::{Id, ProductId};
use crate::message::keys;
use crate::money::Money;
use crate::product_research;
use crate::state::{GameSettings, SiteId, StartForm};

pub(crate) fn catalog() -> Catalog {
    let mut c = test_support::research();
    let iron = c.products.id("eisen").unwrap();
    let template = c.products.get(iron).clone();
    let steel = c.products.insert("stahl", template.clone()).unwrap();
    let cable = c.products.insert("kabel", template).unwrap();
    let base = c
        .recipes
        .get(c.recipes.id("eisen_schmelzen").unwrap())
        .clone();
    let turbine = c.technologies.id("turbine").unwrap();
    c.recipes
        .insert(
            "stahl_machen",
            Recipe {
                product: steel,
                technology: Some(turbine),
                inputs: vec![(iron, 1.0)],
                provenance: Provenance::default(),
                ..base.clone()
            },
        )
        .unwrap();
    c.recipes
        .insert(
            "kabel_machen",
            Recipe {
                product: cable,
                technology: Some(turbine),
                inputs: vec![(steel, 1.0)],
                ..base
            },
        )
        .unwrap();
    c.research_model.development.levels = 5;
    c.research_model.development.process_output_max = 0.25;
    c.research_model.development.process_inputs_max = 0.15;
    c.research_model.development.base_effort = 1000.0;
    c
}

pub(crate) fn new_game(catalog: Catalog) -> Game {
    let catalog = Arc::new(catalog);
    let settings = GameSettings {
        seed: 7,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: Money::from_usd(20_000_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Forschung AG".into(),
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

fn product(game: &Game, key: &str) -> ProductId {
    game.catalog().products.id(key).unwrap()
}

/// A research center with one laboratory working at full load.
pub(crate) fn research_center(game: &mut Game) -> SiteId {
    let c = game.catalog().clone();
    game.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: SiteType::ResearchCenter,
    })
    .unwrap();
    let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.apply(Command::BuildFacility {
        site,
        facility: c.facilities.id("labor").unwrap(),
        count: 1,
        size: FacilitySize::Medium,
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site,
        slot: 0,
        recipe: None,
        utilization: 1.0,
    })
    .unwrap();
    site
}

#[test]
fn a_product_is_researched_through_its_missing_technologies() {
    let mut game = new_game(catalog());
    let lab = research_center(&mut game);
    let player = game.player();
    let (steel, cable, iron) = (
        product(&game, "stahl"),
        product(&game, "kabel"),
        product(&game, "eisen"),
    );
    let (c, s) = (game.catalog().clone(), game.state());
    assert!(product_research::known(&c, s, player, iron));
    assert!(!product_research::known(&c, s, player, steel));
    // Steel's input exists; the cable's input (steel) does not yet.
    assert!(product_research::can_research(&c, s, player, steel));
    assert_eq!(product_research::missing_inputs(&c, s, cable), vec![steel]);
    assert!(!product_research::can_research(&c, s, player, cable));
    let (needed, collected) = product_research::effort(&c, s, player, steel, s.date);
    assert!(needed > 300.0 && collected == 0.0, "{needed} {collected}");
    assert!(matches!(
        game.apply(Command::SetProductResearch {
            site: lab,
            product: Some(cable),
        }),
        Err(CommandError::NotResearchableProduct(_))
    ));

    game.apply(Command::SetProductResearch {
        site: lab,
        product: Some(steel),
    })
    .unwrap();
    let turbine = c.technologies.id("turbine").unwrap();
    assert_eq!(game.state().sites[lab.index()].research, Some(turbine));
    let mut done = false;
    for _ in 0..800 {
        let report = game.advance(RoundLength::Day, |_| {});
        done |= report
            .messages
            .iter()
            .any(|m| m.key == keys::PRODUCT_RESEARCHED);
        if done {
            break;
        }
    }
    assert!(done, "steel gets researched");
    let s = game.state();
    assert!(product_research::known(&c, s, player, steel));
    assert_eq!(s.sites[lab.index()].research_product, None);
    // Now the player made steel exist early: cables are researchable.
    assert!(
        product_research::can_research(&c, s, player, cable)
            || product_research::known(&c, s, player, cable)
    );
}

#[test]
fn manufacturing_research_brings_output_and_saves_inputs() {
    let mut game = new_game(catalog());
    let player = game.player();
    let iron = product(&game, "eisen");
    let c = game.catalog().clone();
    let before = development::effect(&c, game.state(), player, iron);
    assert_eq!(before.output, 1.0);
    development::add_process_points(game.state_mut(), player, iron, 2000.0);
    let s = game.state();
    assert_eq!(development::process_share(s, player, iron), 1.0);
    let maturity = 1.0 - (-2.0f64).exp();
    let e = development::effect(&c, s, player, iron);
    assert!((e.output - (1.0 + 0.25 * maturity)).abs() < 1e-12);
    assert!((e.inputs - (1.0 - 0.15 * maturity)).abs() < 1e-12);
    // Development alone does not touch output.
    assert!((e.quality - 0.0).abs() < 1e-12);
    // After loading the leader is derived again.
    let loaded = crate::save::decode(&crate::save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(
        development::process_leader_points(loaded.state(), iron),
        2000.0
    );
}

#[test]
fn a_center_splits_between_development_and_manufacturing() {
    let mut game = new_game(catalog());
    let lab = research_center(&mut game);
    let player = game.player();
    let iron = product(&game, "eisen");
    assert!(matches!(
        game.apply(Command::SetResearchShares {
            site: lab,
            shares: vec![(iron, 0.6)],
            process: vec![(iron, 0.6)],
        }),
        Err(CommandError::InvalidShare)
    ));
    game.apply(Command::SetResearchShares {
        site: lab,
        shares: vec![(iron, 0.5)],
        process: vec![(iron, 0.5)],
    })
    .unwrap();
    for _ in 0..60 {
        game.advance(RoundLength::Day, |_| {});
    }
    let s = game.state();
    let d = development::points(s, player, iron);
    let p = development::process_points(s, player, iron);
    assert!(d > 0.0 && p > 0.0, "{d} {p}");
    assert!(
        (d - p).abs() < 1e-6 * d,
        "equal shares, equal points: {d} {p}"
    );
}

/// Mandatory research (FW1): nails from iron need no technology, iron is the basic
/// product of the branch.
fn mandatory_catalog() -> Catalog {
    let mut c = catalog();
    let iron = c.products.id("eisen").unwrap();
    let template = c.products.get(iron).clone();
    let nails = c.products.insert("naegel", template).unwrap();
    let base = c
        .recipes
        .get(c.recipes.id("eisen_schmelzen").unwrap())
        .clone();
    c.recipes
        .insert(
            "naegel_machen",
            Recipe {
                product: nails,
                technology: None,
                inputs: vec![(iron, 1.0)],
                ..base
            },
        )
        .unwrap();
    let field = c.research_model.development.fields.len();
    let spec = crate::ids::SpecializationId::from_index(0);
    c.research_model.development.fields = vec![Some(spec); c.branches.len().max(field)];
    c.basic_products = vec![iron];
    c.basic_technologies = product_research::basic_technologies(&c);
    c
}

fn mandatory_game() -> Game {
    let catalog = Arc::new(mandatory_catalog());
    let mut settings = new_game(catalog.as_ref().clone()).state().settings.clone();
    settings.research_mandatory = true;
    Game::new(catalog, settings).unwrap()
}

#[test]
fn mandatory_research_starts_with_the_basic_products_only() {
    let game = mandatory_game();
    let player = game.player();
    let (iron, nails, steel) = (
        product(&game, "eisen"),
        product(&game, "naegel"),
        product(&game, "stahl"),
    );
    let (c, s) = (game.catalog().clone(), game.state());
    assert!(product_research::known(&c, s, player, iron));
    assert!(!product_research::known(&c, s, player, nails));
    assert!(!product_research::known(&c, s, player, steel));
    assert!(product_research::can_research(&c, s, player, nails));
    // What the start set makes stays known.
    for r in s
        .sites
        .iter()
        .filter(|x| x.held_by(player))
        .flat_map(|x| &x.slots)
        .filter_map(|sl| sl.recipe)
    {
        assert!(s.knows_product(&c, player, c.recipes.get(r).product));
    }
    // Without mandatory research nails are free as before.
    let free = new_game(mandatory_catalog());
    let p = free.player();
    assert!(product_research::known(&c, free.state(), p, nails));
}

#[test]
fn a_product_without_technology_is_learned_in_the_research_center() {
    let mut game = mandatory_game();
    let lab = research_center(&mut game);
    let player = game.player();
    let nails = product(&game, "naegel");
    let c = game.catalog().clone();
    let (needed, collected) =
        product_research::effort(&c, game.state(), player, nails, game.state().date);
    assert!(needed > 0.0 && collected == 0.0, "{needed} {collected}");
    game.apply(Command::SetProductResearch {
        site: lab,
        product: Some(nails),
    })
    .unwrap();
    assert_eq!(game.state().sites[lab.index()].research, None);
    let cash = game.state().companies[player.index()].ledger.cash();
    let mut done = false;
    for _ in 0..2000 {
        let report = game.advance(RoundLength::Day, |_| {});
        done |= report
            .messages
            .iter()
            .any(|m| m.key == keys::PRODUCT_RESEARCHED);
        if done {
            break;
        }
    }
    let co = &game.state().companies[player.index()];
    let site = &game.state().sites[lab.index()];
    assert!(
        done,
        "nails get learned: needed {needed}, points {:?}, research {:?}/{:?}, staff {:?}",
        co.product_points, site.research, site.research_product, site.workforce
    );
    let s = game.state();
    assert!(product_research::known(&c, s, player, nails));
    assert!(s.companies[player.index()].ledger.cash() < cash);
    assert!(s.companies[player.index()].ledger.is_balanced());
    // The journal replays to the same knowledge.
    let replay = Game::replay(game.catalog().clone(), s.settings.clone(), game.journal()).unwrap();
    assert_eq!(
        replay.state().companies[player.index()].known_products,
        s.companies[player.index()].known_products
    );
}

/// A technology from before 1900 has no effort in the data; under mandatory research it
/// still has to be learned and must not hold the product back for good.
#[test]
fn a_technology_without_effort_is_learned_too() {
    let mut c = mandatory_catalog();
    let turbine = c.technologies.id("turbine").unwrap();
    c.technologies.get_mut(turbine).research_effort = None;
    let catalog = Arc::new(c);
    let mut settings = new_game(catalog.as_ref().clone()).state().settings.clone();
    settings.research_mandatory = true;
    let mut game = Game::new(catalog, settings).unwrap();
    let lab = research_center(&mut game);
    let player = game.player();
    let steel = product(&game, "stahl");
    let c = game.catalog().clone();
    assert!(!product_research::known(&c, game.state(), player, steel));
    game.apply(Command::SetProductResearch {
        site: lab,
        product: Some(steel),
    })
    .unwrap();
    let mut done = false;
    for _ in 0..4000 {
        let report = game.advance(RoundLength::Day, |_| {});
        done |= report
            .messages
            .iter()
            .any(|m| m.key == keys::PRODUCT_RESEARCHED);
        if done {
            break;
        }
    }
    let s = game.state();
    assert!(
        done,
        "steel gets learned: points {:?}, techs {:?}, research {:?}",
        s.companies[player.index()].product_points,
        s.companies[player.index()].research,
        s.sites[lab.index()].research
    );
    assert!(s.knows_product(&c, player, steel));
    assert!(s.knows(&c, player, turbine));
    // The learned product is listed under development as an own product, though no works
    // makes it yet.
    let overview = crate::views::research_overview(&game);
    let d = overview
        .developments
        .iter()
        .find(|d| d.product == "stahl")
        .expect("a learned product can be developed");
    assert!(d.own && d.next.is_some());
}
