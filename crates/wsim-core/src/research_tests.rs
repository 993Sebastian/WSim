//! Tests for research (M9) with `test_support::research`.

use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{Catalog, SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::ids::TechnologyId;
use crate::ledger::CostType;
use crate::message::keys;
use crate::money::Money;
use crate::research;
use crate::state::{GameSettings, SiteId, StartForm};

fn new_game(catalog: Catalog) -> Game {
    let catalog = Arc::new(catalog);
    let settings = GameSettings {
        seed: 5,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: Money::from_usd(5_000_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Labor AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
    };
    Game::new(catalog, settings).unwrap()
}

fn tech(game: &Game, key: &str) -> TechnologyId {
    game.catalog().technologies.id(key).unwrap()
}

/// A research center with one laboratory at full staffing, ready after 10 days.
fn research_center(game: &mut Game) -> SiteId {
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
        size: crate::catalog::FacilitySize::Medium,
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
fn effort_rises_ahead_of_history_and_falls_for_latecomers() {
    let game = new_game(test_support::research());
    let (c, s) = (game.catalog(), game.state());
    let turbine = tech(&game, "turbine");
    let at = |y: i32| research::effort(c, s, turbine, Date::first_of_year(y)).unwrap();
    assert!((at(1902).factor - 1.0).abs() < 1e-12);
    assert!((at(1900).factor - 1.25 * 1.25).abs() < 1e-9);
    assert!((at(1907).factor - 0.9f64.powi(5)).abs() < 1e-9);
    assert_eq!(at(1950).factor, 0.2, "never below the minimum");
    assert_eq!(at(1902).points, 300.0);
    // Far ahead becomes ruinous.
    let late = tech(&game, "hochofen_2000");
    let e = research::effort(c, s, late, Date::first_of_year(1900)).unwrap();
    assert!(e.points > 1.0e12, "{e:?}");
    // Technologies of the start are not researched.
    assert!(research::effort(c, s, tech(&game, "schmelzen"), s.date).is_none());
}

#[test]
fn harder_setting_raises_the_cost_ahead() {
    let mut game = new_game(test_support::research());
    let turbine = tech(&game, "turbine");
    let date = Date::first_of_year(1900);
    let normal = research::effort(game.catalog(), game.state(), turbine, date).unwrap();
    game.state_mut().settings.research_ahead_factor = 2.0;
    let hard = research::effort(game.catalog(), game.state(), turbine, date).unwrap();
    assert!((hard.factor - normal.factor * normal.factor).abs() < 1e-9);
}

#[test]
fn research_center_acquires_a_technology() {
    let mut game = new_game(test_support::research());
    let site = research_center(&mut game);
    let turbine = tech(&game, "turbine");
    game.apply(Command::SetResearch {
        site,
        technology: Some(turbine),
    })
    .unwrap();
    let player = game.player();
    let mut done = None;
    for day in 0..400 {
        let report = game.advance(RoundLength::Day, |_| {});
        if report.messages.iter().any(|m| m.key == keys::RESEARCH_DONE) {
            done = Some(day);
            break;
        }
    }
    let day = done.expect("research finishes");
    assert!(day > 20, "needs staff and time ({day})");
    let state = game.state();
    assert!(state.knows(game.catalog(), player, turbine));
    assert_eq!(state.inventions.get(turbine).map(|d| d.year()), Some(1900));
    assert!(state.sites[site.index()].research.is_none());
    let company = &state.companies[player.index()];
    assert!(company.research.is_empty());
    assert!(company.ledger.year.by_type[&CostType::Research] < Money::ZERO);
    assert!(company.ledger.is_balanced());

    // A second company now pays the latecomer price, not the price ahead of history.
    let before = research::effort(
        game.catalog(),
        &new_game(test_support::research()).state().clone(),
        turbine,
        state.date,
    )
    .unwrap();
    let after = research::effort(game.catalog(), state, turbine, state.date).unwrap();
    assert!(after.points < before.points);
}

#[test]
fn research_needs_prerequisites_and_a_research_center() {
    let mut c = test_support::research();
    let turbine = c.technologies.id("turbine").unwrap();
    let field = c.technologies.get(turbine).field;
    c.technologies.insert(
        "dampfturbine",
        crate::catalog::Technology {
            field,
            invention_year: 1905,
            prerequisites: vec![turbine],
            research_effort: Some(100.0),
            provenance: Default::default(),
        },
    );
    let mut game = new_game(c);
    let site = research_center(&mut game);
    let next = tech(&game, "dampfturbine");
    assert_eq!(
        game.apply(Command::SetResearch {
            site,
            technology: Some(next),
        })
        .unwrap_err(),
        CommandError::NotResearchable("dampfturbine".into())
    );
    let known = tech(&game, "schmelzen");
    assert!(matches!(
        game.apply(Command::SetResearch {
            site,
            technology: Some(known),
        }),
        Err(CommandError::NotResearchable(_))
    ));
    game.apply(Command::FoundSite {
        country: game.catalog().countries.id("AAA").unwrap(),
        kind: SiteType::Warehouse,
    })
    .unwrap();
    let warehouse = SiteId(1);
    assert!(matches!(
        game.apply(Command::SetResearch {
            site: warehouse,
            technology: Some(tech(&game, "turbine")),
        }),
        Err(CommandError::WrongSiteType { .. })
    ));
}

#[test]
fn old_technologies_become_common_knowledge() {
    let mut game = new_game(test_support::research());
    let turbine = tech(&game, "turbine");
    let player = game.player();
    assert!(!game.state().knows(game.catalog(), player, turbine));
    let years = game.catalog().research_model.public_domain_years;
    game.state_mut().date = Date::first_of_year(1902 + years);
    assert!(game.state().knows(game.catalog(), player, turbine));
}

#[test]
fn research_survives_saving() {
    let mut game = new_game(test_support::research());
    let site = research_center(&mut game);
    let turbine = tech(&game, "turbine");
    game.apply(Command::SetResearch {
        site,
        technology: Some(turbine),
    })
    .unwrap();
    game.advance(RoundLength::Month, |_| {});
    let bytes = crate::save::encode(&game);
    let mut loaded = crate::save::decode(&bytes, game.catalog().clone())
        .unwrap()
        .game;
    assert!(!loaded.state().companies[0].research.is_empty());
    game.advance(RoundLength::Quarter, |_| {});
    loaded.advance(RoundLength::Quarter, |_| {});
    assert_eq!(game.state_hash(), loaded.state_hash());
}

/// M19: the technology tree knows each technology's status, what it opens, and how long
/// and how expensive it is with one laboratory.
#[test]
fn the_tree_shows_status_estimates_and_what_a_technology_opens() {
    let mut game = new_game(test_support::research());
    let view = |game: &Game| crate::views::research_overview(game);
    let find = |v: &crate::views::ResearchOverview, key: &str| {
        v.technologies
            .iter()
            .find(|t| t.key == key)
            .cloned()
            .unwrap()
    };
    let v = view(&game);
    assert_eq!(v.laboratory_posts, 10.0);
    let smelting = find(&v, "schmelzen");
    assert_eq!(smelting.status, "bekannt");
    assert_eq!(smelting.leads_to, vec!["hochofen_2000", "turbine"]);
    // Smelting opens the furnace and its recipe, which makes iron from ore.
    assert!(smelting.facilities.iter().any(|f| f.key == "ofen"));
    let recipe = smelting
        .recipes
        .iter()
        .find(|r| r.key == "eisen_schmelzen")
        .unwrap();
    assert_eq!(recipe.product, "eisen");
    assert!((recipe.output_per_day - 50.0).abs() < 1e-9);
    assert_eq!(recipe.inputs_per_day, vec![("erz".to_owned(), 100.0)]);
    assert_eq!(smelting.products, vec!["eisen"]);

    let turbine = find(&v, "turbine");
    assert_eq!(turbine.status, "erforschbar");
    let remaining = turbine.remaining.unwrap();
    assert!((remaining - turbine.needed.unwrap()).abs() < 1e-9);
    // One laboratory: 10 researchers at the efficiency of the home country.
    let lab = turbine.one_lab.clone().unwrap();
    assert_eq!(lab.country, "AAA");
    assert!((lab.days - remaining / lab.points_per_day).abs() < 1e-9);
    assert!(lab.cost_usd > 0.0);
    assert_eq!(turbine.days, None);
    let late = find(&v, "hochofen_2000");
    assert_eq!(late.status, "erforschbar");

    let site = research_center(&mut game);
    game.apply(Command::SetResearch {
        site,
        technology: Some(tech(&game, "turbine")),
    })
    .unwrap();
    for _ in 0..12 {
        game.advance(RoundLength::Day, |_| {});
    }
    let turbine = find(&view(&game), "turbine");
    assert_eq!(turbine.status, "in_arbeit");
    assert!(turbine.points_per_day > 0.0);
    let days = turbine.days.unwrap();
    assert!((days - turbine.remaining.unwrap() / turbine.points_per_day).abs() < 1e-9);
}
