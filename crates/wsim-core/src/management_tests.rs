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
    assert_eq!(keys.len(), 4 + 5);
    for m in managers.values() {
        assert!(m.job.is_none());
        assert_eq!(
            c.countries.get(m.home).continent,
            c.continents.ids().next().unwrap()
        );
        assert!(c.management.function(&m.focus).is_some());
        assert_eq!(m.expertise.len(), 4);
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
