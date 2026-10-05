//! Decisions of the AI companies (Lastenheft §10; docs/FORMELN.md, M10).
//!
//! AI companies act only through commands, checked like the player's. Simple rules
//! whose thresholds depend on competence and aggressiveness steer them: production
//! follows the stock, prices search the market above a floor from the normal cost,
//! purchases keep a stock, loans keep the cash within bounds, good business is
//! expanded, competent companies research, and bankrupt companies make room for new
//! ones.

use std::collections::BTreeMap;

use crate::calendar::Date;
use crate::catalog::{Catalog, ProductKind, SiteType};
use crate::command::{self, Command};
use crate::finance;
use crate::ids::{CountryId, DepositId, FacilityId, Id, ProductId, RecipeId, TechnologyId};
use crate::ledger::{CostType, Ledger};
use crate::market;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::population::{self, SlotFlows};
use crate::research;
use crate::rng::{SimRng, Stream};
use crate::state::{
    AiState, Company, CompanyId, CompanyKind, GameState, Limit, Operation, PriceMode, SiteId,
};

/// Runs the decisions due today, before the day is simulated. Returns news about
/// competitors for the player's round report.
pub(crate) fn decide(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let first_of_month = date.day() == 1;
    // Expansion looks at the sales of the closing month, so it runs on its last day.
    let end_of_quarter = date.next_day().day() == 1 && date.month().is_multiple_of(3);
    let first_of_year = date.ordinal() == 1;
    let due: Vec<CompanyId> = state
        .companies
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            !c.bankrupt
                && c.ai.as_ref().is_some_and(|ai| {
                    first_of_month || end_of_quarter || ai.next_operations <= date
                })
        })
        .map(|(i, _)| company_id(i))
        .collect();
    if due.is_empty() && !first_of_month {
        return news;
    }
    if due.is_empty() {
        found_companies(state, catalog, date, &mut news);
        return news;
    }
    let mut sites: BTreeMap<CompanyId, Vec<SiteId>> = BTreeMap::new();
    for (i, s) in state.sites.iter().enumerate() {
        sites.entry(s.owner).or_default().push(site_id(i));
    }
    let none = Vec::new();
    for id in due {
        let own = sites.get(&id).unwrap_or(&none);
        let ai = state.companies[id.index()].ai.clone().expect("AI company");
        if ai.next_operations <= date {
            operate(state, catalog, id, own, date);
            supply_own(state, catalog, id, own);
            let days = catalog
                .ai_model
                .behavior
                .operations_days
                .at(ai.competence)
                .round()
                .max(1.0);
            if let Some(ai) = state.companies[id.index()].ai.as_mut() {
                // A few days; the cast cannot overflow.
                ai.next_operations = date.add_days(days as i32);
            }
        }
        if first_of_month {
            manage_cash(state, catalog, id, own);
            advertise(state, catalog, id, own);
        }
        if end_of_quarter {
            retire(state, catalog, id, own, date, &mut news);
            expand(state, catalog, id, own, date, &mut news);
        }
        if first_of_year {
            research_plan(state, catalog, id, own, date);
        }
    }
    if first_of_month {
        found_companies(state, catalog, date, &mut news);
    }
    if end_of_quarter {
        diversify(state, catalog, &mut news);
    }
    news
}

/// Products the player makes or offers: competitors' moves in them are news.
fn player_products(state: &GameState, catalog: &Catalog) -> Vec<ProductId> {
    let mut products = Vec::new();
    for s in state.sites.iter().filter(|s| s.owner == state.player) {
        let made = s
            .slots
            .iter()
            .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r).product));
        for p in made.chain(s.offers.keys().copied()) {
            if !products.contains(&p) {
                products.push(p);
            }
        }
    }
    products
}

fn news_expansion(
    state: &GameState,
    catalog: &Catalog,
    id: CompanyId,
    site: SiteId,
    recipe: RecipeId,
    count: u32,
) -> Option<Message> {
    let r = catalog.recipes.get(recipe);
    if !player_products(state, catalog).contains(&r.product) {
        return None;
    }
    Some(
        Message::new(MessageKind::Info, keys::AI_EXPANDS)
            .with(
                "firma",
                Param::Text(state.companies[id.index()].name.clone()),
            )
            .with(
                "land",
                Param::Country(
                    catalog
                        .countries
                        .key(state.sites[site.index()].country)
                        .to_owned(),
                ),
            )
            .with("anzahl", Param::Integer(i64::from(count)))
            .with(
                "anlage",
                Param::TextKey(format!("anlage.{}", catalog.facilities.key(r.facility))),
            )
            .with(
                "produkt",
                Param::TextKey(format!("produkt.{}", catalog.products.key(r.product))),
            ),
    )
}

/// News for the player: a competitor shuts down or sells facilities of a product the
/// player makes or offers (M22).
fn news_capacity(
    state: &GameState,
    catalog: &Catalog,
    key: &'static str,
    (id, site): (CompanyId, SiteId),
    (facility, product): (FacilityId, ProductId),
    count: u32,
) -> Message {
    Message::new(MessageKind::Info, key)
        .with(
            "firma",
            Param::Text(state.companies[id.index()].name.clone()),
        )
        .with(
            "land",
            Param::Country(
                catalog
                    .countries
                    .key(state.sites[site.index()].country)
                    .to_owned(),
            ),
        )
        .with("anzahl", Param::Integer(i64::from(count)))
        .with(
            "anlage",
            Param::TextKey(format!("anlage.{}", catalog.facilities.key(facility))),
        )
        .with(
            "produkt",
            Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
        )
}

/// Whole months from `since` to `date`.
fn months_between(since: Date, date: Date) -> u32 {
    let months = (date.year() - since.year()) * 12 + i32::try_from(date.month()).unwrap_or(0)
        - i32::try_from(since.month()).unwrap_or(0);
    u32::try_from(months.max(0)).unwrap_or(0)
}

/// Overcapacity (M22, docs/FORMELN.md): at the end of a quarter the company sells units
/// that stood still for `verkaufen_nach_monaten` and shuts down the units of a product at
/// a site that it does not need at the target utilization. Power plants and
/// laboratories stay.
fn retire(
    state: &mut GameState,
    catalog: &Catalog,
    id: CompanyId,
    sites: &[SiteId],
    date: Date,
    news: &mut Vec<Message>,
) {
    let b = &catalog.ai_model.behavior;
    let watched = player_products(state, catalog);
    for &site in sites {
        if state.sites[site.index()].kind == SiteType::ResearchCenter {
            continue;
        }
        // The last first: selling a whole facility moves the later ones up.
        let sell: Vec<(usize, u32, FacilityId, Option<ProductId>)> = state.sites[site.index()]
            .slots
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, sl)| {
                matches!(sl.operation, Operation::Mothballed { since }
                    if months_between(since, date) >= b.sell_after_months)
            })
            .map(|(i, sl)| {
                let product = sl.recipe.map(|r| catalog.recipes.get(r).product);
                (i, sl.count, sl.facility, product)
            })
            .collect();
        for (slot, count, facility, product) in sell {
            let sold = run(
                state,
                catalog,
                id,
                &Command::SellFacility { site, slot, count },
            );
            if let Some(p) = product.filter(|p| sold && watched.contains(p)) {
                let key = keys::AI_SELLS;
                news.push(news_capacity(
                    state,
                    catalog,
                    key,
                    (id, site),
                    (facility, p),
                    count,
                ));
            }
        }

        // Per product: the running units with their planned utilization and their cost
        // per unit at full load; what is taken per day against what they could make.
        let (full_output, taken) = output_and_offtake(state, catalog, site, date);
        restart_where_short(state, catalog, id, site, (&full_output, &taken));
        let (full_output, taken) = output_and_offtake(state, catalog, site, date);
        let s = &state.sites[site.index()];
        let wage = 1.0 + s.wage_premium;
        let mut by_product: BTreeMap<ProductId, Vec<(usize, u32, f64, f64)>> = BTreeMap::new();
        for (i, sl) in s.slots.iter().enumerate() {
            let Some(r) = sl.recipe.filter(|_| sl.operating(date)) else {
                continue;
            };
            let recipe = catalog.recipes.get(r);
            let product = recipe.product;
            // Extraction is bound to its concession; goods used or sent elsewhere by the
            // company itself are not measured by sales; and where the product sells at or
            // above its reference price the market wants the capacity.
            if catalog.products.get(product).kind == ProductKind::Energy
                || recipe.extraction
                || !s.offers.contains_key(&product)
                || market::market_price(catalog, state, s.country, product)
                    >= market::local_reference(catalog, state, s.country, product)
                        .scale(b.mothball_price_max)
            {
                continue;
            }
            let flows = population::slot_flows(catalog, state, s.country, r, sl.count, 1.0, wage);
            let unit = flows.cost_per_day.to_usd() / flows.output.max(1e-9);
            by_product
                .entry(product)
                .or_default()
                .push((i, sl.count, sl.utilization, unit));
        }
        for (product, mut slots) in by_product {
            let n: u32 = slots.iter().map(|x| x.1).sum();
            let u = slots.iter().map(|x| f64::from(x.1) * x.2).sum::<f64>() / f64::from(n.max(1));
            // Demand on the running units: what is taken against their full output.
            let full = full_output.get(&product).copied().unwrap_or(0.0);
            let d = taken.get(&product).copied().unwrap_or(0.0) / full.max(1e-9);
            if n < 2 || d >= b.mothball_utilization {
                continue;
            }
            // Small counts; the cast cannot overflow.
            let keep = ((f64::from(n) * d / b.mothball_target_utilization).ceil() as u32).max(1);
            let mut excess = n.saturating_sub(keep);
            // The dearest first.
            slots.sort_by(|a, b| b.3.total_cmp(&a.3));
            let mut shut = 0;
            let mut facility = None;
            for &(slot, count, _, _) in &slots {
                if excess == 0 {
                    break;
                }
                let k = count.min(excess);
                let command = Command::MothballFacility {
                    site,
                    slot,
                    count: k,
                };
                if run(state, catalog, id, &command) {
                    excess -= k;
                    shut += k;
                    facility = Some(state.sites[site.index()].slots[slot].facility);
                }
            }
            let Some(facility) = facility else {
                continue;
            };
            // The units still running make what all of them made.
            let next = (u * f64::from(n) / f64::from((n - shut).max(1))).min(1.0);
            let running: Vec<(usize, RecipeId)> = state.sites[site.index()]
                .slots
                .iter()
                .enumerate()
                .filter(|(_, sl)| sl.operating(date))
                .filter_map(|(i, sl)| {
                    let r = sl.recipe?;
                    (catalog.recipes.get(r).product == product).then_some((i, r))
                })
                .collect();
            for (slot, recipe) in running {
                let command = Command::SetProduction {
                    site,
                    slot,
                    recipe: Some(recipe),
                    utilization: next,
                };
                run(state, catalog, id, &command);
            }
            if watched.contains(&product) {
                let key = keys::AI_MOTHBALLS;
                news.push(news_capacity(
                    state,
                    catalog,
                    key,
                    (id, site),
                    (facility, product),
                    shut,
                ));
            }
        }
    }
}

/// More is taken of a product than its running facilities make at the restart
/// utilization: shut down units of it start up again until those running and starting
/// would be at the target utilization (M22). Not while some are starting up already.
fn restart_where_short(
    state: &mut GameState,
    catalog: &Catalog,
    id: CompanyId,
    site: SiteId,
    (full_output, taken): (&BTreeMap<ProductId, f64>, &BTreeMap<ProductId, f64>),
) {
    let b = &catalog.ai_model.behavior;
    for (&product, &wanted) in taken {
        let full = full_output.get(&product).copied().unwrap_or(0.0);
        let s = &state.sites[site.index()];
        let of = |sl: &crate::state::Slot| {
            sl.recipe
                .is_some_and(|r| catalog.recipes.get(r).product == product)
        };
        let restarting = s
            .slots
            .iter()
            .any(|sl| of(sl) && matches!(sl.operation, Operation::Restarting { .. }));
        if wanted <= b.restart_utilization * full || restarting {
            continue;
        }
        let shut: Vec<(usize, f64)> = s
            .slots
            .iter()
            .enumerate()
            .filter(|(_, sl)| sl.mothballed() && of(sl))
            .map(|(i, sl)| {
                let r = catalog.recipes.get(sl.recipe.expect("filtered"));
                let output = catalog.facilities.get(sl.facility).runs_per_day
                    * f64::from(sl.count)
                    * r.output;
                (i, output)
            })
            .collect();
        let mut capacity = full;
        for (slot, output) in shut {
            if wanted <= b.mothball_target_utilization * capacity {
                break;
            }
            if run(state, catalog, id, &Command::RestartFacility { site, slot }) {
                capacity += output;
            }
        }
    }
}

fn company_id(index: usize) -> CompanyId {
    CompanyId(u32::try_from(index).expect("company count fits u32"))
}

fn site_id(index: usize) -> SiteId {
    SiteId(u32::try_from(index).expect("site count fits u32"))
}

/// Executes a command for an AI company; a refused command is simply not carried out.
fn run(state: &mut GameState, catalog: &Catalog, actor: CompanyId, command: &Command) -> bool {
    command::execute(state, catalog, actor, command).is_ok()
}

fn traits(state: &GameState, id: CompanyId) -> (f64, f64) {
    state.companies[id.index()]
        .ai
        .as_ref()
        .map_or((0.5, 0.5), |ai| (ai.competence, ai.aggressiveness))
}

/// The cheapest recipe a company knows for a product, optionally on one facility.
fn best_recipe(
    catalog: &Catalog,
    state: &GameState,
    id: CompanyId,
    product: ProductId,
    facility: Option<FacilityId>,
    country: Option<CountryId>,
) -> Option<RecipeId> {
    // Electricity where the grid is small is dear: scarce supply counts as a higher
    // price, so that electric recipes win only where power is available.
    let energy = match country {
        Some(c) => {
            let v = state.countries.get(c);
            let own = state.sites.iter().any(|s| {
                s.owner == id && s.country == c && s.kind == crate::catalog::SiteType::PowerPlant
            });
            let share = if own { 1.0 } else { v.grid_share.max(0.05) };
            v.electricity_price_usd_mwh / share
        }
        None => catalog
            .products
            .values()
            .find(|p| p.kind == ProductKind::Energy)
            .map_or(0.0, |p| p.reference_price.to_usd()),
    };
    catalog
        .recipes
        .iter()
        .filter(|&(_, r)| {
            r.product == product
                && facility.is_none_or(|f| f == r.facility)
                && r.technology.is_none_or(|t| state.knows(catalog, id, t))
                && catalog
                    .facilities
                    .get(r.facility)
                    .technology
                    .is_none_or(|t| state.knows(catalog, id, t))
        })
        .map(|(rid, _)| (rid, population::reference_unit_cost(catalog, rid, energy)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(rid, _)| rid)
}

/// Production, prices and purchases of every site.
fn operate(state: &mut GameState, catalog: &Catalog, id: CompanyId, sites: &[SiteId], date: Date) {
    let model = &catalog.ai_model;
    let b = &model.behavior;
    let (_, aggressiveness) = traits(state, id);
    let floor_factor = b.floor_factor.at(aggressiveness);
    for &site in sites {
        let s = &state.sites[site.index()];
        if s.kind == SiteType::ResearchCenter {
            continue;
        }
        let country = s.country;
        let mut commands = Vec::new();
        let mut need: BTreeMap<ProductId, f64> = BTreeMap::new();
        let mut willing: BTreeMap<ProductId, f64> = BTreeMap::new();
        let mut cost: BTreeMap<ProductId, (Money, f64, f64, f64)> = BTreeMap::new();
        let (full_output, taken) = output_and_offtake(state, catalog, site, date);
        for (index, sl) in s.slots.iter().enumerate() {
            // Shut down facilities wait for a restart (M22).
            if sl.mothballed() {
                continue;
            }
            let Some(recipe) = sl.recipe else {
                // A new facility: the company's best recipe for it.
                let products: Vec<ProductId> = catalog
                    .recipes
                    .values()
                    .filter(|r| r.facility == sl.facility)
                    .map(|r| r.product)
                    .collect();
                let fits = |r: RecipeId| {
                    let r = catalog.recipes.get(r);
                    !r.extraction
                        || s.deposit
                            .is_some_and(|d| catalog.deposits.get(d).resource == r.product)
                };
                if let Some(r) = products
                    .iter()
                    .filter_map(|&p| {
                        best_recipe(catalog, state, id, p, Some(sl.facility), Some(s.country))
                    })
                    .find(|&r| fits(r))
                {
                    commands.push(Command::SetProduction {
                        site,
                        slot: index,
                        recipe: Some(r),
                        utilization: model.start.utilization,
                    });
                }
                continue;
            };
            let r = catalog.recipes.get(recipe);
            let product = r.product;
            // A better recipe on the same facility, e.g. after research.
            let recipe = best_recipe(
                catalog,
                state,
                id,
                product,
                Some(sl.facility),
                Some(country),
            )
            .filter(|&better| !catalog.recipes.get(better).extraction || r.extraction)
            .unwrap_or(recipe);
            let mut utilization = sl.utilization;
            if catalog.products.get(product).kind != ProductKind::Energy && sl.operating(date) {
                let full = full_output.get(&product).copied().unwrap_or(0.0).max(1e-9);
                let keep = s.offers.get(&product).map_or(0.0, |o| o.keep);
                let stock = s.inventory.get(&product).map_or(0.0, |x| x.quantity) - keep;
                let wanted = taken.get(&product).copied().unwrap_or(0.0);
                // Sold out while selling above the price floor: the market wants more than
                // the plant makes, whatever it sold so far (M16). Without this a plant that
                // lost buyers once stayed small next to a scarce market.
                let scarce = stock <= wanted.max(1e-9)
                    && s.offers.get(&product).is_some_and(|o| match o.mode {
                        PriceMode::Market { floor, .. } => {
                            o.price > floor.scale(1.0 + b.utilization_step)
                        }
                        PriceMode::Fixed(_) => false,
                    });
                let next = if wanted > 1e-9 {
                    // Make what is taken and steer the stock to its target (M16). The old
                    // steps of ±0.1 needed months to follow the demand.
                    let target = b.stock_target_days * wanted;
                    let next = ((wanted + (target - stock) / b.stock_adjust_days) / full)
                        .clamp(b.utilization_min, 1.0);
                    if scarce {
                        next.max((utilization + b.utilization_step).min(1.0))
                    } else {
                        next
                    }
                } else if stock > b.stock_high_days * full * utilization.max(b.utilization_min) {
                    // Nothing taken yet (a new plant): produce until the stock is high.
                    (utilization - b.utilization_step).max(b.utilization_min)
                } else {
                    utilization
                };
                let planned = catalog.facilities.get(sl.facility).runs_per_day
                    * f64::from(sl.count)
                    * sl.utilization;
                // Held back by missing inputs or labour: planning more would not help.
                // The grid, by contrast, delivers a share of whatever is planned.
                let limited = sl.last_runs < 0.9 * planned && sl.limit != Some(Limit::Electricity);
                let next = next.clamp(
                    utilization - b.utilization_change_max,
                    utilization + b.utilization_change_max,
                );
                if !(limited && next > utilization) {
                    utilization = next;
                }
            }
            if recipe != sl.recipe.expect("set") || (utilization - sl.utilization).abs() > 1e-9 {
                commands.push(Command::SetProduction {
                    site,
                    slot: index,
                    recipe: Some(recipe),
                    utilization,
                });
            }
            let flows: SlotFlows = population::slot_flows(
                catalog,
                state,
                country,
                recipe,
                sl.count,
                utilization,
                1.0 + s.wage_premium,
            );
            for (input, q) in &flows.inputs {
                *need.entry(*input).or_default() += q;
            }
            // What an input may cost at most: the value this slot makes of it after all
            // its other costs (derived demand), at the site's own selling prices.
            let r = catalog.recipes.get(recipe);
            if flows.output > 1e-9 {
                let runs = flows.output / r.output.max(1e-9);
                let value = |p: ProductId| {
                    s.offers
                        .get(&p)
                        .map_or_else(
                            || market::market_price(catalog, state, country, p),
                            |o| o.price,
                        )
                        .to_usd()
                };
                let made = r.output * value(r.product)
                    + r.by_products
                        .iter()
                        .map(|&(p, q)| q * value(p))
                        .sum::<f64>();
                let variable = flows.variable_per_day.to_usd();
                for &(input, q) in &flows.inputs {
                    if q <= 1e-9 {
                        continue;
                    }
                    let own = q * market::market_price(catalog, state, country, input).to_usd();
                    let most = (runs * made - (variable - own)) / q;
                    let w = willing.entry(input).or_insert(0.0);
                    *w = w.max(most);
                }
            }
            // Fixed costs per unit at the normal utilization, so that a quiet month
            // does not push the floor up: plant, maintenance and deposit development.
            let full = catalog.facilities.get(sl.facility).runs_per_day
                * f64::from(sl.count)
                * catalog.recipes.get(recipe).output
                * model.start.utilization;
            let mut fixed = (flows.cost_per_day - flows.variable_per_day).to_usd();
            if catalog.recipes.get(recipe).extraction
                && let Some(d) = s.deposit
                && let Some(c) = state
                    .deposits
                    .get(d)
                    .concessions
                    .iter()
                    .find(|c| c.site == Some(site))
            {
                fixed += c.development_cost.to_usd()
                    / (catalog.production_model.development_lifetime_years * 365.0).max(1.0);
            }
            // Inputs at what the stock cost, not at today's market price: a short spike of
            // an input price does not run through the whole chain at once (M16).
            let mut variable = flows.variable_per_day.to_usd();
            for &(input, q) in &flows.inputs {
                if let Some(st) = s.inventory.get(&input).filter(|st| st.quantity > 1e-9) {
                    let market = market::market_price(catalog, state, country, input).to_usd();
                    variable += q * (st.value.to_usd() / st.quantity - market);
                }
            }
            let e = cost
                .entry(flows.product)
                .or_insert((Money::ZERO, 0.0, 0.0, 0.0));
            e.0 += Money::from_usd(variable.max(0.0)).unwrap_or(Money::ZERO);
            e.1 += flows.output;
            e.2 += fixed;
            e.3 += full;
        }
        // By-products (petrol from the refinery) carry no cost of their own: they are
        // not sold below their worth as fuel (M22).
        let by_products: Vec<ProductId> = s
            .slots
            .iter()
            .filter_map(|sl| sl.recipe)
            .flat_map(|r| catalog.recipes.get(r).by_products.iter().map(|&(p, _)| p))
            .filter(|p| {
                !s.slots.iter().any(|sl| {
                    sl.recipe
                        .is_some_and(|r| catalog.recipes.get(r).product == *p)
                })
            })
            .collect();
        // Prices: the market decides above a floor from the full unit cost.
        for (&product, offer) in &s.offers {
            let Some(&(daily, output, fixed, full)) = cost.get(&product) else {
                if let PriceMode::Market { floor: old, .. } = offer.mode
                    && by_products.contains(&product)
                {
                    let floor = fuel_value(catalog, state, country, product);
                    if (floor.to_usd() - old.to_usd()).abs() > 0.05 * old.to_usd().max(1e-9) {
                        commands.push(Command::SetSale {
                            site,
                            product,
                            mode: Some(PriceMode::Market { markup: 0.0, floor }),
                            keep: offer.keep,
                        });
                    }
                }
                continue;
            };
            if output <= 1e-9 {
                continue;
            }
            let unit = daily.to_usd() / output + fixed / full.max(1e-9);
            let floor = Money::from_usd(unit * floor_factor).unwrap_or(Money::ZERO);
            let keep = need.get(&product).copied().unwrap_or(0.0) * model.start.input_stock_days;
            let old = match offer.mode {
                PriceMode::Market { floor, .. } => floor,
                PriceMode::Fixed(_) => continue,
            };
            let changed = (floor.to_usd() - old.to_usd()).abs() > 0.05 * old.to_usd().max(1e-9)
                || (keep - offer.keep).abs() > 0.1 * offer.keep.max(1.0);
            if changed {
                commands.push(Command::SetSale {
                    site,
                    product,
                    mode: Some(PriceMode::Market { markup: 0.0, floor }),
                    keep,
                });
            }
        }
        // Goods made here without an offer (by-products such as petrol from the
        // refinery) are sold at the market price unless the site needs them itself.
        for (&product, stock) in &s.inventory {
            let made = s.slots.iter().any(|sl| {
                sl.recipe.is_some_and(|r| {
                    let r = catalog.recipes.get(r);
                    r.product == product || r.by_products.iter().any(|&(p, _)| p == product)
                })
            });
            if made
                && stock.quantity > 1e-9
                && !s.offers.contains_key(&product)
                && !need.contains_key(&product)
                && catalog.products.get(product).kind != ProductKind::Energy
            {
                commands.push(Command::SetSale {
                    site,
                    product,
                    mode: Some(PriceMode::Market {
                        markup: 0.0,
                        floor: fuel_value(catalog, state, country, product),
                    }),
                    keep: 0.0,
                });
            }
        }
        // Purchases keep a stock of the inputs; short inputs are bid up.
        for (&product, &per_day) in &need {
            if per_day <= 1e-9 {
                continue;
            }
            let target = per_day * model.start.input_stock_days;
            let price = market::market_price(catalog, state, country, product);
            let stock = s.inventory.get(&product).map_or(0.0, |x| x.quantity);
            let normal = price.scale(1.0 + b.purchase_markup);
            // Short inputs are bid up to what the products made of them can bear (M16;
            // before, to three times a market index that stands still while nothing
            // sells, so plants starved next to imports that cost more).
            let most = willing
                .get(&product)
                .and_then(|&w| Money::from_usd(w))
                .unwrap_or(Money::ZERO)
                .min(
                    market::local_reference(catalog, state, country, product)
                        .scale(catalog.market_model.price_max_factor),
                )
                .max(normal);
            let max_price = match s.orders.get(&product) {
                Some(o) if stock < per_day * b.stock_low_days => o
                    .max_price
                    .scale(1.0 + b.utilization_step)
                    .min(most)
                    .max(normal),
                Some(o) if o.max_price > normal && stock >= per_day * b.stock_high_days => {
                    normal.max(o.max_price.scale(1.0 - b.utilization_step))
                }
                Some(o) => o.max_price.min(most).max(normal),
                None => normal,
            };
            let changed = s.orders.get(&product).is_none_or(|o| {
                (o.target - target).abs() > 0.1 * o.target.max(1e-9) || o.max_price != max_price
            });
            if changed {
                commands.push(Command::SetPurchase {
                    site,
                    product,
                    target,
                    max_price,
                    min_quality: 0.0,
                });
            }
        }
        for &product in s.orders.keys() {
            if !need.contains_key(&product) {
                commands.push(Command::SetPurchase {
                    site,
                    product,
                    target: 0.0,
                    max_price: Money::ZERO,
                    min_quality: 0.0,
                });
            }
        }
        let short_of_staff = s
            .slots
            .iter()
            .any(|sl| matches!(sl.limit, Some(Limit::Labor(_))));
        let premium = next_wage_premium(catalog, s.wage_premium, short_of_staff);
        if (premium - s.wage_premium).abs() > 1e-9 {
            commands.push(Command::SetWagePremium { site, premium });
        }
        for c in &commands {
            run(state, catalog, id, c);
        }
    }
}

/// Per product of a site: full output per day of its running facilities, and what is
/// taken per day (sales over the last month and this one, own facilities' inputs).
/// What a product is worth burnt in place of the cheapest other fuel in the country, per
/// unit (M22); zero for products without a heating value. Below it a refinery would fire
/// its stills with the petrol rather than sell it.
fn fuel_value(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    product: ProductId,
) -> Money {
    let Some(heat) = catalog.products.get(product).heating_value_mwh else {
        return Money::ZERO;
    };
    catalog
        .products
        .iter()
        .filter(|&(p, _)| p != product)
        .filter_map(|(p, x)| {
            let h = x.heating_value_mwh.filter(|&h| h > 0.0)?;
            let per_mwh = market::market_price(catalog, state, country, p).to_usd() / h;
            (per_mwh > 0.0).then_some(per_mwh)
        })
        .min_by(f64::total_cmp)
        .and_then(|per_mwh| Money::from_usd(heat * per_mwh))
        .unwrap_or(Money::ZERO)
}

fn output_and_offtake(
    state: &GameState,
    catalog: &Catalog,
    site: SiteId,
    date: Date,
) -> (BTreeMap<ProductId, f64>, BTreeMap<ProductId, f64>) {
    let s = &state.sites[site.index()];
    let mut full_output: BTreeMap<ProductId, f64> = BTreeMap::new();
    let mut taken: BTreeMap<ProductId, f64> = BTreeMap::new();
    for sl in s.slots.iter().filter(|sl| sl.operating(date)) {
        let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r)) else {
            continue;
        };
        let runs = catalog.facilities.get(sl.facility).runs_per_day * f64::from(sl.count);
        *full_output.entry(r.product).or_default() += runs * r.output;
        for &(input, q) in &r.inputs {
            *taken.entry(input).or_default() += runs * sl.utilization * q;
        }
    }
    // Without sales last month (a new offer, the first month of the game, a seller that
    // was sold out) this month alone tells the rate: averaging over 30 more days would
    // take a new seller for a slow one and stop its plants.
    let days = f64::from(date.day() - 1);
    for (&product, o) in &s.offers {
        let rate = if o.sold_last_month > 1e-9 {
            (o.sold_last_month + o.sold_month) / (30.0 + days)
        } else if days > 0.0 {
            o.sold_month / days
        } else {
            0.0
        };
        *taken.entry(product).or_default() += rate;
    }
    (full_output, taken)
}

/// Wage premium of a site after an operating decision (M18): a step up while facilities
/// wait for workers, a step down otherwise.
fn next_wage_premium(catalog: &Catalog, current: f64, short_of_staff: bool) -> f64 {
    let b = &catalog.ai_model.behavior;
    if short_of_staff {
        (current + b.wage_premium_step)
            .min(b.wage_premium_max)
            .min(catalog.production_model.wage_premium_max)
    } else {
        (current - b.wage_premium_step).max(0.0)
    }
}

/// Inputs the company makes itself go from its producing sites to the sites that need
/// them (a company cannot buy its own goods on the market): same country first.
fn supply_own(state: &mut GameState, catalog: &Catalog, id: CompanyId, sites: &[SiteId]) {
    let mut moves: Vec<Command> = Vec::new();
    let mut taken: BTreeMap<(SiteId, ProductId), f64> = BTreeMap::new();
    let mut underway: BTreeMap<(SiteId, ProductId), f64> = BTreeMap::new();
    for sh in &state.shipments {
        if let crate::state::Consignee::Site(site) = sh.to
            && state.sites[site.index()].owner == id
        {
            *underway.entry((site, sh.product)).or_default() += sh.quantity;
        }
    }
    for &to in sites {
        let s = &state.sites[to.index()];
        for (&product, order) in &s.orders {
            let stock = s.inventory.get(&product).map_or(0.0, |x| x.quantity);
            let coming = underway.get(&(to, product)).copied().unwrap_or(0.0);
            let mut deficit = order.target - stock - coming;
            if deficit <= 1e-9 {
                continue;
            }
            let mut sources: Vec<(bool, SiteId, f64)> = sites
                .iter()
                .filter(|&&from| from != to)
                .filter_map(|&from| {
                    let f = &state.sites[from.index()];
                    let offer = f.offers.get(&product)?;
                    let stock = f.inventory.get(&product).map_or(0.0, |x| x.quantity);
                    let used = taken.get(&(from, product)).copied().unwrap_or(0.0);
                    let free = stock - offer.keep - used;
                    (free > 1e-9).then_some((f.country != s.country, from, free))
                })
                .collect();
            sources.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
            for (_, from, free) in sources {
                if deficit <= 1e-9 {
                    break;
                }
                let quantity = deficit.min(free);
                *taken.entry((from, product)).or_default() += quantity;
                deficit -= quantity;
                moves.push(Command::TransferGoods {
                    from,
                    to,
                    product,
                    quantity,
                });
            }
        }
    }
    for m in &moves {
        run(state, catalog, id, m);
    }
}

/// Running cost per day of a company's sites at their planned production.
fn daily_cost(catalog: &Catalog, state: &GameState, sites: &[SiteId]) -> Money {
    let mut total = Money::ZERO;
    for &site in sites {
        let s = &state.sites[site.index()];
        for sl in &s.slots {
            if let Some(r) = sl.recipe {
                total += population::slot_flows(
                    catalog,
                    state,
                    s.country,
                    r,
                    sl.count,
                    sl.utilization,
                    1.0 + s.wage_premium,
                )
                .cost_per_day;
            }
        }
    }
    total
}

/// Loans keep the cash between the minimum and maximum months of running cost.
fn manage_cash(state: &mut GameState, catalog: &Catalog, id: CompanyId, sites: &[SiteId]) {
    let b = &catalog.ai_model.behavior;
    let monthly = daily_cost(catalog, state, sites).scale(30.0);
    let company = &state.companies[id.index()];
    let cash = company.ledger.cash();
    let target = monthly.scale((b.cash_min_months + b.cash_max_months) / 2.0);
    if cash < monthly.scale(b.cash_min_months) {
        let amount = (target - cash).min(finance::credit_limit(catalog, company));
        if amount > Money::ZERO {
            let years = b.loan_years.min(catalog.finance_model.max_term_years);
            run(state, catalog, id, &Command::TakeLoan { amount, years });
        }
    } else if cash > monthly.scale(b.cash_max_months) && !company.loans.is_empty() {
        let amount = cash - target;
        run(state, catalog, id, &Command::RepayLoan { loan: 0, amount });
    }
}

/// Expands the most profitable product that runs near its capacity and sells what it
/// makes.
fn expand(
    state: &mut GameState,
    catalog: &Catalog,
    id: CompanyId,
    sites: &[SiteId],
    date: Date,
    news: &mut Vec<Message>,
) {
    if own_power(state, catalog, id, sites, date) {
        return;
    }
    let b = &catalog.ai_model.behavior;
    let (_, aggressiveness) = traits(state, id);
    let min_utilization = b.expand_utilization.at(aggressiveness);
    let min_margin = b.expand_margin.at(aggressiveness);
    let mut best: Option<(f64, SiteId, ProductId, u32)> = None;
    for &site in sites {
        let s = &state.sites[site.index()];
        let mut by_product: BTreeMap<ProductId, (f64, u32, Money, f64)> = BTreeMap::new();
        // What the site's own facilities use per day (vertically integrated works).
        let mut own_use: BTreeMap<ProductId, f64> = BTreeMap::new();
        // Products whose facilities wait for workers or run short of an input (stock
        // below `lager_niedrig_tage` of its use, or priced above `ausbau_vorprodukt_preis`
        // times its reference price in the country): more of them would only compete for
        // the same scarce supply (M16).
        let mut held_back: Vec<ProductId> = Vec::new();
        // Full daily output of the extraction facilities: they grow only while the
        // concession allows more (its output follows the raw material's index, M16).
        let mut extracting: BTreeMap<ProductId, f64> = BTreeMap::new();
        for sl in &s.slots {
            let Some(r) = sl.recipe else { continue };
            if !sl.operating(date) {
                // Units starting up again come first; no more until they run (M22).
                if matches!(sl.operation, Operation::Restarting { .. }) {
                    held_back.push(catalog.recipes.get(r).product);
                }
                continue;
            }
            let recipe = catalog.recipes.get(r);
            let runs = catalog.facilities.get(sl.facility).runs_per_day
                * f64::from(sl.count)
                * sl.utilization;
            let short = recipe.inputs.iter().any(|&(input, q)| {
                let stock = s.inventory.get(&input).map_or(0.0, |x| x.quantity);
                let price = market::market_price(catalog, state, s.country, input);
                let reference = market::local_reference(catalog, state, s.country, input);
                stock < q * runs * b.stock_low_days
                    || price > reference.scale(b.expand_input_price_max)
            });
            if short || matches!(sl.limit, Some(Limit::Input(_) | Limit::Labor(_))) {
                held_back.push(recipe.product);
            }
            let flows = population::slot_flows(
                catalog,
                state,
                s.country,
                r,
                sl.count,
                sl.utilization,
                1.0 + s.wage_premium,
            );
            for &(input, q) in &flows.inputs {
                *own_use.entry(input).or_default() += q;
            }
            if recipe.extraction {
                *extracting.entry(recipe.product).or_default() +=
                    catalog.facilities.get(sl.facility).runs_per_day
                        * f64::from(sl.count)
                        * recipe.output;
            }
            let e = by_product
                .entry(flows.product)
                .or_insert((0.0, 0, Money::ZERO, 0.0));
            e.0 += sl.utilization * f64::from(sl.count);
            e.1 += sl.count;
            e.2 += flows.cost_per_day;
            e.3 += flows.output;
        }
        for (product, (weighted, count, daily, output)) in by_product {
            if count == 0 || output <= 1e-9 {
                continue;
            }
            let utilization = weighted / f64::from(count);
            let unit = daily.to_usd() / output;
            let offer = s.offers.get(&product);
            let price = offer.map_or_else(
                || market::market_price(catalog, state, s.country, product),
                |o| o.price,
            );
            let margin = price.to_usd() / unit.max(1e-9) - 1.0;
            // Sold this month, or used at the site, at least nine tenths of the output.
            let made = output * f64::from(date.day());
            let used = own_use.get(&product).copied().unwrap_or(0.0) * f64::from(date.day());
            let sold = offer.map_or(0.0, |o| o.sold_month);
            let sells = sold + used >= 0.9 * made;
            let mut add = (f64::from(count) * 0.25).round().max(1.0);
            if let Some(&made_full) = extracting.get(&product) {
                let allowed = s.deposit.and_then(|d| {
                    let field = state.deposits.get(d).concession_of(site)?;
                    Some(
                        catalog.max_output(d, date.year())
                            * state.settings.market_scale
                            * field.share
                            / 365.0,
                    )
                });
                let per_facility = made_full / f64::from(count);
                add = add.min(((allowed.unwrap_or(0.0) - made_full) / per_facility).floor());
            }
            if utilization >= min_utilization
                && sells
                && margin >= min_margin
                && add >= 1.0
                && !held_back.contains(&product)
                && best.is_none_or(|(m, ..)| margin > m)
            {
                // Small counts; the cast cannot overflow.
                best = Some((margin, site, product, add as u32));
            }
        }
    }
    let Some((_, site, product, mut count)) = best else {
        open_deposit(state, catalog, id, sites, date, news);
        return;
    };
    // Units of the product standing still at the site start up before new ones are
    // built (M22).
    let standing: Vec<(usize, u32)> = state.sites[site.index()]
        .slots
        .iter()
        .enumerate()
        .filter(|(_, sl)| {
            sl.mothballed()
                && sl
                    .recipe
                    .is_some_and(|r| catalog.recipes.get(r).product == product)
        })
        .map(|(i, sl)| (i, sl.count))
        .collect();
    let mut restarted = 0;
    for (slot, units) in standing {
        if restarted >= count {
            break;
        }
        if run(state, catalog, id, &Command::RestartFacility { site, slot }) {
            restarted += units;
        }
    }
    if restarted > 0 {
        return;
    }
    let kind = state.sites[site.index()].kind;
    let place = state.sites[site.index()].country;
    let Some(recipe) = best_recipe(catalog, state, id, product, None, Some(place)).filter(|&r| {
        catalog
            .facilities
            .get(catalog.recipes.get(r).facility)
            .site_type
            == kind
    }) else {
        return;
    };
    let facility = catalog.recipes.get(recipe).facility;
    let company = &state.companies[id.index()];
    let budget = (company.ledger.cash().max(Money::ZERO) + finance::credit_limit(catalog, company))
        .scale(b.invest_share_max);
    let unit = catalog.facilities.get(facility).investment;
    while count > 0 && unit.scale(f64::from(count)) > budget {
        count -= 1;
    }
    if count == 0 {
        return;
    }
    let cash_needed = unit.scale(f64::from(count)) - company.ledger.cash();
    if cash_needed > Money::ZERO {
        let years = b.loan_years.min(catalog.finance_model.max_term_years);
        run(
            state,
            catalog,
            id,
            &Command::TakeLoan {
                amount: cash_needed,
                years,
            },
        );
    }
    let build = Command::BuildFacility {
        site,
        facility,
        count,
    };
    if run(state, catalog, id, &build) {
        let slot = state.sites[site.index()].slots.len() - 1;
        let produce = Command::SetProduction {
            site,
            slot,
            recipe: Some(recipe),
            utilization: catalog.ai_model.start.utilization,
        };
        run(state, catalog, id, &produce);
        news.extend(news_expansion(state, catalog, id, site, recipe, count));
    }
}

/// Own electricity where the grid holds the company's plants back, as at the start
/// (M10): a power plant in the country for the electricity they lack. Returns whether
/// it built one.
fn own_power(
    state: &mut GameState,
    catalog: &Catalog,
    id: CompanyId,
    sites: &[SiteId],
    date: Date,
) -> bool {
    let Some(electricity) = catalog.production_model.electricity else {
        return false;
    };
    // Electricity lacking per country (MWh per day).
    let mut lacking: BTreeMap<CountryId, f64> = BTreeMap::new();
    for &site in sites {
        let s = &state.sites[site.index()];
        for sl in &s.slots {
            if !sl.operating(date) || sl.limit != Some(Limit::Electricity) {
                continue;
            }
            let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r)) else {
                continue;
            };
            let planned = catalog.facilities.get(sl.facility).runs_per_day
                * f64::from(sl.count)
                * sl.utilization;
            *lacking.entry(s.country).or_default() +=
                r.energy_mwh * (planned - sl.last_runs).max(0.0);
        }
    }
    let Some((country, mwh)) = lacking
        .into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
    else {
        return false;
    };
    if mwh <= 1e-9 {
        return false;
    }
    let Some(recipe) = best_recipe(catalog, state, id, electricity, None, Some(country)) else {
        return false;
    };
    let r = catalog.recipes.get(recipe);
    let f = catalog.facilities.get(r.facility);
    let per_plant = f.runs_per_day * r.output * catalog.ai_model.start.utilization;
    // Plant counts are small; the cast cannot overflow.
    let mut count = (mwh / per_plant.max(1e-9)).ceil().max(1.0) as u32;
    let b = &catalog.ai_model.behavior;
    let company = &state.companies[id.index()];
    let budget = (company.ledger.cash().max(Money::ZERO) + finance::credit_limit(catalog, company))
        .scale(b.invest_share_max);
    let site_cost = if sites.iter().any(|&s| {
        let s = &state.sites[s.index()];
        s.country == country && s.kind == f.site_type
    }) {
        Money::ZERO
    } else {
        catalog.production_model.site_cost(f.site_type)
    };
    while count > 0 && f.investment.scale(f64::from(count)) + site_cost > budget {
        count -= 1;
    }
    if count == 0 {
        return false;
    }
    let cash_needed = f.investment.scale(f64::from(count)) + site_cost - company.ledger.cash();
    if cash_needed > Money::ZERO {
        let years = b.loan_years.min(catalog.finance_model.max_term_years);
        run(
            state,
            catalog,
            id,
            &Command::TakeLoan {
                amount: cash_needed,
                years,
            },
        );
    }
    let existing = sites.iter().copied().find(|&s| {
        let s = &state.sites[s.index()];
        s.country == country && s.kind == f.site_type
    });
    let site = match existing {
        Some(site) => site,
        None => {
            let found = Command::FoundSite {
                country,
                kind: f.site_type,
            };
            if !run(state, catalog, id, &found) {
                return false;
            }
            site_id(state.sites.len() - 1)
        }
    };
    let build = Command::BuildFacility {
        site,
        facility: r.facility,
        count,
    };
    if !run(state, catalog, id, &build) {
        return false;
    }
    let slot = state.sites[site.index()].slots.len() - 1;
    run(
        state,
        catalog,
        id,
        &Command::SetProduction {
            site,
            slot,
            recipe: Some(recipe),
            utilization: catalog.ai_model.start.utilization,
        },
    );
    true
}

/// Advertising for every country and goods group where the company sold end products
/// last month: a share of that revenue (M16).
fn advertise(state: &mut GameState, catalog: &Catalog, id: CompanyId, sites: &[SiteId]) {
    let (_, aggressiveness) = traits(state, id);
    let share = catalog
        .ai_model
        .behavior
        .advertising_share
        .at(aggressiveness);
    let mut revenue: BTreeMap<(CountryId, crate::ids::GoodsGroupId), f64> = BTreeMap::new();
    for &site in sites {
        let s = &state.sites[site.index()];
        for (&product, offer) in &s.offers {
            let p = catalog.products.get(product);
            if p.kind == ProductKind::EndProduct && offer.sold_last_month > 0.0 {
                *revenue.entry((s.country, p.goods_group)).or_default() +=
                    offer.sold_last_month * offer.price.to_usd();
            }
        }
    }
    let company = &state.companies[id.index()];
    let mut commands = Vec::new();
    for (&(country, group), &r) in &revenue {
        let budget = Money::from_usd(share * r).unwrap_or(Money::ZERO);
        let old = company
            .advertising
            .iter()
            .find(|a| a.country == country && a.group == group)
            .map_or(Money::ZERO, |a| a.budget);
        if (budget.to_usd() - old.to_usd()).abs() > 0.1 * old.to_usd().max(1.0) {
            commands.push(Command::SetAdvertising {
                country,
                group,
                budget,
            });
        }
    }
    for a in &company.advertising {
        if !revenue.contains_key(&(a.country, a.group)) {
            commands.push(Command::SetAdvertising {
                country: a.country,
                group: a.group,
                budget: Money::ZERO,
            });
        }
    }
    for c in &commands {
        run(state, catalog, id, c);
    }
}

/// Rich companies build where the world's largest unserved demand is (the bottleneck
/// that also guides new companies): at most `diversifications_per_quarter` per quarter,
/// the richest first, each in another chain.
fn diversify(state: &mut GameState, catalog: &Catalog, news: &mut Vec<Message>) {
    let b = &catalog.ai_model.behavior;
    let mut rich: Vec<(Money, CompanyId)> = state
        .companies
        .iter()
        .enumerate()
        .filter(|(_, c)| c.ai.is_some() && !c.bankrupt)
        .map(|(i, c)| {
            let budget = (c.ledger.cash().max(Money::ZERO) + finance::credit_limit(catalog, c))
                .scale(b.invest_share_max);
            (budget, CompanyId(u32::try_from(i).unwrap_or(u32::MAX)))
        })
        .collect();
    rich.sort_by(|a, c| c.0.cmp(&a.0).then(a.1.cmp(&c.1)));
    let mut taken: Vec<ProductId> = Vec::new();
    let mut done = 0;
    for (budget, id) in rich {
        if done >= b.diversifications_per_quarter {
            break;
        }
        // With the processes this company knows: it may have researched one that is not
        // public yet (M22; cracking 1913 was public only in 1938).
        let Some((product, country, deposit, recipe, count)) =
            opportunity(state, catalog, &taken, id)
        else {
            continue;
        };
        let r = catalog.recipes.get(recipe);
        let f = catalog.facilities.get(r.facility);
        let fixed = catalog.production_model.site_cost(f.site_type)
            + deposit.map_or(Money::ZERO, |d| {
                catalog
                    .deposits
                    .get(d)
                    .development_cost
                    .scale(state.settings.market_scale)
            });
        let mut count = count;
        while count > 0 && fixed + f.investment.scale(f64::from(count)) > budget {
            count -= 1;
        }
        if count == 0 {
            // The richest cannot afford it; the others cannot either.
            break;
        }
        taken.push(product);
        if build_in_bottleneck(
            state,
            catalog,
            id,
            (product, country, deposit, recipe, count),
        ) {
            done += 1;
            news.extend(news_expansion(
                state,
                catalog,
                id,
                site_id(state.sites.len() - 1),
                recipe,
                count,
            ));
        }
    }
}

/// Builds a new site for an opportunity in an existing company (loan if needed).
fn build_in_bottleneck(
    state: &mut GameState,
    catalog: &Catalog,
    id: CompanyId,
    o: Opportunity,
) -> bool {
    let (product, country, deposit, recipe, count) = o;
    let b = &catalog.ai_model.behavior;
    let r = catalog.recipes.get(recipe);
    let f = catalog.facilities.get(r.facility);
    let cost =
        catalog.production_model.site_cost(f.site_type) + f.investment.scale(f64::from(count));
    let cash_needed = cost - state.companies[id.index()].ledger.cash();
    if cash_needed > Money::ZERO {
        let years = b.loan_years.min(catalog.finance_model.max_term_years);
        run(
            state,
            catalog,
            id,
            &Command::TakeLoan {
                amount: cash_needed,
                years,
            },
        );
    }
    let found = Command::FoundSite {
        country,
        kind: f.site_type,
    };
    if !run(state, catalog, id, &found) {
        return false;
    }
    let site = site_id(state.sites.len() - 1);
    if let Some(d) = deposit {
        run(
            state,
            catalog,
            id,
            &Command::DevelopDeposit { site, deposit: d },
        );
    }
    let build = Command::BuildFacility {
        site,
        facility: r.facility,
        count,
    };
    if !run(state, catalog, id, &build) {
        return false;
    }
    run(
        state,
        catalog,
        id,
        &Command::SetProduction {
            site,
            slot: 0,
            recipe: Some(recipe),
            utilization: catalog.ai_model.start.utilization,
        },
    );
    run(
        state,
        catalog,
        id,
        &Command::SetSale {
            site,
            product,
            mode: Some(PriceMode::Market {
                markup: 0.0,
                floor: Money::ZERO,
            }),
            keep: 0.0,
        },
    )
}

/// Whether the stocks of a product exceed `stock_high_days` of its current output: then
/// it lacks trade, not production.
fn piling(state: &GameState, catalog: &Catalog, product: ProductId) -> bool {
    let mut stock = 0.0;
    let mut output = 0.0;
    for s in &state.sites {
        if state.companies[s.owner.index()].bankrupt {
            continue;
        }
        stock += s.inventory.get(&product).map_or(0.0, |x| x.quantity);
        for sl in &s.slots {
            let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r)) else {
                continue;
            };
            // By-products count too: petrol from the stills is output, not a heap (M22).
            output += sl.last_runs
                * std::iter::once((r.product, r.output))
                    .chain(r.by_products.iter().copied())
                    .filter(|&(p, _)| p == product)
                    .map(|(_, q)| q)
                    .sum::<f64>();
        }
    }
    stock > catalog.ai_model.behavior.stock_high_days * output.max(1e-9)
}

/// An extraction company whose raw material is short opens a free concession of a
/// deposit of that raw material.
fn open_deposit(
    state: &mut GameState,
    catalog: &Catalog,
    id: CompanyId,
    sites: &[SiteId],
    date: Date,
    news: &mut Vec<Message>,
) {
    let model = &catalog.market_model;
    let b = &catalog.ai_model.behavior;
    let mut products: Vec<ProductId> = Vec::new();
    for &site in sites {
        for sl in &state.sites[site.index()].slots {
            if let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r))
                && r.extraction
                && !products.contains(&r.product)
            {
                products.push(r.product);
            }
        }
    }
    for product in products {
        let open: f64 = state
            .markets
            .get(product)
            .values()
            .map(|m| market::open_demand(m, model, date))
            .sum();
        let Some(recipe) = best_recipe(catalog, state, id, product, None, None) else {
            continue;
        };
        let per_day = population::output_per_day(catalog, recipe);
        if open < 0.5 * per_day || piling(state, catalog, product) {
            continue;
        }
        let Some((deposit, d)) = catalog.deposits.iter().find(|&(d, dep)| {
            dep.resource == product
                && dep.discovered.is_none_or(|y| y <= date.year())
                && state
                    .deposits
                    .get(d)
                    .concessions
                    .iter()
                    .any(|c| c.site.is_none())
        }) else {
            continue;
        };
        let facility = catalog.recipes.get(recipe).facility;
        let count = catalog.ai_model.plants_per_concession.round().max(1.0);
        let cost = catalog.facilities.get(facility).investment.scale(count)
            + d.development_cost.scale(state.settings.market_scale)
            + catalog.production_model.site_cost(SiteType::Extraction);
        let company = &state.companies[id.index()];
        let budget = (company.ledger.cash().max(Money::ZERO)
            + finance::credit_limit(catalog, company))
        .scale(b.invest_share_max);
        if cost > budget {
            continue;
        }
        let missing = cost - company.ledger.cash();
        if missing > Money::ZERO {
            let years = b.loan_years.min(catalog.finance_model.max_term_years);
            run(
                state,
                catalog,
                id,
                &Command::TakeLoan {
                    amount: missing,
                    years,
                },
            );
        }
        let found = Command::FoundSite {
            country: d.country,
            kind: SiteType::Extraction,
        };
        if !run(state, catalog, id, &found) {
            return;
        }
        let site = site_id(state.sites.len() - 1);
        run(
            state,
            catalog,
            id,
            &Command::DevelopDeposit { site, deposit },
        );
        // Plant counts are small; the cast cannot overflow.
        let build = Command::BuildFacility {
            site,
            facility,
            count: count as u32,
        };
        if run(state, catalog, id, &build) {
            let produce = Command::SetProduction {
                site,
                slot: 0,
                recipe: Some(recipe),
                utilization: catalog.ai_model.start.utilization,
            };
            run(state, catalog, id, &produce);
            let sell = Command::SetSale {
                site,
                product,
                mode: Some(PriceMode::Market {
                    markup: 0.0,
                    floor: Money::ZERO,
                }),
                keep: 0.0,
            };
            run(state, catalog, id, &sell);
            // Plant counts are small; the cast cannot overflow.
            news.extend(news_expansion(
                state,
                catalog,
                id,
                site,
                recipe,
                count as u32,
            ));
        }
        return;
    }
}

/// Competent companies with enough revenue research the technology their branch needs
/// next (cheapest first), in their own laboratory.
fn research_plan(
    state: &mut GameState,
    catalog: &Catalog,
    id: CompanyId,
    sites: &[SiteId],
    date: Date,
) {
    let b = &catalog.ai_model.behavior;
    let (competence, _) = traits(state, id);
    if competence < b.research_competence_min {
        return;
    }
    let company = &state.companies[id.index()];
    let revenue = company
        .ledger
        .years
        .last()
        .and_then(|y| y.by_type.get(&CostType::Revenue))
        .copied()
        .unwrap_or(Money::ZERO);
    if revenue.to_usd() < b.research_min_revenue_usd {
        return;
    }
    // Branches the company works in.
    let mut branches = Vec::new();
    for &site in sites {
        for sl in &state.sites[site.index()].slots {
            if let Some(r) = sl.recipe {
                let branch = catalog.products.get(catalog.recipes.get(r).product).branch;
                if !branches.contains(&branch) {
                    branches.push(branch);
                }
            }
        }
    }
    let horizon = f64::from(date.year()) + b.research_lookahead_years.at(competence);
    let target: Option<TechnologyId> = catalog
        .technologies
        .iter()
        .filter(|&(t, tech)| {
            f64::from(tech.invention_year) <= horizon
                && research::can_research(catalog, state, id, t)
                && catalog.recipes.values().any(|r| {
                    (r.technology == Some(t)
                        || catalog.facilities.get(r.facility).technology == Some(t))
                        && branches.contains(&catalog.products.get(r.product).branch)
                })
        })
        .filter_map(|(t, _)| research::effort(catalog, state, t, date).map(|e| (t, e.points)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(t, _)| t);
    let center = sites
        .iter()
        .copied()
        .find(|&s| state.sites[s.index()].kind == SiteType::ResearchCenter);
    let Some(target) = target else {
        return;
    };
    let center = match center {
        Some(c) => c,
        None => {
            let Some(lab) = catalog
                .facilities
                .iter()
                .find(|(_, f)| f.site_type == SiteType::ResearchCenter)
                .map(|(f, _)| f)
            else {
                return;
            };
            let country = state.companies[id.index()].headquarters;
            let found = Command::FoundSite {
                country,
                kind: SiteType::ResearchCenter,
            };
            if !run(state, catalog, id, &found) {
                return;
            }
            let site = site_id(state.sites.len() - 1);
            let build = Command::BuildFacility {
                site,
                facility: lab,
                count: 1,
            };
            if !run(state, catalog, id, &build) {
                return;
            }
            site
        }
    };
    if state.sites[center.index()].research != Some(target) {
        let command = Command::SetResearch {
            site: center,
            technology: Some(target),
        };
        run(state, catalog, id, &command);
    }
}

/// Bankrupt companies leave the market: their staff is released, their offers and
/// orders end and their concessions become free.
pub(crate) fn release_assets(state: &mut GameState, id: CompanyId) {
    for (i, s) in state.sites.iter_mut().enumerate() {
        if s.owner != id {
            continue;
        }
        for w in s.workforce.iter_mut().map(|(_, w)| w) {
            *w = 0.0;
        }
        s.offers.clear();
        s.orders.clear();
        s.research = None;
        s.staffing_due = false;
        if let Some(d) = s.deposit {
            let site = site_id(i);
            for c in &mut state.deposits.get_mut(d).concessions {
                if c.site == Some(site) {
                    c.site = None;
                    c.ready = None;
                    c.development_cost = Money::ZERO;
                }
            }
        }
    }
}

/// New companies take the place of bankrupt ones where demand is unserved.
fn found_companies(state: &mut GameState, catalog: &Catalog, date: Date, news: &mut Vec<Message>) {
    let wanted = usize::try_from(state.settings.ai.companies).unwrap_or(usize::MAX);
    let active = state
        .companies
        .iter()
        .filter(|c| c.ai.is_some() && !c.bankrupt)
        .count();
    let n = wanted
        .saturating_sub(active)
        .min(catalog.ai_model.behavior.foundings_per_month as usize);
    for _ in 0..n {
        let Some((product, country, deposit, recipe, count)) =
            opportunity(state, catalog, &[], CompanyId(u32::MAX))
        else {
            return;
        };
        if found_one(
            state,
            catalog,
            date,
            (product, country, deposit, recipe, count),
        ) {
            let company = state.companies.last().expect("just founded");
            news.push(
                Message::new(MessageKind::Info, keys::AI_FOUNDED)
                    .with("firma", Param::Text(company.name.clone()))
                    .with(
                        "land",
                        Param::Country(catalog.countries.key(country).to_owned()),
                    )
                    .with(
                        "produkt",
                        Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
                    ),
            );
        }
    }
}

type Opportunity = (ProductId, CountryId, Option<DepositId>, RecipeId, u32);

/// Where a new company is most needed: the product with the largest unserved demand
/// by value, or, if its inputs are short, the input that is the bottleneck. Raw
/// materials need a free concession; chains without one are left out. `builder` is the
/// company that would build (an unknown ID for a newcomer: public processes only).
fn opportunity(
    state: &GameState,
    catalog: &Catalog,
    taken: &[ProductId],
    builder: CompanyId,
) -> Option<Opportunity> {
    let model = &catalog.market_model;
    let u = catalog.ai_model.start.utilization;
    let open = |product: ProductId| -> (f64, CountryId, f64) {
        let mut total = 0.0;
        let mut value = 0.0;
        let mut best = (0.0, None);
        for (country, m) in state.markets.get(product).iter() {
            let o = market::open_demand(m, model, state.date);
            total += o;
            value += o * market::market_price(catalog, state, country, product).to_usd();
            if o > best.0 {
                best = (o, Some(country));
            }
        }
        (value, best.1.unwrap_or(CountryId::from_index(0)), total)
    };
    let mut candidates: Vec<(f64, ProductId)> = catalog
        .products
        .iter()
        .filter(|(_, p)| p.kind != ProductKind::Energy)
        .map(|(id, _)| (open(id).0, id))
        .filter(|(v, _)| *v > 0.0)
        .collect();
    candidates.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    'chains: for (_, top) in candidates {
        let mut product = top;
        for _ in 0..6 {
            let Some(recipe) = best_recipe(catalog, state, builder, product, None, None) else {
                continue 'chains;
            };
            let r = catalog.recipes.get(recipe);
            let runs = catalog.facilities.get(r.facility).runs_per_day * u;
            // The input whose unserved demand is largest relative to one new plant's need.
            // An input piling up somewhere is not short: it waits for traders (M22; crude
            // oil kept the petrol shortage of the 1920s from ever reaching a cracker).
            let short = r
                .inputs
                .iter()
                .filter(|(i, _)| catalog.products.get(*i).state_market.is_none())
                .filter(|(i, _)| !piling(state, catalog, *i))
                .map(|&(i, q)| (open(i).2 / (q * runs).max(1e-9), i))
                .filter(|(ratio, _)| *ratio > 0.5)
                .max_by(|a, b| a.0.total_cmp(&b.0));
            if let Some((_, input)) = short {
                product = input;
                continue;
            }
            // Products piling up somewhere need trade, not new companies.
            if piling(state, catalog, product) || taken.contains(&product) {
                continue 'chains;
            }
            let (_, country, total) = open(product);
            let mut place = country;
            let mut deposit = None;
            if r.extraction {
                let usable = |d: DepositId| {
                    let dep = catalog.deposits.get(d);
                    dep.resource == product
                        && dep.discovered.is_none_or(|y| y <= state.date.year())
                        && state
                            .deposits
                            .get(d)
                            .concessions
                            .iter()
                            .any(|c| c.site.is_none())
                };
                // A deposit in the country itself, otherwise anywhere.
                let Some((d, dep)) = catalog
                    .deposits
                    .iter()
                    .find(|&(d, dep)| usable(d) && dep.country == country)
                    .or_else(|| catalog.deposits.iter().find(|&(d, _)| usable(d)))
                else {
                    continue 'chains;
                };
                place = dep.country;
                deposit = Some(d);
            }
            let per_day = population::output_per_day(catalog, recipe) * u;
            // Small counts; the cast cannot overflow.
            let count = (total / per_day.max(1e-9)).round().clamp(1.0, 20.0) as u32;
            return Some((product, place, deposit, recipe, count));
        }
    }
    None
}

/// Founds a company for an opportunity; true when it could start production.
fn found_one(state: &mut GameState, catalog: &Catalog, date: Date, o: Opportunity) -> bool {
    let (product, country, deposit, recipe, count) = o;
    let model = &catalog.ai_model;
    let r = catalog.recipes.get(recipe);
    let f = catalog.facilities.get(r.facility);
    let mut investment =
        f.investment.scale(f64::from(count)) + catalog.production_model.site_cost(f.site_type);
    if let Some(d) = deposit {
        investment += catalog
            .deposits
            .get(d)
            .development_cost
            .scale(state.settings.market_scale);
    }
    let capital = investment.scale(model.behavior.founding_capital_factor);
    let index = u32::try_from(state.companies.len()).unwrap_or(u32::MAX);
    let id = CompanyId(index);
    let mut rng = SimRng::for_stream(state.settings.seed, Stream::Company(index));
    let settings = state.settings.ai;
    let spread = model.trait_spread;
    let competence = (settings.competence + spread * (2.0 * rng.next_f64() - 1.0)).clamp(0.0, 1.0);
    let aggressiveness =
        (settings.aggressiveness + spread * (2.0 * rng.next_f64() - 1.0)).clamp(0.0, 1.0);
    let branch = catalog.products.get(product).branch;
    let name = population::company_name(state, catalog, &mut rng, country, branch);
    state.companies.push(Company {
        brands: Vec::new(),
        advertising: Vec::new(),
        owners: crate::state::Stake::sole(crate::state::Holder::Private),
        name,
        kind: CompanyKind::Ai,
        headquarters: country,
        founded: date,
        rng,
        ledger: Ledger::new(date, capital),
        technologies: Default::default(),
        bankrupt: false,
        loans: Vec::new(),
        loss_carryforward: Money::ZERO,
        sales_policies: Vec::new(),
        research: Default::default(),
        ai: Some(AiState {
            competence,
            aggressiveness,
            real: None,
            next_operations: date.add_days(1),
        }),
    });
    let found = Command::FoundSite {
        country,
        kind: f.site_type,
    };
    if !run(state, catalog, id, &found) {
        return false;
    }
    let site = site_id(state.sites.len() - 1);
    if let Some(d) = deposit {
        run(
            state,
            catalog,
            id,
            &Command::DevelopDeposit { site, deposit: d },
        );
    }
    let build = Command::BuildFacility {
        site,
        facility: r.facility,
        count,
    };
    if !run(state, catalog, id, &build) {
        return false;
    }
    let produce = Command::SetProduction {
        site,
        slot: 0,
        recipe: Some(recipe),
        utilization: model.start.utilization,
    };
    run(state, catalog, id, &produce);
    let sell = Command::SetSale {
        site,
        product,
        mode: Some(PriceMode::Market {
            markup: 0.0,
            floor: Money::ZERO,
        }),
        keep: 0.0,
    };
    run(state, catalog, id, &sell)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::test_support;

    /// An AI company with a works of ten furnaces making iron at `utilization`; it sells
    /// that share of their full output.
    fn idle_works(utilization: f64) -> (crate::game::Game, CompanyId, SiteId) {
        idle_works_in(test_support::production(), utilization)
    }

    fn idle_works_in(catalog: Catalog, utilization: f64) -> (crate::game::Game, CompanyId, SiteId) {
        use crate::state::{GameSettings, Slot, StartForm};
        use std::sync::Arc;
        let catalog = Arc::new(catalog);
        let aaa = catalog.countries.id("AAA").expect("exists");
        let settings = GameSettings {
            seed: 3,
            start_year: 1900,
            start_country: aaa,
            start_capital: Money::from_usd(1_000_000.0).expect("valid"),
            start_form: StartForm::Workshop,
            company_name: "Test AG".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: Default::default(),
        };
        let mut game = crate::game::Game::new(catalog.clone(), settings).expect("valid");
        let state = game.state_mut();
        let date = state.date;
        let index = state.companies.len();
        let id = company_id(index);
        let cost = Money::from_usd(20_000_000.0).expect("valid");
        let mut ledger = Ledger::new(date, Money::from_usd(5_000_000.0).expect("valid"));
        ledger.transfer(
            crate::ledger::Account::FixedAssets,
            crate::ledger::Account::Equity,
            cost,
        );
        state.companies.push(Company {
            brands: Vec::new(),
            advertising: Vec::new(),
            owners: crate::state::Stake::sole(crate::state::Holder::Private),
            name: "Hütte KI".into(),
            kind: CompanyKind::Ai,
            headquarters: aaa,
            founded: date,
            rng: SimRng::for_stream(3, Stream::Company(u32::try_from(index).expect("small"))),
            ledger,
            technologies: Default::default(),
            bankrupt: false,
            loans: Vec::new(),
            loss_carryforward: Money::ZERO,
            sales_policies: Vec::new(),
            research: Default::default(),
            ai: Some(AiState {
                competence: 0.5,
                aggressiveness: 0.5,
                real: None,
                next_operations: date,
            }),
        });
        let found = Command::FoundSite {
            country: aaa,
            kind: SiteType::Factory,
        };
        assert!(run(state, &catalog, id, &found));
        let site = site_id(state.sites.len() - 1);
        // Iron sells at half its reference price: the market has more than it wants.
        let iron = catalog.products.id("eisen").expect("exists");
        let reference = market::local_reference(&catalog, state, aaa, iron);
        state.markets.get_mut(iron).get_mut(aaa).price = reference.scale(0.5);
        state.sites[site.index()].slots.push(Slot {
            facility: catalog.facilities.id("ofen").expect("exists"),
            ready: date,
            count: 10,
            cost,
            recipe: catalog.recipes.id("eisen_schmelzen"),
            utilization,
            automation: 0.0,
            condition: 1.0,
            batches: Vec::new(),
            last_runs: 0.0,
            limit: None,
            operation: Operation::Running,
        });
        // Ten furnaces make 500 t a day; last month sold the planned share.
        let price = reference.scale(0.5);
        state.sites[site.index()].offers.insert(
            iron,
            crate::state::SaleOffer {
                mode: PriceMode::Market {
                    markup: 0.0,
                    floor: Money::ZERO,
                },
                price,
                keep: 0.0,
                sold_today: 0.0,
                sold_month: 0.0,
                sold_last_month: 500.0 * utilization * 30.0,
                to_traders_month: 0.0,
                to_companies_month: 0.0,
            },
        );
        (game, id, site)
    }

    #[test]
    fn idle_capacity_is_shut_down_and_sold_later() {
        let (mut game, id, site) = idle_works(0.2);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        let date = state.date;
        let mut news = Vec::new();
        retire(state, &catalog, id, &[site], date, &mut news);
        // A fifth of ten furnaces is two; at 80 % three are needed, seven stand still.
        let slots = &state.sites[site.index()].slots;
        assert_eq!(slots.len(), 2);
        assert_eq!((slots[0].count, slots[1].count), (3, 7));
        assert!(slots[1].mothballed());
        // The three make what the ten made.
        assert!((slots[0].utilization - 2.0 / 3.0).abs() < 1e-9);

        // Two years and a month later the seven are sold (at the start of a month the
        // last month's sales tell the rate).
        let cash = state.companies[id.index()].ledger.cash();
        state.date = Date::new(1902, 2, 1).expect("valid");
        let later = state.date;
        retire(state, &catalog, id, &[site], later, &mut news);
        let slots = &state.sites[site.index()].slots;
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].count, 3);
        assert!(state.companies[id.index()].ledger.cash() > cash);
        assert!(state.companies[id.index()].ledger.is_balanced());
    }

    #[test]
    fn busy_capacity_stays() {
        let (mut game, id, site) = idle_works(0.6);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        let date = state.date;
        retire(state, &catalog, id, &[site], date, &mut Vec::new());
        assert_eq!(state.sites[site.index()].slots.len(), 1);
        assert_eq!(state.sites[site.index()].slots[0].count, 10);
    }

    #[test]
    fn capacity_stays_where_the_market_pays_the_reference_price() {
        let (mut game, id, site) = idle_works(0.2);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        let date = state.date;
        let aaa = catalog.countries.id("AAA").expect("exists");
        let iron = catalog.products.id("eisen").expect("exists");
        let reference = market::local_reference(&catalog, state, aaa, iron);
        state.markets.get_mut(iron).get_mut(aaa).price = reference;
        retire(state, &catalog, id, &[site], date, &mut Vec::new());
        assert_eq!(state.sites[site.index()].slots.len(), 1);
    }

    #[test]
    fn by_products_are_not_sold_below_their_worth_as_fuel() {
        let mut catalog = test_support::production();
        // Ore burns like coal (8 MWh/t); smelting gives off a gas of 12 MWh/t.
        let ore = catalog.products.id("erz").expect("exists");
        let iron = catalog.products.id("eisen").expect("exists");
        catalog.products.get_mut(ore).heating_value_mwh = Some(8.0);
        let mut gas = catalog.products.get(iron).clone();
        gas.heating_value_mwh = Some(12.0);
        let gas = catalog.products.insert("gas", gas).expect("new");
        let smelting = catalog.recipes.id("eisen_schmelzen").expect("exists");
        catalog.recipes.get_mut(smelting).by_products = vec![(gas, 0.1)];

        let (mut game, id, site) = idle_works_in(catalog, 0.6);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        let date = state.date;
        let aaa = catalog.countries.id("AAA").expect("exists");
        state.sites[site.index()]
            .inventory
            .entry(gas)
            .or_default()
            .add(50.0, Money::ZERO, 50.0);
        operate(state, &catalog, id, &[site], date);
        let ore_price = market::market_price(&catalog, state, aaa, ore).to_usd();
        let offer = state.sites[site.index()].offers.get(&gas).expect("offered");
        let PriceMode::Market { floor, .. } = offer.mode else {
            panic!("market price expected");
        };
        assert!(
            (floor.to_usd() - 12.0 * ore_price / 8.0).abs() < 0.01,
            "{floor:?}"
        );
        // Products without a heating value have no such floor.
        assert_eq!(fuel_value(&catalog, state, aaa, iron), Money::ZERO);
    }

    #[test]
    fn wage_premium_rises_while_short_of_staff_and_falls_back() {
        let catalog = test_support::production();
        let b = &catalog.ai_model.behavior;
        let mut premium = 0.0;
        for _ in 0..100 {
            premium = next_wage_premium(&catalog, premium, true);
        }
        assert!((premium - b.wage_premium_max).abs() < 1e-9, "{premium}");
        premium = next_wage_premium(&catalog, premium, false);
        assert!((premium - (b.wage_premium_max - b.wage_premium_step)).abs() < 1e-9);
        for _ in 0..100 {
            premium = next_wage_premium(&catalog, premium, false);
        }
        assert_eq!(premium, 0.0);
    }
}
