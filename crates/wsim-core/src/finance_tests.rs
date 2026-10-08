//! Tests for loans, interest, taxes and insolvency (M6).

use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::finance;
use crate::game::Game;
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::keys;
use crate::money::Money;
use crate::state::{GameSettings, SiteId, StartForm};

fn usd(v: f64) -> Money {
    Money::from_usd(v).unwrap()
}

fn new_game(capital: f64) -> Game {
    let catalog = Arc::new(test_support::production());
    let settings = GameSettings {
        seed: 5,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(capital),
        start_form: StartForm::Workshop,
        company_name: "Bankkunde".into(),
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

fn ledger(game: &Game) -> &crate::ledger::Ledger {
    &game.state().company(game.player()).unwrap().ledger
}

/// A factory with a furnace as collateral (2 200 000 USD of assets).
fn with_collateral(game: &mut Game) {
    let c = game.catalog().clone();
    game.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: SiteType::Factory,
    })
    .unwrap();
    game.apply(Command::BuildFacility {
        site: SiteId(0),
        facility: c.facilities.id("ofen").unwrap(),
        count: 1,
        size: crate::catalog::FacilitySize::Medium,
    })
    .unwrap();
}

#[test]
fn annuities_match_the_textbook() {
    // 100 000 at 6 % over 10 years: 1 110.21 a month.
    assert_eq!(
        finance::instalment(usd(100_000.0), 0.06, 120),
        usd(1_110.205)
    );
    assert_eq!(finance::instalment(usd(120_000.0), 0.0, 120), usd(1_000.0));
}

#[test]
fn loans_need_collateral() {
    let mut game = new_game(3_000_000.0);
    let err = game
        .apply(Command::TakeLoan {
            amount: usd(1.0),
            years: 5,
        })
        .unwrap_err();
    assert_eq!(err, CommandError::LoanTooLarge { limit: Money::ZERO });
    with_collateral(&mut game);
    let limit = finance::credit_limit(game.catalog(), game.state().company(game.player()).unwrap());
    assert_eq!(limit, usd(2_200_000.0).scale(0.6));
    assert_eq!(
        game.apply(Command::TakeLoan {
            amount: limit + usd(1.0),
            years: 5
        })
        .unwrap_err(),
        CommandError::LoanTooLarge { limit }
    );
    assert_eq!(
        game.apply(Command::TakeLoan {
            amount: usd(1.0),
            years: 31
        })
        .unwrap_err(),
        CommandError::InvalidTerm { max: 30 }
    );
    game.apply(Command::TakeLoan {
        amount: usd(1_000_000.0),
        years: 10,
    })
    .unwrap();
    assert_eq!(ledger(&game).balance(Account::Loans), usd(1_000_000.0));
    assert_eq!(
        ledger(&game).month.cash_flow.financing,
        usd(3_000_000.0) + usd(1_000_000.0)
    );
}

#[test]
fn interest_and_repayment_are_booked_monthly() {
    let mut game = new_game(3_000_000.0);
    with_collateral(&mut game);
    game.apply(Command::TakeLoan {
        amount: usd(1_000_000.0),
        years: 10,
    })
    .unwrap();
    let loan = game.state().company(game.player()).unwrap().loans[0].clone();
    // Base 3 % + minimum premium 1 % + 8 % · debt ratio 1 000 000 / 4 000 000 = 6 %.
    let expected_rate = 0.03 + 0.01 + 0.08 * (1_000_000.0 / 4_000_000.0);
    assert!((loan.rate - expected_rate).abs() < 1e-12, "{}", loan.rate);
    game.advance(RoundLength::Month, |_| {});
    let interest = usd(1_000_000.0).scale(loan.rate / 12.0);
    let january = &ledger(&game).months[0];
    assert_eq!(january.by_type[&CostType::Interest], -interest);
    let balance = game.state().company(game.player()).unwrap().loans[0].balance;
    assert_eq!(balance, usd(1_000_000.0) - (loan.instalment - interest));
    assert!(ledger(&game).is_balanced());

    // Early repayment of the rest.
    game.apply(Command::RepayLoan {
        loan: 0,
        amount: usd(10_000_000.0),
    })
    .unwrap();
    assert!(
        game.state()
            .company(game.player())
            .unwrap()
            .loans
            .is_empty()
    );
    assert_eq!(ledger(&game).balance(Account::Loans), Money::ZERO);
}

#[test]
fn profit_tax_with_loss_carryforward() {
    let mut game = new_game(1_000_000.0);
    let player = game.player();
    let book = |game: &mut Game, amount: f64| {
        let ledger = &mut game.state_mut().company_mut(player).unwrap().ledger;
        if amount >= 0.0 {
            ledger.income(
                CostType::Revenue,
                CostCenter::default(),
                Account::Cash,
                usd(amount),
            );
        } else {
            ledger.expense(
                CostType::Other,
                CostCenter::default(),
                Account::Cash,
                usd(-amount),
            );
        }
    };
    // 1900: loss of 100 000 → no tax, carried forward.
    book(&mut game, -100_000.0);
    game.advance(RoundLength::Quarter, |_| {});
    for _ in 0..3 {
        game.advance(RoundLength::Quarter, |_| {});
    }
    assert_eq!(
        game.state().company(player).unwrap().loss_carryforward,
        usd(100_000.0)
    );
    assert!(
        !ledger(&game).years[0]
            .by_type
            .contains_key(&CostType::Taxes)
    );
    // 1901: profit of 300 000 → tax on 200 000 at 20 %.
    book(&mut game, 300_000.0);
    for _ in 0..4 {
        game.advance(RoundLength::Quarter, |_| {});
    }
    assert_eq!(
        ledger(&game).years[1].by_type[&CostType::Taxes],
        -usd(40_000.0)
    );
    assert_eq!(
        game.state().company(player).unwrap().loss_carryforward,
        Money::ZERO
    );
    assert!(ledger(&game).is_balanced());
}

#[test]
fn overdraft_is_warned_and_insolvency_ends_the_game() {
    let mut game = new_game(100_000.0);
    let player = game.player();
    let spend = |game: &mut Game, amount: f64| {
        let ledger = &mut game.state_mut().company_mut(player).unwrap().ledger;
        ledger.expense(
            CostType::Other,
            CostCenter::default(),
            Account::Cash,
            usd(amount),
        );
    };
    // Slightly overdrawn: within the overdraft limit? No assets → limit 0, but a warning
    // is given during the month and insolvency is decided at the month's end.
    spend(&mut game, 150_000.0);
    let report = game.advance(RoundLength::Day, |_| {});
    assert!(report.messages.iter().any(|m| m.key == keys::OVERDRAFT));
    assert!(!game.is_over());
    let report = game.advance(RoundLength::Month, |_| {});
    assert!(
        report
            .messages
            .iter()
            .any(|m| m.key == keys::GAME_OVER_INSOLVENT),
        "{:?}",
        report.messages
    );
    assert!(game.is_over());
    assert!(game.state().company(player).unwrap().bankrupt);
}

#[test]
fn collateral_saves_from_insolvency() {
    let mut game = new_game(2_400_000.0);
    with_collateral(&mut game);
    let player = game.player();
    // 200 000 short, but the bank would still lend against the furnace.
    let ledger = &mut game.state_mut().company_mut(player).unwrap().ledger;
    ledger.expense(
        CostType::Other,
        CostCenter::default(),
        Account::Cash,
        usd(400_000.0),
    );
    game.advance(RoundLength::Month, |_| {});
    assert!(!game.is_over());
    game.apply(Command::TakeLoan {
        amount: usd(500_000.0),
        years: 5,
    })
    .unwrap();
    assert!(ledger_cash_positive(&game));
}

fn ledger_cash_positive(game: &Game) -> bool {
    ledger(game).cash() > Money::ZERO
}

#[test]
fn cash_flow_is_split_by_activity() {
    let mut game = new_game(3_000_000.0);
    with_collateral(&mut game);
    let cf = ledger(&game).month.cash_flow;
    assert_eq!(cf.investing, -usd(2_200_000.0));
    assert_eq!(cf.financing, usd(3_000_000.0));
    assert_eq!(cf.total(), ledger(&game).cash());
    assert!(Date::new(1900, 1, 1).is_some());
}
