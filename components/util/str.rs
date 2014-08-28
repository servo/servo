/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::iter::Filter;
use std::str::CharSplits;

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

pub fn is_whitespace(s: &str) -> bool {
    s.chars().all(|c| match c {
        '\u0020' | '\u0009' | '\u000D' | '\u000A' => true,
        _ => false
    })
}

/// A "space character" according to:
///
///     http://www.whatwg.org/specs/web-apps/current-work/multipage/common-microsyntaxes.html#
///     space-character
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
    fn as_ascii_digit(c: char) -> Option<i64> {
        match c {
            '0'..'9' => Some(c as i64 - '0' as i64),
            _ => None,
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
        Some(&c) if as_ascii_digit(c).is_some() => (),
        _ => return None,
    }

    let value = input.filter_map(as_ascii_digit).fuse().fold(Some(0i64), |accumulator, d| {
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
