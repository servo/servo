/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! String utils for attributes and similar stuff.

#![deny(missing_docs)]

use num_traits::ToPrimitive;
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::convert::AsRef;
use std::iter::{Filter, Peekable};
use std::str::Split;

/// A static slice of characters.
pub type StaticCharVec = &'static [char];

/// A static slice of `str`s.
pub type StaticStringVec = &'static [&'static str];

/// A "space character" according to:
///
/// https://html.spec.whatwg.org/multipage/#space-character
pub static HTML_SPACE_CHARACTERS: StaticCharVec = &[
    '\u{0020}',
    '\u{0009}',
    '\u{000a}',
    '\u{000c}',
    '\u{000d}',
];

/// Whether a character is a HTML whitespace character.
#[inline]
pub fn char_is_whitespace(c: char) -> bool {
    HTML_SPACE_CHARACTERS.contains(&c)
}

/// Whether all the string is HTML whitespace.
#[inline]
pub fn is_whitespace(s: &str) -> bool {
    s.chars().all(char_is_whitespace)
}

#[inline]
fn not_empty(&split: &&str) -> bool { !split.is_empty() }

/// Split a string on HTML whitespace.
#[inline]
pub fn split_html_space_chars<'a>(s: &'a str) ->
                                  Filter<Split<'a, StaticCharVec>, fn(&&str) -> bool> {
    s.split(HTML_SPACE_CHARACTERS).filter(not_empty as fn(&&str) -> bool)
}

/// Split a string on commas.
#[inline]
pub fn split_commas<'a>(s: &'a str) -> Filter<Split<'a, char>, fn(&&str) -> bool> {
    s.split(',').filter(not_empty as fn(&&str) -> bool)
}

fn is_ascii_digit(c: &char) -> bool {
    match *c {
        '0'...'9' => true,
        _ => false,
    }
}

fn is_decimal_point(c: char) -> bool {
    c == '.'
}

fn is_exponent_char(c: char) -> bool {
    match c {
        'e' | 'E' => true,
        _ => false,
    }
}

/// Read a set of ascii digits and read them into a number.
pub fn read_numbers<I: Iterator<Item=char>>(mut iter: Peekable<I>) -> (Option<i64>, usize) {
    match iter.peek() {
        Some(c) if is_ascii_digit(c) => (),
        _ => return (None, 0),
    }

    iter.take_while(is_ascii_digit).map(|d| {
        d as i64 - '0' as i64
    }).fold((Some(0i64), 0), |accumulator, d| {
        let digits = accumulator.0.and_then(|accumulator| {
            accumulator.checked_mul(10)
        }).and_then(|accumulator| {
            accumulator.checked_add(d)
        });
        (digits, accumulator.1 + 1)
    })
}

/// Read a decimal fraction.
pub fn read_fraction<I: Iterator<Item=char>>(mut iter: Peekable<I>,
                                             mut divisor: f64,
                                             value: f64) -> (f64, usize) {
    match iter.peek() {
        Some(c) if is_decimal_point(*c) => (),
        _ => return (value, 0),
    }
    iter.next();

    iter.take_while(is_ascii_digit).map(|d|
        d as i64 - '0' as i64
    ).fold((value, 1), |accumulator, d| {
        divisor *= 10f64;
        (accumulator.0 + d as f64 / divisor, accumulator.1 + 1)
    })
}

/// Reads an exponent from an iterator over chars, for example `e100`.
pub fn read_exponent<I: Iterator<Item=char>>(mut iter: Peekable<I>) -> Option<i32> {
    match iter.peek() {
        Some(c) if is_exponent_char(*c) => (),
        _ => return None,
    }
    iter.next();

    match iter.peek() {
        None => None,
        Some(&'-') => {
            iter.next();
            read_numbers(iter).0.map(|exp| -exp.to_i32().unwrap_or(0))
        }
        Some(&'+') => {
            iter.next();
            read_numbers(iter).0.map(|exp| exp.to_i32().unwrap_or(0))
        }
        Some(_) => read_numbers(iter).0.map(|exp| exp.to_i32().unwrap_or(0))
    }
}

/// Join a set of strings with a given delimiter `join`.
pub fn str_join<I, T>(strs: I, join: &str) -> String
    where I: IntoIterator<Item=T>,
          T: AsRef<str>,
{
    strs.into_iter().enumerate().fold(String::new(), |mut acc, (i, s)| {
        if i > 0 { acc.push_str(join); }
        acc.push_str(s.as_ref());
        acc
    })
}

/// Like AsciiExt::to_ascii_lowercase, but avoids allocating when the input is already lower-case.
pub fn cow_into_ascii_lowercase<'a, S: Into<Cow<'a, str>>>(s: S) -> Cow<'a, str> {
    let mut cow = s.into();
    if let Some(first_uppercase) = cow.bytes().position(|byte| byte >= b'A' && byte <= b'Z') {
        cow.to_mut()[first_uppercase..].make_ascii_lowercase();
    }
    cow
}
