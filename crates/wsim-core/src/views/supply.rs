//! Internal supply and transfers between own sites (docs/FORMELN.md, W5).

use serde::{Deserialize, Serialize};

use super::usd;
use crate::command::site_type_key;
use crate::game::Game;
use crate::ids::ProductId;
use crate::state::SiteId;
use crate::supply::{self, SupplyMode};

/// A site of the company or group as seen from another: where it is and what the way
/// between them costs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SupplySiteView {
    pub site: u32,
    pub name: Option<String>,
    pub kind_text: String,
    pub country: String,
    pub company: String,
    /// The site belongs to the player's own company (not another of its group).
    pub own_company: bool,
    pub stock: f64,
    /// Made there a day at the planned utilization.
    pub output_per_day: f64,
    /// What it can give now (stock above its reserve and a day of its own use).
    pub spare: f64,
    /// Production cost of its stock per unit (the price for another company of the group).
    pub unit_value_usd: Option<f64>,
    /// Freight per unit and days of the way (none within a country) and the customs rate.
    pub freight_per_unit_usd: f64,
    pub travel_days: u32,
    pub customs_rate: f64,
}

/// What a site draws from another site.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SupplyView {
    pub product: String,
    pub source: SupplySiteView,
    /// `fest` or `bedarf`.
    pub mode: String,
    pub per_day: Option<f64>,
    pub days: Option<f64>,
    /// The site's need a day (recipes and sales) and what it asks today.
    pub demand_per_day: f64,
    pub wanted_today: f64,
    pub in_transit: f64,
    pub delivered_month: f64,
    pub delivered_last_month: f64,
}

fn site_view(game: &Game, from: SiteId, to: SiteId, p: ProductId) -> Option<SupplySiteView> {
    let (state, catalog) = (game.state(), game.catalog());
    let s = state.site(from)?;
    let (way, rate) = supply::route(state, catalog, (from, to), p)?;
    let stock = s.inventory.get(&p);
    Some(SupplySiteView {
        site: from.0,
        name: s.name.clone(),
        kind_text: site_type_key(s.kind),
        country: catalog.countries.key(s.country).to_owned(),
        company: state.companies[s.owner.index()].name.clone(),
        own_company: s.held_by(game.player()),
        stock: stock.map_or(0.0, |x| x.quantity),
        output_per_day: crate::contracts::daily_output(state, catalog, from, p),
        spare: supply::spare(state, catalog, from, p),
        unit_value_usd: stock
            .filter(|x| x.quantity > 1e-9)
            .map(|x| usd(x.value) / x.quantity),
        freight_per_unit_usd: way.map_or(0.0, |(m, _)| usd(m)),
        travel_days: way.map_or(0, |(_, d)| d),
        customs_rate: rate,
    })
}

/// The internal supplies of a site.
pub fn site_supplies(game: &Game, site: SiteId) -> Vec<SupplyView> {
    let (state, catalog) = (game.state(), game.catalog());
    let Some(s) = state.site(site) else {
        return Vec::new();
    };
    let today = state.date;
    s.supplies
        .iter()
        .filter_map(|(&p, x)| {
            let (mode, per_day, days) = match x.mode {
                SupplyMode::Fixed { per_day } => ("fest", Some(per_day), None),
                SupplyMode::Need { days } => ("bedarf", None, Some(days)),
            };
            Some(SupplyView {
                product: catalog.products.key(p).to_owned(),
                source: site_view(game, x.source, site, p)?,
                mode: mode.to_owned(),
                per_day,
                days,
                demand_per_day: supply::daily_demand(state, catalog, site, p, today),
                wanted_today: supply::wanted(state, catalog, site, p, x.mode, today),
                in_transit: supply::in_transit(state, site, p),
                delivered_month: x.delivered_month,
                delivered_last_month: x.delivered_last_month,
            })
        })
        .collect()
}

/// Sites of the company and its group a site can draw a product from: those that make
/// it or hold it, with the way to the site.
pub fn internal_sources(game: &Game, site: u32, product: &str) -> Option<Vec<SupplySiteView>> {
    let (state, catalog) = (game.state(), game.catalog());
    let to = SiteId(site);
    state.site(to).filter(|s| s.held_by(game.player()))?;
    let p = catalog.products.id(product)?;
    let list = (0..state.sites.len())
        .map(|i| SiteId(u32::try_from(i).expect("site count fits u32")))
        .filter(|&from| supply::allowed(state, to, from))
        .filter(|&from| {
            let s = &state.sites[from.index()];
            s.inventory.get(&p).is_some_and(|x| x.quantity > 1e-9)
                || crate::contracts::daily_output(state, catalog, from, p) > 1e-9
        })
        .filter_map(|from| site_view(game, from, to, p))
        .collect();
    Some(list)
}

/// Other sites of the player's company a site can send a product to, with the way.
pub fn transfer_targets(game: &Game, site: u32, product: &str) -> Option<Vec<SupplySiteView>> {
    let (state, catalog) = (game.state(), game.catalog());
    let from = SiteId(site);
    state.site(from).filter(|s| s.held_by(game.player()))?;
    let p = catalog.products.id(product)?;
    let list = (0..state.sites.len())
        .map(|i| SiteId(u32::try_from(i).expect("site count fits u32")))
        .filter(|&to| to != from && state.sites[to.index()].held_by(game.player()))
        .filter_map(|to| {
            // The way seen from the target, with the target's place in the fields.
            let way = site_view(game, from, to, p)?;
            let t = &state.sites[to.index()];
            Some(SupplySiteView {
                site: to.0,
                name: t.name.clone(),
                kind_text: site_type_key(t.kind),
                country: catalog.countries.key(t.country).to_owned(),
                company: state.companies[t.owner.index()].name.clone(),
                own_company: true,
                stock: t.inventory.get(&p).map_or(0.0, |x| x.quantity),
                output_per_day: crate::contracts::daily_output(state, catalog, to, p),
                ..way
            })
        })
        .collect();
    Some(list)
}
