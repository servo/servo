/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style::str::{split_html_space_chars, str_join};

#[test]
pub fn split_html_space_chars_whitespace() {
    assert!(split_html_space_chars("").collect::<Vec<_>>().is_empty());
    assert!(split_html_space_chars("\u{0020}\u{0009}\u{000a}\u{000c}\u{000d}").collect::<Vec<_>>().is_empty());
}

#[test]
pub fn test_str_join_empty() {
    let slice: [&str; 0] = [];
    let actual = str_join(&slice, "-");
    let expected = "";
    assert_eq!(actual, expected);
}

#[test]
pub fn test_str_join_one() {
    let slice = ["alpha"];
    let actual = str_join(&slice, "-");
    let expected = "alpha";
    assert_eq!(actual, expected);
}

#[test]
pub fn test_str_join_many() {
    let slice = ["", "alpha", "", "beta", "gamma", ""];
    let actual = str_join(&slice, "-");
    let expected = "-alpha--beta-gamma-";
    assert_eq!(actual, expected);
}
