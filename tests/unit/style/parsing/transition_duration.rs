/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::properties::longhands::transition_duration;

use crate::parsing::parse;

#[test]
fn test_positive_transition_duration() {
    assert!(parse(transition_duration::parse, "5s").is_ok());
    assert!(parse(transition_duration::parse, "0s").is_ok());
}

#[test]
fn test_negative_transition_duration() {
    assert!(parse(transition_duration::parse, "-5s").is_err());
}
