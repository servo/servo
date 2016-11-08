/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use style::values::specified::position::*;
use style_traits::ToCss;

#[test]
fn test_position() {
    // Serialization is not actually specced
    // though these are the values expected by basic-shape
    // https://github.com/w3c/csswg-drafts/issues/368
    assert_roundtrip!(Position::parse, "center", "center center");
    assert_roundtrip!(Position::parse, "top left", "left top");
    assert_roundtrip!(Position::parse, "left top", "left top");
    assert_roundtrip!(Position::parse, "top right", "right top");
    assert_roundtrip!(Position::parse, "right top", "right top");
    assert_roundtrip!(Position::parse, "bottom left", "left bottom");
    assert_roundtrip!(Position::parse, "left bottom", "left bottom");
    assert_roundtrip!(Position::parse, "left center", "left center");
    assert_roundtrip!(Position::parse, "right center", "right center");
    assert_roundtrip!(Position::parse, "center top", "center top");
    assert_roundtrip!(Position::parse, "center bottom", "center bottom");
    assert_roundtrip!(Position::parse, "center 10px", "center 10px");
    assert_roundtrip!(Position::parse, "center 10%", "center 10%");
    assert_roundtrip!(Position::parse, "right 10%", "right 10%");

    // Only keywords can be reordered
    assert!(parse(Position::parse, "top 40%").is_err());
    assert!(parse(Position::parse, "40% left").is_err());

    // 3 and 4 value serialization
    assert_roundtrip!(Position::parse, "left 10px top 15px", "left 10px top 15px");
    assert_roundtrip!(Position::parse, "top 15px left 10px", "left 10px top 15px");
    assert_roundtrip!(Position::parse, "left 10% top 15px", "left 10% top 15px");
    assert_roundtrip!(Position::parse, "top 15px left 10%", "left 10% top 15px");
    assert_roundtrip!(Position::parse, "left top 15px", "left top 15px");
    assert_roundtrip!(Position::parse, "top 15px left", "left top 15px");
    assert_roundtrip!(Position::parse, "left 10px top", "left 10px top");
    assert_roundtrip!(Position::parse, "top left 10px", "left 10px top");
    assert_roundtrip!(Position::parse, "right 10px bottom", "right 10px bottom");
    assert_roundtrip!(Position::parse, "bottom right 10px", "right 10px bottom");
    assert_roundtrip!(Position::parse, "center right 10px", "right 10px center");
    assert_roundtrip!(Position::parse, "center bottom 10px", "center bottom 10px");

    // Only horizontal and vertical keywords can have positions
    assert!(parse(Position::parse, "center 10px left 15px").is_err());
    assert!(parse(Position::parse, "center 10px 15px").is_err());
    assert!(parse(Position::parse, "center 10px bottom").is_err());

    // "Horizontal Horizontal" or "Vertical Vertical" positions cause error
    assert!(parse(Position::parse, "left right").is_err());
    assert!(parse(Position::parse, "left 10px right").is_err());
    assert!(parse(Position::parse, "left 10px right 15%").is_err());
    assert!(parse(Position::parse, "top bottom").is_err());
    assert!(parse(Position::parse, "top 10px bottom").is_err());
    assert!(parse(Position::parse, "top 10px bottom 15%").is_err());

}
