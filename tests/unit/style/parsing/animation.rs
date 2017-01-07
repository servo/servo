/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use parsing::parse;
use style::parser::{Parse, ParserContext};
use style::properties::longhands::animation_iteration_count::single_value::computed_value::T as AnimationIterationCount;
use style::stylesheets::Origin;
use style_traits::ToCss;

#[test]
fn test_animation_iteration() {
    assert_roundtrip_with_context!(AnimationIterationCount::parse, "0", "0");
    assert_roundtrip_with_context!(AnimationIterationCount::parse, "0.1", "0.1");
    assert_roundtrip_with_context!(AnimationIterationCount::parse, "infinite", "infinite");

    // Negative numbers are invalid
    assert!(parse(AnimationIterationCount::parse, "-1").is_err());
}
