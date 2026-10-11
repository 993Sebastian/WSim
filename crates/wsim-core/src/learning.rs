//! Learning curve of the sites (docs/FORMELN.md „Lernkurve“, BP1): the hours a run needs
//! fall with the cumulative quantity a site has made of a product.

use std::collections::BTreeMap;

use crate::catalog::{Catalog, Recipe};
use crate::ids::ProductId;
use crate::state::{Experience, GameState, Site, Slot};

/// Reference quantity `n_0`: a year's output of a medium facility of the recipe.
pub fn reference_units(catalog: &Catalog, recipe: &Recipe) -> f64 {
    recipe.output * catalog.facilities.get(recipe.facility).runs_per_day * 365.0
}

/// Learning factor for `units` made: 1 up to `n_0`, then `(n / n_0)^(−b)` with
/// `b = −log₂(rate)`, at least the minimum.
pub fn factor_of(catalog: &Catalog, recipe: &Recipe, units: f64) -> f64 {
    let m = &catalog.production_model.learning;
    if !m.enabled {
        return 1.0;
    }
    let n0 = reference_units(catalog, recipe);
    if n0 <= 0.0 || units <= n0 {
        return 1.0;
    }
    let rate = m.rate(catalog.products.get(recipe.product).branch);
    let b = -libm::log2(rate);
    libm::pow(units / n0, -b).max(m.minimum)
}

/// Learning factor of a site for a recipe.
pub fn factor(catalog: &Catalog, site: &Site, recipe: &Recipe) -> f64 {
    factor_of(catalog, recipe, units(site, recipe.product))
}

/// Experience of a site with a product (cumulative units).
pub fn units(site: &Site, product: ProductId) -> f64 {
    site.experience.get(&product).map_or(0.0, |e| e.units)
}

/// Counts units made.
pub(crate) fn record(site: &mut Site, product: ProductId, quantity: f64) {
    if quantity > 0.0 {
        let e = site.experience.entry(product).or_default();
        e.units += quantity;
        e.month += quantity;
    }
}

/// At the end of a month: products not made this month lose a share of the experience.
pub(crate) fn month_end(state: &mut GameState, catalog: &Catalog) {
    let m = &catalog.production_model.learning;
    if !m.enabled {
        return;
    }
    for site in &mut state.sites {
        for e in site.experience.values_mut() {
            if e.month <= 0.0 {
                e.units *= 1.0 - m.forget_per_month;
            }
            e.month = 0.0;
            e.cut = false;
        }
        site.experience.retain(|_, e| e.units >= 1e-6);
    }
}

/// Layoffs: when more than a third of the workforce goes, every product loses a share of
/// its experience, once a month.
pub(crate) fn laid_off(catalog: &Catalog, site: &mut Site, before: f64, after: f64) {
    let m = &catalog.production_model.learning;
    if !m.enabled || before <= 0.0 || after >= before * (2.0 / 3.0) {
        return;
    }
    for e in site.experience.values_mut() {
        if !e.cut {
            e.units *= 1.0 - m.layoff_loss;
            e.cut = true;
        }
    }
}

/// A site switches to `product`: when it succeeds a product the site made (`ersetzt`),
/// it starts with a share of that experience.
pub(crate) fn switched(catalog: &Catalog, site: &mut Site, product: ProductId) {
    let m = &catalog.production_model.learning;
    if !m.enabled {
        return;
    }
    let inherited = catalog
        .products
        .get(product)
        .replaces
        .iter()
        .map(|&p| units(site, p))
        .fold(0.0, f64::max)
        * m.transfer;
    if inherited > units(site, product) {
        site.experience.entry(product).or_default().units = inherited;
    }
}

/// Experience of the start companies: years of output of a medium facility for every
/// product their facilities make.
pub(crate) fn established(catalog: &Catalog, slots: &[Slot]) -> BTreeMap<ProductId, Experience> {
    let m = &catalog.production_model.learning;
    let mut out = BTreeMap::new();
    if !m.enabled || m.established_years <= 0.0 {
        return out;
    }
    for sl in slots {
        if let Some(r) = sl.recipe {
            let recipe = catalog.recipes.get(r);
            out.insert(
                recipe.product,
                Experience {
                    units: m.established_years * reference_units(catalog, recipe),
                    ..Experience::default()
                },
            );
        }
    }
    out
}
