//! Development of researched products (M37; formulas in docs/FORMELN.md).
//!
//! Every company can improve each product it may make, level by level, in its research
//! centers: more quality, fewer labor hours and fewer inputs per run in all its recipes
//! for the product. A level the first company reached long enough ago is common
//! knowledge.

use crate::calendar::Date;
use crate::catalog::{Catalog, Recipe};
use crate::ids::{Id, ProductId, SpecializationId};
use crate::state::{CompanyId, GameState};

/// What a company's development level changes in its recipes for a product.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Effect {
    /// Quality points added before the limit to 0–100.
    pub quality: f64,
    /// Factor on the labor hours per run.
    pub labor: f64,
    /// Factor on the inputs per run.
    pub inputs: f64,
}

impl Effect {
    pub const NONE: Effect = Effect {
        quality: 0.0,
        labor: 1.0,
        inputs: 1.0,
    };

    /// The effect of a level.
    pub fn of_level(catalog: &Catalog, level: u8) -> Effect {
        let m = &catalog.research_model.development;
        let l = f64::from(level);
        Effect {
            quality: l * m.quality_per_level,
            labor: 1.0 - l * m.labor_per_level,
            inputs: 1.0 - l * m.inputs_per_level,
        }
    }
}

/// Levels of a product that became common knowledge by `date`: those the first company
/// reached at least `public_domain_years` ago.
pub fn public_level(catalog: &Catalog, state: &GameState, product: ProductId, date: Date) -> u8 {
    let years = catalog.research_model.development.public_domain_years;
    let now = date.year_fraction();
    let firsts = state.developments.get(product);
    let count = firsts
        .iter()
        .take_while(|d| d.year_fraction() + years <= now)
        .count();
    u8::try_from(count).unwrap_or(u8::MAX)
}

/// Development level of a company for a product: its own or the common one.
pub fn level(catalog: &Catalog, state: &GameState, company: CompanyId, product: ProductId) -> u8 {
    let own = state
        .company(company)
        .map_or(0, |c| c.development.level(product));
    own.max(public_level(catalog, state, product, state.date))
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
    match level(catalog, state, company, product) {
        0 => Effect::NONE,
        l => Effect::of_level(catalog, l),
    }
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
        .fold(m.base_effort, f64::max);
    Some((field, base))
}

/// Whether a company may develop a product: it may use a recipe for it and has not
/// reached the top level.
pub fn can_develop(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    product: ProductId,
) -> bool {
    basis(catalog, product).is_some()
        && level(catalog, state, company, product) < catalog.research_model.development.levels
        && catalog.recipes.values().any(|r| {
            r.product == product
                && r.technology
                    .is_none_or(|t| state.knows(catalog, company, t))
        })
}

/// The next level of a product for a company and the research points it takes on
/// `date`; `None` at the top level or for products that cannot be developed.
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

/// Research field of the project of a research center: its technology's or that of the
/// product it develops.
pub fn project_field(
    catalog: &Catalog,
    state: &GameState,
    site: crate::state::SiteId,
) -> Option<SpecializationId> {
    let s = state.site(site)?;
    match (s.research, s.development) {
        (Some(t), _) => Some(catalog.technologies.get(t).field),
        (None, Some(p)) => basis(catalog, p).map(|(field, _)| field),
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
