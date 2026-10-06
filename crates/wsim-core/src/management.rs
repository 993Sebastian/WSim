//! Managers (MA1, docs/FORMELN.md, docs/MANAGER.md): the positions of the sites, the
//! market of candidates per continent, salaries and the routine of the site positions.

use std::collections::{BTreeMap, BTreeSet};

use crate::calendar::{Date, days_in_month};
use crate::catalog::{Catalog, SiteType};
use crate::command::CommandError;
use crate::decision::{Decider, Decision, Topic, Verdict};
use crate::ids::{CountryId, Id};
use crate::ledger::{Account, CostCenter, CostType};
use crate::math;
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, GameState, Job, Manager, ManagerId, Position, Role, SiteId};

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

/// The decider of a company's site positions on a day: a topic runs at a site where a
/// position noticed it; everything else stays as it is.
struct Staff {
    noticed: BTreeSet<(SiteId, Topic)>,
}

impl Decider for Staff {
    fn looks(&self) -> bool {
        true
    }

    fn decide(&mut self, _: &GameState, _: &Catalog, decision: &Decision) -> Verdict {
        match decision.site {
            Some(site) if self.noticed.contains(&(site, decision.topic)) => Verdict::Rule,
            _ => Verdict::Hold,
        }
    }
}

/// Positions held, per company and site.
type Held = BTreeMap<CompanyId, BTreeMap<SiteId, Vec<(Role, ManagerId)>>>;

/// The routine of the site positions due today (docs/FORMELN.md, MA1), before the AI
/// companies decide.
pub fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) {
    let m = &catalog.management;
    let Some(level) = m.levels.first() else {
        return;
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
    for (company, sites) in held {
        if state.companies[company.index()].bankrupt {
            continue;
        }
        let mut noticed = BTreeSet::new();
        let mut due = Vec::new();
        let mut rngs: BTreeMap<ManagerId, SimRng> = BTreeMap::new();
        for (&site, roles) in &sites {
            // Staggered by the site number, like the AI companies. Laboratories have no
            // routine yet (their topics follow with the budgets of MA2).
            let kind = state.sites[site.index()].kind;
            if !(day + site.0).is_multiple_of(level.check_days) || kind == SiteType::ResearchCenter
            {
                continue;
            }
            let specialists = m.specialists_of(kind);
            let head = roles.iter().find(|(r, _)| *r == Role::Head).map(|r| r.1);
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
                let skill = f64::from(expertise(x, &function.key)) * (1.0 - discount);
                let diligence = (skill + f64::from(x.detection)) / 2.0;
                let chance = m.notice_base + (1.0 - m.notice_base) * diligence / 100.0;
                let seed = state.settings.seed;
                let rng = rngs.entry(manager).or_insert_with(|| {
                    SimRng::for_stream(seed, Stream::Manager { id: manager.0, day })
                });
                if rng.chance(chance) {
                    noticed.extend(function.topics.iter().map(|&t| (site, t)));
                    if due.last() != Some(&site) {
                        due.push(site);
                    }
                }
            }
        }
        if due.is_empty() {
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
        crate::ai::site_routine(
            state,
            catalog,
            company,
            (&own, &due),
            date,
            &mut Staff { noticed },
        );
    }
}
