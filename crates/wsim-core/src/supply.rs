//! Moving goods between sites of a company or its group: one-off transfers and standing
//! internal supply by a fixed quantity or by need (docs/FORMELN.md, W5 Interne Belieferung).

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::contracts::{self, DAYS_PER_MONTH};
use crate::ids::{CountryId, ProductId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::money::Money;
use crate::plots;
use crate::state::{Consignee, GameState, Shipment, SiteId};

const EPS: f64 = 1e-9;

/// How much a site draws from its source.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum SupplyMode {
    /// A fixed quantity a day.
    Fixed { per_day: f64 },
    /// Stock for this many days of the site's need, counting goods on the way.
    Need { days: f64 },
}

/// A product a site draws from another site of its company or group.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InternalSupply {
    pub source: SiteId,
    pub mode: SupplyMode,
    #[serde(default)]
    pub delivered_month: f64,
    #[serde(default)]
    pub delivered_last_month: f64,
}

/// Freight per unit and days of the way between two sites, and the customs rate; `None`
/// without a route or under an embargo. Within a country there is neither.
pub fn route(
    state: &GameState,
    catalog: &Catalog,
    (from, to): (SiteId, SiteId),
    product: ProductId,
) -> Option<(Option<(Money, u32)>, f64)> {
    let (a, b) = (&state.sites[from.index()], &state.sites[to.index()]);
    let (from_country, to_country): (CountryId, CountryId) = (a.country, b.country);
    // Sea freight is cheaper where either end lies at a port (M35).
    let sea = plots::sea_freight(catalog, state, a).min(plots::sea_freight(catalog, state, b));
    let way = if from_country == to_country {
        None
    } else {
        Some(
            state
                .routes
                .for_product_via(catalog, product, (from_country, to_country), sea)?,
        )
    };
    let rate = state
        .tariffs
        .for_product(catalog, from_country, to_country, product)?;
    Some((way, rate))
}

/// Moves goods from one site to another of the same group. The receiving company pays
/// freight and customs, and the goods' value when they come from another company of the
/// group (docs/FORMELN.md).
pub(crate) fn transfer(
    state: &mut GameState,
    catalog: &Catalog,
    (from, to): (SiteId, SiteId),
    product: ProductId,
    quantity: f64,
    today: Date,
) -> Result<(), CommandError> {
    let payer = state.sites[to.index()].owner;
    let seller = state.sites[from.index()].owner;
    let (from_country, to_country) = (
        state.sites[from.index()].country,
        state.sites[to.index()].country,
    );
    let (way, rate) = route(state, catalog, (from, to), product).ok_or_else(|| {
        if state
            .tariffs
            .for_product(catalog, from_country, to_country, product)
            .is_none()
        {
            CommandError::Embargo {
                from: catalog.countries.key(from_country).to_owned(),
                to: catalog.countries.key(to_country).to_owned(),
            }
        } else {
            CommandError::NoRoute {
                product: catalog.products.key(product).to_owned(),
                from: catalog.countries.key(from_country).to_owned(),
                to: catalog.countries.key(to_country).to_owned(),
            }
        }
    })?;
    let held = state.sites[from.index()].inventory.get(&product);
    let available = held.map_or(0.0, |s| s.quantity);
    if quantity > available + 1e-9 {
        return Err(CommandError::NotEnoughGoods { available });
    }
    // Customs on the value of the goods in stock (W3).
    let customs = held.map_or(Money::ZERO, |s| {
        s.value
            .scale(rate * (quantity / s.quantity.max(1e-12)).min(1.0))
    });
    // Another company of the group sells at the value of its stock.
    let price = if seller == payer {
        Money::ZERO
    } else {
        held.map_or(Money::ZERO, |s| {
            s.value.scale((quantity / s.quantity.max(1e-12)).min(1.0))
        })
    };
    // The way of the load (W5): market, state or own fleet.
    let plan = way.map(|(per_unit, _)| {
        crate::logistics::plan(
            state,
            catalog,
            payer,
            (product, quantity),
            (from_country, to_country),
            Money::times(per_unit, quantity),
        )
    });
    let cost = plan.as_ref().map_or(Money::ZERO, |p| p.cost);
    let needed = if way.is_some() {
        cost + customs + price
    } else {
        price
    };
    if needed > Money::ZERO && state.companies[payer.index()].ledger.cash() < needed {
        return Err(CommandError::NotEnoughCash { needed });
    }
    let days = if let (Some((_, days)), Some(plan)) = (way, &plan) {
        let ledger = &mut state.companies[payer.index()].ledger;
        ledger.expense(
            CostType::Transport,
            CostCenter::product(from, product),
            Account::Cash,
            plan.cost,
        );
        if customs > Money::ZERO {
            ledger.expense(
                CostType::Customs,
                CostCenter::product(to, product),
                Account::Cash,
                customs,
            );
        }
        Some(days)
    } else {
        None
    };
    let stock = state.sites[from.index()]
        .inventory
        .entry(product)
        .or_default();
    let quality = stock.quality;
    let value = stock.take(quantity);
    if seller != payer {
        let center = CostCenter::product(from, product);
        let ledger = &mut state.companies[seller.index()].ledger;
        ledger.income(CostType::Revenue, center, Account::Cash, value);
        ledger.expense(CostType::InventoryChange, center, Account::Inventory, value);
        state.companies[payer.index()]
            .ledger
            .transfer(Account::Inventory, Account::Cash, value);
    }
    let lost = plan.is_some_and(|p| crate::logistics::book(state, catalog, payer, &p));
    match days {
        None => state.sites[to.index()]
            .inventory
            .entry(product)
            .or_default()
            .add(quantity, value, quality),
        Some(days) => state.shipments.push(Shipment {
            product,
            quantity,
            quality,
            value,
            from: from_country,
            to: Consignee::Site(to),
            arrival: today.add_days(i32::try_from(days).expect("short route")),
            lost,
        }),
    }
    Ok(())
}

/// What a site needs of a product a day: its recipes at the planned utilization and the
/// sales of its offer (last month, else this month so far).
pub fn daily_demand(
    state: &GameState,
    catalog: &Catalog,
    site: SiteId,
    product: ProductId,
    today: Date,
) -> f64 {
    let s = &state.sites[site.index()];
    let sales = s.offers.get(&product).map_or(0.0, |o| {
        if o.sold_last_month > EPS {
            o.sold_last_month / DAYS_PER_MONTH
        } else if today.day() > 1 {
            o.sold_month / f64::from(today.day() - 1)
        } else {
            0.0
        }
    });
    contracts::daily_need(state, catalog, site, product) + sales
}

/// Goods on the way to a site.
pub fn in_transit(state: &GameState, site: SiteId, product: ProductId) -> f64 {
    state
        .shipments
        .iter()
        .filter(|s| s.product == product && !s.lost && s.to == Consignee::Site(site))
        .map(|s| s.quantity)
        .sum()
}

/// What a source can give: its stock above the reserve of its offer and a day of its own
/// use.
pub fn spare(state: &GameState, catalog: &Catalog, source: SiteId, product: ProductId) -> f64 {
    let s = &state.sites[source.index()];
    let stock = s.inventory.get(&product).map_or(0.0, |x| x.quantity);
    let keep = s.offers.get(&product).map_or(0.0, |o| o.keep);
    (stock - keep - contracts::daily_need(state, catalog, source, product)).max(0.0)
}

/// What a site asks of its source today.
pub fn wanted(
    state: &GameState,
    catalog: &Catalog,
    site: SiteId,
    product: ProductId,
    mode: SupplyMode,
    today: Date,
) -> f64 {
    match mode {
        SupplyMode::Fixed { per_day } => per_day,
        SupplyMode::Need { days } => {
            let stock = state.sites[site.index()]
                .inventory
                .get(&product)
                .map_or(0.0, |x| x.quantity);
            let target = daily_demand(state, catalog, site, product, today) * days;
            (target - stock - in_transit(state, site, product)).max(0.0)
        }
    }
}

/// Whether a site may draw from a source: another site of the same group, not bankrupt.
pub fn allowed(state: &GameState, site: SiteId, source: SiteId) -> bool {
    let (Some(a), Some(b)) = (state.site(site), state.site(source)) else {
        return false;
    };
    site != source
        && crate::group::same_group(state, a.owner, b.owner)
        && !state.companies[a.owner.index()].bankrupt
        && !state.companies[b.owner.index()].bankrupt
}

/// Checks a mode against the data (docs/FORMELN.md).
pub(crate) fn check_mode(catalog: &Catalog, mode: SupplyMode) -> Result<(), CommandError> {
    let ok = match mode {
        SupplyMode::Fixed { per_day } => per_day.is_finite() && per_day > 0.0,
        SupplyMode::Need { days } => {
            let max = catalog.logistics.supply_days_max;
            days.is_finite() && days > 0.0 && (max <= 0.0 || days <= max)
        }
    };
    if ok {
        Ok(())
    } else {
        Err(CommandError::InvalidQuantity)
    }
}

/// Each day after production and contracts: every site draws what it asks of its source,
/// as far as the source can spare it and the receiver can pay.
pub(crate) fn simulate_day(state: &mut GameState, catalog: &Catalog, today: Date) {
    if today.day() == 1 {
        for s in &mut state.sites {
            for x in s.supplies.values_mut() {
                x.delivered_last_month = x.delivered_month;
                x.delivered_month = 0.0;
            }
        }
    }
    let jobs: Vec<(SiteId, ProductId)> = state
        .sites
        .iter()
        .enumerate()
        .flat_map(|(i, s)| {
            let site = SiteId(u32::try_from(i).expect("site count fits u32"));
            s.supplies.keys().map(move |&p| (site, p))
        })
        .collect();
    for (site, product) in jobs {
        let Some(x) = state.sites[site.index()].supplies.get(&product).cloned() else {
            continue;
        };
        if !allowed(state, site, x.source) {
            continue;
        }
        let quantity = wanted(state, catalog, site, product, x.mode, today)
            .min(spare(state, catalog, x.source, product));
        if quantity <= EPS {
            continue;
        }
        if transfer(state, catalog, (x.source, site), product, quantity, today).is_ok()
            && let Some(y) = state.sites[site.index()].supplies.get_mut(&product)
        {
            y.delivered_month += quantity;
        }
    }
}
