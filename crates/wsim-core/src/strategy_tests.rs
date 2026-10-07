//! Scenario tests for strategies (MA4) with `test_support::management`.

use std::sync::Arc;

use crate::calendar::RoundLength;
use crate::catalog::{FacilitySize, SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::decision::{Choice, ChoiceKind, Decision, Recorder, Topic, Verdict};
use crate::game::Game;
use crate::ledger::{Account, CostCenter, CostType};
use crate::management;
use crate::management_tests::{
    cheap_furnaces, days, found, free, head, mine_and_works, new_game, usd, weak_works,
};
use crate::money::Money;
use crate::save;
use crate::state::{ConcernReason, PriceMode, SiteId, Unit};
use crate::strategy::{
    self, PriceStrategy, StockStrategy, StrategyField, StrategyScope, StrategyValue,
    SupplyStrategy, WageStrategy,
};

fn set(game: &mut Game, scope: StrategyScope, value: StrategyValue) {
    game.apply(Command::SetStrategy {
        scope,
        field: value.field(),
        value: Some(value),
    })
    .unwrap();
}

fn remove(game: &mut Game, scope: StrategyScope, field: StrategyField) {
    game.apply(Command::SetStrategy {
        scope,
        field,
        value: None,
    })
    .unwrap();
}

fn found_in(game: &mut Game, country: &str, kind: SiteType) -> SiteId {
    let country = game.catalog().countries.id(country).unwrap();
    game.apply(Command::FoundSite { country, kind }).unwrap();
    SiteId(u32::try_from(game.state().sites.len() - 1).unwrap())
}

fn price_at(game: &Game, site: SiteId) -> PriceStrategy {
    strategy::for_site(game.catalog(), game.state(), game.player(), site).price
}

/// The player's sites.
fn own(game: &Game) -> Vec<SiteId> {
    (0..game.state().sites.len())
        .map(|i| SiteId(u32::try_from(i).unwrap()))
        .filter(|&s| game.state().sites[s.index()].owner == game.player())
        .collect()
}

/// The commands the routine of a site proposes by the rules today (on a copy).
fn proposed(game: &Game, site: SiteId) -> Vec<Command> {
    let mut state = game.state().clone();
    let mut recorder = Recorder::default();
    crate::ai::site_routine(
        &mut state,
        game.catalog(),
        game.player(),
        (&own(game), &[site]),
        game.date(),
        &mut recorder,
    );
    recorder
        .decisions
        .iter()
        .flat_map(|d| d.chosen().steps.iter().map(|s| s.command.clone()))
        .collect()
}

#[test]
fn strategies_are_inherited_and_overridden() {
    let mut game = new_game(test_support::management());
    let c = game.catalog().clone();
    let (aaa, europa) = (
        c.countries.id("AAA").unwrap(),
        c.continents.id("europa").unwrap(),
    );
    let a1 = found(&mut game, SiteType::Factory);
    let a2 = found(&mut game, SiteType::Factory);
    let b1 = found_in(&mut game, "BBB", SiteType::Factory);
    let all = [a1, a2, b1];
    assert!(
        all.iter()
            .all(|&s| price_at(&game, s) == PriceStrategy::Market)
    );

    set(
        &mut game,
        StrategyScope::Company,
        StrategyValue::Price(PriceStrategy::Premium),
    );
    assert!(
        all.iter()
            .all(|&s| price_at(&game, s) == PriceStrategy::Premium)
    );
    set(
        &mut game,
        StrategyScope::Continent(europa),
        StrategyValue::Price(PriceStrategy::Fight),
    );
    assert!(
        all.iter()
            .all(|&s| price_at(&game, s) == PriceStrategy::Fight)
    );
    let margin = PriceStrategy::MinMargin(0.2);
    set(
        &mut game,
        StrategyScope::Country(aaa),
        StrategyValue::Price(margin),
    );
    set(
        &mut game,
        StrategyScope::Site(a1),
        StrategyValue::Price(PriceStrategy::Market),
    );
    assert_eq!(price_at(&game, a1), PriceStrategy::Market, "the site's own");
    assert_eq!(price_at(&game, a2), margin, "its country's");
    assert_eq!(price_at(&game, b1), PriceStrategy::Fight, "its continent's");
    assert_eq!(
        strategy::effective(
            &c,
            game.state(),
            game.player(),
            Unit::Site(a2),
            StrategyField::Price
        ),
        (
            Some(StrategyValue::Price(margin)),
            Some(StrategyScope::Country(aaa))
        )
    );
    // Without its own setting the site follows its country again.
    remove(&mut game, StrategyScope::Site(a1), StrategyField::Price);
    assert_eq!(price_at(&game, a1), margin);
    // Changing a setting replaces it.
    set(
        &mut game,
        StrategyScope::Country(aaa),
        StrategyValue::Price(PriceStrategy::Premium),
    );
    assert_eq!(price_at(&game, a2), PriceStrategy::Premium);
    assert_eq!(game.state().companies[0].strategies.len(), 3);

    // The view: per unit what holds, where it comes from, whether the unit set it.
    let v = crate::views::strategy(&game);
    let entry = |unit: &str, field: &str| {
        v.units
            .iter()
            .find(|u| u.key == unit)
            .unwrap_or_else(|| panic!("{unit}"))
            .entries
            .iter()
            .find(|e| e.field == field)
            .unwrap()
            .clone()
    };
    let keys: Vec<&str> = v.units.iter().map(|u| u.key.as_str()).collect();
    let site = |s: SiteId| format!("standort:{}", s.0);
    assert_eq!(
        keys,
        [
            "firma",
            "kontinent:europa",
            "land:AAA",
            &site(a1),
            &site(a2),
            "land:BBB",
            &site(b1)
        ]
    );
    let firma = entry("firma", "preis");
    assert!(firma.own);
    assert_eq!(
        firma.value,
        Some(StrategyValue::Price(PriceStrategy::Premium))
    );
    let europe = entry("kontinent:europa", "preis");
    assert!(europe.own);
    assert_eq!(europe.origin.as_deref(), Some("kontinent:europa"));
    let at_a1 = entry(&site(a1), "preis");
    assert!(!at_a1.own);
    assert_eq!(at_a1.origin.as_deref(), Some("land:AAA"));
    let at_b1 = entry(&site(b1), "preis");
    assert_eq!(at_b1.origin.as_deref(), Some("kontinent:europa"));
    // Other fields keep their defaults.
    let stock = entry(&site(a1), "lager");
    assert_eq!(stock.origin, None);
    assert_eq!(
        stock.value,
        Some(StrategyValue::Stock(
            strategy::SiteStrategy::defaults(&c).stock
        ))
    );
    assert_eq!(entry("firma", "investition").value, None);
    // Nobody carries the strategies out: the player decides himself.
    assert!(at_a1.carrier.is_none());
    let id = free(&game)[0];
    game.apply(Command::HireManager {
        manager: id,
        position: head(a1),
    })
    .unwrap();
    let v = crate::views::strategy(&game);
    let carrier = v.units.iter().find(|u| u.key == site(a1)).unwrap().entries[0]
        .carrier
        .clone()
        .expect("the head carries the price out");
    assert_eq!(carrier.position.role, "leitung");
    assert_eq!(carrier.manager, game.state().managers[&id].name);
}

#[test]
fn strategies_are_checked() {
    let mut game = new_game(test_support::management());
    let works = found(&mut game, SiteType::Factory);
    let max_wage = game.catalog().production_model.wage_premium_max;
    let wrong = [
        StrategyValue::Price(PriceStrategy::MinMargin(-0.1)),
        StrategyValue::Price(PriceStrategy::MinMargin(5.0)),
        StrategyValue::Stock(StockStrategy {
            input_min_days: 0.0,
            input_max_days: 20.0,
            output_days: 14.0,
        }),
        StrategyValue::Stock(StockStrategy {
            input_min_days: 30.0,
            input_max_days: 20.0,
            output_days: 14.0,
        }),
        StrategyValue::Stock(StockStrategy {
            input_min_days: 7.0,
            input_max_days: 500.0,
            output_days: 14.0,
        }),
        StrategyValue::Stock(StockStrategy {
            input_min_days: 7.0,
            input_max_days: 20.0,
            output_days: f64::NAN,
        }),
        StrategyValue::Wages(WageStrategy { min: 0.2, max: 0.1 }),
        StrategyValue::Wages(WageStrategy {
            min: 0.0,
            max: max_wage + 0.1,
        }),
        StrategyValue::Investment(usd(-1.0)),
        StrategyValue::Reserve(30.0),
    ];
    for value in wrong {
        assert_eq!(
            game.apply(Command::SetStrategy {
                scope: StrategyScope::Company,
                field: value.field(),
                value: Some(value),
            }),
            Err(CommandError::InvalidStrategy),
            "{value:?}"
        );
    }
    // A value for another field.
    assert_eq!(
        game.apply(Command::SetStrategy {
            scope: StrategyScope::Company,
            field: StrategyField::Stock,
            value: Some(StrategyValue::Reserve(1.0)),
        }),
        Err(CommandError::InvalidStrategy)
    );
    assert_eq!(
        game.apply(Command::SetStrategy {
            scope: StrategyScope::Site(SiteId(99)),
            field: StrategyField::Price,
            value: Some(StrategyValue::Price(PriceStrategy::Premium)),
        }),
        Err(CommandError::UnknownSite)
    );
    // Without managers there is nobody to follow strategies.
    let mut plain = test_support::management();
    plain.management.functions.clear();
    let mut alone = new_game(plain);
    assert_eq!(
        alone.apply(Command::SetStrategy {
            scope: StrategyScope::Company,
            field: StrategyField::Reserve,
            value: Some(StrategyValue::Reserve(1.0)),
        }),
        Err(CommandError::InvalidStrategy)
    );
    assert!(game.state().companies[0].strategies.is_empty());
    set(
        &mut game,
        StrategyScope::Site(works),
        StrategyValue::Wages(WageStrategy { min: 0.1, max: 0.2 }),
    );
    assert_eq!(game.state().companies[0].strategies.len(), 1);
}

#[test]
fn the_price_strategy_sets_floor_and_start_markup() {
    let (mut game, works, _) = weak_works(test_support::management());
    let iron = game.catalog().products.id("eisen").unwrap();
    let sale = |game: &Game| {
        proposed(game, works)
            .into_iter()
            .find_map(|c| match c {
                Command::SetSale {
                    product,
                    mode: Some(PriceMode::Market { markup, floor }),
                    ..
                } if product == iron => Some((markup, floor.to_usd())),
                _ => None,
            })
            .expect("the routine prices the iron")
    };
    let (markup, market) = sale(&game);
    assert_eq!(markup, 0.0);
    assert!(market > 0.0);
    let rules = game.catalog().ai_model.behavior.floor_factor.at(0.5);
    let check = |game: &mut Game, price: PriceStrategy, (factor, start): (f64, f64)| {
        set(
            game,
            StrategyScope::Site(works),
            StrategyValue::Price(price),
        );
        let (markup, floor) = sale(game);
        assert_eq!(markup, start, "{price:?}");
        // Floors are rounded to whole money units.
        assert!(
            (floor / market - factor / rules).abs() < 1e-4,
            "{price:?}: {floor} against {market}"
        );
    };
    let m = game.catalog().management.strategy;
    check(&mut game, PriceStrategy::Premium, m.premium);
    check(&mut game, PriceStrategy::Fight, m.fight);
    check(&mut game, PriceStrategy::MinMargin(0.5), (1.5, 0.0));
    // A fixed price of the player stays.
    game.apply(Command::SetSale {
        site: works,
        product: iron,
        mode: Some(PriceMode::Fixed(usd(500.0))),
        keep: 0.0,
    })
    .unwrap();
    assert!(!proposed(&game, works).iter().any(|c| matches!(
        c,
        Command::SetSale { product, .. } if *product == iron
    )));
}

#[test]
fn the_stock_strategy_sets_the_reach() {
    let (mut game, works, _) = weak_works(test_support::management());
    let ore = game.catalog().products.id("erz").unwrap();
    let target = |game: &Game| {
        proposed(game, works)
            .into_iter()
            .find_map(|c| match c {
                Command::SetPurchase {
                    product, target, ..
                } if product == ore => Some(target),
                _ => None,
            })
            .expect("the routine orders ore")
    };
    let normal = target(&game);
    let d = strategy::SiteStrategy::defaults(game.catalog()).stock;
    set(
        &mut game,
        StrategyScope::Company,
        StrategyValue::Stock(StockStrategy {
            input_max_days: 2.0 * d.input_max_days,
            ..d
        }),
    );
    assert!((target(&game) - 2.0 * normal).abs() < 1e-9 * normal);

    // The stock of finished goods: without a target the furnaces slow down, with a long
    // one they speed up (once the iron sells and its stock is low: moved to the mine).
    days(&mut game, 20);
    let iron = game.catalog().products.id("eisen").unwrap();
    let stock = game.state().sites[works.index()].inventory[&iron].quantity;
    game.apply(Command::TransferGoods {
        from: works,
        to: SiteId(0),
        product: iron,
        quantity: stock - 1.0,
    })
    .unwrap();
    let utilization = |game: &Game| {
        proposed(game, works).into_iter().find_map(|c| match c {
            Command::SetProduction {
                site, utilization, ..
            } if site == works => Some(utilization),
            _ => None,
        })
    };
    let with_output = |game: &mut Game, days: f64| {
        set(
            game,
            StrategyScope::Company,
            StrategyValue::Stock(StockStrategy {
                output_days: days,
                ..d
            }),
        );
        utilization(game).unwrap_or(1.0)
    };
    let low = with_output(&mut game, 0.0);
    let high = with_output(&mut game, 180.0);
    assert!(low < high, "{low} {high}");
}

#[test]
fn wages_stay_within_the_strategy() {
    let (mut game, works, _) = weak_works(test_support::management());
    let premium = |game: &Game| {
        proposed(game, works).into_iter().find_map(|c| match c {
            Command::SetWagePremium { premium, .. } => Some(premium),
            _ => None,
        })
    };
    assert_eq!(premium(&game), None, "no premium without a shortage");
    set(
        &mut game,
        StrategyScope::Site(works),
        StrategyValue::Wages(WageStrategy { min: 0.1, max: 0.2 }),
    );
    assert_eq!(premium(&game), Some(0.1), "up to the least premium");
}

#[test]
fn make_or_buy_steers_deliveries() {
    let mut game = new_game(test_support::management());
    let c = game.catalog().clone();
    let (mine, works) = mine_and_works(&mut game);
    let ore = c.products.id("erz").unwrap();
    game.apply(Command::SetSale {
        site: mine,
        product: ore,
        mode: Some(PriceMode::Market {
            markup: 0.0,
            floor: Money::ZERO,
        }),
        keep: 0.0,
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site: works,
        slot: 0,
        recipe: c.recipes.id("eisen_schmelzen"),
        utilization: 1.0,
    })
    .unwrap();
    days(&mut game, 45);
    let delivers = |game: &Game| {
        proposed(game, works)
            .iter()
            .any(|c| matches!(c, Command::TransferGoods { from, .. } if *from == mine))
    };
    assert!(delivers(&game), "own ore first");
    set(
        &mut game,
        StrategyScope::Site(works),
        StrategyValue::Supply(SupplyStrategy::Buy),
    );
    assert!(!delivers(&game), "bought on the market");
    // By price: only while the mine's price is not above the market.
    set(
        &mut game,
        StrategyScope::Site(works),
        StrategyValue::Supply(SupplyStrategy::ByPrice),
    );
    let market = crate::market::market_price(&c, game.state(), c.countries.id("AAA").unwrap(), ore);
    game.apply(Command::SetSale {
        site: mine,
        product: ore,
        mode: Some(PriceMode::Fixed(market.scale(3.0).max(usd(1.0)))),
        keep: 0.0,
    })
    .unwrap();
    assert!(!delivers(&game), "the mine sells dearer than the market");
    game.apply(Command::SetSale {
        site: mine,
        product: ore,
        mode: Some(PriceMode::Fixed(Money::from_units(1))),
        keep: 0.0,
    })
    .unwrap();
    assert!(delivers(&game), "the mine's ore is cheaper");
}

/// A decision to build a furnace at the works.
fn build(game: &Game, works: SiteId) -> Decision {
    let ofen = game.catalog().facilities.id("ofen").unwrap();
    Decision::new(
        Topic::Expansion,
        game.player(),
        Choice::one(
            ChoiceKind::Build,
            Command::BuildFacility {
                site: works,
                facility: ofen,
                count: 1,
                size: FacilitySize::Medium,
            },
        ),
    )
    .at(works)
}

#[test]
fn investments_stay_within_budget_and_reserve() {
    let (mut game, works, _) = weak_works(cheap_furnaces());
    let d = build(&game, works);
    let invest = crate::decision::amount(
        game.catalog(),
        game.state(),
        game.player(),
        &d.choices[0].steps[0].command,
    );
    assert!(invest > Money::ZERO);
    // Within the head's budget and without strategies the head builds.
    let (verdict, concerns, invested) = management::decide_now(game.state(), game.catalog(), &d);
    assert_eq!(verdict, Verdict::Rule);
    assert!(concerns.is_empty());
    assert_eq!(invested, vec![(Unit::Site(works), invest)]);

    // A budget of the company too small: the player is asked.
    let aaa = game.catalog().countries.id("AAA").unwrap();
    set(
        &mut game,
        StrategyScope::Company,
        StrategyValue::Investment(invest.scale(10.0)),
    );
    set(
        &mut game,
        StrategyScope::Country(aaa),
        StrategyValue::Investment(invest.scale(0.5)),
    );
    let (verdict, concerns, invested) = management::decide_now(game.state(), game.catalog(), &d);
    assert_eq!(verdict, Verdict::Hold);
    assert!(invested.is_empty());
    assert_eq!(concerns[0].reason, ConcernReason::Investment);
    // The concern names the budget that binds and what is left of it.
    let mut concern = concerns[0].clone();
    concern.id = 1;
    game.state_mut().concerns.push(concern);
    let v = crate::views::concerns(&game);
    let shown = &v.open[0].concerns[0];
    assert_eq!(shown.reason, "investition");
    assert_eq!(shown.strategy_scope.as_deref(), Some("land:AAA"));
    assert_eq!(shown.strategy_limit_usd, Some(invest.scale(0.5).to_usd()));
    game.state_mut().concerns.clear();

    // Within both budgets the head builds; what it decided counts against both.
    set(
        &mut game,
        StrategyScope::Country(aaa),
        StrategyValue::Investment(invest.scale(1.5)),
    );
    let (verdict, _, invested) = management::decide_now(game.state(), game.catalog(), &d);
    assert_eq!(verdict, Verdict::Rule);
    let year = game.date().year();
    let catalog = game.catalog().clone();
    let player = game.player();
    for (place, amount) in invested {
        strategy::count_investment(&catalog, game.state_mut(), player, place, (amount, year));
    }
    let left: Vec<Money> = strategy::investment_budgets(
        &catalog,
        game.state(),
        game.player(),
        Unit::Site(works),
        year,
    )
    .iter()
    .map(|b| b.left)
    .collect();
    assert_eq!(left, vec![invest.scale(9.0), invest.scale(0.5)]);
    let (verdict, concerns, _) = management::decide_now(game.state(), game.catalog(), &d);
    assert_eq!(verdict, Verdict::Hold, "the country's budget is used up");
    assert_eq!(concerns[0].reason, ConcernReason::Investment);
    // A new year starts every budget afresh.
    let next: Vec<Money> = strategy::investment_budgets(
        &catalog,
        game.state(),
        game.player(),
        Unit::Site(works),
        year + 1,
    )
    .iter()
    .map(|b| b.left)
    .collect();
    assert_eq!(next, vec![invest.scale(10.0), invest.scale(1.5)]);

    // The liquidity reserve: no investment that brings the cash below it.
    remove(&mut game, StrategyScope::Company, StrategyField::Investment);
    remove(
        &mut game,
        StrategyScope::Country(aaa),
        StrategyField::Investment,
    );
    set(
        &mut game,
        StrategyScope::Company,
        StrategyValue::Reserve(1.0),
    );
    let monthly = strategy::monthly_cost(&catalog, game.state(), player);
    assert!(monthly > Money::ZERO);
    let cash = game.state().companies[player.index()].ledger.cash();
    // Spend down to just above the reserve (as if by other costs).
    let spend = cash - monthly - invest.scale(0.5);
    game.state_mut().companies[player.index()].ledger.expense(
        CostType::Other,
        CostCenter {
            site: None,
            product: None,
        },
        Account::Cash,
        spend,
    );
    let (verdict, concerns, _) = management::decide_now(game.state(), game.catalog(), &d);
    assert_eq!(verdict, Verdict::Hold);
    assert_eq!(concerns[0].reason, ConcernReason::Reserve);
    let mut concern = concerns[0].clone();
    concern.id = 2;
    game.state_mut().concerns.push(concern);
    let v = crate::views::concerns(&game);
    assert_eq!(v.open[0].concerns[0].reason, "reserve");
    assert_eq!(
        v.open[0].concerns[0].strategy_limit_usd,
        Some(monthly.to_usd())
    );
    // A reserve set lower for the works lets it build.
    set(
        &mut game,
        StrategyScope::Site(works),
        StrategyValue::Reserve(0.0),
    );
    let (verdict, _, _) = management::decide_now(game.state(), game.catalog(), &d);
    assert_eq!(verdict, Verdict::Rule);
    assert!(game.state().companies[player.index()].ledger.is_balanced());
}

#[test]
fn strategies_replay_and_load_identically() {
    let catalog = Arc::new(test_support::management());
    let setup = |game: &mut Game| {
        let (mine, works) = mine_and_works(game);
        let ids = free(game);
        for (manager, site) in [(ids[0], mine), (ids[1], works)] {
            game.apply(Command::HireManager {
                manager,
                position: head(site),
            })
            .unwrap();
        }
        let europa = game.catalog().continents.id("europa").unwrap();
        set(
            game,
            StrategyScope::Continent(europa),
            StrategyValue::Price(PriceStrategy::Premium),
        );
        set(
            game,
            StrategyScope::Site(works),
            StrategyValue::Stock(StockStrategy {
                input_min_days: 10.0,
                input_max_days: 40.0,
                output_days: 20.0,
            }),
        );
        set(
            game,
            StrategyScope::Company,
            StrategyValue::Investment(usd(5_000.0)),
        );
    };
    let play = |game: &mut Game| {
        for _ in 0..5 {
            game.advance(RoundLength::Week, |_| {});
        }
    };
    let mut a = new_game((*catalog).clone());
    setup(&mut a);
    play(&mut a);
    play(&mut a);

    let mut b = new_game((*catalog).clone());
    setup(&mut b);
    play(&mut b);
    let mut loaded = save::decode(&save::encode(&b), catalog.clone())
        .unwrap()
        .game;
    assert_eq!(
        loaded.state().companies[0].strategies,
        b.state().companies[0].strategies
    );
    play(&mut loaded);
    assert_eq!(loaded.state_hash(), a.state_hash());

    let replayed = Game::replay(catalog, a.state().settings.clone(), a.journal()).unwrap();
    assert_eq!(replayed.state_hash(), a.state_hash());
}

#[test]
fn a_site_sold_takes_no_strategy_along() {
    let mut game = new_game(test_support::management());
    let works = found(&mut game, SiteType::Factory);
    set(
        &mut game,
        StrategyScope::Site(works),
        StrategyValue::Price(PriceStrategy::Premium),
    );
    set(
        &mut game,
        StrategyScope::Company,
        StrategyValue::Reserve(2.0),
    );
    management::release_site(game.state_mut(), works);
    let left: Vec<StrategyScope> = game.state().companies[0]
        .strategies
        .iter()
        .map(|s| s.scope)
        .collect();
    assert_eq!(left, vec![StrategyScope::Company]);
}

#[test]
fn sales_channels_show_in_the_strategy_view() {
    use crate::policy::{BuyerGroup, SalesRule, Scope};
    let (mut game, _, _) = weak_works(test_support::management());
    let c = game.catalog().clone();
    let (iron, aaa) = (
        c.products.id("eisen").unwrap(),
        c.countries.id("AAA").unwrap(),
    );
    for (buyer, scope, rule) in [
        (
            BuyerGroup::Traders,
            Scope::ProductInCountry(iron, aaa),
            SalesRule {
                allowed: true,
                min_price: Some(usd(40.0)),
                max_per_month: Some(500.0),
            },
        ),
        (
            BuyerGroup::Companies,
            Scope::Company,
            SalesRule {
                allowed: false,
                min_price: None,
                max_per_month: None,
            },
        ),
    ] {
        game.apply(Command::SetSalesPolicy {
            buyer,
            scope,
            rule: Some(rule),
        })
        .unwrap();
    }
    let v = crate::views::strategy(&game);
    let rules: Vec<(&str, Option<&str>, Option<&str>, bool)> = v
        .sales
        .iter()
        .map(|r| {
            (
                r.buyer.as_str(),
                r.product.as_deref(),
                r.country.as_deref(),
                r.allowed,
            )
        })
        .collect();
    // The company's rule first, then the narrower one.
    assert_eq!(
        rules,
        vec![
            ("firmen", None, None, false),
            ("haendler", Some("eisen"), Some("AAA"), true)
        ]
    );
    let narrow = &v.sales[1];
    assert_eq!(narrow.min_price_usd, Some(40.0));
    assert_eq!(narrow.max_per_month, Some(500.0));
    assert!(
        narrow
            .unit
            .as_deref()
            .is_some_and(|u| u.starts_with("einheit."))
    );
    // What new rules can name: the iron the works offers, the country of the sites.
    assert!(v.sale_products.iter().any(|p| p.product == "eisen"));
    assert_eq!(v.sale_countries, vec!["AAA".to_owned()]);
}
