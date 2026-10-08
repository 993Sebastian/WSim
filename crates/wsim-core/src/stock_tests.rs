//! Tests of the stock market (K1).

use crate::catalog::{Catalog, StockModel, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::keys;
use crate::money::Money;
use crate::save;
use crate::state::{CompanyId, Holder, Stake};
use crate::stock::{self, Listing};
use crate::trade_tests::{competitor, days, new_game, usd};

fn with_stock(mut catalog: Catalog, crises: Vec<(i32, u32, f64)>) -> Catalog {
    catalog.stock = StockModel {
        enabled: true,
        book_weight: 0.4,
        pe: 12.0,
        book_floor: 0.3,
        assumed_return: 0.08,
        earnings_months: 12,
        sentiment_volatility: 0.0,
        sentiment_reversion: 0.1,
        inertia: 0.5,
        noise: 0.0,
        crises,
        ipo_equity_min: usd(100_000.0),
        ipo_share_max: 0.4,
        ipo_discount: 0.1,
        ipo_cost_share: 0.05,
        dividend_month: 1,
        ai_payout: 0.4,
        dividend_cash_max: 0.5,
        trade_premium: 0.02,
        trade_discount: 0.02,
        trade_impact: 0.5,
        trade_share_max: 0.5,
        start_equity_min: usd(1e12),
        start_free_float: 0.6,
        ai_ipo_chance: 0.0,
        ai_ipo_share: 0.3,
        takeover_premium: 0.3,
        takeover_cost_share: 0.02,
        buyback_share_max: 0.1,
        ai_portfolio_cash_share: 0.0,
        ai_portfolio_stake_max: 0.1,
        ai_undervaluation: 0.25,
        ai_takeover_chance: 0.0,
        ai_takeover_cash_share: 0.5,
        provenance: Default::default(),
    };
    catalog
}

fn game() -> Game {
    new_game(with_stock(test_support::trading(), Vec::new()))
}

fn cash(game: &Game, c: CompanyId) -> Money {
    game.state().companies[c.index()].ledger.cash()
}

/// A listed company outside the player's group with all its shares at investors.
fn listed_competitor(game: &mut Game, value: f64) -> CompanyId {
    let id = competitor(game);
    let date = game.state().date;
    let c = &mut game.state_mut().companies[id.index()];
    c.owners = vec![Stake {
        holder: Holder::Investors,
        share: 1.0,
    }];
    c.listing = Some(Listing::new(usd(value), date));
    id
}

#[test]
fn going_public_sells_new_shares() {
    let mut game = game();
    let player = game.player();
    let before_cash = cash(&game, player);
    let catalog = game.catalog().clone();
    let target = stock::target(
        &catalog,
        &game.state().stock,
        &game.state().companies[player.index()],
    );
    game.apply(Command::GoPublic { share: 0.25 }).unwrap();
    let c = &game.state().companies[player.index()];
    let value_before = target.scale(0.9);
    let proceeds = value_before.scale(0.25 / 0.75);
    assert_eq!(
        c.ledger.cash(),
        before_cash + proceeds - proceeds.scale(0.05)
    );
    assert!((stock::stake(c, Holder::Player) - 0.75).abs() < 1e-9);
    assert!((stock::stake(c, Holder::Investors) - 0.25).abs() < 1e-9);
    assert_eq!(c.listing.as_ref().unwrap().value, value_before + proceeds);
    assert!(c.ledger.is_balanced());
    assert_eq!(
        game.apply(Command::GoPublic { share: 0.1 }),
        Err(CommandError::AlreadyListed)
    );
    assert_eq!(
        game.apply(Command::IssueShares { share: 0.5 }),
        Err(CommandError::ShareOutOfRange { max: 0.4 })
    );
    game.apply(Command::IssueShares { share: 0.2 }).unwrap();
    let c = &game.state().companies[player.index()];
    assert!((stock::stake(c, Holder::Player) - 0.6).abs() < 1e-9);
    // New shares raise the count: 0.75 of them were the old ones.
    assert_eq!(c.listing.as_ref().unwrap().shares, 1_250_000);
    // Since K3 the player may lose the majority: 0.6 · 0.7 = 0.42.
    assert!(stock::loses_majority(c, 0.3));
    game.apply(Command::IssueShares { share: 0.3 }).unwrap();
    let c = &game.state().companies[player.index()];
    assert!((stock::stake(c, Holder::Player) - 0.42).abs() < 1e-9);
}

#[test]
fn small_companies_stay_private() {
    let mut catalog = with_stock(test_support::trading(), Vec::new());
    catalog.stock.ipo_equity_min = usd(1e9);
    let mut game = new_game(catalog);
    assert_eq!(
        game.apply(Command::GoPublic { share: 0.2 }),
        Err(CommandError::EquityTooLow { min: usd(1e9) })
    );
    let mut plain = new_game(test_support::trading());
    assert_eq!(
        plain.apply(Command::GoPublic { share: 0.2 }),
        Err(CommandError::NoStockMarket)
    );
}

#[test]
fn values_follow_the_target_and_crises_hit_all() {
    let mut game = new_game(with_stock(test_support::trading(), vec![(1900, 2, 0.5)]));
    let player = game.player();
    game.apply(Command::GoPublic { share: 0.2 }).unwrap();
    // Far above its value: the market brings it down.
    let high = usd(1e9);
    game.state_mut().companies[player.index()]
        .listing
        .as_mut()
        .unwrap()
        .value = high;
    // January closes, and with February 1900 comes the crisis of the data.
    let mut crisis = false;
    for _ in 0..31 {
        let report = game.advance(crate::calendar::RoundLength::Day, |_| {});
        crisis |= report.messages.iter().any(|m| m.key == keys::STOCK_CRISIS);
    }
    let l = game.state().companies[player.index()]
        .listing
        .clone()
        .unwrap();
    assert!(l.value < high);
    assert_eq!(l.history.len(), 2);
    assert!(game.state().stock.index < 100.0);
    assert!(crisis);
    assert!(game.state().stock.sentiment < libm::log(0.6));
}

#[test]
fn shares_of_others_are_bought_and_sold() {
    let mut game = game();
    let player = game.player();
    let other = listed_competitor(&mut game, 1_000_000.0);
    let start = cash(&game, player);
    game.apply(Command::BuyShares {
        company: other,
        share: 0.1,
    })
    .unwrap();
    let price = usd(1_000_000.0 * 0.1 * (1.0 + 0.02 + 0.05));
    assert_eq!(cash(&game, player), start - price);
    let p = &game.state().companies[player.index()];
    assert_eq!(p.ledger.balance(Account::Participations), price);
    assert_eq!(p.stock_cost[&other], price);
    let o = &game.state().companies[other.index()];
    assert!((stock::stake(o, Holder::Company(player)) - 0.1).abs() < 1e-9);
    assert_eq!(o.listing.as_ref().unwrap().value, usd(1_050_000.0));
    // At most half of a company until K3.
    assert_eq!(
        game.apply(Command::BuyShares {
            company: other,
            share: 0.45
        }),
        Err(CommandError::ShareOutOfRange { max: 0.4 })
    );
    assert_eq!(
        game.apply(Command::SellShares {
            company: other,
            share: 0.2
        }),
        Err(CommandError::NotEnoughStock { held: 0.1 })
    );
    game.apply(Command::SellShares {
        company: other,
        share: 0.1,
    })
    .unwrap();
    let p = &game.state().companies[player.index()];
    assert!(p.stock_cost.is_empty());
    assert_eq!(p.ledger.balance(Account::Participations), Money::ZERO);
    // A round trip costs the spreads and the price effect.
    assert!(cash(&game, player) < start);
    assert!(p.ledger.is_balanced());
    assert_eq!(
        game.apply(Command::BuyShares {
            company: player,
            share: 0.1
        }),
        Err(CommandError::WithinGroup)
    );
}

#[test]
fn dividends_go_to_the_owners() {
    let mut game = game();
    let player = game.player();
    let other = listed_competitor(&mut game, 1_000_000.0);
    game.apply(Command::BuyShares {
        company: other,
        share: 0.2,
    })
    .unwrap();
    // The other company earns 100 000 in 1900 and pays 40 % (AI default without AI
    // state: the player's default 0 – so it sets its quote).
    game.state_mut().companies[other.index()].dividend_payout = Some(0.4);
    game.state_mut().companies[other.index()].ledger.income(
        CostType::Revenue,
        CostCenter::default(),
        Account::Cash,
        usd(100_000.0),
    );
    let before = cash(&game, player);
    let mut paid = false;
    for _ in 0..365 {
        let report = game.advance(crate::calendar::RoundLength::Day, |_| {});
        paid |= report
            .messages
            .iter()
            .any(|m| m.key == keys::STOCK_DIVIDEND_RECEIVED);
    }
    assert!(paid);
    let o = &game.state().companies[other.index()];
    let dividend = o.listing.as_ref().unwrap().last_dividend;
    assert!(dividend > Money::ZERO);
    // The player's company books its fifth as income from participations.
    let income = game.state().companies[player.index()].ledger.year.by_type[&CostType::Investments];
    assert!(
        (income - dividend.scale(0.2)).abs() <= usd(0.01),
        "{income:?} {dividend:?}"
    );
    assert!(cash(&game, player) > before);
    assert!(o.ledger.is_balanced());
    assert!(game.state().companies[player.index()].ledger.is_balanced());
}

#[test]
fn failed_listed_companies_are_written_off() {
    let mut game = game();
    let player = game.player();
    let other = listed_competitor(&mut game, 1_000_000.0);
    game.apply(Command::BuyShares {
        company: other,
        share: 0.1,
    })
    .unwrap();
    game.state_mut().companies[other.index()].bankrupt = true;
    let mut written = false;
    for _ in 0..31 {
        let report = game.advance(crate::calendar::RoundLength::Day, |_| {});
        written |= report
            .messages
            .iter()
            .any(|m| m.key == keys::STOCK_WRITTEN_OFF);
    }
    assert!(written);
    let p = &game.state().companies[player.index()];
    assert_eq!(p.ledger.balance(Account::Participations), Money::ZERO);
    assert!(game.state().companies[other.index()].listing.is_none());
    assert!(p.ledger.is_balanced());
}

#[test]
fn the_stock_market_survives_saving() {
    let mut game = new_game(with_stock(test_support::trading(), vec![(1900, 3, 0.3)]));
    game.apply(Command::GoPublic { share: 0.2 }).unwrap();
    game.apply(Command::SetDividend { payout: 0.3 }).unwrap();
    let other = listed_competitor(&mut game, 500_000.0);
    game.apply(Command::BuyShares {
        company: other,
        share: 0.05,
    })
    .unwrap();
    days(&mut game, 40);
    let mut loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(game.state_hash(), loaded.state_hash());
    days(&mut game, 60);
    days(&mut loaded, 60);
    assert_eq!(game.state_hash(), loaded.state_hash());
}

/// A competitor listed at `value` with founders, investors and the player's company as
/// owners.
fn mixed_competitor(game: &mut Game, value: f64) -> CompanyId {
    let id = listed_competitor(game, value);
    game.state_mut().companies[id.index()].owners = vec![
        Stake {
            holder: Holder::Private,
            share: 0.4,
        },
        Stake {
            holder: Holder::Investors,
            share: 0.6,
        },
    ];
    id
}

#[test]
fn a_takeover_makes_a_subsidiary() {
    let mut game = game();
    let player = game.player();
    let other = mixed_competitor(&mut game, 400_000.0);
    // 10 % on the market first: 400 000 · 0.1 · 1.07 = 42 800.
    game.apply(Command::BuyShares {
        company: other,
        share: 0.1,
    })
    .unwrap();
    let value = game.state().companies[other.index()]
        .listing
        .as_ref()
        .unwrap()
        .value;
    let before = cash(&game, player);
    let (price, cost) = stock::takeover_price(
        game.catalog(),
        &game.state().companies[other.index()],
        player,
    );
    assert_eq!(price, value.scale(0.9 * 1.3));
    assert_eq!(cost, price.scale(0.02));
    game.apply(Command::TakeOver { company: other }).unwrap();
    assert_eq!(cash(&game, player), before - price - cost);
    let o = &game.state().companies[other.index()];
    assert!(o.listing.is_none());
    assert_eq!(o.subsidiary_of.map(|s| s.parent), Some(player));
    assert_eq!(o.owners.len(), 1);
    assert!((stock::stake(o, Holder::Company(player)) - 1.0).abs() < 1e-9);
    let p = &game.state().companies[player.index()];
    assert!(p.stock_cost.is_empty());
    assert_eq!(
        p.ledger.balance(Account::Participations),
        usd(42_800.0) + price
    );
    assert!(p.ledger.is_balanced());
    assert!(crate::group::same_group(game.state(), player, other));
    let group = crate::group::consolidated(game.state(), player);
    let claims: Money = Account::ALL
        .iter()
        .filter(|a| !a.is_asset())
        .map(|&a| group.balance(a))
        .sum();
    assert_eq!(group.total_assets(), claims);
    // Not twice, and not without a listing.
    assert_eq!(
        game.apply(Command::TakeOver { company: other }),
        Err(CommandError::WithinGroup)
    );
}

#[test]
fn owners_are_paid_out_in_a_takeover() {
    let mut game = game();
    let player = game.player();
    let target = mixed_competitor(&mut game, 400_000.0);
    let bidder = listed_competitor(&mut game, 1_000_000.0);
    game.apply(Command::BuyShares {
        company: target,
        share: 0.2,
    })
    .unwrap();
    let paid = game.state().companies[player.index()].stock_cost[&target];
    let value = game.state().companies[target.index()]
        .listing
        .as_ref()
        .unwrap()
        .value;
    let catalog = game.catalog().clone();
    game.state_mut().companies[bidder.index()].ledger.income(
        CostType::Revenue,
        CostCenter::default(),
        Account::Cash,
        usd(5_000_000.0),
    );
    crate::command::execute(
        game.state_mut(),
        &catalog,
        bidder,
        &Command::TakeOver { company: target },
    )
    .unwrap();
    // The player's company sold its fifth at the bid: the gain is income.
    let p = &game.state().companies[player.index()];
    let proceeds = value.scale(1.3).scale(0.2);
    assert_eq!(
        p.ledger.year.by_type[&CostType::Investments],
        proceeds - paid
    );
    assert_eq!(p.ledger.balance(Account::Participations), Money::ZERO);
    assert!(p.ledger.is_balanced());
    assert!(!game.state().game_over);
}

#[test]
fn a_takeover_of_the_player_ends_the_game() {
    let mut game = game();
    let player = game.player();
    game.apply(Command::GoPublic { share: 0.4 }).unwrap();
    let raider = listed_competitor(&mut game, 1_000_000.0);
    game.state_mut().companies[raider.index()].ledger.income(
        CostType::Revenue,
        CostCenter::default(),
        Account::Cash,
        usd(100_000_000.0),
    );
    let catalog = game.catalog().clone();
    // With 60 % the player keeps its company.
    assert_eq!(
        crate::command::execute(
            game.state_mut(),
            &catalog,
            raider,
            &Command::TakeOver { company: player },
        ),
        Err(CommandError::TakeoverNoMajority { held: 0.6 })
    );
    game.apply(Command::IssueShares { share: 0.25 }).unwrap();
    crate::command::execute(
        game.state_mut(),
        &catalog,
        raider,
        &Command::TakeOver { company: player },
    )
    .unwrap();
    assert!(game.state().game_over);
    let c = &game.state().companies[player.index()];
    assert!((stock::stake(c, Holder::Player) - 0.45).abs() < 1e-9);
    assert!((stock::stake(c, Holder::Company(raider)) - 0.55).abs() < 1e-9);
}

#[test]
fn a_buyback_retires_shares() {
    let mut game = game();
    let player = game.player();
    game.apply(Command::GoPublic { share: 0.4 }).unwrap();
    let before = game.state().companies[player.index()].clone();
    let l = before.listing.as_ref().unwrap();
    let price = stock::buy_price(game.catalog(), l, 0.1);
    game.apply(Command::BuyBackShares { share: 0.1 }).unwrap();
    let c = &game.state().companies[player.index()];
    assert_eq!(c.ledger.cash(), before.ledger.cash() - price);
    assert_eq!(
        crate::ranking::equity(c),
        crate::ranking::equity(&before) - price
    );
    // The player held 0.6 of 1; now 0.6 of 0.9.
    assert!((stock::stake(c, Holder::Player) - 0.6 / 0.9).abs() < 1e-9);
    let after = c.listing.as_ref().unwrap();
    assert_eq!(after.shares, 900_000);
    assert!(after.price() > l.price());
    assert!(c.ledger.is_balanced());
    assert_eq!(
        game.apply(Command::BuyBackShares { share: 0.2 }),
        Err(CommandError::ShareOutOfRange { max: 0.1 })
    );
}

#[test]
fn ai_investors_buy_cheap_and_sell_dear() {
    let mut catalog = with_stock(test_support::trading(), Vec::new());
    catalog.stock.ai_portfolio_cash_share = 0.5;
    let mut game = new_game(catalog);
    let investor = competitor(&mut game);
    let target = mixed_competitor(&mut game, 100_000.0);
    let fair = stock::fair(game.catalog(), &game.state().companies[target.index()]);
    // Far below its fair value: the investor buys up to a tenth.
    game.state_mut().companies[target.index()]
        .listing
        .as_mut()
        .unwrap()
        .value = fair.scale(0.5);
    let trades = stock::ai_trades(game.state(), game.catalog(), investor, usd(1e9));
    assert_eq!(
        trades,
        vec![Command::BuyShares {
            company: target,
            share: 0.1
        }]
    );
    let catalog = game.catalog().clone();
    crate::command::execute(game.state_mut(), &catalog, investor, &trades[0]).unwrap_or(());
    // Far above it: it sells everything.
    game.state_mut().companies[target.index()]
        .owners
        .push(Stake {
            holder: Holder::Company(investor),
            share: 0.1,
        });
    game.state_mut().companies[investor.index()]
        .stock_cost
        .insert(target, usd(1.0));
    game.state_mut().companies[target.index()]
        .listing
        .as_mut()
        .unwrap()
        .value = fair.scale(2.0);
    let trades = stock::ai_trades(game.state(), game.catalog(), investor, Money::ZERO);
    assert!(matches!(
        trades.as_slice(),
        [Command::SellShares { company, .. }] if *company == target
    ));
}
