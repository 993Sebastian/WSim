//! Scenario tests for regulation and environment (H2) with the small chain of
//! `test_support::production`: the mine of AAA emits.

use crate::calendar::{Date, RoundLength};
use crate::catalog::{
    Catalog, CountrySeries, Regulation, RegulationKind, RetrofitLevel, test_support,
};
use crate::command::{Command, CommandError};
use crate::facility_tests::{mine, new_game};
use crate::game::Game;
use crate::ledger::{Account, CostType};
use crate::message::keys;
use crate::money::Money;
use crate::regulation;
use crate::save;
use crate::state::{Holder, Limit, PriceMode, SaleOffer, SiteId, Stake};
use crate::strategy::{StrategyField, StrategyScope, StrategyValue};
use crate::time_series::TimeSeries;

fn date(y: i32, m: u32, d: u32) -> Date {
    Date::new(y, m, d).unwrap()
}

fn flat(v: f64) -> CountrySeries {
    CountrySeries {
        default: Some(TimeSeries::new(vec![(1900, v), (2100, v)]).expect("valid")),
        countries: Vec::new(),
    }
}

/// The ore mine emits 1 t CO2 and 10 kg of pollutants per t; CO2 costs 10 USD per t; two
/// retrofit levels, the second from 1950; `rules` apply in AAA.
fn catalog(rules: Vec<(Date, RegulationKind)>) -> Catalog {
    let mut c = test_support::production();
    let ore = c.recipes.id("erz_abbau").unwrap();
    let r = c.recipes.get_mut(ore);
    r.co2_t = 1.0;
    r.pollutant_kg = 10.0;
    c.environment.retrofit = vec![
        RetrofitLevel {
            from_year: 1900,
            reduction: 0.5,
            cost_share: 0.1,
        },
        RetrofitLevel {
            from_year: 1950,
            reduction: 0.5,
            cost_share: 0.1,
        },
    ];
    c.environment.co2_price = flat(10.0);
    c.environment.antitrust_share_max = 0.4;
    c.environment.image_weight = 0.3;
    let aaa = c.countries.id("AAA").unwrap();
    c.regulations = rules
        .into_iter()
        .enumerate()
        .map(|(i, (date, kind))| Regulation {
            key: format!("regel_{i}"),
            date,
            countries: vec![aaa],
            kind,
            provenance: Default::default(),
        })
        .collect();
    c
}

fn days(game: &mut Game, n: u32) -> Vec<String> {
    let mut keys = Vec::new();
    for _ in 0..n {
        let report = game.advance(RoundLength::Day, |_| {});
        keys.extend(report.messages.into_iter().map(|m| m.key));
    }
    keys
}

fn balanced(game: &Game) -> bool {
    game.state()
        .companies
        .iter()
        .all(|c| c.ledger.is_balanced())
}

#[test]
fn production_pays_the_co2_price_and_counts_the_emissions() {
    let mut game = new_game(catalog(Vec::new()));
    let site = mine(&mut game);
    days(&mut game, 45);
    let player = game.player();
    let c = &game.state().companies[player.index()];
    let co2 = c.emissions.co2_t + c.emissions_last.co2_t;
    assert!(co2 > 0.0);
    let paid = -c.ledger.year.by_type[&CostType::Environment];
    assert!((paid.to_usd() - co2 * 10.0).abs() < 0.01 * co2 * 10.0 + 0.01);
    // Pollutants: 10 kg per t, none retrofitted.
    let p = c.emissions.pollutant_kg + c.emissions_last.pollutant_kg;
    assert!((p - co2 * 10.0).abs() < 1e-6 * p);
    // The unit cost the pricing sees includes the CO2 price.
    let ore = game.catalog().products.id("erz").unwrap();
    let costs = crate::production::unit_costs(game.catalog(), game.state(), site);
    assert!(costs.iter().find(|u| u.product == ore).unwrap().rent >= 10.0);
    assert!(balanced(&game));
}

#[test]
fn a_retrofit_rule_is_met_by_the_sites_themselves() {
    let rules = vec![(
        date(1900, 2, 10),
        RegulationKind::Retrofit {
            level: 1,
            months: 6,
        },
    )];
    let mut game = new_game(catalog(rules));
    let site = mine(&mut game);
    let before = game.state().site(site).unwrap().slots[0].cost;
    let keys = days(&mut game, 45);
    assert!(keys.iter().any(|k| k == keys::REGULATION_NEW), "{keys:?}");
    assert!(keys.iter().any(|k| k == keys::REGULATION_RETROFITTED));
    let sl = &game.state().site(site).unwrap().slots[0];
    assert_eq!(sl.retrofit, 1);
    // A tenth of the investment, booked as fixed assets.
    assert_eq!(sl.cost, before + before.scale(0.1));
    // Half the pollutants per t from now on.
    let recipe = game.catalog().recipes.get(sl.recipe.unwrap());
    assert_eq!(regulation::pollutant(game.catalog(), recipe, 1), 5.0);
    assert!(balanced(&game));
}

#[test]
fn units_short_of_the_required_level_stand_still() {
    // Level 2 is only available from 1950: the mine cannot meet the rule.
    let rules = vec![(
        date(1900, 1, 1),
        RegulationKind::Retrofit {
            level: 2,
            months: 1,
        },
    )];
    let mut game = new_game(catalog(rules));
    let site = mine(&mut game);
    days(&mut game, 45);
    let sl = &game.state().site(site).unwrap().slots[0];
    assert_eq!(sl.retrofit, 1, "the level available is taken");
    assert_eq!(sl.limit, Some(Limit::Regulation));
    assert_eq!(sl.last_runs, 0.0);
}

#[test]
fn bans_stop_production_and_safety_raises_wages() {
    let ore = test_support::production().products.id("erz").unwrap();
    let rules = vec![
        (
            date(1900, 1, 1),
            RegulationKind::Ban {
                products: vec![ore],
                production: true,
                sales: true,
            },
        ),
        (
            date(1900, 1, 1),
            RegulationKind::Safety {
                wage_surcharge: 0.1,
            },
        ),
    ];
    let plain = new_game(catalog(Vec::new()));
    let mut game = new_game(catalog(rules));
    let aaa = game.catalog().countries.id("AAA").unwrap();
    let w0 = plain.state().countries.get(aaa).hourly_wage_usd[0];
    let w1 = game.state().countries.get(aaa).hourly_wage_usd[0];
    assert!((w1 - w0 * 1.1).abs() < 1e-9 * w0);
    let site = mine(&mut game);
    days(&mut game, 45);
    let sl = &game.state().site(site).unwrap().slots[0];
    assert_eq!(sl.limit, Some(Limit::Regulation));
    assert!(game.state().regulation.sales_banned(aaa, ore));
}

#[test]
fn retrofitting_by_command_and_overfulfilling_by_strategy() {
    let mut game = new_game(catalog(Vec::new()));
    let site = mine(&mut game);
    days(&mut game, 1);
    // A facility without pollutants has nothing to retrofit.
    assert_eq!(
        game.apply(Command::Retrofit {
            site: SiteId(99),
            slot: 0
        }),
        Err(CommandError::UnknownSite)
    );
    game.apply(Command::Retrofit { site, slot: 0 }).unwrap();
    assert_eq!(game.state().site(site).unwrap().slots[0].retrofit, 1);
    // Level 2 comes in 1950.
    assert_eq!(
        game.apply(Command::Retrofit { site, slot: 0 }),
        Err(CommandError::NoRetrofit)
    );
    // Overfulfilling without any rule: the best level available, at the month start.
    let mut game = new_game(catalog(Vec::new()));
    let site = mine(&mut game);
    // The test chain has no management: the setting goes straight into the state.
    let player = game.player();
    game.state_mut().companies[player.index()]
        .strategies
        .push(crate::strategy::StrategySetting {
            scope: StrategyScope::Company,
            value: StrategyValue::Environment(true),
            spent: Money::ZERO,
            year: 0,
        });
    assert_eq!(
        StrategyValue::Environment(true).field(),
        StrategyField::Environment
    );
    days(&mut game, 35);
    assert_eq!(game.state().site(site).unwrap().slots[0].retrofit, 1);
}

#[test]
fn dirtier_companies_gain_less_awareness() {
    let mut game = new_game(catalog(Vec::new()));
    let other = crate::trade_tests::competitor(&mut game);
    let player = game.player();
    let s = game.state_mut();
    for (c, p) in [(player, 100.0), (other, 10.0)] {
        let co = &mut s.companies[c.index()];
        co.emissions_last.pollutant_kg = p;
        let mut m = co.ledger.month.clone();
        m.by_type
            .insert(CostType::Revenue, Money::from_usd(1000.0).unwrap());
        co.ledger.months.push(m);
    }
    let f = regulation::image_factors(game.catalog(), game.state());
    assert!(f[player.index()] < 1.0);
    assert!(f[other.index()] > 1.0);
    // x / x̄ = (100 / 1000) / (110 / 2000) = 1.82: the factor is 1 − 0.3 · 0.82.
    let expected = 1.0 - 0.3 * (0.1 / 0.055 - 1.0);
    assert!((f[player.index()] - expected).abs() < 1e-9, "{f:?}");
}

#[test]
fn antitrust_stops_a_takeover_that_dominates_a_market() {
    let rules = vec![(date(1900, 1, 1), RegulationKind::Antitrust)];
    let mut game = new_game(catalog(rules));
    let site = mine(&mut game);
    let ore = game.catalog().products.id("erz").unwrap();
    let other = crate::trade_tests::competitor(&mut game);
    let player = game.player();
    // A second site of the competitor; both sold ore last month.
    let offer = SaleOffer {
        mode: PriceMode::Fixed(Money::from_usd(10.0).unwrap()),
        price: Money::from_usd(10.0).unwrap(),
        keep: 0.0,
        sold_today: 0.0,
        sold_month: 0.0,
        sold_last_month: 30.0,
        to_traders_month: 0.0,
        to_companies_month: 0.0,
    };
    {
        let s = game.state_mut();
        let mut second = s.sites[site.index()].clone();
        second.owner = other;
        second.offers.insert(ore, offer.clone());
        s.sites.push(second);
        s.sites[site.index()].offers.insert(ore, offer);
        s.companies[other.index()].owners = vec![Stake {
            holder: Holder::Private,
            share: 1.0,
        }];
    }
    let catalog = game.catalog().clone();
    let (p, _, share) = regulation::antitrust(&catalog, game.state(), player, other).unwrap();
    assert_eq!(p, ore);
    assert_eq!(share, 1.0);
    let ask = crate::holdings::ask(&catalog, game.state(), other, 0.6);
    assert!(matches!(
        game.apply(Command::BidForStake {
            company: other,
            holder: Holder::Private,
            share: 0.6,
            price: ask,
        }),
        Err(CommandError::Antitrust { .. })
    ));
    // A minority stays allowed.
    let ask = crate::holdings::ask(&catalog, game.state(), other, 0.3);
    game.apply(Command::BidForStake {
        company: other,
        holder: Holder::Private,
        share: 0.3,
        price: ask,
    })
    .unwrap();
    assert!(
        game.state().companies[player.index()]
            .ledger
            .balance(Account::Participations)
            > Money::ZERO
    );
}

#[test]
fn regulation_state_survives_saving() {
    let rules = vec![(
        date(1900, 2, 10),
        RegulationKind::Retrofit {
            level: 1,
            months: 6,
        },
    )];
    let mut game = new_game(catalog(rules));
    mine(&mut game);
    days(&mut game, 45);
    let mut loaded = save::decode(&save::encode(&game), game.catalog().clone())
        .unwrap()
        .game;
    assert_eq!(game.state_hash(), loaded.state_hash());
    days(&mut game, 40);
    days(&mut loaded, 40);
    assert_eq!(game.state_hash(), loaded.state_hash());
}
