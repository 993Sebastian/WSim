//! Tests of the development projects PM0–PM7 (PD2) with the research catalog: steel from
//! iron needs the turbine (invented 1902, 300 points).

use crate::calendar::RoundLength;
use crate::catalog::{
    Catalog, LabRole, LabStaffModel, PhaseRole, ProjectLeadModel, ProjectModel, ProjectPhase,
    ProjectRiskModel, ProjectTempo, ProjectTrim, SiteType,
};
use crate::command::{Command, CommandError};
use crate::game::Game;
use crate::ids::{Id, ProductId};
use crate::message::keys;
use crate::product_research;
use crate::projects::{self, ProjectStatus};
use crate::state::SiteId;

fn phase(key: &str, share: (f64, f64), roles: [(f64, f64); 4]) -> ProjectPhase {
    ProjectPhase {
        key: key.into(),
        cost: share.0,
        time: share.1,
        roles: [
            LabRole::ProductManager,
            LabRole::Developer,
            LabRole::ProductionPlanner,
            LabRole::Technician,
        ]
        .into_iter()
        .zip(roles)
        .map(|(role, (need, effect))| PhaseRole { role, need, effect })
        .collect(),
    }
}

fn model() -> ProjectModel {
    // Roles: product manager, developer, production planner, technician (need, effect).
    let r = [(1.0, 0.1), (2.0, 0.6), (1.0, 0.1), (1.0, 0.2)];
    ProjectModel {
        cost_per_point_usd: 100.0,
        reference_rate: 10.0,
        reference_effort: 300.0,
        need_exponent: 0.5,
        over_max: 0.3,
        over_tau: 0.5,
        rest_share: 0.1,
        shortfall_penalty: 0.5,
        shortfall_cap: 0.25,
        spread: 0.1,
        gate_cost: 0.05,
        phases: vec![
            phase("pm0_pm1", (0.05, 0.10), r),
            phase("pm1_pm2", (0.10, 0.10), r),
            phase("pm2_pm3", (0.10, 0.10), r),
            phase("pm4_pm5", (0.30, 0.30), r),
            phase("pm5_pm6", (0.25, 0.25), r),
            phase("pm6_pm7", (0.15, 0.15), r),
        ],
        tempos: vec![
            ProjectTempo {
                key: "gruendlich".into(),
                time: 1.25,
                cost: 1.1,
                risk: 0.7,
            },
            ProjectTempo {
                key: "normal".into(),
                time: 1.0,
                cost: 1.0,
                risk: 1.0,
            },
            ProjectTempo {
                key: "beschleunigt".into(),
                time: 0.8,
                cost: 1.15,
                risk: 1.5,
            },
        ],
        trims: ["basis", "standard", "premium"]
            .into_iter()
            .zip([0.8, 1.0, 1.35])
            .map(|(key, cost)| ProjectTrim {
                key: key.into(),
                cost,
                quality: 0.0,
            })
            .collect(),
        lead: ProjectLeadModel {
            salary_usd_year: 90_000.0,
            capacity_new: 1,
            capacity_further: 2,
            candidates: 3,
            experience_gain: 0.1,
            age_min: 32,
            age_max: 58,
            characters: vec!["optimistisch".into(), "vorsichtig".into()],
        },
        risk: ProjectRiskModel {
            base: 0.02,
            shortfall: 1.5,
            experience: 0.4,
            branch: 0.3,
            buffer: 0.5,
            fast_track: 1.0,
        },
        reports: None,
    }
}

fn catalog() -> Catalog {
    let mut c = product_research_tests_catalog();
    c.research_model.lab_staff = LabStaffModel {
        roles: vec![
            (LabRole::Developer, 1.0),
            (LabRole::Technician, 0.5),
            (LabRole::ProductManager, 0.0),
            (LabRole::ProductionPlanner, 0.0),
        ],
    };
    c.research_model.projects = Some(model());
    c
}

fn product_research_tests_catalog() -> Catalog {
    crate::product_research_tests::catalog()
}

fn lab(game: &mut Game) -> SiteId {
    crate::product_research_tests::research_center(game)
}

fn days(game: &mut Game, n: u32) -> Vec<crate::message::Message> {
    let mut news = Vec::new();
    for _ in 0..n {
        news.extend(game.advance(RoundLength::Day, |_| {}).messages);
    }
    news
}

fn steel(game: &Game) -> ProductId {
    game.catalog().products.id("stahl").unwrap()
}

/// Lab with all posts filled by developers, a hired leader, a project for steel.
fn running(game: &mut Game) -> (SiteId, u32) {
    let site = lab(game);
    days(game, 15);
    let posts = crate::lab_staff::capacity(
        game.catalog(),
        &game.state().sites[site.index()],
        game.date(),
    )
    .floor() as u32;
    game.apply(Command::HireLabStaff {
        site,
        role: LabRole::Developer,
        count: posts,
    })
    .unwrap();
    let lead = game
        .state()
        .project_leads
        .iter()
        .position(|l| l.employer.is_none())
        .expect("the market has leaders") as u32;
    game.apply(Command::HireProjectLead { lead }).unwrap();
    game.apply(Command::StartProject {
        site,
        product: steel(game),
        lead,
        tempo: 1,
        trim: 1,
    })
    .unwrap();
    (site, lead)
}

#[test]
fn the_market_has_leaders_and_products_are_not_researched_directly() {
    let mut game = crate::product_research_tests::new_game(catalog());
    let state = game.state();
    assert!(
        state.project_leads.len() >= 3,
        "{}",
        state.project_leads.len()
    );
    let l = &state.project_leads[0];
    assert!(l.employer.is_none() && (0.0..=1.0).contains(&l.experience));
    let site = lab(&mut game);
    assert_eq!(
        game.apply(Command::SetProductResearch {
            site,
            product: Some(steel(&game)),
        }),
        Err(CommandError::UseProjects)
    );
    // No leader of the company yet.
    assert_eq!(
        game.apply(Command::StartProject {
            site,
            product: steel(&game),
            lead: 0,
            tempo: 1,
            trim: 1,
        }),
        Err(CommandError::LeadUnavailable)
    );
}

#[test]
fn a_project_runs_through_the_milestones_and_releases_the_product() {
    let mut game = crate::product_research_tests::new_game(catalog());
    let (_, lead) = running(&mut game);
    let player = game.player();
    let before = game.state().companies[player.index()].ledger.cash();
    let news = days(&mut game, 1500);
    let milestones: Vec<u32> = news
        .iter()
        .filter(|m| m.key == keys::PROJECT_MILESTONE)
        .map(
            |m| match m.params.iter().find(|(k, _)| k == "meilenstein") {
                Some((_, crate::message::Param::Number(n))) => *n as u32,
                _ => 0,
            },
        )
        .collect();
    assert_eq!(milestones, vec![1, 2, 3, 4, 5, 6, 7]);
    assert!(news.iter().any(|m| m.key == keys::PRODUCT_RESEARCHED));
    let (c, s) = (game.catalog().clone(), game.state());
    assert!(product_research::known(&c, s, player, steel(&game)));
    let p = &s.dev_projects[0];
    assert_eq!(p.status, ProjectStatus::Released);
    assert!(p.finished.is_some() && p.spent.to_usd() > 0.0);
    assert!(s.companies[player.index()].ledger.cash() < before);
    let l = &s.project_leads[lead as usize];
    assert_eq!(l.finished, 1);
    // The leader is free for the next project.
    assert!(
        game.apply(Command::DismissProjectLead { lead }).is_ok(),
        "idle leader can go"
    );
}

#[test]
fn staffing_changes_the_duration_but_not_the_cost() {
    let (m, mut p) = (
        model(),
        crate::projects::DevProject {
            company: crate::state::CompanyId(0),
            site: SiteId(0),
            product: ProductId::from_index(0),
            lead: 0,
            tempo: 1,
            trim: 1,
            started: crate::calendar::Date::first_of_year(1900),
            effort: 400.0,
            phase: 0,
            progress: 0.0,
            wanted: [0; 4],
            assigned: [0; 4],
            auto: true,
            status: ProjectStatus::Running,
            spent: crate::money::Money::ZERO,
            finished: None,
        },
    );
    let normal = projects::scenario(&m, &p, 0.5, 0);
    let more = projects::scenario(&m, &p, 0.5, 2);
    let fewer = projects::scenario(&m, &p, 0.5, -1);
    assert!(more.days < normal.days && normal.days < fewer.days);
    assert!(more.risk <= normal.risk && normal.risk < fewer.risk);
    assert!(normal.penalty == 0.0 && fewer.penalty > 0.0 && fewer.penalty <= m.shortfall_cap);
    // Overstaffing speeds up by at most 30 %.
    assert!(normal.days / more.days <= 1.0 + m.over_max + 1e-9);
    // The experienced leader runs fewer risks.
    let green = projects::scenario(&m, &p, 0.0, 0);
    assert!(projects::scenario(&m, &p, 1.0, 0).risk < green.risk);
    // Costs do not depend on the people.
    p.tempo = 2;
    let cost: f64 = (0..6)
        .map(|k| projects::phase_cost_usd(&m, (2, 1), k, p.effort))
        .sum::<f64>()
        + projects::gate_cost_usd(&m, (2, 1), p.effort);
    assert!((cost - 400.0 * 100.0 * 1.15).abs() < 1e-6, "{cost}");
    // Needs grow with the effort.
    assert!(
        projects::need(&m, 3, LabRole::Developer, 1200.0)
            > projects::need(&m, 3, LabRole::Developer, 300.0)
    );
}

#[test]
fn a_lead_works_on_one_new_product_and_an_aborted_project_frees_him() {
    let mut game = crate::product_research_tests::new_game(catalog());
    let (site, lead) = running(&mut game);
    let other = game
        .catalog()
        .products
        .id("kabel")
        .expect("test catalog has cables");
    // The cable's input does not exist yet; and the leader is busy anyway.
    assert!(
        game.apply(Command::StartProject {
            site,
            product: other,
            lead,
            tempo: 1,
            trim: 1,
        })
        .is_err()
    );
    assert_eq!(
        game.apply(Command::StartProject {
            site,
            product: steel(&game),
            lead,
            tempo: 1,
            trim: 1,
        }),
        Err(CommandError::LeadUnavailable)
    );
    assert_eq!(
        game.apply(Command::DismissProjectLead { lead }),
        Err(CommandError::LeadUnavailable)
    );
    days(&mut game, 20);
    assert!(game.state().dev_projects[0].progress > 0.0 || game.state().dev_projects[0].phase > 0);
    game.apply(Command::AbortProject { project: 0 }).unwrap();
    assert_eq!(game.state().dev_projects[0].status, ProjectStatus::Aborted);
    assert_eq!(game.state().project_leads[lead as usize].aborted, 1);
    game.apply(Command::StartProject {
        site,
        product: steel(&game),
        lead,
        tempo: 0,
        trim: 2,
    })
    .unwrap();
    assert_eq!(game.state().dev_projects.len(), 2);
}

#[test]
fn people_are_assigned_from_the_free_ones_and_can_be_set_by_hand() {
    let mut game = crate::product_research_tests::new_game(catalog());
    let (site, _) = running(&mut game);
    days(&mut game, 2);
    let staff = game.state().sites[site.index()].lab_staff;
    let devs = staff.get(LabRole::Developer);
    let p = &game.state().dev_projects[0];
    assert!(p.auto && p.assigned[LabRole::Developer.index()] <= devs);
    assert_eq!(staff.assigned, p.assigned);
    game.apply(Command::AssignProjectStaff {
        project: 0,
        role: LabRole::Developer,
        count: 1,
    })
    .unwrap();
    days(&mut game, 1);
    let p = &game.state().dev_projects[0];
    assert!(!p.auto);
    assert_eq!(p.assigned[LabRole::Developer.index()], 1);
    // Dismissing a person the project works with is not possible beyond the free ones.
    game.apply(Command::AutoAssignProject { project: 0 })
        .unwrap();
    assert!(game.state().dev_projects[0].auto);
    let _ = SiteType::ResearchCenter;
}

#[test]
fn the_view_shows_projects_leaders_scenarios_and_offers() {
    let mut game = crate::product_research_tests::new_game(catalog());
    // Before any project: leaders on the market, steel on offer, no projects.
    let v = crate::views::research_overview(&game).projects.unwrap();
    assert!(v.leads.len() >= 3 && v.leads.iter().all(|l| !l.hired && !l.busy));
    assert!(v.projects.is_empty());
    let offer = v.offers.iter().find(|o| o.product == "stahl").unwrap();
    assert!(offer.effort > 300.0 && offer.cost_usd > 0.0 && offer.days > 0.0);
    // The cable's input does not exist yet: no offer.
    assert!(v.offers.iter().all(|o| o.product != "kabel"));

    let (site, lead) = running(&mut game);
    days(&mut game, 6);
    let v = crate::views::research_overview(&game).projects.unwrap();
    let p = &v.projects[0];
    assert_eq!(
        (p.product.as_str(), p.status.as_str()),
        ("stahl", "laufend")
    );
    assert_eq!((p.site, p.lead_id), (site.0, lead));
    assert!(p.progress > 0.0 && p.progress < 1.0 && p.days_left > 0.0);
    assert!(p.spent_usd > 0.0 && p.spent_usd < p.budget_usd);
    assert!(p.next_milestone.is_some() && p.milestone <= 4);
    assert_eq!(p.roles.len(), 4);
    assert_eq!(p.scenarios.len(), 5);
    let days_of = |d: i32| p.scenarios.iter().find(|s| s.delta == d).unwrap().days;
    assert!(days_of(2) < days_of(0) && days_of(0) < days_of(-2));
    let l = v.leads.iter().find(|l| l.id == lead).unwrap();
    assert!(l.hired && l.busy);
    // The view is plain data for the interface.
    let json = serde_json::to_string(&v).unwrap();
    assert!(json.contains("\"projects\""));
}
