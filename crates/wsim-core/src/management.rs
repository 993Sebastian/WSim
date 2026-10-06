//! Managers (MA1, docs/FORMELN.md, docs/MANAGER.md): the positions of the sites, the
//! market of candidates per continent, salaries and the routine of the site positions.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::calendar::{Date, days_in_month};
use crate::catalog::{Catalog, ConcernModel, SiteType};
use crate::command::{Command, CommandError, site_type_key};
use crate::decision::{self, Assessment, ChoiceKind, Decider, Decision, Topic, Verdict};
use crate::ids::{CountryId, Id, ProductId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::math;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{
    CompanyId, Concern, ConcernOption, ConcernStatus, Followup, GameState, Job, Manager, ManagerId,
    Position, PositionLog, PositionState, Role, SiteId,
};

/// Keys of the skills besides the expertise per function (impressions, views).
pub const DETECTION: &str = "erkennen";
pub const JUDGMENT: &str = "urteil";
pub const LEADERSHIP: &str = "fuehrung";
pub const RISK: &str = "risiko";
pub const TALKATIVENESS: &str = "fragefreude";

/// Number of levels the player sees a skill in.
pub const SHOWN_LEVELS: u8 = 5;

/// Key of the expertise in a function among the skills.
pub fn expertise_key(function: &str) -> String {
    format!("fach.{function}")
}

/// Keys of all skills of a manager, in the order they are drawn.
pub fn skill_keys(catalog: &Catalog) -> Vec<String> {
    let m = &catalog.management;
    m.functions
        .iter()
        .map(|f| expertise_key(&f.key))
        .chain(
            [DETECTION, JUDGMENT, LEADERSHIP, RISK, TALKATIVENESS]
                .iter()
                .map(|k| (*k).to_owned()),
        )
        .collect()
}

/// A skill of a manager by its key (0–100).
pub fn skill(manager: &Manager, key: &str) -> Option<u8> {
    match key {
        DETECTION => Some(manager.detection),
        JUDGMENT => Some(manager.judgment),
        LEADERSHIP => Some(manager.leadership),
        RISK => Some(manager.risk),
        TALKATIVENESS => Some(manager.talkativeness),
        _ => manager.expertise.get(key.strip_prefix("fach.")?).copied(),
    }
}

/// The level (0 = weak … 4 = outstanding) the player sees of a skill: value plus the
/// fixed impression, in equal bands over 0–100.
pub fn shown_level(value: u8, impression: i8) -> u8 {
    let seen = (i16::from(value) + i16::from(impression)).clamp(0, 100);
    let level = seen * i16::from(SHOWN_LEVELS) / 100;
    // At most SHOWN_LEVELS; the cast is exact.
    level.min(i16::from(SHOWN_LEVELS) - 1) as u8
}

/// Expertise of a manager in a function (0–100).
pub fn expertise(manager: &Manager, function: &str) -> u8 {
    manager.expertise.get(function).copied().unwrap_or(0)
}

/// What a salary follows: expertise in the focus, detection and judgment.
pub fn strength(manager: &Manager) -> f64 {
    (f64::from(expertise(manager, &manager.focus))
        + f64::from(manager.detection)
        + f64::from(manager.judgment))
        / 3.0
}

/// The positions of a site: its head and the specialists of its type.
pub fn positions(catalog: &Catalog, state: &GameState, site: SiteId) -> Vec<Position> {
    let m = &catalog.management;
    let Some(s) = state.sites.get(site.index()) else {
        return Vec::new();
    };
    if !m.enabled() {
        return Vec::new();
    }
    std::iter::once(Role::Head)
        .chain(
            m.specialists_of(s.kind)
                .iter()
                .map(|&f| Role::Specialist(m.functions[f].key.clone())),
        )
        .map(|role| Position { site, role })
        .collect()
}

/// Who holds a position.
pub fn holder(state: &GameState, position: &Position) -> Option<ManagerId> {
    state
        .managers
        .iter()
        .find(|(_, m)| m.job.as_ref().is_some_and(|j| j.position == *position))
        .map(|(&id, _)| id)
}

/// Yearly wage of the salary group in a country (USD, docs/FORMELN.md, MA1).
fn yearly_wage(catalog: &Catalog, state: &GameState, country: CountryId) -> f64 {
    let Some(group) = catalog.management.salary_group else {
        return 0.0;
    };
    let hourly = state
        .countries
        .get(country)
        .hourly_wage_usd
        .get(group.index())
        .copied()
        .unwrap_or(0.0);
    hourly
        * catalog
            .country_model
            .annual_hours
            .value_at(f64::from(state.date.year()))
}

/// What a manager asks for a position per year (docs/FORMELN.md, MA1).
pub fn salary_demand(
    catalog: &Catalog,
    state: &GameState,
    manager: &Manager,
    position: &Position,
) -> Money {
    let Some(level) = catalog.management.levels.first() else {
        return Money::ZERO;
    };
    let factor = match position.role {
        Role::Head => level.salary_head,
        Role::Specialist(_) => level.salary_specialist,
    };
    let country = state.sites[position.site.index()].country;
    let wage = yearly_wage(catalog, state, country);
    Money::from_usd(factor * (0.5 + strength(manager) / 100.0) * wage).unwrap_or(Money::ZERO)
}

/// Salary of a job for the days from `from` up to (not including) `to`, within a month:
/// a twelfth of the yearly salary per month, by days.
fn salary_for(job: &Job, from: Date, to: Date) -> Money {
    let days = i64::from(from.days_until(to).max(0));
    let month = i64::from(days_in_month(from.year(), from.month()));
    Money::from_units(job.salary.units() / 12 * days / month.max(1))
}

fn book_personnel(state: &mut GameState, company: CompanyId, site: SiteId, amount: Money) {
    if amount <= Money::ZERO {
        return;
    }
    state.companies[company.index()].ledger.expense(
        CostType::Personnel,
        CostCenter::site(site),
        Account::Cash,
        amount,
    );
}

/// Ends a job on a day: the salary of the month so far is booked, the manager returns to
/// the market.
fn end_job(state: &mut GameState, manager: ManagerId, today: Date) {
    let Some(job) = state.managers.get_mut(&manager).and_then(|m| m.job.take()) else {
        return;
    };
    let from = job.since.max(today.first_of_month());
    let pay = salary_for(&job, from, today);
    if !state.companies[job.company.index()].bankrupt {
        book_personnel(state, job.company, job.position.site, pay);
    }
}

/// Checks that a position belongs to the acting company and is free.
fn check_position(
    catalog: &Catalog,
    state: &GameState,
    actor: CompanyId,
    position: &Position,
) -> Result<(), CommandError> {
    let site = state
        .sites
        .get(position.site.index())
        .ok_or(CommandError::UnknownSite)?;
    if site.owner != actor {
        return Err(CommandError::NotOwner);
    }
    if !positions(catalog, state, position.site).contains(position) {
        return Err(CommandError::UnknownPosition);
    }
    if holder(state, position).is_some() {
        return Err(CommandError::PositionTaken);
    }
    Ok(())
}

/// A manager of the acting company.
fn own_manager(
    state: &GameState,
    actor: CompanyId,
    manager: ManagerId,
) -> Result<&Manager, CommandError> {
    let m = state
        .managers
        .get(&manager)
        .ok_or(CommandError::UnknownManager)?;
    match &m.job {
        Some(j) if j.company == actor => Ok(m),
        _ => Err(CommandError::NotYourManager),
    }
}

/// `HireManager`: a free candidate takes a free position of the company at his demand.
pub(crate) fn hire(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    manager: ManagerId,
    position: &Position,
) -> Result<(), CommandError> {
    let m = state
        .managers
        .get(&manager)
        .ok_or(CommandError::UnknownManager)?;
    if m.job.is_some() {
        return Err(CommandError::ManagerEmployed);
    }
    check_position(catalog, state, actor, position)?;
    let salary = salary_demand(catalog, state, m, position);
    let since = state.date;
    state.managers.get_mut(&manager).expect("checked").job = Some(Job {
        company: actor,
        position: position.clone(),
        salary,
        since,
    });
    Ok(())
}

/// `MoveManager`: a manager of the company takes another free position of it; the salary
/// is the higher of the old one and the demand for the new position.
pub(crate) fn move_to(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    manager: ManagerId,
    position: &Position,
) -> Result<(), CommandError> {
    let m = own_manager(state, actor, manager)?;
    check_position(catalog, state, actor, position)?;
    let demand = salary_demand(catalog, state, m, position);
    let today = state.date;
    let job = state
        .managers
        .get_mut(&manager)
        .and_then(|m| m.job.as_mut())
        .expect("checked");
    // The old site pays the days of the month so far, the new one the rest.
    let old = job.clone();
    job.position = position.clone();
    job.salary = job.salary.max(demand);
    job.since = today;
    let pay = salary_for(&old, old.since.max(today.first_of_month()), today);
    book_personnel(state, actor, old.position.site, pay);
    Ok(())
}

/// `DismissManager`: the job ends against a severance pay; the manager returns to the
/// market of his home continent.
pub(crate) fn dismiss(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    manager: ManagerId,
) -> Result<(), CommandError> {
    let job = own_manager(state, actor, manager)?
        .job
        .clone()
        .expect("checked");
    let severance = job.salary.scale(catalog.management.severance_months / 12.0);
    let today = state.date;
    end_job(state, manager, today);
    book_personnel(state, actor, job.position.site, severance);
    Ok(())
}

/// The jobs at a site that changed hands end (M30, M38); the managers return to the
/// market.
pub(crate) fn release_site(state: &mut GameState, site: SiteId) {
    let ended: Vec<ManagerId> = state
        .managers
        .iter()
        .filter(|(_, m)| m.job.as_ref().is_some_and(|j| j.position.site == site))
        .map(|(&id, _)| id)
        .collect();
    let today = state.date;
    for id in ended {
        end_job(state, id, today);
    }
}

/// Jobs that no longer exist end: the company went bankrupt, the site has another owner
/// or its type no longer has the position.
fn end_void_jobs(state: &mut GameState, catalog: &Catalog, today: Date) {
    let void: Vec<ManagerId> = state
        .managers
        .iter()
        .filter(|(_, m)| {
            m.job.as_ref().is_some_and(|j| {
                state.companies[j.company.index()].bankrupt
                    || state.sites[j.position.site.index()].owner != j.company
                    || !positions(catalog, state, j.position.site).contains(&j.position)
            })
        })
        .map(|(&id, _)| id)
        .collect();
    for id in void {
        end_job(state, id, today);
    }
}

/// Salaries of the month, booked on its last day (docs/FORMELN.md, MA1).
pub fn month_end(state: &mut GameState, last: Date) {
    let next = last.next_day();
    let pay: Vec<(CompanyId, SiteId, Money)> = state
        .managers
        .values()
        .filter_map(|m| {
            let j = m.job.as_ref()?;
            let from = j.since.max(last.first_of_month());
            Some((j.company, j.position.site, salary_for(j, from, next)))
        })
        .collect();
    for (company, site, amount) in pay {
        if !state.companies[company.index()].bankrupt {
            book_personnel(state, company, site, amount);
        }
    }
}

/// Academics per country (persons of all fields of the salary group's qualification).
fn academics(catalog: &Catalog, state: &GameState) -> Vec<f64> {
    let groups: Vec<usize> = match catalog.management.salary_group {
        Some(g) => {
            let q = catalog.labor_groups.get(g).qualification;
            catalog
                .labor_groups
                .iter()
                .filter(|(_, x)| x.qualification == q)
                .map(|(id, _)| id.index())
                .collect()
        }
        None => Vec::new(),
    };
    catalog
        .countries
        .ids()
        .map(|c| {
            let pool = &state.countries.get(c).labor_pool;
            groups
                .iter()
                .map(|&g| pool.get(g).copied().unwrap_or(0.0))
                .sum()
        })
        .collect()
}

/// Candidates the market of a continent holds with so many academics (docs/FORMELN.md,
/// MA1).
pub fn pool_size(catalog: &Catalog, academics: f64) -> usize {
    let p = &catalog.management.pool;
    let n = (academics / 1e6 * p.per_million_academics)
        .round()
        .clamp(f64::from(p.min), f64::from(p.max));
    // A few dozen; the cast is exact.
    n as usize
}

/// Month number of a date for the market's random stream.
fn month_number(date: Date) -> u32 {
    u32::try_from(date.year()).unwrap_or(0) * 12 + date.month()
}

/// Day number of a date for the checks and the managers' random streams.
fn day_number(date: Date) -> u32 {
    let epoch = Date::first_of_year(crate::EARLIEST_START_YEAR);
    u32::try_from(epoch.days_until(date)).unwrap_or(0)
}

/// The next day (from `date` on) the positions of a site check their topics.
pub fn next_check(catalog: &Catalog, site: SiteId, date: Date) -> Option<Date> {
    let days = catalog.management.levels.first()?.check_days.max(1);
    let wait = (days - (day_number(date) + site.0) % days) % days;
    Some(date.add_days(i32::try_from(wait).unwrap_or(0)))
}

/// At the start of a month (and when a game starts): jobs that no longer exist end, free
/// candidates leave the market and new ones come until each continent has its number
/// (docs/FORMELN.md, MA1). Old saves get their market at their first month start.
pub fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) {
    let m = &catalog.management;
    if !m.enabled() {
        return;
    }
    end_void_jobs(state, catalog, date);
    tidy_concerns(state, catalog, date);
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::ManagerMarket {
            month: month_number(date),
        },
    );
    let leaving: Vec<ManagerId> = state
        .managers
        .iter()
        .filter(|(_, x)| x.job.is_none())
        .map(|(&id, _)| id)
        .filter(|_| rng.chance(m.pool.leave_per_month))
        .collect();
    for id in leaving {
        state.managers.remove(&id);
    }
    let academics = academics(catalog, state);
    let mut names: BTreeSet<String> = state.managers.values().map(|x| x.name.clone()).collect();
    for continent in catalog.continents.ids() {
        let countries: Vec<(CountryId, f64)> = catalog
            .countries
            .iter()
            .filter(|(_, c)| c.continent == continent)
            .map(|(id, _)| (id, academics[id.index()]))
            .collect();
        if countries.is_empty() {
            continue;
        }
        let total: f64 = countries.iter().map(|c| c.1).sum();
        let free = state
            .managers
            .values()
            .filter(|x| x.job.is_none() && catalog.countries.get(x.home).continent == continent)
            .count();
        for _ in free..pool_size(catalog, total) {
            let home = pick_country(&mut rng, &countries, total);
            let manager = draw(catalog, &mut rng, home, &mut names);
            state
                .managers
                .insert(ManagerId(state.next_manager), manager);
            state.next_manager += 1;
        }
    }
}

/// A country of a continent, weighted by its academics.
fn pick_country(rng: &mut SimRng, countries: &[(CountryId, f64)], total: f64) -> CountryId {
    if total <= 0.0 {
        let n = u64::try_from(countries.len()).unwrap_or(1);
        return countries[usize::try_from(rng.below(n)).unwrap_or(0)].0;
    }
    let mut left = rng.next_f64() * total;
    for &(c, w) in countries {
        if left < w {
            return c;
        }
        left -= w;
    }
    countries[countries.len() - 1].0
}

/// A new candidate (docs/FORMELN.md, MA1).
fn draw(
    catalog: &Catalog,
    rng: &mut SimRng,
    home: CountryId,
    names: &mut BTreeSet<String>,
) -> Manager {
    let m = &catalog.management;
    let s = &m.skills;
    let normal = |rng: &mut SimRng, (mean, spread): (f64, f64)| -> u8 {
        let u = rng.next_f64().clamp(1e-9, 1.0 - 1e-9);
        // Within 0–100; the cast is exact.
        (mean + spread * math::normal_inverse(u))
            .round()
            .clamp(0.0, 100.0) as u8
    };
    // Within 0–100; the casts are exact.
    let uniform = |rng: &mut SimRng| rng.below(101) as u8;
    let n = u64::try_from(m.functions.len()).unwrap_or(1);
    let focus = usize::try_from(rng.below(n)).unwrap_or(0);
    let expertise = m
        .functions
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let spread = if i == focus { s.focus } else { s.other };
            (f.key.clone(), normal(rng, spread))
        })
        .collect();
    let detection = normal(rng, s.general);
    let judgment = normal(rng, s.general);
    let leadership = normal(rng, s.general);
    let risk = uniform(rng);
    let talkativeness = uniform(rng);
    let impression = skill_keys(catalog)
        .into_iter()
        .map(|key| {
            let offset = ((rng.next_f64() * 2.0 - 1.0) * s.impression_blur).round();
            // Within ±50; the cast is exact.
            (key, offset as i8)
        })
        .collect();
    let name = manager_name(catalog, rng, home, names);
    Manager {
        name,
        home,
        focus: m.functions[focus].key.clone(),
        expertise,
        detection,
        judgment,
        leadership,
        risk,
        talkativeness,
        impression,
        job: None,
    }
}

/// First and family name from the name group of the home country, not yet taken.
fn manager_name(
    catalog: &Catalog,
    rng: &mut SimRng,
    home: CountryId,
    taken: &mut BTreeSet<String>,
) -> String {
    let groups = &catalog.name_groups;
    let usable =
        |g: &&crate::catalog::NameGroup| !g.first_names.is_empty() && !g.surnames.is_empty();
    let Some(group) = groups
        .iter()
        .filter(usable)
        .find(|g| g.countries.contains(&home))
        .or_else(|| groups.iter().filter(usable).find(|g| g.is_default))
        .or_else(|| groups.iter().find(usable))
    else {
        let name = format!("{}", taken.len() + 1);
        taken.insert(name.clone());
        return name;
    };
    let pick = |rng: &mut SimRng, list: &[String]| -> String {
        let n = u64::try_from(list.len()).unwrap_or(1);
        list[usize::try_from(rng.below(n)).unwrap_or(0)].clone()
    };
    let join = |first: &str, family: &str| {
        if group.surname_first {
            format!("{family} {first}")
        } else {
            format!("{first} {family}")
        }
    };
    for attempt in 0..40 {
        let first = pick(rng, &group.first_names);
        let family = pick(rng, &group.surnames);
        // Rarely all tried names are taken: an initial tells them apart.
        let name = if attempt < 20 {
            join(&first, &family)
        } else {
            // A letter A–Z; the cast is exact.
            let initial = char::from(b'A' + rng.below(26) as u8);
            join(&format!("{first} {initial}."), &family)
        };
        if taken.insert(name.clone()) {
            return name;
        }
    }
    let name = format!(
        "{} {}",
        join(&group.first_names[0], &group.surnames[0]),
        taken.len()
    );
    taken.insert(name.clone());
    name
}

/// Decisions kept per position for display (MA2).
const LOG_KEPT: usize = 12;
/// Closed months a site's result is averaged over for the effect of a decision (MA2).
const RESULT_MONTHS: usize = 3;

/// The player's answer to a concern (MA2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConcernAnswer {
    /// Carry out this option.
    Choose(usize),
    /// Let the position decide: it carries out its recommendation.
    Delegate,
    /// Not to be asked about the topic again: the position decides it within its budget,
    /// beyond it things stay as they are.
    NeverAsk,
    /// The topic rests at the position for a while.
    Decline,
}

/// The recorded state of a position of a company, if any.
pub fn position_state<'a>(
    state: &'a GameState,
    company: CompanyId,
    position: &Position,
) -> Option<&'a PositionState> {
    state.companies[company.index()]
        .positions
        .iter()
        .find(|p| p.position == *position)
}

fn position_state_mut<'a>(
    state: &'a mut GameState,
    company: CompanyId,
    position: &Position,
) -> &'a mut PositionState {
    let year = state.date.year();
    let positions = &mut state.companies[company.index()].positions;
    let index = match positions.iter().position(|p| p.position == *position) {
        Some(i) => i,
        None => {
            positions.push(PositionState {
                position: position.clone(),
                budget: None,
                spent: Money::ZERO,
                year,
                muted: BTreeSet::new(),
                blocked: BTreeMap::new(),
                log: Vec::new(),
            });
            positions.len() - 1
        }
    };
    let p = &mut positions[index];
    // The budget of a year starts anew on 1 January.
    if p.year != year {
        p.year = year;
        p.spent = Money::ZERO;
    }
    p
}

/// What a position's budget refers to (docs/FORMELN.md, MA2): the site's revenue in the
/// last twelve closed months, without revenue its costs.
pub fn budget_base(state: &GameState, company: CompanyId, site: SiteId) -> Money {
    let months = &state.companies[company.index()].ledger.months;
    let recent = &months[months.len().saturating_sub(12)..];
    let revenue: Money = recent
        .iter()
        .filter_map(|m| m.site_revenue.get(&site))
        .copied()
        .sum();
    if revenue > Money::ZERO {
        return revenue;
    }
    let result: Money = recent
        .iter()
        .filter_map(|m| m.by_site.get(&site))
        .copied()
        .sum();
    (-result).max(Money::ZERO)
}

/// The shares of the reference a position may spend per decision and per year: the
/// player's, else the defaults of its level and role.
pub fn budget_shares(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    position: &Position,
) -> (f64, f64) {
    if let Some(set) = position_state(state, company, position).and_then(|p| p.budget) {
        return set;
    }
    catalog
        .management
        .levels
        .first()
        .map_or((0.0, 0.0), |l| match position.role {
            Role::Head => l.budget_head,
            Role::Specialist(_) => l.budget_specialist,
        })
}

/// A position's budget per decision and per year (docs/FORMELN.md, MA2); a share of 0
/// means "always ask", without the floor.
pub fn budget(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    position: &Position,
    salary: Money,
) -> (Money, Money) {
    let (a, b) = budget_shares(catalog, state, company, position);
    let base = budget_base(state, company, position.site);
    let (floor_decision, floor_year) = catalog.management.budget_floor;
    let limit = |share: f64, floor: f64| {
        if share <= 0.0 {
            Money::ZERO
        } else {
            base.scale(share).max(salary.scale(floor))
        }
    };
    (limit(a, floor_decision), limit(b, floor_year))
}

/// What a position spent of its budget this year.
pub fn spent(state: &GameState, company: CompanyId, position: &Position) -> Money {
    position_state(state, company, position)
        .filter(|p| p.year == state.date.year())
        .map_or(Money::ZERO, |p| p.spent)
}

/// Mean monthly result of a site in the last closed months.
fn mean_result(state: &GameState, company: CompanyId, site: SiteId) -> Money {
    let months = &state.companies[company.index()].ledger.months;
    let recent = &months[months.len().saturating_sub(RESULT_MONTHS)..];
    if recent.is_empty() {
        return Money::ZERO;
    }
    let sum: Money = recent
        .iter()
        .filter_map(|m| m.by_site.get(&site))
        .copied()
        .sum();
    // A few months; the cast is exact.
    Money::from_units(sum.units() / recent.len() as i64)
}

/// The amount an option counts against a budget (docs/MANAGER.md 5.1).
fn choice_amount(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    choice: &decision::Choice,
) -> Money {
    choice
        .steps
        .iter()
        .map(|s| decision::amount(catalog, state, company, &s.command))
        .sum()
}

/// Loans are for the finance department and the CEO (MA5).
fn needs_finance(choice: &decision::Choice) -> bool {
    choice.steps.iter().any(|s| {
        matches!(
            s.command,
            Command::TakeLoan { .. } | Command::RepayLoan { .. }
        )
    })
}

/// Forecast range of an effect with a relative error.
fn forecast(effect: Money, error: f64) -> (Money, Money) {
    let a = effect.scale(1.0 - error);
    let b = effect.scale(1.0 + error);
    (a.min(b), a.max(b))
}

/// An amount a position counted against its budget, with its log entry.
type Spent = (Position, Money, Option<PositionLog>);

/// Who takes care of topics at a site on a day, with what it may still spend.
struct Responsible {
    position: Position,
    manager: ManagerId,
    /// Expertise in the function (a head standing in with its discount).
    expertise: f64,
    judgment: f64,
    per_decision: Money,
    left: Money,
    muted: BTreeSet<Topic>,
    blocked: BTreeSet<Topic>,
    open: u32,
}

/// The decider of a company's site positions on a day (docs/FORMELN.md, MA1, MA2): a
/// topic runs at a site where a position noticed it; within its budget the position
/// carries out its recommendation, beyond it the player gets a concern. Everything else
/// stays as it is.
struct Staff<'a> {
    noticed: BTreeSet<(SiteId, Topic)>,
    responsible: BTreeMap<(SiteId, Topic), usize>,
    positions: Vec<Responsible>,
    model: &'a ConcernModel,
    routine: &'a [Topic],
    rngs: BTreeMap<ManagerId, SimRng>,
    seed: u64,
    day: u32,
    today: Date,
    open: BTreeSet<(Topic, SiteId, Option<ProductId>)>,
    spent: Vec<Spent>,
    concerns: Vec<Concern>,
    followups: Vec<Followup>,
}

impl Staff<'_> {
    fn rng(&mut self, manager: ManagerId) -> &mut SimRng {
        let (seed, day) = (self.seed, self.day);
        self.rngs
            .entry(manager)
            .or_insert_with(|| SimRng::for_stream(seed, Stream::Manager { id: manager.0, day }))
    }

    /// The option a position recommends (docs/FORMELN.md, MA2): the best by the true
    /// effects with a chance after its judgment, else the best by its own estimate.
    fn recommend(
        &mut self,
        decision: &Decision,
        assessments: &[Assessment],
        (manager, expertise, judgment): (ManagerId, f64, f64),
    ) -> usize {
        if assessments.iter().all(|a| a.effect.is_none()) {
            return decision.rule;
        }
        let values: Vec<f64> = assessments
            .iter()
            .map(|a| a.effect.unwrap_or(Money::ZERO).to_usd() + a.once.to_usd())
            .collect();
        let best = |values: &[f64]| {
            let mut best = decision.rule;
            for (i, &v) in values.iter().enumerate() {
                if v > values[best] {
                    best = i;
                }
            }
            best
        };
        let base = self.model.recommend_base;
        let error = self.model.estimate_error * (1.0 - expertise / 100.0);
        let rng = self.rng(manager);
        if rng.chance(base + (1.0 - base) * judgment / 100.0) {
            return best(&values);
        }
        let estimated: Vec<f64> = values
            .iter()
            .map(|v| v * (1.0 + (rng.next_f64() * 2.0 - 1.0) * error))
            .collect();
        best(&estimated)
    }
}

impl Decider for Staff<'_> {
    fn looks(&self) -> bool {
        true
    }

    fn decide(&mut self, state: &GameState, catalog: &Catalog, d: &Decision) -> Verdict {
        let Some(site) = d.site else {
            return Verdict::Hold;
        };
        if !self.noticed.contains(&(site, d.topic)) {
            return Verdict::Hold;
        }
        let Some(&index) = self.responsible.get(&(site, d.topic)) else {
            return Verdict::Hold;
        };
        let routine = self.routine.contains(&d.topic);
        // The routine follows the rules; other topics are assessed and recommended.
        let (choice, assessments) = if routine {
            (d.rule, None)
        } else {
            let a = decision::assess(catalog, state, d);
            let r = &self.positions[index];
            let who = (r.manager, r.expertise, r.judgment);
            (self.recommend(d, &a, who), Some(a))
        };
        let Some(option) = d.choices.get(choice) else {
            return Verdict::Hold;
        };
        let amount = assessments.as_ref().map_or_else(
            || choice_amount(catalog, state, d.company, option),
            |a| a[choice].amount,
        );
        let r = &mut self.positions[index];
        if !needs_finance(option) && amount <= r.per_decision && amount <= r.left {
            r.left -= amount;
            let kind = option.kind;
            let effect = assessments.as_ref().and_then(|a| a[choice].effect);
            let acted = !routine && kind != ChoiceKind::Keep;
            let log = (amount > Money::ZERO || acted).then_some(PositionLog {
                date: self.today,
                topic: d.topic,
                kind,
                product: d.product,
                amount,
                effect,
            });
            self.spent.push((r.position.clone(), amount, log));
            if let Some(effect) = effect.filter(|_| acted) {
                self.followups.push(Followup {
                    company: d.company,
                    site,
                    topic: d.topic,
                    kind,
                    product: d.product,
                    forecast: effect,
                    baseline: mean_result(state, d.company, site),
                    due: self
                        .today
                        .add_days(i32::try_from(self.model.followup_days).unwrap_or(i32::MAX)),
                });
            }
            return if choice == d.rule {
                Verdict::Rule
            } else {
                Verdict::Choice(choice)
            };
        }
        let key = (d.topic, site, d.product);
        if r.muted.contains(&d.topic)
            || r.blocked.contains(&d.topic)
            || r.open >= self.model.open_per_position
            || self.open.contains(&key)
        {
            return Verdict::Hold;
        }
        r.open += 1;
        let (position, manager, expertise) = (r.position.clone(), r.manager, r.expertise);
        self.open.insert(key);
        let assessments = assessments.unwrap_or_else(|| decision::assess(catalog, state, d));
        let error = self.model.estimate_error * (1.0 - expertise / 100.0);
        let options = assessments
            .iter()
            .map(|a| ConcernOption {
                amount: a.amount,
                forecast: a.effect.map(|e| forecast(e, error)),
                once: a.once,
            })
            .collect();
        self.concerns.push(Concern {
            id: 0,
            company: d.company,
            position,
            manager,
            decision: d.clone(),
            recommended: choice,
            options,
            created: self.today,
            deadline: self
                .today
                .add_days(i32::try_from(self.model.deadline_days).unwrap_or(i32::MAX)),
            status: ConcernStatus::Open,
            closed: None,
        });
        Verdict::Hold
    }
}

/// Positions held, per company and site.
type Held = BTreeMap<CompanyId, BTreeMap<SiteId, Vec<(Role, ManagerId)>>>;

/// The work of the site positions due today (docs/FORMELN.md, MA1, MA2), before the AI
/// companies decide: concerns expire, effects are reported, the routine runs, at the end
/// of a quarter the structure of the sites, in idle laboratories the next target.
/// Returns the messages for the round report.
pub fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    expire_concerns(state, catalog, date, &mut news);
    report_followups(state, catalog, date, &mut news);
    let m = &catalog.management;
    let Some(level) = m.levels.first() else {
        return news;
    };
    let mut held: Held = BTreeMap::new();
    for (&id, x) in &state.managers {
        if let Some(j) = &x.job {
            held.entry(j.company)
                .or_default()
                .entry(j.position.site)
                .or_default()
                .push((j.position.role.clone(), id));
        }
    }
    let day = day_number(date);
    let quarter_end = date.next_day().day() == 1 && date.month().is_multiple_of(3);
    let first_of_year = date.ordinal() == 1;
    for (company, sites) in held {
        if state.companies[company.index()].bankrupt {
            continue;
        }
        let mut staff = Staff {
            noticed: BTreeSet::new(),
            responsible: BTreeMap::new(),
            positions: Vec::new(),
            model: &m.concerns,
            routine: &m.routine_topics,
            rngs: BTreeMap::new(),
            seed: state.settings.seed,
            day,
            today: date,
            open: state
                .concerns
                .iter()
                .filter(|c| c.company == company && c.status == ConcernStatus::Open)
                .filter_map(|c| Some((c.decision.topic, c.decision.site?, c.decision.product)))
                .collect(),
            spent: Vec::new(),
            concerns: Vec::new(),
            followups: Vec::new(),
        };
        let (mut weekly, mut structural, mut labs) = (Vec::new(), Vec::new(), Vec::new());
        for (&site, roles) in &sites {
            let s = &state.sites[site.index()];
            // Staggered by the site number, like the AI companies.
            let check = (day + site.0).is_multiple_of(level.check_days);
            let lab = s.kind == SiteType::ResearchCenter;
            let idle = s.research.is_none() && s.development.is_none();
            let lab_due = lab && ((check && idle) || first_of_year);
            if !(lab_due || (!lab && (check || quarter_end))) {
                continue;
            }
            let any = notice(state, catalog, (company, site), roles, &mut staff);
            if !any {
                continue;
            }
            if lab_due {
                labs.push(site);
            } else {
                if check {
                    weekly.push(site);
                }
                if quarter_end {
                    structural.push(site);
                }
            }
        }
        if weekly.is_empty() && structural.is_empty() && labs.is_empty() {
            continue;
        }
        let own: Vec<SiteId> = state
            .sites
            .iter()
            .enumerate()
            .filter(|(_, s)| s.owner == company)
            // Few sites; the cast is exact.
            .map(|(i, _)| SiteId(i as u32))
            .collect();
        if !weekly.is_empty() {
            crate::ai::site_routine(state, catalog, company, (&own, &weekly), date, &mut staff);
        }
        if !structural.is_empty() {
            crate::ai::site_structure(
                state,
                catalog,
                company,
                (&own, &structural),
                date,
                &mut staff,
            );
        }
        for lab in labs {
            crate::ai::lab_target(state, catalog, company, (&own, lab), date, &mut staff);
        }
        let Staff {
            spent,
            concerns,
            followups,
            ..
        } = staff;
        record(
            state,
            catalog,
            company,
            (spent, concerns, followups),
            &mut news,
        );
    }
    news
}

/// The positions at a site that notice their topics today (docs/FORMELN.md, MA1): one
/// draw per function from the stream of the responsible manager. Registers who is
/// responsible with what budget; returns whether anything was noticed.
fn notice(
    state: &GameState,
    catalog: &Catalog,
    (company, site): (CompanyId, SiteId),
    roles: &[(Role, ManagerId)],
    staff: &mut Staff,
) -> bool {
    let m = &catalog.management;
    let specialists = m.specialists_of(state.sites[site.index()].kind);
    let head = roles.iter().find(|(r, _)| *r == Role::Head).map(|r| r.1);
    let mut any = false;
    for (index, function) in m.functions.iter().enumerate() {
        if function.topics.is_empty() {
            continue;
        }
        let specialist = roles
            .iter()
            .find(|(r, _)| matches!(r, Role::Specialist(k) if *k == function.key))
            .filter(|_| specialists.contains(&index))
            .map(|r| r.1);
        let (manager, discount) = match (specialist, head) {
            (Some(s), _) => (s, 0.0),
            (None, Some(h)) => (h, m.head_discount),
            (None, None) => continue,
        };
        let x = &state.managers[&manager];
        let Some(job) = &x.job else { continue };
        let skill = f64::from(expertise(x, &function.key)) * (1.0 - discount);
        let diligence = (skill + f64::from(x.detection)) / 2.0;
        let chance = m.notice_base + (1.0 - m.notice_base) * diligence / 100.0;
        if !staff.rng(manager).chance(chance) {
            continue;
        }
        any = true;
        let index = match staff
            .positions
            .iter()
            .position(|p| p.position == job.position)
        {
            Some(i) => i,
            None => {
                let (per_decision, per_year) =
                    budget(catalog, state, company, &job.position, job.salary);
                let ps = position_state(state, company, &job.position);
                let today = state.date;
                staff.positions.push(Responsible {
                    position: job.position.clone(),
                    manager,
                    expertise: skill,
                    judgment: f64::from(x.judgment),
                    per_decision,
                    left: (per_year - spent(state, company, &job.position)).max(Money::ZERO),
                    muted: ps.map(|p| p.muted.clone()).unwrap_or_default(),
                    blocked: ps
                        .map(|p| {
                            p.blocked
                                .iter()
                                .filter(|(_, until)| **until > today)
                                .map(|(&t, _)| t)
                                .collect()
                        })
                        .unwrap_or_default(),
                    open: u32::try_from(
                        state
                            .concerns
                            .iter()
                            .filter(|c| {
                                c.company == company
                                    && c.position == job.position
                                    && c.status == ConcernStatus::Open
                            })
                            .count(),
                    )
                    .unwrap_or(u32::MAX),
                });
                staff.positions.len() - 1
            }
        };
        for &topic in &function.topics {
            staff.noticed.insert((site, topic));
            staff.responsible.insert((site, topic), index);
        }
    }
    any
}

/// Records what the positions of a company did and asked on a day.
fn record(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    (spent, concerns, followups): (Vec<Spent>, Vec<Concern>, Vec<Followup>),
    news: &mut Vec<Message>,
) {
    for (position, amount, log) in spent {
        let p = position_state_mut(state, company, &position);
        p.spent += amount;
        if let Some(log) = log {
            p.log.push(log);
            let excess = p.log.len().saturating_sub(LOG_KEPT);
            p.log.drain(..excess);
        }
    }
    for mut concern in concerns {
        concern.id = state.next_concern;
        state.next_concern += 1;
        if company == state.player {
            news.push(concern_message(
                catalog,
                state,
                keys::CONCERN_NEW,
                MessageKind::Warning,
                &concern,
            ));
        }
        state.concerns.push(concern);
    }
    state.followups.extend(followups);
}

/// Name of a position for messages: the head of a site type or a function.
fn position_param(state: &GameState, position: &Position) -> Param {
    match &position.role {
        Role::Head => {
            let kind = site_type_key(state.sites[position.site.index()].kind);
            Param::TextKey(kind.replace("standorttyp.", "leitung."))
        }
        Role::Specialist(f) => Param::TextKey(format!("bereich.{f}")),
    }
}

fn concern_message(
    catalog: &Catalog,
    state: &GameState,
    key: &str,
    kind: MessageKind,
    c: &Concern,
) -> Message {
    let site = &state.sites[c.position.site.index()];
    Message::new(kind, key)
        .with("stelle", position_param(state, &c.position))
        .with("standort", Param::TextKey(site_type_key(site.kind)))
        .with(
            "land",
            Param::Country(catalog.countries.key(site.country).to_owned()),
        )
        .with(
            "thema",
            Param::TextKey(format!("thema.{}", c.decision.topic.key())),
        )
        .with("frist", Param::Date(c.deadline))
}

/// Open concerns whose deadline passed expire: nothing changes.
fn expire_concerns(state: &mut GameState, catalog: &Catalog, today: Date, news: &mut Vec<Message>) {
    let player = state.player;
    let expired: Vec<usize> = state
        .concerns
        .iter()
        .enumerate()
        .filter(|(_, c)| c.status == ConcernStatus::Open && c.deadline < today)
        .map(|(i, _)| i)
        .collect();
    for i in expired {
        let c = &mut state.concerns[i];
        c.status = ConcernStatus::Expired;
        c.closed = Some(today);
        if c.company == player {
            let c = state.concerns[i].clone();
            news.push(concern_message(
                catalog,
                state,
                keys::CONCERN_EXPIRED,
                MessageKind::Info,
                &c,
            ));
        }
    }
}

/// Reports on the effects of decisions that are due (docs/FORMELN.md, MA2).
fn report_followups(
    state: &mut GameState,
    catalog: &Catalog,
    today: Date,
    news: &mut Vec<Message>,
) {
    let due: Vec<Followup> = state
        .followups
        .iter()
        .filter(|f| f.due <= today)
        .cloned()
        .collect();
    state.followups.retain(|f| f.due > today);
    for f in due {
        let site = &state.sites[f.site.index()];
        if f.company != state.player || site.owner != f.company {
            continue;
        }
        let actual =
            Money::from_units((mean_result(state, f.company, f.site) - f.baseline).units() * 12);
        let mut m = Message::new(MessageKind::Info, keys::CONCERN_EFFECT)
            .with(
                "massnahme",
                Param::TextKey(format!("option.{}", f.kind.key())),
            )
            .with("standort", Param::TextKey(site_type_key(site.kind)))
            .with(
                "land",
                Param::Country(catalog.countries.key(site.country).to_owned()),
            )
            .with("prognose", Param::Money(f.forecast))
            .with("ist", Param::Money(actual));
        if let Some(p) = f.product {
            m = m.with(
                "produkt",
                Param::TextKey(format!("produkt.{}", catalog.products.key(p))),
            );
        }
        news.push(m);
    }
}

/// `AnswerConcern`: the player answers a concern of one of the company's positions.
pub(crate) fn answer(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    concern: u32,
    answer: ConcernAnswer,
) -> Result<(), CommandError> {
    let index = state
        .concerns
        .iter()
        .position(|c| c.id == concern && c.company == actor)
        .ok_or(CommandError::UnknownConcern)?;
    let c = state.concerns[index].clone();
    if c.status != ConcernStatus::Open {
        return Err(CommandError::ConcernClosed);
    }
    let today = state.date;
    let carry_out = |state: &mut GameState, choice: usize| -> Result<(), CommandError> {
        let option = c
            .decision
            .choices
            .get(choice)
            .ok_or(CommandError::UnknownOption)?;
        // The first step decides whether anything happens; later ones may fail.
        if let Some(first) = option.steps.first() {
            crate::command::execute(state, catalog, actor, &first.command)?;
        }
        let rest = decision::Choice {
            kind: option.kind,
            steps: option.steps.iter().skip(1).cloned().collect(),
        };
        decision::execute(state, catalog, actor, &rest);
        if let (Some(site), Some((low, high))) = (
            c.decision.site,
            c.options.get(choice).and_then(|o| o.forecast),
        ) && option.kind != ChoiceKind::Keep
        {
            let days = catalog.management.concerns.followup_days;
            let baseline = mean_result(state, actor, site);
            state.followups.push(Followup {
                company: actor,
                site,
                topic: c.decision.topic,
                kind: option.kind,
                product: c.decision.product,
                forecast: Money::from_units((low.units() + high.units()) / 2),
                baseline,
                due: today.add_days(i32::try_from(days).unwrap_or(i32::MAX)),
            });
        }
        Ok(())
    };
    let status = match answer {
        ConcernAnswer::Choose(choice) => {
            carry_out(state, choice)?;
            ConcernStatus::Chosen(choice)
        }
        ConcernAnswer::Delegate => {
            carry_out(state, c.recommended)?;
            ConcernStatus::Delegated(c.recommended)
        }
        ConcernAnswer::NeverAsk => {
            position_state_mut(state, actor, &c.position)
                .muted
                .insert(c.decision.topic);
            ConcernStatus::Muted
        }
        ConcernAnswer::Decline => {
            let days = catalog.management.concerns.block_days;
            let until = today.add_days(i32::try_from(days).unwrap_or(i32::MAX));
            position_state_mut(state, actor, &c.position)
                .blocked
                .insert(c.decision.topic, until);
            ConcernStatus::Declined
        }
    };
    let c = &mut state.concerns[index];
    c.status = status;
    c.closed = Some(today);
    Ok(())
}

/// `SetBudget`: the shares of a position's reference per decision and per year; `None`
/// restores the defaults of its level.
pub(crate) fn set_budget(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    position: &Position,
    shares: Option<(f64, f64)>,
) -> Result<(), CommandError> {
    let site = state
        .sites
        .get(position.site.index())
        .ok_or(CommandError::UnknownSite)?;
    if site.owner != actor {
        return Err(CommandError::NotOwner);
    }
    if !positions(catalog, state, position.site).contains(position) {
        return Err(CommandError::UnknownPosition);
    }
    if let Some((a, b)) = shares
        && !((0.0..=1.0).contains(&a) && (0.0..=1.0).contains(&b) && a <= b)
    {
        return Err(CommandError::InvalidShare);
    }
    position_state_mut(state, actor, position).budget = shares;
    Ok(())
}

/// Concerns closed more than a year ago are dropped; open ones of positions the company
/// no longer has expire, and so do the settings of such positions.
fn tidy_concerns(state: &mut GameState, catalog: &Catalog, today: Date) {
    let gone: Vec<usize> = state
        .concerns
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            c.status == ConcernStatus::Open
                && (state.sites[c.position.site.index()].owner != c.company
                    || !positions(catalog, state, c.position.site).contains(&c.position))
        })
        .map(|(i, _)| i)
        .collect();
    for i in gone {
        let c = &mut state.concerns[i];
        c.status = ConcernStatus::Expired;
        c.closed = Some(today);
    }
    let year_ago = today.add_days(-365);
    state
        .concerns
        .retain(|c| c.closed.is_none_or(|closed| closed >= year_ago));
    for (i, company) in state.companies.iter_mut().enumerate() {
        // Few companies; the cast is exact.
        let id = CompanyId(i as u32);
        company.positions.retain(|p| {
            state
                .sites
                .get(p.position.site.index())
                .is_some_and(|s| s.owner == id)
        });
    }
}
