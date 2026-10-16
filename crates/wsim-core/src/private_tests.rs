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
        research_mandatory: false,
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
    assert!(crate::private::salary_total(state) > Money::ZERO);
    assert!(
        state
            .person
            .history
            .iter()
            .any(|e| matches!(e.kind, LifeEventKind::Founded { .. }))
    );
    // A second founding (PE5): the first stays main company, the person its CEO.
    let country = game.catalog().countries.id("AAA").unwrap();
    game.apply(Command::FoundCompany {
        name: "Zweite AG".into(),
        form: StartForm::Investor,
        country,
        capital: usd(20_000.0),
    })
    .unwrap();
    let state = game.state();
    assert_eq!(state.main_company, Some(id));
    assert_eq!(state.person.ceo, Some(id));
    assert_eq!(crate::private::controlled(state).len(), 2);
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
            company: None,
            amount: max + usd(1.0)
        }),
        Err(CommandError::SalaryTooHigh { max })
    );
    game.apply(Command::SetPersonSalary {
        company: None,
        amount: max,
    })
    .unwrap();
    assert_eq!(crate::private::salary_total(game.state()), max);
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
    // Founded at the start: the account begins empty, the company keeps the game going.
    let mut game = new_game(true, 100_000.0);
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

#[test]
fn a_bonus_leaves_the_company_and_is_taxed_as_income() {
    let mut game = new_game(false, 1_000_000.0);
    let id = found(&mut game, 200_000.0);
    let cash = game.state().companies[id.index()].ledger.cash();
    let before = balance(&game);
    let bonus = |amount: f64| Command::PayBonus {
        company: id,
        amount: usd(amount),
    };
    assert_eq!(
        game.apply(bonus(1e9)),
        Err(CommandError::NotEnoughCash { needed: usd(1e9) })
    );
    game.apply(bonus(10_000.0)).unwrap();
    assert_eq!(
        game.state().companies[id.index()].ledger.cash(),
        cash - usd(10_000.0)
    );
    assert_eq!(balance(&game), before + usd(10_000.0));
    to_next_month(&mut game);
    assert_eq!(flow(&game, PrivateFlow::Bonus), usd(10_000.0));
    // Taxed with the salary at the month's end.
    assert!(flow(&game, PrivateFlow::IncomeTax) < Money::ZERO);
    assert!(game.state().companies[id.index()].ledger.is_balanced());
}

#[test]
fn the_income_tax_is_progressive_with_brackets_and_flat_without() {
    let game = new_game(true, 1_000_000.0);
    let mut c = catalog();
    let flat = private::earned_tax(&c, game.state(), usd(100_000.0));
    assert_eq!(flat, usd(20_000.0));
    c.person.income_brackets = vec![(0.0, 0.5), (1.0, 1.5)];
    let unit = crate::management::yearly_wage(&c, game.state(), game.state().person.home);
    let low = private::earned_tax(&c, game.state(), usd(unit * 0.5));
    let high = private::earned_tax(&c, game.state(), usd(unit * 4.0));
    // The lower bracket: half the base rate; above it the rate rises.
    assert!((low.to_usd() - unit * 0.5 * 0.1).abs() < 1.0);
    assert!(high.to_usd() / (unit * 4.0) > low.to_usd() / (unit * 0.5));
    assert_eq!(
        private::earned_tax(&c, game.state(), Money::ZERO),
        Money::ZERO
    );
}

#[test]
fn the_capital_tax_falls_back_to_the_income_tax_rate() {
    let game = new_game(false, 100_000.0);
    let mut c = catalog();
    assert_eq!(private::capital_tax_rate(&c, game.state()), 0.2);
    c.person.capital_tax = Some(c.person.income_tax.clone());
    c.person.capital_tax.as_mut().unwrap().default =
        Some(crate::time_series::TimeSeries::new(vec![(1900, 0.1), (2100, 0.1)]).unwrap());
    assert_eq!(private::capital_tax_rate(&c, game.state()), 0.1);
}

#[test]
fn the_wealth_classes_add_up_to_the_net_worth() {
    let mut game = new_game(true, 1_000_000.0);
    to_next_month(&mut game);
    let w = private::wealth(game.catalog(), game.state());
    let sum: Money = w.classes().iter().map(|(_, v)| *v).sum();
    assert_eq!(sum, w.total());
    let view = views::person(&game).money.expect("money");
    let shown: f64 = view.classes.iter().map(|(_, v)| *v).sum();
    assert!((shown - view.wealth_usd).abs() < 0.01);
    assert_eq!(view.classes.len(), 8);
}

fn wellbeing_model() -> crate::catalog::WellbeingModel {
    use crate::catalog::{BackgroundEffect, WellbeingModel};
    let none = BackgroundEffect::default();
    WellbeingModel {
        backgrounds: [
            BackgroundEffect {
                research: 0.05,
                ..none
            },
            BackgroundEffect {
                loan_rate: -0.005,
                ..none
            },
            BackgroundEffect {
                loyalty: 0.1,
                ..none
            },
            none,
        ],
        cap: 0.1,
        experience_per_year: 0.1,
        time_default: [60, 15, 10, 15],
        min_firm: [20.0, 10.0, 80.0],
        delay_days: 30.0,
        health_start: 85.0,
        aging_from: 40,
        aging_per_year: 0.01,
        overload_from: 70.0,
        overload_loss: 2.0,
        rest_gain: 2.0,
        lifestyle_gain: [-0.2, 0.0, 0.2, 0.3],
        care_gain: 0.2,
        healthy_from: 80.0,
        mortality_at_zero: 3.0,
        ill_below: 35.0,
        ill_chance: 1.0,
        ill_months: 3,
    }
}

fn wellbeing_game(background: Option<crate::state::Background>) -> Game {
    let mut c = catalog();
    c.person.wellbeing = Some(wellbeing_model());
    let catalog = Arc::new(c);
    let mut s = settings(&catalog, true, 1_000_000.0);
    s.person.background = background;
    Game::new(catalog, s).unwrap()
}

fn months(game: &mut Game, n: u32) {
    for _ in 0..n {
        to_next_month(game);
    }
}

#[test]
fn the_time_budget_must_add_up_and_a_short_firm_share_shortens_concerns() {
    let mut game = wellbeing_game(None);
    let bad = Command::SetTimeBudget {
        firm: 50,
        family: 10,
        network: 10,
        rest: 10,
    };
    assert_eq!(game.apply(bad), Err(CommandError::TimeBudgetSum));
    assert_eq!(
        crate::wellbeing::delay_days(game.catalog(), game.state()),
        0
    );
    game.apply(Command::SetTimeBudget {
        firm: 10,
        family: 30,
        network: 30,
        rest: 30,
    })
    .unwrap();
    let least = crate::wellbeing::min_firm(game.catalog(), game.state());
    assert_eq!(least, 30.0);
    // (30 − 10) / 30 of 30 days.
    assert_eq!(
        crate::wellbeing::delay_days(game.catalog(), game.state()),
        20
    );
    let view = views::person(&game).wellbeing.unwrap();
    assert_eq!(view.time, [10, 30, 30, 30]);
}

#[test]
fn overload_wears_the_health_down_and_rest_builds_it_up() {
    let mut busy = wellbeing_game(None);
    busy.apply(Command::SetTimeBudget {
        firm: 95,
        family: 5,
        network: 0,
        rest: 0,
    })
    .unwrap();
    let mut calm = wellbeing_game(None);
    calm.apply(Command::SetTimeBudget {
        firm: 30,
        family: 20,
        network: 0,
        rest: 50,
    })
    .unwrap();
    months(&mut busy, 6);
    months(&mut calm, 6);
    let hb = busy.state().person.health.unwrap();
    let hc = calm.state().person.health.unwrap();
    assert!(hb < 85.0 && hc > 85.0, "{hb} {hc}");
    assert!(hc <= 100.0);
}

#[test]
fn poor_health_raises_the_mortality_and_brings_an_illness_that_blocks_answers() {
    let mut game = wellbeing_game(None);
    let healthy = crate::wellbeing::mortality_factor(game.catalog(), game.state());
    assert_eq!(healthy, 1.0);
    game.state_mut().person.health = Some(10.0);
    let weak = crate::wellbeing::mortality_factor(game.catalog(), game.state());
    assert!(weak > 2.0, "{weak}");
    game.state_mut().person.health = Some(0.0);
    game.apply(Command::SetTimeBudget {
        firm: 90,
        family: 10,
        network: 0,
        rest: 0,
    })
    .unwrap();
    let keys = to_next_month(&mut game);
    assert!(keys.iter().any(|k| k == keys::PERSON_ILL), "{keys:?}");
    let today = game.state().date;
    assert!(crate::wellbeing::is_ill(game.state(), today));
    let view = views::person(&game).wellbeing.unwrap();
    assert!(view.ill_until.is_some());
    // Recovered and rested, the person is back.
    game.state_mut().person.health = Some(90.0);
    months(&mut game, 4);
    assert!(!crate::wellbeing::is_ill(game.state(), game.state().date));
}

#[test]
fn backgrounds_give_capped_effects_that_grow_with_experience() {
    use crate::state::Background;
    let mut tinkerer = wellbeing_game(Some(Background::Tinkerer));
    let start = crate::wellbeing::effect(tinkerer.catalog(), tinkerer.state()).research;
    assert!((start - 0.025).abs() < 1e-12);
    months(&mut tinkerer, 24);
    let later = crate::wellbeing::effect(tinkerer.catalog(), tinkerer.state()).research;
    assert!(later > start && later <= 0.05 + 1e-12, "{later}");
    let company = tinkerer.player();
    assert!(crate::wellbeing::research_factor(tinkerer.catalog(), tinkerer.state(), company) > 1.0);
    let merchant = wellbeing_game(Some(Background::Merchant));
    let plain = wellbeing_game(None);
    let c = merchant.player();
    assert!(
        private::loan_rate_offset(merchant.catalog(), merchant.state(), c)
            < private::loan_rate_offset(plain.catalog(), plain.state(), c)
    );
}

#[test]
fn the_wellbeing_replays_from_the_journal() {
    let mut game = wellbeing_game(Some(crate::state::Background::Heir));
    game.apply(Command::SetTimeBudget {
        firm: 80,
        family: 10,
        network: 5,
        rest: 5,
    })
    .unwrap();
    months(&mut game, 8);
    let settings = game.state().settings.clone();
    let again = Game::replay(game.catalog().clone(), settings, game.journal()).unwrap();
    assert_eq!(again.state_hash(), game.state_hash());
}

pub(crate) fn asset_model() -> crate::catalog::AssetModel {
    use crate::catalog::{AssetModel, FundTerms};
    AssetModel {
        deposit_premium: 0.01,
        deposit_months: [3, 120],
        deposit_penalty: 0.5,
        bond_premium: 0.005,
        bond_months: [12, 360],
        bond_duration_factor: 0.8,
        bond_spread: 0.01,
        gold_price: crate::time_series::TimeSeries::new(vec![(1900, 1000.0), (2100, 1000.0)])
            .unwrap(),
        gold_crisis_beta: 0.8,
        gold_noise: 0.0,
        gold_reversion: 0.1,
        gold_spread: 0.01,
        gold_fee: 0.005,
        property: None,
        services: None,
        stakes: Some(crate::catalog::StakeRules {
            thresholds: vec![0.03, 0.05, 0.10, 0.25, 0.30, 0.50],
            announce_premium: vec![0.01, 0.02, 0.03],
            blocking: 0.25,
            offer_from: 0.30,
            offer_premium: 0.10,
            offer_acceptance: 0.4,
        }),
        funds: [
            FundTerms {
                from: 1900,
                cost: 0.015,
                load: 0.05,
                fee: 0.005,
            },
            FundTerms {
                from: 1990,
                cost: 0.002,
                load: 0.0,
                fee: 0.005,
            },
        ],
    }
}

fn asset_game() -> Game {
    let mut c = catalog();
    c.person.assets = Some(asset_model());
    let catalog = Arc::new(c);
    // A person without a company holds its money on the account.
    let s = settings(&catalog, false, 1_000_000.0);
    let mut game = Game::new(catalog, s).unwrap();
    game.advance(RoundLength::Month, |_| {});
    game
}

fn invest(game: &mut Game, action: crate::savings::Investment) -> Result<(), CommandError> {
    game.apply(Command::Invest { action }).map(|_| ())
}

#[test]
fn a_deposit_pays_interest_and_comes_back_and_a_break_costs_interest() {
    use crate::savings::Investment;
    let mut game = asset_game();
    let start = balance(&game);
    invest(
        &mut game,
        Investment::OpenDeposit {
            amount: usd(100_000.0),
            months: 3,
        },
    )
    .unwrap();
    assert_eq!(balance(&game), start - usd(100_000.0));
    // Out of range terms are refused.
    assert_eq!(
        invest(
            &mut game,
            Investment::OpenDeposit {
                amount: usd(1.0),
                months: 1
            }
        ),
        Err(CommandError::NotPossible)
    );
    assert_eq!(
        invest(
            &mut game,
            Investment::OpenDeposit {
                amount: usd(1e12),
                months: 12
            }
        )
        .unwrap_err()
        .clone(),
        CommandError::NotEnoughPrivateMoney {
            needed: usd(1e12) - balance(&game)
        }
    );
    let w = private::wealth(game.catalog(), game.state());
    assert!(w.safe >= usd(100_000.0));
    months(&mut game, 1);
    let paid = game.state().person.portfolio.deposits[0].paid;
    assert!(paid > Money::ZERO);
    // Break after one month: the principal less half of the interest paid.
    let before = balance(&game);
    invest(&mut game, Investment::BreakDeposit { index: 0 }).unwrap();
    assert_eq!(balance(&game) - before, usd(100_000.0) - paid.scale(0.5));
    // A second deposit runs to its end.
    invest(
        &mut game,
        Investment::OpenDeposit {
            amount: usd(50_000.0),
            months: 3,
        },
    )
    .unwrap();
    months(&mut game, 4);
    assert!(game.state().person.portfolio.deposits.is_empty());
}

#[test]
fn gold_costs_the_spread_and_fees_and_a_gain_is_taxed() {
    use crate::savings::Investment;
    let mut game = asset_game();
    let price = game.state().assets.gold_price;
    assert!(price > Money::ZERO);
    let start = balance(&game);
    invest(
        &mut game,
        Investment::BuyGold {
            amount: usd(100_000.0),
        },
    )
    .unwrap();
    invest(&mut game, Investment::SellGold { share: 1.0 }).unwrap();
    let lost = start - balance(&game);
    // Fees both ways and the spread: about 0.5 % + 0.5 % + 1 %.
    assert!(lost > usd(1_500.0) && lost < usd(2_600.0), "{lost:?}");
    assert_eq!(game.state().person.portfolio.gold_ounces, 0.0);
    // A rise of the price brings a gain that is taxed.
    invest(
        &mut game,
        Investment::BuyGold {
            amount: usd(100_000.0),
        },
    )
    .unwrap();
    game.state_mut().assets.gold_price = price.scale(1.5);
    let before = balance(&game);
    invest(&mut game, Investment::SellGold { share: 1.0 }).unwrap();
    let got = balance(&game) - before;
    // 150 000 less fees and spread, less 20 % tax on the gain.
    assert!(got > usd(135_000.0) && got < usd(140_000.0), "{got:?}");
    let tax = game
        .state()
        .person
        .account
        .month
        .get(&PrivateFlow::GainTax)
        .copied()
        .unwrap_or(Money::ZERO);
    assert!(tax < Money::ZERO);
}

#[test]
fn funds_follow_the_index_less_costs_and_the_classes_add_up() {
    use crate::savings::{FundKind, Investment};
    let mut game = asset_game();
    game.state_mut().stock.index = 100.0;
    months(&mut game, 1);
    // The index fund opens in 1990 only.
    assert_eq!(
        invest(
            &mut game,
            Investment::BuyFund {
                fund: FundKind::Index,
                amount: usd(1_000.0)
            }
        ),
        Err(CommandError::NotPossible)
    );
    invest(
        &mut game,
        Investment::BuyFund {
            fund: FundKind::Managed,
            amount: usd(100_000.0),
        },
    )
    .unwrap();
    let pos = game.state().person.portfolio.funds[0];
    // 5 % load and 0.5 % fee are gone from the money paid in.
    assert!((pos.cost.to_usd() - 94_500.0).abs() < 1.0, "{pos:?}");
    game.state_mut().stock.index *= 1.2;
    months(&mut game, 1);
    let value = crate::savings::funds_value(game.state());
    assert!(
        value > usd(94_500.0 * 1.15) && value < usd(94_500.0 * 1.21),
        "{value:?}"
    );
    let w = private::wealth(game.catalog(), game.state());
    let sum: Money = w.classes().iter().map(|(_, v)| *v).sum();
    assert_eq!(sum, w.total());
    assert!(w.funds > Money::ZERO);
}

#[test]
fn a_savings_plan_buys_every_month_and_ends_when_the_account_is_short() {
    use crate::savings::{FundKind, Investment};
    let mut game = asset_game();
    invest(
        &mut game,
        Investment::SetPlan {
            fund: FundKind::Managed,
            amount: usd(10_000.0),
        },
    )
    .unwrap();
    months(&mut game, 3);
    let units = game.state().person.portfolio.funds[0].units;
    assert!(units > 0.0);
    assert!(game.state().person.portfolio.plan.is_some());
    // Too big for the account: it ends with a message.
    invest(
        &mut game,
        Investment::SetPlan {
            fund: FundKind::Managed,
            amount: usd(1e11),
        },
    )
    .unwrap();
    let keys = to_next_month(&mut game);
    assert!(
        keys.iter().any(|k| k == keys::SAVINGS_PLAN_ENDED),
        "{keys:?}"
    );
    assert!(game.state().person.portfolio.plan.is_none());
}

#[test]
fn the_investments_replay_from_the_journal() {
    use crate::savings::{FundKind, Investment};
    let mut game = asset_game();
    invest(
        &mut game,
        Investment::BuyGold {
            amount: usd(20_000.0),
        },
    )
    .unwrap();
    invest(
        &mut game,
        Investment::BuyBond {
            face: usd(30_000.0),
            months: 24,
        },
    )
    .unwrap();
    invest(
        &mut game,
        Investment::SetPlan {
            fund: FundKind::Managed,
            amount: usd(5_000.0),
        },
    )
    .unwrap();
    months(&mut game, 6);
    let settings = game.state().settings.clone();
    let again = Game::replay(game.catalog().clone(), settings, game.journal()).unwrap();
    assert_eq!(again.state_hash(), game.state_hash());
}

fn property_model() -> crate::catalog::PropertyModel {
    use crate::catalog::{PropertyModel, PropertyTerms};
    let t = |years, rent_yield, vacancy, upkeep, decay| PropertyTerms {
        price_years: years,
        rent_yield,
        vacancy,
        upkeep,
        decay,
    };
    PropertyModel {
        kinds: [
            t(5.0, 0.045, 0.04, 0.012, 2.0),
            t(25.0, 0.05, 0.05, 0.014, 2.0),
            t(30.0, 0.055, 0.08, 0.015, 2.5),
            t(20.0, 0.06, 0.06, 0.012, 2.0),
            t(8.0, 0.01, 0.0, 0.002, 0.0),
        ],
        costs: 0.09,
        equity_min: 0.2,
        mortgage_premium: 0.015,
        repay_year: 0.03,
        rent_band: [0.8, 1.2],
        vacancy_slope: 1.5,
        damage_chance: 0.0,
        damage_cost: 0.02,
        damage_points: 8.0,
        reno_cost: 0.12,
        reno_points: 40.0,
        reno_months: 3,
        site_effect: 0.02,
        site_effect_max: 0.08,
        sale_discount: 0.03,
        distress_discount: 0.15,
        condition_weight: 0.3,
        offer_months: 3,
        offer_spread: [0.0, 0.0],
        offer_quality: [1.0, 1.0],
        offer_condition: [80.0, 80.0],
        offer_countries: 5,
        company_relief: 0.15,
        company_relief_max: 0.5,
    }
}

fn property_game() -> Game {
    property_game_with(false)
}

fn property_game_with(found: bool) -> Game {
    let mut c = catalog();
    let mut a = asset_model();
    a.property = Some(property_model());
    c.person.assets = Some(a);
    let catalog = Arc::new(c);
    let s = settings(&catalog, found, 5_000_000.0);
    let mut game = Game::new(catalog, s).unwrap();
    // The market opens in the first quarter month.
    for _ in 0..3 {
        to_next_month(&mut game);
        if !game.state().assets.offers.is_empty() {
            break;
        }
    }
    game
}

fn offer_of(game: &Game, kind: crate::property::PropertyKind) -> u32 {
    game.state()
        .assets
        .offers
        .iter()
        .find(|o| o.kind == kind)
        .expect("offer")
        .id
}

#[test]
fn a_property_is_bought_with_a_mortgage_earns_rent_and_adds_to_the_wealth() {
    use crate::property::PropertyKind;
    use crate::savings::Investment;
    let mut game = property_game();
    assert_eq!(game.state().assets.offers.len(), 5);
    let id = offer_of(&game, PropertyKind::Apartment);
    // Too little equity is refused.
    assert_eq!(
        invest(
            &mut game,
            Investment::BuyProperty {
                offer: id,
                mortgage: 0.95
            }
        ),
        Err(CommandError::NotPossible)
    );
    let before = private::wealth(game.catalog(), game.state()).total();
    invest(
        &mut game,
        Investment::BuyProperty {
            offer: id,
            mortgage: 0.8,
        },
    )
    .unwrap();
    let p = game.state().person.portfolio.properties[0].clone();
    assert!(p.mortgage.is_some());
    // The wealth falls by the costs of buying only.
    let after = private::wealth(game.catalog(), game.state());
    let costs = p.paid.scale(0.09);
    // (The offer was priced a day earlier, so the wage may differ a little.)
    assert!(
        (before - after.total() - costs).abs() < p.paid.scale(0.002),
        "{before:?} {:?}",
        after.total()
    );
    let sum: Money = after.classes().iter().map(|(_, v)| *v).sum();
    assert_eq!(sum, after.total());
    // A month later: rent comes in, interest and repayment go out.
    months(&mut game, 1);
    let flows = |k: PrivateFlow| {
        game.state()
            .person
            .account
            .months
            .last()
            .and_then(|m| m.flows.get(&k).copied())
            .unwrap_or(Money::ZERO)
    };
    assert!(flows(PrivateFlow::Rent) > Money::ZERO);
    assert!(flows(PrivateFlow::MortgageInterest) < Money::ZERO);
    assert!(flows(PrivateFlow::MortgageRepay) < Money::ZERO);
    let p = &game.state().person.portfolio.properties[0];
    assert!(p.mortgage.unwrap().balance < p.paid.scale(0.8));
    // The sale repays the mortgage.
    invest(&mut game, Investment::SellProperty { index: 0 }).unwrap();
    assert!(game.state().person.portfolio.properties.is_empty());
}

#[test]
fn a_higher_rent_brings_more_vacancy_and_the_band_is_kept() {
    use crate::property::PropertyKind;
    use crate::savings::Investment;
    let mut game = property_game();
    let id = offer_of(&game, PropertyKind::Office);
    invest(
        &mut game,
        Investment::BuyProperty {
            offer: id,
            mortgage: 0.0,
        },
    )
    .unwrap();
    let m = property_model();
    let base = crate::property::vacancy(&m, &game.state().person.portfolio.properties[0]);
    assert_eq!(
        invest(
            &mut game,
            Investment::SetRent {
                index: 0,
                factor: 1.5
            }
        ),
        Err(CommandError::NotPossible)
    );
    invest(
        &mut game,
        Investment::SetRent {
            index: 0,
            factor: 1.2,
        },
    )
    .unwrap();
    let high = crate::property::vacancy(&m, &game.state().person.portfolio.properties[0]);
    assert!((high - base - 0.3).abs() < 1e-9, "{base} {high}");
}

#[test]
fn a_renovation_lifts_the_condition_and_stops_the_rent_for_a_while() {
    use crate::property::PropertyKind;
    use crate::savings::Investment;
    let mut game = property_game();
    let id = offer_of(&game, PropertyKind::Apartment);
    invest(
        &mut game,
        Investment::BuyProperty {
            offer: id,
            mortgage: 0.0,
        },
    )
    .unwrap();
    let c0 = game.state().person.portfolio.properties[0].condition;
    invest(&mut game, Investment::Renovate { index: 0 }).unwrap();
    let p = &game.state().person.portfolio.properties[0];
    assert!(p.condition > c0 && p.renovating_until.is_some());
    assert_eq!(
        invest(&mut game, Investment::Renovate { index: 0 }),
        Err(CommandError::NotPossible)
    );
    months(&mut game, 1);
    let rent = game
        .state()
        .person
        .account
        .months
        .last()
        .and_then(|m| m.flows.get(&PrivateFlow::Rent).copied())
        .unwrap_or(Money::ZERO);
    assert_eq!(rent, Money::ZERO);
    months(&mut game, 4);
    assert!(
        game.state().person.portfolio.properties[0]
            .renovating_until
            .is_none()
    );
}

#[test]
fn what_the_account_cannot_pay_forces_a_sale() {
    use crate::property::PropertyKind;
    use crate::savings::Investment;
    let mut game = property_game();
    let id = offer_of(&game, PropertyKind::Multi);
    invest(
        &mut game,
        Investment::BuyProperty {
            offer: id,
            mortgage: 0.8,
        },
    )
    .unwrap();
    // Empty the account but for a rest.
    let rest = balance(&game);
    game.state_mut().person.account.balance = usd(1.0);
    let _ = rest;
    let keys = to_next_month(&mut game);
    assert!(
        keys.iter().any(|k| k == keys::PROPERTY_DISTRESS),
        "{keys:?}"
    );
    assert!(game.state().person.portfolio.properties.is_empty());
    assert!(!game.state().person.account.balance.is_negative());
}

#[test]
fn the_properties_replay_from_the_journal() {
    use crate::property::PropertyKind;
    use crate::savings::Investment;
    let mut game = property_game();
    let id = offer_of(&game, PropertyKind::Hall);
    invest(
        &mut game,
        Investment::BuyProperty {
            offer: id,
            mortgage: 0.5,
        },
    )
    .unwrap();
    invest(
        &mut game,
        Investment::SetRent {
            index: 0,
            factor: 1.1,
        },
    )
    .unwrap();
    months(&mut game, 7);
    let settings = game.state().settings.clone();
    let again = Game::replay(game.catalog().clone(), settings, game.journal()).unwrap();
    assert_eq!(again.state_hash(), game.state_hash());
}

fn service_model() -> crate::catalog::ServiceModel {
    crate::catalog::ServiceModel {
        housing: vec![[0.05, 0.25, 0.4], [0.09, 0.5, 0.7]],
        tax: vec![[0.5, 0.05], [1.5, 0.12]],
        manager_fee: 0.01,
        dishonest_chance: 1.0,
        embezzlement: 0.1,
        reserve_months: 12,
        strategies: [
            [0.5, 0.2, 0.3, 0.0],
            [0.2, 0.2, 0.5, 0.1],
            [0.0, 0.0, 0.9, 0.1],
        ],
    }
}

fn service_game() -> Game {
    let mut c = catalog();
    let mut a = asset_model();
    a.property = Some(property_model());
    a.services = Some(service_model());
    c.person.assets = Some(a);
    let catalog = Arc::new(c);
    let s = settings(&catalog, false, 5_000_000.0);
    let mut game = Game::new(catalog, s).unwrap();
    for _ in 0..3 {
        to_next_month(&mut game);
        if !game.state().assets.offers.is_empty() {
            break;
        }
    }
    game
}

fn last_flow(game: &Game, kind: PrivateFlow) -> Money {
    game.state()
        .person
        .account
        .months
        .last()
        .and_then(|m| m.flows.get(&kind).copied())
        .unwrap_or(Money::ZERO)
}

#[test]
fn the_property_management_costs_a_fee_and_cuts_vacancy() {
    use crate::property::PropertyKind;
    use crate::savings::Investment;
    let run = |level: u32| {
        let mut game = service_game();
        let id = offer_of(&game, PropertyKind::Office);
        invest(
            &mut game,
            Investment::BuyProperty {
                offer: id,
                mortgage: 0.0,
            },
        )
        .unwrap();
        invest(&mut game, Investment::SetHousingManagement { level }).unwrap();
        months(&mut game, 1);
        (
            last_flow(&game, PrivateFlow::Rent),
            last_flow(&game, PrivateFlow::PropertyCost),
        )
    };
    let (rent0, cost0) = run(0);
    let (rent2, cost2) = run(2);
    assert!(rent2 > rent0, "{rent2:?} {rent0:?}");
    // The fee of 9 % of the rent on top of the upkeep.
    assert!(cost2 < cost0 - rent2.scale(0.05), "{cost2:?} {cost0:?}");
    let mut game = service_game();
    assert_eq!(
        invest(&mut game, Investment::SetHousingManagement { level: 9 }),
        Err(CommandError::NotPossible)
    );
}

#[test]
fn a_tax_advisor_lowers_the_tax_and_charges_his_fee() {
    use crate::savings::Investment;
    let run = |level: u32| {
        let mut game = service_game();
        invest(&mut game, Investment::SetTaxAdvisor { level }).unwrap();
        months(&mut game, 2);
        (
            last_flow(&game, PrivateFlow::IncomeTax),
            last_flow(&game, PrivateFlow::AdvisorFee),
        )
    };
    let (tax0, fee0) = run(0);
    let (tax2, fee2) = run(2);
    assert_eq!(fee0, Money::ZERO);
    assert!(fee2 < Money::ZERO);
    // The tax is a negative flow: less of it with the advisor.
    assert!(tax0 < Money::ZERO && tax2 > tax0, "{tax0:?} {tax2:?}");
}

#[test]
fn the_asset_manager_invests_by_his_strategy_charges_and_may_embezzle() {
    use crate::savings::Investment;
    let mut game = service_game();
    invest(
        &mut game,
        Investment::HireAssetManager {
            strategy: 1,
            budget: 0.5,
        },
    )
    .unwrap();
    assert_eq!(
        invest(
            &mut game,
            Investment::HireAssetManager {
                strategy: 7,
                budget: 0.5
            }
        ),
        Err(CommandError::NotPossible)
    );
    // An honest one.
    game.state_mut()
        .person
        .services
        .manager
        .as_mut()
        .unwrap()
        .honesty = 100;
    months(&mut game, 2);
    let p = &game.state().person.portfolio;
    assert!(!p.deposits.is_empty() && !p.bonds.is_empty() && p.gold_ounces > 0.0);
    assert!(p.funds.iter().any(|f| f.units > 0.0));
    assert!(last_flow(&game, PrivateFlow::AdvisorFee) < Money::ZERO);
    // A dishonest one takes a tenth of the funds and is gone.
    let units: f64 = game
        .state()
        .person
        .portfolio
        .funds
        .iter()
        .map(|f| f.units)
        .sum();
    game.state_mut()
        .person
        .services
        .manager
        .as_mut()
        .unwrap()
        .honesty = 0;
    let keys = to_next_month(&mut game);
    assert!(
        keys.iter().any(|k| k == keys::MANAGER_EMBEZZLED),
        "{keys:?}"
    );
    assert!(game.state().person.services.manager.is_none());
    let after: f64 = game
        .state()
        .person
        .portfolio
        .funds
        .iter()
        .map(|f| f.units)
        .sum();
    assert!(after < units);
}

#[test]
fn flats_let_to_a_site_relieve_its_wages_and_an_office_is_leased_to_the_firm() {
    use crate::property::PropertyKind;
    use crate::savings::Investment;
    let mut game = property_game_with(true);
    let country = game.catalog().countries.id("AAA").unwrap();
    game.apply(Command::FoundSite {
        country,
        kind: crate::catalog::SiteType::Warehouse,
    })
    .unwrap();
    assert!(!game.state().sites.is_empty());
    game.state_mut().person.account.balance = usd(4_000_000.0);
    let buy = |game: &mut Game, kind| {
        let id = offer_of(game, kind);
        invest(
            game,
            Investment::BuyProperty {
                offer: id,
                mortgage: 0.0,
            },
        )
        .unwrap();
    };
    buy(&mut game, PropertyKind::Apartment);
    buy(&mut game, PropertyKind::Office);
    // Land and unknown sites are refused.
    assert_eq!(
        invest(
            &mut game,
            Investment::AssignProperty {
                index: 0,
                site: Some(99),
            },
        ),
        Err(CommandError::NotPossible)
    );
    invest(
        &mut game,
        Investment::AssignProperty {
            index: 0,
            site: Some(0),
        },
    )
    .unwrap();
    invest(
        &mut game,
        Investment::AssignProperty {
            index: 1,
            site: Some(0),
        },
    )
    .unwrap();
    months(&mut game, 1);
    let site = &game.state().sites[0];
    assert!(
        (site.housing_relief - 0.15).abs() < 1e-9,
        "{}",
        site.housing_relief
    );
    let rent = game
        .state()
        .person
        .account
        .months
        .last()
        .and_then(|m| m.flows.get(&PrivateFlow::Rent).copied())
        .unwrap_or(Money::ZERO);
    assert!(rent > Money::ZERO);
    // Ending the use takes the relief away again.
    invest(
        &mut game,
        Investment::AssignProperty {
            index: 0,
            site: None,
        },
    )
    .unwrap();
    months(&mut game, 1);
    assert_eq!(game.state().sites[0].housing_relief, 0.0);
}

// --- PV6a: standing, clubs, contacts, possessions ---

fn standing_model() -> crate::catalog::StandingModel {
    use crate::catalog::{ClubTerms, ContactRole, GoodTerms};
    crate::catalog::StandingModel {
        target_base: 10.0,
        target_lifestyle: [0.0, 5.0, 15.0, 30.0],
        target_per_company: 5.0,
        rate: 0.5,
        donation_points: 10.0,
        donation_cap: 15.0,
        interest_per_point: -0.0001,
        contact_years: [20.0, 30.0],
        care_gain: 1.0,
        care_decay: 1.0,
        banker_interest: -0.01,
        broker_discount: 0.1,
        tip_chance: 0.0,
        clubs: vec![
            ClubTerms {
                key: "stammtisch".into(),
                standing: 0.0,
                lifestyle: 0,
                fee: 0.5,
                // More than one contact a month: one every month.
                contacts_year: 1_200.0,
                roles: vec![ContactRole::Banker, ContactRole::Broker],
            },
            ClubTerms {
                key: "herrenclub".into(),
                standing: 60.0,
                lifestyle: 2,
                fee: 3.0,
                contacts_year: 1.0,
                roles: vec![ContactRole::Politician],
            },
        ],
        goods: vec![GoodTerms {
            key: "yacht".into(),
            from: 1900,
            price: 2.0,
            upkeep: 0.06,
            representation: 12.0,
        }],
    }
}

fn standing_game() -> Game {
    let mut c = catalog();
    c.person.assets = Some(asset_model());
    c.person.standing = Some(standing_model());
    let catalog = Arc::new(c);
    let s = settings(&catalog, false, 2_000_000.0);
    let mut game = Game::new(catalog, s).unwrap();
    game.advance(RoundLength::Month, |_| {});
    game
}

#[test]
fn a_donation_buys_standing_up_to_a_cap_and_the_standing_drifts_to_its_target() {
    use crate::savings::Investment;
    let mut game = standing_game();
    let catalog = game.catalog().clone();
    let before = crate::standing::standing(&catalog, game.state());
    let cash = balance(&game);
    // A small donation gives points in proportion; a huge one stops at the cap.
    invest(
        &mut game,
        Investment::Donate {
            amount: usd(1_000_000.0),
        },
    )
    .unwrap();
    let after = crate::standing::standing(&catalog, game.state());
    assert!((after - before - 15.0).abs() < 1e-9, "{before} {after}");
    assert!(balance(&game) < cash);
    assert_eq!(
        invest(&mut game, Investment::Donate { amount: usd(1e12) }),
        Err(CommandError::NotEnoughPrivateMoney {
            needed: usd(1e12) - balance(&game)
        })
    );
    // Months later it has fallen back towards the target of lifestyle and base.
    months(&mut game, 6);
    let goal = crate::standing::target(&catalog, game.state());
    let now = crate::standing::standing(&catalog, game.state());
    assert!((now - goal).abs() < 1.0, "{now} vs {goal}");
}

#[test]
fn clubs_admit_by_standing_and_bring_contacts_with_their_use() {
    use crate::savings::Investment;
    let mut game = standing_game();
    let catalog = game.catalog().clone();
    // The herrenclub wants more standing and a better life.
    assert_eq!(
        invest(&mut game, Investment::JoinClub { club: 1 }),
        Err(CommandError::NotPossible)
    );
    invest(&mut game, Investment::JoinClub { club: 0 }).unwrap();
    assert_eq!(
        invest(&mut game, Investment::JoinClub { club: 0 }),
        Err(CommandError::NotPossible)
    );
    months(&mut game, 4);
    let contacts = &game.state().person.circle.contacts;
    assert!(contacts.len() >= 3, "a contact a month: {}", contacts.len());
    assert!(
        flow(&game, PrivateFlow::CircleCost) < Money::ZERO,
        "the fee"
    );
    // A banker lowers the loan rate, a broker the prices of properties.
    let offset = crate::standing::loan_rate_offset(&catalog, game.state());
    assert!(offset < 0.0);
    let bank_only = -0.0001 * crate::standing::standing(&catalog, game.state());
    if game
        .state()
        .person
        .circle
        .contacts
        .iter()
        .any(|c| c.role == crate::catalog::ContactRole::Banker)
    {
        assert!(offset < bank_only, "{offset} {bank_only}");
    }
    // Leaving ends the fee.
    invest(&mut game, Investment::LeaveClub { club: 0 }).unwrap();
    assert!(game.state().person.circle.clubs.is_empty());
}

#[test]
fn possessions_cost_upkeep_add_to_the_target_and_sell_with_a_loss() {
    use crate::savings::Investment;
    let mut game = standing_game();
    let catalog = game.catalog().clone();
    let target = crate::standing::target(&catalog, game.state());
    let cash = balance(&game);
    invest(&mut game, Investment::BuyGood { good: 0 }).unwrap();
    let price = cash - balance(&game);
    assert!(price > Money::ZERO);
    let raised = crate::standing::target(&catalog, game.state());
    assert!((raised - target - 12.0).abs() < 1e-9);
    months(&mut game, 1);
    assert!(
        flow(&game, PrivateFlow::CircleCost) < Money::ZERO,
        "the upkeep"
    );
    let before = balance(&game);
    invest(&mut game, Investment::SellGood { index: 0 }).unwrap();
    let back = balance(&game) - before;
    assert!(back > Money::ZERO && back < price);
    assert_eq!(
        invest(&mut game, Investment::SellGood { index: 0 }),
        Err(CommandError::NotPossible)
    );
}

// --- PV6b: relationship, personal events, nepotism ---

pub(crate) fn personal_model() -> crate::catalog::PersonalModel {
    use crate::catalog::{EventCondition, EventOption, EventTerms, PersonalModel};
    PersonalModel {
        chance_factor: 1.0,
        wait_days: 45,
        bond_start: 70.0,
        bond_holds_at: 15.0,
        bond_rate: 1.5,
        bond_warn: 35.0,
        bond_separate: 12.0,
        share_without: 0.5,
        share_with: 0.15,
        contract_cost: 3.0,
        nepotism_malus: 15,
        low_health: 40.0,
        pending_max: 2,
        events: vec![EventTerms {
            key: "partner_zeit".into(),
            condition: EventCondition::Married,
            chance_year: 1.0,
            options: vec![
                EventOption {
                    key: "urlaub".into(),
                    cost: 8.0,
                    bond: 12.0,
                    ..EventOption::default()
                },
                EventOption {
                    key: "spaeter".into(),
                    bond: -8.0,
                    ..EventOption::default()
                },
            ],
        }],
    }
}

fn personal_game(family_time: u32) -> Game {
    let mut c = catalog();
    c.person.assets = Some(asset_model());
    let mut w = wellbeing_model();
    w.time_default = [100 - family_time - 10 - 15, family_time, 10, 15];
    c.person.wellbeing = Some(w);
    c.person.personal = Some(personal_model());
    let catalog = Arc::new(c);
    let s = settings(&catalog, false, 2_000_000.0);
    let mut game = Game::new(catalog, s).unwrap();
    game.state_mut().person.married = false;
    game.advance(RoundLength::Month, |_| {});
    game
}

#[test]
fn a_wedding_asks_for_a_contract_and_an_unanswered_question_means_none() {
    let mut game = personal_game(15);
    let catalog = game.catalog().clone();
    let date = game.state().date;
    game.state_mut().person.married = true;
    crate::personal::married(game.state_mut(), &catalog, date);
    let event = game.state().person.life.pending[0];
    assert_eq!(event.kind, crate::personal::CONTRACT);
    let cash = balance(&game);
    game.apply(Command::AnswerEvent {
        event: event.id,
        option: 0,
    })
    .unwrap();
    assert!(game.state().person.life.contract);
    assert!(balance(&game) < cash);
    assert!(game.state().person.life.pending.is_empty());
    // Answering twice, or with an unknown option, is refused.
    assert_eq!(
        game.apply(Command::AnswerEvent {
            event: event.id,
            option: 0
        }),
        Err(CommandError::NotPossible)
    );
    // A new wedding, left unanswered: the contract lapses.
    game.state_mut().person.life.contract = false;
    let date = game.state().date;
    crate::personal::married(game.state_mut(), &catalog, date);
    months(&mut game, 3);
    assert!(!game.state().person.life.contract);
    assert!(
        game.state()
            .person
            .life
            .pending
            .iter()
            .all(|e| e.kind != crate::personal::CONTRACT)
    );
}

#[test]
fn an_answered_event_changes_the_relationship_and_costs_money() {
    let mut game = personal_game(15);
    game.state_mut().person.married = true;
    game.state_mut().person.life.bond = Some(50.0);
    months(&mut game, 1);
    let event = game
        .state()
        .person
        .life
        .pending
        .iter()
        .find(|e| e.kind != crate::personal::CONTRACT)
        .copied()
        .expect("an event (chance 1)");
    let cash = balance(&game);
    game.apply(Command::AnswerEvent {
        event: event.id,
        option: 0,
    })
    .unwrap();
    assert!(balance(&game) < cash);
    let bond = crate::personal::bond(game.catalog(), game.state()).unwrap();
    assert!(bond > 50.0, "{bond}");
}

#[test]
fn too_little_time_for_the_family_ends_in_a_separation_that_shares_the_wealth() {
    let mut game = personal_game(0);
    game.state_mut().person.married = true;
    let catalog = game.catalog().clone();
    let before = private::wealth(&catalog, game.state()).total();
    let mut keys = Vec::new();
    for _ in 0..40 {
        keys.extend(to_next_month(&mut game));
        if !game.state().person.married {
            break;
        }
    }
    assert!(!game.state().person.married, "the couple separated");
    assert!(keys.iter().any(|k| k == keys::SEPARATED));
    assert!(keys.iter().any(|k| k == keys::BOND_WARNING));
    let after = private::wealth(&catalog, game.state()).total();
    // Without a contract about half of the free wealth is gone.
    assert!(after < before.scale(0.7), "{before:?} → {after:?}");
    assert!(game.state().person.life.bond.is_none());
}
