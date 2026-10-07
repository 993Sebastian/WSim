//! The living market of managers (MA6, docs/FORMELN.md): experience, satisfaction and
//! resignations, heads that fill their unit's positions, AI companies that hire and
//! poach, and offers to the managers of other companies.

use std::collections::BTreeMap;

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::{self, Command, CommandError};
use crate::decision::{Choice, ChoiceKind, Decision, Topic};
use crate::ledger::CostType;
use crate::management::{self, expertise, strength};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{
    CompanyId, Concern, ConcernOption, ConcernReason, ConcernStatus, GameState, Job, Manager,
    ManagerId, PoachOffer, Position, Role, SiteId, Unit,
};

/// The function whose expertise a job uses: a specialist's own, a head's focus.
fn job_function<'a>(manager: &'a Manager, job: &'a Job) -> &'a str {
    match &job.position.role {
        Role::Specialist(f) => f,
        Role::Head => &manager.focus,
    }
}

/// A manager's satisfaction (0–100); the start value of the data where none was set.
pub fn satisfaction(catalog: &Catalog, job: &Job) -> u8 {
    job.satisfaction
        .unwrap_or(catalog.management.market.satisfaction.start)
}

/// The level of a satisfaction: 0 unhappy, 1 mixed, 2 happy.
pub fn satisfaction_level(catalog: &Catalog, value: u8) -> u8 {
    let bands = catalog.management.market.satisfaction.bands;
    if value < bands[0] {
        0
    } else if value < bands[1] {
        1
    } else {
        2
    }
}

/// What a manager is worth today: his demand for his position (MA1).
pub fn market_value(catalog: &Catalog, state: &GameState, manager: &Manager) -> Money {
    manager.job.as_ref().map_or(Money::ZERO, |j| {
        management::salary_demand(catalog, state, j.company, manager, &j.position)
    })
}

/// The month's random streams of the managers, each drawn in a fixed order.
struct Streams {
    seed: u64,
    month: u32,
    rngs: BTreeMap<ManagerId, SimRng>,
}

impl Streams {
    fn of(&mut self, id: ManagerId) -> &mut SimRng {
        let (seed, month) = (self.seed, self.month);
        self.rngs
            .entry(id)
            .or_insert_with(|| SimRng::for_stream(seed, Stream::ManagerMonth { id: id.0, month }))
    }
}

/// Managers drawn without a potential get one (docs/FORMELN.md, MA6): their highest
/// expertise and a share of the room, from a stream of their own.
pub(crate) fn set_potentials(state: &mut GameState, catalog: &Catalog) {
    let room = catalog.management.market.experience_room;
    let seed = state.settings.seed;
    for (id, m) in &mut state.managers {
        if m.potential.is_some() {
            continue;
        }
        let best = m.expertise.values().copied().max().unwrap_or(0);
        let mut rng = SimRng::for_stream(seed, Stream::ManagerPotential { id: id.0 });
        // At most the room (≤ 100); the cast is exact.
        let extra = rng.below(u64::from(room) + 1) as u8;
        m.potential = Some(best.saturating_add(extra).min(100));
    }
}

/// At the start of a month, after the market was refilled (docs/FORMELN.md, MA6):
/// experience, satisfaction, resignations, heads that hire, then the AI companies'
/// salaries, hiring and poaching, and last their competence. Returns the news for the
/// player.
pub fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    if !catalog.management.enabled() {
        return news;
    }
    let mut streams = Streams {
        seed: state.settings.seed,
        month: management::month_number(date),
        rngs: BTreeMap::new(),
    };
    experience(state, catalog, &mut streams);
    update_satisfaction(state, catalog, date);
    resignations(state, catalog, date, &mut streams, &mut news);
    heads_hire(state, catalog, &mut streams, &mut news);
    ai_staffing(state, catalog, date, &mut news);
    ai_competence(state, catalog);
    news
}

/// A point of expertise in the function of the job, with a chance, up to the potential.
fn experience(state: &mut GameState, catalog: &Catalog, streams: &mut Streams) {
    let chance = catalog.management.market.experience_chance;
    // Training by a personnel department (ZA2).
    let trained: BTreeMap<CompanyId, f64> = (0..state.companies.len())
        // Few companies; the cast is exact.
        .map(|i| CompanyId(i as u32))
        .filter(|&c| !state.companies[c.index()].departments.is_empty())
        .map(|c| {
            (
                c,
                crate::central::experience_chance(catalog, state, c, chance),
            )
        })
        .collect();
    for (&id, m) in &mut state.managers {
        let Some(job) = &m.job else {
            continue;
        };
        let p = trained.get(&job.company).copied().unwrap_or(chance);
        let gains = streams.of(id).chance(p);
        let function = job_function(m, job).to_owned();
        let cap = m.potential.unwrap_or(100);
        if let Some(x) = m
            .expertise
            .get_mut(&function)
            .filter(|x| gains && **x < cap)
        {
            *x += 1;
        }
    }
}

/// The result of a company's unit in the last twelve closed months: its sites' results.
pub fn unit_result(catalog: &Catalog, state: &GameState, company: CompanyId, unit: Unit) -> Money {
    let months = &state.companies[company.index()].ledger.months;
    months[months.len().saturating_sub(12)..]
        .iter()
        .flat_map(|m| &m.by_site)
        .filter(|(s, _)| management::in_unit(catalog, state, **s, unit))
        .map(|(_, v)| *v)
        .sum()
}

/// Concerns of each manager closed in the last twelve months against his recommendation.
fn overruled(state: &GameState, date: Date) -> BTreeMap<ManagerId, u32> {
    let year_ago = date.add_days(-365);
    let mut out: BTreeMap<ManagerId, u32> = BTreeMap::new();
    for c in &state.concerns {
        let against = match c.status {
            ConcernStatus::Chosen(i) => i != c.recommended,
            ConcernStatus::Declined => true,
            _ => false,
        };
        if against && c.closed.is_some_and(|d| d >= year_ago) {
            *out.entry(c.manager).or_default() += 1;
        }
    }
    out
}

/// The satisfaction a manager tends to (docs/FORMELN.md, MA6).
pub fn satisfaction_target(
    catalog: &Catalog,
    state: &GameState,
    manager: &Manager,
    overruled: u32,
) -> f64 {
    let s = &catalog.management.market.satisfaction;
    let Some(job) = &manager.job else {
        return s.base;
    };
    let market = market_value(catalog, state, manager);
    let pay = if market > Money::ZERO {
        100.0 * (job.salary.to_usd() / market.to_usd() - 1.0)
    } else {
        0.0
    };
    let loss = unit_result(catalog, state, job.company, job.position.unit) < Money::ZERO;
    s.base + s.salary_weight * pay
        - if loss { s.loss_penalty } else { 0.0 }
        - s.overruled_penalty * f64::from(overruled)
}

/// Every employed manager's satisfaction moves a share of the way to its target.
fn update_satisfaction(state: &mut GameState, catalog: &Catalog, date: Date) {
    let s = &catalog.management.market.satisfaction;
    let against = overruled(state, date);
    let new: Vec<(ManagerId, u8)> = state
        .managers
        .iter()
        .filter_map(|(&id, m)| {
            let job = m.job.as_ref()?;
            let target =
                satisfaction_target(catalog, state, m, against.get(&id).copied().unwrap_or(0));
            let old = f64::from(satisfaction(catalog, job));
            // Within 0–100; the cast is exact.
            let value = (old + s.adjust * (target - old)).round().clamp(0.0, 100.0) as u8;
            Some((id, value))
        })
        .collect();
    for (id, value) in new {
        if let Some(job) = state.managers.get_mut(&id).and_then(|m| m.job.as_mut()) {
            job.satisfaction = Some(value);
        }
    }
}

/// Where a position is, for messages: the country of a site or country, the continent,
/// or the board.
pub fn place_param(catalog: &Catalog, state: &GameState, unit: Unit) -> Param {
    match unit {
        Unit::Site(s) => Param::Country(
            catalog
                .countries
                .key(state.sites[s.index()].country)
                .to_owned(),
        ),
        Unit::Country(k) => Param::Country(catalog.countries.key(k).to_owned()),
        Unit::Continent(k) => Param::TextKey(format!("kontinent.{}", catalog.continents.key(k))),
        Unit::Board => Param::TextKey("ebene.vorstand".into()),
    }
}

/// A message about a manager of the player: his name, position and its place.
fn about(
    catalog: &Catalog,
    state: &GameState,
    kind: MessageKind,
    key: &str,
    manager: ManagerId,
) -> Message {
    let m = &state.managers[&manager];
    let mut message = Message::new(kind, key).with("name", Param::Text(m.name.clone()));
    if let Some(job) = &m.job {
        message = message
            .with("stelle", management::position_param(state, &job.position))
            .with("ort", place_param(catalog, state, job.position.unit));
    }
    message
}

/// Managers below the threshold resign with a chance (docs/FORMELN.md, MA6): salary to
/// the day, no severance; they return to the market.
fn resignations(
    state: &mut GameState,
    catalog: &Catalog,
    date: Date,
    streams: &mut Streams,
    news: &mut Vec<Message>,
) {
    let market = &catalog.management.market;
    let threshold = f64::from(market.resignation_threshold);
    let mut leaving = Vec::new();
    for (&id, m) in &state.managers {
        let Some(job) = &m.job else {
            continue;
        };
        let value = f64::from(satisfaction(catalog, job));
        if value >= threshold {
            continue;
        }
        let chance = market.resignation_chance * (threshold - value) / threshold;
        if streams.of(id).chance(chance) {
            leaving.push(id);
        }
    }
    for id in leaving {
        let company = state.managers[&id].job.as_ref().map(|j| j.company);
        if company == Some(state.player) {
            news.push(about(
                catalog,
                state,
                MessageKind::Warning,
                keys::MANAGER_RESIGNED,
                id,
            ));
        }
        management::end_job(state, id, date);
    }
}

/// Whether a specialist position has something to do (MA5, MA6): topics of its function
/// come up in its unit, or it is the board's personnel member.
pub fn has_work(catalog: &Catalog, state: &GameState, position: &Position) -> bool {
    let Role::Specialist(f) = &position.role else {
        return true;
    };
    let m = &catalog.management;
    let Some(i) = m.function(f) else {
        return false;
    };
    m.functions[i]
        .topics
        .iter()
        .any(|&t| management::arises(catalog, state, position.unit, t))
        || (position.unit == Unit::Board
            && management::function_of(catalog, Topic::Wage) == Some(i))
}

/// Whether a company offered someone one of its positions.
fn offered(state: &GameState, company: CompanyId, position: &Position) -> bool {
    state
        .poach_offers
        .iter()
        .any(|o| o.bidder == company && o.position == *position)
}

/// Free candidates from the continent of a unit's seat country.
fn candidates(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    unit: Unit,
) -> Vec<ManagerId> {
    let Some(country) = management::seat_country(catalog, state, company, unit) else {
        return Vec::new();
    };
    let continent = catalog.countries.get(country).continent;
    state
        .managers
        .iter()
        .filter(|(_, m)| m.job.is_none() && catalog.countries.get(m.home).continent == continent)
        .map(|(&id, _)| id)
        .collect()
}

/// The candidate a head picks for a specialist position (docs/FORMELN.md, MA6): the one
/// with the most expertise with a chance after its judgment, else the one that looks
/// best to the company.
fn head_pick(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    (pool, function): (&[ManagerId], &str),
    (judgment, rng): (u8, &mut SimRng),
) -> Option<ManagerId> {
    let base = catalog.management.concerns.recommend_base;
    let sees_true = rng.chance(base + (1.0 - base) * f64::from(judgment) / 100.0);
    let key = management::expertise_key(function);
    let share = management::impression_share(catalog, state, company);
    let score = |id: &ManagerId| -> u8 {
        let m = &state.managers[id];
        let value = expertise(m, function);
        if sees_true {
            value
        } else {
            let offset = m.impression.get(&key).copied().unwrap_or(0);
            management::shown_level(value, management::shown_impression(offset, share))
        }
    };
    // The first of the best: candidates in the order of their numbers.
    let mut best: Option<(ManagerId, u8)> = None;
    for id in pool {
        let s = score(id);
        if best.is_none_or(|(_, b)| s > b) {
            best = Some((*id, s));
        }
    }
    best.map(|(id, _)| id)
}

/// Heads with the switch fill a free specialist position of their unit with work, at most
/// one a month, within what is left of their yearly budget (docs/FORMELN.md, MA6).
fn heads_hire(
    state: &mut GameState,
    catalog: &Catalog,
    streams: &mut Streams,
    news: &mut Vec<Message>,
) {
    let heads: Vec<(CompanyId, Position)> = state
        .companies
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.bankrupt)
        .flat_map(|(i, c)| {
            c.positions
                .iter()
                .filter(|p| p.hires && p.position.role == Role::Head)
                // Few companies; the cast is exact.
                .map(move |p| (CompanyId(i as u32), p.position.clone()))
        })
        .collect();
    for (company, head) in heads {
        let Some(boss) = management::holder(state, company, &head) else {
            continue;
        };
        let free: Vec<Position> = management::positions(catalog, state, company, head.unit)
            .into_iter()
            .filter(|p| {
                matches!(p.role, Role::Specialist(_))
                    && management::holder(state, company, p).is_none()
                    && has_work(catalog, state, p)
                    && !offered(state, company, p)
            })
            .collect();
        for position in free {
            let Role::Specialist(function) = &position.role else {
                continue;
            };
            let all = candidates(catalog, state, company, head.unit);
            let focused: Vec<ManagerId> = all
                .iter()
                .copied()
                .filter(|id| state.managers[id].focus == *function)
                .collect();
            let pool = if focused.is_empty() { all } else { focused };
            let judgment = state.managers[&boss].judgment;
            let Some(pick) = head_pick(
                catalog,
                state,
                company,
                (&pool, function),
                (judgment, streams.of(boss)),
            ) else {
                continue;
            };
            let salary = management::salary_demand(
                catalog,
                state,
                company,
                &state.managers[&pick],
                &position,
            );
            let head_salary = state.managers[&boss]
                .job
                .as_ref()
                .map_or(Money::ZERO, |j| j.salary);
            let (_, per_year) = management::budget(catalog, state, company, &head, head_salary);
            if salary > per_year - management::spent(state, company, &head) {
                continue;
            }
            let hire = Command::HireManager {
                manager: pick,
                position: position.clone(),
            };
            if command::execute(state, catalog, company, &hire).is_err() {
                continue;
            }
            management::count_spent(state, company, &head, salary);
            if company == state.player {
                news.push(
                    about(
                        catalog,
                        state,
                        MessageKind::Info,
                        keys::MANAGER_HIRED_BY_HEAD,
                        pick,
                    )
                    .with("leitung", management::position_param(state, &head))
                    .with("gehalt", Param::Money(salary)),
                );
            }
            break;
        }
    }
}

/// `SetHiringByHead`: a head of the company fills the free positions of its unit itself.
pub(crate) fn set_hiring(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    position: &Position,
    enabled: bool,
) -> Result<(), CommandError> {
    management::check_own_position(catalog, state, actor, position)?;
    if position.role != Role::Head {
        return Err(CommandError::NotAHead);
    }
    management::position_state_mut(state, actor, position).hires = enabled;
    Ok(())
}

/// `RaiseSalary`: a higher yearly salary for a manager of the company.
pub(crate) fn raise_salary(
    state: &mut GameState,
    actor: CompanyId,
    manager: ManagerId,
    salary: Money,
) -> Result<(), CommandError> {
    let job = management::own_manager(state, actor, manager)?
        .job
        .as_ref()
        .expect("checked");
    if salary <= job.salary {
        return Err(CommandError::SalaryNotHigher);
    }
    if let Some(job) = state
        .managers
        .get_mut(&manager)
        .and_then(|m| m.job.as_mut())
    {
        job.salary = salary;
    }
    Ok(())
}

/// What a company offers another company's manager for one of its positions
/// (docs/FORMELN.md, MA6): the more of his demand and his salary with a markup.
pub fn poach_salary(
    catalog: &Catalog,
    state: &GameState,
    bidder: CompanyId,
    manager: &Manager,
    position: &Position,
) -> Money {
    let markup = catalog.management.market.poaching.markup;
    let salary = manager.job.as_ref().map_or(Money::ZERO, |j| j.salary);
    management::salary_demand(catalog, state, bidder, manager, position)
        .max(salary.scale(1.0 + markup))
}

/// The first day another offer may reach a manager (docs/FORMELN.md, MA6).
pub fn courted_until(catalog: &Catalog, manager: &Manager) -> Option<Date> {
    let pause = catalog.management.market.poaching.pause_months;
    manager.courted.map(|d| d.add_months(pause))
}

/// `PoachManager`: an offer to another company's manager for a free position of the
/// company; it stands until the deadline of concerns.
pub(crate) fn poach(
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
    let employer = match &m.job {
        None => return Err(CommandError::ManagerFree),
        Some(j) if j.company == actor => return Err(CommandError::OwnManager),
        Some(j) => j.company,
    };
    management::check_position(catalog, state, actor, position)?;
    if state.poach_offers.iter().any(|o| o.manager == manager) {
        return Err(CommandError::ManagerHasOffer);
    }
    let today = state.date;
    if let Some(until) = courted_until(catalog, m).filter(|&d| d > today) {
        return Err(CommandError::ManagerCourted { until });
    }
    if offered(state, actor, position) {
        return Err(CommandError::PositionTaken);
    }
    let salary = poach_salary(catalog, state, actor, m, position);
    let days = catalog.management.concerns.deadline_days;
    if let Some(m) = state.managers.get_mut(&manager) {
        m.courted = Some(today);
    }
    state.poach_offers.push(PoachOffer {
        manager,
        employer,
        bidder: actor,
        position: position.clone(),
        salary,
        made: today,
        until: today.add_days(i32::try_from(days).unwrap_or(i32::MAX)),
        asked: false,
    });
    Ok(())
}

/// The open offer to a manager of the acting company.
fn offer_to(
    state: &GameState,
    actor: CompanyId,
    manager: ManagerId,
) -> Result<usize, CommandError> {
    management::own_manager(state, actor, manager)?;
    state
        .poach_offers
        .iter()
        .position(|o| o.manager == manager && o.employer == actor)
        .ok_or(CommandError::NoPoachOffer)
}

/// The raise a counteroffer means for a year (counted against budgets).
pub fn raise_of(state: &GameState, manager: ManagerId) -> Money {
    let Some(offer) = state.poach_offers.iter().find(|o| o.manager == manager) else {
        return Money::ZERO;
    };
    let salary = state
        .managers
        .get(&manager)
        .and_then(|m| m.job.as_ref())
        .map_or(Money::ZERO, |j| j.salary);
    (offer.salary - salary).max(Money::ZERO)
}

/// `MatchOffer`: the company keeps its manager at the salary offered; he is content.
pub(crate) fn match_offer(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    manager: ManagerId,
) -> Result<(), CommandError> {
    let index = offer_to(state, actor, manager)?;
    let offer = state.poach_offers.remove(index);
    let start = catalog.management.market.satisfaction.start;
    if let Some(job) = state
        .managers
        .get_mut(&manager)
        .and_then(|m| m.job.as_mut())
    {
        job.salary = job.salary.max(offer.salary);
        job.satisfaction = Some(start);
    }
    close_concerns(state, manager, ConcernStatus::Settled);
    Ok(())
}

/// Open concerns about an offer to a manager close: answered otherwise, or void. An
/// answer to the concern itself sets its own status afterwards.
fn close_concerns(state: &mut GameState, manager: ManagerId, status: ConcernStatus) {
    let today = state.date;
    for c in &mut state.concerns {
        if c.status == ConcernStatus::Open
            && c.decision.topic == Topic::Poaching
            && poached(c) == Some(manager)
        {
            c.status = status;
            c.closed = Some(today);
        }
    }
}

/// `LetGo`: the manager goes to the company that made the offer, at its salary; his old
/// job pays to the day, without severance.
pub(crate) fn let_go(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    manager: ManagerId,
) -> Result<(), CommandError> {
    let index = offer_to(state, actor, manager)?;
    let offer = state.poach_offers[index].clone();
    if state.companies[offer.bidder.index()].bankrupt {
        return Err(CommandError::CompanyBankrupt);
    }
    management::check_position(catalog, state, offer.bidder, &offer.position)?;
    state.poach_offers.remove(index);
    close_concerns(state, manager, ConcernStatus::Settled);
    let today = state.date;
    management::end_job(state, manager, today);
    let start = catalog.management.market.satisfaction.start;
    if let Some(m) = state.managers.get_mut(&manager) {
        m.job = Some(Job {
            company: offer.bidder,
            position: offer.position,
            salary: offer.salary,
            since: today,
            satisfaction: Some(start),
        });
    }
    Ok(())
}

/// Whether a company keeps a manager against an offer (docs/FORMELN.md, MA6): the offer
/// at most a factor of his salary, and the cash pays a year of it.
pub fn keeps(catalog: &Catalog, state: &GameState, offer: &PoachOffer) -> bool {
    let factor = catalog.management.market.poaching.ai_counter_max;
    let salary = state
        .managers
        .get(&offer.manager)
        .and_then(|m| m.job.as_ref())
        .map_or(Money::ZERO, |j| j.salary);
    offer.salary <= salary.scale(factor)
        && state.companies[offer.employer.index()].ledger.cash() >= offer.salary
}

/// Every day (docs/FORMELN.md, MA6): offers that no longer fit end, offers past their
/// deadline lapse – the manager stays, disappointed – and new offers are taken up by
/// their employers: AI companies answer at once, the player's positions ask or decide.
pub fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    if state.poach_offers.is_empty() {
        return news;
    }
    drop_void_offers(state, catalog);
    let penalty = catalog.management.market.poaching.ignored_penalty;
    let (lapsed, kept): (Vec<PoachOffer>, Vec<PoachOffer>) =
        std::mem::take(&mut state.poach_offers)
            .into_iter()
            .partition(|o| o.until < date);
    state.poach_offers = kept;
    for offer in lapsed {
        if let Some(job) = state
            .managers
            .get_mut(&offer.manager)
            .and_then(|m| m.job.as_mut())
        {
            let value = satisfaction(catalog, job).saturating_sub(penalty);
            job.satisfaction = Some(value);
        }
        if offer.employer == state.player {
            news.push(
                about(
                    catalog,
                    state,
                    MessageKind::Info,
                    keys::MANAGER_STAYED,
                    offer.manager,
                )
                .with("firma", company_param(state, offer.bidder)),
            );
        }
    }
    let new: Vec<PoachOffer> = state
        .poach_offers
        .iter()
        .filter(|o| !o.asked)
        .cloned()
        .collect();
    for offer in new {
        for o in &mut state.poach_offers {
            if o.manager == offer.manager {
                o.asked = true;
            }
        }
        let employer = offer.employer;
        if state.companies[employer.index()].ai.is_some() {
            let answer = if keeps(catalog, state, &offer) {
                Command::MatchOffer {
                    manager: offer.manager,
                }
            } else {
                Command::LetGo {
                    manager: offer.manager,
                }
            };
            if command::execute(state, catalog, employer, &answer).is_err() {
                // Nothing came of it: the offer ends.
                state.poach_offers.retain(|o| o.manager != offer.manager);
            }
        } else {
            news.extend(ask(state, catalog, &offer, date));
        }
    }
    news
}

/// The name of a company for messages.
fn company_param(state: &GameState, company: CompanyId) -> Param {
    Param::Text(
        state
            .companies
            .get(company.index())
            .map_or_else(String::new, |c| c.name.clone()),
    )
}

/// Offers end that can no longer be taken up: the manager left his employer or the
/// bidder went bankrupt or lost the position; their concerns close.
fn drop_void_offers(state: &mut GameState, catalog: &Catalog) {
    let void: Vec<ManagerId> = state
        .poach_offers
        .iter()
        .filter(|o| {
            let employed = state
                .managers
                .get(&o.manager)
                .and_then(|m| m.job.as_ref())
                .is_some_and(|j| j.company == o.employer);
            !employed
                || state.companies[o.bidder.index()].bankrupt
                || management::check_position(catalog, state, o.bidder, &o.position).is_err()
        })
        .map(|o| o.manager)
        .collect();
    state.poach_offers.retain(|o| !void.contains(&o.manager));
    for manager in void {
        close_concerns(state, manager, ConcernStatus::Expired);
    }
}

/// The manager a poaching concern is about.
pub fn poached(c: &Concern) -> Option<ManagerId> {
    c.decision
        .choices
        .iter()
        .flat_map(|o| &o.steps)
        .find_map(|s| match s.command {
            Command::MatchOffer { manager } | Command::LetGo { manager } => Some(manager),
            _ => None,
        })
}

/// The open offer a poaching concern is about.
pub fn offer_of<'a>(state: &'a GameState, c: &Concern) -> Option<&'a PoachOffer> {
    let manager = poached(c)?;
    state.poach_offers.iter().find(|o| o.manager == manager)
}

/// Who asks the player about an offer to his manager (docs/FORMELN.md, MA6): the
/// personnel position of his unit, else the next filled one above, else he himself.
fn asker(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    manager: ManagerId,
) -> Option<(Position, ManagerId)> {
    let job = state.managers.get(&manager)?.job.as_ref()?;
    let own = (job.position.clone(), manager);
    let Some(f) = management::function_of(catalog, Topic::Poaching) else {
        return Some(own);
    };
    let key = &catalog.management.functions[f].key;
    let units = management::units_up(catalog, state, job.position.unit);
    let found = units.into_iter().find_map(|unit| {
        let position = Position {
            unit,
            role: Role::Specialist(key.clone()),
        };
        if !management::positions(catalog, state, company, unit).contains(&position) {
            return None;
        }
        management::holder(state, company, &position)
            .filter(|&id| id != manager)
            .map(|id| (position, id))
    });
    Some(found.unwrap_or(own))
}

/// The player's positions take up an offer to one of his managers: a concern, or the
/// position's own decision where the player let it decide the topic; nothing where the
/// topic rests.
fn ask(state: &mut GameState, catalog: &Catalog, offer: &PoachOffer, date: Date) -> Vec<Message> {
    let company = offer.employer;
    let Some((position, by)) = asker(catalog, state, company, offer.manager) else {
        return Vec::new();
    };
    let keep = keeps(catalog, state, offer);
    let recommended = usize::from(!keep);
    let choices = vec![
        Choice::one(
            ChoiceKind::Counter,
            Command::MatchOffer {
                manager: offer.manager,
            },
        ),
        Choice::one(
            ChoiceKind::LetGo,
            Command::LetGo {
                manager: offer.manager,
            },
        ),
    ];
    let raise = raise_of(state, offer.manager);
    let ps = management::position_state(state, company, &position);
    let muted = ps.is_some_and(|p| p.muted.contains(&Topic::Poaching));
    let resting = ps.is_some_and(|p| {
        p.blocked
            .get(&Topic::Poaching)
            .is_some_and(|&until| until > date)
    });
    if resting {
        return Vec::new();
    }
    if muted {
        // Within its budget the position decides itself; beyond it the offer lapses.
        let salary = state.managers[&by]
            .job
            .as_ref()
            .map_or(Money::ZERO, |j| j.salary);
        let (per_decision, per_year) =
            management::budget(catalog, state, company, &position, salary);
        let amount = if keep { raise } else { Money::ZERO };
        let left = per_year - management::spent(state, company, &position);
        if amount > per_decision || amount > left {
            return Vec::new();
        }
        let message = about(
            catalog,
            state,
            MessageKind::Info,
            keys::MANAGER_POACH_DECIDED,
            offer.manager,
        )
        .with("firma", company_param(state, offer.bidder))
        .with("angebot", Param::Money(offer.salary))
        .with(
            "option",
            Param::TextKey(format!("option.{}", choices[recommended].kind.key())),
        );
        if crate::decision::execute(state, catalog, company, &choices[recommended])
            .first()
            .copied()
            .unwrap_or(false)
        {
            management::count_spent(state, company, &position, amount);
            return vec![message];
        }
        return Vec::new();
    }
    let site = match offer_unit(state, offer) {
        Some(Unit::Site(s)) => Some(s),
        _ => None,
    };
    let message = about(
        catalog,
        state,
        MessageKind::Warning,
        keys::MANAGER_POACH,
        offer.manager,
    )
    .with("firma", company_param(state, offer.bidder))
    .with("angebot", Param::Money(offer.salary))
    .with(
        "gehalt",
        Param::Money(
            state.managers[&offer.manager]
                .job
                .as_ref()
                .map_or(Money::ZERO, |j| j.salary),
        ),
    )
    .with("frist", Param::Date(offer.until));
    let id = state.next_concern;
    state.next_concern += 1;
    state.concerns.push(Concern {
        id,
        company,
        position,
        manager: by,
        decision: Decision {
            topic: Topic::Poaching,
            company,
            site,
            product: None,
            choices,
            rule: recommended,
        },
        recommended,
        options: vec![
            ConcernOption {
                amount: raise,
                forecast: None,
                once: Money::ZERO,
            },
            ConcernOption {
                amount: Money::ZERO,
                forecast: None,
                once: Money::ZERO,
            },
        ],
        reason: ConcernReason::Poaching,
        path: Vec::new(),
        parts: Vec::new(),
        created: date,
        deadline: offer.until,
        status: ConcernStatus::Open,
        closed: None,
    });
    vec![message]
}

/// The unit of the manager an offer is about.
fn offer_unit(state: &GameState, offer: &PoachOffer) -> Option<Unit> {
    Some(
        state
            .managers
            .get(&offer.manager)?
            .job
            .as_ref()?
            .position
            .unit,
    )
}

/// Revenue of a company in the last twelve closed months, all of it or of one site.
fn revenue(state: &GameState, company: CompanyId, site: Option<SiteId>) -> Money {
    let months = &state.companies[company.index()].ledger.months;
    months[months.len().saturating_sub(12)..]
        .iter()
        .map(|m| match site {
            Some(s) => m.site_revenue.get(&s).copied().unwrap_or(Money::ZERO),
            None => m
                .by_type
                .get(&CostType::Revenue)
                .copied()
                .unwrap_or(Money::ZERO),
        })
        .sum()
}

/// The position an AI company fills next and the revenue its salary is measured on
/// (docs/FORMELN.md, MA6): the CEO from a revenue on, then the heads of its largest
/// sites; none while it offered someone a position.
fn next_position(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
) -> Option<(Position, Money)> {
    let ai = &catalog.management.market.ai;
    if state.poach_offers.iter().any(|o| o.bidder == company) {
        return None;
    }
    let ceo = Position {
        unit: Unit::Board,
        role: Role::Head,
    };
    let total = revenue(state, company, None);
    if total >= ai.ceo_revenue
        && management::positions(catalog, state, company, Unit::Board).contains(&ceo)
        && management::holder(state, company, &ceo).is_none()
    {
        return Some((ceo, total));
    }
    let mut sites: Vec<(SiteId, Money)> = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.owner == company)
        // Few sites; the cast is exact.
        .map(|(i, _)| SiteId(i as u32))
        .map(|s| (s, revenue(state, company, Some(s))))
        .filter(|&(_, r)| r >= ai.site_revenue)
        .collect();
    // The largest revenue first; on a tie the older site.
    sites.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    sites.into_iter().find_map(|(site, r)| {
        let head = Position::at_site(site, Role::Head);
        (management::positions(catalog, state, company, Unit::Site(site)).contains(&head)
            && management::holder(state, company, &head).is_none())
        .then_some((head, r))
    })
}

/// Hiring, salaries and poaching of the AI companies at a month start (docs/FORMELN.md,
/// MA6), through the same commands as the player.
fn ai_staffing(state: &mut GameState, catalog: &Catalog, date: Date, news: &mut Vec<Message>) {
    let market = &catalog.management.market;
    let month = management::month_number(date);
    let companies: Vec<CompanyId> = state
        .companies
        .iter()
        .enumerate()
        .filter(|(_, c)| c.ai.is_some() && !c.bankrupt)
        // Few companies; the cast is exact.
        .map(|(i, _)| CompanyId(i as u32))
        .collect();
    for company in companies {
        raise_salaries(state, catalog, company);
        let mut rng = SimRng::for_stream(
            state.settings.seed,
            Stream::Staffing {
                company: company.0,
                month,
            },
        );
        for _ in 0..market.ai.per_month {
            let Some((position, base)) = next_position(catalog, state, company) else {
                break;
            };
            if !fill(state, catalog, company, (&position, base), &mut rng, news) {
                break;
            }
        }
    }
}

/// An AI company raises the salary of an unhappy manager to his market value where its
/// cash pays a year of it.
fn raise_salaries(state: &mut GameState, catalog: &Catalog, company: CompanyId) {
    let mixed = catalog.management.market.satisfaction.bands[1];
    let raises: Vec<(ManagerId, Money)> = state
        .managers
        .iter()
        .filter_map(|(&id, m)| {
            let job = m.job.as_ref().filter(|j| j.company == company)?;
            let value = market_value(catalog, state, m);
            (satisfaction(catalog, job) < mixed && value > job.salary).then_some((id, value))
        })
        .collect();
    for (manager, salary) in raises {
        if state.companies[company.index()].ledger.cash() < salary {
            break;
        }
        let raise = Command::RaiseSalary { manager, salary };
        let _ = command::execute(state, catalog, company, &raise);
    }
}

/// An AI company fills a position: the strongest free candidate with the chance of its
/// competence, else one of the three strongest; or it poaches a much stronger manager of
/// another company. Returns whether it hired or made an offer.
fn fill(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    (position, base): (&Position, Money),
    rng: &mut SimRng,
    news: &mut Vec<Message>,
) -> bool {
    let market = &catalog.management.market;
    let limit = base
        .scale(market.ai.salary_share)
        .min(state.companies[company.index()].ledger.cash());
    let affordable = |state: &GameState, id: &ManagerId| {
        management::salary_demand(catalog, state, company, &state.managers[id], position) <= limit
    };
    let mut pool: Vec<(ManagerId, f64)> = candidates(catalog, state, company, position.unit)
        .into_iter()
        .filter(|id| affordable(state, id))
        .map(|id| (id, strength(&state.managers[&id])))
        .collect();
    // The strongest first; on a tie the lower number.
    pool.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let skill = state.companies[company.index()]
        .ai
        .as_ref()
        .map_or(0.0, |a| a.skill());
    let pick = if pool.is_empty() {
        None
    } else if rng.chance(skill) {
        Some(pool[0])
    } else {
        let n = pool.len().min(3);
        // At most three; the casts are exact.
        Some(pool[rng.below(n as u64) as usize])
    };
    if let Some(target) = poach_target(catalog, state, company, position, pick.map(|p| p.1)) {
        let salary = poach_salary(catalog, state, company, &state.managers[&target], position);
        if salary <= limit {
            let offer = Command::PoachManager {
                manager: target,
                position: position.clone(),
            };
            if command::execute(state, catalog, company, &offer).is_ok() {
                // The employer takes it up today: at once if it is an AI company.
                news.extend(simulate_day(state, catalog, state.date));
                return true;
            }
        }
    }
    let Some((manager, _)) = pick else {
        return false;
    };
    let hire = Command::HireManager {
        manager,
        position: position.clone(),
    };
    command::execute(state, catalog, company, &hire).is_ok()
}

/// The manager of another company an AI company would rather have (docs/FORMELN.md,
/// MA6): from the continent of the position, strong enough, without an offer, and
/// clearly stronger than the candidate it would take.
fn poach_target(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    position: &Position,
    candidate: Option<f64>,
) -> Option<ManagerId> {
    let p = &catalog.management.market.poaching;
    let country = management::seat_country(catalog, state, company, position.unit)?;
    let continent = catalog.countries.get(country).continent;
    let mut best: Option<(ManagerId, f64)> = None;
    for (&id, m) in &state.managers {
        let Some(job) = &m.job else {
            continue;
        };
        if job.company == company
            || state.companies[job.company.index()].bankrupt
            || catalog.countries.get(m.home).continent != continent
            || state.poach_offers.iter().any(|o| o.manager == id)
            || courted_until(catalog, m).is_some_and(|d| d > state.date)
        {
            continue;
        }
        let s = strength(m);
        if s >= p.min_strength && best.is_none_or(|(_, b)| s > b) {
            best = Some((id, s));
        }
    }
    let (id, s) = best?;
    (s >= candidate.unwrap_or(0.0) + p.lead).then_some(id)
}

/// What their managers add to the AI companies' competence (docs/FORMELN.md, MA6): the
/// CEO's strength, and the strength of the site heads by the share of sites with one.
fn ai_competence(state: &mut GameState, catalog: &Catalog) {
    let ai = &catalog.management.market.ai;
    let mut ceo: BTreeMap<CompanyId, f64> = BTreeMap::new();
    let mut heads: BTreeMap<CompanyId, (f64, u32)> = BTreeMap::new();
    for m in state.managers.values() {
        let Some(job) = &m.job else {
            continue;
        };
        match (job.position.unit, &job.position.role) {
            (Unit::Board, Role::Head) => {
                ceo.insert(job.company, strength(m));
            }
            (Unit::Site(_), Role::Head) => {
                let e = heads.entry(job.company).or_default();
                e.0 += strength(m);
                e.1 += 1;
            }
            _ => {}
        }
    }
    let mut sites: BTreeMap<CompanyId, u32> = BTreeMap::new();
    for s in &state.sites {
        *sites.entry(s.owner).or_default() += 1;
    }
    for (i, c) in state.companies.iter_mut().enumerate() {
        let Some(a) = c.ai.as_mut() else {
            continue;
        };
        // Few companies; the cast is exact.
        let id = CompanyId(i as u32);
        let mut staff = 0.0;
        if let Some(s) = ceo.get(&id) {
            staff += ai.competence_ceo * (s - 50.0) / 50.0;
        }
        if let (Some(&(sum, n)), Some(&count)) = (heads.get(&id), sites.get(&id))
            && n > 0
            && count > 0
        {
            let share = (f64::from(n) / f64::from(count)).min(1.0);
            staff += ai.competence_heads * share * (sum / f64::from(n) - 50.0) / 50.0;
        }
        a.staff = staff;
    }
}
