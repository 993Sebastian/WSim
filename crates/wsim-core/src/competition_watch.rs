//! The strategy department watches the competition (MG9, docs/FORMELN.md, MG9): in the
//! countries it observes and in those where the company has sites it reports what other
//! companies built in the last month.

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::message::{Message, MessageKind, Param, keys};
use crate::state::{CompanyId, GameState};

/// The new sites of other companies in the observed countries and in the countries of the
/// company's own sites, founded since the last month start, as one message each. Only the
/// player's main company has a department that reports.
pub(crate) fn month_start(state: &GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let Some(me) = state.main_company else {
        return news;
    };
    if !catalog.management.enabled() || state.companies[me.index()].bankrupt {
        return news;
    }
    let observed = crate::central::observed_countries(catalog, state, me);
    if observed.is_empty() {
        return news;
    }
    let own: std::collections::BTreeSet<_> = state
        .sites
        .iter()
        .filter(|s| s.held_by(me))
        .map(|s| s.country)
        .collect();
    let since = date.add_months(0).first_of_month();
    let from = previous_month(since);
    for site in &state.sites {
        let other = site.owner;
        if other == me || site.closed.is_some() {
            continue;
        }
        if site.founded < from || site.founded >= since {
            continue;
        }
        if !(observed.contains(&site.country) || own.contains(&site.country)) {
            continue;
        }
        news.push(report(state, catalog, other, site));
    }
    news
}

fn previous_month(first: Date) -> Date {
    first.add_days(-1).first_of_month()
}

fn report(
    state: &GameState,
    catalog: &Catalog,
    other: CompanyId,
    site: &crate::state::Site,
) -> Message {
    Message::new(MessageKind::Info, keys::COMPETITION_BUILT)
        .with(
            "firma",
            Param::Text(state.companies[other.index()].name.clone()),
        )
        .with(
            "land",
            Param::Country(catalog.countries.key(site.country).to_owned()),
        )
        .with(
            "art",
            Param::TextKey(crate::command::site_type_key(site.kind)),
        )
}
