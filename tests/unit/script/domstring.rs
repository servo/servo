// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use script::test::DOMString;

#[test]
fn test_domstring_is_valid_floating_point_number_string_leading_whitespace() {
    assert!(!DOMString::from("\t1").is_valid_floating_point_number_string());
    assert!(!DOMString::from("\n1").is_valid_floating_point_number_string());
    // \x0C - form feed
    assert!(!DOMString::from("\x0C1").is_valid_floating_point_number_string());
    assert!(!DOMString::from("\r1").is_valid_floating_point_number_string());
    assert!(!DOMString::from(" 1").is_valid_floating_point_number_string());
}
