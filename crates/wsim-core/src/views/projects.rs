//! The development projects for the interface (PD2, docs/PRODUKTENTWICKLUNG.md): the
//! company's projects with staffing and scenarios, the project leaders (own and on the
//! market) and the products a project can start for.

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::catalog::LabRole;
use crate::game::Game;
use crate::ids::{Id, ProductId};
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
    pub leads: Vec<ProjectLeadView>,
    pub projects: Vec<ProjectRowView>,
    pub offers: Vec<ProjectOfferView>,
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
        leads,
        projects,
        offers,
    })
}
