//! Scenario tests for the headquarters and the central departments (ZA1, ZA2) with
//! `test_support::management`.

use crate::board_tests::{buyer_game, buyer_game_with, member, news};
use crate::calendar::{Date, RoundLength};
use crate::catalog::{DepartmentKind, test_support};
use crate::central;
use crate::command::{Command, CommandError};
use crate::deals::DealObject;
use crate::decision::Topic;
use crate::finance;
use crate::game::Game;
use crate::ledger::CostType;
use crate::management;
use crate::management_tests::{days, hire_sharp, mine_and_works, new_game, usd};
use crate::message::{Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, ConcernReason, Unit};

#[test]
fn moving_the_headquarters_costs_and_takes_months() {
    let mut c = test_support::management();
    c.central.headquarters.months = 2;
    c.central.headquarters.cost_base = usd(250_000.0);
    let mut game = new_game(c);
    mine_and_works(&mut game);
    let catalog = game.catalog().clone();
    let aaa = catalog.countries.id("AAA").unwrap();
    let bbb = catalog.countries.id("BBB").unwrap();
    let player = CompanyId(0);
    assert_eq!(game.state().companies[0].headquarters, aaa);
    assert_eq!(
        game.apply(Command::SetHeadquarters { country: aaa }),
        Err(CommandError::SameHeadquarters)
    );
    let cash = game.state().companies[0].ledger.cash();
    game.apply(Command::SetHeadquarters { country: bbb })
        .unwrap();
    let state = game.state();
    assert_eq!(state.companies[0].ledger.cash(), cash - usd(250_000.0));
    let other = state.companies[0]
        .ledger
        .month
        .by_type
        .get(&CostType::Other)
        .copied()
        .unwrap_or(Money::ZERO);
    assert!(other <= -usd(250_000.0), "booked as other costs");
    assert!(state.companies[0].ledger.is_balanced());
    let until = state.companies[0].relocation.unwrap().until;
    assert_eq!(until, Date::new(1900, 3, 1).unwrap());
    // Only one move at a time.
    assert_eq!(
        game.apply(Command::SetHeadquarters { country: aaa }),
        Err(CommandError::RelocationUnderWay { until })
    );
    // Still the old seat until the move is done.
    let mut keys_seen = Vec::new();
    while game.state().date < until {
        assert_eq!(game.state().companies[0].headquarters, aaa);
        let report = game.advance(RoundLength::Day, |_| {});
        keys_seen.extend(report.messages.into_iter().map(|m| m.key));
    }
    let state = game.state();
    assert_eq!(state.companies[0].headquarters, bbb);
    assert!(state.companies[0].relocation.is_none());
    assert!(keys_seen.iter().any(|k| k == keys::HEADQUARTERS_MOVED));
    // The board's salaries follow the new seat.
    assert_eq!(
        management::seat_country(&catalog, state, player, Unit::Board),
        Some(bbb)
    );
    days(&mut game, 1);
}

#[test]
fn a_move_needs_the_cash() {
    let mut c = test_support::management();
    c.central.headquarters.cost_base = usd(1e12);
    let mut game = new_game(c);
    let bbb = game.catalog().countries.id("BBB").unwrap();
    assert!(matches!(
        game.apply(Command::SetHeadquarters { country: bbb }),
        Err(CommandError::NotEnoughCash { .. })
    ));
    assert!(game.state().companies[0].relocation.is_none());
}

fn staff(game: &mut Game, department: DepartmentKind, n: u32) {
    game.apply(Command::StaffDepartment {
        department,
        staff: n,
    })
    .unwrap();
}

fn participations(budget: Option<Money>, risk: f64, limits: &[(DepartmentKind, Money)]) -> Command {
    Command::SetParticipations {
        budget,
        risk,
        limits: limits.iter().copied().collect(),
    }
}

#[test]
fn departments_cost_every_month() {
    let mut game = new_game(test_support::management());
    mine_and_works(&mut game);
    let catalog = game.catalog().clone();
    let player = game.player();
    staff(&mut game, DepartmentKind::Finance, 3);
    staff(&mut game, DepartmentKind::Marketing, 2);
    let state = game.state();
    assert_eq!(central::employees(state, player), 5);
    let h = &catalog.central.headquarters;
    assert_eq!(
        central::relocation_cost(&catalog, state, player),
        h.cost_base + h.cost_per_employee.scale(5.0)
    );
    let (personnel, office) = central::monthly_cost(&catalog, state, player);
    let d = catalog.central.department(DepartmentKind::Finance).unwrap();
    let hq = state.companies[0].headquarters;
    let wage = management::group_yearly_wage(&catalog, state, hq, d.labor_group);
    assert!(wage > 0.0);
    assert!(
        (personnel.to_usd() - 5.0 * wage / 12.0).abs() < 0.01,
        "{personnel:?}"
    );
    assert!(
        (office.to_usd() - 5.0 * 15_000.0 / 12.0).abs() < 0.01,
        "{office:?}"
    );
    // Booked at the end of the month as overhead of the headquarters.
    let mut later = state.clone();
    let cash = later.companies[0].ledger.cash();
    central::month_end(&mut later, &catalog);
    let ledger = &later.companies[0].ledger;
    assert_eq!(ledger.cash(), cash - personnel - office);
    assert!(ledger.is_balanced());
    // In the game, too.
    let before = game.state().companies[0].ledger.cash();
    days(&mut game, 31);
    let overhead = game.state().companies[0]
        .ledger
        .months
        .last()
        .unwrap()
        .by_type[&CostType::Overhead];
    assert!(-overhead >= office, "{overhead:?}");
    assert!(game.state().companies[0].ledger.cash() < before);
    // Without a head the departments do not work, the employees cost all the same.
    assert!(
        central::performance(&catalog, game.state(), player, DepartmentKind::Finance).is_none()
    );
    assert_eq!(
        central::strength(&catalog, game.state(), player, DepartmentKind::Finance),
        0.0
    );
    // None left: the department is gone.
    staff(&mut game, DepartmentKind::Marketing, 0);
    assert!(
        !game.state().companies[0]
            .departments
            .contains_key(&DepartmentKind::Marketing)
    );
    // Only departments of the data.
    let mut c = test_support::management();
    c.central
        .departments
        .retain(|d| d.kind != DepartmentKind::Legal);
    let mut game = new_game(c);
    assert_eq!(
        game.apply(Command::StaffDepartment {
            department: DepartmentKind::Legal,
            staff: 1
        }),
        Err(CommandError::UnknownDepartment)
    );
}

#[test]
fn a_department_works_with_its_head_and_enough_staff() {
    let mut game = new_game(test_support::management());
    mine_and_works(&mut game);
    let catalog = game.catalog().clone();
    let player = game.player();
    staff(&mut game, DepartmentKind::Finance, 1);
    let boss = hire_sharp(&mut game, member("finanzen"));
    game.state_mut()
        .managers
        .get_mut(&boss)
        .unwrap()
        .expertise
        .insert("finanzen".into(), 60);
    let p = central::performance(&catalog, game.state(), player, DepartmentKind::Finance).unwrap();
    assert_eq!(p.capacity, 4.0);
    assert!((p.quality - 0.6).abs() < 1e-12);
    assert_eq!(p.coverage, 1.0);
    assert!((p.strength - 0.4 * 0.6).abs() < 1e-12);
    // The finance department saves part of the risk premium of new loans.
    let state = game.state();
    let company = &state.companies[0];
    let amount = usd(10_000.0);
    let cut = central::premium_cut(&catalog, state, player);
    assert!((cut - p.strength).abs() < 1e-12);
    let plain = finance::loan_rate(&catalog, company, amount, state.date, 0.0);
    let cheaper = finance::loan_rate(&catalog, company, amount, state.date, cut);
    let base = finance::base_rate(&catalog, state.date);
    assert!(((cheaper - base) - (plain - base) * (1.0 - cut)).abs() < 1e-12);
    game.apply(Command::TakeLoan { amount, years: 5 }).unwrap();
    assert!((game.state().companies[0].loans[0].rate - cheaper).abs() < 1e-12);
    // More loans than it can handle: the coverage falls.
    for _ in 0..6 {
        game.apply(Command::TakeLoan { amount, years: 5 }).unwrap();
    }
    let p = central::performance(&catalog, game.state(), player, DepartmentKind::Finance).unwrap();
    assert!((p.coverage - 4.0 / 8.0).abs() < 1e-12, "{p:?}");
    // The head errs less in its function, by the accuracy times the coverage.
    let accuracy = central::accuracy(&catalog, game.state(), player, &member("finanzen"));
    assert!((accuracy - 0.5 * 0.5).abs() < 1e-12);
    assert_eq!(
        central::accuracy(&catalog, game.state(), player, &member("personal")),
        0.0
    );
    // A personnel department trains the managers.
    assert_eq!(
        central::experience_chance(&catalog, game.state(), player, 0.3),
        0.3
    );
    staff(&mut game, DepartmentKind::Personnel, 1);
    hire_sharp(&mut game, member("personal"));
    let trained = central::experience_chance(&catalog, game.state(), player, 0.3);
    let w = central::strength(&catalog, game.state(), player, DepartmentKind::Personnel);
    assert!(w > 0.0);
    assert!((trained - 0.3 * (1.0 + w)).abs() < 1e-12);
}

#[test]
fn a_marketing_department_strengthens_advertising() {
    let mut game = new_game(test_support::management());
    mine_and_works(&mut game);
    let catalog = game.catalog().clone();
    let aaa = catalog.countries.id("AAA").unwrap();
    let group = catalog
        .products
        .get(catalog.products.id("eisen").unwrap())
        .goods_group;
    game.apply(Command::SetAdvertising {
        country: aaa,
        group,
        budget: usd(5_000.0),
    })
    .unwrap();
    hire_sharp(&mut game, member("vertrieb_marketing"));
    let plain = game.state().clone();
    staff(&mut game, DepartmentKind::Marketing, 1);
    let w = central::strength(
        &catalog,
        game.state(),
        game.player(),
        DepartmentKind::Marketing,
    );
    assert!(w > 0.0);
    let mut boosted = game.state().clone();
    let mut plain = plain;
    let next = Date::new(1900, 2, 1).unwrap();
    crate::brand::month_start(&mut plain, &catalog, next);
    crate::brand::month_start(&mut boosted, &catalog, next);
    let a = plain.companies[0].awareness(aaa, group);
    let b = boosted.companies[0].awareness(aaa, group);
    assert!(a > 0.0 && b > a, "{a} {b}");
}

#[test]
fn a_move_keeps_part_of_the_staff() {
    let mut c = test_support::management();
    c.central.headquarters.months = 1;
    c.central.headquarters.moving_share = 0.6;
    let mut game = new_game(c);
    mine_and_works(&mut game);
    staff(&mut game, DepartmentKind::Finance, 5);
    staff(&mut game, DepartmentKind::Legal, 1);
    let bbb = game.catalog().countries.id("BBB").unwrap();
    game.apply(Command::SetHeadquarters { country: bbb })
        .unwrap();
    let mut seen = Vec::new();
    while game.state().companies[0].relocation.is_some() {
        let report = game.advance(RoundLength::Day, |_| {});
        seen.extend(report.messages);
    }
    let departments = &game.state().companies[0].departments;
    assert_eq!(departments.len(), 1);
    assert_eq!(departments[&DepartmentKind::Finance], 3);
    let left = seen
        .iter()
        .find(|m| m.key == keys::HEADQUARTERS_STAFF_LEFT)
        .expect("a message");
    assert_eq!(
        left.params
            .iter()
            .find(|(k, _)| k == "anzahl")
            .map(|(_, v)| v.clone()),
        Some(Param::Integer(3))
    );
}

#[test]
fn the_participations_policy_is_checked_and_binds_the_board() {
    let (mut game, _, _) = buyer_game(1_000.0);
    let player = game.player();
    for wrong in [
        participations(Some(usd(-1.0)), 0.5, &[]),
        participations(None, 1.5, &[]),
        participations(None, 0.5, &[(DepartmentKind::Legal, usd(-1.0))]),
    ] {
        assert_eq!(game.apply(wrong), Err(CommandError::InvalidParticipations));
    }
    game.apply(participations(Some(usd(1.0)), 0.3, &[]))
        .unwrap();
    let p = &game.state().companies[0].participations;
    assert_eq!((p.budget, p.risk), (Some(usd(1.0)), 0.3));
    // The budget is too small for the bid: the board asks.
    days(&mut game, 31);
    assert!(!game.state().offers.iter().any(|o| o.buyer == player));
    let concern = &game.state().concerns[0];
    assert_eq!(concern.decision.topic, Topic::Offer);
    assert_eq!(concern.reason, ConcernReason::Participations);
    let view = crate::views::concerns(&game);
    assert_eq!(view.open[0].concerns[0].reason, "beteiligung");

    // A release limit: the head of strategy decides only below it.
    let (mut game, _, _) = buyer_game(1_000.0);
    game.apply(participations(
        None,
        0.5,
        &[(DepartmentKind::Strategy, usd(1.0))],
    ))
    .unwrap();
    days(&mut game, 31);
    assert!(!game.state().offers.iter().any(|o| o.buyer == player));
    let concern = &game.state().concerns[0];
    assert_eq!(concern.reason, ConcernReason::Limit);
    let view = crate::views::concerns(&game);
    let shown = &view.open[0].concerns[0];
    assert_eq!(shown.reason, "freigabe");
    assert_eq!(shown.strategy_limit_usd, Some(1.0));

    // Without them the board bids, and a purchase counts against the budget of the year.
    let (mut game, _, _) = buyer_game(1_000.0);
    days(&mut game, 31);
    assert!(game.state().offers.iter().any(|o| o.buyer == player));
    let price = usd(1_000.0);
    central::count_purchase(game.state_mut(), player, price);
    let year = game.state().date.year();
    assert_eq!(
        game.state().companies[0].participations.spent_in(year),
        price
    );
    assert_eq!(
        game.state().companies[0].participations.spent_in(year + 1),
        Money::ZERO
    );
}

#[test]
fn strategy_observes_countries_and_legal_checks_licences() {
    // A second iron recipe needs the turbine, which only the rival knows.
    let mut c = test_support::management();
    let turbine = c.technologies.id("turbine").unwrap();
    let smelt = c.recipes.id("eisen_schmelzen").unwrap();
    let mut recipe = c.recipes.get(smelt).clone();
    recipe.technology = Some(turbine);
    c.recipes.insert("eisen_turbine", recipe);
    let (mut game, rival, _) = buyer_game_with(c, 1_000.0);
    let catalog = game.catalog().clone();
    let player = game.player();
    game.state_mut().companies[rival.index()]
        .technologies
        .insert(turbine);
    let (aaa, bbb) = (
        catalog.countries.id("AAA").unwrap(),
        catalog.countries.id("BBB").unwrap(),
    );
    assert!(central::observed_countries(&catalog, game.state(), player).is_empty());
    assert!(central::legal_technologies(&catalog, game.state(), player).is_empty());
    staff(&mut game, DepartmentKind::Strategy, 1);
    staff(&mut game, DepartmentKind::Legal, 1);
    // The strategy member is hired in the game; the legal one not yet.
    assert_eq!(
        central::observed_countries(&catalog, game.state(), player),
        vec![bbb]
    );
    assert!(!central::observed_countries(&catalog, game.state(), player).contains(&aaa));
    assert!(central::legal_technologies(&catalog, game.state(), player).is_empty());
    hire_sharp(&mut game, member("recht"));
    assert_eq!(
        central::legal_technologies(&catalog, game.state(), player),
        vec![turbine]
    );
    // The board offers the rival a licence.
    let keys = news(&mut game, 31);
    let offer = game
        .state()
        .offers
        .iter()
        .find(|o| o.buyer == player && o.object == DealObject::License(turbine))
        .expect("a licence offer");
    assert_eq!(offer.seller, rival);
    assert!(
        keys.iter().any(|k| k == keys::BOARD_OFFER_LICENSE),
        "{keys:?}"
    );
}

#[test]
fn a_loan_is_refinanced_only_with_an_advantage() {
    let mut game = new_game(test_support::management());
    mine_and_works(&mut game);
    let catalog = game.catalog().clone();
    let player = game.player();
    let amount = usd(20_000.0);
    game.apply(Command::TakeLoan { amount, years: 5 }).unwrap();
    // As taken: no advantage.
    assert_eq!(
        game.apply(Command::RefinanceLoan { loan: 0 }),
        Err(CommandError::NoAdvantage)
    );
    assert_eq!(
        game.apply(Command::RefinanceLoan { loan: 3 }),
        Err(CommandError::UnknownLoan)
    );
    // Rates fell since: the loan runs at five points more than a new one would.
    let rate = central::refinance_rate(&catalog, game.state(), player);
    game.state_mut().companies[0].loans[0].rate = rate + 0.05;
    days(&mut game, 70);
    let before = game.state().companies[0].loans[0].clone();
    let cash = game.state().companies[0].ledger.cash();
    let rate = central::refinance_rate(&catalog, game.state(), player);
    game.apply(Command::RefinanceLoan { loan: 0 }).unwrap();
    let state = game.state();
    let after = &state.companies[0].loans[0];
    assert_eq!(after.balance, before.balance);
    assert_eq!(after.principal, before.balance);
    assert!((after.rate - rate).abs() < 1e-12);
    assert_eq!(after.start, state.date);
    // Two or three months of the five years have passed.
    assert_eq!(after.months, central::months_left(&before, state.date));
    assert!((57..=58).contains(&after.months), "{}", after.months);
    let fee = before.balance.scale(catalog.central.refinance.fee);
    assert_eq!(state.companies[0].ledger.cash(), cash - fee);
    assert!(state.companies[0].ledger.is_balanced());
    // Booked as interest.
    let interest = state.companies[0].ledger.month.by_type[&CostType::Interest];
    assert!(-interest >= fee);
    // Now it runs at today's rate: no second time.
    assert_eq!(
        game.apply(Command::RefinanceLoan { loan: 0 }),
        Err(CommandError::NoAdvantage)
    );
}
