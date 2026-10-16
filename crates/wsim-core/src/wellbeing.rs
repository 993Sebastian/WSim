//! Background, time budget and health of the person (PV1b, docs/FORMELN.md).

use crate::calendar::Date;
use crate::catalog::{BackgroundEffect, Catalog, WellbeingModel};
use crate::command::CommandError;
use crate::message::{Message, MessageKind, Param, keys};
use crate::rng::{SimRng, Stream};
use crate::state::{GameState, TimeBudget};

/// Offset of the stream of the illness among the streams of the person.
const ILL_STREAM: u32 = 2 << 24;

fn model(catalog: &Catalog) -> Option<&WellbeingModel> {
    catalog.person.wellbeing.as_ref()
}

/// The shares of the time: as set, else the default of the data.
pub fn time(catalog: &Catalog, state: &GameState) -> Option<TimeBudget> {
    let m = model(catalog)?;
    Some(state.person.time.unwrap_or(TimeBudget {
        firm: m.time_default[0],
        family: m.time_default[1],
        network: m.time_default[2],
        rest: m.time_default[3],
    }))
}

/// Number of companies the person leads.
fn companies_led(state: &GameState) -> usize {
    (0..state.companies.len())
        .filter(|&i| {
            crate::private::is_controlled(
                state,
                crate::state::CompanyId(u32::try_from(i).unwrap_or(u32::MAX)),
            )
        })
        .count()
}

/// The least share of the time for the firm, in percent.
pub fn min_firm(catalog: &Catalog, state: &GameState) -> f64 {
    model(catalog).map_or(0.0, |m| {
        (m.min_firm[0] + m.min_firm[1] * companies_led(state) as f64).min(m.min_firm[2])
    })
}

/// Whether the person is absent through illness on a day.
pub fn is_ill(state: &GameState, today: Date) -> bool {
    state.person.ill_until.is_some_and(|d| today < d)
}

/// The health 0–100; the start value before the first month.
pub fn health(catalog: &Catalog, state: &GameState) -> Option<f64> {
    let m = model(catalog)?;
    Some(state.person.health.unwrap_or(m.health_start))
}

/// The effects of the background now: proportional ones grow with the experience
/// and stay under the cap.
pub fn effect(catalog: &Catalog, state: &GameState) -> BackgroundEffect {
    let (Some(m), Some(b)) = (model(catalog), state.person.background) else {
        return BackgroundEffect::default();
    };
    let base = m.backgrounds[b as usize];
    let grown = 0.5 + 0.5 * state.person.experience.clamp(0.0, 1.0);
    BackgroundEffect {
        research: (base.research * grown).min(m.cap),
        loyalty: (base.loyalty * grown).min(m.cap),
        ..base
    }
}

/// Factor on the pace of research and projects of a company the person leads.
pub fn research_factor(
    catalog: &Catalog,
    state: &GameState,
    company: crate::state::CompanyId,
) -> f64 {
    if crate::private::is_controlled(state, company) {
        1.0 + effect(catalog, state).research
    } else {
        1.0
    }
}

/// Points on the target satisfaction of the managers of a company the person leads.
pub fn loyalty_points(
    catalog: &Catalog,
    state: &GameState,
    company: crate::state::CompanyId,
) -> f64 {
    if crate::private::is_controlled(state, company) {
        100.0 * effect(catalog, state).loyalty
    } else {
        0.0
    }
}

/// Factor on the chance to die by health.
pub fn mortality_factor(catalog: &Catalog, state: &GameState) -> f64 {
    let (Some(m), Some(g)) = (model(catalog), health(catalog, state)) else {
        return 1.0;
    };
    if g >= m.healthy_from {
        1.0
    } else {
        1.0 + (m.mortality_at_zero - 1.0) * (m.healthy_from - g) / m.healthy_from
    }
}

/// Days the concerns wait less because the firm gets too little time.
pub fn delay_days(catalog: &Catalog, state: &GameState) -> i32 {
    let (Some(m), Some(t)) = (model(catalog), time(catalog, state)) else {
        return 0;
    };
    let least = min_firm(catalog, state);
    if least <= 0.0 || f64::from(t.firm) >= least {
        return 0;
    }
    // Whole days; the product is small.
    #[allow(clippy::cast_possible_truncation)]
    {
        (m.delay_days * (least - f64::from(t.firm)) / least).round() as i32
    }
}

/// `SetTimeBudget`.
pub(crate) fn set_time(
    state: &mut GameState,
    catalog: &Catalog,
    budget: TimeBudget,
) -> Result<(), CommandError> {
    if model(catalog).is_none() {
        return Err(CommandError::NotPossible);
    }
    if budget.total() != 100 {
        return Err(CommandError::TimeBudgetSum);
    }
    state.person.time = Some(budget);
    Ok(())
}

/// At a month start: health from age, overload, rest, lifestyle and care; experience;
/// perhaps an illness, or the end of one.
pub(crate) fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let Some(m) = model(catalog) else {
        return news;
    };
    if state.person.name.is_empty() || state.game_over {
        return news;
    }
    if state.person.ill_until.is_some_and(|d| d <= date) {
        state.person.ill_until = None;
        news.push(Message::new(MessageKind::Info, keys::PERSON_WELL_AGAIN));
    }
    let led = companies_led(state);
    if led > 0 {
        state.person.experience = (state.person.experience + m.experience_per_year / 12.0).min(1.0);
    }
    let t = time(catalog, state).unwrap_or(TimeBudget {
        firm: 0,
        family: 0,
        network: 0,
        rest: 0,
    });
    let ill = is_ill(state, date);
    let firm = if ill { 0.0 } else { f64::from(t.firm) };
    let age = f64::from(crate::person::age(state, date));
    let level = crate::private::lifestyle_at(catalog, &state.person, date);
    let aging = m.aging_per_year * (age - f64::from(m.aging_from)).max(0.0);
    let overload =
        m.overload_loss * (firm - m.overload_from).max(0.0) / (100.0 - m.overload_from).max(1.0);
    let rest = if ill {
        m.rest_gain
    } else {
        m.rest_gain * f64::from(t.rest) / 100.0
    };
    let care = if state.person.health_care {
        m.care_gain
    } else {
        0.0
    };
    let before = state.person.health.unwrap_or(m.health_start);
    let after = (before - aging - overload + rest + m.lifestyle_gain[level as usize] + care)
        .clamp(0.0, 100.0);
    state.person.health = Some(after);
    if !ill && after < m.ill_below && m.ill_below > 0.0 {
        let mut rng = SimRng::for_stream(
            state.settings.seed,
            Stream::Person {
                month: crate::management::month_number(date) + ILL_STREAM,
            },
        );
        let chance = m.ill_chance * (m.ill_below - after) / m.ill_below;
        if rng.chance(chance) {
            state.person.ill_until = Some(date.add_months(m.ill_months));
            news.push(
                Message::new(MessageKind::Warning, keys::PERSON_ILL)
                    .with("monate", Param::Integer(i64::from(m.ill_months))),
            );
        }
    }
    news
}
