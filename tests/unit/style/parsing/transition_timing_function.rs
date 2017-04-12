/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use parsing::parse;
use style::parser::ParserContext;
use style::properties::longhands::transition_timing_function;
use style::stylesheets::{CssRuleType, Origin};
use style_traits::ToCss;

#[test]
fn test_cubic_bezier() {
    assert_roundtrip_with_context!(transition_timing_function::parse, "cubic-bezier(0, 0, 0, 0)");
    assert_roundtrip_with_context!(transition_timing_function::parse, "cubic-bezier(0.25, 0, 0.5, 0)");
    assert_roundtrip_with_context!(transition_timing_function::parse, "cubic-bezier(1, 1, 1, 1)");

    // p1x and p2x values must be in range [0, 1]
    assert!(parse(transition_timing_function::parse, "cubic-bezier(-1, 0, 0, 0").is_err());
    assert!(parse(transition_timing_function::parse, "cubic-bezier(0, 0, -1, 0").is_err());
    assert!(parse(transition_timing_function::parse, "cubic-bezier(-1, 0, -1, 0").is_err());

    assert!(parse(transition_timing_function::parse, "cubic-bezier(2, 0, 0, 0").is_err());
    assert!(parse(transition_timing_function::parse, "cubic-bezier(0, 0, 2, 0").is_err());
    assert!(parse(transition_timing_function::parse, "cubic-bezier(2, 0, 2, 0").is_err());
}

#[test]
fn test_steps() {
    assert_roundtrip_with_context!(transition_timing_function::parse, "steps(1)");

    // Step interval value must be an integer greater than 0
    assert!(parse(transition_timing_function::parse, "steps(0)").is_err());
    assert!(parse(transition_timing_function::parse, "steps(0.5)").is_err());
    assert!(parse(transition_timing_function::parse, "steps(-1)").is_err());
}
