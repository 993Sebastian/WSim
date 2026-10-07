//! Headquarters and central departments (ZA1–ZA3, docs/FORMELN.md, docs/BETEILIGUNGEN.md).

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::command::CommandError;
use crate::ids::{CountryId, Id};
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::state::{CompanyId, GameState, Relocation};

/// Employees of a company's central departments.
pub fn employees(state: &GameState, company: CompanyId) -> u32 {
    let _ = (state, company);
    0
}

/// What moving the headquarters costs now (docs/FORMELN.md, ZA1).
pub fn relocation_cost(catalog: &Catalog, state: &GameState, company: CompanyId) -> Money {
    let h = &catalog.central.headquarters;
    h.cost_base
        + h.cost_per_employee
            .scale(f64::from(employees(state, company)))
}

/// `SetHeadquarters`: the move to another country starts; it costs at once and is done
/// after the months of the data.
pub(crate) fn set_headquarters(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    country: CountryId,
) -> Result<(), CommandError> {
    if country.index() >= catalog.countries.len() {
        return Err(CommandError::UnknownCountry);
    }
    let c = &state.companies[actor.index()];
    if let Some(r) = c.relocation {
        return Err(CommandError::RelocationUnderWay { until: r.until });
    }
    if c.headquarters == country {
        return Err(CommandError::SameHeadquarters);
    }
    let cost = relocation_cost(catalog, state, actor);
    if c.ledger.cash() < cost {
        return Err(CommandError::NotEnoughCash { needed: cost });
    }
    let until = state.date.add_months(catalog.central.headquarters.months);
    let c = &mut state.companies[actor.index()];
    if cost > Money::ZERO {
        c.ledger
            .expense(CostType::Other, CostCenter::default(), Account::Cash, cost);
    }
    c.relocation = Some(Relocation { country, until });
    Ok(())
}

/// At a month start (docs/FORMELN.md, ZA1): moves that are due are done. Returns the news
/// for the player.
pub fn month_start(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let mut news = Vec::new();
    let player = state.player;
    for (i, c) in state.companies.iter_mut().enumerate() {
        let Some(r) = c.relocation.filter(|r| r.until <= date) else {
            continue;
        };
        c.relocation = None;
        if c.bankrupt {
            continue;
        }
        c.headquarters = r.country;
        // Few companies; the cast is exact.
        if CompanyId(i as u32) == player {
            news.push(
                Message::new(MessageKind::Info, keys::HEADQUARTERS_MOVED).with(
                    "land",
                    Param::Country(catalog.countries.key(r.country).to_owned()),
                ),
            );
        }
    }
    news
}
