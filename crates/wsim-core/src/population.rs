//! The AI companies at the start and later foundings (Lastenheft §10; docs/FORMELN.md,
//! M10).

use crate::catalog::Catalog;
use crate::ids::DepositId;

/// Yearly output of one extraction facility for a raw material, if a recipe extracts it.
pub fn extraction_capacity(catalog: &Catalog, resource: crate::ids::ProductId) -> Option<f64> {
    catalog
        .recipes
        .values()
        .filter(|r| r.extraction && r.product == resource)
        .map(|r| {
            catalog.facilities.get(r.facility).runs_per_day * r.output * 365.0
                / f64::from(r.duration_days.max(1))
        })
        .reduce(f64::max)
}

/// Number of concessions of a deposit: enough for `plants_per_concession` extraction
/// facilities each at the market scale, within 1 and the maximum.
pub fn concession_count(catalog: &Catalog, deposit: DepositId, scale: f64) -> u32 {
    let d = catalog.deposits.get(deposit);
    let model = &catalog.ai_model;
    let Some(capacity) = extraction_capacity(catalog, d.resource) else {
        return 1;
    };
    let plants = d.max_output_per_year * scale / (model.plants_per_concession * capacity);
    // Far below u32::MAX; the clamp keeps it within the data limit.
    (plants.floor().max(1.0) as u32).min(model.max_concessions.max(1))
}
