//! Maintenance level of the sites, breakdowns of worn facilities and overhauls
//! (docs/FORMELN.md „BP2“).

use crate::calendar::Date;
use crate::catalog::{Catalog, SiteType};
use crate::command::{self, Command, CommandError};
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, Downtime, GameState, SiteId, Slot};

/// Capacity factor of a facility by its condition.
pub fn capacity_factor(catalog: &Catalog, condition: f64) -> f64 {
    let m = &catalog.production_model.maintenance;
    if !m.enabled {
        return 1.0;
    }
    1.0 - m.capacity_weight * (1.0 - condition.clamp(0.0, 1.0))
}

/// Factor on the daily wear for a maintenance level.
pub fn wear_factor(catalog: &Catalog, level: f64) -> f64 {
    let m = &catalog.production_model.maintenance;
    if !m.enabled || level <= 0.0 {
        return 1.0;
    }
    libm::pow(1.0 / level, m.exponent)
}

/// Daily breakdown chance of a facility.
pub fn failure_chance(catalog: &Catalog, condition: f64) -> f64 {
    let m = &catalog.production_model.maintenance;
    let floor = catalog.production_model.condition_min;
    if !m.enabled || floor >= 1.0 {
        return 0.0;
    }
    let worn = ((1.0 - condition) / (1.0 - floor)).clamp(0.0, 1.0);
    m.failure_chance * worn * worn
}

/// Whether the facility stands still on `date` (breakdown or overhaul).
pub fn down_on(slot: &Slot, date: Date) -> Option<Downtime> {
    slot.down.filter(|d| d.until > date)
}

/// Sets the maintenance level of a site (`SetMaintenance`).
pub(crate) fn set_level(
    state: &mut GameState,
    catalog: &Catalog,
    site: SiteId,
    level: f64,
) -> Result<(), CommandError> {
    let m = &catalog.production_model.maintenance;
    if !m.enabled {
        return Err(CommandError::NotPossible);
    }
    if !(level.is_finite() && (m.level_min - 1e-9..=m.level_max + 1e-9).contains(&level)) {
        return Err(CommandError::MaintenanceOutOfRange {
            min: m.level_min,
            max: m.level_max,
        });
    }
    state.sites[site.index()].maintenance = level;
    Ok(())
}

/// Cost of overhauling a facility.
pub fn overhaul_cost(catalog: &Catalog, slot: &Slot) -> Money {
    slot.investment(catalog)
        .scale(catalog.production_model.maintenance.overhaul_cost_share)
}

/// Overhauls a facility (`Overhaul`): paid from the cash, stands for the overhaul days,
/// then as new.
pub(crate) fn overhaul(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    site: SiteId,
    slot: usize,
) -> Result<(), CommandError> {
    let m = &catalog.production_model.maintenance;
    let date = state.date;
    let sl = &state.sites[site.index()].slots[slot];
    if !m.enabled || sl.ready > date || sl.mothballed() || down_on(sl, date).is_some() {
        return Err(CommandError::NotPossible);
    }
    let cost = overhaul_cost(catalog, sl);
    let ledger = &mut state.companies[actor.index()].ledger;
    if ledger.cash() < cost {
        return Err(CommandError::NotEnoughCash { needed: cost });
    }
    ledger.expense(
        CostType::Maintenance,
        CostCenter::site(site),
        Account::Cash,
        cost,
    );
    let sl = &mut state.sites[site.index()].slots[slot];
    sl.condition = 1.0;
    sl.down = Some(Downtime {
        until: date.add_days(i32::try_from(m.overhaul_days).unwrap_or(i32::MAX)),
        overhaul: true,
    });
    state.sites[site.index()].staffing_due = true;
    Ok(())
}

/// Breakdowns of the day: every running facility fails with its chance; the repair is
/// paid and the facility stands a few days.
pub(crate) fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let m = &catalog.production_model.maintenance;
    let mut news = Vec::new();
    if !m.enabled || m.failure_chance <= 0.0 {
        return news;
    }
    let day = u32::try_from(Date::first_of_year(1800).days_until(date)).unwrap_or(0);
    for index in 0..state.sites.len() {
        let s = &state.sites[index];
        if s.closed.is_some() || s.kind == SiteType::ResearchCenter {
            continue;
        }
        let chances: Vec<f64> = s
            .slots
            .iter()
            .map(|sl| {
                if sl.operating(date) && down_on(sl, date).is_none() {
                    failure_chance(catalog, sl.condition)
                } else {
                    0.0
                }
            })
            .collect();
        if chances.iter().all(|&c| c <= 0.0) {
            continue;
        }
        let site = SiteId(u32::try_from(index).unwrap_or(u32::MAX));
        let mut rng = SimRng::for_stream(
            state.settings.seed,
            Stream::Operations { site: site.0, day },
        );
        for (slot, chance) in chances.into_iter().enumerate() {
            if chance <= 0.0 || rng.next_f64() >= chance {
                continue;
            }
            let (lo, hi) = m.failure_days;
            let span = f64::from(hi.saturating_sub(lo) + 1);
            let days = lo + (rng.next_f64() * span).floor().min(span - 1.0) as u32;
            let s = &mut state.sites[index];
            let cost = s.slots[slot]
                .investment(catalog)
                .scale(m.failure_cost_share);
            s.slots[slot].down = Some(Downtime {
                until: date.add_days(i32::try_from(days).unwrap_or(0)),
                overhaul: false,
            });
            s.failures += 1;
            let (owner, country, failures, facility) =
                (s.owner, s.country, s.failures, s.slots[slot].facility);
            state.companies[owner.index()].ledger.expense(
                CostType::Maintenance,
                CostCenter::site(site),
                Account::Cash,
                cost,
            );
            if let Some(company) = state.news_of(owner) {
                let key = if failures > m.failures_warning {
                    keys::BREAKDOWN_OFTEN
                } else {
                    keys::BREAKDOWN
                };
                news.push(
                    Message::new(MessageKind::Warning, key)
                        .with(
                            "anlage",
                            Param::TextKey(format!("anlage.{}", catalog.facilities.key(facility))),
                        )
                        .with(
                            "land",
                            Param::Country(catalog.countries.key(country).to_owned()),
                        )
                        .with("tage", Param::Integer(i64::from(days)))
                        .with("kosten", Param::Money(cost))
                        .with("anzahl", Param::Integer(i64::from(failures)))
                        .of(company),
                );
            }
        }
    }
    news
}

/// On 1 January: the yearly count of breakdowns starts anew.
pub(crate) fn new_year(state: &mut GameState) {
    for s in &mut state.sites {
        s.failures = 0;
    }
}

/// AI companies at a month start: the maintenance level by skill, and an overhaul of
/// worn facilities when the cash bears it twice over – the player's commands.
pub(crate) fn ai_month(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    own: &[SiteId],
) {
    let m = &catalog.production_model.maintenance;
    let Some(ai) = &state.companies[company.index()].ai else {
        return;
    };
    if !m.enabled {
        return;
    }
    let wanted = ((0.8 + 0.5 * ai.skill()).clamp(m.level_min, m.level_max) * 20.0).round() / 20.0;
    for &site in own {
        let Some(s) = state.site(site) else { continue };
        if (s.maintenance - wanted).abs() > 1e-9 {
            let _ = command::execute(
                state,
                catalog,
                company,
                &Command::SetMaintenance {
                    site,
                    level: wanted,
                },
            );
        }
        let worn: Vec<usize> = state.sites[site.index()]
            .slots
            .iter()
            .enumerate()
            .filter(|(_, sl)| sl.condition < m.ai_overhaul_below && sl.recipe.is_some())
            .map(|(i, _)| i)
            .collect();
        for slot in worn {
            let cost = overhaul_cost(catalog, &state.sites[site.index()].slots[slot]);
            if state.companies[company.index()].ledger.cash() >= cost.scale(2.0) {
                let _ =
                    command::execute(state, catalog, company, &Command::Overhaul { site, slot });
            }
        }
    }
}
