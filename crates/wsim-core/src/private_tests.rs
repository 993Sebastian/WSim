//! Scenario tests for the private account, founding and lifestyle (PE3) with
//! `test_support::aging` and the person's money of `test_support::private`.

use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{Catalog, test_support};
use crate::command::{Command, CommandError};
use crate::game::{Game, NewGameError};
use crate::ledger::{Account, CostType};
use crate::management_tests::usd;
use crate::message::keys;
use crate::money::Money;
use crate::private;
use crate::save;
use crate::state::{
    CompanyId, GameSettings, Holder, LifeEventKind, Lifestyle, PersonSettings, PrivateFlow, Stake,
    StartForm, Unit,
};
use crate::views;

fn catalog() -> Catalog {
    let mut c = test_support::aging();
    c.person = test_support::private();
    c
}

fn settings(catalog: &Catalog, found: bool, money: f64) -> GameSettings {
    GameSettings {
        seed: 7,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(money),
        start_form: StartForm::Workshop,
        company_name: "Hütte AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: found,
        person: PersonSettings::default(),
    }
}

fn new_game(found: bool, money: f64) -> Game {
    let catalog = Arc::new(catalog());
    let settings = settings(&catalog, found, money);
    Game::new(catalog, settings).unwrap()
}

/// Days up to and including the next month start; the message keys on the way.
fn to_next_month(game: &mut Game) -> Vec<String> {
    let mut keys = Vec::new();
    loop {
        let report = game.advance(RoundLength::Day, |_| {});
        keys.extend(report.messages.into_iter().map(|m| m.key));
        if game.state().date.day() == 1 {
            return keys;
        }
    }
}

/// An investment firm: cash and nothing else, so only the person moves its money.
fn found(game: &mut Game, capital: f64) -> CompanyId {
    let country = game.catalog().countries.id("AAA").unwrap();
    game.apply(Command::FoundCompany {
        name: "Neue AG".into(),
        form: StartForm::Investor,
        country,
        capital: usd(capital),
    })
    .unwrap();
    game.player()
}

fn balance(game: &Game) -> Money {
    game.state().person.account.balance
}

fn flow(game: &Game, kind: PrivateFlow) -> Money {
    let months = &game.state().person.account.months;
    months
        .last()
        .and_then(|m| m.flows.get(&kind))
        .copied()
        .unwrap_or(Money::ZERO)
}

#[test]
fn a_game_without_a_company_runs_its_rounds() {
    let mut game = new_game(false, 1_000_000.0);
    let state = game.state();
    assert_eq!(state.main_company, None);
    assert_eq!(balance(&game), usd(1_000_000.0));
    assert_eq!(state.person.ceo, None);
    assert!(matches!(
        state.person.history[0].kind,
        LifeEventKind::Began { .. }
    ));
    assert_eq!(
        game.apply(Command::TakeLoan {
            amount: usd(1000.0),
            years: 1
        }),
        Err(CommandError::NoCompany)
    );
    let overview = views::overview(&game);
    assert!(overview.company.is_none());
    assert_eq!(overview.person.as_ref().unwrap().cash_usd, 1_000_000.0);
    assert_eq!(overview.hints[0].message.key, keys::HINT_FOUND_COMPANY);
    for _ in 0..3 {
        to_next_month(&mut game);
    }
    // Interest less tax less the lifestyle, each month.
    let wage =
        private::academic_monthly_wage(game.catalog(), game.state(), game.state().person.home);
    assert!(wage > Money::ZERO);
    assert!(flow(&game, PrivateFlow::Interest) > Money::ZERO);
    assert!(flow(&game, PrivateFlow::IncomeTax) < Money::ZERO);
    assert!(flow(&game, PrivateFlow::Lifestyle) < Money::ZERO);
    assert!(balance(&game) < usd(1_000_000.0));
    assert_eq!(game.state().person.account.wealth.len(), 4);
    // The round report works without a company.
    let before = views::snapshot(&game);
    let report = game.advance(RoundLength::Month, |_| {});
    let view = views::round_report(&game, &report, &before);
    assert_eq!(view.cash_before_usd, 0.0);
}

#[test]
fn founding_takes_the_capital_and_its_costs() {
    let mut game = new_game(false, 1_000_000.0);
    let country = game.catalog().countries.id("AAA").unwrap();
    let attempt = |capital: f64| Command::FoundCompany {
        name: "Neue AG".into(),
        form: StartForm::Investor,
        country,
        capital: usd(capital),
    };
    assert_eq!(game.apply(attempt(0.0)), Err(CommandError::InvalidAmount));
    assert!(matches!(
        game.apply(attempt(1_000_000.0)),
        Err(CommandError::NotEnoughPrivateMoney { .. })
    ));
    let cost = private::founding_cost(game.catalog(), game.state(), country, usd(900_000.0));
    assert!(cost >= usd(4_500.0));
    let id = found(&mut game, 900_000.0);
    let state = game.state();
    assert_eq!(state.main_company, Some(id));
    assert_eq!(balance(&game), usd(100_000.0) - cost);
    let c = &state.companies[id.index()];
    assert_eq!(c.ledger.cash(), usd(900_000.0));
    assert_eq!(c.ledger.balance(Account::Equity), usd(900_000.0));
    assert_eq!(c.owners, Stake::sole(Holder::Player));
    assert_eq!(state.person.ceo, Some(id));
    assert_eq!(state.person.cost_basis[&id], usd(900_000.0));
    assert!(state.person.salary > Money::ZERO);
    assert!(
        state
            .person
            .history
            .iter()
            .any(|e| matches!(e.kind, LifeEventKind::Founded { .. }))
    );
    assert_eq!(
        game.apply(attempt(1000.0)),
        Err(CommandError::AlreadyFounded)
    );
    // The company acts from now on.
    assert!(views::overview(&game).company.is_some());
}

#[test]
fn the_money_between_person_and_company_stays_in_the_circuit() {
    let mut game = new_game(false, 1_000_000.0);
    let id = found(&mut game, 500_000.0);
    game.apply(Command::LendToCompany {
        company: id,
        amount: usd(100_000.0),
        rate: 0.06,
        years: 2,
    })
    .unwrap();
    game.apply(Command::ContributeCapital {
        company: id,
        amount: usd(50_000.0),
    })
    .unwrap();
    let total = |g: &Game| balance(g) + g.state().companies[id.index()].ledger.cash();
    assert_eq!(
        total(&game),
        usd(1_000_000.0) - {
            let s = game.state();
            -s.person.account.month[&PrivateFlow::FoundingCost]
        }
    );
    for month in 0..14 {
        let before = total(&game);
        to_next_month(&mut game);
        // Only interest on the balance, income tax and the lifestyle leave or enter.
        let outside = flow(&game, PrivateFlow::Interest)
            + flow(&game, PrivateFlow::IncomeTax)
            + flow(&game, PrivateFlow::Lifestyle);
        assert_eq!(total(&game) - before, outside, "month {month}");
        assert!(flow(&game, PrivateFlow::Salary) > Money::ZERO);
        assert!(game.state().companies[id.index()].ledger.is_balanced());
    }
    assert!(flow(&game, PrivateFlow::LoanRepaid) > Money::ZERO);
    assert!(flow(&game, PrivateFlow::LoanInterest) > Money::ZERO);
    // The salary is the board's personnel cost.
    let c = &game.state().companies[id.index()];
    let personnel = -c.ledger.months.last().unwrap().by_type[&CostType::Personnel];
    assert_eq!(personnel, flow(&game, PrivateFlow::Salary));
    assert!(!balance(&game).is_negative());
}

#[test]
fn a_loan_of_the_person_goes_back_to_it() {
    let mut game = new_game(false, 1_000_000.0);
    let id = found(&mut game, 500_000.0);
    let lend = |rate: f64, years: u32| Command::LendToCompany {
        company: id,
        amount: usd(100_000.0),
        rate,
        years,
    };
    let terms = Err(CommandError::LoanTerms {
        max_rate: 0.15,
        max_years: 30,
    });
    assert_eq!(game.apply(lend(0.2, 2)), terms);
    assert_eq!(game.apply(lend(0.05, 0)), terms);
    let before = balance(&game);
    game.apply(lend(0.05, 2)).unwrap();
    assert_eq!(balance(&game), before - usd(100_000.0));
    let loan = &game.state().companies[id.index()].loans[0];
    assert!(loan.from_person && loan.lender.is_none());
    // Paid back early by the company: the money goes to the person.
    let before = balance(&game);
    game.apply(Command::RepayLoan {
        loan: 0,
        amount: usd(40_000.0),
    })
    .unwrap();
    assert_eq!(balance(&game), before + usd(40_000.0));
    // A bank does not refinance it.
    assert_eq!(
        game.apply(Command::RefinanceLoan { loan: 0 }),
        Err(CommandError::UnknownLoan)
    );
}

#[test]
fn capital_goes_back_only_to_the_sole_owner() {
    let mut game = new_game(false, 1_000_000.0);
    let id = found(&mut game, 500_000.0);
    let withdraw = |amount: f64| Command::WithdrawCapital {
        company: id,
        amount: usd(amount),
    };
    assert!(matches!(
        game.apply(withdraw(600_000.0)),
        Err(CommandError::WithdrawalTooHigh { .. })
    ));
    let before = balance(&game);
    game.apply(withdraw(100_000.0)).unwrap();
    assert_eq!(balance(&game), before + usd(100_000.0));
    let state = game.state();
    assert_eq!(state.person.cost_basis[&id], usd(400_000.0));
    let c = &state.companies[id.index()];
    assert_eq!(c.ledger.balance(Account::Equity), usd(400_000.0));
    // With co-owners nothing goes back, and new capital raises the share.
    game.state_mut().companies[id.index()].owners = vec![
        Stake {
            holder: Holder::Player,
            share: 0.6,
        },
        Stake {
            holder: Holder::Private,
            share: 0.4,
        },
    ];
    assert_eq!(
        game.apply(withdraw(1000.0)),
        Err(CommandError::NotSoleOwner)
    );
    game.apply(Command::ContributeCapital {
        company: id,
        amount: usd(100_000.0),
    })
    .unwrap();
    let owners = &game.state().companies[id.index()].owners;
    let sum: f64 = owners.iter().map(|s| s.share).sum();
    assert!((sum - 1.0).abs() < 1e-9);
    assert!(owners[0].holder == Holder::Player && owners[0].share > 0.6);
    // The salary is capped with co-owners.
    let max = private::salary_max(game.catalog(), game.state(), id).unwrap();
    assert_eq!(
        game.apply(Command::SetPersonSalary {
            amount: max + usd(1.0)
        }),
        Err(CommandError::SalaryTooHigh { max })
    );
    game.apply(Command::SetPersonSalary { amount: max })
        .unwrap();
    assert_eq!(game.state().person.salary, max);
}

#[test]
fn the_lifestyle_changes_from_the_next_month_and_once_a_year() {
    let mut game = new_game(false, 1_000_000.0);
    let home = game.state().person.home;
    let wage = private::academic_monthly_wage(game.catalog(), game.state(), home);
    game.apply(Command::SetLifestyle {
        level: Lifestyle::Upscale,
    })
    .unwrap();
    // Still the old one this month.
    to_next_month(&mut game);
    assert_eq!(flow(&game, PrivateFlow::Lifestyle), -wage.scale(1.5));
    let next = Date::new(1900, 2, 1).unwrap();
    assert_eq!(
        private::lifestyle_at(game.catalog(), &game.state().person, next),
        Lifestyle::Upscale
    );
    to_next_month(&mut game);
    let wage = private::academic_monthly_wage(game.catalog(), game.state(), home);
    assert_eq!(flow(&game, PrivateFlow::Lifestyle), -wage.scale(4.0));
    assert_eq!(
        game.apply(Command::SetLifestyle {
            level: Lifestyle::Luxury
        }),
        Err(CommandError::LifestyleChangedRecently {
            from: Date::new(1901, 2, 1).unwrap()
        })
    );
}

#[test]
fn an_empty_account_lowers_the_lifestyle() {
    let mut game = new_game(false, 100.0);
    let keys = to_next_month(&mut game);
    assert!(
        keys.iter().any(|k| k == keys::PERSON_ACCOUNT_SHORT),
        "{keys:?}"
    );
    assert_eq!(balance(&game), Money::ZERO);
    assert!(views::overview(&game).person.unwrap().short);
    let keys = to_next_month(&mut game);
    assert!(
        keys.iter().any(|k| k == keys::PERSON_LIFESTYLE_LOWERED),
        "{keys:?}"
    );
    let today = game.state().date;
    assert_eq!(
        private::lifestyle_at(game.catalog(), &game.state().person, today),
        Lifestyle::Modest
    );
    // Lowered by the account, not chosen: the person may choose again.
    game.apply(Command::SetLifestyle {
        level: Lifestyle::Middle,
    })
    .unwrap();
}

#[test]
fn the_lifestyle_moves_loans_salaries_and_education() {
    let mut game = new_game(false, 1_000_000.0);
    let id = found(&mut game, 500_000.0);
    game.apply(Command::SetLifestyle {
        level: Lifestyle::Luxury,
    })
    .unwrap();
    to_next_month(&mut game);
    let (c, s) = (game.catalog().clone(), game.state());
    assert!((private::loan_rate_offset(&c, s, id) + 0.005).abs() < 1e-12);
    assert!((private::salary_demand_factor(&c, s, id, Unit::Board) - 0.9).abs() < 1e-12);
    assert!(
        (private::salary_demand_factor(&c, s, id, Unit::Country(c.countries.id("AAA").unwrap()))
            - 1.0)
            .abs()
            < 1e-12
    );
    // Not for a company the person does not control.
    let other = CompanyId(0);
    assert!(!private::is_controlled(s, other) || other == id);
    // Half the childhood middle class (5), half luxury (15).
    let p = &s.person;
    let born = Date::new(1900, 2, 1).unwrap().add_days(-365 * 10);
    let card = Date::new(1900, 2, 1).unwrap().add_days(365 * 10);
    let e = private::education(&c, p, born, card);
    assert!((e - 10.0).abs() < 0.1, "{e}");
}

#[test]
fn older_saves_get_the_account_of_their_company() {
    let catalog = Arc::new(catalog());
    let settings = settings(&catalog, true, 1_000_000.0);
    let mut game = Game::new(catalog.clone(), settings).unwrap();
    let id = game.player();
    let p = &mut game.state_mut().person;
    p.lifestyles.clear();
    p.cost_basis.clear();
    p.salary = Money::ZERO;
    let loaded = save::decode(&save::encode(&game), catalog).unwrap().game;
    let p = &loaded.state().person;
    assert_eq!(p.lifestyles.len(), 1);
    assert_eq!(p.cost_basis[&id], usd(1_000_000.0));
    assert_eq!(p.salary, Money::ZERO, "no salary until the player sets one");
    assert_eq!(p.account.balance, Money::ZERO);
}

#[test]
fn a_founding_replays_from_the_journal() {
    let catalog = Arc::new(catalog());
    let settings = settings(&catalog, false, 1_000_000.0);
    let mut game = Game::new(catalog.clone(), settings.clone()).unwrap();
    game.advance(RoundLength::Month, |_| {});
    found(&mut game, 300_000.0);
    game.apply(Command::SetLifestyle {
        level: Lifestyle::Modest,
    })
    .unwrap();
    game.advance(RoundLength::Month, |_| {});
    let again = Game::replay(catalog, settings, game.journal()).unwrap();
    assert_eq!(again.state_hash(), game.state_hash());
}

#[test]
fn too_little_start_money_or_an_unknown_command_target_is_refused() {
    let catalog = Arc::new(catalog());
    let mut s = settings(&catalog, false, 1.0);
    s.start_capital = Money::ZERO;
    assert_eq!(
        Game::new(catalog.clone(), s).unwrap_err(),
        NewGameError::StartCapital
    );
    let mut game = new_game(false, 1_000_000.0);
    assert_eq!(
        game.apply(Command::ContributeCapital {
            company: CompanyId(0),
            amount: usd(1.0),
        }),
        Err(CommandError::UnknownCompany(CompanyId(0)))
    );
}
