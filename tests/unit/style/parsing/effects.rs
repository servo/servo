/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use parsing::parse;
use servo_url::ServoUrl;
use style::parser::ParserContext;
use style::properties::longhands::{self, perspective_origin, transform_origin};
use style::stylesheets::{CssRuleType, Origin};
use style_traits::ToCss;

#[test]
fn test_clip() {
    use style::properties::longhands::clip;

    assert_roundtrip_with_context!(clip::parse, "auto");
    assert_roundtrip_with_context!(clip::parse, "rect(1px, 2px, 3px, 4px)");
    assert_roundtrip_with_context!(clip::parse, "rect(1px, auto, auto, 4px)");
    assert_roundtrip_with_context!(clip::parse, "rect(auto, auto, auto, auto)");

    // Non-standard syntax
    assert_roundtrip_with_context!(clip::parse,
                                   "rect(1px 2px 3px 4px)",
                                   "rect(1px, 2px, 3px, 4px)");
    assert_roundtrip_with_context!(clip::parse,
                                   "rect(auto 2px 3px auto)",
                                   "rect(auto, 2px, 3px, auto)");
    assert_roundtrip_with_context!(clip::parse,
                                   "rect(1px auto auto 4px)",
                                   "rect(1px, auto, auto, 4px)");
    assert_roundtrip_with_context!(clip::parse,
                                   "rect(auto auto auto auto)",
                                   "rect(auto, auto, auto, auto)");
}

#[test]
fn test_longhands_parse_origin() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter, Some(CssRuleType::Style));

    let mut parser = Parser::new("1px some-rubbish");
    let parsed = longhands::parse_origin(&context, &mut parser);
    assert!(parsed.is_ok());
    assert_eq!(parser.is_exhausted(), false);

    let mut parser = Parser::new("1px 2px");
    let parsed = longhands::parse_origin(&context, &mut parser);
    assert!(parsed.is_ok());
    assert_eq!(parser.is_exhausted(), true);

    let mut parser = Parser::new("1px");
    let parsed = longhands::parse_origin(&context, &mut parser);
    assert!(parsed.is_ok());
    assert_eq!(parser.is_exhausted(), true);
}

#[test]
fn test_effects_parser_exhaustion() {
    assert_parser_exhausted!(perspective_origin, "1px 1px", true);
    assert_parser_exhausted!(transform_origin, "1px 1px", true);

    assert_parser_exhausted!(perspective_origin, "1px some-rubbish", false);
    assert_parser_exhausted!(transform_origin, "1px some-rubbish", false);
}

#[test]
fn test_parse_factor() {
    use parsing::parse;
    use style::properties::longhands::filter;

    assert!(parse(filter::parse, "brightness(0)").is_ok());
    assert!(parse(filter::parse, "brightness(55)").is_ok());
    assert!(parse(filter::parse, "brightness(100)").is_ok());

    assert!(parse(filter::parse, "contrast(0)").is_ok());
    assert!(parse(filter::parse, "contrast(55)").is_ok());
    assert!(parse(filter::parse, "contrast(100)").is_ok());

    assert!(parse(filter::parse, "grayscale(0)").is_ok());
    assert!(parse(filter::parse, "grayscale(55)").is_ok());
    assert!(parse(filter::parse, "grayscale(100)").is_ok());

    assert!(parse(filter::parse, "invert(0)").is_ok());
    assert!(parse(filter::parse, "invert(55)").is_ok());
    assert!(parse(filter::parse, "invert(100)").is_ok());

    assert!(parse(filter::parse, "opacity(0)").is_ok());
    assert!(parse(filter::parse, "opacity(55)").is_ok());
    assert!(parse(filter::parse, "opacity(100)").is_ok());

    assert!(parse(filter::parse, "sepia(0)").is_ok());
    assert!(parse(filter::parse, "sepia(55)").is_ok());
    assert!(parse(filter::parse, "sepia(100)").is_ok());

    assert!(parse(filter::parse, "saturate(0)").is_ok());
    assert!(parse(filter::parse, "saturate(55)").is_ok());
    assert!(parse(filter::parse, "saturate(100)").is_ok());

    // Negative numbers are invalid for certain filters
    assert!(parse(filter::parse, "brightness(-1)").is_err());
    assert!(parse(filter::parse, "contrast(-1)").is_err());
    assert!(parse(filter::parse, "grayscale(-1)").is_err());
    assert!(parse(filter::parse, "invert(-1)").is_err());
    assert!(parse(filter::parse, "opacity(-1)").is_err());
    assert!(parse(filter::parse, "sepia(-1)").is_err());
    assert!(parse(filter::parse, "saturate(-1)").is_err());
}

#[test]
fn blur_radius_should_not_accept_negavite_values() {
    use style::properties::longhands::box_shadow;
    assert!(parse(box_shadow::parse, "1px 1px -1px").is_err());// for -ve values
    assert!(parse(box_shadow::parse, "1px 1px 0").is_ok());// for zero
    assert!(parse(box_shadow::parse, "1px 1px 1px").is_ok());// for +ve value
}
