/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use style::parser::Parse;
use style::values::specified::basic_shape::*;
use style_traits::ToCss;

// Ensure that basic-shape sub-functions parse as both basic shapes
// and their individual components
macro_rules! assert_roundtrip_basicshape {
    ($fun:expr, $input:expr, $output:expr) => {
        assert_roundtrip_with_context!($fun, $input, $output);
        assert_roundtrip_with_context!(BasicShape::parse, $input, $output);
    };
    ($fun:expr, $input:expr) => {
        assert_roundtrip_basicshape!($fun, $input, $input);
    };
}

#[test]
fn test_inset() {
    assert_roundtrip_basicshape!(InsetRect::parse, "inset(10px)");
    assert_roundtrip_basicshape!(InsetRect::parse, "inset(10px 20%)");

    assert_roundtrip_basicshape!(InsetRect::parse, "inset(10px round 10px)");
    assert_roundtrip_basicshape!(InsetRect::parse, "inset(10px round 10px 20px 30px 40px)");
    assert_roundtrip_basicshape!(InsetRect::parse, "inset(10px round 10px 20px 30px 40px / 1px 2px 3px 4px)");
}

#[test]
fn test_circle() {
    assert_roundtrip_basicshape!(Circle::parse, "circle(at center)", "circle(at 50% 50%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle()", "circle(at 50% 50%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(at left bottom)", "circle(at 0% 100%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(at bottom left)", "circle(at 0% 100%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(at top left)", "circle(at 0% 0%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(at center left)", "circle(at 0% 50%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(at left center)", "circle(at 0% 50%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(at top center)", "circle(at 50% 0%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(at center top)", "circle(at 50% 0%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(at 40% top)", "circle(at 40% 0%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(at 10px 100px)", "circle(at 10px 100px)");
    // closest-side is omitted, because it is the default
    assert_roundtrip_basicshape!(Circle::parse, "circle(closest-side at center)", "circle(at 50% 50%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(farthest-side at center)",
                                                "circle(farthest-side at 50% 50%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(10px)",
                                                "circle(10px at 50% 50%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(20px at center)", "circle(20px at 50% 50%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(calc(1px + 50%) at center)",
                                                "circle(calc(1px + 50%) at 50% 50%)");

    assert_roundtrip_basicshape!(Circle::parse, "circle(at right 5px bottom 10px)",
                                                "circle(at right 5px bottom 10px)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(at bottom 5px right 10px)",
                                                "circle(at right 10px bottom 5px)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(at right 5% top 0px)",
                                                "circle(at 95% 0%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(at right 5% bottom 0px)",
                                                "circle(at 95% 100%)");
    assert_roundtrip_basicshape!(Circle::parse, "circle(at right 5% bottom 1px)",
                                                "circle(at left 95% bottom 1px)");

    assert!(parse(Circle::parse, "circle(at 5% bottom 1px)").is_err());
    assert!(parse(Circle::parse, "circle(at top 40%)").is_err());
    assert!(parse(Circle::parse, "circle(-10px)").is_err());
}

#[test]
fn test_ellipse() {
    assert_roundtrip_basicshape!(Ellipse::parse, "ellipse(at center)", "ellipse(at 50% 50%)");
    assert_roundtrip_basicshape!(Ellipse::parse, "ellipse()", "ellipse(at 50% 50%)");
    assert_roundtrip_basicshape!(Ellipse::parse, "ellipse(at left bottom)", "ellipse(at 0% 100%)");
    assert_roundtrip_basicshape!(Ellipse::parse, "ellipse(at bottom left)", "ellipse(at 0% 100%)");
    assert_roundtrip_basicshape!(Ellipse::parse, "ellipse(at 10px 100px)", "ellipse(at 10px 100px)");
    // closest-side is omitted, because it is the default
    assert_roundtrip_basicshape!(Ellipse::parse, "ellipse(closest-side closest-side at center)",
                                                 "ellipse(at 50% 50%)");
    assert_roundtrip_basicshape!(Ellipse::parse, "ellipse(farthest-side closest-side at center)",
                                                 "ellipse(farthest-side closest-side at 50% 50%)");
    assert_roundtrip_basicshape!(Ellipse::parse, "ellipse(20px 10% at center)", "ellipse(20px 10% at 50% 50%)");
    assert_roundtrip_basicshape!(Ellipse::parse, "ellipse(calc(1px + 50%) 10px at center)",
                                                 "ellipse(calc(1px + 50%) 10px at 50% 50%)");
}

#[test]
fn test_polygon() {
    // surprisingly, polygons are only required to have at least one vertex,
    // not at least 3
    assert_roundtrip_basicshape!(Polygon::parse, "polygon(10px 10px)", "polygon(10px 10px)");
    assert_roundtrip_basicshape!(Polygon::parse, "polygon(10px 10px, 10px 10px)", "polygon(10px 10px, 10px 10px)");
    assert_roundtrip_basicshape!(Polygon::parse, "polygon(nonzero, 10px 10px, 10px 10px)",
                                                 "polygon(10px 10px, 10px 10px)");
    assert_roundtrip_basicshape!(Polygon::parse, "polygon(evenodd, 10px 10px, 10px 10px)",
                                                 "polygon(evenodd, 10px 10px, 10px 10px)");
    assert_roundtrip_basicshape!(Polygon::parse, "polygon(evenodd, 10px 10px, 10px calc(10px + 50%))",
                                                 "polygon(evenodd, 10px 10px, 10px calc(10px + 50%))");
    assert_roundtrip_basicshape!(Polygon::parse, "polygon(evenodd, 10px 10px, 10px 10px, 10px 10px, 10px 10px, 10px \
                                                  10px, 10px 10px, 10px 10px, 10px 10px, 10px 10px, 10px 10px, \
                                                  10px 10px, 10px 10px, 10px 10px)",
                                                 "polygon(evenodd, 10px 10px, 10px 10px, 10px 10px, 10px 10px, 10px \
                                                  10px, 10px 10px, 10px 10px, 10px 10px, 10px 10px, 10px 10px, \
                                                  10px 10px, 10px 10px, 10px 10px)");

    assert!(parse(Polygon::parse, "polygon()").is_err());
}
