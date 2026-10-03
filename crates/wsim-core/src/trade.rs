//! Trade between countries (Lastenheft §8, §9.2; formulas in docs/FORMELN.md, M8).
//!
//! Traders link the country markets: they buy from sale offers that their policy allows,
//! ship the goods and sell them on markets where demand is open and the price covers
//! purchase, transport and their margin. Stage 1 treats them as one competitive
//! trading network outside the companies; their profits leave the game. Companies
//! move their own goods with `TransferGoods`.

use std::collections::BTreeMap;

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::ids::{CountryId, ProductId};
use crate::market;
use crate::money::Money;
use crate::policy::{self, BuyerGroup};
use crate::state::{Consignee, GameState, SiteId};

/// A purchase the traders plan for today.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlannedBuy {
    pub site: SiteId,
    pub destination: CountryId,
    pub quantity: f64,
    pub transport_per_unit: Money,
    pub days: u32,
}

/// Delivers all shipments that arrive today, in the order they were sent.
pub(crate) fn deliver(state: &mut GameState, today: Date) {
    if state.shipments.iter().all(|s| s.arrival > today) {
        return;
    }
    let (arrived, underway) = std::mem::take(&mut state.shipments)
        .into_iter()
        .partition(|s| s.arrival <= today);
    state.shipments = underway;
    for s in arrived {
        let stock = match s.to {
            Consignee::Site(site) => state.sites[site.index()]
                .inventory
                .entry(s.product)
                .or_default(),
            Consignee::Importer(country) => {
                &mut state.markets.get_mut(s.product).get_mut(country).imports
            }
        };
        stock.add(s.quantity, s.value, s.quality);
    }
}

/// Quantities on the way to the traders, by product and destination.
pub(crate) fn to_importers(state: &GameState) -> BTreeMap<(ProductId, CountryId), f64> {
    let mut sums = BTreeMap::new();
    for s in &state.shipments {
        if let Consignee::Importer(country) = s.to {
            *sums.entry((s.product, country)).or_default() += s.quantity;
        }
    }
    sums
}

/// An offer traders may buy from.
struct Source {
    site: SiteId,
    country: CountryId,
    price: Money,
    available: f64,
}

/// Plans today's purchases of the traders for one product: each market with open
/// demand is supplied from the offers with the lowest landed cost, markets with the
/// highest margin first.
pub(crate) fn plan(
    state: &GameState,
    catalog: &Catalog,
    product: ProductId,
    sites_by_country: &[Vec<SiteId>],
    in_transit: &BTreeMap<(ProductId, CountryId), f64>,
) -> Vec<PlannedBuy> {
    let model = &catalog.market_model;
    let mut sources: Vec<Source> = Vec::new();
    for sites in sites_by_country {
        for &site in sites {
            let s = &state.sites[site.index()];
            let Some(offer) = s.offers.get(&product) else {
                continue;
            };
            let owner = &state.companies[s.owner.index()];
            let (rule, _) = policy::sales_rule(owner, BuyerGroup::Traders, product, s.country);
            let allowance = rule.allowance(offer.price, offer.to_traders_month);
            let stock = s.inventory.get(&product).map_or(0.0, |st| st.quantity);
            let available = (stock - offer.keep).max(0.0).min(allowance);
            if available > 1e-9 {
                sources.push(Source {
                    site,
                    country: s.country,
                    price: offer.price,
                    available,
                });
            }
        }
    }
    if sources.is_empty() {
        return Vec::new();
    }

    // Markets with open demand and the sources whose landed cost the price covers.
    struct Destination {
        country: CountryId,
        need: f64,
        margin: f64,
        /// (landed cost, source index, transport per unit, days), cheapest first
        candidates: Vec<(Money, usize, Money, u32)>,
    }
    let mut destinations: Vec<Destination> = Vec::new();
    for country in catalog.countries.ids() {
        let m = state.markets.get(product).get(country);
        let transit = in_transit.get(&(product, country)).copied().unwrap_or(0.0);
        let need = model.trader_cover_days * m.open_demand - m.imports.quantity - transit;
        if need <= 1e-9 {
            continue;
        }
        let price = market::market_price(catalog, state, country, product);
        let mut candidates: Vec<(Money, usize, Money, u32)> = sources
            .iter()
            .enumerate()
            .filter(|(_, s)| s.country != country)
            .filter_map(|(i, s)| {
                let (transport, days) = state
                    .routes
                    .for_product(catalog, product, s.country, country)?;
                let landed = s.price + transport;
                (landed.scale(1.0 + model.trader_margin) <= price)
                    .then_some((landed, i, transport, days))
            })
            .collect();
        if candidates.is_empty() {
            continue;
        }
        candidates.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        let margin = (price - candidates[0].0).to_usd() / price.to_usd().max(1e-9);
        destinations.push(Destination {
            country,
            need,
            margin,
            candidates,
        });
    }
    destinations.sort_by(|a, b| {
        b.margin
            .total_cmp(&a.margin)
            .then(a.country.cmp(&b.country))
    });

    let mut remaining: Vec<f64> = sources.iter().map(|s| s.available).collect();
    let mut plan = Vec::new();
    for d in destinations {
        let mut need = d.need;
        for (_, i, transport, days) in d.candidates {
            if need <= 1e-9 {
                break;
            }
            let quantity = need.min(remaining[i]);
            if quantity <= 1e-9 {
                continue;
            }
            remaining[i] -= quantity;
            need -= quantity;
            plan.push(PlannedBuy {
                site: sources[i].site,
                destination: d.country,
                quantity,
                transport_per_unit: transport,
                days,
            });
        }
    }
    plan
}
