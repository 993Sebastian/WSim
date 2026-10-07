//! Training of the sites (W1; Lastenheft §5.4; rules in docs/FORMELN.md).
//!
//! Every site has a training level that moves towards its target month by month. The
//! target costs a share of the wage bill every day; the level reached saves labor
//! hours and raises the quality of the products.

use crate::calendar::Date;
use crate::catalog::{Catalog, SiteType};
use crate::command::{self, Command};
use crate::state::{CompanyId, GameState, SiteId};
use crate::strategy;

/// Where a site's training target comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetSource {
    /// The site's own target.
    Site,
    /// The company's strategy for the site.
    Strategy,
    /// No target: no training.
    None,
}

/// The training target of a site and where it comes from.
pub fn target(catalog: &Catalog, state: &GameState, site: SiteId) -> (f64, TargetSource) {
    let Some(s) = state.sites.get(site.index()) else {
        return (0.0, TargetSource::None);
    };
    if let Some(t) = s.training_target {
        return (t, TargetSource::Site);
    }
    match strategy::training_for_site(state, s.owner, site, catalog) {
        Some(t) => (t, TargetSource::Strategy),
        None => (0.0, TargetSource::None),
    }
}

/// Factor on the labor hours of a site with a training level.
pub fn labor_factor(catalog: &Catalog, level: f64) -> f64 {
    1.0 - catalog.production_model.training.labor_saving * level.clamp(0.0, 1.0)
}

/// Quality points a training level adds.
pub fn quality(catalog: &Catalog, level: f64) -> f64 {
    catalog.production_model.training.quality_points * level.clamp(0.0, 1.0)
}

/// Daily cost of a site's training for its wage bill of the day (USD).
pub fn daily_cost(catalog: &Catalog, state: &GameState, site: SiteId, wage_bill: f64) -> f64 {
    let (t, _) = target(catalog, state, site);
    wage_bill * catalog.production_model.training.cost_share * t
}

/// At a month start: every site's level moves towards its target.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog) {
    let m = &catalog.production_model.training;
    for index in 0..state.sites.len() {
        let site = SiteId(u32::try_from(index).unwrap_or(u32::MAX));
        let (t, _) = target(catalog, state, site);
        let s = &mut state.sites[index];
        s.training = if s.training < t {
            (s.training + m.gain_per_month).min(t)
        } else {
            (s.training - m.loss_per_month).max(t)
        };
    }
}

/// At a month start: AI companies set their sites' targets by competence, in steps of
/// 0.05, with the same command as the player.
pub(crate) fn ai_month_start(state: &mut GameState, catalog: &Catalog, _date: Date) {
    for index in 0..state.companies.len() {
        let id = CompanyId(u32::try_from(index).unwrap_or(u32::MAX));
        let c = &state.companies[index];
        let Some(ai) = &c.ai else { continue };
        if c.bankrupt {
            continue;
        }
        let wanted = (catalog.ai_model.behavior.training.at(ai.skill()) * 20.0).round() / 20.0;
        let sites: Vec<SiteId> = state
            .sites
            .iter()
            .enumerate()
            .filter(|(_, s)| s.owner == id && s.kind != SiteType::ResearchCenter)
            .filter(|(_, s)| s.training_target.is_none_or(|t| (t - wanted).abs() > 1e-9))
            .map(|(i, _)| SiteId(u32::try_from(i).unwrap_or(u32::MAX)))
            .collect();
        for site in sites {
            let set = Command::SetTraining {
                site,
                target: Some(wanted),
            };
            // A site of its own; nothing can fail but a bug.
            let _ = command::execute(state, catalog, id, &set);
        }
    }
}
