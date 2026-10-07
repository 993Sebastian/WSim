//! The headquarters and the central departments (ZA1–ZA3; docs/BEDIENUNG.md,
//! "Organisation").

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::central;
use crate::game::Game;
use crate::management;

/// A country the headquarters could move to: what it would mean for taxes and salaries.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeatCountryView {
    pub country: String,
    /// Profit tax (share).
    pub tax: f64,
    /// Yearly wage of the salary group of managers (USD): what the board's salaries
    /// follow.
    pub wage_usd: f64,
}

/// A move under way.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RelocationView {
    pub country: String,
    /// The new seat holds from the first month start on or after this day.
    pub until: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CentralView {
    /// The country of the headquarters.
    pub country: String,
    pub tax: f64,
    pub wage_usd: f64,
    pub relocation: Option<RelocationView>,
    /// What a move costs now, how long it takes and the share of employees moving along.
    pub move_cost_usd: f64,
    pub move_months: u32,
    pub moving_share: f64,
    /// All countries to choose from, in the order of the data.
    pub countries: Vec<SeatCountryView>,
}

/// The player's headquarters.
pub fn central(game: &Game) -> CentralView {
    let c = game.catalog();
    let state = game.state();
    let player = game.player();
    let company = &state.companies[player.index()];
    let h = &c.central.headquarters;
    let seat = |country: crate::ids::CountryId| SeatCountryView {
        country: c.countries.key(country).to_owned(),
        tax: state.countries.get(country).corporate_tax,
        wage_usd: management::yearly_wage(c, state, country),
    };
    let here = seat(company.headquarters);
    CentralView {
        country: here.country,
        tax: here.tax,
        wage_usd: here.wage_usd,
        relocation: company.relocation.map(|r| RelocationView {
            country: c.countries.key(r.country).to_owned(),
            until: iso(r.until),
        }),
        move_cost_usd: usd(central::relocation_cost(c, state, player)),
        move_months: h.months,
        moving_share: h.moving_share,
        countries: c.countries.ids().map(seat).collect(),
    }
}
