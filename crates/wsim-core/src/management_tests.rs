//! Scenario tests for managers (MA1) with `test_support::management`.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::calendar::{Date, RoundLength};
use crate::catalog::{Catalog, SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::ledger::CostType;
use crate::management::{self, shown_level};
use crate::money::Money;
use crate::save;
use crate::state::{GameSettings, ManagerId, Position, Role, SiteId, StartForm};

fn usd(v: f64) -> Money {
    Money::from_usd(v).unwrap()
}

fn new_game(catalog: Catalog) -> Game {
    let catalog = Arc::new(catalog);
    let settings = GameSettings {
        seed: 7,
        start_year: 1900,
        start_country: catalog.countries.id("AAA").unwrap(),
        start_capital: usd(10_000_000.0),
        start_form: StartForm::Workshop,
        company_name: "Hütte AG".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: Default::default(),
    };
    Game::new(catalog, settings).unwrap()
}

fn days(game: &mut Game, n: u32) {
    for _ in 0..n {
        game.advance(RoundLength::Day, |_| {});
    }
}

fn found(game: &mut Game, kind: SiteType) -> SiteId {
    let aaa = game.catalog().countries.id("AAA").unwrap();
    game.apply(Command::FoundSite { country: aaa, kind })
        .unwrap();
    SiteId(u32::try_from(game.state().sites.len() - 1).unwrap())
}

/// A mine producing ore (as the player sets it up) and a works with a furnace on which
/// nobody chose a recipe yet.
fn mine_and_works(game: &mut Game) -> (SiteId, SiteId) {
    let c = game.catalog().clone();
    let mine = found(game, SiteType::Extraction);
    game.apply(Command::BuildFacility {
        site: mine,
        facility: c.facilities.id("mine").unwrap(),
        count: 1,
        size: crate::catalog::FacilitySize::Medium,
    })
    .unwrap();
    game.apply(Command::DevelopDeposit {
        site: mine,
        deposit: c.deposits.id("grube").unwrap(),
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site: mine,
        slot: 0,
        recipe: c.recipes.id("erz_abbau"),
        utilization: 1.0,
    })
    .unwrap();
    let works = found(game, SiteType::Factory);
    game.apply(Command::BuildFacility {
        site: works,
        facility: c.facilities.id("ofen").unwrap(),
        count: 1,
        size: crate::catalog::FacilitySize::Medium,
    })
    .unwrap();
    (mine, works)
}

/// Free candidates, in ID order.
fn free(game: &Game) -> Vec<ManagerId> {
    game.state()
        .managers
        .iter()
        .filter(|(_, m)| m.job.is_none())
        .map(|(&id, _)| id)
        .collect()
}

fn head(site: SiteId) -> Position {
    Position {
        site,
        role: Role::Head,
    }
}

fn specialist(site: SiteId, function: &str) -> Position {
    Position {
        site,
        role: Role::Specialist(function.into()),
    }
}

fn personnel(game: &Game, site: SiteId) -> Money {
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    -ledger.month.site_type(site, CostType::Personnel)
}

#[test]
fn the_market_has_candidates_from_the_start() {
    let game = new_game(test_support::management());
    let c = game.catalog().clone();
    let managers = &game.state().managers;
    // Few academics in the small countries: the minimum.
    assert_eq!(managers.len(), 6);
    let names: BTreeSet<&str> = managers.values().map(|m| m.name.as_str()).collect();
    assert_eq!(names.len(), managers.len(), "names are unique");
    let keys = management::skill_keys(&c);
    assert_eq!(keys.len(), 5 + 5);
    for m in managers.values() {
        assert!(m.job.is_none());
        assert_eq!(
            c.countries.get(m.home).continent,
            c.continents.ids().next().unwrap()
        );
        assert!(c.management.function(&m.focus).is_some());
        assert_eq!(m.expertise.len(), 5);
        for key in &keys {
            let value = management::skill(m, key).unwrap();
            assert!(value <= 100);
            let offset = m.impression[key];
            assert!((-15..=15).contains(&offset), "{key}: {offset}");
        }
        let parts: Vec<&str> = m.name.split(' ').collect();
        assert_eq!(parts.len(), 2, "{}", m.name);
    }
}

#[test]
fn the_market_turns_over_every_month() {
    let mut game = new_game(test_support::management());
    let before: BTreeSet<ManagerId> = game.state().managers.keys().copied().collect();
    for _ in 0..6 {
        game.advance(RoundLength::Month, |_| {});
    }
    let after: BTreeSet<ManagerId> = game.state().managers.keys().copied().collect();
    assert_eq!(after.len(), 6, "refilled to the size of the market");
    assert!(before.difference(&after).count() > 0, "candidates left");
    assert!(game.state().next_manager > 6, "new candidates came");
}

#[test]
fn hiring_moving_and_dismissing_are_checked() {
    let mut game = new_game(test_support::management());
    let works = found(&mut game, SiteType::Factory);
    let lab = found(&mut game, SiteType::Extraction);
    let ids = free(&game);
    let (a, b) = (ids[0], ids[1]);

    let hire = |manager, position| Command::HireManager { manager, position };
    assert_eq!(
        game.apply(hire(a, specialist(works, "logistik"))),
        Err(CommandError::UnknownPosition)
    );
    assert_eq!(
        game.apply(hire(a, specialist(lab, "einkauf_lager"))),
        Err(CommandError::UnknownPosition),
        "mines have no purchasing position"
    );
    assert_eq!(
        game.apply(hire(a, head(SiteId(99)))),
        Err(CommandError::UnknownSite)
    );
    assert_eq!(
        game.apply(hire(ManagerId(999), head(works))),
        Err(CommandError::UnknownManager)
    );
    game.apply(hire(a, head(works))).unwrap();
    assert_eq!(
        game.apply(hire(a, specialist(works, "produktion"))),
        Err(CommandError::ManagerEmployed)
    );
    assert_eq!(
        game.apply(hire(b, head(works))),
        Err(CommandError::PositionTaken)
    );
    let job = game.state().managers[&a].job.clone().unwrap();
    assert_eq!(job.company, game.player());
    let m = &game.state().managers[&a];
    let demand = management::salary_demand(game.catalog(), game.state(), m, &head(works));
    assert_eq!(job.salary, demand);
    assert!(job.salary > Money::ZERO);

    // Moved to a specialist position: the salary does not fall.
    game.apply(Command::MoveManager {
        manager: a,
        position: specialist(works, "produktion"),
    })
    .unwrap();
    let moved = game.state().managers[&a].job.clone().unwrap();
    assert_eq!(moved.position, specialist(works, "produktion"));
    assert_eq!(moved.salary, job.salary);
    assert_eq!(
        game.apply(Command::MoveManager {
            manager: b,
            position: head(works),
        }),
        Err(CommandError::NotYourManager)
    );
    assert_eq!(
        game.apply(Command::DismissManager { manager: b }),
        Err(CommandError::NotYourManager)
    );

    // Dismissed: three monthly salaries, back in the market.
    let before = personnel(&game, works);
    game.apply(Command::DismissManager { manager: a }).unwrap();
    let severance = moved.salary.scale(3.0 / 12.0);
    assert_eq!(personnel(&game, works) - before, severance);
    assert!(game.state().managers[&a].job.is_none());
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    assert!(ledger.is_balanced());
}

#[test]
fn salaries_are_booked_at_the_end_of_the_month() {
    let mut game = new_game(test_support::management());
    let office = found(&mut game, SiteType::Factory);
    let a = free(&game)[0];
    game.apply(Command::HireManager {
        manager: a,
        position: head(office),
    })
    .unwrap();
    let salary = game.state().managers[&a].job.as_ref().unwrap().salary;
    days(&mut game, 30);
    assert_eq!(
        personnel(&game, office),
        Money::ZERO,
        "booked at the month end"
    );
    days(&mut game, 1);
    assert_eq!(game.date(), Date::new(1900, 2, 1).unwrap());
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    let january = ledger.months.last().unwrap();
    assert_eq!(
        -january.site_type(office, CostType::Personnel),
        Money::from_units(salary.units() / 12)
    );
    assert!(ledger.is_balanced());

    // Hired in the middle of a month: paid by days.
    let b = free(&game)[0];
    let other = found(&mut game, SiteType::Factory);
    days(&mut game, 14);
    game.apply(Command::HireManager {
        manager: b,
        position: head(other),
    })
    .unwrap();
    let salary_b = game.state().managers[&b].job.as_ref().unwrap().salary;
    days(&mut game, 14);
    assert_eq!(game.date(), Date::new(1900, 3, 1).unwrap());
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    let february = ledger.months.last().unwrap();
    assert_eq!(
        -february.site_type(other, CostType::Personnel),
        Money::from_units(salary_b.units() / 12 * 14 / 28),
        "hired on 15 February: paid for 15 to 28 February"
    );
}

#[test]
fn a_works_with_heads_runs_by_itself() {
    let iron = |game: &Game, works: SiteId| {
        let iron = game.catalog().products.id("eisen").unwrap();
        let s = game.state().site(works).unwrap();
        let made = s.inventory.get(&iron).map_or(0.0, |x| x.quantity);
        let sold = s
            .offers
            .get(&iron)
            .map_or(0.0, |o| o.sold_month + o.sold_last_month);
        (s.slots[0].recipe, made + sold)
    };

    // Without managers the furnace waits for the player.
    let mut alone = new_game(test_support::management());
    let (_, works) = mine_and_works(&mut alone);
    days(&mut alone, 90);
    assert_eq!(iron(&alone, works), (None, 0.0));

    let mut game = new_game(test_support::management());
    let (mine, works) = mine_and_works(&mut game);
    let ids = free(&game);
    game.apply(Command::HireManager {
        manager: ids[0],
        position: head(mine),
    })
    .unwrap();
    game.apply(Command::HireManager {
        manager: ids[1],
        position: head(works),
    })
    .unwrap();
    days(&mut game, 90);
    let (recipe, made) = iron(&game, works);
    assert_eq!(recipe, game.catalog().recipes.id("eisen_schmelzen"));
    assert!(made > 0.0, "the works makes iron from its own ore");
    let ore = game.catalog().products.id("erz").unwrap();
    assert!(
        game.state().site(mine).unwrap().offers.contains_key(&ore),
        "the mine offers its ore"
    );
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    assert!(ledger.is_balanced());
}

#[test]
fn a_position_that_notices_nothing_changes_nothing() {
    let mut catalog = test_support::management();
    catalog.management.notice_base = 0.0;
    let mut game = new_game(catalog);
    let (_, works) = mine_and_works(&mut game);
    let a = free(&game)[0];
    game.apply(Command::HireManager {
        manager: a,
        position: head(works),
    })
    .unwrap();
    // Without any diligence nothing is ever noticed.
    {
        let m = game.state_mut().managers.get_mut(&a).unwrap();
        m.detection = 0;
        for v in m.expertise.values_mut() {
            *v = 0;
        }
    }
    days(&mut game, 60);
    assert_eq!(game.state().site(works).unwrap().slots[0].recipe, None);
}

#[test]
fn managers_lose_their_position_with_the_site() {
    let mut game = new_game(test_support::management());
    let works = found(&mut game, SiteType::Factory);
    let a = free(&game)[0];
    game.apply(Command::HireManager {
        manager: a,
        position: head(works),
    })
    .unwrap();
    days(&mut game, 10);
    let salary = game.state().managers[&a].job.as_ref().unwrap().salary;
    management::release_site(game.state_mut(), works);
    assert!(game.state().managers[&a].job.is_none());
    assert_eq!(
        personnel(&game, works),
        Money::from_units(salary.units() / 12 * 10 / 31),
        "the days of the month so far are paid"
    );
}

#[test]
fn old_saves_get_their_market_at_the_next_month() {
    let mut game = new_game(test_support::management());
    game.state_mut().managers.clear();
    days(&mut game, 30);
    assert!(game.state().managers.is_empty());
    days(&mut game, 1);
    assert_eq!(game.state().managers.len(), 6);
}

#[test]
fn managed_games_replay_and_load_identically() {
    let catalog = Arc::new(test_support::management());
    let setup = |game: &mut Game| {
        let (mine, works) = mine_and_works(game);
        let ids = free(game);
        game.apply(Command::HireManager {
            manager: ids[0],
            position: head(mine),
        })
        .unwrap();
        game.apply(Command::HireManager {
            manager: ids[1],
            position: specialist(works, "produktion"),
        })
        .unwrap();
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
    play(&mut loaded);
    assert_eq!(loaded.state_hash(), a.state_hash());

    let replayed = Game::replay(catalog, a.state().settings.clone(), a.journal()).unwrap();
    assert_eq!(replayed.state_hash(), a.state_hash());
}

#[test]
fn skills_show_in_five_levels() {
    assert_eq!(shown_level(0, 0), 0);
    assert_eq!(shown_level(19, 0), 0);
    assert_eq!(shown_level(20, 0), 1);
    assert_eq!(shown_level(59, 0), 2);
    assert_eq!(shown_level(60, 0), 3);
    assert_eq!(shown_level(80, 0), 4);
    assert_eq!(shown_level(100, 15), 4);
    assert_eq!(shown_level(70, 15), 4, "the impression shifts the level");
    assert_eq!(shown_level(10, -15), 0);
}

// MA2: budgets and concerns.

/// A works with three furnaces for a market that takes a small part of their iron, with
/// ore for months and a head who notices everything and judges well.
fn weak_works(catalog: Catalog) -> (Game, SiteId, ManagerId) {
    let mut game = new_game(catalog);
    let (mine, works) = mine_and_works(&mut game);
    let c = game.catalog().clone();
    game.apply(Command::BuildFacility {
        site: works,
        facility: c.facilities.id("ofen").unwrap(),
        count: 2,
        size: crate::catalog::FacilitySize::Medium,
    })
    .unwrap();
    days(&mut game, 40);
    let ore = c.products.id("erz").unwrap();
    let iron = c.products.id("eisen").unwrap();
    game.apply(Command::TransferGoods {
        from: mine,
        to: works,
        product: ore,
        quantity: 900.0,
    })
    .unwrap();
    // Ore for months (without value: no booking), so that purchasing stays quiet.
    game.state_mut().sites[works.index()]
        .inventory
        .get_mut(&ore)
        .unwrap()
        .quantity += 100_000.0;
    for slot in 0..2 {
        game.apply(Command::SetProduction {
            site: works,
            slot,
            recipe: c.recipes.id("eisen_schmelzen"),
            utilization: 1.0,
        })
        .unwrap();
    }
    game.apply(Command::SetSale {
        site: works,
        product: iron,
        mode: Some(crate::state::PriceMode::Market {
            markup: 0.0,
            floor: Money::ZERO,
        }),
        keep: 0.0,
    })
    .unwrap();
    let head_id = free(&game)[0];
    game.apply(Command::HireManager {
        manager: head_id,
        position: head(works),
    })
    .unwrap();
    let m = game.state_mut().managers.get_mut(&head_id).unwrap();
    m.detection = 100;
    m.judgment = 100;
    for v in m.expertise.values_mut() {
        *v = 100;
    }
    (game, works, head_id)
}

/// Units of the works standing still.
fn mothballed(game: &Game, works: SiteId) -> u32 {
    game.state().sites[works.index()]
        .slots
        .iter()
        .filter(|sl| sl.mothballed())
        .map(|sl| sl.count)
        .sum()
}

/// Advances to the last day of March and through it (the end of the quarter).
fn through_quarter_end(game: &mut Game) {
    while game.date() < Date::new(1900, 4, 1).unwrap() {
        days(game, 1);
    }
}

/// Cheap furnaces: shutting two down stays within the floor of the budget.
fn cheap_furnaces() -> Catalog {
    let mut c = test_support::management();
    let ofen = c.facilities.id("ofen").unwrap();
    c.facilities.get_mut(ofen).investment = usd(1_000.0);
    c
}

#[test]
fn with_a_budget_the_head_shuts_down_a_weak_facility() {
    let (mut game, works, _) = weak_works(cheap_furnaces());
    assert_eq!(mothballed(&game, works), 0);
    through_quarter_end(&mut game);
    assert!(
        mothballed(&game, works) > 0,
        "the head shut units down within its budget"
    );
    assert!(game.state().concerns.is_empty());
    let ps = management::position_state(game.state(), game.player(), &head(works)).unwrap();
    assert!(
        ps.spent > Money::ZERO,
        "the book value counts against the budget"
    );
    assert!(
        ps.log
            .iter()
            .any(|l| l.topic == crate::decision::Topic::Overcapacity)
    );
    let ledger = &game.state().company(game.player()).unwrap().ledger;
    assert!(ledger.is_balanced());
}

#[test]
fn without_a_budget_a_concern_asks_and_its_deadline_changes_nothing() {
    let (mut game, works, _) = weak_works(cheap_furnaces());
    game.apply(Command::SetBudget {
        position: head(works),
        shares: Some((0.0, 0.0)),
    })
    .unwrap();
    through_quarter_end(&mut game);
    assert_eq!(mothballed(&game, works), 0, "nothing without an answer");
    let concerns = &game.state().concerns;
    assert_eq!(concerns.len(), 1, "{concerns:?}");
    let c = &concerns[0];
    assert_eq!(c.decision.topic, crate::decision::Topic::Overcapacity);
    assert_eq!(c.status, crate::state::ConcernStatus::Open);
    assert_eq!(c.position, head(works));
    assert!(c.options.len() >= 2);
    assert!(c.options[c.recommended].amount > Money::ZERO);
    assert_eq!(c.deadline, c.created.add_days(30));
    // The deadline passes: nothing changes.
    days(&mut game, 31);
    let c = &game.state().concerns[0];
    assert_eq!(c.status, crate::state::ConcernStatus::Expired);
    assert_eq!(mothballed(&game, works), 0);
}

#[test]
fn answers_carry_out_mute_or_decline() {
    let catalog = cheap_furnaces();
    let ask = |game: &mut Game, works: SiteId| {
        game.apply(Command::SetBudget {
            position: head(works),
            shares: Some((0.0, 0.0)),
        })
        .unwrap();
        through_quarter_end(game);
        game.state().concerns[0].id
    };
    // An option chosen is carried out at once.
    let (mut game, works, _) = weak_works(catalog.clone());
    let id = ask(&mut game, works);
    let c = game.state().concerns[0].clone();
    let shut = c
        .decision
        .choices
        .iter()
        .position(|o| o.kind == crate::decision::ChoiceKind::Mothball)
        .unwrap();
    game.apply(Command::AnswerConcern {
        concern: id,
        answer: management::ConcernAnswer::Choose(shut),
    })
    .unwrap();
    assert!(mothballed(&game, works) > 0);
    assert_eq!(
        game.state().concerns[0].status,
        crate::state::ConcernStatus::Chosen(shut)
    );
    assert_eq!(
        game.apply(Command::AnswerConcern {
            concern: id,
            answer: management::ConcernAnswer::Delegate,
        }),
        Err(CommandError::ConcernClosed)
    );
    assert_eq!(
        game.apply(Command::AnswerConcern {
            concern: 99,
            answer: management::ConcernAnswer::Delegate,
        }),
        Err(CommandError::UnknownConcern)
    );

    // "Decide yourself": the recommendation.
    let (mut game, works, _) = weak_works(catalog.clone());
    let id = ask(&mut game, works);
    let recommended = game.state().concerns[0].recommended;
    game.apply(Command::AnswerConcern {
        concern: id,
        answer: management::ConcernAnswer::Delegate,
    })
    .unwrap();
    assert_eq!(
        game.state().concerns[0].status,
        crate::state::ConcernStatus::Delegated(recommended)
    );

    // Muted: no more questions on the topic, also at the next quarter.
    let topic = crate::decision::Topic::Overcapacity;
    let (mut game, works, _) = weak_works(catalog.clone());
    let id = ask(&mut game, works);
    game.apply(Command::AnswerConcern {
        concern: id,
        answer: management::ConcernAnswer::NeverAsk,
    })
    .unwrap();
    assert_eq!(
        game.state().concerns[0].status,
        crate::state::ConcernStatus::Muted
    );
    let ps = management::position_state(game.state(), game.player(), &head(works)).unwrap();
    assert!(ps.muted.contains(&topic));
    while game.date() < Date::new(1900, 7, 1).unwrap() {
        days(&mut game, 1);
    }
    assert_eq!(game.state().concerns.len(), 1);
    assert_eq!(mothballed(&game, works), 0);

    // Declined: the topic rests at the position for the blocking days.
    let (mut game, works, _) = weak_works(catalog);
    let id = ask(&mut game, works);
    let asked = game.date();
    game.apply(Command::AnswerConcern {
        concern: id,
        answer: management::ConcernAnswer::Decline,
    })
    .unwrap();
    assert_eq!(
        game.state().concerns[0].status,
        crate::state::ConcernStatus::Declined
    );
    let ps = management::position_state(game.state(), game.player(), &head(works)).unwrap();
    assert_eq!(ps.blocked.get(&topic), Some(&asked.add_days(90)));
    assert_eq!(mothballed(&game, works), 0);
}

#[test]
fn budgets_follow_the_revenue_with_a_floor_of_salaries() {
    let mut game = new_game(test_support::management());
    let works = found(&mut game, SiteType::Factory);
    let a = free(&game)[0];
    game.apply(Command::HireManager {
        manager: a,
        position: head(works),
    })
    .unwrap();
    let salary = game.state().managers[&a].job.as_ref().unwrap().salary;
    let (c, s, p) = (game.catalog().clone(), game.state(), game.player());
    // Without revenue: the floor of one and three salaries.
    assert_eq!(
        management::budget(&c, s, p, &head(works), salary),
        (salary, salary.scale(3.0))
    );
    // With revenue: 5 % and 10 % of the last twelve months for a head.
    let month = &mut game.state_mut().companies[0].ledger.months;
    month.push(Default::default());
    month
        .last_mut()
        .unwrap()
        .site_revenue
        .insert(works, usd(100_000_000.0));
    let s = game.state();
    assert_eq!(
        management::budget(&c, s, p, &head(works), salary),
        (usd(5_000_000.0), usd(10_000_000.0))
    );
    // Shares set by the player; 0 always asks.
    assert_eq!(
        game.apply(Command::SetBudget {
            position: head(works),
            shares: Some((0.2, 0.1)),
        }),
        Err(CommandError::InvalidShare)
    );
    game.apply(Command::SetBudget {
        position: head(works),
        shares: Some((0.0, 0.0)),
    })
    .unwrap();
    assert_eq!(
        management::budget(&c, game.state(), p, &head(works), salary),
        (Money::ZERO, Money::ZERO)
    );
}

#[test]
fn the_position_reports_the_effect_of_its_decision() {
    let (mut game, works, _) = weak_works(cheap_furnaces());
    through_quarter_end(&mut game);
    assert!(mothballed(&game, works) > 0);
    assert_eq!(game.state().followups.len(), 1);
    let due = game.state().followups[0].due;
    assert_eq!(due, Date::new(1900, 3, 31).unwrap().add_days(91));
    let mut reported = Vec::new();
    while game.date() <= due {
        let report = game.advance(RoundLength::Day, |_| {});
        reported.extend(
            report
                .messages
                .into_iter()
                .filter(|m| m.key == crate::message::keys::CONCERN_EFFECT),
        );
    }
    assert_eq!(reported.len(), 1, "one report on the effect");
    assert!(game.state().followups.is_empty());
}

#[test]
fn an_idle_laboratory_gets_its_next_target_from_its_head() {
    use crate::catalog::{Recipe, Technology};
    let mut catalog = test_support::management();
    // A better way to make iron, to be researched.
    let metal = catalog.specializations.id("metall").unwrap();
    let smelting = catalog.technologies.id("schmelzen").unwrap();
    let casting = catalog
        .technologies
        .insert(
            "giessen",
            Technology {
                field: metal,
                // Invented after the start: not known to everybody.
                invention_year: 1901,
                prerequisites: vec![smelting],
                research_effort: Some(300.0),
                provenance: Default::default(),
            },
        )
        .unwrap();
    let base = catalog.recipes.id("eisen_schmelzen").unwrap();
    let recipe = Recipe {
        technology: Some(casting),
        ..catalog.recipes.get(base).clone()
    };
    catalog.recipes.insert("eisen_giessen", recipe);
    let mut game = new_game(catalog);
    let (_, works) = mine_and_works(&mut game);
    game.apply(Command::SetProduction {
        site: works,
        slot: 0,
        recipe: game.catalog().recipes.id("eisen_schmelzen"),
        utilization: 1.0,
    })
    .unwrap();
    let lab = found(&mut game, SiteType::ResearchCenter);
    game.apply(Command::BuildFacility {
        site: lab,
        facility: game.catalog().facilities.id("labor").unwrap(),
        count: 1,
        size: crate::catalog::FacilitySize::Medium,
    })
    .unwrap();
    let a = free(&game)[0];
    game.apply(Command::HireManager {
        manager: a,
        position: head(lab),
    })
    .unwrap();
    let m = game.state_mut().managers.get_mut(&a).unwrap();
    m.detection = 100;
    for v in m.expertise.values_mut() {
        *v = 100;
    }
    // The laboratory is built in ten days; the head looks at it every week.
    days(&mut game, 24);
    assert_eq!(
        game.state().sites[lab.index()].research,
        game.catalog().technologies.id("giessen"),
        "the head of the laboratory chose the technology of the own branch"
    );
}

/// A works whose head always asks, with its concern about the overcapacity at the end of
/// the first quarter.
fn asked(catalog: Catalog) -> (Game, SiteId) {
    let (mut game, works, _) = weak_works(catalog);
    game.apply(Command::SetBudget {
        position: head(works),
        shares: Some((0.0, 0.0)),
    })
    .unwrap();
    through_quarter_end(&mut game);
    assert_eq!(game.state().concerns.len(), 1);
    (game, works)
}

/// The slot an option of the open concern acts on.
fn concern_slot(game: &Game, kind: crate::decision::ChoiceKind) -> usize {
    let c = &game.state().concerns[0];
    let option = c.decision.choices.iter().find(|o| o.kind == kind).unwrap();
    match option.steps[0].command {
        Command::MothballFacility { slot, .. } | Command::SellFacility { slot, .. } => slot,
        ref other => panic!("{other:?}"),
    }
}

#[test]
fn concerns_tell_why_the_position_asks() {
    use crate::state::ConcernReason;
    let (game, _) = asked(cheap_furnaces());
    assert_eq!(game.state().concerns[0].reason, ConcernReason::Always);
    // With its default budget the head asks only when the furnaces cost more than it may
    // spend on one decision.
    let (mut game, _, _) = weak_works(test_support::management());
    through_quarter_end(&mut game);
    let concerns = &game.state().concerns;
    assert_eq!(concerns.len(), 1, "{concerns:?}");
    assert_eq!(concerns[0].reason, ConcernReason::Decision);
}

#[test]
fn a_decision_of_the_player_settles_the_concern() {
    use crate::decision::ChoiceKind;
    use crate::state::ConcernStatus;
    let (mut game, works) = asked(cheap_furnaces());
    // Something else at the works leaves the concern open.
    game.apply(Command::SetWagePremium {
        site: works,
        premium: 0.1,
    })
    .unwrap();
    assert_eq!(game.state().concerns[0].status, ConcernStatus::Open);
    // The player shuts a unit of the same facility down in the plant view.
    let slot = concern_slot(&game, ChoiceKind::Mothball);
    game.apply(Command::MothballFacility {
        site: works,
        slot,
        count: 1,
    })
    .unwrap();
    let c = &game.state().concerns[0];
    assert_eq!(c.status, ConcernStatus::Settled);
    assert_eq!(c.closed, Some(game.date()));
    assert_eq!(
        game.apply(Command::AnswerConcern {
            concern: c.id,
            answer: management::ConcernAnswer::Delegate,
        }),
        Err(CommandError::ConcernClosed)
    );
    assert_eq!(crate::views::overview(&game).concerns_open, 0);
}

#[test]
fn a_muted_topic_can_be_asked_about_again() {
    let topic = crate::decision::Topic::Overcapacity;
    let (mut game, works) = asked(cheap_furnaces());
    let id = game.state().concerns[0].id;
    game.apply(Command::AnswerConcern {
        concern: id,
        answer: management::ConcernAnswer::NeverAsk,
    })
    .unwrap();
    let quiet = |game: &Game| {
        management::position_state(game.state(), game.player(), &head(works))
            .unwrap()
            .muted
            .contains(&topic)
    };
    assert!(quiet(&game));
    assert_eq!(
        game.apply(Command::AskAgain {
            position: specialist(works, "forschung"),
            topic,
        }),
        Err(CommandError::UnknownPosition)
    );
    game.apply(Command::AskAgain {
        position: head(works),
        topic,
    })
    .unwrap();
    assert!(!quiet(&game));
    // At the end of the next quarter it asks again.
    while game.date() < Date::new(1900, 7, 1).unwrap() {
        days(&mut game, 1);
    }
    let open = game
        .state()
        .concerns
        .iter()
        .filter(|c| c.status == crate::state::ConcernStatus::Open)
        .count();
    assert_eq!(open, 1);
}

#[test]
fn the_views_show_concerns_budgets_and_decisions() {
    use crate::views;
    let (mut game, works) = asked(cheap_furnaces());
    // A second concern alike (as from another site) joins the first in a group.
    let mut twin = game.state().concerns[0].clone();
    twin.id = 77;
    game.state_mut().concerns.push(twin);
    let v = views::concerns(&game);
    assert_eq!(v.open.len(), 1);
    let g = &v.open[0];
    assert_eq!(g.topic, "ueberkapazitaet");
    assert_eq!(g.concerns.len(), 2);
    let c = &g.concerns[0];
    assert_eq!(c.reason, "immer");
    assert_eq!(c.status, "offen");
    assert_eq!(c.role, "leitung");
    assert_eq!(g.kind, c.options[c.recommended].kind);
    let shut = c.options.iter().find(|o| o.kind == "stilllegen").unwrap();
    assert_eq!(shut.steps[0].key, crate::message::keys::STEP_MOTHBALL);
    assert!(shut.amount_usd > 0.0);
    assert!(c.important, "overcapacity is beyond the routine");
    assert_eq!(views::overview(&game).concerns_open, 2);
    // Answered, it moves to the closed ones.
    game.apply(Command::AnswerConcern {
        concern: 77,
        answer: management::ConcernAnswer::Decline,
    })
    .unwrap();
    let v = views::concerns(&game);
    assert_eq!(v.open[0].concerns.len(), 1);
    assert_eq!(v.closed.len(), 1);
    assert_eq!(v.closed[0].status, "abgelehnt");
    // The position: its budget of nothing, and the declined topic resting.
    let org = views::organisation(&game);
    let site = org.continents[0].countries[0]
        .sites
        .iter()
        .find(|s| s.site == works.0)
        .unwrap();
    let p = &site.positions[0];
    let b = p.budget.as_ref().unwrap();
    assert!(b.custom);
    assert_eq!(b.per_decision_usd, 0.0);
    assert_eq!(p.open_concerns, 1);
    assert_eq!(p.quiet.len(), 1);
    assert_eq!(p.quiet[0].topic, "ueberkapazitaet");
    assert!(p.quiet[0].until.is_some());
}

#[test]
fn a_position_takes_the_hints_of_its_functions() {
    use crate::message::keys;
    use crate::views;
    let mut game = new_game(test_support::management());
    let (_, works) = mine_and_works(&mut game);
    // The furnace is ready and stands still without a recipe.
    days(&mut game, 40);
    let idle = |game: &Game| {
        views::overview(game)
            .hints
            .iter()
            .any(|h| h.site == Some(works.0) && h.message.key == keys::HINT_NO_RECIPE)
    };
    assert!(idle(&game), "a furnace without a recipe");
    let a = free(&game)[0];
    game.apply(Command::HireManager {
        manager: a,
        position: head(works),
    })
    .unwrap();
    // The same state: the hint is gone at once, before the head's first check.
    assert!(!idle(&game), "the head takes care of production");
}

#[test]
fn games_with_concerns_load_identically() {
    let catalog = Arc::new(cheap_furnaces());
    let (mut a, _) = asked((*catalog).clone());
    let id = a.state().concerns[0].id;
    a.apply(Command::AnswerConcern {
        concern: id,
        answer: management::ConcernAnswer::Decline,
    })
    .unwrap();
    let mut loaded = save::decode(&save::encode(&a), catalog).unwrap().game;
    assert_eq!(loaded.state_hash(), a.state_hash());
    for game in [&mut a, &mut loaded] {
        for _ in 0..4 {
            game.advance(RoundLength::Month, |_| {});
        }
    }
    assert_eq!(loaded.state_hash(), a.state_hash());
    assert!(a.state().companies[0].ledger.is_balanced());
}
