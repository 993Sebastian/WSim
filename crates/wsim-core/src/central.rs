//! Headquarters and central departments (ZA1–ZA3, docs/FORMELN.md, docs/BETEILIGUNGEN.md).

use std::collections::BTreeMap;

use crate::calendar::Date;
use crate::catalog::{Catalog, DepartmentKind};
use crate::command::CommandError;
use crate::ids::{CountryId, Id, TechnologyId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::management;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, GameState, ManagerId, Position, Relocation, Role, Unit};

/// Employees of a company's central departments.
pub fn employees(state: &GameState, company: CompanyId) -> u32 {
    state.companies[company.index()].departments.values().sum()
}

/// Employees of one department.
pub fn staff(state: &GameState, company: CompanyId, kind: DepartmentKind) -> u32 {
    state.companies[company.index()]
        .departments
        .get(&kind)
        .copied()
        .unwrap_or(0)
}

/// The position heading a department: the board's specialist of its function.
pub fn head_position(catalog: &Catalog, kind: DepartmentKind) -> Option<Position> {
    let d = catalog.central.department(kind)?;
    let f = catalog.management.functions.get(d.function)?;
    Some(Position {
        unit: Unit::Board,
        role: Role::Specialist(f.key.clone()),
    })
}

/// The manager heading a department of a company.
pub fn head(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    kind: DepartmentKind,
) -> Option<ManagerId> {
    management::holder(state, company, &head_position(catalog, kind)?)
}

/// What a department has to work on in a month (docs/FORMELN.md, ZA2).
pub fn workload(state: &GameState, company: CompanyId, kind: DepartmentKind) -> f64 {
    let c = &state.companies[company.index()];
    // Small counts; the casts are exact.
    match kind {
        DepartmentKind::Strategy | DepartmentKind::Legal => 1.0,
        DepartmentKind::Finance => c.loans.len() as f64 + 1.0,
        DepartmentKind::Personnel => state
            .managers
            .values()
            .filter(|m| m.job.as_ref().is_some_and(|j| j.company == company))
            .count() as f64,
        DepartmentKind::Marketing => {
            c.advertising
                .iter()
                .filter(|a| a.budget > Money::ZERO)
                .count() as f64
                + 1.0
        }
    }
}

/// What a department achieves (docs/FORMELN.md, ZA2).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Performance {
    /// Cases per month: employees · cases each.
    pub capacity: f64,
    /// The head's expertise / 100.
    pub quality: f64,
    /// Share of the workload covered, 0–1.
    pub coverage: f64,
    /// Effect · quality · coverage.
    pub strength: f64,
}

/// A department's performance; `None` without employees or head – it does not work.
pub fn performance(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    kind: DepartmentKind,
) -> Option<Performance> {
    let n = staff(state, company, kind);
    if n == 0 {
        return None;
    }
    let d = catalog.central.department(kind)?;
    let head = head(catalog, state, company, kind)?;
    let function = &catalog.management.functions[d.function].key;
    let quality = f64::from(management::expertise(&state.managers[&head], function)) / 100.0;
    let capacity = f64::from(n) * d.cases;
    let load = workload(state, company, kind);
    let coverage = if load > 0.0 {
        (capacity / load).min(1.0)
    } else {
        1.0
    };
    Some(Performance {
        capacity,
        quality,
        coverage,
        strength: d.effect * quality * coverage,
    })
}

/// The strength W of a department, 0 where it does not work.
pub fn strength(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    kind: DepartmentKind,
) -> f64 {
    if state.companies[company.index()].departments.is_empty() {
        return 0.0;
    }
    performance(catalog, state, company, kind).map_or(0.0, |p| p.strength)
}

/// Cases a department works on at a check: ⌊K⌋, none where it does not work.
pub fn cases(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    kind: DepartmentKind,
) -> usize {
    // Capacities are small; the cast saturates.
    performance(catalog, state, company, kind).map_or(0, |p| p.capacity.floor() as usize)
}

/// The countries a strategy department observes (docs/FORMELN.md, ZA2): ⌊K⌋ of the
/// largest economies (inhabitants · GDP per capita) without a site of the company.
pub fn observed_countries(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
) -> Vec<CountryId> {
    let n = cases(catalog, state, company, DepartmentKind::Strategy);
    if n == 0 {
        return Vec::new();
    }
    let c = &state.companies[company.index()];
    let own: std::collections::BTreeSet<CountryId> = state
        .sites
        .iter()
        .filter(|s| s.owner == company)
        .map(|s| s.country)
        .chain(std::iter::once(c.headquarters))
        .collect();
    let mut economies: Vec<(f64, CountryId)> = catalog
        .countries
        .ids()
        .filter(|id| !own.contains(id))
        .map(|id| {
            let v = state.countries.get(id);
            (v.population * v.gdp_per_capita_usd, id)
        })
        .filter(|(size, _)| *size > 0.0)
        .collect();
    economies.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    economies.into_iter().take(n).map(|(_, id)| id).collect()
}

/// The technologies a legal department checks for licences (docs/FORMELN.md, ZA2): up to
/// ⌊K⌋ of the most valuable that recipes of the company's goods groups need, which it
/// neither knows nor researches and another company could license.
pub fn legal_technologies(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
) -> Vec<TechnologyId> {
    let n = cases(catalog, state, company, DepartmentKind::Legal);
    if n == 0 {
        return Vec::new();
    }
    let c = &state.companies[company.index()];
    let mut groups = std::collections::BTreeSet::new();
    for s in state.sites.iter().filter(|s| s.owner == company) {
        let products = s
            .slots
            .iter()
            .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r).product))
            .chain(s.offers.keys().copied());
        groups.extend(products.map(|p| catalog.products.get(p).goods_group));
    }
    let researching: std::collections::BTreeSet<TechnologyId> = c
        .research
        .keys()
        .copied()
        .chain(
            state
                .sites
                .iter()
                .filter(|s| s.owner == company)
                .filter_map(|s| s.research),
        )
        .collect();
    let candidates: std::collections::BTreeSet<TechnologyId> = catalog
        .recipes
        .iter()
        .filter(|(_, r)| groups.contains(&catalog.products.get(r.product).goods_group))
        .filter_map(|(_, r)| r.technology)
        .filter(|&t| !researching.contains(&t) && !state.knows(catalog, company, t))
        .filter(|&t| {
            (0..state.companies.len()).any(|i| {
                // Few companies; the cast is exact.
                let other = CompanyId(i as u32);
                other != company && !state.companies[i].bankrupt && state.knows(catalog, other, t)
            })
        })
        .collect();
    let mut valued: Vec<(Money, TechnologyId)> = candidates
        .into_iter()
        .filter_map(|t| Some((crate::deals::license_value(state, catalog, company, t)?, t)))
        .collect();
    valued.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    valued.into_iter().take(n).map(|(_, t)| t).collect()
}

/// The monthly chance of a point of experience of a company's managers (ZA2): a
/// personnel department trains them, the chance × (1 + W).
pub fn experience_chance(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    base: f64,
) -> f64 {
    let w = strength(catalog, state, company, DepartmentKind::Personnel);
    if w > 0.0 {
        (base * (1.0 + w)).min(1.0)
    } else {
        base
    }
}

/// The share of the risk premium of new loans the finance department saves (ZA2).
pub fn premium_cut(catalog: &Catalog, state: &GameState, company: CompanyId) -> f64 {
    strength(catalog, state, company, DepartmentKind::Finance)
}

/// How a department's head errs less in the decisions of its function (ZA2): the factor
/// on the estimate error and the share of the gap to a judgment of 100 closed.
pub fn accuracy(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    position: &Position,
) -> f64 {
    if position.unit != Unit::Board || state.companies[company.index()].departments.is_empty() {
        return 0.0;
    }
    let Role::Specialist(function) = &position.role else {
        return 0.0;
    };
    catalog
        .central
        .departments
        .iter()
        .find(|d| catalog.management.functions[d.function].key == *function)
        .and_then(|d| performance(catalog, state, company, d.kind))
        .map_or(0.0, |p| catalog.central.accuracy * p.coverage)
}

/// The monthly costs of a company's departments (docs/FORMELN.md, ZA2): salaries of the
/// employees at the wage of their group in the country of the headquarters, and offices.
pub fn monthly_cost(catalog: &Catalog, state: &GameState, company: CompanyId) -> (Money, Money) {
    let c = &state.companies[company.index()];
    let (mut personnel, mut office) = (Money::ZERO, Money::ZERO);
    for (&kind, &n) in &c.departments {
        let Some(d) = catalog.central.department(kind) else {
            continue;
        };
        let wage = management::group_yearly_wage(catalog, state, c.headquarters, d.labor_group);
        personnel += Money::from_usd(f64::from(n) * wage / 12.0).unwrap_or(Money::ZERO);
        office += d.office.scale(f64::from(n) / 12.0);
    }
    (personnel, office)
}

/// The costs of the month, booked on its last day as overhead of the headquarters.
pub fn month_end(state: &mut GameState, catalog: &Catalog) {
    for i in 0..state.companies.len() {
        let c = &state.companies[i];
        if c.bankrupt || c.departments.is_empty() {
            continue;
        }
        // Few companies; the cast is exact.
        let (personnel, office) = monthly_cost(catalog, state, CompanyId(i as u32));
        let ledger = &mut state.companies[i].ledger;
        if personnel > Money::ZERO {
            ledger.expense(
                CostType::Personnel,
                CostCenter::default(),
                Account::Cash,
                personnel,
            );
        }
        if office > Money::ZERO {
            ledger.expense(
                CostType::Overhead,
                CostCenter::default(),
                Account::Cash,
                office,
            );
        }
    }
}

/// `StaffDepartment`: the number of employees of a department; hired and let go at once.
pub(crate) fn staff_department(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    kind: DepartmentKind,
    staff: u32,
) -> Result<(), CommandError> {
    if catalog.central.department(kind).is_none() {
        return Err(CommandError::UnknownDepartment);
    }
    let departments = &mut state.companies[actor.index()].departments;
    if staff == 0 {
        departments.remove(&kind);
    } else {
        departments.insert(kind, staff);
    }
    Ok(())
}

/// `SetParticipations`: the policy on takeovers, licences and start-ups (ZA2).
pub(crate) fn set_participations(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    (budget, risk): (Option<Money>, f64),
    limits: &BTreeMap<DepartmentKind, Money>,
) -> Result<(), CommandError> {
    if limits
        .keys()
        .any(|&k| catalog.central.department(k).is_none())
    {
        return Err(CommandError::UnknownDepartment);
    }
    if budget.is_some_and(|b| b < Money::ZERO)
        || !(0.0..=1.0).contains(&risk)
        || limits.values().any(|&l| l < Money::ZERO)
    {
        return Err(CommandError::InvalidParticipations);
    }
    let p = &mut state.companies[actor.index()].participations;
    p.budget = budget;
    p.risk = risk;
    p.limits = limits.clone();
    Ok(())
}

/// What is left of a company's budget for takeovers and licences this year after its
/// purchases and open bids; `None` without a budget.
pub fn participations_left(state: &GameState, company: CompanyId) -> Option<Money> {
    let p = &state.companies[company.index()].participations;
    let budget = p.budget?;
    let bids: Money = state
        .offers
        .iter()
        .filter(|o| o.status == crate::deals::OfferStatus::Open && o.buyer == company)
        .map(|o| o.price)
        .sum();
    Some((budget - p.spent_in(state.date.year()) - bids).max(Money::ZERO))
}

/// A takeover or licence bought: counted against the budget of the year.
pub(crate) fn count_purchase(state: &mut GameState, buyer: CompanyId, price: Money) {
    let year = state.date.year();
    let p = &mut state.companies[buyer.index()].participations;
    if p.year != year {
        p.year = year;
        p.spent = Money::ZERO;
    }
    p.spent += price;
}

/// Up to which amount the head of the department of a position decides alone; `None`
/// where nothing limits it.
pub fn release_limit(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    position: &Position,
) -> Option<Money> {
    let limits = &state.companies[company.index()].participations.limits;
    if limits.is_empty() || position.unit != Unit::Board {
        return None;
    }
    let Role::Specialist(function) = &position.role else {
        return None;
    };
    catalog
        .central
        .departments
        .iter()
        .find(|d| catalog.management.functions[d.function].key == *function)
        .and_then(|d| limits.get(&d.kind).copied())
}

/// The rate of a new loan replacing one of a company's loans (ZA3): the debt stays.
pub fn refinance_rate(catalog: &Catalog, state: &GameState, company: CompanyId) -> f64 {
    let c = &state.companies[company.index()];
    crate::finance::rate_for_debt(
        catalog,
        (
            c.loans.iter().map(|l| l.balance).sum(),
            c.ledger.total_assets(),
        ),
        state.date,
        premium_cut(catalog, state, company),
    )
}

/// The months a loan still runs, at least one.
pub fn months_left(loan: &crate::state::Loan, today: Date) -> u32 {
    let month = |d: Date| i64::from(d.year()) * 12 + i64::from(d.month());
    let elapsed = (month(today) - month(loan.start)).max(0);
    u32::try_from(i64::from(loan.months) - elapsed)
        .unwrap_or(0)
        .max(1)
}

/// `RefinanceLoan` (docs/FORMELN.md, ZA3): the loan at today's rate for its balance and
/// the months left, against a fee booked as interest. Only with the advantage the data
/// ask for.
pub(crate) fn refinance(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    loan: usize,
) -> Result<(), CommandError> {
    let Some(old) = state.companies[actor.index()].loans.get(loan).cloned() else {
        return Err(CommandError::UnknownLoan);
    };
    let rate = refinance_rate(catalog, state, actor);
    let model = &catalog.central.refinance;
    if old.rate - rate < model.min_advantage || rate >= old.rate {
        return Err(CommandError::NoAdvantage);
    }
    let fee = old.balance.scale(model.fee);
    let today = state.date;
    let c = &mut state.companies[actor.index()];
    if c.ledger.cash() < fee {
        return Err(CommandError::NotEnoughCash { needed: fee });
    }
    if fee > Money::ZERO {
        c.ledger.expense(
            CostType::Interest,
            CostCenter::default(),
            Account::Cash,
            fee,
        );
    }
    let months = months_left(&old, today);
    c.loans[loan] = crate::state::Loan {
        principal: old.balance,
        balance: old.balance,
        rate,
        start: today,
        months,
        instalment: crate::finance::instalment(old.balance, rate, months),
    };
    Ok(())
}

/// What moving the headquarters costs now (docs/FORMELN.md, ZA1).
pub fn relocation_cost(catalog: &Catalog, state: &GameState, company: CompanyId) -> Money {
    let h = &catalog.central.headquarters;
    h.cost_base
        + h.cost_per_employee
            .scale(f64::from(employees(state, company)))
}

/// `SetHeadquarters`: the move to another country starts; it costs at once and is done
/// after the months of the data.
pub(crate) fn set_headquarters(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    country: CountryId,
) -> Result<(), CommandError> {
    if country.index() >= catalog.countries.len() {
        return Err(CommandError::UnknownCountry);
    }
    let c = &state.companies[actor.index()];
    if let Some(r) = c.relocation {
        return Err(CommandError::RelocationUnderWay { until: r.until });
    }
    if c.headquarters == country {
        return Err(CommandError::SameHeadquarters);
    }
    let cost = relocation_cost(catalog, state, actor);
    if c.ledger.cash() < cost {
        return Err(CommandError::NotEnoughCash { needed: cost });
    }
    let until = state.date.add_months(catalog.central.headquarters.months);
    let c = &mut state.companies[actor.index()];
    if cost > Money::ZERO {
        c.ledger
            .expense(CostType::Other, CostCenter::default(), Account::Cash, cost);
    }
    c.relocation = Some(Relocation { country, until });
    Ok(())
}

/// At a month start (docs/FORMELN.md, ZA1): moves that are due are done. Returns the news
/// for the player.
pub fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let player = state.player;
    for (i, c) in state.companies.iter_mut().enumerate() {
        let Some(r) = c.relocation.filter(|r| r.until <= date) else {
            continue;
        };
        c.relocation = None;
        if c.bankrupt {
            continue;
        }
        c.headquarters = r.country;
        let share = catalog.central.headquarters.moving_share;
        let before: u32 = c.departments.values().sum();
        for n in c.departments.values_mut() {
            // Staff counts are small; the cast is exact.
            *n = (f64::from(*n) * share).floor() as u32;
        }
        c.departments.retain(|_, n| *n > 0);
        let left = before - c.departments.values().sum::<u32>();
        // Few companies; the cast is exact.
        if CompanyId(i as u32) == player {
            news.push(
                Message::new(MessageKind::Info, keys::HEADQUARTERS_MOVED).with(
                    "land",
                    Param::Country(catalog.countries.key(r.country).to_owned()),
                ),
            );
            if left > 0 {
                news.push(
                    Message::new(MessageKind::Info, keys::HEADQUARTERS_STAFF_LEFT)
                        .with("anzahl", Param::Integer(i64::from(left))),
                );
            }
        }
    }
    news
}
