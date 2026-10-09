//! Plots of commercial land for the sites (M35; formulas in docs/FORMELN.md).
//!
//! Every site except extraction sites stands on one plot. Countries offer plots in
//! proportion to their economy; new ones come on the market every 1 January. A site
//! grows with its facilities until its plot is full.

use crate::calendar::Date;
use crate::catalog::{Catalog, FacilitySize, Location, SiteType};
use crate::ids::{CountryId, FacilityId, Id};
use crate::ledger::{Account, CostCenter, CostType};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{GameState, Plot, PlotId, Site, SiteId, Tenure};

/// Land one unit of a facility takes (ha).
pub fn facility_area(catalog: &Catalog, facility: FacilityId) -> f64 {
    let f = catalog.facilities.get(facility);
    f.area_ha
        .unwrap_or_else(|| f.investment.to_usd() / catalog.plot_model.investment_per_ha_usd)
}

/// Land one unit of a facility of a size takes (ha, M36).
pub fn unit_area(catalog: &Catalog, facility: FacilityId, size: FacilitySize) -> f64 {
    facility_area(catalog, facility) * catalog.production_model.sizes.area(size)
}

/// Land of the units standing or under construction at a site, without the overhead.
fn units_area(catalog: &Catalog, site: &Site) -> f64 {
    site.slots
        .iter()
        .map(|sl| unit_area(catalog, sl.facility, sl.size) * f64::from(sl.count))
        .sum()
}

/// Land a site needs, with `extra` more units of a facility (ha). Standing units and
/// units under construction count too.
pub fn site_area(
    catalog: &Catalog,
    site: &Site,
    extra: Option<(FacilityId, FacilitySize, u32)>,
) -> f64 {
    let m = &catalog.plot_model;
    let units = units_area(catalog, site)
        + extra.map_or(0.0, |(f, size, n)| {
            unit_area(catalog, f, size) * f64::from(n)
        });
    (units * (1.0 + m.overhead)).max(m.min_site_area_ha)
}

/// Land a new site needs for `count` units of a facility of a size (ha).
pub fn project_area(
    catalog: &Catalog,
    facility: FacilityId,
    size: FacilitySize,
    count: u32,
) -> f64 {
    let m = &catalog.plot_model;
    (unit_area(catalog, facility, size) * f64::from(count) * (1.0 + m.overhead))
        .max(m.min_site_area_ha)
}

/// Whether sites of a kind stand on a plot (extraction sites stand on their concession).
pub fn needs_plot(catalog: &Catalog, kind: SiteType) -> bool {
    catalog.plot_model.enabled() && kind != SiteType::Extraction
}

/// All land of a country and the occupied part (ha).
pub fn land(state: &GameState, country: CountryId) -> (f64, f64) {
    state
        .plots
        .iter()
        .filter(|p| p.country == country)
        .fold((0.0, 0.0), |(all, used), p| {
            (
                all + p.area_ha,
                used + if p.site.is_some() { p.area_ha } else { 0.0 },
            )
        })
}

/// Land price per ha in a country before the factor of the location (USD).
fn base_price_per_ha(catalog: &Catalog, state: &GameState, country: CountryId) -> f64 {
    let m = &catalog.plot_model;
    let (all, used) = land(state, country);
    let occupied = if all > 0.0 { used / all } else { 0.0 };
    m.land_price_usd_per_ha
        * state.countries.get(country).price_level
        * (1.0 + m.scarcity * occupied)
}

/// Land price per ha in a country at a location.
pub fn price_per_ha(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    location: Location,
) -> Money {
    let usd = base_price_per_ha(catalog, state, country)
        * catalog.plot_model.location(location).price_factor;
    Money::from_usd(usd).unwrap_or(Money::ZERO)
}

/// Today's value of a plot.
pub fn value(catalog: &Catalog, state: &GameState, plot: PlotId) -> Money {
    let p = &state.plots[plot.index()];
    price_per_ha(catalog, state, p.country, p.location).scale(p.area_ha)
}

/// Units of a facility of a size that still fit on a site's plot; unlimited without one.
pub fn units_that_fit(
    catalog: &Catalog,
    state: &GameState,
    site: SiteId,
    (facility, size): (FacilityId, FacilitySize),
) -> u32 {
    let s = &state.sites[site.index()];
    let Some(plot) = s.plot else {
        return u32::MAX;
    };
    let m = &catalog.plot_model;
    let area = state.plots[plot.index()].area_ha;
    if area + 1e-9 < m.min_site_area_ha {
        return 0;
    }
    let per_unit = unit_area(catalog, facility, size) * (1.0 + m.overhead);
    if per_unit <= 0.0 {
        return u32::MAX;
    }
    // The minimum area of a site is no extra need: the units fill it first.
    let free = area - units_area(catalog, s) * (1.0 + m.overhead);
    // Small counts; the cast saturates.
    (free / per_unit + 1e-9).floor().max(0.0) as u32
}

/// The free plot for a project that needs `need_ha` in a country and sells for
/// `revenue_usd` a year in it: among those that hold it the one with the lowest yearly
/// cost – rent on its value plus deliveries on the revenue – (the smallest of equals),
/// else the largest free one.
pub fn choose(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    need_ha: f64,
    revenue_usd: f64,
) -> Option<PlotId> {
    let m = &catalog.plot_model;
    let base = base_price_per_ha(catalog, state, country);
    let free = state
        .plots
        .iter()
        .enumerate()
        .filter(|(_, p)| p.country == country && p.site.is_none());
    let mut fitting: Option<(f64, f64, usize)> = None;
    let mut largest: Option<(f64, usize)> = None;
    for (i, p) in free {
        if p.area_ha >= need_ha {
            let l = m.location(p.location);
            let cost =
                base * l.price_factor * p.area_ha * m.rent_share + l.delivery_cost * revenue_usd;
            if fitting.is_none_or(|f| (cost, p.area_ha) < (f.0, f.1)) {
                fitting = Some((cost, p.area_ha, i));
            }
        } else if largest.is_none_or(|(a, _)| p.area_ha > a) {
            largest = Some((p.area_ha, i));
        }
    }
    fitting
        .map(|(_, _, i)| i)
        .or(largest.map(|(_, i)| i))
        .map(plot_id)
}

fn plot_id(index: usize) -> PlotId {
    PlotId(u32::try_from(index).expect("plot count fits u32"))
}

/// Size class of new plots and the factor they grew by in a year.
fn growth(catalog: &Catalog, year: i32) -> f64 {
    let m = &catalog.plot_model;
    (1.0 + f64::from(year - m.growth_from_year) / m.growth_years).max(0.1)
}

/// Index drawn by weights.
fn pick(rng: &mut SimRng, weights: &[f64]) -> usize {
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return 0;
    }
    let mut r = rng.next_f64() * total;
    for (i, &w) in weights.iter().enumerate() {
        if r < w {
            return i;
        }
        r -= w;
    }
    weights.iter().rposition(|&w| w > 0.0).unwrap_or(0)
}

/// New plots up to the commercial land each country's economy carries, and at least so
/// much that `frei_min_anteil` of it stays free (L1): at the start of a game and every
/// 1 January.
pub(crate) fn supply(state: &mut GameState, catalog: &Catalog, year: i32) {
    let m = &catalog.plot_model;
    if !m.enabled() {
        return;
    }
    let grown = growth(catalog, year);
    for (country, c) in catalog.countries.iter() {
        let v = state.countries.get(country);
        let gdp_bn = v.population * v.gdp_per_capita_usd / 1e9;
        let (mut offered, used) = land(state, country);
        // Land is zoned where industry needs it: with a fixed area per GDP, the land of
        // every large country was taken by 1950 and nobody could build anew (L1).
        let target = (m.area_per_gdp_bn_ha * gdp_bn * state.settings.market_scale)
            .max(used / (1.0 - m.free_share_min));
        if offered >= target {
            continue;
        }
        let wealth = if v.gdp_per_capita_usd >= m.rich_from_usd {
            0
        } else if v.gdp_per_capita_usd < m.poor_below_usd {
            2
        } else {
            1
        };
        let class_shares: Vec<f64> = m.classes.iter().map(|k| k.shares[wealth]).collect();
        let location_shares: Vec<f64> = Location::ALL
            .iter()
            .map(|&l| {
                if l == Location::Port && c.landlocked {
                    0.0
                } else {
                    m.location(l).share
                }
            })
            .collect();
        let mut rng = SimRng::for_stream(
            state.settings.seed,
            Stream::Plots {
                country: u16::try_from(country.index()).unwrap_or(u16::MAX),
                year: u16::try_from(year).unwrap_or(u16::MAX),
            },
        );
        while offered < target {
            let location = Location::ALL[pick(&mut rng, &location_shares)];
            let class = pick(&mut rng, &class_shares);
            let (lo, hi) = m.classes[class].area_ha;
            let area = (lo + (hi - lo) * rng.next_f64()) * grown * m.location(location).area_factor;
            let area = ((area * 100.0).round() / 100.0).max(0.01);
            state.plots.push(Plot {
                country,
                location,
                area_ha: area,
                class: u8::try_from(class).unwrap_or(u8::MAX),
                since: year,
                site: None,
                tenure: Tenure::Leased,
            });
            offered += area;
        }
    }
}

/// A plot for a site that must exist (start of a game, saves from before M35): a free
/// one that holds it, else a new one in the city of exactly that size.
pub(crate) fn for_existing(
    state: &mut GameState,
    catalog: &Catalog,
    country: CountryId,
    (need_ha, revenue_usd): (f64, f64),
) -> PlotId {
    if let Some(p) = choose(catalog, state, country, need_ha, revenue_usd)
        .filter(|p| state.plots[p.index()].area_ha >= need_ha)
    {
        return p;
    }
    let m = &catalog.plot_model;
    let year = state.date.year();
    let area = (need_ha * 100.0).ceil() / 100.0;
    // The class whose plots of this year reach the area, else the largest.
    let grown = growth(catalog, year) * m.location(Location::City).area_factor;
    let class = m
        .classes
        .iter()
        .position(|k| k.area_ha.1 * grown >= area)
        .unwrap_or(m.classes.len().saturating_sub(1));
    state.plots.push(Plot {
        country,
        location: Location::City,
        area_ha: area,
        class: u8::try_from(class).unwrap_or(u8::MAX),
        since: year,
        site: None,
        tenure: Tenure::Leased,
    });
    plot_id(state.plots.len() - 1)
}

/// Puts a site on a plot.
pub(crate) fn occupy(state: &mut GameState, plot: PlotId, site: SiteId, tenure: Tenure) {
    let p = &mut state.plots[plot.index()];
    p.site = Some(site);
    p.tenure = tenure;
    state.sites[site.index()].plot = Some(plot);
}

/// Gives a site's plot back to the market (the site is given up).
pub(crate) fn release(state: &mut GameState, site: SiteId) {
    if let Some(plot) = state.sites[site.index()].plot.take() {
        let p = &mut state.plots[plot.index()];
        p.site = None;
        p.tenure = Tenure::Leased;
    }
}

/// Rent of the leased plots for the month that begins: `pacht_anteil` / 12 of today's
/// value.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) {
    let m = &catalog.plot_model;
    if !m.enabled() {
        return;
    }
    if date.ordinal() == 1 {
        supply(state, catalog, date.year());
    }
    let mut rents: Vec<(SiteId, Money)> = Vec::new();
    for (i, p) in state.plots.iter().enumerate() {
        let (Some(site), Tenure::Leased) = (p.site, p.tenure) else {
            continue;
        };
        let owner = state.sites[site.index()].owner;
        if state.companies[owner.index()].bankrupt {
            continue;
        }
        let rent = value(catalog, state, plot_id(i)).scale(m.rent_share / 12.0);
        if rent > Money::ZERO {
            rents.push((site, rent));
        }
    }
    for (site, rent) in rents {
        let owner = state.sites[site.index()].owner;
        state.companies[owner.index()].ledger.expense(
            CostType::Rent,
            CostCenter::site(site),
            Account::Cash,
            rent,
        );
    }
}

/// Saves from before M35: the plots of the year, and a bought plot for every site that
/// needs one (its price was part of the site costs, so nothing is booked).
pub(crate) fn fit_loaded(state: &mut GameState, catalog: &Catalog) {
    if !catalog.plot_model.enabled() {
        return;
    }
    let missing: Vec<usize> = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            s.plot.is_none()
                && needs_plot(catalog, s.kind)
                && !state.companies[s.owner.index()].bankrupt
        })
        .map(|(i, _)| i)
        .collect();
    if missing.is_empty() {
        return;
    }
    if state.plots.is_empty() {
        supply(state, catalog, state.date.year());
    }
    let reserve = 1.0 + catalog.plot_model.ai_reserve;
    for i in missing {
        let site = SiteId(u32::try_from(i).expect("site count fits u32"));
        let s = &state.sites[i];
        let (country, need) = (s.country, site_area(catalog, s, None) * reserve);
        let revenue = planned_revenue(state, catalog, s);
        let plot = for_existing(state, catalog, country, (need, revenue));
        occupy(state, plot, site, Tenure::Owned(Money::ZERO));
    }
}

/// Yearly revenue of a site's planned production of goods it offers, at the market
/// prices of its country (USD; for the choice of plots).
pub fn planned_revenue(state: &GameState, catalog: &Catalog, site: &Site) -> f64 {
    site.slots
        .iter()
        .filter_map(|sl| {
            let r = catalog.recipes.get(sl.recipe?);
            site.offers.contains_key(&r.product).then(|| {
                sl.full_runs(catalog)
                    * sl.utilization
                    * r.output
                    * crate::market::market_price(catalog, state, site.country, r.product).to_usd()
            })
        })
        .sum::<f64>()
        * 365.0
}

/// Yearly revenue of `capacity` (units of the data size, M36) of a facility making
/// `product` at the start utilization, at the market price of the country (USD; for the
/// choice of plots).
pub fn project_revenue(
    state: &GameState,
    catalog: &Catalog,
    country: CountryId,
    recipe: crate::ids::RecipeId,
    capacity: f64,
) -> f64 {
    let r = catalog.recipes.get(recipe);
    let runs = catalog.facilities.get(r.facility).runs_per_day
        * capacity
        * catalog.ai_model.start.utilization;
    runs * r.output
        * crate::market::market_price(catalog, state, country, r.product).to_usd()
        * 365.0
}

/// The location of a site's plot, if it has one.
pub fn location_of(state: &GameState, site: &Site) -> Option<Location> {
    site.plot.map(|p| state.plots[p.index()].location)
}

/// Head start of a site when hiring (added to its wage premium; M35).
pub fn hiring(catalog: &Catalog, state: &GameState, site: &Site) -> f64 {
    location_of(state, site).map_or(0.0, |l| catalog.plot_model.location(l).hiring)
}

/// Factor on freight by sea to and from a site.
pub fn sea_freight(catalog: &Catalog, state: &GameState, site: &Site) -> f64 {
    location_of(state, site).map_or(1.0, |l| catalog.plot_model.location(l).sea_freight)
}

/// Share of its sales on the home market a site pays for deliveries.
pub fn delivery_cost(catalog: &Catalog, state: &GameState, site: &Site) -> f64 {
    location_of(state, site).map_or(0.0, |l| catalog.plot_model.location(l).delivery_cost)
}
