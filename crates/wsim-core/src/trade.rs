//! Trade between countries (Lastenheft §8, §9.2; formulas in docs/FORMELN.md, M8).
//!
//! Traders link the country markets: they buy from sale offers that their policy allows,
//! ship the goods and sell them on markets where demand is open and the price covers
//! purchase, transport and their margin. Stage 1 treats them as one competitive
//! trading network outside the companies; their profits leave the game. Companies
//! move their own goods with `TransferGoods`.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::ids::{CountryId, ProductId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::market;
use crate::message::Message;
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

/// Delivers all shipments that arrive today, in the order they were sent. A lost load
/// (W5) is written off at its owner instead; the player hears of it.
pub(crate) fn deliver(state: &mut GameState, catalog: &Catalog, today: Date) -> Vec<Message> {
    let mut news = Vec::new();
    if state.shipments.iter().all(|s| s.arrival > today) {
        return news;
    }
    let (arrived, underway) = std::mem::take(&mut state.shipments)
        .into_iter()
        .partition(|s| s.arrival <= today);
    state.shipments = underway;
    for s in arrived {
        if s.lost {
            if let Consignee::Site(site) = s.to {
                let (owner, to) = {
                    let x = &state.sites[site.index()];
                    (x.owner, x.country)
                };
                state.companies[owner.index()].ledger.expense(
                    CostType::Other,
                    CostCenter::product(site, s.product),
                    Account::Inventory,
                    s.value,
                );
                crate::logistics::note_loss(state, owner, s.value);
                if owner == state.player {
                    news.push(crate::logistics::loss_message(
                        catalog,
                        (s.product, s.quantity),
                        (s.from, to),
                    ));
                }
            }
            continue;
        }
        let stock = match s.to {
            Consignee::Site(site) => state.sites[site.index()]
                .inventory
                .entry(s.product)
                .or_default(),
            Consignee::Importer(country) => {
                state.import_markets.insert((s.product, country));
                &mut state.markets.get_mut(s.product).get_mut(country).imports
            }
        };
        stock.add(s.quantity, s.value, s.quality);
    }
    news
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

/// Transport per unit, days and tariff from one country to another.
type Leg = (Money, u32, f64);

/// An offer traders may buy from.
struct Source {
    site: SiteId,
    country: CountryId,
    price: Money,
    available: f64,
    /// Factor on freight by sea from the site's plot (M35).
    sea_freight: f64,
}

/// Today's plan of the traders for one product.
pub(crate) struct Plan {
    pub buys: Vec<PlannedBuy>,
    /// Need the offers could not cover: sellers who sold out may raise their prices for
    /// it (M16).
    pub unserved: f64,
    /// Per country holding imports: the lowest landed cost of fresh goods from abroad.
    pub replacement: BTreeMap<CountryId, Money>,
}

/// Plans today's purchases of the traders for one product: each market with open
/// demand is supplied from the offers with the lowest landed cost, markets with the
/// highest margin first.
pub(crate) fn plan(
    state: &GameState,
    catalog: &Catalog,
    product: ProductId,
    traders: &market::Traders,
    in_transit: &BTreeMap<(ProductId, CountryId), f64>,
) -> Plan {
    let model = &catalog.market_model;
    let mut sources: Vec<Source> = Vec::new();
    for sites in traders.values() {
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
                    sea_freight: crate::plots::sea_freight(catalog, state, s),
                });
            }
        }
    }
    if sources.is_empty() {
        // Nobody has goods left: open demand anywhere is unserved.
        let open: f64 = catalog
            .countries
            .ids()
            .map(|c| market::open_demand(state.markets.get(product).get(c), model, state.date))
            .sum();
        return Plan {
            buys: Vec::new(),
            unserved: open,
            replacement: BTreeMap::new(),
        };
    }
    // What fresh goods would cost where traders hold stock: other traders would sell at
    // that, so old stock bought dearly cannot ask more (M16).
    // The freight depends on the countries and the sea freight factor only: the
    // cheapest offer of each.
    let mut cheapest_from: BTreeMap<(CountryId, u64), Money> = BTreeMap::new();
    for s in &sources {
        cheapest_from
            .entry((s.country, s.sea_freight.to_bits()))
            .and_modify(|p| *p = (*p).min(s.price))
            .or_insert(s.price);
    }
    let mut replacement = BTreeMap::new();
    for &(p, country) in &state.import_markets {
        if p != product {
            continue;
        }
        let cheapest = cheapest_from
            .iter()
            .filter(|&(&(from, _), _)| from != country)
            .filter_map(|(&(from, sea), &price)| {
                let (transport, _) = state.routes.for_product_via(
                    catalog,
                    product,
                    (from, country),
                    f64::from_bits(sea),
                )?;
                let tariff = state.tariffs.for_product(catalog, from, country, product)?;
                Some((price + transport).scale(1.0 + tariff))
            })
            .min();
        if let Some(landed) = cheapest {
            replacement.insert(country, landed);
        }
    }

    // Markets with open demand and the sources whose landed cost the price covers.
    struct Destination {
        country: CountryId,
        /// For the open demand; any source may cover it.
        need: f64,
        /// For a price island (C1); only sources far enough below the price cover it.
        arbitrage: f64,
        /// Landed cost below which a source covers the arbitrage need.
        arbitrage_below: Money,
        margin: f64,
        /// (landed cost, source index, transport and customs per unit, days), cheapest
        /// on top
        candidates: BinaryHeap<Reverse<(Money, usize, Money, u32)>>,
    }
    let mut destinations: Vec<Destination> = Vec::new();
    for country in catalog.countries.ids() {
        let m = state.markets.get(product).get(country);
        let open = market::open_demand(m, model, state.date);
        // Sales at home last month per day, the base of the arbitrage (C1).
        let sales = (m.last_month.sold - m.last_month.exported).max(0.0) / 30.0;
        if open <= 1e-9 && (model.arbitrage_share <= 0.0 || sales <= 1e-9) {
            continue;
        }
        // Companies that keep a stock state what they would pay; a market without
        // sellers has no price movement that could show it.
        let bid = traders
            .get(&country)
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .filter_map(|&s| state.sites[s.index()].orders.get(&product))
            .filter(|o| o.target > 0.0)
            .map(|o| o.max_price)
            .max()
            .unwrap_or(Money::ZERO);
        let price = market::market_price(catalog, state, country, product).max(bid);
        // Open demand is worth supplying up to the highest price the market accepts; the
        // demand adapts to the price. Waiting for the index to rise would leave a market
        // without sellers unserved for good: without sales its index never moves.
        let ceiling = market::local_reference(catalog, state, country, product)
            .scale(model.price_max_factor)
            .max(price);
        // The route depends on the countries and the sea freight factor only: looked up
        // once for each.
        // With the tariff on top (W3); `None` under an embargo.
        let mut routes: BTreeMap<(CountryId, u64), Option<Leg>> = BTreeMap::new();
        let mut candidates: Vec<Reverse<(Money, usize, Money, u32)>> = Vec::new();
        for (i, s) in sources.iter().enumerate() {
            if s.country == country {
                continue;
            }
            let route = *routes
                .entry((s.country, s.sea_freight.to_bits()))
                .or_insert_with(|| {
                    let tariff = state
                        .tariffs
                        .for_product(catalog, s.country, country, product)?;
                    let (transport, days) = state.routes.for_product_via(
                        catalog,
                        product,
                        (s.country, country),
                        s.sea_freight,
                    )?;
                    Some((transport, days, tariff))
                });
            let Some((transport, days, tariff)) = route else {
                continue;
            };
            // Transport and customs per unit go into the value of the imported goods.
            let landed = (s.price + transport).scale(1.0 + tariff);
            if landed.scale(1.0 + model.trader_margin) <= ceiling {
                candidates.push(Reverse((landed, i, landed - s.price, days)));
            }
        }
        // Taken cheapest first, as long as the need lasts: a heap instead of sorting
        // all. The source index makes every key unique, so the order is that of a sort.
        let candidates = BinaryHeap::from(candidates);
        let Some(&Reverse(cheapest)) = candidates.peek() else {
            continue;
        };
        // A price island (C1): the cheapest goods from abroad with margin and gap stay
        // below the price; traders then bring a share of the sales as well.
        let island = (1.0 + model.trader_margin) * (1.0 + model.arbitrage_gap);
        let arbitrage_below = price.scale(1.0 / island);
        let extra = if cheapest.0 < arbitrage_below {
            model.arbitrage_share * sales
        } else {
            0.0
        };
        // Stock and goods on the way cover the days at sea as well: with the cover
        // alone, a route longer than it would only ever bring part of the demand.
        let transit = in_transit.get(&(product, country)).copied().unwrap_or(0.0);
        let days = f64::from(cheapest.3);
        let cover = model.trader_cover_days + days;
        let held = m.imports.quantity + transit;
        let need = cover * open - held;
        let total = cover * (open + extra) - held;
        if total <= 1e-9 {
            continue;
        }
        let margin = (price - cheapest.0).to_usd() / price.to_usd().max(1e-9);
        destinations.push(Destination {
            country,
            need: need.max(0.0),
            arbitrage: total - need.max(0.0),
            arbitrage_below,
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
    let mut unserved = 0.0;
    for d in destinations {
        let mut need = d.need;
        let mut arbitrage = d.arbitrage;
        let mut candidates = d.candidates;
        while let Some(Reverse((landed, i, transport, days))) = candidates.pop() {
            // Cheapest first: once a source is too dear for the arbitrage, all are.
            if landed >= d.arbitrage_below {
                arbitrage = 0.0;
            }
            if need + arbitrage <= 1e-9 {
                break;
            }
            let quantity = (need + arbitrage).min(remaining[i]);
            if quantity <= 1e-9 {
                continue;
            }
            remaining[i] -= quantity;
            let from_open = quantity.min(need);
            need -= from_open;
            arbitrage -= quantity - from_open;
            plan.push(PlannedBuy {
                site: sources[i].site,
                destination: d.country,
                quantity,
                transport_per_unit: transport,
                days,
            });
        }
        unserved += need.max(0.0);
    }
    Plan {
        buys: plan,
        unserved,
        replacement,
    }
}
