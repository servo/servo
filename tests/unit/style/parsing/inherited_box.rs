/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::parser::ParserContext;
use style::stylesheets::{CssRuleType, Origin};

#[test]
fn image_orientation_longhand_should_parse_properly() {
    use style::properties::longhands::image_orientation;
    use style::properties::longhands::image_orientation::SpecifiedValue;
    use style::values::specified::Angle;

    let from_image = parse_longhand!(image_orientation, "from-image");
    assert_eq!(from_image, SpecifiedValue { angle: None, flipped: false });

    let flip = parse_longhand!(image_orientation, "flip");
    assert_eq!(flip, SpecifiedValue { angle: Some(Angle::from_degrees(0.0)), flipped: true });

    let zero = parse_longhand!(image_orientation, "0deg");
    assert_eq!(zero, SpecifiedValue { angle: Some(Angle::from_degrees(0.0)), flipped: false });

    let negative_rad = parse_longhand!(image_orientation, "-1rad");
    assert_eq!(negative_rad, SpecifiedValue { angle: Some(Angle::from_radians(-1.0)), flipped: false });

    let flip_with_180 = parse_longhand!(image_orientation, "180deg flip");
    assert_eq!(flip_with_180, SpecifiedValue { angle: Some(Angle::from_degrees(180.0)), flipped: true });
}
