//! PD7 – gate council, fast-track, pilot series and the risk fields of the phases
//! (docs/FORMELN.md).
//!
//! The council gives three voices at PM3, PM4 and PM7; fast-track (phases 3 and 4 overlap)
//! and the pilot series (before PM7) are proposed by the leader at PM3 and PM6 and decided
//! together with the milestone. Without the data blocks none of it exists.

use serde::{Deserialize, Serialize};

use crate::catalog::{Catalog, DepartmentKind, ProjectModel};
use crate::project_reports::{Decision, Kpis, LightColor};
use crate::projects::{self, DevProject};
use crate::rng::{SimRng, Stream};
use crate::state::GameState;

/// Milestones with a council.
pub const COUNCIL_AT: [u8; 3] = [3, 4, 7];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceRole {
    Finance,
    Technology,
    Market,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stance {
    For,
    Doubt,
    Against,
}

/// What one voice of the council said.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Voice {
    pub role: VoiceRole,
    pub stance: Stance,
    /// The voice turned against the other two.
    pub dissent: bool,
}

fn project_id(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

/// Factors on the duration and cost of a phase from fast-track and the pilot series.
pub fn phase_adjust(m: &ProjectModel, p: &DevProject, phase: usize) -> (f64, f64) {
    let mut time = 1.0;
    let mut cost = 1.0;
    if let (Some(f), true, 3 | 4) = (&m.fast_track, p.fast_track, phase) {
        time *= 1.0 - f.overlap;
        cost *= f.cost_factor;
    }
    if let (Some(x), 1, 5) = (&m.pilot, p.pilot, phase) {
        time *= 1.0 + x.duration_share;
        cost *= 1.0 + x.cost_share;
    }
    if matches!(phase, 3 | 4) && p.platform > 0.0 {
        time *= 1.0 - p.platform;
        cost *= 1.0 - p.platform;
    }
    (time, cost)
}

/// Plants of the company in the branch of the product (0–1 of the reference).
fn branch_strength(state: &GameState, catalog: &Catalog, p: &DevProject, reference: f64) -> f64 {
    if reference <= 0.0 {
        return 0.0;
    }
    let branch = catalog.products.get(p.product).branch;
    let mut plants = 0.0;
    for s in state.sites.iter().filter(|s| s.held_by(p.company)) {
        for sl in &s.slots {
            if let Some(r) = sl.recipe
                && catalog.products.get(catalog.recipes.get(r).product).branch == branch
            {
                plants += 1.0;
            }
        }
    }
    (plants / reference).min(1.0)
}

/// Multiplier on the event risk from branch strength, budget buffer and fast-track.
pub fn risk_factor(state: &GameState, catalog: &Catalog, index: usize) -> f64 {
    let Some(m) = catalog.research_model.projects.as_ref() else {
        return 1.0;
    };
    let p = &state.dev_projects[index];
    let r = &m.risk;
    let branch = branch_strength(state, catalog, p, r.branch_ref);
    let buffer = if r.buffer_ref > 0.0 {
        let cash = state.companies[p.company.index()].ledger.cash().to_usd()
            / projects::price_level(state, p.site).max(1e-9);
        let open = crate::project_reports::open_cost(m, p).max(1e-9);
        ((cash / open - 1.0) / r.buffer_ref).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let ft = if p.fast_track && matches!(p.phase, 3 | 4) {
        1.0
    } else {
        0.0
    };
    ((1.0 - r.branch * branch) * (1.0 - r.buffer * buffer) * (1.0 + r.fast_track * ft)).max(0.0)
}

/// Hidden defect risk the fast-track adds (the rest is `project_events::defect_risk`).
pub fn extra_defect(catalog: &Catalog, p: &DevProject) -> f64 {
    match (
        catalog
            .research_model
            .projects
            .as_ref()
            .and_then(|m| m.fast_track.as_ref()),
        p.fast_track,
    ) {
        (Some(f), true) => f.defect,
        _ => 0.0,
    }
}

/// The voices of the council at a milestone.
pub fn council(
    state: &GameState,
    catalog: &Catalog,
    index: usize,
    milestone: u8,
    truth: &Kpis,
) -> Vec<Voice> {
    let Some(m) = catalog.research_model.projects.as_ref() else {
        return Vec::new();
    };
    let (Some(c), Some(rm)) = (m.council.as_ref(), m.reports.as_ref()) else {
        return Vec::new();
    };
    if !COUNCIL_AT.contains(&milestone) {
        return Vec::new();
    }
    let p = &state.dev_projects[index];
    let lead = state.project_leads.get(p.lead as usize);
    let experience = lead.map_or(0.0, |l| l.experience);
    let dissent = lead
        .and_then(|l| rm.traits.get(usize::from(l.character)))
        .map_or(0.0, |t| t.dissent);
    let risk = projects::scenario(m, p, experience, 0).risk;
    let finance = 1.0 - 2.0 * truth.payback_years / rm.payback_max_years.max(1e-9);
    let technology = 1.0 - 2.0 * risk / rm.lights[3].red.max(1e-9);
    let market = p.baseline.map_or(0.0, |b| {
        2.0 * (truth.revenue_year / b.truth.revenue_year.max(1e-9) - 1.0)
    });
    let strength = crate::central::strength(catalog, state, p.company, DepartmentKind::Strategy);
    let sigma = c.uncertainty * (1.0 - c.accuracy * strength);
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::GateCouncil {
            project: project_id(index),
            milestone,
        },
    );
    let seen = [
        (VoiceRole::Finance, finance),
        (VoiceRole::Technology, technology),
        (VoiceRole::Market, market),
    ]
    .map(|(role, w)| {
        let u = 2.0 * rng.next_f64() - 1.0;
        let w = w.clamp(-1.0, 1.0) + sigma * u;
        let stance = if w >= c.threshold {
            Stance::For
        } else if w <= -c.threshold {
            Stance::Against
        } else {
            Stance::Doubt
        };
        (role, stance)
    });
    // A voice turns against the other two when they agree.
    let mut out = Vec::new();
    for (i, (role, stance)) in seen.iter().enumerate() {
        let others: Vec<Stance> = seen
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, (_, s))| *s)
            .collect();
        let roll = rng.next_f64();
        let agree = others[0] == others[1] && others[0] != Stance::Doubt;
        let flip = agree && roll < dissent;
        let stance = if flip {
            if others[0] == Stance::For {
                Stance::Against
            } else {
                Stance::For
            }
        } else {
            *stance
        };
        out.push(Voice {
            role: *role,
            stance,
            dissent: flip,
        });
    }
    out
}

/// Voices for minus voices against.
pub fn majority(voices: &[Voice]) -> i32 {
    voices
        .iter()
        .map(|v| match v.stance {
            Stance::For => 1,
            Stance::Doubt => 0,
            Stance::Against => -1,
        })
        .sum()
}

/// The decision when nobody asked decides: the leader's recommendation, one step back where
/// the council is against. Returns the decision and whether the council changed it.
pub fn adjusted(
    state: &GameState,
    catalog: &Catalog,
    index: usize,
    milestone: u8,
    recommendation: Decision,
    voices: &[Voice],
) -> (Decision, bool) {
    let Some(c) = catalog
        .research_model
        .projects
        .as_ref()
        .and_then(|m| m.council.as_ref())
    else {
        return (recommendation, false);
    };
    let majority = majority(voices);
    if voices.is_empty() || majority >= 0 {
        return (recommendation, false);
    }
    let p = &state.dev_projects[index];
    let strong = milestone == 7 && majority <= -i32::try_from(c.pause_majority).unwrap_or(3);
    let decision = match recommendation {
        Decision::Continue | Decision::Conditions if strong && state.is_main(p.company) => {
            Decision::Pause
        }
        Decision::Continue => Decision::Conditions,
        Decision::Conditions if p.loops < c.max_loops => Decision::Loop,
        other => other,
    };
    (decision, decision != recommendation)
}

/// Whether the leader proposes fast-track (PM3) or a pilot series (PM6).
pub fn propose(
    state: &GameState,
    catalog: &Catalog,
    index: usize,
    milestone: u8,
    lights: &[LightColor; 4],
) -> bool {
    let Some(m) = catalog.research_model.projects.as_ref() else {
        return false;
    };
    let p = &state.dev_projects[index];
    let lead = state.project_leads.get(p.lead as usize);
    let character = lead.and_then(|l| {
        m.reports
            .as_ref()
            .and_then(|r| r.traits.get(usize::from(l.character)))
    });
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::ProjectProposal {
            project: project_id(index),
            milestone,
        },
    );
    let roll = rng.next_f64();
    match milestone {
        3 => {
            let Some(f) = &m.fast_track else {
                return false;
            };
            let experience = lead.map_or(0.0, |l| l.experience);
            let hasty = m
                .tempos
                .get(usize::from(p.tempo))
                .is_some_and(|t| t.risk >= 1.0);
            let reason = lights[0] != LightColor::Green;
            let chance = character.map_or(0.0, |t| t.fast_track);
            !p.fast_track && hasty && experience >= f.experience_min && (reason || roll < chance)
        }
        6 => {
            let Some(x) = &m.pilot else {
                return false;
            };
            let defect = crate::project_events::total_defect(catalog, p);
            let reason = defect >= x.risk_from || lights[2] != LightColor::Green;
            let chance = character.map_or(0.0, |t| t.pilot);
            p.pilot == 0 && (reason || roll < chance)
        }
        _ => false,
    }
}

/// What a pilot series found at the end of the last phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PilotResult {
    Found,
    Clear,
}

/// The pilot series ends: a defect is found or not; returns the result (the project is
/// left to do the rework in the first case).
pub fn pilot_result(state: &mut GameState, catalog: &Catalog, index: usize) -> Option<PilotResult> {
    let x = catalog
        .research_model
        .projects
        .as_ref()
        .and_then(|m| m.pilot.clone())?;
    if state.dev_projects[index].pilot != 1 {
        return None;
    }
    let total = crate::project_events::total_defect(catalog, &state.dev_projects[index]);
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::ProjectPilot {
            project: project_id(index),
        },
    );
    let found = rng.chance((total * x.detection).min(1.0));
    let p = &mut state.dev_projects[index];
    p.pilot = 2;
    if found {
        p.defect -= x.repair * total;
        p.progress = (1.0 - x.rework).clamp(0.0, 1.0);
        Some(PilotResult::Found)
    } else {
        p.defect -= x.calm * total;
        p.bonus_quality += x.quality_bonus;
        Some(PilotResult::Clear)
    }
}
