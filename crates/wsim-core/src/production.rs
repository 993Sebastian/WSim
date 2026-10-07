//! Daily production: construction, staffing, extraction and manufacturing, costs
//! (Lastenheft §5.2, §6.1–6.3; formulas in docs/FORMELN.md, section M5).

use crate::calendar::Date;
use crate::catalog::{Catalog, FacilitySize, Recipe, SiteType};
use crate::development;
use crate::ids::{CountryId, Id, LaborGroupId, ProductId, RecipeId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{Batch, CompanyId, GameState, Limit, Operation, Site, SiteId, Slot};

/// Working hours one employee provides per calendar day.
pub fn hours_per_worker_day(catalog: &Catalog, date: Date) -> f64 {
    catalog
        .country_model
        .annual_hours
        .value_at(f64::from(date.year()))
        / 365.0
}

/// Factor on the labor hours of a recipe for a degree of automation.
pub fn labor_factor(catalog: &Catalog, automation: f64, affinity: f64) -> f64 {
    1.0 - automation * catalog.production_model.automation_labor_saving * (0.5 + 0.5 * affinity)
}

/// Administration, sales and logistics in USD (M16): a share of the conversion cost
/// (labor, electricity, plant) per product kind, like an overhead rate on manufacturing
/// costs. It falls with mass production and stays put when only market prices fall.
pub fn overhead_usd(catalog: &Catalog, recipe: &Recipe, conversion_usd: f64) -> f64 {
    let share = catalog
        .production_model
        .overhead_share(catalog.products.get(recipe.product).kind);
    share * conversion_usd.max(0.0)
}

/// Land rent and royalties per run in USD (M16): a share of the main product's reference
/// price in the country for each unit extracted. Without them a raw material that is not
/// scarce fell to its bare extraction cost, a third of its price, and pulled the whole
/// chain below its reference prices.
pub fn rent_per_run_usd(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    recipe: &Recipe,
) -> f64 {
    let p = catalog.products.get(recipe.product);
    if p.rent_share <= 0.0 {
        return 0.0;
    }
    p.rent_share
        * recipe.output
        * crate::market::local_reference(catalog, state, country, recipe.product).to_usd()
}

/// Depreciation and maintenance of a recipe's facility of a size per run at normal
/// utilization (M36: the investment per capacity falls with the size).
pub fn capital_per_run_usd(catalog: &Catalog, recipe: &Recipe, size: FacilitySize) -> f64 {
    let f = catalog.facilities.get(recipe.facility);
    let sizes = &catalog.production_model.sizes;
    f.investment.to_usd()
        * (1.0 / f64::from(f.lifetime_years.max(1)) + f.maintenance_share)
        * sizes.investment(size)
        / (365.0
            * f.runs_per_day.max(1e-9)
            * sizes.capacity(size)
            * catalog.market_model.normal_utilization.max(1e-9))
}

/// Labor hours per run for each group, including automation, deposit difficulty, the
/// country's labor productivity and the size of the facility (M36).
fn hours_per_run(
    catalog: &Catalog,
    (recipe, size): (&Recipe, FacilitySize),
    automation: f64,
    (affinity, productivity): (f64, f64),
    cost_factor: f64,
) -> Vec<(LaborGroupId, f64)> {
    let factor = labor_factor(catalog, automation, affinity)
        * cost_factor
        * catalog.production_model.sizes.labor(size)
        / productivity.max(1e-9);
    recipe
        .labor_hours
        .iter()
        .map(|&(g, h)| (g, h * factor))
        .collect()
}

/// Allowed yearly output of a deposit (before the market scale) with its decline (C2,
/// docs/FORMELN.md): below `foerderkurve_ab` of the reserve left, it falls in proportion
/// to what is left. Renewable deposits keep their full output.
pub fn deposit_output(
    catalog: &Catalog,
    state: &GameState,
    deposit: crate::ids::DepositId,
    year: i32,
) -> f64 {
    let full = catalog.max_output(deposit, year);
    let from = catalog.production_model.decline_from;
    let Some(reserve) = catalog
        .reserve(deposit, year)
        .filter(|&r| r > 0.0 && from > 0.0)
    else {
        return full;
    };
    let scale = state.settings.market_scale.max(1e-12);
    let left = (reserve - state.deposits.get(deposit).extracted / scale).max(0.0) / reserve;
    full * (left / from).min(1.0)
}

/// One simulated day for all sites. `date` is the day being simulated. Power plants
/// run first so that factories can use the electricity of the same day.
pub(crate) fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) {
    complete_constructions(state, catalog, date);
    staff_sites(state, catalog, date);
    for power_plants in [true, false] {
        for index in 0..state.sites.len() {
            if (state.sites[index].kind == SiteType::PowerPlant) != power_plants {
                continue;
            }
            let site = SiteId(u32::try_from(index).expect("site count fits u32"));
            if state
                .company(state.sites[index].owner)
                .is_some_and(|c| c.bankrupt)
            {
                continue;
            }
            produce(state, catalog, site, date);
            finish_batches(state, catalog, site, date);
            running_costs(state, catalog, site, date);
        }
    }
    feed_in(state, catalog);
}

/// Own electricity of a company in a country (all its sites there).
fn own_electricity(
    catalog: &Catalog,
    state: &GameState,
    owner: CompanyId,
    country: CountryId,
) -> f64 {
    let Some(power) = catalog.production_model.electricity else {
        return 0.0;
    };
    state
        .sites
        .iter()
        .filter(|s| s.owner == owner && s.country == country)
        .filter_map(|s| s.inventory.get(&power))
        .map(|s| s.quantity)
        .sum()
}

/// Takes own electricity from the company's sites in a country; returns its value.
fn use_own_electricity(
    catalog: &Catalog,
    state: &mut GameState,
    owner: CompanyId,
    country: CountryId,
    mut mwh: f64,
) -> Money {
    let Some(power) = catalog.production_model.electricity else {
        return Money::ZERO;
    };
    let mut value = Money::ZERO;
    for s in &mut state.sites {
        if mwh <= 0.0 {
            break;
        }
        if s.owner != owner || s.country != country {
            continue;
        }
        if let Some(stock) = s.inventory.get_mut(&power) {
            let taken = mwh.min(stock.quantity);
            value += stock.take(taken);
            mwh -= taken;
        }
    }
    value
}

/// Electricity cannot be stored: what is left at the end of the day goes into the
/// public grid at a share of the industrial price.
fn feed_in(state: &mut GameState, catalog: &Catalog) {
    let Some(power) = catalog.production_model.electricity else {
        return;
    };
    for index in 0..state.sites.len() {
        let s = &mut state.sites[index];
        let Some(stock) = s.inventory.get_mut(&power) else {
            continue;
        };
        if stock.quantity <= 0.0 {
            continue;
        }
        let quantity = stock.quantity;
        let value = stock.take(quantity);
        let price = state.countries.get(s.country).electricity_price_usd_mwh
            * catalog.production_model.feed_in_share;
        let revenue = Money::from_usd(quantity * price).unwrap_or(Money::ZERO);
        let site = SiteId(u32::try_from(index).expect("site count fits u32"));
        let ledger = &mut state.companies[s.owner.index()].ledger;
        let center = CostCenter::product(site, power);
        ledger.income(CostType::Revenue, center, Account::Cash, revenue);
        ledger.expense(CostType::InventoryChange, center, Account::Inventory, value);
    }
}

fn complete_constructions(state: &mut GameState, catalog: &Catalog, date: Date) {
    // Facilities started up again work from today on (M22).
    for s in &mut state.sites {
        for sl in &mut s.slots {
            if matches!(sl.operation, Operation::Restarting { until } if until <= date) {
                sl.operation = Operation::Running;
                s.staffing_due = true;
            }
        }
    }
    for site in 0..state.sites.len() {
        let owner = state.sites[site].owner;
        let finished: Money = state.sites[site]
            .slots
            .iter()
            .filter(|s| s.ready == date)
            .map(|s| s.investment(catalog))
            .sum();
        if finished != Money::ZERO {
            state.sites[site].staffing_due = true;
            let ledger = &mut state.companies[owner.index()].ledger;
            ledger.transfer(
                Account::FixedAssets,
                Account::AssetsUnderConstruction,
                finished,
            );
        }
    }
    for (_, deposit) in state.deposits.iter_mut() {
        for c in &deposit.concessions {
            if c.ready == Some(date)
                && let Some(site) = c.site
            {
                let owner = state.sites[site.index()].owner;
                let ledger = &mut state.companies[owner.index()].ledger;
                ledger.transfer(
                    Account::FixedAssets,
                    Account::AssetsUnderConstruction,
                    c.development_cost,
                );
                state.sites[site.index()].staffing_due = true;
            }
        }
    }
}

/// Planned runs per day of a slot (0 while under construction or idle).
fn planned_runs(
    catalog: &Catalog,
    state: &GameState,
    site: SiteId,
    slot: usize,
    date: Date,
) -> Option<(RecipeId, f64)> {
    let s = &state.sites[site.index()];
    let sl = &s.slots[slot];
    let recipe = sl.recipe?;
    if !sl.operating(date) || sl.utilization <= 0.0 {
        return None;
    }
    if catalog.recipes.get(recipe).extraction {
        let deposit = s.deposit?;
        let field = state.deposits.get(deposit).concession_of(site)?;
        if field.ready.is_none_or(|r| r > date) {
            return None;
        }
    }
    Some((recipe, sl.full_runs(catalog) * sl.utilization))
}

fn deposit_cost_factor(catalog: &Catalog, state: &GameState, site: SiteId, recipe: &Recipe) -> f64 {
    let s = &state.sites[site.index()];
    match (recipe.extraction, s.deposit) {
        (true, Some(d)) => catalog.deposits.get(d).cost_factor,
        _ => 1.0,
    }
}

/// Expected cost of one unit of a product made at a site, by kind of cost (USD).
#[derive(Clone, Debug, PartialEq)]
pub struct UnitCost {
    pub product: ProductId,
    /// Output per day at the planned utilization (idle facilities at full utilization).
    pub output_per_day: f64,
    pub material: f64,
    pub labor: f64,
    pub energy: f64,
    pub overhead: f64,
    pub rent: f64,
    /// Depreciation and maintenance of the facilities.
    pub facility: f64,
}

impl UnitCost {
    pub fn total(&self) -> f64 {
        self.variable() + self.facility
    }

    /// What one more unit costs: everything but the facilities.
    pub fn variable(&self) -> f64 {
        self.material + self.labor + self.energy + self.overhead + self.rent
    }

    fn add(&mut self, other: &UnitCost) {
        self.output_per_day += other.output_per_day;
        self.material += other.material;
        self.labor += other.labor;
        self.energy += other.energy;
        self.overhead += other.overhead;
        self.rent += other.rent;
        self.facility += other.facility;
    }

    fn per_unit(mut self) -> Self {
        let q = self.output_per_day.max(1e-12);
        for v in [
            &mut self.material,
            &mut self.labor,
            &mut self.energy,
            &mut self.overhead,
            &mut self.rent,
            &mut self.facility,
        ] {
            *v /= q;
        }
        self
    }
}

/// Expected unit costs of the products a site makes (M18, docs/FORMELN.md): inputs at
/// the country's market price (those made at the site at their own unit cost), labor
/// with the site's wage premium, electricity from the grid, overhead and rent as in
/// production, and depreciation and maintenance of the facilities at the planned
/// utilization. The whole cost goes to the main product.
pub fn unit_costs(catalog: &Catalog, state: &GameState, site: SiteId) -> Vec<UnitCost> {
    let s = &state.sites[site.index()];
    let c = state.countries.get(s.country);
    let affinity = (c.automation_affinity, c.labor_productivity);
    let wage_factor = 1.0 + s.wage_premium;
    let made: Vec<ProductId> = s
        .slots
        .iter()
        .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r).product))
        .collect();
    let mut costs: Vec<UnitCost> = Vec::new();
    // Inputs made here are valued at the estimate of the previous pass; chains are at
    // most a few levels deep.
    for _ in 0..=made.len() {
        let price = |p: ProductId| {
            costs.iter().find(|u| u.product == p).map_or_else(
                || crate::market::market_price(catalog, state, s.country, p).to_usd(),
                UnitCost::total,
            )
        };
        let mut sums: Vec<UnitCost> = Vec::new();
        for sl in &s.slots {
            // Shut down facilities make nothing (M22).
            let Some(recipe) = sl
                .recipe
                .filter(|_| !sl.mothballed())
                .map(|r| catalog.recipes.get(r))
            else {
                continue;
            };
            let f = catalog.facilities.get(sl.facility);
            let utilization = if sl.utilization > 0.0 {
                sl.utilization
            } else {
                1.0
            };
            let runs = sl.full_runs(catalog) * utilization;
            let developed = development::effect(catalog, state, s.owner, recipe.product);
            let cost_factor = deposit_cost_factor(catalog, state, site, recipe)
                * developed.labor
                * crate::training::labor_factor(catalog, s.training);
            let labor: f64 = hours_per_run(
                catalog,
                (recipe, sl.size),
                sl.automation,
                affinity,
                cost_factor,
            )
            .iter()
            .map(|&(g, h)| h * c.hourly_wage_usd.get(g.index()).copied().unwrap_or(0.0))
            .sum::<f64>()
                * wage_factor;
            let energy = recipe.energy_mwh * c.electricity_price_usd_mwh;
            let material: f64 = recipe
                .inputs
                .iter()
                .map(|&(p, q)| q * developed.inputs * price(p))
                .sum();
            let overhead = overhead_usd(
                catalog,
                recipe,
                labor + energy + capital_per_run_usd(catalog, recipe, sl.size),
            );
            let rent = rent_per_run_usd(catalog, state, s.country, recipe);
            let facility = sl.cost.to_usd()
                * (1.0 / f64::from(f.lifetime_years.max(1)) + f.maintenance_share)
                / 365.0;
            let line = UnitCost {
                product: recipe.product,
                output_per_day: runs * recipe.output,
                material: material * runs,
                labor: labor * runs,
                energy: energy * runs,
                overhead: overhead * runs,
                rent: rent * runs,
                facility,
            };
            match sums.iter_mut().find(|u| u.product == recipe.product) {
                Some(u) => u.add(&line),
                None => sums.push(line),
            }
        }
        costs = sums.into_iter().map(UnitCost::per_unit).collect();
    }
    costs
}

/// Workers per labor group that a site's planned production (and research) needs.
pub fn needed_workers(catalog: &Catalog, state: &GameState, site: SiteId, date: Date) -> Vec<f64> {
    let index = site.index();
    let worker_hours = hours_per_worker_day(catalog, date);
    let c = state.countries.get(state.sites[index].country);
    let affinity = (c.automation_affinity, c.labor_productivity);
    let mut needed = vec![0.0; catalog.labor_groups.len()];
    for slot in 0..state.sites[index].slots.len() {
        let Some((recipe_id, runs)) = planned_runs(catalog, state, site, slot, date) else {
            continue;
        };
        let recipe = catalog.recipes.get(recipe_id);
        let sl = &state.sites[index].slots[slot];
        let owner = state.sites[index].owner;
        let cost_factor = deposit_cost_factor(catalog, state, site, recipe)
            * development::effect(catalog, state, owner, recipe.product).labor
            * crate::training::labor_factor(catalog, state.sites[index].training);
        for (g, h) in hours_per_run(
            catalog,
            (recipe, sl.size),
            sl.automation,
            affinity,
            cost_factor,
        ) {
            needed[g.index()] += runs * h / worker_hours;
        }
    }
    if let Some(field) = development::project_field(catalog, state, site)
        && let Some(group) = catalog
            .research_model
            .researchers
            .get(field.index())
            .copied()
            .flatten()
    {
        needed[group.index()] += crate::research::wanted_researchers(catalog, state, site, date);
    }
    needed
}

/// Hires and dismisses staff so that every site has the workers its planned production
/// needs, within the free labor pool of its country (Lastenheft §5.3). Sites with a
/// higher wage premium are served first; when the pool runs short they hire workers
/// away from sites in the same country that pay a lower premium (M18).
fn staff_sites(state: &mut GameState, catalog: &Catalog, date: Date) {
    if !state.sites.iter().any(|s| s.staffing_due) {
        return;
    }
    let groups = catalog.labor_groups.len();
    let mut employed: Vec<Vec<f64>> = vec![vec![0.0; groups]; catalog.countries.len()];
    for s in &state.sites {
        for (g, &w) in s.workforce.iter() {
            employed[s.country.index()][g.index()] += w;
        }
    }
    // What draws workers: the wage premium and the location of the plot (M35).
    let pull: Vec<f64> = state
        .sites
        .iter()
        .map(|s| s.wage_premium + crate::plots::hiring(catalog, state, s))
        .collect();
    let mut due: Vec<usize> = (0..state.sites.len())
        .filter(|&i| state.sites[i].staffing_due)
        .collect();
    due.sort_by(|&a, &b| pull[b].total_cmp(&pull[a]).then(a.cmp(&b)));
    for index in due {
        let site = SiteId(u32::try_from(index).expect("site count fits u32"));
        let country = state.sites[index].country;
        let premium = pull[index];
        let needed = needed_workers(catalog, state, site, date);
        // Who can be hired away: same country, less pull; least pull first, the youngest
        // site first among equals.
        let mut rivals: Vec<usize> = (0..state.sites.len())
            .filter(|&i| i != index && state.sites[i].country == country && pull[i] < premium)
            .collect();
        rivals.sort_by(|&a, &b| pull[a].total_cmp(&pull[b]).then(b.cmp(&a)));
        let pool = &state.countries.get(country).labor_available;
        let free: Vec<f64> = (0..groups)
            .map(|g| (pool.get(g).copied().unwrap_or(0.0) - employed[country.index()][g]).max(0.0))
            .collect();
        for g in 0..groups {
            let id = LaborGroupId::from_index(g);
            let current = *state.sites[index].workforce.get(id);
            let target = needed[g];
            if target <= current {
                employed[country.index()][g] -= current - target;
                *state.sites[index].workforce.get_mut(id) = target;
                continue;
            }
            let hired = (target - current).min(free[g]);
            employed[country.index()][g] += hired;
            let mut new = current + hired;
            for &r in &rivals {
                let missing = target - new;
                if missing <= 1e-9 {
                    break;
                }
                let theirs = state.sites[r].workforce.get_mut(id);
                let taken = missing.min(*theirs);
                if taken > 0.0 {
                    *theirs -= taken;
                    new += taken;
                    state.sites[r].staffing_due = true;
                }
            }
            *state.sites[index].workforce.get_mut(id) = new;
        }
        state.sites[index].staffing_due = false;
    }
}

fn produce(state: &mut GameState, catalog: &Catalog, site: SiteId, date: Date) {
    let index = site.index();
    let country = state.sites[index].country;
    let worker_hours = hours_per_worker_day(catalog, date);
    let (affinity, grid_share, electricity_price, wages) = {
        let c = state.countries.get(country);
        let factor = 1.0 + state.sites[index].wage_premium;
        (
            (c.automation_affinity, c.labor_productivity),
            c.grid_share,
            c.electricity_price_usd_mwh,
            c.hourly_wage_usd
                .iter()
                .map(|w| w * factor)
                .collect::<Vec<f64>>(),
        )
    };
    // Hours the site's staff can work today, shared by the facilities in order.
    let mut hours: Vec<f64> = state.sites[index]
        .workforce
        .values()
        .map(|w| w * worker_hours)
        .collect();
    let owner = state.sites[index].owner;
    let model = &catalog.production_model;

    for slot in 0..state.sites[index].slots.len() {
        state.sites[index].slots[slot].last_runs = 0.0;
        state.sites[index].slots[slot].limit = None;
        let Some((recipe_id, planned)) = planned_runs(catalog, state, site, slot, date) else {
            continue;
        };
        let recipe = catalog.recipes.get(recipe_id);
        let (automation, size) = {
            let sl = &state.sites[index].slots[slot];
            (sl.automation, sl.size)
        };
        // The company's development level of the product (M37).
        let developed = development::effect(catalog, state, owner, recipe.product);
        let cost_factor = deposit_cost_factor(catalog, state, site, recipe)
            * developed.labor
            * crate::training::labor_factor(catalog, state.sites[index].training);
        let per_run = hours_per_run(catalog, (recipe, size), automation, affinity, cost_factor);
        let rent_per_run = rent_per_run_usd(catalog, state, country, recipe);

        let mut runs = planned;
        let mut limit = None;
        // Rounding (e.g. workers hired for exactly the planned runs) is no bottleneck.
        let mut bound = |runs: &mut f64, cap: f64, why: Limit| {
            if cap < *runs {
                *runs = cap;
                if cap < planned * (1.0 - 1e-9) {
                    limit = Some(why);
                }
            }
        };
        for &(p, q) in &recipe.inputs {
            let available = state.sites[index]
                .inventory
                .get(&p)
                .map_or(0.0, |s| s.quantity);
            bound(
                &mut runs,
                available / (q * developed.inputs),
                Limit::Input(p),
            );
        }
        for &(g, h) in &per_run {
            if h > 0.0 {
                bound(&mut runs, hours[g.index()] / h, Limit::Labor(g));
            }
        }
        let own_power = if recipe.energy_mwh > 0.0 {
            let own = own_electricity(catalog, state, owner, country);
            bound(
                &mut runs,
                planned * grid_share + own / recipe.energy_mwh,
                Limit::Electricity,
            );
            own
        } else {
            0.0
        };
        if recipe.extraction {
            let deposit = state.sites[index].deposit.expect("checked in planned_runs");
            let ds = state.deposits.get(deposit);
            let field = ds.concession_of(site).expect("checked in planned_runs");
            let scale = state.settings.market_scale;
            // The yearly output spreads over the year: up to today at most the share of
            // the year gone by (M33; before, large mines used up their year in weeks and
            // stood still until January).
            let year_gone =
                f64::from(date.ordinal()) / f64::from(crate::calendar::days_in_year(date.year()));
            let mut room = deposit_output(catalog, state, deposit, date.year())
                * scale
                * field.share
                * year_gone
                - field.extracted_this_year;
            if let Some(reserve) = catalog.reserve(deposit, date.year()) {
                room = room.min(reserve * scale - ds.extracted);
            }
            bound(&mut runs, room.max(0.0) / recipe.output, Limit::Deposit);
        }
        state.sites[index].slots[slot].limit = limit;
        if runs <= 1e-9 {
            continue;
        }

        // Consume inputs.
        let mut value = Money::ZERO;
        let mut input_quality = 0.0;
        let mut input_quantity = 0.0;
        let s = &mut state.sites[index];
        for &(p, q) in &recipe.inputs {
            let stock = s.inventory.entry(p).or_default();
            let used = q * developed.inputs * runs;
            input_quality += stock.quality * used;
            input_quantity += used;
            value += stock.take(used);
        }
        let ledger = &mut state.companies[owner.index()].ledger;
        let center = CostCenter::product(site, recipe.product);
        ledger.expense(CostType::Material, center, Account::Inventory, value);
        let materials = value;
        // Labor used (paid with the wages of the day) and electricity. The wages of the
        // hours used move from the site's wage bill to the product (M18).
        let mut labor = Money::ZERO;
        for &(g, h) in &per_run {
            hours[g.index()] -= h * runs;
            labor += Money::from_usd(h * runs * wages.get(g.index()).copied().unwrap_or(0.0))
                .unwrap_or(Money::ZERO);
        }
        ledger.allocate(CostType::Personnel, CostCenter::site(site), center, labor);
        value += labor;
        // Own electricity first, the rest from the grid.
        let needed = recipe.energy_mwh * runs;
        let own = needed.min(own_power);
        let grid = Money::from_usd((needed - own) * electricity_price).unwrap_or(Money::ZERO);
        ledger.expense(CostType::Energy, center, Account::Cash, grid);
        value += grid;
        if own > 0.0 {
            let own_value = use_own_electricity(catalog, state, owner, country, own);
            let ledger = &mut state.companies[owner.index()].ledger;
            ledger.expense(CostType::Energy, center, Account::Inventory, own_value);
            value += own_value;
        }
        // Administration, sales and logistics on labor, electricity and plant.
        let conversion =
            (value - materials).to_usd() + capital_per_run_usd(catalog, recipe, size) * runs;
        let overhead =
            Money::from_usd(overhead_usd(catalog, recipe, conversion)).unwrap_or(Money::ZERO);
        if overhead > Money::ZERO {
            let ledger = &mut state.companies[owner.index()].ledger;
            ledger.expense(CostType::Overhead, center, Account::Cash, overhead);
            value += overhead;
        }
        let rent = Money::from_usd(rent_per_run * runs).unwrap_or(Money::ZERO);
        if rent > Money::ZERO {
            let ledger = &mut state.companies[owner.index()].ledger;
            ledger.expense(CostType::Rent, center, Account::Cash, rent);
            value += rent;
        }
        let ledger = &mut state.companies[owner.index()].ledger;
        ledger.income(CostType::InventoryChange, center, Account::Inventory, value);

        // Quality (docs/FORMELN.md).
        let sl = &state.sites[index].slots[slot];
        let average_input = if input_quantity > 0.0 {
            input_quality / input_quantity
        } else {
            50.0
        };
        let quality = (recipe.base_quality
            + model.quality_inputs * (average_input - 50.0)
            + model.quality_automation * sl.automation
            - model.quality_condition * (1.0 - sl.condition)
            + developed.quality
            + crate::training::quality(catalog, state.sites[index].training))
        .clamp(0.0, 100.0);

        let mut outputs = vec![(recipe.product, recipe.output * runs)];
        outputs.extend(recipe.by_products.iter().map(|&(p, q)| (p, q * runs)));
        let finish = date.add_days(i32::try_from(recipe.duration_days).unwrap_or(1) - 1);
        let sl = &mut state.sites[index].slots[slot];
        sl.batches.push(Batch {
            finish,
            outputs,
            quality,
            value,
        });
        sl.last_runs = runs;

        if recipe.extraction {
            let deposit = state.sites[index].deposit.expect("checked in planned_runs");
            let ds = state.deposits.get_mut(deposit);
            ds.extracted += recipe.output * runs;
            if let Some(field) = ds.concession_of_mut(site) {
                field.extracted_this_year += recipe.output * runs;
            }
        }
    }
}

fn finish_batches(state: &mut GameState, catalog: &Catalog, site: SiteId, date: Date) {
    let s = &mut state.sites[site.index()];
    let mut done = Vec::new();
    for slot in &mut s.slots {
        let (finished, open): (Vec<Batch>, Vec<Batch>) =
            slot.batches.drain(..).partition(|b| b.finish <= date);
        slot.batches = open;
        done.extend(finished);
    }
    deliver(catalog, s, done);
}

/// Puts finished batches into the site's stock.
pub(crate) fn deliver(catalog: &Catalog, s: &mut Site, batches: Vec<Batch>) {
    let days = catalog.production_model.by_product_stock_days;
    for batch in batches {
        // By-products beyond `days` of this output are disposed of (flared, dumped): a
        // refinery does not store petrol nobody buys for decades (M16). The batch's value
        // goes to what is kept.
        let kept: Vec<(ProductId, f64)> = batch
            .outputs
            .iter()
            .enumerate()
            .map(|(i, &(product, quantity))| {
                if i == 0 {
                    return (product, quantity);
                }
                let stock = s.inventory.get(&product).map_or(0.0, |x| x.quantity);
                (product, quantity.min((quantity * days - stock).max(0.0)))
            })
            .collect();
        let kept: Vec<(ProductId, f64)> = kept
            .into_iter()
            .enumerate()
            .filter(|&(i, (_, q))| i == 0 || q > 0.0)
            .map(|(_, k)| k)
            .collect();
        let total: f64 = kept.iter().map(|(_, q)| q).sum();
        let mut remaining = batch.value;
        for (i, &(product, quantity)) in kept.iter().enumerate() {
            let value = if i + 1 == kept.len() {
                remaining
            } else {
                batch.value.scale(quantity / total)
            };
            remaining -= value;
            s.inventory
                .entry(product)
                .or_default()
                .add(quantity, value, batch.quality);
        }
    }
}

/// Book value and sale proceeds of `units` of a facility on `date` (M22).
pub fn sale_value(catalog: &Catalog, slot: &Slot, units: u32, date: Date) -> (Money, Money) {
    let model = &catalog.production_model;
    let f = catalog.facilities.get(slot.facility);
    let book = slot.book_value(f.lifetime_years, units, date);
    let scrap = slot.share_of_cost(units).scale(model.scrap_share);
    (book, book.scale(model.sale_proceeds_share).max(scrap))
}

/// Wages, maintenance, depreciation and wear of one site for one day.
fn running_costs(state: &mut GameState, catalog: &Catalog, site: SiteId, date: Date) {
    let index = site.index();
    let s = &state.sites[index];
    let owner = s.owner;
    let worker_hours = hours_per_worker_day(catalog, date);
    let wages = &state.countries.get(s.country).hourly_wage_usd;
    let wage_bill: f64 = s
        .workforce
        .iter()
        .map(|(g, &w)| w * worker_hours * wages.get(g.index()).copied().unwrap_or(0.0))
        .sum::<f64>()
        * (1.0 + s.wage_premium);
    let model = &catalog.production_model;
    let mut maintenance = Money::ZERO;
    let mut depreciation = Money::ZERO;
    for sl in &s.slots {
        if sl.ready > date {
            continue;
        }
        let f = catalog.facilities.get(sl.facility);
        // A shut down facility is kept, not run (M22).
        let kept = if sl.mothballed() {
            model.mothball_maintenance_share
        } else {
            1.0
        };
        maintenance += sl.cost.scale(f.maintenance_share * kept / 365.0);
        let life_days = f64::from(f.lifetime_years) * 365.0;
        if f64::from(sl.ready.days_until(date)) < life_days {
            depreciation += sl.cost.scale(1.0 / life_days);
        }
    }
    let building_days = model.building_lifetime_years * 365.0;
    if f64::from(s.founded.days_until(date)) < building_days {
        depreciation += s.building_cost.scale(1.0 / building_days);
    }
    if let Some(field) = s
        .deposit
        .and_then(|d| state.deposits.get(d).concession_of(site))
    {
        let dev_days = model.development_lifetime_years * 365.0;
        if field
            .ready
            .is_some_and(|r| r <= date && f64::from(r.days_until(date)) < dev_days)
        {
            depreciation += field.development_cost.scale(1.0 / dev_days);
        }
    }
    // Researchers' wages are research costs (Lastenheft §14.2).
    let wage_type = if s.kind == SiteType::ResearchCenter {
        CostType::Research
    } else {
        CostType::Personnel
    };
    // Training costs a share of the wage bill (W1).
    let training = crate::training::daily_cost(catalog, state, site, wage_bill);
    let ledger = &mut state.companies[owner.index()].ledger;
    let center = CostCenter::site(site);
    ledger.expense(
        wage_type,
        center,
        Account::Cash,
        Money::from_usd(wage_bill + training).unwrap_or(Money::ZERO),
    );
    ledger.expense(CostType::Maintenance, center, Account::Cash, maintenance);
    ledger.expense(
        CostType::Depreciation,
        center,
        Account::FixedAssets,
        depreciation,
    );
    // Goodwill of a bought site, written off from the day of purchase (M30).
    if let Some(g) = state.sites[index].goodwill {
        let life_days = catalog.deal_model.goodwill_years * 365.0;
        if f64::from(g.from.days_until(date)) < life_days {
            let ledger = &mut state.companies[owner.index()].ledger;
            ledger.expense(
                CostType::Depreciation,
                center,
                Account::Goodwill,
                g.amount.scale(1.0 / life_days),
            );
        }
    }

    // Wear: the condition falls linearly over the lifetime, not while shut down.
    for sl in &mut state.sites[index].slots {
        if sl.ready <= date && !sl.mothballed() {
            let life_days = f64::from(catalog.facilities.get(sl.facility).lifetime_years) * 365.0;
            sl.condition = (sl.condition - 1.0 / life_days).max(model.condition_min);
        }
    }
}

/// Resets the yearly extraction counters (called on 1 January).
pub(crate) fn new_year(state: &mut GameState) {
    for (_, d) in state.deposits.iter_mut() {
        for field in &mut d.concessions {
            field.extracted_this_year = 0.0;
        }
    }
}

/// Marks all sites for staffing (called on the first of each month).
pub(crate) fn new_month(state: &mut GameState) {
    for s in &mut state.sites {
        s.staffing_due = true;
    }
}

/// Warnings for the player at the end of a round: facilities that stood still or ran
/// slower on the last day because an input was missing (Lastenheft §13.2).
pub(crate) fn input_warnings(state: &GameState, catalog: &Catalog) -> Vec<Message> {
    let mut messages = Vec::new();
    let date = state.date.add_days(-1);
    for (index, s) in state.sites.iter().enumerate() {
        if s.owner != state.player {
            continue;
        }
        let site = SiteId(u32::try_from(index).expect("site count fits u32"));
        let mut reported: Vec<ProductId> = Vec::new();
        for (slot, sl) in s.slots.iter().enumerate() {
            let Some((recipe, planned)) = planned_runs(catalog, state, site, slot, date) else {
                continue;
            };
            if sl.last_runs >= 0.99 * planned {
                continue;
            }
            let r = catalog.recipes.get(recipe);
            let developed = development::effect(catalog, state, s.owner, r.product);
            for &(input, q) in &r.inputs {
                let stock = s.inventory.get(&input).map_or(0.0, |x| x.quantity);
                if stock < q * developed.inputs && !reported.contains(&input) {
                    reported.push(input);
                    messages.push(
                        Message::new(MessageKind::Warning, keys::INPUT_MISSING)
                            .with(
                                "produkt",
                                Param::TextKey(format!("produkt.{}", catalog.products.key(input))),
                            )
                            .with(
                                "anlage",
                                Param::TextKey(format!(
                                    "anlage.{}",
                                    catalog.facilities.key(sl.facility)
                                )),
                            )
                            .with(
                                "land",
                                Param::Country(catalog.countries.key(s.country).to_owned()),
                            ),
                    );
                }
            }
        }
    }
    messages
}
