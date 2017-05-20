/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::context::QuirksMode;
use style::parser::{PARSING_MODE_ALLOW_ALL_NUMERIC_VALUES, ParserContext};
use style::stylesheets::{CssRuleType, Origin};
use style::values::HasViewportPercentage;
use style::values::specified::{AbsoluteLength, NoCalcLength, Number, ViewportPercentageLength};

#[test]
fn length_has_viewport_percentage() {
    let l = NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vw(100.));
    assert!(l.has_viewport_percentage());
    let l = NoCalcLength::Absolute(AbsoluteLength::Px(Au(100).to_f32_px()));
    assert!(!l.has_viewport_percentage());
}

#[test]
fn test_parsing_allo_all_numeric_values() {
    // In SVG length mode, non-zero lengths are assumed to be px.
    let url = ::servo_url::ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter,
                                     Some(CssRuleType::Style), PARSING_MODE_ALLOW_ALL_NUMERIC_VALUES,
                                     QuirksMode::NoQuirks);
    let mut parser = Parser::new("-1");
    let result = Number::parse_non_negative(&context, &mut parser);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Number::new(-1.));
}

