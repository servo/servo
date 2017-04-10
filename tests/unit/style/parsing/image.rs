/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use euclid::size::TypedSize2D;
use media_queries::CSSErrorReporterTest;
use std::f32::consts::PI;
use style::font_metrics::ServoMetricsProvider;
use style::media_queries::{Device, MediaType};
use style::parser::ParserContext;
use style::properties::ComputedValues;
use style::stylesheets::{CssRuleType, Origin};
use style::values::computed;
use style::values::computed::{Angle, Context, ToComputedValue};
use style::values::specified;
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

    // AngleOrCorner::None should become AngleOrCorner::Angle(Angle(PI)) when parsed
    // Note that Angle(PI) is correct for top-to-bottom rendering, whereas Angle(0) would render bottom-to-top.
    // ref: https://developer.mozilla.org/en-US/docs/Web/CSS/angle
    let viewport_size = TypedSize2D::new(0., 0.);
    let initial_style = ComputedValues::initial_values();
    let device = Device::new(MediaType::Screen, viewport_size);
    let specified_context = Context {
        is_root_element: true,
        device: &device,
        inherited_style: initial_style,
        layout_parent_style: initial_style,
        style: initial_style.clone(),
        font_metrics_provider: &ServoMetricsProvider,
        in_media_query: false,
    };
    assert_eq!(specified::AngleOrCorner::None.to_computed_value(&specified_context),
               computed::AngleOrCorner::Angle(Angle::from_radians(PI)));
}

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
