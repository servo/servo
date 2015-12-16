/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use cssparser::{self, Color, RGBA};
use js::conversions::{FromJSValConvertible, ToJSValConvertible, latin1_to_string};
use js::jsapi::{JSContext, JSString, HandleValue, MutableHandleValue};
use js::jsapi::{JS_GetTwoByteStringCharsAndLength, JS_StringHasLatin1Chars};
use js::rust::ToString;
use libc::c_char;
use num_lib::ToPrimitive;
use opts;
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::char;
use std::convert::AsRef;
use std::ffi::CStr;
use std::fmt;
use std::iter::{Filter, Peekable};
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::slice;
use std::str::{CharIndices, FromStr, Split, from_utf8};

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Deserialize, Serialize, Hash, Debug)]
pub struct DOMString(String);

impl !Send for DOMString {}

impl DOMString {
    pub fn new() -> DOMString {
        DOMString(String::new())
    }
    // FIXME(ajeffrey): implement more of the String methods on DOMString?
    pub fn push_str(&mut self, string: &str) {
        self.0.push_str(string)
    }
    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl Default for DOMString {
    fn default() -> Self {
        DOMString(String::new())
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
        DOMString(contents)
    }
}

impl<'a> From<&'a str> for DOMString {
    fn from(contents: &str) -> DOMString {
        DOMString::from(String::from(contents))
    }
}

impl From<DOMString> for String {
    fn from(contents: DOMString) -> String {
        contents.0
    }
}

impl Into<Vec<u8>> for DOMString {
    fn into(self) -> Vec<u8> {
        self.0.into()
    }
}

// https://heycam.github.io/webidl/#es-DOMString
impl ToJSValConvertible for DOMString {
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        (**self).to_jsval(cx, rval);
    }
}

/// Behavior for stringification of `JSVal`s.
#[derive(PartialEq)]
pub enum StringificationBehavior {
    /// Convert `null` to the string `"null"`.
    Default,
    /// Convert `null` to the empty string.
    Empty,
}

/// Convert the given `JSString` to a `DOMString`. Fails if the string does not
/// contain valid UTF-16.
pub unsafe fn jsstring_to_str(cx: *mut JSContext, s: *mut JSString) -> DOMString {
    let latin1 = JS_StringHasLatin1Chars(s);
    DOMString(if latin1 {
        latin1_to_string(cx, s)
    } else {
        let mut length = 0;
        let chars = JS_GetTwoByteStringCharsAndLength(cx, ptr::null(), s, &mut length);
        assert!(!chars.is_null());
        let potentially_ill_formed_utf16 = slice::from_raw_parts(chars, length as usize);
        let mut s = String::with_capacity(length as usize);
        for item in char::decode_utf16(potentially_ill_formed_utf16.iter().cloned()) {
            match item {
                Ok(c) => s.push(c),
                Err(_) => {
                    // FIXME: Add more info like document URL in the message?
                    macro_rules! message {
                        () => {
                            "Found an unpaired surrogate in a DOM string. \
                             If you see this in real web content, \
                             please comment on https://github.com/servo/servo/issues/6564"
                        }
                    }
                    if opts::get().replace_surrogates {
                        error!(message!());
                        s.push('\u{FFFD}');
                    } else {
                        panic!(concat!(message!(), " Use `-Z replace-surrogates` \
                            on the command line to make this non-fatal."));
                    }
                }
            }
        }
        s
    })
}

// https://heycam.github.io/webidl/#es-DOMString
impl FromJSValConvertible for DOMString {
    type Config = StringificationBehavior;
    unsafe fn from_jsval(cx: *mut JSContext,
                         value: HandleValue,
                         null_behavior: StringificationBehavior)
                         -> Result<DOMString, ()> {
        if null_behavior == StringificationBehavior::Empty &&
           value.get().is_null() {
            Ok(DOMString::new())
        } else {
            let jsstr = ToString(cx, value);
            if jsstr.is_null() {
                debug!("ToString failed");
                Err(())
            } else {
                Ok(jsstring_to_str(cx, jsstr))
            }
        }
    }
}

impl Extend<char> for DOMString {
    fn extend<I>(&mut self, iterable: I) where I: IntoIterator<Item=char> {
        self.0.extend(iterable)
    }
}

pub type StaticCharVec = &'static [char];
pub type StaticStringVec = &'static [&'static str];

/// Whitespace as defined by HTML5 § 2.4.1.
// TODO(SimonSapin) Maybe a custom Pattern can be more efficient?
const WHITESPACE: &'static [char] = &[' ', '\t', '\x0a', '\x0c', '\x0d'];

pub fn is_whitespace(s: &str) -> bool {
    s.chars().all(char_is_whitespace)
}

#[inline]
pub fn char_is_whitespace(c: char) -> bool {
    WHITESPACE.contains(&c)
}

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

pub fn split_html_space_chars<'a>(s: &'a str) ->
                                  Filter<Split<'a, StaticCharVec>, fn(&&str) -> bool> {
    fn not_empty(&split: &&str) -> bool { !split.is_empty() }
    s.split(HTML_SPACE_CHARACTERS).filter(not_empty as fn(&&str) -> bool)
}


fn is_ascii_digit(c: &char) -> bool {
    match *c {
        '0'...'9' => true,
        _ => false,
    }
}


fn read_numbers<I: Iterator<Item=char>>(mut iter: Peekable<I>) -> Option<i64> {
    match iter.peek() {
        Some(c) if is_ascii_digit(c) => (),
        _ => return None,
    }

    iter.take_while(is_ascii_digit).map(|d| {
        d as i64 - '0' as i64
    }).fold(Some(0i64), |accumulator, d| {
        accumulator.and_then(|accumulator| {
            accumulator.checked_mul(10)
        }).and_then(|accumulator| {
            accumulator.checked_add(d)
        })
    })
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LengthOrPercentageOrAuto {
    Auto,
    Percentage(f32),
    Length(Au),
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

/// https://html.spec.whatwg.org/multipage/#rules-for-parsing-a-legacy-font-size
pub fn parse_legacy_font_size(mut input: &str) -> Option<&'static str> {
    // Steps 1 & 2 are not relevant

    // Step 3
    input = input.trim_matches(WHITESPACE);

    enum ParseMode {
        RelativePlus,
        RelativeMinus,
        Absolute,
    }
    let mut input_chars = input.chars().peekable();
    let parse_mode = match input_chars.peek() {
        // Step 4
        None => return None,

        // Step 5
        Some(&'+') => {
            let _ = input_chars.next();  // consume the '+'
            ParseMode::RelativePlus
        }
        Some(&'-') => {
            let _ = input_chars.next();  // consume the '-'
            ParseMode::RelativeMinus
        }
        Some(_) => ParseMode::Absolute,
    };

    // Steps 6, 7, 8
    let mut value = match read_numbers(input_chars) {
        Some(v) => v,
        None => return None,
    };

    // Step 9
    match parse_mode {
        ParseMode::RelativePlus => value = 3 + value,
        ParseMode::RelativeMinus => value = 3 - value,
        ParseMode::Absolute => (),
    }

    // Steps 10, 11, 12
    Some(match value {
        n if n >= 7 => "xxx-large",
        6 => "xx-large",
        5 => "x-large",
        4 => "large",
        3 => "medium",
        2 => "small",
        n if n <= 1 => "x-small",
        _ => unreachable!(),
    })
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
        match (input.as_bytes()[0],
               hex(input.as_bytes()[1] as char),
               hex(input.as_bytes()[2] as char),
               hex(input.as_bytes()[3] as char)) {
            (b'#', Ok(r), Ok(g), Ok(b)) => {
                return Ok(RGBA {
                    red: (r as f32) * 17.0 / 255.0,
                    green: (g as f32) * 17.0 / 255.0,
                    blue: (b as f32) * 17.0 / 255.0,
                    alpha: 1.0,
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


#[derive(Clone, Eq, PartialEq, Hash, Debug, Deserialize, Serialize)]
pub struct LowercaseString {
    inner: String,
}

impl LowercaseString {
    pub fn new(s: &str) -> LowercaseString {
        LowercaseString {
            inner: s.to_lowercase(),
        }
    }
}

impl Deref for LowercaseString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &*self.inner
    }
}

/// Creates a String from the given null-terminated buffer.
/// Panics if the buffer does not contain UTF-8.
pub unsafe fn c_str_to_string(s: *const c_char) -> String {
    from_utf8(CStr::from_ptr(s).to_bytes()).unwrap().to_owned()
}

pub fn str_join<I, T>(strs: I, join: &str) -> String
    where I: IntoIterator<Item=T>, T: AsRef<str>,
{
    strs.into_iter().enumerate().fold(String::new(), |mut acc, (i, s)| {
        if i > 0 { acc.push_str(join); }
        acc.push_str(s.as_ref());
        acc
    })
}

// Lifted from Rust's StrExt implementation, which is being removed.
pub fn slice_chars(s: &str, begin: usize, end: usize) -> &str {
    assert!(begin <= end);
    let mut count = 0;
    let mut begin_byte = None;
    let mut end_byte = None;

    // This could be even more efficient by not decoding,
    // only finding the char boundaries
    for (idx, _) in s.char_indices() {
        if count == begin { begin_byte = Some(idx); }
        if count == end { end_byte = Some(idx); break; }
        count += 1;
    }
    if begin_byte.is_none() && count == begin { begin_byte = Some(s.len()) }
    if end_byte.is_none() && count == end { end_byte = Some(s.len()) }

    match (begin_byte, end_byte) {
        (None, _) => panic!("slice_chars: `begin` is beyond end of string"),
        (_, None) => panic!("slice_chars: `end` is beyond end of string"),
        (Some(a), Some(b)) => unsafe { s.slice_unchecked(a, b) }
    }
}

// searches a character index in CharIndices
// returns indices.count if not found
pub fn search_index(index: usize, indices: CharIndices) -> isize {
    let mut character_count = 0;
    for (character_index, _) in indices {
        if character_index == index {
            return character_count;
        }
        character_count += 1
    }
    character_count
}
