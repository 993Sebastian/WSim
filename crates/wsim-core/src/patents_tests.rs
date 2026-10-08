//! Scenario tests for patents and licences (P7) with `test_support::research`.

use std::sync::Arc;

use crate::calendar::RoundLength;
use crate::catalog::{Catalog, PatentModel, SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::ids::{CountryId, TechnologyId};
use crate::ledger::CostType;
use crate::message::keys;
use crate::money::Money;
use crate::patents;
use crate::state::{GameSettings, Limit, SiteId, StartForm};

fn catalog() -> Catalog {
    let mut c = test_support::research();
    c.research_model.patents = Some(PatentModel {
        term_years: 1,
        filing_days: 60,
        cost_per_country_usd: 10_000.0,
        ai_largest_markets: 1,
        provenance: Default::default(),
    });
    c
}

fn new_game(catalog: Catalog) -> Game {
    let catalog = Arc::new(catalog);
    let settings = GameSettings {
        seed: 7,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: Money::from_usd(5_000_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Patent AG".into(),
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

fn turbine(game: &Game) -> TechnologyId {
    game.catalog().technologies.id("turbine").unwrap()
}

fn country(game: &Game, key: &str) -> CountryId {
    game.catalog().countries.id(key).unwrap()
}

fn days(game: &mut Game, n: u32) -> Vec<String> {
    let mut keys = Vec::new();
    for _ in 0..n {
        let report = game.advance(RoundLength::Day, |_| {});
        keys.extend(report.messages.into_iter().map(|m| m.key));
    }
    keys
}

/// A research center of the player working on the turbine until it is invented.
fn invent(game: &mut Game) -> Vec<String> {
    let c = game.catalog().clone();
    game.apply(Command::FoundSite {
        country: country(game, "AAA"),
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
    let t = turbine(game);
    game.apply(Command::SetResearch {
        site,
        technology: Some(t),
    })
    .unwrap();
    let mut seen = Vec::new();
    for _ in 0..400 {
        let report = game.advance(RoundLength::Day, |_| {});
        let done = report.messages.iter().any(|m| m.key == keys::RESEARCH_DONE);
        seen.extend(report.messages.into_iter().map(|m| m.key));
        if done {
            return seen;
        }
    }
    panic!("research finishes");
}

#[test]
fn the_first_inventor_may_file_within_the_period() {
    let mut game = new_game(catalog());
    let seen = invent(&mut game);
    let t = turbine(&game);
    let player = game.player();
    assert!(seen.iter().any(|k| k == keys::PATENT_CLAIM), "{seen:?}");
    let claim = game.state().patents.get(t).clone().expect("claim");
    assert_eq!(claim.holder, player);
    assert!(claim.filed.is_none());
    // Not in force before filing.
    assert!(patents::in_force(game.state(), game.catalog(), t).is_none());

    let cash = game.state().companies[player.index()].ledger.cash();
    let aaa = country(&game, "AAA");
    game.apply(Command::FilePatent {
        technology: t,
        countries: vec![aaa],
    })
    .unwrap();
    let state = game.state();
    let paid = cash - state.companies[player.index()].ledger.cash();
    assert_eq!(paid, patents::filing_cost(state, game.catalog(), aaa));
    assert!(paid > Money::ZERO);
    let ledger = &state.companies[player.index()].ledger;
    assert!(ledger.year.by_type[&CostType::Licenses] < Money::ZERO);
    assert!(ledger.is_balanced());
    assert!(patents::in_force(state, game.catalog(), t).is_some());

    // The same country again is nothing new; a technology without a claim fails.
    assert_eq!(
        game.apply(Command::FilePatent {
            technology: t,
            countries: vec![aaa],
        }),
        Err(CommandError::NoPatentClaim)
    );
    let smelting = game.catalog().technologies.id("schmelzen").unwrap();
    assert_eq!(
        game.apply(Command::FilePatent {
            technology: smelting,
            countries: vec![aaa],
        }),
        Err(CommandError::NoPatentClaim)
    );
    // After the filing period no further country.
    days(&mut game, 61);
    let bbb = country(&game, "BBB");
    assert_eq!(
        game.apply(Command::FilePatent {
            technology: t,
            countries: vec![bbb],
        }),
        Err(CommandError::NoPatentClaim)
    );
}

#[test]
fn a_claim_not_filed_lapses_and_a_patent_ends_after_its_term() {
    let mut game = new_game(catalog());
    invent(&mut game);
    let t = turbine(&game);
    let seen = days(&mut game, 100);
    assert!(
        seen.iter().any(|k| k == keys::PATENT_CLAIM_LAPSED),
        "{seen:?}"
    );
    assert!(game.state().patents.get(t).is_none());

    // A filed patent ends with its term (one year in this catalog).
    let player = game.player();
    let date = game.state().date;
    let aaa = country(&game, "AAA");
    *game.state_mut().patents.get_mut(t) = Some(patents::Patent {
        holder: player,
        invented: date,
        filed: Some(date),
        countries: [aaa].into_iter().collect(),
        licensees: Default::default(),
    });
    let seen = days(&mut game, 400);
    assert!(seen.iter().any(|k| k == keys::PATENT_ENDED), "{seen:?}");
    assert!(game.state().patents.get(t).is_none());
}

/// A competitor holding a patent on the turbine in AAA; smelting needs the turbine.
fn patented_smelting() -> (Game, crate::state::CompanyId, TechnologyId) {
    let mut c = catalog();
    let turbine = c.technologies.id("turbine").unwrap();
    let smelting = c.recipes.id("eisen_schmelzen").unwrap();
    c.recipes.get_mut(smelting).technology = Some(turbine);
    let mut game = new_game(c);
    let holder = crate::trade_tests::competitor(&mut game);
    let player = game.player();
    let date = game.state().date;
    let aaa = country(&game, "AAA");
    let state = game.state_mut();
    state.companies[player.index()].technologies.insert(turbine);
    state.companies[holder.index()].technologies.insert(turbine);
    *state.patents.get_mut(turbine) = Some(patents::Patent {
        holder,
        invented: date,
        filed: Some(date),
        countries: [aaa].into_iter().collect(),
        licensees: Default::default(),
    });
    (game, holder, turbine)
}

/// A works with a furnace in `key`, ready after 40 days.
fn furnace(game: &mut Game, key: &str) -> SiteId {
    let c = game.catalog().clone();
    game.apply(Command::FoundSite {
        country: country(game, key),
        kind: SiteType::Factory,
    })
    .unwrap();
    let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.apply(Command::BuildFacility {
        site,
        facility: c.facilities.id("ofen").unwrap(),
        count: 1,
        size: crate::catalog::FacilitySize::Medium,
    })
    .unwrap();
    site
}

#[test]
fn a_patent_blocks_others_in_its_countries_only() {
    let (mut game, holder, t) = patented_smelting();
    let player = game.player();
    let recipe = game.catalog().recipes.id("eisen_schmelzen");
    let (aaa, bbb) = (country(&game, "AAA"), country(&game, "BBB"));
    let (state, catalog) = (game.state(), game.catalog());
    assert!(patents::blocks(state, catalog, player, t, aaa));
    assert!(!patents::blocks(state, catalog, player, t, bbb));
    assert!(!patents::blocks(state, catalog, holder, t, aaa));
    assert!(patents::needs_license(state, catalog, player, t));

    let home = furnace(&mut game, "AAA");
    let abroad = furnace(&mut game, "BBB");
    let set = |site| Command::SetProduction {
        site,
        slot: 0,
        recipe,
        utilization: 1.0,
    };
    assert!(matches!(
        game.apply(set(home)),
        Err(CommandError::Patented { .. })
    ));
    game.apply(set(abroad)).unwrap();
}

#[test]
fn running_facilities_stop_and_a_licence_of_the_holder_frees_them() {
    let (mut game, holder, t) = patented_smelting();
    let player = game.player();
    // The patent comes after the furnace runs.
    let patent = game.state_mut().patents.get_mut(t).take();
    let home = furnace(&mut game, "AAA");
    game.apply(Command::SetProduction {
        site: home,
        slot: 0,
        recipe: game.catalog().recipes.id("eisen_schmelzen"),
        utilization: 1.0,
    })
    .unwrap();
    days(&mut game, 40);
    *game.state_mut().patents.get_mut(t) = patent;
    let seen = days(&mut game, 40);
    let slot = &game.state().sites[home.index()].slots[0];
    assert_eq!(slot.limit, Some(Limit::Patent));
    assert_eq!(slot.last_runs, 0.0);
    assert!(seen.iter().any(|k| k == keys::PATENT_BLOCKED), "{seen:?}");

    // Only the holder licenses; afterwards the player may use it everywhere.
    let other = crate::trade_tests::competitor(&mut game);
    game.state_mut().companies[other.index()]
        .technologies
        .insert(t);
    let (state, catalog) = (game.state(), game.catalog());
    assert!(!patents::may_license(state, catalog, other, player, t));
    assert!(patents::may_license(state, catalog, holder, player, t));
    assert!(
        crate::deals::license_value(state, catalog, player, t).is_some_and(|v| v > Money::ZERO)
    );
    patents::licensed(game.state_mut(), t, player, holder);
    assert!(!patents::needs_license(
        game.state(),
        game.catalog(),
        player,
        t
    ));
    days(&mut game, 1);
    assert_ne!(
        game.state().sites[home.index()].slots[0].limit,
        Some(Limit::Patent)
    );
}

#[test]
fn prior_users_keep_using_it_and_patents_survive_saving() {
    let mut game = new_game(catalog());
    invent(&mut game);
    let t = turbine(&game);
    let rival = crate::trade_tests::competitor(&mut game);
    game.state_mut().companies[rival.index()]
        .technologies
        .insert(t);
    let aaa = country(&game, "AAA");
    game.apply(Command::FilePatent {
        technology: t,
        countries: vec![aaa],
    })
    .unwrap();
    let state = game.state();
    assert!(!patents::blocks(state, game.catalog(), rival, t, aaa));
    let bytes = crate::save::encode(&game);
    let loaded = crate::save::decode(&bytes, game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(loaded.state().patents, game.state().patents);
    assert_eq!(loaded.state_hash(), game.state_hash());
}
