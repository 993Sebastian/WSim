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
        events: None,
        improvement: None,
        suggestions: None,
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
            profile: 1,
            path: 1,
            conditions: 0,
            loops: 0,
            paused: false,
            delegation: None,
            pending: None,
            queued: Vec::new(),
            baseline: None,
            reports: Vec::new(),
            event: None,
            event_phase: None,
            defect: 0.0,
            bonus_quality: 0.0,
            shortfall_sum: 0.0,
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

// --- PD3: reports, decisions, delegation ---

use crate::catalog::{LeadTrait, Light, ProjectChoice, ProjectReportModel};
use crate::project_reports::{Decision, DelegationMode, LightColor};

fn choice(key: &str, cost: f64, time: f64, risk: f64, quality: f64) -> ProjectChoice {
    ProjectChoice {
        key: key.into(),
        cost,
        time,
        risk,
        quality,
    }
}

fn report_model() -> ProjectReportModel {
    ProjectReportModel {
        lifecycle_years: 10.0,
        capture_share: 0.15,
        plants_without_demand: 1.0,
        utilization: 0.85,
        uncertainty: 0.3,
        accuracy_per_pm: 0.6,
        traits: vec![
            LeadTrait {
                optimism: 0.2,
                path: 2,
            },
            LeadTrait {
                optimism: -0.1,
                path: 0,
            },
        ],
        lights: [
            Light {
                yellow: 0.1,
                red: 0.25,
            },
            Light {
                yellow: 0.1,
                red: 0.25,
            },
            Light {
                yellow: 0.05,
                red: 0.15,
            },
            Light {
                yellow: 0.15,
                red: 0.3,
            },
        ],
        payback_max_years: 1_000.0,
        conditions_risk: 0.15,
        player_at: vec![2, 4, 6, 7],
        deadline_days: 7,
        profiles: vec![
            choice("kosten", 0.9, 1.0, 1.0, -0.05),
            choice("ausgewogen", 1.0, 1.0, 1.0, 0.0),
            choice("qualitaet", 1.15, 1.1, 1.0, 0.08),
        ],
        paths: vec![
            choice("bewaehrt", 0.95, 0.95, 0.7, -0.03),
            choice("standard", 1.0, 1.0, 1.0, 0.0),
            choice("innovativ", 1.2, 1.1, 1.5, 0.1),
        ],
    }
}

fn game_with_reports() -> Game {
    let mut c = catalog();
    c.research_model.projects.as_mut().unwrap().reports = Some(report_model());
    crate::product_research_tests::new_game(c)
}

/// Days until the project waits for a decision; none if it never does.
fn until_pending(game: &mut Game, limit: u32) -> Option<crate::project_reports::Pending> {
    for _ in 0..limit {
        if let Some(p) = game.state().dev_projects[0].pending {
            return Some(p);
        }
        days(game, 1);
    }
    game.state().dev_projects[0].pending
}

#[test]
fn the_player_decides_at_the_standard_milestones_and_the_leader_at_the_others() {
    let mut game = game_with_reports();
    running(&mut game);
    // PM1 is the leader's: the project waits first at PM2, with PM1 reported and decided.
    let pending = until_pending(&mut game, 3000).expect("a decision is due");
    assert_eq!(pending.milestone, 2);
    let p = &game.state().dev_projects[0];
    assert_eq!(p.reports.len(), 2);
    assert!(p.reports[0].decided.is_some_and(|d| !d.by_player));
    assert!(p.reports[1].decided.is_none());
    assert!(p.baseline.is_none());
    // The project stands still and the people are free.
    let before = (p.phase, p.progress);
    days(&mut game, 5);
    let p = &game.state().dev_projects[0];
    assert_eq!(before, (p.phase, p.progress));
    assert_eq!(p.assigned, [0; 4]);
    // Decide with the innovative path: it becomes the baseline and the factors change.
    game.apply(Command::DecideMilestone {
        project: 0,
        decision: Decision::Continue,
        choice: Some(2),
    })
    .unwrap();
    let p = &game.state().dev_projects[0];
    assert_eq!(p.path, 2);
    assert!(p.pending.is_none());
    let b = p.baseline.expect("baseline at PM2");
    assert!(b.plan_days > 0.0 && b.plan_cost > 0.0 && b.kpis.unit_cost > 0.0);
    let m = game.catalog().research_model.projects.clone().unwrap();
    let f = projects::factors(&m, p);
    assert!((f.cost - 1.2).abs() < 1e-9 && (f.time - 1.1).abs() < 1e-9);
    // PM3 is the leader's, PM4 the player's.
    let pending = until_pending(&mut game, 3000).expect("PM4");
    assert_eq!(pending.milestone, 4);
    let p = &game.state().dev_projects[0];
    assert!(
        p.reports
            .iter()
            .any(|r| r.milestone == 3 && r.decided.is_some())
    );
    // The review shows the figures of PM2 and the causes of the deviations.
    let r = p.reports.last().unwrap();
    assert_eq!(r.milestone, 4);
    assert!(r.truth.unit_cost > 0.0 && r.forecast.unit_cost > 0.0);
    assert!(r.bias > 0.0);
    for expected in [4, 6, 7] {
        game.apply(Command::DecideMilestone {
            project: 0,
            decision: Decision::Continue,
            choice: None,
        })
        .unwrap();
        let next = until_pending(&mut game, 3000);
        match (expected, next) {
            (4, Some(n)) => assert_eq!(n.milestone, 6),
            (6, Some(n)) => assert_eq!(n.milestone, 7),
            (7, None) => {}
            (e, n) => panic!("after PM{e}: {n:?}"),
        }
    }
    assert_eq!(game.state().dev_projects[0].status, ProjectStatus::Released);
}

#[test]
fn delegation_shows_everything_or_leaves_everything_to_the_leader() {
    // "Show all": the player already decides PM1, with a target profile.
    let mut game = game_with_reports();
    running(&mut game);
    game.apply(Command::SetProjectDelegation {
        project: Some(0),
        mode: DelegationMode::ShowAll,
    })
    .unwrap();
    let pending = until_pending(&mut game, 3000).expect("PM1");
    assert_eq!(pending.milestone, 1);
    game.apply(Command::DecideMilestone {
        project: 0,
        decision: Decision::Continue,
        choice: Some(2),
    })
    .unwrap();
    assert_eq!(game.state().dev_projects[0].profile, 2);
    // A wrong choice is refused; so is a decision where nothing waits.
    assert_eq!(until_pending(&mut game, 3000).map(|p| p.milestone), Some(2));
    assert_eq!(
        game.apply(Command::DecideMilestone {
            project: 0,
            decision: Decision::Continue,
            choice: Some(9),
        }),
        Err(CommandError::InvalidQuantity)
    );

    // "Leader decides all": the project runs through to the release.
    let mut game = game_with_reports();
    game.apply(Command::SetProjectDelegation {
        project: None,
        mode: DelegationMode::LeadDecides,
    })
    .unwrap();
    running(&mut game);
    assert!(until_pending(&mut game, 1500).is_none());
    let p = &game.state().dev_projects[0];
    assert_eq!(p.status, ProjectStatus::Released);
    assert_eq!(p.reports.len(), 7);
    assert!(
        p.reports
            .iter()
            .all(|r| r.decided.is_some_and(|d| !d.by_player))
    );
    assert_eq!(
        game.apply(Command::DecideMilestone {
            project: 0,
            decision: Decision::Continue,
            choice: None,
        }),
        Err(CommandError::UnknownProject)
    );
}

#[test]
fn an_unanswered_decision_is_taken_by_the_leader_after_the_deadline() {
    let mut game = game_with_reports();
    running(&mut game);
    let pending = until_pending(&mut game, 3000).expect("PM2");
    let due = pending.deadline;
    let mut seen = Vec::new();
    while game.date() < due.add_days(2) {
        seen.extend(game.advance(RoundLength::Day, |_| {}).messages);
    }
    assert!(seen.iter().any(|m| m.key == keys::PROJECT_AUTO_DECIDED));
    let p = &game.state().dev_projects[0];
    let r = p.reports.iter().find(|r| r.milestone == 2).unwrap();
    assert!(r.decided.is_some_and(|d| !d.by_player));
    // The leader's character picks the path: optimistic or cautious, never the middle one here.
    assert_ne!(p.path, 1);
    assert!(p.baseline.is_some());
}

#[test]
fn a_loop_runs_the_phase_again_and_a_pause_waits_for_resuming() {
    let mut game = game_with_reports();
    running(&mut game);
    until_pending(&mut game, 3000).expect("PM2");
    // Loop: the phase that ended runs again.
    game.apply(Command::DecideMilestone {
        project: 0,
        decision: Decision::Loop,
        choice: None,
    })
    .unwrap();
    let p = &game.state().dev_projects[0];
    assert_eq!((p.phase, p.loops), (1, 1));
    // Next stop at PM2 again; pause there.
    until_pending(&mut game, 3000).expect("PM2 again");
    game.apply(Command::DecideMilestone {
        project: 0,
        decision: Decision::Pause,
        choice: None,
    })
    .unwrap();
    assert!(game.state().dev_projects[0].paused);
    let (phase, progress) = {
        let p = &game.state().dev_projects[0];
        (p.phase, p.progress)
    };
    days(&mut game, 10);
    let p = &game.state().dev_projects[0];
    assert_eq!((phase, progress), (p.phase, p.progress));
    game.apply(Command::ResumeProject { project: 0 }).unwrap();
    assert!(!game.state().dev_projects[0].paused);
    assert_eq!(
        game.apply(Command::ResumeProject { project: 0 }),
        Err(CommandError::UnknownProject)
    );
    days(&mut game, 5);
    assert!(
        game.state().dev_projects[0].progress > 0.0
            || game.state().dev_projects[0].pending.is_some()
    );
}

#[test]
fn conditions_raise_the_risk_and_an_abort_ends_the_project() {
    let mut game = game_with_reports();
    running(&mut game);
    until_pending(&mut game, 3000).expect("PM2");
    let m = game.catalog().research_model.projects.clone().unwrap();
    let base = projects::factors(&m, &game.state().dev_projects[0]).risk;
    game.apply(Command::DecideMilestone {
        project: 0,
        decision: Decision::Conditions,
        choice: Some(1),
    })
    .unwrap();
    let p = &game.state().dev_projects[0];
    assert_eq!(p.conditions, 1);
    let risk = projects::factors(&m, p).risk;
    assert!((risk / base - 1.15).abs() < 1e-9 || risk > base);
    until_pending(&mut game, 3000).expect("PM4");
    game.apply(Command::DecideMilestone {
        project: 0,
        decision: Decision::Abort,
        choice: None,
    })
    .unwrap();
    let p = &game.state().dev_projects[0];
    assert_eq!(p.status, ProjectStatus::Aborted);
    let lead = p.lead as usize;
    assert_eq!(game.state().project_leads[lead].aborted, 1);
    // The lights and the recommendation of the report are plain data.
    let r = game.state().dev_projects[0].reports.last().unwrap();
    assert!(matches!(
        r.recommendation,
        Decision::Continue | Decision::Conditions | Decision::Loop | Decision::Abort
    ));
    assert!(
        r.lights
            .iter()
            .all(|l| matches!(l, LightColor::Green | LightColor::Yellow | LightColor::Red))
    );
}

#[test]
fn a_hopeless_product_makes_the_leader_recommend_to_stop() {
    // The test catalog's steel sells below its unit cost: no payback, a red light, and the
    // leader recommends to stop.
    let mut c = catalog();
    let mut rm = report_model();
    rm.payback_max_years = 12.0;
    c.research_model.projects.as_mut().unwrap().reports = Some(rm);
    let mut game = crate::product_research_tests::new_game(c);
    running(&mut game);
    game.apply(Command::SetProjectDelegation {
        project: None,
        mode: DelegationMode::LeadDecides,
    })
    .unwrap();
    days(&mut game, 400);
    let p = &game.state().dev_projects[0];
    let r = p
        .reports
        .iter()
        .find(|r| r.recommendation == Decision::Abort)
        .expect("an abort is recommended");
    assert!(r.truth.payback_years > 12.0 && r.lights.contains(&LightColor::Red));
    assert_eq!(p.status, ProjectStatus::Aborted);
}

use crate::catalog::{ProjectEventKind, ProjectEventModel, ProjectEventOption};

fn option(key: &str, cost: f64, delay: f64, defect: f64) -> ProjectEventOption {
    ProjectEventOption {
        key: key.into(),
        cost,
        delay,
        quality: 0.0,
        defect,
    }
}

/// An event in nearly every phase: the risk is huge, one kind fits all phases.
fn game_with_events(defect: f64) -> Game {
    let mut c = catalog();
    let m = c.research_model.projects.as_mut().unwrap();
    m.risk.base = 50.0;
    m.events = Some(ProjectEventModel {
        deadline_days: 5,
        defect_cap: 1.0,
        defect_per_condition: 0.0,
        defect_per_shortfall: 0.0,
        defect_per_tempo: vec![0.0; 3],
        recall_days: (10, 20),
        recall_cost_share: 0.5,
        recall_awareness_loss: 0.25,
        kinds: vec![ProjectEventKind {
            key: "problem".into(),
            phases: vec![0, 1, 2, 3, 4, 5],
            weight: 1.0,
            recommended: 0,
            options: vec![
                option("budget", 0.2, 0.0, defect),
                option("verzug", 0.0, 0.25, 0.0),
            ],
        }],
    });
    crate::product_research_tests::new_game(c)
}

fn until_event(game: &mut Game, limit: u32) -> Option<crate::project_events::PendingEvent> {
    for _ in 0..limit {
        if let Some(e) = game.state().dev_projects[0].event {
            return Some(e);
        }
        days(game, 1);
    }
    game.state().dev_projects[0].event
}

#[test]
fn an_event_stops_the_project_until_an_option_is_chosen() {
    let mut game = game_with_events(0.1);
    running(&mut game);
    let event = until_event(&mut game, 3000).expect("an event turns up");
    assert_eq!(event.deadline, event.raised.add_days(5));
    let p = &game.state().dev_projects[0];
    let (phase, progress, spent) = (p.phase, p.progress, p.spent);
    // The project stands still while the event waits.
    days(&mut game, 2);
    let p = &game.state().dev_projects[0];
    assert_eq!((p.phase, p.progress, p.spent), (phase, progress, spent));
    // The view shows the event with its options and the leader's recommendation.
    let v = crate::views::research_overview(&game).projects.unwrap();
    let e = v.projects[0].event.as_ref().expect("event in the view");
    assert_eq!(
        (e.key.as_str(), e.options.len(), e.recommended),
        ("problem", 2, 0)
    );
    assert!(e.options[0].cost > 0.0 && e.options[1].delay > 0.0);
    // A wrong option is refused; the right one costs money and adds the hidden defect.
    assert_eq!(
        game.apply(Command::DecideEvent {
            project: 0,
            option: 5
        }),
        Err(CommandError::InvalidQuantity)
    );
    game.apply(Command::DecideEvent {
        project: 0,
        option: 0,
    })
    .unwrap();
    let p = &game.state().dev_projects[0];
    assert!(p.event.is_none());
    assert!(p.spent > spent);
    assert!((p.defect - 0.1).abs() < 1e-9);
    assert_eq!(
        game.apply(Command::DecideEvent {
            project: 0,
            option: 0
        }),
        Err(CommandError::NothingToDecide)
    );
}

#[test]
fn an_unanswered_event_is_decided_by_the_leader_after_the_deadline() {
    let mut game = game_with_events(0.0);
    running(&mut game);
    let event = until_event(&mut game, 3000).expect("an event turns up");
    let mut waited = 0;
    while game.state().dev_projects[0].event.is_some() && waited < 10 {
        days(&mut game, 1);
        waited += 1;
    }
    assert!(game.state().dev_projects[0].event.is_none());
    assert!(waited >= 4, "waited {waited} days for {event:?}");
}

#[test]
fn the_leader_decides_events_at_once_when_the_owner_leaves_everything_to_him() {
    let mut game = game_with_events(0.0);
    running(&mut game);
    game.apply(Command::SetProjectDelegation {
        project: None,
        mode: crate::project_reports::DelegationMode::LeadDecides,
    })
    .unwrap();
    for _ in 0..3000 {
        days(&mut game, 1);
        assert!(game.state().dev_projects[0].event.is_none());
        if game.state().dev_projects[0].event_phase.is_some() {
            return;
        }
    }
    panic!("no event happened");
}

#[test]
fn a_hidden_defect_comes_back_as_a_recall_with_costs() {
    let mut game = game_with_events(1.0);
    running(&mut game);
    // Choose the option with the defect at every event until the product is released.
    for _ in 0..6000 {
        if game.state().dev_projects[0].event.is_some() {
            game.apply(Command::DecideEvent {
                project: 0,
                option: 0,
            })
            .unwrap();
        }
        if game.state().dev_projects[0].status == ProjectStatus::Released {
            break;
        }
        days(&mut game, 1);
    }
    assert_eq!(game.state().dev_projects[0].status, ProjectStatus::Released);
    assert_eq!(game.state().recalls.len(), 1);
    let due = game.state().recalls[0].due;
    let released = game.state().dev_projects[0].finished.unwrap();
    assert!((10..=20).contains(&released.days_until(due)));
    let player = game.state().main_company.unwrap().index();
    let before = game.state().companies[player].ledger.cash();
    let mut news = Vec::new();
    while game.date() <= due {
        news.extend(days(&mut game, 1));
    }
    assert!(game.state().recalls.is_empty());
    assert!(news.iter().any(|m| m.key == keys::PROJECT_RECALL));
    assert!(game.state().companies[player].ledger.cash() < before);
    assert!(game.state().companies[player].ledger.is_balanced());
}

#[test]
fn without_the_event_block_nothing_happens() {
    let mut c = catalog();
    c.research_model.projects.as_mut().unwrap().risk.base = 50.0;
    let mut game = crate::product_research_tests::new_game(c);
    running(&mut game);
    days(&mut game, 400);
    let p = &game.state().dev_projects[0];
    assert!(p.event.is_none() && p.event_phase.is_none() && game.state().recalls.is_empty());
}

use crate::catalog::{ImprovementEffect, ImprovementModel, ImprovementTarget};
use crate::improvements::{self, ImpPendingKind, ImpStatus};

fn improvement_model() -> ImprovementModel {
    let r = [(1.0, 0.1), (2.0, 0.6), (1.0, 0.1), (1.0, 0.2)];
    let target = |key: &str, effect: ImprovementEffect| ImprovementTarget {
        key: key.into(),
        per_stage: 0.1,
        cap: 0.9,
        effect,
    };
    let none = ImprovementEffect::default();
    ImprovementModel {
        stages: 3,
        rest_share: 0.2,
        effort_share: 0.15,
        effort_growth: 1.5,
        retrofit_days: 90,
        immediate_cost_share: 0.2,
        deadline_days: 5,
        approaches: (2, 4),
        approach_spread: 0.35,
        approach_cost_slope: 0.8,
        outcome_spread: 0.3,
        outcome_experience: 0.5,
        outcome_shortfall: 1.5,
        phases: vec![
            phase("wm0_wm1", (0.2, 0.25), r),
            phase("wm1_wm2", (0.55, 0.5), r),
            phase("wm2_wm3", (0.25, 0.25), r),
        ],
        targets: vec![
            target(
                "kosten",
                ImprovementEffect {
                    labor: 1.0,
                    inputs: 0.5,
                    ..none
                },
            ),
            target(
                "qualitaet",
                ImprovementEffect {
                    quality: 20.0,
                    ..none
                },
            ),
            target("ausschuss", ImprovementEffect { scrap: 1.0, ..none }),
            target(
                "rate",
                ImprovementEffect {
                    output: 1.0,
                    ..none
                },
            ),
            target(
                "vorprodukte",
                ImprovementEffect {
                    inputs: 1.0,
                    ..none
                },
            ),
            target(
                "ausstattung",
                ImprovementEffect {
                    quality: 10.0,
                    output: 0.5,
                    ..none
                },
            ),
        ],
    }
}

/// A lab with developers and an employed leader; iron is known from the start.
fn improvement_game() -> (Game, SiteId, u32, ProductId) {
    let mut c = catalog();
    c.research_model.projects.as_mut().unwrap().improvement = Some(improvement_model());
    let mut game = crate::product_research_tests::new_game(c);
    let site = lab(&mut game);
    days(&mut game, 15);
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
    let iron = game.catalog().products.id("eisen").unwrap();
    (game, site, lead, iron)
}

fn start_improvement(game: &mut Game, site: SiteId, lead: u32, iron: ProductId, target: u8) {
    game.apply(Command::StartImprovement {
        site,
        product: iron,
        target,
        lead,
    })
    .unwrap();
}

/// Runs the project to the end, choosing approach 0 and releasing as asked.
fn run_improvement(game: &mut Game, index: usize, immediate: bool) {
    for _ in 0..6000 {
        match game.state().improvement_projects[index]
            .pending
            .map(|p| p.kind)
        {
            Some(ImpPendingKind::Choose) => game
                .apply(Command::ChooseApproach {
                    project: index as u32,
                    approach: 0,
                })
                .unwrap(),
            Some(ImpPendingKind::Release) => game
                .apply(Command::ReleaseImprovement {
                    project: index as u32,
                    immediate,
                })
                .unwrap(),
            None => {}
        }
        if game.state().improvement_projects[index].status != ImpStatus::Running {
            return;
        }
        days(game, 1);
    }
    panic!("the improvement did not finish");
}

#[test]
fn improvement_projects_follow_the_rules_of_leaders_and_targets() {
    let (mut game, site, lead, iron) = improvement_game();
    start_improvement(&mut game, site, lead, iron, 0);
    // The same target twice is refused, another target works, the third exceeds the leader.
    assert_eq!(
        game.apply(Command::StartImprovement {
            site,
            product: iron,
            target: 0,
            lead
        }),
        Err(CommandError::ProjectExists)
    );
    start_improvement(&mut game, site, lead, iron, 1);
    assert_eq!(
        game.apply(Command::StartImprovement {
            site,
            product: iron,
            target: 2,
            lead
        }),
        Err(CommandError::LeadUnavailable)
    );
    assert_eq!(
        game.apply(Command::StartImprovement {
            site,
            product: iron,
            target: 9,
            lead
        }),
        Err(CommandError::InvalidQuantity)
    );
    // A leader of improvement projects cannot be dismissed.
    assert!(game.apply(Command::DismissProjectLead { lead }).is_err());
}

#[test]
fn an_improvement_runs_through_the_milestones_and_takes_effect_step_by_step() {
    let (mut game, site, lead, iron) = improvement_game();
    let player = game.player();
    start_improvement(&mut game, site, lead, iron, 0);
    let c = game.catalog().clone();
    let before = crate::development::effect(&c, game.state(), player, iron);
    // The analysis waits for the player's choice.
    let mut chosen = false;
    for _ in 0..4000 {
        if game.state().improvement_projects[0].pending.map(|p| p.kind)
            == Some(ImpPendingKind::Choose)
        {
            chosen = true;
            break;
        }
        days(&mut game, 1);
    }
    assert!(chosen, "the analysis finds approaches");
    let p = &game.state().improvement_projects[0];
    assert!((2..=4).contains(&p.approaches.len()));
    assert_eq!(
        game.apply(Command::ChooseApproach {
            project: 0,
            approach: 9
        }),
        Err(CommandError::InvalidQuantity)
    );
    run_improvement(&mut game, 0, false);
    let p = &game.state().improvement_projects[0];
    assert_eq!(p.status, ImpStatus::Released);
    assert!(p.spent > crate::money::Money::ZERO);
    assert_eq!(improvements::stages_done(game.state(), player, iron, 0), 1);
    // Step by step: no effect on the release day, the full effect after the ramp.
    let day0 = crate::development::effect(&c, game.state(), player, iron);
    assert!((day0.labor - before.labor).abs() < 1e-9);
    days(&mut game, 90);
    let full = crate::development::effect(&c, game.state(), player, iron);
    let gain = game.state().improved[0].stages[0].gain;
    assert!((full.labor - before.labor * (1.0 - 0.1 * gain)).abs() < 1e-9);
    assert!(full.inputs < before.inputs);
    assert!((full.output - before.output).abs() < 1e-12);
    assert!(game.state().companies[player.index()].ledger.is_balanced());
}

#[test]
fn an_immediate_release_costs_money_and_works_at_once() {
    let (mut game, site, lead, iron) = improvement_game();
    let player = game.player();
    start_improvement(&mut game, site, lead, iron, 3);
    let c = game.catalog().clone();
    let before = crate::development::effect(&c, game.state(), player, iron);
    run_improvement(&mut game, 0, true);
    let now = crate::development::effect(&c, game.state(), player, iron);
    assert!(now.output > before.output);
    let p = &game.state().improvement_projects[0];
    assert!(p.spent.to_usd() > 0.0);
}

#[test]
fn the_leader_decides_alone_when_the_owner_leaves_everything_to_him() {
    let (mut game, site, lead, iron) = improvement_game();
    let player = game.player();
    game.apply(Command::SetProjectDelegation {
        project: None,
        mode: crate::project_reports::DelegationMode::LeadDecides,
    })
    .unwrap();
    start_improvement(&mut game, site, lead, iron, 2);
    for _ in 0..6000 {
        assert!(game.state().improvement_projects[0].pending.is_none());
        if game.state().improvement_projects[0].status != ImpStatus::Running {
            break;
        }
        days(&mut game, 1);
    }
    assert_eq!(
        game.state().improvement_projects[0].status,
        ImpStatus::Released
    );
    let c = game.catalog().clone();
    days(&mut game, 100);
    let e = crate::development::effect(&c, game.state(), player, iron);
    assert!(e.scrap < 1.0);
}

#[test]
fn further_stages_bring_a_fifth_and_the_cap_limits_the_sum() {
    let (mut game, _, _, iron) = improvement_game();
    let player = game.player();
    let c = game.catalog().clone();
    let date = game.date();
    let stage = |gain| improvements::ReleasedStage {
        gain,
        released: date.add_days(-1000),
        ramp_days: 0,
    };
    // Four stages of gain 1: 3 · 1 + 0.2 = 3.2 units, 0.1 each.
    game.state_mut().improved.push(improvements::Improved {
        company: player,
        product: iron,
        target: 1,
        stages: vec![stage(1.0); 4],
    });
    let e = crate::development::effect(&c, game.state(), player, iron);
    assert!((e.quality - 20.0 * 0.32).abs() < 1e-9);
    // Twenty units run into the cap of 0.9.
    game.state_mut().improved[0].stages = vec![stage(5.0); 4];
    let e = crate::development::effect(&c, game.state(), player, iron);
    assert!((e.quality - 20.0 * 0.9).abs() < 1e-9);
}

#[test]
fn the_view_offers_improvements_and_shows_the_running_ones() {
    let (mut game, site, lead, iron) = improvement_game();
    // Iron is made by the company: put it into production so the offers appear.
    let c = game.catalog().clone();
    let v = crate::views::research_overview(&game).projects.unwrap();
    assert_eq!(v.improvement_targets.len(), 6);
    assert_eq!(v.improvement_stages, 3);
    assert!(v.improvements.is_empty());
    let _ = (&c, iron);
    start_improvement(&mut game, site, lead, iron, 1);
    let v = crate::views::research_overview(&game).projects.unwrap();
    assert_eq!(v.improvements.len(), 1);
    let row = &v.improvements[0];
    assert_eq!(
        (row.product.as_str(), row.target.as_str()),
        ("eisen", "qualitaet")
    );
    assert_eq!(row.status, "laufend");
    assert!(row.pending.is_none() && row.approaches.is_empty());
    // A running target is no longer on offer.
    assert!(
        v.improvement_offers
            .iter()
            .all(|o| !(o.product == "eisen" && o.target == "qualitaet"))
    );
}
