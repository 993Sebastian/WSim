//! Tests of corporate bonds (K2).

use crate::bonds;
use crate::calendar::{Date, RoundLength};
use crate::catalog::{BondGrade, BondModel, Catalog, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::keys;
use crate::money::Money;
use crate::save;
use crate::trade_tests::{days, new_game, usd};

fn grade(key: &str, debt_ratio_max: f64, coverage_min: f64, spread: f64) -> BondGrade {
    BondGrade {
        key: key.into(),
        debt_ratio_max,
        coverage_min,
        spread,
    }
}

fn with_bonds(mut catalog: Catalog) -> Catalog {
    catalog.bonds = BondModel {
        enabled: true,
        equity_min: usd(500_000.0),
        volume_min: usd(100_000.0),
        term_min_years: 1,
        term_max_years: 30,
        cost_share: 0.01,
        redeem_premium: 0.02,
        earnings_months: 12,
        grades: vec![
            grade("aaa", 0.15, 10.0, 0.004),
            grade("bbb", 0.45, 3.0, 0.018),
            grade("b", 0.65, 1.2, 0.05),
        ],
        ai_term_years: 10,
        ai_advantage_min: 0.005,
        provenance: Default::default(),
    };
    catalog
}

/// A game whose player earned 50 000 in a closed month: 600 000 a year before interest.
fn game() -> Game {
    let mut game = new_game(with_bonds(test_support::trading()));
    let player = game.player();
    let next = game.state().date.first_of_next_month();
    let ledger = &mut game.state_mut().companies[player.index()].ledger;
    ledger.income(
        CostType::Revenue,
        CostCenter::default(),
        Account::Cash,
        usd(50_000.0),
    );
    ledger.close_month(next);
    game
}

fn base(game: &Game) -> f64 {
    crate::finance::base_rate(game.catalog(), game.state().date)
}

#[test]
fn the_grade_follows_debt_and_coverage() {
    let game = game();
    let (catalog, date) = (game.catalog(), game.state().date);
    let c = &game.state().companies[game.player().index()];
    let s = bonds::standing(catalog, c);
    assert_eq!(s.ebit, Some(usd(600_000.0)));
    assert_eq!(s.debt, Money::ZERO);
    // Small: the best grade; half a million: 32 % debt, the middle grade.
    let (g, coupon) = bonds::grade_for(catalog, c, usd(100_000.0), date, 0.0).unwrap();
    assert_eq!((g, coupon), (0, base(&game) + 0.004));
    let (g, coupon) = bonds::grade_for(catalog, c, usd(500_000.0), date, 0.0).unwrap();
    assert_eq!(g, 1);
    assert!((coupon - (base(&game) + 0.018)).abs() < 1e-12);
    // Beyond 65 % debt no investor buys: A / (1.05 M + A) = 0.65 at A = 1.95 M.
    assert!(bonds::grade_for(catalog, c, usd(2_000_000.0), date, 0.0).is_none());
    let max = bonds::max_amount(catalog, c, date, 0.0);
    assert!((max.to_usd() - 1_950_000.0).abs() < 10.0, "{max:?}");
    // The finance department saves a part of the spread.
    let (_, cheaper) = bonds::grade_for(catalog, c, usd(500_000.0), date, 0.5).unwrap();
    assert!((cheaper - (base(&game) + 0.009)).abs() < 1e-12);
}

#[test]
fn a_bond_pays_coupons_and_is_repaid() {
    let mut game = game();
    let player = game.player();
    let cash = game.state().companies[player.index()].ledger.cash();
    game.apply(Command::IssueBond {
        amount: usd(500_000.0),
        years: 5,
    })
    .unwrap();
    let c = &game.state().companies[player.index()];
    assert_eq!(c.ledger.cash(), cash + usd(500_000.0) - usd(5_000.0));
    assert_eq!(c.ledger.balance(Account::Bonds), usd(500_000.0));
    assert_eq!(crate::ranking::equity(c), cash - usd(5_000.0));
    let bond = c.bonds[0].clone();
    assert_eq!(bond.grade, "bbb");
    assert_eq!(bond.maturity, Date::new(1905, 1, 1).unwrap());
    // January's coupon: a twelfth of 4.8 %.
    days(&mut game, 31);
    let c = &game.state().companies[player.index()];
    let interest = c.ledger.months.last().unwrap().by_type[&CostType::Interest];
    assert_eq!(interest, -usd(500_000.0).scale(bond.coupon / 12.0));
    assert!(c.ledger.is_balanced());
    // Due at the end of the month it matures in.
    let date = game.state().date;
    game.state_mut().companies[player.index()].bonds[0].maturity = date.add_days(10);
    let before = game.state().companies[player.index()].ledger.cash();
    let mut repaid = false;
    for _ in 0..31 {
        let report = game.advance(RoundLength::Day, |_| {});
        repaid |= report.messages.iter().any(|m| m.key == keys::BOND_REPAID);
    }
    assert!(repaid);
    let c = &game.state().companies[player.index()];
    assert!(c.bonds.is_empty());
    assert_eq!(c.ledger.balance(Account::Bonds), Money::ZERO);
    assert!(c.ledger.cash() < before - usd(499_000.0));
    assert!(c.ledger.is_balanced());
}

#[test]
fn bonds_have_limits() {
    let mut fresh = new_game(with_bonds(test_support::trading()));
    // Without a closed month no investor knows the company.
    assert_eq!(
        fresh.apply(Command::IssueBond {
            amount: usd(100_000.0),
            years: 5
        }),
        Err(CommandError::NoBondInvestors { max: Money::ZERO })
    );
    let mut game = game();
    assert_eq!(
        game.apply(Command::IssueBond {
            amount: usd(100_000.0),
            years: 31
        }),
        Err(CommandError::BondTerm { min: 1, max: 30 })
    );
    assert_eq!(
        game.apply(Command::IssueBond {
            amount: usd(50_000.0),
            years: 5
        }),
        Err(CommandError::BondTooSmall {
            min: usd(100_000.0)
        })
    );
    assert!(matches!(
        game.apply(Command::IssueBond {
            amount: usd(3_000_000.0),
            years: 5
        }),
        Err(CommandError::NoBondInvestors { max }) if max > usd(1_900_000.0)
    ));
    let mut catalog = with_bonds(test_support::trading());
    catalog.bonds.equity_min = usd(5_000_000.0);
    let mut small = new_game(catalog);
    assert_eq!(
        small.apply(Command::IssueBond {
            amount: usd(100_000.0),
            years: 5
        }),
        Err(CommandError::BondCompanyTooSmall {
            min: usd(5_000_000.0)
        })
    );
    let mut plain = new_game(test_support::trading());
    assert_eq!(
        plain.apply(Command::IssueBond {
            amount: usd(100_000.0),
            years: 5
        }),
        Err(CommandError::NoBonds)
    );
}

#[test]
fn bonds_are_bought_back_above_par() {
    let mut game = game();
    let player = game.player();
    game.apply(Command::IssueBond {
        amount: usd(200_000.0),
        years: 10,
    })
    .unwrap();
    assert_eq!(
        game.apply(Command::RedeemBond { bond: 1 }),
        Err(CommandError::UnknownBond)
    );
    let cash = game.state().companies[player.index()].ledger.cash();
    game.apply(Command::RedeemBond { bond: 0 }).unwrap();
    let c = &game.state().companies[player.index()];
    assert_eq!(c.ledger.cash(), cash - usd(204_000.0));
    assert_eq!(c.ledger.year.by_type[&CostType::Interest], -usd(4_000.0));
    assert!(c.bonds.is_empty());
    assert!(c.ledger.is_balanced());
}

#[test]
fn the_ai_takes_a_bond_where_it_is_cheaper() {
    let game = game();
    let (state, catalog) = (game.state(), game.catalog());
    let me = game.player();
    let c = &state.companies[me.index()];
    let amount = usd(300_000.0);
    let (_, coupon) = bonds::grade_for(catalog, c, amount, state.date, 0.0).unwrap();
    let loan = crate::finance::loan_rate(catalog, c, amount, state.date, 0.0);
    let expected = (coupon + 0.005 <= loan).then_some(10);
    assert_eq!(bonds::ai_prefers(state, catalog, me, amount), expected);
    // Below the smallest volume never.
    assert_eq!(bonds::ai_prefers(state, catalog, me, usd(50_000.0)), None);
}

#[test]
fn bonds_survive_saving() {
    let mut game = game();
    game.apply(Command::IssueBond {
        amount: usd(300_000.0),
        years: 3,
    })
    .unwrap();
    days(&mut game, 20);
    let mut loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(game.state_hash(), loaded.state_hash());
    days(&mut game, 40);
    days(&mut loaded, 40);
    assert_eq!(game.state_hash(), loaded.state_hash());
}
