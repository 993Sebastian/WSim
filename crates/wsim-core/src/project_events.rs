//! Events in the phases of a development project, hidden defects and recalls (PD4,
//! docs/PRODUKTENTWICKLUNG.md §6; formulas in docs/FORMELN.md, PD4).
//!
//! While a project works, a problem or a chance can turn up. The project waits until the
//! owner chose an option, or the leader chose his recommendation. What a project buys with
//! conditions, fast tempo and thin staffing adds to a hidden risk of a defect that shows
//! only after the release, as a recall.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, ProjectEventModel};
use crate::command::CommandError;
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::project_reports::DelegationMode;
use crate::projects::{self, DevProject};
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, GameState};

/// An event the project waits to be decided.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingEvent {
    /// Index into the catalogue's kinds.
    pub kind: u16,
    pub raised: Date,
    pub deadline: Date,
    pub phase: u8,
}

/// A recall of a released product with a hidden defect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recall {
    pub company: CompanyId,
    pub project: u32,
    pub due: Date,
}

fn model(catalog: &Catalog) -> Option<&ProjectEventModel> {
    catalog.research_model.projects.as_ref()?.events.as_ref()
}

fn project_id(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

/// Whether the owner decides the events of the project (not the leader).
fn owner_decides(state: &GameState, p: &DevProject) -> bool {
    let mode = p
        .delegation
        .or_else(|| state.project_delegation.get(&p.company).copied())
        .unwrap_or(DelegationMode::Standard);
    state.is_main(p.company) && mode != DelegationMode::LeadDecides
}

/// The hidden risk of a defect (0–cap): what the options, the conditions, the tempo and
/// the thin staffing of the finished phases added.
pub fn defect_risk(m: &ProjectEventModel, p: &DevProject) -> f64 {
    let tempo = m
        .defect_per_tempo
        .get(usize::from(p.tempo))
        .copied()
        .unwrap_or(0.0);
    let shortfall = p.shortfall_sum / 6.0;
    (p.defect
        + m.defect_per_condition * f64::from(p.conditions)
        + tempo
        + m.defect_per_shortfall * shortfall)
        .clamp(0.0, m.defect_cap)
}

/// Rolls today's event of a working project: `step` is today's share of the phase,
/// `shortfall` the staffing gap today.
pub(crate) fn roll(
    state: &mut GameState,
    catalog: &Catalog,
    index: usize,
    date: Date,
    step: f64,
    shortfall: f64,
) -> Vec<Message> {
    let (Some(m), Some(pm)) = (model(catalog), catalog.research_model.projects.as_ref()) else {
        return Vec::new();
    };
    let p = &state.dev_projects[index];
    let phase = p.phase;
    if p.event.is_some() || p.event_phase == Some(phase) || step <= 0.0 {
        return Vec::new();
    }
    let experience = state
        .project_leads
        .get(p.lead as usize)
        .map_or(0.0, |l| l.experience);
    let f = projects::factors(pm, p);
    let risk = projects::phase_risk(pm, usize::from(p.tempo), shortfall, experience) * f.risk;
    let chance = (risk * step).min(1.0);
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::ProjectEvent {
            project: project_id(index),
            day: crate::management::day_number(date),
        },
    );
    if !rng.chance(chance) {
        return Vec::new();
    }
    let total: f64 = m
        .kinds
        .iter()
        .filter(|k| k.phases.contains(&phase))
        .map(|k| k.weight)
        .sum();
    if total <= 0.0 {
        return Vec::new();
    }
    let mut pick = rng.next_f64() * total;
    let mut kind = None;
    for (i, k) in m.kinds.iter().enumerate() {
        if !k.phases.contains(&phase) {
            continue;
        }
        kind = Some(i);
        if pick < k.weight {
            break;
        }
        pick -= k.weight;
    }
    let Some(kind) = kind else {
        return Vec::new();
    };
    state.dev_projects[index].event_phase = Some(phase);
    if owner_decides(state, &state.dev_projects[index]) {
        let deadline = date.add_days(i32::try_from(m.deadline_days).unwrap_or(7));
        state.dev_projects[index].event = Some(PendingEvent {
            kind: u16::try_from(kind).unwrap_or(0),
            raised: date,
            deadline,
            phase,
        });
        return vec![
            Message::new(MessageKind::Warning, keys::PROJECT_EVENT)
                .with("produkt", product_param(state, catalog, index))
                .with(
                    "ereignis",
                    Param::TextKey(format!("projektereignis.{}.titel", m.kinds[kind].key)),
                ),
        ];
    }
    let recommended = m.kinds[kind].recommended;
    apply(state, catalog, index, kind, recommended, false)
}

fn product_param(state: &GameState, catalog: &Catalog, index: usize) -> Param {
    let product = state.dev_projects[index].product;
    Param::TextKey(format!("produkt.{}", catalog.products.key(product)))
}

/// Carries out the option: cost, delay, quality and hidden defect.
fn apply(
    state: &mut GameState,
    catalog: &Catalog,
    index: usize,
    kind: usize,
    option: usize,
    by_player: bool,
) -> Vec<Message> {
    let (Some(m), Some(pm)) = (model(catalog), catalog.research_model.projects.as_ref()) else {
        return Vec::new();
    };
    let Some(k) = m.kinds.get(kind) else {
        return Vec::new();
    };
    let Some(o) = k.options.get(option) else {
        return Vec::new();
    };
    let (phase, tempo, trim, effort, site) = {
        let p = &state.dev_projects[index];
        (
            usize::from(p.phase),
            usize::from(p.tempo),
            usize::from(p.trim),
            p.effort,
            p.site,
        )
    };
    let f = projects::factors(pm, &state.dev_projects[index]);
    let cost = Money::from_usd(
        o.cost
            * projects::phase_cost_usd(pm, (tempo, trim), phase, effort)
            * f.cost
            * projects::price_level(state, site),
    )
    .unwrap_or(Money::ZERO);
    if cost > Money::ZERO {
        projects::book(state, index, cost);
    }
    let p = &mut state.dev_projects[index];
    p.event = None;
    p.progress = (p.progress - o.delay).clamp(-1.0, 0.99);
    p.bonus_quality += o.quality;
    p.defect += o.defect;
    if !state.is_main(state.dev_projects[index].company) {
        return Vec::new();
    }
    vec![
        Message::new(
            if by_player {
                MessageKind::Info
            } else {
                MessageKind::Warning
            },
            if by_player {
                keys::PROJECT_EVENT_DECIDED
            } else {
                keys::PROJECT_EVENT_AUTO
            },
        )
        .with("produkt", product_param(state, catalog, index))
        .with(
            "ereignis",
            Param::TextKey(format!("projektereignis.{}.titel", k.key)),
        )
        .with(
            "option",
            Param::TextKey(format!("projektereignis.{}.{}", k.key, o.key)),
        ),
    ]
}

/// `DecideEvent`: the owner chooses an option of the pending event.
pub(crate) fn decide(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    project: u32,
    option: u8,
) -> Result<(), CommandError> {
    let Some(m) = model(catalog) else {
        return Err(CommandError::ProjectsOff);
    };
    let index = project as usize;
    let Some(p) = state.dev_projects.get(index) else {
        return Err(CommandError::UnknownProject);
    };
    if p.company != actor || !p.running() {
        return Err(CommandError::UnknownProject);
    }
    let Some(pending) = p.event else {
        return Err(CommandError::NothingToDecide);
    };
    let kind = usize::from(pending.kind);
    if usize::from(option) >= m.kinds[kind].options.len() {
        return Err(CommandError::InvalidQuantity);
    }
    let news = apply(state, catalog, index, kind, usize::from(option), true);
    state.project_news.extend(news);
    Ok(())
}

/// Every day: events left open past their deadline are decided by the leader.
pub(crate) fn resolve_due(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let Some(m) = model(catalog) else {
        return Vec::new();
    };
    let mut news = Vec::new();
    for index in 0..state.dev_projects.len() {
        let p = &state.dev_projects[index];
        let Some(pending) = p.event else {
            continue;
        };
        if !p.running() || date < pending.deadline {
            continue;
        }
        let kind = usize::from(pending.kind);
        news.extend(apply(
            state,
            catalog,
            index,
            kind,
            m.kinds[kind].recommended,
            false,
        ));
    }
    news
}

/// At the release: with the hidden risk the product has a defect, and a recall is due.
pub(crate) fn on_release(state: &mut GameState, catalog: &Catalog, index: usize, date: Date) {
    let Some(m) = model(catalog) else {
        return;
    };
    let risk = defect_risk(m, &state.dev_projects[index]);
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::ProjectDefect {
            project: project_id(index),
        },
    );
    if !rng.chance(risk) {
        return;
    }
    let (lo, hi) = m.recall_days;
    let span = hi.saturating_sub(lo) + 1;
    let days = lo + (rng.next_f64() * f64::from(span)) as u32;
    let company = state.dev_projects[index].company;
    state.recalls.push(Recall {
        company,
        project: project_id(index),
        due: date.add_days(i32::try_from(days).unwrap_or(i32::MAX)),
    });
}

/// Every day: recalls that are due cost money and brand awareness.
pub(crate) fn run_recalls(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let Some(m) = model(catalog) else {
        return Vec::new();
    };
    let mut news = Vec::new();
    let mut open = Vec::new();
    for recall in std::mem::take(&mut state.recalls) {
        if recall.due > date {
            open.push(recall);
            continue;
        }
        let Some(p) = state.dev_projects.get(recall.project as usize) else {
            continue;
        };
        let (company, product, spent) = (p.company, p.product, p.spent);
        let c = &mut state.companies[company.index()];
        if c.bankrupt {
            continue;
        }
        let cost = Money::from_usd(spent.to_usd() * m.recall_cost_share).unwrap_or(Money::ZERO);
        c.ledger.expense(
            CostType::Other,
            CostCenter {
                site: None,
                product: Some(product),
            },
            Account::Cash,
            cost,
        );
        let group = catalog.products.get(product).goods_group;
        for b in c.brands.iter_mut().filter(|b| b.group == group) {
            b.awareness *= 1.0 - m.recall_awareness_loss;
        }
        if state.is_main(company) {
            news.push(
                Message::new(MessageKind::Warning, keys::PROJECT_RECALL)
                    .with(
                        "produkt",
                        Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
                    )
                    .with("kosten", Param::Money(cost)),
            );
        }
    }
    state.recalls = open;
    news
}
