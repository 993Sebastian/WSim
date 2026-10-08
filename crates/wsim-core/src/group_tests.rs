//! Tests of subsidiaries and the group (W6).

use crate::catalog::{Catalog, SubsidiaryModel, test_support};
use crate::command::{Command, CommandError, NameError};
use crate::game::Game;
use crate::group::{self, SubsidiaryFocus};
use crate::ledger::{Account, CostCenter, CostType};
use crate::money::Money;
use crate::save;
use crate::state::{CompanyId, Holder};
use crate::trade_tests::{country, days, inventory_matches, iron, new_game, usd, warehouse};

fn with_subsidiaries(mut catalog: Catalog) -> Catalog {
    catalog.subsidiaries = SubsidiaryModel {
        enabled: true,
        min_capital: usd(10_000.0),
        founding_cost: usd(1_000.0),
        competence: 0.5,
        aggressiveness: 0.5,
        logistics_cash_share: 0.5,
        logistics_min_return: 0.0,
        provenance: Default::default(),
    };
    catalog
}

fn found(game: &mut Game, name: &str, capital: f64) -> CompanyId {
    let before = game.state().companies.len();
    game.apply(Command::FoundSubsidiary {
        name: name.into(),
        country: country(game, "AAA"),
        capital: usd(capital),
        focus: SubsidiaryFocus::Production,
    })
    .unwrap();
    assert_eq!(game.state().companies.len(), before + 1);
    CompanyId(u32::try_from(before).unwrap())
}

fn balance(game: &Game, c: CompanyId, account: Account) -> Money {
    game.state().companies[c.index()].ledger.balance(account)
}

fn equity(game: &Game, c: CompanyId) -> Money {
    [Account::Equity, Account::RetainedEarnings, Account::Result]
        .iter()
        .map(|&a| balance(game, c, a))
        .sum()
}

/// The group balance holds: assets = loans + equity, and the equity is the companies'
/// equity less the subsidiaries' subscribed capital.
fn group_holds(game: &Game) {
    let state = game.state();
    let g = group::consolidated(state, state.player);
    assert_eq!(g.total_assets(), g.balance(Account::Loans) + g.equity());
    let members = group::members(state, state.player);
    let sum: Money = members.iter().map(|&c| equity(game, c)).sum();
    let paid_in: Money = members
        .iter()
        .skip(1)
        .map(|&c| balance(game, c, Account::Equity))
        .sum();
    assert_eq!(g.equity(), sum - paid_in);
    for &c in &members {
        assert!(state.companies[c.index()].ledger.is_balanced());
    }
}

#[test]
fn founding_moves_capital_into_a_company_of_its_own() {
    let mut game = new_game(with_subsidiaries(test_support::trading()));
    let player = game.player();
    let cash = balance(&game, player, Account::Cash);
    let group_equity = group::consolidated(game.state(), player).equity();
    let sub = found(&mut game, "Spedition Nord", 50_000.0);
    assert_eq!(balance(&game, player, Account::Cash), cash - usd(51_000.0));
    assert_eq!(
        balance(&game, player, Account::Participations),
        usd(50_000.0)
    );
    assert_eq!(balance(&game, sub, Account::Cash), usd(50_000.0));
    assert_eq!(balance(&game, sub, Account::Equity), usd(50_000.0));
    let c = &game.state().companies[sub.index()];
    assert_eq!(c.owners[0].holder, Holder::Company(player));
    assert!(c.ai.is_some());
    assert_eq!(group::top(game.state(), sub), player);
    assert!(group::same_group(game.state(), sub, player));
    // The group lost only the founding cost.
    assert_eq!(
        group::consolidated(game.state(), player).equity(),
        group_equity - usd(1_000.0)
    );
    group_holds(&game);
}

#[test]
fn founding_is_checked() {
    let mut plain = new_game(test_support::trading());
    let aaa = country(&plain, "AAA");
    let command = |name: &str, capital: f64| Command::FoundSubsidiary {
        name: name.into(),
        country: aaa,
        capital: usd(capital),
        focus: SubsidiaryFocus::Logistics,
    };
    assert_eq!(
        plain.apply(command("Tochter", 50_000.0)),
        Err(CommandError::NoSubsidiaries)
    );
    let mut game = new_game(with_subsidiaries(test_support::trading()));
    assert_eq!(
        game.apply(command("Tochter", 5_000.0)),
        Err(CommandError::CapitalTooLow { min: usd(10_000.0) })
    );
    let own = game.state().companies[0].name.clone();
    assert!(matches!(
        game.apply(command(&own, 50_000.0)),
        Err(CommandError::Name(NameError::Taken { .. }))
    ));
    assert!(matches!(
        game.apply(command("Tochter", 1e9)),
        Err(CommandError::NotEnoughCash { .. })
    ));
}

#[test]
fn capital_goes_in_and_profit_comes_back_free_of_income() {
    let mut game = new_game(with_subsidiaries(test_support::trading()));
    let player = game.player();
    let sub = found(&mut game, "Werke Süd", 20_000.0);
    game.apply(Command::MoveCapital {
        company: sub,
        amount: usd(10_000.0),
    })
    .unwrap();
    assert_eq!(balance(&game, sub, Account::Equity), usd(30_000.0));
    assert_eq!(
        balance(&game, player, Account::Participations),
        usd(30_000.0)
    );
    // The subsidiary earns 5 000.
    game.state_mut().companies[sub.index()].ledger.income(
        CostType::Revenue,
        CostCenter::default(),
        Account::Cash,
        usd(5_000.0),
    );
    let result = balance(&game, player, Account::Result);
    let too_much = Command::MoveCapital {
        company: sub,
        amount: usd(-40_000.0),
    };
    assert_eq!(
        game.apply(too_much),
        Err(CommandError::AmountTooHigh { max: usd(35_000.0) })
    );
    game.apply(Command::MoveCapital {
        company: sub,
        amount: usd(-8_000.0),
    })
    .unwrap();
    // 5 000 dividend into the parent's reserves, 3 000 capital back.
    assert_eq!(balance(&game, player, Account::Result), result);
    assert_eq!(
        balance(&game, player, Account::RetainedEarnings),
        usd(5_000.0)
    );
    assert_eq!(
        balance(&game, player, Account::Participations),
        usd(27_000.0)
    );
    assert_eq!(balance(&game, sub, Account::Equity), usd(27_000.0));
    group_holds(&game);
    // Only the parent moves capital.
    let other = found(&mut game, "Dritte", 10_000.0);
    assert_eq!(
        game.apply_as(
            other,
            Command::MoveCapital {
                company: sub,
                amount: usd(1_000.0)
            }
        ),
        Err(CommandError::NotOwnSubsidiary)
    );
}

#[test]
fn sites_move_within_the_group_at_book_value() {
    let mut game = new_game(with_subsidiaries(test_support::trading()));
    let player = game.player();
    let site = warehouse(&mut game, player, "AAA", 500.0);
    let sub = found(&mut game, "Lager GmbH", 20_000.0);
    let assets = group::consolidated(game.state(), player).total_assets();
    let stock = balance(&game, player, Account::Inventory);
    game.apply(Command::TransferSite { site, to: sub }).unwrap();
    assert_eq!(game.state().sites[site.index()].owner, sub);
    assert_eq!(balance(&game, sub, Account::Inventory), stock);
    assert_eq!(balance(&game, player, Account::Inventory), Money::ZERO);
    assert_eq!(
        group::consolidated(game.state(), player).total_assets(),
        assets
    );
    assert!(inventory_matches(&game, sub));
    group_holds(&game);
    // And back.
    game.apply(Command::TransferSite { site, to: player })
        .unwrap();
    assert_eq!(game.state().sites[site.index()].owner, player);
    assert!(inventory_matches(&game, player));
    group_holds(&game);
}

#[test]
fn no_business_within_the_group() {
    let catalog = crate::contracts_tests::with_contracts(test_support::trading(), 0.0);
    let mut game = new_game(with_subsidiaries(catalog));
    let player = game.player();
    let sub = found(&mut game, "Handel AG", 20_000.0);
    let theirs = warehouse(&mut game, sub, "AAA", 10.0);
    let mine = warehouse(&mut game, player, "BBB", 0.0);
    assert_eq!(
        game.apply(Command::MakeOffer {
            seller: sub,
            object: crate::deals::DealObject::Site(theirs),
            price: usd(1_000.0),
        }),
        Err(CommandError::WithinGroup)
    );
    let product = iron(&game);
    assert_eq!(
        game.apply(Command::ProposeContract {
            seller: theirs,
            buyer: mine,
            product,
            per_month: 1.0,
            price: usd(1.0),
            months: 1,
            min_quality: 0.0,
            penalty: 0.0,
        }),
        Err(CommandError::WithinGroup)
    );
    // The subsidiary is no competitor of the player.
    let overview = crate::views::overview(&game);
    assert!(overview.competitors.iter().all(|c| c.name != "Handel AG"));
}

#[test]
fn a_failed_subsidiary_is_written_off() {
    let mut game = new_game(with_subsidiaries(test_support::trading()));
    let player = game.player();
    let sub = found(&mut game, "Pleite KG", 20_000.0);
    game.state_mut().companies[sub.index()].bankrupt = true;
    let news = group::settle_failures(game.state_mut());
    assert_eq!(news.len(), 1);
    assert_eq!(balance(&game, player, Account::Participations), Money::ZERO);
    assert!(game.state().companies[sub.index()].subsidiary_of.is_none());
    assert_eq!(group::members(game.state(), player), vec![player]);
    assert!(game.state().companies[player.index()].ledger.is_balanced());
}

#[test]
fn a_logistics_subsidiary_buys_vehicles_for_the_freight_of_others() {
    let mut catalog = with_subsidiaries(test_support::trading());
    let ship = catalog.vehicles.id("dampfer").unwrap();
    let rail = catalog.vehicles.id("bahn").unwrap();
    for (v, payload) in [(ship, 4000.0), (rail, 300.0)] {
        catalog.vehicles.get_mut(v).fleet = Some(crate::catalog::FleetVehicle {
            payload_t: crate::time_series::TimeSeries::new(vec![(1900, payload)]).unwrap(),
            price_usd: crate::time_series::TimeSeries::new(vec![(1900, 50_000.0)]).unwrap(),
        });
    }
    catalog.logistics = crate::catalog::LogisticsModel {
        enabled: true,
        state_surcharge: 0.5,
        state_risk_factor: 0.5,
        market_margin: 0.15,
        load: 0.6,
        upkeep_share: 0.06,
        life_years: 20.0,
        sale_share: 0.6,
        rental_share: 0.5,
        rental_market_share: 0.25,
        risk_land: None,
        risk_sea: None,
        ai_share: 0.5,
        ai_cash_share: 0.2,
        provenance: Default::default(),
    };
    let mut game = new_game(catalog);
    let sub = found(&mut game, "Reederei", 500_000.0);
    game.apply(Command::SetSubsidiaryFocus {
        company: sub,
        focus: SubsidiaryFocus::Logistics,
    })
    .unwrap();
    // Without freight on the market there is nothing to carry.
    let catalog = game.catalog().clone();
    assert!(crate::logistics::rental_purchase(game.state(), &catalog, sub).is_none());
    let fm = &mut game.state_mut().freight_market;
    (fm.land_last, fm.sea_last) = (1e10, 1e10);
    let (_, n) = crate::logistics::rental_purchase(game.state(), &catalog, sub).unwrap();
    assert!((1..=5).contains(&n), "{n}");
    // Its own management buys at the next month start.
    days(&mut game, 31);
    let c = &game.state().companies[sub.index()];
    assert!(!c.logistics.fleet.is_empty());
    assert!(c.logistics.carry_for_others);
    group_holds(&game);
}

#[test]
fn subsidiaries_survive_saving() {
    let mut game = new_game(with_subsidiaries(test_support::trading()));
    let player = game.player();
    let site = warehouse(&mut game, player, "AAA", 100.0);
    let sub = found(&mut game, "Speicher", 20_000.0);
    game.apply(Command::TransferSite { site, to: sub }).unwrap();
    let mut loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(game.state_hash(), loaded.state_hash());
    days(&mut game, 40);
    days(&mut loaded, 40);
    assert_eq!(game.state_hash(), loaded.state_hash());
    group_holds(&game);
}
