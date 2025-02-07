/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::LazyLock;

use num_traits::Zero;
use regex::Regex;
pub use script_bindings::str::*;
use time::{Date, Month, OffsetDateTime, Time, Weekday};

/// <https://html.spec.whatwg.org/multipage/#parse-a-month-component>
fn parse_month_component(value: &str) -> Option<(i32, u32)> {
    // Step 3
    let mut iterator = value.split('-');
    let year = iterator.next()?;
    let month = iterator.next()?;

    // Step 1, 2
    let year_int = year.parse::<i32>().ok()?;
    if year.len() < 4 || year_int == 0 {
        return None;
    }

    // Step 4, 5
    let month_int = month.parse::<u32>().ok()?;
    if month.len() != 2 || !(1..=12).contains(&month_int) {
        return None;
    }

    // Step 6
    Some((year_int, month_int))
}

/// <https://html.spec.whatwg.org/multipage/#parse-a-date-component>
fn parse_date_component(value: &str) -> Option<(i32, u32, u32)> {
    // Step 1
    let (year_int, month_int) = parse_month_component(value)?;

    // Step 3, 4
    let day = value.split('-').nth(2)?;
    let day_int = day.parse::<u32>().ok()?;
    if day.len() != 2 {
        return None;
    }

    // Step 2, 5
    let max_day = max_day_in_month(year_int, month_int)?;
    if day_int == 0 || day_int > max_day {
        return None;
    }

    // Step 6
    Some((year_int, month_int, day_int))
}

/// <https://html.spec.whatwg.org/multipage/#parse-a-time-component>
fn parse_time_component(value: &str) -> Option<(u8, u8, u8, u16)> {
    // Step 1: Collect a sequence of code points that are ASCII digits from input given
    // position. If the collected sequence is not exactly two characters long, then fail.
    // Otherwise, interpret the resulting sequence as a base-ten integer. Let that number
    // be the hour.
    let mut iterator = value.split(':');
    let hour = iterator.next()?;
    if hour.len() != 2 {
        return None;
    }
    // Step 2: If hour is not a number in the range 0 ≤ hour ≤ 23, then fail.
    let hour_int = hour.parse::<u8>().ok()?;
    if hour_int > 23 {
        return None;
    }

    // Step 3: If position is beyond the end of input or if the character at position is
    // not a U+003A COLON character, then fail. Otherwise, move position forwards one
    // character.
    // Step 4: Collect a sequence of code points that are ASCII digits from input given
    // position. If the collected sequence is not exactly two characters long, then fail.
    // Otherwise, interpret the resulting sequence as a base-ten integer. Let that number
    // be the minute.
    // Step 5: If minute is not a number in the range 0 ≤ minute ≤ 59, then fail.
    let minute = iterator.next()?;
    if minute.len() != 2 {
        return None;
    }
    let minute_int = minute.parse::<u8>().ok()?;
    if minute_int > 59 {
        return None;
    }

    // Step 6, 7: Asks us to parse the seconds as a floating point number, but below this
    // is done as integral parts in order to avoid floating point precision issues.
    let Some(seconds_and_milliseconds) = iterator.next() else {
        return Some((hour_int, minute_int, 0, 0));
    };

    // Parse the seconds portion.
    let mut second_iterator = seconds_and_milliseconds.split('.');
    let second = second_iterator.next()?;
    if second.len() != 2 {
        return None;
    }
    let second_int = second.parse::<u8>().ok()?;

    // Parse the milliseconds portion as a u16 (milliseconds can be up to 1000) and
    // make sure that it has the proper value based on how long the string is.
    let Some(millisecond) = second_iterator.next() else {
        return Some((hour_int, minute_int, second_int, 0));
    };
    let millisecond_length = millisecond.len() as u32;
    if millisecond_length > 3 {
        return None;
    }
    let millisecond_int = millisecond.parse::<u16>().ok()?;
    let millisecond_int = millisecond_int * 10_u16.pow(3 - millisecond_length);

    // Step 8: Return hour, minute, and second (and in our case the milliseconds due to the note
    // above about floating point precision).
    Some((hour_int, minute_int, second_int, millisecond_int))
}

fn max_day_in_month(year_num: i32, month_num: u32) -> Option<u32> {
    match month_num {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 => {
            if is_leap_year(year_num) {
                Some(29)
            } else {
                Some(28)
            }
        },
        _ => None,
    }
}

/// <https://html.spec.whatwg.org/multipage/#week-number-of-the-last-day>
///
/// > A week-year with a number year has 53 weeks if it corresponds to either a year year
/// > in the proleptic Gregorian calendar that has a Thursday as its first day (January
/// > 1st), or a year year in the proleptic Gregorian calendar that has a Wednesday as its
/// > first day (January 1st) and where year is a number divisible by 400, or a number
/// > divisible by 4 but not by 100. All other week-years have 52 weeks.
fn max_week_in_year(year: i32) -> u32 {
    let Ok(date) = Date::from_calendar_date(year, Month::January, 1) else {
        return 52;
    };

    match OffsetDateTime::new_utc(date, Time::MIDNIGHT).weekday() {
        Weekday::Thursday => 53,
        Weekday::Wednesday if is_leap_year(year) => 53,
        _ => 52,
    }
}

#[inline]
fn is_leap_year(year: i32) -> bool {
    year % 400 == 0 || (year % 4 == 0 && year % 100 != 0)
}

pub(crate) trait ToInputValueString {
    fn to_date_string(&self) -> String;
    fn to_month_string(&self) -> String;
    fn to_week_string(&self) -> String;
    fn to_time_string(&self) -> String;

    /// A valid normalized local date and time string should be "{date}T{time}"
    /// where date and time are both valid, and the time string must be as short as possible
    /// <https://html.spec.whatwg.org/multipage/#valid-normalised-local-date-and-time-string>
    fn to_local_date_time_string(&self) -> String;
}

impl ToInputValueString for OffsetDateTime {
    fn to_date_string(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}",
            self.year(),
            self.month() as u8,
            self.day()
        )
    }

    fn to_month_string(&self) -> String {
        format!("{:04}-{:02}", self.year(), self.month() as u8)
    }

    fn to_week_string(&self) -> String {
        // NB: The ISO week year might be different than the year of the day.
        let (year, week, _) = self.to_iso_week_date();
        format!("{:04}-W{:02}", year, week)
    }

    fn to_time_string(&self) -> String {
        if self.second().is_zero() && self.millisecond().is_zero() {
            format!("{:02}:{:02}", self.hour(), self.minute())
        } else {
            // This needs to trim off the zero parts of the milliseconds.
            format!(
                "{:02}:{:02}:{:02}.{:03}",
                self.hour(),
                self.minute(),
                self.second(),
                self.millisecond()
            )
            .trim_end_matches(['.', '0'])
            .to_owned()
        }
    }

    fn to_local_date_time_string(&self) -> String {
        format!("{}T{}", self.to_date_string(), self.to_time_string())
    }
}

pub(crate) trait FromInputValueString {
    /// <https://html.spec.whatwg.org/multipage/#parse-a-date-string>
    ///
    /// Parse the date string and return an [`OffsetDateTime`] on midnight of the
    /// given date in UTC.
    ///
    /// A valid date string should be "YYYY-MM-DD"
    /// YYYY must be four or more digits, MM and DD both must be two digits
    /// <https://html.spec.whatwg.org/multipage/#valid-date-string>
    fn parse_date_string(&self) -> Option<OffsetDateTime>;

    /// <https://html.spec.whatwg.org/multipage/#parse-a-month-string>
    ///
    /// Parse the month and return an [`OffsetDateTime`] on midnight of UTC of the morning of
    /// the first day of the parsed month.
    ///
    /// A valid month string should be "YYYY-MM" YYYY must be four or more digits, MM both
    /// must be two digits <https://html.spec.whatwg.org/multipage/#valid-month-string>
    fn parse_month_string(&self) -> Option<OffsetDateTime>;

    /// <https://html.spec.whatwg.org/multipage/#parse-a-week-string>
    ///
    /// Parse the week string, returning an [`OffsetDateTime`] on the Monday of the parsed
    /// week.
    ///
    /// A valid week string should be like {YYYY}-W{WW}, such as "2017-W52" YYYY must be
    /// four or more digits, WW both must be two digits
    /// <https://html.spec.whatwg.org/multipage/#valid-week-string>
    fn parse_week_string(&self) -> Option<OffsetDateTime>;

    /// Parse this value as a time string according to
    /// <https://html.spec.whatwg.org/multipage/#valid-time-string>.
    fn parse_time_string(&self) -> Option<OffsetDateTime>;

    /// <https://html.spec.whatwg.org/multipage/#parse-a-local-date-and-time-string>
    ///
    /// Parse the local date and time, returning an [`OffsetDateTime`] in UTC or None.
    fn parse_local_date_time_string(&self) -> Option<OffsetDateTime>;

    /// Validates whether or not this value is a valid date string according to
    /// <https://html.spec.whatwg.org/multipage/#valid-date-string>.
    fn is_valid_date_string(&self) -> bool {
        self.parse_date_string().is_some()
    }

    /// Validates whether or not this value is a valid month string according to
    /// <https://html.spec.whatwg.org/multipage/#valid-month-string>.
    fn is_valid_month_string(&self) -> bool {
        self.parse_month_string().is_some()
    }
    /// Validates whether or not this value is a valid week string according to
    /// <https://html.spec.whatwg.org/multipage/#valid-week-string>.
    fn is_valid_week_string(&self) -> bool {
        self.parse_week_string().is_some()
    }
    /// Validates whether or not this value is a valid time string according to
    /// <https://html.spec.whatwg.org/multipage/#valid-time-string>.
    fn is_valid_time_string(&self) -> bool;

    /// Validates whether or not this value is a valid local date time string according to
    /// <https://html.spec.whatwg.org/multipage/#valid-week-string>.
    fn is_valid_local_date_time_string(&self) -> bool {
        self.parse_local_date_time_string().is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#valid-simple-colour>
    fn is_valid_simple_color_string(&self) -> bool;

    /// <https://html.spec.whatwg.org/multipage/#valid-e-mail-address>
    fn is_valid_email_address_string(&self) -> bool;
}

impl FromInputValueString for &str {
    fn parse_date_string(&self) -> Option<OffsetDateTime> {
        // Step 1, 2, 3
        let (year_int, month_int, day_int) = parse_date_component(self)?;

        // Step 4
        if self.split('-').nth(3).is_some() {
            return None;
        }

        // Step 5, 6
        let month = (month_int as u8).try_into().ok()?;
        let date = Date::from_calendar_date(year_int, month, day_int as u8).ok()?;
        Some(OffsetDateTime::new_utc(date, Time::MIDNIGHT))
    }

    fn parse_month_string(&self) -> Option<OffsetDateTime> {
        // Step 1, 2, 3
        let (year_int, month_int) = parse_month_component(self)?;

        // Step 4
        if self.split('-').nth(2).is_some() {
            return None;
        }
        // Step 5
        let month = (month_int as u8).try_into().ok()?;
        let date = Date::from_calendar_date(year_int, month, 1).ok()?;
        Some(OffsetDateTime::new_utc(date, Time::MIDNIGHT))
    }

    fn parse_week_string(&self) -> Option<OffsetDateTime> {
        // Step 1, 2, 3
        let mut iterator = self.split('-');
        let year = iterator.next()?;

        // Step 4
        let year_int = year.parse::<i32>().ok()?;
        if year.len() < 4 || year_int == 0 {
            return None;
        }

        // Step 5, 6
        let week = iterator.next()?;
        let (week_first, week_last) = week.split_at(1);
        if week_first != "W" {
            return None;
        }

        // Step 7
        let week_int = week_last.parse::<u32>().ok()?;
        if week_last.len() != 2 {
            return None;
        }

        // Step 8
        let max_week = max_week_in_year(year_int);

        // Step 9
        if week_int < 1 || week_int > max_week {
            return None;
        }

        // Step 10
        if iterator.next().is_some() {
            return None;
        }

        // Step 11
        let date = Date::from_iso_week_date(year_int, week_int as u8, Weekday::Monday).ok()?;
        Some(OffsetDateTime::new_utc(date, Time::MIDNIGHT))
    }

    fn parse_time_string(&self) -> Option<OffsetDateTime> {
        // Step 1, 2, 3
        let (hour, minute, second, millisecond) = parse_time_component(self)?;

        // Step 4
        if self.split(':').nth(3).is_some() {
            return None;
        }

        // Step 5, 6
        let time = Time::from_hms_milli(hour, minute, second, millisecond).ok()?;
        Some(OffsetDateTime::new_utc(
            OffsetDateTime::UNIX_EPOCH.date(),
            time,
        ))
    }

    fn parse_local_date_time_string(&self) -> Option<OffsetDateTime> {
        // Step 1, 2, 4
        let mut iterator = if self.contains('T') {
            self.split('T')
        } else {
            self.split(' ')
        };

        // Step 3
        let date = iterator.next()?;
        let (year, month, day) = parse_date_component(date)?;

        // Step 5
        let time = iterator.next()?;
        let (hour, minute, second, millisecond) = parse_time_component(time)?;

        // Step 6
        if iterator.next().is_some() {
            return None;
        }

        // Step 7, 8, 9
        // TODO: Is this supposed to know the locale's daylight-savings-time rules?
        let month = (month as u8).try_into().ok()?;
        let date = Date::from_calendar_date(year, month, day as u8).ok()?;
        let time = Time::from_hms_milli(hour, minute, second, millisecond).ok()?;
        Some(OffsetDateTime::new_utc(date, time))
    }

    fn is_valid_time_string(&self) -> bool {
        enum State {
            HourHigh,
            HourLow09,
            HourLow03,
            MinuteColon,
            MinuteHigh,
            MinuteLow,
            SecondColon,
            SecondHigh,
            SecondLow,
            MilliStop,
            MilliHigh,
            MilliMiddle,
            MilliLow,
            Done,
            Error,
        }
        let next_state = |valid: bool, next: State| -> State {
            if valid {
                next
            } else {
                State::Error
            }
        };

        let state = self.chars().fold(State::HourHigh, |state, c| {
            match state {
                // Step 1 "HH"
                State::HourHigh => match c {
                    '0' | '1' => State::HourLow09,
                    '2' => State::HourLow03,
                    _ => State::Error,
                },
                State::HourLow09 => next_state(c.is_ascii_digit(), State::MinuteColon),
                State::HourLow03 => next_state(c.is_digit(4), State::MinuteColon),

                // Step 2 ":"
                State::MinuteColon => next_state(c == ':', State::MinuteHigh),

                // Step 3 "mm"
                State::MinuteHigh => next_state(c.is_digit(6), State::MinuteLow),
                State::MinuteLow => next_state(c.is_ascii_digit(), State::SecondColon),

                // Step 4.1 ":"
                State::SecondColon => next_state(c == ':', State::SecondHigh),
                // Step 4.2 "ss"
                State::SecondHigh => next_state(c.is_digit(6), State::SecondLow),
                State::SecondLow => next_state(c.is_ascii_digit(), State::MilliStop),

                // Step 4.3.1 "."
                State::MilliStop => next_state(c == '.', State::MilliHigh),
                // Step 4.3.2 "SSS"
                State::MilliHigh => next_state(c.is_ascii_digit(), State::MilliMiddle),
                State::MilliMiddle => next_state(c.is_ascii_digit(), State::MilliLow),
                State::MilliLow => next_state(c.is_ascii_digit(), State::Done),

                _ => State::Error,
            }
        });

        match state {
            State::Done |
            // Step 4 (optional)
            State::SecondColon |
            // Step 4.3 (optional)
            State::MilliStop |
            // Step 4.3.2 (only 1 digit required)
            State::MilliMiddle | State::MilliLow => true,
            _ => false
        }
    }

    fn is_valid_simple_color_string(&self) -> bool {
        let mut chars = self.chars();
        if self.len() == 7 && chars.next() == Some('#') {
            chars.all(|c| c.is_ascii_hexdigit())
        } else {
            false
        }
    }

    fn is_valid_email_address_string(&self) -> bool {
        static RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(concat!(
                r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?",
                r"(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
            ))
            .unwrap()
        });
        RE.is_match(self)
    }
}
