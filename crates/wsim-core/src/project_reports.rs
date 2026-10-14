//! Milestone reports, decisions and delegation of the development projects (PD3,
//! docs/PRODUKTENTWICKLUNG.md §4 and §7; formulas in docs/FORMELN.md, PD3).
//!
//! At every milestone the project leader reports the figures of the product: what it
//! costs to make, what it sells for, what the project and the plants cost, when it pays
//! off. The figures of PM2 become the baseline of the later reviews. The player decides
//! at the milestones the delegation names; the leader decides the others with his
//! recommendation, exactly as the AI does.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, LabRole, ProjectModel, ProjectReportModel};
use crate::command::CommandError;
use crate::ids::ProductId;
use crate::message::{Message, MessageKind, Param, keys};
use crate::projects::{self, DevProject, ProjectStatus};
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, GameState};
use crate::{health, market, product_research};

/// Who decides at the milestones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelegationMode {
    /// The player decides at every milestone.
    ShowAll,
    /// The player decides at the milestones of the data, the leader at the others.
    Standard,
    /// The leader decides everything.
    LeadDecides,
}

/// What can be decided at a milestone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Continue,
    /// Go on with conditions: more risk in the phases that follow.
    Conditions,
    /// The phase that just ended runs again.
    Loop,
    Pause,
    Abort,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightColor {
    Green,
    Yellow,
    Red,
}

/// The figures a report shows (USD at price level 1 unless named otherwise).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Kpis {
    pub unit_cost: f64,
    pub price: f64,
    pub volume_year: f64,
    pub revenue_year: f64,
    pub lifecycle_revenue: f64,
    pub contribution_year: f64,
    pub plants: f64,
    pub investment: f64,
    pub project_cost: f64,
    pub project_days: f64,
    pub people: f64,
    pub payback_years: f64,
    pub quality: f64,
}

/// The figures at PM2 and what the later reviews compare with.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Baseline {
    /// What the report showed (with the leader's bias).
    pub kpis: Kpis,
    /// The unbiased figures at that time, for the causes of deviations.
    pub truth: Kpis,
    pub plan_days: f64,
    pub plan_cost: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub decision: Decision,
    pub choice: Option<u8>,
    pub by_player: bool,
    pub date: Date,
}

/// The report at one milestone.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MilestoneReport {
    pub milestone: u8,
    pub date: Date,
    /// What the leader shows.
    pub forecast: Kpis,
    /// The unbiased figures; never shown, used for the after-calculation (PD6).
    pub truth: Kpis,
    /// Schedule, budget, quality, risk.
    pub lights: [LightColor; 4],
    pub recommendation: Decision,
    /// Keys of what moved since PM2 (`marktpreis`, `herstellkosten`, `verzug` …).
    pub causes: Vec<String>,
    /// The factor of the leader's bias on volume (1 + b).
    pub bias: f64,
    pub decided: Option<DecisionRecord>,
}

/// A decision the project waits for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pending {
    pub milestone: u8,
    pub raised: Date,
    pub deadline: Date,
    /// The phase that ended; a loop runs it again.
    pub ended_phase: u8,
    pub recommendation: Decision,
}

fn models(catalog: &Catalog) -> Option<(&ProjectModel, &ProjectReportModel)> {
    let m = catalog.research_model.projects.as_ref()?;
    Some((m, m.reports.as_ref()?))
}

/// Who decides at a milestone: the player, or the project leader.
pub fn player_decides(
    state: &GameState,
    rm: &ProjectReportModel,
    p: &DevProject,
    milestone: u8,
) -> bool {
    if !state.is_main(p.company) {
        return false;
    }
    let mode = p
        .delegation
        .or_else(|| state.project_delegation.get(&p.company).copied())
        .unwrap_or(DelegationMode::Standard);
    match mode {
        DelegationMode::ShowAll => true,
        DelegationMode::LeadDecides => false,
        DelegationMode::Standard => rm.player_at.contains(&milestone),
    }
}

/// The biggest need of a project in people over its phases.
fn people(m: &ProjectModel, effort: f64) -> f64 {
    (0..m.phases.len())
        .map(|phase| {
            LabRole::ALL
                .iter()
                .map(|&r| f64::from(projects::need(m, phase, r, effort)))
                .sum::<f64>()
        })
        .fold(0.0, f64::max)
}

/// Cost still to come in USD at price level 1.
fn remaining_cost(m: &ProjectModel, p: &DevProject) -> f64 {
    let f = projects::factors(m, p);
    let at = (usize::from(p.tempo), usize::from(p.trim));
    let mut rest = 0.0;
    for phase in usize::from(p.phase).min(6)..6 {
        let left = if phase == usize::from(p.phase) {
            1.0 - p.progress
        } else {
            1.0
        };
        rest += projects::phase_cost_usd(m, at, phase, p.effort) * f.cost * left;
        if phase == 2 {
            rest += projects::gate_cost_usd(m, at, p.effort) * f.cost;
        }
    }
    rest
}

/// The plan of a project before PM2: duration and cost at its tempo and trim.
fn initial_plan(m: &ProjectModel, p: &DevProject) -> (f64, f64) {
    let at = (usize::from(p.tempo), usize::from(p.trim));
    let days = (0..6)
        .map(|k| projects::phase_days(m, at.0, k, p.effort))
        .sum::<f64>();
    let cost = (0..6)
        .map(|k| projects::phase_cost_usd(m, at, k, p.effort))
        .sum::<f64>()
        + projects::gate_cost_usd(m, at, p.effort);
    (days, cost)
}

/// The unbiased figures of the project today.
fn truth_kpis(state: &GameState, catalog: &Catalog, p: &DevProject) -> Kpis {
    let Some((m, rm)) = models(catalog) else {
        return Kpis::default();
    };
    let f = projects::factors(m, p);
    let at = (usize::from(p.tempo), usize::from(p.trim));
    let site = &state.sites[p.site.index()];
    let country = site.country;
    let cs = state.countries.get(country);
    let scenario = projects::scenario(
        m,
        p,
        state
            .project_leads
            .get(p.lead as usize)
            .map_or(0.0, |l| l.experience),
        0,
    );
    let mut k = Kpis {
        project_cost: p.effort
            * m.cost_per_point_usd
            * m.tempos[at.0].cost
            * m.trims[at.1].cost
            * f.cost,
        project_days: f64::from(p.started.days_until(state.date)) + projects::days_left(m, p),
        people: people(m, p.effort),
        quality: 1.0 + m.trims[at.1].quality + f.quality - scenario.penalty,
        ..Kpis::default()
    };
    let Some(recipe) = product_research::main_recipe(catalog, p.product) else {
        return k;
    };
    let r = catalog.recipes.get(recipe);
    let fac = catalog.facilities.get(r.facility);
    let price_of = |q: ProductId| market::market_price(catalog, state, country, q).to_usd();
    k.price = market::market_price(catalog, state, country, p.product).to_usd();
    k.unit_cost = health::unit_cost(
        catalog,
        cs,
        recipe,
        catalog.market_model.normal_utilization,
        price_of,
    )
    .total();
    let open: f64 = state
        .markets
        .get(p.product)
        .iter()
        .map(|(_, mk)| market::open_demand(mk, &catalog.market_model, state.date))
        .sum();
    let plant_year = (r.output * fac.runs_per_day * 365.0 * rm.utilization).max(1e-9);
    k.volume_year = if open > 0.0 {
        open * 365.0 * rm.capture_share
    } else {
        rm.plants_without_demand * plant_year
    };
    k.plants = (k.volume_year / plant_year).ceil().max(1.0);
    k.investment = k.plants * fac.investment.to_usd();
    k.revenue_year = k.volume_year * k.price;
    k.lifecycle_revenue = k.revenue_year * rm.lifecycle_years;
    k.contribution_year = k.volume_year * (k.price - k.unit_cost);
    k.payback_years = payback(&k);
    k
}

fn payback(k: &Kpis) -> f64 {
    if k.contribution_year > 0.0 {
        (k.project_cost + k.investment) / k.contribution_year
    } else {
        999.0
    }
}

/// What the leader shows for the true figures: the volume and thus revenue, margin and
/// plants are too high by his bias, the unit cost too low by half of it.
fn forecast(truth: &Kpis, bias: f64) -> Kpis {
    let mut k = *truth;
    let volume = truth.volume_year * bias;
    let per_plant = if truth.plants > 0.0 {
        truth.investment / truth.plants
    } else {
        0.0
    };
    k.volume_year = volume;
    k.unit_cost = truth.unit_cost * (2.0 - bias).clamp(0.2, 1.8).mul_add(0.5, 0.5);
    k.revenue_year = volume * truth.price;
    k.lifecycle_revenue = k.revenue_year * (truth.lifecycle_revenue / truth.revenue_year.max(1e-9));
    k.contribution_year = volume * (truth.price - k.unit_cost);
    k.plants = (truth.plants * bias).ceil().max(1.0);
    k.investment = k.plants * per_plant;
    k.payback_years = payback(&k);
    k
}

fn light(value: f64, l: crate::catalog::Light) -> LightColor {
    if value >= l.red {
        LightColor::Red
    } else if value >= l.yellow {
        LightColor::Yellow
    } else {
        LightColor::Green
    }
}

/// The plan the traffic lights compare with: the baseline once PM2 is passed.
fn plan(m: &ProjectModel, p: &DevProject) -> (f64, f64) {
    p.baseline
        .map_or_else(|| initial_plan(m, p), |b| (b.plan_days, b.plan_cost))
}

fn lights(
    m: &ProjectModel,
    rm: &ProjectReportModel,
    p: &DevProject,
    truth: &Kpis,
    experience: f64,
    price_level: f64,
) -> [LightColor; 4] {
    let (plan_days, plan_cost) = plan(m, p);
    let scenario = projects::scenario(m, p, experience, 0);
    let spent = p.spent.to_usd() / price_level.max(1e-9);
    let projected = spent + remaining_cost(m, p);
    [
        light(
            (truth.project_days - plan_days) / plan_days.max(1e-9),
            rm.lights[0],
        ),
        light((projected - plan_cost) / plan_cost.max(1e-9), rm.lights[1]),
        light(scenario.penalty, rm.lights[2]),
        light(scenario.risk, rm.lights[3]),
    ]
}

fn recommend(
    rm: &ProjectReportModel,
    lights: &[LightColor; 4],
    truth: &Kpis,
    milestone: u8,
) -> Decision {
    let red = |i: usize| lights[i] == LightColor::Red;
    if truth.payback_years > rm.payback_max_years && lights.contains(&LightColor::Red) {
        Decision::Abort
    } else if red(2) && milestone >= 3 {
        Decision::Loop
    } else if (red(0) || red(1)) && !red(3) {
        Decision::Conditions
    } else {
        Decision::Continue
    }
}

/// Keys of what moved since PM2.
fn causes(
    rm: &ProjectReportModel,
    p: &DevProject,
    truth: &Kpis,
    lights: &[LightColor; 4],
) -> Vec<String> {
    let _ = rm;
    let mut out = Vec::new();
    if let Some(b) = &p.baseline {
        let moved = |now: f64, then: f64| then > 0.0 && ((now - then) / then).abs() > 0.02;
        if moved(truth.price, b.truth.price) {
            out.push("marktpreis".to_owned());
        }
        if moved(truth.unit_cost, b.truth.unit_cost) {
            out.push("herstellkosten".to_owned());
        }
        if lights[0] != LightColor::Green {
            out.push("verzug".to_owned());
        }
    }
    if lights[2] != LightColor::Green {
        out.push("unterbesetzung".to_owned());
    }
    if p.conditions > 0 {
        out.push("auflagen".to_owned());
    }
    if p.loops > 0 {
        out.push("schleife".to_owned());
    }
    if p.path != 1 {
        out.push("weg".to_owned());
    }
    out
}

/// The path a leader prefers by his character, the middle one without data.
fn lead_path(state: &GameState, rm: &ProjectReportModel, p: &DevProject) -> u8 {
    state
        .project_leads
        .get(p.lead as usize)
        .and_then(|l| rm.traits.get(usize::from(l.character)))
        .map_or(1, |t| u8::try_from(t.path).unwrap_or(1))
}

/// Works through the milestones a phase ended at, one after the other, until one needs
/// the player.
pub(crate) fn process_queue(
    state: &mut GameState,
    catalog: &Catalog,
    index: usize,
    date: Date,
) -> Vec<Message> {
    let mut news = Vec::new();
    loop {
        let p = &mut state.dev_projects[index];
        if !p.running() || p.pending.is_some() || p.paused || p.queued.is_empty() {
            break;
        }
        let milestone = p.queued.remove(0);
        news.extend(raise(state, catalog, index, milestone, date));
    }
    news
}

/// A milestone is reached: the report is made and the decision taken or asked for.
fn raise(
    state: &mut GameState,
    catalog: &Catalog,
    index: usize,
    milestone: u8,
    date: Date,
) -> Vec<Message> {
    let Some((m, rm)) = models(catalog) else {
        return Vec::new();
    };
    let p = &state.dev_projects[index];
    let lead = state.project_leads.get(p.lead as usize);
    let (experience, character) =
        lead.map_or((0.0, 0), |l| (l.experience, usize::from(l.character)));
    let truth = truth_kpis(state, catalog, p);
    // The leader's bias: his optimism and the spread of the forecast, narrower with
    // product managers at work.
    let pm_q = {
        let phase = usize::from(p.phase).saturating_sub(1).min(5);
        let n = projects::need(m, phase, LabRole::ProductManager, p.effort);
        if n == 0 {
            1.0
        } else {
            (f64::from(p.assigned[LabRole::ProductManager.index()]) / f64::from(n)).min(1.0)
        }
    };
    let sigma = rm.uncertainty * (1.0 - rm.accuracy_per_pm * pm_q);
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::ProjectReport {
            project: u32::try_from(index).unwrap_or(u32::MAX),
            milestone,
        },
    );
    let u = 2.0 * rng.next_f64() - 1.0;
    let optimism = rm.traits.get(character).map_or(0.0, |t| t.optimism);
    let bias = (1.0 + optimism + sigma * u).max(0.1);
    let price_level = projects::price_level(state, p.site);
    let lights = lights(m, rm, p, &truth, experience, price_level);
    let recommendation = recommend(rm, &lights, &truth, milestone);
    let report = MilestoneReport {
        milestone,
        date,
        forecast: forecast(&truth, bias),
        truth,
        lights,
        recommendation,
        causes: causes(rm, p, &truth, &lights),
        bias,
        decided: None,
    };
    let ask = player_decides(state, rm, p, milestone);
    let ended_phase = p.phase.saturating_sub(1);
    let deadline = date.add_days(i32::try_from(rm.deadline_days).unwrap_or(7));
    let own = state.is_main(p.company);
    let name = format!("produkt.{}", catalog.products.key(p.product));
    let default_choice = match milestone {
        1 => Some(1),
        2 => Some(lead_path(state, rm, p)),
        _ => None,
    };
    let p = &mut state.dev_projects[index];
    p.reports.push(report);
    p.pending = Some(Pending {
        milestone,
        raised: date,
        deadline,
        ended_phase,
        recommendation,
    });
    let mut news = Vec::new();
    if ask {
        news.push(
            Message::new(MessageKind::Warning, keys::PROJECT_DECISION_DUE)
                .with("produkt", Param::TextKey(name))
                .with("meilenstein", Param::Number(f64::from(milestone))),
        );
    } else {
        let decided = apply(
            state,
            catalog,
            index,
            date,
            (recommendation, default_choice),
            false,
        );
        if own {
            news.push(
                Message::new(MessageKind::Info, keys::PROJECT_DECIDED)
                    .with("produkt", Param::TextKey(name))
                    .with("meilenstein", Param::Number(f64::from(milestone)))
                    .with(
                        "entscheidung",
                        Param::TextKey(format!(
                            "projekte.entscheidung.{}",
                            decision_key(recommendation)
                        )),
                    ),
            );
        }
        news.extend(decided);
    }
    news
}

/// The key of a decision in the texts and the views.
pub fn decision_key(d: Decision) -> &'static str {
    match d {
        Decision::Continue => "weiter",
        Decision::Conditions => "auflagen",
        Decision::Loop => "schleife",
        Decision::Pause => "pause",
        Decision::Abort => "abbruch",
    }
}

/// Carries out the decision on the pending milestone.
fn apply(
    state: &mut GameState,
    catalog: &Catalog,
    index: usize,
    date: Date,
    (decision, choice): (Decision, Option<u8>),
    by_player: bool,
) -> Vec<Message> {
    let Some((m, rm)) = models(catalog) else {
        return Vec::new();
    };
    let Some(pending) = state.dev_projects[index].pending.take() else {
        return Vec::new();
    };
    let mut news = Vec::new();
    let milestone = pending.milestone;
    let p = &mut state.dev_projects[index];
    if let Some(r) = p
        .reports
        .iter_mut()
        .rev()
        .find(|r| r.milestone == milestone)
    {
        r.decided = Some(DecisionRecord {
            decision,
            choice,
            by_player,
            date,
        });
    }
    // The choices of PM1 and PM2 count whatever is decided about the milestone.
    if let Some(c) = choice {
        match milestone {
            1 if usize::from(c) < rm.profiles.len() => p.profile = c,
            2 if usize::from(c) < rm.paths.len() => p.path = c,
            _ => {}
        }
    }
    if milestone == 2 {
        // The figures of PM2 with the chosen path become the baseline.
        let snapshot = state.dev_projects[index].clone();
        let truth = truth_kpis(state, catalog, &snapshot);
        let bias = snapshot
            .reports
            .iter()
            .rev()
            .find(|r| r.milestone == 2)
            .map_or(1.0, |r| r.bias);
        let (plan_days, plan_cost) = {
            let f = projects::factors(m, &snapshot);
            let (d, c) = initial_plan(m, &snapshot);
            (d * f.time, c * f.cost)
        };
        state.dev_projects[index].baseline = Some(Baseline {
            kpis: forecast(&truth, bias),
            truth,
            plan_days,
            plan_cost,
        });
    }
    let p = &mut state.dev_projects[index];
    match decision {
        Decision::Continue => {}
        Decision::Conditions => p.conditions += 1,
        Decision::Loop => {
            p.loops += 1;
            p.phase = pending.ended_phase;
            p.progress = 0.0;
            p.queued.clear();
        }
        Decision::Pause => p.paused = true,
        Decision::Abort => {
            p.queued.clear();
            p.status = ProjectStatus::Aborted;
            p.finished = Some(date);
            let lead = p.lead as usize;
            if let Some(l) = state.project_leads.get_mut(lead) {
                l.aborted += 1;
            }
        }
    }
    if milestone == 7 && matches!(decision, Decision::Continue | Decision::Conditions) {
        news.extend(projects::release(state, catalog, index, date));
    }
    news
}

/// `DecideMilestone`: the owner decides the pending milestone of a project.
pub(crate) fn decide(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    project: u32,
    (decision, choice): (Decision, Option<u8>),
) -> Result<(), CommandError> {
    let Some((_, rm)) = models(catalog) else {
        return Err(CommandError::ProjectsOff);
    };
    let index = project as usize;
    let Some(p) = state.dev_projects.get(index) else {
        return Err(CommandError::UnknownProject);
    };
    if p.company != actor || !p.running() {
        return Err(CommandError::UnknownProject);
    }
    let Some(pending) = p.pending else {
        return Err(CommandError::NothingToDecide);
    };
    let valid_choice = match (pending.milestone, choice) {
        (_, None) => true,
        (1, Some(c)) => usize::from(c) < rm.profiles.len(),
        (2, Some(c)) => usize::from(c) < rm.paths.len(),
        _ => false,
    };
    if !valid_choice {
        return Err(CommandError::InvalidQuantity);
    }
    let date = state.date;
    let news = apply(state, catalog, index, date, (decision, choice), true);
    let more = process_queue(state, catalog, index, date);
    state.project_news.extend(news);
    state.project_news.extend(more);
    Ok(())
}

/// `ResumeProject`: a project on hold goes on.
pub(crate) fn resume(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    project: u32,
) -> Result<(), CommandError> {
    let index = project as usize;
    let Some(p) = state.dev_projects.get_mut(index) else {
        return Err(CommandError::UnknownProject);
    };
    if p.company != actor || !p.running() || !p.paused {
        return Err(CommandError::UnknownProject);
    }
    p.paused = false;
    let date = state.date;
    let more = process_queue(state, catalog, index, date);
    state.project_news.extend(more);
    Ok(())
}

/// `SetProjectDelegation`: who decides at the milestones, for the company or one project.
pub(crate) fn set_delegation(
    state: &mut GameState,
    actor: CompanyId,
    project: Option<u32>,
    mode: DelegationMode,
) -> Result<(), CommandError> {
    match project {
        None => {
            state.project_delegation.insert(actor, mode);
        }
        Some(i) => {
            let Some(p) = state.dev_projects.get_mut(i as usize) else {
                return Err(CommandError::UnknownProject);
            };
            if p.company != actor {
                return Err(CommandError::UnknownProject);
            }
            p.delegation = Some(mode);
        }
    }
    Ok(())
}

/// Every day: decisions the player left open past the deadline are taken by the leader
/// with his recommendation.
pub(crate) fn resolve_due(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let Some((_, rm)) = models(catalog) else {
        return Vec::new();
    };
    let mut news = std::mem::take(&mut state.project_news);
    for index in 0..state.dev_projects.len() {
        let p = &state.dev_projects[index];
        let Some(pending) = p.pending else {
            continue;
        };
        if !p.running() || date < pending.deadline {
            continue;
        }
        let choice = match pending.milestone {
            1 => Some(1),
            2 => Some(lead_path(state, rm, p)),
            _ => None,
        };
        let name = format!("produkt.{}", catalog.products.key(p.product));
        let own = state.is_main(p.company);
        news.extend(apply(
            state,
            catalog,
            index,
            date,
            (pending.recommendation, choice),
            false,
        ));
        if own {
            news.push(
                Message::new(MessageKind::Info, keys::PROJECT_AUTO_DECIDED)
                    .with("produkt", Param::TextKey(name))
                    .with("meilenstein", Param::Number(f64::from(pending.milestone)))
                    .with(
                        "entscheidung",
                        Param::TextKey(format!(
                            "projekte.entscheidung.{}",
                            decision_key(pending.recommendation)
                        )),
                    ),
            );
        }
        news.extend(process_queue(state, catalog, index, date));
    }
    news
}
