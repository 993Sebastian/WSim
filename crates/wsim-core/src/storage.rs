//! Storage of the sites: capacity, costs, spoilage and expansion (docs/FORMELN.md
//! „Lager: Kapazität, Kosten, Verderb“). Stocks are measured by their value at reference
//! prices, so goods of different units compare.

use crate::calendar::Date;
use crate::catalog::{Catalog, SiteType};
use crate::command::CommandError;
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, SiteId};
use crate::state::{GameState, Site, Stock};

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

/// Capacity including outside storage (BP3): beyond it the site stops buying.
pub fn hard_capacity(catalog: &Catalog, site: &Site) -> Option<Money> {
    let m = &catalog.production_model.storage;
    capacity(catalog, site).map(|c| c.scale(m.external_capacity.max(1.0)))
}

/// Whether the site's store is full, outside storage included: its orders do not buy.
/// Power plants feed the grid and never count as full.
pub fn is_full(catalog: &Catalog, site: &Site) -> bool {
    site.kind != SiteType::PowerPlant
        && hard_capacity(catalog, site).is_some_and(|c| fill(catalog, site) >= c)
}

/// Whether goods lie in outside storage (BP3): the store is over its own capacity.
pub fn external(catalog: &Catalog, site: &Site) -> bool {
    catalog.production_model.storage.external_capacity > 1.0
        && site.kind != SiteType::PowerPlant
        && capacity(catalog, site).is_some_and(|c| fill(catalog, site) > c)
}

/// Whether the facilities stop: the store is full and the site's own products take at
/// least half of it. A store full of inputs keeps running, so that it empties.
pub fn stops_output(catalog: &Catalog, site: &Site) -> bool {
    if !is_full(catalog, site) {
        return false;
    }
    let Some(cap) = hard_capacity(catalog, site) else {
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

/// Storage cost of a day: a yearly share of the stored value by storage class; what lies
/// in outside storage costs `external_rate` times as much (BP3).
pub(crate) fn daily_cost(catalog: &Catalog, site: &Site) -> Money {
    let m = &catalog.production_model.storage;
    if !m.enabled {
        return Money::ZERO;
    }
    let mut value = Money::ZERO;
    let mut cost = Money::ZERO;
    for (&p, s) in &site.inventory {
        let v = catalog
            .products
            .get(p)
            .reference_price
            .scale(s.quantity.max(0.0));
        value += v;
        cost += v.scale(m.cost_share(p) / 365.0);
    }
    if m.external_capacity > 1.0
        && value > Money::ZERO
        && let Some(cap) = capacity(catalog, site)
        && value > cap
    {
        let outside = (value - cap).to_usd() / value.to_usd();
        cost += cost.scale(outside * (m.external_rate - 1.0).max(0.0));
    }
    cost
}

/// Lots of perishable goods (BP3), checked every day: what came in since yesterday gets
/// a lot with the day, what was taken leaves the oldest lots, and lots past their shelf
/// life are disposed of.
pub(crate) fn day(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let m = &catalog.production_model.storage;
    let mut news = Vec::new();
    if !m.enabled || m.shelf_life.iter().all(Option::is_none) {
        return news;
    }
    for index in 0..state.sites.len() {
        if state.sites[index].closed.is_some() {
            continue;
        }
        let site = SiteId(u32::try_from(index).expect("site count fits u32"));
        let mut losses = Vec::new();
        for (&p, stock) in &mut state.sites[index].inventory {
            let Some(days) = m.shelf_life(p) else {
                continue;
            };
            settle_lots(stock, date);
            let expired: f64 = stock
                .lots
                .iter()
                .filter(|(since, _)| {
                    since.days_until(date) >= i32::try_from(days).unwrap_or(i32::MAX)
                })
                .map(|&(_, q)| q)
                .sum();
            if expired > 1e-9 {
                let before = stock.quantity;
                let lost = stock.take(expired);
                settle_lots(stock, date);
                if lost > Money::ZERO || expired > 0.0 {
                    losses.push((p, lost, expired / before.max(1e-9)));
                }
            }
        }
        let (owner, country) = (state.sites[index].owner, state.sites[index].country);
        let ledger = &mut state.companies[owner.index()].ledger;
        for &(p, lost, _) in &losses {
            if lost > Money::ZERO {
                ledger.expense(
                    CostType::Storage,
                    CostCenter::product(site, p),
                    Account::Inventory,
                    lost,
                );
            }
        }
        if let Some(company) = state.news_of(owner) {
            for (p, lost, share) in losses {
                if share >= m.spoil_message_share {
                    news.push(
                        Message::new(MessageKind::Warning, keys::SPOILED)
                            .with(
                                "produkt",
                                Param::TextKey(format!("produkt.{}", catalog.products.key(p))),
                            )
                            .with(
                                "land",
                                Param::Country(catalog.countries.key(country).to_owned()),
                            )
                            .with("anteil", Param::Number((share * 1000.0).round() / 10.0))
                            .with("wert", Param::Money(lost))
                            .of(company.clone()),
                    );
                }
            }
        }
    }
    news
}

/// Brings the lots of a stock in line with its quantity: takings leave the oldest lots
/// first, new goods get a lot with `date`.
pub(crate) fn settle_lots(stock: &mut Stock, date: Date) {
    let in_lots: f64 = stock.lots.iter().map(|&(_, q)| q).sum();
    if in_lots > stock.quantity + 1e-9 {
        let mut gone = in_lots - stock.quantity;
        while gone > 1e-12 {
            let Some(first) = stock.lots.first_mut() else {
                break;
            };
            if first.1 <= gone {
                gone -= first.1;
                stock.lots.remove(0);
            } else {
                first.1 -= gone;
                gone = 0.0;
            }
        }
    } else if stock.quantity > in_lots + 1e-9 {
        let added = stock.quantity - in_lots;
        match stock.lots.last_mut() {
            Some(last) if last.0 == date => last.1 += added,
            _ => stock.lots.push((date, added)),
        }
    }
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
