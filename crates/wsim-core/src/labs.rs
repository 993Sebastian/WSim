//! Projects per laboratory (LB1, docs/FORMELN.md): a laboratory of a research center may
//! have a project of its own; the others work on the center's project. The researchers
//! are split by the labs' weights; points belong to the company, so equal projects add up.

use crate::calendar::Date;
use crate::catalog::{Catalog, SiteType};
use crate::ids::{ProductId, SpecializationId, TechnologyId};
use crate::message::{Message, MessageKind, Param, keys};
use crate::state::{GameState, LabWork, Site, SiteId};
use crate::{development, product_research};

/// Weight of a laboratory: its researcher posts at the planned utilization.
fn lab_weight(catalog: &Catalog, slot: &crate::state::Slot, date: Date) -> f64 {
    if slot.operating(date) {
        slot.full_runs(catalog) * slot.utilization
    } else {
        0.0
    }
}

/// Share of the researchers for the center's own project (`w0`) and for each laboratory
/// with a project of its own (slot, project, `w`).
pub struct Weights {
    pub center: f64,
    pub labs: Vec<(usize, LabWork, f64)>,
}

pub fn weights(catalog: &Catalog, site: &Site, date: Date) -> Weights {
    if site.kind != SiteType::ResearchCenter || site.slots.iter().all(|s| s.work.is_none()) {
        return Weights {
            center: 1.0,
            labs: Vec::new(),
        };
    }
    let total: f64 = site
        .slots
        .iter()
        .map(|s| lab_weight(catalog, s, date))
        .sum();
    if total <= 0.0 {
        return Weights {
            center: 1.0,
            labs: Vec::new(),
        };
    }
    let mut center = 0.0;
    let mut labs = Vec::new();
    for (i, s) in site.slots.iter().enumerate() {
        let w = lab_weight(catalog, s, date) / total;
        match s.work {
            Some(work) => labs.push((i, work, w)),
            None => center += w,
        }
    }
    Weights { center, labs }
}

/// Share of the center's researchers working in one laboratory (all labs count, with or
/// without a project of their own).
pub fn lab_share(catalog: &Catalog, site: &Site, slot: usize, date: Date) -> f64 {
    let total: f64 = site
        .slots
        .iter()
        .map(|s| lab_weight(catalog, s, date))
        .sum();
    if total <= 0.0 {
        return 0.0;
    }
    site.slots
        .get(slot)
        .map_or(0.0, |s| lab_weight(catalog, s, date) / total)
}

/// Research field of a laboratory's own project.
fn work_field(catalog: &Catalog, work: LabWork) -> Option<SpecializationId> {
    match work {
        LabWork::Technology(t)
        | LabWork::Product {
            technology: Some(t),
            ..
        } => Some(catalog.technologies.get(t).field),
        LabWork::Product { product, .. } | LabWork::Development(product) => {
            development::basis(catalog, product).map(|(field, _)| field)
        }
    }
}

/// Researcher posts of a research center by research field: each laboratory in the field
/// of its own project, the others in that of the center's.
pub(crate) fn posts(
    catalog: &Catalog,
    state: &GameState,
    site: SiteId,
    date: Date,
) -> Vec<(SpecializationId, f64)> {
    let mut out: Vec<(SpecializationId, f64)> = Vec::new();
    let Some(s) = state.site(site) else {
        return out;
    };
    if s.kind != SiteType::ResearchCenter {
        return out;
    }
    // PD1: the hired people fill the posts (all of them without roles in the data).
    let staffed = crate::lab_staff::research_share(catalog, s, date);
    let center_field = development::project_field(catalog, state, site)
        .filter(|_| !development::idle(s) || development::learning(state, s));
    for slot in s.slots.iter().filter(|sl| sl.operating(date)) {
        let field = match slot.work {
            Some(work) => work_field(catalog, work),
            None => center_field,
        };
        let Some(field) = field else {
            continue;
        };
        let posts = slot.full_runs(catalog) * slot.utilization * staffed;
        match out.iter_mut().find(|(f, _)| *f == field) {
            Some(e) => e.1 += posts,
            None => out.push((field, posts)),
        }
    }
    out
}

/// Whether nothing at the center is worked on: no project of the center, no lab project.
pub fn all_idle(state: &GameState, site: &Site) -> bool {
    development::idle(site)
        && !development::learning(state, site)
        && site.slots.iter().all(|s| s.work.is_none())
}

/// Technologies researched with their shares of the center's researchers.
pub fn technology_shares(catalog: &Catalog, site: &Site, date: Date) -> Vec<(TechnologyId, f64)> {
    let w = weights(catalog, site, date);
    let mut out = Vec::new();
    if let Some(t) = site.research {
        out.push((t, development::technology_share(site) * w.center));
    }
    for (_, work, share) in &w.labs {
        match work {
            LabWork::Technology(t) => out.push((*t, *share)),
            LabWork::Product {
                technology: Some(t),
                ..
            } => out.push((*t, *share)),
            _ => {}
        }
    }
    out
}

fn merge(out: &mut Vec<(ProductId, f64)>, product: ProductId, share: f64) {
    match out.iter_mut().find(|(p, _)| *p == product) {
        Some(e) => e.1 += share,
        None => out.push((product, share)),
    }
}

/// Products developed with their shares of the center's researchers (Q1).
pub fn development_shares(catalog: &Catalog, site: &Site, date: Date) -> Vec<(ProductId, f64)> {
    let w = weights(catalog, site, date);
    let mut out: Vec<(ProductId, f64)> = Vec::new();
    for (p, s) in development::site_shares(site) {
        merge(&mut out, p, s * w.center);
    }
    for (_, work, share) in &w.labs {
        if let LabWork::Development(p) = work {
            merge(&mut out, *p, share * 0.5);
        }
    }
    out
}

/// Products with manufacturing research and their shares of the researchers (Q6).
pub fn process_shares(catalog: &Catalog, site: &Site, date: Date) -> Vec<(ProductId, f64)> {
    let w = weights(catalog, site, date);
    let mut out: Vec<(ProductId, f64)> = Vec::new();
    for (p, s) in development::site_process_shares(site) {
        merge(&mut out, p, s * w.center);
    }
    for (_, work, share) in &w.labs {
        if let LabWork::Development(p) = work {
            merge(&mut out, *p, share * 0.5);
        }
    }
    out
}

/// Products learned under mandatory research (FW1): all their technologies are known.
pub fn learn_shares(
    catalog: &Catalog,
    state: &GameState,
    site: &Site,
    date: Date,
) -> Vec<(ProductId, f64)> {
    let w = weights(catalog, site, date);
    let owner = site.owner;
    let ready = |p: ProductId| product_research::missing(catalog, state, owner, p).is_empty();
    let mut out = Vec::new();
    if let (Some(p), None) = (site.research_product, site.research)
        && ready(p)
    {
        merge(&mut out, p, development::free_share(site) * w.center);
    }
    for (_, work, share) in &w.labs {
        if let LabWork::Product {
            product,
            technology: None,
        } = work
            && ready(*product)
        {
            merge(&mut out, *product, *share);
        }
    }
    out
}

/// Every day before the research: finished lab projects end, a lab researching a product
/// takes up the next technology for it.
pub(crate) fn steer(state: &mut GameState, catalog: &Catalog) -> Vec<Message> {
    let mut news = Vec::new();
    for index in 0..state.sites.len() {
        if state.sites[index].slots.iter().all(|s| s.work.is_none()) {
            continue;
        }
        let owner = state.sites[index].owner;
        for i in 0..state.sites[index].slots.len() {
            let Some(work) = state.sites[index].slots[i].work else {
                continue;
            };
            let next = match work {
                LabWork::Technology(t) => state.knows(catalog, owner, t).then_some(None),
                LabWork::Development(p) => {
                    (!development::can_develop(catalog, state, owner, p)).then_some(None)
                }
                LabWork::Product {
                    product,
                    technology,
                } => {
                    if product_research::known(catalog, state, owner, product) {
                        if state.is_main(owner) {
                            news.push(
                                Message::new(MessageKind::Success, keys::PRODUCT_RESEARCHED).with(
                                    "produkt",
                                    Param::TextKey(format!(
                                        "produkt.{}",
                                        catalog.products.key(product)
                                    )),
                                ),
                            );
                        }
                        Some(None)
                    } else if technology.is_none_or(|t| state.knows(catalog, owner, t)) {
                        let tech = product_research::next(catalog, state, owner, product);
                        (tech != technology).then_some(Some(LabWork::Product {
                            product,
                            technology: tech,
                        }))
                    } else {
                        None
                    }
                }
            };
            if let Some(change) = next {
                state.sites[index].slots[i].work = change;
            }
        }
    }
    news
}
