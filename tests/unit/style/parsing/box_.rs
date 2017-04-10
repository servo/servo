/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use parsing::parse;
use style::parser::ParserContext;
use style::stylesheets::{CssRuleType, Origin};
use style_traits::ToCss;

#[test]
fn test_will_change() {
    use style::properties::longhands::will_change;

    assert_roundtrip_with_context!(will_change::parse, "auto");
    assert_roundtrip_with_context!(will_change::parse, "scroll-position");
    assert_roundtrip_with_context!(will_change::parse, "contents");
    assert_roundtrip_with_context!(will_change::parse, "transition");
    assert_roundtrip_with_context!(will_change::parse, "opacity, transform");

    assert!(parse(will_change::parse, "will-change").is_err());
    assert!(parse(will_change::parse, "all").is_err());
    assert!(parse(will_change::parse, "none").is_err());
    assert!(parse(will_change::parse, "contents, auto").is_err());
    assert!(parse(will_change::parse, "contents, inherit, initial").is_err());
    assert!(parse(will_change::parse, "transform scroll-position").is_err());
}

#[test]
fn test_transform_translate() {
    use style::properties::longhands::transform;
    assert_roundtrip_with_context!(transform::parse, "translate(2px)");
    assert_roundtrip_with_context!(transform::parse, "translate(2px, 5px)");
    assert!(parse(transform::parse, "translate(2px foo)").is_err());
    assert!(parse(transform::parse, "perspective(-10px)").is_err());
}
