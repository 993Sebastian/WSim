//! Thresholds of the person's holdings in listed companies and the mandatory offer
//! (PV5a, docs/FORMELN.md).

use crate::catalog::{Catalog, StakeRules};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, GameState, Holder};

fn rules(catalog: &Catalog) -> Option<&StakeRules> {
    catalog
        .person
        .assets
        .as_ref()
        .and_then(|a| a.stakes.as_ref())
}

/// How many thresholds a share reaches.
pub fn level(rules: &StakeRules, share: f64) -> u32 {
    let n = rules
        .thresholds
        .iter()
        .filter(|&&t| share + 1e-9 >= t)
        .count();
    u32::try_from(n).unwrap_or(0)
}

/// The mandatory offer a purchase of `share` from `holder` triggers: the share of the free
/// float taken and what it costs, if the person crosses the threshold of a listed company.
pub(crate) fn mandatory_offer(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    holder: Holder,
    share: f64,
) -> Option<(f64, Money)> {
    let r = rules(catalog)?;
    let c = state.company(company)?;
    let listing = c.listing.as_ref()?;
    let before = crate::person::share(state, company);
    if before >= r.offer_from || before + share < r.offer_from {
        return None;
    }
    let float = crate::stock::stake(c, Holder::Investors)
        - if holder == Holder::Investors {
            share
        } else {
            0.0
        };
    let taken = (float.max(0.0) * r.offer_acceptance).min(1.0);
    if taken <= 0.0 {
        return None;
    }
    let cost = listing.value.scale(taken * (1.0 + r.offer_premium));
    Some((taken, cost))
}

/// At a month end: new thresholds become public, the market reacts.
pub(crate) fn month_end(state: &mut GameState, catalog: &Catalog) -> Vec<Message> {
    let mut news = Vec::new();
    let Some(r) = rules(catalog).cloned() else {
        return news;
    };
    if state.person.name.is_empty() {
        return news;
    }
    for i in 0..state.companies.len() {
        // Few companies; the cast is exact.
        let id = CompanyId(i as u32);
        let c = &state.companies[i];
        if c.bankrupt || c.listing.is_none() {
            state.person.disclosed.remove(&id);
            continue;
        }
        let share = crate::person::share(state, id);
        let k = level(&r, share);
        let known = state.person.disclosed.get(&id).copied().unwrap_or(0);
        if k > known {
            let reached = r.thresholds[(k - 1) as usize];
            if let Some(&premium) = r.announce_premium.get((k - 1) as usize)
                && let Some(l) = state.companies[i].listing.as_mut()
            {
                l.value = l.value.scale(1.0 + premium);
            }
            let name = state.companies[i].name.clone();
            let kind = if reached >= 0.5 {
                MessageKind::Success
            } else {
                MessageKind::Info
            };
            news.push(
                Message::new(kind, keys::STAKE_THRESHOLD)
                    .with("firma", Param::Text(name.clone()))
                    .with("anteil", Param::Number((reached * 1000.0).round() / 10.0)),
            );
            if known
                < r.thresholds
                    .iter()
                    .filter(|&&t| t < r.blocking + 1e-9)
                    .count() as u32
                && reached + 1e-9 >= r.blocking
                && known_below(&r, known, r.blocking)
            {
                news.push(
                    Message::new(MessageKind::Info, keys::STAKE_BLOCKING)
                        .with("firma", Param::Text(name))
                        .with("anteil", Param::Number((share * 1000.0).round() / 10.0)),
                );
            }
        }
        if k == 0 {
            state.person.disclosed.remove(&id);
        } else {
            state.person.disclosed.insert(id, k);
        }
    }
    news
}

/// Whether the stage already announced lies below the blocking minority.
fn known_below(r: &StakeRules, known: u32, blocking: f64) -> bool {
    known == 0 || r.thresholds[(known - 1) as usize] + 1e-9 < blocking
}
