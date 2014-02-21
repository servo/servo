/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub type DOMString = ~str;

pub fn null_str_as_empty(s: &Option<DOMString>) -> DOMString {
    // We don't use map_default because it would allocate ~"" even for Some.
    match *s {
        Some(ref s) => s.clone(),
        None => ~""
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
