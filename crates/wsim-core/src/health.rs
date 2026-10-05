//! Market health of the last closed month (plausibility, M16): how well consumers,
//! governments and facilities are supplied, per product and country, and where prices
//! stand against the reference prices. The balance protocol and the plausibility tests
//! read it; it changes nothing in the game.

use crate::EARLIEST_START_YEAR;
use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::country_model::{self, CountryState};
use crate::ids::{CountryId, Id, ProductId, RecipeId};
use crate::market;
use crate::state::{GameState, Limit};

/// Health of one product in the last closed month.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductHealth {
    pub product: ProductId,
    /// Demand of consumers and governments, worldwide.
    pub outside_demand: f64,
    /// Of that, served.
    pub outside_sold: f64,
    /// Per country with outside demand: (country, demand, served).
    pub countries: Vec<(CountryId, f64, f64)>,
    /// Planned daily output of the finished facilities making the product.
    pub planned: f64,
    /// Of that, held back by missing inputs on the last production day.
    pub input_limited: f64,
    /// Of that, held back by missing workers.
    pub labor_limited: f64,
    /// Full daily output of these facilities.
    pub capacity: f64,
    /// Daily full cost of these facilities at their planned utilization: inputs at the
    /// country's market prices, wages, electricity, depreciation and maintenance.
    pub cost_usd: f64,
    /// Companies' sales of the month and their value at the selling price and at the
    /// local reference price (for the price level against the reference).
    pub company_sold: f64,
    pub revenue_usd: f64,
    pub reference_value_usd: f64,
}

impl ProductHealth {
    /// Share of the outside demand that was served (1 without demand).
    pub fn coverage(&self) -> f64 {
        if self.outside_demand > 1e-9 {
            (self.outside_sold / self.outside_demand).min(1.0)
        } else {
            1.0
        }
    }

    /// Average selling price against the reference price, if companies sold.
    pub fn price_ratio(&self) -> Option<f64> {
        (self.reference_value_usd > 1e-9).then(|| self.revenue_usd / self.reference_value_usd)
    }

    /// Full cost per unit of the planned output, if any is planned.
    pub fn unit_cost_usd(&self) -> Option<f64> {
        (self.planned > 1e-9).then(|| self.cost_usd / self.planned)
    }

    /// Margin of the average selling price over the full unit cost, if both are known.
    pub fn margin(&self) -> Option<f64> {
        let cost = self.unit_cost_usd()?;
        (self.company_sold > 1e-9 && self.revenue_usd > 1e-9).then(|| {
            let price = self.revenue_usd / self.company_sold;
            1.0 - cost / price
        })
    }

    /// Share of the planned output held back by missing inputs.
    pub fn input_shortage(&self) -> f64 {
        if self.planned > 1e-9 {
            self.input_limited / self.planned
        } else {
            0.0
        }
    }
}

/// Health of every product in the last closed month.
pub fn last_month(state: &GameState, catalog: &Catalog) -> Vec<ProductHealth> {
    let mut health: Vec<ProductHealth> = catalog
        .products
        .ids()
        .map(|product| ProductHealth {
            product,
            outside_demand: 0.0,
            outside_sold: 0.0,
            countries: Vec::new(),
            planned: 0.0,
            input_limited: 0.0,
            labor_limited: 0.0,
            capacity: 0.0,
            cost_usd: 0.0,
            company_sold: 0.0,
            revenue_usd: 0.0,
            reference_value_usd: 0.0,
        })
        .collect();
    for (product, h) in catalog.products.ids().zip(health.iter_mut()) {
        for country in catalog.countries.ids() {
            let t = &state.markets.get(product).get(country).last_month;
            h.outside_demand += t.outside_demand;
            h.outside_sold += t.outside_sold;
            if t.outside_demand > 1e-9 {
                h.countries
                    .push((country, t.outside_demand, t.outside_sold));
            }
        }
    }
    for site in &state.sites {
        if state.companies[site.owner.index()].bankrupt {
            continue;
        }
        for sl in site.slots.iter().filter(|sl| sl.ready <= state.date) {
            let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r)) else {
                continue;
            };
            let runs = catalog.facilities.get(sl.facility).runs_per_day * f64::from(sl.count);
            let flows = crate::population::slot_flows(
                catalog,
                state,
                site.country,
                sl.recipe.expect("set"),
                sl.count,
                sl.utilization,
            );
            let h = &mut health[r.product.index()];
            h.capacity += runs * r.output;
            h.planned += runs * sl.utilization * r.output;
            h.cost_usd += flows.cost_per_day.to_usd();
            let held_back = ((runs * sl.utilization - sl.last_runs) * r.output).max(0.0);
            match sl.limit {
                Some(Limit::Input(_)) => h.input_limited += held_back,
                Some(Limit::Labor(_)) => h.labor_limited += held_back,
                _ => {}
            }
        }
        for (&product, offer) in &site.offers {
            let h = &mut health[product.index()];
            h.company_sold += offer.sold_last_month;
            h.revenue_usd += offer.sold_last_month * offer.price.to_usd();
            h.reference_value_usd += offer.sold_last_month
                * market::local_reference(catalog, state, site.country, product).to_usd();
        }
    }
    health
}

/// Full cost of one unit made by a recipe, by kind of cost (USD per unit of the main
/// product; by-products are credited).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UnitCost {
    pub inputs: f64,
    pub labor: f64,
    pub energy: f64,
    /// Depreciation and maintenance of the facility.
    pub capital: f64,
    /// Administration, sales and logistics.
    pub overhead: f64,
    /// Land rent and royalties of extraction.
    pub rent: f64,
    pub by_products: f64,
}

impl UnitCost {
    pub fn total(&self) -> f64 {
        self.inputs + self.labor + self.energy + self.capital + self.overhead + self.rent
            - self.by_products
    }
}

/// Full cost of one unit made by `recipe` in a country, apart from the market: inputs
/// and by-products at `price`, the country's wages, labor productivity and electricity,
/// overhead, and depreciation and maintenance of the facility at `utilization` (no
/// automation, average deposits).
pub fn unit_cost(
    catalog: &Catalog,
    country: &CountryState,
    recipe: RecipeId,
    utilization: f64,
    price: impl Fn(ProductId) -> f64,
) -> UnitCost {
    let r = catalog.recipes.get(recipe);
    let f = catalog.facilities.get(r.facility);
    let per_unit = 1.0 / r.output.max(1e-9);
    let labor = r
        .labor_hours
        .iter()
        .map(|&(g, h)| {
            h * country
                .hourly_wage_usd
                .get(g.index())
                .copied()
                .unwrap_or(0.0)
        })
        .sum::<f64>()
        / country.labor_productivity.max(1e-9);
    let capital = f.investment.to_usd()
        * (1.0 / f64::from(f.lifetime_years.max(1)) + f.maintenance_share)
        / (365.0 * f.runs_per_day.max(1e-9) * utilization.max(1e-9));
    let energy = r.energy_mwh * country.electricity_price_usd_mwh;
    let conversion = labor + energy + crate::production::capital_per_run_usd(catalog, r);
    UnitCost {
        inputs: r.inputs.iter().map(|&(p, q)| q * price(p)).sum::<f64>() * per_unit,
        labor: labor * per_unit,
        energy: energy * per_unit,
        capital: capital * per_unit,
        overhead: crate::production::overhead_usd(catalog, r, conversion) * per_unit,
        rent: catalog.products.get(r.product).rent_share
            * catalog.products.get(r.product).reference_price.to_usd()
            * market::level_factor(catalog, country.price_level, r.product),
        by_products: r
            .by_products
            .iter()
            .map(|&(p, q)| q * price(p))
            .sum::<f64>()
            * per_unit,
    }
}

/// Whether some recipe makes the product as its main product in a year; otherwise it
/// only comes as a by-product (petrol before cracking), and its price follows the
/// main product's output rather than its own cost.
pub fn made_as_main(catalog: &Catalog, product: ProductId, year: i32) -> bool {
    catalog
        .recipes
        .iter()
        .any(|(id, r)| r.product == product && first_year(catalog, id) <= year)
}

/// The first year in which a recipe can be used (its own and its facility's
/// technology, not before the earliest start).
pub fn first_year(catalog: &Catalog, recipe: RecipeId) -> i32 {
    let r = catalog.recipes.get(recipe);
    [r.technology, catalog.facilities.get(r.facility).technology]
        .into_iter()
        .flatten()
        .map(|t| catalog.technologies.get(t).invention_year)
        .fold(EARLIEST_START_YEAR, i32::max)
}

/// Margin of the reference price over the full unit cost of a recipe in a country, with
/// all goods at their reference prices in that country and the facility at normal
/// utilization.
pub fn margin_in(catalog: &Catalog, country: &CountryState, recipe: RecipeId) -> Option<f64> {
    let price = |p: ProductId| {
        catalog.products.get(p).reference_price.to_usd()
            * market::level_factor(catalog, country.price_level, p)
    };
    let reference = price(catalog.recipes.get(recipe).product);
    let utilization = catalog.market_model.normal_utilization;
    let cost = unit_cost(catalog, country, recipe, utilization, price).total();
    (reference > 0.0).then(|| 1.0 - cost / reference)
}

/// Margin of a recipe at reference prices (plausibility of the data): `margin_in` the
/// price reference country in the middle of the first year the recipe can be used.
/// `None` without a price reference country.
pub fn reference_margin(catalog: &Catalog, recipe: RecipeId) -> Option<f64> {
    let country = catalog.country_model.price_reference?;
    let date = Date::new(first_year(catalog, recipe), 7, 1)?;
    margin_in(
        catalog,
        &country_model::compute(catalog, country, date),
        recipe,
    )
}
