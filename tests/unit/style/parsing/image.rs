/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::parser::ParserContext;
use style::stylesheets::Origin;
use style::values::specified::image::*;
use url::Url;

#[test]
fn test_radial_gradient() {
    // Parsing with all values
    assert_roundtrip_with_context!(Image::parse, "radial-gradient(circle closest-side at 20px 30px, red, green)");
    assert_roundtrip_with_context!(Image::parse, "radial-gradient(ellipse closest-side at 20px 30px, red, green)");
    assert_roundtrip_with_context!(Image::parse, "radial-gradient(closest-side circle at 20px 30px, red, green)",
                                                 "radial-gradient(circle closest-side at 20px 30px, red, green)");
    assert_roundtrip_with_context!(Image::parse, "radial-gradient(closest-side ellipse at 20px 30px, red, green)",
                                                 "radial-gradient(ellipse closest-side at 20px 30px, red, green)");

    // Parsing with <shape-keyword> and <size> reversed
    assert_roundtrip_with_context!(Image::parse, "radial-gradient(closest-side circle at 20px 30px, red, green)",
                                                 "radial-gradient(circle closest-side at 20px 30px, red, green)");
    assert_roundtrip_with_context!(Image::parse, "radial-gradient(closest-corner ellipse at 20px 30px, red, green)",
                                                 "radial-gradient(ellipse closest-corner at 20px 30px, red, green)");
    assert_roundtrip_with_context!(Image::parse, "radial-gradient(30px circle, red, green)",
                                                 "radial-gradient(circle 30px at center center, red, green)");
    assert_roundtrip_with_context!(Image::parse, "radial-gradient(30px 40px ellipse, red, green)",
                                                 "radial-gradient(ellipse 30px 40px at center center, red, green)");

    // Parsing without <size>
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(circle, red, green)",
                                   "radial-gradient(circle farthest-corner at center center, red, green)");
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(ellipse, red, green)",
                                   "radial-gradient(ellipse farthest-corner at center center, red, green)");
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(circle at 20px 30px, red, green)",
                                   "radial-gradient(circle farthest-corner at 20px 30px, red, green)");
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(ellipse at 20px 30px, red, green)",
                                   "radial-gradient(ellipse farthest-corner at 20px 30px, red, green)");


    // Parsing without <shape-keyword>
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(20px at 20px 30px, red, green)",
                                   "radial-gradient(circle 20px at 20px 30px, red, green)");
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(20px 30px at left center, red, green)",
                                   "radial-gradient(ellipse 20px 30px at left center, red, green)");
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(closest-side at center, red, green)",
                                   "radial-gradient(ellipse closest-side at center center, red, green)");
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(20px, red, green)",
                                   "radial-gradient(circle 20px at center center, red, green)");
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(20px 30px, red, green)",
                                   "radial-gradient(ellipse 20px 30px at center center, red, green)");
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(closest-side, red, green)",
                                   "radial-gradient(ellipse closest-side at center center, red, green)");

    // Parsing without <shape-keyword> and <size>
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(at center, red, green)",
                                   "radial-gradient(ellipse farthest-corner at center center, red, green)");
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(at center bottom, red, green)",
                                   "radial-gradient(ellipse farthest-corner at center bottom, red, green)");
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(at 40px 50px, red, green)",
                                   "radial-gradient(ellipse farthest-corner at 40px 50px, red, green)");

    // Parsing with just color stops
    assert_roundtrip_with_context!(Image::parse,
                                   "radial-gradient(red, green)",
                                   "radial-gradient(ellipse farthest-corner at center center, red, green)");

    // Parsing repeating radial gradient
    assert_roundtrip_with_context!(Image::parse,
                                   "repeating-radial-gradient(red, green)",
                                   "repeating-radial-gradient(ellipse farthest-corner at center center, red, green)");
}
