//! Yearly values with linear interpolation (Lastenheft §3.2).

use serde::{Deserialize, Serialize};

/// Values given for individual years. Between two points the value is interpolated
/// linearly; before the first and after the last point it stays constant.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeSeries {
    points: Vec<(i32, f64)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeSeriesError {
    Empty,
    NotFinite { year: i32 },
}

impl TimeSeries {
    /// Builds a series from `(year, value)` points in any order. Duplicate years are not
    /// possible because callers pass map entries.
    pub fn new(mut points: Vec<(i32, f64)>) -> Result<Self, TimeSeriesError> {
        if points.is_empty() {
            return Err(TimeSeriesError::Empty);
        }
        if let Some(&(year, _)) = points.iter().find(|(_, v)| !v.is_finite()) {
            return Err(TimeSeriesError::NotFinite { year });
        }
        points.sort_by_key(|&(year, _)| year);
        Ok(Self { points })
    }

    pub fn points(&self) -> &[(i32, f64)] {
        &self.points
    }

    /// Value at a point in time given as fractional year (1900.5 = middle of 1900).
    pub fn value_at(&self, year: f64) -> f64 {
        let first = self.points[0];
        let last = self.points[self.points.len() - 1];
        if year <= f64::from(first.0) {
            return first.1;
        }
        if year >= f64::from(last.0) {
            return last.1;
        }
        // `partition_point` gives the first point after `year`; it exists and is > 0
        // because of the bounds checks above.
        let next = self.points.partition_point(|&(y, _)| f64::from(y) <= year);
        let (y0, v0) = self.points[next - 1];
        let (y1, v1) = self.points[next];
        let t = (year - f64::from(y0)) / f64::from(y1 - y0);
        v0 + (v1 - v0) * t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series() -> TimeSeries {
        TimeSeries::new(vec![(1913, 67.0), (1900, 56.0), (1930, 65.0)]).unwrap()
    }

    #[test]
    fn hits_given_points() {
        assert_eq!(series().value_at(1900.0), 56.0);
        assert_eq!(series().value_at(1913.0), 67.0);
        assert_eq!(series().value_at(1930.0), 65.0);
    }

    #[test]
    fn interpolates_linearly() {
        let s = TimeSeries::new(vec![(1900, 10.0), (1910, 20.0)]).unwrap();
        assert_eq!(s.value_at(1905.0), 15.0);
        assert_eq!(s.value_at(1902.5), 12.5);
    }

    #[test]
    fn stays_constant_outside_range() {
        assert_eq!(series().value_at(1850.0), 56.0);
        assert_eq!(series().value_at(2000.0), 65.0);
    }

    #[test]
    fn single_point_is_constant() {
        let s = TimeSeries::new(vec![(1900, 3.0)]).unwrap();
        assert_eq!(s.value_at(1800.0), 3.0);
        assert_eq!(s.value_at(2100.0), 3.0);
    }

    #[test]
    fn rejects_invalid_input() {
        assert_eq!(TimeSeries::new(vec![]), Err(TimeSeriesError::Empty));
        assert_eq!(
            TimeSeries::new(vec![(1900, f64::NAN)]),
            Err(TimeSeriesError::NotFinite { year: 1900 })
        );
    }
}
