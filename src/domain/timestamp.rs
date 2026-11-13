//! Points in time as ai-ctl records them: UTC, to the second.

use std::fmt;

/// Seconds since the Unix epoch, in UTC, written in RFC 3339 form:
/// `2026-10-09T18:02:11Z`. Years run from 0 to 9999, so the text always has
/// the same width and sorts in time order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(i64);

const SECONDS_PER_DAY: i64 = 86_400;
/// `0000-01-01T00:00:00Z` and `9999-12-31T23:59:59Z`.
const MIN: i64 = -62_167_219_200;
const MAX: i64 = 253_402_300_799;

impl Timestamp {
    /// 1970-01-01T00:00:00Z.
    pub const EPOCH: Self = Self(0);

    /// The timestamp `seconds` after the Unix epoch, or `None` outside years
    /// 0 to 9999.
    pub fn from_unix(seconds: i64) -> Option<Self> {
        (MIN..=MAX).contains(&seconds).then_some(Self(seconds))
    }

    /// Parses exactly `YYYY-MM-DDTHH:MM:SSZ`, the form [`Timestamp`] writes.
    /// Out-of-range fields, such as February 30, are rejected.
    pub fn parse(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        let shape = bytes.len() == 20
            && bytes.iter().enumerate().all(|(i, &b)| match i {
                4 | 7 => b == b'-',
                10 => b == b'T',
                13 | 16 => b == b':',
                19 => b == b'Z',
                _ => b.is_ascii_digit(),
            });
        if !shape {
            return None;
        }
        let field = |range: std::ops::Range<usize>| text[range].parse::<i64>().ok();
        let (year, month, day) = (field(0..4)?, field(5..7)?, field(8..10)?);
        let (hour, minute, second) = (field(11..13)?, field(14..16)?, field(17..19)?);
        if !(1..=12).contains(&month)
            || !(1..=31).contains(&day)
            || hour > 23
            || minute > 59
            || second > 59
        {
            return None;
        }
        let days = days_from_civil(year, month, day);
        let timestamp = Self(days * SECONDS_PER_DAY + hour * 3600 + minute * 60 + second);
        // A day past the end of its month moves into the next one: writing
        // the value back exposes it.
        (timestamp.to_string() == text).then_some(timestamp)
    }

    /// `2026-10-09 18:02 UTC`, for listings.
    pub fn minutes(self) -> String {
        let text = self.to_string();
        format!("{} {} UTC", &text[..10], &text[11..16])
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let days = self.0.div_euclid(SECONDS_PER_DAY);
        let seconds = self.0.rem_euclid(SECONDS_PER_DAY);
        let (year, month, day) = civil_from_days(days);
        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
            seconds / 3600,
            seconds % 3600 / 60,
            seconds % 60
        )
    }
}

/// Days since 1970-01-01 of a proleptic Gregorian date (Howard Hinnant's
/// `days_from_civil`).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The date `days` after 1970-01-01 (Howard Hinnant's `civil_from_days`).
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> Timestamp {
        Timestamp::parse(text).unwrap_or_else(|| panic!("{text} does not parse"))
    }

    #[test]
    fn known_instants_format_in_rfc_3339() {
        let cases = [
            (0, "1970-01-01T00:00:00Z"),
            (951_782_400, "2000-02-29T00:00:00Z"),
            (1_791_568_931, "2026-10-09T18:02:11Z"),
            (-1, "1969-12-31T23:59:59Z"),
            (MIN, "0000-01-01T00:00:00Z"),
            (MAX, "9999-12-31T23:59:59Z"),
        ];
        for (seconds, text) in cases {
            let timestamp = Timestamp::from_unix(seconds).unwrap();
            assert_eq!(timestamp.to_string(), text);
            assert_eq!(at(text), timestamp, "{text}");
        }
    }

    #[test]
    fn every_day_of_a_leap_cycle_round_trips() {
        let start = 946_684_800;
        assert_eq!(
            at("2000-01-01T00:00:00Z"),
            Timestamp::from_unix(start).unwrap()
        );
        for day in 0..(4 * 366) {
            let timestamp = Timestamp::from_unix(start + day * SECONDS_PER_DAY + 43_199).unwrap();
            assert_eq!(Timestamp::parse(&timestamp.to_string()), Some(timestamp));
        }
    }

    #[test]
    fn impossible_dates_are_rejected() {
        for text in [
            "2026-02-29T00:00:00Z",
            "2100-02-29T00:00:00Z",
            "2026-04-31T00:00:00Z",
            "2026-13-01T00:00:00Z",
            "2026-00-10T00:00:00Z",
            "2026-10-09T24:00:00Z",
            "2026-10-09T18:60:00Z",
            "2026-10-09T18:02:60Z",
        ] {
            assert_eq!(Timestamp::parse(text), None, "{text}");
        }
        assert!(Timestamp::parse("2024-02-29T00:00:00Z").is_some());
    }

    #[test]
    fn other_shapes_are_rejected() {
        for text in [
            "",
            "2026-10-09",
            "2026-10-09T18:02:11",
            "2026-10-09T18:02:11+00:00",
            "2026-10-09 18:02:11Z",
            "2026-10-09T18:02:11.5Z",
            "+026-10-09T18:02:11Z",
        ] {
            assert_eq!(Timestamp::parse(text), None, "{text:?}");
        }
    }

    #[test]
    fn years_beyond_four_digits_are_out_of_range() {
        assert_eq!(Timestamp::from_unix(MAX + 1), None);
        assert_eq!(Timestamp::from_unix(MIN - 1), None);
    }

    #[test]
    fn timestamps_order_in_time_and_list_to_the_minute() {
        assert!(at("2026-10-09T18:02:11Z") < at("2026-10-10T00:00:00Z"));
        assert_eq!(at("2026-10-09T18:02:11Z").minutes(), "2026-10-09 18:02 UTC");
    }
}
