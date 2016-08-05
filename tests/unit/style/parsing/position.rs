/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use style::values::specified::position::*;

#[test]
fn test_position() {
    // Serialization is not actually specced
    // though these are the values expected by basic-shape
    // https://github.com/w3c/csswg-drafts/issues/368
    assert_roundtrip!(Position::parse, "center", "50% 50%");
    assert_roundtrip!(Position::parse, "top left", "0% 0%");
    assert_roundtrip!(Position::parse, "left top", "0% 0%");
    assert_roundtrip!(Position::parse, "top right", "100% 0%");
    assert_roundtrip!(Position::parse, "right top", "100% 0%");
    assert_roundtrip!(Position::parse, "bottom left", "0% 100%");
    assert_roundtrip!(Position::parse, "left bottom", "0% 100%");
    assert_roundtrip!(Position::parse, "left center", "0% 50%");
    assert_roundtrip!(Position::parse, "right center", "100% 50%");
    assert_roundtrip!(Position::parse, "center top", "50% 0%");
    assert_roundtrip!(Position::parse, "center bottom", "50% 100%");
    assert_roundtrip!(Position::parse, "center 10px", "50% 10px");
    assert_roundtrip!(Position::parse, "center 10%", "50% 10%");
    assert_roundtrip!(Position::parse, "right 10%", "100% 10%");

    // Only keywords can be reordered
    assert!(parse(Position::parse, "top 40%").is_err());
    assert!(parse(Position::parse, "40% left").is_err());

    // we don't yet handle 4-valued positions
    // https://github.com/servo/servo/issues/12690
}
