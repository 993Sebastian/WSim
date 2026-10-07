//! Scenario tests for shutting down, restarting and selling facilities (M22) with the
//! small chain of `test_support::production`.

use std::sync::Arc;

use crate::calendar::RoundLength;
use crate::catalog::{Catalog, SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::ledger::{Account, CostType};
use crate::money::Money;
use crate::save;
use crate::state::{GameSettings, Operation, SiteId, StartForm};

fn usd(v: f64) -> Money {
    Money::from_usd(v).unwrap()
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
    };
    Game::new(catalog, settings).unwrap()
}

fn days(game: &mut Game, n: u32) {
    for _ in 0..n {
        game.advance(RoundLength::Day, |_| {});
    }
}

/// A mine with its deposit, producing from day 30 on.
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

/// A works with `count` furnaces (ready after 20 days).
fn works(game: &mut Game, count: u32) -> SiteId {
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
        count,
        size: crate::catalog::FacilitySize::Medium,
    })
    .unwrap();
    site
}

fn ore(game: &Game, site: SiteId) -> f64 {
    let p = game.catalog().products.id("erz").unwrap();
    game.state()
        .site(site)
        .unwrap()
        .inventory
        .get(&p)
        .map_or(0.0, |s| s.quantity)
}

/// Maintenance booked so far in the running month.
fn maintenance(game: &Game) -> f64 {
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    -ledger
        .month
        .by_type
        .get(&CostType::Maintenance)
        .map_or(0.0, |m| m.to_usd())
}

fn cash(game: &Game) -> Money {
    game.state().company(game.player()).unwrap().ledger.cash()
}

#[test]
fn a_shut_down_facility_neither_produces_nor_wears_and_costs_less() {
    let mut game = new_game(test_support::production());
    let site = mine(&mut game);
    days(&mut game, 32);
    assert!(ore(&game, site) > 0.0, "the mine produces");
    game.apply(Command::MothballFacility {
        site,
        slot: 0,
        count: 1,
    })
    .unwrap();
    let before = ore(&game, site);
    let condition = game.state().site(site).unwrap().slots[0].condition;
    let maintenance_before = maintenance(&game);
    days(&mut game, 5);
    let s = game.state().site(site).unwrap();
    assert_eq!(ore(&game, site), before, "no production");
    assert_eq!(s.slots[0].condition, condition, "no wear");
    assert!(
        s.workforce.values().sum::<f64>() < 1e-9,
        "the staff is released"
    );
    // A quarter of the maintenance: 1 000 000 · 3.65 % / 365 · 0.25 = 25 USD a day.
    let per_day = (maintenance(&game) - maintenance_before) / 5.0;
    assert!((per_day - 25.0).abs() < 0.01, "{per_day}");
    assert!(matches!(s.slots[0].operation, Operation::Mothballed { .. }));
    assert!(
        game.state()
            .company(game.player())
            .unwrap()
            .ledger
            .is_balanced()
    );
}

#[test]
fn a_restart_costs_money_and_takes_its_time() {
    let mut game = new_game(test_support::production());
    let site = mine(&mut game);
    days(&mut game, 32);
    game.apply(Command::MothballFacility {
        site,
        slot: 0,
        count: 1,
    })
    .unwrap();
    days(&mut game, 3);
    let before = cash(&game);
    game.apply(Command::RestartFacility { site, slot: 0 })
        .unwrap();
    // 2 % of the investment of 1 000 000.
    assert_eq!(before - cash(&game), usd(20_000.0));
    let stock = ore(&game, site);
    days(&mut game, 29);
    assert_eq!(ore(&game, site), stock, "still starting up");
    days(&mut game, 3);
    assert!(ore(&game, site) > stock, "producing again");
    assert_eq!(
        game.state().site(site).unwrap().slots[0].operation,
        Operation::Running
    );
}

#[test]
fn selling_brings_part_of_the_book_value_and_keeps_the_books_balanced() {
    let mut game = new_game(test_support::production());
    let site = works(&mut game, 3);
    days(&mut game, 20 + 365);
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    let (cash_before, assets_before) = (ledger.cash(), ledger.balance(Account::FixedAssets));
    // One of three furnaces: 2 000 000 · (1 − 365 / 7300) = 1 900 000 book value, half
    // of it as proceeds.
    game.apply(Command::SellFacility {
        site,
        slot: 0,
        count: 1,
    })
    .unwrap();
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    assert_eq!(ledger.cash() - cash_before, usd(950_000.0));
    assert_eq!(
        assets_before - ledger.balance(Account::FixedAssets),
        usd(1_900_000.0)
    );
    assert_eq!(
        ledger.month.by_type.get(&CostType::Other).copied(),
        Some(usd(-950_000.0))
    );
    assert!(ledger.is_balanced());
    let s = game.state().site(site).unwrap();
    assert_eq!(s.slots[0].count, 2);
    assert_eq!(s.slots[0].cost, usd(4_000_000.0));

    // Selling the rest removes the facility.
    game.apply(Command::SellFacility {
        site,
        slot: 0,
        count: 2,
    })
    .unwrap();
    assert!(game.state().site(site).unwrap().slots.is_empty());
    assert!(
        game.state()
            .company(game.player())
            .unwrap()
            .ledger
            .is_balanced()
    );
}

#[test]
fn an_old_facility_brings_its_scrap_value() {
    let mut game = new_game(test_support::production());
    let site = works(&mut game, 1);
    days(&mut game, 20);
    // Past its lifetime of 20 years the book value is 0.
    let s = &mut game.state_mut().sites[site.index()];
    s.slots[0].ready = s.slots[0].ready.add_days(-365 * 21);
    let before = cash(&game);
    game.apply(Command::SellFacility {
        site,
        slot: 0,
        count: 1,
    })
    .unwrap();
    // 3 % of 2 000 000, booked as other income.
    assert_eq!(cash(&game) - before, usd(60_000.0));
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    assert!(ledger.is_balanced());
}

#[test]
fn part_of_a_facility_can_be_shut_down() {
    let mut game = new_game(test_support::production());
    let site = works(&mut game, 4);
    days(&mut game, 21);
    game.apply(Command::MothballFacility {
        site,
        slot: 0,
        count: 1,
    })
    .unwrap();
    let s = game.state().site(site).unwrap();
    assert_eq!(s.slots.len(), 2);
    assert_eq!((s.slots[0].count, s.slots[1].count), (3, 1));
    assert_eq!(s.slots[0].cost + s.slots[1].cost, usd(8_000_000.0));
    assert_eq!(s.slots[0].operation, Operation::Running);
    assert!(s.slots[1].mothballed());
}

#[test]
fn shutting_down_and_selling_are_checked() {
    let mut game = new_game(test_support::production());
    let site = works(&mut game, 2);
    let sell = |count| Command::SellFacility {
        site,
        slot: 0,
        count,
    };
    let shut = |count| Command::MothballFacility {
        site,
        slot: 0,
        count,
    };
    assert_eq!(game.apply(sell(1)), Err(CommandError::UnderConstruction));
    assert_eq!(game.apply(shut(1)), Err(CommandError::UnderConstruction));
    days(&mut game, 21);
    assert_eq!(game.apply(sell(0)), Err(CommandError::InvalidQuantity));
    assert_eq!(
        game.apply(shut(3)),
        Err(CommandError::TooManyUnits { count: 2 })
    );
    assert_eq!(
        game.apply(Command::RestartFacility { site, slot: 0 }),
        Err(CommandError::NotMothballed)
    );
    game.apply(shut(2)).unwrap();
    assert_eq!(game.apply(shut(1)), Err(CommandError::AlreadyMothballed));
    // A shut down facility can still be sold.
    game.apply(sell(2)).unwrap();
    assert_eq!(
        game.apply(Command::RestartFacility { site, slot: 0 }),
        Err(CommandError::UnknownSlot)
    );
}

#[test]
fn shut_down_facilities_survive_saving_and_replaying() {
    let mut game = new_game(test_support::production());
    let site = works(&mut game, 3);
    days(&mut game, 21);
    game.apply(Command::MothballFacility {
        site,
        slot: 0,
        count: 2,
    })
    .unwrap();
    game.apply(Command::SellFacility {
        site,
        slot: 0,
        count: 1,
    })
    .unwrap();
    days(&mut game, 3);
    let bytes = save::encode(&game);
    let loaded = save::decode(&bytes, game.catalog().clone()).unwrap();
    assert_eq!(loaded.game.state(), game.state());
    let replayed = Game::replay(
        game.catalog().clone(),
        game.state().settings.clone(),
        game.journal(),
    )
    .unwrap();
    assert_eq!(replayed.state(), game.state());
}
