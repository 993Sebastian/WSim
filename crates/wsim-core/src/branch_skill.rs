//! Industry experience of managers and its effect in the works (MG5; docs/FORMELN.md MG5).
//!
//! The manager responsible for production at a works (the specialist, else the head with
//! part of the effect) lends his strength in the works' branch to its production: fewer
//! labour hours, less scrap, less material. The strength is set at each month start and
//! kept on the site, so the effect is a plain factor the production reads.

use std::collections::BTreeMap;

use crate::catalog::Catalog;
use crate::ids::BranchId;
use crate::state::{GameState, Manager, Role, SiteId, Unit};

/// Key of the production function among the specialists.
const PRODUCTION: &str = "produktion";

/// The branch of a works: the fixed one (FW1), else that of its first product (MG5).
pub fn site_branch(catalog: &Catalog, site: &crate::state::Site) -> Option<BranchId> {
    site.branch.or_else(|| {
        site.slots
            .iter()
            .find_map(|s| s.recipe)
            .map(|r| catalog.products.get(catalog.recipes.get(r).product).branch)
    })
}

/// The strength 0–1 of a manager in a branch.
pub fn strength(catalog: &Catalog, manager: &Manager, branch: BranchId) -> f64 {
    catalog
        .management
        .branch
        .strength(manager.branch.get(&branch).copied().unwrap_or(0))
}

/// Sets the strength of every works from its production manager (month start).
pub(crate) fn refresh(state: &mut GameState, catalog: &Catalog) {
    let m = &catalog.management.branch;
    // Per works: the specialist for production and the head.
    let mut staff: BTreeMap<SiteId, (Option<&Manager>, Option<&Manager>)> = BTreeMap::new();
    for manager in state.managers.values() {
        let Some(job) = &manager.job else { continue };
        if job.successor {
            continue;
        }
        let Unit::Site(site) = job.position.unit else {
            continue;
        };
        let entry = staff.entry(site).or_default();
        match &job.position.role {
            Role::Specialist(f) if f == PRODUCTION => entry.0 = Some(manager),
            Role::Head => entry.1 = Some(manager),
            Role::Specialist(_) => {}
        }
    }
    let strengths: Vec<(usize, f64)> = state
        .sites
        .iter()
        .enumerate()
        .map(|(i, site)| {
            let own = site_branch(catalog, site).and_then(|b| {
                // Few works per manager; the cast is exact.
                let id = SiteId(i as u32);
                let (specialist, head) = staff.get(&id).copied().unwrap_or_default();
                match (specialist, head) {
                    (Some(x), _) if site.owner == job_company(x) => Some(strength(catalog, x, b)),
                    (_, Some(x)) if site.owner == job_company(x) => {
                        Some(strength(catalog, x, b) * m.head_share)
                    }
                    _ => None,
                }
            });
            (i, own.unwrap_or(0.0))
        })
        .collect();
    for (i, value) in strengths {
        state.sites[i].branch_strength = value;
    }
}

fn job_company(manager: &Manager) -> crate::state::CompanyId {
    manager
        .job
        .as_ref()
        .map_or(crate::state::CompanyId(u32::MAX), |j| j.company)
}

/// Factor on labour hours (and on the scrap rate with its own effect) of a works.
pub fn hours_factor(catalog: &Catalog, branch_strength: f64) -> f64 {
    1.0 - catalog.management.branch.hours * branch_strength
}

/// Factor on the scrap rate.
pub fn scrap_factor(catalog: &Catalog, branch_strength: f64) -> f64 {
    1.0 - catalog.management.branch.scrap * branch_strength
}

/// Factor on the material input.
pub fn material_factor(catalog: &Catalog, branch_strength: f64) -> f64 {
    1.0 - catalog.management.branch.material * branch_strength
}

/// The branches of the works in the unit of a company (the points a position there can
/// gain experience in).
fn unit_branches(
    catalog: &Catalog,
    state: &GameState,
    company: crate::state::CompanyId,
    unit: Unit,
    sites: &[SiteId],
) -> Vec<BranchId> {
    let mut out: Vec<BranchId> = match unit {
        Unit::Site(s) => state
            .sites
            .get(s.index())
            .and_then(|x| x.branch)
            .into_iter()
            .collect(),
        _ => sites
            .iter()
            .filter(|&&s| {
                state.sites[s.index()].owner == company
                    && crate::management::in_unit(catalog, state, s, unit)
            })
            .filter_map(|&s| site_branch(catalog, &state.sites[s.index()]))
            .collect(),
    };
    out.sort();
    out.dedup();
    out
}

/// The works of every company (month start), to find the branches of a unit.
pub(crate) fn sites_by_company(
    state: &GameState,
) -> BTreeMap<crate::state::CompanyId, Vec<SiteId>> {
    let mut out: BTreeMap<crate::state::CompanyId, Vec<SiteId>> = BTreeMap::new();
    for (i, s) in state.sites.iter().enumerate() {
        // Few sites; the cast is exact.
        out.entry(s.owner).or_default().push(SiteId(i as u32));
    }
    out
}

/// A point in a branch of the unit with the monthly chance, up to the potential (MG5).
/// Returns the managers and branches in the order of the managers' numbers; the chance is
/// drawn by `gain` per manager and branch so that the streams stay with the caller.
pub(crate) fn growth_targets(
    catalog: &Catalog,
    state: &GameState,
) -> Vec<(crate::state::ManagerId, Vec<BranchId>)> {
    let sites = sites_by_company(state);
    let none: Vec<SiteId> = Vec::new();
    let mut memo: BTreeMap<(crate::state::CompanyId, Unit), Vec<BranchId>> = BTreeMap::new();
    let mut out = Vec::new();
    for (&id, m) in &state.managers {
        let Some(job) = &m.job else { continue };
        let key = (job.company, job.position.unit);
        let branches = memo.entry(key).or_insert_with(|| {
            unit_branches(
                catalog,
                state,
                job.company,
                job.position.unit,
                sites.get(&job.company).unwrap_or(&none),
            )
        });
        if !branches.is_empty() {
            out.push((id, branches.clone()));
        }
    }
    out
}

/// The production factors of one facility group (MG5): each alone and all together.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FactorRow {
    pub site: SiteId,
    pub learning: f64,
    pub training: f64,
    pub size: f64,
    /// Development, trim and machine offer together.
    pub development: f64,
    pub branch: f64,
    /// Product of the factors on labour hours, before the floor, and on the material and
    /// the scrap rate.
    pub hours: f64,
    pub material: f64,
    pub scrap: f64,
}

/// The factors of every facility group with a recipe (the CLI protocol „produktionsfaktoren“).
pub fn factor_rows(catalog: &Catalog, state: &GameState) -> Vec<FactorRow> {
    let mut out = Vec::new();
    for (i, site) in state.sites.iter().enumerate() {
        for sl in &site.slots {
            let Some(r) = sl.recipe else { continue };
            let recipe = catalog.recipes.get(r);
            let e =
                crate::production::slot_effect_unfloored(catalog, state, site, sl, recipe.product);
            let branch = hours_factor(catalog, site.branch_strength);
            let learning = crate::learning::factor(catalog, site, recipe);
            let training = crate::training::labor_factor(catalog, site.training);
            let size = catalog.production_model.sizes.labor(sl.size);
            let has_scrap =
                crate::operations::scrap_rate_for(catalog, state, site, sl, recipe) > 0.0;
            out.push(FactorRow {
                // Few sites; the cast is exact.
                site: SiteId(i as u32),
                learning,
                training,
                size,
                development: e.labor / branch,
                branch,
                hours: e.labor * training * learning * size,
                material: e.inputs,
                scrap: if has_scrap {
                    crate::operations::scrap_factors(catalog, state, site, sl, recipe)
                } else {
                    1.0
                },
            });
        }
    }
    out
}
