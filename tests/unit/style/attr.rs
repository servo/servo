/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style::attr::AttrValue;
use util::str::DOMString;

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
