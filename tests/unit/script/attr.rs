/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::dom::attr::AttrValue;
use script::dom::bindings::error::Error;

#[test]
fn test_from_limited_i32_should_be_an_index_error_when_value_is_less_than_0() {
    match AttrValue::from_limited_i32("-1".to_owned(), 0) {
        Err(Error::IndexSize) => (),
        _ => panic!("expected an IndexSize error")
    }
}

#[test]
fn test_from_limited_i32_should_parse_a_uint_when_value_is_0_or_greater() {
    match AttrValue::from_limited_i32("1".to_owned(), 0) {
        Ok(AttrValue::Int(_, result)) => assert_eq!(result, 1),
        Ok(_) => panic!("expected a Int result"),
        _ => panic!("expected a successful parse")
    }
}
