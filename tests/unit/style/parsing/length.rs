/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use style::parser::Parse;
use style::values::specified::length::Length;

#[test]
fn test_calc() {
    assert!(parse(Length::parse, "calc(1px+ 2px)").is_err());
    assert!(parse(Length::parse, "calc( 1px + 2px )").is_ok());
    assert!(parse(Length::parse, "calc(1px + 2px )").is_ok());
    assert!(parse(Length::parse, "calc( 1px + 2px)").is_ok());
}
