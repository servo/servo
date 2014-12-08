/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use geometry::Au;

use cssparser::{mod, RGBAColor};
use std::ascii::AsciiExt;
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

pub fn split_html_space_chars<'a>(s: &'a str)
                                  -> Filter<'a, &'a str, CharSplits<'a, StaticCharVec>> {
    s.split(HTML_SPACE_CHARACTERS).filter(|&split| !split.is_empty())
}

/// Shared implementation to parse an integer according to
/// <http://www.whatwg.org/html/#rules-for-parsing-integers> or
/// <http://www.whatwg.org/html/#rules-for-parsing-non-negative-integers>.
fn do_parse_integer<T: Iterator<char>>(input: T) -> Option<i64> {
    fn is_ascii_digit(c: &char) -> bool {
        match *c {
            '0'...'9' => true,
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
            '0'...'9' => continue,
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

/// A "simple color" per HTML5 § 2.4.6.
#[deriving(Show)]
pub struct SimpleColor {
    /// The red component of the color, [0, 255].
    pub red: u8,
    /// The green component of the color, [0, 255].
    pub green: u8,
    /// The blue component of the color, [0, 255].
    pub blue: u8,
}

/// Parses a legacy color per HTML5 § 2.4.6. If unparseable, `Err` is returned.
pub fn parse_legacy_color(mut input: &str) -> Result<SimpleColor,()> {
    // Steps 1 and 2.
    if input.len() == 0 {
        return Err(())
    }

    // Step 3.
    input = input.trim_left_chars(Whitespace).trim_right_chars(Whitespace);

    // Step 4.
    if input.eq_ignore_ascii_case("transparent") {
        return Err(())
    }

    // Step 5.
    match cssparser::parse_color_keyword(input) {
        Ok(RGBAColor(rgba)) => {
            return Ok(SimpleColor {
                red: (rgba.red * 255.0) as u8,
                green: (rgba.green * 255.0) as u8,
                blue: (rgba.blue * 255.0) as u8,
            })
        }
        _ => {}
    }

    // Step 6.
    if input.len() == 4 {
        match (input.char_at(0),
               hex(input.char_at(1)),
               hex(input.char_at(2)),
               hex(input.char_at(3))) {
            ('#', Ok(r), Ok(g), Ok(b)) => {
                return Ok(SimpleColor {
                    red: r * 17,
                    green: g * 17,
                    blue: b * 17,
                })
            }
            _ => {}
        }
    }

    // Step 7.
    let mut new_input = String::new();
    for ch in input.chars() {
        if ch as u32 > 0xffff {
            new_input.push_str("00")
        } else {
            new_input.push(ch)
        }
    }
    let mut input = new_input.as_slice();

    // Step 8.
    if input.len() > 128 {
        input = input.slice_to(128)
    }

    // Step 9.
    if input.char_at(0) == '#' {
        input = input.slice_from(1)
    }

    // Step 10.
    let mut new_input = Vec::new();
    for ch in input.chars() {
        if hex(ch).is_ok() {
            new_input.push(ch as u8)
        } else {
            new_input.push(b'0')
        }
    }
    let mut input = new_input;

    // Step 11.
    while input.len() == 0 || (input.len() % 3) != 0 {
        input.push(b'0')
    }

    // Step 12.
    let mut length = input.len() / 3;
    let (mut red, mut green, mut blue) = (input.slice_to(length),
                                          input.slice(length, length * 2),
                                          input.slice_from(length * 2));

    // Step 13.
    if length > 8 {
        red = red.slice_from(length - 8);
        green = green.slice_from(length - 8);
        blue = blue.slice_from(length - 8);
        length = 8
    }

    // Step 14.
    while length > 2 && red[0] == b'0' && green[0] == b'0' && blue[0] == b'0' {
        red = red.slice_from(1);
        green = green.slice_from(1);
        blue = blue.slice_from(1);
        length -= 1
    }

    // Steps 15-20.
    return Ok(SimpleColor {
        red: (hex(red[0] as char).unwrap() << 4) | hex(red[1] as char).unwrap(),
        green: (hex(green[0] as char).unwrap() << 4) | hex(green[1] as char).unwrap(),
        blue: (hex(blue[0] as char).unwrap() << 4) | hex(blue[1] as char).unwrap(),
    });

    fn hex(ch: char) -> Result<u8,()> {
        match ch {
            '0'...'9' => Ok((ch as u8) - b'0'),
            'a'...'f' => Ok((ch as u8) - b'a' + 10),
            'A'...'F' => Ok((ch as u8) - b'A' + 10),
            _ => Err(()),
        }
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
