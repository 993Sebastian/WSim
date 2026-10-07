//! Simulation core of WSim.
//!
//! The core has no dependency on the UI, Tauri or file IO. It receives a compiled
//! catalog from `wsim-data`, accepts commands and advances the world in daily ticks.

pub mod ai;
pub mod brand;
pub mod calendar;
pub mod catalog;
pub mod command;
pub mod competition;
pub mod country_model;
pub mod currency;
pub mod deals;
pub mod decision;
pub mod development;
pub mod finance;
pub mod game;
pub mod health;
pub mod ids;
pub mod ledger;
pub mod management;
pub mod mandate;
pub mod market;
pub mod math;
pub mod message;
pub mod milestones;
pub mod money;
pub mod plots;
pub mod policy;
pub mod population;
pub mod product_names;
pub mod production;
pub mod ranking;
pub mod reports;
pub mod research;
pub mod review;
pub mod rng;
pub mod save;
pub mod staffing;
pub mod state;
pub mod strategy;
pub mod time_series;
pub mod trade;
pub mod transport;
pub mod views;

#[cfg(test)]
mod board_tests;
#[cfg(test)]
mod deals_tests;
#[cfg(test)]
mod determinism_tests;
#[cfg(test)]
mod development_tests;
#[cfg(test)]
mod facility_tests;
#[cfg(test)]
mod finance_tests;
#[cfg(test)]
mod management_tests;
#[cfg(test)]
mod market_tests;
#[cfg(test)]
mod plots_tests;
#[cfg(test)]
mod product_names_tests;
#[cfg(test)]
mod production_tests;
#[cfg(test)]
mod research_tests;
#[cfg(test)]
mod size_tests;
#[cfg(test)]
mod staffing_tests;
#[cfg(test)]
mod strategy_tests;
#[cfg(test)]
mod trade_tests;

pub use calendar::{Date, RoundLength};
pub use game::{Game, JournalEntry, RoundReport, StateHash};

use serde::Serialize;

/// Earliest selectable start year (Lastenheft §3.1). Technologies invented up to this
/// year are known to everyone at the start.
pub const EARLIEST_START_YEAR: i32 = 1900;

/// Version of the simulation core.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Basic information about the core, shown by the UI and the CLI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CoreInfo {
    pub version: &'static str,
}

/// Returns basic information about this build of the core.
pub fn core_info() -> CoreInfo {
    CoreInfo { version: VERSION }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_info_reports_crate_version() {
        assert_eq!(core_info().version, env!("CARGO_PKG_VERSION"));
    }
}
