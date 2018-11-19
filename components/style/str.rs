/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! String utils for attributes and similar stuff.

#![deny(missing_docs)]

use num_traits::ToPrimitive;
use std::borrow::Cow;
use std::convert::AsRef;
use std::fmt::{self, Write};
use std::iter::{Filter, Peekable};
use std::str::Split;

/// A static slice of characters.
pub type StaticCharVec = &'static [char];

/// A static slice of `str`s.
pub type StaticStringVec = &'static [&'static str];

/// A "space character" according to:
///
/// <https://html.spec.whatwg.org/multipage/#space-character>
pub static HTML_SPACE_CHARACTERS: StaticCharVec =
    &['\u{0020}', '\u{0009}', '\u{000a}', '\u{000c}', '\u{000d}'];

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
fn not_empty(&split: &&str) -> bool {
    !split.is_empty()
}

/// Split a string on HTML whitespace.
#[inline]
pub fn split_html_space_chars<'a>(
    s: &'a str,
) -> Filter<Split<'a, StaticCharVec>, fn(&&str) -> bool> {
    s.split(HTML_SPACE_CHARACTERS)
        .filter(not_empty as fn(&&str) -> bool)
}

/// Split a string on commas.
#[inline]
pub fn split_commas<'a>(s: &'a str) -> Filter<Split<'a, char>, fn(&&str) -> bool> {
    s.split(',').filter(not_empty as fn(&&str) -> bool)
}

/// Character is ascii digit
pub fn is_ascii_digit(c: &char) -> bool {
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
pub fn read_numbers<I: Iterator<Item = char>>(mut iter: Peekable<I>) -> (Option<i64>, usize) {
    match iter.peek() {
        Some(c) if is_ascii_digit(c) => (),
        _ => return (None, 0),
    }

    iter.take_while(is_ascii_digit)
        .map(|d| d as i64 - '0' as i64)
        .fold((Some(0i64), 0), |accumulator, d| {
            let digits = accumulator
                .0
                .and_then(|accumulator| accumulator.checked_mul(10))
                .and_then(|accumulator| accumulator.checked_add(d));
            (digits, accumulator.1 + 1)
        })
}

/// Read a decimal fraction.
pub fn read_fraction<I: Iterator<Item = char>>(
    mut iter: Peekable<I>,
    mut divisor: f64,
    value: f64,
) -> (f64, usize) {
    match iter.peek() {
        Some(c) if is_decimal_point(*c) => (),
        _ => return (value, 0),
    }
    iter.next();

    iter.take_while(is_ascii_digit)
        .map(|d| d as i64 - '0' as i64)
        .fold((value, 1), |accumulator, d| {
            divisor *= 10f64;
            (accumulator.0 + d as f64 / divisor, accumulator.1 + 1)
        })
}

/// Reads an exponent from an iterator over chars, for example `e100`.
pub fn read_exponent<I: Iterator<Item = char>>(mut iter: Peekable<I>) -> Option<i32> {
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
        },
        Some(&'+') => {
            iter.next();
            read_numbers(iter).0.map(|exp| exp.to_i32().unwrap_or(0))
        },
        Some(_) => read_numbers(iter).0.map(|exp| exp.to_i32().unwrap_or(0)),
    }
}

/// Join a set of strings with a given delimiter `join`.
pub fn str_join<I, T>(strs: I, join: &str) -> String
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    strs.into_iter()
        .enumerate()
        .fold(String::new(), |mut acc, (i, s)| {
            if i > 0 {
                acc.push_str(join);
            }
            acc.push_str(s.as_ref());
            acc
        })
}

/// Returns true if a given string has a given prefix with case-insensitive match.
pub fn starts_with_ignore_ascii_case(string: &str, prefix: &str) -> bool {
    string.len() >= prefix.len() &&
        string.as_bytes()[0..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes())
}

/// Returns an ascii lowercase version of a string, only allocating if needed.
pub fn string_as_ascii_lowercase<'a>(input: &'a str) -> Cow<'a, str> {
    if input.bytes().any(|c| matches!(c, b'A'...b'Z')) {
        input.to_ascii_lowercase().into()
    } else {
        // Already ascii lowercase.
        Cow::Borrowed(input)
    }
}

/// To avoid accidentally instantiating multiple monomorphizations of large
/// serialization routines, we define explicit concrete types and require
/// them in those routines. This primarily avoids accidental mixing of UTF8
/// with UTF16 serializations in Gecko.
#[cfg(feature = "gecko")]
pub type CssStringWriter = ::nsstring::nsAString;

/// String type that coerces to CssStringWriter, used when serialization code
/// needs to allocate a temporary string.
#[cfg(feature = "gecko")]
pub type CssString = ::nsstring::nsString;

/// Certain serialization code needs to interact with borrowed strings, which
/// are sometimes native UTF8 Rust strings, and other times serialized UTF16
/// strings. This enum multiplexes the two cases.
#[cfg(feature = "gecko")]
pub enum CssStringBorrow<'a> {
    /// A borrow of a UTF16 CssString.
    UTF16(&'a ::nsstring::nsString),
    /// A borrow of a regular Rust UTF8 string.
    UTF8(&'a str),
}

#[cfg(feature = "gecko")]
impl<'a> CssStringBorrow<'a> {
    /// Writes the borrowed string to the provided writer.
    pub fn append_to(&self, dest: &mut CssStringWriter) -> fmt::Result {
        match *self {
            CssStringBorrow::UTF16(s) => {
                dest.append(s);
                Ok(())
            },
            CssStringBorrow::UTF8(s) => dest.write_str(s),
        }
    }

    /// Returns true of the borrowed string is empty.
    pub fn is_empty(&self) -> bool {
        match *self {
            CssStringBorrow::UTF16(s) => s.is_empty(),
            CssStringBorrow::UTF8(s) => s.is_empty(),
        }
    }
}

#[cfg(feature = "gecko")]
impl<'a> From<&'a str> for CssStringBorrow<'a> {
    fn from(s: &'a str) -> Self {
        CssStringBorrow::UTF8(s)
    }
}

#[cfg(feature = "gecko")]
impl<'a> From<&'a ::nsstring::nsString> for CssStringBorrow<'a> {
    fn from(s: &'a ::nsstring::nsString) -> Self {
        CssStringBorrow::UTF16(s)
    }
}

/// String. The comments for the Gecko types explain the need for this abstraction.
#[cfg(feature = "servo")]
pub type CssStringWriter = String;

/// String. The comments for the Gecko types explain the need for this abstraction.
#[cfg(feature = "servo")]
pub type CssString = String;

/// Borrowed string. The comments for the Gecko types explain the need for this abstraction.
#[cfg(feature = "servo")]
pub struct CssStringBorrow<'a>(&'a str);

#[cfg(feature = "servo")]
impl<'a> CssStringBorrow<'a> {
    /// Appends the borrowed string to the given string.
    pub fn append_to(&self, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str(self.0)
    }

    /// Returns true if the borrowed string is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(feature = "servo")]
impl<'a> From<&'a str> for CssStringBorrow<'a> {
    fn from(s: &'a str) -> Self {
        CssStringBorrow(s)
    }
}

#[cfg(feature = "servo")]
impl<'a> From<&'a String> for CssStringBorrow<'a> {
    fn from(s: &'a String) -> Self {
        CssStringBorrow(&*s)
    }
}
