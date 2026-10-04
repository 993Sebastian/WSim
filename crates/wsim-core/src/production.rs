//! Daily production: construction, staffing, extraction and manufacturing, costs
//! (Lastenheft §5.2, §6.1–6.3; formulas in docs/FORMELN.md, section M5).

use crate::calendar::Date;
use crate::catalog::{Catalog, Recipe, SiteType};
use crate::ids::{CountryId, Id, LaborGroupId, ProductId, RecipeId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{Batch, CompanyId, GameState, Limit, SiteId};

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

/// Labor hours per run for each group, including automation and deposit difficulty.
fn hours_per_run(
    catalog: &Catalog,
    recipe: &Recipe,
    automation: f64,
    affinity: f64,
    cost_factor: f64,
) -> Vec<(LaborGroupId, f64)> {
    let factor = labor_factor(catalog, automation, affinity) * cost_factor;
    recipe
        .labor_hours
        .iter()
        .map(|&(g, h)| (g, h * factor))
        .collect()
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
            finish_batches(state, site, date);
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
    for site in 0..state.sites.len() {
        let owner = state.sites[site].owner;
        let finished: Money = state.sites[site]
            .slots
            .iter()
            .filter(|s| s.ready == date)
            .map(|s| {
                catalog
                    .facilities
                    .get(s.facility)
                    .investment
                    .scale(f64::from(s.count))
            })
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
    if sl.ready > date || sl.utilization <= 0.0 {
        return None;
    }
    if catalog.recipes.get(recipe).extraction {
        let deposit = s.deposit?;
        let field = state.deposits.get(deposit).concession_of(site)?;
        if field.ready.is_none_or(|r| r > date) {
            return None;
        }
    }
    Some((
        recipe,
        catalog.facilities.get(sl.facility).runs_per_day * f64::from(sl.count) * sl.utilization,
    ))
}

fn deposit_cost_factor(catalog: &Catalog, state: &GameState, site: SiteId, recipe: &Recipe) -> f64 {
    let s = &state.sites[site.index()];
    match (recipe.extraction, s.deposit) {
        (true, Some(d)) => catalog.deposits.get(d).cost_factor,
        _ => 1.0,
    }
}

/// Hires and dismisses staff so that every site has the workers its planned production
/// needs, within the free labor pool of its country (Lastenheft §5.3). Sites are served
/// in the order they were founded.
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
    let worker_hours = hours_per_worker_day(catalog, date);
    for index in 0..state.sites.len() {
        if !state.sites[index].staffing_due {
            continue;
        }
        let site = SiteId(u32::try_from(index).expect("site count fits u32"));
        let country = state.sites[index].country;
        let affinity = state.countries.get(country).automation_affinity;
        let mut needed = vec![0.0; groups];
        for slot in 0..state.sites[index].slots.len() {
            let Some((recipe_id, runs)) = planned_runs(catalog, state, site, slot, date) else {
                continue;
            };
            let recipe = catalog.recipes.get(recipe_id);
            let automation = state.sites[index].slots[slot].automation;
            let cost_factor = deposit_cost_factor(catalog, state, site, recipe);
            for (g, h) in hours_per_run(catalog, recipe, automation, affinity, cost_factor) {
                needed[g.index()] += runs * h / worker_hours;
            }
        }
        if let Some(technology) = state.sites[index].research {
            let field = catalog.technologies.get(technology).field;
            if let Some(group) = catalog
                .research_model
                .researchers
                .get(field.index())
                .copied()
                .flatten()
            {
                needed[group.index()] +=
                    crate::research::wanted_researchers(catalog, state, site, date);
            }
        }
        let pool = &state.countries.get(country).labor_available;
        let s = &mut state.sites[index];
        for g in 0..groups {
            let id = LaborGroupId::from_index(g);
            let current = *s.workforce.get(id);
            let target = needed[g];
            let new = if target > current {
                let free =
                    (pool.get(g).copied().unwrap_or(0.0) - employed[country.index()][g]).max(0.0);
                current + (target - current).min(free)
            } else {
                target
            };
            employed[country.index()][g] += new - current;
            *s.workforce.get_mut(id) = new;
        }
        s.staffing_due = false;
    }
}

fn produce(state: &mut GameState, catalog: &Catalog, site: SiteId, date: Date) {
    let index = site.index();
    let country = state.sites[index].country;
    let worker_hours = hours_per_worker_day(catalog, date);
    let (affinity, grid_share, electricity_price, wages) = {
        let c = state.countries.get(country);
        (
            c.automation_affinity,
            c.grid_share,
            c.electricity_price_usd_mwh,
            c.hourly_wage_usd.clone(),
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
        let automation = state.sites[index].slots[slot].automation;
        let cost_factor = deposit_cost_factor(catalog, state, site, recipe);
        let per_run = hours_per_run(catalog, recipe, automation, affinity, cost_factor);

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
            bound(&mut runs, available / q, Limit::Input(p));
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
            let d = catalog.deposits.get(deposit);
            let ds = state.deposits.get(deposit);
            let field = ds.concession_of(site).expect("checked in planned_runs");
            let scale = state.settings.market_scale;
            let mut room = d.max_output_per_year * scale * field.share - field.extracted_this_year;
            if let Some(reserve) = d.reserve {
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
            input_quality += stock.quality * q * runs;
            input_quantity += q * runs;
            value += stock.take(q * runs);
        }
        let ledger = &mut state.companies[owner.index()].ledger;
        let center = CostCenter::product(site, recipe.product);
        ledger.expense(CostType::Material, center, Account::Inventory, value);
        // Labor used (paid with the wages of the day) and electricity.
        for &(g, h) in &per_run {
            hours[g.index()] -= h * runs;
            value += Money::from_usd(h * runs * wages.get(g.index()).copied().unwrap_or(0.0))
                .unwrap_or(Money::ZERO);
        }
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
            - model.quality_condition * (1.0 - sl.condition))
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

fn finish_batches(state: &mut GameState, site: SiteId, date: Date) {
    let s = &mut state.sites[site.index()];
    for slot in &mut s.slots {
        let (done, open): (Vec<Batch>, Vec<Batch>) =
            slot.batches.drain(..).partition(|b| b.finish <= date);
        slot.batches = open;
        for batch in done {
            let total: f64 = batch.outputs.iter().map(|(_, q)| q).sum();
            let mut remaining = batch.value;
            for (i, &(product, quantity)) in batch.outputs.iter().enumerate() {
                let value = if i + 1 == batch.outputs.len() {
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
        .sum();
    let model = &catalog.production_model;
    let mut maintenance = Money::ZERO;
    let mut depreciation = Money::ZERO;
    for sl in &s.slots {
        if sl.ready > date {
            continue;
        }
        let f = catalog.facilities.get(sl.facility);
        maintenance += sl.cost.scale(f.maintenance_share / 365.0);
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
    let ledger = &mut state.companies[owner.index()].ledger;
    let center = CostCenter::site(site);
    ledger.expense(
        wage_type,
        center,
        Account::Cash,
        Money::from_usd(wage_bill).unwrap_or(Money::ZERO),
    );
    ledger.expense(CostType::Maintenance, center, Account::Cash, maintenance);
    ledger.expense(
        CostType::Depreciation,
        center,
        Account::FixedAssets,
        depreciation,
    );

    // Wear: the condition falls linearly over the lifetime.
    for sl in &mut state.sites[index].slots {
        if sl.ready <= date {
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
            for &(input, q) in &r.inputs {
                let stock = s.inventory.get(&input).map_or(0.0, |x| x.quantity);
                if stock < q && !reported.contains(&input) {
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
