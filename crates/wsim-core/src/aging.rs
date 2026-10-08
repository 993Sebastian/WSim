//! Age, retirement and death of the managers (docs/FORMELN.md, PE1).
//!
//! Every manager has a day of birth; his age follows from the day. Age changes how fast
//! he learns, lowers his readiness for risks and, late, his detection and leadership. He
//! retires at his country's retirement age – a year before, the player's company is asked
//! about his successor – or dies earlier by a simple life table.

use crate::calendar::Date;
use crate::catalog::{AgeSpan, Catalog, LifeModel};
use crate::command::{Command, CommandError};
use crate::decision::{self, Choice, ChoiceKind, Decision, Topic};
use crate::management;
use crate::math;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::staffing;
use crate::state::{
    CompanyId, Concern, ConcernOption, ConcernReason, ConcernStatus, Departure, FormerManager,
    GameState, Hop, Job, Manager, ManagerId, Position, Role,
};

/// Full years of age on a day.
pub fn age(born: Date, today: Date) -> u32 {
    let mut years = today.year() - born.year();
    if (today.month(), today.day()) < (born.month(), born.day()) {
        years -= 1;
    }
    u32::try_from(years).unwrap_or(0)
}

/// A manager's age on a day; none without a day of birth.
pub fn age_of(manager: &Manager, today: Date) -> Option<u32> {
    manager.born.map(|b| age(b, today))
}

/// The level a new candidate is drawn for, by his strength: 0 site … 3 board.
fn level_by_strength(life: &LifeModel, strength: f64) -> usize {
    life.level_strength
        .iter()
        .filter(|&&s| strength >= s)
        .count()
}

/// Day of retirement: the first month start on or after the day he reaches his
/// retirement age (docs/FORMELN.md, PE1); none while the managers do not age.
pub fn retirement(catalog: &Catalog, manager: &Manager) -> Option<Date> {
    let life = &catalog.life;
    let born = manager.born.filter(|_| life.enabled)?;
    let year = f64::from(born.year()) + life.reference_age;
    let base = life.retirement_age.value(manager.home, year);
    let years = base + f64::from(manager.retire_offset) + f64::from(manager.extended);
    // Some hundred months; the cast is exact.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let reached = born.add_months((years * 12.0).round().max(0.0) as u32);
    Some(if reached.day() == 1 {
        reached
    } else {
        reached.first_of_next_month()
    })
}

/// The age at which he plans to retire.
pub fn retirement_age(catalog: &Catalog, manager: &Manager) -> Option<u32> {
    let born = manager.born?;
    retirement(catalog, manager).map(|d| age(born, d))
}

/// Factor on the chance to gain experience (MA6) by age.
pub fn experience_factor(catalog: &Catalog, manager: &Manager, today: Date) -> f64 {
    let life = &catalog.life;
    match age_of(manager, today)
        .filter(|_| life.enabled)
        .map(f64::from)
    {
        Some(a) if a < life.young_until => life.young_factor,
        Some(a) if a >= life.old_from => life.old_factor,
        _ => 1.0,
    }
}

/// Chance to die within a month (docs/FORMELN.md, PE1).
pub fn death_chance(catalog: &Catalog, manager: &Manager, today: Date) -> f64 {
    let life = &catalog.life;
    let Some(a) = age_of(manager, today)
        .filter(|_| life.enabled)
        .map(f64::from)
    else {
        return 0.0;
    };
    if a < life.mortality_from {
        return 0.0;
    }
    let expectancy = life
        .life_expectancy
        .value(manager.home, today.year_fraction());
    let doubling = life.doubling_years.max(1.0);
    (life.mortality_chance * math::pow(2.0, (a - expectancy) / doubling)).clamp(0.0, 1.0)
}

/// Chance that he agrees to stay longer: his satisfaction, less with age.
pub fn acceptance(catalog: &Catalog, manager: &Manager, today: Date) -> f64 {
    let Some(job) = &manager.job else {
        return 0.0;
    };
    let [from, until] = catalog.life.acceptance_ages;
    let a = age_of(manager, today).map_or(0.0, f64::from);
    let by_age = (1.0 - (a - from) / (until - from).max(1.0)).clamp(0.0, 1.0);
    f64::from(staffing::satisfaction(catalog, job)) / 100.0 * by_age
}

/// Whether he retires within the warning period.
fn retires_soon(catalog: &Catalog, manager: &Manager, today: Date) -> bool {
    retirement(catalog, manager).is_some_and(|d| d <= today.add_months(catalog.life.warning_months))
}

/// An age drawn from a truncated normal distribution.
fn draw_age(rng: &mut SimRng, span: AgeSpan) -> f64 {
    let spread = span.spread.max(1e-9);
    let low = math::normal_cdf((span.min - span.mean) / spread);
    let high = math::normal_cdf((span.max - span.mean) / spread);
    let u = (low + rng.next_f64() * (high - low)).clamp(1e-9, 1.0 - 1e-9);
    (span.mean + spread * math::normal_inverse(u)).clamp(span.min, span.max)
}

/// Managers without a day of birth get one (docs/FORMELN.md, PE1): new candidates by
/// their strength, managers of older saves by the level of their position.
pub(crate) fn set_births(state: &mut GameState, catalog: &Catalog, today: Date) {
    let life = &catalog.life;
    if !life.enabled {
        return;
    }
    let seed = state.settings.seed;
    for (id, m) in &mut state.managers {
        if m.born.is_some() {
            continue;
        }
        let level = m.job.as_ref().map_or_else(
            || level_by_strength(life, management::strength(m)),
            |j| management::level_of(j.position.unit),
        );
        let mut rng = SimRng::for_stream(seed, Stream::ManagerBirth { id: id.0 });
        let years = draw_age(&mut rng, life.entry_age[level.min(3)]);
        // Some ten thousand days and a few years; the casts are exact.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            m.born = Some(today.add_days(-((years * 365.25).round() as i32)));
            let spread = life.retirement_spread.round().max(0.0) as u64;
            m.retire_offset = (rng.below(2 * spread + 1) as i64 - spread as i64) as i8;
        }
    }
}

/// At a month start, before the market is refilled (docs/FORMELN.md, PE1): birthdays,
/// deaths and retirements of all managers, then the warnings of the player's company.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let life = &catalog.life;
    if !life.enabled {
        return news;
    }
    let before = date.add_days(-1).first_of_month();
    let (seed, month) = (state.settings.seed, management::month_number(date));
    let mut leaving = Vec::new();
    for (&id, m) in &mut state.managers {
        let Some(born) = m.born else {
            continue;
        };
        let mut rng = SimRng::for_stream(seed, Stream::ManagerAge { id: id.0, month });
        let now = age(born, date);
        if now > age(born, before) {
            birthday(life, m, now, &mut rng);
        }
        if rng.chance(death_chance(catalog, m, date)) {
            leaving.push((id, Departure::Died));
        } else if retirement(catalog, m).is_some_and(|d| d <= date) {
            leaving.push((id, Departure::Retired));
        }
    }
    for (id, reason) in leaving {
        depart(state, catalog, id, reason, date, &mut news);
    }
    tidy(state, catalog, date);
    warn(state, catalog, date, &mut news);
    news
}

/// A birthday: less readiness for risks from 40, from 65 perhaps less detection and
/// leadership.
fn birthday(life: &LifeModel, m: &mut Manager, age: u32, rng: &mut SimRng) {
    let a = f64::from(age);
    let total = |years: f64| {
        (life.risk_per_year * (years - life.risk_from).max(0.0))
            .floor()
            .min(life.risk_max)
    };
    // At most the cap (≤ 100); the cast is exact.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let drop = (total(a) - total(a - 1.0)).max(0.0) as u8;
    m.risk = m.risk.saturating_sub(drop);
    if a >= life.decline_from {
        if rng.chance(life.decline_chance) {
            m.detection = m.detection.saturating_sub(1);
        }
        if rng.chance(life.decline_chance) {
            m.leadership = m.leadership.saturating_sub(1);
        }
    }
}

/// The successor waiting for a position (PE1).
pub fn successor_of(
    state: &GameState,
    company: CompanyId,
    position: &Position,
) -> Option<ManagerId> {
    state
        .managers
        .iter()
        .find(|(_, m)| {
            m.job
                .as_ref()
                .is_some_and(|j| j.successor && j.company == company && j.position == *position)
        })
        .map(|(&id, _)| id)
}

/// A manager leaves for good: a free candidate leaves the market; an employed one leaves
/// his position to his successor, or free, and stays in his company's history.
fn depart(
    state: &mut GameState,
    catalog: &Catalog,
    id: ManagerId,
    reason: Departure,
    date: Date,
    news: &mut Vec<Message>,
) {
    let Some(m) = state.managers.get(&id) else {
        return;
    };
    if m.family {
        match reason {
            Departure::Died => crate::person::manager_died(state, id, date, news),
            Departure::Retired => crate::person::manager_retired(state, id),
        }
    }
    let Some(m) = state.managers.get(&id) else {
        return;
    };
    let Some(job) = m.job.clone() else {
        state.managers.remove(&id);
        return;
    };
    let years = age_of(m, date).unwrap_or(0);
    let successor = if job.successor {
        None
    } else {
        successor_of(state, job.company, &job.position)
    };
    let player = state.is_main(job.company) && !state.companies[job.company.index()].bankrupt;
    let message = player.then(|| {
        let key = match (job.successor, reason, successor.is_some()) {
            (true, ..) => keys::MANAGER_SUCCESSOR_GONE,
            (false, Departure::Retired, false) => keys::MANAGER_RETIRED,
            (false, Departure::Retired, true) => keys::MANAGER_RETIRED_SUCCESSOR,
            (false, Departure::Died, false) => keys::MANAGER_DIED,
            (false, Departure::Died, true) => keys::MANAGER_DIED_SUCCESSOR,
        };
        let kind = match reason {
            Departure::Retired => MessageKind::Info,
            Departure::Died => MessageKind::Warning,
        };
        let mut message = staffing::about(catalog, state, kind, key, id)
            .with("alter", Param::Number(f64::from(years)));
        if let Some(s) = successor {
            message = message.with("nachfolger", Param::Text(state.managers[&s].name.clone()));
        }
        message
    });
    management::end_job(state, id, date);
    state.poach_offers.retain(|o| o.manager != id);
    staffing::close_concerns(state, id, ConcernStatus::Expired);
    let Some(m) = state.managers.remove(&id) else {
        return;
    };
    if let Some(job) = successor.and_then(|s| state.managers.get_mut(&s)?.job.as_mut()) {
        job.successor = false;
    }
    if job.successor {
        // The holder's succession is open again.
        if let Some(h) = management::holder(state, job.company, &job.position)
            && let Some(x) = state.managers.get_mut(&h)
        {
            x.succession_asked = false;
        }
    }
    hand_over(state, catalog, id, &job, successor, date);
    state.companies[job.company.index()]
        .former_managers
        .push(FormerManager {
            name: m.name,
            born: m.born,
            position: job.position.clone(),
            since: job.since,
            until: date,
            judged: m.judged,
            hits: m.hits,
            reason,
        });
    news.extend(message);
    if player && reason == Departure::Died && successor.is_none() && !job.successor {
        ask(state, catalog, job.company, &job.position, None, date, news);
    }
}

/// The open concerns of a manager who left: those about his succession end; the others
/// go to his successor, else to the next filled position above, else they expire.
fn hand_over(
    state: &mut GameState,
    catalog: &Catalog,
    id: ManagerId,
    job: &Job,
    successor: Option<ManagerId>,
    date: Date,
) {
    let above = if successor.is_none() {
        asker(catalog, state, job.company, &job.position, Some(id)).filter(|(_, m)| *m != id)
    } else {
        None
    };
    for i in 0..state.concerns.len() {
        let c = &state.concerns[i];
        if c.status != ConcernStatus::Open {
            continue;
        }
        if c.decision.topic == Topic::Succession
            && c.reason == ConcernReason::Retirement
            && subject(state, c).as_ref() == Some(&job.position)
        {
            let c = &mut state.concerns[i];
            c.status = ConcernStatus::Expired;
            c.closed = Some(date);
            continue;
        }
        if c.manager != id {
            continue;
        }
        let c = &mut state.concerns[i];
        match (successor, &above) {
            (Some(s), _) => {
                c.manager = s;
                for h in &mut c.path {
                    if h.manager == id {
                        h.manager = s;
                    }
                }
            }
            (None, Some((position, m))) => {
                if c.path.is_empty() {
                    c.path.push(Hop {
                        position: c.position.clone(),
                        manager: id,
                        recommended: c.recommended,
                    });
                }
                c.path.push(Hop {
                    position: position.clone(),
                    manager: *m,
                    recommended: c.recommended,
                });
                c.manager = *m;
            }
            (None, None) => {
                c.status = ConcernStatus::Expired;
                c.closed = Some(date);
            }
        }
    }
}

/// The position a concern about a succession is about.
pub fn subject(state: &GameState, c: &Concern) -> Option<Position> {
    c.decision
        .choices
        .iter()
        .flat_map(|choice| &choice.steps)
        .find_map(|s| match &s.command {
            Command::AppointSuccessor { position, .. } | Command::LeaveVacant { position } => {
                Some(position.clone())
            }
            Command::ExtendContract { manager, .. } => state
                .managers
                .get(manager)?
                .job
                .as_ref()
                .map(|j| j.position.clone()),
            _ => None,
        })
}

/// Concerns about successions that were settled otherwise close; a holder whose chosen
/// successor left is asked again.
fn tidy(state: &mut GameState, catalog: &Catalog, date: Date) {
    let mut settled = Vec::new();
    for (i, c) in state.concerns.iter().enumerate() {
        if c.status != ConcernStatus::Open || c.decision.topic != Topic::Succession {
            continue;
        }
        let Some(position) = subject(state, c) else {
            settled.push(i);
            continue;
        };
        let holder = management::holder(state, c.company, &position);
        let done = match c.reason {
            ConcernReason::Vacancy => holder.is_some(),
            _ => holder.is_none() || successor_of(state, c.company, &position).is_some(),
        };
        if done {
            settled.push(i);
        }
    }
    for i in settled {
        let c = &mut state.concerns[i];
        c.status = ConcernStatus::Settled;
        c.closed = Some(date);
    }
    // Holders whose successor was chosen, but left before taking over.
    let Some(player) = state.main_company else {
        return;
    };
    let again: Vec<ManagerId> = state
        .managers
        .iter()
        .filter(|(_, m)| m.succession_asked)
        .filter_map(|(&id, m)| {
            let job = m
                .job
                .as_ref()
                .filter(|j| j.company == player && !j.successor)?;
            let open = state.concerns.iter().any(|c| {
                c.status == ConcernStatus::Open
                    && c.decision.topic == Topic::Succession
                    && subject(state, c).as_ref() == Some(&job.position)
            });
            let chosen = state
                .concerns
                .iter()
                .filter(|c| {
                    c.decision.topic == Topic::Succession
                        && subject(state, c).as_ref() == Some(&job.position)
                })
                .max_by_key(|c| (c.created, c.id))
                .is_some_and(|c| match c.status {
                    ConcernStatus::Chosen(k) => c.decision.choices.get(k).is_some_and(|o| {
                        matches!(o.kind, ChoiceKind::Successor | ChoiceKind::Promote)
                    }),
                    _ => false,
                });
            let waiting = successor_of(state, player, &job.position).is_some();
            (!open && chosen && !waiting && retires_soon(catalog, m, date)).then_some(id)
        })
        .collect();
    for id in again {
        if let Some(m) = state.managers.get_mut(&id) {
            m.succession_asked = false;
        }
    }
}

/// The player's managers who retire within the warning period are asked about.
fn warn(state: &mut GameState, catalog: &Catalog, date: Date, news: &mut Vec<Message>) {
    let Some(player) = state.main_company else {
        return;
    };
    if state.companies[player.index()].bankrupt {
        return;
    }
    let due: Vec<(ManagerId, Position)> = state
        .managers
        .iter()
        .filter(|(_, m)| !m.succession_asked && retires_soon(catalog, m, date))
        .filter_map(|(&id, m)| {
            let job = m
                .job
                .as_ref()
                .filter(|j| j.company == player && !j.successor)?;
            Some((id, job.position.clone()))
        })
        .collect();
    for (id, position) in due {
        if successor_of(state, player, &position).is_none() {
            ask(state, catalog, player, &position, Some(id), date, news);
        }
        if let Some(m) = state.managers.get_mut(&id) {
            m.succession_asked = true;
        }
    }
}

/// Who asks the player about a succession: the personnel position of the unit or one
/// above, else the next filled head above; for a retiring manager without either, he
/// himself.
fn asker(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    position: &Position,
    leaving: Option<ManagerId>,
) -> Option<(Position, ManagerId)> {
    let personnel = management::function_of(catalog, Topic::Succession)
        .map(|f| catalog.management.functions[f].key.clone());
    for unit in management::units_up(catalog, state, position.unit) {
        let own = unit == position.unit;
        if own && position.role == Role::Head {
            continue;
        }
        let roles = personnel
            .iter()
            .map(|k| Role::Specialist(k.clone()))
            .chain(std::iter::once(Role::Head));
        for role in roles {
            let p = Position { unit, role };
            if p == *position || !management::positions(catalog, state, company, unit).contains(&p)
            {
                continue;
            }
            if let Some(h) = management::holder(state, company, &p).filter(|&h| Some(h) != leaving)
            {
                return Some((p, h));
            }
        }
    }
    leaving.map(|m| (position.clone(), m))
}

/// Whether a position lies below another one for a promotion: a specialist position of
/// the same function in a unit below, for a head also the specialists of its own unit.
fn below(catalog: &Catalog, state: &GameState, upper: &Position, lower: &Position) -> bool {
    let under = management::level_of(lower.unit) < management::level_of(upper.unit)
        && management::units_up(catalog, state, lower.unit).contains(&upper.unit);
    match (&upper.role, &lower.role) {
        (Role::Specialist(a), Role::Specialist(b)) => a == b && under,
        (Role::Head, Role::Head) => under,
        (Role::Head, Role::Specialist(_)) => lower.unit == upper.unit,
        (Role::Specialist(_), Role::Head) => false,
    }
}

/// The strongest free candidate of the position's continent with its focus (for a
/// specialist position) who stays long enough, and his demand.
fn from_market(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    position: &Position,
    horizon: Date,
) -> Option<(ManagerId, Money)> {
    let country = management::seat_country(catalog, state, company, position.unit)?;
    let continent = catalog.countries.get(country).continent;
    let focus = match &position.role {
        Role::Specialist(k) => Some(k),
        Role::Head => None,
    };
    state
        .managers
        .iter()
        .filter(|(_, m)| {
            m.job.is_none()
                && catalog.countries.get(m.home).continent == continent
                && focus.is_none_or(|k| m.focus == *k)
                && retirement(catalog, m).is_none_or(|d| d > horizon)
        })
        .max_by(|a, b| {
            management::strength(a.1)
                .total_cmp(&management::strength(b.1))
                .then(b.0.cmp(a.0))
        })
        .map(|(&id, m)| {
            (
                id,
                management::salary_demand(catalog, state, company, m, position),
            )
        })
}

/// The strongest manager of the company on a position below who stays long enough, and
/// his salary on the position.
fn from_below(
    catalog: &Catalog,
    state: &GameState,
    company: CompanyId,
    position: &Position,
    horizon: Date,
) -> Option<(ManagerId, Money)> {
    state
        .managers
        .iter()
        .filter(|(_, m)| {
            m.job.as_ref().is_some_and(|j| {
                j.company == company && !j.successor && below(catalog, state, position, &j.position)
            }) && retirement(catalog, m).is_none_or(|d| d > horizon)
        })
        .max_by(|a, b| {
            management::strength(a.1)
                .total_cmp(&management::strength(b.1))
                .then(b.0.cmp(a.0))
        })
        .map(|(&id, m)| {
            let demand = management::salary_demand(catalog, state, company, m, position);
            let now = m.job.as_ref().map_or(Money::ZERO, |j| j.salary);
            (id, demand.max(now))
        })
}

/// The concern „Nachfolge regeln“ about a manager who retires soon, or „Stelle neu
/// besetzen“ about a free position (docs/FORMELN.md, PE1); where the player let the
/// asking position decide the topic, it decides by its recommendation.
fn ask(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    position: &Position,
    retiring: Option<ManagerId>,
    date: Date,
    news: &mut Vec<Message>,
) {
    let life = &catalog.life;
    let Some((by_position, by)) = asker(catalog, state, company, position, retiring) else {
        return;
    };
    let due = retiring.and_then(|r| retirement(catalog, &state.managers[&r]));
    let horizon = due.unwrap_or(date).add_months(life.warning_months);
    let mut choices = Vec::new();
    let mut options = Vec::new();
    let mut strengths = Vec::new();
    let option = |amount| ConcernOption {
        amount,
        forecast: None,
        once: Money::ZERO,
    };
    let candidates = [
        (
            ChoiceKind::Successor,
            from_market(catalog, state, company, position, horizon),
        ),
        (
            ChoiceKind::Promote,
            from_below(catalog, state, company, position, horizon),
        ),
    ];
    for (kind, found) in candidates {
        if let Some((manager, salary)) = found {
            choices.push(Choice::one(
                kind,
                Command::AppointSuccessor {
                    manager,
                    position: position.clone(),
                },
            ));
            options.push(option(salary));
            strengths.push(management::strength(&state.managers[&manager]));
        }
    }
    let mut extend = None;
    if let Some(r) = retiring {
        let m = &state.managers[&r];
        let max = u8::try_from(life.extension_years_max).unwrap_or(u8::MAX);
        let years = max.saturating_sub(m.extended);
        if years > 0 && !m.extension_refused {
            let salary = m.job.as_ref().map_or(Money::ZERO, |j| j.salary);
            extend = Some(choices.len());
            choices.push(Choice::one(
                ChoiceKind::Extend,
                Command::ExtendContract { manager: r, years },
            ));
            options.push(option(salary.scale(life.extension_raise)));
        }
    }
    choices.push(Choice::one(
        ChoiceKind::Vacant,
        Command::LeaveVacant {
            position: position.clone(),
        },
    ));
    options.push(option(Money::ZERO));
    // The stronger candidate; without one staying longer, else leaving it free.
    let strongest = strengths
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(&a.0)))
        .map(|(i, _)| i);
    let recommended = strongest.or(extend).unwrap_or(choices.len() - 1);
    let muted = management::position_state(state, company, &by_position)
        .is_some_and(|p| p.muted.contains(&Topic::Succession));
    if muted {
        let message = Message::new(MessageKind::Info, keys::MANAGER_SUCCESSION_DECIDED)
            .with("stelle", management::position_param(state, position))
            .with("ort", staffing::place_param(catalog, state, position.unit))
            .with(
                "option",
                Param::TextKey(format!("option.{}", choices[recommended].kind.key())),
            );
        decision::execute(state, catalog, company, &choices[recommended]);
        news.push(message);
        return;
    }
    let deadline = due.map_or_else(
        || date.add_days(i32::try_from(catalog.management.concerns.deadline_days).unwrap_or(30)),
        |d| d.add_days(-1),
    );
    let message = match retiring {
        Some(r) => {
            let refused = state.managers[&r].extension_refused;
            let key = if refused {
                keys::MANAGER_RETIRING_REFUSED
            } else {
                keys::MANAGER_RETIRING
            };
            let m = &state.managers[&r];
            let years = due.and_then(|d| m.born.map(|b| age(b, d))).unwrap_or(0);
            Some(
                staffing::about(catalog, state, MessageKind::Warning, key, r)
                    .with("termin", Param::Date(due.unwrap_or(date)))
                    .with("alter", Param::Number(f64::from(years)))
                    .with("frist", Param::Date(deadline)),
            )
        }
        None => None,
    };
    let id = state.next_concern;
    state.next_concern += 1;
    state.concerns.push(Concern {
        id,
        company,
        position: by_position,
        manager: by,
        decision: Decision {
            topic: Topic::Succession,
            company,
            site: position.site(),
            product: None,
            choices,
            rule: recommended,
        },
        recommended,
        options,
        reason: if retiring.is_some() {
            ConcernReason::Retirement
        } else {
            ConcernReason::Vacancy
        },
        path: Vec::new(),
        parts: Vec::new(),
        created: date,
        deadline,
        status: ConcernStatus::Open,
        closed: None,
    });
    news.extend(message);
}

/// `AppointSuccessor`: a free candidate or a manager of the company from another
/// position becomes successor of a holder who retires within the warning period, or
/// takes the free position at once (PE1).
pub(crate) fn appoint(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    manager: ManagerId,
    position: &Position,
) -> Result<(), CommandError> {
    management::check_own_position(catalog, state, actor, position)?;
    let m = state
        .managers
        .get(&manager)
        .ok_or(CommandError::UnknownManager)?;
    crate::person::check_family(state, actor, m)?;
    let own = match &m.job {
        None => false,
        Some(j) if j.company == actor && !j.successor && j.position != *position => true,
        Some(j) if j.company == actor => return Err(CommandError::PositionTaken),
        Some(_) => return Err(CommandError::ManagerEmployed),
    };
    let today = state.date;
    let holder = management::holder(state, actor, position);
    if let Some(h) = holder
        && (successor_of(state, actor, position).is_some()
            || !retires_soon(catalog, &state.managers[&h], today))
    {
        return Err(CommandError::PositionTaken);
    }
    let demand = management::salary_demand(catalog, state, actor, m, position);
    let waits = holder.is_some();
    if own {
        let job = state
            .managers
            .get_mut(&manager)
            .and_then(|m| m.job.as_mut())
            .expect("checked");
        // The old position pays the days of the month so far, the new one the rest.
        let old = job.clone();
        job.position = position.clone();
        job.salary = job.salary.max(demand);
        job.since = today;
        job.successor = waits;
        let pay = management::salary_for(&old, old.since.max(today.first_of_month()), today);
        management::book_personnel(state, actor, old.position.unit, pay);
    } else {
        state.managers.get_mut(&manager).expect("checked").job = Some(Job {
            company: actor,
            position: position.clone(),
            salary: demand,
            since: today,
            satisfaction: Some(catalog.management.market.satisfaction.start),
            successor: waits,
        });
    }
    Ok(())
}

/// `ExtendContract`: the company offers a manager who retires soon to stay longer at a
/// higher salary (PE1); he agrees with a chance. Either way the succession is asked
/// about again: after a refusal at once, else before the new day.
pub(crate) fn extend(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    manager: ManagerId,
    years: u8,
) -> Result<(), CommandError> {
    let life = &catalog.life;
    let today = state.date;
    let m = management::own_manager(state, actor, manager)?;
    if m.job.as_ref().is_some_and(|j| j.successor) || !retires_soon(catalog, m, today) {
        return Err(CommandError::NotRetiring);
    }
    let max = u8::try_from(life.extension_years_max)
        .unwrap_or(u8::MAX)
        .saturating_sub(m.extended);
    if years == 0 || years > max {
        return Err(CommandError::ExtensionTooLong { max });
    }
    let chance = acceptance(catalog, m, today);
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::Extension {
            id: manager.0,
            day: management::day_number(today),
        },
    );
    let agrees = rng.chance(chance);
    let m = state.managers.get_mut(&manager).expect("checked");
    m.succession_asked = false;
    if agrees {
        m.extended += years;
        if let Some(job) = m.job.as_mut() {
            job.salary = job.salary.scale(1.0 + life.extension_raise);
        }
    } else {
        m.extension_refused = true;
    }
    Ok(())
}

/// `LeaveVacant`: the position stays free when its holder leaves (PE1).
pub(crate) fn leave_vacant(
    state: &GameState,
    catalog: &Catalog,
    actor: CompanyId,
    position: &Position,
) -> Result<(), CommandError> {
    management::check_own_position(catalog, state, actor, position)
}
