//! The development projects for the interface (PD2, docs/PRODUKTENTWICKLUNG.md): the
//! company's projects with staffing and scenarios, the project leaders (own and on the
//! market) and the products a project can start for.

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::catalog::LabRole;
use crate::game::Game;
use crate::ids::{Id, ProductId};
use crate::project_reports::{self, DelegationMode, Kpis, LightColor};
use crate::projects::{self, ProjectStatus};

/// A project leader.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectLeadView {
    pub id: u32,
    pub name: String,
    pub home: String,
    pub age: f64,
    /// Character key (`optimistisch`, `vorsichtig` …).
    pub character: String,
    pub experience: f64,
    pub finished: u32,
    pub aborted: u32,
    /// Works for the player's company (else on the market).
    pub hired: bool,
    /// Leads a running project.
    pub busy: bool,
}

/// Staffing of one role in the current phase of a project.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectRoleView {
    pub role: String,
    pub need: u32,
    pub assigned: u32,
    /// People of the role in the laboratory not working on any project.
    pub free: u32,
}

/// What the rest of the project takes with `delta` more or fewer people per role than needed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectScenarioView {
    pub delta: i32,
    pub days: f64,
    pub risk: f64,
    pub penalty: f64,
}

/// A choice at PM1 (target profile) or PM2 (solution path) with its factors.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectChoiceView {
    pub key: String,
    pub cost: f64,
    pub time: f64,
    pub risk: f64,
    pub quality: f64,
    /// The leader's own pick.
    pub preferred: bool,
}

/// A report at a milestone (PD3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MilestoneReportView {
    pub milestone: u8,
    pub date: String,
    /// What the leader shows.
    pub kpis: Kpis,
    /// The figures at PM2 and the relative deviation of the report from them (from PM4 on).
    pub baseline: Option<Kpis>,
    pub deviation: Option<Kpis>,
    /// `gruen`, `gelb` or `rot` for schedule, budget, quality and risk.
    pub lights: Vec<String>,
    /// `weiter`, `auflagen`, `schleife`, `pause` or `abbruch`.
    pub recommendation: String,
    /// Keys of what moved since PM2.
    pub causes: Vec<String>,
    /// How it was decided: key of the decision and whether the player did.
    pub decided: Option<(String, bool)>,
}

/// A decision the project waits for.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingView {
    pub milestone: u8,
    pub deadline: String,
    pub recommendation: String,
    /// Target profiles at PM1, solution paths at PM2.
    pub choices: Vec<ProjectChoiceView>,
    /// What kind the choices are: `profil` or `weg`; empty without choices.
    pub choice_kind: String,
}

/// One option of an event (PD4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventOptionView {
    pub key: String,
    pub cost: f64,
    pub delay: f64,
    pub quality: f64,
}

/// An event the project waits for (PD4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventView {
    pub key: String,
    pub deadline: String,
    pub recommended: usize,
    pub options: Vec<EventOptionView>,
}

/// One project of the company.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectRowView {
    pub id: u32,
    pub product: String,
    pub site: u32,
    pub site_name: Option<String>,
    pub lead: String,
    pub lead_id: u32,
    pub tempo: String,
    pub trim: String,
    /// `laufend`, `freigegeben` or `abgebrochen`.
    pub status: String,
    /// Milestone reached last (0 = started) and the next one.
    pub milestone: u8,
    pub next_milestone: Option<u8>,
    /// Share of the duration done, 0–1.
    pub progress: f64,
    pub started: String,
    pub finished: Option<String>,
    pub days_left: f64,
    /// Chance of a problem over the rest of the project.
    pub risk: f64,
    /// Loss on the figures from understaffing.
    pub penalty: f64,
    pub spent_usd: f64,
    /// Cost of the whole project.
    pub budget_usd: f64,
    /// The project takes the people each phase needs.
    pub auto: bool,
    pub roles: Vec<ProjectRoleView>,
    pub scenarios: Vec<ProjectScenarioView>,
    /// Keys of the chosen target profile and solution path (PD3; empty without reports).
    pub profile: String,
    pub path: String,
    pub conditions: u32,
    pub loops: u32,
    pub paused: bool,
    /// Who decides at the milestones: `alles`, `standard` or `leiter`.
    pub delegation: String,
    pub pending: Option<PendingView>,
    pub reports: Vec<MilestoneReportView>,
    pub event: Option<EventView>,
}

/// A product a project can start for, with cost and duration at normal tempo and standard trim.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectOfferView {
    pub product: String,
    pub effort: f64,
    pub cost_usd: f64,
    pub days: f64,
}

/// The development projects of the player's company.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectsView {
    pub tempos: Vec<String>,
    pub trims: Vec<String>,
    pub salary_usd_year: f64,
    /// The company's setting for who decides at the milestones (PD3).
    pub delegation: String,
    /// Milestones where the player decides under the standard setting.
    pub player_at: Vec<u8>,
    pub leads: Vec<ProjectLeadView>,
    pub projects: Vec<ProjectRowView>,
    pub offers: Vec<ProjectOfferView>,
}

fn delegation_key(d: DelegationMode) -> &'static str {
    match d {
        DelegationMode::ShowAll => "alles",
        DelegationMode::Standard => "standard",
        DelegationMode::LeadDecides => "leiter",
    }
}

fn light_key(l: LightColor) -> &'static str {
    match l {
        LightColor::Green => "gruen",
        LightColor::Yellow => "gelb",
        LightColor::Red => "rot",
    }
}

/// Relative deviation of every figure from the baseline (0 where the baseline is 0).
fn deviation(now: &Kpis, base: &Kpis) -> Kpis {
    let d = |a: f64, b: f64| if b.abs() > 1e-9 { (a - b) / b } else { 0.0 };
    Kpis {
        unit_cost: d(now.unit_cost, base.unit_cost),
        price: d(now.price, base.price),
        volume_year: d(now.volume_year, base.volume_year),
        revenue_year: d(now.revenue_year, base.revenue_year),
        lifecycle_revenue: d(now.lifecycle_revenue, base.lifecycle_revenue),
        contribution_year: d(now.contribution_year, base.contribution_year),
        plants: d(now.plants, base.plants),
        investment: d(now.investment, base.investment),
        project_cost: d(now.project_cost, base.project_cost),
        project_days: d(now.project_days, base.project_days),
        people: d(now.people, base.people),
        payback_years: d(now.payback_years, base.payback_years),
        quality: d(now.quality, base.quality),
    }
}

/// The projects view; none where the data have no development projects.
pub fn projects_view(game: &Game) -> Option<ProjectsView> {
    let catalog = game.catalog();
    let m = catalog.research_model.projects.as_ref()?;
    let state = game.state();
    let player = game.player();
    let busy = |lead: usize| {
        state
            .dev_projects
            .iter()
            .any(|p| p.running() && p.lead as usize == lead)
    };
    let leads = state
        .project_leads
        .iter()
        .enumerate()
        .filter(|(_, l)| l.employer.is_none() || l.employer == Some(player))
        .map(|(i, l)| ProjectLeadView {
            id: u32::try_from(i).unwrap_or(u32::MAX),
            name: l.name.clone(),
            home: catalog.countries.key(l.home).to_owned(),
            age: projects::age(l.born, state.date),
            character: m
                .lead
                .characters
                .get(usize::from(l.character))
                .cloned()
                .unwrap_or_default(),
            experience: l.experience,
            finished: l.finished,
            aborted: l.aborted,
            hired: l.employer == Some(player),
            busy: busy(i),
        })
        .collect();
    let company_delegation = state
        .project_delegation
        .get(&player)
        .copied()
        .unwrap_or(DelegationMode::Standard);
    let projects = state
        .dev_projects
        .iter()
        .enumerate()
        .filter(|(_, p)| p.company == player)
        .map(|(i, p)| {
            let lead = state.project_leads.get(p.lead as usize);
            let experience = lead.map_or(0.0, |l| l.experience);
            let running = p.status == ProjectStatus::Running;
            let phase = usize::from(p.phase).min(5);
            let site = &state.sites[p.site.index()];
            let roles = LabRole::ALL
                .into_iter()
                .map(|role| ProjectRoleView {
                    role: role.key().to_owned(),
                    need: if running {
                        projects::need(m, phase, role, p.effort)
                    } else {
                        0
                    },
                    assigned: p.assigned[role.index()],
                    free: site.lab_staff.free(role),
                })
                .collect();
            let scenarios = if running {
                [-2, -1, 0, 1, 2]
                    .into_iter()
                    .map(|delta| {
                        let s = projects::scenario(m, p, experience, delta);
                        ProjectScenarioView {
                            delta,
                            days: s.days,
                            risk: s.risk,
                            penalty: s.penalty,
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };
            let now = projects::scenario(m, p, experience, 0);
            ProjectRowView {
                id: u32::try_from(i).unwrap_or(u32::MAX),
                product: catalog.products.key(p.product).to_owned(),
                site: u32::try_from(p.site.index()).unwrap_or(u32::MAX),
                site_name: site.name.clone(),
                lead: lead.map(|l| l.name.clone()).unwrap_or_default(),
                lead_id: p.lead,
                tempo: m.tempos[usize::from(p.tempo)].key.clone(),
                trim: m.trims[usize::from(p.trim)].key.clone(),
                status: match p.status {
                    ProjectStatus::Running => "laufend",
                    ProjectStatus::Released => "freigegeben",
                    ProjectStatus::Aborted => "abgebrochen",
                }
                .to_owned(),
                milestone: projects::last_milestone(p),
                next_milestone: (running && phase < 6)
                    .then(|| projects::PHASE_END_MILESTONE[phase]),
                progress: projects::overall_progress(m, p),
                started: iso(p.started),
                finished: p.finished.map(iso),
                days_left: if running {
                    projects::days_left(m, p)
                } else {
                    0.0
                },
                risk: if running { now.risk } else { 0.0 },
                penalty: if running { now.penalty } else { 0.0 },
                spent_usd: usd(p.spent),
                budget_usd: p.effort
                    * m.cost_per_point_usd
                    * m.tempos[usize::from(p.tempo)].cost
                    * m.trims[usize::from(p.trim)].cost,
                auto: p.auto,
                roles,
                scenarios,
                profile: m
                    .reports
                    .as_ref()
                    .and_then(|r| r.profiles.get(usize::from(p.profile)))
                    .map(|c| c.key.clone())
                    .unwrap_or_default(),
                path: m
                    .reports
                    .as_ref()
                    .and_then(|r| r.paths.get(usize::from(p.path)))
                    .map(|c| c.key.clone())
                    .unwrap_or_default(),
                conditions: p.conditions,
                loops: p.loops,
                paused: p.paused,
                delegation: delegation_key(p.delegation.unwrap_or(company_delegation)).to_owned(),
                pending: pending_view(m, state, p, lead),
                event: event_view(catalog, p),
                reports: p
                    .reports
                    .iter()
                    .map(|r| MilestoneReportView {
                        milestone: r.milestone,
                        date: iso(r.date),
                        kpis: r.forecast,
                        baseline: p.baseline.map(|b| b.kpis).filter(|_| r.milestone >= 3),
                        deviation: p
                            .baseline
                            .filter(|_| r.milestone >= 4)
                            .map(|b| deviation(&r.forecast, &b.kpis)),
                        lights: r.lights.iter().map(|&l| light_key(l).to_owned()).collect(),
                        recommendation: project_reports::decision_key(r.recommendation).to_owned(),
                        causes: r.causes.clone(),
                        decided: r.decided.map(|d| {
                            (
                                project_reports::decision_key(d.decision).to_owned(),
                                d.by_player,
                            )
                        }),
                    })
                    .collect(),
            }
        })
        .collect();
    let offers = (0..catalog.products.len())
        .map(ProductId::from_index)
        .filter(|&product| {
            !state
                .dev_projects
                .iter()
                .any(|p| p.running() && p.company == player && p.product == product)
        })
        .filter_map(|product| {
            projects::project_effort(catalog, state, player, product, state.date).map(|effort| {
                ProjectOfferView {
                    product: catalog.products.key(product).to_owned(),
                    effort,
                    cost_usd: effort * m.cost_per_point_usd,
                    days: effort / m.reference_rate,
                }
            })
        })
        .collect();
    Some(ProjectsView {
        tempos: m.tempos.iter().map(|t| t.key.clone()).collect(),
        trims: m.trims.iter().map(|t| t.key.clone()).collect(),
        salary_usd_year: m.lead.salary_usd_year,
        delegation: delegation_key(company_delegation).to_owned(),
        player_at: m
            .reports
            .as_ref()
            .map(|r| r.player_at.clone())
            .unwrap_or_default(),
        leads,
        projects,
        offers,
    })
}

/// The decision a project waits for, with the choices of PM1 and PM2.
fn pending_view(
    m: &crate::catalog::ProjectModel,
    state: &crate::state::GameState,
    p: &projects::DevProject,
    lead: Option<&projects::ProjectLead>,
) -> Option<PendingView> {
    let pending = p.pending?;
    let rm = m.reports.as_ref()?;
    let (kind, list, preferred) = match pending.milestone {
        1 => ("profil", &rm.profiles, 1),
        2 => (
            "weg",
            &rm.paths,
            lead.and_then(|l| rm.traits.get(usize::from(l.character)))
                .map_or(1, |t| t.path),
        ),
        _ => ("", &rm.profiles, 0),
    };
    let _ = state;
    Some(PendingView {
        milestone: pending.milestone,
        deadline: iso(pending.deadline),
        recommendation: project_reports::decision_key(pending.recommendation).to_owned(),
        choices: if kind.is_empty() {
            Vec::new()
        } else {
            list.iter()
                .enumerate()
                .map(|(i, c)| ProjectChoiceView {
                    key: c.key.clone(),
                    cost: c.cost,
                    time: c.time,
                    risk: c.risk,
                    quality: c.quality,
                    preferred: i == preferred,
                })
                .collect()
        },
        choice_kind: kind.to_owned(),
    })
}

/// The event a project waits for.
fn event_view(catalog: &crate::catalog::Catalog, p: &projects::DevProject) -> Option<EventView> {
    let pending = p.event?;
    let kind = catalog
        .research_model
        .projects
        .as_ref()?
        .events
        .as_ref()?
        .kinds
        .get(usize::from(pending.kind))?;
    Some(EventView {
        key: kind.key.clone(),
        deadline: iso(pending.deadline),
        recommended: kind.recommended,
        options: kind
            .options
            .iter()
            .map(|o| EventOptionView {
                key: o.key.clone(),
                cost: o.cost,
                delay: o.delay,
                quality: o.quality,
            })
            .collect(),
    })
}
