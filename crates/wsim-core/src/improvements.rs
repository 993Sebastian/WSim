//! Improvement projects WM0–WM3 for a product (PD5, docs/PRODUKTENTWICKLUNG.md §8; formulas
//! in docs/FORMELN.md, PD5).
//!
//! A laboratory improves one figure of a product: cost, quality, scrap, rate, inputs or
//! equipment. The analysis finds approaches (WM1), the player or the leader picks one, the
//! optimization brings a result that lies above or below the expectation (WM2), and the
//! release (WM3) lets the gain take effect in all plants at once or step by step. The gains
//! stack on the development of M37 within a cap per target.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, ImprovementModel, LabRole, ProjectModel, SiteType};
use crate::command::CommandError;
use crate::development::{self, Effect};
use crate::ids::{Id, ProductId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::project_reports::DelegationMode;
use crate::projects;
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, GameState, SiteId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpStatus {
    Running,
    Released,
    Aborted,
}

/// One approach the analysis found: expected gain and cost factor.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Approach {
    pub gain: f64,
    pub cost: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpPendingKind {
    /// WM1: choose an approach.
    Choose,
    /// WM3: release at once or step by step.
    Release,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpPending {
    pub kind: ImpPendingKind,
    pub raised: Date,
    pub deadline: Date,
}

/// An improvement project of a company.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImprovementProject {
    pub company: CompanyId,
    pub site: SiteId,
    pub product: ProductId,
    /// Index into the targets of the model.
    pub target: u8,
    pub lead: u32,
    pub started: Date,
    /// Research points of the stage.
    pub effort: f64,
    /// Phase 0–2; 3 once released.
    pub phase: u8,
    pub progress: f64,
    pub assigned: [u32; 4],
    pub status: ImpStatus,
    pub spent: Money,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished: Option<Date>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approaches: Vec<Approach>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen: Option<u8>,
    /// The result of WM2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending: Option<ImpPending>,
}

impl ImprovementProject {
    pub fn running(&self) -> bool {
        self.status == ImpStatus::Running
    }

    fn active(&self) -> bool {
        self.running() && self.pending.is_none()
    }

    /// Cost factor of the chosen approach (1 before the choice).
    pub fn cost_factor(&self) -> f64 {
        self.chosen
            .and_then(|c| self.approaches.get(usize::from(c)))
            .map_or(1.0, |a| a.cost)
    }
}

/// A stage a company released.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReleasedStage {
    pub gain: f64,
    pub released: Date,
    pub ramp_days: u32,
}

/// The stages a company released for a target of a product; the list is kept sorted by key.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Improved {
    pub company: CompanyId,
    pub product: ProductId,
    pub target: u8,
    pub stages: Vec<ReleasedStage>,
}

fn key(i: &Improved) -> (usize, usize, u8) {
    (i.company.index(), i.product.index(), i.target)
}

fn models(catalog: &Catalog) -> Option<(&ProjectModel, &ImprovementModel)> {
    let m = catalog.research_model.projects.as_ref()?;
    Some((m, m.improvement.as_ref()?))
}

/// Stages a company released for a target of a product.
pub fn stages_done(state: &GameState, company: CompanyId, product: ProductId, target: u8) -> usize {
    let k = (company.index(), product.index(), target);
    state
        .improved
        .binary_search_by(|i| key(i).cmp(&k))
        .map_or(0, |i| state.improved[i].stages.len())
}

/// Weighted sum of the released gains of a target at `date` (ramp-up included).
fn gain_sum(w: &ImprovementModel, stages: &[ReleasedStage], date: Date) -> f64 {
    stages
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let weight = if i < usize::from(w.stages) {
                1.0
            } else {
                w.rest_share
            };
            let ramp = if s.ramp_days == 0 {
                1.0
            } else {
                (f64::from(s.released.days_until(date)) / f64::from(s.ramp_days)).clamp(0.0, 1.0)
            };
            s.gain * weight * ramp
        })
        .sum()
}

/// Puts the released improvements of the company for the product on top of `e` (PD5).
pub fn apply(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
    e: &mut Effect,
) {
    let Some((_, w)) = models(catalog) else {
        return;
    };
    if state.improved.is_empty() {
        return;
    }
    let from = (company.index(), product.index(), 0u8);
    let start = state.improved.partition_point(|i| key(i) < from);
    for imp in state.improved[start..]
        .iter()
        .take_while(|i| i.company == company && i.product == product)
    {
        let Some(t) = w.targets.get(usize::from(imp.target)) else {
            continue;
        };
        let level = (gain_sum(w, &imp.stages, state.date) * t.per_stage).min(t.cap);
        e.labor *= 1.0 - level * t.effect.labor;
        e.inputs *= 1.0 - level * t.effect.inputs;
        e.quality += level * t.effect.quality;
        e.scrap *= 1.0 - level * t.effect.scrap;
        e.output *= 1.0 + level * t.effect.output;
    }
}

fn busy(state: &GameState, lead: u32) -> usize {
    state
        .improvement_projects
        .iter()
        .filter(|p| p.running() && p.lead == lead)
        .count()
}

/// Whether the leader leads an improvement project.
pub(crate) fn lead_busy(state: &GameState, lead: u32) -> bool {
    busy(state, lead) > 0
}

fn need(w: &ImprovementModel, m: &ProjectModel, phase: usize, role: LabRole, effort: f64) -> u32 {
    let Some(pr) = w.phases[phase].roles.iter().find(|r| r.role == role) else {
        return 0;
    };
    (pr.need * projects::scale(m, effort)).ceil() as u32
}

/// Speed of a phase from the people assigned.
fn speed(
    w: &ImprovementModel,
    m: &ProjectModel,
    phase: usize,
    effort: f64,
    assigned: [u32; 4],
) -> (f64, f64) {
    let mut speed = 0.0;
    let mut shortfall = 0.0;
    for pr in &w.phases[phase].roles {
        let n = need(w, m, phase, pr.role, effort);
        let q = if n == 0 {
            1.0
        } else {
            f64::from(assigned[pr.role.index()]) / f64::from(n)
        };
        speed += pr.effect * projects::gain(m, q);
        shortfall += pr.effect * (1.0 - q.min(1.0));
    }
    (speed, shortfall)
}

fn id32(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

fn product_param(catalog: &Catalog, product: ProductId) -> Param {
    Param::TextKey(format!("produkt.{}", catalog.products.key(product)))
}

fn target_param(w: &ImprovementModel, target: u8) -> Param {
    Param::TextKey(format!(
        "weiterentwicklung.ziel.{}",
        w.targets
            .get(usize::from(target))
            .map_or("", |t| t.key.as_str())
    ))
}

/// `StartImprovement`: WM0, a project for one target of a product.
pub(crate) fn start(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    (site, product, target, lead): (SiteId, ProductId, u8, u32),
) -> Result<(), CommandError> {
    let Some((m, w)) = models(catalog) else {
        return Err(CommandError::ProjectsOff);
    };
    let Some(s) = state.sites.get(site.index()) else {
        return Err(CommandError::UnknownSite);
    };
    if !s.held_by(actor) {
        return Err(CommandError::NotOwner);
    }
    if s.kind != SiteType::ResearchCenter {
        return Err(CommandError::WrongSiteType {
            required: SiteType::ResearchCenter,
        });
    }
    if usize::from(target) >= w.targets.len() {
        return Err(CommandError::InvalidQuantity);
    }
    let Some(leader) = state.project_leads.get(lead as usize) else {
        return Err(CommandError::LeadUnavailable);
    };
    if leader.employer != Some(actor) || busy(state, lead) >= m.lead.capacity_further as usize {
        return Err(CommandError::LeadUnavailable);
    }
    if state
        .improvement_projects
        .iter()
        .any(|p| p.running() && p.company == actor && p.product == product && p.target == target)
    {
        return Err(CommandError::ProjectExists);
    }
    if !development::can_develop(catalog, state, actor, product) {
        return Err(CommandError::NotResearchableProduct(
            catalog.products.key(product).to_owned(),
        ));
    }
    let Some((_, base)) = development::basis(catalog, product) else {
        return Err(CommandError::NotResearchableProduct(
            catalog.products.key(product).to_owned(),
        ));
    };
    let n = stages_done(state, actor, product, target) + 1;
    let stage = n.min(usize::from(w.stages)).max(1);
    let effort = base * w.effort_share * crate::math::pow(w.effort_growth, (stage - 1) as f64);
    state.improvement_projects.push(ImprovementProject {
        company: actor,
        site,
        product,
        target,
        lead,
        started: state.date,
        effort,
        phase: 0,
        progress: 0.0,
        assigned: [0; 4],
        status: ImpStatus::Running,
        spent: Money::ZERO,
        finished: None,
        approaches: Vec::new(),
        chosen: None,
        result: None,
        pending: None,
    });
    state.sites[site.index()].staffing_due = true;
    Ok(())
}

/// `AbortImprovement`.
pub(crate) fn abort(
    state: &mut GameState,
    actor: CompanyId,
    project: u32,
) -> Result<(), CommandError> {
    let date = state.date;
    let Some(p) = state.improvement_projects.get_mut(project as usize) else {
        return Err(CommandError::UnknownProject);
    };
    if p.company != actor || !p.running() {
        return Err(CommandError::UnknownProject);
    }
    p.status = ImpStatus::Aborted;
    p.finished = Some(date);
    p.pending = None;
    Ok(())
}

fn owner_decides(state: &GameState, company: CompanyId) -> bool {
    let mode = state
        .project_delegation
        .get(&company)
        .copied()
        .unwrap_or(DelegationMode::Standard);
    state.is_main(company) && mode != DelegationMode::LeadDecides
}

/// The approach the leader picks: optimistic leaders the biggest gain, careful ones the
/// cheapest, the others the best gain for the cost.
fn lead_choice(state: &GameState, catalog: &Catalog, p: &ImprovementProject) -> u8 {
    let optimism = state
        .project_leads
        .get(p.lead as usize)
        .and_then(|l| {
            catalog
                .research_model
                .projects
                .as_ref()?
                .reports
                .as_ref()?
                .traits
                .get(usize::from(l.character))
        })
        .map_or(0.0, |t| t.optimism);
    let score = |a: &Approach| {
        if optimism > 0.0 {
            a.gain
        } else if optimism < 0.0 {
            -a.cost
        } else {
            a.gain / a.cost.max(1e-9)
        }
    };
    let mut best = 0;
    for (i, a) in p.approaches.iter().enumerate() {
        if score(a) > score(&p.approaches[best]) {
            best = i;
        }
    }
    u8::try_from(best).unwrap_or(0)
}

/// WM1: the analysis found approaches.
fn analysis_done(
    state: &mut GameState,
    catalog: &Catalog,
    index: usize,
    date: Date,
) -> Vec<Message> {
    let Some((_, w)) = models(catalog) else {
        return Vec::new();
    };
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::ProjectApproach {
            project: id32(index),
        },
    );
    let (lo, hi) = w.approaches;
    let k = usize::from(lo) + (rng.next_f64() * f64::from(hi - lo + 1)) as usize;
    let k = k.min(usize::from(hi));
    let mut approaches = Vec::with_capacity(k);
    for _ in 0..k {
        let u = 2.0 * rng.next_f64() - 1.0;
        let gain = (1.0 + w.approach_spread * u).max(0.1);
        let cost = (1.0 + w.approach_cost_slope * (gain - 1.0)).max(0.5);
        approaches.push(Approach { gain, cost });
    }
    let (company, product, target) = {
        let p = &mut state.improvement_projects[index];
        p.approaches = approaches;
        p.phase = 1;
        p.progress = 0.0;
        (p.company, p.product, p.target)
    };
    if owner_decides(state, company) {
        state.improvement_projects[index].pending = Some(ImpPending {
            kind: ImpPendingKind::Choose,
            raised: date,
            deadline: date.add_days(i32::try_from(w.deadline_days).unwrap_or(7)),
        });
        return vec![
            Message::new(MessageKind::Warning, keys::IMPROVEMENT_CHOOSE)
                .with("produkt", product_param(catalog, product))
                .with("ziel", target_param(w, target)),
        ];
    }
    let pick = lead_choice(state, catalog, &state.improvement_projects[index]);
    state.improvement_projects[index].chosen = Some(pick);
    Vec::new()
}

/// WM2: the optimization is done; the result lies above or below the expectation.
fn optimization_done(state: &mut GameState, catalog: &Catalog, index: usize) -> Vec<Message> {
    let Some((_, w)) = models(catalog) else {
        return Vec::new();
    };
    let p = &state.improvement_projects[index];
    let expected = p
        .chosen
        .and_then(|c| p.approaches.get(usize::from(c)))
        .map_or(1.0, |a| a.gain);
    let experience = state
        .project_leads
        .get(p.lead as usize)
        .map_or(0.0, |l| l.experience);
    // The staffing gap of the last day, as a share of the roles' effects.
    let shortfall = {
        let (_, s) = speed(
            w,
            catalog.research_model.projects.as_ref().expect("checked"),
            1,
            p.effort,
            p.assigned,
        );
        s
    };
    let sigma = w.outcome_spread
        * (1.0 - w.outcome_experience * experience).max(0.0)
        * (1.0 + w.outcome_shortfall * shortfall);
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::ProjectOutcome {
            project: id32(index),
        },
    );
    let u = 2.0 * rng.next_f64() - 1.0;
    let result = (expected * (1.0 + sigma * u)).max(0.0);
    let (company, product, target) = (p.company, p.product, p.target);
    let p = &mut state.improvement_projects[index];
    p.result = Some(result);
    p.phase = 2;
    p.progress = 0.0;
    if !state.is_main(company) {
        return Vec::new();
    }
    vec![
        Message::new(
            if result >= expected {
                MessageKind::Success
            } else {
                MessageKind::Warning
            },
            keys::IMPROVEMENT_RESULT,
        )
        .with("produkt", product_param(catalog, product))
        .with("ziel", target_param(w, target))
        .with("erwartet", Param::Number(expected))
        .with("erreicht", Param::Number(result)),
    ]
}

/// WM3 is ready: the owner decides how the gain is released, or the leader releases it step by step.
fn preparation_done(
    state: &mut GameState,
    catalog: &Catalog,
    index: usize,
    date: Date,
) -> Vec<Message> {
    let Some((_, w)) = models(catalog) else {
        return Vec::new();
    };
    let (company, product, target) = {
        let p = &state.improvement_projects[index];
        (p.company, p.product, p.target)
    };
    if owner_decides(state, company) {
        state.improvement_projects[index].pending = Some(ImpPending {
            kind: ImpPendingKind::Release,
            raised: date,
            deadline: date.add_days(i32::try_from(w.deadline_days).unwrap_or(7)),
        });
        return vec![
            Message::new(MessageKind::Warning, keys::IMPROVEMENT_RELEASE)
                .with("produkt", product_param(catalog, product))
                .with("ziel", target_param(w, target)),
        ];
    }
    release(state, catalog, index, date, false)
}

/// WM3: the gain takes effect.
fn release(
    state: &mut GameState,
    catalog: &Catalog,
    index: usize,
    date: Date,
    immediate: bool,
) -> Vec<Message> {
    let Some((_, w)) = models(catalog) else {
        return Vec::new();
    };
    let (company, product, target, gain, spent) = {
        let p = &mut state.improvement_projects[index];
        p.status = ImpStatus::Released;
        p.finished = Some(date);
        p.pending = None;
        p.phase = 3;
        (
            p.company,
            p.product,
            p.target,
            p.result.unwrap_or(1.0),
            p.spent,
        )
    };
    if immediate && w.immediate_cost_share > 0.0 {
        let extra = Money::from_usd(spent.to_usd() * w.immediate_cost_share).unwrap_or(Money::ZERO);
        if extra > Money::ZERO {
            let site = state.improvement_projects[index].site;
            state.companies[company.index()].ledger.expense(
                CostType::Research,
                CostCenter::site(site),
                Account::Cash,
                extra,
            );
            state.improvement_projects[index].spent += extra;
        }
    }
    let stage = ReleasedStage {
        gain,
        released: date,
        ramp_days: if immediate { 0 } else { w.retrofit_days },
    };
    let k = (company.index(), product.index(), target);
    match state.improved.binary_search_by(|i| key(i).cmp(&k)) {
        Ok(i) => state.improved[i].stages.push(stage),
        Err(i) => state.improved.insert(
            i,
            Improved {
                company,
                product,
                target,
                stages: vec![stage],
            },
        ),
    }
    let lead = state.improvement_projects[index].lead as usize;
    if let Some(l) = state.project_leads.get_mut(lead) {
        l.finished += 1;
        l.experience = (l.experience
            + catalog
                .research_model
                .projects
                .as_ref()
                .map_or(0.0, |m| m.lead.experience_gain))
        .min(1.0);
    }
    state.sites[state.improvement_projects[index].site.index()].staffing_due = true;
    if !state.is_main(company) {
        return Vec::new();
    }
    vec![
        Message::new(MessageKind::Success, keys::IMPROVEMENT_DONE)
            .with("produkt", product_param(catalog, product))
            .with("ziel", target_param(w, target)),
    ]
}

/// `ChooseApproach`: the owner picks an approach at WM1.
pub(crate) fn choose(
    state: &mut GameState,
    actor: CompanyId,
    project: u32,
    approach: u8,
) -> Result<(), CommandError> {
    let Some(p) = state.improvement_projects.get_mut(project as usize) else {
        return Err(CommandError::UnknownProject);
    };
    if p.company != actor || !p.running() {
        return Err(CommandError::UnknownProject);
    }
    match p.pending {
        Some(ImpPending {
            kind: ImpPendingKind::Choose,
            ..
        }) => {}
        _ => return Err(CommandError::NothingToDecide),
    }
    if usize::from(approach) >= p.approaches.len() {
        return Err(CommandError::InvalidQuantity);
    }
    p.chosen = Some(approach);
    p.pending = None;
    Ok(())
}

/// `ReleaseImprovement`: the owner releases the gain at WM3.
pub(crate) fn release_command(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    project: u32,
    immediate: bool,
) -> Result<(), CommandError> {
    let index = project as usize;
    let Some(p) = state.improvement_projects.get(index) else {
        return Err(CommandError::UnknownProject);
    };
    if p.company != actor || !p.running() {
        return Err(CommandError::UnknownProject);
    }
    match p.pending {
        Some(ImpPending {
            kind: ImpPendingKind::Release,
            ..
        }) => {}
        _ => return Err(CommandError::NothingToDecide),
    }
    let date = state.date;
    let news = release(state, catalog, index, date, immediate);
    state.project_news.extend(news);
    Ok(())
}

/// Decisions left open past their deadline are taken by the leader.
fn resolve_due(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    for index in 0..state.improvement_projects.len() {
        let p = &state.improvement_projects[index];
        let Some(pending) = p.pending else {
            continue;
        };
        if !p.running() || date < pending.deadline {
            continue;
        }
        match pending.kind {
            ImpPendingKind::Choose => {
                let pick = lead_choice(state, catalog, p);
                let p = &mut state.improvement_projects[index];
                p.chosen = Some(pick);
                p.pending = None;
            }
            ImpPendingKind::Release => news.extend(release(state, catalog, index, date, false)),
        }
    }
    news
}

/// Today's step of all improvement projects: people from what the development projects left
/// free, progress, and the milestones WM1–WM3.
pub(crate) fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let Some((m, w)) = models(catalog) else {
        return Vec::new();
    };
    let mut news = resolve_due(state, catalog, date);
    for index in 0..state.improvement_projects.len() {
        if !state.improvement_projects[index].running() {
            continue;
        }
        let (company, site, phase, effort) = {
            let p = &state.improvement_projects[index];
            (p.company, p.site, usize::from(p.phase), p.effort)
        };
        if state.companies[company.index()].bankrupt {
            let p = &mut state.improvement_projects[index];
            p.status = ImpStatus::Aborted;
            p.finished = Some(date);
            continue;
        }
        if !state.improvement_projects[index].active() {
            state.improvement_projects[index].assigned = [0; 4];
            continue;
        }
        let mut assigned = [0u32; 4];
        for role in LabRole::ALL {
            let wish = need(w, m, phase, role, effort);
            let staff = &mut state.sites[site.index()].lab_staff;
            let give = wish.min(staff.free(role));
            staff.assigned[role.index()] += give;
            assigned[role.index()] = give;
        }
        let (rate, _) = speed(w, m, phase, effort, assigned);
        let days = (effort / m.reference_rate * w.phases[phase].time).max(1e-9);
        let progress = {
            let p = &mut state.improvement_projects[index];
            p.assigned = assigned;
            p.progress
        };
        let step = (rate / days).min(1.0 - progress);
        let factor = if phase == 0 {
            1.0
        } else {
            state.improvement_projects[index].cost_factor()
        };
        let cost = Money::from_usd(
            effort
                * m.cost_per_point_usd
                * w.phases[phase].cost
                * factor
                * step
                * projects::price_level(state, site),
        )
        .unwrap_or(Money::ZERO);
        if cost > Money::ZERO {
            state.companies[company.index()].ledger.expense(
                CostType::Research,
                CostCenter::site(site),
                Account::Cash,
                cost,
            );
            state.improvement_projects[index].spent += cost;
        }
        state.improvement_projects[index].progress += step;
        if state.improvement_projects[index].progress < 1.0 - 1e-9 {
            continue;
        }
        news.extend(match phase {
            0 => analysis_done(state, catalog, index, date),
            1 => optimization_done(state, catalog, index),
            _ => preparation_done(state, catalog, index, date),
        });
    }
    news
}

/// `Game` start/load: the lists stay sorted and the projects fit the data.
pub(crate) fn fit_loaded(state: &mut GameState) {
    state.improved.sort_by_key(key);
}

/// The rules of the AI: a company with a laboratory improves its best seller step by step,
/// one project at a time, with the target of the fewest stages.
pub(crate) fn ai_start(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    (site, product): (SiteId, ProductId),
) -> Option<Command> {
    let (m, w) = models(catalog)?;
    if state
        .improvement_projects
        .iter()
        .any(|p| p.running() && p.company == company)
    {
        return None;
    }
    let lead = (0..state.project_leads.len()).find(|&l| {
        state.project_leads[l].employer == Some(company)
            && busy(state, id32(l)) < m.lead.capacity_further as usize
    })?;
    let target = (0..w.targets.len())
        .min_by_key(|&t| stages_done(state, company, product, u8::try_from(t).unwrap_or(0)))?;
    Some(Command::StartImprovement {
        site,
        product,
        target: u8::try_from(target).unwrap_or(0),
        lead: id32(lead),
    })
}

use crate::command::Command;
