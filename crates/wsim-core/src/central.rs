//! Headquarters and central departments (ZA1–ZA3, docs/FORMELN.md, docs/BETEILIGUNGEN.md).

use std::collections::BTreeMap;

use crate::calendar::Date;
use crate::catalog::{Catalog, City, DepartmentKind};
use crate::command::{self, Command, CommandError};
use crate::ids::{CountryId, Id, LaborGroupId, TechnologyId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::management;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{
    Appraisal, CompanyId, GameState, Judgment, Manager, ManagerId, Position, Relocation, Role, Unit,
};

/// Employees of a company's central departments: the posts filled (W2).
pub fn employees(state: &GameState, company: CompanyId) -> u32 {
    state.companies[company.index()]
        .departments_staffed
        .values()
        .sum()
}

/// The posts of a department the company wants (`StaffDepartment`).
pub fn staff(state: &GameState, company: CompanyId, kind: DepartmentKind) -> u32 {
    state.companies[company.index()]
        .departments
        .get(&kind)
        .copied()
        .unwrap_or(0)
}

/// The posts of a department filled from the academics of the city (W2).
pub fn staffed(state: &GameState, company: CompanyId, kind: DepartmentKind) -> u32 {
    state.companies[company.index()]
        .departments_staffed
        .get(&kind)
        .copied()
        .unwrap_or(0)
}

/// A city of a country (W2): the one named, else the capital, else the largest; `None`
/// for a country without cities in the data or an unknown name.
pub fn city_in<'a>(
    catalog: &'a Catalog,
    country: CountryId,
    key: Option<&str>,
) -> Option<&'a City> {
    let cities = &catalog.countries.get(country).cities;
    match key {
        Some(k) => cities.iter().find(|c| c.key == k),
        None => cities.iter().find(|c| c.capital).or_else(|| cities.first()),
    }
}

/// The city of a company's headquarters (W2).
pub fn hq_city<'a>(
    catalog: &'a Catalog,
    state: &GameState,
    company: CompanyId,
) -> Option<&'a City> {
    let c = &state.companies[company.index()];
    city_in(catalog, c.headquarters, c.hq_city.as_deref())
        .or_else(|| city_in(catalog, c.headquarters, None))
}

/// The city's share of its country's population (W2); 1 for a country without cities.
fn city_share(catalog: &Catalog, country: CountryId, city: Option<&City>) -> f64 {
    let (Some(city), Some(m)) = (city, &catalog.central.city) else {
        return 1.0;
    };
    let people = catalog
        .countries
        .get(country)
        .values
        .population
        .value_at(f64::from(m.population_year));
    if people > 0.0 {
        (city.population / people).min(1.0)
    } else {
        1.0
    }
}

/// Inhabitants of a city this year (W2): its share of the country's population.
pub fn city_population(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    city: Option<&City>,
) -> f64 {
    city_share(catalog, country, city) * state.countries.get(country).population
}

/// Academics of a group open to the central departments of all companies in a city (W2);
/// unlimited without the model.
pub fn city_academics(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    city: Option<&City>,
    group: LaborGroupId,
) -> f64 {
    let Some(m) = &catalog.central.city else {
        return f64::INFINITY;
    };
    let pool = state
        .countries
        .get(country)
        .labor_pool
        .get(group.index())
        .copied()
        .unwrap_or(0.0);
    pool * (m.academics_concentration * city_share(catalog, country, city)).min(1.0) * m.hq_share
}

/// Office costs per employee and year in a city (W2): the data's amount at the price
/// level of the country, more in large cities.
pub fn office_per_employee(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    city: Option<&City>,
    office: Money,
) -> Money {
    let Some(m) = &catalog.central.city else {
        return office;
    };
    let people = city_population(catalog, state, country, city).max(1.0);
    let factor = state.countries.get(country).price_level
        * crate::math::pow(people / m.office_reference_population, m.office_elasticity);
    office.scale(factor)
}

/// Posts all companies with their headquarters in a city want (W2).
pub fn city_wanted(catalog: &Catalog, state: &GameState, country: CountryId, city: &str) -> u32 {
    state
        .companies
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            // Few companies; the cast is exact.
            !c.bankrupt
                && c.headquarters == country
                && hq_city(catalog, state, CompanyId(*i as u32)).is_some_and(|x| x.key == city)
        })
        .map(|(_, c)| c.departments.values().sum::<u32>())
        .sum()
}

/// Where central departments hire (W2): country, city key and labor group.
type Place = (CountryId, String, LaborGroupId);

/// Fills the posts of all central departments from the academics of their cities (W2):
/// where all companies of a city want more than there are, each gets its share.
pub(crate) fn refresh_staffing(state: &mut GameState, catalog: &Catalog) {
    // Posts wanted per country, city and labor group.
    let mut wanted: BTreeMap<Place, u64> = BTreeMap::new();
    let mut posts: Vec<Vec<(DepartmentKind, u32, Place)>> =
        Vec::with_capacity(state.companies.len());
    for (i, c) in state.companies.iter().enumerate() {
        let mut own = Vec::new();
        if !c.bankrupt && !c.departments.is_empty() {
            // Few companies; the cast is exact.
            let city = hq_city(catalog, state, CompanyId(i as u32))
                .map(|x| x.key.clone())
                .unwrap_or_default();
            for (&kind, &n) in &c.departments {
                let Some(d) = catalog.central.department(kind) else {
                    continue;
                };
                let place = (c.headquarters, city.clone(), d.labor_group);
                *wanted.entry(place.clone()).or_default() += u64::from(n);
                own.push((kind, n, place));
            }
        }
        posts.push(own);
    }
    let open: BTreeMap<Place, f64> = wanted
        .keys()
        .map(|place| {
            let (country, city, group) = place;
            let city = city_in(
                catalog,
                *country,
                (!city.is_empty()).then_some(city.as_str()),
            );
            (
                place.clone(),
                city_academics(catalog, state, *country, city, *group),
            )
        })
        .collect();
    for (c, own) in state.companies.iter_mut().zip(posts) {
        c.departments_staffed.clear();
        for (kind, n, place) in own {
            let all = wanted[&place] as f64;
            let pool = open[&place];
            let filled = if all <= pool {
                n
            } else {
                // Posts are small counts; the cast saturates.
                (f64::from(n) * pool / all).floor() as u32
            };
            if filled > 0 {
                c.departments_staffed.insert(kind, filled);
            }
        }
    }
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
    let n = staffed(state, company, kind);
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
    let city = hq_city(catalog, state, company);
    let (mut personnel, mut office) = (Money::ZERO, Money::ZERO);
    for (&kind, &n) in &c.departments_staffed {
        let Some(d) = catalog.central.department(kind) else {
            continue;
        };
        let wage = management::group_yearly_wage(catalog, state, c.headquarters, d.labor_group);
        personnel += Money::from_usd(f64::from(n) * wage / 12.0).unwrap_or(Money::ZERO);
        office += office_per_employee(catalog, state, c.headquarters, city, d.office)
            .scale(f64::from(n) / 12.0);
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
    refresh_staffing(state, catalog);
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
            c.loans
                .iter()
                .filter(|l| !l.from_person)
                .map(|l| l.balance)
                .sum(),
            c.ledger.total_assets(),
        ),
        state.date,
        premium_cut(catalog, state, company),
    ) + crate::private::loan_rate_offset(catalog, state, company)
}

/// Loans a finance department proposes to refinance (docs/FORMELN.md, ZA3): those whose
/// rate lies the least advantage of the data above a new loan's, the dearest first.
pub fn refinance_candidates(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
) -> Vec<usize> {
    let rate = refinance_rate(catalog, state, company);
    let min = catalog.central.refinance.min_advantage;
    let mut loans: Vec<(usize, f64)> = state.companies[company.index()]
        .loans
        .iter()
        .enumerate()
        .filter(|(_, l)| !l.from_person && l.rate - rate >= min && l.rate > rate)
        .map(|(i, l)| (i, l.rate))
        .collect();
    loans.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    loans.into_iter().map(|(i, _)| i).collect()
}

/// Managers a personnel department proposes to raise (docs/FORMELN.md, ZA3): less than
/// content and paid below their market value, the unhappiest first; with that value.
pub fn salary_round_candidates(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
) -> Vec<(ManagerId, Money)> {
    let mixed = catalog.management.market.satisfaction.bands[1];
    let mut out: Vec<(u8, ManagerId, Money)> = state
        .managers
        .iter()
        .filter_map(|(&id, m)| {
            let job = m.job.as_ref().filter(|j| j.company == company)?;
            let happy = crate::staffing::satisfaction(catalog, job);
            let value = crate::staffing::market_value(catalog, state, m);
            (happy < mixed && value > job.salary).then_some((happy, id, value))
        })
        .collect();
    out.sort_by_key(|&(happy, id, _)| (happy, id));
    out.into_iter().map(|(_, id, value)| (id, value)).collect()
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
    let Some(old) = state.companies[actor.index()]
        .loans
        .get(loan)
        .filter(|l| !l.from_person)
        .cloned()
    else {
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
        lender: None,
        from_person: false,
    };
    // The market's bank pays off the player's bank that lent the old loan (K4).
    if let Some(bank) = old.lender {
        crate::bank::receive(state, bank, old.balance, Money::ZERO);
    }
    Ok(())
}

/// A head's hit rate with the prior (docs/FORMELN.md, ZA3): few cases do not deceive.
pub fn hit_rate(catalog: &Catalog, manager: &Manager) -> f64 {
    let h = &catalog.central.hit_rate;
    let n = f64::from(manager.judged) + h.prior;
    if n > 0.0 {
        (f64::from(manager.hits) + h.mean * h.prior) / n
    } else {
        h.mean
    }
}

/// The factor of the hit rate on the salary demand and the strength others see
/// (docs/FORMELN.md, ZA3): e^(k · (q − mean)); 1 before the first judgment.
pub fn hit_factor(catalog: &Catalog, manager: &Manager) -> f64 {
    if manager.judged == 0 {
        return 1.0;
    }
    let h = &catalog.central.hit_rate;
    crate::math::exp(h.k * (hit_rate(catalog, manager) - h.mean))
}

/// An estimate of a target is judged after the months of the data (ZA3).
pub(crate) fn record_judgment(
    state: &mut GameState,
    catalog: &Catalog,
    manager: ManagerId,
    hit: bool,
) {
    let due = state.date.add_months(catalog.central.hit_rate.months);
    state.judgments.push(Judgment {
        manager,
        due,
        hit,
        appraisal: None,
    });
}

/// An estimate judged by how things turned out when it is due (ZA4). A start-up counts
/// once for a head while its judgment is pending.
pub(crate) fn record_appraisal(
    state: &mut GameState,
    catalog: &Catalog,
    manager: ManagerId,
    appraisal: Appraisal,
) {
    if let Appraisal::Venture { venture, .. } = appraisal && state.judgments.iter().any(|j| {
        j.manager == manager
            && matches!(j.appraisal, Some(Appraisal::Venture { venture: v, .. }) if v == venture)
    }) {
        return;
    }
    let due = state.date.add_months(catalog.central.hit_rate.months);
    state.judgments.push(Judgment {
        manager,
        due,
        hit: false,
        appraisal: Some(appraisal),
    });
}

/// What a takeover estimate is judged by (docs/FORMELN.md, ZA4): the sites of the object,
/// the brand and base value today, the value by the rules and the price; none for a
/// licence.
pub(crate) fn takeover_appraisal(
    state: &GameState,
    catalog: &Catalog,
    (seller, object): (CompanyId, crate::deals::DealObject),
    value: Money,
    price: Money,
) -> Option<Appraisal> {
    use crate::deals::DealObject;
    let (sites, brand, base) = match object {
        DealObject::Site(site) => (
            vec![site],
            Money::ZERO,
            crate::deals::site_value(state, catalog, site).base,
        ),
        DealObject::Area(group) => {
            let v = crate::deals::area_value(state, catalog, seller, group);
            (v.sites.iter().map(|&(s, _)| s).collect(), v.brand, v.base)
        }
        DealObject::License(_) => return None,
    };
    Some(Appraisal::Takeover {
        sites,
        brand,
        base,
        value,
        price,
    })
}

/// Whether a judgment that is due is a hit (docs/FORMELN.md, ZA3, ZA4).
pub fn judged_hit(catalog: &Catalog, state: &GameState, j: &Judgment) -> bool {
    match &j.appraisal {
        None => j.hit,
        Some(Appraisal::Takeover {
            sites,
            brand,
            base,
            value,
            price,
        }) => {
            if *base <= Money::ZERO {
                return price <= value;
            }
            let now = sites
                .iter()
                .filter(|s| s.index() < state.sites.len())
                .map(|&s| crate::deals::site_value(state, catalog, s).base)
                .sum::<Money>()
                + *brand;
            price.to_usd() <= value.to_usd() * now.to_usd() / base.to_usd()
        }
        Some(Appraisal::Venture {
            venture,
            amount,
            share,
        }) => {
            let m = &catalog.ventures;
            let Some(v) = state.ventures.iter().find(|v| v.id == *venture) else {
                return false;
            };
            match v.status {
                crate::state::VentureStatus::Succeeded(_) => true,
                crate::state::VentureStatus::Failed(..) => false,
                crate::state::VentureStatus::Active => {
                    let expected = crate::ventures::success_chance(m, v)
                        * crate::ventures::expected_success_value(state, m, v).to_usd()
                        * share;
                    expected >= amount.to_usd()
                }
            }
        }
    }
}

/// Judgments that are due count for their heads.
fn judge(state: &mut GameState, catalog: &Catalog, date: Date) {
    let (due, rest): (Vec<Judgment>, Vec<Judgment>) =
        state.judgments.iter().cloned().partition(|j| j.due <= date);
    state.judgments = rest;
    for j in due {
        let hit = judged_hit(catalog, state, &j);
        if let Some(m) = state.managers.get_mut(&j.manager) {
            m.judged += 1;
            m.hits += u32::from(hit);
        }
    }
}

/// Revenue and result before taxes of a company over the last twelve closed months.
pub fn year_figures(state: &GameState, company: CompanyId) -> (Money, Money) {
    let months = &state.companies[company.index()].ledger.months;
    let (mut revenue, mut result) = (Money::ZERO, Money::ZERO);
    for m in &months[months.len().saturating_sub(12)..] {
        for (&t, &amount) in &m.by_type {
            if t == CostType::Revenue {
                revenue += amount;
            }
            if t != CostType::Taxes {
                result += amount;
            }
        }
    }
    (revenue, result)
}

/// What a department of `staff` employees costs a company a year (docs/FORMELN.md, ZA4):
/// their wages and offices in the country of the headquarters and its head's salary – the
/// one paid, else what a middling manager asks.
pub fn yearly_cost(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    kind: DepartmentKind,
    staff: u32,
) -> Money {
    let Some(d) = catalog.central.department(kind) else {
        return Money::ZERO;
    };
    let hq = state.companies[company.index()].headquarters;
    let wage = management::group_yearly_wage(catalog, state, hq, d.labor_group);
    let office = office_per_employee(
        catalog,
        state,
        hq,
        hq_city(catalog, state, company),
        d.office,
    );
    let employees = Money::from_usd(f64::from(staff) * wage).unwrap_or(Money::ZERO)
        + office.scale(f64::from(staff));
    let salary = match head(catalog, state, company, kind) {
        Some(id) => state.managers[&id]
            .job
            .as_ref()
            .map_or(Money::ZERO, |j| j.salary),
        None => {
            let factor = catalog
                .management
                .levels
                .get(management::level_of(Unit::Board))
                .map_or(0.0, |l| l.salary_specialist);
            Money::from_usd(factor * management::yearly_wage(catalog, state, hq))
                .unwrap_or(Money::ZERO)
        }
    };
    employees + salary
}

/// The departments an AI company wants this year (docs/FORMELN.md, ZA4): those of the
/// order whose workload reaches its least one, with full coverage, as long as all of
/// them together cost at most its share of the revenue; none after a loss.
pub fn ai_plan(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
) -> BTreeMap<DepartmentKind, u32> {
    let model = &catalog.central.ai;
    let mut wanted = BTreeMap::new();
    let (competence, _) = crate::ai::traits(catalog, state, company);
    let (revenue, result) = year_figures(state, company);
    if result <= Money::ZERO || !catalog.management.enabled() {
        return wanted;
    }
    let budget = revenue.scale(model.revenue_share.at(competence));
    let mut spent = Money::ZERO;
    for &(kind, least) in &model.order {
        let Some(d) = catalog.central.department(kind) else {
            continue;
        };
        let load = workload(state, company, kind);
        if load < least || d.cases <= 0.0 {
            continue;
        }
        // Workloads are small; the cast is exact.
        let n = (load / d.cases).ceil().max(1.0) as u32;
        let cost = yearly_cost(catalog, state, company, kind, n);
        if spent + cost > budget {
            break;
        }
        spent += cost;
        wanted.insert(kind, n);
    }
    wanted
}

/// An AI company sets up the departments it wants and closes the others, letting their
/// heads go (docs/FORMELN.md, ZA4).
fn ai_departments(state: &mut GameState, catalog: &Catalog, company: CompanyId) {
    let wanted = ai_plan(catalog, state, company);
    for kind in catalog
        .central
        .departments
        .iter()
        .map(|d| d.kind)
        .collect::<Vec<_>>()
    {
        let n = wanted.get(&kind).copied().unwrap_or(0);
        if n != staff(state, company, kind) {
            let command = Command::StaffDepartment {
                department: kind,
                staff: n,
            };
            let _ = command::execute(state, catalog, company, &command);
        }
        if n == 0
            && let Some(manager) = head(catalog, state, company, kind)
        {
            let _ = command::execute(
                state,
                catalog,
                company,
                &Command::DismissManager { manager },
            );
        }
    }
}

/// What moving to a country would save a company a year (docs/FORMELN.md, ZA4): profit
/// tax on its result before taxes and the wages of its departments' employees.
pub fn seat_saving(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    country: CountryId,
) -> Money {
    let c = &state.companies[company.index()];
    let (_, result) = year_figures(state, company);
    let hq = c.headquarters;
    let tax = (state.countries.get(hq).corporate_tax - state.countries.get(country).corporate_tax)
        * result.max(Money::ZERO).to_usd();
    let mut wages = 0.0;
    for (&kind, &n) in &c.departments {
        let Some(d) = catalog.central.department(kind) else {
            continue;
        };
        let now = management::group_yearly_wage(catalog, state, hq, d.labor_group);
        let there = management::group_yearly_wage(catalog, state, country, d.labor_group);
        wages += f64::from(n) * (now - there);
    }
    Money::from_usd(tax + wages).unwrap_or(Money::ZERO)
}

/// The country an AI company would move its headquarters to (docs/FORMELN.md, ZA4): of
/// those where its sites bring enough of its revenue and that are not much poorer, the
/// one saving most, if the savings pay for the move in the years of the data; none during
/// a move or the years after one.
pub fn ai_seat(catalog: &Catalog, state: &GameState, company: CompanyId) -> Option<CountryId> {
    let own: Vec<crate::state::SiteId> = (0..state.sites.len())
        // Few sites; the cast is exact.
        .map(|i| crate::state::SiteId(i as u32))
        .filter(|s| state.sites[s.index()].owner == company)
        .collect();
    ai_seat_among(catalog, state, company, &own)
}

/// `ai_seat` with the company's sites given (a January looks at all companies at once).
fn ai_seat_among(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    own: &[crate::state::SiteId],
) -> Option<CountryId> {
    let model = &catalog.central.ai;
    let c = &state.companies[company.index()];
    if model.payback_years <= 0.0 || c.relocation.is_some() {
        return None;
    }
    if c.relocated
        .is_some_and(|d| d.add_months(model.lock_years * 12) > state.date)
    {
        return None;
    }
    let (revenue, _) = year_figures(state, company);
    if revenue <= Money::ZERO {
        return None;
    }
    let months = &c.ledger.months;
    let recent = &months[months.len().saturating_sub(12)..];
    let mut by_country: BTreeMap<CountryId, Money> = BTreeMap::new();
    for &site in own {
        let s = &state.sites[site.index()];
        let earned: Money = recent
            .iter()
            .map(|m| m.site_revenue.get(&site).copied().unwrap_or(Money::ZERO))
            .sum();
        *by_country.entry(s.country).or_default() += earned;
    }
    let cost = relocation_cost(catalog, state, company);
    let least = revenue.scale(model.seat_revenue_share);
    let gdp = |country: CountryId| state.countries.get(country).gdp_per_capita_usd;
    let poorest = gdp(c.headquarters) * model.seat_gdp_share;
    let mut best: Option<(Money, CountryId)> = None;
    for (country, earned) in by_country {
        if country == c.headquarters
            || earned < least
            || earned <= Money::ZERO
            || gdp(country) < poorest
        {
            continue;
        }
        let saving = seat_saving(catalog, state, company, country);
        if saving <= Money::ZERO || saving.scale(model.payback_years) < cost {
            continue;
        }
        // The largest saving; on a tie the first country of the data.
        if best.is_none_or(|(b, _)| saving > b) {
            best = Some((saving, country));
        }
    }
    best.map(|(_, country)| country)
}

/// The city of its country an AI company moves its headquarters to (docs/FORMELN.md, W2):
/// where a department is short of academics, the city with the most of them, if it has
/// more than today's and the cash pays for the move.
pub fn ai_city(catalog: &Catalog, state: &GameState, company: CompanyId) -> Option<String> {
    catalog.central.city.as_ref()?;
    let c = &state.companies[company.index()];
    if c.relocation.is_some() {
        return None;
    }
    let (&kind, _) = c
        .departments
        .iter()
        .find(|&(&k, &n)| staffed(state, company, k) < n)?;
    let group = catalog.central.department(kind)?.labor_group;
    let here = hq_city(catalog, state, company);
    let now = city_academics(catalog, state, c.headquarters, here, group);
    let mut best: Option<(f64, &City)> = None;
    for city in &catalog.countries.get(c.headquarters).cities {
        let open = city_academics(catalog, state, c.headquarters, Some(city), group);
        // The most academics; on a tie the larger city (first in the data).
        if best.is_none_or(|(b, _)| open > b) {
            best = Some((open, city));
        }
    }
    let (open, city) = best?;
    if open <= now || here.is_some_and(|h| h.key == city.key) {
        return None;
    }
    let (cost, _) = move_terms(catalog, state, company, c.headquarters);
    (c.ledger.cash() >= cost).then(|| city.key.clone())
}

/// The AI companies' central departments and headquarters at a month start
/// (docs/FORMELN.md, ZA4): in January they set up their departments and may move; every
/// month their finance departments refinance loans. Returns the news for the player.
pub fn ai_month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let companies: Vec<CompanyId> = state
        .companies
        .iter()
        .enumerate()
        .filter(|(_, c)| c.ai.is_some() && !c.bankrupt)
        // Few companies; the cast is exact.
        .map(|(i, _)| CompanyId(i as u32))
        .collect();
    let mut sites: BTreeMap<CompanyId, Vec<crate::state::SiteId>> = BTreeMap::new();
    if date.month() == 1 {
        for (i, s) in state.sites.iter().enumerate() {
            // Few sites; the cast is exact.
            sites
                .entry(s.owner)
                .or_default()
                .push(crate::state::SiteId(i as u32));
        }
    }
    let none = Vec::new();
    for company in companies {
        if date.month() == 1 {
            ai_departments(state, catalog, company);
            let own = sites.get(&company).unwrap_or(&none);
            let seat = ai_seat_among(catalog, state, company, own);
            if seat.is_none()
                && let Some(city) = ai_city(catalog, state, company)
            {
                let country = state.companies[company.index()].headquarters;
                let command = Command::SetHeadquarters {
                    country,
                    city: Some(city),
                };
                let _ = command::execute(state, catalog, company, &command);
            }
            if let Some(country) = seat {
                let command = Command::SetHeadquarters {
                    country,
                    city: None,
                };
                if command::execute(state, catalog, company, &command).is_ok() {
                    let c = &state.companies[company.index()];
                    let until = c.relocation.as_ref().map_or(date, |r| r.until);
                    news.push(
                        Message::new(MessageKind::Info, keys::RIVAL_HEADQUARTERS)
                            .with("firma", Param::Text(c.name.clone()))
                            .with(
                                "land",
                                Param::Country(catalog.countries.key(country).to_owned()),
                            )
                            .with("datum", Param::Date(until)),
                    );
                }
            }
        }
        let cases = cases(catalog, state, company, DepartmentKind::Finance);
        for loan in refinance_candidates(catalog, state, company)
            .into_iter()
            .take(cases)
        {
            let _ = command::execute(state, catalog, company, &Command::RefinanceLoan { loan });
        }
    }
    news
}

/// What moving the headquarters to another country costs now (docs/FORMELN.md, ZA1).
pub fn relocation_cost(catalog: &Catalog, state: &GameState, company: CompanyId) -> Money {
    let h = &catalog.central.headquarters;
    h.cost_base
        + h.cost_per_employee
            .scale(f64::from(employees(state, company)))
}

/// What a move costs and how many months it takes (W2): to another city of the same
/// country only the share of the data and half the time.
pub fn move_terms(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    country: CountryId,
) -> (Money, u32) {
    let cost = relocation_cost(catalog, state, company);
    let months = catalog.central.headquarters.months;
    match &catalog.central.city {
        Some(m) if state.companies[company.index()].headquarters == country => {
            (cost.scale(m.move_within_country), months.div_ceil(2))
        }
        _ => (cost, months),
    }
}

/// `SetHeadquarters`: the move to another country or city starts (W2: without a city the
/// capital); it costs at once and is done after the months of the data.
pub(crate) fn set_headquarters(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    (country, city): (CountryId, Option<&str>),
) -> Result<(), CommandError> {
    if country.index() >= catalog.countries.len() {
        return Err(CommandError::UnknownCountry);
    }
    let target = match city {
        Some(key) => Some(
            city_in(catalog, country, Some(key))
                .ok_or(CommandError::UnknownCity)?
                .key
                .clone(),
        ),
        None => city_in(catalog, country, None).map(|c| c.key.clone()),
    };
    if crate::events::closed_to(state, actor, country) {
        return Err(CommandError::CountryClosed);
    }
    let c = &state.companies[actor.index()];
    if let Some(r) = &c.relocation {
        return Err(CommandError::RelocationUnderWay { until: r.until });
    }
    let here = hq_city(catalog, state, actor).map(|c| c.key.clone());
    if c.headquarters == country && here == target {
        return Err(CommandError::SameHeadquarters);
    }
    let (cost, months) = move_terms(catalog, state, actor, country);
    if c.ledger.cash() < cost {
        return Err(CommandError::NotEnoughCash { needed: cost });
    }
    let until = state.date.add_months(months);
    let c = &mut state.companies[actor.index()];
    if cost > Money::ZERO {
        c.ledger
            .expense(CostType::Other, CostCenter::default(), Account::Cash, cost);
    }
    c.relocation = Some(Relocation {
        country,
        until,
        city: target,
    });
    Ok(())
}

/// At a month start (docs/FORMELN.md, ZA1): moves that are due are done. Returns the news
/// for the player.
pub fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    judge(state, catalog, date);
    let mut news = Vec::new();
    let player = state.main_company;
    for (i, c) in state.companies.iter_mut().enumerate() {
        let Some(r) = c.relocation.clone().filter(|r| r.until <= date) else {
            continue;
        };
        c.relocation = None;
        if c.bankrupt {
            continue;
        }
        c.headquarters = r.country;
        c.hq_city = r.city.clone();
        c.relocated = Some(date);
        let share = catalog.central.headquarters.moving_share;
        let before: u32 = c.departments.values().sum();
        for n in c.departments.values_mut() {
            // Staff counts are small; the cast is exact.
            *n = (f64::from(*n) * share).floor() as u32;
        }
        c.departments.retain(|_, n| *n > 0);
        let left = before - c.departments.values().sum::<u32>();
        // Few companies; the cast is exact.
        if Some(CompanyId(i as u32)) == player {
            let land = Param::Country(catalog.countries.key(r.country).to_owned());
            news.push(match &r.city {
                Some(city) => Message::new(MessageKind::Info, keys::HEADQUARTERS_MOVED_CITY)
                    .with("land", land)
                    .with("stadt", Param::TextKey(city_text(catalog, r.country, city))),
                None => {
                    Message::new(MessageKind::Info, keys::HEADQUARTERS_MOVED).with("land", land)
                }
            });
            if left > 0 {
                news.push(
                    Message::new(MessageKind::Info, keys::HEADQUARTERS_STAFF_LEFT)
                        .with("anzahl", Param::Integer(i64::from(left))),
                );
            }
        }
    }
    refresh_staffing(state, catalog);
    news
}

/// Key of a city's display text (`stadt.<ISO>.<key>`, W2).
pub fn city_text(catalog: &Catalog, country: CountryId, city: &str) -> String {
    format!("stadt.{}.{city}", catalog.countries.key(country))
}
