/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `ByteString` struct.
use std::borrow::{Borrow, Cow, ToOwned};
use std::default::Default;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::{fmt, ops, str};

use chrono::prelude::{Utc, Weekday};
use chrono::{Datelike, TimeZone};
use cssparser::CowRcStr;
use html5ever::{LocalName, Namespace};
use lazy_static::lazy_static;
use num_traits::Zero;
use regex::Regex;
use servo_atoms::Atom;

/// Encapsulates the IDL `ByteString` type.
#[derive(Clone, Debug, Default, Eq, JSTraceable, MallocSizeOf, PartialEq)]
pub struct ByteString(Vec<u8>);

impl ByteString {
    /// Creates a new `ByteString`.
    pub fn new(value: Vec<u8>) -> ByteString {
        ByteString(value)
    }

    /// Returns `self` as a string, if it encodes valid UTF-8, and `None`
    /// otherwise.
    pub fn as_str(&self) -> Option<&str> {
        str::from_utf8(&self.0).ok()
    }

    /// Returns the length.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if the ByteString is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns `self` with A–Z replaced by a–z.
    pub fn to_lower(&self) -> ByteString {
        ByteString::new(self.0.to_ascii_lowercase())
    }
}

impl From<ByteString> for Vec<u8> {
    fn from(byte_string: ByteString) -> Vec<u8> {
        byte_string.0
    }
}

impl Hash for ByteString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl FromStr for ByteString {
    type Err = ();
    fn from_str(s: &str) -> Result<ByteString, ()> {
        Ok(ByteString::new(s.to_owned().into_bytes()))
    }
}

impl ops::Deref for ByteString {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

/// A string that is constructed from a UCS-2 buffer by replacing invalid code
/// points with the replacement character.
#[derive(Clone, Default, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub struct USVString(pub String);

impl Borrow<str> for USVString {
    #[inline]
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Deref for USVString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &self.0
    }
}

impl DerefMut for USVString {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl AsRef<str> for USVString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for USVString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl PartialEq<str> for USVString {
    fn eq(&self, other: &str) -> bool {
        &**self == other
    }
}

impl<'a> PartialEq<&'a str> for USVString {
    fn eq(&self, other: &&'a str) -> bool {
        &**self == *other
    }
}

impl From<String> for USVString {
    fn from(contents: String) -> USVString {
        USVString(contents)
    }
}

/// Returns whether `s` is a `token`, as defined by
/// [RFC 2616](http://tools.ietf.org/html/rfc2616#page-17).
pub fn is_token(s: &[u8]) -> bool {
    if s.is_empty() {
        return false; // A token must be at least a single character
    }
    s.iter().all(|&x| {
        // http://tools.ietf.org/html/rfc2616#section-2.2
        match x {
            0..=31 | 127 => false, // CTLs
            40 | 41 | 60 | 62 | 64 | 44 | 59 | 58 | 92 | 34 | 47 | 91 | 93 | 63 | 61 | 123 |
            125 | 32 => false, // separators
            x if x > 127 => false, // non-CHARs
            _ => true,
        }
    })
}

/// A DOMString.
///
/// This type corresponds to the [`DOMString`] type in WebIDL.
///
/// [`DOMString`]: https://webidl.spec.whatwg.org/#idl-DOMString
///
/// Conceptually, a DOMString has the same value space as a JavaScript String,
/// i.e., an array of 16-bit *code units* representing UTF-16, potentially with
/// unpaired surrogates present (also sometimes called WTF-16).
///
/// Currently, this type stores a Rust `String`, in order to avoid issues when
/// integrating with the rest of the Rust ecosystem and even the rest of the
/// browser itself.
///
/// However, Rust `String`s are guaranteed to be valid UTF-8, and as such have
/// a *smaller value space* than WTF-16 (i.e., some JavaScript String values
/// can not be represented as a Rust `String`). This introduces the question of
/// what to do with values being passed from JavaScript to Rust that contain
/// unpaired surrogates.
///
/// The hypothesis is that it does not matter much how exactly those values are
/// transformed, because passing unpaired surrogates into the DOM is very rare.
/// In order to test this hypothesis, Servo will panic when encountering any
/// unpaired surrogates on conversion to `DOMString` by default. (The command
/// line option `-Z replace-surrogates` instead causes Servo to replace the
/// unpaired surrogate by a U+FFFD replacement character.)
///
/// Currently, the lack of crash reports about this issue provides some
/// evidence to support the hypothesis. This evidence will hopefully be used to
/// convince other browser vendors that it would be safe to replace unpaired
/// surrogates at the boundary between JavaScript and native code. (This would
/// unify the `DOMString` and `USVString` types, both in the WebIDL standard
/// and in Servo.)
///
/// This type is currently `!Send`, in order to help with an independent
/// experiment to store `JSString`s rather than Rust `String`s.
#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub struct DOMString(String, PhantomData<*const ()>);

impl DOMString {
    /// Creates a new `DOMString`.
    pub fn new() -> DOMString {
        DOMString(String::new(), PhantomData)
    }

    /// Creates a new `DOMString` from a `String`.
    pub fn from_string(s: String) -> DOMString {
        DOMString(s, PhantomData)
    }

    /// Appends a given string slice onto the end of this String.
    pub fn push_str(&mut self, string: &str) {
        self.0.push_str(string)
    }

    /// Clears this `DOMString`, removing all contents.
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Shortens this String to the specified length.
    pub fn truncate(&mut self, new_len: usize) {
        self.0.truncate(new_len);
    }

    /// Removes newline characters according to <https://infra.spec.whatwg.org/#strip-newlines>.
    pub fn strip_newlines(&mut self) {
        self.0.retain(|c| c != '\r' && c != '\n');
    }

    /// Removes leading and trailing ASCII whitespaces according to
    /// <https://infra.spec.whatwg.org/#strip-leading-and-trailing-ascii-whitespace>.
    pub fn strip_leading_and_trailing_ascii_whitespace(&mut self) {
        if self.0.is_empty() {
            return;
        }

        let trailing_whitespace_len = self
            .0
            .trim_end_matches(|ref c| char::is_ascii_whitespace(c))
            .len();
        self.0.truncate(trailing_whitespace_len);
        if self.0.is_empty() {
            return;
        }

        let first_non_whitespace = self.0.find(|ref c| !char::is_ascii_whitespace(c)).unwrap();
        self.0.replace_range(0..first_non_whitespace, "");
    }

    /// Validates this `DOMString` is a time string according to
    /// <https://html.spec.whatwg.org/multipage/#valid-time-string>.
    pub fn is_valid_time_string(&self) -> bool {
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

    /// A valid date string should be "YYYY-MM-DD"
    /// YYYY must be four or more digits, MM and DD both must be two digits
    /// <https://html.spec.whatwg.org/multipage/#valid-date-string>
    pub fn is_valid_date_string(&self) -> bool {
        self.parse_date_string().is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#parse-a-date-string>
    pub fn parse_date_string(&self) -> Option<(i32, u32, u32)> {
        let value = &self.0;
        // Step 1, 2, 3
        let (year_int, month_int, day_int) = parse_date_component(value)?;

        // Step 4
        if value.split('-').nth(3).is_some() {
            return None;
        }

        // Step 5, 6
        Some((year_int, month_int, day_int))
    }

    /// <https://html.spec.whatwg.org/multipage/#parse-a-time-string>
    pub fn parse_time_string(&self) -> Option<(u32, u32, f64)> {
        let value = &self.0;
        // Step 1, 2, 3
        let (hour_int, minute_int, second_float) = parse_time_component(value)?;

        // Step 4
        if value.split(':').nth(3).is_some() {
            return None;
        }

        // Step 5, 6
        Some((hour_int, minute_int, second_float))
    }

    /// A valid month string should be "YYYY-MM"
    /// YYYY must be four or more digits, MM both must be two digits
    /// <https://html.spec.whatwg.org/multipage/#valid-month-string>
    pub fn is_valid_month_string(&self) -> bool {
        self.parse_month_string().is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#parse-a-month-string>
    pub fn parse_month_string(&self) -> Option<(i32, u32)> {
        let value = &self;
        // Step 1, 2, 3
        let (year_int, month_int) = parse_month_component(value)?;

        // Step 4
        if value.split('-').nth(2).is_some() {
            return None;
        }
        // Step 5
        Some((year_int, month_int))
    }

    /// A valid week string should be like {YYYY}-W{WW}, such as "2017-W52"
    /// YYYY must be four or more digits, WW both must be two digits
    /// <https://html.spec.whatwg.org/multipage/#valid-week-string>
    pub fn is_valid_week_string(&self) -> bool {
        self.parse_week_string().is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#parse-a-week-string>
    pub fn parse_week_string(&self) -> Option<(i32, u32)> {
        let value = &self.0;
        // Step 1, 2, 3
        let mut iterator = value.split('-');
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
        Some((year_int, week_int))
    }

    /// <https://html.spec.whatwg.org/multipage/#valid-floating-point-number>
    pub fn is_valid_floating_point_number_string(&self) -> bool {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^-?(?:\d+\.\d+|\d+|\.\d+)(?:(e|E)(\+|\-)?\d+)?$").unwrap();
        }
        RE.is_match(&self.0) && self.parse_floating_point_number().is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#rules-for-parsing-floating-point-number-values>
    pub fn parse_floating_point_number(&self) -> Option<f64> {
        // Steps 15-16 are telling us things about IEEE rounding modes
        // for floating-point significands; this code assumes the Rust
        // compiler already matches them in any cases where
        // that actually matters. They are not
        // related to f64::round(), which is for rounding to integers.
        let input = &self.0;
        if let Ok(val) = input.trim().parse::<f64>() {
            if !(
                // A valid number is the same as what rust considers to be valid,
                // except for +1., NaN, and Infinity.
                val.is_infinite() || val.is_nan() || input.ends_with('.') || input.starts_with('+')
            ) {
                return Some(val);
            }
        }
        None
    }

    /// Applies the same processing as `parse_floating_point_number` with some additional handling
    /// according to ECMA's string conversion steps.
    ///
    /// Used for specific elements when handling floating point values, namely the `number` and
    /// `range` inputs, as well as `meter` and `progress` elements.
    ///
    /// <https://html.spec.whatwg.org/multipage/#best-representation-of-the-number-as-a-floating-point-number>
    /// <https://tc39.es/ecma262/#sec-numeric-types-number-tostring>
    pub fn set_best_representation_of_the_floating_point_number(&mut self) {
        if let Some(val) = self.parse_floating_point_number() {
            // [tc39] Step 2: If x is either +0 or -0, return "0".
            let parsed_value = if val.is_zero() { 0.0_f64 } else { val };

            self.0 = parsed_value.to_string()
        }
    }

    /// A valid normalized local date and time string should be "{date}T{time}"
    /// where date and time are both valid, and the time string must be as short as possible
    /// <https://html.spec.whatwg.org/multipage/#valid-normalised-local-date-and-time-string>
    pub fn convert_valid_normalized_local_date_and_time_string(&mut self) -> Option<()> {
        let date = self.parse_local_date_and_time_string()?;
        if date.seconds == 0.0 {
            self.0 = format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}",
                date.year, date.month, date.day, date.hour, date.minute
            );
        } else if date.seconds < 10.0 {
            // we need exactly one leading zero on the seconds,
            // whatever their total string length might be
            self.0 = format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:0{}",
                date.year, date.month, date.day, date.hour, date.minute, date.seconds
            );
        } else {
            // we need no leading zeroes on the seconds
            self.0 = format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{}",
                date.year, date.month, date.day, date.hour, date.minute, date.seconds
            );
        }
        Some(())
    }

    /// <https://html.spec.whatwg.org/multipage/#parse-a-local-date-and-time-string>
    pub(crate) fn parse_local_date_and_time_string(&self) -> Option<ParsedDate> {
        let value = &self;
        // Step 1, 2, 4
        let mut iterator = if value.contains('T') {
            value.split('T')
        } else {
            value.split(' ')
        };

        // Step 3
        let date = iterator.next()?;
        let (year, month, day) = parse_date_component(date)?;

        // Step 5
        let time = iterator.next()?;
        let (hour, minute, seconds) = parse_time_component(time)?;

        // Step 6
        if iterator.next().is_some() {
            return None;
        }

        // Step 7, 8, 9
        Some(ParsedDate {
            year,
            month,
            day,
            hour,
            minute,
            seconds,
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#valid-e-mail-address>
    pub fn is_valid_email_address_string(&self) -> bool {
        lazy_static! {
            static ref RE: Regex = Regex::new(concat!(
                r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?",
                r"(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
            ))
            .unwrap();
        }
        RE.is_match(&self.0)
    }

    /// <https://html.spec.whatwg.org/multipage/#valid-simple-colour>
    pub fn is_valid_simple_color_string(&self) -> bool {
        let mut chars = self.0.chars();
        if self.0.len() == 7 && chars.next() == Some('#') {
            chars.all(|c| c.is_ascii_hexdigit())
        } else {
            false
        }
    }
}

impl Borrow<str> for DOMString {
    #[inline]
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Default for DOMString {
    fn default() -> Self {
        DOMString(String::new(), PhantomData)
    }
}

impl Deref for DOMString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &self.0
    }
}

impl DerefMut for DOMString {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl AsRef<str> for DOMString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DOMString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl PartialEq<str> for DOMString {
    fn eq(&self, other: &str) -> bool {
        &**self == other
    }
}

impl<'a> PartialEq<&'a str> for DOMString {
    fn eq(&self, other: &&'a str) -> bool {
        &**self == *other
    }
}

impl From<String> for DOMString {
    fn from(contents: String) -> DOMString {
        DOMString(contents, PhantomData)
    }
}

impl<'a> From<&'a str> for DOMString {
    fn from(contents: &str) -> DOMString {
        DOMString::from(String::from(contents))
    }
}

impl<'a> From<Cow<'a, str>> for DOMString {
    fn from(contents: Cow<'a, str>) -> DOMString {
        match contents {
            Cow::Owned(s) => DOMString::from(s),
            Cow::Borrowed(s) => DOMString::from(s),
        }
    }
}

impl From<DOMString> for LocalName {
    fn from(contents: DOMString) -> LocalName {
        LocalName::from(contents.0)
    }
}

impl From<DOMString> for Namespace {
    fn from(contents: DOMString) -> Namespace {
        Namespace::from(contents.0)
    }
}

impl From<DOMString> for Atom {
    fn from(contents: DOMString) -> Atom {
        Atom::from(contents.0)
    }
}

impl From<DOMString> for String {
    fn from(contents: DOMString) -> String {
        contents.0
    }
}

impl From<DOMString> for Vec<u8> {
    fn from(contents: DOMString) -> Vec<u8> {
        contents.0.into()
    }
}

impl<'a> From<DOMString> for Cow<'a, str> {
    fn from(contents: DOMString) -> Cow<'a, str> {
        contents.0.into()
    }
}

impl<'a> From<DOMString> for CowRcStr<'a> {
    fn from(contents: DOMString) -> CowRcStr<'a> {
        contents.0.into()
    }
}

impl Extend<char> for DOMString {
    fn extend<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = char>,
    {
        self.0.extend(iterable)
    }
}

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
fn parse_time_component(value: &str) -> Option<(u32, u32, f64)> {
    // Step 1
    let mut iterator = value.split(':');
    let hour = iterator.next()?;
    if hour.len() != 2 {
        return None;
    }
    let hour_int = hour.parse::<u32>().ok()?;

    // Step 2
    if hour_int > 23 {
        return None;
    }

    // Step 3, 4
    let minute = iterator.next()?;
    if minute.len() != 2 {
        return None;
    }
    let minute_int = minute.parse::<u32>().ok()?;

    // Step 5
    if minute_int > 59 {
        return None;
    }

    // Step 6, 7
    let second_float = match iterator.next() {
        Some(second) => {
            let mut second_iterator = second.split('.');
            if second_iterator.next()?.len() != 2 {
                return None;
            }
            if let Some(second_last) = second_iterator.next() {
                if second_last.len() > 3 {
                    return None;
                }
            }

            second.parse::<f64>().ok()?
        },
        None => 0.0,
    };

    // Step 8
    Some((hour_int, minute_int, second_float))
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
fn max_week_in_year(year: i32) -> u32 {
    Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0)
        .earliest()
        .map(|date_time| match date_time.weekday() {
            Weekday::Thu => 53,
            Weekday::Wed if is_leap_year(year) => 53,
            _ => 52,
        })
        .unwrap_or(52)
}

#[inline]
fn is_leap_year(year: i32) -> bool {
    year % 400 == 0 || (year % 4 == 0 && year % 100 != 0)
}

#[derive(Clone, Debug, Default, MallocSizeOf, PartialEq)]
pub(crate) struct ParsedDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub seconds: f64,
}
