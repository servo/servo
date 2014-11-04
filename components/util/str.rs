/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use geometry::Au;

use std::from_str::FromStr;
use std::iter::Filter;
use std::str::{CharEq, CharSplits};
use unicode::char::to_lowercase;

pub type DOMString = String;
pub type StaticCharVec = &'static [char];
pub type StaticStringVec = &'static [&'static str];

pub fn null_str_as_empty(s: &Option<DOMString>) -> DOMString {
    // We don't use map_default because it would allocate "".to_string() even for Some.
    match *s {
        Some(ref s) => s.clone(),
        None => "".to_string()
    }
}

pub fn null_str_as_empty_ref<'a>(s: &'a Option<DOMString>) -> &'a str {
    match *s {
        Some(ref s) => s.as_slice(),
        None => ""
    }
}

/// Whitespace as defined by HTML5 § 2.4.1.
struct Whitespace;

impl CharEq for Whitespace {
    #[inline]
    fn matches(&mut self, ch: char) -> bool {
        match ch {
            ' ' | '\t' | '\x0a' | '\x0c' | '\x0d' => true,
            _ => false,
        }
    }

    #[inline]
    fn only_ascii(&self) -> bool {
        true
    }
}

pub fn is_whitespace(s: &str) -> bool {
    s.chars().all(|c| Whitespace.matches(c))
}

/// A "space character" according to:
///
/// http://www.whatwg.org/specs/web-apps/current-work/multipage/common-microsyntaxes.html#space-character
pub static HTML_SPACE_CHARACTERS: StaticCharVec = &[
    '\u0020',
    '\u0009',
    '\u000a',
    '\u000c',
    '\u000d',
];

pub fn split_html_space_chars<'a>(s: &'a str) -> Filter<'a, &'a str, CharSplits<'a, StaticCharVec>> {
    s.split(HTML_SPACE_CHARACTERS).filter(|&split| !split.is_empty())
}

/// Shared implementation to parse an integer according to
/// <http://www.whatwg.org/html/#rules-for-parsing-integers> or
/// <http://www.whatwg.org/html/#rules-for-parsing-non-negative-integers>.
fn do_parse_integer<T: Iterator<char>>(input: T) -> Option<i64> {
    fn is_ascii_digit(c: &char) -> bool {
        match *c {
            '0'..'9' => true,
            _ => false,
        }
    }


    let mut input = input.skip_while(|c| {
        HTML_SPACE_CHARACTERS.iter().any(|s| s == c)
    }).peekable();

    let sign = match input.peek() {
        None => return None,
        Some(&'-') => {
            input.next();
            -1
        },
        Some(&'+') => {
            input.next();
            1
        },
        Some(_) => 1,
    };

    match input.peek() {
        Some(c) if is_ascii_digit(c) => (),
        _ => return None,
    }

    let value = input.take_while(is_ascii_digit).map(|d| {
        d as i64 - '0' as i64
    }).fold(Some(0i64), |accumulator, d| {
        accumulator.and_then(|accumulator| {
            accumulator.checked_mul(&10)
        }).and_then(|accumulator| {
            accumulator.checked_add(&d)
        })
    });

    return value.and_then(|value| value.checked_mul(&sign));
}

/// Parse an integer according to
/// <http://www.whatwg.org/html/#rules-for-parsing-integers>.
pub fn parse_integer<T: Iterator<char>>(input: T) -> Option<i32> {
    do_parse_integer(input).and_then(|result| {
        result.to_i32()
    })
}

/// Parse an integer according to
/// <http://www.whatwg.org/html/#rules-for-parsing-non-negative-integers>.
pub fn parse_unsigned_integer<T: Iterator<char>>(input: T) -> Option<u32> {
    do_parse_integer(input).and_then(|result| {
        result.to_u32()
    })
}

pub enum LengthOrPercentageOrAuto {
    AutoLpa,
    PercentageLpa(f64),
    LengthLpa(Au),
}

/// Parses a length per HTML5 § 2.4.4.4. If unparseable, `AutoLpa` is returned.
pub fn parse_length(mut value: &str) -> LengthOrPercentageOrAuto {
    value = value.trim_left_chars(Whitespace);
    if value.len() == 0 {
        return AutoLpa
    }
    if value.starts_with("+") {
        value = value.slice_from(1)
    }
    value = value.trim_left_chars('0');
    if value.len() == 0 {
        return AutoLpa
    }

    let mut end_index = value.len();
    let (mut found_full_stop, mut found_percent) = (false, false);
    for (i, ch) in value.chars().enumerate() {
        match ch {
            '0'..'9' => continue,
            '%' => {
                found_percent = true;
                end_index = i;
                break
            }
            '.' if !found_full_stop => {
                found_full_stop = true;
                continue
            }
            _ => {
                end_index = i;
                break
            }
        }
    }
    value = value.slice_to(end_index);

    if found_percent {
        let result: Option<f64> = FromStr::from_str(value);
        match result {
            Some(number) => return PercentageLpa((number as f64) / 100.0),
            None => return AutoLpa,
        }
    }

    match FromStr::from_str(value) {
        Some(number) => LengthLpa(Au::from_px(number)),
        None => AutoLpa,
    }
}


#[deriving(Clone, Eq, PartialEq, Hash, Show)]
pub struct LowercaseString {
    inner: String,
}

impl LowercaseString {
    pub fn new(s: &str) -> LowercaseString {
        LowercaseString {
            inner: s.chars().map(to_lowercase).collect(),
        }
    }
}

impl Str for LowercaseString {
    #[inline]
    fn as_slice(&self) -> &str {
        self.inner.as_slice()
    }
}
