//! Tenders of governments (MN2, docs/FORMELN.md): each quarter a government puts the
//! largest item of its demand per tendered goods group out as a lot; companies bid from
//! sites in the country, the best score wins a state order with daily deliveries, payment
//! by the state and a penalty on missing quantities.

use serde::{Deserialize, Serialize};

use crate::calendar::{Date, days_in_month};
use crate::catalog::{Catalog, SiteType};
use crate::command::{self, Command, CommandError};
use crate::ids::{CountryId, Id, ProductId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::market;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, GameState, SiteId};

const EPS: f64 = 1e-9;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bid {
    pub site: SiteId,
    pub company: CompanyId,
    /// Per unit, delivered from the site.
    pub price: Money,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tender {
    pub id: u32,
    pub country: CountryId,
    pub product: ProductId,
    pub per_day: f64,
    pub opened: Date,
    /// Bids are taken before this day; the lot is awarded at the first month start on or
    /// after it.
    pub deadline: Date,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bids: Vec<Bid>,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub winner: Option<CompanyId>,
}

impl Tender {
    pub fn open_on(&self, date: Date) -> bool {
        !self.closed && date < self.deadline
    }
}

/// A won lot: deliveries per day from a site to the government.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StateOrder {
    /// The tender's id.
    pub id: u32,
    pub site: SiteId,
    pub company: CompanyId,
    pub country: CountryId,
    pub product: ProductId,
    pub per_day: f64,
    pub price: Money,
    pub start: Date,
    /// First day without deliveries.
    pub end: Date,
    #[serde(default)]
    pub delivered_month: f64,
    #[serde(default)]
    pub delivered_total: f64,
    #[serde(default)]
    pub penalties: Money,
}

fn product_param(catalog: &Catalog, product: ProductId) -> Param {
    Param::TextKey(format!("produkt.{}", catalog.products.key(product)))
}

fn country_param(catalog: &Catalog, country: CountryId) -> Param {
    Param::Country(catalog.countries.key(country).to_owned())
}

/// A bid for an open lot (`BidTender`); a company's later bid replaces its earlier one.
pub(crate) fn bid(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    (tender, site, price): (u32, SiteId, Money),
) -> Result<(), CommandError> {
    let today = state.date;
    let Some(t) = state.tenders.iter().position(|t| t.id == tender) else {
        return Err(CommandError::TenderClosed);
    };
    if !state.tenders[t].open_on(today) {
        return Err(CommandError::TenderClosed);
    }
    let (country, product) = (state.tenders[t].country, state.tenders[t].product);
    let s = &state.sites[site.index()];
    if s.country != country || !s.offers.contains_key(&product) {
        return Err(CommandError::NoOffer(
            catalog.products.key(product).to_owned(),
        ));
    }
    if price <= Money::ZERO {
        return Err(CommandError::InvalidPrice);
    }
    let bids = &mut state.tenders[t].bids;
    bids.retain(|b| b.company != company);
    bids.push(Bid {
        site,
        company,
        price,
    });
    Ok(())
}

/// Whether a company has a working factory in a country.
fn has_factory(state: &GameState, company: CompanyId, country: CountryId) -> bool {
    state.sites.iter().any(|s| {
        s.owner == company
            && s.country == country
            && s.closed.is_none()
            && s.kind == SiteType::Factory
    })
}

/// Score of a bid (docs/FORMELN.md): price against the reference, quality of the site's
/// stock, home bonus (stronger at war) and reputation.
pub fn score(state: &GameState, catalog: &Catalog, tender: &Tender, bid: &Bid) -> f64 {
    let m = &catalog.market_model.tenders;
    let reference = market::local_reference(catalog, state, tender.country, tender.product);
    let relative = bid.price.to_usd() / reference.to_usd().max(EPS);
    let quality = state.sites[bid.site.index()]
        .inventory
        .get(&tender.product)
        .filter(|s| s.quantity > EPS)
        .map_or(50.0, |s| s.quality);
    let war = state.countries.get(tender.country).war_intensity;
    let home = if has_factory(state, bid.company, tender.country) {
        m.home_bonus * (1.0 + (m.war_factor - 1.0) * war)
    } else {
        0.0
    };
    let reputation = state.companies[bid.company.index()].reputation(tender.country);
    -m.price_weight * crate::math::ln(relative.max(EPS))
        + m.quality_weight * (quality - 50.0) / 25.0
        + home
        + m.reputation_weight * (reputation - 0.5)
}

/// Month start: penalties and ends of the orders, awards of lots past their deadline,
/// new lots at a quarter's start, the government's daily demand less the running orders.
/// Runs after the markets' month start.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, today: Date) -> Vec<Message> {
    let mut news = Vec::new();
    if !catalog.market_model.tenders.enabled {
        return news;
    }
    settle_orders(state, catalog, today, &mut news);
    award(state, catalog, today, &mut news);
    if today.month() % 3 == 1 {
        open(state, catalog, today, &mut news);
    }
    // Old lots stay a year for the views.
    let keep = today.add_days(-365);
    state.tenders.retain(|t| !t.closed || t.deadline >= keep);
    for o in &state.state_orders {
        let market = state.markets.get_mut(o.product).get_mut(o.country);
        market.state_rate = (market.state_rate - o.per_day).max(0.0);
    }
    news
}

fn settle_orders(state: &mut GameState, catalog: &Catalog, today: Date, news: &mut Vec<Message>) {
    let last = today.add_days(-1);
    let days = f64::from(days_in_month(last.year(), last.month()));
    let penalty = catalog.market_model.tenders.penalty;
    let player = state.main_company;
    for i in 0..state.state_orders.len() {
        let o = &state.state_orders[i];
        if o.start >= today {
            continue;
        }
        // Orders start on a month's first day.
        let due = o.per_day * days;
        let missing = (due - o.delivered_month).max(0.0);
        let charge = Money::times(o.price, missing).scale(penalty);
        let (company, site, product) = (o.company, o.site, o.product);
        if charge > Money::ZERO && !state.companies[company.index()].bankrupt {
            state.companies[company.index()].ledger.expense(
                CostType::Other,
                CostCenter::product(site, product),
                Account::Cash,
                charge,
            );
            state.state_orders[i].penalties += charge;
            if Some(company) == player {
                news.push(
                    Message::new(MessageKind::Warning, keys::STATE_ORDER_SHORT)
                        .with("produkt", product_param(catalog, product))
                        .with("menge", Param::Number(missing))
                        .with("kosten", Param::Money(charge)),
                );
            }
        }
        state.state_orders[i].delivered_month = 0.0;
    }
    state.state_orders.retain(|o| {
        o.end > today
            && state.sites[o.site.index()].owner == o.company
            && !state.companies[o.company.index()].bankrupt
    });
}

fn award(state: &mut GameState, catalog: &Catalog, today: Date, news: &mut Vec<Message>) {
    let months = catalog.market_model.tenders.months;
    let player = state.main_company;
    for i in 0..state.tenders.len() {
        let t = &state.tenders[i];
        if t.closed || t.deadline > today {
            continue;
        }
        let limit = market::state_price_limit(catalog, state, t.country, t.product);
        let mut best: Option<(f64, usize)> = None;
        for (j, b) in t.bids.iter().enumerate() {
            let valid = b.price <= limit
                && state.sites[b.site.index()].owner == b.company
                && state.sites[b.site.index()].closed.is_none()
                && !state.companies[b.company.index()].bankrupt;
            if !valid {
                continue;
            }
            let s = score(state, catalog, t, b);
            if best.is_none_or(|(top, _)| s > top) {
                best = Some((s, j));
            }
        }
        let (country, product, per_day, id) = (t.country, t.product, t.per_day, t.id);
        let bidders: Vec<CompanyId> = t.bids.iter().map(|b| b.company).collect();
        let winner = best.map(|(_, j)| t.bids[j].clone());
        state.tenders[i].closed = true;
        state.tenders[i].winner = winner.as_ref().map(|b| b.company);
        if let Some(b) = &winner {
            state.state_orders.push(StateOrder {
                id,
                site: b.site,
                company: b.company,
                country,
                product,
                per_day,
                price: b.price,
                start: today,
                end: today.add_months(months),
                delivered_month: 0.0,
                delivered_total: 0.0,
                penalties: Money::ZERO,
            });
        }
        let Some(player) = player else { continue };
        let won = winner.as_ref().is_some_and(|b| b.company == player);
        if won || bidders.contains(&player) {
            let key = if won {
                keys::TENDER_WON
            } else {
                keys::TENDER_LOST
            };
            let kind = if won {
                MessageKind::Success
            } else {
                MessageKind::Info
            };
            news.push(
                Message::new(kind, key)
                    .with("produkt", product_param(catalog, product))
                    .with("land", country_param(catalog, country))
                    .with("menge", Param::Number(per_day)),
            );
        }
    }
}

/// Lots of a quarter: per country and tendered group the product with the largest
/// government demand by value, unless a lot or an order for it runs.
fn open(state: &mut GameState, catalog: &Catalog, today: Date, news: &mut Vec<Message>) {
    let m = &catalog.market_model.tenders;
    let deadline = today.add_days(i32::try_from(m.deadline_days).unwrap_or(30));
    let player = state.main_company;
    let mut next_id = state
        .tenders
        .iter()
        .map(|t| t.id + 1)
        .max()
        .unwrap_or(0)
        .max(
            state
                .state_orders
                .iter()
                .map(|o| o.id + 1)
                .max()
                .unwrap_or(0),
        );
    let mut by_group: Vec<Vec<ProductId>> = vec![Vec::new(); m.groups.len()];
    for p in market_products(catalog) {
        let group = catalog.products.get(p).goods_group;
        if let Some(g) = m.groups.iter().position(|&x| x == group) {
            by_group[g].push(p);
        }
    }
    for c in 0..catalog.countries.len() {
        let country = CountryId::from_index(c);
        for products in &by_group {
            let busy = |p: ProductId| {
                state
                    .tenders
                    .iter()
                    .any(|t| !t.closed && t.country == country && t.product == p)
                    || state
                        .state_orders
                        .iter()
                        .any(|o| o.country == country && o.product == p)
            };
            let mut best: Option<(f64, ProductId)> = None;
            for &p in products {
                let rate = state.markets.get(p).get(country).state_rate;
                if rate <= EPS || busy(p) {
                    continue;
                }
                let value = rate * market::local_reference(catalog, state, country, p).to_usd();
                if best.is_none_or(|(v, _)| value > v) {
                    best = Some((value, p));
                }
            }
            let Some((_, product)) = best else { continue };
            let per_day = m.lot_share * state.markets.get(product).get(country).state_rate;
            if per_day <= EPS {
                continue;
            }
            state.tenders.push(Tender {
                id: next_id,
                country,
                product,
                per_day,
                opened: today,
                deadline,
                bids: Vec::new(),
                closed: false,
                winner: None,
            });
            next_id += 1;
            let offers_it = player.is_some_and(|pl| {
                state.sites.iter().any(|s| {
                    s.owner == pl && s.country == country && s.offers.contains_key(&product)
                })
            });
            if offers_it {
                news.push(
                    Message::new(MessageKind::Info, keys::TENDER_OPEN)
                        .with("produkt", product_param(catalog, product))
                        .with("land", country_param(catalog, country))
                        .with("menge", Param::Number(per_day)),
                );
            }
        }
    }
}

fn market_products(catalog: &Catalog) -> impl Iterator<Item = ProductId> + '_ {
    (0..catalog.products.len())
        .map(ProductId::from_index)
        .filter(|&p| catalog.products.get(p).state_demand.is_some())
}

/// Every day after the supply contracts: the orders' quantities from the sites' stock,
/// paid by the government, the missing part caught up on later days of the month.
pub(crate) fn deliver(state: &mut GameState, catalog: &Catalog, today: Date) {
    for i in 0..state.state_orders.len() {
        let o = &state.state_orders[i];
        if o.start > today || o.end <= today {
            continue;
        }
        let due = o.per_day * f64::from(today.day()) - o.delivered_month;
        if due <= EPS {
            continue;
        }
        let (site, company, product, price, country) =
            (o.site, o.company, o.product, o.price, o.country);
        let Some(stock) = state.sites[site.index()].inventory.get_mut(&product) else {
            continue;
        };
        let quantity = due.min(stock.quantity);
        if quantity <= EPS {
            continue;
        }
        let quality = stock.quality;
        let value = stock.take(quantity);
        let amount = Money::times(price, quantity);
        let center = CostCenter::product(site, product);
        let ledger = &mut state.companies[company.index()].ledger;
        ledger.income(CostType::Revenue, center, Account::Cash, amount);
        ledger.expense(CostType::InventoryChange, center, Account::Inventory, value);
        state.sites[site.index()]
            .totals
            .entry(product)
            .or_default()
            .sold += quantity;
        crate::reputation::record_sale(
            state,
            catalog,
            (company, country),
            quantity,
            quality,
            value,
        );
        let o = &mut state.state_orders[i];
        o.delivered_month += quantity;
        o.delivered_total += quantity;
    }
}

/// AI companies at a month start bid for open lots of products their sites in the
/// country offer: from the site with the most stock, at its asking price less a discount.
pub(crate) fn ai_bids(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    own: &[SiteId],
) {
    let m = &catalog.market_model.tenders;
    if !m.enabled {
        return;
    }
    let today = state.date;
    let open: Vec<(u32, CountryId, ProductId)> = state
        .tenders
        .iter()
        .filter(|t| t.open_on(today) && t.bids.iter().all(|b| b.company != company))
        .map(|t| (t.id, t.country, t.product))
        .collect();
    for (id, country, product) in open {
        let mut best: Option<(f64, SiteId)> = None;
        for &site in own {
            let s = &state.sites[site.index()];
            if s.country != country || s.closed.is_some() || !s.offers.contains_key(&product) {
                continue;
            }
            let stock = s.inventory.get(&product).map_or(0.0, |x| x.quantity);
            if best.is_none_or(|(q, _)| stock > q) {
                best = Some((stock, site));
            }
        }
        let Some((_, site)) = best else { continue };
        let asking = state.sites[site.index()].offers[&product].price;
        let base = if asking > Money::ZERO {
            asking
        } else {
            market::local_reference(catalog, state, country, product)
        };
        let price = base.scale(1.0 - m.ai_discount);
        let _ = command::execute(
            state,
            catalog,
            company,
            &Command::BidTender {
                tender: id,
                site,
                price,
            },
        );
    }
}
