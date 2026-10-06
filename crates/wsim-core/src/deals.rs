//! Offers between companies for sites and licences (M30; Lastenheft §18.4; formulas in
//! docs/FORMELN.md, M30). Player and AI companies use the same commands and checks: a
//! buyer offers a price, the seller accepts, declines or names a higher price once, and
//! the buyer answers that. An accepted offer hands over the site with everything on it,
//! or gives the buyer the technology while the seller keeps it.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, SiteType};
use crate::command::{self, Command, CommandError, site_type_key};
use crate::ids::{CountryId, GoodsGroupId, Id, ProductId, TechnologyId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::math;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, Consignee, GameState, Goodwill, SiteId};

/// What an offer is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DealObject {
    /// A site with everything on it.
    Site(SiteId),
    /// A licence: the buyer gets the technology, the seller keeps it.
    License(TechnologyId),
    /// All the seller's sites of a goods group with its brand (M31).
    Area(GoodsGroupId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfferStatus {
    Open,
    Accepted,
    Declined,
    Expired,
    Withdrawn,
}

/// How the company whose turn it is answers.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum OfferAnswer {
    Accept,
    Decline,
    /// The seller names a higher price (once); then the buyer answers.
    Counter {
        price: Money,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Offer {
    pub id: u32,
    pub buyer: CompanyId,
    pub seller: CompanyId,
    pub object: DealObject,
    pub price: Money,
    /// The seller named the price (counter-offer): the buyer answers.
    pub counter: bool,
    /// Day the current price was named.
    pub date: Date,
    pub status: OfferStatus,
    /// Day the offer was closed.
    pub closed: Option<Date>,
}

impl Offer {
    /// The company whose answer is awaited.
    pub fn answering(&self) -> CompanyId {
        if self.counter {
            self.buyer
        } else {
            self.seller
        }
    }

    /// The company that named the current price.
    pub fn bidding(&self) -> CompanyId {
        if self.counter {
            self.seller
        } else {
            self.buyer
        }
    }

    /// Last day for an answer.
    pub fn deadline(&self, catalog: &Catalog) -> Date {
        self.date.add_months(catalog.deal_model.valid_months)
    }
}

/// What a site is worth (docs/FORMELN.md, M30).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SiteValue {
    /// Book value on the fixed assets: finished facilities, building, development.
    pub fixed_assets: Money,
    /// Facilities and development under construction, at cost (U).
    pub under_construction: Money,
    /// What is left of the goodwill of an earlier purchase.
    pub goodwill: Money,
    /// Stocks at the site and on the way there (L).
    pub inventory: Money,
    /// Result of the last twelve closed months (R, extrapolated); `None` for sites with
    /// too few months.
    pub result_year: Option<Money>,
    /// E = max(0, R) × earnings years.
    pub earnings_value: Money,
    /// Q: proceeds of selling all finished facilities (M22).
    pub liquidation: Money,
    /// Bought plot at its purchase price (M35).
    pub land_book: Money,
    /// Bought plot at today's value (M35); leased plots count nothing.
    pub land: Money,
    /// G = max(E, Q) + U + L + land.
    pub base: Money,
}

impl SiteValue {
    /// B: everything on the books except the stocks.
    pub fn book(&self) -> Money {
        self.fixed_assets + self.under_construction + self.goodwill + self.land_book
    }
}

/// Full months from `from` to `to`.
fn months_between(from: Date, to: Date) -> u32 {
    let months = (to.year() - from.year()) * 12 + i32::try_from(to.month()).unwrap_or(0)
        - i32::try_from(from.month()).unwrap_or(0)
        - i32::from(to.day() < from.day());
    u32::try_from(months.max(0)).unwrap_or(0)
}

pub fn site_value(state: &GameState, catalog: &Catalog, site: SiteId) -> SiteValue {
    let date = state.date;
    let model = &catalog.deal_model;
    let pm = &catalog.production_model;
    let s = &state.sites[site.index()];
    let mut v = SiteValue::default();
    for sl in &s.slots {
        let f = catalog.facilities.get(sl.facility);
        if sl.ready > date {
            // Completion moves the catalog investment off the construction account.
            v.under_construction += sl.investment(catalog);
        } else {
            v.fixed_assets += sl.book_value(f.lifetime_years, sl.count, date);
            v.liquidation += crate::production::sale_value(catalog, sl, sl.count, date).1;
        }
    }
    let remaining = |cost: Money, from: Date, years: f64| {
        let days = years * 365.0;
        let age = f64::from(from.days_until(date).max(0));
        cost.scale((1.0 - age / days).max(0.0))
    };
    v.fixed_assets += remaining(s.building_cost, s.founded, pm.building_lifetime_years);
    if let Some(field) = s
        .deposit
        .and_then(|d| state.deposits.get(d).concession_of(site))
    {
        match field.ready {
            Some(ready) if ready <= date => {
                v.fixed_assets +=
                    remaining(field.development_cost, ready, pm.development_lifetime_years);
            }
            _ => v.under_construction += field.development_cost,
        }
    }
    v.goodwill = s
        .goodwill
        .map_or(Money::ZERO, |g| g.book_value(model.goodwill_years, date));
    v.inventory = s.inventory.values().map(|stock| stock.value).sum::<Money>()
        + state
            .shipments
            .iter()
            .filter(|sh| sh.to == Consignee::Site(site))
            .map(|sh| sh.value)
            .sum::<Money>();
    let since = s.acquired.unwrap_or(s.founded).first_of_month();
    let ledger = &state.companies[s.owner.index()].ledger;
    let months: Vec<Money> = ledger
        .months
        .iter()
        .rev()
        .take(12)
        .filter(|m| m.start.is_some_and(|start| start >= since))
        .map(|m| m.by_site.get(&site).copied().unwrap_or(Money::ZERO))
        .collect();
    let n = months.len();
    if n >= usize::try_from(model.earnings_min_months).unwrap_or(usize::MAX) && n > 0 {
        let total: Money = months.iter().copied().sum();
        // At most twelve months: the cast is exact.
        v.result_year = Some(total.scale(12.0 / n as f64));
    }
    v.earnings_value = v.result_year.map_or(Money::ZERO, |r| {
        r.max(Money::ZERO).scale(model.earnings_years)
    });
    if let Some(plot) = s.plot
        && let crate::state::Tenure::Owned(price) = state.plots[plot.index()].tenure
    {
        v.land_book = price;
        v.land = crate::plots::value(catalog, state, plot);
    }
    v.base = v.earnings_value.max(v.liquidation) + v.under_construction + v.inventory + v.land;
    v
}

/// Awareness counted for the brand value: full awareness would cost endless advertising.
const AWARENESS_VALUED_MAX: f64 = 0.99;

/// Whether a site can belong to an area: power plants and laboratories serve the whole
/// company.
pub fn area_kind(kind: SiteType) -> bool {
    !matches!(kind, SiteType::PowerPlant | SiteType::ResearchCenter)
}

/// The goods groups a site makes or offers products of.
pub fn site_groups(state: &GameState, catalog: &Catalog, site: SiteId) -> BTreeSet<GoodsGroupId> {
    site_products(state, catalog, site)
        .into_iter()
        .map(|p| catalog.products.get(p).goods_group)
        .collect()
}

/// The sites of a company's area for a goods group (docs/FORMELN.md, M31).
pub fn area_sites(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    group: GoodsGroupId,
) -> Vec<SiteId> {
    (0..state.sites.len())
        .map(|i| SiteId(u32::try_from(i).unwrap_or(u32::MAX)))
        .filter(|&site| {
            let s = &state.sites[site.index()];
            s.owner == company
                && area_kind(s.kind)
                && site_groups(state, catalog, site).contains(&group)
        })
        .collect()
}

/// W: the advertising that would build the company's awareness for a goods group in
/// every country (docs/FORMELN.md, M31).
pub fn brand_value(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    group: GoodsGroupId,
) -> Money {
    let effect = catalog
        .market_model
        .brand
        .medium(state.date.year())
        .map_or(0.0, |m| m.effect);
    if effect <= 0.0 {
        return Money::ZERO;
    }
    let usd: f64 = state.companies[company.index()]
        .brands
        .iter()
        .filter(|b| b.group == group)
        .map(|b| {
            let months = -math::ln(1.0 - b.awareness.clamp(0.0, AWARENESS_VALUED_MAX)) / effect;
            crate::brand::reach_usd(state, catalog, b.country) * months
        })
        .sum();
    Money::from_usd(usd).unwrap_or(Money::ZERO)
}

/// What an area is worth (docs/FORMELN.md, M31).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AreaValue {
    pub sites: Vec<(SiteId, SiteValue)>,
    /// W: the brand.
    pub brand: Money,
    /// G_B = sum of the sites' base values + W.
    pub base: Money,
    /// N_B = sum of the sites' new-build costs + W.
    pub new_build: Money,
}

impl AreaValue {
    /// The sum of the sites' parts, with the brand for the views.
    pub fn sum(&self) -> SiteValue {
        let mut v = SiteValue::default();
        for (_, s) in &self.sites {
            v.fixed_assets += s.fixed_assets;
            v.under_construction += s.under_construction;
            v.goodwill += s.goodwill;
            v.inventory += s.inventory;
            v.earnings_value += s.earnings_value;
            v.liquidation += s.liquidation;
            v.land_book += s.land_book;
            v.land += s.land;
            if let Some(r) = s.result_year {
                v.result_year = Some(v.result_year.unwrap_or(Money::ZERO) + r);
            }
        }
        v.base = self.base;
        v
    }
}

pub fn area_value_of(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    group: GoodsGroupId,
    sites: &[SiteId],
) -> AreaValue {
    let brand = brand_value(state, catalog, company, group);
    let sites: Vec<(SiteId, SiteValue)> = sites
        .iter()
        .map(|&s| (s, site_value(state, catalog, s)))
        .collect();
    let base = sites.iter().map(|(_, v)| v.base).sum::<Money>() + brand;
    let new_build = sites
        .iter()
        .map(|&(s, _)| new_site_cost(state, catalog, s))
        .sum::<Money>()
        + brand;
    AreaValue {
        sites,
        brand,
        base,
        new_build,
    }
}

pub fn area_value(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    group: GoodsGroupId,
) -> AreaValue {
    let sites = area_sites(state, catalog, company, group);
    area_value_of(state, catalog, company, group, &sites)
}

/// Whether the sites hold all the company's works and extraction sites with facilities
/// (docs/FORMELN.md, M31).
fn whole_production(state: &GameState, company: CompanyId, sites: &[SiteId]) -> bool {
    state.sites.iter().enumerate().all(|(i, s)| {
        s.owner != company
            || !matches!(s.kind, SiteType::Extraction | SiteType::Factory)
            || s.slots.is_empty()
            || sites.iter().any(|x| x.index() == i)
    })
}

/// Shares above the base value that a site is worth to a buyer (docs/FORMELN.md, M30).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Advantages {
    /// The seller's sales in markets where the buyer sells too.
    pub competition: f64,
    /// Qualified workers where they are scarce.
    pub staff: f64,
    /// The buyer works in this business: buying saves construction time.
    pub own_business: f64,
    /// Part of what a new site of the same kind would cost more.
    pub new_build: f64,
}

impl Advantages {
    pub fn total(&self) -> f64 {
        self.competition + self.staff + self.own_business + self.new_build
    }
}

/// N: today's investment in the site's facilities and the cost of the site.
pub fn new_site_cost(state: &GameState, catalog: &Catalog, site: SiteId) -> Money {
    let s = &state.sites[site.index()];
    s.slots
        .iter()
        .map(|sl| sl.investment(catalog))
        .sum::<Money>()
        + catalog.production_model.site_cost(s.kind)
}

/// S and W (docs/FORMELN.md, M30): the electricity a company's facilities in a country
/// use per day and what its power plants there can make, leaving out `except`.
fn electricity_balance(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    country: CountryId,
    except: Option<SiteId>,
) -> (f64, f64) {
    let Some(power) = catalog.production_model.electricity else {
        return (0.0, 0.0);
    };
    let (mut need, mut capacity) = (0.0, 0.0);
    for (i, s) in state.sites.iter().enumerate() {
        if s.owner != company || s.country != country || except.is_some_and(|e| e.index() == i) {
            continue;
        }
        for sl in s.slots.iter().filter(|sl| !sl.mothballed()) {
            let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r)) else {
                continue;
            };
            let runs = sl.full_runs(catalog);
            need += r.energy_mwh * runs * sl.utilization;
            if r.product == power {
                capacity += runs * r.output;
            }
        }
    }
    (need, capacity)
}

/// Whether a company has a laboratory other than `except`.
fn has_laboratory(state: &GameState, company: CompanyId, except: Option<SiteId>) -> bool {
    state.sites.iter().enumerate().any(|(i, s)| {
        s.owner == company
            && s.kind == SiteType::ResearchCenter
            && except.is_none_or(|e| e.index() != i)
    })
}

/// Whether the owner would have to build the site anew without it: a power plant whose
/// electricity its facilities in the country need, or its only laboratory while it
/// researches or (an AI company) wants to (docs/FORMELN.md, M30).
pub fn needed_by_owner(state: &GameState, catalog: &Catalog, site: SiteId) -> bool {
    let s = &state.sites[site.index()];
    match s.kind {
        SiteType::PowerPlant => {
            let (need, capacity) =
                electricity_balance(state, catalog, s.owner, s.country, Some(site));
            need > capacity
        }
        SiteType::ResearchCenter => {
            let wants = state.companies[s.owner.index()].ai.is_some()
                && crate::ai::wants_research(state, catalog, s.owner);
            (s.research.is_some() || wants) && !has_laboratory(state, s.owner, Some(site))
        }
        _ => false,
    }
}

/// Whether the buyer works in the site's business: it makes one of the site's products,
/// or it would build such a site itself – a laboratory if it wants to research and has
/// none, a power plant where it lacks electricity (docs/FORMELN.md, M30).
fn in_business(
    state: &GameState,
    catalog: &Catalog,
    buyer: CompanyId,
    site: SiteId,
    made: &BTreeSet<ProductId>,
) -> bool {
    let s = &state.sites[site.index()];
    match s.kind {
        SiteType::ResearchCenter => {
            crate::ai::wants_research(state, catalog, buyer) && !has_laboratory(state, buyer, None)
        }
        // Electricity cannot be traded: only power the buyer lacks there counts.
        SiteType::PowerPlant => {
            let (need, capacity) = electricity_balance(state, catalog, buyer, s.country, None);
            need > capacity
        }
        _ => site_products(state, catalog, site)
            .iter()
            .any(|p| made.contains(p)),
    }
}

/// Products a site makes (by its recipes) or offers.
fn site_products(state: &GameState, catalog: &Catalog, site: SiteId) -> BTreeSet<ProductId> {
    let s = &state.sites[site.index()];
    s.slots
        .iter()
        .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r).product))
        .chain(s.offers.keys().copied())
        .collect()
}

/// Products a company makes anywhere and what it offers in which country.
fn business(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
) -> (BTreeSet<ProductId>, BTreeSet<(ProductId, CountryId)>) {
    let mut made = BTreeSet::new();
    let mut offered = BTreeSet::new();
    for s in state.sites.iter().filter(|s| s.owner == company) {
        made.extend(
            s.slots
                .iter()
                .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r).product)),
        );
        offered.extend(s.offers.keys().map(|&p| (p, s.country)));
    }
    (made, offered)
}

pub fn advantages(
    state: &GameState,
    catalog: &Catalog,
    buyer: CompanyId,
    site: SiteId,
    value: &SiteValue,
) -> Advantages {
    let business = business(state, catalog, buyer);
    let new_build = new_site_cost(state, catalog, site);
    advantages_for(
        state,
        catalog,
        buyer,
        &business,
        &[site],
        value.base,
        new_build,
    )
}

/// The shares of `advantages` for several sites bought together (an area, M31) worth
/// `base`, which would cost `new_build` anew; `business` as from `business`.
fn advantages_for(
    state: &GameState,
    catalog: &Catalog,
    buyer: CompanyId,
    business: &(BTreeSet<ProductId>, BTreeSet<(ProductId, CountryId)>),
    sites: &[SiteId],
    base: Money,
    new_build: Money,
) -> Advantages {
    let model = &catalog.deal_model;
    let ai = &model.ai;
    let (_, aggressiveness) = crate::ai::traits(state, buyer);
    let (made, offered) = business;

    let mut share = 0.0;
    let (mut workers, mut qualified, mut scarce) = (0.0, 0.0, 0.0);
    for &site in sites {
        let s = &state.sites[site.index()];
        for (&p, offer) in &s.offers {
            if offer.sold_last_month <= 0.0 || !offered.contains(&(p, s.country)) {
                continue;
            }
            let sold = state.markets.get(p).get(s.country).last_month.sold;
            if sold > 0.0 {
                share += offer.sold_last_month / sold;
            }
        }
        let country = state.countries.get(s.country);
        for (g, &w) in s.workforce.iter() {
            workers += w;
            let rank = catalog
                .qualifications
                .get(catalog.labor_groups.get(g).qualification)
                .rank;
            if rank < model.qualified_rank || w <= 0.0 {
                continue;
            }
            qualified += w;
            let pool = country.labor_pool.get(g.index()).copied().unwrap_or(0.0);
            let free = country
                .labor_available
                .get(g.index())
                .copied()
                .unwrap_or(0.0);
            let scarcity = if pool > 0.0 {
                (1.0 - free / pool).clamp(0.0, 1.0)
            } else {
                1.0
            };
            scarce += w * scarcity;
        }
    }
    let competition = ai.competition_markup.at(aggressiveness) * share.min(1.0);
    let staff = if qualified > 0.0 {
        ai.staff_markup * (qualified / workers) * (scarce / qualified)
    } else {
        0.0
    };

    let in_business = sites
        .iter()
        .any(|&site| in_business(state, catalog, buyer, site, made));
    let (own_business, new_build) = if in_business {
        let saved = if base > Money::ZERO {
            (new_build - base).max(Money::ZERO).to_usd() / base.to_usd()
        } else {
            0.0
        };
        (ai.build_time_markup, ai.new_build_share * saved)
    } else {
        (0.0, 0.0)
    };
    Advantages {
        competition,
        staff,
        own_business,
        new_build,
    }
}

/// What a licence saves a buyer: the research cost of the points it still misses
/// (docs/FORMELN.md, M30). `None` for technologies known from the start.
pub fn license_value(
    state: &GameState,
    catalog: &Catalog,
    buyer: CompanyId,
    technology: TechnologyId,
) -> Option<Money> {
    let date = state.date;
    let effort = crate::research::effort(catalog, state, technology, date)?;
    let company = &state.companies[buyer.index()];
    let collected = company.research.get(&technology).copied().unwrap_or(0.0);
    let missing = (effort.points - collected).max(0.0);
    let field = catalog.technologies.get(technology).field;
    let group = catalog
        .research_model
        .researchers
        .get(field.index())
        .copied()
        .flatten()?;
    let country = state.countries.get(company.headquarters);
    let wage = country
        .hourly_wage_usd
        .get(group.index())
        .copied()
        .unwrap_or(0.0)
        * crate::production::hours_per_worker_day(catalog, date);
    let material = catalog.research_model.material_usd_per_day * country.price_level;
    let efficiency = country
        .research_efficiency
        .get(field.index())
        .copied()
        .unwrap_or(1.0)
        .max(0.01);
    Money::from_usd(missing * (wage + material) / efficiency)
}

/// Months a site has existed.
fn age_months(state: &GameState, site: SiteId) -> u32 {
    months_between(state.sites[site.index()].founded, state.date)
}

/// Whether `object` belongs to `seller` and can go to `buyer`.
fn check_object(
    state: &GameState,
    catalog: &Catalog,
    buyer: CompanyId,
    seller: CompanyId,
    object: DealObject,
) -> Result<(), CommandError> {
    match object {
        DealObject::Site(site) => {
            let s = state.site(site).ok_or(CommandError::UnknownSite)?;
            if s.owner != seller {
                return Err(CommandError::NotSellersObject);
            }
            let min = catalog.deal_model.min_age_months;
            if age_months(state, site) < min {
                return Err(CommandError::SiteTooYoung { months: min });
            }
        }
        DealObject::License(t) => {
            if t.index() >= catalog.technologies.len()
                || !state.knows(catalog, seller, t)
                || state.knows(catalog, buyer, t)
            {
                return Err(CommandError::LicenseNotPossible);
            }
        }
        DealObject::Area(group) => {
            if group.index() >= catalog.goods_groups.len() {
                return Err(CommandError::NotSellersObject);
            }
            let sites = area_sites(state, catalog, seller, group);
            if sites.is_empty() {
                return Err(CommandError::NotSellersObject);
            }
            let min = catalog.deal_model.min_age_months;
            if sites.iter().all(|&s| age_months(state, s) < min) {
                return Err(CommandError::SiteTooYoung { months: min });
            }
        }
    }
    Ok(())
}

/// Until when a buyer may not offer a seller again for an object (declined or expired).
fn blocked_until(
    state: &GameState,
    catalog: &Catalog,
    buyer: CompanyId,
    seller: CompanyId,
    object: DealObject,
) -> Option<Date> {
    state
        .offers
        .iter()
        .filter(|o| {
            o.buyer == buyer
                && o.seller == seller
                && o.object == object
                && matches!(o.status, OfferStatus::Declined | OfferStatus::Expired)
        })
        .filter_map(|o| o.closed)
        .map(|closed| closed.add_months(catalog.deal_model.block_months))
        .filter(|&until| until > state.date)
        .max()
}

pub(crate) fn make_offer(
    state: &mut GameState,
    catalog: &Catalog,
    buyer: CompanyId,
    seller: CompanyId,
    object: DealObject,
    price: Money,
) -> Result<(), CommandError> {
    if seller == buyer {
        return Err(CommandError::OwnObject);
    }
    let other = state
        .company(seller)
        .ok_or(CommandError::UnknownCompany(seller))?;
    if other.bankrupt {
        return Err(CommandError::SellerBankrupt);
    }
    check_object(state, catalog, buyer, seller, object)?;
    if price <= Money::ZERO {
        return Err(CommandError::InvalidPrice);
    }
    if state.companies[buyer.index()].ledger.cash() < price {
        return Err(CommandError::NotEnoughCash { needed: price });
    }
    if state.offers.iter().any(|o| {
        o.status == OfferStatus::Open
            && o.buyer == buyer
            && o.seller == seller
            && o.object == object
    }) {
        return Err(CommandError::OfferExists);
    }
    if let Some(until) = blocked_until(state, catalog, buyer, seller, object) {
        return Err(CommandError::OfferBlocked { until });
    }
    let id = state.next_offer;
    state.next_offer += 1;
    state.offers.push(Offer {
        id,
        buyer,
        seller,
        object,
        price,
        counter: false,
        date: state.date,
        status: OfferStatus::Open,
        closed: None,
    });
    Ok(())
}

fn open_offer(state: &GameState, id: u32) -> Result<usize, CommandError> {
    state
        .offers
        .iter()
        .position(|o| o.id == id && o.status == OfferStatus::Open)
        .ok_or(CommandError::UnknownOffer)
}

pub(crate) fn answer_offer(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    id: u32,
    answer: OfferAnswer,
) -> Result<(), CommandError> {
    let index = open_offer(state, id)?;
    let offer = state.offers[index].clone();
    if offer.answering() != actor {
        return Err(CommandError::NotYourTurn);
    }
    let today = state.date;
    match answer {
        OfferAnswer::Decline => {
            let o = &mut state.offers[index];
            o.status = OfferStatus::Declined;
            o.closed = Some(today);
        }
        OfferAnswer::Counter { price } => {
            if offer.counter || price <= offer.price {
                return Err(CommandError::NoCounter);
            }
            let o = &mut state.offers[index];
            o.price = price;
            o.counter = true;
            o.date = today;
        }
        OfferAnswer::Accept => {
            check_object(state, catalog, offer.buyer, offer.seller, offer.object)?;
            let buyer = &state.companies[offer.buyer.index()];
            if buyer.bankrupt || buyer.ledger.cash() < offer.price {
                return Err(CommandError::BuyerCannotPay);
            }
            let moved = match offer.object {
                DealObject::Site(site) => {
                    hand_over(state, catalog, offer.buyer, offer.seller, site, offer.price);
                    vec![site]
                }
                DealObject::License(t) => {
                    license(state, offer.buyer, offer.seller, t, offer.price);
                    Vec::new()
                }
                DealObject::Area(group) => hand_over_area(
                    state,
                    catalog,
                    offer.buyer,
                    offer.seller,
                    group,
                    offer.price,
                ),
            };
            let o = &mut state.offers[index];
            o.status = OfferStatus::Accepted;
            o.closed = Some(today);
            // Other open offers for the sites handed over are void now, and so are those
            // for the seller's areas, whose content changed; licences can be sold to
            // several buyers, but a buyer needs only one.
            for o in &mut state.offers {
                let void = o.status == OfferStatus::Open
                    && match o.object {
                        DealObject::Site(site) => moved.contains(&site),
                        DealObject::Area(_) => !moved.is_empty() && o.seller == offer.seller,
                        DealObject::License(_) => {
                            o.object == offer.object && o.buyer == offer.buyer
                        }
                    };
                if void {
                    o.status = OfferStatus::Withdrawn;
                    o.closed = Some(today);
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn withdraw_offer(
    state: &mut GameState,
    actor: CompanyId,
    id: u32,
) -> Result<(), CommandError> {
    let index = open_offer(state, id)?;
    if state.offers[index].bidding() != actor {
        return Err(CommandError::NotYourTurn);
    }
    let today = state.date;
    let o = &mut state.offers[index];
    o.status = OfferStatus::Withdrawn;
    o.closed = Some(today);
    Ok(())
}

/// Hands a site over and books it on both sides (docs/FORMELN.md, M30).
fn hand_over(
    state: &mut GameState,
    catalog: &Catalog,
    buyer: CompanyId,
    seller: CompanyId,
    site: SiteId,
    price: Money,
) {
    let v = site_value(state, catalog, site);
    let today = state.date;
    let center = CostCenter::site(site);

    let l = &mut state.companies[seller.index()].ledger;
    l.transfer(Account::Cash, Account::FixedAssets, v.fixed_assets);
    l.transfer(
        Account::Cash,
        Account::AssetsUnderConstruction,
        v.under_construction,
    );
    l.transfer(Account::Cash, Account::Goodwill, v.goodwill);
    l.transfer(Account::Cash, Account::Inventory, v.inventory);
    l.transfer(Account::Cash, Account::Land, v.land_book);
    let book = v.book() + v.inventory;
    if price >= book {
        l.income(CostType::Other, center, Account::Cash, price - book);
    } else {
        l.expense(CostType::Other, center, Account::Cash, book - price);
    }

    let l = &mut state.companies[buyer.index()].ledger;
    l.transfer(Account::FixedAssets, Account::Cash, v.fixed_assets);
    l.transfer(
        Account::AssetsUnderConstruction,
        Account::Cash,
        v.under_construction,
    );
    l.transfer(Account::Inventory, Account::Cash, v.inventory);
    l.transfer(Account::Land, Account::Cash, v.land);
    let assets = v.fixed_assets + v.under_construction + v.inventory + v.land;
    let goodwill = if price >= assets {
        let amount = price - assets;
        l.transfer(Account::Goodwill, Account::Cash, amount);
        (amount > Money::ZERO).then_some(Goodwill {
            amount,
            from: today,
        })
    } else {
        // Bought below the book values: the difference is a gain.
        l.income(CostType::Other, center, Account::Cash, assets - price);
        None
    };

    let s = &mut state.sites[site.index()];
    s.owner = buyer;
    s.acquired = Some(today);
    s.goodwill = goodwill;
    s.staffing_due = true;
    // A bought plot goes over at today's value; a leased one stays leased (M35).
    if let Some(plot) = s.plot
        && matches!(
            state.plots[plot.index()].tenure,
            crate::state::Tenure::Owned(_)
        )
    {
        state.plots[plot.index()].tenure = crate::state::Tenure::Owned(v.land);
    }
}

/// Hands an area over: its sites, with the price split by their base values, and the
/// brand (docs/FORMELN.md, M31). Returns the sites.
fn hand_over_area(
    state: &mut GameState,
    catalog: &Catalog,
    buyer: CompanyId,
    seller: CompanyId,
    group: GoodsGroupId,
    price: Money,
) -> Vec<SiteId> {
    let value = area_value(state, catalog, seller, group);
    let total = value.sites.iter().map(|(_, v)| v.base).sum::<Money>();
    let count = value.sites.len();
    let mut rest = price;
    for (i, &(site, v)) in value.sites.iter().enumerate() {
        let part = if i + 1 == count {
            rest
        } else if total > Money::ZERO {
            price.scale(v.base.to_usd() / total.to_usd()).min(rest)
        } else {
            // Few sites; the cast is exact.
            price.scale(1.0 / count as f64).min(rest)
        };
        rest -= part;
        hand_over(state, catalog, buyer, seller, site, part);
    }
    let sold: Vec<crate::state::Brand> = state.companies[seller.index()]
        .brands
        .iter()
        .filter(|b| b.group == group)
        .copied()
        .collect();
    let s = &mut state.companies[seller.index()];
    s.brands.retain(|b| b.group != group);
    s.advertising.retain(|a| a.group != group);
    let brands = &mut state.companies[buyer.index()].brands;
    for b in sold {
        match brands
            .iter_mut()
            .find(|o| o.country == b.country && o.group == group)
        {
            Some(own) => own.awareness = own.awareness.max(b.awareness),
            None => brands.push(b),
        }
    }
    value.sites.into_iter().map(|(site, _)| site).collect()
}

/// Gives the buyer the technology; the seller keeps it.
fn license(
    state: &mut GameState,
    buyer: CompanyId,
    seller: CompanyId,
    technology: TechnologyId,
    price: Money,
) {
    let b = &mut state.companies[buyer.index()];
    b.ledger.expense(
        CostType::Licenses,
        CostCenter::default(),
        Account::Cash,
        price,
    );
    b.research.remove(&technology);
    b.technologies.insert(technology);
    state.companies[seller.index()].ledger.income(
        CostType::Licenses,
        CostCenter::default(),
        Account::Cash,
        price,
    );
}

/// Message parameters naming the object of an offer to `seller`.
pub(crate) fn describe(
    message: Message,
    state: &GameState,
    catalog: &Catalog,
    seller: CompanyId,
    object: DealObject,
) -> Message {
    match object {
        DealObject::Site(site) => {
            let s = &state.sites[site.index()];
            message
                .with("art", Param::TextKey(site_type_key(s.kind)))
                .with(
                    "land",
                    Param::Country(catalog.countries.key(s.country).to_owned()),
                )
        }
        DealObject::License(t) => message.with(
            "technologie",
            Param::TextKey(format!("technologie.{}", catalog.technologies.key(t))),
        ),
        DealObject::Area(group) => {
            let sites = area_sites(state, catalog, seller, group).len();
            message
                .with(
                    "warengruppe",
                    Param::TextKey(format!("warengruppe.{}", catalog.goods_groups.key(group))),
                )
                .with(
                    "standorte",
                    Param::Integer(i64::try_from(sites).unwrap_or(i64::MAX)),
                )
        }
    }
}

/// The key of a message by the kind of object: site, licence or area.
pub(crate) fn object_key(object: DealObject, keys: [&'static str; 3]) -> &'static str {
    match object {
        DealObject::Site(_) => keys[0],
        DealObject::License(_) => keys[1],
        DealObject::Area(_) => keys[2],
    }
}

/// A message about an offer for the player: the key by object kind.
fn news(
    state: &GameState,
    catalog: &Catalog,
    keys: [&'static str; 3],
    offer: &Offer,
    other: CompanyId,
) -> Message {
    let key = object_key(offer.object, keys);
    let message = Message::new(MessageKind::Info, key)
        .with(
            "firma",
            Param::Text(state.companies[other.index()].name.clone()),
        )
        .with("preis", Param::Money(offer.price))
        .with("frist", Param::Date(offer.deadline(catalog)));
    describe(message, state, catalog, offer.seller, offer.object)
}

/// The lowest price an AI seller accepts.
fn ai_minimum(state: &GameState, catalog: &Catalog, offer: &Offer) -> Money {
    let ai = &catalog.deal_model.ai;
    let (_, aggressiveness) = crate::ai::traits(state, offer.seller);
    match offer.object {
        DealObject::Site(site) => {
            let value = site_value(state, catalog, site);
            let mut minimum = value.base.scale(1.0 + ai.sale_markup.at(aggressiveness));
            if core_site(state, catalog, offer.seller, site) {
                minimum = minimum.scale(1.0 + ai.core_markup);
            }
            if needed_by_owner(state, catalog, site) {
                minimum = minimum.max(new_site_cost(state, catalog, site));
            }
            minimum
        }
        DealObject::License(t) => {
            let value = license_value(state, catalog, offer.buyer, t).unwrap_or(Money::ZERO);
            let minimum = value.scale(ai.license_min);
            if competitors(state, catalog, offer.buyer, offer.seller, t) {
                minimum.scale(1.0 + ai.license_competition)
            } else {
                minimum
            }
        }
        DealObject::Area(group) => {
            let sites = area_sites(state, catalog, offer.seller, group);
            let value = area_value_of(state, catalog, offer.seller, group, &sites);
            let minimum = value.base.scale(1.0 + ai.sale_markup.at(aggressiveness));
            // The core of the business, as for a single site.
            if revenue_share_reached(state, catalog, offer.seller, &sites)
                || whole_production(state, offer.seller, &sites)
            {
                minimum.scale(1.0 + ai.core_markup)
            } else {
                minimum
            }
        }
    }
}

/// The highest price an AI buyer pays.
fn ai_maximum(
    state: &GameState,
    catalog: &Catalog,
    buyer: CompanyId,
    seller: CompanyId,
    object: DealObject,
) -> Money {
    match object {
        DealObject::Site(site) => {
            let value = site_value(state, catalog, site);
            let adv = advantages(state, catalog, buyer, site, &value);
            value.base.scale(1.0 + adv.total())
        }
        DealObject::License(t) => license_value(state, catalog, buyer, t)
            .unwrap_or(Money::ZERO)
            .scale(catalog.deal_model.ai.license_max),
        DealObject::Area(group) => {
            let sites = area_sites(state, catalog, seller, group);
            let value = area_value_of(state, catalog, seller, group, &sites);
            let business = business(state, catalog, buyer);
            let adv = advantages_for(
                state,
                catalog,
                buyer,
                &business,
                &sites,
                value.base,
                value.new_build,
            );
            value.base.scale(1.0 + adv.total())
        }
    }
}

/// A site that carries the company: a large share of last month's revenue, or its only
/// site with facilities.
fn core_site(state: &GameState, catalog: &Catalog, company: CompanyId, site: SiteId) -> bool {
    let with_facilities = state
        .sites
        .iter()
        .filter(|s| s.owner == company && !s.slots.is_empty())
        .count();
    if with_facilities <= 1 && !state.sites[site.index()].slots.is_empty() {
        return true;
    }
    revenue_share_reached(state, catalog, company, &[site])
}

/// Whether sites brought at least `kern_anteil` of the company's revenue last month.
fn revenue_share_reached(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    sites: &[SiteId],
) -> bool {
    let ledger = &state.companies[company.index()].ledger;
    let Some(month) = ledger.months.last() else {
        return false;
    };
    let total = month
        .by_type
        .get(&CostType::Revenue)
        .copied()
        .unwrap_or(Money::ZERO);
    let share: f64 = sites
        .iter()
        .map(|&s| month.site_type(s, CostType::Revenue).to_usd())
        .sum();
    total > Money::ZERO && share / total.to_usd() >= catalog.deal_model.ai.core_share
}

/// Whether buyer and seller offer a product made with the technology in the same country.
fn competitors(
    state: &GameState,
    catalog: &Catalog,
    buyer: CompanyId,
    seller: CompanyId,
    technology: TechnologyId,
) -> bool {
    let enabled: BTreeSet<ProductId> = catalog
        .recipes
        .values()
        .filter(|r| {
            r.technology == Some(technology)
                || catalog.facilities.get(r.facility).technology == Some(technology)
        })
        .map(|r| r.product)
        .collect();
    let (_, sold_by_buyer) = business(state, catalog, buyer);
    let (_, sold_by_seller) = business(state, catalog, seller);
    sold_by_buyer
        .intersection(&sold_by_seller)
        .any(|(p, _)| enabled.contains(p))
}

/// How an AI company answers an offer it has to answer.
fn ai_answer(state: &GameState, catalog: &Catalog, offer: &Offer) -> OfferAnswer {
    let ai = &catalog.deal_model.ai;
    if offer.counter {
        let cash = state.companies[offer.buyer.index()].ledger.cash();
        let affordable = offer.price <= cash.scale(ai.cash_share_max);
        let highest = ai_maximum(state, catalog, offer.buyer, offer.seller, offer.object);
        return if affordable && offer.price <= highest {
            OfferAnswer::Accept
        } else {
            OfferAnswer::Decline
        };
    }

    let minimum = ai_minimum(state, catalog, offer);
    if offer.price >= minimum {
        OfferAnswer::Accept
    } else if offer.price >= minimum.scale(ai.counter_threshold) {
        OfferAnswer::Counter { price: minimum }
    } else {
        OfferAnswer::Decline
    }
}

/// One day of offers: expiry, the AI companies' answers, and cleaning up. Returns the
/// messages for the player.
pub(crate) fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut messages = Vec::new();
    let player = state.player;
    if state.offers.is_empty() {
        return messages;
    }
    // Offers of companies that went bankrupt are void.
    for o in &mut state.offers {
        let gone =
            state.companies[o.buyer.index()].bankrupt || state.companies[o.seller.index()].bankrupt;
        if o.status == OfferStatus::Open && gone {
            o.status = OfferStatus::Withdrawn;
            o.closed = Some(date);
        }
    }
    for i in 0..state.offers.len() {
        let o = &state.offers[i];
        if o.status != OfferStatus::Open || o.deadline(catalog) >= date {
            continue;
        }
        let o = o.clone();
        if o.buyer == player || o.seller == player {
            let other = if o.buyer == player { o.seller } else { o.buyer };
            messages.push(news(
                state,
                catalog,
                [
                    keys::OFFER_EXPIRED_SITE,
                    keys::OFFER_EXPIRED_LICENSE,
                    keys::OFFER_EXPIRED_AREA,
                ],
                &o,
                other,
            ));
        }
        let o = &mut state.offers[i];
        o.status = OfferStatus::Expired;
        o.closed = Some(date);
    }

    // AI companies answer on the day after an offer came in.
    let due: Vec<Offer> = state
        .offers
        .iter()
        .filter(|o| {
            o.status == OfferStatus::Open
                && o.date < date
                && state.companies[o.answering().index()].ai.is_some()
                && !state.companies[o.answering().index()].bankrupt
        })
        .cloned()
        .collect();
    for offer in due {
        let answer = ai_answer(state, catalog, &offer);
        let who = offer.answering();
        // Deals are described before the hand-over, while the seller still owns what it
        // sells.
        let accepted_news = if answer != OfferAnswer::Accept {
            None
        } else if offer.buyer == player {
            let keys = [
                keys::OFFER_BOUGHT_SITE,
                keys::OFFER_BOUGHT_LICENSE,
                keys::OFFER_BOUGHT_AREA,
            ];
            Some(news(state, catalog, keys, &offer, offer.seller))
        } else if offer.seller == player {
            let keys = [
                keys::OFFER_SOLD_SITE,
                keys::OFFER_SOLD_LICENSE,
                keys::OFFER_SOLD_AREA,
            ];
            Some(news(state, catalog, keys, &offer, offer.buyer))
        } else {
            ai_deal_news(state, catalog, &offer)
        };
        let command = Command::AnswerOffer {
            offer: offer.id,
            answer,
        };
        if command::execute(state, catalog, who, &command).is_err() {
            // The deal fell through (e.g. no cash any more): the AI declines.
            let decline = Command::AnswerOffer {
                offer: offer.id,
                answer: OfferAnswer::Decline,
            };
            let _ = command::execute(state, catalog, who, &decline);
        }
        let Some(after) = state.offers.iter().find(|o| o.id == offer.id).cloned() else {
            continue;
        };
        let player_side = if after.buyer == player {
            Some(after.seller)
        } else if after.seller == player {
            Some(after.buyer)
        } else {
            None
        };
        match (after.status, player_side) {
            (OfferStatus::Accepted, _) => messages.extend(accepted_news),
            (OfferStatus::Declined, Some(other)) => messages.push(news(
                state,
                catalog,
                [
                    keys::OFFER_DECLINED_SITE,
                    keys::OFFER_DECLINED_LICENSE,
                    keys::OFFER_DECLINED_AREA,
                ],
                &after,
                other,
            )),
            (OfferStatus::Open, Some(other)) if after.counter => messages.push(news(
                state,
                catalog,
                [
                    keys::OFFER_COUNTER_SITE,
                    keys::OFFER_COUNTER_LICENSE,
                    keys::OFFER_COUNTER_AREA,
                ],
                &after,
                other,
            )),
            _ => {}
        }
    }

    // Closed offers are kept as long as they can block a new offer.
    let keep_months = catalog.deal_model.block_months;
    state.offers.retain(|o| {
        o.status == OfferStatus::Open || o.closed.is_some_and(|c| c.add_months(keep_months) > date)
    });
    messages
}

/// News of a deal between AI companies that touches the player: a site that sells
/// what the player sells, or in a country where the player has a site.
fn ai_deal_news(state: &GameState, catalog: &Catalog, offer: &Offer) -> Option<Message> {
    let (sites, key) = match offer.object {
        DealObject::Site(site) => (vec![site], keys::AI_BUYS_SITE),
        DealObject::Area(group) => (
            area_sites(state, catalog, offer.seller, group),
            keys::AI_BUYS_AREA,
        ),
        DealObject::License(_) => return None,
    };
    let player = state.player;
    let (_, offered) = business(state, catalog, player);
    let relevant = sites.iter().any(|&site| {
        let s = &state.sites[site.index()];
        state
            .sites
            .iter()
            .any(|o| o.owner == player && o.country == s.country)
            || s.offers
                .keys()
                .any(|&p| offered.iter().any(|&(q, _)| q == p))
    });
    if !relevant {
        return None;
    }
    let message = Message::new(MessageKind::Info, key)
        .with(
            "kaeufer",
            Param::Text(state.companies[offer.buyer.index()].name.clone()),
        )
        .with(
            "verkaeufer",
            Param::Text(state.companies[offer.seller.index()].name.clone()),
        )
        .with("preis", Param::Money(offer.price));
    Some(describe(
        message,
        state,
        catalog,
        offer.seller,
        offer.object,
    ))
}

/// The best deal an AI company could offer for now: object, seller and price.
pub(crate) fn best_deal(
    state: &GameState,
    catalog: &Catalog,
    buyer: CompanyId,
) -> Option<(DealObject, CompanyId, Money)> {
    let model = &catalog.deal_model;
    let ai = &model.ai;
    let (_, aggressiveness) = crate::ai::traits(state, buyer);
    let company = &state.companies[buyer.index()];
    let budget = company.ledger.cash().scale(ai.cash_share_max);
    let min_price = Money::from_usd(ai.min_price_usd).unwrap_or(Money::ZERO);
    let date = state.date;
    let player = state.player;
    let to_player_this_month = state
        .offers
        .iter()
        .filter(|o| {
            o.seller == player && !o.counter && o.date.first_of_month() == date.first_of_month()
        })
        .count();
    let player_full =
        u32::try_from(to_player_this_month).unwrap_or(u32::MAX) >= ai.player_offers_per_month;
    let taken = |seller: CompanyId, object: DealObject| {
        state.offers.iter().any(|o| {
            o.status == OfferStatus::Open
                && o.buyer == buyer
                && o.seller == seller
                && o.object == object
        }) || blocked_until(state, catalog, buyer, seller, object).is_some()
    };
    let mut countries: BTreeSet<CountryId> = state
        .sites
        .iter()
        .filter(|s| s.owner == buyer)
        .map(|s| s.country)
        .collect();
    countries.insert(company.headquarters);

    // The deal with the most room per dollar: small but attractive objects count as
    // much as large ones.
    let mut best: Option<(f64, DealObject, CompanyId, Money)> = None;
    let mut consider = |room: Money, object: DealObject, seller: CompanyId, price: Money| {
        if price < min_price || price > budget || room <= Money::ZERO || price <= Money::ZERO {
            return;
        }
        let score = room.to_usd() / price.to_usd();
        if best.as_ref().is_none_or(|b| score > b.0) {
            best = Some((score, object, seller, price));
        }
    };

    let business = business(state, catalog, buyer);
    let available = |seller: CompanyId| {
        seller != buyer
            && !state.companies[seller.index()].bankrupt
            && !(seller == player && player_full)
    };
    for (i, s) in state.sites.iter().enumerate() {
        let site = SiteId(u32::try_from(i).unwrap_or(u32::MAX));
        let seller = s.owner;
        if !available(seller)
            || !countries.contains(&s.country)
            || age_months(state, site) < model.min_age_months
            || taken(seller, DealObject::Site(site))
        {
            continue;
        }
        // Laboratories and power plants serve only the owner's own work.
        if matches!(s.kind, SiteType::ResearchCenter | SiteType::PowerPlant)
            && !in_business(state, catalog, buyer, site, &business.0)
        {
            continue;
        }
        let value = site_value(state, catalog, site);
        if value.base <= Money::ZERO {
            continue;
        }
        let anew = new_site_cost(state, catalog, site);
        let adv = advantages_for(state, catalog, buyer, &business, &[site], value.base, anew);
        if adv.total() < ai.min_advantage {
            continue;
        }
        let highest = value.base.scale(1.0 + adv.total());
        let mut price = value
            .base
            .scale(1.0 + ai.bid_markup.at(aggressiveness))
            .min(highest);
        // An owner gives up a site it needs only for what a new one would cost.
        if needed_by_owner(state, catalog, site) {
            if highest < anew {
                continue;
            }
            price = price.max(anew);
        }
        consider(highest - price, DealObject::Site(site), seller, price);
    }

    // Areas (M31): with at least two sites or a brand – else it is the single site – and
    // a site in one of the buyer's countries.
    let mut areas: std::collections::BTreeMap<(CompanyId, GoodsGroupId), Vec<SiteId>> =
        std::collections::BTreeMap::new();
    for (i, s) in state.sites.iter().enumerate() {
        if !area_kind(s.kind) || !available(s.owner) {
            continue;
        }
        let site = SiteId(u32::try_from(i).unwrap_or(u32::MAX));
        for group in site_groups(state, catalog, site) {
            areas.entry((s.owner, group)).or_default().push(site);
        }
    }
    for ((seller, group), sites) in areas {
        let object = DealObject::Area(group);
        let brand = brand_value(state, catalog, seller, group);
        if (sites.len() < 2 && brand <= Money::ZERO)
            || !sites
                .iter()
                .any(|s| countries.contains(&state.sites[s.index()].country))
            || sites
                .iter()
                .all(|&s| age_months(state, s) < model.min_age_months)
            || taken(seller, object)
        {
            continue;
        }
        let value = area_value_of(state, catalog, seller, group, &sites);
        if value.base <= Money::ZERO {
            continue;
        }
        let adv = advantages_for(
            state,
            catalog,
            buyer,
            &business,
            &sites,
            value.base,
            value.new_build,
        );
        if adv.total() < ai.min_advantage {
            continue;
        }
        let highest = value.base.scale(1.0 + adv.total());
        let price = value
            .base
            .scale(1.0 + ai.bid_markup.at(aggressiveness))
            .min(highest);
        consider(highest - price, object, seller, price);
    }

    // Licences for what the company is researching, from the first company that knows it.
    let researching: BTreeSet<TechnologyId> = company
        .research
        .keys()
        .copied()
        .chain(
            state
                .sites
                .iter()
                .filter(|s| s.owner == buyer)
                .filter_map(|s| s.research),
        )
        .collect();
    for t in researching {
        let object = DealObject::License(t);
        if state.knows(catalog, buyer, t) {
            continue;
        }
        let Some(seller) = (0..state.companies.len())
            .map(|i| CompanyId(u32::try_from(i).unwrap_or(u32::MAX)))
            .find(|&id| available(id) && state.knows(catalog, id, t) && !taken(id, object))
        else {
            continue;
        };
        let Some(value) = license_value(state, catalog, buyer, t) else {
            continue;
        };
        let price = value.scale(ai.license_bid.at(aggressiveness));
        let highest = value.scale(ai.license_max);
        consider(highest - price, object, seller, price);
    }
    best.map(|(_, object, seller, price)| (object, seller, price))
}

/// An AI company's monthly look for a deal (docs/FORMELN.md, M30). Returns the news for
/// the player when the offer goes to the player.
pub(crate) fn ai_offers(
    state: &mut GameState,
    catalog: &Catalog,
    buyer: CompanyId,
) -> Option<Message> {
    let ai = &catalog.deal_model.ai;
    let (_, aggressiveness) = crate::ai::traits(state, buyer);
    if !state.companies[buyer.index()]
        .rng
        .chance(ai.chance.at(aggressiveness))
    {
        return None;
    }
    let open = state
        .offers
        .iter()
        .filter(|o| o.status == OfferStatus::Open && o.buyer == buyer)
        .count();
    if u32::try_from(open).unwrap_or(u32::MAX) >= ai.open_max {
        return None;
    }
    let (object, seller, price) = best_deal(state, catalog, buyer)?;
    let command = Command::MakeOffer {
        seller,
        object,
        price,
    };
    command::execute(state, catalog, buyer, &command).ok()?;
    if seller != state.player {
        return None;
    }
    let offer = state.offers.last()?.clone();
    Some(news(
        state,
        catalog,
        [
            keys::OFFER_RECEIVED_SITE,
            keys::OFFER_RECEIVED_LICENSE,
            keys::OFFER_RECEIVED_AREA,
        ],
        &offer,
        buyer,
    ))
}
