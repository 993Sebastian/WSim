//! Family holding and foundation of the person (PV6c, docs/FORMELN.md).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::catalog::{Catalog, TrustModel};
use crate::command::CommandError;
use crate::message::Message;
use crate::money::Money;
use crate::savings::Investment;
use crate::state::{CompanyId, GameState, PrivateFlow};

/// Holding and foundation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Trust {
    /// The holding exists: shares of companies stay together with the successor.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub holding: bool,
    /// Per company the share of the person's dividends that goes on to the co-heirs.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub claims: BTreeMap<CompanyId, f64>,
    /// The capital of the foundation; it cannot be taken out again.
    #[serde(default, skip_serializing_if = "Money::is_zero")]
    pub capital: Money,
}

impl Trust {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

pub fn model(catalog: &Catalog) -> Option<&TrustModel> {
    catalog.person.trust.as_ref()
}

/// The share of the person's dividends of a company that goes to the co-heirs.
pub fn claim(state: &GameState, company: CompanyId) -> f64 {
    state
        .person
        .trust
        .claims
        .get(&company)
        .copied()
        .unwrap_or(0.0)
}

/// The foundation's capital in academic yearly wages of the home country.
fn wages(catalog: &Catalog, state: &GameState, amount: Money) -> f64 {
    let year = crate::private::academic_monthly_wage(catalog, state, state.person.home)
        .scale(12.0)
        .to_usd()
        .max(1.0);
    amount.to_usd() / year
}

/// Points the foundation adds to the standing the person drifts to.
pub fn standing_points(catalog: &Catalog, state: &GameState) -> f64 {
    model(catalog).map_or(0.0, |m| {
        (m.standing_per_wage * wages(catalog, state, state.person.trust.capital))
            .min(m.standing_cap)
    })
}

/// Yearly payout to the family now.
pub fn payout_year(catalog: &Catalog, state: &GameState) -> Money {
    model(catalog).map_or(Money::ZERO, |m| {
        state.person.trust.capital.scale(m.payout_year)
    })
}

/// An action on holding and foundation.
pub(crate) fn apply(
    state: &mut GameState,
    catalog: &Catalog,
    action: Investment,
) -> Result<(), CommandError> {
    let Some(m) = model(catalog).cloned() else {
        return Err(CommandError::NotPossible);
    };
    match action {
        Investment::FoundHolding => {
            if state.person.trust.holding {
                return Err(CommandError::NotPossible);
            }
            let cost = crate::private::academic_monthly_wage(catalog, state, state.person.home)
                .scale(m.holding_cost);
            pay(state, cost)?;
            state.person.trust.holding = true;
        }
        Investment::EndowFoundation { amount } => {
            if amount <= Money::ZERO || wages(catalog, state, amount) < m.foundation_min {
                return Err(CommandError::NotPossible);
            }
            let tax = amount.scale(m.gift_tax);
            pay(state, amount + tax)?;
            state.person.trust.capital += amount;
        }
        _ => return Err(CommandError::NotPossible),
    }
    Ok(())
}

fn pay(state: &mut GameState, cost: Money) -> Result<(), CommandError> {
    if cost <= Money::ZERO {
        return Ok(());
    }
    if state.person.account.balance < cost {
        return Err(CommandError::NotEnoughPrivateMoney {
            needed: cost - state.person.account.balance,
        });
    }
    crate::private::book_flow(state, PrivateFlow::Trust, -cost);
    Ok(())
}

/// At a month start: the capital of the foundation yields, the family gets its payout.
pub(crate) fn month_start(
    state: &mut GameState,
    catalog: &Catalog,
    _date: crate::calendar::Date,
) -> Vec<Message> {
    let Some(m) = model(catalog) else {
        return Vec::new();
    };
    let capital = state.person.trust.capital;
    if capital <= Money::ZERO {
        return Vec::new();
    }
    let grown = capital + capital.scale(m.yield_year / 12.0);
    let payout = capital.scale(m.payout_year / 12.0).min(grown);
    state.person.trust.capital = grown - payout;
    if payout > Money::ZERO {
        crate::private::book_flow(state, PrivateFlow::FoundationPayout, payout);
    }
    Vec::new()
}
