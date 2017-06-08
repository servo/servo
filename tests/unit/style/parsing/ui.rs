/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use parsing::parse;
use style::values::{Auto, Either};
use style::values::specified::Color;
use style_traits::ToCss;

#[test]
fn test_moz_user_select() {
    use style::properties::longhands::_moz_user_select;

    assert_roundtrip_with_context!(_moz_user_select::parse, "auto");
    assert_roundtrip_with_context!(_moz_user_select::parse, "text");
    assert_roundtrip_with_context!(_moz_user_select::parse, "none");
    assert_roundtrip_with_context!(_moz_user_select::parse, "element");
    assert_roundtrip_with_context!(_moz_user_select::parse, "elements");
    assert_roundtrip_with_context!(_moz_user_select::parse, "toggle");
    assert_roundtrip_with_context!(_moz_user_select::parse, "tri-state");
    assert_roundtrip_with_context!(_moz_user_select::parse, "-moz-all");
    assert_roundtrip_with_context!(_moz_user_select::parse, "-moz-none");
    assert_roundtrip_with_context!(_moz_user_select::parse, "-moz-text");

    assert!(parse(_moz_user_select::parse, "potato").is_err());
}

#[test]
fn test_caret_color() {
    use style::properties::longhands::caret_color;

    let auto = parse_longhand!(caret_color, "auto");
    assert_eq!(auto, Either::Second(Auto));

    let blue_color = Color::Numeric {
        parsed: RGBA {
            red: 0,
            green: 0,
            blue: 255,
            alpha: 255,
        },
        authored: Some(String::from("blue").into_boxed_str()),
    };

    let color = parse_longhand!(caret_color, "blue");
    assert_eq!(color, Either::First(blue_color));
}
