/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Tests for :dir() pseudo-class representation

use style::gecko::selector_parser::Direction;

#[test]
fn creation() {
    // basic variant creation
    assert_eq!(Direction::from("ltr"), Direction::Ltr);
    assert_eq!(Direction::from("rtl"), Direction::Rtl);
    assert_eq!(Direction::from("nonstandard"), Direction::Other("nonstandard".to_owned()));

    // case-insensitive variant creation
    assert_eq!(Direction::from("LTR"), Direction::Ltr);
    assert_eq!(Direction::from("RTL"), Direction::Rtl);
    assert_eq!(Direction::from("NONSTANDARD"), Direction::Other("nonstandard".to_owned()));
}

#[test]
fn stringification() {
    let ltr = Direction::from("ltr");
    let rtl = Direction::from("Rtl");
    let other = Direction::from("nonSTANDARD");

    assert_eq!(String::from(&ltr), "ltr".to_owned());
    assert_eq!(String::from(&rtl), "rtl".to_owned());
    assert_eq!(String::from(&other), "nonstandard".to_owned());
}
