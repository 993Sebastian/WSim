//! Research (Lastenheft §7.1; formulas in docs/FORMELN.md, M9).
//!
//! Research centers work on one technology each; their researchers (academics of the
//! technology's field) collect research points for the company. A technology is
//! acquired once the points reach its effort, which depends on how far the research is
//! ahead of history or how long ago the technology was invented.

use crate::calendar::Date;
use crate::catalog::{Catalog, SiteType};
use crate::ids::{Id, TechnologyId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, GameState, SiteId};

/// Research points a company needs for a technology, with the reason.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Effort {
    /// Points at the historical invention year (data).
    pub base: f64,
    /// Above 1 when ahead of history, below 1 for latecomers.
    pub factor: f64,
    pub points: f64,
}

/// Effort for a technology on `date`; `None` for technologies known from the start.
pub fn effort(
    catalog: &Catalog,
    state: &GameState,
    technology: TechnologyId,
    date: Date,
) -> Option<Effort> {
    let t = catalog.technologies.get(technology);
    let base = t.research_effort?;
    let model = &catalog.research_model;
    let now = date.year_fraction();
    let historical = f64::from(t.invention_year);
    let first = state
        .inventions
        .get(technology)
        .map(|d| d.year_fraction())
        .filter(|&y| y < historical);
    let invented = first.unwrap_or(historical);
    let factor = if now < invented {
        let ahead = (historical - now) * state.settings.research_ahead_factor;
        crate::math::pow(model.ahead_base, ahead)
    } else {
        crate::math::pow(1.0 - model.latecomer_discount, now - invented).max(model.latecomer_min)
    };
    Some(Effort {
        base,
        factor,
        points: base * factor,
    })
}

/// Whether a company could start research on a technology: not yet known, all
/// prerequisites known.
pub fn can_research(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    technology: TechnologyId,
) -> bool {
    !state.knows(catalog, company, technology)
        && catalog
            .technologies
            .get(technology)
            .prerequisites
            .iter()
            .all(|&p| state.knows(catalog, company, p))
}

/// Researchers a research center wants for its current project, by labor group.
pub(crate) fn wanted_researchers(
    catalog: &Catalog,
    state: &GameState,
    site: SiteId,
    date: Date,
) -> f64 {
    let s = &state.sites[site.index()];
    if s.kind != SiteType::ResearchCenter || s.research.is_none() {
        return 0.0;
    }
    s.slots
        .iter()
        .filter(|sl| sl.operating(date))
        .map(|sl| sl.full_runs(catalog) * sl.utilization)
        .sum()
}

/// One day of research for all companies. Returns messages for the player.
pub(crate) fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let model = &catalog.research_model;
    for index in 0..state.sites.len() {
        let s = &state.sites[index];
        let Some(technology) = s.research else {
            continue;
        };
        let owner = s.owner;
        if state.companies[owner.index()].bankrupt {
            continue;
        }
        if state.knows(catalog, owner, technology) {
            state.sites[index].research = None;
            continue;
        }
        let field = catalog.technologies.get(technology).field;
        let Some(group) = model.researchers.get(field.index()).copied().flatten() else {
            continue;
        };
        let researchers = *s.workforce.get(group);
        if researchers <= 0.0 {
            continue;
        }
        let country = state.countries.get(s.country);
        let efficiency = country
            .research_efficiency
            .get(field.index())
            .copied()
            .unwrap_or(1.0);
        let material =
            Money::from_usd(researchers * model.material_usd_per_day * country.price_level)
                .unwrap_or(Money::ZERO);
        let site = SiteId(u32::try_from(index).expect("site count fits u32"));
        let company = &mut state.companies[owner.index()];
        company.ledger.expense(
            CostType::Research,
            CostCenter::site(site),
            Account::Cash,
            material,
        );
        *company.research.entry(technology).or_default() += researchers * efficiency;
    }

    let mut messages = Vec::new();
    for c in 0..state.companies.len() {
        let done: Vec<TechnologyId> = state.companies[c]
            .research
            .iter()
            .filter(|&(&t, &points)| {
                effort(catalog, state, t, date).is_some_and(|e| points >= e.points)
            })
            .map(|(&t, _)| t)
            .collect();
        for technology in done {
            let company = &mut state.companies[c];
            company.research.remove(&technology);
            company.technologies.insert(technology);
            let first = state.inventions.get_mut(technology);
            let new_to_the_world = first.is_none();
            if new_to_the_world {
                *first = Some(date);
            }
            if new_to_the_world && c != state.player.index() {
                messages.push(
                    Message::new(MessageKind::Info, keys::AI_INVENTION)
                        .with("firma", Param::Text(state.companies[c].name.clone()))
                        .with(
                            "technologie",
                            Param::TextKey(format!(
                                "technologie.{}",
                                catalog.technologies.key(technology)
                            )),
                        ),
                );
            }
            for site in &mut state.sites {
                if site.owner.index() == c && site.research == Some(technology) {
                    site.research = None;
                }
            }
            if c == state.player.index() {
                messages.push(
                    Message::new(MessageKind::Success, keys::RESEARCH_DONE).with(
                        "technologie",
                        Param::TextKey(format!(
                            "technologie.{}",
                            catalog.technologies.key(technology)
                        )),
                    ),
                );
            }
        }
    }
    messages
}
