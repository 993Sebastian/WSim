//! The AI companies at the start and later foundings (Lastenheft §10; docs/FORMELN.md,
//! M10).
//!
//! The start population covers the demand of the start year top-down: end products
//! first, then the inputs their plants need, down to the raw materials. Plants go to
//! countries by demand and development level (raw materials to their deposits) and are
//! grouped into as many companies as the settings ask for; historical companies come
//! first with their own plants.

use std::collections::BTreeMap;

use crate::calendar::Date;
use crate::catalog::{Catalog, ProductKind, RealCompany, SiteType};
use crate::ids::{BranchId, CountryId, DepositId, FacilityId, Id, ProductId, RecipeId};
use crate::ledger::{Account, Ledger};
use crate::market;
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{
    AiState, Company, CompanyId, CompanyKind, GameState, PerId, PriceMode, PurchaseOrder,
    SaleOffer, Site, SiteId, Slot, Stock,
};

/// Daily output of one facility running a recipe at full capacity.
pub fn output_per_day(catalog: &Catalog, recipe: RecipeId) -> f64 {
    let r = catalog.recipes.get(recipe);
    catalog.facilities.get(r.facility).runs_per_day * r.output
}

/// Yearly output of one extraction facility for a raw material, if a recipe extracts it.
pub fn extraction_capacity(catalog: &Catalog, resource: ProductId) -> Option<f64> {
    catalog
        .recipes
        .iter()
        .filter(|(_, r)| r.extraction && r.product == resource)
        .map(|(id, _)| output_per_day(catalog, id) * 365.0)
        .reduce(f64::max)
}

/// Number of concessions of a deposit: enough for `plants_per_concession` extraction
/// facilities each at the market scale, within 1 and the maximum.
pub fn concession_count(catalog: &Catalog, deposit: DepositId, scale: f64) -> u32 {
    let d = catalog.deposits.get(deposit);
    let model = &catalog.ai_model;
    let Some(capacity) = extraction_capacity(catalog, d.resource) else {
        return 1;
    };
    let plants = d.max_output_per_year * scale / (model.plants_per_concession * capacity);
    // Far below u32::MAX; the clamp keeps it within the data limit.
    (plants.floor().max(1.0) as u32).min(model.max_concessions.max(1))
}

/// Whether a recipe and its facility only need technologies invented by `year`.
fn usable(catalog: &Catalog, recipe: RecipeId, year: i32) -> bool {
    let r = catalog.recipes.get(recipe);
    let known = |t: Option<crate::ids::TechnologyId>| {
        t.is_none_or(|t| catalog.technologies.get(t).invention_year <= year)
    };
    known(r.technology) && known(catalog.facilities.get(r.facility).technology)
}

/// Cost of one unit at reference prices, the reference wage and an electricity price,
/// including the facility's depreciation (for choosing among recipes).
pub(crate) fn reference_unit_cost(catalog: &Catalog, recipe: RecipeId, energy_usd_mwh: f64) -> f64 {
    let r = catalog.recipes.get(recipe);
    let f = catalog.facilities.get(r.facility);
    let inputs: f64 = r
        .inputs
        .iter()
        .map(|&(p, q)| q * catalog.products.get(p).reference_price.to_usd())
        .sum();
    let hours: f64 = r.labor_hours.iter().map(|&(_, h)| h).sum();
    let capital = f.investment.to_usd()
        / (f64::from(f.lifetime_years.max(1)) * 365.0 * f.runs_per_day.max(1e-9));
    let energy = r.energy_mwh * energy_usd_mwh;
    (inputs + hours * catalog.ai_model.start.reference_wage_usd + energy + capital)
        / r.output.max(1e-9)
}

/// The cheapest recipe for a product usable in `year`.
pub fn start_recipe(
    catalog: &Catalog,
    product: ProductId,
    year: i32,
    energy_usd_mwh: f64,
) -> Option<RecipeId> {
    catalog
        .recipes
        .iter()
        .filter(|&(id, r)| r.product == product && usable(catalog, id, year))
        .map(|(id, _)| (id, reference_unit_cost(catalog, id, energy_usd_mwh)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

/// Facilities of one recipe at one place.
#[derive(Clone, Debug)]
struct Placement {
    country: CountryId,
    deposit: Option<DepositId>,
    recipe: RecipeId,
    count: u32,
}

/// Plants planned for one company.
#[derive(Clone, Debug, Default)]
struct Plan {
    placements: Vec<Placement>,
}

impl Plan {
    fn value(&self, catalog: &Catalog) -> f64 {
        self.placements
            .iter()
            .map(|p| {
                let f = catalog.recipes.get(p.recipe).facility;
                catalog.facilities.get(f).investment.to_usd() * f64::from(p.count)
            })
            .sum()
    }

    /// Branch and country with the largest investment.
    fn main(&self, catalog: &Catalog) -> (BranchId, CountryId) {
        let mut by: BTreeMap<(BranchId, CountryId), f64> = BTreeMap::new();
        for p in &self.placements {
            let r = catalog.recipes.get(p.recipe);
            let value = catalog.facilities.get(r.facility).investment.to_usd() * f64::from(p.count);
            *by.entry((catalog.products.get(r.product).branch, p.country))
                .or_default() += value;
        }
        by.into_iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(k, _)| k)
            .unwrap_or((BranchId::from_index(0), CountryId::from_index(0)))
    }
}

/// Demand per day by product and country that the planned plants still have to cover.
struct Need {
    rate: Vec<Vec<f64>>,
}

impl Need {
    fn add_inputs(&mut self, catalog: &Catalog, p: &Placement, utilization: f64) {
        let r = catalog.recipes.get(p.recipe);
        let runs =
            catalog.facilities.get(r.facility).runs_per_day * f64::from(p.count) * utilization;
        for &(input, q) in &r.inputs {
            self.rate[input.index()][p.country.index()] += runs * q;
        }
    }

    fn cover(&mut self, catalog: &Catalog, p: &Placement, utilization: f64) {
        let product = catalog.recipes.get(p.recipe).product;
        let out = output_per_day(catalog, p.recipe) * f64::from(p.count) * utilization;
        let n = &mut self.rate[product.index()][p.country.index()];
        *n = (*n - out).max(0.0);
    }
}

/// Products in planning order: every product after all products whose recipes use it.
fn planning_order(catalog: &Catalog, recipes: &[Option<RecipeId>]) -> Vec<ProductId> {
    let mut level = vec![0usize; catalog.products.len()];
    for _ in 0..catalog.products.len() {
        let mut changed = false;
        for (product, recipe) in recipes.iter().enumerate() {
            let Some(r) = recipe else { continue };
            for &(input, _) in &catalog.recipes.get(*r).inputs {
                if level[input.index()] < level[product] + 1 {
                    level[input.index()] = level[product] + 1;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let mut order: Vec<ProductId> = catalog.products.ids().collect();
    order.sort_by_key(|p| (level[p.index()], p.index()));
    order
}

/// Splits `total` facilities over places by weight: whole numbers by the largest
/// remainder, at least one facility in all.
fn distribute(total: f64, weights: &[f64]) -> Vec<u32> {
    let sum: f64 = weights.iter().sum();
    if sum <= 0.0 || total <= 0.0 {
        return vec![0; weights.len()];
    }
    let target = total.round().max(1.0);
    let shares: Vec<f64> = weights.iter().map(|w| target * w / sum).collect();
    // Facility counts are small; the casts cannot overflow.
    let mut counts: Vec<u32> = shares.iter().map(|s| s.floor() as u32).collect();
    let missing = target as u32 - counts.iter().sum::<u32>();
    let mut rest: Vec<(usize, f64)> = shares
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s - s.floor()))
        .collect();
    rest.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    for &(i, _) in rest.iter().take(missing as usize) {
        counts[i] += 1;
    }
    counts
}

/// Places the AI companies of a new game (after the player's start setup).
pub(crate) fn populate(state: &mut GameState, catalog: &Catalog) {
    let wanted = state.settings.ai.companies;
    if wanted == 0 {
        return;
    }
    let year = state.settings.start_year;
    let start = &catalog.ai_model.start;
    let u = start.utilization;
    let energy = state
        .countries
        .values()
        .map(|c| c.electricity_price_usd_mwh)
        .sum::<f64>()
        / state.countries.len().max(1) as f64;
    let recipes: Vec<Option<RecipeId>> = catalog
        .products
        .ids()
        .map(|p| start_recipe(catalog, p, year, energy))
        .collect();
    let mut need = Need {
        rate: catalog
            .products
            .ids()
            .map(|p| {
                catalog
                    .countries
                    .ids()
                    .map(|c| {
                        let m = state.markets.get(p).get(c);
                        m.consumer_rate.iter().sum::<f64>() + m.state_rate
                    })
                    .collect()
            })
            .collect(),
    };
    let mut free: Vec<u32> = catalog
        .deposits
        .ids()
        .map(|d| {
            let n = state.deposits.get(d).concessions.iter();
            u32::try_from(n.filter(|c| c.site.is_none()).count()).unwrap_or(0)
        })
        .collect();

    // Historical companies, at most half of all, keep their own plants.
    let max_real = (wanted / 2) as usize;
    let mut real: Vec<(&RealCompany, Plan)> = Vec::new();
    for rc in catalog.real_companies.iter().filter(|c| c.founded <= year) {
        if real.len() >= max_real
            || state
                .companies
                .iter()
                .any(|c| c.name.eq_ignore_ascii_case(&rc.name))
        {
            continue;
        }
        let mut plan = Plan::default();
        for site in &rc.sites {
            if let Some(d) = site.deposit {
                if free[d.index()] == 0 {
                    continue;
                }
                free[d.index()] -= 1;
            }
            for &(facility, count, recipe) in &site.facilities {
                let recipe = recipe.or_else(|| first_recipe(catalog, facility, year, site.deposit));
                let Some(recipe) = recipe.filter(|&r| usable(catalog, r, year)) else {
                    continue;
                };
                // Plant counts are small; the cast cannot overflow.
                let count = (count * state.settings.market_scale).round().max(1.0) as u32;
                plan.placements.push(Placement {
                    country: site.country,
                    deposit: site.deposit,
                    recipe,
                    count,
                });
            }
        }
        for p in &plan.placements {
            need.cover(catalog, p, u);
            need.add_inputs(catalog, p, u);
        }
        real.push((rc, plan));
    }

    // Plants for the remaining demand, product by product.
    let mut planned: Vec<Placement> = Vec::new();
    for product in planning_order(catalog, &recipes) {
        let Some(recipe) = recipes[product.index()] else {
            continue;
        };
        let p = catalog.products.get(product);
        if p.kind == ProductKind::Energy {
            continue;
        }
        let per_day = output_per_day(catalog, recipe);
        let rates = &need.rate[product.index()];
        // Saturated markets (M16): at full capacity the established companies could
        // make `market_cover` times the demand. More spare capacity would start a
        // lasting price war, since idle plants lower their prices.
        let facilities: f64 =
            rates.iter().sum::<f64>() * start.market_cover[p.kind.index()] / per_day;
        if facilities < start.min_plant_share {
            continue;
        }
        let mut places: Vec<Placement> = Vec::new();
        if catalog.recipes.get(recipe).extraction {
            let deposits: Vec<DepositId> = catalog
                .deposits
                .iter()
                .filter(|(id, d)| {
                    d.resource == product
                        && d.discovered.is_none_or(|y| y <= year)
                        && free[id.index()] > 0
                })
                .map(|(id, _)| id)
                .collect();
            let capacity = |d: DepositId| {
                catalog.max_output(d, year) * state.settings.market_scale / 365.0 / per_day
            };
            let weights: Vec<f64> = deposits
                .iter()
                .map(|&d| capacity(d) / catalog.deposits.get(d).cost_factor)
                .collect();
            for (&d, n) in deposits.iter().zip(distribute(facilities, &weights)) {
                // Plant counts are small; the cast cannot overflow.
                let n = n.min(capacity(d).floor().max(1.0) as u32);
                let fields = free[d.index()].min(n);
                for i in 0..fields {
                    let count = n / fields + u32::from(i < n % fields);
                    free[d.index()] -= 1;
                    places.push(Placement {
                        country: catalog.deposits.get(d).country,
                        deposit: Some(d),
                        recipe,
                        count,
                    });
                }
            }
        } else {
            let exponent = start.development_weight[p.kind.index()];
            let weights: Vec<f64> = catalog
                .countries
                .ids()
                .map(|c| {
                    let dev = state.countries.get(c).development.max(0.01);
                    rates[c.index()] * crate::math::pow(dev, exponent)
                })
                .collect();
            for (c, n) in catalog
                .countries
                .ids()
                .zip(distribute(facilities, &weights))
            {
                if n > 0 {
                    places.push(Placement {
                        country: c,
                        deposit: None,
                        recipe,
                        count: n,
                    });
                }
            }
        }
        for place in places {
            need.add_inputs(catalog, &place, u);
            planned.push(place);
        }
    }

    let order = planning_order(catalog, &recipes);
    let real_placements: Vec<&Placement> = real.iter().flat_map(|(_, p)| &p.placements).collect();
    let planned = fit_to_inputs(catalog, &order, planned, &real_placements, u);
    let generated = usize::try_from(wanted).unwrap_or(usize::MAX) - real.len();
    let plans = group(catalog, planned, generated);
    let date = state.date;
    for (rc, plan) in real {
        let traits = (rc.competence, rc.aggressiveness);
        found_company(state, catalog, &plan, Some(rc), traits, date);
    }
    for plan in plans {
        found_company(state, catalog, &plan, None, (None, None), date);
    }
    stock_traders(state, catalog);
}

/// The trade network of the start year (M16): where a country lacks what its plants and
/// consumers need, the traders already hold the gap for `trader_cover_days` plus the
/// days at sea in stock and keep importing it. Without this, plants far from their suppliers stand still for
/// weeks until the first shipments arrive.
fn stock_traders(state: &mut GameState, catalog: &Catalog) {
    let n = catalog.countries.len();
    let mut supply: Vec<Vec<f64>> = vec![vec![0.0; n]; catalog.products.len()];
    let mut need: Vec<Vec<f64>> = vec![vec![0.0; n]; catalog.products.len()];
    for s in &state.sites {
        for sl in &s.slots {
            let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r)) else {
                continue;
            };
            let runs = sl.full_runs(catalog) * sl.utilization;
            supply[r.product.index()][s.country.index()] += runs * r.output;
            for &(p, q) in &r.inputs {
                need[p.index()][s.country.index()] += runs * q;
            }
        }
    }
    let model = &catalog.market_model;
    for (product, p) in catalog.products.iter() {
        if p.kind == ProductKind::Energy || p.state_market.is_some() {
            continue;
        }
        let gaps: Vec<f64> = catalog
            .countries
            .ids()
            .map(|c| {
                let m = state.markets.get(product).get(c);
                need[product.index()][c.index()]
                    + m.consumer_rate.iter().sum::<f64>()
                    + m.state_rate
                    - supply[product.index()][c.index()]
            })
            .collect();
        let exporters: Vec<CountryId> = catalog
            .countries
            .ids()
            .filter(|c| gaps[c.index()] < -1e-9)
            .collect();
        for country in catalog.countries.ids() {
            let gap = gaps[country.index()];
            if gap <= 1e-9 {
                continue;
            }
            // Imports cost what the cheapest exporting country asks plus transport, so
            // that the traders keep buying when the first stock is sold.
            let local = crate::market::local_reference(catalog, state, country, product);
            let (cost, days) = exporters
                .iter()
                .filter_map(|&from| {
                    let (transport, days) =
                        state.routes.for_product(catalog, product, from, country)?;
                    let cost =
                        crate::market::local_reference(catalog, state, from, product) + transport;
                    Some((cost, days))
                })
                .min()
                .unwrap_or((local, 0));
            let price = local.max(cost.scale(1.0 + model.trader_margin));
            // Enough until the first new shipments arrive, then the usual cover.
            let quantity = gap * (model.trader_cover_days + f64::from(days));
            let m = state.markets.get_mut(product).get_mut(country);
            m.imports.add(quantity, Money::times(cost, quantity), 50.0);
            m.import_price = price;
            m.price = m.price.max(price);
            m.open_demand = gap;
            state.import_markets.insert((product, country));
        }
    }
}

/// Scales the planned plants down to what their inputs allow, from the raw materials
/// upwards: deposits limit the raw materials, and a plant that would lack inputs would
/// only make losses. Historical plants stay as they are.
fn fit_to_inputs(
    catalog: &Catalog,
    order: &[ProductId],
    mut planned: Vec<Placement>,
    real: &[&Placement],
    u: f64,
) -> Vec<Placement> {
    let n = catalog.products.len();
    // Supply at full output against the use at the start utilization times the market
    // cover: every input keeps the spare capacity the markets start with (M16). Raw
    // materials limited by their deposits would otherwise start short, and the spare
    // plants after them would bid their price up to the limit.
    let cover =
        |p: ProductId| catalog.ai_model.start.market_cover[catalog.products.get(p).kind.index()];
    let flows = |placements: &mut dyn Iterator<Item = &Placement>| {
        let mut supply = vec![0.0; n];
        let mut used = vec![0.0; n];
        for p in placements {
            let r = catalog.recipes.get(p.recipe);
            let full = catalog.facilities.get(r.facility).runs_per_day * f64::from(p.count);
            supply[r.product.index()] += full * r.output;
            for &(i, q) in &r.inputs {
                used[i.index()] += full * u * q * cover(i);
            }
        }
        (supply, used)
    };
    for &product in order.iter().rev() {
        let (supply, used) = flows(&mut planned.iter().chain(real.iter().copied()));
        let Some(first) = planned
            .iter()
            .find(|p| catalog.recipes.get(p.recipe).product == product)
        else {
            continue;
        };
        let r = catalog.recipes.get(first.recipe);
        let ratio = r
            .inputs
            .iter()
            .filter(|(i, _)| {
                let p = catalog.products.get(*i);
                p.state_market.is_none() && p.kind != ProductKind::Energy
            })
            .map(|(i, _)| {
                let u = used[i.index()];
                if u > 1e-9 {
                    (supply[i.index()] / u).min(1.0)
                } else {
                    1.0
                }
            })
            .fold(1.0, f64::min);
        if ratio >= 0.95 {
            continue;
        }
        for p in planned
            .iter_mut()
            .filter(|p| catalog.recipes.get(p.recipe).product == product)
        {
            // Plant counts are small; the cast cannot overflow.
            p.count = (f64::from(p.count) * ratio).round() as u32;
        }
        planned.retain(|p| p.count > 0);
    }
    planned
}

/// The first recipe usable on a facility (for historical plants without a recipe).
fn first_recipe(
    catalog: &Catalog,
    facility: FacilityId,
    year: i32,
    deposit: Option<DepositId>,
) -> Option<RecipeId> {
    catalog
        .recipes
        .iter()
        .find(|&(id, r)| {
            r.facility == facility
                && usable(catalog, id, year)
                && (!r.extraction
                    || deposit.is_some_and(|d| catalog.deposits.get(d).resource == r.product))
        })
        .map(|(id, _)| id)
}

/// Groups plants into `companies` plans: one per branch and country, then the smallest
/// merged into the smallest of the same branch, or the largest split.
fn group(catalog: &Catalog, planned: Vec<Placement>, companies: usize) -> Vec<Plan> {
    if companies == 0 {
        return Vec::new();
    }
    let mut by_key: BTreeMap<(BranchId, CountryId), Plan> = BTreeMap::new();
    for p in planned {
        let branch = catalog
            .products
            .get(catalog.recipes.get(p.recipe).product)
            .branch;
        by_key
            .entry((branch, p.country))
            .or_default()
            .placements
            .push(p);
    }
    let mut plans: Vec<((BranchId, CountryId), Plan)> = by_key.into_iter().collect();
    while plans.len() > companies {
        let smallest = (0..plans.len())
            .min_by(|&a, &b| {
                plans[a]
                    .1
                    .value(catalog)
                    .total_cmp(&plans[b].1.value(catalog))
            })
            .expect("not empty");
        let (key, plan) = plans.remove(smallest);
        let target = (0..plans.len())
            .filter(|&i| plans[i].0.0 == key.0)
            .chain(0..plans.len())
            .min_by(|&a, &b| {
                let same = |i: usize| plans[i].0.0 != key.0;
                same(a).cmp(&same(b)).then(
                    plans[a]
                        .1
                        .value(catalog)
                        .total_cmp(&plans[b].1.value(catalog)),
                )
            })
            .expect("at least one plan left");
        plans[target].1.placements.extend(plan.placements);
    }
    while plans.len() < companies {
        let largest = (0..plans.len())
            .filter(|&i| splittable(&plans[i].1))
            .max_by(|&a, &b| {
                plans[a]
                    .1
                    .value(catalog)
                    .total_cmp(&plans[b].1.value(catalog))
            });
        let Some(largest) = largest else {
            break;
        };
        let key = plans[largest].0;
        let half = split(catalog, &mut plans[largest].1);
        plans.push((key, half));
    }
    plans.into_iter().map(|(_, p)| p).collect()
}

fn splittable(plan: &Plan) -> bool {
    plan.placements.len() > 1
        || plan
            .placements
            .iter()
            .any(|p| p.count > 1 && p.deposit.is_none())
}

/// Takes about half the investment out of a plan.
fn split(catalog: &Catalog, plan: &mut Plan) -> Plan {
    let half = plan.value(catalog) / 2.0;
    let mut taken = Plan::default();
    if plan.placements.len() > 1 {
        plan.placements.sort_by(|a, b| {
            let v = |p: &Placement| {
                let f = catalog.recipes.get(p.recipe).facility;
                catalog.facilities.get(f).investment.to_usd() * f64::from(p.count)
            };
            v(b).total_cmp(&v(a))
        });
        while plan.placements.len() > 1 && taken.value(catalog) < half {
            let p = plan.placements.pop().expect("more than one");
            taken.placements.push(p);
            if taken.value(catalog) >= half {
                break;
            }
        }
        if !taken.placements.is_empty() {
            return taken;
        }
    }
    let p = plan
        .placements
        .iter_mut()
        .find(|p| p.count > 1 && p.deposit.is_none())
        .expect("splittable");
    let moved = p.count / 2;
    p.count -= moved;
    taken.placements.push(Placement {
        count: moved,
        ..p.clone()
    });
    taken
}

/// Daily input and labor cost and output of a slot at a utilization.
pub(crate) struct SlotFlows {
    pub product: ProductId,
    pub output: f64,
    pub inputs: Vec<(ProductId, f64)>,
    pub cost_per_day: Money,
    /// Cost per day without depreciation and maintenance (inputs, labor, energy).
    pub variable_per_day: Money,
}

pub(crate) fn slot_flows(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    recipe: RecipeId,
    (count, size): (u32, crate::catalog::FacilitySize),
    utilization: f64,
    (wage_factor, developed): (f64, crate::development::Effect),
) -> SlotFlows {
    let r = catalog.recipes.get(recipe);
    let f = catalog.facilities.get(r.facility);
    let sizes = &catalog.production_model.sizes;
    let runs = f.runs_per_day * f64::from(count) * sizes.capacity(size) * utilization;
    let c = state.countries.get(country);
    let inputs: Vec<(ProductId, f64)> = r
        .inputs
        .iter()
        .map(|&(p, q)| (p, q * developed.inputs * runs))
        .collect();
    let input_cost: f64 = inputs
        .iter()
        .map(|&(p, q)| q * market::market_price(catalog, state, country, p).to_usd())
        .sum();
    let labor: f64 = r
        .labor_hours
        .iter()
        .map(|&(g, h)| h * runs * c.hourly_wage_usd.get(g.index()).copied().unwrap_or(0.0))
        .sum::<f64>()
        * wage_factor
        * developed.labor
        * sizes.labor(size)
        / c.labor_productivity.max(1e-9);
    let energy = r.energy_mwh * runs * c.electricity_price_usd_mwh;
    let conversion =
        labor + energy + crate::production::capital_per_run_usd(catalog, r, size) * runs;
    let overhead = crate::production::overhead_usd(catalog, r, conversion)
        + crate::production::rent_per_run_usd(catalog, state, country, r) * runs;
    let capital = f.investment.to_usd()
        * sizes.investment(size)
        * f64::from(count)
        * (1.0 / f64::from(f.lifetime_years.max(1)) + f.maintenance_share)
        / 365.0;
    let variable = input_cost + labor + energy + overhead;
    SlotFlows {
        product: r.product,
        output: runs * r.output,
        inputs,
        cost_per_day: Money::from_usd(variable + capital).unwrap_or(Money::ZERO),
        variable_per_day: Money::from_usd(variable).unwrap_or(Money::ZERO),
    }
}

fn found_company(
    state: &mut GameState,
    catalog: &Catalog,
    plan: &Plan,
    real: Option<&RealCompany>,
    traits: (Option<f64>, Option<f64>),
    date: Date,
) {
    if plan.placements.is_empty() {
        return;
    }
    let model = &catalog.ai_model;
    let start = &model.start;
    let index = u32::try_from(state.companies.len()).unwrap_or(u32::MAX);
    let id = CompanyId(index);
    let mut rng = SimRng::for_stream(state.settings.seed, Stream::Company(index));
    let mut vary = |base: f64, fixed: Option<f64>| {
        fixed.unwrap_or_else(|| {
            (base + model.trait_spread * (2.0 * rng.next_f64() - 1.0)).clamp(0.0, 1.0)
        })
    };
    let settings = state.settings.ai;
    let competence = vary(settings.competence, traits.0);
    let aggressiveness = vary(settings.aggressiveness, traits.1);
    let (branch, main_country) = plan.main(catalog);
    let headquarters = real.map_or(main_country, |r| r.headquarters);
    let name = match real {
        Some(r) => r.name.clone(),
        None => company_name(state, catalog, &mut rng, headquarters, branch),
    };

    // Sites: one per country and site type, one per deposit concession.
    let mut sites: Vec<(CountryId, SiteType, Option<DepositId>, Vec<Placement>)> = Vec::new();
    let mut power_plants: Vec<(usize, RecipeId, u32)> = Vec::new();
    for p in &plan.placements {
        let kind = catalog
            .facilities
            .get(catalog.recipes.get(p.recipe).facility)
            .site_type;
        match sites
            .iter_mut()
            .find(|s| s.0 == p.country && s.1 == kind && s.2 == p.deposit && p.deposit.is_none())
        {
            Some(s) => s.3.push(p.clone()),
            None => sites.push((p.country, kind, p.deposit, vec![p.clone()])),
        }
    }
    // Electricity the grid cannot deliver comes from an own power plant in the country.
    if let Some(power) = catalog
        .production_model
        .electricity
        .and_then(|e| start_recipe(catalog, e, date.year(), 0.0))
    {
        let mut by_country: BTreeMap<CountryId, f64> = BTreeMap::new();
        for p in &plan.placements {
            let r = catalog.recipes.get(p.recipe);
            let runs = catalog.facilities.get(r.facility).runs_per_day
                * f64::from(p.count)
                * start.utilization;
            let grid = state.countries.get(p.country).grid_share;
            *by_country.entry(p.country).or_default() += r.energy_mwh * runs * (1.0 - grid);
        }
        let per_plant = output_per_day(catalog, power) * start.utilization;
        let kind = catalog
            .facilities
            .get(catalog.recipes.get(power).facility)
            .site_type;
        for (country, mwh) in by_country {
            if mwh > 1e-9 {
                // Plant counts are small; the cast cannot overflow.
                let count = (mwh / per_plant).ceil().max(1.0) as u32;
                sites.push((country, kind, None, Vec::new()));
                power_plants.push((sites.len() - 1, power, count));
            }
        }
    }
    let floor_factor = model.behavior.floor_factor.at(aggressiveness);
    let mut fixed = Money::ZERO;
    // Facilities and deposits are completed on the first day (`complete_constructions`).
    let mut construction = Money::ZERO;
    let mut stock_value = Money::ZERO;
    let mut daily_cost = Money::ZERO;
    let mut new_sites = Vec::new();
    for (index, recipe, count) in power_plants {
        let country = sites[index].0;
        sites[index].3.push(Placement {
            country,
            deposit: None,
            recipe,
            count,
        });
    }
    for (country, kind, deposit, placements) in sites {
        let site_id =
            SiteId(u32::try_from(state.sites.len() + new_sites.len()).unwrap_or(u32::MAX));
        let building = catalog.production_model.site_cost(kind);
        fixed += building;
        let mut slots = Vec::new();
        let mut produced: BTreeMap<ProductId, (f64, Money, Money)> = BTreeMap::new();
        let mut used: BTreeMap<ProductId, f64> = BTreeMap::new();
        for p in &placements {
            let r = catalog.recipes.get(p.recipe);
            let cost = catalog
                .facilities
                .get(r.facility)
                .investment
                .scale(f64::from(p.count));
            construction += cost;
            slots.push(Slot {
                facility: r.facility,
                ready: date,
                count: p.count,
                cost,
                recipe: Some(p.recipe),
                utilization: start.utilization,
                automation: 0.0,
                condition: 1.0,
                batches: Vec::new(),
                last_runs: 0.0,
                limit: None,
                operation: crate::state::Operation::Running,
                size: crate::catalog::FacilitySize::Medium,
            });
            let flows = slot_flows(
                catalog,
                state,
                country,
                p.recipe,
                (p.count, crate::catalog::FacilitySize::Medium),
                start.utilization,
                (1.0, crate::development::Effect::NONE),
            );
            daily_cost += flows.cost_per_day;
            let e = produced
                .entry(flows.product)
                .or_insert((0.0, Money::ZERO, Money::ZERO));
            e.0 += flows.output;
            e.1 += flows.cost_per_day;
            e.2 += flows.variable_per_day;
            for (input, q) in flows.inputs {
                *used.entry(input).or_default() += q;
            }
        }
        if let Some(d) = deposit {
            let ds = state.deposits.get_mut(d);
            if let Some(field) = ds.concessions.iter_mut().find(|c| c.site.is_none()) {
                let cost = catalog
                    .deposits
                    .get(d)
                    .development_cost
                    .scale(state.settings.market_scale * field.share);
                field.site = Some(site_id);
                field.ready = Some(date);
                field.development_cost = cost;
                construction += cost;
            }
        }
        let mut inventory = BTreeMap::new();
        let mut offers = BTreeMap::new();
        let mut orders = BTreeMap::new();
        for (&product, &(output, cost, variable)) in &produced {
            if catalog.products.get(product).kind == ProductKind::Energy {
                continue;
            }
            let unit = if output > 0.0 {
                cost.scale(1.0 / output)
            } else {
                Money::ZERO
            };
            let qty = output * start.output_stock_days;
            let value = Money::times(unit, qty);
            stock_value += value;
            inventory
                .entry(product)
                .or_insert_with(Stock::default)
                .add(qty, value, 50.0);
            let own_use = used.get(&product).copied().unwrap_or(0.0);
            let keep = own_use * start.input_stock_days;
            offers.insert(
                product,
                SaleOffer {
                    mode: PriceMode::Market {
                        markup: 0.0,
                        floor: if output > 0.0 {
                            variable.scale(floor_factor / output)
                        } else {
                            Money::ZERO
                        },
                    },
                    price: market::market_price(catalog, state, country, product),
                    keep,
                    sold_today: 0.0,
                    sold_month: 0.0,
                    // The start is an equilibrium: last month the site sold what it makes.
                    // Without this the AI would judge its sales by the first days, while
                    // markets and traders start, and throttle the whole chain at once.
                    sold_last_month: (output - own_use).max(0.0) * 30.0,
                    to_traders_month: 0.0,
                    to_companies_month: 0.0,
                },
            );
        }
        for (&product, &per_day) in &used {
            let price = market::market_price(catalog, state, country, product);
            let target = per_day * start.input_stock_days;
            let value = Money::times(price, target);
            stock_value += value;
            inventory
                .entry(product)
                .or_insert_with(Stock::default)
                .add(target, value, 50.0);
            orders.insert(
                product,
                PurchaseOrder {
                    target,
                    max_price: price.scale(1.0 + model.behavior.purchase_markup),
                    min_quality: 0.0,
                    bought_month: 0.0,
                    bought_last_month: 0.0,
                },
            );
        }
        new_sites.push(Site {
            owner: id,
            country,
            kind,
            founded: date,
            building_cost: building,
            deposit,
            slots,
            inventory,
            workforce: PerId::from_fn(catalog.labor_groups.len(), |_| 0.0),
            staffing_due: true,
            offers,
            orders,
            research: None,
            development: None,
            wage_premium: 0.0,
            acquired: None,
            goodwill: None,
            plot: None,
        });
    }
    let cash = daily_cost.scale(30.0 * start.cash_months);
    // Start companies own their plots (M35): a free one that holds the site with room to
    // grow, else one of that size; the land is part of the start capital.
    let mut land = Money::ZERO;
    let reserve = 1.0 + catalog.plot_model.ai_reserve;
    for (i, site) in new_sites.iter_mut().enumerate() {
        if !crate::plots::needs_plot(catalog, site.kind) {
            continue;
        }
        let need = crate::plots::site_area(catalog, site, None) * reserve;
        let revenue = crate::plots::planned_revenue(state, catalog, site);
        let plot = crate::plots::for_existing(state, catalog, site.country, (need, revenue));
        let value = crate::plots::value(catalog, state, plot);
        land += value;
        let p = &mut state.plots[plot.index()];
        p.site = Some(SiteId(
            u32::try_from(state.sites.len() + i).unwrap_or(u32::MAX),
        ));
        p.tenure = crate::state::Tenure::Owned(value);
        site.plot = Some(plot);
    }
    let mut ledger = Ledger::new(date, fixed + construction + stock_value + cash + land);
    ledger.transfer(Account::FixedAssets, Account::Cash, fixed);
    ledger.transfer(Account::Land, Account::Cash, land);
    ledger.transfer(
        Account::AssetsUnderConstruction,
        Account::Cash,
        construction,
    );
    ledger.transfer(Account::Inventory, Account::Cash, stock_value);
    let operations = model.behavior.operations_days.at(competence);
    // A random first day spreads the decisions of the companies over the period.
    let first = (rng.next_f64() * operations).floor();
    // Established companies are known where they sell their end products (M16).
    let brand_model = &catalog.market_model.brand;
    let start_awareness = if real.is_some() {
        brand_model.start_awareness_real
    } else {
        brand_model.start_awareness
    };
    let mut brands: Vec<crate::state::Brand> = Vec::new();
    for p in &plan.placements {
        let product = catalog.products.get(catalog.recipes.get(p.recipe).product);
        if product.kind == ProductKind::EndProduct
            && !brands
                .iter()
                .any(|b| b.country == p.country && b.group == product.goods_group)
        {
            brands.push(crate::state::Brand {
                country: p.country,
                group: product.goods_group,
                awareness: start_awareness,
            });
        }
    }
    brands.sort_by_key(|b| (b.country, b.group));
    state.companies.push(Company {
        brands,
        advertising: Vec::new(),
        auction_until: None,
        development: Default::default(),
        owners: crate::state::Stake::sole(crate::state::Holder::Private),
        name,
        kind: CompanyKind::Ai,
        headquarters,
        founded: real.map_or(date, |r| Date::first_of_year(r.founded)),
        rng,
        ledger,
        technologies: Default::default(),
        bankrupt: false,
        loans: Vec::new(),
        loss_carryforward: Money::ZERO,
        sales_policies: Vec::new(),
        research: Default::default(),
        ai: Some(AiState {
            competence,
            aggressiveness,
            real: real.map(|r| r.key.clone()),
            // Below a year of days; the cast cannot overflow.
            next_operations: date.add_days(first as i32 + 1),
        }),
    });
    state.sites.extend(new_sites);
}

/// A generated name from the name group of the country, unique among all companies and
/// the historical ones.
pub(crate) fn company_name(
    state: &GameState,
    catalog: &Catalog,
    rng: &mut SimRng,
    country: CountryId,
    branch: BranchId,
) -> String {
    let groups = &catalog.name_groups;
    let Some(group) = groups
        .iter()
        .find(|g| g.countries.contains(&country))
        .or_else(|| groups.iter().find(|g| g.is_default))
        .or_else(|| groups.first())
    else {
        return format!("KI {}", state.companies.len());
    };
    let taken = |name: &str| {
        state
            .companies
            .iter()
            .any(|c| c.name.eq_ignore_ascii_case(name))
            || catalog
                .real_companies
                .iter()
                .any(|r| r.name.eq_ignore_ascii_case(name))
    };
    let mut pick = |list: &[String]| -> String {
        if list.is_empty() {
            return String::new();
        }
        let n = u64::try_from(list.len()).unwrap_or(1);
        list[usize::try_from(rng.below(n)).unwrap_or(0)].clone()
    };
    let word = group.branch_words.get(branch.index()).cloned().flatten();
    let mut name = String::new();
    for _ in 0..20 {
        let patterns: Vec<String> = group
            .patterns
            .iter()
            .filter(|p| word.is_some() || !p.contains("{branche}"))
            .cloned()
            .collect();
        let mut text = pick(if patterns.is_empty() {
            &group.patterns
        } else {
            &patterns
        });
        while text.contains("{familienname}") {
            let surname = pick(&group.surnames);
            text = text.replacen("{familienname}", &surname, 1);
        }
        text = text.replace("{ort}", &pick(&group.places));
        text = text.replace("{rechtsform}", &pick(&group.legal_forms));
        text = text.replace("{branche}", word.as_deref().unwrap_or(""));
        name = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if !taken(&name) {
            return name;
        }
    }
    let mut n = 2;
    loop {
        let numbered = format!("{name} {n}");
        if !taken(&numbered) {
            return numbered;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::distribute;

    #[test]
    fn distribute_keeps_the_total_and_follows_the_weights() {
        assert_eq!(distribute(10.0, &[1.0, 1.0, 2.0]), vec![3, 2, 5]);
        assert_eq!(distribute(0.4, &[1.0, 3.0]), vec![0, 1]);
        assert_eq!(distribute(5.0, &[0.0, 0.0]), vec![0, 0]);
        let counts = distribute(7.3, &[0.2, 0.5, 0.1, 0.2]);
        assert_eq!(counts.iter().sum::<u32>(), 7);
    }
}
