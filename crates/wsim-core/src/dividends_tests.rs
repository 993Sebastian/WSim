//! Scenario tests for dividends (PE4): policy, limits, owners and tax, special dividend,
//! AI companies by character, the CEO's proposal.

use std::sync::Arc;

use crate::calendar::RoundLength;
use crate::catalog::{Catalog, test_support};
use crate::command::{Command, CommandError};
use crate::decision::Topic;
use crate::dividends::{self, DividendPolicy};
use crate::game::Game;
use crate::ledger::{Account, CostCenter, CostType};
use crate::management::ConcernAnswer;
use crate::management_tests::{hire_sharp, usd};
use crate::message::keys;
use crate::money::Money;
use crate::save;
use crate::state::{
    AiState, CompanyId, ConcernStatus, GameSettings, Holder, PersonSettings, Position, PrivateFlow,
    Role, Stake, StartForm, Unit,
};

fn catalog() -> Catalog {
    let mut c = test_support::aging();
    c.person = test_support::private();
    c
}

/// A game of the person with an investment firm as main company (no sites: no reserve).
fn game() -> Game {
    let catalog = Arc::new(catalog());
    let settings = GameSettings {
        seed: 11,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(2_000_000.0),
        start_form: StartForm::Investor,
        company_name: "Haus AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures: 1.0,
        tariff_dynamics: 1.0,
        event_effects: true,
        found_at_start: false,
        person: PersonSettings::default(),
    };
    let mut game = Game::new(catalog, settings).unwrap();
    let country = game.catalog().countries.id("AAA").unwrap();
    game.apply(Command::FoundCompany {
        name: "Haus AG".into(),
        form: StartForm::Investor,
        country,
        capital: usd(1_000_000.0),
    })
    .unwrap();
    // Without a salary only the test's bookings move the result.
    game.apply(Command::SetPersonSalary {
        amount: Money::ZERO,
    })
    .unwrap();
    game
}

/// Another company with these owners and a profit of `profit` in the running year.
fn company(game: &mut Game, owners: Vec<Stake>, profit: f64) -> CompanyId {
    let id = crate::trade_tests::competitor(game);
    let c = &mut game.state_mut().companies[id.index()];
    c.owners = owners;
    earn(c, profit);
    id
}

fn earn(c: &mut crate::state::Company, profit: f64) {
    let amount = usd(profit.abs());
    if profit >= 0.0 {
        c.ledger.income(
            CostType::Revenue,
            CostCenter::default(),
            Account::Cash,
            amount,
        );
    } else {
        c.ledger.expense(
            CostType::Other,
            CostCenter::default(),
            Account::Cash,
            amount,
        );
    }
}

fn stake(holder: Holder, share: f64) -> Stake {
    Stake { holder, share }
}

/// Days until `month`/`day` of `year`; the message keys on the way.
fn until(game: &mut Game, (year, month, day): (i32, u32, u32)) -> Vec<String> {
    let mut keys = Vec::new();
    let end = crate::calendar::Date::new(year, month, day).unwrap();
    while game.state().date < end {
        let report = game.advance(RoundLength::Day, |_| {});
        keys.extend(report.messages.into_iter().map(|m| m.key));
    }
    keys
}

fn flow(game: &Game, f: PrivateFlow) -> Money {
    crate::private::last_months(game.state())
        .get(&f)
        .copied()
        .unwrap_or_default()
}

fn last(game: &Game, id: CompanyId) -> Option<(i32, Money)> {
    game.state().companies[id.index()].dividend.last
}

#[test]
fn a_policy_pays_its_share_to_the_owners_after_the_withholding_tax() {
    let mut game = game();
    let main = game.player();
    let x = company(
        &mut game,
        vec![
            stake(Holder::Player, 0.3),
            stake(Holder::Company(main), 0.6),
            stake(Holder::Private, 0.1),
        ],
        200_000.0,
    );
    game.state_mut().companies[x.index()].dividend.policy = Some(DividendPolicy::Share(0.5));
    let main_reserves = |g: &Game| {
        g.state().companies[main.index()]
            .ledger
            .balance(Account::RetainedEarnings)
    };
    let before = main_reserves(&game);
    until(&mut game, (1901, 1, 31));
    assert_eq!(last(&game, x), None, "not before the end of January");
    let profit = dividends::year_profit(&game.state().companies[x.index()]);
    assert!(profit > Money::ZERO && profit < usd(200_000.0), "after tax");
    let keys = until(&mut game, (1901, 2, 1));
    let d = profit.scale(0.5);
    assert_eq!(last(&game, x), Some((1900, d)));
    // The main company holds 60 %: free of tax into its reserves, no income.
    assert!(keys.iter().any(|k| k == keys::DIVIDEND_RECEIVED));
    let group = main_reserves(&game) - before;
    assert!((group - d.scale(0.6)).abs() <= usd(0.01), "{group:?} {d:?}");
    let s = game.state();
    assert!(
        !s.companies[main.index()]
            .ledger
            .year
            .by_type
            .contains_key(&CostType::Investments)
    );
    // The person: 30 % before the withholding tax of the headquarters' country.
    let tax = s
        .countries
        .get(s.companies[x.index()].headquarters)
        .dividend_tax;
    assert!(tax > 0.0);
    let gross = flow(&game, PrivateFlow::Dividend);
    assert!((gross - d.scale(0.3)).abs() <= usd(0.01));
    assert!((flow(&game, PrivateFlow::DividendTax) + gross.scale(tax)).abs() <= usd(0.01));
    for c in &game.state().companies {
        assert!(c.ledger.is_balanced());
    }
}

#[test]
fn a_company_up_to_half_books_the_dividend_as_taxed_income() {
    let mut game = game();
    let main = game.player();
    let x = company(
        &mut game,
        vec![
            stake(Holder::Company(main), 0.4),
            stake(Holder::Investors, 0.6),
        ],
        100_000.0,
    );
    game.state_mut().companies[x.index()].dividend.policy = Some(DividendPolicy::Share(1.0));
    until(&mut game, (1901, 2, 1));
    let (_, d) = last(&game, x).unwrap();
    let s = game.state();
    let tax = s
        .countries
        .get(s.companies[x.index()].headquarters)
        .dividend_tax;
    let income = s.companies[main.index()].ledger.year.by_type[&CostType::Investments];
    assert!((income - d.scale(0.4).scale(1.0 - tax)).abs() <= usd(0.01));
}

#[test]
fn the_retained_earnings_and_the_reserve_cut_the_dividend() {
    let mut game = game();
    let main = game.player();
    earn(&mut game.state_mut().companies[main.index()], 50_000.0);
    game.apply(Command::SetDividendPolicy {
        policy: DividendPolicy::Amount(usd(5_000_000.0)),
    })
    .unwrap();
    let keys = until(&mut game, (1901, 2, 1));
    assert!(keys.iter().any(|k| k == keys::DIVIDEND_CUT));
    assert!(keys.iter().any(|k| k == keys::DIVIDEND_PAID));
    let c = &game.state().companies[main.index()];
    let (_, paid) = c.dividend.last.unwrap();
    // Only the reserves: the year's profit after tax.
    assert!(paid > Money::ZERO && paid < usd(50_000.0));
    assert!(c.ledger.balance(Account::RetainedEarnings) <= usd(0.01));
    assert!(c.ledger.is_balanced());
}

#[test]
fn nothing_is_paid_after_a_loss_or_without_a_policy() {
    let mut game = game();
    let loss = company(&mut game, vec![stake(Holder::Player, 1.0)], -80_000.0);
    game.state_mut().companies[loss.index()].dividend.policy = Some(DividendPolicy::Share(0.5));
    let none = company(&mut game, vec![stake(Holder::Player, 1.0)], 80_000.0);
    until(&mut game, (1901, 2, 1));
    assert_eq!(last(&game, loss), None);
    assert_eq!(last(&game, none), None, "the player's default is 0 %");
    assert_eq!(flow(&game, PrivateFlow::Dividend), Money::ZERO);
}

#[test]
fn a_special_dividend_is_paid_at_once_within_the_limits() {
    let mut game = game();
    let main = game.player();
    earn(&mut game.state_mut().companies[main.index()], 30_000.0);
    let max = dividends::payable(game.catalog(), game.state(), main);
    assert_eq!(max, usd(30_000.0));
    assert_eq!(
        game.apply(Command::SpecialDividend {
            amount: usd(30_001.0)
        }),
        Err(CommandError::DividendTooHigh { max })
    );
    assert_eq!(
        game.apply(Command::SpecialDividend {
            amount: Money::ZERO
        }),
        Err(CommandError::InvalidAmount)
    );
    let before = game.state().person.account.balance;
    game.apply(Command::SpecialDividend {
        amount: usd(10_000.0),
    })
    .unwrap();
    let s = game.state();
    let tax = s
        .countries
        .get(s.companies[main.index()].headquarters)
        .dividend_tax;
    let got = s.person.account.balance - before;
    assert!((got - usd(10_000.0).scale(1.0 - tax)).abs() <= usd(0.01));
    assert_eq!(last(&game, main), Some((1900, usd(10_000.0))));
}

#[test]
fn a_policy_is_checked() {
    let mut game = game();
    assert_eq!(
        game.apply(Command::SetDividendPolicy {
            policy: DividendPolicy::Share(1.2)
        }),
        Err(CommandError::ShareOutOfRange { max: 1.0 })
    );
    assert_eq!(
        game.apply(Command::SetDividendPolicy {
            policy: DividendPolicy::Amount(usd(-1.0))
        }),
        Err(CommandError::InvalidAmount)
    );
    // The short form of K1 sets a share.
    game.apply(Command::SetDividend { payout: 0.25 }).unwrap();
    let c = &game.state().companies[game.player().index()];
    assert_eq!(dividends::policy(c), Some(DividendPolicy::Share(0.25)));
}

#[test]
fn ai_companies_pay_by_their_character() {
    let mut game = game();
    let date = game.state().date;
    let ai = |aggressiveness| AiState {
        competence: 0.5,
        aggressiveness,
        real: None,
        next_operations: date.add_days(400),
        staff: 0.0,
    };
    let cautious = company(&mut game, vec![stake(Holder::Player, 1.0)], 100_000.0);
    let bold = company(&mut game, vec![stake(Holder::Player, 1.0)], 100_000.0);
    {
        let s = game.state_mut();
        s.companies[cautious.index()].ai = Some(ai(0.0));
        s.companies[bold.index()].ai = Some(ai(1.0));
    }
    until(&mut game, (1901, 1, 1));
    let m = game.catalog().finance_model.dividends.clone();
    let profit = dividends::year_profit(&game.state().companies[cautious.index()]);
    let catalog = game.catalog().clone();
    let state = game.state();
    assert_eq!(
        dividends::wanted(&catalog, state, cautious),
        profit.scale(m.payout_max)
    );
    assert_eq!(
        dividends::wanted(&catalog, state, bold),
        profit.scale(m.payout_min)
    );
    until(&mut game, (1901, 2, 1));
    assert!(last(&game, cautious).unwrap().1 > last(&game, bold).unwrap().1);
}

#[test]
fn ai_companies_keep_their_money_after_a_loss_or_when_cash_is_short() {
    let mut game = game();
    let date = game.state().date;
    let poor = company(&mut game, vec![stake(Holder::Private, 1.0)], 100_000.0);
    let losing = company(&mut game, vec![stake(Holder::Private, 1.0)], -100_000.0);
    for id in [poor, losing] {
        game.state_mut().companies[id.index()].ai = Some(AiState {
            competence: 0.5,
            aggressiveness: 0.0,
            real: None,
            next_operations: date.add_days(400),
            staff: 0.0,
        });
    }
    until(&mut game, (1901, 1, 1));
    // Spent its cash on stock: below any reserve of running costs.
    let c = &mut game.state_mut().companies[poor.index()];
    let spend = c.ledger.cash() + usd(1.0);
    c.ledger.transfer(Account::Inventory, Account::Cash, spend);
    let catalog = game.catalog().clone();
    assert_eq!(dividends::wanted(&catalog, game.state(), poor), Money::ZERO);
    assert_eq!(
        dividends::wanted(&catalog, game.state(), losing),
        Money::ZERO
    );
    until(&mut game, (1901, 2, 1));
    assert_eq!(last(&game, poor), None);
    assert_eq!(last(&game, losing), None);
}

#[test]
fn the_ceo_proposes_and_an_expired_concern_keeps_the_policy() {
    let mut game = game();
    let main = game.player();
    earn(&mut game.state_mut().companies[main.index()], 1_000_000.0);
    // A board needs a site.
    crate::trade_tests::warehouse(&mut game, main, "AAA", 0.0);
    let ceo = Position::new(Unit::Board, Role::Head);
    let m = hire_sharp(&mut game, ceo);
    game.state_mut().managers.get_mut(&m).unwrap().risk = 0;
    until(&mut game, (1901, 1, 2));
    let concern = game
        .state()
        .concerns
        .iter()
        .find(|c| c.decision.topic == Topic::Dividend)
        .cloned()
        .expect("the CEO proposes a dividend");
    assert_eq!(concern.company, main);
    assert_eq!(concern.recommended, 1);
    assert!(concern.deadline < crate::calendar::Date::new(1901, 1, 31).unwrap());
    let profit = dividends::year_profit(&game.state().companies[main.index()]);
    let q = game.catalog().finance_model.dividends.payout_max;
    assert_eq!(concern.options[0].amount, Money::ZERO);
    assert_eq!(concern.options[1].amount, profit.scale(q));
    // Nobody answers: the policy stays (0 %), nothing is paid.
    until(&mut game, (1901, 2, 1));
    let c = game
        .state()
        .concerns
        .iter()
        .find(|c| c.id == concern.id)
        .unwrap();
    assert_eq!(c.status, ConcernStatus::Expired);
    assert_eq!(last(&game, main), None);
}

#[test]
fn choosing_the_ceos_proposal_sets_the_policy_and_pays() {
    let mut game = game();
    let main = game.player();
    earn(&mut game.state_mut().companies[main.index()], 1_000_000.0);
    crate::trade_tests::warehouse(&mut game, main, "AAA", 0.0);
    let m = hire_sharp(&mut game, Position::new(Unit::Board, Role::Head));
    game.state_mut().managers.get_mut(&m).unwrap().risk = 100;
    until(&mut game, (1901, 1, 2));
    let id = game
        .state()
        .concerns
        .iter()
        .find(|c| c.decision.topic == Topic::Dividend)
        .unwrap()
        .id;
    game.apply(Command::AnswerConcern {
        concern: id,
        answer: ConcernAnswer::Choose(1),
    })
    .unwrap();
    let q = game.catalog().finance_model.dividends.payout_min;
    let c = &game.state().companies[main.index()];
    assert_eq!(dividends::policy(c), Some(DividendPolicy::Share(q)));
    until(&mut game, (1901, 2, 1));
    let profit = dividends::year_profit(&game.state().companies[main.index()]);
    assert_eq!(last(&game, main), Some((1900, profit.scale(q))));
    assert!(flow(&game, PrivateFlow::Dividend) > Money::ZERO);
}

#[test]
fn dividends_survive_saving_and_replay() {
    let mut game = game();
    let main = game.player();
    earn(&mut game.state_mut().companies[main.index()], 40_000.0);
    game.apply(Command::SetDividendPolicy {
        policy: DividendPolicy::Share(0.5),
    })
    .unwrap();
    until(&mut game, (1900, 12, 1));
    let mut loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    until(&mut game, (1901, 2, 2));
    until(&mut loaded, (1901, 2, 2));
    assert_eq!(game.state_hash(), loaded.state_hash());
    assert!(last(&game, main).is_some());
}
