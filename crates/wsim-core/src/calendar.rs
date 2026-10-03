//! Calendar dates and round lengths (Lastenheft §3.1, §13.1).

use std::fmt;

use serde::{Deserialize, Serialize};

/// A day in the Gregorian calendar, stored as days since 1970-01-01.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Date(i32);

/// First day after the game: the last simulated day is 31.12.2100.
pub const GAME_END: Date = Date(days_from_civil(2101, 1, 1));

impl Date {
    /// Returns `None` for days that do not exist (e.g. 30 February).
    pub fn new(year: i32, month: u32, day: u32) -> Option<Date> {
        let valid = (1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month);
        valid.then(|| Date(days_from_civil(year, month, day)))
    }

    /// 1 January of `year`.
    pub fn first_of_year(year: i32) -> Date {
        Date(days_from_civil(year, 1, 1))
    }

    pub fn year(self) -> i32 {
        civil_from_days(self.0).0
    }

    pub fn month(self) -> u32 {
        civil_from_days(self.0).1
    }

    pub fn day(self) -> u32 {
        civil_from_days(self.0).2
    }

    /// Day of the year, 1 = 1 January.
    pub fn ordinal(self) -> u32 {
        u32::try_from(self.0 - days_from_civil(self.year(), 1, 1) + 1).expect("positive")
    }

    #[must_use]
    pub fn add_days(self, days: i32) -> Date {
        Date(self.0 + days)
    }

    #[must_use]
    pub fn next_day(self) -> Date {
        self.add_days(1)
    }

    /// Number of days from `self` to `later` (negative if `later` is earlier).
    pub fn days_until(self, later: Date) -> i32 {
        later.0 - self.0
    }

    #[must_use]
    pub fn first_of_month(self) -> Date {
        let (year, month, _) = civil_from_days(self.0);
        Date(days_from_civil(year, month, 1))
    }

    #[must_use]
    pub fn first_of_next_month(self) -> Date {
        let (year, month, _) = civil_from_days(self.0);
        if month == 12 {
            Date(days_from_civil(year + 1, 1, 1))
        } else {
            Date(days_from_civil(year, month + 1, 1))
        }
    }

    #[must_use]
    pub fn first_of_next_quarter(self) -> Date {
        let (year, month, _) = civil_from_days(self.0);
        let next_quarter_month = (month - 1) / 3 * 3 + 4;
        if next_quarter_month > 12 {
            Date(days_from_civil(year + 1, 1, 1))
        } else {
            Date(days_from_civil(year, next_quarter_month, 1))
        }
    }

    /// Point in time as fractional year: 1 January 1900 = 1900.0.
    pub fn year_fraction(self) -> f64 {
        let year = self.year();
        let days = f64::from(days_in_year(year));
        f64::from(year) + f64::from(self.ordinal() - 1) / days
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (y, m, d) = civil_from_days(self.0);
        write!(f, "{y:04}-{m:02}-{d:02}")
    }
}

impl fmt::Debug for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub fn days_in_year(year: i32) -> u32 {
    if is_leap_year(year) { 366 } else { 365 }
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

// Conversion between civil dates and day numbers after Howard Hinnant,
// "chrono-Compatible Low-Level Date Algorithms".
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
const fn days_from_civil(year: i32, month: u32, day: u32) -> i32 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe as i32 - 719_468
}

#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
fn civil_from_days(days: i32) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i32 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Length of a round, chosen by the player when ending a round (Lastenheft §13.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoundLength {
    Day,
    Week,
    /// Until the first day of the next calendar month.
    Month,
    /// Until the first day of the next calendar quarter.
    Quarter,
}

impl RoundLength {
    pub const ALL: [RoundLength; 4] = [
        RoundLength::Day,
        RoundLength::Week,
        RoundLength::Month,
        RoundLength::Quarter,
    ];

    /// First day after a round that starts on `from`, never beyond the end of the game.
    pub fn end(self, from: Date) -> Date {
        let end = match self {
            RoundLength::Day => from.next_day(),
            RoundLength::Week => from.add_days(7),
            RoundLength::Month => from.first_of_next_month(),
            RoundLength::Quarter => from.first_of_next_quarter(),
        };
        end.min(GAME_END)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> Date {
        Date::new(y, m, d).unwrap()
    }

    #[test]
    fn round_trips_civil_dates() {
        for (y, m, d) in [
            (1900, 1, 1),
            (1900, 2, 28),
            (1900, 3, 1),
            (2000, 2, 29),
            (2100, 12, 31),
            (1970, 1, 1),
        ] {
            let date = date(y, m, d);
            assert_eq!((date.year(), date.month(), date.day()), (y, m, d));
        }
        assert_eq!(date(1970, 1, 1), Date(0));
    }

    #[test]
    fn walks_every_day_of_two_centuries() {
        let mut day = date(1900, 1, 1);
        let mut count = 0;
        while day < GAME_END {
            let next = day.next_day();
            assert!(
                next.day() == day.day() + 1 || next.day() == 1,
                "{day} → {next}"
            );
            day = next;
            count += 1;
        }
        // 201 years, 49 leap years (1900 and 2100 are not leap years).
        assert_eq!(count, 201 * 365 + 49);
    }

    #[test]
    fn rejects_invalid_dates() {
        assert!(Date::new(1900, 2, 29).is_none());
        assert!(Date::new(1904, 2, 30).is_none());
        assert!(Date::new(1900, 13, 1).is_none());
        assert!(Date::new(1900, 4, 31).is_none());
        assert!(Date::new(2000, 2, 29).is_some());
    }

    #[test]
    fn computes_ordinals_and_fractions() {
        assert_eq!(date(1900, 1, 1).ordinal(), 1);
        assert_eq!(date(1900, 12, 31).ordinal(), 365);
        assert_eq!(date(1904, 12, 31).ordinal(), 366);
        assert_eq!(date(1900, 1, 1).year_fraction(), 1900.0);
        let mid = date(1900, 7, 2).year_fraction();
        assert!((mid - (1900.0 + 182.0 / 365.0)).abs() < 1e-12, "{mid}");
    }

    #[test]
    fn round_lengths_follow_the_calendar() {
        let start = date(1900, 1, 1);
        assert_eq!(RoundLength::Day.end(start), date(1900, 1, 2));
        assert_eq!(RoundLength::Week.end(start), date(1900, 1, 8));
        assert_eq!(RoundLength::Month.end(start), date(1900, 2, 1));
        assert_eq!(RoundLength::Quarter.end(start), date(1900, 4, 1));
        // A month round after a week round only runs to the end of the month.
        assert_eq!(RoundLength::Month.end(date(1900, 1, 8)), date(1900, 2, 1));
        assert_eq!(
            RoundLength::Quarter.end(date(1900, 11, 15)),
            date(1901, 1, 1)
        );
        assert_eq!(RoundLength::Month.end(date(1900, 12, 1)), date(1901, 1, 1));
    }

    #[test]
    fn rounds_stop_at_the_end_of_the_game() {
        assert_eq!(RoundLength::Week.end(date(2100, 12, 28)), GAME_END);
        assert_eq!(RoundLength::Quarter.end(date(2100, 12, 31)), GAME_END);
        assert_eq!(GAME_END, date(2101, 1, 1));
    }

    #[test]
    fn displays_iso_dates() {
        assert_eq!(date(1900, 3, 7).to_string(), "1900-03-07");
    }
}
