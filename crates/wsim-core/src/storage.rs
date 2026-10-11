//! Storage of the sites: capacity, costs, spoilage and expansion (docs/FORMELN.md
//! „Lager: Kapazität, Kosten, Verderb“). Stocks are measured by their value at reference
//! prices, so goods of different units compare.

use crate::catalog::{Catalog, SiteType};
use crate::command::CommandError;
use crate::ledger::{Account, CostCenter, CostType};
use crate::money::Money;
use crate::state::{CompanyId, SiteId};
use crate::state::{GameState, Site};

/// Value of a site's stocks at reference prices.
pub fn fill(catalog: &Catalog, site: &Site) -> Money {
    site.inventory
        .iter()
        .map(|(&p, s)| {
            catalog
                .products
                .get(p)
                .reference_price
                .scale(s.quantity.max(0.0))
        })
        .fold(Money::ZERO, |a, b| a + b)
}

/// Throughput of the site's built facilities per day at full load, at reference prices:
/// what goes in and what comes out.
fn throughput_usd(catalog: &Catalog, site: &Site) -> f64 {
    let price = |p| catalog.products.get(p).reference_price.to_usd();
    site.slots
        .iter()
        .filter_map(|sl| {
            let recipe = catalog.recipes.get(sl.recipe?);
            let per_run = price(recipe.product) * recipe.output
                + recipe
                    .by_products
                    .iter()
                    .chain(recipe.inputs.iter())
                    .map(|&(p, q)| price(p) * q)
                    .sum::<f64>();
            Some(per_run * sl.full_runs(catalog))
        })
        .sum()
}

/// What the site can store; `None` when storage is unlimited (no data).
pub fn capacity(catalog: &Catalog, site: &Site) -> Option<Money> {
    let m = &catalog.production_model.storage;
    if !m.enabled {
        return None;
    }
    let by_throughput =
        Money::from_usd(m.days * throughput_usd(catalog, site)).unwrap_or(Money::ZERO);
    let base = m.minimum(site.kind).max(by_throughput);
    Some(base + m.step_capacity.scale(f64::from(site.storage_steps)))
}

/// Whether the site's store is full: its orders do not buy. Power plants feed the grid
/// and never count as full.
pub fn is_full(catalog: &Catalog, site: &Site) -> bool {
    site.kind != SiteType::PowerPlant
        && capacity(catalog, site).is_some_and(|c| fill(catalog, site) >= c)
}

/// Whether the facilities stop: the store is full and the site's own products take at
/// least half of it. A store full of inputs keeps running, so that it empties.
pub fn stops_output(catalog: &Catalog, site: &Site) -> bool {
    if !is_full(catalog, site) {
        return false;
    }
    let Some(cap) = capacity(catalog, site) else {
        return false;
    };
    let mut made: Vec<crate::ids::ProductId> = Vec::new();
    for sl in &site.slots {
        if let Some(r) = sl.recipe {
            let recipe = catalog.recipes.get(r);
            made.push(recipe.product);
            made.extend(recipe.by_products.iter().map(|&(p, _)| p));
        }
    }
    let output = site
        .inventory
        .iter()
        .filter(|(p, _)| made.contains(p))
        .map(|(&p, s)| {
            catalog
                .products
                .get(p)
                .reference_price
                .scale(s.quantity.max(0.0))
        })
        .fold(Money::ZERO, |a, b| a + b);
    output.scale(2.0) >= cap
}

/// Storage cost of a day: a yearly share of the stored value.
pub(crate) fn daily_cost(catalog: &Catalog, site: &Site) -> Money {
    let m = &catalog.production_model.storage;
    if !m.enabled || m.cost_share_year <= 0.0 {
        return Money::ZERO;
    }
    fill(catalog, site).scale(m.cost_share_year / 365.0)
}

/// Spoilage at the end of a month: a share of the stocks of perishable goods groups is
/// lost (quantity and value).
pub(crate) fn month_end(state: &mut GameState, catalog: &Catalog) {
    let m = &catalog.production_model.storage;
    if !m.enabled || m.spoilage.is_empty() {
        return;
    }
    for index in 0..state.sites.len() {
        if state.sites[index].closed.is_some() {
            continue;
        }
        let site = SiteId(u32::try_from(index).expect("site count fits u32"));
        let owner = state.sites[index].owner;
        let mut losses = Vec::new();
        for (&p, stock) in &mut state.sites[index].inventory {
            let share = m.spoilage(catalog.products.get(p).goods_group);
            if share > 0.0 && stock.quantity > 0.0 {
                let lost = stock.take(stock.quantity * share);
                if lost > Money::ZERO {
                    losses.push((p, lost));
                }
            }
        }
        let ledger = &mut state.companies[owner.index()].ledger;
        for (p, lost) in losses {
            ledger.expense(
                CostType::Storage,
                CostCenter::product(site, p),
                Account::Inventory,
                lost,
            );
        }
    }
}

/// Builds `steps` steps more storage at a site (`BuildStorage`), paid from the cash as
/// an expense of the storage (shelving and fittings, no lasting asset).
pub(crate) fn build(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    site: SiteId,
    steps: u32,
) -> Result<(), CommandError> {
    let m = &catalog.production_model.storage;
    if !m.enabled || steps == 0 {
        return Err(CommandError::NotPossible);
    }
    let cost = m.step_cost.scale(f64::from(steps));
    let ledger = &mut state.companies[actor.index()].ledger;
    if ledger.cash() < cost {
        return Err(CommandError::NotEnoughCash { needed: cost });
    }
    ledger.expense(
        CostType::Storage,
        CostCenter::site(site),
        Account::Cash,
        cost,
    );
    state.sites[site.index()].storage_steps += steps;
    Ok(())
}

/// Steps an AI company builds at a site whose store is filled beyond its threshold: one
/// when the site earned money in the last closed month (a store full of goods nobody
/// buys is not worth more room) and the cash bears it twice over.
pub(crate) fn ai_steps(catalog: &Catalog, state: &GameState, site: SiteId) -> Option<u32> {
    let m = &catalog.production_model.storage;
    let s = state.site(site)?;
    let cap = capacity(catalog, s)?;
    if s.kind == SiteType::PowerPlant || fill(catalog, s) < cap.scale(m.ai_expand_at) {
        return None;
    }
    let ledger = &state.companies[s.owner.index()].ledger;
    let earned = ledger
        .months
        .last()
        .and_then(|p| p.by_site.get(&site))
        .is_some_and(|&r| r > Money::ZERO);
    (earned && ledger.cash() >= m.step_cost.scale(2.0)).then_some(1)
}
