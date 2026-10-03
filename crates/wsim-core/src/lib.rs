//! Simulation core of WSim.
//!
//! The core has no dependency on the UI, Tauri or file IO. It receives a compiled
//! catalog from `wsim-data`, accepts commands and advances the world in daily ticks.

pub mod calendar;
pub mod catalog;
pub mod command;
pub mod game;
pub mod ids;
pub mod message;
pub mod money;
pub mod rng;
pub mod save;
pub mod state;
pub mod time_series;

#[cfg(test)]
mod determinism_tests;

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
