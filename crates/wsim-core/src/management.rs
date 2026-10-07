//! Managers (MA1, docs/FORMELN.md, docs/MANAGER.md): the positions of the sites, the
//! market of candidates per continent, salaries and the routine of the site positions.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::calendar::{Date, days_in_month};
use crate::catalog::{Catalog, ConcernModel, DepartmentKind, SiteType};
use crate::command::{Command, CommandError, site_type_key};
use crate::decision::{self, Assessment, ChoiceKind, Decider, Decision, Topic, Verdict};
use crate::ids::{CountryId, Id, LaborGroupId, ProductId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::math;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{
    BudgetRule, CompanyId, Concern, ConcernOption, ConcernPart, ConcernReason, ConcernStatus,
    Followup, GameState, Hop, Job, Manager, ManagerId, Position, PositionKind, PositionLog,
    PositionState, Role, RuleScope, SiteId, Unit, UnitLevel,
};
use crate::strategy;

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

/// The share of the impression in the levels the player sees of a company's candidates
/// and managers (docs/FORMELN.md, MA5): all of it without a board member for personnel,
/// less the more expertise that member has.
pub fn impression_share(catalog: &Catalog, state: &GameState, company: CompanyId) -> f64 {
    let Some(f) = function_of(catalog, Topic::Wage) else {
        return 1.0;
    };
    let key = &catalog.management.functions[f].key;
    let position = Position {
        unit: Unit::Board,
        role: Role::Specialist(key.clone()),
    };
    if !positions(catalog, state, company, Unit::Board).contains(&position) {
        return 1.0;
    }
    holder(state, company, &position).map_or(1.0, |id| {
        let x = &state.managers[&id];
        let sharpness = catalog.management.mandate.personnel_sharpness;
        (1.0 - sharpness * f64::from(expertise(x, key)) / 100.0).clamp(0.0, 1.0)
    })
}

/// The impression shown with a share of it.
pub fn shown_impression(impression: i8, share: f64) -> i8 {
    // |impression| ≤ 100 and share in 0–1; the cast is exact.
    (f64::from(impression) * share).round() as i8
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

/// The level of a unit: 0 site, 1 country, 2 continent (index into the levels).
pub fn level_of(unit: Unit) -> usize {
    match unit {
        Unit::Site(_) => 0,
        Unit::Country(_) => 1,
        Unit::Continent(_) => 2,
        Unit::Board => 3,
    }
}

/// Number of a unit for staggering the checks.
fn unit_number(unit: Unit) -> u32 {
    match unit {
        // Indices of the data and of sites; the casts are exact.
        Unit::Site(s) => s.0,
        Unit::Country(c) => c.index() as u32,
        Unit::Continent(k) => k.index() as u32,
        Unit::Board => 0,
    }
}

/// Whether a company has a unit (MA3, MA5): its site, a site in the country or
/// continent, or any site for the board.
pub fn has_unit(catalog: &Catalog, state: &GameState, company: CompanyId, unit: Unit) -> bool {
    match unit {
        Unit::Board => state.sites.iter().any(|x| x.owner == company),
        Unit::Site(s) => state
            .sites
            .get(s.index())
            .is_some_and(|x| x.owner == company),
        Unit::Country(c) => state
            .sites
            .iter()
            .any(|x| x.owner == company && x.country == c),
        Unit::Continent(k) => state
            .sites
            .iter()
            .any(|x| x.owner == company && catalog.countries.get(x.country).continent == k),
    }
}

/// The units of a company: its sites, then its countries and continents (MA3), then its
/// board (MA5).
pub fn units(catalog: &Catalog, state: &GameState, company: CompanyId) -> Vec<Unit> {
    let mut countries = BTreeSet::new();
    let mut continents = BTreeSet::new();
    let mut out = Vec::new();
    for (i, s) in state.sites.iter().enumerate() {
        if s.owner != company {
            continue;
        }
        // Few sites; the cast is exact.
        out.push(Unit::Site(SiteId(i as u32)));
        countries.insert(s.country);
        continents.insert(catalog.countries.get(s.country).continent);
    }
    out.extend(countries.into_iter().map(Unit::Country));
    out.extend(continents.into_iter().map(Unit::Continent));
    if !out.is_empty() {
        out.push(Unit::Board);
    }
    out
}

/// The specialist functions of a unit (indices into the functions).
fn specialist_functions<'a>(catalog: &'a Catalog, state: &GameState, unit: Unit) -> &'a [usize] {
    let m = &catalog.management;
    match unit {
        Unit::Site(s) => state
            .sites
            .get(s.index())
            .map_or(&[], |x| m.specialists_of(x.kind)),
        _ => m
            .levels
            .get(level_of(unit))
            .map_or(&[], |l| l.specialists.as_slice()),
    }
}

/// The positions of a unit of a company: its head and its specialists (MA1, MA3); none
/// for a unit the company does not have or a level without data.
pub fn positions(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    unit: Unit,
) -> Vec<Position> {
    let m = &catalog.management;
    if !m.enabled()
        || m.levels.get(level_of(unit)).is_none()
        || !has_unit(catalog, state, company, unit)
    {
        return Vec::new();
    }
    std::iter::once(Role::Head)
        .chain(
            specialist_functions(catalog, state, unit)
                .iter()
                .map(|&f| Role::Specialist(m.functions[f].key.clone())),
        )
        .map(|role| Position { unit, role })
        .collect()
}

/// Whether the positions of a unit take up a topic themselves (MA2, MA3): the topics of
/// its level; research and development only in laboratories, nothing else there.
pub fn arises(catalog: &Catalog, state: &GameState, unit: Unit, topic: Topic) -> bool {
    let listed = catalog
        .management
        .levels
        .get(level_of(unit))
        .is_some_and(|l| l.topics.contains(&topic));
    match unit {
        Unit::Site(s) => {
            let lab = state
                .sites
                .get(s.index())
                .is_some_and(|x| x.kind == SiteType::ResearchCenter);
            listed && matches!(topic, Topic::Research | Topic::Development) == lab
        }
        _ => listed,
    }
}

/// The function a topic belongs to.
pub(crate) fn function_of(catalog: &Catalog, topic: Topic) -> Option<usize> {
    catalog
        .management
        .functions
        .iter()
        .position(|f| f.topics.contains(&topic))
}

/// The country of a unit at or below the country level.
fn country_of(state: &GameState, unit: Unit) -> Option<CountryId> {
    match unit {
        Unit::Site(s) => state.sites.get(s.index()).map(|x| x.country),
        Unit::Country(c) => Some(c),
        Unit::Continent(_) | Unit::Board => None,
    }
}

/// The chain of positions for a topic at a site, in a country or at the board
/// (docs/FORMELN.md, MA3, MA5): the specialist of its function and the head of each unit
/// from the place up to the board – at a site only where the topic comes up there.
/// Filled or not.
pub fn chain(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    place: Unit,
    topic: Topic,
) -> Vec<Position> {
    let m = &catalog.management;
    let function = function_of(catalog, topic);
    let mut out = Vec::new();
    let add = |unit: Unit, out: &mut Vec<Position>| {
        if !has_unit(catalog, state, company, unit) || m.levels.get(level_of(unit)).is_none() {
            return;
        }
        if let Some(f) = function.filter(|f| specialist_functions(catalog, state, unit).contains(f))
        {
            out.push(Position {
                unit,
                role: Role::Specialist(m.functions[f].key.clone()),
            });
        }
        out.push(Position {
            unit,
            role: Role::Head,
        });
    };
    if let Unit::Site(_) = place
        && arises(catalog, state, place, topic)
    {
        add(place, &mut out);
    }
    let continent = match place {
        Unit::Continent(k) => Some(k),
        Unit::Board => None,
        _ => country_of(state, place).map(|c| {
            add(Unit::Country(c), &mut out);
            catalog.countries.get(c).continent
        }),
    };
    if let Some(k) = continent {
        add(Unit::Continent(k), &mut out);
    }
    add(Unit::Board, &mut out);
    out
}

/// Whether a filled position takes up a topic at a site itself (MA2, MA3): one of the
/// chain whose own level has the topic.
pub fn covered(catalog: &Catalog, state: &GameState, site: SiteId, topic: Topic) -> bool {
    let Some(s) = state.sites.get(site.index()) else {
        return false;
    };
    chain(catalog, state, s.owner, Unit::Site(site), topic)
        .iter()
        .any(|p| arises(catalog, state, p.unit, topic) && holder(state, s.owner, p).is_some())
}

/// Who holds a position of a company (positions of countries, continents and the board
/// exist in every company, MA6).
pub fn holder(state: &GameState, company: CompanyId, position: &Position) -> Option<ManagerId> {
    state
        .managers
        .iter()
        .find(|(_, m)| {
            m.job
                .as_ref()
                .is_some_and(|j| j.company == company && j.position == *position)
        })
        .map(|(&id, _)| id)
}

/// Yearly wage of the salary group in a country (USD, docs/FORMELN.md, MA1).
pub fn yearly_wage(catalog: &Catalog, state: &GameState, country: CountryId) -> f64 {
    catalog.management.salary_group.map_or(0.0, |group| {
        group_yearly_wage(catalog, state, country, group)
    })
}

/// The yearly wage of a labour group in a country today.
pub fn group_yearly_wage(
    catalog: &Catalog,
    state: &GameState,
    country: CountryId,
    group: LaborGroupId,
) -> f64 {
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

/// The country whose wages a unit's salaries follow (docs/FORMELN.md, MA3, MA5): a
/// site's and a country's own; for a continent the headquarters' country if it lies
/// there, else the one with the most of the company's sites; for the board the
/// headquarters' country.
pub fn seat_country(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    unit: Unit,
) -> Option<CountryId> {
    match unit {
        Unit::Site(_) | Unit::Country(_) => country_of(state, unit),
        Unit::Board => Some(state.companies.get(company.index())?.headquarters),
        Unit::Continent(k) => {
            let hq = state.companies.get(company.index())?.headquarters;
            if catalog.countries.get(hq).continent == k {
                return Some(hq);
            }
            let mut count: BTreeMap<CountryId, usize> = BTreeMap::new();
            for s in &state.sites {
                if s.owner == company && catalog.countries.get(s.country).continent == k {
                    *count.entry(s.country).or_default() += 1;
                }
            }
            // The most sites; on a tie the first country of the data.
            count
                .into_iter()
                .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
                .map(|(c, _)| c)
        }
    }
}

/// What a manager asks for a position of a company per year (docs/FORMELN.md, MA1, MA3).
pub fn salary_demand(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    manager: &Manager,
    position: &Position,
) -> Money {
    let Some(level) = catalog.management.levels.get(level_of(position.unit)) else {
        return Money::ZERO;
    };
    let factor = match position.role {
        Role::Head => level.salary_head,
        Role::Specialist(_) => level.salary_specialist,
    };
    let Some(country) = seat_country(catalog, state, company, position.unit) else {
        return Money::ZERO;
    };
    let wage = yearly_wage(catalog, state, country);
    // The hit rate of a head of strategy or legal (ZA3).
    let hits = crate::central::hit_factor(catalog, manager);
    Money::from_usd(factor * (0.5 + strength(manager) / 100.0) * wage * hits).unwrap_or(Money::ZERO)
}

/// Salary of a job for the days from `from` up to (not including) `to`, within a month:
/// a twelfth of the yearly salary per month, by days.
fn salary_for(job: &Job, from: Date, to: Date) -> Money {
    let days = i64::from(from.days_until(to).max(0));
    let month = i64::from(days_in_month(from.year(), from.month()));
    Money::from_units(job.salary.units() / 12 * days / month.max(1))
}

/// Salaries are personnel costs of the site, those of higher positions overhead (MA3).
fn book_personnel(state: &mut GameState, company: CompanyId, unit: Unit, amount: Money) {
    if amount <= Money::ZERO {
        return;
    }
    let center = match unit {
        Unit::Site(site) => CostCenter::site(site),
        _ => CostCenter {
            site: None,
            product: None,
        },
    };
    state.companies[company.index()].ledger.expense(
        CostType::Personnel,
        center,
        Account::Cash,
        amount,
    );
}

/// Ends a job on a day: the salary of the month so far is booked, the manager returns to
/// the market.
pub(crate) fn end_job(state: &mut GameState, manager: ManagerId, today: Date) {
    let Some(job) = state.managers.get_mut(&manager).and_then(|m| m.job.take()) else {
        return;
    };
    let from = job.since.max(today.first_of_month());
    let pay = salary_for(&job, from, today);
    if !state.companies[job.company.index()].bankrupt {
        book_personnel(state, job.company, job.position.unit, pay);
    }
}

/// Checks that a position belongs to the acting company.
pub(crate) fn check_own_position(
    catalog: &Catalog,
    state: &GameState,
    actor: CompanyId,
    position: &Position,
) -> Result<(), CommandError> {
    if let Some(site) = position.site() {
        let site = state
            .sites
            .get(site.index())
            .ok_or(CommandError::UnknownSite)?;
        if site.owner != actor {
            return Err(CommandError::NotOwner);
        }
    }
    if !positions(catalog, state, actor, position.unit).contains(position) {
        return Err(CommandError::UnknownPosition);
    }
    Ok(())
}

/// Checks that a position belongs to the acting company and is free.
pub(crate) fn check_position(
    catalog: &Catalog,
    state: &GameState,
    actor: CompanyId,
    position: &Position,
) -> Result<(), CommandError> {
    check_own_position(catalog, state, actor, position)?;
    if holder(state, actor, position).is_some() {
        return Err(CommandError::PositionTaken);
    }
    Ok(())
}

/// A manager of the acting company.
pub(crate) fn own_manager(
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
    let salary = salary_demand(catalog, state, actor, m, position);
    let since = state.date;
    state.managers.get_mut(&manager).expect("checked").job = Some(Job {
        company: actor,
        position: position.clone(),
        salary,
        since,
        satisfaction: Some(catalog.management.market.satisfaction.start),
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
    let demand = salary_demand(catalog, state, actor, m, position);
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
    book_personnel(state, actor, old.position.unit, pay);
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
    book_personnel(state, actor, job.position.unit, severance);
    Ok(())
}

/// The jobs at a site that changed hands end (M30, M38); the managers return to the
/// market.
pub(crate) fn release_site(state: &mut GameState, site: SiteId) {
    let ended: Vec<ManagerId> = state
        .managers
        .iter()
        .filter(|(_, m)| {
            m.job
                .as_ref()
                .is_some_and(|j| j.position.unit == Unit::Site(site))
        })
        .map(|(&id, _)| id)
        .collect();
    let today = state.date;
    for id in ended {
        end_job(state, id, today);
    }
    // Strategies for the site were its former owner's (MA4).
    for c in &mut state.companies {
        c.strategies
            .retain(|s| s.scope != strategy::StrategyScope::Site(site));
    }
}

/// Jobs that no longer exist end: the company went bankrupt, no longer has the unit
/// (another owner of the site, no site left in the country or continent) or its type no
/// longer has the position.
fn end_void_jobs(state: &mut GameState, catalog: &Catalog, today: Date) {
    let void: Vec<ManagerId> = state
        .managers
        .iter()
        .filter(|(_, m)| {
            m.job.as_ref().is_some_and(|j| {
                state.companies[j.company.index()].bankrupt
                    || !positions(catalog, state, j.company, j.position.unit).contains(&j.position)
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
    let pay: Vec<(CompanyId, Unit, Money)> = state
        .managers
        .values()
        .filter_map(|m| {
            let j = m.job.as_ref()?;
            let from = j.since.max(last.first_of_month());
            Some((j.company, j.position.unit, salary_for(j, from, next)))
        })
        .collect();
    for (company, unit, amount) in pay {
        if !state.companies[company.index()].bankrupt {
            book_personnel(state, company, unit, amount);
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
pub(crate) fn month_number(date: Date) -> u32 {
    u32::try_from(date.year()).unwrap_or(0) * 12 + date.month()
}

/// Day number of a date for the checks and the managers' random streams.
fn day_number(date: Date) -> u32 {
    let epoch = Date::first_of_year(crate::EARLIEST_START_YEAR);
    u32::try_from(epoch.days_until(date)).unwrap_or(0)
}

/// Whether the positions of a unit check their topics on a day (MA1, MA3): every
/// `pruefung_tage` of its level, staggered by the unit's number.
fn checks(catalog: &Catalog, unit: Unit, day: u32) -> bool {
    catalog
        .management
        .levels
        .get(level_of(unit))
        .is_some_and(|l| (day + unit_number(unit)).is_multiple_of(l.check_days.max(1)))
}

/// The next day (from `date` on) the positions of a unit check their topics.
pub fn next_check(catalog: &Catalog, unit: Unit, date: Date) -> Option<Date> {
    let days = catalog
        .management
        .levels
        .get(level_of(unit))?
        .check_days
        .max(1);
    let wait = (days - (day_number(date) + unit_number(unit)) % days) % days;
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
    crate::staffing::set_potentials(state, catalog);
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
        potential: None,
        courted: None,
        judged: 0,
        hits: 0,
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

pub(crate) fn position_state_mut<'a>(
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
                hires: false,
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

/// Counts an amount against a position's budget of the year.
pub(crate) fn count_spent(
    state: &mut GameState,
    company: CompanyId,
    position: &Position,
    amount: Money,
) {
    position_state_mut(state, company, position).spent += amount;
}

/// A unit and the units above it, up to the board (MA3, MA5).
pub(crate) fn units_up(catalog: &Catalog, state: &GameState, unit: Unit) -> Vec<Unit> {
    let mut out = vec![unit];
    if let Unit::Site(_) = unit
        && let Some(c) = country_of(state, unit)
    {
        out.push(Unit::Country(c));
    }
    if let Some(c) = country_of(state, unit) {
        out.push(Unit::Continent(catalog.countries.get(c).continent));
    }
    if unit != Unit::Board {
        out.push(Unit::Board);
    }
    out
}

/// Whether a site belongs to a unit.
pub(crate) fn in_unit(catalog: &Catalog, state: &GameState, site: SiteId, unit: Unit) -> bool {
    let Some(s) = state.sites.get(site.index()) else {
        return false;
    };
    match unit {
        Unit::Site(x) => x == site,
        Unit::Country(c) => s.country == c,
        Unit::Continent(k) => catalog.countries.get(s.country).continent == k,
        Unit::Board => true,
    }
}

/// What a position's budget refers to (docs/FORMELN.md, MA2, MA3): the revenue of the
/// unit's sites in the last twelve closed months, without revenue their costs.
pub fn budget_base(catalog: &Catalog, state: &GameState, company: CompanyId, unit: Unit) -> Money {
    let months = &state.companies[company.index()].ledger.months;
    let recent = &months[months.len().saturating_sub(12)..];
    let revenue: Money = recent
        .iter()
        .flat_map(|m| &m.site_revenue)
        .filter(|(s, _)| in_unit(catalog, state, **s, unit))
        .map(|(_, v)| *v)
        .sum();
    if revenue > Money::ZERO {
        return revenue;
    }
    let result: Money = recent
        .iter()
        .flat_map(|m| &m.by_site)
        .filter(|(s, _)| in_unit(catalog, state, **s, unit))
        .map(|(_, v)| *v)
        .sum();
    (-result).max(Money::ZERO)
}

/// The type of a position (MA3).
pub fn kind_of(state: &GameState, position: &Position) -> Option<PositionKind> {
    let level = match position.unit {
        Unit::Site(s) => UnitLevel::Site(state.sites.get(s.index())?.kind),
        Unit::Country(_) => UnitLevel::Country,
        Unit::Continent(_) => UnitLevel::Continent,
        Unit::Board => UnitLevel::Board,
    };
    Some(PositionKind {
        level,
        role: position.role.clone(),
    })
}

/// The scopes a position falls into, the narrowest first: its country, its continent,
/// the company.
fn scopes(catalog: &Catalog, state: &GameState, unit: Unit) -> Vec<RuleScope> {
    let mut out = Vec::new();
    if let Some(c) = country_of(state, unit) {
        out.push(RuleScope::Country(c));
        out.push(RuleScope::Continent(catalog.countries.get(c).continent));
    }
    if let Unit::Continent(k) = unit {
        out.push(RuleScope::Continent(k));
    }
    out.push(RuleScope::Company);
    out
}

/// The budget rule that holds for a position, if any (docs/FORMELN.md, MA3).
pub fn rule_for<'a>(
    catalog: &Catalog,
    state: &'a GameState,
    company: CompanyId,
    position: &Position,
) -> Option<&'a BudgetRule> {
    let kind = kind_of(state, position)?;
    let rules = &state.companies[company.index()].budget_rules;
    scopes(catalog, state, position.unit)
        .into_iter()
        .find_map(|scope| rules.iter().find(|r| r.kind == kind && r.scope == scope))
}

/// The shares of the reference a position may spend per decision and per year: the
/// player's for the position, else the rule for its type, else the defaults of its level
/// and role.
pub fn budget_shares(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    position: &Position,
) -> (f64, f64) {
    position_state(state, company, position)
        .and_then(|p| p.budget)
        .or_else(|| rule_for(catalog, state, company, position).map(|r| r.shares))
        .unwrap_or_else(|| default_shares(catalog, position))
}

/// The budget shares of a position's level and role from the data.
pub fn default_shares(catalog: &Catalog, position: &Position) -> (f64, f64) {
    catalog
        .management
        .levels
        .get(level_of(position.unit))
        .map_or((0.0, 0.0), |l| match position.role {
            Role::Head => l.budget_head,
            Role::Specialist(_) => l.budget_specialist,
        })
}

/// The next filled head above a position (MA3, MA5): for a specialist the head of its
/// unit, for a head that of the next higher unit up to the CEO; units without a filled
/// head are skipped.
pub fn superior(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    position: &Position,
) -> Option<(Position, ManagerId)> {
    let mut units: Vec<Unit> = Vec::new();
    if matches!(position.role, Role::Specialist(_)) {
        units.push(position.unit);
    }
    match position.unit {
        Unit::Site(_) | Unit::Country(_) => {
            let c = country_of(state, position.unit)?;
            if !matches!(position.unit, Unit::Country(_)) {
                units.push(Unit::Country(c));
            }
            units.push(Unit::Continent(catalog.countries.get(c).continent));
            units.push(Unit::Board);
        }
        Unit::Continent(_) => units.push(Unit::Board),
        Unit::Board => {}
    }
    units.into_iter().find_map(|unit| {
        let head = Position {
            unit,
            role: Role::Head,
        };
        if !has_unit(catalog, state, company, unit) {
            return None;
        }
        holder(state, company, &head).map(|id| (head, id))
    })
}

/// A position's budget per decision and per year (docs/FORMELN.md, MA2, MA3): a share of
/// the unit's reference, at least the floor in salaries (a share of 0 means "always ask",
/// without the floor), at most the budget of the next filled head above.
pub fn budget(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    position: &Position,
    salary: Money,
) -> (Money, Money) {
    let (a, b) = budget_shares(catalog, state, company, position);
    let base = budget_base(catalog, state, company, position.unit);
    let (floor_decision, floor_year) = catalog.management.budget_floor;
    let limit = |share: f64, floor: f64| {
        if share <= 0.0 {
            Money::ZERO
        } else {
            base.scale(share).max(salary.scale(floor))
        }
    };
    let own = (limit(a, floor_decision), limit(b, floor_year));
    let Some((head, id)) = superior(catalog, state, company, position) else {
        return own;
    };
    let head_salary = state.managers[&id]
        .job
        .as_ref()
        .map_or(Money::ZERO, |j| j.salary);
    let cap = budget(catalog, state, company, &head, head_salary);
    (own.0.min(cap.0), own.1.min(cap.1))
}

/// What a position spent of its budget this year.
pub fn spent(state: &GameState, company: CompanyId, position: &Position) -> Money {
    position_state(state, company, position)
        .filter(|p| p.year == state.date.year())
        .map_or(Money::ZERO, |p| p.spent)
}

/// Mean monthly result of a site in the last closed months.
pub fn mean_result(state: &GameState, company: CompanyId, site: SiteId) -> Money {
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

/// Whether a position may decide about loans (MA5): the CEO and the board's specialist
/// of the function that handles the cash.
fn lends(catalog: &Catalog, position: &Position) -> bool {
    if position.unit != Unit::Board {
        return false;
    }
    match &position.role {
        Role::Head => true,
        Role::Specialist(f) => function_of(catalog, Topic::Cash)
            .is_some_and(|i| catalog.management.functions[i].key == *f),
    }
}

/// Whether an option bids for a takeover or a licence (ZA2).
fn bids(choice: &decision::Choice) -> bool {
    choice
        .steps
        .iter()
        .any(|s| matches!(s.command, Command::MakeOffer { .. }))
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
pub(crate) fn forecast(effect: Money, error: f64) -> (Money, Money) {
    let a = effect.scale(1.0 - error);
    let b = effect.scale(1.0 + error);
    (a.min(b), a.max(b))
}

/// An amount a position counted against its budget, with its log entry.
type Spent = (Position, Money, Option<PositionLog>);

/// A filled position taking part in the decisions of a day, with what it may still spend.
struct Responsible {
    position: Position,
    judgment: f64,
    per_decision: Money,
    /// The release limit of its department, where it is below the budget per decision
    /// (ZA2).
    limit: Option<Money>,
    /// How much a working department narrows its errors (ZA2).
    accuracy: f64,
    left: Money,
    muted: BTreeSet<Topic>,
    blocked: BTreeSet<Topic>,
    open: u32,
}

/// The investments of an option (docs/FORMELN.md, MA4): building, developing, founding,
/// buying land and restarting.
fn investment(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    choice: &decision::Choice,
) -> Money {
    choice
        .steps
        .iter()
        .filter(|s| {
            matches!(
                s.command,
                Command::BuildFacility { .. }
                    | Command::DevelopDeposit { .. }
                    | Command::FoundSite { .. }
                    | Command::FoundSiteOnPlot { .. }
                    | Command::BuyPlot { .. }
                    | Command::RestartFacility { .. }
            )
        })
        .map(|s| decision::amount(catalog, state, company, &s.command))
        .sum()
}

/// Where a decision is taken (MA3, MA5): its site, else where the first step of the
/// rules' option acts (a new site's country, an advertising country); loans alone and
/// offers at the board.
pub fn place_of(state: &GameState, d: &Decision) -> Option<Unit> {
    if let Some(site) = d.site {
        return Some(Unit::Site(site));
    }
    let steps = &d.choices.get(d.rule)?.steps;
    let Some(first) = steps
        .iter()
        .map(|s| &s.command)
        .find(|c| !matches!(c, Command::TakeLoan { .. } | Command::RepayLoan { .. }))
    else {
        return (!steps.is_empty()).then_some(Unit::Board);
    };
    match *first {
        Command::MakeOffer { .. }
        | Command::AnswerOffer { .. }
        | Command::RefinanceLoan { .. }
        | Command::RaiseSalary { .. } => Some(Unit::Board),
        Command::FoundSite { country, .. } | Command::SetAdvertising { country, .. } => {
            Some(Unit::Country(country))
        }
        Command::FoundSiteOnPlot { plot, .. } => state
            .plots
            .get(plot.index())
            .map(|p| Unit::Country(p.country)),
        Command::BuildFacility { site, .. } | Command::DevelopDeposit { site, .. } => {
            state.sites.get(site.index()).map(|_| Unit::Site(site))
        }
        _ => None,
    }
}

/// Topics whose decisions a company's positions take at most once per country and
/// product a day: building in one place must not be repeated in another.
fn structural(topic: Topic) -> bool {
    matches!(topic, Topic::Expansion | Topic::Power | Topic::Deposit)
}

/// The decider of a company's positions on a day (docs/FORMELN.md, MA1–MA3): a decision
/// starts at the first filled position of its chain that noticed its topic; within its
/// budget a position carries out its recommendation, beyond it the decision goes up the
/// chain, and from the top as a concern to the player. Everything else stays as it is.
struct Staff<'a> {
    company: CompanyId,
    /// Units whose positions noticed a topic today.
    noticed: BTreeSet<(Unit, Topic)>,
    positions: Vec<Responsible>,
    model: &'a ConcernModel,
    routine: &'a [Topic],
    head_discount: f64,
    rngs: BTreeMap<ManagerId, SimRng>,
    seed: u64,
    day: u32,
    today: Date,
    /// Open concerns: topic, place, product.
    open: BTreeSet<(Topic, Unit, Option<ProductId>)>,
    /// Structural decisions taken today: topic, country, product.
    done: BTreeSet<(Topic, Option<CountryId>, Option<ProductId>)>,
    spent: Vec<Spent>,
    concerns: Vec<Concern>,
    followups: Vec<Followup>,
    /// Investments decided today and their places (MA4).
    invested: Vec<(Unit, Money)>,
}

impl Staff<'_> {
    /// Why an investment at a place does not fit the company's strategies (MA4): the
    /// liquidity reserve, then the investment budgets with what was decided today;
    /// `None` if it fits.
    fn beyond_strategy(
        &self,
        catalog: &Catalog,
        state: &GameState,
        place: Unit,
        invest: Money,
    ) -> Option<ConcernReason> {
        if invest <= Money::ZERO {
            return None;
        }
        let company = self.company;
        if state.companies[company.index()].strategies.is_empty() {
            return None;
        }
        let months = strategy::reserve_months(catalog, state, company, place);
        if months > 0.0 {
            let reserve = strategy::monthly_cost(catalog, state, company).scale(months);
            if state.companies[company.index()].ledger.cash() - invest < reserve {
                return Some(ConcernReason::Reserve);
            }
        }
        for b in strategy::investment_budgets(catalog, state, company, place, self.today.year()) {
            let today: Money = self
                .invested
                .iter()
                .filter(|(u, _)| b.scope.contains(catalog, state, *u))
                .map(|&(_, m)| m)
                .sum();
            if invest > b.left - today {
                return Some(ConcernReason::Investment);
            }
        }
        None
    }

    fn rng(&mut self, manager: ManagerId) -> &mut SimRng {
        let (seed, day) = (self.seed, self.day);
        self.rngs
            .entry(manager)
            .or_insert_with(|| SimRng::for_stream(seed, Stream::Manager { id: manager.0, day }))
    }

    /// A filled position by its index among those of the day, registered with its budget
    /// on its first decision.
    fn responsible(
        &mut self,
        state: &GameState,
        catalog: &Catalog,
        position: &Position,
        manager: ManagerId,
    ) -> usize {
        if let Some(i) = self.positions.iter().position(|r| r.position == *position) {
            return i;
        }
        let company = self.company;
        let x = &state.managers[&manager];
        let salary = x.job.as_ref().map_or(Money::ZERO, |j| j.salary);
        let (per_decision, per_year) = budget(catalog, state, company, position, salary);
        let limit = crate::central::release_limit(catalog, state, company, position)
            .filter(|&l| l < per_decision);
        let accuracy = crate::central::accuracy(catalog, state, company, position);
        let judgment = f64::from(x.judgment);
        let ps = position_state(state, company, position);
        let today = state.date;
        self.positions.push(Responsible {
            position: position.clone(),
            judgment: if accuracy > 0.0 {
                judgment + (100.0 - judgment) * accuracy
            } else {
                judgment
            },
            per_decision: limit.unwrap_or(per_decision),
            limit,
            accuracy,
            left: (per_year - spent(state, company, position)).max(Money::ZERO),
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
                            && c.position == *position
                            && c.status == ConcernStatus::Open
                    })
                    .count(),
            )
            .unwrap_or(u32::MAX),
        });
        self.positions.len() - 1
    }

    /// The option a position recommends (docs/FORMELN.md, MA2).
    fn recommend(
        &mut self,
        decision: &Decision,
        assessments: &[Assessment],
        (manager, expertise, judgment): (ManagerId, f64, f64),
    ) -> usize {
        let model = self.model;
        recommend_with(
            model,
            self.rng(manager),
            (decision, assessments),
            (expertise, judgment),
        )
    }
}

/// The option a manager recommends (docs/FORMELN.md, MA2): the best by the true effects
/// with a chance after its judgment, else the best by its own estimate.
pub(crate) fn recommend_with(
    model: &ConcernModel,
    rng: &mut SimRng,
    (decision, assessments): (&Decision, &[Assessment]),
    (expertise, judgment): (f64, f64),
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
    let base = model.recommend_base;
    let error = model.estimate_error * (1.0 - expertise / 100.0);
    if rng.chance(base + (1.0 - base) * judgment / 100.0) {
        return best(&values);
    }
    let estimated: Vec<f64> = values
        .iter()
        .map(|v| v * (1.0 + (rng.next_f64() * 2.0 - 1.0) * error))
        .collect();
    best(&estimated)
}

/// The CEO of a company (MA5).
pub fn ceo_of(state: &GameState, company: CompanyId) -> Option<ManagerId> {
    state
        .managers
        .iter()
        .find(|(_, m)| {
            m.job.as_ref().is_some_and(|j| {
                j.company == company
                    && j.position.unit == Unit::Board
                    && j.position.role == Role::Head
            })
        })
        .map(|(&id, _)| id)
}

impl Decider for Staff<'_> {
    fn looks(&self) -> bool {
        true
    }

    /// The estimate of the board's member of the topic's function, else of the CEO with
    /// less expertise (docs/FORMELN.md, ZA3): f = 1 + (2u − 1) · e with
    /// e = error · (1 − expertise/100) · (1 − accuracy · A).
    fn estimate(
        &mut self,
        state: &GameState,
        catalog: &Catalog,
        topic: Topic,
    ) -> Option<(ManagerId, f64)> {
        let function = &catalog.management.functions[function_of(catalog, topic)?].key;
        let member = Position::new(Unit::Board, Role::Specialist(function.clone()));
        let ceo = Position::new(Unit::Board, Role::Head);
        let (position, manager, discount) = match holder(state, self.company, &member) {
            Some(id) => (member, id, 0.0),
            None => {
                let id = holder(state, self.company, &ceo)?;
                (ceo, id, self.head_discount)
            }
        };
        let skill = f64::from(expertise(&state.managers[&manager], function)) * (1.0 - discount);
        let accuracy = crate::central::accuracy(catalog, state, self.company, &position);
        let error = self.model.estimate_error * (1.0 - skill / 100.0) * (1.0 - accuracy);
        let u = self.rng(manager).next_f64();
        Some((manager, 1.0 + (2.0 * u - 1.0) * error))
    }

    fn decide(&mut self, state: &GameState, catalog: &Catalog, d: &Decision) -> Verdict {
        let Some(place) = place_of(state, d) else {
            return Verdict::Hold;
        };
        if crate::mandate::banned(catalog, state, d, d.chosen()) {
            return Verdict::Hold;
        }
        let filled: Vec<(Position, ManagerId)> =
            chain(catalog, state, self.company, place, d.topic)
                .into_iter()
                .filter_map(|p| holder(state, self.company, &p).map(|id| (p, id)))
                .collect();
        let Some(start) = filled
            .iter()
            .position(|(p, _)| self.noticed.contains(&(p.unit, d.topic)))
        else {
            return Verdict::Hold;
        };
        let done = (d.topic, country_of(state, place), d.product);
        if structural(d.topic) && self.done.contains(&done) {
            return Verdict::Hold;
        }
        let routine = self.routine.contains(&d.topic);
        // The routine and topics whose effect counts only costs follow the rules; other
        // topics are assessed and recommended.
        let by_rule = routine || catalog.management.rule_topics.contains(&d.topic);
        let assessments = (!by_rule).then(|| decision::assess(catalog, state, d));
        let function = function_of(catalog, d.topic).map(|f| &catalog.management.functions[f].key);
        // Takeovers and licences count against the yearly budget of the participations
        // policy (ZA2); the open bids are in it.
        let participations = crate::central::participations_left(state, d.company);
        let mut hops: Vec<Hop> = Vec::new();
        let mut asker = (0, 0.0);
        for (position, manager) in &filled[start..] {
            let index = self.responsible(state, catalog, position, *manager);
            let x = &state.managers[manager];
            let discount = match position.role {
                Role::Head => self.head_discount,
                Role::Specialist(_) => 0.0,
            };
            let skill = function.map_or(0.0, |f| f64::from(expertise(x, f))) * (1.0 - discount);
            // A working department narrows the estimate error of its head (ZA2).
            let accuracy = self.positions[index].accuracy;
            let skill = if accuracy > 0.0 {
                100.0 - (100.0 - skill) * (1.0 - accuracy)
            } else {
                skill
            };
            let choice = match &assessments {
                None => d.rule,
                Some(a) => {
                    let judgment = self.positions[index].judgment;
                    self.recommend(d, a, (*manager, skill, judgment))
                }
            };
            let Some(option) = d.choices.get(choice) else {
                return Verdict::Hold;
            };
            let amount = assessments.as_ref().map_or_else(
                || choice_amount(catalog, state, d.company, option),
                |a| a[choice].amount,
            );
            let invest = investment(catalog, state, d.company, option);
            let beyond = self.beyond_strategy(catalog, state, place, invest);
            let borrows = needs_finance(option);
            let may_borrow = !borrows || lends(catalog, position);
            let debt = borrows && crate::mandate::over_debt(state, d.company, option);
            let over_participations =
                participations.is_some_and(|left| bids(option) && amount > left);
            let r = &mut self.positions[index];
            if may_borrow
                && !debt
                && beyond.is_none()
                && !over_participations
                && amount <= r.per_decision
                && amount <= r.left
            {
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
                if let (Some(site), Some(effect)) = (d.site, effect.filter(|_| acted)) {
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
                if structural(d.topic) && kind != ChoiceKind::Keep {
                    self.done.insert(done);
                }
                if invest > Money::ZERO {
                    self.invested.push((place, invest));
                }
                return if choice == d.rule {
                    Verdict::Rule
                } else {
                    Verdict::Choice(choice)
                };
            }
            hops.push(Hop {
                position: position.clone(),
                manager: *manager,
                recommended: choice,
            });
            asker = (index, skill);
        }
        // Nobody could decide: a concern of the position at the top to the player.
        let key = (d.topic, place, d.product);
        let index_of = |staff: &Self, p: &Position| {
            staff
                .positions
                .iter()
                .position(|r| r.position == *p)
                .expect("registered")
        };
        let quiet = hops.iter().any(|h| {
            let r = &self.positions[index_of(self, &h.position)];
            r.muted.contains(&d.topic) || r.blocked.contains(&d.topic)
        });
        let first = index_of(self, &hops[0].position);
        if quiet
            || self.positions[first].open >= self.model.open_per_position
            || self.open.contains(&key)
        {
            return Verdict::Hold;
        }
        self.positions[first].open += 1;
        self.open.insert(key);
        let top = hops.last().expect("one at least").clone();
        let option = &d.choices[top.recommended];
        let assessments = assessments.unwrap_or_else(|| decision::assess(catalog, state, d));
        let amount = assessments[top.recommended].amount;
        let (index, skill) = asker;
        let beyond = self.beyond_strategy(
            catalog,
            state,
            place,
            investment(catalog, state, d.company, option),
        );
        let r = &self.positions[index];
        let borrows = needs_finance(option);
        let reason = if borrows && !lends(catalog, &top.position) {
            ConcernReason::Finance
        } else if borrows && crate::mandate::over_debt(state, d.company, option) {
            ConcernReason::Debt
        } else if let Some(reason) = beyond {
            reason
        } else if participations.is_some_and(|left| bids(option) && amount > left) {
            ConcernReason::Participations
        } else if r.per_decision == Money::ZERO {
            ConcernReason::Always
        } else if r.limit.is_some_and(|l| amount > l) {
            ConcernReason::Limit
        } else if amount > r.per_decision {
            ConcernReason::Decision
        } else {
            ConcernReason::Year
        };
        let error = self.model.estimate_error * (1.0 - skill / 100.0);
        let options = assessments
            .iter()
            .map(|a| ConcernOption {
                amount: a.amount,
                forecast: a.effect.map(|e| forecast(e, error)),
                once: a.once,
            })
            .collect();
        let path = if hops.len() > 1 {
            hops.clone()
        } else {
            Vec::new()
        };
        self.concerns.push(Concern {
            id: 0,
            company: d.company,
            position: hops[0].position.clone(),
            manager: top.manager,
            decision: d.clone(),
            recommended: top.recommended,
            options,
            reason,
            path,
            parts: Vec::new(),
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

/// The estimates of a company's head of a topic on a number of days (tests, ZA3).
#[cfg(test)]
pub(crate) fn estimates(
    state: &GameState,
    catalog: &Catalog,
    (company, topic): (CompanyId, Topic),
    days: std::ops::Range<u32>,
) -> Vec<(ManagerId, f64)> {
    let m = &catalog.management;
    days.filter_map(|day| {
        let mut staff = Staff {
            company,
            noticed: BTreeSet::new(),
            positions: Vec::new(),
            model: &m.concerns,
            routine: &m.routine_topics,
            head_discount: m.head_discount,
            rngs: BTreeMap::new(),
            seed: state.settings.seed,
            day,
            today: state.date,
            open: BTreeSet::new(),
            done: BTreeSet::new(),
            spent: Vec::new(),
            concerns: Vec::new(),
            followups: Vec::new(),
            invested: Vec::new(),
        };
        staff.estimate(state, catalog, topic)
    })
    .collect()
}

/// Puts one decision to a company's positions as on a day of theirs on which every unit
/// noticed its topic (tests): the verdict, the concerns and the investments decided.
#[cfg(test)]
pub(crate) fn decide_now(
    state: &GameState,
    catalog: &Catalog,
    decision: &Decision,
) -> (Verdict, Vec<Concern>, Vec<(Unit, Money)>) {
    let m = &catalog.management;
    let company = decision.company;
    let mut staff = Staff {
        company,
        noticed: units(catalog, state, company)
            .into_iter()
            .map(|u| (u, decision.topic))
            .collect(),
        positions: Vec::new(),
        model: &m.concerns,
        routine: &m.routine_topics,
        head_discount: m.head_discount,
        rngs: BTreeMap::new(),
        seed: state.settings.seed,
        day: day_number(state.date),
        today: state.date,
        open: open_keys(state, company),
        done: BTreeSet::new(),
        spent: Vec::new(),
        concerns: Vec::new(),
        followups: Vec::new(),
        invested: Vec::new(),
    };
    let verdict = staff.decide(state, catalog, decision);
    (verdict, staff.concerns, staff.invested)
}

/// The position that asks the player about a concern: the last of its way.
pub fn asker(c: &Concern) -> &Position {
    c.path.last().map_or(&c.position, |h| &h.position)
}

/// The options of a strategic concern: all parts' recommendations together, or nothing.
fn part_options(parts: &[ConcernPart]) -> Vec<ConcernOption> {
    let amount = parts.iter().map(|p| p.option.amount).sum();
    let once = parts.iter().map(|p| p.option.once).sum();
    let forecast = parts
        .iter()
        .map(|p| p.option.forecast)
        .try_fold((Money::ZERO, Money::ZERO), |(a, b), f| {
            f.map(|(x, y)| (a + x, b + y))
        });
    vec![
        ConcernOption {
            amount,
            forecast,
            once,
        },
        ConcernOption {
            amount: Money::ZERO,
            forecast: None,
            once: Money::ZERO,
        },
    ]
}

/// Alike concerns of several sites to the same position on a day become one strategic
/// concern of that position (docs/FORMELN.md, MA3).
pub(crate) fn bundle(concerns: Vec<Concern>, from: u32) -> Vec<Concern> {
    type Group = (Position, Topic, Option<ChoiceKind>, Vec<Concern>);
    let mut groups: Vec<Group> = Vec::new();
    for c in concerns {
        let who = asker(&c).clone();
        let kind = c.decision.choices.get(c.recommended).map(|o| o.kind);
        let topic = c.decision.topic;
        match groups
            .iter_mut()
            .find(|g| g.0 == who && g.1 == topic && g.2 == kind)
        {
            Some(g) => g.3.push(c),
            None => groups.push((who, topic, kind, vec![c])),
        }
    }
    let from = usize::try_from(from).unwrap_or(usize::MAX);
    let mut out = Vec::new();
    for (who, topic, kind, list) in groups {
        let sites: BTreeSet<SiteId> = list.iter().filter_map(|c| c.decision.site).collect();
        let (Some(kind), true) = (kind, sites.len() >= from && list.len() >= from) else {
            out.extend(list);
            continue;
        };
        let first = &list[0];
        let manager = first.path.last().map_or(first.manager, |h| h.manager);
        let product = first
            .decision
            .product
            .filter(|p| list.iter().all(|c| c.decision.product == Some(*p)));
        let reason = if list.iter().any(|c| c.reason == ConcernReason::Finance) {
            ConcernReason::Finance
        } else {
            first.reason
        };
        let parts: Vec<ConcernPart> = list
            .iter()
            .map(|c| ConcernPart {
                position: c.position.clone(),
                decision: c.decision.clone(),
                recommended: c.recommended,
                option: c.options[c.recommended].clone(),
            })
            .collect();
        out.push(Concern {
            id: 0,
            company: first.company,
            position: who,
            manager,
            decision: Decision {
                topic,
                company: first.company,
                site: None,
                product,
                choices: vec![
                    decision::Choice {
                        kind,
                        steps: Vec::new(),
                    },
                    decision::Choice::keep(),
                ],
                rule: 0,
            },
            recommended: 0,
            options: part_options(&parts),
            reason,
            path: Vec::new(),
            parts,
            created: first.created,
            deadline: first.deadline,
            status: ConcernStatus::Open,
            closed: None,
        });
    }
    out
}

/// Positions held, per company and unit.
type Held = BTreeMap<CompanyId, BTreeMap<Unit, Vec<(Role, ManagerId)>>>;

/// The first filled position of the chain for a topic at a place that takes the topic up
/// itself (MA3).
pub fn first_taker(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    place: Unit,
    topic: Topic,
) -> Option<Position> {
    chain(catalog, state, company, place, topic)
        .into_iter()
        .find(|p| arises(catalog, state, p.unit, topic) && holder(state, company, p).is_some())
}

/// The company's sites whose structure the positions of a country or continent take
/// care of: those where no position below takes up the topic (MA3).
fn sites_cared_for(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    (own, unit): (&[SiteId], Unit),
) -> Vec<SiteId> {
    own.iter()
        .copied()
        .filter(|&s| {
            in_unit(catalog, state, s, unit)
                && state.sites[s.index()].kind != SiteType::ResearchCenter
                && first_taker(catalog, state, company, Unit::Site(s), Topic::Expansion)
                    .is_some_and(|p| p.unit == unit)
        })
        .collect()
}

/// The countries whose advertising the positions of a unit take care of (MA3).
fn countries_cared_for(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    (own, unit): (&[SiteId], Unit),
) -> Vec<CountryId> {
    let countries: BTreeSet<CountryId> = own
        .iter()
        .filter(|&&s| in_unit(catalog, state, s, unit))
        .map(|&s| state.sites[s.index()].country)
        .collect();
    countries
        .into_iter()
        .filter(|&c| {
            first_taker(
                catalog,
                state,
                company,
                Unit::Country(c),
                Topic::Advertising,
            )
            .is_some_and(|p| p.unit == unit)
        })
        .collect()
}

/// The work of the positions due today (docs/FORMELN.md, MA1–MA3), before the AI
/// companies decide: concerns expire, effects are reported; the sites' routine, at the
/// end of a quarter the structure of the sites and of the countries and continents, in
/// idle laboratories the next target, at their checks the advertising of countries and
/// continents. Returns the messages for the round report.
pub fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    expire_concerns(state, catalog, date, &mut news);
    settle_closed_offers(state, date);
    report_followups(state, catalog, date, &mut news);
    let m = &catalog.management;
    if !m.enabled() {
        return news;
    }
    let mut held: Held = BTreeMap::new();
    for (&id, x) in &state.managers {
        if let Some(j) = &x.job {
            held.entry(j.company)
                .or_default()
                .entry(j.position.unit)
                .or_default()
                .push((j.position.role.clone(), id));
        }
    }
    let day = day_number(date);
    let quarter_end = date.next_day().day() == 1 && date.month().is_multiple_of(3);
    let first_of_year = date.ordinal() == 1;
    for (company, units) in held {
        // AI companies' managers act through its competence (MA6).
        let c = &state.companies[company.index()];
        if c.bankrupt || c.ai.is_some() {
            continue;
        }
        let mut staff = Staff {
            company,
            noticed: BTreeSet::new(),
            positions: Vec::new(),
            model: &m.concerns,
            routine: &m.routine_topics,
            head_discount: m.head_discount,
            rngs: BTreeMap::new(),
            seed: state.settings.seed,
            day,
            today: date,
            open: open_keys(state, company),
            done: BTreeSet::new(),
            spent: Vec::new(),
            concerns: Vec::new(),
            followups: Vec::new(),
            invested: Vec::new(),
        };
        let (mut weekly, mut structural, mut labs) = (Vec::new(), Vec::new(), Vec::new());
        let (mut unit_structure, mut unit_checks) = (Vec::new(), Vec::new());
        let mut board_check = false;
        for (&unit, roles) in &units {
            let check = checks(catalog, unit, day);
            match unit {
                Unit::Site(site) => {
                    let s = &state.sites[site.index()];
                    let lab = s.kind == SiteType::ResearchCenter;
                    let idle = s.research.is_none() && s.development.is_none();
                    let lab_due = lab && ((check && idle) || first_of_year);
                    if !(lab_due || (!lab && (check || quarter_end))) {
                        continue;
                    }
                    if !notice(state, catalog, unit, roles, &mut staff) {
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
                _ => {
                    if !(check || quarter_end) || !notice(state, catalog, unit, roles, &mut staff) {
                        continue;
                    }
                    if quarter_end {
                        unit_structure.push(unit);
                    }
                    if check {
                        unit_checks.push(unit);
                        board_check |= unit == Unit::Board;
                    }
                }
            }
        }
        if weekly.is_empty()
            && structural.is_empty()
            && labs.is_empty()
            && unit_structure.is_empty()
            && unit_checks.is_empty()
        {
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
        for unit in unit_structure {
            let sites = sites_cared_for(catalog, state, company, (&own, unit));
            if !sites.is_empty() {
                crate::ai::unit_structure(
                    state,
                    catalog,
                    company,
                    (&own, &sites),
                    date,
                    &mut staff,
                );
            }
        }
        for unit in unit_checks {
            let countries = countries_cared_for(catalog, state, company, (&own, unit));
            if !countries.is_empty() {
                let sites: Vec<SiteId> = own
                    .iter()
                    .copied()
                    .filter(|&s| countries.contains(&state.sites[s.index()].country))
                    .collect();
                crate::ai::unit_advertising(
                    state,
                    catalog,
                    company,
                    (&sites, &countries),
                    &mut staff,
                );
            }
        }
        if board_check {
            let done = board_work(state, catalog, company, &own, &mut staff);
            if company == state.player {
                news.extend(done);
            }
        }
        let Staff {
            spent,
            concerns,
            followups,
            invested,
            ..
        } = staff;
        let concerns = bundle(concerns, m.concerns.bundle_from);
        record(
            state,
            catalog,
            company,
            (spent, concerns, followups),
            &mut news,
        );
        for (place, amount) in invested {
            strategy::count_investment(catalog, state, company, place, (amount, date.year()));
        }
    }
    news
}

/// The board's own topics at its check (docs/FORMELN.md, MA5): the cash, the answers to
/// offers, new offers. Returns the news of what it decided.
fn board_work(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    own: &[SiteId],
    staff: &mut Staff,
) -> Vec<Message> {
    let mut news = Vec::new();
    let noticed = |staff: &Staff, topic: Topic| staff.noticed.contains(&(Unit::Board, topic));
    if noticed(staff, Topic::Cash) {
        crate::ai::manage_cash(state, catalog, company, own, staff);
    }
    let answers = (
        noticed(staff, Topic::OfferAnswer),
        noticed(staff, Topic::License),
    );
    if answers.0 || answers.1 {
        news.extend(crate::deals::board_answers(
            state, catalog, company, staff, answers,
        ));
    }
    // The recommendations of the finance and personnel departments (ZA3).
    if noticed(staff, Topic::Refinance) {
        let cases = crate::central::cases(catalog, state, company, DepartmentKind::Finance);
        for loan in crate::central::refinance_candidates(catalog, state, company)
            .into_iter()
            .take(cases)
        {
            let command = Command::RefinanceLoan { loan };
            recommend(
                state,
                catalog,
                (company, Topic::Refinance),
                (ChoiceKind::Refinance, command),
                staff,
            );
        }
    }
    if noticed(staff, Topic::SalaryRound) {
        let cases = crate::central::cases(catalog, state, company, DepartmentKind::Personnel);
        for (manager, salary) in crate::central::salary_round_candidates(catalog, state, company)
            .into_iter()
            .take(cases)
        {
            let command = Command::RaiseSalary { manager, salary };
            recommend(
                state,
                catalog,
                (company, Topic::SalaryRound),
                (ChoiceKind::Adjust, command),
                staff,
            );
        }
    }
    let plan = crate::deals::BoardPlan {
        offers: noticed(staff, Topic::Offer),
        licenses: noticed(staff, Topic::License),
        countries: crate::central::observed_countries(catalog, state, company),
        technologies: crate::central::legal_technologies(catalog, state, company),
        offer_cases: crate::central::cases(catalog, state, company, DepartmentKind::Strategy)
            .max(1),
        license_cases: crate::central::cases(catalog, state, company, DepartmentKind::Legal).max(1),
    };
    if plan.offers || plan.licenses {
        news.extend(crate::deals::board_offers(
            state, catalog, company, staff, &plan,
        ));
    }
    news
}

/// A department's case put to the positions (ZA3): carried out where one decides it, else
/// a recommendation to the player.
fn recommend(
    state: &mut GameState,
    catalog: &Catalog,
    (company, topic): (CompanyId, Topic),
    (kind, command): (ChoiceKind, Command),
    staff: &mut Staff,
) {
    let decided = decision::decided(staff, state, catalog, |_| {
        Decision::new(topic, company, decision::Choice::one(kind, command.clone()))
    });
    if decided {
        // Checked on the day; a failure leaves things as they are.
        let _ = crate::command::execute(state, catalog, company, &command);
    }
}

/// The offer a decision answers, if it does.
fn answered_offer(d: &Decision) -> Option<u32> {
    d.choices
        .iter()
        .flat_map(|c| &c.steps)
        .find_map(|s| match s.command {
            Command::AnswerOffer { offer, .. } => Some(offer),
            _ => None,
        })
}

/// Concerns about answering offers that closed meanwhile are settled (MA5).
fn settle_closed_offers(state: &mut GameState, today: Date) {
    let open: BTreeSet<u32> = state
        .offers
        .iter()
        .filter(|o| o.status == crate::deals::OfferStatus::Open)
        .map(|o| o.id)
        .collect();
    for c in state.concerns.iter_mut().filter(|c| {
        c.status == ConcernStatus::Open
            && matches!(c.decision.topic, Topic::OfferAnswer | Topic::License)
    }) {
        if answered_offer(&c.decision).is_some_and(|id| !open.contains(&id)) {
            c.status = ConcernStatus::Settled;
            c.closed = Some(today);
        }
    }
}

/// The places of a company's open concerns and their parts: topic, place, product.
fn open_keys(state: &GameState, company: CompanyId) -> BTreeSet<(Topic, Unit, Option<ProductId>)> {
    let mut out = BTreeSet::new();
    for c in state
        .concerns
        .iter()
        .filter(|c| c.company == company && c.status == ConcernStatus::Open)
    {
        let decisions = std::iter::once(&c.decision).chain(c.parts.iter().map(|p| &p.decision));
        for d in decisions {
            if let Some(place) = place_of(state, d) {
                out.insert((d.topic, place, d.product));
            }
        }
    }
    out
}

/// The positions of a unit that notice their topics today (docs/FORMELN.md, MA1, MA3):
/// for every function with topics the unit takes up, one draw from the stream of the
/// manager responsible for it there (its specialist, else the head with less expertise).
/// Returns whether anything was noticed.
fn notice(
    state: &GameState,
    catalog: &Catalog,
    unit: Unit,
    roles: &[(Role, ManagerId)],
    staff: &mut Staff,
) -> bool {
    let m = &catalog.management;
    let specialists = specialist_functions(catalog, state, unit);
    let head = roles.iter().find(|(r, _)| *r == Role::Head).map(|r| r.1);
    let mut any = false;
    for (index, function) in m.functions.iter().enumerate() {
        let topics: Vec<Topic> = function
            .topics
            .iter()
            .copied()
            .filter(|&t| arises(catalog, state, unit, t))
            .collect();
        if topics.is_empty() {
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
        if x.job.is_none() {
            continue;
        }
        let skill = f64::from(expertise(x, &function.key)) * (1.0 - discount);
        let diligence = (skill + f64::from(x.detection)) / 2.0;
        let chance = m.notice_base + (1.0 - m.notice_base) * diligence / 100.0;
        if !staff.rng(manager).chance(chance) {
            continue;
        }
        any = true;
        for topic in topics {
            staff.noticed.insert((unit, topic));
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
                NEW_KEYS,
                MessageKind::Warning,
                &concern,
            ));
        }
        state.concerns.push(concern);
    }
    state.followups.extend(followups);
}

/// Name of a position for messages: the head of a site type or a function.
pub fn position_param(state: &GameState, position: &Position) -> Param {
    match (&position.role, position.unit) {
        (Role::Head, Unit::Site(s)) => {
            let kind = site_type_key(state.sites[s.index()].kind);
            Param::TextKey(kind.replace("standorttyp.", "leitung."))
        }
        (Role::Head, Unit::Country(_)) => Param::TextKey("leitung.land".into()),
        (Role::Head, Unit::Continent(_)) => Param::TextKey("leitung.kontinent".into()),
        (Role::Head, Unit::Board) => Param::TextKey("leitung.vorstand".into()),
        (Role::Specialist(f), _) => Param::TextKey(format!("bereich.{f}")),
    }
}

/// A message about a concern, from the position that asks (site, country, continent or
/// board).
fn concern_message(
    catalog: &Catalog,
    state: &GameState,
    (site_key, country_key, continent_key, board_key): (&str, &str, &str, &str),
    kind: MessageKind,
    c: &Concern,
) -> Message {
    let who = asker(c);
    let m = match who.unit {
        Unit::Site(s) => {
            let site = &state.sites[s.index()];
            Message::new(kind, site_key)
                .with("standort", Param::TextKey(site_type_key(site.kind)))
                .with(
                    "land",
                    Param::Country(catalog.countries.key(site.country).to_owned()),
                )
        }
        Unit::Country(country) => Message::new(kind, country_key).with(
            "land",
            Param::Country(catalog.countries.key(country).to_owned()),
        ),
        Unit::Continent(k) => Message::new(kind, continent_key).with(
            "kontinent",
            Param::TextKey(format!("kontinent.{}", catalog.continents.key(k))),
        ),
        Unit::Board => Message::new(kind, board_key),
    };
    m.with("stelle", position_param(state, who))
        .with(
            "thema",
            Param::TextKey(format!("thema.{}", c.decision.topic.key())),
        )
        .with("frist", Param::Date(c.deadline))
}

const NEW_KEYS: (&str, &str, &str, &str) = (
    keys::CONCERN_NEW,
    keys::CONCERN_NEW_COUNTRY,
    keys::CONCERN_NEW_CONTINENT,
    keys::CONCERN_NEW_BOARD,
);
const EXPIRED_KEYS: (&str, &str, &str, &str) = (
    keys::CONCERN_EXPIRED,
    keys::CONCERN_EXPIRED_COUNTRY,
    keys::CONCERN_EXPIRED_CONTINENT,
    keys::CONCERN_EXPIRED_BOARD,
);

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
                EXPIRED_KEYS,
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

/// A followup on an option carried out at a site, with the middle of its forecast.
fn followup_for(
    state: &GameState,
    catalog: &Catalog,
    actor: CompanyId,
    (decision, kind, forecast): (&Decision, ChoiceKind, Option<(Money, Money)>),
) -> Option<Followup> {
    let site = decision.site?;
    let (low, high) = forecast?;
    if kind == ChoiceKind::Keep {
        return None;
    }
    let days = catalog.management.concerns.followup_days;
    Some(Followup {
        company: actor,
        site,
        topic: decision.topic,
        kind,
        product: decision.product,
        forecast: Money::from_units((low.units() + high.units()) / 2),
        baseline: mean_result(state, actor, site),
        due: state.date.add_days(i32::try_from(days).unwrap_or(i32::MAX)),
    })
}

/// Carries out an option of a decision; the first step decides whether anything
/// happens, later ones may fail (a required one skips the rest). The option was formed
/// earlier: a site it founds gets the next number now, and the steps that follow are
/// pointed at it.
fn carry_out(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    option: &decision::Choice,
) -> Result<(), CommandError> {
    let mut steps: Vec<decision::Step> = option.steps.clone();
    let mut skip = false;
    for i in 0..steps.len() {
        if skip {
            break;
        }
        let ran = crate::command::execute(state, catalog, actor, &steps[i].command);
        if i == 0 {
            ran.clone()?;
        }
        skip = steps[i].required && ran.is_err();
        let founds = matches!(
            steps[i].command,
            Command::FoundSite { .. } | Command::FoundSiteOnPlot { .. }
        );
        if ran.is_ok() && founds {
            let actual = SiteId(u32::try_from(state.sites.len() - 1).unwrap_or(u32::MAX));
            let planned = steps[i + 1..]
                .iter()
                .find_map(|s| decision::site_of(&s.command));
            if let Some(planned) = planned.filter(|&p| p != actual) {
                for s in &mut steps[i + 1..] {
                    decision::move_site(&mut s.command, planned, actual);
                }
            }
        }
    }
    Ok(())
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
    let decide = |state: &mut GameState, choice: usize| -> Result<(), CommandError> {
        let option = c
            .decision
            .choices
            .get(choice)
            .ok_or(CommandError::UnknownOption)?;
        if c.parts.is_empty() {
            carry_out(state, catalog, actor, option)?;
            let forecast = c.options.get(choice).and_then(|o| o.forecast);
            if let Some(f) =
                followup_for(state, catalog, actor, (&c.decision, option.kind, forecast))
            {
                state.followups.push(f);
            }
            return Ok(());
        }
        // A strategic concern: every part on its own, keeping things as they are does nothing.
        if option.kind == ChoiceKind::Keep {
            return Ok(());
        }
        for part in &c.parts {
            let Some(o) = part.decision.choices.get(part.recommended) else {
                continue;
            };
            if carry_out(state, catalog, actor, o).is_ok()
                && let Some(f) = followup_for(
                    state,
                    catalog,
                    actor,
                    (&part.decision, o.kind, part.option.forecast),
                )
            {
                state.followups.push(f);
            }
        }
        Ok(())
    };
    let who = asker(&c).clone();
    let status = match answer {
        ConcernAnswer::Choose(choice) => {
            decide(state, choice)?;
            ConcernStatus::Chosen(choice)
        }
        ConcernAnswer::Delegate => {
            decide(state, c.recommended)?;
            ConcernStatus::Delegated(c.recommended)
        }
        ConcernAnswer::NeverAsk => {
            position_state_mut(state, actor, &who)
                .muted
                .insert(c.decision.topic);
            ConcernStatus::Muted
        }
        ConcernAnswer::Decline => {
            let days = catalog.management.concerns.block_days;
            let until = today.add_days(i32::try_from(days).unwrap_or(i32::MAX));
            position_state_mut(state, actor, &who)
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
/// restores the rule for its type or the defaults of its level.
pub(crate) fn set_budget(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    position: &Position,
    shares: Option<(f64, f64)>,
) -> Result<(), CommandError> {
    check_own_position(catalog, state, actor, position)?;
    check_shares(shares)?;
    position_state_mut(state, actor, position).budget = shares;
    Ok(())
}

fn check_shares(shares: Option<(f64, f64)>) -> Result<(), CommandError> {
    match shares {
        Some((a, b)) if !((0.0..=1.0).contains(&a) && (0.0..=1.0).contains(&b) && a <= b) => {
            Err(CommandError::InvalidShare)
        }
        _ => Ok(()),
    }
}

/// `SetBudgetRule`: the shares for all positions of a type in a scope (MA3); `None`
/// removes the rule.
pub(crate) fn set_budget_rule(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    kind: &PositionKind,
    scope: RuleScope,
    shares: Option<(f64, f64)>,
) -> Result<(), CommandError> {
    check_shares(shares)?;
    let m = &catalog.management;
    let functions: &[usize] = match kind.level {
        UnitLevel::Site(site_type) => m.specialists_of(site_type),
        UnitLevel::Country => m.levels.get(1).map_or(&[], |l| l.specialists.as_slice()),
        UnitLevel::Continent => m.levels.get(2).map_or(&[], |l| l.specialists.as_slice()),
        UnitLevel::Board => m.levels.get(3).map_or(&[], |l| l.specialists.as_slice()),
    };
    let known = match &kind.role {
        Role::Head => true,
        Role::Specialist(f) => functions.iter().any(|&i| m.functions[i].key == *f),
    };
    if !known || !m.enabled() {
        return Err(CommandError::UnknownPosition);
    }
    let rules = &mut state.companies[actor.index()].budget_rules;
    rules.retain(|r| !(r.kind == *kind && r.scope == scope));
    if let Some(shares) = shares {
        rules.push(BudgetRule {
            kind: kind.clone(),
            scope,
            shares,
        });
    }
    Ok(())
}

/// What a command acts on, to find the concerns it settles (MA2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Object {
    Slot(SiteId, usize),
    Sale(SiteId, ProductId),
    Purchase(SiteId, ProductId),
    Supply(SiteId, ProductId),
    Wage(SiteId),
    Build(SiteId),
    Laboratory(SiteId),
    Deposit(SiteId),
    Advertising(CountryId, crate::ids::GoodsGroupId),
    Answer(u32),
    Bid(CompanyId, crate::deals::DealObject),
    /// A loan by its number (ZA3).
    Loan(usize),
    /// A manager's salary (ZA3).
    Salary(ManagerId),
}

fn object(command: &Command) -> Option<Object> {
    Some(match *command {
        Command::SetProduction { site, slot, .. }
        | Command::SetAutomation { site, slot, .. }
        | Command::MothballFacility { site, slot, .. }
        | Command::RestartFacility { site, slot }
        | Command::SellFacility { site, slot, .. } => Object::Slot(site, slot),
        Command::SetSale { site, product, .. } | Command::SetPrice { site, product, .. } => {
            Object::Sale(site, product)
        }
        Command::SetPurchase { site, product, .. } => Object::Purchase(site, product),
        Command::TransferGoods { to, product, .. } => Object::Supply(to, product),
        Command::SetWagePremium { site, .. } => Object::Wage(site),
        Command::BuildFacility { site, .. } => Object::Build(site),
        Command::SetResearch { site, .. } | Command::SetDevelopment { site, .. } => {
            Object::Laboratory(site)
        }
        Command::DevelopDeposit { site, .. } => Object::Deposit(site),
        Command::SetAdvertising { country, group, .. } => Object::Advertising(country, group),
        Command::AnswerOffer { offer, .. } => Object::Answer(offer),
        Command::MakeOffer { seller, object, .. } => Object::Bid(seller, object),
        Command::RepayLoan { loan, .. } | Command::RefinanceLoan { loan } => Object::Loan(loan),
        Command::RaiseSalary { manager, .. } => Object::Salary(manager),
        _ => return None,
    })
}

fn touches(d: &Decision, target: Object) -> bool {
    d.choices
        .iter()
        .flat_map(|choice| &choice.steps)
        .any(|s| object(&s.command) == Some(target))
}

/// A decision taken settles the company's open concerns about the same thing
/// (docs/MANAGER.md 6.3): the player's in a view, a position's within its budget. A
/// strategic concern loses the parts it settles, and with the last part itself.
pub(crate) fn settle(state: &mut GameState, company: CompanyId, command: &Command) {
    if state.concerns.is_empty() {
        return;
    }
    let Some(target) = object(command) else {
        return;
    };
    let today = state.date;
    for c in state
        .concerns
        .iter_mut()
        .filter(|c| c.company == company && c.status == ConcernStatus::Open)
    {
        if c.parts.is_empty() {
            if touches(&c.decision, target) {
                c.status = ConcernStatus::Settled;
                c.closed = Some(today);
            }
            continue;
        }
        let before = c.parts.len();
        c.parts.retain(|p| !touches(&p.decision, target));
        if c.parts.is_empty() {
            c.status = ConcernStatus::Settled;
            c.closed = Some(today);
        } else if c.parts.len() < before {
            c.options = part_options(&c.parts);
        }
    }
}

/// A company's open concerns of a topic are settled (ZA3): the loans they name changed
/// their numbers.
pub(crate) fn settle_topic(state: &mut GameState, company: CompanyId, topic: Topic) {
    let today = state.date;
    for c in state.concerns.iter_mut().filter(|c| {
        c.company == company && c.status == ConcernStatus::Open && c.decision.topic == topic
    }) {
        c.status = ConcernStatus::Settled;
        c.closed = Some(today);
    }
}

/// Whether a concern counts as important for halting a run of rounds (docs/MANAGER.md
/// 6.6): everything beyond the routine of the sites.
pub fn important(catalog: &Catalog, concern: &Concern) -> bool {
    !catalog
        .management
        .routine_topics
        .contains(&concern.decision.topic)
}

/// `AskAgain`: a position asks about a topic again that the player muted or declined.
pub(crate) fn ask_again(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    position: &Position,
    topic: Topic,
) -> Result<(), CommandError> {
    check_own_position(catalog, state, actor, position)?;
    let p = position_state_mut(state, actor, position);
    p.muted.remove(&topic);
    p.blocked.remove(&topic);
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
                && !positions(catalog, state, c.company, asker(c).unit).contains(asker(c))
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
    let kept: Vec<Vec<bool>> = state
        .companies
        .iter()
        .enumerate()
        .map(|(i, company)| {
            // Few companies; the cast is exact.
            let id = CompanyId(i as u32);
            company
                .positions
                .iter()
                .map(|p| has_unit(catalog, state, id, p.position.unit))
                .collect()
        })
        .collect();
    for (company, keep) in state.companies.iter_mut().zip(kept) {
        let mut keep = keep.into_iter();
        company.positions.retain(|_| keep.next().unwrap_or(false));
    }
}
