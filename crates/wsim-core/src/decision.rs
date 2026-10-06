//! Decisions of a company's routine (MA0; docs/MANAGER.md 6.2, docs/FORMELN.md MA0).
//!
//! Where the rules of a company settle something, they first form a decision: its
//! topic, the options they weighed and the one they take. AI companies take the rules'
//! option at once (`Rules`); managers (from MA2) weigh the same options against their
//! budget. Assessments are computed only on request, so that the AI stays as fast as
//! before.

use serde::{Deserialize, Serialize};

use crate::catalog::{Catalog, FacilitySize};
use crate::command::{self, Command};
use crate::ids::{ProductId, RecipeId};
use crate::market;
use crate::money::Money;
use crate::population;
use crate::state::{CompanyId, GameState, Operation, SiteId};

/// What a decision is about (text key `thema.<key>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Topic {
    Production,
    Sale,
    Purchase,
    Wage,
    OwnSupply,
    ProductName,
    Cash,
    Advertising,
    Offer,
    Overcapacity,
    Idle,
    Restart,
    Expansion,
    Power,
    Deposit,
    Bottleneck,
    Research,
    Development,
}

impl Topic {
    pub const ALL: [Topic; 18] = [
        Topic::Production,
        Topic::Sale,
        Topic::Purchase,
        Topic::Wage,
        Topic::OwnSupply,
        Topic::ProductName,
        Topic::Cash,
        Topic::Advertising,
        Topic::Offer,
        Topic::Overcapacity,
        Topic::Idle,
        Topic::Restart,
        Topic::Expansion,
        Topic::Power,
        Topic::Deposit,
        Topic::Bottleneck,
        Topic::Research,
        Topic::Development,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Topic::Production => "produktion",
            Topic::Sale => "verkauf",
            Topic::Purchase => "einkauf",
            Topic::Wage => "lohn",
            Topic::OwnSupply => "eigenversorgung",
            Topic::ProductName => "produktname",
            Topic::Cash => "kasse",
            Topic::Advertising => "werbung",
            Topic::Offer => "kaufangebot",
            Topic::Overcapacity => "ueberkapazitaet",
            Topic::Idle => "stillgelegt",
            Topic::Restart => "wiederanfahren",
            Topic::Expansion => "ausbau",
            Topic::Power => "kraftwerk",
            Topic::Deposit => "lagerstaette",
            Topic::Bottleneck => "engpass",
            Topic::Research => "forschung",
            Topic::Development => "weiterentwicklung",
        }
    }
}

/// Kind of an option (text key `option.<key>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoiceKind {
    Keep,
    Adjust,
    Supply,
    Name,
    Borrow,
    Repay,
    Mothball,
    Sell,
    Restart,
    Build,
    Offer,
    Accept,
    Decline,
    Research,
    Develop,
}

impl ChoiceKind {
    pub fn key(self) -> &'static str {
        match self {
            ChoiceKind::Keep => "beibehalten",
            ChoiceKind::Adjust => "anpassen",
            ChoiceKind::Supply => "liefern",
            ChoiceKind::Name => "benennen",
            ChoiceKind::Borrow => "kredit",
            ChoiceKind::Repay => "tilgen",
            ChoiceKind::Mothball => "stilllegen",
            ChoiceKind::Sell => "verkaufen",
            ChoiceKind::Restart => "anfahren",
            ChoiceKind::Build => "bauen",
            ChoiceKind::Offer => "anbieten",
            ChoiceKind::Accept => "annehmen",
            ChoiceKind::Decline => "ablehnen",
            ChoiceKind::Research => "forschen",
            ChoiceKind::Develop => "weiterentwickeln",
        }
    }
}

/// One command of an option.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub command: Command,
    /// When it is rejected, the following steps are skipped.
    pub required: bool,
}

/// One option of a decision.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    pub kind: ChoiceKind,
    pub steps: Vec<Step>,
}

impl Choice {
    /// Leaves things as they are.
    pub fn keep() -> Self {
        Choice {
            kind: ChoiceKind::Keep,
            steps: Vec::new(),
        }
    }

    /// One command.
    pub fn one(kind: ChoiceKind, command: Command) -> Self {
        Choice {
            kind,
            steps: vec![Step {
                command,
                required: true,
            }],
        }
    }

    /// Commands that run one after the other whatever becomes of the others.
    pub fn each(kind: ChoiceKind, commands: impl IntoIterator<Item = Command>) -> Self {
        Choice {
            kind,
            steps: commands
                .into_iter()
                .map(|command| Step {
                    command,
                    required: false,
                })
                .collect(),
        }
    }

    /// Adds a step.
    pub fn then(mut self, command: Command, required: bool) -> Self {
        self.steps.push(Step { command, required });
        self
    }
}

/// A decision of a company: the options its rules weighed and the one they take.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub topic: Topic,
    pub company: CompanyId,
    pub site: Option<SiteId>,
    pub product: Option<ProductId>,
    pub choices: Vec<Choice>,
    /// The choice the rules take.
    pub rule: usize,
}

impl Decision {
    /// The rules' choice, then keeping things as they are.
    pub fn new(topic: Topic, company: CompanyId, choice: Choice) -> Self {
        Decision {
            topic,
            company,
            site: None,
            product: None,
            choices: vec![choice, Choice::keep()],
            rule: 0,
        }
    }

    pub fn at(mut self, site: SiteId) -> Self {
        self.site = Some(site);
        self
    }

    pub fn of(mut self, product: ProductId) -> Self {
        self.product = Some(product);
        self
    }

    /// Another option, before keeping things as they are.
    pub fn or(mut self, choice: Choice) -> Self {
        let keep = self.choices.len() - 1;
        self.choices.insert(keep, choice);
        self
    }

    pub fn chosen(&self) -> &Choice {
        &self.choices[self.rule]
    }
}

/// Runs the steps of a company's choice in order; a rejected required step skips the
/// rest. Returns for each step whether it ran.
pub(crate) fn execute(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    choice: &Choice,
) -> Vec<bool> {
    let mut skip = false;
    choice
        .steps
        .iter()
        .map(|step| {
            let ran = !skip && command::execute(state, catalog, company, &step.command).is_ok();
            skip |= step.required && !ran;
            ran
        })
        .collect()
}

/// What a decider makes of a decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The rules carry out their own option, as they always did.
    Rule,
    /// Another option, carried out at once.
    Choice(usize),
    /// Nothing now: kept as it is, or asked.
    Hold,
}

/// Who rules on a company's decisions.
pub trait Decider {
    /// Whether the decider looks at decisions at all. The AI's rules do not, so that
    /// forming decisions costs them nothing.
    fn looks(&self) -> bool;

    fn decide(&mut self, state: &GameState, catalog: &Catalog, decision: &Decision) -> Verdict;
}

/// The AI: its rules decide.
pub struct Rules;

impl Decider for Rules {
    fn looks(&self) -> bool {
        false
    }

    fn decide(&mut self, _: &GameState, _: &Catalog, _: &Decision) -> Verdict {
        Verdict::Rule
    }
}

/// Looks at every decision, keeps it and lets the rules act (tests, diagnosis).
#[derive(Clone, Debug, Default)]
pub struct Recorder {
    pub decisions: Vec<Decision>,
}

impl Decider for Recorder {
    fn looks(&self) -> bool {
        true
    }

    fn decide(&mut self, _: &GameState, _: &Catalog, decision: &Decision) -> Verdict {
        self.decisions.push(decision.clone());
        Verdict::Rule
    }
}

/// Puts a decision to a decider (formed only if it looks) and carries out another option
/// it chooses. Returns whether the rules are to carry out their own option.
pub(crate) fn decided(
    decider: &mut dyn Decider,
    state: &mut GameState,
    catalog: &Catalog,
    form: impl FnOnce(&GameState) -> Decision,
) -> bool {
    if !decider.looks() {
        return true;
    }
    let decision = form(state);
    match decider.decide(state, catalog, &decision) {
        Verdict::Rule => true,
        Verdict::Choice(i) if i == decision.rule => true,
        Verdict::Choice(i) => {
            if let Some(choice) = decision.choices.get(i) {
                execute(state, catalog, decision.company, choice);
            }
            false
        }
        Verdict::Hold => false,
    }
}

/// What an option means for its company (docs/FORMELN.md, MA0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Assessment {
    /// Counted against a manager's budget (docs/MANAGER.md 5.1).
    pub amount: Money,
    /// Estimated effect on the result of a year against keeping things as they are;
    /// none where the rules make no estimate.
    pub effect: Option<Money>,
    /// One-off effect on the result: proceeds less book value, restart costs.
    pub once: Money,
}

/// Assesses every option of a decision.
pub fn assess(catalog: &Catalog, state: &GameState, decision: &Decision) -> Vec<Assessment> {
    let keep = effect_base(catalog, state, decision);
    decision
        .choices
        .iter()
        .map(|choice| {
            let amount = choice
                .steps
                .iter()
                .map(|s| amount(catalog, state, decision.company, &s.command))
                .fold(Money::ZERO, |a, b| a + b);
            let once = choice
                .steps
                .iter()
                .map(|s| once(catalog, state, &s.command))
                .fold(Money::ZERO, |a, b| a + b);
            let effect = if choice.kind == ChoiceKind::Keep {
                keep.map(|_| Money::ZERO)
            } else {
                effect(catalog, state, decision, choice, keep)
            };
            Assessment {
                amount,
                effect,
                once,
            }
        })
        .collect()
}

/// The amount a command counts against a budget (docs/MANAGER.md 5.1).
pub fn amount(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    command: &Command,
) -> Money {
    let model = &catalog.production_model;
    match command {
        Command::BuildFacility {
            facility,
            count,
            size,
            ..
        } => catalog
            .facilities
            .get(*facility)
            .investment
            .scale(model.sizes.investment(*size) * f64::from(*count)),
        Command::DevelopDeposit { deposit, .. } => {
            let share = state
                .deposits
                .get(*deposit)
                .concessions
                .iter()
                .find(|c| c.site.is_none())
                .map_or(0.0, |c| c.share);
            catalog
                .deposits
                .get(*deposit)
                .development_cost
                .scale(state.settings.market_scale * share)
        }
        Command::FoundSite { kind, .. } => model.site_cost(*kind),
        Command::FoundSiteOnPlot { plot, kind, lease } => {
            let land = crate::plots::value(catalog, state, *plot);
            let land = if *lease {
                land.scale(catalog.plot_model.rent_share)
            } else {
                land
            };
            model.site_cost(*kind) + land
        }
        Command::BuyPlot { site } => state
            .sites
            .get(site.index())
            .and_then(|s| s.plot)
            .map_or(Money::ZERO, |p| crate::plots::value(catalog, state, p)),
        Command::MakeOffer { price, .. } => *price,
        Command::SetWagePremium { site, premium } => {
            let Some(s) = state.sites.get(site.index()) else {
                return Money::ZERO;
            };
            if *premium <= s.wage_premium {
                return Money::ZERO;
            }
            let labor = |factor: f64| -> f64 {
                s.slots
                    .iter()
                    .filter_map(|sl| sl.recipe.map(|r| (sl, r)))
                    .map(|(sl, r)| {
                        population::slot_flows(
                            catalog,
                            state,
                            s.country,
                            r,
                            (sl.count, sl.size),
                            sl.utilization,
                            (factor, crate::development::Effect::NONE),
                        )
                        .variable_per_day
                        .to_usd()
                    })
                    .sum()
            };
            Money::from_usd((labor(1.0 + premium) - labor(1.0 + s.wage_premium)) * 365.0)
                .unwrap_or(Money::ZERO)
                .max(Money::ZERO)
        }
        Command::SetPurchase {
            site,
            product,
            target,
            max_price,
            ..
        } => {
            let old = state
                .sites
                .get(site.index())
                .and_then(|s| s.orders.get(product))
                .map_or(Money::ZERO, |o| o.max_price);
            if *max_price <= old {
                return Money::ZERO;
            }
            let per_day = target / catalog.ai_model.start.input_stock_days.max(1.0);
            Money::times(*max_price - old, per_day * 365.0)
        }
        Command::SetAdvertising {
            country,
            group,
            budget,
        } => {
            let old = state.companies[company.index()]
                .advertising
                .iter()
                .find(|a| a.country == *country && a.group == *group)
                .map_or(Money::ZERO, |a| a.budget);
            (*budget - old).max(Money::ZERO).scale(12.0)
        }
        Command::MothballFacility { site, slot, count }
        | Command::SellFacility { site, slot, count } => state
            .sites
            .get(site.index())
            .and_then(|s| s.slots.get(*slot))
            .map_or(Money::ZERO, |sl| {
                crate::production::sale_value(catalog, sl, *count, state.date).0
            }),
        Command::RestartFacility { site, slot } => state
            .sites
            .get(site.index())
            .and_then(|s| s.slots.get(*slot))
            .map_or(Money::ZERO, |sl| sl.cost.scale(model.restart_cost_share)),
        Command::TakeLoan { amount, .. } | Command::RepayLoan { amount, .. } => *amount,
        _ => Money::ZERO,
    }
}

/// One-off effect of a command on the result.
fn once(catalog: &Catalog, state: &GameState, command: &Command) -> Money {
    match command {
        Command::SellFacility { site, slot, count } => state
            .sites
            .get(site.index())
            .and_then(|s| s.slots.get(*slot))
            .map_or(Money::ZERO, |sl| {
                let (book, proceeds) =
                    crate::production::sale_value(catalog, sl, *count, state.date);
                proceeds - book
            }),
        Command::RestartFacility { site, slot } => state
            .sites
            .get(site.index())
            .and_then(|s| s.slots.get(*slot))
            .map_or(Money::ZERO, |sl| {
                Money::ZERO - sl.cost.scale(catalog.production_model.restart_cost_share)
            }),
        _ => Money::ZERO,
    }
}

/// Units of a facility at a site as an option would leave them.
#[derive(Clone, Copy, Debug)]
struct Unit {
    count: u32,
    size: FacilitySize,
    recipe: Option<RecipeId>,
    utilization: f64,
    running: bool,
}

/// The site, products and units a decision about production looks at, and the result
/// of a year if things stay as they are.
#[derive(Clone, Debug)]
struct Base {
    site: SiteId,
    products: Vec<ProductId>,
    units: Vec<Unit>,
    wage_premium: f64,
    result: f64,
}

fn units_of(state: &GameState, site: SiteId) -> Vec<Unit> {
    state.sites[site.index()]
        .slots
        .iter()
        .map(|sl| Unit {
            count: sl.count,
            size: sl.size,
            recipe: sl.recipe,
            utilization: sl.utilization,
            running: sl.operating(state.date)
                || matches!(sl.operation, Operation::Restarting { .. }),
        })
        .collect()
}

/// The result of a year if things stay as they are, for decisions estimated at a site.
fn effect_base(catalog: &Catalog, state: &GameState, decision: &Decision) -> Option<f64> {
    base(catalog, state, decision).map(|b| b.result)
}

/// The site, products and units a decision about production looks at: the decision's
/// product, or all products made at the site (wage premium).
fn base(catalog: &Catalog, state: &GameState, decision: &Decision) -> Option<Base> {
    let estimated = matches!(
        decision.topic,
        Topic::Production | Topic::Wage | Topic::Overcapacity | Topic::Idle | Topic::Restart
    );
    if !estimated {
        return None;
    }
    let site = decision.site?;
    let s = state.sites.get(site.index())?;
    let units = units_of(state, site);
    let products: Vec<ProductId> = match decision.product {
        Some(p) => vec![p],
        None => {
            let mut all: Vec<ProductId> = s
                .slots
                .iter()
                .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r).product))
                .collect();
            all.sort();
            all.dedup();
            all
        }
    };
    let result = site_result(catalog, state, site, &products, &units, s.wage_premium, 0.0);
    Some(Base {
        site,
        products,
        units,
        wage_premium: s.wage_premium,
        result,
    })
}

/// Result of a year of the products at a site with the given units (docs/FORMELN.md,
/// MA0): 365 · (min(A, Q) · p − Q · v) − F, with `extra` of the output counted as sold.
fn site_result(
    catalog: &Catalog,
    state: &GameState,
    site: SiteId,
    products: &[ProductId],
    units: &[Unit],
    wage_premium: f64,
    extra: f64,
) -> f64 {
    let s = &state.sites[site.index()];
    let owner = s.owner;
    let sites: Vec<SiteId> = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, x)| x.owner == owner)
        .map(|(i, _)| SiteId(u32::try_from(i).unwrap_or(u32::MAX)))
        .collect();
    let (_, taken) = crate::ai::output_and_offtake(state, catalog, site, &sites, state.date);
    let model = &catalog.production_model;
    let mut total = 0.0;
    for &product in products {
        let (mut output, mut variable, mut fixed) = (0.0, 0.0, 0.0);
        for u in units {
            let Some(r) = u
                .recipe
                .filter(|&r| catalog.recipes.get(r).product == product)
            else {
                continue;
            };
            let f = catalog.facilities.get(catalog.recipes.get(r).facility);
            let investment =
                f.investment.to_usd() * model.sizes.investment(u.size) * f64::from(u.count);
            if u.running {
                let flows = population::slot_flows(
                    catalog,
                    state,
                    s.country,
                    r,
                    (u.count, u.size),
                    u.utilization,
                    (
                        1.0 + wage_premium,
                        crate::ai::developed(state, catalog, owner, r),
                    ),
                );
                output += flows.output;
                variable += flows.variable_per_day.to_usd();
                fixed += (flows.cost_per_day - flows.variable_per_day).to_usd() * 365.0;
            } else {
                // Idle units: depreciation goes on, a share of the maintenance stays (on
                // the same basis as running units, the investment at today's prices).
                fixed += investment
                    * (1.0 / f64::from(f.lifetime_years.max(1))
                        + f.maintenance_share * model.mothball_maintenance_share);
            }
        }
        let price = s
            .offers
            .get(&product)
            .map_or_else(
                || market::market_price(catalog, state, s.country, product),
                |o| o.price,
            )
            .to_usd();
        let sold = (taken.get(&product).copied().unwrap_or(0.0) + extra).min(output);
        total += 365.0 * (sold * price - variable) - fixed;
    }
    total
}

/// Estimated effect of an option on the result of a year (docs/FORMELN.md, MA0).
fn effect(
    catalog: &Catalog,
    state: &GameState,
    decision: &Decision,
    choice: &Choice,
    keep: Option<f64>,
) -> Option<Money> {
    match decision.topic {
        Topic::Cash => {
            let company = &state.companies[decision.company.index()];
            let usd: f64 = choice
                .steps
                .iter()
                .map(|s| match &s.command {
                    Command::TakeLoan { amount, .. } => {
                        -amount.to_usd()
                            * crate::finance::loan_rate(catalog, company, *amount, state.date)
                    }
                    Command::RepayLoan { loan, amount } => company
                        .loans
                        .get(*loan)
                        .map_or(0.0, |l| amount.to_usd() * l.rate),
                    _ => 0.0,
                })
                .sum();
            Money::from_usd(usd)
        }
        Topic::Advertising => {
            let usd: f64 = choice
                .steps
                .iter()
                .map(|s| match &s.command {
                    Command::SetAdvertising {
                        country,
                        group,
                        budget,
                    } => {
                        let old = state.companies[decision.company.index()]
                            .advertising
                            .iter()
                            .find(|a| a.country == *country && a.group == *group)
                            .map_or(Money::ZERO, |a| a.budget);
                        -(*budget - old).to_usd() * 12.0
                    }
                    _ => 0.0,
                })
                .sum();
            Money::from_usd(usd)
        }
        Topic::Expansion | Topic::Power | Topic::Deposit | Topic::Bottleneck => {
            new_capacity(catalog, state, decision.company, choice)
        }
        _ => {
            let keep = keep?;
            let b = base(catalog, state, decision)?;
            let (units, premium) = apply(&b, choice);
            let result = site_result(catalog, state, b.site, &b.products, &units, premium, 0.0);
            Money::from_usd(result - keep)
        }
    }
}

/// New capacity (docs/FORMELN.md, MA0): its output at the start utilization counts as
/// sold, 365 · ΔQ · (p − v) − ΔF − interest of a loan taken for it; deposits add the
/// depreciation of their development.
fn new_capacity(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    choice: &Choice,
) -> Option<Money> {
    let model = &catalog.production_model;
    let owner = &state.companies[company.index()];
    // Sites founded by the option get the next ids, in order.
    let mut founded: Vec<crate::ids::CountryId> = Vec::new();
    let country_of = |site: SiteId, founded: &[crate::ids::CountryId]| {
        state
            .sites
            .get(site.index())
            .map(|s| s.country)
            .or_else(|| {
                founded
                    .get(site.index().checked_sub(state.sites.len())?)
                    .copied()
            })
    };
    let mut built: Vec<(SiteId, u32, FacilitySize)> = Vec::new();
    let mut usd = 0.0;
    for step in &choice.steps {
        match &step.command {
            Command::FoundSite { country, .. } => founded.push(*country),
            Command::FoundSiteOnPlot { plot, .. } => {
                founded.push(state.plots[plot.index()].country);
            }
            Command::BuildFacility {
                site, count, size, ..
            } => built.push((*site, *count, *size)),
            Command::SetProduction {
                site,
                recipe: Some(r),
                utilization,
                ..
            } => {
                let Some(&(_, count, size)) = built.iter().rev().find(|b| b.0 == *site) else {
                    continue;
                };
                let country = country_of(*site, &founded)?;
                let flows = population::slot_flows(
                    catalog,
                    state,
                    country,
                    *r,
                    (count, size),
                    *utilization,
                    (1.0, crate::ai::developed(state, catalog, company, *r)),
                );
                let product = catalog.recipes.get(*r).product;
                let price =
                    if catalog.products.get(product).kind == crate::catalog::ProductKind::Energy {
                        state.countries.get(country).electricity_price_usd_mwh
                    } else {
                        market::market_price(catalog, state, country, product).to_usd()
                    };
                usd += 365.0 * (flows.output * price - flows.cost_per_day.to_usd());
            }
            Command::DevelopDeposit { .. } => {
                usd -= amount(catalog, state, company, &step.command).to_usd()
                    / model.development_lifetime_years.max(1.0);
            }
            Command::TakeLoan { amount, .. } => {
                usd -= amount.to_usd()
                    * crate::finance::loan_rate(catalog, owner, *amount, state.date);
            }
            _ => {}
        }
    }
    Money::from_usd(usd)
}

/// The units and the wage premium of a site after an option's commands.
fn apply(base: &Base, choice: &Choice) -> (Vec<Unit>, f64) {
    let mut units = base.units.clone();
    let mut premium = base.wage_premium;
    for step in &choice.steps {
        match &step.command {
            Command::SetProduction {
                site,
                slot,
                recipe,
                utilization,
            } if *site == base.site => {
                if let Some(u) = units.get_mut(*slot) {
                    u.recipe = *recipe;
                    u.utilization = *utilization;
                }
            }
            Command::MothballFacility { site, slot, count } if *site == base.site => {
                if let Some(u) = units.get(*slot).copied() {
                    let k = (*count).min(u.count);
                    units[*slot].count -= k;
                    units.push(Unit {
                        count: k,
                        running: false,
                        ..u
                    });
                }
            }
            Command::SellFacility { site, slot, count } if *site == base.site => {
                if let Some(u) = units.get_mut(*slot) {
                    u.count -= (*count).min(u.count);
                }
            }
            Command::RestartFacility { site, slot } if *site == base.site => {
                if let Some(u) = units.get_mut(*slot) {
                    u.running = true;
                }
            }
            Command::SetWagePremium { site, premium: p } if *site == base.site => {
                premium = *p;
            }
            _ => {}
        }
    }
    (units, premium)
}
