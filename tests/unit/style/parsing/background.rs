/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use servo_url::ServoUrl;
use style::parser::ParserContext;
use style::properties::longhands::{background_attachment, background_clip, background_color, background_image};
use style::properties::longhands::{background_origin, background_position_x, background_position_y, background_repeat};
use style::properties::longhands::background_size;
use style::properties::shorthands::background;
use style::stylesheets::{CssRuleType, Origin};

#[test]
fn background_shorthand_should_parse_all_available_properties_when_specified() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter, Some(CssRuleType::Style));
    let mut parser = Parser::new("url(\"http://servo/test.png\") top center / 200px 200px repeat-x fixed padding-box \
        content-box red");
    let result = background::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.background_image, parse_longhand!(background_image, "url(\"http://servo/test.png\")"));
    assert_eq!(result.background_position_x, parse_longhand!(background_position_x, "center"));
    assert_eq!(result.background_position_y, parse_longhand!(background_position_y, "top"));
    assert_eq!(result.background_size, parse_longhand!(background_size, "200px 200px"));
    assert_eq!(result.background_repeat, parse_longhand!(background_repeat, "repeat-x"));
    assert_eq!(result.background_attachment, parse_longhand!(background_attachment, "fixed"));
    assert_eq!(result.background_origin, parse_longhand!(background_origin, "padding-box"));
    assert_eq!(result.background_clip, parse_longhand!(background_clip, "content-box"));
    assert_eq!(result.background_color, parse_longhand!(background_color, "red"));
}

#[test]
fn background_shorthand_should_parse_when_some_fields_set() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter, Some(CssRuleType::Style));
    let mut parser = Parser::new("14px 40px repeat-y");
    let result = background::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.background_position_x, parse_longhand!(background_position_x, "14px"));
    assert_eq!(result.background_position_y, parse_longhand!(background_position_y, "40px"));
    assert_eq!(result.background_repeat, parse_longhand!(background_repeat, "repeat-y"));

    let mut parser = Parser::new("url(\"http://servo/test.png\") repeat blue");
    let result = background::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.background_image, parse_longhand!(background_image, "url(\"http://servo/test.png\")"));
    assert_eq!(result.background_repeat, parse_longhand!(background_repeat, "repeat"));
    assert_eq!(result.background_color, parse_longhand!(background_color, "blue"));

    let mut parser = Parser::new("padding-box");
    let result = background::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.background_origin, parse_longhand!(background_origin, "padding-box"));
    assert_eq!(result.background_clip, parse_longhand!(background_clip, "padding-box"));

    let mut parser = Parser::new("url(\"http://servo/test.png\")");
    let result = background::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.background_image, parse_longhand!(background_image, "url(\"http://servo/test.png\")"));
}

#[test]
fn background_shorthand_should_parse_comma_separated_declarations() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter, Some(CssRuleType::Style));
    let mut parser = Parser::new("url(\"http://servo/test.png\") top left no-repeat, url(\"http://servo/test.png\") \
        center / 100% 100% no-repeat, white");
    let result = background::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.background_image, parse_longhand!(background_image, "url(\"http://servo/test.png\"), \
        url(\"http://servo/test.png\"), none"));
    assert_eq!(result.background_position_x, parse_longhand!(background_position_x, "left, center, 0%"));
    assert_eq!(result.background_position_y, parse_longhand!(background_position_y, "top, center, 0%"));
    assert_eq!(result.background_repeat, parse_longhand!(background_repeat, "no-repeat, no-repeat, repeat"));
    assert_eq!(result.background_clip, parse_longhand!(background_clip, "border-box, border-box, border-box"));
    assert_eq!(result.background_origin, parse_longhand!(background_origin, "padding-box, padding-box, \
        padding-box"));
    assert_eq!(result.background_size, parse_longhand!(background_size, "auto auto, 100% 100%, auto auto"));
    assert_eq!(result.background_attachment, parse_longhand!(background_attachment, "scroll, scroll, scroll"));
    assert_eq!(result.background_color, parse_longhand!(background_color, "white"));
}

#[test]
fn background_shorthand_should_parse_position_and_size_correctly() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter, Some(CssRuleType::Style));
    let mut parser = Parser::new("7px 4px");
    let result = background::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.background_position_x, parse_longhand!(background_position_x, "7px"));
    assert_eq!(result.background_position_y, parse_longhand!(background_position_y, "4px"));

    let mut parser = Parser::new("7px 4px / 30px 20px");
    let result = background::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.background_position_x, parse_longhand!(background_position_x, "7px"));
    assert_eq!(result.background_position_y, parse_longhand!(background_position_y, "4px"));
    assert_eq!(result.background_size, parse_longhand!(background_size, "30px 20px"));

    let mut parser = Parser::new("/ 30px 20px");
    assert!(background::parse_value(&context, &mut parser).is_err());

    let mut parser = Parser::new("repeat-x / 30px 20px");
    assert!(background::parse_value(&context, &mut parser).is_err());
}

#[test]
fn background_shorthand_should_parse_origin_and_clip_correctly() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter, Some(CssRuleType::Style));
    let mut parser = Parser::new("padding-box content-box");
    let result = background::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.background_origin, parse_longhand!(background_origin, "padding-box"));
    assert_eq!(result.background_clip, parse_longhand!(background_clip, "content-box"));

    let mut parser = Parser::new("padding-box padding-box");
    let result = background::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.background_origin, parse_longhand!(background_origin, "padding-box"));
    assert_eq!(result.background_clip, parse_longhand!(background_clip, "padding-box"));

    let mut parser = Parser::new("padding-box");
    let result = background::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.background_origin, parse_longhand!(background_origin, "padding-box"));
    assert_eq!(result.background_clip, parse_longhand!(background_clip, "padding-box"));
}
