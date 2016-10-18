/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::parser::ParserContext;
use style::stylesheets::Origin;
use style::values::specified::image::*;
use url::Url;

macro_rules! assert_roundtrip_image {
    ($string:expr) => {
        assert_roundtrip_image!($string, $string);
    };
    ($input:expr, $output:expr) => {
        let url = Url::parse("http://localhost").unwrap();
        let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
        let mut parser = Parser::new($input);
        let parsed = Image::parse(&context, &mut parser)
                     .expect(&format!("Failed to parse {}", $input));
        let serialized = ::cssparser::ToCss::to_css_string(&parsed);
        assert_eq!(serialized, $output);

        let mut parser = Parser::new(&serialized);
        let re_parsed = Image::parse(&context, &mut parser)
                     .expect(&format!("Failed to parse {}", $input));
        let re_serialized = ::cssparser::ToCss::to_css_string(&re_parsed);
        assert_eq!(serialized, re_serialized);
    }
}

#[test]
fn test_radial_gradient() {
    // Parsing with all values
    assert_roundtrip_image!("radial-gradient(circle closest-side at 20px 30px, red, green)");
    assert_roundtrip_image!("radial-gradient(ellipse closest-side at 20px 30px, red, green)");
    assert_roundtrip_image!("radial-gradient(closest-side circle at 20px 30px, red, green)",
                            "radial-gradient(circle closest-side at 20px 30px, red, green)");
    assert_roundtrip_image!("radial-gradient(closest-side ellipse at 20px 30px, red, green)",
                            "radial-gradient(ellipse closest-side at 20px 30px, red, green)");

    // Parsing with <basic-shape> and <size> reversed
    assert_roundtrip_image!("radial-gradient(closest-side circle at 20px 30px, red, green)",
                            "radial-gradient(circle closest-side at 20px 30px, red, green)");
    assert_roundtrip_image!("radial-gradient(closest-corner ellipse at 20px 30px, red, green)",
                            "radial-gradient(ellipse closest-corner at 20px 30px, red, green)");
    assert_roundtrip_image!("radial-gradient(30px circle, red, green)",
                            "radial-gradient(circle 30px at center center, red, green)");
    assert_roundtrip_image!("radial-gradient(30px 40px ellipse, red, green)",
                            "radial-gradient(ellipse 30px 40px at center center, red, green)");

    // Parsing without <size>
    assert_roundtrip_image!("radial-gradient(circle, red, green)",
                            "radial-gradient(circle farthest-corner at center center, red, green)");
    assert_roundtrip_image!("radial-gradient(ellipse, red, green)",
                            "radial-gradient(ellipse farthest-corner at center center, red, green)");
    assert_roundtrip_image!("radial-gradient(circle at 20px 30px, red, green)",
                            "radial-gradient(circle farthest-corner at 20px 30px, red, green)");
    assert_roundtrip_image!("radial-gradient(ellipse at 20px 30px, red, green)",
                            "radial-gradient(ellipse farthest-corner at 20px 30px, red, green)");


    // Parsing without <basic-shape>
    assert_roundtrip_image!("radial-gradient(20px at 20px 30px, red, green)",
                            "radial-gradient(circle 20px at 20px 30px, red, green)");
    assert_roundtrip_image!("radial-gradient(20px 30px at left center, red, green)",
                            "radial-gradient(ellipse 20px 30px at left center, red, green)");
    assert_roundtrip_image!("radial-gradient(closest-side at center, red, green)",
                            "radial-gradient(ellipse closest-side at center center, red, green)");
    assert_roundtrip_image!("radial-gradient(20px, red, green)",
                            "radial-gradient(circle 20px at center center, red, green)");
    assert_roundtrip_image!("radial-gradient(20px 30px, red, green)",
                            "radial-gradient(ellipse 20px 30px at center center, red, green)");
    assert_roundtrip_image!("radial-gradient(closest-side, red, green)",
                            "radial-gradient(ellipse closest-side at center center, red, green)");

    // Parsing without <basic-shape> and <size>
    assert_roundtrip_image!("radial-gradient(at center, red, green)",
                            "radial-gradient(ellipse farthest-corner at center center, red, green)");
    assert_roundtrip_image!("radial-gradient(at center bottom, red, green)",
                            "radial-gradient(ellipse farthest-corner at center bottom, red, green)");
    assert_roundtrip_image!("radial-gradient(at 40px 50px, red, green)",
                            "radial-gradient(ellipse farthest-corner at 40px 50px, red, green)");

    // Parsing with just color stops
    assert_roundtrip_image!("radial-gradient(red, green)",
                            "radial-gradient(ellipse farthest-corner at center center, red, green)");

    // Parsing repeating radial gradient
    assert_roundtrip_image!("repeating-radial-gradient(red, green)",
                            "repeating-radial-gradient(ellipse farthest-corner at center center, red, green)");
}
