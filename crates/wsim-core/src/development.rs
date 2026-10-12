//! Development of researched products (Q1, before M37; formulas in docs/FORMELN.md).
//!
//! Every company collects research points for each product it may make. Its level is
//! its points against the leader's (the leader has 100 %); the effect in all its recipes
//! for the product follows that level: more quality, and – scaled by how far the product
//! is developed at all – fewer labor hours and fewer inputs per run. Points do not decay
//! and do not spread to others. Generations (point thresholds) still mark successor
//! models and the targets of start-ups.

use crate::calendar::Date;
use crate::catalog::{Catalog, Recipe};
use crate::ids::{Id, ProductId, SpecializationId};
use crate::state::{CompanyId, GameState, Site};

/// What a company's development level changes in its recipes for a product.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Effect {
    /// Quality points added before the limit to 0–100.
    pub quality: f64,
    /// Factor on the labor hours per run.
    pub labor: f64,
    /// Factor on the inputs per run.
    pub inputs: f64,
    /// Factor on the output of the main product per run (manufacturing research, Q6).
    pub output: f64,
}

impl Effect {
    pub const NONE: Effect = Effect {
        quality: 0.0,
        labor: 1.0,
        inputs: 1.0,
        output: 1.0,
    };

    /// The effect of a level (0–1 of the leader's points) at a maturity (0–1).
    pub fn of(catalog: &Catalog, level: f64, maturity: f64) -> Effect {
        let m = &catalog.research_model.development;
        Effect {
            quality: m.quality_max * level,
            labor: 1.0 - m.labor_max * level * maturity,
            inputs: 1.0 - m.inputs_max * level * maturity,
            output: 1.0,
        }
    }

    /// Adds the effect of manufacturing research (Q6) at a level and maturity.
    pub fn with_process(self, catalog: &Catalog, level: f64, maturity: f64) -> Effect {
        let m = &catalog.research_model.development;
        Effect {
            inputs: self.inputs * (1.0 - m.process_inputs_max * level * maturity),
            output: self.output * (1.0 + m.process_output_max * level * maturity),
            ..self
        }
    }
}

/// Generation of a company for a product: the point thresholds it passed by own research.
pub fn level(_catalog: &Catalog, state: &GameState, company: CompanyId, product: ProductId) -> u8 {
    state
        .company(company)
        .map_or(0, |c| c.development.level(product))
}

/// All points a company collected for a product.
pub fn points(state: &GameState, company: CompanyId, product: ProductId) -> f64 {
    state
        .company(company)
        .map_or(0.0, |c| c.development.total(product))
}

/// Most points any company collected for a product.
pub fn leader_points(state: &GameState, product: ProductId) -> f64 {
    state
        .development_leader
        .get(&product)
        .copied()
        .unwrap_or(0.0)
}

/// Level of a company for a product: its points against the leader's (0–1).
pub fn share(state: &GameState, company: CompanyId, product: ProductId) -> f64 {
    let lead = leader_points(state, product);
    if lead <= 0.0 {
        return 0.0;
    }
    (points(state, company, product) / lead).clamp(0.0, 1.0)
}

/// How far a product is developed at all (0–1): `1 − exp(−leader's points / B)`.
pub fn maturity(catalog: &Catalog, state: &GameState, product: ProductId) -> f64 {
    let Some((_, base)) = basis(catalog, product) else {
        return 0.0;
    };
    1.0 - libm::exp(-leader_points(state, product) / base.max(1e-9))
}

/// What the company's level of a product changes in its recipes for it.
pub fn effect(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
) -> Effect {
    if catalog.research_model.development.levels == 0 {
        return Effect::NONE;
    }
    let level = share(state, company, product);
    let developed = if level <= 0.0 {
        Effect::NONE
    } else {
        Effect::of(catalog, level, maturity(catalog, state, product))
    };
    let process = process_share(state, company, product);
    if process <= 0.0 {
        return developed;
    }
    developed.with_process(catalog, process, process_maturity(catalog, state, product))
}

/// Points of manufacturing research a company collected for a product (Q6).
pub fn process_points(state: &GameState, company: CompanyId, product: ProductId) -> f64 {
    state
        .company(company)
        .and_then(|c| c.process.get(&product).copied())
        .unwrap_or(0.0)
}

/// Most points of manufacturing research any company has for a product.
pub fn process_leader_points(state: &GameState, product: ProductId) -> f64 {
    state.process_leader.get(&product).copied().unwrap_or(0.0)
}

/// Manufacturing level of a company for a product against the leader (0–1).
pub fn process_share(state: &GameState, company: CompanyId, product: ProductId) -> f64 {
    let lead = process_leader_points(state, product);
    if lead <= 0.0 {
        return 0.0;
    }
    (process_points(state, company, product) / lead).clamp(0.0, 1.0)
}

/// How far the manufacturing of a product is researched at all (0–1).
pub fn process_maturity(catalog: &Catalog, state: &GameState, product: ProductId) -> f64 {
    let Some((_, base)) = basis(catalog, product) else {
        return 0.0;
    };
    1.0 - libm::exp(-process_leader_points(state, product) / base.max(1e-9))
}

/// Adds points of manufacturing research of a company for a product.
pub fn add_process_points(
    state: &mut GameState,
    company: CompanyId,
    product: ProductId,
    amount: f64,
) {
    let total = state.companies[company.index()]
        .process
        .entry(product)
        .or_default();
    *total += amount;
    let total = *total;
    let lead = state.process_leader.entry(product).or_default();
    *lead = lead.max(total);
}

/// Points of the generations 1 to `n` of a product, without a latecomer's discount.
pub fn threshold(catalog: &Catalog, product: ProductId, n: u8) -> f64 {
    let m = &catalog.research_model.development;
    let Some((_, base)) = basis(catalog, product) else {
        return 0.0;
    };
    (1..=n)
        .map(|k| base * m.effort_share * crate::math::pow(m.effort_growth, f64::from(k - 1)))
        .sum()
}

/// Adds research points of a company for a product: to all its points, the leader's,
/// and the progress towards its next generation.
pub fn add_points(state: &mut GameState, company: CompanyId, product: ProductId, amount: f64) {
    let d = &mut state.companies[company.index()].development;
    let total = d.total.entry(product).or_default();
    *total += amount;
    let total = *total;
    *d.points.entry(product).or_default() += amount;
    let lead = state.development_leader.entry(product).or_default();
    *lead = lead.max(total);
}

/// A company gets a generation of a product (a start-up's result): its points rise to the
/// generation's threshold.
pub fn grant_level(
    catalog: &Catalog,
    state: &mut GameState,
    company: CompanyId,
    product: ProductId,
    level: u8,
) {
    let need = threshold(catalog, product, level);
    let d = &mut state.companies[company.index()].development;
    let own = d.levels.entry(product).or_default();
    *own = (*own).max(level);
    d.points.remove(&product);
    let total = d.total.entry(product).or_default();
    *total = total.max(need);
    let total = *total;
    let lead = state.development_leader.entry(product).or_default();
    *lead = lead.max(total);
}

/// A company gives up its progress towards the next generation of a product (it went
/// into a spin-off): the points leave its total too.
pub fn remove_progress(state: &mut GameState, company: CompanyId, product: ProductId) {
    let d = &mut state.companies[company.index()].development;
    let progress = d.points.remove(&product).unwrap_or(0.0);
    if let Some(total) = d.total.get_mut(&product) {
        *total = (*total - progress).max(0.0);
    }
    refresh_leader(state, product);
}

/// Recomputes the leader's points of a product.
fn refresh_leader(state: &mut GameState, product: ProductId) {
    let lead = state
        .companies
        .iter()
        .map(|c| c.development.total(product))
        .fold(0.0, f64::max);
    if lead > 0.0 {
        state.development_leader.insert(product, lead);
    } else {
        state.development_leader.remove(&product);
    }
}

/// After loading: saves before Q1 get all points from their generations and progress;
/// the leaders are derived.
pub(crate) fn fit_loaded(state: &mut GameState, catalog: &Catalog) {
    for c in &mut state.companies {
        let d = &mut c.development;
        let products: Vec<ProductId> = d.levels.keys().chain(d.points.keys()).copied().collect();
        for p in products {
            if d.total.contains_key(&p) {
                continue;
            }
            let total =
                threshold(catalog, p, d.level(p)) + d.points.get(&p).copied().unwrap_or(0.0);
            if total > 0.0 {
                d.total.insert(p, total);
            }
        }
    }
    state.development_leader.clear();
    state.process_leader.clear();
    for c in &state.companies {
        for (&p, &t) in &c.development.total {
            let lead = state.development_leader.entry(p).or_default();
            *lead = lead.max(t);
        }
        for (&p, &t) in &c.process {
            let lead = state.process_leader.entry(p).or_default();
            *lead = lead.max(t);
        }
    }
}

/// The products a research center develops with their shares of its researchers: the
/// shares set (Q1), else its single product with all of them.
pub fn site_shares(site: &Site) -> Vec<(ProductId, f64)> {
    if site.development_shares.is_empty() && site.process_shares.is_empty() {
        // A single product (the AI's way): half development, half manufacturing (Q6).
        site.development.map(|p| vec![(p, 0.5)]).unwrap_or_default()
    } else {
        site.development_shares
            .iter()
            .map(|(&p, &s)| (p, s))
            .collect()
    }
}

/// The products a research center does manufacturing research on, with their shares.
pub fn site_process_shares(site: &Site) -> Vec<(ProductId, f64)> {
    if site.development_shares.is_empty() && site.process_shares.is_empty() {
        site.development.map(|p| vec![(p, 0.5)]).unwrap_or_default()
    } else {
        site.process_shares.iter().map(|(&p, &s)| (p, s)).collect()
    }
}

/// Share of a research center's researchers working on its technology.
pub fn technology_share(site: &Site) -> f64 {
    if site.research.is_none() {
        return 0.0;
    }
    let products: f64 = site_shares(site)
        .iter()
        .chain(site_process_shares(site).iter())
        .map(|(_, s)| s)
        .sum();
    (1.0 - products).max(0.0)
}

/// Whether a research center learns a product under mandatory research (FW1): it
/// researches a product whose technologies the company knows.
pub fn learning(state: &crate::state::GameState, site: &Site) -> bool {
    state.settings.research_mandatory && site.research.is_none() && site.research_product.is_some()
}

/// Share of a research center's researchers not given to developed products.
pub fn free_share(site: &Site) -> f64 {
    let products: f64 = site_shares(site)
        .iter()
        .chain(site_process_shares(site).iter())
        .map(|(_, s)| s)
        .sum();
    (1.0 - products).max(0.0)
}

/// Whether a research center has nothing to work on.
pub fn idle(site: &Site) -> bool {
    site.research.is_none() && site_shares(site).is_empty() && site_process_shares(site).is_empty()
}

/// Effort of a product from its own data: the base effort times a factor from the depth
/// of its input chain and the number of distinct parts in it (without `complexity`
/// just the base).
fn own_base(catalog: &Catalog, product: ProductId) -> f64 {
    let m = &catalog.research_model.development;
    let Some(c) = m.complexity else {
        return m.base_effort;
    };
    let mut parts = Vec::new();
    let depth = collect_parts(catalog, product, 0, &mut parts);
    let factor = c.min + c.per_level * f64::from(depth) + c.per_part * parts.len() as f64;
    m.base_effort * factor.min(c.max)
}

/// Collects the distinct products below a product through its first recipe and returns
/// the levels of the chain (0 for raw materials).
fn collect_parts(
    catalog: &Catalog,
    product: ProductId,
    guard: u32,
    parts: &mut Vec<ProductId>,
) -> u32 {
    if guard >= 12 {
        return 0;
    }
    let Some(r) = catalog.recipes.values().find(|r| r.product == product) else {
        return 0;
    };
    let mut deepest = 0;
    for &(p, _) in &r.inputs {
        if !parts.contains(&p) {
            parts.push(p);
        }
        deepest = deepest.max(1 + collect_parts(catalog, p, guard + 1, parts));
    }
    deepest
}

/// Research field and base effort of a product, `None` when it cannot be developed (no
/// recipe, no field, or development turned off): the field of the technology of its
/// first recipe with one, else that of its branch; the largest research effort of its
/// recipes' technologies, at least the base effort of the data.
pub fn basis(catalog: &Catalog, product: ProductId) -> Option<(SpecializationId, f64)> {
    let m = &catalog.research_model.development;
    if m.levels == 0 {
        return None;
    }
    let recipes: Vec<&Recipe> = catalog
        .recipes
        .values()
        .filter(|r| r.product == product)
        .collect();
    if recipes.is_empty() {
        return None;
    }
    let field = recipes
        .iter()
        .find_map(|r| r.technology)
        .map(|t| catalog.technologies.get(t).field)
        .or_else(|| {
            let branch = catalog.products.get(product).branch;
            m.fields.get(branch.index()).copied().flatten()
        })?;
    let base = recipes
        .iter()
        .filter_map(|r| r.technology)
        .filter_map(|t| catalog.technologies.get(t).research_effort)
        .fold(own_base(catalog, product), f64::max);
    Some((field, base))
}

/// Whether a company may develop a product: it may use a recipe for it. There is no end
/// level (Q1).
pub fn can_develop(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
) -> bool {
    basis(catalog, product).is_some()
        && state.knows_product(catalog, company, product)
        && catalog.recipes.values().any(|r| {
            r.product == product
                && r.technology
                    .is_none_or(|t| state.knows(catalog, company, t))
        })
}

/// The next generation of a product for a company and the research points it takes on
/// `date`; `None` after the last generation or for products that cannot be developed.
pub fn next_effort(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
    date: Date,
) -> Option<(u8, f64)> {
    let m = &catalog.research_model.development;
    let next = level(catalog, state, company, product).checked_add(1)?;
    if next > m.levels {
        return None;
    }
    let (_, base) = basis(catalog, product)?;
    let points = base * m.effort_share * crate::math::pow(m.effort_growth, f64::from(next - 1));
    // Latecomers pay less, as for technologies (M9).
    let r = &catalog.research_model;
    let factor = state
        .developments
        .get(product)
        .get(usize::from(next - 1))
        .map_or(1.0, |first| {
            let years = (date.year_fraction() - first.year_fraction()).max(0.0);
            crate::math::pow(1.0 - r.latecomer_discount, years).max(r.latecomer_min)
        });
    Some((next, points * factor))
}

/// Points a company's next step on a product takes: the next generation's effort, after
/// the last generation as much as the last one took (research goes on, Q1).
pub fn step_effort(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
    date: Date,
) -> Option<f64> {
    if let Some((_, needed)) = next_effort(catalog, state, company, product, date) {
        return Some(needed);
    }
    let last = catalog.research_model.development.levels;
    (last > 0 && basis(catalog, product).is_some())
        .then(|| threshold(catalog, product, last) - threshold(catalog, product, last - 1))
}

/// Research field of the project of a research center: its technology's or that of the
/// product it develops.
pub fn project_field(
    catalog: &Catalog,
    state: &GameState,
    site: crate::state::SiteId,
) -> Option<SpecializationId> {
    let s = state.site(site)?;
    match (s.research, site_shares(s).first()) {
        (Some(t), _) => Some(catalog.technologies.get(t).field),
        (None, Some(&(p, _))) => basis(catalog, p).map(|(field, _)| field),
        (None, None) if learning(state, s) => s
            .research_product
            .and_then(|p| basis(catalog, p))
            .map(|(field, _)| field),
        (None, None) => None,
    }
}

/// What one research point costs in a country (USD): the researcher's wage for a day and
/// the material, divided by the country's research efficiency in the field (M9).
pub fn cost_per_point(
    catalog: &Catalog,
    state: &GameState,
    country: crate::ids::CountryId,
    field: SpecializationId,
) -> Option<f64> {
    let r = &catalog.research_model;
    let group = r.researchers.get(field.index()).copied().flatten()?;
    let c = state.countries.get(country);
    let efficiency = c
        .research_efficiency
        .get(field.index())
        .copied()
        .unwrap_or(1.0);
    if efficiency <= 0.0 {
        return None;
    }
    let hours = crate::production::hours_per_worker_day(catalog, state.date);
    let wage = c.hourly_wage_usd.get(group.index()).copied().unwrap_or(0.0);
    Some((wage * hours + r.material_usd_per_day * c.price_level) / efficiency)
}
