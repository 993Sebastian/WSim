//! Tests for projects per laboratory (LB1).

use crate::calendar::RoundLength;
use crate::catalog::{FacilitySize, SiteType, test_support};
use crate::command::{Command, LabTarget};
use crate::game::Game;
use crate::state::SiteId;

fn game_with_labs(labs: usize) -> (Game, SiteId) {
    let mut game = crate::research_tests::new_game(test_support::research());
    let c = game.catalog().clone();
    game.apply(Command::FoundSite {
        country: c.countries.id("AAA").unwrap(),
        kind: SiteType::ResearchCenter,
    })
    .unwrap();
    let site = SiteId(u32::try_from(game.state().sites.len() - 1).unwrap());
    for slot in 0..labs {
        game.apply(Command::BuildFacility {
            site,
            facility: c.facilities.id("labor").unwrap(),
            count: 1,
            size: FacilitySize::Medium,
        })
        .unwrap();
        game.apply(Command::SetProduction {
            site,
            slot,
            recipe: None,
            utilization: 1.0,
        })
        .unwrap();
    }
    (game, site)
}

/// Research points of the player's company for the turbine after `days`.
fn points_after(game: &mut Game, days: usize) -> f64 {
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    for _ in 0..days {
        game.advance(RoundLength::Day, |_| {});
    }
    let player = game.player();
    game.state().companies[player.index()]
        .research
        .get(&turbine)
        .copied()
        .unwrap_or(0.0)
}

fn lab_work(game: &mut Game, site: SiteId, slot: usize, turbine: bool) {
    let t = game.catalog().technologies.id("turbine").unwrap();
    game.apply(Command::SetLabWork {
        site,
        slot,
        work: turbine.then_some(LabTarget::Technology(t)),
    })
    .unwrap();
}

#[test]
fn labs_with_the_same_project_add_their_points() {
    let t = |site: SiteId, game: &mut Game| {
        let turbine = game.catalog().technologies.id("turbine").unwrap();
        game.apply(Command::SetResearch {
            site,
            technology: Some(turbine),
        })
        .unwrap();
    };
    // Two labs on the center's project.
    let (mut game, site) = game_with_labs(2);
    t(site, &mut game);
    let center = points_after(&mut game, 25);
    // Two labs, each with the project of its own: the same points.
    let (mut game, site) = game_with_labs(2);
    lab_work(&mut game, site, 0, true);
    lab_work(&mut game, site, 1, true);
    let own = points_after(&mut game, 25);
    assert!(center > 0.0, "the center researches");
    assert!(
        (own - center).abs() <= center * 1e-9,
        "own {own} against center {center}"
    );
    // One lab of the center and one with the project of its own: the same again.
    let (mut game, site) = game_with_labs(2);
    t(site, &mut game);
    lab_work(&mut game, site, 1, true);
    let mixed = points_after(&mut game, 25);
    assert!(
        (mixed - center).abs() <= center * 1e-9,
        "mixed {mixed} against center {center}"
    );
}

#[test]
fn a_lab_without_a_project_waits_while_the_other_works() {
    // Two labs; only the first has a project, the center has none: half the posts.
    let (mut game, site) = game_with_labs(2);
    lab_work(&mut game, site, 0, true);
    let one = points_after(&mut game, 25);
    // Two labs, both with the project.
    let (mut game, site) = game_with_labs(2);
    lab_work(&mut game, site, 0, true);
    lab_work(&mut game, site, 1, true);
    let two = points_after(&mut game, 25);
    assert!(one > 0.0);
    assert!(
        two > one * 1.5,
        "a second lab with the same project adds its points: {one} → {two}"
    );
}

#[test]
fn a_finished_project_frees_the_lab() {
    let (mut game, site) = game_with_labs(1);
    lab_work(&mut game, site, 0, true);
    let turbine = game.catalog().technologies.id("turbine").unwrap();
    for _ in 0..400 {
        game.advance(RoundLength::Day, |_| {});
        if game.state().knows(game.catalog(), game.player(), turbine) {
            break;
        }
    }
    game.advance(RoundLength::Day, |_| {});
    assert!(game.state().knows(game.catalog(), game.player(), turbine));
    assert!(game.state().sites[site.index()].slots[0].work.is_none());
}

#[test]
fn the_view_shows_the_project_of_each_lab() {
    let (mut game, site) = game_with_labs(2);
    lab_work(&mut game, site, 1, true);
    let view = crate::views::research_overview(&game);
    let center = view
        .centers
        .iter()
        .find(|c| c.site == site.0)
        .expect("center");
    assert!(center.labs[0].work.is_none());
    let work = center.labs[1].work.as_ref().expect("own project");
    assert_eq!(
        (work.kind.as_str(), work.key.as_str()),
        ("technologie", "turbine")
    );
    assert!(
        (center.labs[0].share + center.labs[1].share - 1.0).abs() < 1e-9
            || center.labs[0].share == 0.0
    );
}
