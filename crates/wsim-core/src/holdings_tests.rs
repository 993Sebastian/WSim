//! Scenario tests for shares bought and sold by the person, several companies and the end
//! of the game (PE5).

use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{Catalog, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::holdings;
use crate::ledger::{Account, CostCenter, CostType};
use crate::management_tests::usd;
use crate::message::keys;
use crate::money::Money;
use crate::save;
use crate::state::{
    AiState, CompanyId, GameSettings, Holder, LifeEventKind, PersonSettings, PrivateFlow, Stake,
    StartForm,
};

fn catalog() -> Catalog {
    let mut c = test_support::aging();
    c.person = test_support::private();
    c
}

/// The person with 3 000 000 USD founds an investment firm with a third of it.
fn game() -> Game {
    let catalog = Arc::new(catalog());
    let settings = GameSettings {
        seed: 5,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(3_000_000.0),
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
    game.apply(Command::SetPersonSalary {
        amount: Money::ZERO,
    })
    .unwrap();
    game
}

/// An AI company of founders with 1 000 000 USD equity and a profit of `profit` last year.
fn ai_company(game: &mut Game, profit: f64) -> CompanyId {
    let id = crate::trade_tests::competitor(game);
    let date = game.state().date;
    let c = &mut game.state_mut().companies[id.index()];
    c.ai = Some(AiState {
        competence: 0.5,
        aggressiveness: 0.5,
        real: None,
        next_operations: date.add_days(4000),
        staff: 0.0,
    });
    c.owners = Stake::sole(Holder::Private);
    c.ledger.income(
        CostType::Revenue,
        CostCenter::default(),
        Account::Cash,
        usd(profit),
    );
    let mut closed = c.ledger.year.clone();
    closed.by_type.insert(CostType::Revenue, usd(profit));
    c.ledger.years.push(closed);
    id
}

fn flow(game: &Game, f: PrivateFlow) -> Money {
    let a = &game.state().person.account;
    a.month.get(&f).copied().unwrap_or_default()
}

fn until(game: &mut Game, (y, m, d): (i32, u32, u32)) -> Vec<String> {
    let end = Date::new(y, m, d).unwrap();
    let mut keys = Vec::new();
    while game.state().date < end {
        let report = game.advance(RoundLength::Day, |_| {});
        keys.extend(report.messages.into_iter().map(|m| m.key));
    }
    keys
}

#[test]
fn the_person_buys_an_ai_company_leads_both_and_sells_one() {
    let mut game = game();
    let main = game.player();
    let x = ai_company(&mut game, 200_000.0);
    let catalog = game.catalog().clone();
    // Good earnings (17 % on equity): the founders ask the value plus 50 %.
    let ask = holdings::ask(&catalog, game.state(), x, 0.6);
    let value = crate::private::company_value(&catalog, game.state(), x);
    assert_eq!(ask, value.scale(0.6 * 1.5));
    assert_eq!(
        game.apply(Command::BuyStake {
            company: x,
            holder: Holder::Private,
            share: 0.6,
            price: ask - usd(1.0),
        }),
        Err(CommandError::PriceTooLow { min: ask })
    );
    assert_eq!(
        game.apply(Command::BuyStake {
            company: x,
            holder: Holder::Player,
            share: 0.1,
            price: ask,
        }),
        Err(CommandError::NotSeller)
    );
    game.apply(Command::BuyStake {
        company: x,
        holder: Holder::Private,
        share: 0.6,
        price: ask,
    })
    .unwrap();
    let s = game.state();
    // The majority makes it the person's company: no AI any more.
    assert!(crate::private::is_controlled(s, x));
    assert!(s.companies[x.index()].ai.is_none());
    assert_eq!(s.person.cost_basis[&x], ask);
    assert_eq!(flow(&game, PrivateFlow::StakeBought), -ask);
    assert!(
        s.person
            .history
            .iter()
            .any(|e| matches!(e.kind, LifeEventKind::TookControl { company } if company == x))
    );
    assert_eq!(holdings::selectable(s), vec![main, x]);
    // Both are steered: the second as main company, then back.
    game.apply(Command::SelectCompany { company: x }).unwrap();
    assert_eq!(game.player(), x);
    game.apply(Command::SetDividendPolicy {
        policy: crate::dividends::DividendPolicy::Share(0.3),
    })
    .unwrap();
    game.apply(Command::SelectCompany { company: main })
        .unwrap();
    assert_eq!(game.player(), main);
    // Selling the majority to investors: a tax on the gain, the company is AI again.
    let before = game.state().person.account.balance;
    let proceeds = holdings::bid_of_investors(&catalog, game.state(), x, 0.6);
    game.apply(Command::SellStake {
        company: x,
        share: 0.6,
    })
    .unwrap();
    let s = game.state();
    assert!(s.companies[x.index()].ai.is_some());
    assert!(!crate::private::is_controlled(s, x));
    assert_eq!(s.main_company, Some(main));
    // Bought at value · 1.5, sold at value · 0.95: a loss, no tax.
    assert!(proceeds < ask);
    assert_eq!(s.person.account.balance - before, proceeds);
    assert_eq!(flow(&game, PrivateFlow::GainTax), Money::ZERO);
    assert_eq!(s.person.cost_basis[&x], Money::ZERO);
    assert!(s.companies[x.index()].ledger.is_balanced());
    assert_eq!(
        game.apply(Command::SelectCompany { company: x }),
        Err(CommandError::NotControlled)
    );
}

#[test]
fn a_company_of_the_person_buys_shares_from_an_ai_company() {
    let mut game = game();
    let main = game.player();
    let x = ai_company(&mut game, 0.0);
    let seller = ai_company(&mut game, 0.0);
    // The seller holds 40 % it bought for 100 000.
    {
        let s = game.state_mut();
        s.companies[x.index()].owners = vec![
            Stake {
                holder: Holder::Company(seller),
                share: 0.4,
            },
            Stake {
                holder: Holder::Private,
                share: 0.6,
            },
        ];
        let l = &mut s.companies[seller.index()];
        l.ledger
            .transfer(Account::Participations, Account::Cash, usd(100_000.0));
        l.stock_cost.insert(x, usd(100_000.0));
    }
    let catalog = game.catalog().clone();
    // Poor earnings: the value plus 20 %.
    let ask = holdings::ask(&catalog, game.state(), x, 0.4);
    let value = crate::private::company_value(&catalog, game.state(), x);
    assert_eq!(ask, value.scale(0.4 * 1.2));
    game.apply(Command::BidForStake {
        company: x,
        holder: Holder::Company(seller),
        share: 0.4,
        price: ask,
    })
    .unwrap();
    let s = game.state();
    let buyer = &s.companies[main.index()];
    assert_eq!(buyer.stock_cost[&x], ask);
    assert_eq!(buyer.ledger.balance(Account::Participations), ask);
    // The seller books its gain over what it paid.
    let sold = &s.companies[seller.index()];
    assert!(!sold.stock_cost.contains_key(&x));
    assert_eq!(
        sold.ledger.year.by_type[&CostType::Investments],
        ask - usd(100_000.0)
    );
    // 40 % is no control: x stays an AI company.
    assert!(s.companies[x.index()].ai.is_some());
    for c in &s.companies {
        assert!(c.ledger.is_balanced());
    }
}

#[test]
fn a_bankrupt_main_company_leaves_the_person_as_investor() {
    let mut game = game();
    let main = game.player();
    let x = ai_company(&mut game, 0.0);
    let catalog = game.catalog().clone();
    let ask = holdings::ask(&catalog, game.state(), x, 0.2);
    game.apply(Command::BuyStake {
        company: x,
        holder: Holder::Private,
        share: 0.2,
        price: ask,
    })
    .unwrap();
    // The main company spends far more than it has.
    game.state_mut().companies[main.index()].ledger.expense(
        CostType::Other,
        CostCenter::default(),
        Account::Cash,
        usd(5_000_000.0),
    );
    let keys = until(&mut game, (1900, 3, 1));
    assert!(
        keys.iter().any(|k| k == keys::GAME_OVER_INSOLVENT),
        "{keys:?}"
    );
    assert!(keys.iter().any(|k| k == keys::MAIN_COMPANY_NONE));
    let s = game.state();
    assert!(s.companies[main.index()].bankrupt);
    assert_eq!(s.main_company, None);
    assert_eq!(s.person.ceo, None);
    assert!(!game.is_over(), "an investor goes on");
    // A new company becomes the main company again.
    let country = catalog.countries.id("AAA").unwrap();
    game.apply(Command::FoundCompany {
        name: "Neuanfang AG".into(),
        form: StartForm::Investor,
        country,
        capital: usd(100_000.0),
    })
    .unwrap();
    assert_ne!(game.main_company(), Some(main));
    assert!(game.main_company().is_some());
}

#[test]
fn without_shares_and_money_the_game_ends() {
    let mut game = game();
    let main = game.player();
    // The person sells everything and spends the money.
    game.apply(Command::SellStake {
        company: main,
        share: 1.0,
    })
    .unwrap();
    assert_eq!(game.main_company(), None);
    game.state_mut().person.account.balance = usd(10.0);
    let keys = until(&mut game, (1900, 2, 1));
    assert!(keys.iter().any(|k| k == keys::GAME_OVER_BROKE), "{keys:?}");
    assert!(game.is_over());
}

#[test]
fn trades_survive_saving() {
    let mut game = game();
    let x = ai_company(&mut game, 100_000.0);
    let catalog = game.catalog().clone();
    let ask = holdings::ask(&catalog, game.state(), x, 0.3);
    game.apply(Command::BuyStake {
        company: x,
        holder: Holder::Private,
        share: 0.3,
        price: ask,
    })
    .unwrap();
    let mut loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(game.state_hash(), loaded.state_hash());
    until(&mut game, (1900, 3, 1));
    until(&mut loaded, (1900, 3, 1));
    assert_eq!(game.state_hash(), loaded.state_hash());
}
