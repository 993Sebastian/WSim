//! Markets and prices (Lastenheft §9.1–9.2; formulas in docs/FORMELN.md, section M7).
//!
//! Each product has a market in each country. Sellers are sites with sale offers and
//! the state market; buyers are sites with purchase orders, the government and the
//! consumers in five income layers. Consumer and government demand are set at the start
//! of each month; markets with sellers or buyers are cleared every day.

use crate::calendar::Date;
use crate::catalog::{Catalog, ConsumerDemand, ConsumptionType, MarketModel};
use crate::ids::{CountryId, Id, ProductId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::math;
use crate::money::Money;
use crate::policy::{self, BuyerGroup};
use crate::state::{CompanyId, Consignee, GameState, PriceMode, Shipment, SiteId, Stock, Trade};
use crate::trade::{self, PlannedBuy};

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

/// Who sells on a market.
#[derive(Clone, Copy, PartialEq)]
enum Seller {
    Site {
        site: SiteId,
        owner: CompanyId,
    },
    StateMarket,
    /// Traders selling imported goods.
    Importer,
}

/// A seller on one market for one day.
struct Offer {
    seller: Seller,
    price: Money,
    available: f64,
    quality: f64,
    sold: f64,
    to_traders: f64,
    to_companies: f64,
}

#[derive(Clone, Copy)]
enum Buyer {
    Site(SiteId),
    /// Traders shipping the goods to another country.
    Trader {
        destination: CountryId,
        transport_per_unit: Money,
        days: u32,
    },
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
    let in_transit = trade::to_importers(state);
    for (product, p) in catalog.products.iter() {
        let state_market = p.state_market.filter(|m| {
            m.available_from.is_none_or(|y| y <= date.year())
                && m.available_until.is_none_or(|y| y >= date.year())
        });
        let plan = trade::plan(state, catalog, product, &sites_by_country, &in_transit);
        for country in catalog.countries.ids() {
            let sites = &sites_by_country[country.index()];
            let has_site_trade = sites.iter().any(|s| {
                let site = &state.sites[s.index()];
                site.offers.contains_key(&product) || site.orders.contains_key(&product)
            });
            let m = state.markets.get(product).get(country);
            let has_outside_demand = m.state_rate > 0.0 || m.consumer_rate.iter().any(|&r| r > 0.0);
            let has_imports = m.imports.quantity > 1e-9 || m.open_demand > 1e-9;
            if !has_site_trade && !has_outside_demand && !has_imports {
                continue;
            }
            let state_price =
                state_market.map(|m| m.price.scale(state.countries.get(country).price_level));
            let exports: Vec<PlannedBuy> = plan
                .iter()
                .filter(|b| state.sites[b.site.index()].country == country)
                .copied()
                .collect();
            clear_market(
                state,
                catalog,
                (country, product, date),
                sites,
                state_price,
                &exports,
            );
        }
    }
}

/// Quantities of one market day that decide the open demand for traders.
#[derive(Default)]
struct Flows {
    /// Consumer and government demand.
    outside_demand: f64,
    /// Of that, served by companies in the country.
    outside_from_sites: f64,
    /// Industry purchases from traders' imports.
    industry_from_imports: f64,
    /// Industry demand nobody served.
    industry_unmet: f64,
}

fn clear_market(
    state: &mut GameState,
    catalog: &Catalog,
    (country, product, date): (CountryId, ProductId, Date),
    sites: &[SiteId],
    state_market_price: Option<Money>,
    exports: &[PlannedBuy],
) {
    let model = &catalog.market_model;
    let reference = local_reference(catalog, state, country, product).to_usd();
    let mut offers: Vec<Offer> = Vec::new();
    let offer = |seller, price, available, quality| Offer {
        seller,
        price,
        available,
        quality,
        sold: 0.0,
        to_traders: 0.0,
        to_companies: 0.0,
    };
    for &site in sites {
        let s = &state.sites[site.index()];
        if let Some(o) = s.offers.get(&product) {
            let stock = s.inventory.get(&product);
            offers.push(offer(
                Seller::Site {
                    site,
                    owner: s.owner,
                },
                o.price,
                stock.map_or(0.0, |st| (st.quantity - o.keep).max(0.0)),
                stock.map_or(50.0, |st| st.quality),
            ));
        }
    }
    let imports = &state.markets.get(product).get(country).imports;
    if imports.quantity > 1e-9 {
        let floor = import_floor(model, imports);
        let price = state
            .markets
            .get(product)
            .get(country)
            .import_price
            .max(floor);
        offers.push(offer(
            Seller::Importer,
            price,
            imports.quantity,
            imports.quality,
        ));
    }
    if let Some(price) = state_market_price {
        offers.push(offer(Seller::StateMarket, price, f64::INFINITY, 50.0));
    }
    let available_before: Vec<f64> = offers.iter().map(|o| o.available).collect();
    let mut by_price: Vec<usize> = (0..offers.len()).collect();
    by_price.sort_by(|&a, &b| offers[a].price.cmp(&offers[b].price).then(a.cmp(&b)));
    let mut day = Trade::default();
    let mut flows = Flows::default();

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
            if o.price > max_price || o.quality < min_quality || o.available <= 0.0 {
                continue;
            }
            let allowance = match o.seller {
                Seller::Site { owner: seller, .. } if seller == owner => continue,
                Seller::Site {
                    site: s,
                    owner: seller,
                } => {
                    let (rule, _) = policy::sales_rule(
                        &state.companies[seller.index()],
                        BuyerGroup::Companies,
                        product,
                        country,
                    );
                    let month = state.sites[s.index()].offers[&product].to_companies_month;
                    rule.allowance(o.price, month + o.to_companies)
                }
                Seller::StateMarket | Seller::Importer => f64::INFINITY,
            };
            let quantity = need.min(o.available).min(allowance);
            if quantity <= 1e-9 {
                continue;
            }
            if o.seller == Seller::Importer {
                flows.industry_from_imports += quantity;
            }
            trade(
                state,
                &mut offers[i],
                quantity,
                Buyer::Site(site),
                (product, country, date),
                &mut day,
            );
            need -= quantity;
        }
        flows.industry_unmet += need.max(0.0);
    }

    // 2. Traders: purchases planned for export (only from company offers).
    for b in exports {
        let Some(i) = offers
            .iter()
            .position(|o| matches!(o.seller, Seller::Site { site, .. } if site == b.site))
        else {
            continue;
        };
        let quantity = b.quantity.min(offers[i].available);
        if quantity <= 1e-9 {
            continue;
        }
        trade(
            state,
            &mut offers[i],
            quantity,
            Buyer::Trader {
                destination: b.destination,
                transport_per_unit: b.transport_per_unit,
                days: b.days,
            },
            (product, country, date),
            &mut day,
        );
    }

    // 3. Government: cheapest offers up to a price cap.
    let state_need = state.markets.get(product).get(country).state_rate;
    if state_need > 0.0 {
        day.demand += state_need;
        flows.outside_demand += state_need;
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
            if matches!(offers[i].seller, Seller::Site { .. }) {
                flows.outside_from_sites += quantity;
            }
            trade(
                state,
                &mut offers[i],
                quantity,
                Buyer::Outside,
                (product, country, date),
                &mut day,
            );
            need -= quantity;
        }
    }

    // 4. Consumers: richest layer first; sellers chosen by attractiveness (logit).
    let rates = state.markets.get(product).get(country).consumer_rate;
    let mut bought = [0.0; 5];
    for q in (0..5).rev() {
        let mut need = rates[q];
        day.demand += need;
        flows.outside_demand += need;
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
                    if matches!(offers[i].seller, Seller::Site { .. }) {
                        flows.outside_from_sites += quantity;
                    }
                    trade(
                        state,
                        &mut offers[i],
                        quantity,
                        Buyer::Outside,
                        (product, country, date),
                        &mut day,
                    );
                    taken += quantity;
                }
            }
            need -= taken;
            bought[q] += taken;
        }
    }

    // Prices of automatic sellers and traders, and the market's price index.
    let unmet = day.unmet() > 1e-9;
    let mut import_price = None;
    for (i, o) in offers.iter().enumerate() {
        let before = available_before[i];
        let scarce = o.sold >= before - 1e-9 && unmet;
        let slow = before > 0.0 && o.sold < before / model.stock_days;
        match o.seller {
            Seller::Site { site, .. } => {
                let Some(offer) = state.sites[site.index()].offers.get_mut(&product) else {
                    continue;
                };
                offer.sold_today = o.sold;
                offer.sold_month += o.sold;
                offer.to_traders_month += o.to_traders;
                offer.to_companies_month += o.to_companies;
                match offer.mode {
                    PriceMode::Fixed(price) => offer.price = price,
                    PriceMode::Market { floor, .. } => {
                        offer.price = adjust_price(model, offer.price, scarce, slow)
                            .max(floor)
                            .max(Money::from_units(1));
                    }
                }
            }
            // Traders compete: their price falls to cost plus margin unless goods are scarce.
            Seller::Importer => {
                import_price = Some(adjust_price(model, o.price, scarce, !scarce));
            }
            Seller::StateMarket => {}
        }
    }
    let market = state.markets.get_mut(product).get_mut(country);
    if let Some(price) = import_price {
        market.import_price = price.max(import_floor(model, &market.imports));
    }
    if day.sold > 1e-9 {
        let average = day.revenue.scale(1.0 / day.sold);
        let s = model.index_smoothing;
        market.price = market.price.scale(1.0 - s) + average.scale(s);
    }
    for (total, today) in market.bought.iter_mut().zip(bought) {
        *total += today;
    }
    let open = (flows.outside_demand - flows.outside_from_sites)
        + flows.industry_from_imports
        + flows.industry_unmet / model.trader_cover_days;
    market.open_demand += (open.max(0.0) - market.open_demand) / model.demand_smoothing_days;
    if market.open_demand < 1e-9 {
        market.open_demand = 0.0;
    }
    market.record_day(day);
}

/// Daily step of an automatic price: up when sold out with demand left, down when
/// little sells.
fn adjust_price(model: &MarketModel, price: Money, scarce: bool, slow: bool) -> Money {
    if scarce {
        price.scale(1.0 + model.price_step_up)
    } else if slow {
        price.scale(1.0 - model.price_step_down)
    } else {
        price
    }
}

/// Traders never sell below their average landed cost plus margin.
fn import_floor(model: &MarketModel, imports: &Stock) -> Money {
    if imports.quantity <= 1e-9 {
        return Money::ZERO;
    }
    imports
        .value
        .scale((1.0 + model.trader_margin) / imports.quantity)
}

fn trade(
    state: &mut GameState,
    offer: &mut Offer,
    quantity: f64,
    buyer: Buyer,
    (product, country, date): (ProductId, CountryId, Date),
    day: &mut Trade,
) {
    let amount = Money::times(offer.price, quantity);
    offer.available -= quantity;
    offer.sold += quantity;
    day.sold += quantity;
    day.revenue += amount;
    match offer.seller {
        Seller::Site { site, owner } => {
            let s = &mut state.sites[site.index()];
            let value = s.inventory.entry(product).or_default().take(quantity);
            let ledger = &mut state.companies[owner.index()].ledger;
            let center = CostCenter::product(site, product);
            ledger.income(CostType::Revenue, center, Account::Cash, amount);
            ledger.expense(CostType::InventoryChange, center, Account::Inventory, value);
        }
        Seller::Importer => {
            state
                .markets
                .get_mut(product)
                .get_mut(country)
                .imports
                .take(quantity);
            day.imported += quantity;
        }
        Seller::StateMarket => {}
    }
    match buyer {
        Buyer::Site(site) => {
            let s = &mut state.sites[site.index()];
            s.inventory
                .entry(product)
                .or_default()
                .add(quantity, amount, offer.quality);
            if let Some(order) = s.orders.get_mut(&product) {
                order.bought_month += quantity;
            }
            let owner = s.owner;
            state.companies[owner.index()].ledger.transfer(
                Account::Inventory,
                Account::Cash,
                amount,
            );
            if matches!(offer.seller, Seller::Site { owner: seller, .. } if seller != owner) {
                offer.to_companies += quantity;
            }
        }
        Buyer::Trader {
            destination,
            transport_per_unit,
            days,
        } => {
            offer.to_traders += quantity;
            day.exported += quantity;
            state.shipments.push(Shipment {
                product,
                quantity,
                quality: offer.quality,
                value: amount + Money::times(transport_per_unit, quantity),
                from: country,
                to: Consignee::Importer(destination),
                arrival: date.add_days(i32::try_from(days).expect("routes take far fewer days")),
            });
        }
        Buyer::Outside => {}
    }
}

/// Resets the monthly sales counters of offers and orders.
pub(crate) fn reset_site_months(state: &mut GameState) {
    for site in &mut state.sites {
        for offer in site.offers.values_mut() {
            offer.sold_month = 0.0;
            offer.to_traders_month = 0.0;
            offer.to_companies_month = 0.0;
        }
        for order in site.orders.values_mut() {
            order.bought_month = 0.0;
        }
    }
}
