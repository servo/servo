/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use util::str::LengthOrPercentageOrAuto;
use util::str::{parse_length, search_index, split_html_space_chars, str_join};


#[test]
pub fn test_parse_length() {
    fn check(input: &str, expected: LengthOrPercentageOrAuto) {
        let parsed = parse_length(input);
        assert_eq!(parsed, expected);
    }

    check("0", LengthOrPercentageOrAuto::Length(Au::from_px(0)));
    check("0.000%", LengthOrPercentageOrAuto::Percentage(0.0));
    check("+5.82%", LengthOrPercentageOrAuto::Percentage(0.0582));
    check("5.82", LengthOrPercentageOrAuto::Length(Au::from_f64_px(5.82)));
    check("invalid", LengthOrPercentageOrAuto::Auto);
    check("12 followed by invalid", LengthOrPercentageOrAuto::Length(Au::from_px(12)));
}

#[test]
pub fn split_html_space_chars_whitespace() {
    assert!(split_html_space_chars("").collect::<Vec<_>>().is_empty());
    assert!(split_html_space_chars("\u{0020}\u{0009}\u{000a}\u{000c}\u{000d}").collect::<Vec<_>>().is_empty());
}

#[test]
pub fn test_str_join_empty() {
    let slice = [] as [&str; 0];
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

#[test]
pub fn test_search_index() {
    let tuples = [("", 1, 0),
                  ("foo", 8, 3),
                  ("føo", 8, 3),
                  ("foo", 2, 2),
                  ("føo", 2, 3)];
    for t in tuples.iter() {
        assert_eq!(search_index(t.1, t.0.char_indices()), t.2);
    };
}
