//! Scenario tests for facility sizes (M36) with the small chain of
//! `test_support::production` and the sizes of the data.

use std::sync::Arc;

use crate::calendar::RoundLength;
use crate::catalog::{Catalog, FacilitySize, SiteType, SizeModel, test_support};
use crate::command::Command;
use crate::game::Game;
use crate::money::Money;
use crate::production;
use crate::save;
use crate::state::{GameSettings, SiteId, StartForm};

fn usd(v: f64) -> Money {
    Money::from_usd(v).unwrap()
}

/// The sizes of `data/parameter/produktionsmodell.yaml`.
fn sizes() -> SizeModel {
    SizeModel {
        capacity: [0.25, 0.5, 1.0, 2.0, 4.0],
        investment_exponent: 0.7,
        labor_exponent: -0.15,
        area_exponent: 0.7,
        build_exponent: 0.3,
    }
}

fn catalog() -> Catalog {
    let mut c = test_support::production();
    c.production_model.sizes = sizes();
    c
}

fn new_game(catalog: Catalog) -> Game {
    let catalog = Arc::new(catalog);
    let settings = GameSettings {
        seed: 3,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(20_000_000.0),
        start_form: StartForm::Workshop,
        company_name: "Hütte AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
        tariff_dynamics: 1.0,
    };
    Game::new(catalog, settings).unwrap()
}

fn cash(game: &Game) -> Money {
    game.state().company(game.player()).unwrap().ledger.cash()
}

/// A works with one furnace of a size, smelting at full utilization.
fn works(game: &mut Game, size: FacilitySize) -> SiteId {
    let c = game.catalog().clone();
    game.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: SiteType::Factory,
    })
    .unwrap();
    let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.apply(Command::BuildFacility {
        site,
        facility: c.facilities.id("ofen").unwrap(),
        count: 1,
        size,
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site,
        slot: 0,
        recipe: c.recipes.id("eisen_schmelzen"),
        utilization: 1.0,
    })
    .unwrap();
    site
}

#[test]
fn the_size_model_plans_sizes_and_numbers() {
    let m = sizes();
    assert_eq!(m.units_for(0.1), (FacilitySize::VerySmall, 1));
    assert_eq!(m.units_for(0.3), (FacilitySize::VerySmall, 1));
    assert_eq!(m.units_for(1.0), (FacilitySize::Medium, 1));
    assert_eq!(m.units_for(3.0), (FacilitySize::Large, 2));
    assert_eq!(m.units_for(5.0), (FacilitySize::VeryLarge, 1));
    assert_eq!(m.units_for(12.0), (FacilitySize::VeryLarge, 3));
    // Investment k^0.7, labor per run k^-0.15, construction time k^0.3.
    assert_eq!(m.investment(FacilitySize::Medium), 1.0);
    assert!((m.investment(FacilitySize::Large) - 2f64.powf(0.7)).abs() < 1e-12);
    assert!((m.labor(FacilitySize::VeryLarge) - 4f64.powf(-0.15)).abs() < 1e-12);
    assert!((m.area(FacilitySize::Small) - 0.5f64.powf(0.7)).abs() < 1e-12);
    assert_eq!(m.build_days(FacilitySize::Large, 20), 25);
    assert_eq!(m.build_days(FacilitySize::VerySmall, 1), 1);

    // Catalogs without sizes treat every size like the data size.
    let neutral = SizeModel::default();
    assert_eq!(neutral.units_for(0.3), (FacilitySize::Medium, 1));
    assert_eq!(neutral.units_for(3.4), (FacilitySize::Medium, 3));
    assert_eq!(neutral.investment(FacilitySize::VeryLarge), 1.0);
}

#[test]
fn a_large_facility_makes_twice_as_much_for_less_per_unit() {
    let mut game = new_game(catalog());
    let c = game.catalog().clone();
    let before = cash(&game);
    let medium = works(&mut game, FacilitySize::Medium);
    let paid_medium = before - cash(&game);
    let before = cash(&game);
    let large = works(&mut game, FacilitySize::Large);
    let paid_large = before - cash(&game);
    // Site 500 000 USD (test catalog: 200 000) plus the furnace: 2 000 000 × 2^0.7.
    let furnace = 2_000_000.0;
    assert!(
        ((paid_large - paid_medium).to_usd() - furnace * (2f64.powf(0.7) - 1.0)).abs() < 1.0,
        "{paid_medium:?} {paid_large:?}"
    );
    let slot = &game.state().sites[large.index()].slots[0];
    assert_eq!(slot.size, FacilitySize::Large);
    assert_eq!(slot.units(&c), 2.0);
    assert_eq!(slot.full_runs(&c), 100.0);
    // Ready after 20 · 2^0.3 = 24.6 days, the medium one after 20.
    let today = game.state().date;
    assert_eq!(today.days_until(slot.ready), 25);
    let medium_ready = game.state().sites[medium.index()].slots[0].ready;
    assert_eq!(today.days_until(medium_ready), 20);

    for _ in 0..26 {
        game.advance(RoundLength::Day, |_| {});
    }
    let (state, date) = (game.state(), game.state().date);
    let workers = |site: SiteId| -> f64 {
        production::needed_workers(&c, state, site, date)
            .iter()
            .sum()
    };
    // Twice the runs, each with 2^-0.15 of the hours: 2^0.85 of the staff.
    let ratio = workers(large) / workers(medium);
    assert!((ratio - 2f64.powf(0.85)).abs() < 1e-9, "{ratio}");

    // Per unit made: labor 2^-0.15, depreciation and maintenance 2^0.7 / 2.
    let costs = |site| production::unit_costs(&c, state, site)[0].clone();
    let (m, l) = (costs(medium), costs(large));
    assert!((l.output_per_day / m.output_per_day - 2.0).abs() < 1e-9);
    assert!((l.labor / m.labor - 2f64.powf(-0.15)).abs() < 1e-9);
    assert!((l.facility / m.facility - 2f64.powf(0.7) / 2.0).abs() < 1e-9);
}

#[test]
fn a_slot_keeps_its_size_in_saves_and_old_slots_are_medium() {
    let mut game = new_game(catalog());
    let small = works(&mut game, FacilitySize::Small);
    let medium = works(&mut game, FacilitySize::Medium);
    let loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    let size = |site: SiteId| loaded.state().sites[site.index()].slots[0].size;
    assert_eq!(size(small), FacilitySize::Small);
    assert_eq!(size(medium), FacilitySize::Medium);
    // A medium slot is written without its size, and a slot without one is medium.
    let slot = &game.state().sites[medium.index()].slots[0];
    let json = serde_json::to_value(slot).unwrap();
    assert!(json.get("size").is_none(), "{json}");
    let read: crate::state::Slot = serde_json::from_value(json).unwrap();
    assert_eq!(read.size, FacilitySize::Medium);
    // A build command from before M36 (without a size) builds a medium unit.
    let command = Command::BuildFacility {
        site: small,
        facility: game.catalog().facilities.id("ofen").unwrap(),
        count: 2,
        size: FacilitySize::VeryLarge,
    };
    let mut json = serde_json::to_value(&command).unwrap();
    json["BuildFacility"]
        .as_object_mut()
        .unwrap()
        .remove("size")
        .expect("written with its size");
    let old: Command = serde_json::from_value(json).unwrap();
    assert!(matches!(
        old,
        Command::BuildFacility {
            size: FacilitySize::Medium,
            count: 2,
            ..
        }
    ));
}
