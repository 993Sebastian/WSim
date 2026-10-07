//! Scenario tests for the start-ups (SU1) with `test_support::research` and
//! `test_support::management`.

use std::sync::Arc;

use crate::board_tests::member;
use crate::calendar::{Date, RoundLength};
use crate::catalog::{
    Catalog, DepartmentKind, Inventor, Provenance, Technology, VentureAiModel, VentureModel,
    VenturePhase, VentureStakeModel, test_support,
};
use crate::command::Command;
use crate::game::Game;
use crate::management_tests::{hire_sharp, mine_and_works, usd};
use crate::message::keys;
use crate::research;
use crate::state::{
    GameSettings, Holder, StartForm, Venture, VentureFailure, VentureStatus, VentureTarget,
};
use crate::ventures;

/// Two short sure phases, investors at once, one start-up a month.
fn model() -> VentureModel {
    let phase = |key: &str, months, capital, valuation| VenturePhase {
        key: key.to_owned(),
        months,
        capital: usd(capital),
        chance: 1.0,
        valuation,
    };
    VentureModel {
        labels: vec![("erfinder".into(), 1900), ("startups".into(), 1990)],
        per_year: 12.0,
        frequencies: vec![("keine".into(), 0.0), ("normal".into(), 1.0)],
        default_frequency: 1,
        new_share: 1.0,
        lead_years_max: 15,
        phases: vec![
            phase("idee", 2, 100_000.0, 1.0),
            phase("prototyp", 3, 200_000.0, 2.0),
        ],
        reference_gdp_usd: 5_000.0,
        capital_factor: (0.05, 2.0),
        lead_capital: 0.1,
        lead_chance: 0.0,
        chance_min: 0.05,
        investor_chance: 1.0,
        deadline_months: 3,
        stakes: VentureStakeModel {
            success_factor: 1.5,
            buy_premium: 0.2,
            sale_discount: 0.2,
            grant_effect: 0.5,
            blocking: 0.25,
            majority: 0.5,
            research_bonus: 0.2,
            chance_max: 1.0,
            fast: (0.5, 0.9),
            thorough: (1.5, 1.1),
            min_return: 1.0,
            cash_share: 0.5,
            spin_off_progress_min: 0.1,
            ai: VentureAiModel {
                check_chance: 1.0,
                cash_min: usd(100_000.0),
                cash_share: 0.5,
                min_return: 0.0,
                takeover_chance_min: 0.3,
                takeover_share_min: 0.1,
                takeover_cash: 0.5,
                spin_off_chance: 1.0,
                spin_off_sale: 0.4,
            },
        },
        blur: 0.5,
        chance_levels: (0.2, 0.4),
        keep_years: 1,
        inventors: Vec::new(),
        provenance: Provenance::default(),
    }
}

fn catalog_with(model: VentureModel) -> Catalog {
    let mut c = test_support::research();
    c.ventures = model;
    c
}

fn game_with(catalog: Catalog, seed: u64, ventures: f64) -> Game {
    let catalog = Arc::new(catalog);
    let settings = GameSettings {
        seed,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(1_000_000.0),
        start_form: StartForm::Workshop,
        company_name: "Labor AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
        ventures,
        tariff_dynamics: 1.0,
    };
    Game::new(catalog, settings).unwrap()
}

/// Days until `date`, with the keys of all messages.
fn until(game: &mut Game, date: Date) -> Vec<String> {
    let mut keys = Vec::new();
    while game.state().date < date {
        let report = game.advance(RoundLength::Day, |_| {});
        keys.extend(report.messages.into_iter().map(|m| m.key));
    }
    keys
}

fn date(y: i32, m: u32, d: u32) -> Date {
    Date::new(y, m, d).unwrap()
}

fn venture(game: &Game, id: u32) -> &Venture {
    game.state().ventures.iter().find(|v| v.id == id).unwrap()
}

/// More technologies a few years ahead, all needing only smelting.
fn add_technologies(c: &mut Catalog, n: usize, year: i32) {
    let smelting = c.technologies.id("schmelzen").unwrap();
    let field = c.technologies.get(smelting).field;
    for i in 0..n {
        c.technologies
            .insert(
                &format!("neu_{i}"),
                Technology {
                    field,
                    invention_year: year,
                    prerequisites: vec![smelting],
                    research_effort: Some(500.0),
                    provenance: Provenance::default(),
                },
            )
            .unwrap();
    }
}

#[test]
fn a_sure_start_up_invents_its_technology_early() {
    let mut game = game_with(catalog_with(model()), 3, 1.0);
    let catalog = game.catalog().clone();
    let turbine = catalog.technologies.id("turbine").unwrap();
    // No start-up before the first month start.
    assert!(game.state().ventures.is_empty());
    until(&mut game, date(1900, 2, 1));
    // The only technology within reach: turbine, two years ahead of history.
    let v = venture(&game, 0).clone();
    assert_eq!(v.target, VentureTarget::Technology(turbine));
    assert_eq!(v.lead, 2.0);
    assert_eq!(v.phase, 0);
    assert_eq!(v.owners.len(), 1);
    assert_eq!(v.owners[0].holder, Holder::Private);
    // Capital at the reference income, 10 % more per year ahead.
    assert_eq!(v.capital, usd(120_000.0));
    assert_eq!(v.round_until, Some(date(1900, 5, 1)));
    // A month later investors fund the round: half the shares at a valuation of 1.
    until(&mut game, date(1900, 3, 1));
    let v = venture(&game, 0).clone();
    assert_eq!(v.round_until, None);
    assert_eq!(v.raised, usd(120_000.0));
    assert_eq!(v.phase_until, Some(date(1900, 5, 1)));
    let share = |v: &Venture, h: Holder| {
        v.owners
            .iter()
            .filter(|s| s.holder == h)
            .map(|s| s.share)
            .sum::<f64>()
    };
    assert!((share(&v, Holder::Private) - 0.5).abs() < 1e-12);
    assert!((share(&v, Holder::Investors) - 0.5).abs() < 1e-12);
    let before = research::effort(&catalog, game.state(), turbine, date(1900, 6, 1))
        .unwrap()
        .factor;
    assert!(before > 1.0, "ahead of history: {before}");
    // Phase two: its own round with a valuation of two dilutes everyone by a third.
    let keys = until(&mut game, date(1900, 9, 2));
    let v = venture(&game, 0).clone();
    assert_eq!(v.status, VentureStatus::Succeeded(date(1900, 9, 1)));
    assert!((share(&v, Holder::Private) - 1.0 / 3.0).abs() < 1e-12);
    assert!((share(&v, Holder::Investors) - 2.0 / 3.0).abs() < 1e-12);
    // The technology counts as invented: researching it gets cheaper for everyone.
    assert_eq!(
        *game.state().inventions.get(turbine),
        Some(date(1900, 9, 1))
    );
    assert!(keys.iter().any(|k| k == keys::VENTURE_INVENTION));
    let after = research::effort(&catalog, game.state(), turbine, date(1901, 1, 1))
        .unwrap()
        .factor;
    assert!(after <= 1.0 && after < before, "{after} after {before}");
    // Nothing left to found a start-up for.
    until(&mut game, date(1901, 1, 1));
    assert_eq!(game.state().ventures.len(), 1);
}

#[test]
fn an_unfunded_round_fails_and_closed_ones_leave_the_list() {
    let mut m = model();
    m.investor_chance = 0.0;
    let mut game = game_with(catalog_with(m), 3, 1.0);
    until(&mut game, date(1900, 5, 2));
    assert_eq!(
        venture(&game, 0).status,
        VentureStatus::Failed(date(1900, 5, 1), VentureFailure::Funding)
    );
    // The technology is free again: a new start-up takes it up.
    until(&mut game, date(1900, 6, 2));
    let next = game.state().ventures.iter().find(|v| v.id == 1).unwrap();
    assert_eq!(next.status, VentureStatus::Active);
    // A year after failing, the first one leaves the list.
    until(&mut game, date(1901, 5, 2));
    assert!(game.state().ventures.iter().all(|v| v.id != 0));
    assert!(
        game.state()
            .ventures
            .iter()
            .all(|v| v.id > 0 && v.owners == crate::state::Stake::sole(Holder::Private))
    );
}

#[test]
fn most_start_ups_fail_by_their_chances() {
    // Thirty technologies, halves at each of two phases: a quarter succeeds.
    let mut m = model();
    m.phases[0].chance = 0.5;
    m.phases[1].chance = 0.5;
    m.per_year = 24.0;
    m.keep_years = 50;
    let mut c = catalog_with(m);
    add_technologies(&mut c, 30, 1914);
    let mut game = game_with(c, 11, 1.0);
    until(&mut game, date(1912, 1, 1));
    let closed: Vec<&Venture> = game
        .state()
        .ventures
        .iter()
        .filter(|v| v.status != VentureStatus::Active)
        .collect();
    let won = closed
        .iter()
        .filter(|v| matches!(v.status, VentureStatus::Succeeded(_)))
        .count();
    // Successes take their technology off the list; at most one each.
    assert!(won <= 31, "{won}");
    let tried = closed.len();
    assert!(tried > 60, "{tried}");
    let failed = (tried - won) as f64 / tried as f64;
    assert!((0.65..0.85).contains(&failed), "{failed}");
}

#[test]
fn the_setting_and_the_mean_set_how_many_are_founded() {
    let mut m = model();
    m.per_year = 6.0;
    m.investor_chance = 0.0;
    let mut c = catalog_with(m);
    add_technologies(&mut c, 10, 1910);
    // Half a start-up a month: about sixty in ten years.
    let mut game = game_with(c.clone(), 5, 1.0);
    until(&mut game, date(1909, 12, 2));
    let founded = game.state().next_venture;
    assert!((40..=80).contains(&founded), "{founded}");
    // Twice the setting, twice as many.
    let mut many = game_with(c.clone(), 5, 2.0);
    until(&mut many, date(1909, 12, 2));
    let more = many.state().next_venture;
    assert!(more > founded + 30, "{more} against {founded}");
    // None with the setting none.
    let mut none = game_with(c, 5, 0.0);
    until(&mut none, date(1903, 1, 1));
    assert_eq!(none.state().next_venture, 0);
    assert!(none.state().ventures.is_empty());
}

#[test]
fn start_ups_are_deterministic() {
    let mut m = model();
    m.phases[0].chance = 0.6;
    m.investor_chance = 0.5;
    m.new_share = 0.5;
    m.keep_years = 10;
    let mut c = catalog_with(m);
    add_technologies(&mut c, 6, 1908);
    c.research_model.development.levels = 3;
    let run = |seed| {
        let mut game = game_with(c.clone(), seed, 1.0);
        until(&mut game, date(1903, 1, 1));
        game.state().ventures.clone()
    };
    let a = run(21);
    assert!(a.len() > 5);
    assert_eq!(a, run(21));
    assert_ne!(a, run(22));
}

#[test]
fn an_improvement_takes_the_next_level_into_the_world() {
    // Without technologies within reach: improvements only, of what the world makes.
    let mut c = test_support::management();
    let mut m = model();
    m.new_share = 0.0;
    m.lead_years_max = 1;
    c.ventures = m;
    c.research_model.development.levels = 3;
    let iron = c.products.id("eisen").unwrap();
    let smelting = c.recipes.id("eisen_schmelzen");
    let mut game = crate::management_tests::new_game(c);
    until(&mut game, date(1900, 2, 2));
    assert!(game.state().ventures.is_empty(), "nobody makes iron yet");
    let (_, works) = mine_and_works(&mut game);
    game.apply(Command::SetProduction {
        site: works,
        slot: 0,
        recipe: smelting,
        utilization: 1.0,
    })
    .unwrap();
    let keys = until(&mut game, date(1901, 3, 1));
    let v = game.state().ventures[0].clone();
    assert_eq!(
        v.target,
        VentureTarget::Development {
            product: iron,
            level: 1
        }
    );
    // No lead for improvements: the capital of the reference.
    assert_eq!(v.lead, 0.0);
    let VentureStatus::Succeeded(when) = v.status else {
        panic!("{:?}", v.status);
    };
    assert_eq!(game.state().developments.get(iron).first(), Some(&when));
    // The player hears of it when it sells iron.
    assert_eq!(
        keys.iter().any(|k| k == keys::VENTURE_DEVELOPMENT),
        research::player_offers(game.state(), iron)
    );
    // The next start-up works on the level after it.
    let next = game
        .state()
        .ventures
        .iter()
        .find(|x| x.id == v.id + 1)
        .unwrap();
    assert_eq!(
        next.target,
        VentureTarget::Development {
            product: iron,
            level: 2
        }
    );
}

#[test]
fn the_world_can_overtake_a_start_up() {
    // Turbine is invented in history in 1902: a slow start-up from 1900 is too late.
    let mut m = model();
    m.phases[1].months = 30;
    let mut game = game_with(catalog_with(m), 3, 1.0);
    until(&mut game, date(1902, 1, 2));
    assert_eq!(
        venture(&game, 0).status,
        VentureStatus::Failed(date(1902, 1, 1), VentureFailure::Overtaken)
    );
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    assert_eq!(*game.state().inventions.get(turbine), None);
}

#[test]
fn a_historical_inventor_founds_once_in_his_country() {
    let mut m = model();
    m.investor_chance = 0.0;
    let mut c = catalog_with(m);
    let turbine = c.technologies.id("turbine").unwrap();
    let bbb = c.countries.id("BBB").unwrap();
    c.ventures.inventors.push(Inventor {
        technology: turbine,
        name: "Ada Muster".into(),
        country: bbb,
        provenance: Provenance::default(),
    });
    let mut game = game_with(c, 3, 1.0);
    until(&mut game, date(1902, 1, 1));
    let first = venture(&game, 0);
    assert_eq!(first.name, "Ada Muster");
    assert_eq!(first.country, bbb);
    assert!(first.inventor);
    // The others after the failure have founders from the name lists.
    let others: Vec<&Venture> = game.state().ventures.iter().filter(|v| v.id > 0).collect();
    assert!(!others.is_empty());
    assert!(others.iter().all(|v| !v.inventor && v.name != "Ada Muster"));
    // The inventor stays in the list after the years of the data.
    assert!(game.state().ventures.iter().any(|v| v.id == 0));
}

#[test]
fn the_strategy_department_sharpens_the_shown_chance() {
    let mut c = test_support::management();
    let mut m = model();
    m.phases[0].chance = 0.5;
    m.phases[1].chance = 0.8;
    m.investor_chance = 0.0;
    m.blur = 0.5;
    c.ventures = m;
    let mut game = crate::management_tests::new_game(c);
    mine_and_works(&mut game);
    until(&mut game, date(1900, 2, 2));
    let catalog = game.catalog().clone();
    let player = game.player();
    let v = game.state().ventures[0].clone();
    let chance = ventures::success_chance(&catalog.ventures, &v);
    assert!((chance - 0.4).abs() < 1e-12);
    // Without a department the draw blurs it by up to half.
    let (shown, works) = ventures::shown_chance(&catalog, game.state(), player, &v);
    assert!(!works);
    assert!((shown - 0.4 * (1.0 + v.blur * 0.5)).abs() < 1e-12);
    // A department with a perfect head sees it as it is.
    game.apply(Command::StaffDepartment {
        department: DepartmentKind::Strategy,
        staff: 1,
    })
    .unwrap();
    hire_sharp(&mut game, member("strategie"));
    let (shown, works) = ventures::shown_chance(&catalog, game.state(), player, &v);
    assert!(works);
    assert!((shown - 0.4).abs() < 1e-12, "{shown}");
}

// --- SU2: stakes of companies ---

use crate::command::CommandError;
use crate::ledger::{Account, CostType};
use crate::money::Money;
use crate::state::{CompanyId, Stake, VentureExit, VenturePace};

/// The turbine start-up of February 1900 with its round still open (no investors).
fn waiting_round() -> Game {
    let mut m = model();
    m.investor_chance = 0.0;
    m.deadline_months = 12;
    let mut game = game_with(catalog_with(m), 3, 1.0);
    until(&mut game, date(1900, 2, 2));
    game
}

fn books(game: &Game) -> &crate::ledger::Ledger {
    &game.state().companies[0].ledger
}

fn invest(game: &mut Game, amount: f64) -> Result<(), CommandError> {
    game.apply(Command::InvestInVenture {
        venture: 0,
        amount: usd(amount),
    })
}

fn by_type(game: &Game, cost: CostType) -> Money {
    books(game)
        .year
        .by_type
        .get(&cost)
        .copied()
        .unwrap_or(Money::ZERO)
}

#[test]
fn pledges_close_the_round_and_buy_shares() {
    let mut game = waiting_round();
    let me = CompanyId(0);
    let cash = books(&game).cash();
    // 120 000 USD wanted: a pledge waits for the rest.
    invest(&mut game, 50_000.0).unwrap();
    let v = venture(&game, 0).clone();
    assert!(v.round_until.is_some());
    assert_eq!(v.raised, usd(50_000.0));
    assert_eq!(books(&game).cash(), cash - usd(50_000.0));
    assert_eq!(books(&game).balance(Account::Participations), usd(50_000.0));
    assert!(books(&game).is_balanced());
    assert_eq!(
        invest(&mut game, 100_000.0),
        Err(CommandError::AmountTooHigh { max: usd(70_000.0) })
    );
    // The rest closes the round today: half the shares at a valuation of 1.
    invest(&mut game, 70_000.0).unwrap();
    let v = venture(&game, 0).clone();
    assert_eq!(v.round_until, None);
    assert_eq!(v.phase_until, Some(date(1900, 4, 2)));
    assert!((ventures::share_of(&v, me) - 0.5).abs() < 1e-12);
    assert!(v.pledges.is_empty());
    assert_eq!(ventures::amount_of(&v.book, me), usd(120_000.0));
    // Counted against the participations budget of the year.
    assert_eq!(
        game.state().companies[0].participations.spent_in(1900),
        usd(120_000.0)
    );
    // Between rounds the founders sell at the value after the round plus 20 %.
    invest(&mut game, 28_800.0).unwrap();
    let v = venture(&game, 0).clone();
    assert!((ventures::share_of(&v, me) - 0.6).abs() < 1e-9);
    let founders: f64 = v
        .owners
        .iter()
        .filter(|s| s.holder == Holder::Private)
        .map(|s| s.share)
        .sum();
    assert!((founders - 0.4).abs() < 1e-9);
    assert_eq!(
        invest(&mut game, 200_000.0),
        Err(CommandError::AmountTooHigh {
            max: usd(115_200.0)
        })
    );
    assert!(books(&game).is_balanced());
}

#[test]
fn a_grant_raises_the_chance_without_shares() {
    let mut m = model();
    m.investor_chance = 0.0;
    m.phases[0].chance = 0.5;
    let mut game = game_with(catalog_with(m), 3, 1.0);
    until(&mut game, date(1900, 2, 2));
    game.apply(Command::GrantVenture {
        venture: 0,
        amount: usd(60_000.0),
    })
    .unwrap();
    // Half the capital closes half the gap times the effect of 0.5.
    let v = venture(&game, 0);
    assert!((v.chance - (0.5 + 0.5 * 0.5 * 0.5)).abs() < 1e-12);
    assert_eq!(ventures::share_of(v, CompanyId(0)), 0.0);
    assert_eq!(by_type(&game, CostType::Research), usd(-60_000.0));
    assert!(books(&game).is_balanced());
}

#[test]
fn a_sale_books_the_gain_or_loss() {
    let mut game = waiting_round();
    invest(&mut game, 120_000.0).unwrap();
    // Worth 240 000 after the round; a quarter sells for 80 % of 60 000.
    game.apply(Command::SellVentureStake {
        venture: 0,
        share: 0.25,
    })
    .unwrap();
    let v = venture(&game, 0).clone();
    assert!((ventures::share_of(&v, CompanyId(0)) - 0.25).abs() < 1e-12);
    assert_eq!(ventures::amount_of(&v.book, CompanyId(0)), usd(60_000.0));
    assert_eq!(by_type(&game, CostType::Investments), usd(-12_000.0));
    assert_eq!(books(&game).balance(Account::Participations), usd(60_000.0));
    assert_eq!(
        game.apply(Command::SellVentureStake {
            venture: 0,
            share: 0.5
        }),
        Err(CommandError::NotEnoughShares)
    );
    assert!(books(&game).is_balanced());
}

#[test]
fn steering_and_integrating_need_the_majority_and_no_blocking_minority() {
    let mut game = waiting_round();
    invest(&mut game, 120_000.0).unwrap();
    let steer = Command::SteerVenture {
        venture: 0,
        pace: VenturePace::Fast,
    };
    let integrate = Command::IntegrateVenture { venture: 0 };
    // Exactly half is no majority.
    assert_eq!(game.apply(steer.clone()), Err(CommandError::NoMajority));
    assert_eq!(game.apply(integrate.clone()), Err(CommandError::NoMajority));
    invest(&mut game, 28_800.0).unwrap();
    game.apply(steer).unwrap();
    assert_eq!(venture(&game, 0).pace, VenturePace::Fast);
    // Another company with a quarter blocks the integration.
    let rival = CompanyId(7);
    let founders = |game: &mut Game| {
        let v = game
            .state_mut()
            .ventures
            .iter_mut()
            .find(|v| v.id == 0)
            .unwrap();
        v.owners.retain(|s| s.holder != Holder::Private);
        v.owners.push(Stake {
            holder: Holder::Company(rival),
            share: 0.4,
        });
    };
    founders(&mut game);
    assert_eq!(
        game.apply(integrate.clone()),
        Err(CommandError::VentureBlocked)
    );
    {
        let v = game
            .state_mut()
            .ventures
            .iter_mut()
            .find(|v| v.id == 0)
            .unwrap();
        v.owners.retain(|s| s.holder != Holder::Company(rival));
        v.owners.push(Stake {
            holder: Holder::Private,
            share: 0.4,
        });
    }
    // The others' 40 % at the value after the round plus 20 %.
    let cash = books(&game).cash();
    game.apply(integrate).unwrap();
    let v = venture(&game, 0).clone();
    assert_eq!(v.parent, Some(CompanyId(0)));
    assert_eq!(v.owners, Stake::sole(Holder::Company(CompanyId(0))));
    assert_eq!(books(&game).cash(), cash - usd(240_000.0 * 1.2 * 0.4));
    assert!(books(&game).is_balanced());
}

#[test]
fn a_subsidiary_hands_its_technology_to_the_parent() {
    let mut game = waiting_round();
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    invest(&mut game, 120_000.0).unwrap();
    invest(&mut game, 28_800.0).unwrap();
    game.apply(Command::IntegrateVenture { venture: 0 })
        .unwrap();
    // The parent funds the next round at once and gets the technology at the end.
    let keys = until(&mut game, date(1900, 12, 2));
    let v = venture(&game, 0).clone();
    let VentureStatus::Succeeded(when) = v.status else {
        panic!("{:?}", v.status);
    };
    assert_eq!(v.exit, Some(VentureExit::Parent(CompanyId(0))));
    assert!(game.state().companies[0].technologies.contains(&turbine));
    assert_eq!(*game.state().inventions.get(turbine), Some(when));
    assert!(keys.iter().any(|k| k == keys::VENTURE_PARENT));
    // What it paid became research.
    assert_eq!(books(&game).balance(Account::Participations), Money::ZERO);
    assert!(by_type(&game, CostType::Research) < Money::ZERO);
    assert!(books(&game).is_balanced());
}

/// A start-up whose rounds investors fund at once; the company pledges the first round,
/// buys to 60 % and pledges the whole second round (73 %).
fn majority_game() -> Game {
    let mut game = game_with(catalog_with(model()), 3, 1.0);
    until(&mut game, date(1900, 2, 2));
    invest(&mut game, 120_000.0).unwrap();
    invest(&mut game, 28_800.0).unwrap();
    until(&mut game, date(1900, 5, 2));
    let v = venture(&game, 0).clone();
    assert_eq!(v.phase, 1);
    invest(&mut game, (v.capital - v.raised).to_usd()).unwrap();
    let v = venture(&game, 0).clone();
    assert!((ventures::share_of(&v, CompanyId(0)) - (0.4 + 1.0 / 3.0)).abs() < 1e-9);
    game
}

#[test]
fn a_majority_buys_out_the_others_when_it_can_pay() {
    let mut game = majority_game();
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    let cash = books(&game).cash();
    let keys = until(&mut game, date(1900, 9, 2));
    let v = venture(&game, 0).clone();
    assert_eq!(v.status, VentureStatus::Succeeded(date(1900, 9, 1)));
    assert_eq!(v.exit, Some(VentureExit::Parent(CompanyId(0))));
    assert!(game.state().companies[0].technologies.contains(&turbine));
    // The founders' 27 % at the value of the success: 240 000 · 3 · 1.5.
    let paid = usd(1_080_000.0).scale(1.0 - (0.4 + 1.0 / 3.0));
    let spent = cash - books(&game).cash();
    assert!(
        (spent - paid).abs() <= usd(1.0),
        "{spent:?} against {paid:?}"
    );
    assert!(keys.iter().any(|k| k == keys::VENTURE_PAID_OUT));
    assert_eq!(books(&game).balance(Account::Participations), Money::ZERO);
    assert!(books(&game).is_balanced());
}

#[test]
fn without_the_cash_the_start_up_goes_public() {
    let mut game = majority_game();
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    // The cash is tied up elsewhere.
    {
        let ledger = &mut game.state_mut().companies[0].ledger;
        let cash = ledger.cash();
        ledger.transfer(Account::FixedAssets, Account::Cash, cash);
    }
    let book = ventures::amount_of(&venture(&game, 0).book, CompanyId(0));
    let keys = until(&mut game, date(1900, 9, 2));
    let v = venture(&game, 0).clone();
    // No AI companies in this game: no new rival.
    assert_eq!(v.exit, Some(VentureExit::Listed(None)));
    assert!(!game.state().companies[0].technologies.contains(&turbine));
    // The company gets the value of its 73 %; the gain is the investments' result.
    let proceeds = usd(1_080_000.0).scale(0.4 + 1.0 / 3.0);
    assert!((books(&game).cash() - proceeds).abs() <= usd(1.0));
    let gain = by_type(&game, CostType::Investments);
    assert!((gain - (proceeds - book)).abs() <= usd(1.0), "{gain:?}");
    assert!(keys.iter().any(|k| k == keys::VENTURE_LISTED));
    assert_eq!(books(&game).balance(Account::Participations), Money::ZERO);
    assert!(books(&game).is_balanced());
}

#[test]
fn a_failure_writes_off_and_leaves_the_majority_some_research() {
    let mut m = model();
    m.phases[0].chance = 0.0;
    let mut game = game_with(catalog_with(m), 3, 1.0);
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    until(&mut game, date(1900, 2, 2));
    invest(&mut game, 120_000.0).unwrap();
    invest(&mut game, 28_800.0).unwrap();
    let keys = until(&mut game, date(1900, 5, 2));
    assert!(matches!(
        venture(&game, 0).status,
        VentureStatus::Failed(_, VentureFailure::Phase)
    ));
    assert_eq!(by_type(&game, CostType::Investments), usd(-148_800.0));
    assert_eq!(books(&game).balance(Account::Participations), Money::ZERO);
    assert!(keys.iter().any(|k| k == keys::VENTURE_LOST));
    // A fifth of the research it would take today.
    let points = game.state().companies[0]
        .research
        .get(&turbine)
        .copied()
        .unwrap_or(0.0);
    let effort = research::effort(game.catalog(), game.state(), turbine, game.state().date)
        .unwrap()
        .points;
    assert!(
        points > 0.0 && points <= 0.2 * effort * 1.01,
        "{points} of {effort}"
    );
    assert!(keys.iter().any(|k| k == keys::VENTURE_BONUS));
    assert!(books(&game).is_balanced());
}

#[test]
fn an_unfunded_round_returns_the_pledges() {
    let mut m = model();
    m.investor_chance = 0.0;
    m.deadline_months = 2;
    let mut game = game_with(catalog_with(m), 3, 1.0);
    until(&mut game, date(1900, 2, 2));
    let cash = books(&game).cash();
    invest(&mut game, 50_000.0).unwrap();
    let keys = until(&mut game, date(1900, 4, 2));
    assert!(matches!(
        venture(&game, 0).status,
        VentureStatus::Failed(_, VentureFailure::Funding)
    ));
    assert!(keys.iter().any(|k| k == keys::VENTURE_REFUND));
    assert_eq!(books(&game).balance(Account::Participations), Money::ZERO);
    assert_eq!(by_type(&game, CostType::Investments), Money::ZERO);
    // The workshop ran meanwhile; the pledge came back in full.
    assert!(books(&game).cash() > cash - usd(50_000.0));
    assert!(books(&game).is_balanced());
}

#[test]
fn a_subsidiary_without_a_parent_paying_is_diluted() {
    // The parent cannot fund the next round: investors come in and the subsidiary ends.
    let mut game = waiting_round();
    invest(&mut game, 120_000.0).unwrap();
    invest(&mut game, 28_800.0).unwrap();
    game.apply(Command::IntegrateVenture { venture: 0 })
        .unwrap();
    {
        let ledger = &mut game.state_mut().companies[0].ledger;
        let cash = ledger.cash();
        ledger.transfer(Account::FixedAssets, Account::Cash, cash);
    }
    until(&mut game, date(1900, 6, 2));
    let v = venture(&game, 0).clone();
    // Still waiting: no investors in this catalog and no cash at the parent.
    assert_eq!(v.parent, Some(CompanyId(0)));
    assert!(v.round_until.is_some());
}

#[test]
fn the_strategy_department_recommends_promising_rounds() {
    let mut c = test_support::management();
    let mut m = model();
    m.investor_chance = 0.0;
    m.deadline_months = 12;
    m.phases[0].chance = 0.9;
    m.phases[1].chance = 0.9;
    c.ventures = m;
    let mut game = crate::management_tests::new_game(c);
    mine_and_works(&mut game);
    // Without a strategy department nothing is recommended.
    until(&mut game, date(1900, 4, 2));
    assert!(
        !game
            .state()
            .concerns
            .iter()
            .any(|c| c.decision.topic == crate::decision::Topic::Venture)
    );
    game.apply(Command::StaffDepartment {
        department: DepartmentKind::Strategy,
        staff: 1,
    })
    .unwrap();
    hire_sharp(&mut game, member("strategie"));
    // Beyond its release limit of 1 000 USD the department asks.
    game.apply(Command::SetParticipations {
        budget: None,
        risk: 1.0,
        limits: [(DepartmentKind::Strategy, usd(1_000.0))]
            .into_iter()
            .collect(),
    })
    .unwrap();
    // Seen exactly: 0.81 chance, worth 1 080 000 at the end, a dollar buys 1/360 000 of
    // it – about 2.4 times the money back.
    let v = venture(&game, 0).clone();
    let e = ventures::expected_return(game.catalog(), game.state(), game.player(), &v);
    assert!((e - 0.81 * 1_080_000.0 / 360_000.0).abs() < 1e-6, "{e}");
    until(&mut game, date(1900, 7, 2));
    let concern = game
        .state()
        .concerns
        .iter()
        .find(|c| {
            c.decision.topic == crate::decision::Topic::Venture
                && c.status == crate::state::ConcernStatus::Open
        })
        .cloned()
        .expect("a recommendation");
    let shown = crate::views::concerns(&game)
        .open
        .iter()
        .flat_map(|g| &g.concerns)
        .find(|c| c.id == concern.id)
        .unwrap()
        .clone();
    assert_eq!(shown.options[0].kind, "beteiligen");
    assert_eq!(shown.options[0].steps[0].key, keys::STEP_INVEST);
    assert_eq!(shown.because.key, keys::BECAUSE_VENTURE);
    game.apply(Command::AnswerConcern {
        concern: concern.id,
        answer: crate::management::ConcernAnswer::Delegate,
    })
    .unwrap();
    // The whole open round: the round closes and the company holds half.
    let v = venture(&game, 0).clone();
    assert!((ventures::share_of(&v, game.player()) - 0.5).abs() < 1e-9);
}

// --- SU3: spin-offs and AI companies ---

use crate::catalog::{FacilitySize, SiteType};
use crate::decision::Rules;
use crate::state::{AiState, CompanyKind, SiteId};

/// A research center of the player working on the turbine (1902) with `progress` of its
/// effort done.
fn turbine_lab(game: &mut Game, progress: f64) -> SiteId {
    let c = game.catalog().clone();
    let turbine = c.technologies.id("turbine").unwrap();
    game.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: SiteType::ResearchCenter,
    })
    .unwrap();
    let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.apply(Command::BuildFacility {
        site,
        facility: c.facilities.id("labor").unwrap(),
        count: 1,
        size: FacilitySize::Medium,
    })
    .unwrap();
    game.apply(Command::SetResearch {
        site,
        technology: Some(turbine),
    })
    .unwrap();
    let date = game.state().date;
    let effort = research::effort(&c, game.state(), turbine, date)
        .unwrap()
        .points;
    game.state_mut().companies[0]
        .research
        .insert(turbine, effort * progress);
    site
}

/// An AI company with `cash` that never hesitates.
fn ai_rival(game: &mut Game, cash: f64) -> CompanyId {
    let mut rival = game.state().companies[0].clone();
    let date = game.state().date;
    let id = CompanyId(u32::try_from(game.state().companies.len()).unwrap());
    rival.name = "Rivale AG".into();
    rival.kind = CompanyKind::Ai;
    rival.ledger = crate::ledger::Ledger::new(date, usd(cash));
    rival.research = Default::default();
    rival.departments = Default::default();
    rival.ai = Some(AiState {
        competence: 1.0,
        aggressiveness: 1.0,
        real: None,
        next_operations: date,
        staff: 0.0,
    });
    game.state_mut().companies.push(rival);
    id
}

fn ai_month(game: &mut Game, company: CompanyId, own: &[SiteId]) -> Vec<String> {
    let catalog = game.catalog().clone();
    let date = game.state().date;
    ventures::ai_month(game.state_mut(), &catalog, company, own, date, &mut Rules)
        .into_iter()
        .map(|m| m.key)
        .collect()
}

#[test]
fn a_spin_off_turns_research_into_a_subsidiary() {
    let mut game = game_with(catalog_with(model()), 3, 0.0);
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    let lab = turbine_lab(&mut game, 0.6);
    let cash = books(&game).cash();
    game.apply(Command::SpinOff {
        site: lab,
        sell: 0.3,
    })
    .unwrap();
    let v = game.state().ventures.last().unwrap().clone();
    assert_eq!(v.target, VentureTarget::Technology(turbine));
    assert_eq!(v.origin, Some(CompanyId(0)));
    // 60 % of the research: the second of two phases, its round open.
    assert_eq!(v.phase, 1);
    assert!(v.round_until.is_some());
    assert!((ventures::share_of(&v, CompanyId(0)) - 0.7).abs() < 1e-9);
    assert_eq!(v.parent, Some(CompanyId(0)));
    // The project left the company and the laboratory.
    assert!(!game.state().companies[0].research.contains_key(&turbine));
    assert_eq!(game.state().sites[lab.index()].research, None);
    // 30 % sold at the value before the round with the discount, all of it a gain.
    let proceeds = ventures::value(&game.catalog().ventures, &v).scale(0.3 * 0.8);
    assert!(proceeds > Money::ZERO);
    assert!((books(&game).cash() - cash - proceeds).abs() <= usd(1.0));
    assert!((by_type(&game, CostType::Investments) - proceeds).abs() <= usd(1.0));
    assert_eq!(books(&game).balance(Account::Participations), Money::ZERO);
    assert!(books(&game).is_balanced());
}

#[test]
fn a_spin_off_needs_an_open_target_and_progress() {
    let mut game = game_with(catalog_with(model()), 3, 0.0);
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    let lab = turbine_lab(&mut game, 0.05);
    assert!(matches!(
        game.apply(Command::SpinOff {
            site: lab,
            sell: 0.0
        }),
        Err(CommandError::SpinOffTooEarly { .. })
    ));
    let points = game.state().companies[0].research[&turbine] * 4.0;
    game.state_mut().companies[0]
        .research
        .insert(turbine, points);
    assert_eq!(
        game.apply(Command::SpinOff {
            site: lab,
            sell: 1.0
        }),
        Err(CommandError::InvalidShare)
    );
    // Not a research center, and an invented technology.
    let c = game.catalog().clone();
    game.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: SiteType::Factory,
    })
    .unwrap();
    let works = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    assert_eq!(
        game.apply(Command::SpinOff {
            site: works,
            sell: 0.0
        }),
        Err(CommandError::NoSpinOff)
    );
    *game.state_mut().inventions.get_mut(turbine) = Some(date(1900, 1, 1));
    assert_eq!(
        game.apply(Command::SpinOff {
            site: lab,
            sell: 0.0
        }),
        Err(CommandError::NoSpinOff)
    );
    assert!(game.state().ventures.is_empty());
}

#[test]
fn a_subsidiary_pays_its_share_of_each_round() {
    let mut game = game_with(catalog_with(model()), 3, 0.0);
    let lab = turbine_lab(&mut game, 0.2);
    game.apply(Command::SpinOff {
        site: lab,
        sell: 0.4,
    })
    .unwrap();
    let v = game.state().ventures.last().unwrap().clone();
    assert_eq!(v.phase, 0);
    let cash = books(&game).cash();
    // At the month start the parent pledges 60 % of the round, investors the rest.
    until(&mut game, date(1900, 2, 2));
    let v = venture(&game, v.id).clone();
    assert!(v.round_until.is_none());
    assert!((ventures::share_of(&v, CompanyId(0)) - 0.6).abs() < 1e-9);
    assert_eq!(v.parent, Some(CompanyId(0)));
    // The laboratory ran meanwhile: its pledge is what went into the financial assets.
    let pledged = books(&game).balance(Account::Participations);
    let expected = v.capital.scale(0.6);
    assert!(
        (pledged - expected).abs() <= usd(1.0),
        "{pledged:?} against {expected:?}"
    );
    assert!(cash - books(&game).cash() >= pledged);
    assert!(books(&game).is_balanced());
}

#[test]
fn ai_companies_pledge_to_promising_rounds() {
    let mut game = waiting_round();
    let rival = ai_rival(&mut game, 1_000_000.0);
    let open = venture(&game, 0).capital - venture(&game, 0).raised;
    ai_month(&mut game, rival, &[]);
    let v = venture(&game, 0).clone();
    // Its pledge covered the round: the phase began, the rival holds its part.
    assert!(v.round_until.is_none());
    assert!(ventures::share_of(&v, rival) > 0.0);
    let ledger = &game.state().companies[rival.index()].ledger;
    assert_eq!(ledger.cash(), usd(1_000_000.0) - open);
    assert_eq!(ledger.balance(Account::Participations), open);
    assert!(ledger.is_balanced());
    // Within its share of the cash: a poorer rival pledges nothing.
    let mut game = waiting_round();
    let poor = ai_rival(&mut game, 50_000.0);
    ai_month(&mut game, poor, &[]);
    assert_eq!(ventures::share_of(venture(&game, 0), poor), 0.0);
}

/// A works of an AI rival that smelts iron, and a recipe for iron that needs the turbine:
/// the turbine start-up serves the rival. Investors fund the first round in February.
fn takeover_game() -> (Game, CompanyId, SiteId) {
    let mut c = catalog_with(model());
    let turbine = c.technologies.id("turbine").unwrap();
    let mut recipe = c
        .recipes
        .get(c.recipes.id("eisen_schmelzen").unwrap())
        .clone();
    recipe.technology = Some(turbine);
    c.recipes.insert("eisen_turbine", recipe).unwrap();
    let mut game = game_with(c, 3, 1.0);
    let c = game.catalog().clone();
    // Enough money for a furnace.
    let start = game.state().date;
    game.state_mut().companies[0].ledger = crate::ledger::Ledger::new(start, usd(10_000_000.0));
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
        size: FacilitySize::Medium,
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site: works,
        slot: 0,
        recipe: c.recipes.id("eisen_schmelzen"),
        utilization: 1.0,
    })
    .unwrap();
    until(&mut game, date(1900, 3, 2));
    let v = venture(&game, 0);
    assert!(v.round_until.is_none(), "{v:?}");
    // The rival arrives now, so that it has not acted yet.
    let rival = ai_rival(&mut game, 2_000_000.0);
    game.state_mut().sites[works.index()].owner = rival;
    (game, rival, works)
}

#[test]
fn ai_companies_take_over_in_the_course_of_the_game() {
    let mut c = catalog_with(model());
    let turbine = c.technologies.id("turbine").unwrap();
    let mut recipe = c
        .recipes
        .get(c.recipes.id("eisen_schmelzen").unwrap())
        .clone();
    recipe.technology = Some(turbine);
    c.recipes.insert("eisen_turbine", recipe).unwrap();
    let mut game = game_with(c, 3, 1.0);
    let c = game.catalog().clone();
    let start = game.state().date;
    game.state_mut().companies[0].ledger = crate::ledger::Ledger::new(start, usd(10_000_000.0));
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
        size: FacilitySize::Medium,
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site: works,
        slot: 0,
        recipe: c.recipes.id("eisen_schmelzen"),
        utilization: 1.0,
    })
    .unwrap();
    let rival = ai_rival(&mut game, 2_000_000.0);
    game.state_mut().sites[works.index()].owner = rival;
    // The rival's own rules at the month starts: a pledge, then the takeover.
    until(&mut game, date(1900, 4, 2));
    let v = venture(&game, 0);
    assert_eq!(v.parent, Some(rival), "{v:?}");
    assert!(game.state().companies[rival.index()].ledger.is_balanced());
}

#[test]
fn ai_companies_take_over_start_ups_they_can_use() {
    let (mut game, rival, works) = takeover_game();
    // The rival holds 15 % from a pledge, the player 10 %: no blocking minority.
    share_out(&mut game, &[(rival, 0.15), (CompanyId(0), 0.1)]);
    let cash = books(&game).cash();
    let worth = ventures::value(&game.catalog().ventures, venture(&game, 0)).scale(1.2);
    let keys = ai_month(&mut game, rival, &[works]);
    let v = venture(&game, 0).clone();
    assert_eq!(v.parent, Some(rival));
    assert_eq!(v.owners, Stake::sole(Holder::Company(rival)));
    // The player got its 10 % at the value with the premium.
    let got = books(&game).cash() - cash;
    assert!((got - worth.scale(0.1)).abs() <= usd(1.0), "{got:?}");
    assert!(keys.iter().any(|k| k == keys::VENTURE_BOUGHT_OUT));
    assert!(books(&game).is_balanced());
    assert!(game.state().companies[rival.index()].ledger.is_balanced());
}

#[test]
fn a_blocking_minority_keeps_the_ai_away() {
    let (mut game, rival, works) = takeover_game();
    share_out(&mut game, &[(rival, 0.15), (CompanyId(0), 0.25)]);
    let keys = ai_month(&mut game, rival, &[works]);
    let v = venture(&game, 0);
    assert_eq!(v.parent, None);
    assert!((ventures::share_of(v, rival) - 0.15).abs() < 1e-9);
    assert!(keys.is_empty());
}

#[test]
fn ai_companies_buy_away_only_what_they_hold_a_share_of() {
    let (mut game, rival, works) = takeover_game();
    let keys = ai_month(&mut game, rival, &[works]);
    let v = venture(&game, 0);
    assert_eq!(v.parent, None);
    assert_eq!(ventures::share_of(v, rival), 0.0);
    assert!(keys.is_empty());
}

/// Gives companies shares of the turbine start-up; the other owners keep the rest in
/// proportion.
fn share_out(game: &mut Game, stakes: &[(CompanyId, f64)]) {
    let v = game
        .state_mut()
        .ventures
        .iter_mut()
        .find(|v| v.id == 0)
        .unwrap();
    let given: f64 = stakes.iter().map(|s| s.1).sum();
    for s in &mut v.owners {
        s.share *= 1.0 - given;
    }
    for &(c, share) in stakes {
        v.owners.push(Stake {
            holder: Holder::Company(c),
            share,
        });
    }
}

#[test]
fn ai_companies_spin_off_research_at_the_start_of_a_year() {
    let mut game = game_with(catalog_with(model()), 3, 1.0);
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    let lab = turbine_lab(&mut game, 0.5);
    let rival = ai_rival(&mut game, 1_000_000.0);
    // The laboratory and its research go to the rival.
    game.state_mut().sites[lab.index()].owner = rival;
    let points = game.state_mut().companies[0]
        .research
        .remove(&turbine)
        .unwrap();
    game.state_mut().companies[rival.index()]
        .research
        .insert(turbine, points);
    assert_eq!(game.state().date.ordinal(), 1);
    let keys = ai_month(&mut game, rival, &[lab]);
    assert!(keys.iter().any(|k| k == keys::VENTURE_SPIN_OFF), "{keys:?}");
    let v = game.state().ventures.last().unwrap();
    assert_eq!(v.origin, Some(rival));
    assert_eq!(v.parent, Some(rival));
    assert!((ventures::share_of(v, rival) - 0.6).abs() < 1e-9);
    assert!(game.state().companies[rival.index()].ledger.is_balanced());
}

#[test]
fn the_view_lists_the_projects_to_spin_off() {
    let mut game = game_with(catalog_with(model()), 3, 0.0);
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    let lab = turbine_lab(&mut game, 0.05);
    let project = |game: &Game| {
        crate::views::ventures(game)
            .spin_offs
            .into_iter()
            .find(|p| p.site == lab.0)
    };
    let p = project(&game).unwrap();
    assert_eq!(p.kind, "technologie");
    assert_eq!(p.target, "turbine");
    assert_eq!(p.reason.as_deref(), Some("fortschritt"));
    assert!((p.progress - 0.05).abs() < 1e-9);
    let points = game.state().companies[0].research[&turbine] * 10.0;
    game.state_mut().companies[0]
        .research
        .insert(turbine, points);
    let p = project(&game).unwrap();
    assert_eq!(p.reason, None);
    assert_eq!(p.phase.as_deref(), Some("prototyp"));
    let (value, sale) = (p.value_usd.unwrap(), p.sale_value_usd.unwrap());
    assert!(value > 0.0 && (sale - value * 0.8).abs() < 1e-6);
    // Spun off: the project is gone, the start-up shows where it came from.
    game.apply(Command::SpinOff {
        site: lab,
        sell: 0.0,
    })
    .unwrap();
    assert!(project(&game).is_none());
    let view = crate::views::ventures(&game);
    let v = &view.active[0];
    assert_eq!(v.origin.as_deref(), Some("Labor AG"));
    assert!(v.subsidiary);
    assert!((v.value_usd - value).abs() < 1e-6);
}

#[test]
fn only_projects_ahead_of_history_are_worth_spinning_off() {
    let m = model();
    let c = catalog_with(model());
    let turbine = c.technologies.id("turbine").unwrap();
    let country = c.countries.id("AAA").unwrap();
    let plan = |lead: f64, phase: usize| ventures::SpinOffPlan {
        target: VentureTarget::Technology(turbine),
        progress: 0.5,
        phase,
        lead,
        country,
    };
    // Phases of two and three months: five from the first, three from the second.
    assert_eq!(ventures::months_from(&m, 0), 5);
    assert_eq!(ventures::months_from(&m, 1), 3);
    assert!(ventures::ahead_of_history(&m, &plan(0.5, 0)));
    assert!(!ventures::ahead_of_history(&m, &plan(0.25, 0)));
    assert!(ventures::ahead_of_history(&m, &plan(0.25, 1)));
}

#[test]
fn ai_companies_keep_contested_projects_in_house() {
    let mut game = game_with(catalog_with(model()), 3, 1.0);
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    let lab = turbine_lab(&mut game, 0.5);
    // The player researches the turbine in a second laboratory.
    let own_lab = turbine_lab(&mut game, 0.5);
    let rival = ai_rival(&mut game, 1_000_000.0);
    game.state_mut().sites[lab.index()].owner = rival;
    let points = game.state().companies[0].research[&turbine];
    game.state_mut().companies[rival.index()]
        .research
        .insert(turbine, points);
    assert_eq!(
        ventures::rivals(game.state(), rival, VentureTarget::Technology(turbine)),
        1
    );
    let keys = ai_month(&mut game, rival, &[lab]);
    assert!(
        !keys.iter().any(|k| k == keys::VENTURE_SPIN_OFF),
        "{keys:?}"
    );
    assert!(
        game.state()
            .ventures
            .iter()
            .all(|v| v.origin != Some(rival))
    );
    // The player sees the rival at the same target.
    let view = crate::views::ventures(&game);
    let p = view.spin_offs.iter().find(|p| p.site == own_lab.0).unwrap();
    assert_eq!(p.rivals, 1);
}

fn offer(game: &mut Game, minimum: Option<f64>) -> Result<(), CommandError> {
    game.apply(Command::OfferVentureStake {
        venture: 0,
        minimum: minimum.map(usd),
    })
}

#[test]
fn companies_bid_for_a_stake_offered_to_them() {
    let mut game = waiting_round();
    invest(&mut game, 120_000.0).unwrap();
    let me = CompanyId(0);
    let rival = ai_rival(&mut game, 10_000_000.0);
    let catalog = game.catalog().clone();
    // Worth 240 000 after the round: half is 120 000 (investors would pay 96 000); the
    // rival bids 20 % more, well below what it expects (sure phases).
    let v = venture(&game, 0).clone();
    assert_eq!(
        ventures::stake_bid(&catalog, game.state(), rival, &v, 0.5),
        Some(usd(144_000.0))
    );
    assert_eq!(
        ventures::stake_bid(&catalog, game.state(), me, &v, 0.5),
        None
    );
    assert_eq!(offer(&mut game, Some(0.0)), Err(CommandError::InvalidPrice));
    // Too high a minimum: the player learns the best bid, the stake stays.
    offer(&mut game, Some(150_000.0)).unwrap();
    assert_eq!(venture(&game, 0).sales, vec![(me, usd(150_000.0))]);
    let keys = until(&mut game, date(1900, 3, 2));
    assert!(
        keys.iter().any(|k| k == keys::VENTURE_STAKE_BEST_BID),
        "{keys:?}"
    );
    assert!((ventures::share_of(venture(&game, 0), me) - 0.5).abs() < 1e-12);
    assert!(venture(&game, 0).sales.is_empty(), "the offer ended");
    // Withdrawn before the month start, nothing happens.
    offer(&mut game, Some(100_000.0)).unwrap();
    offer(&mut game, None).unwrap();
    assert!(venture(&game, 0).sales.is_empty());
    // From a lower minimum the best bid buys at its price.
    offer(&mut game, Some(100_000.0)).unwrap();
    let keys = until(&mut game, date(1900, 4, 1));
    assert!(
        keys.iter().any(|k| k == keys::VENTURE_STAKE_SOLD),
        "{keys:?}"
    );
    let v = venture(&game, 0).clone();
    assert_eq!(ventures::share_of(&v, me), 0.0);
    assert!((ventures::share_of(&v, rival) - 0.5).abs() < 1e-12);
    assert_eq!(ventures::amount_of(&v.book, rival), usd(144_000.0));
    assert_eq!(ventures::amount_of(&v.book, me), Money::ZERO);
    // The player's book of 120 000 leaves; 24 000 are a gain.
    assert_eq!(books(&game).balance(Account::Participations), Money::ZERO);
    assert_eq!(by_type(&game, CostType::Investments), usd(24_000.0));
    // The rival's financial assets hold the purchase besides its pledges elsewhere.
    let r = &game.state().companies[rival.index()];
    let held: Money = game
        .state()
        .ventures
        .iter()
        .map(|v| ventures::amount_of(&v.book, rival))
        .sum();
    assert_eq!(r.ledger.balance(Account::Participations), held);
    assert!(r.participations.spent_in(1900) >= usd(144_000.0));
    assert!(books(&game).is_balanced() && r.ledger.is_balanced());
    // Nothing more to offer.
    assert_eq!(
        offer(&mut game, Some(1.0)),
        Err(CommandError::NotEnoughShares)
    );
}

#[test]
fn a_blocking_minority_keeps_a_competitor_from_the_majority() {
    let mut game = waiting_round();
    invest(&mut game, 120_000.0).unwrap();
    let me = CompanyId(0);
    let rival = ai_rival(&mut game, 10_000_000.0);
    let third = ai_rival(&mut game, 10_000_000.0);
    // The player offers 30 %; the rival holds 25 %, the third company 30 %.
    let v = game
        .state_mut()
        .ventures
        .iter_mut()
        .find(|v| v.id == 0)
        .unwrap();
    let stake = |holder, share| Stake { holder, share };
    v.owners = vec![
        stake(Holder::Private, 0.15),
        stake(Holder::Company(me), 0.3),
        stake(Holder::Company(rival), 0.25),
        stake(Holder::Company(third), 0.3),
    ];
    let catalog = game.catalog().clone();
    let v = venture(&game, 0).clone();
    let mine = ventures::share_of(&v, me);
    // The rival would get over half while the third company holds a blocking minority;
    // the third company could buy (the rival's 25 % block only who gets the majority).
    assert!(ventures::sale_blocked(
        &catalog.ventures,
        &v,
        (rival, me),
        mine
    ));
    assert!(ventures::sale_blocked(
        &catalog.ventures,
        &v,
        (third, me),
        mine
    ));
    let other = ai_rival(&mut game, 10_000_000.0);
    let v = venture(&game, 0).clone();
    assert!(!ventures::sale_blocked(
        &catalog.ventures,
        &v,
        (other, me),
        mine
    ));
    let best = ventures::best_stake_bid(&catalog, game.state(), &v, me).unwrap();
    assert_eq!(best.0, other);
    // A command of a blocked buyer fails like the player's would.
    offer(&mut game, Some(1.0)).unwrap();
    assert_eq!(
        crate::command::execute(
            game.state_mut(),
            &catalog,
            rival,
            &Command::BuyVentureStake {
                venture: 0,
                seller: me,
                price: usd(1_000_000.0),
            },
        ),
        Err(CommandError::VentureBlocked)
    );
    assert_eq!(
        crate::command::execute(
            game.state_mut(),
            &catalog,
            other,
            &Command::BuyVentureStake {
                venture: 0,
                seller: rival,
                price: usd(1_000_000.0),
            },
        ),
        Err(CommandError::NoStakeOffer)
    );
    assert_eq!(
        crate::command::execute(
            game.state_mut(),
            &catalog,
            other,
            &Command::BuyVentureStake {
                venture: 0,
                seller: me,
                price: Money::ZERO,
            },
        ),
        Err(CommandError::BelowMinimumBid { minimum: usd(1.0) })
    );
}

#[test]
fn a_recommended_pledge_counts_by_how_the_start_up_turns_out() {
    let mut game = waiting_round();
    let catalog = game.catalog().clone();
    let v = venture(&game, 0).clone();
    let amount = v.capital - v.raised;
    let share = amount.to_usd() * ventures::share_per_dollar(&catalog.ventures, &v);
    let judgment = crate::state::Judgment {
        manager: crate::state::ManagerId(0),
        due: game.state().date,
        hit: false,
        appraisal: Some(crate::state::Appraisal::Venture {
            venture: 0,
            amount,
            share,
        }),
    };
    let hit = |game: &Game| crate::central::judged_hit(&catalog, game.state(), &judgment);
    // Sure phases: the share is worth more than the pledge.
    assert!(hit(&game));
    // Hardly a chance any more: not worth it.
    game.state_mut().ventures[0].chance = 0.01;
    assert!(!hit(&game));
    let set = |game: &mut Game, status| game.state_mut().ventures[0].status = status;
    set(&mut game, VentureStatus::Succeeded(date(1900, 5, 1)));
    assert!(hit(&game));
    set(
        &mut game,
        VentureStatus::Failed(date(1900, 5, 1), VentureFailure::Phase),
    );
    assert!(!hit(&game));
}
