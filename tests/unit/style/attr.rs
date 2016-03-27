/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use style::attr::{AttrValue, parse_length};
use util::str::{DOMString, LengthOrPercentageOrAuto};

#[test]
fn test_from_limited_i32_should_be_default_when_less_than_0() {
    let value = DOMString::from("-1");
    match AttrValue::from_limited_i32(value, 0) {
        AttrValue::Int(_, 0) => (),
        _ => panic!("expected an IndexSize error")
    }
}

#[test]
fn test_from_limited_i32_should_parse_a_uint_when_value_is_0_or_greater() {
    match AttrValue::from_limited_i32(DOMString::from("1"), 0) {
        AttrValue::Int(_, 1) => (),
        _ => panic!("expected an successful parsing")
    }
}

#[test]
fn test_from_limited_i32_should_keep_parsed_value_when_not_an_int() {
    match AttrValue::from_limited_i32(DOMString::from("parsed-value"), 0) {
        AttrValue::Int(p, 0) => {
            assert_eq!(p, DOMString::from("parsed-value"))
        },
        _ => panic!("expected an successful parsing")
    }
}

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
