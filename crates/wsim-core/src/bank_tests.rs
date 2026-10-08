//! Tests of the player's banks and the investment firm (K4).

use std::sync::Arc;

use crate::bank::{self, BankSettings};
use crate::calendar::RoundLength;
use crate::catalog::{BankModel, Catalog, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::group::SubsidiaryFocus;
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::keys;
use crate::money::Money;
use crate::save;
use crate::state::{CompanyId, GameSettings, StartForm};
use crate::trade_tests::{competitor, days, usd};

fn with_bank(mut catalog: Catalog) -> Catalog {
    catalog.bank = BankModel {
        enabled: true,
        leverage_max: 10.0,
        neutral_spread: -0.01,
        elasticity: 25.0,
        adjustment: 0.5,
        reserve: 0.1,
        start_deposit_spread: -0.01,
        start_loan_discount: 0.1,
        start_max_debt_ratio: 0.5,
        provenance: Default::default(),
    };
    catalog
}

fn game_as(form: StartForm) -> Game {
    let catalog = Arc::new(with_bank(test_support::trading()));
    let settings = GameSettings {
        seed: 3,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(1_000_000.0),
        start_form: form,
        company_name: "Hausbank".into(),
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

/// A borrower with collateral: half of its million in fixed assets.
fn borrower(game: &mut Game) -> CompanyId {
    let id = competitor(game);
    let l = &mut game.state_mut().companies[id.index()].ledger;
    l.transfer(Account::FixedAssets, Account::Cash, usd(500_000.0));
    id
}

#[test]
fn investors_and_banks_start_without_sites() {
    for form in [StartForm::Investor, StartForm::Bank] {
        let game = game_as(form);
        let me = game.player();
        assert!(game.state().sites.iter().all(|s| s.owner != me));
        let c = &game.state().companies[me.index()];
        assert_eq!(c.ledger.cash(), usd(1_000_000.0));
        assert_eq!(c.bank.is_some(), form == StartForm::Bank);
    }
}

#[test]
fn deposits_follow_the_rate() {
    let mut game = game_as(StartForm::Bank);
    let me = game.player();
    // Neutral spread: half the capacity of ten times the equity, half of it a month.
    days(&mut game, 31);
    let c = &game.state().companies[me.index()];
    let deposits = c.ledger.balance(Account::Deposits);
    assert!(
        (deposits.to_usd() - 2_500_000.0).abs() < 1_000.0,
        "{deposits:?}"
    );
    assert!(c.ledger.is_balanced());
    // Two points more: all of the capacity; the interest is paid on the deposits.
    let settings = BankSettings {
        deposit_spread: 0.01,
        ..c.bank.unwrap()
    };
    game.apply(Command::SetBank {
        company: me,
        settings,
    })
    .unwrap();
    let target = bank::deposit_target(game.catalog(), &game.state().companies[me.index()]);
    assert!(target.to_usd() > 9_900_000.0, "{target:?}");
    days(&mut game, 28);
    let c = &game.state().companies[me.index()];
    assert!(c.ledger.balance(Account::Deposits) > deposits);
    let interest = c.ledger.months.last().unwrap().by_type[&CostType::Interest];
    assert!(interest < Money::ZERO);
    assert_eq!(
        game.apply(Command::SetBank {
            company: me,
            settings: BankSettings {
                loan_discount: 1.5,
                ..settings
            },
        }),
        Err(CommandError::InvalidBankSettings)
    );
}

#[test]
fn companies_borrow_from_the_cheaper_bank() {
    let mut game = game_as(StartForm::Bank);
    let me = game.player();
    let other = borrower(&mut game);
    let catalog = game.catalog().clone();
    let cash = game.state().companies[me.index()].ledger.cash();
    let market = crate::finance::loan_rate(
        &catalog,
        &game.state().companies[other.index()],
        usd(100_000.0),
        game.state().date,
        0.0,
    );
    crate::command::execute(
        game.state_mut(),
        &catalog,
        other,
        &Command::TakeLoan {
            amount: usd(100_000.0),
            years: 5,
        },
    )
    .unwrap();
    let loan = game.state().companies[other.index()].loans[0].clone();
    assert_eq!(loan.lender, Some(me));
    assert!((loan.rate - market * 0.9).abs() < 1e-12);
    let b = &game.state().companies[me.index()];
    assert_eq!(b.ledger.cash(), cash - usd(100_000.0));
    assert_eq!(b.ledger.balance(Account::LoansGiven), usd(100_000.0));
    // A month of interest and instalment flows to the bank.
    days(&mut game, 31);
    let b = &game.state().companies[me.index()];
    let paid = loan.balance - game.state().companies[other.index()].loans[0].balance;
    assert_eq!(b.ledger.balance(Account::LoansGiven), usd(100_000.0) - paid);
    let interest = b.ledger.months.last().unwrap().by_type[&CostType::Interest];
    // Interest income on the loan, less the interest on the deposits.
    assert!(interest > -usd(100_000.0).scale(loan.rate / 12.0));
    assert!(b.ledger.is_balanced());
    // Early repayment goes to the bank, too.
    let rest = game.state().companies[other.index()].loans[0].balance;
    game.state_mut().companies[other.index()].ledger.income(
        CostType::Revenue,
        CostCenter::default(),
        Account::Cash,
        usd(200_000.0),
    );
    crate::command::execute(
        game.state_mut(),
        &catalog,
        other,
        &Command::RepayLoan {
            loan: 0,
            amount: rest,
        },
    )
    .unwrap();
    let b = &game.state().companies[me.index()];
    assert_eq!(b.ledger.balance(Account::LoansGiven), Money::ZERO);
    assert!(b.ledger.is_balanced());
}

#[test]
fn a_bank_lends_within_its_standard_and_reserve() {
    let mut game = game_as(StartForm::Bank);
    let me = game.player();
    let other = borrower(&mut game);
    let (state, catalog) = (game.state(), game.catalog());
    let assets = state.companies[other.index()].ledger.total_assets();
    // 0.9 of its assets more: 47 % debt, within 50 %; 1.2 of them: 55 %.
    assert!(
        bank::lender_for(state, catalog, other, assets.scale(0.9), 0.05).is_some(),
        "{assets:?}"
    );
    assert!(bank::lender_for(state, catalog, other, assets.scale(1.2), 0.05).is_none());
    // Not to its own group, and not beyond its cash above the reserve.
    assert!(bank::lender_for(state, catalog, me, usd(10_000.0), 0.05).is_none());
    let room = bank::lending_room(catalog, &state.companies[me.index()]);
    assert_eq!(room, usd(1_000_000.0));
}

#[test]
fn failed_borrowers_are_written_off() {
    let mut game = game_as(StartForm::Bank);
    let me = game.player();
    let other = borrower(&mut game);
    let catalog = game.catalog().clone();
    crate::command::execute(
        game.state_mut(),
        &catalog,
        other,
        &Command::TakeLoan {
            amount: usd(100_000.0),
            years: 5,
        },
    )
    .unwrap();
    game.state_mut().companies[other.index()].bankrupt = true;
    let mut lost = false;
    for _ in 0..31 {
        let report = game.advance(RoundLength::Day, |_| {});
        lost |= report
            .messages
            .iter()
            .any(|m| m.key == keys::BANK_LOAN_LOST);
    }
    assert!(lost);
    let b = &game.state().companies[me.index()];
    assert_eq!(b.ledger.balance(Account::LoansGiven), Money::ZERO);
    assert!(b.ledger.year.by_type[&CostType::Investments] <= -usd(99_000.0));
    assert!(b.ledger.is_balanced());
}

#[test]
fn subsidiaries_can_be_banks_and_investors() {
    let mut catalog = with_bank(test_support::trading());
    catalog.subsidiaries.enabled = true;
    let mut game = crate::trade_tests::new_game(catalog);
    let me = game.player();
    let country = game.state().companies[me.index()].headquarters;
    game.apply(Command::FoundSubsidiary {
        name: "Bank Nord".into(),
        country,
        capital: usd(100_000.0),
        focus: SubsidiaryFocus::Bank,
    })
    .unwrap();
    let sub = CompanyId(u32::try_from(game.state().companies.len() - 1).unwrap());
    assert!(game.state().companies[sub.index()].bank.is_some());
    game.apply(Command::SetBank {
        company: sub,
        settings: BankSettings::start(game.catalog()),
    })
    .unwrap();
    // As an investment firm it is no bank any more; its deposits flow out.
    game.apply(Command::SetSubsidiaryFocus {
        company: sub,
        focus: SubsidiaryFocus::Investment,
    })
    .unwrap();
    assert!(game.state().companies[sub.index()].bank.is_none());
    assert_eq!(
        game.apply(Command::SetBank {
            company: sub,
            settings: BankSettings::start(game.catalog()),
        }),
        Err(CommandError::NotABank)
    );
}

#[test]
fn banks_survive_saving() {
    let mut game = game_as(StartForm::Bank);
    let other = borrower(&mut game);
    let catalog = game.catalog().clone();
    crate::command::execute(
        game.state_mut(),
        &catalog,
        other,
        &Command::TakeLoan {
            amount: usd(50_000.0),
            years: 3,
        },
    )
    .unwrap();
    days(&mut game, 40);
    let mut loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(game.state_hash(), loaded.state_hash());
    days(&mut game, 40);
    days(&mut loaded, 40);
    assert_eq!(game.state_hash(), loaded.state_hash());
}
