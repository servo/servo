/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::iter::Filter;
use std::str::CharSplits;

pub type DOMString = ~str;
pub type StaticCharVec = &'static [char];
pub type StaticStringVec = &'static [&'static str];

pub fn null_str_as_empty(s: &Option<DOMString>) -> DOMString {
    // We don't use map_default because it would allocate "".to_owned() even for Some.
    match *s {
        Some(ref s) => s.clone(),
        None => "".to_owned()
    }
}

pub fn null_str_as_empty_ref<'a>(s: &'a Option<DOMString>) -> &'a str {
    match *s {
        Some(ref s) => s.as_slice(),
        None => &'a ""
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
