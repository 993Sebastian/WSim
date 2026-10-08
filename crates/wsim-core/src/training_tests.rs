//! Scenario tests for the training of the sites (W1) with the small chain of
//! `test_support::production`.

use std::sync::Arc;

use crate::calendar::RoundLength;
use crate::catalog::{Catalog, SiteType, TrainingModel, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::money::Money;
use crate::production;
use crate::state::{GameSettings, SiteId, StartForm};
use crate::strategy::{StrategyField, StrategyScope, StrategyValue};
use crate::training::{self, TargetSource};

fn catalog() -> Catalog {
    // The management catalog: production chain and strategies (MA4).
    let mut c = test_support::management();
    c.production_model.training = TrainingModel {
        cost_share: 0.05,
        gain_per_month: 0.25,
        loss_per_month: 0.1,
        labor_saving: 0.2,
        quality_points: 10.0,
    };
    c
}

fn new_game(catalog: Catalog) -> Game {
    let catalog = Arc::new(catalog);
    let settings = GameSettings {
        seed: 3,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: Money::from_usd(10_000_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "Schule AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
    };
    Game::new(catalog, settings).unwrap()
}

/// A mine running at full capacity once its deposit is developed.
fn mine(game: &mut Game) -> SiteId {
    let c = game.catalog().clone();
    game.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: SiteType::Extraction,
    })
    .unwrap();
    let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.apply(Command::BuildFacility {
        site,
        facility: c.facilities.id("mine").unwrap(),
        count: 1,
        size: crate::catalog::FacilitySize::Medium,
    })
    .unwrap();
    game.apply(Command::DevelopDeposit {
        site,
        deposit: c.deposits.id("grube").unwrap(),
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site,
        slot: 0,
        recipe: c.recipes.id("erz_abbau"),
        utilization: 1.0,
    })
    .unwrap();
    site
}

/// Days up to and including the next month start.
fn to_next_month(game: &mut Game) {
    loop {
        game.advance(RoundLength::Day, |_| {});
        if game.state().date.day() == 1 {
            return;
        }
    }
}

#[test]
fn training_rises_month_by_month_costs_wages_and_saves_labor() {
    let mut game = new_game(catalog());
    let site = mine(&mut game);
    to_next_month(&mut game);
    to_next_month(&mut game);
    let date = game.state().date;
    let untrained = production::needed_workers(game.catalog(), game.state(), site, date);
    game.apply(Command::SetTraining {
        site,
        target: Some(1.0),
    })
    .unwrap();
    // The level follows the target a step a month.
    to_next_month(&mut game);
    assert_eq!(game.state().sites[site.index()].training, 0.25);
    to_next_month(&mut game);
    assert_eq!(game.state().sites[site.index()].training, 0.5);
    // Half the level: a tenth fewer labor hours, five quality points.
    let date = game.state().date;
    let trained = production::needed_workers(game.catalog(), game.state(), site, date);
    for (t, u) in trained.iter().zip(&untrained) {
        assert!((t - 0.9 * u).abs() < 1e-9, "{t} vs {u}");
    }
    assert_eq!(training::quality(game.catalog(), 0.5), 5.0);
    // The cost follows the target: 5 % of the wages at the target 1.
    let view = crate::views::production(&game);
    let s = view
        .sites
        .iter()
        .find(|s| s.index == site.0)
        .expect("the mine");
    assert!(
        (s.training_cost_per_day_usd - 0.05 * s.wage_cost_per_day_usd).abs() < 1e-6,
        "{} vs {}",
        s.training_cost_per_day_usd,
        s.wage_cost_per_day_usd
    );
    assert_eq!((s.training, s.training_target), (0.5, 1.0));
    assert_eq!(s.training_source, "standort");
    assert!((s.training_labor_saving - 0.1).abs() < 1e-12);
    // Without a target the level falls back slowly.
    game.apply(Command::SetTraining { site, target: None })
        .unwrap();
    to_next_month(&mut game);
    assert!((game.state().sites[site.index()].training - 0.4).abs() < 1e-12);
    let ledger = &game.state().companies[game.player().index()].ledger;
    assert!(ledger.is_balanced());
}

#[test]
fn the_strategy_sets_the_target_unless_the_site_has_its_own() {
    let mut game = new_game(catalog());
    let site = mine(&mut game);
    let target = |game: &Game| training::target(game.catalog(), game.state(), site);
    assert_eq!(target(&game), (0.0, TargetSource::None));
    game.apply(Command::SetStrategy {
        scope: StrategyScope::Company,
        field: StrategyField::Training,
        value: Some(StrategyValue::Training(0.6)),
    })
    .unwrap();
    assert_eq!(target(&game), (0.6, TargetSource::Strategy));
    game.apply(Command::SetTraining {
        site,
        target: Some(0.3),
    })
    .unwrap();
    assert_eq!(target(&game), (0.3, TargetSource::Site));
    game.apply(Command::SetTraining { site, target: None })
        .unwrap();
    assert_eq!(target(&game).1, TargetSource::Strategy);
    // Bounds of the command and of the strategy.
    assert_eq!(
        game.apply(Command::SetTraining {
            site,
            target: Some(1.5),
        }),
        Err(CommandError::InvalidTraining)
    );
    assert!(
        game.apply(Command::SetStrategy {
            scope: StrategyScope::Company,
            field: StrategyField::Training,
            value: Some(StrategyValue::Training(-0.1)),
        })
        .is_err()
    );
}

#[test]
fn ai_companies_train_by_competence() {
    let mut c = catalog();
    c.ai_model.behavior.training = crate::catalog::Span {
        at_0: 0.0,
        at_1: 0.8,
    };
    let mut game = new_game(c);
    let site = mine(&mut game);
    // The player's company as an AI company of competence 0.5.
    let state = game.state_mut();
    state.companies[0].ai = Some(crate::state::AiState {
        competence: 0.5,
        aggressiveness: 0.5,
        real: None,
        next_operations: crate::calendar::Date::new(2100, 1, 1).unwrap(),
        staff: 0.0,
    });
    to_next_month(&mut game);
    let s = &game.state().sites[site.index()];
    assert_eq!(s.training_target, Some(0.4));
    assert_eq!(s.training, 0.25);
}
