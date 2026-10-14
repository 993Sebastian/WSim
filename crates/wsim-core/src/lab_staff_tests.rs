//! Tests for the staff of the laboratories (PD1) with `test_support::research` and the
//! roles of the data switched on.

use crate::calendar::RoundLength;
use crate::catalog::{Catalog, LabRole, LabStaffModel, SiteType, test_support};
use crate::command::{Command, CommandError};
use crate::lab_staff;
use crate::message::keys;
use crate::research_tests::new_game;
use crate::state::SiteId;

fn with_roles() -> Catalog {
    let mut c = test_support::research();
    c.research_model.lab_staff = LabStaffModel {
        roles: vec![
            (LabRole::Developer, 1.0),
            (LabRole::Technician, 0.5),
            (LabRole::ProductManager, 0.0),
            (LabRole::ProductionPlanner, 0.0),
        ],
    };
    c
}

fn center(game: &mut crate::game::Game) -> SiteId {
    let c = game.catalog().clone();
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
        size: crate::catalog::FacilitySize::Medium,
    })
    .unwrap();
    game.apply(Command::SetProduction {
        site,
        slot: 0,
        recipe: None,
        utilization: 1.0,
    })
    .unwrap();
    site
}

fn days(game: &mut crate::game::Game, n: u32) {
    for _ in 0..n {
        game.advance(RoundLength::Day, |_| {});
    }
}

#[test]
fn people_are_hired_up_to_the_posts_and_dismissed() {
    let mut game = new_game(with_roles());
    let site = center(&mut game);
    // The laboratory is ready after a few days.
    days(&mut game, 15);
    let date = game.date();
    let posts = lab_staff::capacity(game.catalog(), &game.state().sites[site.index()], date);
    assert!(posts >= 1.0, "posts {posts}");
    let posts = posts.floor() as u32;
    assert!(game.state().sites[site.index()].lab_staff.is_empty());
    game.apply(Command::HireLabStaff {
        site,
        role: LabRole::Developer,
        count: posts,
    })
    .unwrap();
    assert_eq!(
        game.apply(Command::HireLabStaff {
            site,
            role: LabRole::Technician,
            count: 1,
        }),
        Err(CommandError::LabFull)
    );
    game.apply(Command::DismissLabStaff {
        site,
        role: LabRole::Developer,
        count: 2,
    })
    .unwrap();
    assert_eq!(
        game.state().sites[site.index()]
            .lab_staff
            .get(LabRole::Developer),
        posts - 2
    );
    assert_eq!(
        game.apply(Command::DismissLabStaff {
            site,
            role: LabRole::Developer,
            count: posts,
        }),
        Err(CommandError::InvalidQuantity)
    );
    // Not in a works, and not without the roles in the data.
    game.apply(Command::FoundSite {
        country: game.catalog().countries.id("AAA").unwrap(),
        kind: SiteType::Factory,
    })
    .unwrap();
    let works = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    assert_eq!(
        game.apply(Command::HireLabStaff {
            site: works,
            role: LabRole::Developer,
            count: 1,
        }),
        Err(CommandError::WrongSiteType {
            required: SiteType::ResearchCenter
        })
    );
    let mut plain = new_game(test_support::research());
    let site = center(&mut plain);
    assert_eq!(
        plain.apply(Command::HireLabStaff {
            site,
            role: LabRole::Developer,
            count: 1,
        }),
        Err(CommandError::InvalidQuantity)
    );
}

#[test]
fn research_follows_the_hired_people() {
    let finish = |share: f64| -> Option<u32> {
        let mut game = new_game(with_roles());
        let site = center(&mut game);
        days(&mut game, 15);
        let turbine = game.catalog().technologies.id("turbine").unwrap();
        game.apply(Command::SetResearch {
            site,
            technology: Some(turbine),
        })
        .unwrap();
        let posts = lab_staff::capacity(
            game.catalog(),
            &game.state().sites[site.index()],
            game.date(),
        );
        let hire = (posts * share).floor() as u32;
        if hire > 0 {
            game.apply(Command::HireLabStaff {
                site,
                role: LabRole::Developer,
                count: hire,
            })
            .unwrap();
        }
        for day in 0..1500 {
            let report = game.advance(RoundLength::Day, |_| {});
            if report.messages.iter().any(|m| m.key == keys::RESEARCH_DONE) {
                return Some(day);
            }
        }
        None
    };
    assert_eq!(
        finish(0.0),
        None,
        "a laboratory without people does not research"
    );
    let few = finish(0.4).expect("a few people finish");
    let many = finish(1.0).expect("a full staff finishes");
    assert!(many < few, "{many} < {few}");
}

#[test]
fn people_without_a_part_in_the_research_cost_wages() {
    let mut game = new_game(with_roles());
    let site = center(&mut game);
    days(&mut game, 15);
    let cost = |game: &mut crate::game::Game| -> f64 {
        days(game, 31);
        game.state().sites[site.index()].workforce.values().sum()
    };
    let none = cost(&mut game);
    game.apply(Command::HireLabStaff {
        site,
        role: LabRole::ProductManager,
        count: 3,
    })
    .unwrap();
    let three = cost(&mut game);
    assert!((three - none - 3.0).abs() < 1e-6, "{none} → {three}");
}
