//! The owners of the AI companies as persons (PV8, docs/FORMELN.md): private wealth,
//! investing by profile, taxes, death and succession. They follow the rules of the player's
//! person: their data stands in `state.person` while the shared functions run, so no rule is
//! written twice.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{AiPersonModel, Catalog};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::services::AssetManager;
use crate::state::{CompanyId, GameState, Holder, Person};

/// Offset of the streams of the AI persons among the streams of the person.
const AI_STREAM: u32 = 9 << 24;

/// The person behind an AI company.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiPerson {
    pub company: CompanyId,
    /// 0 cautious, 1 balanced, 2 risky: the strategy of the asset manager.
    pub profile: u32,
    pub person: Person,
}

pub fn model(catalog: &Catalog) -> Option<&AiPersonModel> {
    catalog.person.ai_persons.as_ref()
}

/// The index of the person of an AI company.
pub fn index_of(state: &GameState, company: CompanyId) -> Option<usize> {
    state.ai_persons.iter().position(|p| p.company == company)
}

/// Runs `f` with the data of an AI person in place of the player's.
pub(crate) fn with_person<R>(
    state: &mut GameState,
    index: usize,
    f: impl FnOnce(&mut GameState) -> R,
) -> R {
    std::mem::swap(&mut state.person, &mut state.ai_persons[index].person);
    state.person_swapped = true;
    let out = f(state);
    state.person_swapped = false;
    std::mem::swap(&mut state.person, &mut state.ai_persons[index].person);
    out
}

fn stream(company: CompanyId, date: Date) -> Stream {
    Stream::Person {
        month: AI_STREAM + company.0 % 4096 * 4096 + crate::management::month_number(date) % 4096,
    }
}

/// A new person of the given age: name, birth, home and private wealth.
fn new_person(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    rng: &mut SimRng,
    (age, wealth_years): (f64, f64),
) -> Person {
    let c = &state.companies[company.index()];
    let mut taken = std::collections::BTreeSet::new();
    let name = crate::management::manager_name(catalog, rng, c.headquarters, &mut taken);
    let born = state.date.add_days(-(age * 365.25) as i32);
    let mut p = Person {
        name,
        born,
        home: c.headquarters,
        ..Person::default()
    };
    let year = crate::management::yearly_wage(catalog, state, c.headquarters);
    p.account.balance = Money::from_usd(year * wealth_years).unwrap_or(Money::ZERO);
    p
}

fn span(m: [f64; 2], rng: &mut SimRng) -> f64 {
    m[0] + (m[1] - m[0]) * rng.next_f64()
}

/// Gives every AI company with private owners its person.
fn create_missing(state: &mut GameState, catalog: &Catalog, m: &AiPersonModel, date: Date) {
    for i in 0..state.companies.len() {
        let id = CompanyId(u32::try_from(i).unwrap_or(u32::MAX));
        let c = &state.companies[i];
        let private = c
            .owners
            .iter()
            .any(|s| s.holder == Holder::Private && s.share > 0.0);
        if c.bankrupt
            || state.main_company == Some(id)
            || !private
            || c.ai.is_none()
            || index_of(state, id).is_some()
        {
            continue;
        }
        let mut rng = SimRng::for_stream(state.settings.seed, stream(id, date));
        let age = span(m.age, &mut rng);
        let wealth = span(m.start_wealth, &mut rng);
        let roll = rng.next_f64();
        let total: f64 = m.profile_weights.iter().sum();
        let mut acc = 0.0;
        let mut profile = 2;
        for (k, w) in m.profile_weights.iter().enumerate() {
            acc += w / total.max(1e-12);
            if roll < acc {
                profile = k as u32;
                break;
            }
        }
        let person = new_person(state, catalog, id, &mut rng, (age, wealth));
        state.ai_persons.push(AiPerson {
            company: id,
            profile,
            person,
        });
    }
}

/// The dividend the owners of an AI company get goes to its person (gross and tax).
pub(crate) fn receive_dividend(state: &mut GameState, index: usize, gross: Money, tax: Money) {
    with_person(state, index, |s| {
        crate::private::receive_dividend(s, gross, tax);
    });
}

/// At a month start: new persons, then for each the same monthly routine as the player's
/// person (holdings, investing, tax and lifestyle, records), then death and succession.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) {
    let Some(m) = model(catalog).cloned() else {
        return;
    };
    let Some(sm) = catalog
        .person
        .assets
        .as_ref()
        .and_then(|a| a.services.clone())
    else {
        return;
    };
    create_missing(state, catalog, &m, date);
    for i in 0..state.ai_persons.len() {
        let company = state.ai_persons[i].company;
        if state.companies[company.index()].bankrupt {
            continue;
        }
        let profile = state.ai_persons[i].profile.min(2);
        let manager = AssetManager {
            strategy: profile,
            budget: m.budget,
            honesty: 100,
        };
        let last = date.add_days(-1);
        with_person(state, i, |s| {
            crate::private::month_start(s, catalog, date);
            crate::savings::holdings_step(s, catalog, date);
            crate::services::invest(s, catalog, &sm, manager);
            let mut ignored = Vec::new();
            crate::private::account_month(s, catalog, last, &mut ignored);
            // Only the last values are kept: the records of a thousand persons would grow.
            let a = &mut s.person.account;
            let keep = a.wealth.len().saturating_sub(2);
            a.wealth.drain(..keep);
            let keep = a.months.len().saturating_sub(2);
            a.months.drain(..keep);
            s.person.history.clear();
        });
        // Death: the table of the managers, the same chance as the player's.
        let mut rng = SimRng::for_stream(state.settings.seed, stream(company, date));
        let dies = with_person(state, i, |s| {
            rng.chance(crate::heirs::death_chance(catalog, s, date))
        });
        if dies {
            succeed(state, catalog, &m, i, &mut rng);
        }
    }
}

/// The person dies: the assets are sold, the inheritance tax is paid, the successor gets
/// the share of the rest that stays with the family.
fn succeed(
    state: &mut GameState,
    catalog: &Catalog,
    m: &AiPersonModel,
    index: usize,
    rng: &mut SimRng,
) {
    let company = state.ai_persons[index].company;
    let estate = with_person(state, index, |s| {
        crate::savings::liquidate(s, catalog, Money::from_usd(1e15).unwrap_or(Money::ZERO));
        let tax = crate::heirs::tax_due(catalog, s);
        let cash = s.person.account.balance;
        let paid = tax.min(cash);
        s.person.account.balance = cash - paid;
        s.person.account.balance
    });
    let age = span(m.successor_age, rng);
    let keep = estate.scale(m.successor_share);
    let mut p = new_person(state, catalog, company, rng, (age, 0.0));
    p.account.balance = keep;
    state.ai_persons[index].person = p;
}

#[cfg(test)]
pub(crate) fn succeed_for_test(
    state: &mut GameState,
    catalog: &Catalog,
    m: &AiPersonModel,
    index: usize,
    rng: &mut SimRng,
) {
    succeed(state, catalog, m, index, rng);
}
