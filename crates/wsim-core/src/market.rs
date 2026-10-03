//! Markets and prices (Lastenheft §9.1–9.2; formulas in docs/FORMELN.md, section M7).
//!
//! Each product has a market in each country. Sellers are sites with sale offers and
//! the state market; buyers are sites with purchase orders, the government and the
//! consumers in five income layers. Consumer and government demand are set at the start
//! of each month; markets with sellers or buyers are cleared every day.

use crate::calendar::Date;
use crate::catalog::{Catalog, ConsumerDemand, ConsumptionType};
use crate::ids::{CountryId, Id, ProductId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::math;
use crate::money::Money;
use crate::state::{CompanyId, GameState, PriceMode, SiteId, Trade};

/// Market price of a product in a country: the price index, or before the first sale
/// the reference price at the country's price level.
pub fn market_price(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    product: ProductId,
) -> Money {
    let market = state.markets.get(product).get(country);
    if market.price > Money::ZERO {
        market.price
    } else {
        local_reference(catalog, state, country, product)
    }
}

/// Reference price at the country's price level.
pub fn local_reference(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    product: ProductId,
) -> Money {
    catalog
        .products
        .get(product)
        .reference_price
        .scale(state.countries.get(country).price_level)
}

/// Share of a household layer that buys (0–1), from income and price.
pub fn propensity(demand: &ConsumerDemand, income: f64, price: f64, reference: f64) -> f64 {
    if income <= 0.0 || price <= 0.0 || reference <= 0.0 {
        return 0.0;
    }
    let a = math::pow(
        income / (demand.purchase_threshold * reference),
        demand.income_sensitivity,
    ) * math::pow(reference / price, demand.price_sensitivity);
    a / (1.0 + a)
}

/// Start of a month: updates the ownership of durables from last month's purchases
/// and sets the consumer and government demand per day.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) {
    let model = &catalog.market_model;
    let month = usize::try_from(date.month() - 1).expect("month 1-12");
    for (product, p) in catalog.products.iter() {
        if p.consumer_demand.is_none() && p.state_demand.is_none() {
            continue;
        }
        // Products that displace this one (§6.4): their ownership lowers our target.
        let successors: Vec<(ProductId, f64)> = catalog
            .products
            .iter()
            .filter(|(_, q)| q.replaces.contains(&product))
            .filter_map(|(id, q)| match q.consumer_demand.as_ref()?.consumption {
                ConsumptionType::Durable { max_ownership, .. } => Some((id, max_ownership)),
                ConsumptionType::Consumable { .. } => None,
            })
            .collect();
        for country in catalog.countries.ids() {
            let cs = state.countries.get(country);
            let reference = p.reference_price.to_usd() * cs.price_level;
            let per_layer = cs.population / 5.0;
            let incomes = cs.income_quintiles_usd;
            let gdp = cs.population * cs.gdp_per_capita_usd * cs.price_level;
            let displaced: [f64; 5] = std::array::from_fn(|q| {
                successors.iter().fold(1.0, |f, &(s, max)| {
                    f * (1.0 - (state.markets.get(s).get(country).ownership[q] / max).min(1.0))
                })
            });
            let market = state.markets.get_mut(product).get_mut(country);
            market.close_month();
            if market.price == Money::ZERO {
                market.price = Money::from_usd(reference).unwrap_or(Money::ZERO);
            }
            let price = market.price.to_usd();
            if let Some(d) = &p.consumer_demand {
                let season = d.seasonality.map_or(1.0, |s| s[month]);
                for q in 0..5 {
                    let share = propensity(d, incomes[q], price, reference);
                    market.consumer_rate[q] = match d.consumption {
                        ConsumptionType::Consumable {
                            per_capita_per_year,
                        } => per_capita_per_year * share * per_layer / 365.0 * season,
                        ConsumptionType::Durable {
                            service_life_years,
                            max_ownership,
                        } => {
                            if per_layer > 0.0 {
                                let owned = &mut market.ownership[q];
                                *owned += market.bought[q] / per_layer
                                    - *owned / (service_life_years * 12.0);
                                *owned = owned.max(0.0);
                            }
                            let target = max_ownership * share * displaced[q];
                            let owned = market.ownership[q];
                            let gap = (target - owned).max(0.0);
                            (gap * model.adoption_per_year + owned / service_life_years) * per_layer
                                / 365.0
                        }
                    };
                    market.bought[q] = 0.0;
                }
            }
            if let Some(s) = &p.state_demand {
                market.state_rate = s.per_million_gdp * gdp / 1.0e6 / 365.0;
            }
        }
    }
}

/// A seller on one market for one day.
struct Offer {
    site: Option<SiteId>,
    owner: Option<CompanyId>,
    price: Money,
    available: f64,
    quality: f64,
    sold: f64,
}

enum Buyer {
    Site(SiteId),
    Outside,
}

/// Clears all markets that have sellers or buyers today.
pub(crate) fn clear(state: &mut GameState, catalog: &Catalog, date: Date) {
    let countries = catalog.countries.len();
    let mut sites_by_country: Vec<Vec<SiteId>> = vec![Vec::new(); countries];
    for (i, site) in state.sites.iter().enumerate() {
        let active = !site.offers.is_empty() || !site.orders.is_empty();
        if active && !state.companies[site.owner.index()].bankrupt {
            sites_by_country[site.country.index()]
                .push(SiteId(u32::try_from(i).expect("fits u32")));
        }
    }
    for (product, p) in catalog.products.iter() {
        let state_market = p.state_market.filter(|m| {
            m.available_from.is_none_or(|y| y <= date.year())
                && m.available_until.is_none_or(|y| y >= date.year())
        });
        for country in catalog.countries.ids() {
            let sites = &sites_by_country[country.index()];
            let has_site_trade = sites.iter().any(|s| {
                let site = &state.sites[s.index()];
                site.offers.contains_key(&product) || site.orders.contains_key(&product)
            });
            let has_outside_demand = {
                let m = state.markets.get(product).get(country);
                m.state_rate > 0.0 || m.consumer_rate.iter().any(|&r| r > 0.0)
            };
            if !has_site_trade && !(state_market.is_some() && has_outside_demand) {
                continue;
            }
            let state_price =
                state_market.map(|m| m.price.scale(state.countries.get(country).price_level));
            clear_market(state, catalog, country, product, sites, state_price);
        }
    }
}

fn clear_market(
    state: &mut GameState,
    catalog: &Catalog,
    country: CountryId,
    product: ProductId,
    sites: &[SiteId],
    state_market_price: Option<Money>,
) {
    let model = &catalog.market_model;
    let reference = local_reference(catalog, state, country, product).to_usd();
    let mut offers: Vec<Offer> = Vec::new();
    for &site in sites {
        let s = &state.sites[site.index()];
        if let Some(offer) = s.offers.get(&product) {
            let stock = s.inventory.get(&product);
            offers.push(Offer {
                site: Some(site),
                owner: Some(s.owner),
                price: offer.price,
                available: stock.map_or(0.0, |st| (st.quantity - offer.keep).max(0.0)),
                quality: stock.map_or(50.0, |st| st.quality),
                sold: 0.0,
            });
        }
    }
    if let Some(price) = state_market_price {
        offers.push(Offer {
            site: None,
            owner: None,
            price,
            available: f64::INFINITY,
            quality: 50.0,
            sold: 0.0,
        });
    }
    let available_before: Vec<f64> = offers.iter().map(|o| o.available).collect();
    let mut by_price: Vec<usize> = (0..offers.len()).collect();
    by_price.sort_by(|&a, &b| offers[a].price.cmp(&offers[b].price).then(a.cmp(&b)));
    let mut day = Trade::default();

    // 1. Industry: purchase orders, highest willingness to pay first.
    let mut buyers: Vec<(SiteId, f64, Money, f64)> = sites
        .iter()
        .filter_map(|&site| {
            let s = &state.sites[site.index()];
            let order = s.orders.get(&product)?;
            let stock = s.inventory.get(&product).map_or(0.0, |st| st.quantity);
            Some((
                site,
                (order.target - stock).max(0.0),
                order.max_price,
                order.min_quality,
            ))
        })
        .collect();
    buyers.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    for (site, mut need, max_price, min_quality) in buyers {
        day.demand += need;
        let owner = state.sites[site.index()].owner;
        for &i in &by_price {
            if need <= 1e-9 {
                break;
            }
            let o = &offers[i];
            if o.owner == Some(owner)
                || o.price > max_price
                || o.quality < min_quality
                || o.available <= 0.0
            {
                continue;
            }
            let quantity = need.min(o.available);
            trade(
                state,
                &mut offers[i],
                quantity,
                Buyer::Site(site),
                product,
                &mut day,
            );
            need -= quantity;
        }
    }

    // 2. Government: cheapest offers up to a price cap.
    let state_need = state.markets.get(product).get(country).state_rate;
    if state_need > 0.0 {
        day.demand += state_need;
        let cap = Money::from_usd(reference * model.state_price_cap).unwrap_or(Money::ZERO);
        let mut need = state_need;
        for &i in &by_price {
            if need <= 1e-9 {
                break;
            }
            if offers[i].price > cap || offers[i].available <= 0.0 {
                continue;
            }
            let quantity = need.min(offers[i].available);
            trade(
                state,
                &mut offers[i],
                quantity,
                Buyer::Outside,
                product,
                &mut day,
            );
            need -= quantity;
        }
    }

    // 3. Consumers: richest layer first; sellers chosen by attractiveness (logit).
    let rates = state.markets.get(product).get(country).consumer_rate;
    let mut bought = [0.0; 5];
    for q in (0..5).rev() {
        let mut need = rates[q];
        day.demand += need;
        for _ in 0..8 {
            if need <= 1e-9 {
                break;
            }
            let weights: Vec<f64> = offers
                .iter()
                .map(|o| {
                    if o.available <= 1e-12 || reference <= 0.0 {
                        return 0.0;
                    }
                    let relative = (o.price.to_usd() / reference).max(1e-6);
                    math::exp(
                        -model.price_weight[q] * math::ln(relative)
                            + model.quality_weight[q] * (o.quality - 50.0) / 25.0,
                    )
                })
                .collect();
            let total: f64 = weights.iter().sum();
            if total <= 0.0 {
                break;
            }
            let mut taken = 0.0;
            for i in 0..offers.len() {
                if weights[i] <= 0.0 {
                    continue;
                }
                let quantity = (need * weights[i] / total).min(offers[i].available);
                if quantity > 0.0 {
                    trade(
                        state,
                        &mut offers[i],
                        quantity,
                        Buyer::Outside,
                        product,
                        &mut day,
                    );
                    taken += quantity;
                }
            }
            need -= taken;
            bought[q] += taken;
        }
    }

    // Prices of automatic sellers and the market's price index.
    let unmet = day.unmet() > 1e-9;
    for (i, o) in offers.iter().enumerate() {
        let Some(site) = o.site else { continue };
        let Some(offer) = state.sites[site.index()].offers.get_mut(&product) else {
            continue;
        };
        offer.sold_today = o.sold;
        offer.sold_month += o.sold;
        match offer.mode {
            PriceMode::Fixed(price) => offer.price = price,
            PriceMode::Market { floor, .. } => {
                let before = available_before[i];
                if o.sold >= before - 1e-9 && unmet {
                    offer.price = offer.price.scale(1.0 + model.price_step_up);
                } else if before > 0.0 && o.sold < before / model.stock_days {
                    offer.price = offer.price.scale(1.0 - model.price_step_down);
                }
                offer.price = offer.price.max(floor).max(Money::from_units(1));
            }
        }
    }
    let market = state.markets.get_mut(product).get_mut(country);
    if day.sold > 1e-9 {
        let average = day.revenue.scale(1.0 / day.sold);
        let s = model.index_smoothing;
        market.price = market.price.scale(1.0 - s) + average.scale(s);
    }
    for (total, today) in market.bought.iter_mut().zip(bought) {
        *total += today;
    }
    market.record_day(day);
}

fn trade(
    state: &mut GameState,
    offer: &mut Offer,
    quantity: f64,
    buyer: Buyer,
    product: ProductId,
    day: &mut Trade,
) {
    let amount = Money::times(offer.price, quantity);
    offer.available -= quantity;
    offer.sold += quantity;
    day.sold += quantity;
    day.revenue += amount;
    if let Some(site) = offer.site {
        let s = &mut state.sites[site.index()];
        let value = s.inventory.entry(product).or_default().take(quantity);
        let ledger = &mut state.companies[s.owner.index()].ledger;
        let center = CostCenter::product(site, product);
        ledger.income(CostType::Revenue, center, Account::Cash, amount);
        ledger.expense(CostType::InventoryChange, center, Account::Inventory, value);
    }
    if let Buyer::Site(site) = buyer {
        let s = &mut state.sites[site.index()];
        s.inventory
            .entry(product)
            .or_default()
            .add(quantity, amount, offer.quality);
        if let Some(order) = s.orders.get_mut(&product) {
            order.bought_month += quantity;
        }
        state.companies[s.owner.index()]
            .ledger
            .transfer(Account::Inventory, Account::Cash, amount);
    }
}

/// Resets the monthly sales counters of offers and orders.
pub(crate) fn reset_site_months(state: &mut GameState) {
    for site in &mut state.sites {
        for offer in site.offers.values_mut() {
            offer.sold_month = 0.0;
        }
        for order in site.orders.values_mut() {
            order.bought_month = 0.0;
        }
    }
}
