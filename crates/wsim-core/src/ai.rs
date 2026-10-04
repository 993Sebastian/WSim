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
use crate::state::{AiState, Company, CompanyId, CompanyKind, GameState, PriceMode, SiteId};

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
        let mut cost: BTreeMap<ProductId, (Money, f64, f64, f64)> = BTreeMap::new();
        for (index, sl) in s.slots.iter().enumerate() {
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
            if catalog.products.get(product).kind != ProductKind::Energy && sl.ready <= date {
                let full = catalog.facilities.get(sl.facility).runs_per_day
                    * f64::from(sl.count)
                    * r.output;
                let keep = s.offers.get(&product).map_or(0.0, |o| o.keep);
                let stock = s.inventory.get(&product).map_or(0.0, |x| x.quantity) - keep;
                // Stock in days of sales; measured against the own output it would grow
                // with every cut and drive production to the minimum.
                let sold = s.offers.get(&product).map_or(0.0, |o| o.sold_last_month) / 30.0;
                let rate = if sold > 1e-9 {
                    sold
                } else {
                    full * utilization.max(b.utilization_min)
                };
                let days = stock.max(0.0) / rate.max(1e-9);
                let planned = catalog.facilities.get(sl.facility).runs_per_day
                    * f64::from(sl.count)
                    * sl.utilization;
                let limited = sl.last_runs < 0.9 * planned;
                if days > b.stock_high_days {
                    utilization = (utilization - b.utilization_step).max(b.utilization_min);
                } else if days < b.stock_low_days && !limited {
                    utilization = (utilization + b.utilization_step).min(1.0);
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
            let flows: SlotFlows =
                population::slot_flows(catalog, state, country, recipe, sl.count, utilization);
            for (input, q) in &flows.inputs {
                *need.entry(*input).or_default() += q;
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
            let e = cost
                .entry(flows.product)
                .or_insert((Money::ZERO, 0.0, 0.0, 0.0));
            e.0 += flows.variable_per_day;
            e.1 += flows.output;
            e.2 += fixed;
            e.3 += full;
        }
        // Prices: the market decides above a floor from the full unit cost.
        for (&product, offer) in &s.offers {
            let Some(&(daily, output, fixed, full)) = cost.get(&product) else {
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
                        floor: Money::ZERO,
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
            let max_price = match s.orders.get(&product) {
                Some(o) if stock < per_day * b.stock_low_days => o
                    .max_price
                    .scale(1.0 + b.utilization_step)
                    .min(price.scale(3.0))
                    .max(normal),
                Some(o) if o.max_price > normal && stock >= per_day * b.stock_high_days => {
                    normal.max(o.max_price.scale(1.0 - b.utilization_step))
                }
                Some(o) => o.max_price.max(normal),
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
        for c in &commands {
            run(state, catalog, id, c);
        }
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
                total +=
                    population::slot_flows(catalog, state, s.country, r, sl.count, sl.utilization)
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
        for sl in &s.slots {
            let Some(r) = sl.recipe else { continue };
            if sl.ready > date {
                continue;
            }
            let flows =
                population::slot_flows(catalog, state, s.country, r, sl.count, sl.utilization);
            for &(input, q) in &flows.inputs {
                *own_use.entry(input).or_default() += q;
            }
            if catalog.recipes.get(r).extraction {
                continue;
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
            if utilization >= min_utilization
                && sells
                && margin >= min_margin
                && best.is_none_or(|(m, ..)| margin > m)
            {
                let add = (f64::from(count) * 0.25).round().max(1.0);
                // Small counts; the cast cannot overflow.
                best = Some((margin, site, product, add as u32));
            }
        }
    }
    let Some((_, site, product, mut count)) = best else {
        open_deposit(state, catalog, id, sites, date, news);
        return;
    };
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
        let Some((product, country, deposit, recipe, count)) = opportunity(state, catalog, &taken)
        else {
            break;
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
            if let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r))
                && r.product == product
            {
                output += sl.last_runs * r.output;
            }
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
        let Some((product, country, deposit, recipe, count)) = opportunity(state, catalog, &[])
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
/// materials need a free concession; chains without one are left out.
fn opportunity(state: &GameState, catalog: &Catalog, taken: &[ProductId]) -> Option<Opportunity> {
    let model = &catalog.market_model;
    let newcomer = CompanyId(u32::MAX);
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
            let Some(recipe) = best_recipe(catalog, state, newcomer, product, None, None) else {
                continue 'chains;
            };
            let r = catalog.recipes.get(recipe);
            let runs = catalog.facilities.get(r.facility).runs_per_day * u;
            // The input whose unserved demand is largest relative to one new plant's need.
            let short = r
                .inputs
                .iter()
                .filter(|(i, _)| catalog.products.get(*i).state_market.is_none())
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
