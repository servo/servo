/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use cssparser::{self, Color, RGBA};
use euclid::num::Zero;
use num::ToPrimitive;
use std::ascii::AsciiExt;
use std::ops::Deref;
use std::str::FromStr;
use string_cache::{Atom, Namespace};
use url::Url;
use util::str::{DOMString, LengthOrPercentageOrAuto, HTML_SPACE_CHARACTERS, WHITESPACE};
use util::str::{read_numbers, split_html_space_chars, str_join};
use values::specified::{Length};

// Duplicated from script::dom::values.
const UNSIGNED_LONG_MAX: u32 = 2147483647;

#[derive(PartialEq, Clone, HeapSizeOf)]
pub enum AttrValue {
    String(DOMString),
    TokenList(DOMString, Vec<Atom>),
    UInt(DOMString, u32),
    Int(DOMString, i32),
    Atom(Atom),
    Length(DOMString, Option<Length>),
    Color(DOMString, Option<RGBA>),
    Dimension(DOMString, LengthOrPercentageOrAuto),
    Url(DOMString, Option<Url>),
}

/// Shared implementation to parse an integer according to
/// <https://html.spec.whatwg.org/multipage/#rules-for-parsing-integers> or
/// <https://html.spec.whatwg.org/multipage/#rules-for-parsing-non-negative-integers>
fn do_parse_integer<T: Iterator<Item=char>>(input: T) -> Option<i64> {
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

    let value = read_numbers(input);

    value.and_then(|value| value.checked_mul(sign))
}

/// Parse an integer according to
/// <https://html.spec.whatwg.org/multipage/#rules-for-parsing-integers>.
pub fn parse_integer<T: Iterator<Item=char>>(input: T) -> Option<i32> {
    do_parse_integer(input).and_then(|result| {
        result.to_i32()
    })
}

/// Parse an integer according to
/// <https://html.spec.whatwg.org/multipage/#rules-for-parsing-non-negative-integers>
pub fn parse_unsigned_integer<T: Iterator<Item=char>>(input: T) -> Option<u32> {
    do_parse_integer(input).and_then(|result| {
        result.to_u32()
    })
}

impl AttrValue {
    pub fn from_serialized_tokenlist(tokens: DOMString) -> AttrValue {
        let atoms =
            split_html_space_chars(&tokens)
            .map(Atom::from)
            .fold(vec![], |mut acc, atom| {
                if !acc.contains(&atom) { acc.push(atom) }
                acc
            });
        AttrValue::TokenList(tokens, atoms)
    }

    pub fn from_atomic_tokens(atoms: Vec<Atom>) -> AttrValue {
        // TODO(ajeffrey): effecient conversion of Vec<Atom> to DOMString
        let tokens = DOMString::from(str_join(&atoms, "\x20"));
        AttrValue::TokenList(tokens, atoms)
    }

    // https://html.spec.whatwg.org/multipage/#reflecting-content-attributes-in-idl-attributes:idl-unsigned-long
    pub fn from_u32(string: DOMString, default: u32) -> AttrValue {
        let result = parse_unsigned_integer(string.chars()).unwrap_or(default);
        let result = if result > UNSIGNED_LONG_MAX {
            default
        } else {
            result
        };
        AttrValue::UInt(string, result)
    }

    pub fn from_i32(string: DOMString, default: i32) -> AttrValue {
        let result = parse_integer(string.chars()).unwrap_or(default);
        AttrValue::Int(string, result)
    }

    // https://html.spec.whatwg.org/multipage/#limited-to-only-non-negative-numbers
    pub fn from_limited_i32(string: DOMString, default: i32) -> AttrValue {
        let result = parse_integer(string.chars()).unwrap_or(default);

        if result < 0 {
            AttrValue::Int(string, default)
        } else {
            AttrValue::Int(string, result)
        }
    }

    // https://html.spec.whatwg.org/multipage/#limited-to-only-non-negative-numbers-greater-than-zero
    pub fn from_limited_u32(string: DOMString, default: u32) -> AttrValue {
        let result = parse_unsigned_integer(string.chars()).unwrap_or(default);
        let result = if result == 0 || result > UNSIGNED_LONG_MAX {
            default
        } else {
            result
        };
        AttrValue::UInt(string, result)
    }

    pub fn from_atomic(string: DOMString) -> AttrValue {
        let value = Atom::from(string);
        AttrValue::Atom(value)
    }

    pub fn from_url(base: &Url, url: DOMString) -> AttrValue {
        let joined = base.join(&url).ok();
        AttrValue::Url(url, joined)
    }

    pub fn from_legacy_color(string: DOMString) -> AttrValue {
        let parsed = parse_legacy_color(&string).ok();
        AttrValue::Color(string, parsed)
    }

    pub fn from_dimension(string: DOMString) -> AttrValue {
        let parsed = parse_length(&string);
        AttrValue::Dimension(string, parsed)
    }

    pub fn from_nonzero_dimension(string: DOMString) -> AttrValue {
        let parsed = parse_nonzero_length(&string);
        AttrValue::Dimension(string, parsed)
    }

    /// Assumes the `AttrValue` is a `TokenList` and returns its tokens
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `TokenList`
    pub fn as_tokens(&self) -> &[Atom] {
        match *self {
            AttrValue::TokenList(_, ref tokens) => tokens,
            _ => panic!("Tokens not found"),
        }
    }

    /// Assumes the `AttrValue` is an `Atom` and returns its value
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not an `Atom`
    pub fn as_atom(&self) -> &Atom {
        match *self {
            AttrValue::Atom(ref value) => value,
            _ => panic!("Atom not found"),
        }
    }

    /// Assumes the `AttrValue` is a `Color` and returns its value
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `Color`
    pub fn as_color(&self) -> Option<&RGBA> {
        match *self {
            AttrValue::Color(_, ref color) => color.as_ref(),
            _ => panic!("Color not found"),
        }
    }

    /// Assumes the `AttrValue` is a `Length` and returns its value
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `Length`
    pub fn as_length(&self) -> Option<&Length> {
        match *self {
            AttrValue::Length(_, ref length) => length.as_ref(),
            _ => panic!("Length not found"),
        }
    }

    /// Assumes the `AttrValue` is a `Dimension` and returns its value
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `Dimension`
    pub fn as_dimension(&self) -> &LengthOrPercentageOrAuto {
        match *self {
            AttrValue::Dimension(_, ref l) => l,
            _ => panic!("Dimension not found"),
        }
    }

    /// Assumes the `AttrValue` is a `Url` and returns its value
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `Url`
    pub fn as_url(&self) -> Option<&Url> {
        match *self {
            AttrValue::Url(_, ref url) => url.as_ref(),
            _ => panic!("Url not found"),
        }
    }

    /// Return the AttrValue as its integer representation, if any.
    /// This corresponds to attribute values returned as `AttrValue::UInt(_)`
    /// by `VirtualMethods::parse_plain_attribute()`.
    ///
    /// ## Panics
    ///
    /// Panics if the `AttrValue` is not a `UInt`
    pub fn as_uint(&self) -> u32 {
        if let AttrValue::UInt(_, value) = *self {
            value
        } else {
            panic!("Uint not found");
        }
    }
}

impl Deref for AttrValue {
    type Target = str;

    fn deref(&self) -> &str {
        match *self {
            AttrValue::String(ref value) |
                AttrValue::TokenList(ref value, _) |
                AttrValue::UInt(ref value, _) |
                AttrValue::Length(ref value, _) |
                AttrValue::Color(ref value, _) |
                AttrValue::Int(ref value, _) |
                AttrValue::Url(ref value, _) |
                AttrValue::Dimension(ref value, _) => &value,
            AttrValue::Atom(ref value) => &value,
        }
    }
}

/// HTML5 § 2.4.4.5.
///
/// https://html.spec.whatwg.org/multipage/#rules-for-parsing-non-zero-dimension-values
pub fn parse_nonzero_length(value: &str) -> LengthOrPercentageOrAuto {
    match parse_length(value) {
        LengthOrPercentageOrAuto::Length(x) if x == Au::zero() => LengthOrPercentageOrAuto::Auto,
        LengthOrPercentageOrAuto::Percentage(0.) => LengthOrPercentageOrAuto::Auto,
        x => x,
    }
}

/// Parses a legacy color per HTML5 § 2.4.6. If unparseable, `Err` is returned.
pub fn parse_legacy_color(mut input: &str) -> Result<RGBA, ()> {
    // Steps 1 and 2.
    if input.is_empty() {
        return Err(())
    }

    // Step 3.
    input = input.trim_matches(WHITESPACE);

    // Step 4.
    if input.eq_ignore_ascii_case("transparent") {
        return Err(())
    }

    // Step 5.
    if let Ok(Color::RGBA(rgba)) = cssparser::parse_color_keyword(input) {
        return Ok(rgba);
    }

    // Step 6.
    if input.len() == 4 {
        if let (b'#', Ok(r), Ok(g), Ok(b)) =
                (input.as_bytes()[0],
                hex(input.as_bytes()[1] as char),
                hex(input.as_bytes()[2] as char),
                hex(input.as_bytes()[3] as char)) {
            return Ok(RGBA {
                red: (r as f32) * 17.0 / 255.0,
                green: (g as f32) * 17.0 / 255.0,
                blue: (b as f32) * 17.0 / 255.0,
                alpha: 1.0,
            })
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
    let mut input = &*new_input;

    // Step 8.
    for (char_count, (index, _)) in input.char_indices().enumerate() {
        if char_count == 128 {
            input = &input[..index];
            break
        }
    }

    // Step 9.
    if input.as_bytes()[0] == b'#' {
        input = &input[1..]
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
    while input.is_empty() || (input.len() % 3) != 0 {
        input.push(b'0')
    }

    // Step 12.
    let mut length = input.len() / 3;
    let (mut red, mut green, mut blue) = (&input[..length],
                                          &input[length..length * 2],
                                          &input[length * 2..]);

    // Step 13.
    if length > 8 {
        red = &red[length - 8..];
        green = &green[length - 8..];
        blue = &blue[length - 8..];
        length = 8
    }

    // Step 14.
    while length > 2 && red[0] == b'0' && green[0] == b'0' && blue[0] == b'0' {
        red = &red[1..];
        green = &green[1..];
        blue = &blue[1..];
        length -= 1
    }

    // Steps 15-20.
    return Ok(RGBA {
        red: hex_string(red).unwrap() as f32 / 255.0,
        green: hex_string(green).unwrap() as f32 / 255.0,
        blue: hex_string(blue).unwrap() as f32 / 255.0,
        alpha: 1.0,
    });

    fn hex(ch: char) -> Result<u8, ()> {
        match ch {
            '0'...'9' => Ok((ch as u8) - b'0'),
            'a'...'f' => Ok((ch as u8) - b'a' + 10),
            'A'...'F' => Ok((ch as u8) - b'A' + 10),
            _ => Err(()),
        }
    }

    fn hex_string(string: &[u8]) -> Result<u8, ()> {
        match string.len() {
            0 => Err(()),
            1 => hex(string[0] as char),
            _ => {
                let upper = try!(hex(string[0] as char));
                let lower = try!(hex(string[1] as char));
                Ok((upper << 4) | lower)
            }
        }
    }
}

/// TODO: this function can be rewritten to return Result<LengthOrPercentage, _>
/// Parses a dimension value per HTML5 § 2.4.4.4. If unparseable, `Auto` is
/// returned.
/// https://html.spec.whatwg.org/multipage/#rules-for-parsing-dimension-values
pub fn parse_length(mut value: &str) -> LengthOrPercentageOrAuto {
    // Steps 1 & 2 are not relevant

    // Step 3
    value = value.trim_left_matches(WHITESPACE);

    // Step 4
    if value.is_empty() {
        return LengthOrPercentageOrAuto::Auto
    }

    // Step 5
    if value.starts_with("+") {
        value = &value[1..]
    }

    // Steps 6 & 7
    match value.chars().nth(0) {
        Some('0'...'9') => {},
        _ => return LengthOrPercentageOrAuto::Auto,
    }

    // Steps 8 to 13
    // We trim the string length to the minimum of:
    // 1. the end of the string
    // 2. the first occurence of a '%' (U+0025 PERCENT SIGN)
    // 3. the second occurrence of a '.' (U+002E FULL STOP)
    // 4. the occurrence of a character that is neither a digit nor '%' nor '.'
    // Note: Step 10 is directly subsumed by FromStr::from_str
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
    value = &value[..end_index];

    if found_percent {
        let result: Result<f32, _> = FromStr::from_str(value);
        match result {
            Ok(number) => return LengthOrPercentageOrAuto::Percentage((number as f32) / 100.0),
            Err(_) => return LengthOrPercentageOrAuto::Auto,
        }
    }

    match FromStr::from_str(value) {
        Ok(number) => LengthOrPercentageOrAuto::Length(Au::from_f64_px(number)),
        Err(_) => LengthOrPercentageOrAuto::Auto,
    }
}

#[derive(Clone, HeapSizeOf, Debug)]
pub struct AttrIdentifier {
    pub local_name: Atom,
    pub name: Atom,
    pub namespace: Namespace,
    pub prefix: Option<Atom>,
}
