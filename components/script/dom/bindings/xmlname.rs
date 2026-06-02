/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Functions for validating and extracting qualified XML names.

/// Check if an element name is valid. See <http://www.w3.org/TR/xml/#NT-Name>
/// for details.
pub(crate) fn is_valid_start(c: char) -> bool {
    matches!(c, ':' |
        'A'..='Z' |
        '_' |
        'a'..='z' |
        '\u{C0}'..='\u{D6}' |
        '\u{D8}'..='\u{F6}' |
        '\u{F8}'..='\u{2FF}' |
        '\u{370}'..='\u{37D}' |
        '\u{37F}'..='\u{1FFF}' |
        '\u{200C}'..='\u{200D}' |
        '\u{2070}'..='\u{218F}' |
        '\u{2C00}'..='\u{2FEF}' |
        '\u{3001}'..='\u{D7FF}' |
        '\u{F900}'..='\u{FDCF}' |
        '\u{FDF0}'..='\u{FFFD}' |
        '\u{10000}'..='\u{EFFFF}')
}

pub(crate) fn is_valid_continuation(c: char) -> bool {
    is_valid_start(c) ||
        matches!(c,
            '-' |
            '.' |
            '0'..='9' |
            '\u{B7}' |
            '\u{300}'..='\u{36F}' |
            '\u{203F}'..='\u{2040}')
}

pub(crate) fn matches_name_production(name: &str) -> bool {
    let mut iter = name.chars();

    if iter.next().is_none_or(|c| !is_valid_start(c)) {
        return false;
    }
    iter.all(is_valid_continuation)
}
