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
use crate::catalog::{Catalog, FacilitySize, ProductKind, Recipe, SiteType};
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
    let gaps = if first_of_year {
        gap_technologies(state, catalog)
    } else {
        Vec::new()
    };
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
            news.extend(crate::deals::ai_offers(state, catalog, id));
        }
        if end_of_quarter {
            retire(state, catalog, id, own, date, &mut news);
            expand(state, catalog, id, own, date, &mut news);
        }
        if first_of_year {
            research_plan(state, catalog, id, own, date, &gaps);
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

/// Counts in the AI rules (researchers per gap, foundings, diversifications, makers of a
/// dear market) are meant for the default number of companies; with more companies the
/// markets are larger (market scale), so the counts grow alike. 0 stays off.
fn per_companies(state: &GameState, catalog: &Catalog, count: u32) -> usize {
    let m = &catalog.ai_model;
    let factor =
        (f64::from(state.settings.ai.companies) / f64::from(m.default_companies.max(1))).max(1.0);
    // Counts stay far below u32::MAX; the cast saturates anyway.
    (f64::from(count) * factor).round() as usize
}

/// Units of a facility to build: the facility, their size (M36) and how many.
type Units = (FacilityId, FacilitySize, u32);

/// Size and number of units for `wanted` capacity in units of the data size (M36): the
/// plan of the size model, as many of them as `room` (units of a size that fit on the
/// plot) and the budget hold with `fixed` costs on top; else the next smaller size.
fn plan_units(
    catalog: &Catalog,
    want: (FacilityId, f64),
    room: impl Fn(FacilitySize) -> u32,
    money: (Money, Money),
) -> Option<(FacilitySize, u32)> {
    plan_units_by(catalog, want, room, money, f64::round)
}

/// As `plan_units`, but with enough units to reach the wanted capacity (own power).
fn plan_units_up(
    catalog: &Catalog,
    want: (FacilityId, f64),
    room: impl Fn(FacilitySize) -> u32,
    money: (Money, Money),
) -> Option<(FacilitySize, u32)> {
    plan_units_by(catalog, want, room, money, f64::ceil)
}

fn plan_units_by(
    catalog: &Catalog,
    (facility, wanted): (FacilityId, f64),
    room: impl Fn(FacilitySize) -> u32,
    (fixed, budget): (Money, Money),
    round: fn(f64) -> f64,
) -> Option<(FacilitySize, u32)> {
    let sizes = &catalog.production_model.sizes;
    let investment = catalog.facilities.get(facility).investment;
    let (mut size, _) = sizes.units_for(wanted);
    loop {
        // Small counts; the cast cannot overflow.
        let count = round(wanted / sizes.capacity(size)).max(1.0) as u32;
        let unit = investment.scale(sizes.investment(size));
        let mut n = count.min(room(size));
        while n > 0 && fixed + unit.scale(f64::from(n)) > budget {
            n -= 1;
        }
        if n > 0 {
            return Some((size, n));
        }
        size = size.smaller()?;
    }
}

/// A loan for what the cash lacks, if anything.
fn take_loan(state: &mut GameState, catalog: &Catalog, id: CompanyId, amount: Money) {
    if amount > Money::ZERO {
        let b = &catalog.ai_model.behavior;
        let years = b.loan_years.min(catalog.finance_model.max_term_years);
        run(state, catalog, id, &Command::TakeLoan { amount, years });
    }
}

/// Where a company would found a site of `kind` for `units` of a facility (M35): the
/// command and how many of the units fit. Extraction sites need no plot; other sites go
/// on the free plot that holds the units with the reserve of the plot model (the
/// cheapest per ha, else the largest free one), bought while the cash keeps
/// `kasse_min_monate` of running costs, else leased. `None` without a plot for one unit.
fn site_plan(
    state: &GameState,
    catalog: &Catalog,
    id: CompanyId,
    (country, kind): (CountryId, SiteType),
    (units, revenue): (Option<Units>, f64),
) -> Option<(Command, u32)> {
    let wanted = units.map_or(u32::MAX, |(_, _, n)| n);
    if !crate::plots::needs_plot(catalog, kind) {
        return Some((Command::FoundSite { country, kind }, wanted));
    }
    let m = &catalog.plot_model;
    let need = units.map_or(m.min_site_area_ha, |(f, size, n)| {
        crate::plots::project_area(catalog, f, size, n)
    }) * (1.0 + m.ai_reserve);
    let plot = crate::plots::choose(catalog, state, country, need, revenue)?;
    let area = state.plots[plot.index()].area_ha;
    let fit = units.map_or(u32::MAX, |(f, size, n)| {
        let per_unit = crate::plots::unit_area(catalog, f, size) * (1.0 + m.overhead);
        // Small counts; the cast saturates.
        let fit = if per_unit > 0.0 {
            (area / per_unit + 1e-9).floor() as u32
        } else {
            n
        };
        if area + 1e-9 < m.min_site_area_ha {
            0
        } else {
            fit.min(n)
        }
    });
    if fit == 0 {
        return None;
    }
    let own: Vec<SiteId> = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.owner == id)
        .map(|(i, _)| site_id(i))
        .collect();
    let price =
        catalog.production_model.site_cost(kind) + crate::plots::value(catalog, state, plot);
    let reserve =
        daily_cost(catalog, state, &own).scale(30.0 * catalog.ai_model.behavior.cash_min_months);
    let lease = state.companies[id.index()].ledger.cash() < price + reserve;
    Some((Command::FoundSiteOnPlot { plot, kind, lease }, fit))
}

/// Founds a site as `site_plan` says; the new site and how many units fit there.
fn found_site_for(
    state: &mut GameState,
    catalog: &Catalog,
    id: CompanyId,
    place: (CountryId, SiteType),
    project: (Option<Units>, f64),
) -> Option<(SiteId, u32)> {
    let (found, fit) = site_plan(state, catalog, id, place, project)?;
    run(state, catalog, id, &found).then(|| (site_id(state.sites.len() - 1), fit))
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
    (size, count): (FacilitySize, u32),
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
                "groesse",
                Param::TextKey(format!("anlagengroesse.{}", size.key())),
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
        let (full_output, taken) = output_and_offtake(state, catalog, site, sites, date);
        restart_where_short(state, catalog, id, site, (&full_output, &taken));
        let (full_output, taken) = output_and_offtake(state, catalog, site, sites, date);
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
            let flows = population::slot_flows(
                catalog,
                state,
                s.country,
                r,
                (sl.count, sl.size),
                1.0,
                (wage, developed(state, catalog, s.owner, r)),
            );
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
                let output = sl.full_runs(catalog) * r.output;
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

pub(crate) fn traits(state: &GameState, id: CompanyId) -> (f64, f64) {
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
        .map(|(rid, r)| {
            let mut cost = population::reference_unit_cost(catalog, rid, energy);
            // At a site, inputs dearer than their reference count with the premium (M33):
            // a process whose input is scarce must not replace one that runs (all
            // stocking mills switched to nylon while nylon cost three times its reference).
            if let Some(c) = country {
                for &(input, q) in &r.inputs {
                    let premium = market::market_price(catalog, state, c, input).to_usd()
                        - market::local_reference(catalog, state, c, input).to_usd();
                    cost += q * premium.max(0.0) / r.output.max(1e-9);
                }
            }
            (rid, cost)
        })
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
        let (full_output, taken) = output_and_offtake(state, catalog, site, sites, date);
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
                let planned = sl.full_runs(catalog) * sl.utilization;
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
                (sl.count, sl.size),
                utilization,
                (
                    1.0 + s.wage_premium,
                    developed(state, catalog, s.owner, recipe),
                ),
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
            let full = sl.full_runs(catalog)
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
    own: &[SiteId],
    date: Date,
) -> (BTreeMap<ProductId, f64>, BTreeMap<ProductId, f64>) {
    let s = &state.sites[site.index()];
    let mut full_output: BTreeMap<ProductId, f64> = BTreeMap::new();
    let mut taken: BTreeMap<ProductId, f64> = BTreeMap::new();
    for sl in s.slots.iter().filter(|sl| sl.operating(date)) {
        let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r)) else {
            continue;
        };
        let runs = sl.full_runs(catalog);
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
    // What the company's other sites use of the goods offered here (M32): they get them
    // by transfer, not over the market, and a works that fed only its own company's plants
    // saw no buyers and ran down (aero engines for the company's aircraft works). Shared
    // equally by the company's sites that make the goods.
    for &product in s.offers.keys() {
        let makers = own
            .iter()
            .filter(|&&o| {
                state.sites[o.index()].slots.iter().any(|sl| {
                    sl.operating(date)
                        && sl
                            .recipe
                            .is_some_and(|r| catalog.recipes.get(r).product == product)
                })
            })
            .count();
        if makers == 0 {
            continue;
        }
        let used: f64 = own
            .iter()
            .filter(|&&o| o != site)
            .flat_map(|&o| &state.sites[o.index()].slots)
            .filter(|sl| sl.operating(date))
            .filter_map(|sl| sl.recipe.map(|r| (sl, catalog.recipes.get(r))))
            .map(|(sl, r)| {
                let runs = sl.full_runs(catalog) * sl.utilization;
                r.inputs
                    .iter()
                    .filter(|&&(i, _)| i == product)
                    .map(|&(_, q)| runs * q)
                    .sum::<f64>()
            })
            .sum();
        // Few sites; the cast is exact.
        *taken.entry(product).or_default() += used / makers as f64;
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
                    (sl.count, sl.size),
                    sl.utilization,
                    (1.0 + s.wage_premium, developed(state, catalog, s.owner, r)),
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
    // Best expansion: margin, site, product, wanted capacity and the most the
    // concession allows (in units of the data size, M36).
    let mut best: Option<(f64, SiteId, ProductId, f64, f64)> = None;
    for &site in sites {
        let s = &state.sites[site.index()];
        let mut by_product: BTreeMap<ProductId, (f64, f64, Money, f64)> = BTreeMap::new();
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
            let runs = sl.full_runs(catalog) * sl.utilization;
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
                (sl.count, sl.size),
                sl.utilization,
                (1.0 + s.wage_premium, developed(state, catalog, s.owner, r)),
            );
            for &(input, q) in &flows.inputs {
                *own_use.entry(input).or_default() += q;
            }
            if recipe.extraction {
                *extracting.entry(recipe.product).or_default() +=
                    sl.full_runs(catalog) * recipe.output;
            }
            let e = by_product
                .entry(flows.product)
                .or_insert((0.0, 0.0, Money::ZERO, 0.0));
            e.0 += sl.utilization * sl.units(catalog);
            e.1 += sl.units(catalog);
            e.2 += flows.cost_per_day;
            e.3 += flows.output;
        }
        for (product, (weighted, capacity, daily, output)) in by_product {
            if capacity <= 1e-9 || output <= 1e-9 {
                continue;
            }
            let utilization = weighted / capacity;
            let unit = daily.to_usd() / output;
            let offer = s.offers.get(&product);
            let price = offer.map_or_else(
                || market::market_price(catalog, state, s.country, product),
                |o| o.price,
            );
            let margin = price.to_usd() / unit.max(1e-9) - 1.0;
            // Sold since the start of last month, or used at the site, at least nine tenths
            // of the output: one month alone missed goods sold a few pieces at a time
            // (airliners, M33).
            let days = 30.0 + f64::from(date.day());
            let made = output * days;
            let used = own_use.get(&product).copied().unwrap_or(0.0) * days;
            let sold = offer.map_or(0.0, |o| o.sold_last_month + o.sold_month);
            let sells = sold + used >= 0.9 * made;
            // Small works double, large ones grow by a quarter, at least by one unit of
            // the data size (M36).
            let add = (capacity * 0.25).max(capacity.min(1.0));
            let mut cap = f64::INFINITY;
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
                let per_unit = made_full / capacity;
                cap = (allowed.unwrap_or(0.0) - made_full) / per_unit;
            }
            let smallest = catalog
                .production_model
                .sizes
                .capacity(FacilitySize::VerySmall);
            if utilization >= min_utilization
                && sells
                && margin >= min_margin
                && add.min(cap) + 1e-9 >= smallest
                && !held_back.contains(&product)
                && best.is_none_or(|(m, ..)| margin > m)
            {
                best = Some((margin, site, product, add.min(cap), cap));
            }
        }
    }
    let Some((_, site, product, add, cap)) = best else {
        open_deposit(state, catalog, id, sites, date, news);
        return;
    };
    // Units of the product standing still at the site start up before new ones are
    // built (M22).
    let standing: Vec<(usize, f64)> = state.sites[site.index()]
        .slots
        .iter()
        .enumerate()
        .filter(|(_, sl)| {
            sl.mothballed()
                && sl
                    .recipe
                    .is_some_and(|r| catalog.recipes.get(r).product == product)
        })
        .map(|(i, sl)| (i, sl.units(catalog)))
        .collect();
    let mut restarted = 0.0;
    for (slot, units) in standing {
        if restarted >= add {
            break;
        }
        if run(state, catalog, id, &Command::RestartFacility { site, slot }) {
            restarted += units;
        }
    }
    if restarted > 0.0 {
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
    let sizes = &catalog.production_model.sizes;
    let company = &state.companies[id.index()];
    let budget = (company.ledger.cash().max(Money::ZERO) + finance::credit_limit(catalog, company))
        .scale(b.invest_share_max);
    // Units of a size the concession still allows (extraction).
    let within = |size: FacilitySize| -> u32 {
        if cap.is_finite() {
            // Small counts; the cast saturates.
            (cap / sizes.capacity(size) + 1e-9).floor().max(0.0) as u32
        } else {
            u32::MAX
        }
    };
    let plan_at = |o: SiteId| {
        plan_units(
            catalog,
            (facility, add),
            |size| {
                crate::plots::units_that_fit(catalog, state, o, (facility, size)).min(within(size))
            },
            (Money::ZERO, budget),
        )
    };
    // A full plot (M35): another own site of the kind in the country with room – one
    // already making the product first, then the one with the most room – else a new
    // plot. Without this, the full oldest site stayed the best one and every expansion
    // founded a site of its own (185 one-mill sites of one company by 1965).
    let target = plan_at(site).map(|p| (site, p)).or_else(|| {
        let makes = |o: SiteId| {
            state.sites[o.index()].slots.iter().any(|sl| {
                sl.recipe
                    .is_some_and(|r| catalog.recipes.get(r).product == product)
            })
        };
        sites
            .iter()
            .copied()
            .filter(|&o| {
                let s = &state.sites[o.index()];
                o != site && s.country == place && s.kind == kind
            })
            .filter_map(|o| {
                let room = crate::plots::units_that_fit(
                    catalog,
                    state,
                    o,
                    (facility, FacilitySize::Medium),
                );
                plan_at(o).map(|p| (makes(o), room, std::cmp::Reverse(o), p))
            })
            .max_by(|a, b| (a.0, a.1, a.2).cmp(&(b.0, b.1, b.2)))
            .map(|(.., std::cmp::Reverse(o), p)| (o, p))
    });
    let Some((site, (size, count))) = target else {
        let Some(units) = affordable(state, catalog, (None, recipe, (add, cap)), budget) else {
            return;
        };
        let o = (product, place, None, recipe, units);
        if build_in_bottleneck(state, catalog, id, o) {
            let new = site_id(state.sites.len() - 1);
            news.extend(news_expansion(state, catalog, id, new, recipe, units));
        }
        return;
    };
    let unit = catalog
        .facilities
        .get(facility)
        .investment
        .scale(sizes.investment(size));
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
        size,
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
        if !state.sites[site.index()].offers.contains_key(&product) {
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
        }
        news.extend(news_expansion(
            state,
            catalog,
            id,
            site,
            recipe,
            (size, count),
        ));
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
            let planned = sl.full_runs(catalog) * sl.utilization;
            *lacking.entry(s.country).or_default() +=
                r.energy_mwh * (planned - sl.last_runs).max(0.0);
        }
    }
    // Plants still being built cover their share once they run; without them every
    // decision during the construction time would add another plant.
    for &site in sites {
        let s = &state.sites[site.index()];
        if s.kind != SiteType::PowerPlant {
            continue;
        }
        let Some(lack) = lacking.get_mut(&s.country) else {
            continue;
        };
        for sl in s
            .slots
            .iter()
            .filter(|sl| !sl.operating(date) && !sl.mothballed())
        {
            let output = sl.recipe.map_or(1.0, |r| catalog.recipes.get(r).output);
            *lack -= sl.full_runs(catalog) * output * catalog.ai_model.start.utilization;
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
    // Enough plants for the electricity lacking, in units of the data size (M36).
    let wanted = mwh / per_plant.max(1e-9);
    let b = &catalog.ai_model.behavior;
    let company = &state.companies[id.index()];
    let budget = (company.ledger.cash().max(Money::ZERO) + finance::credit_limit(catalog, company))
        .scale(b.invest_share_max);
    let unit_of = |size: FacilitySize| {
        f.investment
            .scale(catalog.production_model.sizes.investment(size))
    };
    // An existing power plant site of the country while its plot has room (M35), else a
    // new one.
    let existing = sites.iter().copied().find_map(|s| {
        let st = &state.sites[s.index()];
        if st.country != country || st.kind != f.site_type {
            return None;
        }
        let room = |size| crate::plots::units_that_fit(catalog, state, s, (r.facility, size));
        plan_units_up(catalog, (r.facility, wanted), room, (Money::ZERO, budget))
            .map(|units| (s, units))
    });
    let (site, size, count) = match existing {
        Some((site, (size, count))) => {
            let cash_needed = unit_of(size).scale(f64::from(count)) - company.ledger.cash();
            take_loan(state, catalog, id, cash_needed);
            (site, size, count)
        }
        None => {
            let site_cost = catalog.production_model.site_cost(f.site_type);
            let Some((size, count)) = plan_units_up(
                catalog,
                (r.facility, wanted),
                |_| u32::MAX,
                (site_cost, budget),
            ) else {
                return false;
            };
            let cash_needed =
                unit_of(size).scale(f64::from(count)) + site_cost - company.ledger.cash();
            take_loan(state, catalog, id, cash_needed);
            let place = (country, f.site_type);
            let Some((site, fit)) = found_site_for(
                state,
                catalog,
                id,
                place,
                (Some((r.facility, size, count)), 0.0),
            ) else {
                return false;
            };
            (site, size, count.min(fit))
        }
    };
    let build = Command::BuildFacility {
        site,
        facility: r.facility,
        count,
        size,
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
    let most = per_companies(
        state,
        catalog,
        catalog.ai_model.behavior.diversifications_per_quarter,
    );
    pioneer(state, catalog, &by_budget(state, catalog), news);
    let mut taken: Vec<ProductId> = Vec::new();
    let mut done = 0;
    let mut scan = Scan::new(state, catalog);
    for (budget, id) in by_budget(state, catalog) {
        if done >= most {
            break;
        }
        // With the processes this company knows: it may have researched one that is not
        // public yet (M22; cracking 1913 was public only in 1938).
        let Some((product, country, deposit, recipe, want)) =
            opportunity_in(&scan, state, catalog, &taken, id, None)
        else {
            continue;
        };
        let Some(units) = affordable(state, catalog, (deposit, recipe, want), budget) else {
            // The richest cannot afford it; the others cannot either.
            break;
        };
        taken.push(product);
        let built = build_in_bottleneck(
            state,
            catalog,
            id,
            (product, country, deposit, recipe, units),
        );
        scan = Scan::new(state, catalog);
        if built {
            done += 1;
            news.extend(news_expansion(
                state,
                catalog,
                id,
                site_id(state.sites.len() - 1),
                recipe,
                units,
            ));
        }
    }
}

/// AI companies by investment budget, the richest first.
fn by_budget(state: &GameState, catalog: &Catalog) -> Vec<(Money, CompanyId)> {
    let b = &catalog.ai_model.behavior;
    let mut rich: Vec<(Money, CompanyId)> = state
        .companies
        .iter()
        .enumerate()
        .filter(|(_, c)| c.ai.is_some() && !c.bankrupt)
        .map(|(i, c)| {
            let budget = (c.ledger.cash().max(Money::ZERO) + finance::credit_limit(catalog, c))
                .scale(b.invest_share_max);
            (budget, company_id(i))
        })
        .collect();
    rich.sort_by(|a, c| c.0.cmp(&a.0).then(a.1.cmp(&c.1)));
    rich
}

/// Products in demand that no site makes yet (M32): the richest company that can make
/// one – or the bottleneck below it – builds, one product per company and quarter. New
/// processes would otherwise wait until one of the richest companies learned them.
fn pioneer(
    state: &mut GameState,
    catalog: &Catalog,
    rich: &[(Money, CompanyId)],
    news: &mut Vec<Message>,
) {
    let mut made: Vec<ProductId> = Vec::new();
    for s in &state.sites {
        if state.companies[s.owner.index()].bankrupt {
            continue;
        }
        for r in s.slots.iter().filter_map(|sl| sl.recipe) {
            let r = catalog.recipes.get(r);
            for p in std::iter::once(r.product).chain(r.by_products.iter().map(|&(b, _)| b)) {
                if !made.contains(&p) {
                    made.push(p);
                }
            }
        }
    }
    let new: Vec<ProductId> = catalog
        .products
        .ids()
        .filter(|p| !made.contains(p))
        .collect();
    let mut taken: Vec<ProductId> = Vec::new();
    let mut scan = Scan::new(state, catalog);
    for &(budget, id) in rich {
        let Some((product, country, deposit, recipe, want)) =
            opportunity_in(&scan, state, catalog, &taken, id, Some(&new))
        else {
            continue;
        };
        let Some(units) = affordable(state, catalog, (deposit, recipe, want), budget) else {
            continue;
        };
        taken.push(product);
        let built = build_in_bottleneck(
            state,
            catalog,
            id,
            (product, country, deposit, recipe, units),
        );
        scan = Scan::new(state, catalog);
        if built {
            news.extend(news_expansion(
                state,
                catalog,
                id,
                site_id(state.sites.len() - 1),
                recipe,
                units,
            ));
        }
    }
}

/// Size and number of facilities for the wanted capacity that a budget pays for, with
/// the site and the development of the deposit (M36: a smaller size if not even one
/// unit); `None` if not even the smallest.
fn affordable(
    state: &GameState,
    catalog: &Catalog,
    (deposit, recipe, (wanted, cap)): (Option<DepositId>, RecipeId, Want),
    budget: Money,
) -> Option<(FacilitySize, u32)> {
    let facility = catalog.recipes.get(recipe).facility;
    let f = catalog.facilities.get(facility);
    let fixed = catalog.production_model.site_cost(f.site_type)
        + deposit.map_or(Money::ZERO, |d| {
            catalog
                .deposits
                .get(d)
                .development_cost
                .scale(state.settings.market_scale)
        });
    plan_units(
        catalog,
        (facility, wanted),
        |size| within_cap(catalog, cap, size),
        (fixed, budget),
    )
}

/// Units of a size that stay within `cap` capacity (units of the data size; infinite for
/// no limit).
fn within_cap(catalog: &Catalog, cap: f64, size: FacilitySize) -> u32 {
    if cap.is_finite() {
        // Small counts; the cast saturates.
        (cap / catalog.production_model.sizes.capacity(size) + 1e-9)
            .floor()
            .max(0.0) as u32
    } else {
        u32::MAX
    }
}

/// Builds a new site for an opportunity in an existing company (loan if needed).
fn build_in_bottleneck(
    state: &mut GameState,
    catalog: &Catalog,
    id: CompanyId,
    o: Planned,
) -> bool {
    let (product, country, deposit, recipe, (size, count)) = o;
    let b = &catalog.ai_model.behavior;
    let r = catalog.recipes.get(recipe);
    let f = catalog.facilities.get(r.facility);
    let unit = f
        .investment
        .scale(catalog.production_model.sizes.investment(size));
    let cost = catalog.production_model.site_cost(f.site_type) + unit.scale(f64::from(count));
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
    let place = (country, f.site_type);
    let capacity = f64::from(count) * catalog.production_model.sizes.capacity(size);
    let revenue = crate::plots::project_revenue(state, catalog, country, recipe, capacity);
    let Some((site, fit)) = found_site_for(
        state,
        catalog,
        id,
        place,
        (Some((r.facility, size, count)), revenue),
    ) else {
        return false;
    };
    let count = count.min(fit);
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
        size,
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
    piling_all(state, catalog)[product.index()]
}

/// `piling` for every product in one pass over the sites (the search for bottlenecks
/// asks for many products in turn).
fn piling_all(state: &GameState, catalog: &Catalog) -> Vec<bool> {
    let n = catalog.products.len();
    let mut stock = vec![0.0; n];
    let mut output = vec![0.0; n];
    let mut made_here: Vec<usize> = Vec::new();
    for s in &state.sites {
        if state.companies[s.owner.index()].bankrupt {
            continue;
        }
        made_here.clear();
        for sl in &s.slots {
            let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r)) else {
                continue;
            };
            // By-products count too: petrol from the stills is output, not a heap (M22).
            let outputs =
                || std::iter::once((r.product, r.output)).chain(r.by_products.iter().copied());
            for (i, (product, _)) in outputs().enumerate() {
                // Each product once per slot, its quantities summed in order.
                if outputs().take(i).any(|(p, _)| p == product) {
                    continue;
                }
                let made = outputs()
                    .filter(|&(p, _)| p == product)
                    .map(|(_, q)| q)
                    .sum::<f64>();
                if made > 0.0 && !made_here.contains(&product.index()) {
                    made_here.push(product.index());
                }
                output[product.index()] += sl.last_runs * made;
            }
        }
        // Only what sellers hold: the stocks plants keep of their inputs are no heap
        // (M33; the refineries' crude oil hid the shortage of the 1960s).
        for (&product, stock_of) in &s.inventory {
            let p = product.index();
            if made_here.contains(&p) || s.offers.contains_key(&product) {
                stock[p] += stock_of.quantity;
            }
        }
    }
    let high = catalog.ai_model.behavior.stock_high_days;
    (0..n)
        .map(|p| stock[p] > high * output[p].max(1e-9))
        .collect()
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
        // Where the most of it is wanted (M33; before: the first deposit in the data).
        let demand = state
            .markets
            .get(product)
            .iter()
            .map(|(c, m)| (market::open_demand(m, model, date), c))
            .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)))
            .map_or(CountryId::from_index(0), |(_, c)| c);
        let Some(deposit) = deposit_for(state, catalog, product, demand) else {
            continue;
        };
        let d = catalog.deposits.get(deposit);
        let facility = catalog.recipes.get(recipe).facility;
        let sizes = &catalog.production_model.sizes;
        let cap = concession_capacity(state, catalog, deposit, recipe);
        let wanted = catalog
            .ai_model
            .plants_per_concession
            .round()
            .max(1.0)
            .min(cap);
        let (size, count) = sizes.units_for(wanted);
        let count = count.min(within_cap(catalog, cap, size)).max(1);
        let cost = catalog
            .facilities
            .get(facility)
            .investment
            .scale(sizes.investment(size) * f64::from(count))
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
        let build = Command::BuildFacility {
            site,
            facility,
            count,
            size,
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
            news.extend(news_expansion(
                state,
                catalog,
                id,
                site,
                recipe,
                (size, count),
            ));
        }
        return;
    }
}

/// What the development level of a company changes in a recipe (M37).
fn developed(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    recipe: RecipeId,
) -> crate::development::Effect {
    crate::development::effect(catalog, state, company, catalog.recipes.get(recipe).product)
}

/// Whether an AI company is competent and large enough to research (M10).
pub(crate) fn wants_research(state: &GameState, catalog: &Catalog, id: CompanyId) -> bool {
    let b = &catalog.ai_model.behavior;
    let (competence, _) = traits(state, id);
    let revenue = state.companies[id.index()]
        .ledger
        .years
        .last()
        .and_then(|y| y.by_type.get(&CostType::Revenue))
        .copied()
        .unwrap_or(Money::ZERO);
    competence >= b.research_competence_min && revenue.to_usd() >= b.research_min_revenue_usd
}

/// Competent companies with enough revenue research the technology their branch needs
/// next or one for a market gap (cheapest first), in their own laboratory.
fn research_plan(
    state: &mut GameState,
    catalog: &Catalog,
    id: CompanyId,
    sites: &[SiteId],
    date: Date,
    gaps: &[TechnologyId],
) {
    let b = &catalog.ai_model.behavior;
    let (competence, _) = traits(state, id);
    if !wants_research(state, catalog, id) {
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
    let view: &GameState = state;
    let cheapest = |qualifies: &dyn Fn(TechnologyId) -> bool| -> Option<TechnologyId> {
        catalog
            .technologies
            .iter()
            .filter(|&(t, tech)| {
                f64::from(tech.invention_year) <= horizon
                    && research::can_research(catalog, view, id, t)
                    && qualifies(t)
            })
            .filter_map(|(t, _)| research::effort(catalog, view, t, date).map(|e| (t, e.points)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(t, _)| t)
    };
    let own_branch = |t: TechnologyId| {
        catalog.recipes.values().any(|r| {
            (r.technology == Some(t) || catalog.facilities.get(r.facility).technology == Some(t))
                && branches.contains(&catalog.products.get(r.product).branch)
        })
    };
    // Market gaps (M32) only without work in the own branches, and by a few companies
    // at a time; a company keeps the gap it works on.
    let gap = |t: TechnologyId| {
        let mine = sites
            .iter()
            .any(|&s| view.sites[s.index()].research == Some(t));
        let others = view
            .sites
            .iter()
            .filter(|s| s.research == Some(t) && !view.companies[s.owner.index()].bankrupt)
            .count();
        gaps.contains(&t)
            && (mine || others < per_companies(view, catalog, b.research_gap_companies))
    };
    let target = cheapest(&own_branch).or_else(|| cheapest(&gap));
    let center = sites
        .iter()
        .copied()
        .find(|&s| state.sites[s.index()].kind == SiteType::ResearchCenter);
    // Nothing to research: develop the best-selling product further (M37).
    let develop = if target.is_none() {
        development_target(state, catalog, id, center)
    } else {
        None
    };
    if target.is_none() && develop.is_none() {
        return;
    }
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
            let place = (country, SiteType::ResearchCenter);
            let Some((site, _)) = found_site_for(
                state,
                catalog,
                id,
                place,
                (Some((lab, FacilitySize::Medium, 1)), 0.0),
            ) else {
                return;
            };
            let build = Command::BuildFacility {
                site,
                facility: lab,
                count: 1,
                size: FacilitySize::Medium,
            };
            if !run(state, catalog, id, &build) {
                return;
            }
            site
        }
    };
    let command = match (target, develop) {
        (Some(t), _) if state.sites[center.index()].research != Some(t) => Command::SetResearch {
            site: center,
            technology: Some(t),
        },
        (None, Some(p)) if state.sites[center.index()].development != Some(p) => {
            Command::SetDevelopment {
                site: center,
                product: Some(p),
            }
        }
        _ => return,
    };
    run(state, catalog, id, &command);
}

/// The product an AI company develops when it has nothing to research (M37): the one it
/// is developing while its next level still pays, else its best seller of the last year
/// that sells at least the research revenue threshold and whose next level pays for
/// itself within the payback years.
fn development_target(
    state: &GameState,
    catalog: &Catalog,
    id: CompanyId,
    center: Option<SiteId>,
) -> Option<ProductId> {
    use crate::development;
    let b = &catalog.ai_model.behavior;
    let company = &state.companies[id.index()];
    let year = company.ledger.years.last()?;
    let country = center.map_or(company.headquarters, |c| state.sites[c.index()].country);
    let revenue = |p: ProductId| year.product_type(p, CostType::Revenue).to_usd();
    let pays = |p: ProductId| {
        let Some((_, needed)) = development::next_effort(catalog, state, id, p, state.date) else {
            return false;
        };
        let Some((field, _)) = development::basis(catalog, p) else {
            return false;
        };
        development::cost_per_point(catalog, state, country, field).is_some_and(|per_point| {
            revenue(p) * b.development_benefit_per_level * b.development_payback_years
                >= needed * per_point
        })
    };
    let worth = |p: ProductId| {
        revenue(p) >= b.research_min_revenue_usd
            && development::can_develop(catalog, state, id, p)
            && pays(p)
    };
    let current = center.and_then(|c| state.sites[c.index()].development);
    if let Some(p) = current
        && worth(p)
    {
        return Some(p);
    }
    let mut products: Vec<ProductId> = year.by_center.keys().filter_map(|c| c.product).collect();
    products.sort_unstable();
    products.dedup();
    products
        .into_iter()
        .filter(|&p| worth(p))
        .map(|p| (p, revenue(p)))
        .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
        .map(|(p, _)| p)
}

/// Bankrupt companies leave the market: their staff is released, their offers and
/// orders end and their concessions become free.
pub(crate) fn release_assets(state: &mut GameState, id: CompanyId) {
    stop_operations(state, id);
    let given_up: Vec<SiteId> = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.owner == id)
        .map(|(i, _)| site_id(i))
        .collect();
    for site in given_up {
        give_up_site(state, site);
    }
}

/// An insolvent company stops working: its staff is released, its offers and purchases
/// end, its research stops (M38: its sites wait for the auction).
pub(crate) fn stop_operations(state: &mut GameState, id: CompanyId) {
    for s in state.sites.iter_mut().filter(|s| s.owner == id) {
        for w in s.workforce.iter_mut().map(|(_, w)| w) {
            *w = 0.0;
        }
        s.offers.clear();
        s.orders.clear();
        s.research = None;
        s.development = None;
        s.staffing_due = false;
    }
}

/// A site nobody took over: its plot and its concession become free.
pub(crate) fn give_up_site(state: &mut GameState, site: SiteId) {
    crate::plots::release(state, site);
    if let Some(d) = state.sites[site.index()].deposit {
        for c in &mut state.deposits.get_mut(d).concessions {
            if c.site == Some(site) {
                c.site = None;
                c.ready = None;
                c.development_cost = Money::ZERO;
            }
        }
    }
}

/// New companies take the place of bankrupt ones where demand is unserved.
fn found_companies(state: &mut GameState, catalog: &Catalog, date: Date, news: &mut Vec<Message>) {
    let wanted = usize::try_from(state.settings.ai.companies).unwrap_or(usize::MAX);
    // A company whose sites are being auctioned still counts (M38): its works stand still
    // for the auction only, and a newcomer in that gap would double the capacity.
    let active = state
        .companies
        .iter()
        .filter(|c| c.ai.is_some() && (!c.bankrupt || c.auction_until.is_some()))
        .count();
    let n = wanted.saturating_sub(active).min(per_companies(
        state,
        catalog,
        catalog.ai_model.behavior.foundings_per_month,
    ));
    for _ in 0..n {
        let Some((product, country, deposit, recipe, count)) =
            opportunity(state, catalog, &[], CompanyId(u32::MAX), None)
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

/// Wanted capacity in units of the data size and the most a concession allows (M36;
/// infinite without a limit).
type Want = (f64, f64);

/// A product to make, where, from which deposit, with which recipe and how much.
type Opportunity = (ProductId, CountryId, Option<DepositId>, RecipeId, Want);

/// An opportunity with its size and number of facilities.
type Planned = (
    ProductId,
    CountryId,
    Option<DepositId>,
    RecipeId,
    (FacilitySize, u32),
);

/// The companies making each product (main product of a recipe set on a facility,
/// bankrupt companies left out).
fn makers(state: &GameState, catalog: &Catalog) -> BTreeMap<ProductId, Vec<CompanyId>> {
    let mut makers: BTreeMap<ProductId, Vec<CompanyId>> = BTreeMap::new();
    for s in &state.sites {
        if state.companies[s.owner.index()].bankrupt {
            continue;
        }
        for r in s.slots.iter().filter_map(|sl| sl.recipe) {
            let owners = makers.entry(catalog.recipes.get(r).product).or_default();
            if !owners.contains(&s.owner) {
                owners.push(s.owner);
            }
        }
    }
    makers
}

/// Last month's sales of a product in all countries.
struct Sales {
    sold: f64,
    /// Revenue (USD).
    paid: f64,
    /// The same quantities at the reference price of each country (USD).
    reference: f64,
    /// The country where buyers paid the most above the reference.
    dearest: Option<CountryId>,
}

fn last_month_sales(state: &GameState, catalog: &Catalog, product: ProductId) -> Sales {
    let mut s = Sales {
        sold: 0.0,
        paid: 0.0,
        reference: 0.0,
        dearest: None,
    };
    let mut best = 0.0;
    for (country, m) in state.markets.get(product).iter() {
        let q = m.last_month.sold;
        if q <= 1e-9 {
            continue;
        }
        let p = m.last_month.revenue.to_usd();
        let r = q * market::local_reference(catalog, state, country, product).to_usd();
        s.sold += q;
        s.paid += p;
        s.reference += r;
        if p - r > best {
            best = p - r;
            s.dearest = Some(country);
        }
    }
    s
}

/// Whether buyers paid at least `entry_price_factor` times the reference price last
/// month (M33).
fn is_dear(state: &GameState, catalog: &Catalog, product: ProductId) -> bool {
    let s = last_month_sales(state, catalog, product);
    s.sold > 1e-9 && s.paid >= s.reference * catalog.ai_model.behavior.entry_price_factor
}

/// Freight per unit of a product (USD); unreachable countries cost infinitely much.
fn freight(
    state: &GameState,
    catalog: &Catalog,
    product: ProductId,
    from: CountryId,
    to: CountryId,
) -> f64 {
    state
        .routes
        .for_product(catalog, product, from, to)
        .map_or(f64::INFINITY, |(cost, _)| cost.to_usd())
}

/// The deposit of a raw material where a new concession goes: discovered, with a free
/// field whose remaining reserve lasts `reserve_years_min` years of its full output (M33:
/// fields of exhausted deposits stood still), and the cheapest way to the country that
/// wants it (M32).
fn deposit_for(
    state: &GameState,
    catalog: &Catalog,
    product: ProductId,
    demand: CountryId,
) -> Option<DepositId> {
    let scale = state.settings.market_scale;
    let year = state.date.year();
    let years = catalog.ai_model.behavior.reserve_years_min;
    let usable = |d: DepositId| {
        let dep = catalog.deposits.get(d);
        let ds = state.deposits.get(d);
        let Some(field) = ds.concessions.iter().find(|c| c.site.is_none()) else {
            return false;
        };
        let lasts = dep.reserve.is_none_or(|reserve| {
            reserve * scale - ds.extracted
                >= catalog.max_output(d, year) * scale * field.share * years
        });
        dep.resource == product && dep.discovered.is_none_or(|y| y <= year) && lasts
    };
    catalog
        .deposits
        .iter()
        .filter(|&(d, _)| usable(d))
        .min_by(|a, b| {
            let to_demand = |c: CountryId| freight(state, catalog, product, c, demand);
            to_demand(a.1.country)
                .total_cmp(&to_demand(b.1.country))
                .then(a.0.cmp(&b.0))
        })
        .map(|(d, _)| d)
}

/// Capacity of an extraction recipe (units of the data size, in steps of the smallest
/// size, M36) that a free concession of a deposit keeps busy at the start utilization,
/// at least the smallest size (M33: twenty oil wells on a field that allowed two used
/// up its year in weeks and stood still for the rest).
fn concession_capacity(
    state: &GameState,
    catalog: &Catalog,
    deposit: DepositId,
    recipe: RecipeId,
) -> f64 {
    let share = state
        .deposits
        .get(deposit)
        .concessions
        .iter()
        .find(|c| c.site.is_none())
        .map_or(0.0, |c| c.share);
    let allowed =
        catalog.max_output(deposit, state.date.year()) * state.settings.market_scale * share
            / 365.0;
    let per_unit = population::output_per_day(catalog, recipe) * catalog.ai_model.start.utilization;
    let smallest = catalog
        .production_model
        .sizes
        .capacity(FacilitySize::VerySmall);
    ((allowed / per_unit.max(1e-9) / smallest).ceil() * smallest).clamp(smallest, 20.0)
}

/// Whether some active company could use a recipe: it knows the technologies of the
/// recipe and of its facility.
fn usable_somewhere(state: &GameState, catalog: &Catalog, recipe: &Recipe) -> bool {
    let known = |t: Option<TechnologyId>| {
        t.is_none_or(|t| {
            (0..state.companies.len())
                .any(|i| !state.companies[i].bankrupt && state.knows(catalog, company_id(i), t))
        })
    };
    known(recipe.technology) && known(catalog.facilities.get(recipe.facility).technology)
}

/// A recipe for a product that some site uses, or else that some active company knows
/// (M32): the chain to a bottleneck goes through it.
fn recipe_known_somewhere(
    state: &GameState,
    catalog: &Catalog,
    product: ProductId,
) -> Option<RecipeId> {
    state
        .sites
        .iter()
        .flat_map(|s| &s.slots)
        .filter_map(|sl| sl.recipe)
        .find(|&r| catalog.recipes.get(r).product == product)
        .or_else(|| {
            catalog
                .recipes
                .iter()
                .find(|(_, r)| r.product == product && usable_somewhere(state, catalog, r))
                .map(|(id, _)| id)
        })
}

/// Technologies for market gaps (M32): products that consumers or the state ask for, and
/// the inputs of their usable recipes, that no active company can make – the
/// technologies of their recipes and facilities with all prerequisites.
fn gap_technologies(state: &GameState, catalog: &Catalog) -> Vec<TechnologyId> {
    // Dear markets with few makers (M33): their processes are worth learning although
    // a company knows them; they become common property only decades after their
    // invention, so newcomers would otherwise never enter (penicillin, airliners).
    let b = &catalog.ai_model.behavior;
    let makers = makers(state, catalog);
    let dear: Vec<ProductId> = catalog
        .products
        .ids()
        .filter(|p| {
            let n = makers.get(p).map_or(0, Vec::len);
            n > 0
                && n < per_companies(state, catalog, b.entry_companies_max)
                && is_dear(state, catalog, *p)
        })
        .collect();
    let mut todo: Vec<ProductId> = catalog
        .products
        .ids()
        .filter(|&p| {
            state
                .markets
                .get(p)
                .iter()
                .any(|(_, m)| m.state_rate > 0.0 || m.consumer_rate.iter().any(|&r| r > 0.0))
        })
        .collect();
    let mut seen = todo.clone();
    let mut techs: Vec<TechnologyId> = Vec::new();
    while let Some(product) = todo.pop() {
        let recipes: Vec<&Recipe> = catalog
            .recipes
            .values()
            .filter(|r| r.product == product)
            .collect();
        let usable: Vec<&Recipe> = recipes
            .iter()
            .copied()
            .filter(|r| usable_somewhere(state, catalog, r))
            .collect();
        if usable.is_empty() || dear.contains(&product) {
            let mut needed: Vec<TechnologyId> = recipes
                .iter()
                .flat_map(|r| [r.technology, catalog.facilities.get(r.facility).technology])
                .flatten()
                .collect();
            while let Some(t) = needed.pop() {
                if !techs.contains(&t) {
                    techs.push(t);
                    needed.extend(catalog.technologies.get(t).prerequisites.iter().copied());
                }
            }
        }
        for r in usable {
            for &(input, _) in &r.inputs {
                if !seen.contains(&input) {
                    seen.push(input);
                    todo.push(input);
                }
            }
        }
    }
    techs
}

/// What the search for opportunities reads of the world apart from the company that
/// looks. It stays valid until something is built: with hundreds of companies asking
/// in turn, the search was most of a quarter's work.
struct Scan {
    makers: BTreeMap<ProductId, Vec<CompanyId>>,
    piling: Vec<bool>,
    /// Per product: value of the unserved demand per day, the country with the most and
    /// the quantity, less what facilities under construction will make.
    unserved: Vec<(f64, Option<CountryId>, f64)>,
    /// Per product: the same for a newcomer to a dear market (M33), for a company that
    /// does not make the product yet.
    dear: Vec<(f64, Option<CountryId>, f64)>,
}

impl Scan {
    fn new(state: &GameState, catalog: &Catalog) -> Self {
        let model = &catalog.market_model;
        // Facilities under construction will serve part of the demand (M32): without
        // this, the same bottleneck was built again every quarter until the first plant
        // ran.
        let u = catalog.ai_model.start.utilization;
        let mut coming: BTreeMap<ProductId, f64> = BTreeMap::new();
        for s in &state.sites {
            let owner = &state.companies[s.owner.index()];
            // Works being auctioned will run again under a new owner (M38).
            let auctioned = owner.bankrupt && owner.auction_until.is_some();
            if owner.bankrupt && !auctioned {
                continue;
            }
            for sl in s
                .slots
                .iter()
                .filter(|sl| sl.ready > state.date || (auctioned && !sl.mothballed()))
            {
                if let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r)) {
                    *coming.entry(r.product).or_default() += sl.full_runs(catalog) * r.output * u;
                }
            }
        }
        let makers = makers(state, catalog);
        let unserved = |product: ProductId| -> (f64, Option<CountryId>, f64) {
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
            let left = (total - coming.get(&product).copied().unwrap_or(0.0)).max(0.0);
            let value = if total > 0.0 {
                value * left / total
            } else {
                0.0
            };
            (value, best.1, left)
        };
        // Markets that pay far more than the reference price draw newcomers (M33): a
        // single producer earning several times its costs otherwise stayed alone for
        // decades (penicillin, nylon). The newcomer plans for a share of last month's
        // sales, in the country where buyers pay the most above the reference.
        let b = &catalog.ai_model.behavior;
        let last = state.date.first_of_month().add_days(-1);
        let days = f64::from(crate::calendar::days_in_month(last.year(), last.month()));
        let entry_max = per_companies(state, catalog, b.entry_companies_max);
        let dear = |product: ProductId| -> (f64, Option<CountryId>, f64) {
            let owners = makers.get(&product).map_or(&[][..], Vec::as_slice);
            if owners.is_empty() || owners.len() >= entry_max || !is_dear(state, catalog, product) {
                return (0.0, None, 0.0);
            }
            let s = last_month_sales(state, catalog, product);
            let quantity = (b.entry_share * s.sold / days
                - coming.get(&product).copied().unwrap_or(0.0))
            .max(0.0);
            (quantity * s.paid / s.sold, s.dearest, quantity)
        };
        let unserved = catalog.products.ids().map(unserved).collect();
        let dear = catalog.products.ids().map(dear).collect();
        Self {
            piling: piling_all(state, catalog),
            makers,
            unserved,
            dear,
        }
    }

    /// Unserved demand of a product as `builder` sees it: value per day, country with
    /// the most, quantity.
    fn open(&self, product: ProductId, builder: CompanyId) -> (f64, CountryId, f64) {
        let (value, country, left) = self.unserved[product.index()];
        let makes = self
            .makers
            .get(&product)
            .is_some_and(|owners| owners.contains(&builder));
        let (more, dear_country, quantity) = if makes {
            (0.0, None, 0.0)
        } else {
            self.dear[product.index()]
        };
        let country = country
            .filter(|_| left > 0.0)
            .or(dear_country)
            .or(country)
            .unwrap_or(CountryId::from_index(0));
        (value + more, country, left + quantity)
    }
}

/// Where a new company is most needed: the product with the largest unserved demand
/// by value, or, if its inputs are short, the input that is the bottleneck. Raw
/// materials need a free concession; chains without one are left out. `builder` is the
/// company that would build (an unknown ID for a newcomer: public processes only);
/// `only` limits the products at the top of the chains.
fn opportunity(
    state: &GameState,
    catalog: &Catalog,
    taken: &[ProductId],
    builder: CompanyId,
    only: Option<&[ProductId]>,
) -> Option<Opportunity> {
    opportunity_in(
        &Scan::new(state, catalog),
        state,
        catalog,
        taken,
        builder,
        only,
    )
}

/// `opportunity` with the world as `scan` saw it (unchanged since).
fn opportunity_in(
    scan: &Scan,
    state: &GameState,
    catalog: &Catalog,
    taken: &[ProductId],
    builder: CompanyId,
    only: Option<&[ProductId]>,
) -> Option<Opportunity> {
    let open = |product: ProductId| scan.open(product, builder);
    let mut candidates: Vec<(f64, ProductId)> = catalog
        .products
        .iter()
        .filter(|(id, p)| p.kind != ProductKind::Energy && only.is_none_or(|o| o.contains(id)))
        .map(|(id, _)| (open(id).0, id))
        .filter(|(v, _)| *v > 0.0)
        .collect();
    candidates.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let chain = Chain {
        state,
        catalog,
        taken,
        builder,
        piling: &scan.piling,
        open: &open,
    };
    candidates
        .into_iter()
        .find_map(|(_, top)| chain.bottleneck(top, 0))
}

/// The search for a bottleneck below a product with unserved demand.
struct Chain<'a> {
    state: &'a GameState,
    catalog: &'a Catalog,
    taken: &'a [ProductId],
    builder: CompanyId,
    /// `piling` per product.
    piling: &'a [bool],
    /// Unserved demand of a product: value per day, country with the most, quantity.
    open: &'a dyn Fn(ProductId) -> (f64, CountryId, f64),
}

impl Chain<'_> {
    /// Freight per unit of a product (USD); unreachable countries cost infinitely much.
    fn freight(&self, product: ProductId, from: CountryId, to: CountryId) -> f64 {
        freight(self.state, self.catalog, product, from, to)
    }

    /// Where a works for an input of other works goes (M32): the country with the most
    /// unserved demand, unless the inputs would travel far – then the country among those
    /// where an input is made or mined that minimises the freight of inputs (each from
    /// its nearest source) and output (to the demand). Weight-losing processes such as
    /// alumina from bauxite thus move to the raw material.
    fn site_for(&self, r: &Recipe, demand: CountryId) -> CountryId {
        let (state, catalog) = (self.state, self.catalog);
        let year = state.date.year();
        let mut inputs: Vec<(ProductId, f64, Vec<CountryId>)> = Vec::new();
        for &(input, q) in &r.inputs {
            if catalog.products.get(input).state_market.is_some() {
                continue;
            }
            let mut from: Vec<CountryId> = state
                .sites
                .iter()
                .filter(|s| {
                    !state.companies[s.owner.index()].bankrupt
                        && s.slots.iter().any(|sl| {
                            sl.recipe
                                .is_some_and(|x| catalog.recipes.get(x).product == input)
                        })
                })
                .map(|s| s.country)
                .chain(
                    catalog
                        .deposits
                        .values()
                        .filter(|d| d.resource == input && d.discovered.is_none_or(|y| y <= year))
                        .map(|d| d.country),
                )
                .collect();
            from.sort();
            from.dedup();
            if !from.is_empty() {
                inputs.push((input, q, from));
            }
        }
        let cost = |c: CountryId| -> f64 {
            inputs
                .iter()
                .map(|(input, q, from)| {
                    q * from
                        .iter()
                        .map(|&s| self.freight(*input, s, c))
                        .fold(f64::INFINITY, f64::min)
                })
                .sum::<f64>()
                + r.output * self.freight(r.product, c, demand)
        };
        let mut candidates: Vec<CountryId> = inputs
            .iter()
            .flat_map(|(_, _, from)| from.iter().copied())
            .collect();
        candidates.sort();
        candidates.dedup();
        let mut best = (cost(demand), demand);
        for c in candidates {
            let k = cost(c);
            if k < best.0 {
                best = (k, c);
            }
        }
        best.1
    }

    /// The product itself, or – if an input is short – the bottleneck below the shortest
    /// input whose chain leads somewhere, then the next (M32: coal short at every alumina
    /// works hid the missing bauxite mine). At most six stages.
    fn bottleneck(&self, product: ProductId, depth: u32) -> Option<Opportunity> {
        let (state, catalog) = (self.state, self.catalog);
        if depth >= 6 {
            return None;
        }
        // The chain is followed with a recipe some company uses or knows where the builder
        // cannot make the product itself; it builds only what it can make.
        let own = best_recipe(catalog, state, self.builder, product, None, None);
        let recipe = own.or_else(|| recipe_known_somewhere(state, catalog, product))?;
        let r = catalog.recipes.get(recipe);
        let u = catalog.ai_model.start.utilization;
        let runs = catalog.facilities.get(r.facility).runs_per_day * u;
        // Inputs whose unserved demand exceeds half of one new plant's need, shortest
        // first. An input piling up somewhere is not short: it waits for traders (M22;
        // crude oil kept the petrol shortage of the 1920s from ever reaching a cracker).
        let mut short: Vec<(f64, ProductId)> = r
            .inputs
            .iter()
            .filter(|(i, _)| catalog.products.get(*i).state_market.is_none())
            .filter(|(i, _)| !self.piling[i.index()])
            .map(|&(i, q)| ((self.open)(i).2 / (q * runs).max(1e-9), i))
            .filter(|(ratio, _)| *ratio > 0.5)
            .collect();
        if !short.is_empty() {
            short.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
            return short
                .into_iter()
                .find_map(|(_, input)| self.bottleneck(input, depth + 1));
        }
        // Products piling up somewhere need trade, not new companies.
        if self.piling[product.index()] || self.taken.contains(&product) {
            return None;
        }
        let recipe = own?;
        let r = catalog.recipes.get(recipe);
        let (_, country, total) = (self.open)(product);
        let place;
        let mut deposit = None;
        if r.extraction {
            let d = deposit_for(state, catalog, product, country)?;
            place = catalog.deposits.get(d).country;
            deposit = Some(d);
        } else {
            // Goods for consumers and the state are made where they are wanted: traders
            // hardly bring them into poorer countries with their low price levels (mills
            // far from Indian demand were built again and again).
            let p = catalog.products.get(product);
            place = if p.consumer_demand.is_none() && p.state_demand.is_none() {
                self.site_for(r, country)
            } else {
                country
            };
        }
        let per_day = population::output_per_day(catalog, recipe) * u;
        // Capacity in units of the data size (M36), from the smallest size to twenty.
        let smallest = catalog
            .production_model
            .sizes
            .capacity(FacilitySize::VerySmall);
        let wanted = (total / per_day.max(1e-9)).clamp(smallest, 20.0);
        let cap = deposit.map_or(f64::INFINITY, |d| {
            concession_capacity(state, catalog, d, recipe)
        });
        Some((product, place, deposit, recipe, (wanted.min(cap), cap)))
    }
}

/// Founds a company for an opportunity; true when it could start production.
fn found_one(state: &mut GameState, catalog: &Catalog, date: Date, o: Opportunity) -> bool {
    let (product, country, deposit, recipe, (wanted, cap)) = o;
    let model = &catalog.ai_model;
    let r = catalog.recipes.get(recipe);
    let f = catalog.facilities.get(r.facility);
    let sizes = &catalog.production_model.sizes;
    // The capital follows the plan; there is no budget to keep within.
    let (size, count) = sizes.units_for(wanted);
    let count = count.min(within_cap(catalog, cap, size)).max(1);
    let capacity = f64::from(count) * sizes.capacity(size);
    let mut investment = f
        .investment
        .scale(sizes.investment(size) * f64::from(count))
        + catalog.production_model.site_cost(f.site_type);
    if let Some(d) = deposit {
        investment += catalog
            .deposits
            .get(d)
            .development_cost
            .scale(state.settings.market_scale);
    }
    // Without a plot for the works there is no company (M35); its capital buys the land.
    if crate::plots::needs_plot(catalog, f.site_type) {
        let m = &catalog.plot_model;
        let need =
            crate::plots::project_area(catalog, r.facility, size, count) * (1.0 + m.ai_reserve);
        let revenue = crate::plots::project_revenue(state, catalog, country, recipe, capacity);
        let Some(plot) = crate::plots::choose(catalog, state, country, need, revenue) else {
            return false;
        };
        let per_unit = crate::plots::unit_area(catalog, r.facility, size) * (1.0 + m.overhead);
        if state.plots[plot.index()].area_ha + 1e-9 < per_unit.max(m.min_site_area_ha) {
            return false;
        }
        investment += crate::plots::value(catalog, state, plot);
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
        auction_until: None,
        development: Default::default(),
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
    let place = (country, f.site_type);
    let revenue = crate::plots::project_revenue(state, catalog, country, recipe, capacity);
    let Some((site, fit)) = found_site_for(
        state,
        catalog,
        id,
        place,
        (Some((r.facility, size, count)), revenue),
    ) else {
        return false;
    };
    let count = count.min(fit);
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
        size,
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

    #[test]
    fn large_demand_gets_large_units_and_short_money_or_land_smaller_ones() {
        let mut catalog = test_support::production();
        catalog.production_model.sizes = crate::catalog::SizeModel {
            capacity: [0.25, 0.5, 1.0, 2.0, 4.0],
            investment_exponent: 0.7,
            labor_exponent: -0.15,
            area_exponent: 0.7,
            build_exponent: 0.3,
        };
        let furnace = catalog.facilities.id("ofen").expect("exists");
        let rich = (Money::ZERO, Money::from_usd(1e9).expect("valid"));
        let any = |_: FacilitySize| u32::MAX;
        assert_eq!(
            plan_units(&catalog, (furnace, 5.0), any, rich),
            Some((FacilitySize::VeryLarge, 1))
        );
        assert_eq!(
            plan_units(&catalog, (furnace, 0.4), any, rich),
            Some((FacilitySize::VerySmall, 2))
        );
        // A furnace costs 2 000 000 USD, a very large one 5 278 031: 2 500 000 pay for
        // one medium furnace.
        let poor = (Money::ZERO, Money::from_usd(2_500_000.0).expect("valid"));
        assert_eq!(
            plan_units(&catalog, (furnace, 5.0), any, poor),
            Some((FacilitySize::Medium, 1))
        );
        // A plot without room for a very large unit takes two large ones.
        let room = |s: FacilitySize| {
            if s == FacilitySize::VeryLarge {
                0
            } else {
                u32::MAX
            }
        };
        assert_eq!(
            plan_units(&catalog, (furnace, 4.0), room, rich),
            Some((FacilitySize::Large, 2))
        );
        // Own power covers the whole lack.
        assert_eq!(
            plan_units_up(&catalog, (furnace, 1.3), any, rich),
            Some((FacilitySize::Medium, 2))
        );
        assert_eq!(
            plan_units(&catalog, (furnace, 1.3), any, rich),
            Some((FacilitySize::Medium, 1))
        );
        let nothing = (Money::ZERO, Money::from_usd(100.0).expect("valid"));
        assert_eq!(plan_units(&catalog, (furnace, 5.0), any, nothing), None);
    }

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
            auction_until: None,
            development: Default::default(),
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
            size: crate::catalog::FacilitySize::Medium,
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

    /// M30: a power plant under construction covers the electricity it will make, so
    /// the next decision during its construction does not add another.
    #[test]
    fn a_power_plant_under_construction_counts_against_the_lack() {
        let (mut game, id, works) = idle_works_in(test_support::power(), 0.05);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        let date = state.date;
        // 25 runs a day at 2 MWh are held back by the grid: 50 MWh a day are missing.
        let slot = &mut state.sites[works.index()].slots[0];
        slot.limit = Some(Limit::Electricity);
        slot.last_runs = 0.0;
        let owned = |state: &GameState| -> Vec<SiteId> {
            (0..state.sites.len())
                .filter(|&i| state.sites[i].owner == id)
                .map(site_id)
                .collect()
        };
        let plants = |state: &GameState| -> u32 {
            state
                .sites
                .iter()
                .filter(|s| s.owner == id && s.kind == SiteType::PowerPlant)
                .flat_map(|s| &s.slots)
                .map(|sl| sl.count)
                .sum()
        };
        assert!(own_power(state, &catalog, id, &owned(state), date));
        assert_eq!(plants(state), 1);
        assert!(
            state
                .sites
                .iter()
                .flat_map(|s| &s.slots)
                .any(|sl| sl.ready > date)
        );
        // The works still wait for power, but the plant being built covers them.
        assert!(!own_power(state, &catalog, id, &owned(state), date));
        assert_eq!(plants(state), 1);
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

    /// M32: a bottleneck is built by a company that knows how to make it, even if it
    /// cannot make the product at the top of the chain (a bauxite mine needs no knowledge
    /// of cooking pots).
    #[test]
    fn a_bottleneck_is_built_without_knowing_the_top_product() {
        let mut catalog = test_support::research();
        let turbine = catalog.technologies.id("turbine").expect("exists");
        let smelting = catalog.recipes.id("eisen_schmelzen").expect("exists");
        catalog.recipes.get_mut(smelting).technology = Some(turbine);
        // Bread is baked without inputs in a furnace.
        let bread = catalog.products.id("brot").expect("exists");
        let mut baking = catalog.recipes.get(smelting).clone();
        baking.product = bread;
        baking.technology = None;
        baking.inputs = Vec::new();
        catalog.recipes.insert("brot_backen", baking).expect("new");
        let (mut game, id, works) = idle_works_in(catalog, 0.6);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        state.sites[works.index()].slots.clear();
        let aaa = catalog.countries.id("AAA").expect("exists");
        let iron = catalog.products.id("eisen").expect("exists");
        let ore = catalog.products.id("erz").expect("exists");
        // Unserved by value: iron 50 000 USD a day, bread 20 000, ore 10 000. Iron is
        // short of ore; nobody knows how to smelt it yet.
        for (product, open) in [(iron, 1000.0), (bread, 10_000.0), (ore, 1000.0)] {
            let m = state.markets.get_mut(product).get_mut(aaa);
            m.open_demand = open;
            m.idle_since = None;
        }
        let first = |state: &GameState| opportunity(state, &catalog, &[], id, None).map(|o| o.0);
        assert_eq!(first(state), Some(bread));
        // Another company knows how: the chain of iron leads the builder to the mine.
        let player = state.player;
        state.companies[player.index()].technologies.insert(turbine);
        let (product, country, deposit, recipe, _) =
            opportunity(state, &catalog, &[], id, None).expect("a bottleneck");
        assert_eq!((product, country), (ore, aaa));
        assert_eq!(deposit, catalog.deposits.id("grube"));
        assert_eq!(Some(recipe), catalog.recipes.id("erz_abbau"));
    }

    /// M37: a competent company with nothing to research develops its best seller and
    /// stays with it until the top level.
    #[test]
    fn nothing_to_research_develops_the_best_seller() {
        let mut catalog = test_support::research();
        let metal = catalog.specializations.id("metall").expect("exists");
        let mut fields = vec![None; catalog.branches.len()];
        fields[0] = Some(metal);
        catalog.research_model.development = crate::catalog::DevelopmentModel {
            levels: 5,
            fields,
            ..Default::default()
        };
        let iron = catalog.products.id("eisen").expect("exists");
        let ore = catalog.products.id("erz").expect("exists");
        let (mut game, id, works) = idle_works_in(catalog, 0.6);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        state.date = Date::new(1903, 1, 1).expect("valid");
        let date = state.date;
        let usd = |v: f64| Money::from_usd(v).expect("valid");
        let mut last_year = crate::ledger::PeriodResult::default();
        last_year
            .by_type
            .insert(CostType::Revenue, usd(15_000_000.0));
        let center = |p| crate::ledger::CostCenter::product(works, p);
        last_year
            .by_center
            .insert(center(iron), [(CostType::Revenue, usd(9_000_000.0))].into());
        last_year
            .by_center
            .insert(center(ore), [(CostType::Revenue, usd(6_000_000.0))].into());
        state.companies[id.index()].ledger.years.push(last_year);
        let developing = |state: &GameState| {
            state
                .sites
                .iter()
                .find(|s| s.owner == id && s.kind == SiteType::ResearchCenter)
                .and_then(|s| s.development)
        };
        research_plan(state, &catalog, id, &[works], date, &[]);
        assert_eq!(developing(state), Some(iron), "the best seller");
        // It stays with iron while there is a next level, then turns to the ore.
        let lab = state
            .sites
            .iter()
            .position(|s| s.owner == id && s.kind == SiteType::ResearchCenter)
            .map(site_id)
            .expect("built");
        state.companies[id.index()]
            .development
            .levels
            .insert(iron, 5);
        assert_eq!(
            development_target(state, &catalog, id, Some(lab)),
            Some(ore)
        );
        research_plan(state, &catalog, id, &[works, lab], date, &[]);
        assert_eq!(developing(state), Some(ore));
    }

    /// M32: a competent company researches a technology outside its branches when
    /// consumers ask for a product that nobody can make.
    #[test]
    fn research_follows_a_market_gap() {
        let mut catalog = test_support::research();
        catalog.ai_model.behavior.entry_companies_max = 3;
        let turbine = catalog.technologies.id("turbine").expect("exists");
        let furnace = catalog.facilities.id("ofen").expect("exists");
        let iron = catalog.products.id("eisen").expect("exists");
        let bicycle = catalog.products.id("rad").expect("exists");
        let vehicles = catalog
            .branches
            .insert("fahrzeuge", crate::catalog::Branch)
            .expect("new");
        catalog.products.get_mut(bicycle).branch = vehicles;
        let smelting = catalog.recipes.id("eisen_schmelzen").expect("exists");
        let mut building = catalog.recipes.get(smelting).clone();
        building.product = bicycle;
        building.facility = furnace;
        building.technology = Some(turbine);
        building.inputs = vec![(iron, 0.1)];
        catalog.recipes.insert("rad_bauen", building).expect("new");
        let (mut game, id, works) = idle_works_in(catalog, 0.6);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        state.date = Date::new(1903, 1, 1).expect("valid");
        let date = state.date;
        let mut last_year = crate::ledger::PeriodResult::default();
        last_year.by_type.insert(
            CostType::Revenue,
            Money::from_usd(10_000_000.0).expect("valid"),
        );
        state.companies[id.index()].ledger.years.push(last_year);
        let researching = |state: &GameState| {
            state
                .sites
                .iter()
                .find(|s| s.owner == id && s.kind == SiteType::ResearchCenter)
                .and_then(|s| s.research)
        };
        // Nobody asks for bicycles: the iron works have nothing to research.
        assert!(gap_technologies(state, &catalog).is_empty());
        let gaps = gap_technologies(state, &catalog);
        research_plan(state, &catalog, id, &[works], date, &gaps);
        assert_eq!(researching(state), None);
        // Consumers ask for them; nobody knows how to make them.
        let aaa = catalog.countries.id("AAA").expect("exists");
        state.markets.get_mut(bicycle).get_mut(aaa).consumer_rate[2] = 5.0;
        // With the prerequisite (known to all since 1800).
        let mut gaps = gap_technologies(state, &catalog);
        gaps.sort();
        assert_eq!(
            gaps,
            vec![
                catalog.technologies.id("schmelzen").expect("exists"),
                turbine
            ]
        );
        research_plan(state, &catalog, id, &[works], date, &gaps);
        assert_eq!(researching(state), Some(turbine));
        // Once a company knows it, the gap is closed.
        let player = state.player;
        state.companies[player.index()].technologies.insert(turbine);
        assert!(gap_technologies(state, &catalog).is_empty());
        // M33: the only maker sells far above the reference price: worth learning.
        let rad_bauen = catalog.recipes.id("rad_bauen");
        state.sites[works.index()].slots[0].recipe = rad_bauen;
        let reference = market::local_reference(&catalog, state, aaa, bicycle);
        let m = state.markets.get_mut(bicycle).get_mut(aaa);
        m.last_month.sold = 100.0;
        m.last_month.revenue = reference.scale(110.0);
        assert!(gap_technologies(state, &catalog).is_empty());
        state
            .markets
            .get_mut(bicycle)
            .get_mut(aaa)
            .last_month
            .revenue = reference.scale(200.0);
        let mut gaps = gap_technologies(state, &catalog);
        gaps.sort();
        assert!(gaps.contains(&turbine), "{gaps:?}");
    }

    /// M32: a product in demand that nobody makes is taken up by a company that can make
    /// it; the plant under construction then counts against the demand.
    #[test]
    fn a_new_product_is_taken_up_by_a_company_that_can_make_it() {
        let mut catalog = test_support::research();
        let turbine = catalog.technologies.id("turbine").expect("exists");
        let bicycle = catalog.products.id("rad").expect("exists");
        let smelting = catalog.recipes.id("eisen_schmelzen").expect("exists");
        let mut building = catalog.recipes.get(smelting).clone();
        building.product = bicycle;
        building.technology = Some(turbine);
        building.inputs = Vec::new();
        let building = catalog.recipes.insert("rad_bauen", building).expect("new");
        let (mut game, id, _) = idle_works_in(catalog, 0.6);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        let date = state.date;
        let aaa = catalog.countries.id("AAA").expect("exists");
        let m = state.markets.get_mut(bicycle).get_mut(aaa);
        m.open_demand = 20.0;
        m.idle_since = None;
        let cash = Money::from_usd(50_000_000.0).expect("valid");
        let ledger = &mut state.companies[id.index()].ledger;
        ledger.transfer(
            crate::ledger::Account::Cash,
            crate::ledger::Account::Equity,
            cash,
        );
        let making = |state: &GameState| {
            state
                .sites
                .iter()
                .filter(|s| s.owner == id)
                .flat_map(|s| &s.slots)
                .filter(|sl| sl.recipe == Some(building))
                .count()
        };
        // Nobody knows how to build bicycles.
        let mut news = Vec::new();
        pioneer(state, &catalog, &by_budget(state, &catalog), &mut news);
        assert_eq!(making(state), 0);
        // The company learns it and builds the first works.
        state.companies[id.index()].technologies.insert(turbine);
        pioneer(state, &catalog, &by_budget(state, &catalog), &mut news);
        assert_eq!(making(state), 1);
        let new = state.sites.last().expect("built");
        assert!(new.slots[0].ready > date);
        // The works under construction will serve the demand: nothing more to build.
        assert_eq!(
            opportunity(state, &catalog, &[], id, Some(&[bicycle])),
            None
        );
        pioneer(state, &catalog, &by_budget(state, &catalog), &mut news);
        assert_eq!(making(state), 1);
    }

    /// M32: a works for an input whose inputs weigh more than its output goes to its raw
    /// material; otherwise, and for goods the state buys, it stays where the demand is.
    #[test]
    fn a_weight_losing_works_goes_to_its_raw_material() {
        for (ore_per_t, state_buys, expected) in
            [(2.0, false, "AAA"), (0.5, false, "BBB"), (2.0, true, "BBB")]
        {
            let mut catalog = test_support::trading();
            let smelting = catalog.recipes.id("eisen_schmelzen").expect("exists");
            catalog.recipes.get_mut(smelting).inputs[0].1 = ore_per_t;
            let iron = catalog.products.id("eisen").expect("exists");
            if !state_buys {
                catalog.products.get_mut(iron).state_demand = None;
            }
            let (mut game, id, works) = idle_works_in(catalog, 0.6);
            let catalog = game.catalog().clone();
            let state = game.state_mut();
            state.sites[works.index()].slots.clear();
            // Iron is wanted in BBB; the ore deposit lies in AAA.
            let bbb = catalog.countries.id("BBB").expect("exists");
            let iron = catalog.products.id("eisen").expect("exists");
            let m = state.markets.get_mut(iron).get_mut(bbb);
            m.open_demand = 1000.0;
            m.idle_since = None;
            let (product, country, ..) =
                opportunity(state, &catalog, &[], id, Some(&[iron])).expect("iron");
            assert_eq!(product, iron);
            assert_eq!(
                catalog.countries.key(country),
                expected,
                "{ore_per_t} t ore, state buys: {state_buys}"
            );
        }
    }

    /// M32: a works that supplies the company's own plants counts what they use as taken,
    /// though nothing goes over the market.
    #[test]
    fn a_works_feeding_its_own_plants_counts_their_use() {
        use crate::state::Slot;
        let (mut game, id, works) = idle_works(0.6);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        let date = state.date;
        let aaa = catalog.countries.id("AAA").expect("exists");
        let ore = catalog.products.id("erz").expect("exists");
        let found = Command::FoundSite {
            country: aaa,
            kind: SiteType::Extraction,
        };
        assert!(run(state, &catalog, id, &found));
        let mine = site_id(state.sites.len() - 1);
        state.sites[mine.index()].slots.push(Slot {
            facility: catalog.facilities.id("mine").expect("exists"),
            ready: date,
            count: 1,
            cost: Money::from_usd(1_000_000.0).expect("valid"),
            recipe: catalog.recipes.id("erz_abbau"),
            utilization: 0.5,
            automation: 0.0,
            condition: 1.0,
            batches: Vec::new(),
            last_runs: 0.0,
            limit: None,
            operation: Operation::Running,
            size: crate::catalog::FacilitySize::Medium,
        });
        state.sites[mine.index()].offers.insert(
            ore,
            crate::state::SaleOffer {
                mode: PriceMode::Market {
                    markup: 0.0,
                    floor: Money::ZERO,
                },
                price: Money::from_usd(10.0).expect("valid"),
                keep: 0.0,
                sold_today: 0.0,
                sold_month: 0.0,
                sold_last_month: 0.0,
                to_traders_month: 0.0,
                to_companies_month: 0.0,
            },
        );
        // Ten furnaces at 60 % take 2 t of ore per run, 50 runs a day each.
        let (_, taken) = output_and_offtake(state, &catalog, mine, &[works, mine], date);
        assert!(
            (taken[&ore] - 10.0 * 50.0 * 0.6 * 2.0).abs() < 1e-9,
            "{taken:?}"
        );
        // Without the works nobody takes the ore.
        let (_, alone) = output_and_offtake(state, &catalog, mine, &[mine], date);
        assert_eq!(alone.get(&ore).copied().unwrap_or(0.0), 0.0);
    }

    /// M33: a new mine gets the units its concession keeps busy, however much ore is
    /// wanted; a deposit whose reserve does not last `reserve_years_min` years of full
    /// output gets no new concession.
    #[test]
    fn a_mine_fits_its_concession_and_its_reserve() {
        let (mut game, id, works) = idle_works_in(test_support::trading(), 0.6);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        state.sites[works.index()].slots.clear();
        let aaa = catalog.countries.id("AAA").expect("exists");
        let ore = catalog.products.id("erz").expect("exists");
        let grube = catalog.deposits.id("grube").expect("exists");
        let m = state.markets.get_mut(ore).get_mut(aaa);
        m.open_demand = 10_000.0;
        m.idle_since = None;
        let (product, _, deposit, _, want) =
            opportunity(state, &catalog, &[], id, Some(&[ore])).expect("a mine");
        assert_eq!((product, deposit), (ore, Some(grube)));
        // 36 500 t a year are 100 t a day: two mines of 100 t at 90 %, not twenty.
        assert_eq!(want, (2.0, 2.0));
        // Most of the reserve is gone: ten years at full output no longer fit.
        let reserve = catalog.deposits.get(grube).reserve.expect("finite");
        state.deposits.get_mut(grube).extracted = reserve - 10.0 * 36_500.0 + 1.0;
        assert_eq!(opportunity(state, &catalog, &[], id, Some(&[ore])), None);
    }

    /// M33: the stocks plants keep of their inputs are no heap; a seller's stock is.
    #[test]
    fn only_sellers_stocks_pile_up() {
        let (mut game, _, works) = idle_works_in(test_support::trading(), 0.6);
        let catalog = game.catalog().clone();
        let state = game.state_mut();
        let ore = catalog.products.id("erz").expect("exists");
        // The furnaces keep a year of ore; nobody makes or sells any.
        state.sites[works.index()]
            .inventory
            .entry(ore)
            .or_default()
            .quantity = 100_000.0;
        assert!(!piling(state, &catalog, ore));
        // Offered by the works, the same stock waits for buyers.
        let offer = state.sites[works.index()].offers
            [&catalog.products.id("eisen").expect("exists")]
            .clone();
        state.sites[works.index()].offers.insert(ore, offer);
        assert!(piling(state, &catalog, ore));
    }

    /// M33: a market paying far above the reference price draws a newcomer even though
    /// every buyer is served; not at a normal price, not the maker itself, not once
    /// enough companies make the product.
    #[test]
    fn a_dear_market_draws_a_newcomer() {
        // With three times the default number of companies, three may make a product
        // where one would do (counts grow with the companies).
        for (entry_max, companies, paid_factor, newcomer, expected) in [
            (3, 100, 2.0, true, true),
            (0, 100, 2.0, true, false),
            (3, 100, 1.1, true, false),
            (3, 100, 2.0, false, false),
            (1, 100, 2.0, true, false),
            (1, 300, 2.0, true, true),
            (0, 300, 2.0, true, false),
        ] {
            let mut catalog = test_support::trading();
            catalog.ai_model.behavior.entry_companies_max = entry_max;
            let (mut game, id, _) = idle_works_in(catalog, 0.9);
            let catalog = game.catalog().clone();
            let state = game.state_mut();
            state.settings.ai.companies = companies;
            let aaa = catalog.countries.id("AAA").expect("exists");
            let iron = catalog.products.id("eisen").expect("exists");
            let reference = market::local_reference(&catalog, state, aaa, iron);
            // Every buyer got iron last month, at a price set by its only maker.
            let m = state.markets.get_mut(iron).get_mut(aaa);
            m.open_demand = 0.0;
            m.idle_since = None;
            m.last_month.sold = 31_000.0;
            m.last_month.revenue = reference.scale(31_000.0 * paid_factor);
            let builder = if newcomer { CompanyId(u32::MAX) } else { id };
            let found = opportunity(state, &catalog, &[], builder, Some(&[iron]));
            assert_eq!(
                found.is_some(),
                expected,
                "{entry_max} {companies} {paid_factor} {newcomer}: {found:?}"
            );
            if let Some((product, country, deposit, _, (wanted, _))) = found {
                assert_eq!((product, country, deposit), (iron, aaa, None));
                // A quarter of 1000 t a day, furnaces of 50 t at 90 %.
                assert_eq!(
                    catalog.production_model.sizes.units_for(wanted),
                    (FacilitySize::Medium, 6)
                );
            }
        }
    }
}
