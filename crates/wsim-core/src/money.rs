//! Money in the lead currency (USD, purchasing power 2026).
//!
//! Stored as an integer number of hundredths of a cent so that sums are exact and
//! identical on every machine.

use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Money(i64);

impl Money {
    pub const ZERO: Money = Money(0);
    /// Internal units per US dollar.
    pub const UNITS_PER_USD: i64 = 10_000;

    pub const fn from_units(units: i64) -> Self {
        Money(units)
    }

    pub const fn units(self) -> i64 {
        self.0
    }

    /// Converts a dollar amount, rounding to the nearest internal unit.
    /// Returns `None` for non-finite values or amounts outside the representable range.
    pub fn from_usd(usd: f64) -> Option<Self> {
        // Leave headroom so that sums of many large amounts cannot overflow.
        const MAX_USD: f64 = 1.0e14;
        if !usd.is_finite() || usd.abs() > MAX_USD {
            return None;
        }
        // Range checked above, so the cast cannot saturate.
        #[allow(clippy::cast_possible_truncation)]
        Some(Money((usd * Self::UNITS_PER_USD as f64).round() as i64))
    }

    pub fn to_usd(self) -> f64 {
        self.0 as f64 / Self::UNITS_PER_USD as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_dollars_exactly() {
        assert_eq!(
            Money::from_usd(60_000_000.0).unwrap().units(),
            600_000_000_000
        );
        assert_eq!(Money::from_usd(0.015).unwrap().units(), 150);
        assert_eq!(Money::from_usd(12.345_67).unwrap().to_usd(), 12.3457);
    }

    #[test]
    fn rejects_invalid_amounts() {
        assert!(Money::from_usd(f64::NAN).is_none());
        assert!(Money::from_usd(f64::INFINITY).is_none());
        assert!(Money::from_usd(1.0e15).is_none());
    }
}
