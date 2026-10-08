//! Markets and prices (Lastenheft §9.1–9.2; formulas in docs/FORMELN.md, section M7).
//!
//! Each product has a market in each country. Sellers are sites with sale offers and
//! the state market; buyers are sites with purchase orders, the government and the
//! consumers in five income layers. Consumer and government demand are set at the start
//! of each month; markets with sellers or buyers are cleared every day.

use std::collections::{BTreeMap, BTreeSet};

use crate::calendar::Date;
use crate::catalog::{Catalog, ConsumerDemand, ConsumptionType, MarketModel};
use crate::ids::{CountryId, Id, ProductId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::math;
use crate::money::Money;
use crate::policy::{self, BuyerGroup};
use crate::state::{
    CompanyId, Consignee, GameState, Market, PriceMode, Shipment, SiteId, Stock, Trade,
};
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
    let price = match (
        market.idle_since,
        state_price(catalog, state, country, product),
    ) {
        // Only the state market sold since then: the index moved towards its price.
        (Some(from), Some(sp)) if from < state.date && market.price > Money::ZERO => {
            let keep = math::pow(
                1.0 - catalog.market_model.index_smoothing,
                f64::from(from.days_until(state.date)),
            );
            sp + (market.price - sp).scale(keep)
        }
        (Some(from), Some(sp)) if from < state.date => sp,
        _ => market.price,
    };
    if price > Money::ZERO {
        price
    } else {
        local_reference(catalog, state, country, product)
    }
}

/// Price of the state market for a product in a country, if it sells it this year.
pub fn state_price(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    product: ProductId,
) -> Option<Money> {
    let year = state.date.year();
    catalog
        .products
        .get(product)
        .state_market
        .filter(|m| {
            m.available_from.is_none_or(|y| y <= year)
                && m.available_until.is_none_or(|y| y >= year)
        })
        .map(|m| {
            m.price
                .scale(price_factor(catalog, state, country, product))
        })
}

/// What governments pay at most for a product: `state_price_cap` times the reference
/// price at the country's price level, but at least at world prices (price level 1).
/// Governments of poorer countries buy rails or sheet metal abroad at world prices; with
/// the local level alone imports would stay unsold for good (M16).
pub fn state_price_limit(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    product: ProductId,
) -> Money {
    let world = catalog.products.get(product).reference_price;
    local_reference(catalog, state, country, product)
        .max(world)
        .scale(catalog.market_model.state_price_cap)
}

/// Reference price in a country: the world reference at the share of the country's
/// price level that carries into the product's prices.
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
        .scale(price_factor(catalog, state, country, product))
}

/// Factor of a country's prices for a product against world prices (M16): the price
/// level to the power of the share that carries into prices of this kind of product.
pub fn price_factor(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    product: ProductId,
) -> f64 {
    state.countries.get(country).price_factors[catalog.products.get(product).kind.index()]
}

/// `price_factor` for a price level.
pub fn level_factor(catalog: &Catalog, price_level: f64, product: ProductId) -> f64 {
    let share = catalog.market_model.price_level_share[catalog.products.get(product).kind.index()];
    math::pow(price_level, share)
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
    // Markets of goods without consumer or government demand (wire, steel …) close
    // their month here; `update_demand` closes the others.
    for (product, p) in catalog.products.iter() {
        if p.consumer_demand.is_none() && p.state_demand.is_none() {
            for country in catalog.countries.ids() {
                state
                    .markets
                    .get_mut(product)
                    .get_mut(country)
                    .close_month();
            }
        }
    }
    update_demand(state, catalog, date, false);
}

/// Demand at the start of a game. Households already own durables: their ownership
/// starts at its equilibrium, so the first years show replacement and growth instead
/// of a catch-up of everything people owned in 1900. Two passes, so that complements
/// and displaced products see the ownership of the goods they depend on.
pub(crate) fn initial_demand(state: &mut GameState, catalog: &Catalog, date: Date) {
    update_demand(state, catalog, date, true);
    update_demand(state, catalog, date, true);
}

/// Whether households could own a product at the start: some recipe for it uses only
/// technologies known by then, or the state market sells it.
pub(crate) fn available_at_start(catalog: &Catalog, product: ProductId, year: i32) -> bool {
    let known = |t: Option<crate::ids::TechnologyId>| {
        t.is_none_or(|t| catalog.technologies.get(t).invention_year <= year)
    };
    let p = catalog.products.get(product);
    p.state_market
        .is_some_and(|m| m.available_from.is_none_or(|y| y <= year))
        || catalog.recipes.values().any(|r| {
            r.product == product
                && known(r.technology)
                && known(catalog.facilities.get(r.facility).technology)
        })
}

/// Whether a product can be bought at all (docs/FORMELN.md, M32): not while every recipe
/// that makes it (also as a by-product) needs a technology that is neither invented by
/// `year` nor known to a company, unless the state market sells it. Goods without any
/// recipe are always available.
pub fn available(state: &GameState, catalog: &Catalog, product: ProductId, year: i32) -> bool {
    let p = catalog.products.get(product);
    if p.state_market
        .is_some_and(|m| m.available_from.is_none_or(|y| y <= year))
    {
        return true;
    }
    let usable = |t: Option<crate::ids::TechnologyId>| {
        t.is_none_or(|t| {
            catalog.technologies.get(t).invention_year <= year
                || state
                    .companies
                    .iter()
                    .any(|c| !c.bankrupt && c.technologies.contains(&t))
        })
    };
    let mut recipes = catalog
        .recipes
        .values()
        .filter(|r| r.product == product || r.by_products.iter().any(|&(b, _)| b == product))
        .peekable();
    recipes.peek().is_none()
        || recipes
            .any(|r| usable(r.technology) && usable(catalog.facilities.get(r.facility).technology))
}

/// The year, as a fraction, from which a product can be made (M33): the earliest among
/// its recipes in which all technologies of recipe and facility were invented –
/// historically or earlier in the game. `None` if it has no recipe.
pub fn available_since(state: &GameState, catalog: &Catalog, product: ProductId) -> Option<f64> {
    let since = |t: Option<crate::ids::TechnologyId>| {
        t.map_or(f64::NEG_INFINITY, |t| {
            let historical = f64::from(catalog.technologies.get(t).invention_year);
            state
                .inventions
                .get(t)
                .map_or(historical, |d| d.year_fraction().min(historical))
        })
    };
    catalog
        .recipes
        .values()
        .filter(|r| r.product == product)
        .map(|r| since(r.technology).max(since(catalog.facilities.get(r.facility).technology)))
        .min_by(f64::total_cmp)
}

/// Share of a product's state demand left while successors with state demand take
/// over (M33): it falls evenly to zero over `verdraengung_staat_jahre` from the time each
/// successor could first be made.
pub(crate) fn state_demand_left(
    state: &GameState,
    catalog: &Catalog,
    product: ProductId,
    date: Date,
) -> f64 {
    let years = catalog.market_model.state_displacement_years;
    catalog
        .products
        .iter()
        .filter(|(_, q)| q.replaces.contains(&product) && q.state_demand.is_some())
        .filter_map(|(id, _)| available_since(state, catalog, id))
        .fold(1.0, |left, since| {
            left * (1.0 - (date.year_fraction() - since) / years).clamp(0.0, 1.0)
        })
}

/// Durables that displace a product (§6.4) with their highest ownership per head.
pub(crate) fn successors(catalog: &Catalog, product: ProductId) -> Vec<(ProductId, f64)> {
    catalog
        .products
        .iter()
        .filter(|(_, q)| q.replaces.contains(&product))
        .filter_map(|(id, q)| match q.consumer_demand.as_ref()?.consumption {
            ConsumptionType::Durable { max_ownership, .. } => Some((id, max_ownership)),
            ConsumptionType::Consumable { .. } | ConsumptionType::Complement { .. } => None,
        })
        .collect()
}

/// Share of the target ownership left per income fifth once the successors are owned.
pub(crate) fn displaced(
    state: &GameState,
    country: CountryId,
    successors: &[(ProductId, f64)],
) -> [f64; 5] {
    std::array::from_fn(|q| {
        successors.iter().fold(1.0, |f, &(s, max)| {
            f * (1.0 - (state.markets.get(s).get(country).ownership[q] / max).min(1.0))
        })
    })
}

fn update_demand(state: &mut GameState, catalog: &Catalog, date: Date, initial: bool) {
    let model = &catalog.market_model;
    let month = usize::try_from(date.month() - 1).expect("month 1-12");
    for (product, p) in catalog.products.iter() {
        if p.consumer_demand.is_none() && p.state_demand.is_none() {
            continue;
        }
        let owned_at_start = initial && available_at_start(catalog, product, date.year());
        // Nobody asks for what cannot be made yet.
        let open = available(state, catalog, product, date.year());
        let state_left = state_demand_left(state, catalog, product, date);
        let successors = successors(catalog, product);
        for country in catalog.countries.ids() {
            let cs = state.countries.get(country);
            let reference = p.reference_price.to_usd() * cs.price_level;
            let per_layer = cs.market_population / 5.0;
            let incomes = cs.income_quintiles_usd;
            let gdp = cs.market_population * cs.gdp_per_capita_usd * cs.price_level;
            let displaced = displaced(state, country, &successors);
            // Units of the durable a complement is used with, per inhabitant.
            let complement_owned: [f64; 5] = match p.consumer_demand.as_ref().map(|d| d.consumption)
            {
                Some(ConsumptionType::Complement { of, .. }) => {
                    state.markets.get(of).get(country).ownership
                }
                _ => [0.0; 5],
            };
            // A market without sales starts at the price the sellers ask there.
            let start_price = local_reference(catalog, state, country, product);
            let market = state.markets.get_mut(product).get_mut(country);
            market.close_month();
            // Markets rest until something happens (see `clear`).
            market.idle_since = Some(date);
            if market.price == Money::ZERO {
                market.price = start_price;
            }
            let price = market.price.to_usd();
            if !open {
                market.consumer_rate = [0.0; 5];
                market.bought = [0.0; 5];
                market.state_rate = 0.0;
                continue;
            }
            if let Some(d) = &p.consumer_demand {
                let season = d.seasonality.map_or(1.0, |s| s[month]);
                let grid = if d.needs_grid { cs.grid_share } else { 1.0 };
                for q in 0..5 {
                    let share = propensity(d, incomes[q], price, reference);
                    market.consumer_rate[q] = match d.consumption {
                        ConsumptionType::Consumable {
                            per_capita_per_year,
                        } => per_capita_per_year * share * grid * per_layer / 365.0 * season,
                        ConsumptionType::Durable {
                            service_life_years,
                            max_ownership,
                        } => {
                            let target = max_ownership * share * displaced[q] * grid;
                            if owned_at_start {
                                market.ownership[q] = target;
                            } else if !initial && per_layer > 0.0 {
                                let owned = &mut market.ownership[q];
                                *owned += market.bought[q] / per_layer
                                    - *owned / (service_life_years * 12.0);
                                *owned = owned.max(0.0);
                            }
                            let owned = market.ownership[q];
                            let gap = (target - owned).max(0.0);
                            // Owners above the target switch at their next purchase
                            // (Lastenheft §6.4, §9.1): only part of the worn-out units
                            // is replaced.
                            let replaced = if owned > target { target / owned } else { 1.0 };
                            (gap * model.adoption_per_year + owned / service_life_years * replaced)
                                * per_layer
                                / 365.0
                        }
                        ConsumptionType::Complement {
                            per_unit_per_year, ..
                        } => {
                            complement_owned[q] * per_layer * per_unit_per_year * share * grid
                                / 365.0
                                * season
                        }
                    };
                    market.bought[q] = 0.0;
                }
            }
            if let Some(s) = &p.state_demand {
                market.state_rate = s.per_million_gdp_at(date) * gdp / 1.0e6 / 365.0 * state_left;
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
    /// Share of sales in the country the seller pays for deliveries (M35).
    delivery: f64,
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

/// Clears all markets that have sellers or buyers today. All other markets with
/// consumer or government demand rest: nobody but perhaps the state market sells there,
/// and their days are booked together later (`settle_idle`).
pub(crate) fn clear(state: &mut GameState, catalog: &Catalog, date: Date) {
    // Sites that offer or order a product, by product and country, in site order. With
    // many companies, scanning all sites of a country for every product was most of a
    // day's work.
    let mut traders: Vec<Traders> = vec![BTreeMap::new(); catalog.products.len()];
    // Markets to clear today, by product.
    let mut active: BTreeMap<ProductId, BTreeSet<CountryId>> = BTreeMap::new();
    for (i, site) in state.sites.iter().enumerate() {
        let trades = !site.offers.is_empty() || !site.orders.is_empty();
        if trades && !state.companies[site.owner.index()].bankrupt {
            let id = SiteId(u32::try_from(i).expect("fits u32"));
            for &product in site.offers.keys().chain(site.orders.keys()) {
                let list = traders[product.index()].entry(site.country).or_default();
                if list.last() != Some(&id) {
                    list.push(id);
                }
                active.entry(product).or_default().insert(site.country);
            }
        }
    }
    for &(product, country) in &state.import_markets {
        active.entry(product).or_default().insert(country);
    }
    let in_transit = trade::to_importers(state);
    let model = &catalog.market_model;
    for (product, _) in catalog.products.iter() {
        let traders = &traders[product.index()];
        let plan = trade::plan(state, catalog, product, traders, &in_transit);
        // Demand abroad the traders could not buy for counts as scarcity too.
        let export_shortage = plan.unserved > 1e-9;
        let mut markets = active.remove(&product).unwrap_or_default();
        markets.extend(
            plan.buys
                .iter()
                .map(|b| state.sites[b.site.index()].country),
        );
        for country in markets {
            let exports: Vec<PlannedBuy> = plan
                .buys
                .iter()
                .filter(|b| state.sites[b.site.index()].country == country)
                .copied()
                .collect();
            let state_price = state_price(catalog, state, country, product);
            let limit = state_price_limit(catalog, state, country, product);
            settle_idle(
                state.markets.get_mut(product).get_mut(country),
                model,
                date,
                state_price,
                limit,
            );
            clear_market(
                state,
                catalog,
                (country, product, date),
                traders.get(&country).map_or(&[], Vec::as_slice),
                (state_price, plan.replacement.get(&country).copied()),
                (&exports, export_shortage),
            );
            // From tomorrow on the market rests again unless something happens.
            let m = state.markets.get_mut(product).get_mut(country);
            if m.state_rate > 0.0 || m.consumer_rate.iter().any(|&r| r > 0.0) {
                m.idle_since = Some(date.next_day());
            }
            if m.imports.quantity > 1e-9 {
                state.import_markets.insert((product, country));
            } else {
                state.import_markets.remove(&(product, country));
            }
        }
    }
}

/// Sites that offer or order one product, by country, in site order.
pub(crate) type Traders = BTreeMap<CountryId, Vec<SiteId>>;

/// Books the idle days of all markets before `until`, with the prices of the running
/// month (called before the country values change at the start of a month).
pub(crate) fn settle_all_idle(state: &mut GameState, catalog: &Catalog, until: Date) {
    for (product, _) in catalog.products.iter() {
        for country in catalog.countries.ids() {
            if state.markets.get(product).get(country).idle_since.is_none() {
                continue;
            }
            let state_price = state_price(catalog, state, country, product);
            let limit = state_price_limit(catalog, state, country, product);
            settle_idle(
                state.markets.get_mut(product).get_mut(country),
                &catalog.market_model,
                until,
                state_price,
                limit,
            );
        }
    }
}

/// Demand per day of consumers and government (constant within a month).
fn outside_rate(market: &Market) -> f64 {
    let mut demand = market.state_rate;
    for q in (0..5).rev() {
        demand += market.consumer_rate[q];
    }
    demand
}

/// Open demand after `days` idle days in which all outside demand stayed open.
fn open_after_idle(market: &Market, model: &MarketModel, days: i32) -> f64 {
    let demand = outside_rate(market);
    let keep = math::pow(1.0 - 1.0 / model.demand_smoothing_days, f64::from(days));
    let open = demand + (market.open_demand - demand) * keep;
    if open < 1e-9 { 0.0 } else { open }
}

/// Open demand of a market on `date`, including idle days not yet booked.
pub fn open_demand(market: &Market, model: &MarketModel, date: Date) -> f64 {
    match market.idle_since {
        Some(from) if from < date => open_after_idle(market, model, from.days_until(date)),
        _ => market.open_demand,
    }
}

/// Books the idle days before `until`. Nobody sold, or only the state market (at
/// `state_price`, without limit): consumers buy all they want from it, the government
/// up to its price cap. Everything outside demand stays open for traders.
fn settle_idle(
    market: &mut Market,
    model: &MarketModel,
    until: Date,
    state_price: Option<Money>,
    state_limit: Money,
) {
    let Some(from) = market.idle_since.take() else {
        return;
    };
    let days = from.days_until(until);
    if days <= 0 {
        return;
    }
    let n = f64::from(days);
    market.open_demand = open_after_idle(market, model, days);
    let demand = outside_rate(market);
    let mut day = Trade {
        demand,
        ..Trade::default()
    };
    if let Some(price) = state_price {
        let consumers: f64 = market.consumer_rate.iter().sum();
        let government = if price <= state_limit {
            market.state_rate
        } else {
            0.0
        };
        day.sold = consumers + government;
        day.revenue = Money::times(price, day.sold);
        for (bought, rate) in market.bought.iter_mut().zip(market.consumer_rate) {
            *bought += rate * n;
        }
        if day.sold > 1e-9 {
            let keep = math::pow(1.0 - model.index_smoothing, n);
            market.price = if market.price > Money::ZERO {
                price + (market.price - price).scale(keep)
            } else {
                price
            };
        }
    }
    day.outside_demand = day.demand;
    day.outside_sold = day.sold;
    market.month.demand += day.demand * n;
    market.month.sold += day.sold * n;
    market.month.outside_demand += day.outside_demand * n;
    market.month.outside_sold += day.outside_sold * n;
    market.month.revenue += Money::times(day.revenue, n);
    market.today = day;
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
    (state_market_price, replacement): (Option<Money>, Option<Money>),
    (exports, export_shortage): (&[PlannedBuy], bool),
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
        delivery: 0.0,
    };
    for &site in sites {
        let s = &state.sites[site.index()];
        if let Some(o) = s.offers.get(&product) {
            let stock = s.inventory.get(&product);
            offers.push(Offer {
                delivery: crate::plots::delivery_cost(catalog, state, s),
                ..offer(
                    Seller::Site {
                        site,
                        owner: s.owner,
                    },
                    o.price,
                    stock.map_or(0.0, |st| (st.quantity - o.keep).max(0.0)),
                    stock.map_or(50.0, |st| st.quality),
                )
            });
        }
    }
    let imports = &state.markets.get(product).get(country).imports;
    if imports.quantity > 1e-9 {
        let floor = import_floor(model, imports, replacement);
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
    // The highest price a buyer left without goods would have paid (see below).
    let mut unmet_limit: Option<Money> = None;
    let mut consumers_unmet = false;

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
        if need > 1e-9 {
            note_unmet(&mut unmet_limit, max_price);
        }
    }

    // 2. Government: cheapest offers up to a price cap.
    let state_need = state.markets.get(product).get(country).state_rate;
    if state_need > 0.0 {
        day.demand += state_need;
        flows.outside_demand += state_need;
        day.outside_demand += state_need;
        let cap = state_price_limit(catalog, state, country, product);
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
        if need > 1e-9 {
            note_unmet(&mut unmet_limit, cap);
        }
    }

    // 3. Consumers: richest layer first; sellers chosen by attractiveness (logit) of
    // price, quality and brand, weighted by their presence in the shops (M16).
    let rates = state.markets.get(product).get(country).consumer_rate;
    let presence: Vec<f64> = offers
        .iter()
        .map(|o| match o.seller {
            Seller::Site { site, .. } => {
                production_rate(state, catalog, site, product, date)
                    + o.available / model.stock_days
            }
            Seller::Importer => o.available / model.stock_days,
            Seller::StateMarket => rates.iter().sum(),
        })
        .collect();
    let group = catalog.products.get(product).goods_group;
    let brand: Vec<f64> = offers
        .iter()
        .map(|o| match o.seller {
            Seller::Site { owner, .. } => state.companies[owner.index()].awareness(country, group),
            Seller::Importer => model.brand.trade_awareness,
            Seller::StateMarket => model.brand.state_market_awareness,
        })
        .collect();
    let mut bought = [0.0; 5];
    let mut weights = vec![0.0; offers.len()];
    // Price, quality and brand do not change while the layers buy: the attraction of
    // an offer is computed once per layer, only what is left changes.
    let log_relative: Vec<f64> = offers
        .iter()
        .map(|o| math::ln((o.price.to_usd() / reference).max(1e-6)))
        .collect();
    let mut attraction = vec![0.0; offers.len()];
    for q in (0..5).rev() {
        let mut need = rates[q];
        day.demand += need;
        flows.outside_demand += need;
        day.outside_demand += need;
        if need > 1e-9 && reference > 0.0 {
            for (i, a) in attraction.iter_mut().enumerate() {
                *a = presence[i]
                    * math::exp(
                        -model.price_weight[q] * log_relative[i]
                            + model.quality_weight[q] * (offers[i].quality - 50.0) / 25.0
                            + model.brand.weight[q] * brand[i],
                    );
            }
        }
        for _ in 0..8 {
            if need <= 1e-9 {
                break;
            }
            for (i, w) in weights.iter_mut().enumerate() {
                *w = if offers[i].available <= 1e-12 || reference <= 0.0 {
                    0.0
                } else {
                    attraction[i]
                };
            }
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
        consumers_unmet |= need > 1e-9;
    }

    // 4. Traders: purchases planned for export (only from company offers), from what
    // the buyers of the country left (M16: before, exports came first and emptied the
    // markets of the exporting countries).
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

    // Prices of automatic sellers and traders, and the market's price index.
    let reference = local_reference(catalog, state, country, product);
    let max_price = reference.scale(model.price_max_factor);
    // Households and traders take any price up to the market's highest.
    if consumers_unmet || export_shortage {
        note_unmet(&mut unmet_limit, max_price);
    }
    let mut import_price = None;
    for (i, o) in offers.iter().enumerate() {
        let before = available_before[i];
        // Only a seller that had goods and sold out raises its price, and only while a
        // buyer left without goods would pay more. Demand that fails at the price (a
        // government above its cap, a plant above its bid) is no shortage: counted as
        // one, it drove the prices further out of reach (nails, flour 1900).
        let scarce = before > 1e-9
            && o.sold >= before - 1e-9
            && unmet_limit.is_some_and(|limit| limit > o.price);
        // Slow: stock for more than `stock_days`, or own facilities idling below the
        // normal utilization (they compete for customers instead of standing still).
        let idle = match o.seller {
            Seller::Site { site, .. } => utilization(state, catalog, site, product, date)
                .is_some_and(|u| u < model.normal_utilization),
            Seller::Importer | Seller::StateMarket => false,
        };
        let slow = before > 0.0 && (o.sold < before / model.stock_days || idle) && !scarce;
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
                        offer.price = adjust_price(model, offer.price, (scarce, slow), reference)
                            .min(max_price)
                            .max(floor)
                            .max(Money::from_units(1));
                    }
                }
            }
            // Traders compete: their price falls to cost plus margin unless goods are scarce.
            Seller::Importer => {
                import_price =
                    Some(adjust_price(model, o.price, (scarce, !scarce), reference).min(max_price));
            }
            Seller::StateMarket => {}
        }
    }
    let market = state.markets.get_mut(product).get_mut(country);
    if let Some(price) = import_price {
        market.import_price = price.max(import_floor(model, &market.imports, replacement));
    }
    if day.sold > 1e-9 {
        let average = day.revenue.scale(1.0 / day.sold);
        let s = model.index_smoothing;
        // The first sale sets the index; smoothed from zero it would stay far below the
        // prices for weeks, and bids based on it would never reach the sellers.
        market.price = if market.price > Money::ZERO {
            market.price.scale(1.0 - s) + average.scale(s)
        } else {
            average
        };
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

/// Planned share of the capacity of the facilities making a product (main product) at
/// a site, if it has any.
fn utilization(
    state: &GameState,
    catalog: &Catalog,
    site: SiteId,
    product: ProductId,
    date: Date,
) -> Option<f64> {
    let (planned, full) = state.sites[site.index()]
        .slots
        .iter()
        .filter(|sl| sl.operating(date))
        .filter_map(|sl| {
            let r = catalog.recipes.get(sl.recipe?);
            (r.product == product).then(|| {
                let full = sl.full_runs(catalog) * r.output;
                (full * sl.utilization, full)
            })
        })
        .fold((0.0, 0.0), |(p, f), (a, b)| (p + a, f + b));
    (full > 1e-9).then(|| planned / full)
}

/// Planned daily output of a product at a site: main and by-products of its finished
/// facilities at their planned utilization.
fn production_rate(
    state: &GameState,
    catalog: &Catalog,
    site: SiteId,
    product: ProductId,
    date: Date,
) -> f64 {
    state.sites[site.index()]
        .slots
        .iter()
        .filter(|sl| sl.operating(date))
        .filter_map(|sl| {
            let r = catalog.recipes.get(sl.recipe?);
            let per_run = if r.product == product {
                r.output
            } else {
                r.by_products.iter().find(|(p, _)| *p == product)?.1
            };
            let runs = sl.full_runs(catalog) * sl.utilization;
            Some(runs * per_run)
        })
        .sum()
}

/// Daily step of an automatic price: up when sold out with demand left, down when
/// little sells.
/// Raises the highest price an unserved buyer would have paid.
fn note_unmet(limit: &mut Option<Money>, price: Money) {
    *limit = Some(limit.map_or(price, |l| l.max(price)));
}

fn adjust_price(
    model: &MarketModel,
    price: Money,
    (scarce, slow): (bool, bool),
    reference: Money,
) -> Money {
    if scarce {
        // Far below its reference price a scarce good catches up faster (M22): after a
        // glut (petrol around 1910) the price would otherwise need years to recover.
        let gap = (reference.to_usd() / price.to_usd().max(1e-9)).clamp(1.0, model.catch_up_max);
        price.scale(1.0 + model.price_step_up * gap)
    } else if slow {
        price.scale(1.0 - model.price_step_down)
    } else {
        price
    }
}

/// Traders sell at their average landed cost plus margin at least, but not above what
/// fresh goods from abroad would cost (`replacement`): other traders would undercut
/// stock bought dearly.
fn import_floor(model: &MarketModel, imports: &Stock, replacement: Option<Money>) -> Money {
    if imports.quantity <= 1e-9 {
        return Money::ZERO;
    }
    let landed = imports.value.scale(1.0 / imports.quantity);
    replacement
        .map_or(landed, |r| landed.min(r))
        .scale(1.0 + model.trader_margin)
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
            // Deliveries from plots far from the customers (M35); traders collect.
            if offer.delivery > 0.0 && !matches!(buyer, Buyer::Trader { .. }) {
                let delivery = amount.scale(offer.delivery);
                ledger.expense(CostType::Transport, center, Account::Cash, delivery);
            }
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
                lost: false,
            });
        }
        Buyer::Outside => day.outside_sold += quantity,
    }
}

/// Resets the monthly sales counters of offers and orders.
pub(crate) fn reset_site_months(state: &mut GameState) {
    for site in &mut state.sites {
        for offer in site.offers.values_mut() {
            offer.sold_last_month = offer.sold_month;
            offer.sold_month = 0.0;
            offer.to_traders_month = 0.0;
            offer.to_companies_month = 0.0;
        }
        for order in site.orders.values_mut() {
            order.bought_last_month = order.bought_month;
            order.bought_month = 0.0;
        }
    }
}
