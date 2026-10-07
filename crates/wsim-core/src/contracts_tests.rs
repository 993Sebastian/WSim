//! Tests of the supply contracts (W4).

use crate::catalog::{Catalog, ContractAiModel, ContractModel, SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::contracts::{self, ContractStatus, Decline, Terms};
use crate::game::Game;
use crate::ids::ProductId;
use crate::ledger::{Account, CostType};
use crate::market;
use crate::message::keys;
use crate::money::Money;
use crate::state::{CompanyId, PriceMode, SiteId};
use crate::trade_tests::{
    competitor, days, inventory_matches, iron, new_game, stock, usd, warehouse, with_tariff,
};

fn with_contracts(mut catalog: Catalog, proposal_chance: f64) -> Catalog {
    catalog.contracts = ContractModel {
        months_max: 60,
        months_default: 12,
        penalty_max: 1.0,
        penalty_default: 0.2,
        cancel_months: 3,
        proposal_days: 30,
        keep_months: 12,
        ai: ContractAiModel {
            sale_discount: 0.05,
            purchase_premium: 0.05,
            share: 0.5,
            penalty_max: 0.5,
            proposal_chance,
        },
        provenance: Default::default(),
    };
    catalog
}

fn cash(game: &Game, company: CompanyId) -> Money {
    game.state().companies[company.index()].ledger.cash()
}

/// The other company proposes to buy iron from the player's warehouse; the player agrees.
fn agreed(
    game: &mut Game,
    buyer_country: &str,
    per_month: f64,
    months: u32,
) -> (SiteId, SiteId, CompanyId) {
    let player = game.player();
    let seller = warehouse(game, player, "AAA", 1000.0);
    let other = competitor(game);
    let buyer = warehouse(game, other, buyer_country, 0.0);
    let catalog = game.catalog().clone();
    let terms = Terms {
        seller,
        buyer,
        product: iron(game),
        per_month,
        price: usd(2.0),
        months,
        min_quality: 0.0,
        penalty: 0.5,
    };
    contracts::propose(game.state_mut(), &catalog, other, &terms).unwrap();
    assert_eq!(game.state().contracts[0].status, ContractStatus::Proposed);
    game.apply(Command::AnswerContract {
        contract: 0,
        accept: true,
    })
    .unwrap();
    (seller, buyer, other)
}

#[test]
fn deliveries_run_monthly_and_shortfalls_cost_the_penalty() {
    let mut game = new_game(with_contracts(test_support::trading(), 0.0));
    let player = game.player();
    let (seller, buyer, other) = agreed(&mut game, "AAA", 400.0, 3);
    let start = cash(&game, other);
    // Deliveries start with the next month.
    days(&mut game, 31);
    assert_eq!(stock(&game, buyer), 0.0);
    days(&mut game, 14);
    let half = stock(&game, buyer);
    assert!((half - 400.0 * 14.0 / 28.0).abs() < 1e-6, "{half}");
    days(&mut game, 14);
    assert!((stock(&game, buyer) - 400.0).abs() < 1e-6);
    assert!((stock(&game, seller) - 600.0).abs() < 1e-6);
    // March in full, April only the 200 left: the seller pays half the value of 200.
    days(&mut game, 31 + 30);
    let c = &game.state().contracts[0];
    assert_eq!(c.status, ContractStatus::Ended);
    assert!((c.delivered_total - 1000.0).abs() < 1e-6);
    assert_eq!(c.paid_by_seller, usd(200.0));
    let expected = start - usd(2.0 * 1000.0) + usd(200.0);
    // Daily amounts are rounded to the hundredth of a cent.
    assert!((cash(&game, other) - expected).to_usd().abs() < 0.01);
    for company in [player, other] {
        assert!(game.state().companies[company.index()].ledger.is_balanced());
        assert!(inventory_matches(&game, company));
    }
}

#[test]
fn deliveries_abroad_pay_freight_and_customs() {
    let mut game = new_game(with_contracts(with_tariff(0.2, false), 0.0));
    let player = game.player();
    let (_, buyer, other) = agreed(&mut game, "BBB", 300.0, 2);
    days(&mut game, 31 + 10);
    let delivered = game.state().contracts[0].delivered_total;
    assert!(delivered > 0.0);
    // Goods on the way belong to the buyer; the seller paid customs on their price.
    assert!(stock(&game, buyer) < delivered);
    let ledger = &game.state().companies[player.index()].ledger;
    let customs = -ledger.month.by_type[&CostType::Customs];
    assert!(
        (customs.to_usd() - 0.2 * 2.0 * delivered).abs() < 0.01,
        "{customs:?}"
    );
    assert!(ledger.month.by_type[&CostType::Transport] < Money::ZERO);
    for company in [player, other] {
        assert!(game.state().companies[company.index()].ledger.is_balanced());
        assert!(inventory_matches(&game, company));
    }
}

#[test]
fn cancelling_costs_the_penalty_of_three_months() {
    let mut game = new_game(with_contracts(test_support::trading(), 0.0));
    let player = game.player();
    let (_, _, other) = agreed(&mut game, "AAA", 300.0, 12);
    days(&mut game, 31 + 14);
    let before = cash(&game, other);
    game.apply(Command::CancelContract { contract: 0 }).unwrap();
    let c = &game.state().contracts[0];
    assert_eq!(c.status, ContractStatus::Cancelled);
    assert_eq!(c.cancelled_by, Some(player));
    // Half the value of three monthly quantities.
    assert_eq!(c.paid_by_seller, usd(0.5 * 2.0 * 300.0 * 3.0));
    assert_eq!(cash(&game, other), before + usd(900.0));
    // Closed contracts cannot be answered or cancelled again.
    assert_eq!(
        game.apply(Command::CancelContract { contract: 0 }),
        Err(CommandError::UnknownContract)
    );
}

#[test]
fn proposals_are_checked() {
    let mut game = new_game(with_contracts(test_support::trading(), 0.0));
    let player = game.player();
    let own = warehouse(&mut game, player, "AAA", 10.0);
    let other_own = warehouse(&mut game, player, "BBB", 0.0);
    let product = iron(&game);
    let propose = |game: &mut Game, buyer: SiteId, months: u32, per_month: f64| {
        game.apply(Command::ProposeContract {
            seller: own,
            buyer,
            product,
            per_month,
            price: usd(0.01),
            months,
            min_quality: 0.0,
            penalty: 0.2,
        })
    };
    assert_eq!(
        propose(&mut game, other_own, 12, 10.0),
        Err(CommandError::ContractWithItself)
    );
    let other = competitor(&mut game);
    let theirs = warehouse(&mut game, other, "AAA", 0.0);
    assert!(matches!(
        propose(&mut game, theirs, 61, 10.0),
        Err(CommandError::InvalidTerm { max: 60 })
    ));
    assert_eq!(
        propose(&mut game, theirs, 12, 0.0),
        Err(CommandError::InvalidQuantity)
    );
    // A warehouse needs no iron: the other company declines.
    assert_eq!(
        propose(&mut game, theirs, 12, 10.0),
        Err(CommandError::ContractDeclined {
            reason: "menge".into()
        })
    );
    // Without contracts in the data there are none.
    let mut plain = new_game(test_support::trading());
    let p = plain.player();
    let a = warehouse(&mut plain, p, "AAA", 10.0);
    let o = competitor(&mut plain);
    let b = warehouse(&mut plain, o, "AAA", 0.0);
    assert_eq!(
        plain.apply(Command::ProposeContract {
            seller: a,
            buyer: b,
            product,
            per_month: 1.0,
            price: usd(2.0),
            months: 1,
            min_quality: 0.0,
            penalty: 0.0,
        }),
        Err(CommandError::NoContracts)
    );
}

/// A furnace of the player that smelts ore (100 t a day at full load into 50 t iron).
fn furnace(game: &mut Game) -> SiteId {
    let c = game.catalog().clone();
    let me = game.player();
    game.state_mut().companies[me.index()].ledger.transfer(
        Account::Cash,
        Account::Equity,
        usd(5_000_000.0),
    );
    game.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: SiteType::Factory,
    })
    .unwrap();
    let works = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.apply(Command::BuildFacility {
        site: works,
        facility: c.facilities.id("ofen").unwrap(),
        count: 1,
        size: crate::catalog::FacilitySize::Medium,
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site: works,
        slot: 0,
        recipe: c.recipes.id("eisen_schmelzen"),
        utilization: 1.0,
    })
    .unwrap();
    days(game, 40);
    works
}

fn product(game: &Game, key: &str) -> ProductId {
    game.catalog().products.id(key).unwrap()
}

#[test]
fn ai_answers_by_price_quantity_quality_and_penalty() {
    let mut game = new_game(with_contracts(test_support::trading(), 0.0));
    let works = furnace(&mut game);
    let other = competitor(&mut game);
    let store = warehouse(&mut game, other, "AAA", 0.0);
    let catalog = game.catalog().clone();
    let state = game.state();
    let aaa = state.sites[works.index()].country;
    let (ore, iron) = (product(&game, "erz"), product(&game, "eisen"));
    let me = game.player();
    let need = 30.0 * contracts::daily_need(state, &catalog, works, ore);
    let output = 30.0 * contracts::daily_output(state, &catalog, works, iron);
    assert!(need > 0.0 && output > 0.0);

    // The works buys ore: up to 5 % above the market price, up to half its need.
    let ore_price = market::market_price(&catalog, state, aaa, ore);
    let buy = |per_month: f64, factor: f64, penalty: f64| Terms {
        seller: store,
        buyer: works,
        product: ore,
        per_month,
        price: ore_price.scale(factor),
        months: 12,
        min_quality: 0.0,
        penalty,
    };
    let answer = |t: Terms| contracts::ai_answer(state, &catalog, &t, me);
    assert_eq!(answer(buy(0.4 * need, 1.04, 0.2)), Ok(()));
    assert_eq!(answer(buy(0.4 * need, 1.2, 0.2)), Err(Decline::Price));
    assert_eq!(answer(buy(0.6 * need, 1.0, 0.2)), Err(Decline::Quantity));
    assert_eq!(answer(buy(0.4 * need, 1.0, 0.8)), Err(Decline::Penalty));

    // The works sells iron: at least 95 % of its price, up to half its output, only the
    // quality it makes.
    let iron_price = market::market_price(&catalog, state, aaa, iron);
    let sell = |per_month: f64, factor: f64, min_quality: f64| Terms {
        seller: works,
        buyer: store,
        product: iron,
        per_month,
        price: iron_price.scale(factor),
        months: 12,
        min_quality,
        penalty: 0.2,
    };
    assert_eq!(answer(sell(0.4 * output, 1.0, 0.0)), Ok(()));
    assert_eq!(answer(sell(0.4 * output, 0.9, 0.0)), Err(Decline::Price));
    assert_eq!(answer(sell(0.6 * output, 1.0, 0.0)), Err(Decline::Quantity));
    assert_eq!(answer(sell(0.4 * output, 1.0, 90.0)), Err(Decline::Quality));
}

#[test]
fn ai_companies_propose_to_buy_what_the_player_offers() {
    let mut game = new_game(with_contracts(test_support::trading(), 1.0));
    let works = furnace(&mut game);
    let other = competitor(&mut game);
    // The furnace now belongs to the other company; the player sells ore.
    game.state_mut().sites[works.index()].owner = other;
    let me = game.player();
    let shop = warehouse(&mut game, me, "AAA", 0.0);
    let ore = product(&game, "erz");
    game.apply(Command::SetSale {
        site: shop,
        product: ore,
        mode: Some(PriceMode::Fixed(usd(5.0))),
        keep: 0.0,
    })
    .unwrap();
    let catalog = game.catalog().clone();
    let date = game.state().date.first_of_next_month();
    let news = contracts::month_start(game.state_mut(), &catalog, date);
    assert!(news.iter().any(|m| m.key == keys::CONTRACT_PROPOSED));
    let c = game.state().contracts.last().unwrap().clone();
    assert_eq!(c.status, ContractStatus::Proposed);
    assert_eq!((c.seller, c.buyer, c.proposer), (shop, works, other));
    assert_eq!(c.answering(), me);
    // Half of the furnace's need, at the market price, for the standard term.
    let need = 30.0 * contracts::daily_need(game.state(), &catalog, works, ore);
    assert!((c.per_month - 0.5 * need).abs() < 1e-6);
    assert_eq!(c.months, 12);
    // Unanswered, the proposal lapses.
    days(&mut game, 31);
    let c = game
        .state()
        .contracts
        .iter()
        .find(|x| x.id == c.id)
        .unwrap();
    assert_eq!(c.status, ContractStatus::Expired);
}
