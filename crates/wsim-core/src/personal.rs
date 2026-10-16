//! Relationship, personal events and nepotism of the person (PV6b, docs/FORMELN.md).

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, EventCondition, PersonalModel};
use crate::command::CommandError;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, GameState, LifeEvent, LifeEventKind, PrivateFlow};

/// Offset of the stream of the events among the streams of the person.
const EVENT_STREAM: u32 = 7 << 24;

/// The kind of the event about the marriage contract (it is no entry of the data).
pub const CONTRACT: u32 = u32::MAX;

/// An event waiting for the answer of the person.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingEvent {
    pub id: u32,
    /// Index in the data, or `CONTRACT`.
    pub kind: u32,
    pub since: Date,
    pub until: Date,
}

/// The relationship and the events of the person.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Life {
    /// The relationship 0–100 while married; `None` until the first month of it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bond: Option<f64>,
    /// A marriage contract was made.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub contract: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pending: Vec<PendingEvent>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub next_id: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

impl Life {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

pub fn model(catalog: &Catalog) -> Option<&PersonalModel> {
    catalog.person.personal.as_ref()
}

/// The relationship now; none without a partner.
pub fn bond(catalog: &Catalog, state: &GameState) -> Option<f64> {
    let m = model(catalog)?;
    state
        .person
        .married
        .then(|| state.person.life.bond.unwrap_or(m.bond_start))
}

/// The person married: the relationship begins and the question of the contract comes.
pub(crate) fn married(state: &mut GameState, catalog: &Catalog, date: Date) {
    state.person.history.push(LifeEvent {
        date,
        kind: LifeEventKind::Married,
    });
    let Some(m) = model(catalog) else {
        return;
    };
    state.person.life.bond = Some(m.bond_start);
    state.person.life.contract = false;
    let id = state.person.life.next_id;
    state.person.life.next_id += 1;
    let until = date.add_days(i32::try_from(m.wait_days).unwrap_or(60));
    state.person.life.pending.push(PendingEvent {
        id,
        kind: CONTRACT,
        since: date,
        until,
    });
}

/// `AnswerEvent`: the person chooses an option of a waiting event.
pub(crate) fn answer(
    state: &mut GameState,
    catalog: &Catalog,
    event: u32,
    option: u32,
) -> Result<(), CommandError> {
    let m = model(catalog).ok_or(CommandError::NotPossible)?;
    let at = state
        .person
        .life
        .pending
        .iter()
        .position(|e| e.id == event)
        .ok_or(CommandError::NotPossible)?;
    let pending = state.person.life.pending[at];
    resolve(state, catalog, m, pending, option)?;
    state.person.life.pending.remove(at);
    Ok(())
}

fn resolve(
    state: &mut GameState,
    catalog: &Catalog,
    m: &PersonalModel,
    pending: PendingEvent,
    option: u32,
) -> Result<(), CommandError> {
    let wage = crate::private::academic_monthly_wage(catalog, state, state.person.home);
    if pending.kind == CONTRACT {
        // 0: with a contract, 1: without.
        match option {
            0 => {
                let cost = wage.scale(m.contract_cost);
                pay(state, cost)?;
                state.person.life.contract = true;
            }
            1 => {}
            _ => return Err(CommandError::NotPossible),
        }
        state.person.history.push(LifeEvent {
            date: state.date,
            kind: LifeEventKind::Personal {
                key: "ehevertrag".into(),
                option: if option == 0 { "mit" } else { "ohne" }.into(),
            },
        });
        return Ok(());
    }
    let terms = m
        .events
        .get(pending.kind as usize)
        .ok_or(CommandError::NotPossible)?;
    let o = terms
        .options
        .get(option as usize)
        .ok_or(CommandError::NotPossible)?;
    pay(state, wage.scale(o.cost))?;
    if o.bond != 0.0 && state.person.married {
        let now = state.person.life.bond.unwrap_or(m.bond_start);
        state.person.life.bond = Some((now + o.bond).clamp(0.0, 100.0));
    }
    if o.standing != 0.0 && crate::standing::model(catalog).is_some() {
        let now = crate::standing::standing(catalog, state);
        state.person.circle.standing = Some((now + o.standing).clamp(0.0, 100.0));
    }
    if o.health != 0.0
        && let Some(h) = crate::wellbeing::health(catalog, state)
    {
        state.person.health = Some((h + o.health).clamp(0.0, 100.0));
    }
    if o.interest != 0.0 {
        for c in &mut state.person.children {
            // Within 0–100; the casts are exact.
            c.interest = (f64::from(c.interest) + o.interest)
                .clamp(0.0, 100.0)
                .round() as u32;
        }
    }
    if o.loyalty != 0.0 {
        shift_satisfaction(state, o.loyalty);
    }
    if o.gain > 0.0 {
        // An insider trade: a gain, or a fine and a loss of standing if it is found out.
        let mut rng = SimRng::for_stream(
            state.settings.seed,
            Stream::Person {
                month: EVENT_STREAM
                    + (1 << 22)
                    + (pending.id % 512) * 8192
                    + crate::management::month_number(state.date) % 8192,
            },
        );
        if rng.chance(o.detect) {
            let fine = wage.scale(o.gain * o.fine);
            crate::private::book_flow_floor(state, PrivateFlow::Personal, -fine);
            if crate::standing::model(catalog).is_some() {
                let now = crate::standing::standing(catalog, state);
                state.person.circle.standing = Some((now - o.shame).clamp(0.0, 100.0));
            }
            state.person.history.push(LifeEvent {
                date: state.date,
                kind: LifeEventKind::Personal {
                    key: terms.key.clone(),
                    option: "entdeckt".into(),
                },
            });
        } else {
            crate::private::book_flow(state, PrivateFlow::Personal, wage.scale(o.gain));
        }
    }
    state.person.history.push(LifeEvent {
        date: state.date,
        kind: LifeEventKind::Personal {
            key: terms.key.clone(),
            option: o.key.clone(),
        },
    });
    Ok(())
}

/// The satisfaction of the managers of the companies the person leads changes by points.
fn shift_satisfaction(state: &mut GameState, points: f64) {
    let controlled: Vec<CompanyId> = (0..state.companies.len())
        .map(|i| CompanyId(u32::try_from(i).unwrap_or(u32::MAX)))
        .filter(|&c| crate::private::is_controlled(state, c))
        .collect();
    for m in state.managers.values_mut() {
        if let Some(j) = m.job.as_mut()
            && controlled.contains(&j.company)
            && let Some(s) = j.satisfaction
        {
            // Within 0–100; the cast is exact.
            j.satisfaction = Some((f64::from(s) + points).clamp(0.0, 100.0).round() as u8);
        }
    }
}

fn pay(state: &mut GameState, cost: Money) -> Result<(), CommandError> {
    if cost <= Money::ZERO {
        return Ok(());
    }
    if state.person.account.balance < cost {
        return Err(CommandError::NotEnoughPrivateMoney {
            needed: cost - state.person.account.balance,
        });
    }
    crate::private::book_flow(state, PrivateFlow::Personal, -cost);
    Ok(())
}

fn condition_met(
    state: &GameState,
    catalog: &Catalog,
    m: &PersonalModel,
    c: EventCondition,
) -> bool {
    match c {
        EventCondition::Always => true,
        EventCondition::Married => state.person.married,
        EventCondition::Children => state.person.children.iter().any(|k| k.died.is_none()),
        EventCondition::LowHealth => {
            crate::wellbeing::health(catalog, state).is_some_and(|h| h < m.low_health)
        }
        EventCondition::Leader => (0..state.companies.len()).any(|i| {
            let id = CompanyId(u32::try_from(i).unwrap_or(u32::MAX));
            !state.companies[i].bankrupt && crate::private::is_controlled(state, id)
        }),
        EventCondition::Tenants => state.person.portfolio.properties.iter().any(|p| {
            matches!(
                p.kind,
                crate::property::PropertyKind::Apartment | crate::property::PropertyKind::Multi
            )
        }),
    }
}

/// At a month start: unanswered events lapse, the relationship changes, it ends in
/// separation, new events happen.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let Some(m) = model(catalog) else {
        return news;
    };
    if state.person.name.is_empty() {
        return news;
    }
    let before = state.person.life.bond.unwrap_or(m.bond_start);
    // Unanswered: the last option counts (the quiet one).
    let (lapsed, kept): (Vec<PendingEvent>, Vec<PendingEvent>) =
        std::mem::take(&mut state.person.life.pending)
            .into_iter()
            .partition(|e| e.until <= date);
    state.person.life.pending = kept;
    for e in lapsed {
        let last = if e.kind == CONTRACT {
            1
        } else {
            m.events.get(e.kind as usize).map_or(0, |t| {
                u32::try_from(t.options.len().saturating_sub(1)).unwrap_or(0)
            })
        };
        // An option that costs money the person lacks is left out: nothing happens.
        let _ = resolve(state, catalog, m, e, last);
    }
    // The relationship follows the time for the family.
    if state.person.married {
        let family =
            crate::wellbeing::time(catalog, state).map_or(m.bond_holds_at, |t| f64::from(t.family));
        let now = state.person.life.bond.unwrap_or(m.bond_start);
        let next = (now + m.bond_rate * (family - m.bond_holds_at) / 10.0).clamp(0.0, 100.0);
        state.person.life.bond = Some(next);
        if before >= m.bond_warn && next < m.bond_warn {
            news.push(Message::new(MessageKind::Warning, keys::BOND_WARNING));
        }
        if next < m.bond_separate {
            news.push(separate(state, catalog, m, date));
        }
    }
    // New events.
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::Person {
            month: crate::management::month_number(date) + EVENT_STREAM,
        },
    );
    for (i, t) in m.events.iter().enumerate() {
        let kind = u32::try_from(i).unwrap_or(u32::MAX);
        let waiting = u32::try_from(state.person.life.pending.len()).unwrap_or(u32::MAX);
        if waiting >= m.pending_max
            || state.person.life.pending.iter().any(|e| e.kind == kind)
            || !condition_met(state, catalog, m, t.condition)
        {
            continue;
        }
        let yearly = (t.chance_year * m.chance_factor).clamp(0.0, 1.0);
        let monthly = 1.0 - crate::math::pow(1.0 - yearly, 1.0 / 12.0);
        if !rng.chance(monthly) {
            continue;
        }
        let id = state.person.life.next_id;
        state.person.life.next_id += 1;
        state.person.life.pending.push(PendingEvent {
            id,
            kind,
            since: date,
            until: date.add_days(i32::try_from(m.wait_days).unwrap_or(60)),
        });
        news.push(Message::new(MessageKind::Info, keys::PERSONAL_EVENT).with(
            "ereignis",
            Param::TextKey(format!("lebensereignis.{}", t.key)),
        ));
    }
    news
}

/// The couple separates: the partner gets a share of the free wealth.
fn separate(state: &mut GameState, catalog: &Catalog, m: &PersonalModel, date: Date) -> Message {
    let share = if state.person.life.contract {
        m.share_with
    } else {
        m.share_without
    };
    let w = crate::private::wealth(catalog, state);
    let claim = (w.cash + w.safe + w.gold + w.funds).scale(share);
    crate::savings::liquidate(state, catalog, claim);
    let paid = claim.min(state.person.account.balance);
    if paid > Money::ZERO {
        crate::private::book_flow(state, PrivateFlow::Personal, -paid);
    }
    let p = &mut state.person;
    p.married = false;
    p.life.bond = None;
    p.life.contract = false;
    p.life.pending.retain(|e| e.kind != CONTRACT);
    p.history.push(LifeEvent {
        date,
        kind: LifeEventKind::Separated { paid },
    });
    Message::new(MessageKind::Warning, keys::SEPARATED).with("betrag", Param::Money(paid))
}

/// A relative takes a position as head: the other managers of the company and the units
/// below lose satisfaction when he is weaker than their mean (PV6b).
pub(crate) fn nepotism(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    manager: crate::state::ManagerId,
) {
    let Some(m) = model(catalog) else {
        return;
    };
    let Some(x) = state.managers.get(&manager) else {
        return;
    };
    if m.nepotism_malus == 0 || !x.family || !crate::private::is_controlled(state, company) {
        return;
    }
    let own = crate::management::strength(x);
    let others: Vec<f64> = state
        .managers
        .iter()
        .filter(|(id, y)| {
            **id != manager && !y.family && y.job.as_ref().is_some_and(|j| j.company == company)
        })
        .map(|(_, y)| crate::management::strength(y))
        .collect();
    if others.is_empty() {
        return;
    }
    // Few managers; the cast is exact.
    #[allow(clippy::cast_precision_loss)]
    let mean = others.iter().sum::<f64>() / others.len() as f64;
    if own >= mean {
        return;
    }
    let malus = f64::from(m.nepotism_malus);
    for (&id, y) in state.managers.iter_mut() {
        if id == manager || y.family {
            continue;
        }
        if let Some(j) = y.job.as_mut()
            && j.company == company
            && let Some(s) = j.satisfaction
        {
            j.satisfaction = Some((f64::from(s) - malus).clamp(0.0, 100.0).round() as u8);
        }
    }
}
