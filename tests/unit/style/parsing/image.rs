/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::parser::Parse;
use style::values::specified::image::*;
use style_traits::ToCss;

#[test]
fn test_linear_gradient() {
    // Parsing from the right
    assert_roundtrip_with_context!(Image::parse, "linear-gradient(to left, red, green)");

    // Parsing from the left
    assert_roundtrip_with_context!(Image::parse, "linear-gradient(to right, red, green)");

    // Parsing with two values for <side-or-corner>
    assert_roundtrip_with_context!(Image::parse, "linear-gradient(to right top, red, green)");

    // Parsing with <angle>
    assert_roundtrip_with_context!(Image::parse, "linear-gradient(45deg, red, green)");

    // Parsing with more than two entries in <color-stop-list>
    assert_roundtrip_with_context!(Image::parse, "linear-gradient(red, yellow, green)");

    // Parsing with percentage in the <color-stop-list>
    assert_roundtrip_with_context!(Image::parse, "linear-gradient(red, green, yellow 50%)");

    // Parsing without <angle> and <side-or-corner>
    assert_roundtrip_with_context!(Image::parse, "linear-gradient(red, green)");
}
