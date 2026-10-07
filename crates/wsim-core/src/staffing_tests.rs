//! Scenario tests for the living market of managers (MA6) with
//! `test_support::management`.

use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{Catalog, SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::decision::{ChoiceKind, Topic};
use crate::game::Game;
use crate::ledger::{CostType, Ledger, PeriodResult};
use crate::management::{self, ConcernAnswer};
use crate::management_tests::{
    days, free, head, hire_sharp, mine_and_works, new_game, specialist, usd,
};
use crate::message::keys;
use crate::money::Money;
use crate::save;
use crate::staffing;
use crate::state::{
    AiState, CompanyId, ConcernReason, ConcernStatus, ManagerId, Position, Role, SiteId, Unit,
};

fn ceo() -> Position {
    Position::new(Unit::Board, Role::Head)
}

/// Days up to and including the next month start; the message keys on the way.
fn to_next_month(game: &mut Game) -> Vec<String> {
    let mut keys = Vec::new();
    loop {
        let report = game.advance(RoundLength::Day, |_| {});
        keys.extend(report.messages.into_iter().map(|m| m.key));
        if game.state().date.day() == 1 {
            return keys;
        }
    }
}

fn job(game: &Game, id: ManagerId) -> crate::state::Job {
    game.state().managers[&id].job.clone().expect("employed")
}

/// A strong manager: the most expertise, detection and judgment.
fn make_strong(game: &mut Game, id: ManagerId) {
    let m = game.state_mut().managers.get_mut(&id).unwrap();
    m.detection = 100;
    m.judgment = 100;
    for v in m.expertise.values_mut() {
        *v = 100;
    }
}

/// The free candidates are weak: an AI company would rather poach.
fn weaken_market(game: &mut Game) {
    for m in game.state_mut().managers.values_mut() {
        if m.job.is_none() {
            m.detection = 10;
            m.judgment = 10;
            for v in m.expertise.values_mut() {
                *v = 10;
            }
        }
    }
}

#[test]
fn experience_grows_up_to_the_potential() {
    let mut c = test_support::management();
    c.management.market.experience_chance = 1.0;
    let mut game = new_game(c);
    let (_, works) = mine_and_works(&mut game);
    let state = game.state();
    // Every manager has a potential above his best expertise, within the room.
    for m in state.managers.values() {
        let best = m.expertise.values().copied().max().unwrap();
        let p = m.potential.expect("set at the start");
        assert!(
            p >= best && p <= best.saturating_add(20).min(100),
            "{p} {best}"
        );
    }
    let id = free(&game)[0];
    let other = free(&game)[1];
    game.apply(Command::HireManager {
        manager: id,
        position: specialist(works, "produktion"),
    })
    .unwrap();
    let m = game.state_mut().managers.get_mut(&id).unwrap();
    let start = 40;
    m.expertise.insert("produktion".into(), start);
    m.potential = Some(start + 2);
    let before = game.state().managers[&other].expertise.clone();
    for _ in 0..4 {
        to_next_month(&mut game);
    }
    let state = game.state();
    assert_eq!(
        state.managers[&id].expertise["produktion"],
        start + 2,
        "a point a month, up to the potential"
    );
    if let Some(x) = state.managers.get(&other) {
        assert_eq!(x.expertise, before, "candidates do not learn");
    }
}

#[test]
fn old_saves_get_potentials_at_the_next_month() {
    let catalog = Arc::new(test_support::management());
    let mut game = new_game((*catalog).clone());
    for m in game.state_mut().managers.values_mut() {
        m.potential = None;
    }
    let mut loaded = save::decode(&save::encode(&game), catalog).unwrap().game;
    assert!(
        loaded
            .state()
            .managers
            .values()
            .all(|m| m.potential.is_none())
    );
    to_next_month(&mut loaded);
    assert!(
        loaded
            .state()
            .managers
            .values()
            .all(|m| m.potential.is_some())
    );
}

#[test]
fn satisfaction_follows_pay_results_and_overruling() {
    let mut game = new_game(test_support::management());
    let (_, works) = mine_and_works(&mut game);
    let id = hire_sharp(&mut game, head(works));
    let c = game.catalog().clone();
    let s = c.management.market.satisfaction;
    let state = game.state();
    let m = &state.managers[&id];
    let value = staffing::market_value(&c, state, m);
    assert!(value > Money::ZERO);
    let at = |game: &mut Game, salary: Money, overruled: u32| {
        game.state_mut()
            .managers
            .get_mut(&id)
            .unwrap()
            .job
            .as_mut()
            .unwrap()
            .salary = salary;
        let state = game.state();
        staffing::satisfaction_target(&c, state, &state.managers[&id], overruled)
    };
    let fair = at(&mut game, value, 0);
    let loss = staffing::unit_result(&c, game.state(), CompanyId(0), Unit::Site(works));
    assert_eq!(loss, Money::ZERO, "no closed month yet");
    assert!((fair - s.base).abs() < 1e-9);
    let richer = at(&mut game, value.scale(1.1), 0);
    assert!((richer - fair - 10.0 * s.salary_weight).abs() < 1e-6);
    let overruled = at(&mut game, value, 2);
    assert!((fair - overruled - 2.0 * s.overruled_penalty).abs() < 1e-9);
    // A new works loses money in its first month: the target drops by the penalty.
    at(&mut game, value.scale(0.5), 0);
    let start = s.start;
    assert_eq!(job(&game, id).satisfaction, Some(start));
    to_next_month(&mut game);
    let state = game.state();
    assert!(staffing::unit_result(&c, state, CompanyId(0), Unit::Site(works)) < Money::ZERO);
    let target = staffing::satisfaction_target(&c, state, &state.managers[&id], 0);
    let market = staffing::market_value(&c, state, &state.managers[&id]);
    let pay = 100.0 * (job(&game, id).salary.to_usd() / market.to_usd() - 1.0);
    assert!((target - (s.base + s.salary_weight * pay - s.loss_penalty)).abs() < 1e-6);
    let expected = (f64::from(start) + s.adjust * (target - f64::from(start))).round();
    let got = f64::from(job(&game, id).satisfaction.unwrap());
    assert!(
        (got - expected).abs() <= 1.0,
        "a share of the way to the target: {got} {expected}"
    );
    assert_eq!(staffing::satisfaction_level(&c, 10), 0);
    assert_eq!(staffing::satisfaction_level(&c, 45), 1);
    assert_eq!(staffing::satisfaction_level(&c, 80), 2);
}

#[test]
fn a_raise_is_only_upwards() {
    let mut game = new_game(test_support::management());
    let (_, works) = mine_and_works(&mut game);
    let id = hire_sharp(&mut game, head(works));
    let salary = job(&game, id).salary;
    assert_eq!(
        game.apply(Command::RaiseSalary {
            manager: id,
            salary
        }),
        Err(CommandError::SalaryNotHigher)
    );
    let candidate = free(&game)[0];
    assert_eq!(
        game.apply(Command::RaiseSalary {
            manager: candidate,
            salary: salary.scale(2.0)
        }),
        Err(CommandError::NotYourManager)
    );
    game.apply(Command::RaiseSalary {
        manager: id,
        salary: salary.scale(1.25),
    })
    .unwrap();
    assert_eq!(job(&game, id).salary, salary.scale(1.25));
}

#[test]
fn unhappy_managers_resign() {
    let mut c = test_support::management();
    c.management.market.resignation_chance = 1.0;
    c.management.market.satisfaction.adjust = 0.0;
    let mut game = new_game(c);
    let (_, works) = mine_and_works(&mut game);
    let id = hire_sharp(&mut game, head(works));
    let happy = hire_sharp(&mut game, specialist(works, "produktion"));
    game.state_mut()
        .managers
        .get_mut(&id)
        .unwrap()
        .job
        .as_mut()
        .unwrap()
        .satisfaction = Some(0);
    let keys = to_next_month(&mut game);
    assert!(keys.iter().any(|k| k == keys::MANAGER_RESIGNED));
    let state = game.state();
    assert!(state.managers[&id].job.is_none(), "back in the market");
    assert!(state.managers[&happy].job.is_some());
    assert_eq!(management::holder(state, CompanyId(0), &head(works)), None);
    assert!(state.companies[0].ledger.is_balanced());
}

#[test]
fn a_head_fills_the_positions_of_its_unit() {
    let mut c = test_support::management();
    // Room in the head's budget for a few salaries.
    c.management.budget_floor = (5.0, 10.0);
    let mut game = new_game(c);
    let (_, works) = mine_and_works(&mut game);
    let boss = hire_sharp(&mut game, head(works));
    assert_eq!(
        game.apply(Command::SetHiringByHead {
            position: specialist(works, "produktion"),
            enabled: true
        }),
        Err(CommandError::NotAHead)
    );
    game.apply(Command::SetHiringByHead {
        position: head(works),
        enabled: true,
    })
    .unwrap();
    let keys = to_next_month(&mut game);
    assert!(keys.iter().any(|k| k == keys::MANAGER_HIRED_BY_HEAD));
    let state = game.state();
    let player = CompanyId(0);
    let filled: Vec<Position> =
        management::positions(game.catalog(), state, player, Unit::Site(works))
            .into_iter()
            .filter(|p| p.role != Role::Head && management::holder(state, player, p).is_some())
            .collect();
    assert_eq!(
        filled,
        vec![specialist(works, "produktion")],
        "one a month, in order"
    );
    let hired = management::holder(state, player, &filled[0]).unwrap();
    let salary = job(&game, hired).salary;
    // The salary counts against the head's budget of the year, with its other decisions.
    assert!(management::spent(state, player, &head(works)) >= salary);
    assert!(state.managers[&boss].job.is_some());
    // Next month the next position with work.
    to_next_month(&mut game);
    let state = game.state();
    let count = management::positions(game.catalog(), state, player, Unit::Site(works))
        .iter()
        .filter(|p| p.role != Role::Head && management::holder(state, player, p).is_some())
        .count();
    assert_eq!(count, 2);
    // Switched off, nobody more is hired.
    game.apply(Command::SetHiringByHead {
        position: head(works),
        enabled: false,
    })
    .unwrap();
    to_next_month(&mut game);
    let state = game.state();
    let after = management::positions(game.catalog(), state, player, Unit::Site(works))
        .iter()
        .filter(|p| p.role != Role::Head && management::holder(state, player, p).is_some())
        .count();
    assert_eq!(after, 2);
}

/// An AI rival with revenue enough for a CEO and site heads, and a site of its own.
fn with_rival(catalog: Catalog) -> (Game, CompanyId, SiteId) {
    let mut game = new_game(catalog);
    let c = game.catalog().clone();
    let state = game.state_mut();
    let mut rival = state.companies[0].clone();
    "Rivale AG".clone_into(&mut rival.name);
    rival.ledger = Ledger::new(state.date, usd(50_000_000.0));
    rival.positions.clear();
    rival.ai = Some(AiState {
        competence: 1.0,
        aggressiveness: 0.5,
        real: None,
        next_operations: Date::new(2100, 1, 1).unwrap(),
        staff: 0.0,
    });
    state.companies.push(rival);
    let rival = CompanyId(1);
    let aaa = c.countries.id("AAA").unwrap();
    game.apply_as(
        rival,
        Command::FoundSite {
            country: aaa,
            kind: SiteType::Factory,
        },
    )
    .unwrap();
    let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    // A closed month with a large revenue, all of it at its site.
    let mut month = PeriodResult::default();
    month.by_type.insert(CostType::Revenue, usd(80_000_000.0));
    month.site_revenue.insert(site, usd(80_000_000.0));
    game.state_mut().companies[1].ledger.months.push(month);
    (game, rival, site)
}

#[test]
fn ai_companies_hire_a_ceo_then_site_heads() {
    let (mut game, rival, site) = with_rival(test_support::management());
    let before = game.state().companies[1].ai.as_ref().unwrap().skill();
    to_next_month(&mut game);
    let state = game.state();
    let boss = management::holder(state, rival, &ceo()).expect("a CEO");
    let strongest = state
        .managers
        .iter()
        .filter(|(id, m)| m.job.is_none() || **id == boss)
        .map(|(_, m)| management::strength(m))
        .fold(0.0, f64::max);
    // Competence 1: the strongest candidate.
    assert!((management::strength(&state.managers[&boss]) - strongest).abs() < 1e-9);
    let ai = state.companies[1].ai.as_ref().unwrap();
    let expected = 0.15 * (management::strength(&state.managers[&boss]) - 50.0) / 50.0;
    assert!((ai.staff - expected).abs() < 1e-9);
    assert!((ai.skill() - (before + expected).clamp(0.0, 1.0)).abs() < 1e-9);
    // One position a month: next the head of its site.
    assert_eq!(management::holder(state, rival, &head(site)), None);
    to_next_month(&mut game);
    assert!(management::holder(game.state(), rival, &head(site)).is_some());
    // AI companies' managers take no decisions and ask nothing.
    days(&mut game, 60);
    let state = game.state();
    assert!(state.concerns.iter().all(|c| c.company != rival));
    assert!(state.companies[1].positions.is_empty());
    // Their salaries are the company's personnel costs.
    let personnel: Money = state.companies[1]
        .ledger
        .months
        .iter()
        .filter_map(|m| m.by_type.get(&CostType::Personnel))
        .copied()
        .sum();
    assert!(personnel < Money::ZERO);
    assert!(state.companies[1].ledger.is_balanced());
}

/// The rival poaches the player's strong CEO; returns the concern's id.
fn poached_ceo(catalog: Catalog) -> (Game, ManagerId, u32) {
    let (mut game, _, _) = with_rival(catalog);
    mine_and_works(&mut game);
    let boss = hire_sharp(&mut game, ceo());
    make_strong(&mut game, boss);
    weaken_market(&mut game);
    let keys = to_next_month(&mut game);
    assert!(keys.iter().any(|k| k == keys::MANAGER_POACH), "{keys:?}");
    let state = game.state();
    let offer = state.poach_offers.first().expect("an offer");
    assert_eq!(offer.manager, boss);
    assert_eq!(offer.bidder, CompanyId(1));
    assert_eq!(offer.position, ceo());
    let concern = state
        .concerns
        .iter()
        .find(|c| c.status == ConcernStatus::Open && c.decision.topic == Topic::Poaching)
        .expect("a concern");
    assert_eq!(concern.reason, ConcernReason::Poaching);
    let kinds: Vec<ChoiceKind> = concern.decision.choices.iter().map(|o| o.kind).collect();
    assert_eq!(kinds, vec![ChoiceKind::Counter, ChoiceKind::LetGo]);
    // Without a personnel position the CEO asks himself.
    assert_eq!(concern.position, ceo());
    assert!(management::important(game.catalog(), concern));
    // The rival waits for the answer: its CEO position stays free.
    assert_eq!(management::holder(state, CompanyId(1), &ceo()), None);
    let id = concern.id;
    (game, boss, id)
}

#[test]
fn the_player_matches_an_offer() {
    let (mut game, boss, concern) = poached_ceo(test_support::management());
    let offer = game.state().poach_offers[0].salary;
    let old = job(&game, boss).salary;
    assert!(offer >= old.scale(1.2));
    game.apply(Command::AnswerConcern {
        concern,
        answer: ConcernAnswer::Choose(0),
    })
    .unwrap();
    let state = game.state();
    assert!(state.poach_offers.is_empty());
    let j = job(&game, boss);
    assert_eq!(j.company, CompanyId(0));
    assert_eq!(j.salary, offer);
    assert_eq!(
        j.satisfaction,
        Some(game.catalog().management.market.satisfaction.start)
    );
    let c = state.concerns.iter().find(|c| c.id == concern).unwrap();
    assert_eq!(c.status, ConcernStatus::Chosen(0));
    // No new offer to him for a year, also not by command.
    let rival = CompanyId(1);
    assert!(matches!(
        game.apply_as(
            rival,
            Command::PoachManager {
                manager: boss,
                position: ceo()
            }
        ),
        Err(CommandError::ManagerCourted { .. })
    ));
}

#[test]
fn the_player_lets_a_manager_go() {
    let (mut game, boss, concern) = poached_ceo(test_support::management());
    let offer = game.state().poach_offers[0].salary;
    game.apply(Command::AnswerConcern {
        concern,
        answer: ConcernAnswer::Choose(1),
    })
    .unwrap();
    let state = game.state();
    let j = job(&game, boss);
    assert_eq!(j.company, CompanyId(1));
    assert_eq!(j.position, ceo());
    assert_eq!(j.salary, offer);
    assert_eq!(management::holder(state, CompanyId(0), &ceo()), None);
    assert!(state.companies[0].ledger.is_balanced());
}

#[test]
fn an_unanswered_offer_lapses_and_disappoints() {
    let (mut game, boss, concern) = poached_ceo(test_support::management());
    let before = job(&game, boss).satisfaction.unwrap();
    let mut keys = Vec::new();
    for _ in 0..35 {
        let report = game.advance(RoundLength::Day, |_| {});
        keys.extend(report.messages.into_iter().map(|m| m.key));
    }
    assert!(keys.iter().any(|k| k == keys::MANAGER_STAYED));
    let state = game.state();
    assert!(
        state.poach_offers.iter().all(|o| o.manager != boss),
        "the offer lapsed"
    );
    let c = state.concerns.iter().find(|c| c.id == concern).unwrap();
    assert_eq!(c.status, ConcernStatus::Expired);
    let j = job(&game, boss);
    assert_eq!(j.company, CompanyId(0), "he stays");
    let penalty = game.catalog().management.market.poaching.ignored_penalty;
    // The penalty, and a month start that moved it on.
    assert!(j.satisfaction.unwrap() < before.saturating_sub(penalty / 2));
}

#[test]
fn a_personnel_position_asks_and_may_decide_itself() {
    let mut c = test_support::management();
    c.management.budget_floor = (5.0, 10.0);
    let (mut game, _, _) = with_rival(c);
    let player = CompanyId(0);
    mine_and_works(&mut game);
    let boss = hire_sharp(&mut game, ceo());
    make_strong(&mut game, boss);
    let hr = Position::new(Unit::Board, Role::Specialist("personal".into()));
    let hr_manager = hire_sharp(&mut game, hr.clone());
    // The personnel member is no candidate for the rival: weak.
    {
        let m = game.state_mut().managers.get_mut(&hr_manager).unwrap();
        m.detection = 10;
        m.judgment = 10;
        for v in m.expertise.values_mut() {
            *v = 10;
        }
    }
    weaken_market(&mut game);
    // The player let the personnel member decide offers itself.
    game.state_mut().companies[0]
        .positions
        .push(crate::state::PositionState {
            position: hr.clone(),
            budget: None,
            spent: Money::ZERO,
            year: 1900,
            muted: [Topic::Poaching].into_iter().collect(),
            blocked: Default::default(),
            log: Vec::new(),
            hires: false,
        });
    let keys = to_next_month(&mut game);
    assert!(
        keys.iter().any(|k| k == keys::MANAGER_POACH_DECIDED),
        "{keys:?}"
    );
    let state = game.state();
    assert!(
        state
            .concerns
            .iter()
            .all(|c| c.decision.topic != Topic::Poaching)
    );
    // It did what the AI would: keep him (within 1.3 × his salary) or let him go.
    let j = job(&game, boss);
    if j.company == player {
        assert!(management::spent(state, player, &hr) > Money::ZERO);
    } else {
        assert_eq!(j.company, CompanyId(1));
    }
}

#[test]
fn an_ai_company_keeps_its_manager_or_lets_him_go() {
    let (mut game, rival, site) = with_rival(test_support::management());
    // The rival hires its CEO first; then a third company poaches him.
    to_next_month(&mut game);
    let boss = management::holder(game.state(), rival, &ceo()).unwrap();
    make_strong(&mut game, boss);
    let state = game.state_mut();
    let mut third = state.companies[1].clone();
    "Dritte AG".clone_into(&mut third.name);
    state.companies.push(third);
    let third = CompanyId(2);
    let aaa = game.catalog().countries.id("AAA").unwrap();
    game.apply_as(
        third,
        Command::FoundSite {
            country: aaa,
            kind: SiteType::Factory,
        },
    )
    .unwrap();
    let salary = job(&game, boss).salary;
    game.apply_as(
        third,
        Command::PoachManager {
            manager: boss,
            position: ceo(),
        },
    )
    .unwrap();
    let offer = game.state().poach_offers[0].salary;
    days(&mut game, 1);
    let j = job(&game, boss);
    let factor = game.catalog().management.market.poaching.ai_counter_max;
    if offer <= salary.scale(factor) {
        assert_eq!(j.company, rival, "kept");
        assert_eq!(j.salary, offer);
    } else {
        assert_eq!(j.company, third, "let go");
    }
    assert!(game.state().poach_offers.is_empty());
    let _ = site;
}

#[test]
fn poaching_is_checked() {
    let (mut game, rival, _) = with_rival(test_support::management());
    let (_, works) = mine_and_works(&mut game);
    let boss = hire_sharp(&mut game, head(works));
    let candidate = free(&game)[0];
    let poach = |manager| Command::PoachManager {
        manager,
        position: ceo(),
    };
    assert_eq!(
        game.apply_as(rival, poach(candidate)),
        Err(CommandError::ManagerFree)
    );
    assert_eq!(game.apply(poach(boss)), Err(CommandError::OwnManager));
    assert_eq!(
        game.apply(Command::MatchOffer { manager: boss }),
        Err(CommandError::NoPoachOffer)
    );
    game.apply_as(rival, poach(boss)).unwrap();
    assert_eq!(
        game.apply_as(rival, poach(boss)),
        Err(CommandError::ManagerHasOffer)
    );
    // The position stays reserved for the offer.
    let other = hire_sharp(&mut game, specialist(works, "produktion"));
    assert_eq!(
        game.apply_as(rival, poach(other)),
        Err(CommandError::PositionTaken)
    );
    // Dismissing him ends the offer and its concern.
    days(&mut game, 1);
    assert_eq!(game.state().poach_offers.len(), 1);
    game.apply(Command::DismissManager { manager: boss })
        .unwrap();
    days(&mut game, 1);
    let state = game.state();
    assert!(state.poach_offers.is_empty());
    assert!(
        state
            .concerns
            .iter()
            .filter(|c| c.decision.topic == Topic::Poaching)
            .all(|c| c.status != ConcernStatus::Open)
    );
}

#[test]
fn games_with_a_living_market_load_and_replay_identically() {
    let catalog = Arc::new({
        let mut c = test_support::management();
        c.management.budget_floor = (5.0, 10.0);
        c
    });
    let setup = |game: &mut Game| {
        let (_, works) = mine_and_works(game);
        let id = free(game)[0];
        game.apply(Command::HireManager {
            manager: id,
            position: head(works),
        })
        .unwrap();
        game.apply(Command::SetHiringByHead {
            position: head(works),
            enabled: true,
        })
        .unwrap();
        let salary = game.state().managers[&id].job.as_ref().unwrap().salary;
        game.apply(Command::RaiseSalary {
            manager: id,
            salary: salary.scale(1.1),
        })
        .unwrap();
    };
    let play = |game: &mut Game| {
        for _ in 0..2 {
            game.advance(RoundLength::Month, |_| {});
        }
    };
    let mut a = new_game((*catalog).clone());
    setup(&mut a);
    play(&mut a);
    play(&mut a);
    assert!(
        a.state()
            .managers
            .values()
            .filter(|m| m.job.is_some())
            .count()
            > 1,
        "the head hired"
    );

    let mut b = new_game((*catalog).clone());
    setup(&mut b);
    play(&mut b);
    let mut loaded = save::decode(&save::encode(&b), catalog.clone())
        .unwrap()
        .game;
    play(&mut loaded);
    assert_eq!(loaded.state_hash(), a.state_hash());

    let replayed = Game::replay(catalog, a.state().settings.clone(), a.journal()).unwrap();
    assert_eq!(replayed.state_hash(), a.state_hash());
}

#[test]
fn games_with_offers_load_identically() {
    let (game, _, _) = poached_ceo(test_support::management());
    let catalog = game.catalog().clone();
    let loaded = save::decode(&save::encode(&game), catalog).unwrap().game;
    assert_eq!(loaded.state_hash(), game.state_hash());
    assert_eq!(loaded.state().poach_offers, game.state().poach_offers);
}
