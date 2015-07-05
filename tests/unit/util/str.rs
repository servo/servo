/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use util::str::split_html_space_chars;


#[test]
pub fn split_html_space_chars_whitespace() {
    assert!(split_html_space_chars("").collect::<Vec<_>>().is_empty());
    assert!(split_html_space_chars("\u{0020}\u{0009}\u{000a}\u{000c}\u{000d}").collect::<Vec<_>>().is_empty());
}
