//! Scenario tests for the headquarters and the central departments (ZA1, ZA2) with
//! `test_support::management`.

use crate::board_tests::{buyer_game, buyer_game_with, member, news};
use crate::calendar::{Date, RoundLength};
use crate::catalog::{Catalog, CentralAiModel, DepartmentKind, Span, test_support};
use crate::central;
use crate::command::{Command, CommandError};
use crate::deals::DealObject;
use crate::decision::Topic;
use crate::finance;
use crate::game::Game;
use crate::ledger::{CostType, PeriodResult};
use crate::management;
use crate::management_tests::{days, hire_sharp, mine_and_works, new_game, usd};
use crate::message::{Param, keys};
use crate::money::Money;
use crate::state::{AiState, CompanyId, CompanyKind, ConcernReason, SiteId, Unit};

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
        game.apply(Command::SetHeadquarters {
            country: aaa,
            city: None,
        }),
        Err(CommandError::SameHeadquarters)
    );
    let cash = game.state().companies[0].ledger.cash();
    game.apply(Command::SetHeadquarters {
        country: bbb,
        city: None,
    })
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
    let until = state.companies[0].relocation.clone().unwrap().until;
    assert_eq!(until, Date::new(1900, 3, 1).unwrap());
    // Only one move at a time.
    assert_eq!(
        game.apply(Command::SetHeadquarters {
            country: aaa,
            city: None,
        }),
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
        game.apply(Command::SetHeadquarters {
            country: bbb,
            city: None,
        }),
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
    game.apply(Command::SetHeadquarters {
        country: bbb,
        city: None,
    })
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

#[test]
fn the_salary_demand_grows_exponentially_with_the_hit_rate() {
    let mut c = test_support::management();
    c.central.hit_rate.prior = 0.0;
    let catalog = c.clone();
    let mut game = new_game(c);
    mine_and_works(&mut game);
    let id = hire_sharp(&mut game, member("strategie"));
    let state = game.state();
    let m = state.managers[&id].clone();
    let position = m.job.as_ref().unwrap().position.clone();
    let player = game.player();
    let base = management::salary_demand(&catalog, state, player, &m, &position);
    // Before the first judgment the factor is 1.
    assert_eq!(central::hit_factor(&catalog, &m), 1.0);
    let with = |judged: u32, hits: u32| {
        let mut x = m.clone();
        x.judged = judged;
        x.hits = hits;
        x
    };
    let good = with(10, 9);
    assert!((central::hit_rate(&catalog, &good) - 0.9).abs() < 1e-12);
    assert!((central::hit_factor(&catalog, &good) - 2.225_540_928_492_468).abs() < 1e-9);
    let poor = with(10, 2);
    assert!((central::hit_factor(&catalog, &poor) - 0.548_811_636_094_026_4).abs() < 1e-9);
    let demand = |x: &crate::state::Manager| {
        management::salary_demand(&catalog, state, player, x, &position).to_usd()
    };
    assert!((demand(&good) / base.to_usd() - 2.2255).abs() < 1e-3);
    assert!((demand(&poor) / base.to_usd() - 0.5488).abs() < 1e-3);
    // With a prior few cases do not deceive.
    let mut c = catalog.clone();
    c.central.hit_rate.prior = 4.0;
    assert!((central::hit_rate(&c, &with(1, 1)) - 0.6).abs() < 1e-12);
    assert!((central::hit_rate(&c, &with(0, 0)) - 0.5).abs() < 1e-12);
}

#[test]
fn a_weak_head_errs_more_often() {
    let (mut game, _, _) = buyer_game(1_000.0);
    let catalog = game.catalog().clone();
    let player = game.player();
    let head = game
        .state()
        .managers
        .iter()
        .find(|(_, m)| {
            m.job
                .as_ref()
                .is_some_and(|j| j.position == member("strategie"))
        })
        .map(|(&id, _)| id)
        .unwrap();
    let misses = |game: &mut Game, expertise: u8| {
        game.state_mut()
            .managers
            .get_mut(&head)
            .unwrap()
            .expertise
            .insert("strategie".into(), expertise);
        let estimates =
            management::estimates(game.state(), &catalog, (player, Topic::Offer), 0..400);
        assert_eq!(estimates.len(), 400);
        assert!(estimates.iter().all(|&(m, _)| m == head));
        // A target worth 20 % more than the price of the rules.
        estimates.iter().filter(|&&(_, f)| f > 1.2).count()
    };
    let weak = misses(&mut game, 20);
    let strong = misses(&mut game, 90);
    // Error 0.5: weak ± 40 % (a quarter above 1.2), strong ± 5 %.
    assert!((70..=130).contains(&weak), "{weak}");
    assert_eq!(strong, 0);
    // A working department narrows the error further.
    staff(&mut game, DepartmentKind::Strategy, 1);
    let with_department = misses(&mut game, 20);
    assert!(with_department < weak / 2, "{with_department} {weak}");

    // The board's offers are estimated and judged after the months of the data.
    let (mut game, _, _) = buyer_game(1_000.0);
    days(&mut game, 31);
    let judgments = game.state().judgments.clone();
    assert!(!judgments.is_empty());
    assert!(judgments.iter().all(|j| j.manager == head));
    let due = judgments[0].due;
    while game.state().date <= due {
        game.advance(RoundLength::Month, |_| {});
    }
    let m = &game.state().managers[&head];
    assert!(m.judged >= 1, "{m:?}");
    assert!(m.hits <= m.judged);
}

#[test]
fn departments_recommend_refinancing_and_raises() {
    let mut game = new_game(test_support::management());
    let (_, works) = mine_and_works(&mut game);
    let catalog = game.catalog().clone();
    let player = game.player();
    game.apply(Command::TakeLoan {
        amount: usd(20_000.0),
        years: 5,
    })
    .unwrap();
    let dear = central::refinance_rate(&catalog, game.state(), player) + 0.05;
    game.state_mut().companies[0].loans[0].rate = dear;
    // A less than content manager paid below the market value (not so unhappy that he
    // might resign).
    let works_head = hire_sharp(&mut game, crate::management_tests::head(works));
    let low =
        crate::staffing::market_value(&catalog, game.state(), &game.state().managers[&works_head])
            .scale(0.9);
    {
        let m = game.state_mut().managers.get_mut(&works_head).unwrap();
        let job = m.job.as_mut().unwrap();
        job.salary = low;
        job.satisfaction = Some(45);
    }
    hire_sharp(&mut game, member("finanzen"));
    hire_sharp(&mut game, member("personal"));
    // Without departments nobody proposes anything.
    days(&mut game, 31);
    assert!((game.state().companies[0].loans[0].rate - dear).abs() < 1e-12);
    let salary = |game: &Game| {
        game.state().managers[&works_head]
            .job
            .as_ref()
            .unwrap()
            .salary
    };
    assert_eq!(salary(&game), low);
    // With them the members decide within their budgets.
    staff(&mut game, DepartmentKind::Finance, 1);
    staff(&mut game, DepartmentKind::Personnel, 1);
    days(&mut game, 31);
    let state = game.state();
    let rate = central::refinance_rate(&catalog, state, player);
    assert!(state.companies[0].loans[0].rate < dear - 0.04, "refinanced");
    assert!((state.companies[0].loans[0].rate - rate).abs() < 0.001);
    let value = crate::staffing::market_value(&catalog, state, &state.managers[&works_head]);
    assert!(salary(&game) > low);
    assert!((salary(&game).to_usd() - value.to_usd()).abs() / value.to_usd() < 0.05);
    assert!(state.companies[0].ledger.is_balanced());
}

#[test]
fn beyond_their_budget_departments_ask_with_reasons() {
    let mut game = new_game(test_support::management());
    mine_and_works(&mut game);
    let catalog = game.catalog().clone();
    let player = game.player();
    game.apply(Command::TakeLoan {
        amount: usd(20_000.0),
        years: 5,
    })
    .unwrap();
    let dear = central::refinance_rate(&catalog, game.state(), player) + 0.05;
    game.state_mut().companies[0].loans[0].rate = dear;
    hire_sharp(&mut game, member("finanzen"));
    staff(&mut game, DepartmentKind::Finance, 1);
    game.apply(Command::SetBudget {
        position: member("finanzen"),
        shares: Some((0.0, 0.0)),
    })
    .unwrap();
    days(&mut game, 31);
    let concern = game
        .state()
        .concerns
        .iter()
        .find(|c| c.decision.topic == Topic::Refinance)
        .expect("a recommendation")
        .clone();
    assert_eq!(concern.reason, ConcernReason::Always);
    let view = crate::views::concerns(&game);
    let shown = view
        .open
        .iter()
        .flat_map(|g| &g.concerns)
        .find(|c| c.id == concern.id)
        .unwrap();
    assert_eq!(shown.options[0].kind, "umschulden");
    assert_eq!(shown.options[0].steps[0].key, keys::STEP_REFINANCE);
    assert_eq!(shown.because.key, keys::BECAUSE_REFINANCE);
    assert!(shown.options[0].once_usd < 0.0, "the fee");
    assert!(
        shown.options[0]
            .forecast_usd
            .is_some_and(|(_, high)| high > 0.0),
        "interest saved"
    );
    // The player follows it: refinanced.
    game.apply(Command::AnswerConcern {
        concern: concern.id,
        answer: management::ConcernAnswer::Delegate,
    })
    .unwrap();
    assert!(game.state().companies[0].loans[0].rate < dear - 0.04);
}

/// An AI company like the player's with twelve closed months of revenue and result behind
/// it (ZA4), and the sites earning part of the revenue.
fn ai_company(
    game: &mut Game,
    (revenue, result): (f64, f64),
    sites: &[(SiteId, f64)],
) -> CompanyId {
    let state = game.state_mut();
    let id = CompanyId(u32::try_from(state.companies.len()).unwrap());
    let mut c = state.companies[0].clone();
    c.name = "Rivale AG".into();
    c.kind = CompanyKind::Ai;
    c.departments = Default::default();
    c.ai = Some(AiState {
        competence: 0.5,
        aggressiveness: 0.5,
        real: None,
        next_operations: state.date,
        staff: 0.0,
    });
    for _ in 0..12 {
        let mut m = PeriodResult::default();
        m.by_type.insert(CostType::Revenue, usd(revenue / 12.0));
        m.by_type
            .insert(CostType::Other, usd(result / 12.0) - usd(revenue / 12.0));
        for &(site, earned) in sites {
            m.site_revenue.insert(site, usd(earned / 12.0));
        }
        c.ledger.months.push(m);
    }
    state.companies.push(c);
    id
}

/// A sales office in a country, handed to a company: a board needs a site (MA5).
fn office_for(game: &mut Game, company: CompanyId, country: &str) -> SiteId {
    let country = game.catalog().countries.id(country).unwrap();
    game.apply(Command::FoundSite {
        country,
        kind: crate::catalog::SiteType::SalesOffice,
    })
    .unwrap();
    let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    game.state_mut().sites[site.index()].owner = company;
    site
}

fn ai_rules(order: Vec<(DepartmentKind, f64)>) -> Catalog {
    let mut c = test_support::management();
    c.central.ai = CentralAiModel {
        revenue_share: Span::fixed(0.01),
        order,
        seat_revenue_share: 0.25,
        seat_gdp_share: 0.75,
        payback_years: 3.0,
        lock_years: 10,
    };
    c
}

#[test]
fn ai_companies_set_up_the_departments_they_can_afford() {
    use DepartmentKind::{Finance, Legal, Marketing, Strategy};
    let catalog = ai_rules(vec![
        (Finance, 2.0),
        (Strategy, 1.0),
        (Legal, 1.0),
        (Marketing, 1.0),
    ]);
    let mut game = new_game(catalog);
    let catalog = game.catalog().clone();
    let probe = ai_company(&mut game, (0.0, 0.0), &[]);
    let one = central::yearly_cost(&catalog, game.state(), probe, Strategy, 1).to_usd();
    assert!(one > 0.0);
    // Enough for two and a half departments: finance has no loan to work on (its least
    // workload is 2), so strategy and legal; marketing would overdraw the budget.
    let rival = ai_company(&mut game, (2.5 * one / 0.01, 1e6), &[]);
    office_for(&mut game, rival, "AAA");
    let plan = central::ai_plan(&catalog, game.state(), rival);
    assert_eq!(
        plan.into_iter().collect::<Vec<_>>(),
        vec![(Strategy, 1), (Legal, 1)]
    );
    // A company with a loss spares its central.
    let poor = ai_company(&mut game, (2.5 * one / 0.01, -1e6), &[]);
    assert!(central::ai_plan(&catalog, game.state(), poor).is_empty());

    // In January it staffs them with the player's command and hires their heads.
    game.state_mut().date = Date::new(1901, 1, 1).unwrap();
    central::ai_month_start(game.state_mut(), &catalog, Date::new(1901, 1, 1).unwrap());
    let state = game.state();
    assert_eq!(central::staff(state, rival, Strategy), 1);
    assert_eq!(central::staff(state, rival, Legal), 1);
    assert_eq!(central::staff(state, rival, Marketing), 0);
    assert!(state.companies[poor.index()].departments.is_empty());
    for _ in 0..2 {
        crate::staffing::month_start(game.state_mut(), &catalog, Date::new(1901, 1, 1).unwrap());
    }
    let state = game.state();
    assert!(
        central::head(&catalog, state, rival, Strategy).is_some(),
        "head hired"
    );
    assert!(central::performance(&catalog, state, rival, Strategy).is_some());

    // After a year with a loss it closes them and lets the heads go.
    let head = central::head(&catalog, state, rival, Strategy).unwrap();
    for m in &mut game.state_mut().companies[rival.index()].ledger.months {
        m.by_type.insert(CostType::Other, -usd(1e9));
    }
    central::ai_month_start(game.state_mut(), &catalog, Date::new(1901, 1, 1).unwrap());
    let state = game.state();
    assert!(state.companies[rival.index()].departments.is_empty());
    assert!(state.managers[&head].job.is_none(), "dismissed");
    assert!(state.companies[rival.index()].ledger.is_balanced());
}

#[test]
fn an_ai_finance_department_refinances_dear_loans() {
    use DepartmentKind::Finance;
    let mut game = new_game(ai_rules(vec![(Finance, 2.0)]));
    let catalog = game.catalog().clone();
    let rival = ai_company(&mut game, (1e9, 1e8), &[]);
    office_for(&mut game, rival, "AAA");
    let rate = central::refinance_rate(&catalog, game.state(), rival);
    let date = game.state().date;
    let loan = crate::state::Loan {
        principal: usd(1e6),
        balance: usd(1e6),
        rate: rate + 0.05,
        start: date,
        months: 60,
        instalment: finance::instalment(usd(1e6), rate + 0.05, 60),
    };
    game.state_mut().companies[rival.index()].loans.push(loan);
    // In January it sets up a finance department (a loan to work on) and hires its head.
    let january = Date::new(1901, 1, 1).unwrap();
    game.state_mut().date = january;
    central::ai_month_start(game.state_mut(), &catalog, january);
    assert_eq!(central::staff(game.state(), rival, Finance), 1);
    // One hire a month: the CEO first, then the head of finance.
    for _ in 0..2 {
        crate::staffing::month_start(game.state_mut(), &catalog, january);
    }
    assert!(central::head(&catalog, game.state(), rival, Finance).is_some());
    // The next month start it refinances the dear loan.
    let next = Date::new(1901, 2, 1).unwrap();
    game.state_mut().date = next;
    central::ai_month_start(game.state_mut(), &catalog, next);
    let loans = &game.state().companies[rival.index()].loans;
    assert!(
        loans[0].rate < rate + 0.05 - 0.01,
        "refinanced: {}",
        loans[0].rate
    );
}

#[test]
fn ai_companies_move_where_their_business_pays_less_tax() {
    let mut game = new_game(ai_rules(Vec::new()));
    let catalog = game.catalog().clone();
    let (aaa, bbb) = (
        catalog.countries.id("AAA").unwrap(),
        catalog.countries.id("BBB").unwrap(),
    );
    // Half of a revenue of 100 million comes from its site in BBB, a fifth is profit.
    let next = SiteId(u32::try_from(game.state().sites.len()).unwrap());
    let rival = ai_company(&mut game, (100e6, 20e6), &[(next, 50e6)]);
    let site = office_for(&mut game, rival, "BBB");
    assert_eq!(site, next);
    let tax = |game: &mut Game, country, rate| {
        game.state_mut().countries.get_mut(country).corporate_tax = rate;
    };
    tax(&mut game, aaa, 0.3);
    tax(&mut game, bbb, 0.3);
    let gdp = game.state().countries.get(aaa).gdp_per_capita_usd;
    game.state_mut().countries.get_mut(bbb).gdp_per_capita_usd = gdp;
    assert_eq!(
        central::ai_seat(&catalog, game.state(), rival),
        None,
        "no saving"
    );
    tax(&mut game, bbb, 0.2);
    // 10 % of 20 million saved a year pays for the move.
    assert_eq!(
        central::seat_saving(&catalog, game.state(), rival, bbb),
        usd(2e6)
    );
    assert_eq!(central::ai_seat(&catalog, game.state(), rival), Some(bbb));
    // Not into a much poorer country.
    game.state_mut().countries.get_mut(bbb).gdp_per_capita_usd = gdp * 0.5;
    assert_eq!(central::ai_seat(&catalog, game.state(), rival), None);
    game.state_mut().countries.get_mut(bbb).gdp_per_capita_usd = gdp;
    // Not where too little of its business is.
    for m in &mut game.state_mut().companies[rival.index()].ledger.months {
        m.site_revenue.insert(site, usd(1e6));
    }
    assert_eq!(central::ai_seat(&catalog, game.state(), rival), None);
    for m in &mut game.state_mut().companies[rival.index()].ledger.months {
        m.site_revenue.insert(site, usd(50e6 / 12.0));
    }
    // In January it moves through the command; the player hears of it.
    let january = Date::new(1901, 1, 1).unwrap();
    game.state_mut().date = january;
    let news = central::ai_month_start(game.state_mut(), &catalog, january);
    assert!(news.iter().any(|m| m.key == keys::RIVAL_HEADQUARTERS));
    let r = game.state().companies[rival.index()]
        .relocation
        .clone()
        .unwrap();
    assert_eq!(r.country, bbb);
    // When the move is done the seat stays for the years of the data.
    let mut date = january;
    while date < r.until {
        date = date.add_months(1);
    }
    central::month_start(game.state_mut(), &catalog, date);
    let c = &game.state().companies[rival.index()];
    assert_eq!(c.headquarters, bbb);
    assert_eq!(c.relocated, Some(date));
    tax(&mut game, aaa, 0.0);
    game.state_mut().date = date.add_months(12);
    assert_eq!(
        central::ai_seat(&catalog, game.state(), rival),
        None,
        "locked"
    );
}

#[test]
fn a_takeover_estimate_counts_by_what_the_object_is_worth_when_due() {
    let mut game = new_game(test_support::management());
    let (_, works) = mine_and_works(&mut game);
    days(&mut game, 40);
    let catalog = game.catalog().clone();
    let now = crate::deals::site_value(game.state(), &catalog, works).base;
    assert!(now > Money::ZERO);
    let (seller, object) = (game.player(), DealObject::Site(works));
    let judge = |game: &Game, base: Money, price: f64| {
        let mut a = central::takeover_appraisal(
            game.state(),
            &catalog,
            (seller, object),
            usd(1e6),
            usd(price),
        )
        .unwrap();
        if let crate::state::Appraisal::Takeover { base: b, .. } = &mut a {
            *b = base;
        }
        let j = crate::state::Judgment {
            manager: crate::state::ManagerId(0),
            due: game.state().date,
            hit: false,
            appraisal: Some(a),
        };
        central::judged_hit(&catalog, game.state(), &j)
    };
    // The rules would have paid a million for the site as it was then. Kept its base
    // value: a price up to a million was right.
    assert!(judge(&game, now, 1e6));
    assert!(!judge(&game, now, 1.1e6));
    // Halved since: only up to half a million was worth it.
    assert!(!judge(&game, now.scale(2.0), 0.6e6));
    assert!(judge(&game, now.scale(2.0), 0.5e6));
    // A licence is settled when estimated.
    let licence = DealObject::License(catalog.technologies.ids().next().unwrap());
    assert!(
        central::takeover_appraisal(
            game.state(),
            &catalog,
            (seller, licence),
            usd(1.0),
            usd(1.0)
        )
        .is_none()
    );
    // Judgments of older saves keep their result.
    let old = crate::state::Judgment {
        manager: crate::state::ManagerId(0),
        due: game.state().date,
        hit: true,
        appraisal: None,
    };
    assert!(central::judged_hit(&catalog, game.state(), &old));
}
