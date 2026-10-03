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

    /// Amount times a factor, rounded to the nearest unit (factors come from quantities,
    /// shares and prices, which are floating point).
    #[must_use]
    pub fn scale(self, factor: f64) -> Money {
        let value = (self.0 as f64 * factor).round();
        // Amounts in the game stay far below the i64 range.
        #[allow(clippy::cast_possible_truncation)]
        Money(value as i64)
    }

    /// Price per unit times quantity.
    pub fn times(price: Money, quantity: f64) -> Money {
        price.scale(quantity)
    }

    pub fn is_negative(self) -> bool {
        self.0 < 0
    }

    #[must_use]
    pub fn abs(self) -> Money {
        Money(self.0.abs())
    }
}

impl std::ops::Add for Money {
    type Output = Money;
    fn add(self, rhs: Money) -> Money {
        Money(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Money {
    type Output = Money;
    fn sub(self, rhs: Money) -> Money {
        Money(self.0 - rhs.0)
    }
}

impl std::ops::Neg for Money {
    type Output = Money;
    fn neg(self) -> Money {
        Money(-self.0)
    }
}

impl std::ops::AddAssign for Money {
    fn add_assign(&mut self, rhs: Money) {
        self.0 += rhs.0;
    }
}

impl std::ops::SubAssign for Money {
    fn sub_assign(&mut self, rhs: Money) {
        self.0 -= rhs.0;
    }
}

impl std::iter::Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        iter.fold(Money::ZERO, |a, b| a + b)
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
