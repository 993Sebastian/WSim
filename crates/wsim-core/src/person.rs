//! The player as a person (docs/FORMELN.md, PE2): name, age, family, roles and the
//! chronicle of the person's life.

use std::collections::BTreeSet;

use crate::aging;
use crate::calendar::{Date, days_in_year};
use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::management;
use crate::math;
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{
    Child, CompanyId, GameState, Holder, LifeEvent, LifeEventKind, Manager, ManagerId, Person,
    Position, Role, Unit,
};

/// The person's age on a day.
pub fn age(state: &GameState, today: Date) -> u32 {
    aging::age(state.person.born, today)
}

/// Whether a company belongs to the person: only those may employ the children.
pub fn is_own(state: &GameState, company: CompanyId) -> bool {
    crate::private::is_controlled(state, company)
}

/// The person's share of a company (0–1).
pub fn share(state: &GameState, company: CompanyId) -> f64 {
    state.companies.get(company.index()).map_or(0.0, |c| {
        c.owners
            .iter()
            .filter(|s| s.holder == Holder::Player)
            .map(|s| s.share)
            .sum()
    })
}

/// `HireManager`, `PoachManager`, `AppointSuccessor`: a child of the person works only
/// for the person's companies.
pub(crate) fn check_family(
    state: &GameState,
    actor: CompanyId,
    manager: &Manager,
) -> Result<(), CommandError> {
    if manager.family && !is_own(state, actor) {
        return Err(CommandError::FamilyOnly);
    }
    Ok(())
}

/// A day of a year, drawn evenly.
pub(crate) fn day_in(rng: &mut SimRng, year: i32) -> Date {
    let days = u64::from(days_in_year(year));
    // Below 366; the cast is exact.
    Date::first_of_year(year).add_days(rng.below(days) as i32)
}

/// A day between two days, drawn evenly; the first where they are the wrong way round.
pub(crate) fn day_between(rng: &mut SimRng, from: Date, to: Date) -> Date {
    let span = from.days_until(to);
    if span <= 0 {
        return from;
    }
    // A span of decades in days; the casts are exact.
    from.add_days(rng.below(span as u64) as i32)
}

/// A first name of the name group of a country.
/// Further draws for a child's first name that is already in the family.
const FIRST_NAME_DRAWS: usize = 5;

pub(crate) fn first_name(
    catalog: &Catalog,
    rng: &mut SimRng,
    home: crate::ids::CountryId,
) -> String {
    let groups = &catalog.name_groups;
    let group = groups
        .iter()
        .filter(|g| !g.first_names.is_empty())
        .find(|g| g.countries.contains(&home))
        .or_else(|| {
            groups
                .iter()
                .find(|g| g.is_default && !g.first_names.is_empty())
        })
        .or_else(|| groups.iter().find(|g| !g.first_names.is_empty()));
    let Some(group) = group else {
        return String::new();
    };
    let n = u64::try_from(group.first_names.len()).unwrap_or(1);
    group.first_names[usize::try_from(rng.below(n)).unwrap_or(0)].clone()
}

/// The person's family name: the last word of the name (or the first where the name
/// group puts the family name first).
pub(crate) fn family_name(catalog: &Catalog, person: &Person) -> String {
    let first_word = catalog
        .name_groups
        .iter()
        .find(|g| g.countries.contains(&person.home))
        .is_some_and(|g| g.surname_first);
    let mut words = person.name.split_whitespace();
    let word = if first_word {
        words.next()
    } else {
        words.next_back()
    };
    word.unwrap_or_default().to_owned()
}

/// A child's full name in the order of the name group.
pub(crate) fn child_name(catalog: &Catalog, rng: &mut SimRng, person: &Person) -> String {
    // A first name nobody in the family has yet, if a few draws find one.
    let taken = |name: &str| {
        std::iter::once(&person.name)
            .chain(person.children.iter().map(|c| &c.name))
            .any(|n| n.split_whitespace().any(|w| w == name))
    };
    let mut first = first_name(catalog, rng, person.home);
    for _ in 0..FIRST_NAME_DRAWS {
        if !taken(&first) {
            break;
        }
        first = first_name(catalog, rng, person.home);
    }
    let family = family_name(catalog, person);
    let surname_first = catalog
        .name_groups
        .iter()
        .find(|g| g.countries.contains(&person.home))
        .is_some_and(|g| g.surname_first);
    match (first.is_empty(), family.is_empty(), surname_first) {
        (true, _, _) => family,
        (_, true, _) => first,
        (false, false, true) => format!("{family} {first}"),
        (false, false, false) => format!("{first} {family}"),
    }
}

/// The person of a new game (docs/FORMELN.md, PE2): as set, else the data's defaults;
/// the children it already has; CEO of its company.
pub(crate) fn start(state: &mut GameState, catalog: &Catalog, today: Date) {
    let model = &catalog.person;
    let settings = state.settings.person.clone();
    let mut rng = SimRng::for_stream(state.settings.seed, Stream::PersonStart);
    let [default_age, min_age, max_age] = model.start_age;
    let year = settings
        .birth_year
        .unwrap_or(today.year() - i32::try_from(default_age).unwrap_or(30))
        .clamp(
            today.year() - i32::try_from(max_age).unwrap_or(60),
            today.year() - i32::try_from(min_age).unwrap_or(18),
        );
    let home = state.settings.start_country;
    let mut taken: BTreeSet<String> = state.managers.values().map(|m| m.name.clone()).collect();
    let name = if settings.name.trim().is_empty() {
        management::manager_name(catalog, &mut rng, home, &mut taken)
    } else {
        settings.name.trim().to_owned()
    };
    let born = day_in(&mut rng, year).min(today);
    let player = state.main_company;
    let mut person = Person {
        name,
        born,
        home,
        married: settings.married,
        children: Vec::new(),
        ceo: player,
        background: settings.background,
        history: player
            .map(|company| LifeEvent {
                date: today,
                kind: LifeEventKind::Start { company },
            })
            .into_iter()
            .collect(),
        ..Person::default()
    };
    let [from, until] = model.children_ages;
    let first = born.add_months(from * 12).min(today);
    let last = born.add_months(until * 12).min(today);
    let most = u8::try_from(model.children_max).unwrap_or(u8::MAX);
    let mut births: Vec<Date> = (0..settings.children.min(most))
        .map(|_| day_between(&mut rng, first, last))
        .collect();
    births.sort();
    for b in births {
        let name = child_name(catalog, &mut rng, &person);
        person.history.push(LifeEvent {
            date: b,
            kind: LifeEventKind::ChildBorn { name: name.clone() },
        });
        person.children.push(new_child(catalog, &mut rng, name, b));
    }
    person.history.sort_by_key(|e| e.date);
    state.person = person;
    careers(state, catalog, today, &mut Vec::new());
}

/// Older saves get a standard person (docs/FORMELN.md, PE2): 30 years old, married,
/// without children, CEO of the company.
pub(crate) fn fit_loaded(state: &mut GameState, catalog: &Catalog) {
    if !state.person.name.is_empty() {
        return;
    }
    let today = state.date;
    let mut rng = SimRng::for_stream(state.settings.seed, Stream::PersonStart);
    let player = state.main_company;
    let home = player
        .and_then(|p| state.companies.get(p.index()))
        .map_or(state.settings.start_country, |c| c.headquarters);
    let mut taken: BTreeSet<String> = state.managers.values().map(|m| m.name.clone()).collect();
    let led = player.filter(|&p| {
        management::holder(state, p, &Position::new(Unit::Board, Role::Head)).is_none()
    });
    state.person = Person {
        name: management::manager_name(catalog, &mut rng, home, &mut taken),
        born: Date::first_of_year(today.year() - 30),
        home,
        married: true,
        children: Vec::new(),
        ceo: led,
        history: player
            .map(|company| LifeEvent {
                date: today,
                kind: LifeEventKind::Takeover { company },
            })
            .into_iter()
            .collect(),
        ..Person::default()
    };
}

/// At a month start (docs/FORMELN.md, PE2): perhaps a child, manager cards for children
/// of age, and who leads the company.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    if state.person.name.is_empty() {
        return news;
    }
    births(state, catalog, date, &mut news);
    legacy_month(state, catalog, date);
    careers(state, catalog, date, &mut news);
    leadership(state, date, &mut news);
    news
}

/// A child comes with a monthly chance while the person is married and of age for it.
fn births(state: &mut GameState, catalog: &Catalog, date: Date, news: &mut Vec<Message>) {
    let model = &catalog.person;
    let person = &state.person;
    let age = aging::age(person.born, date);
    let [from, until] = model.children_ages;
    let count = u32::try_from(person.children.len()).unwrap_or(u32::MAX);
    if !model.enabled
        || !person.married
        || age < from
        || age >= until
        || count >= model.children_max
    {
        return;
    }
    let mut rng = SimRng::for_stream(
        state.settings.seed,
        Stream::Person {
            month: management::month_number(date),
        },
    );
    let chance = 1.0 - math::pow(1.0 - model.child_chance.clamp(0.0, 1.0), 1.0 / 12.0);
    if !rng.chance(chance) {
        return;
    }
    let name = child_name(catalog, &mut rng, &state.person);
    let person = &mut state.person;
    person
        .children
        .push(new_child(catalog, &mut rng, name.clone(), date));
    person.history.push(LifeEvent {
        date,
        kind: LifeEventKind::ChildBorn { name: name.clone() },
    });
    news.push(
        Message::new(MessageKind::Info, keys::PERSON_CHILD)
            .with("name", Param::Text(name))
            .with(
                "anzahl",
                Param::Integer(i64::try_from(person.children.len()).unwrap_or(0)),
            ),
    );
}

/// A new child with its traits drawn: ambition, interest in the firm, a talent for a branch
/// (PV4; without the data section the traits stay 0).
pub(crate) fn new_child(catalog: &Catalog, rng: &mut SimRng, name: String, born: Date) -> Child {
    let mut child = Child {
        name,
        born,
        ..Child::default()
    };
    if let Some(l) = &catalog.person.legacy {
        let draw = |rng: &mut SimRng| -> u32 {
            // Within 0–100; the cast is exact.
            (l.traits[0] + (l.traits[1] - l.traits[0]) * rng.next_f64()).round() as u32
        };
        child.ambition = draw(rng);
        child.interest = draw(rng);
        let n = catalog.branches.keys().len();
        if n > 0 {
            let pick = usize::try_from(rng.below(n as u64)).unwrap_or(0);
            child.talent = Some(<crate::ids::BranchId as crate::ids::Id>::from_index(pick));
        }
    }
    child
}

/// `SetEducation`: from the age of education until the manager card.
pub(crate) fn set_education(
    state: &mut GameState,
    catalog: &Catalog,
    child: u32,
    education: crate::state::Education,
) -> Result<(), crate::command::CommandError> {
    let Some(l) = &catalog.person.legacy else {
        return Err(crate::command::CommandError::NotPossible);
    };
    let today = state.date;
    let c = state
        .person
        .children
        .get_mut(child as usize)
        .ok_or(crate::command::CommandError::UnknownChild)?;
    let age = aging::age(c.born, today);
    if c.died.is_some() || c.manager.is_some() || age < l.education_from {
        return Err(crate::command::CommandError::NotPossible);
    }
    c.education = Some(education);
    Ok(())
}

/// Each month: the interest of the children in the firm follows the family time; an education
/// that was chosen is paid (PV4).
fn legacy_month(state: &mut GameState, catalog: &Catalog, date: Date) {
    let Some(l) = &catalog.person.legacy else {
        return;
    };
    let family =
        crate::wellbeing::time(catalog, state).map_or(l.interest_base, |t| f64::from(t.family));
    let delta = l.interest_per_month * (family - l.interest_base) / 100.0;
    let wage = crate::private::academic_monthly_wage(catalog, state, state.person.home);
    let cost = wage.scale(l.education_cost);
    let card_age = catalog.person.card_age;
    for i in 0..state.person.children.len() {
        let c = &mut state.person.children[i];
        if c.died.is_some() {
            continue;
        }
        let v = f64::from(c.interest) + delta;
        // Within 0–100; the cast is exact.
        c.interest = v.round().clamp(0.0, 100.0) as u32;
        let age = aging::age(c.born, date);
        if c.education.is_some()
            && c.manager.is_none()
            && age >= l.education_from
            && age < card_age
            && state.person.account.balance >= cost
            && cost > Money::ZERO
        {
            crate::private::book_flow(state, crate::state::PrivateFlow::Education, -cost);
            state.person.children[i].education_months += 1;
        }
    }
}

/// Points of a private school: its points for a whole childhood, by the share of the
/// childhood spent there.
fn school_bonus(catalog: &Catalog, child: &Child, card: Date) -> f64 {
    let days = child.born.days_until(card);
    if days <= 0 {
        return 0.0;
    }
    catalog.person.private_life.school_education * f64::from(child.school_days) / f64::from(days)
}

/// Children of the card age get a manager card like a candidate of the market.
fn careers(state: &mut GameState, catalog: &Catalog, date: Date, news: &mut Vec<Message>) {
    let card_age = catalog.person.card_age;
    let due: Vec<usize> = state
        .person
        .children
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            c.manager.is_none() && c.died.is_none() && aging::age(c.born, date) >= card_age
        })
        .map(|(i, _)| i)
        .collect();
    if due.is_empty() || !catalog.management.enabled() {
        return;
    }
    let mut names: BTreeSet<String> = state.managers.values().map(|m| m.name.clone()).collect();
    for i in due {
        let child = state.person.children[i].clone();
        let id = ManagerId(state.next_manager);
        state.next_manager += 1;
        let mut rng = SimRng::for_stream(state.settings.seed, Stream::ManagerBirth { id: id.0 });
        let mut m = management::draw(catalog, &mut rng, state.person.home, &mut names);
        management::assign_branches(catalog, state.settings.seed, id, &mut m);
        management::assign_style(catalog, state.settings.seed, id, &mut m);
        // Education by the lifestyle of the childhood (PE3).
        let bonus = crate::private::education(catalog, &state.person, child.born, date)
            + school_bonus(catalog, &child, date);
        let up = |v: u8| {
            // Within 0–100; the cast is exact.
            (f64::from(v) + bonus).round().clamp(0.0, 100.0) as u8
        };
        for v in m.expertise.values_mut() {
            *v = up(*v);
        }
        m.detection = up(m.detection);
        m.judgment = up(m.judgment);
        m.leadership = up(m.leadership);
        apply_education(catalog, &child, &mut m, &mut rng);
        m.name.clone_from(&child.name);
        m.born = Some(child.born);
        m.family = true;
        state.managers.insert(id, m);
        state.person.children[i].manager = Some(id);
        state.person.history.push(LifeEvent {
            date,
            kind: LifeEventKind::Career {
                name: child.name.clone(),
            },
        });
        news.push(
            Message::new(MessageKind::Info, keys::PERSON_CAREER)
                .with("name", Param::Text(child.name))
                .with(
                    "alter",
                    Param::Integer(i64::from(aging::age(child.born, date))),
                ),
        );
    }
    crate::staffing::set_potentials(state, catalog);
}

/// The education, ambition and talent of a child raise the card's skills (PV4).
fn apply_education(
    catalog: &Catalog,
    child: &Child,
    m: &mut crate::state::Manager,
    rng: &mut SimRng,
) {
    let Some(l) = &catalog.person.legacy else {
        return;
    };
    let months = f64::from((catalog.person.card_age.saturating_sub(l.education_from)).max(1) * 12);
    let done = (f64::from(child.education_months) / months).min(1.0);
    let functions: Vec<usize> = match child.education {
        None => Vec::new(),
        Some(crate::state::Education::Engineer) => l.educations[0].clone(),
        Some(crate::state::Education::Merchant) => l.educations[1].clone(),
        Some(crate::state::Education::Lawyer) => l.educations[2].clone(),
        Some(crate::state::Education::Scientist) => l.educations[3].clone(),
        Some(crate::state::Education::Free) => {
            let n = catalog.management.functions.len();
            if n == 0 {
                Vec::new()
            } else {
                (0..2)
                    .map(|_| usize::try_from(rng.below(n as u64)).unwrap_or(0))
                    .collect()
            }
        }
    };
    let up = |v: u8, points: f64| -> u8 {
        // Within 0–100; the cast is exact.
        (f64::from(v) + points).round().clamp(0.0, 100.0) as u8
    };
    for f in functions {
        let key = &catalog.management.functions[f].key;
        if let Some(v) = m.expertise.get_mut(key) {
            *v = up(*v, l.education_points * done);
        }
    }
    m.leadership = up(
        m.leadership,
        l.ambition_points * f64::from(child.ambition) / 100.0,
    );
    if let Some(b) = child.talent {
        let v = m.branch.entry(b).or_insert(0);
        *v = up(*v, l.talent_points);
    }
}

/// The person hands the lead of the company to a manager who became CEO, and takes it
/// back when the position is free.
fn leadership(state: &mut GameState, date: Date, news: &mut Vec<Message>) {
    let Some(company) = state.main_company else {
        return;
    };
    if state.companies[company.index()].bankrupt {
        return;
    }
    let ceo = management::holder(state, company, &Position::new(Unit::Board, Role::Head));
    let leads = state.person.ceo == Some(company);
    let name = state.companies[company.index()].name.clone();
    match (ceo, leads) {
        (Some(id), true) => {
            let to = state.managers[&id].name.clone();
            state.person.ceo = None;
            // The CEO's pay ends; as owner the person sets a new one if it wants one.
            state.person.salaries.remove(&company);
            state.person.history.push(LifeEvent {
                date,
                kind: LifeEventKind::CeoHandedOver {
                    company,
                    to: to.clone(),
                },
            });
            news.push(
                Message::new(MessageKind::Info, keys::PERSON_CEO_HANDED_OVER)
                    .with("name", Param::Text(to))
                    .with("firma", Param::Text(name)),
            );
        }
        (None, false) if state.person.ceo.is_none() => {
            state.person.ceo = Some(company);
            state.person.history.push(LifeEvent {
                date,
                kind: LifeEventKind::CeoTakenBack { company },
            });
            news.push(
                Message::new(MessageKind::Info, keys::PERSON_CEO_TAKEN_BACK)
                    .with("firma", Param::Text(name)),
            );
        }
        _ => {}
    }
}

/// A child who was a manager died (PE1): the child is dead.
pub(crate) fn manager_died(
    state: &mut GameState,
    id: ManagerId,
    date: Date,
    news: &mut Vec<Message>,
) {
    let Some(child) = state
        .person
        .children
        .iter_mut()
        .find(|c| c.manager == Some(id))
    else {
        return;
    };
    child.died = Some(date);
    child.manager = None;
    let name = child.name.clone();
    let years = aging::age(child.born, date);
    state.person.history.push(LifeEvent {
        date,
        kind: LifeEventKind::ChildDied { name: name.clone() },
    });
    news.push(
        Message::new(MessageKind::Warning, keys::PERSON_CHILD_DIED)
            .with("name", Param::Text(name))
            .with("alter", Param::Integer(i64::from(years))),
    );
}

/// A child's manager card ended with the retirement (PE1): the child lives on without.
pub(crate) fn manager_retired(state: &mut GameState, id: ManagerId) {
    if let Some(child) = state
        .person
        .children
        .iter_mut()
        .find(|c| c.manager == Some(id))
    {
        child.manager = None;
    }
}
