//! Money for display (Lastenheft §3.6, §18.2; formulas in docs/FORMELN.md, M21).
//!
//! The simulation counts in US dollars of the base year's purchasing power (2026). For
//! display only, amounts are converted into the currency a country uses: at the
//! purchasing power of the base year (its currency of that year) or at the prices of
//! the game date (US inflation since then and the exchange rate of the time). Nothing
//! here feeds back into the simulation.

use crate::catalog::Provenance;
use crate::ids::{CountryId, Id};
use crate::math;

/// Currencies, the periods in which the countries use them, and US consumer prices.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurrencyModel {
    pub currencies: Vec<Currency>,
    /// By country index: from when on (fractional year) which currency, ascending.
    pub periods: Vec<Vec<(f64, usize)>>,
    /// The lead currency (US dollar), the unit of the simulation.
    pub lead: usize,
    /// US consumer prices: (fractional year, index), ascending.
    pub us_prices: Vec<(f64, f64)>,
    /// Year whose purchasing power the game's dollars have.
    pub base_year: i32,
    /// Assumed US inflation per year after the last index value.
    pub inflation_after: f64,
    pub prices_provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Currency {
    pub key: String,
    pub symbol: String,
    pub rate: Rate,
    pub provenance: Provenance,
}

/// Units of a currency per US dollar of the time.
#[derive(Clone, Debug, PartialEq)]
pub enum Rate {
    /// (fractional year, units per dollar), ascending: log-linear in between, constant
    /// before the first and after the last point.
    Points(Vec<(f64, f64)>),
    /// A fixed number of units per unit of another (unpegged) currency.
    Peg { currency: usize, factor: f64 },
}

/// How amounts are shown: a currency and the factor from game dollars to its units.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MoneyDisplay {
    pub currency: String,
    pub symbol: String,
    pub factor: f64,
}

/// Value at `t` of points with interpolation on a logarithmic scale.
fn log_interpolate(points: &[(f64, f64)], t: f64) -> f64 {
    let Some(&(t0, first)) = points.first() else {
        return 1.0;
    };
    let &(t1, last) = points.last().expect("not empty");
    if t <= t0 {
        return first;
    }
    if t >= t1 {
        return last;
    }
    let next = points.partition_point(|&(x, _)| x <= t);
    let (a, va) = points[next - 1];
    let (b, vb) = points[next];
    let s = (t - a) / (b - a);
    math::exp(math::ln(va) + (math::ln(vb) - math::ln(va)) * s)
}

impl CurrencyModel {
    /// Middle of the base year: the game's dollars are the base year's average.
    fn base(&self) -> f64 {
        f64::from(self.base_year) + 0.5
    }

    /// US consumer prices at `t`; after the last value they rise by the assumed rate.
    pub fn us_prices_at(&self, t: f64) -> f64 {
        let Some(&(t0, first)) = self.us_prices.first() else {
            return 1.0;
        };
        let &(t1, last) = self.us_prices.last().expect("not empty");
        if t <= t0 {
            return first;
        }
        if t >= t1 {
            return last * math::pow(1.0 + self.inflation_after, t - t1);
        }
        let next = self.us_prices.partition_point(|&(x, _)| x <= t);
        let (a, va) = self.us_prices[next - 1];
        let (b, vb) = self.us_prices[next];
        va + (vb - va) * (t - a) / (b - a)
    }

    /// US dollars of time `t` per dollar of the base year.
    pub fn inflation_since_base(&self, t: f64) -> f64 {
        self.us_prices_at(t) / self.us_prices_at(self.base())
    }

    /// Units of a currency per US dollar of time `t`.
    pub fn rate(&self, currency: usize, t: f64) -> f64 {
        match &self.currencies[currency].rate {
            Rate::Points(points) => log_interpolate(points, t),
            Rate::Peg { currency, factor } => factor * self.rate(*currency, t),
        }
    }

    /// When a country starts using which currency, ascending (empty without data).
    pub fn periods_of(&self, country: CountryId) -> &[(f64, usize)] {
        self.periods.get(country.index()).map_or(&[], Vec::as_slice)
    }

    /// The currency a country uses at `t` (before its first period: the first one).
    pub fn currency_at(&self, country: CountryId, t: f64) -> Option<usize> {
        let periods = self.periods.get(country.index())?;
        let after = periods.partition_point(|&(from, _)| from <= t);
        periods.get(after.saturating_sub(1)).map(|&(_, c)| c)
    }

    fn display(&self, currency: usize, factor: f64) -> MoneyDisplay {
        let c = &self.currencies[currency];
        MoneyDisplay {
            currency: c.key.clone(),
            symbol: c.symbol.clone(),
            factor,
        }
    }

    /// A country's currency of the base year, at the base year's purchasing power.
    pub fn at_base(&self, country: CountryId) -> Option<MoneyDisplay> {
        let t = self.base();
        let c = self.currency_at(country, t)?;
        Some(self.display(c, self.rate(c, t)))
    }

    /// A country's currency at `t`, in money of that time.
    pub fn at_time(&self, country: CountryId, t: f64) -> Option<MoneyDisplay> {
        let c = self.currency_at(country, t)?;
        Some(self.display(c, self.rate(c, t) * self.inflation_since_base(t)))
    }

    /// The lead currency (game dollars) of the base year, or of time `t` with inflation.
    pub fn lead(&self, t: Option<f64>) -> Option<MoneyDisplay> {
        self.currencies.get(self.lead)?;
        Some(self.display(self.lead, t.map_or(1.0, |t| self.inflation_since_base(t))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn currency(key: &str, rate: Rate) -> Currency {
        Currency {
            key: key.to_owned(),
            symbol: key.to_uppercase(),
            rate,
            provenance: Provenance::default(),
        }
    }

    /// A country with Mark until 1924, then Reichsmark, and a pegged currency elsewhere.
    fn model() -> CurrencyModel {
        CurrencyModel {
            currencies: vec![
                currency("usd", Rate::Points(vec![(1900.0, 1.0)])),
                currency("mark", Rate::Points(vec![(1900.5, 4.2), (1923.5, 4.2e12)])),
                currency("rm", Rate::Points(vec![(1924.5, 4.2)])),
                currency(
                    "bwi",
                    Rate::Peg {
                        currency: 1,
                        factor: 2.0,
                    },
                ),
            ],
            periods: vec![vec![(1900.0, 1), (1924.0, 2)], vec![(1900.0, 3)]],
            lead: 0,
            us_prices: vec![(1900.5, 8.0), (2026.5, 320.0)],
            base_year: 2026,
            inflation_after: 0.02,
            prices_provenance: Provenance::default(),
        }
    }

    #[test]
    fn currencies_change_at_the_start_of_their_period() {
        let m = model();
        let deu = CountryId::from_index(0);
        assert_eq!(m.currency_at(deu, 1900.0), Some(1));
        assert_eq!(m.currency_at(deu, 1923.99), Some(1));
        assert_eq!(m.currency_at(deu, 1924.0), Some(2));
        // Before the first period: the first currency.
        assert_eq!(m.currency_at(deu, 1850.0), Some(1));
        assert_eq!(m.currency_at(CountryId::from_index(9), 1950.0), None);
    }

    #[test]
    fn rates_follow_a_logarithmic_path_and_pegs_follow_their_anchor() {
        let m = model();
        assert!((m.rate(1, 1900.0) - 4.2).abs() < 1e-9);
        // Halfway on the log scale between 4.2 and 4.2e12: 4.2e6.
        let mid = (1900.5 + 1923.5) / 2.0;
        assert!((m.rate(1, mid) / 4.2e6 - 1.0).abs() < 1e-9);
        assert!((m.rate(1, 1950.0) - 4.2e12).abs() < 1.0);
        assert!((m.rate(3, mid) - 2.0 * m.rate(1, mid)).abs() < 1e-3);
    }

    #[test]
    fn money_of_the_time_carries_us_inflation() {
        let m = model();
        assert!((m.inflation_since_base(2026.5) - 1.0).abs() < 1e-12);
        assert!((m.inflation_since_base(1900.5) - 8.0 / 320.0).abs() < 1e-12);
        // After the last index value: the assumed inflation.
        assert!((m.inflation_since_base(2036.5) - 1.02_f64.powi(10)).abs() < 1e-9);
        let then = m
            .at_time(CountryId::from_index(0), 1900.5)
            .expect("currency");
        assert_eq!(then.currency, "mark");
        assert!((then.factor - 4.2 * 8.0 / 320.0).abs() < 1e-12);
        let base = m.at_base(CountryId::from_index(0)).expect("currency");
        assert_eq!(base.currency, "rm");
        assert!((base.factor - 4.2).abs() < 1e-12);
        let lead = m.lead(None).expect("lead");
        assert_eq!((lead.currency.as_str(), lead.factor), ("usd", 1.0));
        assert!((m.lead(Some(1900.5)).expect("lead").factor - 0.025).abs() < 1e-12);
    }

    #[test]
    fn an_empty_model_shows_nothing() {
        let m = CurrencyModel::default();
        assert_eq!(m.at_base(CountryId::from_index(0)), None);
        assert_eq!(m.lead(None), None);
    }
}
