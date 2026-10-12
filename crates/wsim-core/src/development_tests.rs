//! Scenario tests for the development of products (Q1, before M37) with the small chain of
//! `test_support::research` (ore → iron) and a laboratory.

use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{Catalog, DevelopmentModel, FacilitySize, SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::development::{self, Effect};
use crate::game::Game;
use crate::ids::{Id, ProductId};
use crate::ledger::CostType;
use crate::message::{Param, keys};
use crate::money::Money;
use crate::production;
use crate::save;
use crate::state::{GameSettings, SiteId, StartForm};

/// The research catalog with the development model of the data, a small base effort
/// (1 000 points) and the mining branch researched by metallurgists.
fn catalog() -> Catalog {
    let mut c = test_support::research();
    let metal = c.specializations.id("metall").unwrap();
    let mining = c.branches.id("bergbau").unwrap();
    let mut fields = vec![None; c.branches.len()];
    fields[mining.index()] = Some(metal);
    c.research_model.development = DevelopmentModel {
        process_output_max: 0.0,
        process_inputs_max: 0.0,
        levels: 5,
        quality_max: 30.0,
        labor_max: 0.2,
        inputs_max: 0.15,
        effort_share: 0.2,
        effort_growth: 1.6,
        base_effort: 1000.0,
        fields,
    };
    c
}

fn new_game(catalog: Catalog) -> Game {
    let catalog = Arc::new(catalog);
    let settings = GameSettings {
        seed: 7,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: Money::from_usd(20_000_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Entwicklung AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: true,
        person: Default::default(),
    };
    Game::new(catalog, settings).unwrap()
}

fn product(game: &Game, key: &str) -> ProductId {
    game.catalog().products.id(key).unwrap()
}

fn days(game: &mut Game, n: u32) {
    for _ in 0..n {
        game.advance(RoundLength::Day, |_| {});
    }
}

/// A research center with one laboratory, ready after 10 days.
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

/// Mine and furnace; the furnace gets 1 000 t of ore once the mine runs.
fn chain(game: &mut Game) -> SiteId {
    let c = game.catalog().clone();
    let aaa = c.countries.id("AAA").unwrap();
    game.apply(Command::FoundSite {
        country: aaa,
        kind: SiteType::Extraction,
    })
    .unwrap();
    let mine = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.apply(Command::BuildFacility {
        site: mine,
        facility: c.facilities.id("mine").unwrap(),
        count: 1,
        size: FacilitySize::Medium,
    })
    .unwrap();
    game.apply(Command::DevelopDeposit {
        site: mine,
        deposit: c.deposits.id("grube").unwrap(),
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site: mine,
        slot: 0,
        recipe: c.recipes.id("erz_abbau"),
        utilization: 1.0,
    })
    .unwrap();
    game.apply(Command::FoundSite {
        country: aaa,
        kind: SiteType::Factory,
    })
    .unwrap();
    let works = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.apply(Command::BuildFacility {
        site: works,
        facility: c.facilities.id("ofen").unwrap(),
        count: 1,
        size: FacilitySize::Medium,
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site: works,
        slot: 0,
        recipe: c.recipes.id("eisen_schmelzen"),
        utilization: 1.0,
    })
    .unwrap();
    days(game, 40);
    game.apply(Command::TransferGoods {
        from: mine,
        to: works,
        product: c.products.id("erz").unwrap(),
        quantity: 1000.0,
    })
    .unwrap();
    works
}

#[test]
fn the_effort_grows_per_level_and_falls_for_latecomers() {
    let mut game = new_game(catalog());
    let (iron, ore) = (product(&game, "eisen"), product(&game, "erz"));
    let player = game.player();
    let today = game.state().date;
    let effort = |game: &Game, p, date| {
        development::next_effort(game.catalog(), game.state(), player, p, date).unwrap()
    };
    // Base 1 000 points (smelting needs no research): level 1 costs a fifth.
    assert_eq!(effort(&game, iron, today), (1, 200.0));
    // Ore has no technology: its branch names the field (metallurgists).
    let c = game.catalog();
    let (field, base) = development::basis(c, ore).unwrap();
    assert_eq!(c.specializations.key(field), "metall");
    assert_eq!(base, 1000.0);
    // Products without a recipe cannot be developed.
    let bread = c.products.id("brot").unwrap();
    assert!(development::basis(c, bread).is_none());
    assert!(!development::can_develop(c, game.state(), player, bread));

    game.state_mut().companies[player.index()]
        .development
        .levels
        .insert(iron, 2);
    let (next, points) = effort(&game, iron, today);
    assert_eq!(next, 3);
    assert!((points - 1000.0 * 0.2 * 1.6 * 1.6).abs() < 1e-9);
    // Someone reached level 3 in 1900: five years later it costs 0.9^5 of that.
    *game.state_mut().developments.get_mut(iron) = vec![today, today, today];
    let later = Date::first_of_year(1905);
    let (_, cheaper) = effort(&game, iron, later);
    assert!(
        (cheaper - points * 0.9f64.powi(5)).abs() < 1e-6,
        "{cheaper}"
    );
    // After the last generation research goes on (Q1), a step as long as the last one.
    game.state_mut().companies[player.index()]
        .development
        .levels
        .insert(iron, 5);
    assert!(development::next_effort(game.catalog(), game.state(), player, iron, today).is_none());
    assert!(development::can_develop(
        game.catalog(),
        game.state(),
        player,
        iron
    ));
    let step = development::step_effort(game.catalog(), game.state(), player, iron, today);
    assert!((step.unwrap() - 1000.0 * 0.2 * 1.6f64.powi(4)).abs() < 1e-6);
}

/// The level is the points against the leader's (Q1): nothing spreads, the leader has
/// 100 %, and the savings grow with how far the product is developed at all.
#[test]
fn the_level_follows_the_points_against_the_leader() {
    let mut game = new_game(catalog());
    let iron = product(&game, "eisen");
    let player = game.player();
    // A second company to compete with.
    let rival = crate::state::CompanyId(u32::try_from(game.state().companies.len()).unwrap());
    let mut other = game.state().companies[player.index()].clone();
    other.name = "Andere AG".into();
    game.state_mut().companies.push(other);
    let (c, s) = (game.catalog(), game.state());
    assert_eq!(development::effect(c, s, player, iron), Effect::NONE);
    let state = game.state_mut();
    development::add_points(state, rival, iron, 2000.0);
    development::add_points(state, player, iron, 500.0);
    let (c, s) = (game.catalog(), game.state());
    assert_eq!(development::leader_points(s, iron), 2000.0);
    assert_eq!(development::share(s, rival, iron), 1.0);
    assert_eq!(development::share(s, player, iron), 0.25);
    let maturity = 1.0 - (-2.0f64).exp();
    assert!((development::maturity(c, s, iron) - maturity).abs() < 1e-12);
    let e = development::effect(c, s, player, iron);
    assert!((e.quality - 7.5).abs() < 1e-9);
    assert!((e.labor - (1.0 - 0.2 * 0.25 * maturity)).abs() < 1e-12);
    assert!((e.inputs - (1.0 - 0.15 * 0.25 * maturity)).abs() < 1e-12);
    // Generations only mark thresholds; the effect does not depend on them.
    assert_eq!(development::level(c, s, player, iron), 0);
    // Overtaking the rival makes the player the leader; the rival falls back.
    development::add_points(game.state_mut(), player, iron, 3500.0);
    let s = game.state();
    assert_eq!(development::share(s, player, iron), 1.0);
    assert_eq!(development::share(s, rival, iron), 0.5);
    // After loading the leader is derived again.
    let loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(development::leader_points(loaded.state(), iron), 4000.0);
}

/// A research center splits its researchers among products and a technology (Q1).
#[test]
fn a_research_center_splits_its_researchers() {
    let mut game = new_game(catalog());
    let lab = research_center(&mut game);
    let (iron, ore) = (product(&game, "eisen"), product(&game, "erz"));
    let player = game.player();
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    game.apply(Command::SetResearch {
        site: lab,
        technology: Some(turbine),
    })
    .unwrap();
    assert_eq!(
        game.apply(Command::SetResearchShares {
            site: lab,
            shares: vec![(iron, 0.7), (ore, 0.5)],
            process: Vec::new(),
        }),
        Err(CommandError::InvalidShare)
    );
    game.apply(Command::SetResearchShares {
        site: lab,
        shares: vec![(iron, 0.5), (ore, 0.25)],
        process: Vec::new(),
    })
    .unwrap();
    days(&mut game, 60);
    let s = game.state();
    let company = &s.companies[player.index()];
    let (i, o) = (
        company.development.total(iron),
        company.development.total(ore),
    );
    let t = company.research.get(&turbine).copied().unwrap_or(0.0);
    assert!(i > 0.0 && o > 0.0 && t > 0.0, "{i} {o} {t}");
    assert!((i / o - 2.0).abs() < 1e-9, "{i} {o}");
    assert!((t / o - 1.0).abs() < 0.2, "{t} {o}");
    assert_eq!(s.sites[lab.index()].research, Some(turbine));
}

#[test]
fn a_research_center_develops_a_product_level_by_level() {
    let mut game = new_game(catalog());
    let lab = research_center(&mut game);
    let iron = product(&game, "eisen");
    let player = game.player();
    game.apply(Command::SetDevelopment {
        site: lab,
        product: Some(iron),
    })
    .unwrap();
    let mut reached = Vec::new();
    for day in 0..400 {
        let report = game.advance(RoundLength::Day, |_| {});
        for m in &report.messages {
            if m.key == keys::DEVELOPMENT_DONE {
                reached.push(day);
            }
        }
        if reached.len() == 2 {
            break;
        }
    }
    assert_eq!(reached.len(), 2, "two levels within the time");
    // Level 2 takes 1.6 times as long as level 1 once staffed.
    assert!(
        reached[0] > 10,
        "needs a laboratory and staff ({reached:?})"
    );
    let state = game.state();
    let company = &state.companies[player.index()];
    assert_eq!(company.development.level(iron), 2);
    // All points count for the level; the player alone researches iron: 100 %.
    assert!(company.development.total(iron) >= 200.0 + 320.0);
    assert_eq!(development::share(state, player, iron), 1.0);
    // The world remembers who was first, and the laboratory goes on with level 3.
    assert_eq!(state.developments.get(iron).len(), 2);
    assert_eq!(state.sites[lab.index()].development, Some(iron));
    assert!(company.ledger.year.by_type[&CostType::Research] < Money::ZERO);
    assert!(company.ledger.is_balanced());

    // A technology replaces the development project.
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    game.apply(Command::SetResearch {
        site: lab,
        technology: Some(turbine),
    })
    .unwrap();
    assert_eq!(game.state().sites[lab.index()].development, None);
}

/// Iron as an end product with model names (B1).
fn named_catalog() -> Catalog {
    let mut c = catalog();
    let iron = c.products.id("eisen").unwrap();
    c.products.get_mut(iron).kind = crate::catalog::ProductKind::EndProduct;
    let group = c.products.get(iron).goods_group;
    let mut style_of_group = vec![None; c.goods_groups.len()];
    style_of_group[group.index()] = Some(0);
    c.product_naming = crate::catalog::ProductNaming {
        house_brand: 0.5,
        excluded: Vec::new(),
        styles: vec![crate::catalog::NamingStyle {
            key: "technik".into(),
            stems: vec!["Arvon".into()],
            patterns: vec![crate::catalog::NamePattern {
                text: "{stamm} {zahl}".into(),
                from: None,
                until: None,
            }],
            numbers: vec![2, 3, 4],
            letters: Vec::new(),
            additions: Vec::new(),
            successors: vec!["II".into()],
        }],
        style_of_group,
    };
    c
}

#[test]
fn a_new_level_brings_a_successor_model() {
    let mut game = new_game(named_catalog());
    let lab = research_center(&mut game);
    let iron = product(&game, "eisen");
    let player = game.player();
    game.apply(Command::NameProduct {
        product: iron,
        name: Some("Arvon 2".into()),
    })
    .unwrap();
    game.apply(Command::SetDevelopment {
        site: lab,
        product: Some(iron),
    })
    .unwrap();
    // The player gets the successor as a suggestion and keeps his name.
    let mut suggested = None;
    for _ in 0..400 {
        let report = game.advance(RoundLength::Day, |_| {});
        if let Some(m) = report
            .messages
            .iter()
            .find(|m| m.key == keys::DEVELOPMENT_SUCCESSOR)
        {
            suggested = Some(m.params.clone());
            break;
        }
    }
    let params = suggested.expect("a suggestion with the first level");
    assert!(
        params
            .iter()
            .any(|(k, v)| k == "name" && *v == Param::Text("Arvon 3".into()))
    );
    let names = |game: &Game| game.state().companies[player.index()].product_names[&iron].clone();
    assert_eq!(names(&game), "Arvon 2");
    // Another company in the player's place: the laboratory's owner renames itself.
    let state = game.state_mut();
    let mut other = state.companies[player.index()].clone();
    other.name = "Andere AG".into();
    other.product_names.clear();
    state.companies.push(other);
    state.main_company = Some(crate::state::CompanyId(1));
    for _ in 0..800 {
        game.advance(RoundLength::Day, |_| {});
        if names(&game) != "Arvon 2" {
            break;
        }
    }
    assert_eq!(names(&game), "Arvon 3");
}

#[test]
fn developed_products_need_less_labor_and_inputs_and_are_better() {
    let run = |points: f64| {
        let mut game = new_game(catalog());
        let works = chain(&mut game);
        let iron = product(&game, "eisen");
        let player = game.player();
        if points > 0.0 {
            development::add_points(game.state_mut(), player, iron, points);
        }
        let ore = product(&game, "erz");
        let before = game.state().sites[works.index()].inventory[&ore].quantity;
        days(&mut game, 3);
        let state = game.state();
        let s = &state.sites[works.index()];
        let used = before - s.inventory[&ore].quantity;
        let quality = s.inventory[&iron].quality;
        let workers: f64 = production::needed_workers(game.catalog(), state, works, state.date)
            .iter()
            .sum();
        let costs = production::unit_costs(game.catalog(), state, works)[0].clone();
        // Material per unit of ore the stock is worth (its value differs between runs).
        let ore_stock = &s.inventory[&ore];
        let ore_unit = ore_stock.value.to_usd() / ore_stock.quantity;
        (
            used,
            quality,
            workers,
            costs.material / ore_unit,
            costs.labor,
        )
    };
    let (used0, quality0, workers0, material0, labor0) = run(0.0);
    // The leader of a fully developed product (maturity 1 − e^−100 ≈ 1).
    let (used5, quality5, workers5, material5, labor5) = run(100_000.0);
    // 15 % less ore, 20 % fewer workers, 30 quality points more.
    assert!((used5 / used0 - 0.85).abs() < 1e-9, "{used0} {used5}");
    assert!(
        (workers5 / workers0 - 0.8).abs() < 1e-9,
        "{workers0} {workers5}"
    );
    assert!(
        (quality5 - quality0 - 30.0).abs() < 0.2,
        "{quality0} {quality5}"
    );
    assert!((material5 / material0 - 0.85).abs() < 1e-9);
    assert!((labor5 / labor0 - 0.8).abs() < 1e-9);
}

#[test]
fn the_command_checks_the_site_and_the_product() {
    let mut game = new_game(catalog());
    let lab = research_center(&mut game);
    let works = chain(&mut game);
    let iron = product(&game, "eisen");
    assert_eq!(
        game.apply(Command::SetDevelopment {
            site: works,
            product: Some(iron),
        }),
        Err(CommandError::WrongSiteType {
            required: SiteType::ResearchCenter
        })
    );
    // Iron made in a furnace of 2000: unknown technology, nothing to develop.
    let mut c = catalog();
    let late = c.technologies.id("hochofen_2000").unwrap();
    let smelting = c.recipes.id("eisen_schmelzen").unwrap();
    c.recipes.get_mut(smelting).technology = Some(late);
    let mut locked = new_game(c);
    let locked_lab = research_center(&mut locked);
    assert_eq!(
        locked.apply(Command::SetDevelopment {
            site: locked_lab,
            product: Some(iron),
        }),
        Err(CommandError::NotDevelopable("eisen".into()))
    );
    // Stopping always works; the project survives saving.
    game.apply(Command::SetDevelopment {
        site: lab,
        product: Some(iron),
    })
    .unwrap();
    let player = game.player();
    game.state_mut().companies[player.index()]
        .development
        .points
        .insert(iron, 12.5);
    let loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(loaded.state().sites[lab.index()].development, Some(iron));
    assert_eq!(
        loaded.state().companies[player.index()].development.points[&iron],
        12.5
    );
    // A save from before Q1 gets all its points from the progress.
    assert_eq!(
        loaded.state().companies[player.index()]
            .development
            .total(iron),
        12.5
    );
    game.apply(Command::SetDevelopment {
        site: lab,
        product: None,
    })
    .unwrap();
    assert_eq!(game.state().sites[lab.index()].development, None);
}
