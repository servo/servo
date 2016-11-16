/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use servo_url::ServoUrl;
use style::parser::ParserContext;
use style::properties::longhands::{mask_clip, mask_composite, mask_image, mask_mode};
use style::properties::longhands::{mask_origin, mask_position, mask_repeat, mask_size};
use style::properties::shorthands::mask;
use style::stylesheets::Origin;

#[test]
fn mask_shorthand_should_parse_all_available_properties_when_specified() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("url(\"http://servo/test.png\") luminance 7px 4px / 70px 50px \
                                 repeat-x padding-box border-box subtract");
    let result = mask::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.mask_image.unwrap(), parse_longhand!(mask_image, "url(\"http://servo/test.png\")"));
    assert_eq!(result.mask_mode.unwrap(), parse_longhand!(mask_mode, "luminance"));
    assert_eq!(result.mask_position.unwrap(), parse_longhand!(mask_position, "7px 4px"));
    assert_eq!(result.mask_size.unwrap(), parse_longhand!(mask_size, "70px 50px"));
    assert_eq!(result.mask_repeat.unwrap(), parse_longhand!(mask_repeat, "repeat-x"));
    assert_eq!(result.mask_origin.unwrap(), parse_longhand!(mask_origin, "padding-box"));
    assert_eq!(result.mask_clip.unwrap(), parse_longhand!(mask_clip, "border-box"));
    assert_eq!(result.mask_composite.unwrap(), parse_longhand!(mask_composite, "subtract"));
}

#[test]
fn mask_shorthand_should_parse_when_some_fields_set() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("14px 40px repeat-y");
    let result = mask::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.mask_position.unwrap(), parse_longhand!(mask_position, "14px 40px"));
    assert_eq!(result.mask_repeat.unwrap(), parse_longhand!(mask_repeat, "repeat-y"));

    let mut parser = Parser::new("url(\"http://servo/test.png\") repeat add");
    let result = mask::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.mask_image.unwrap(), parse_longhand!(mask_image, "url(\"http://servo/test.png\")"));
    assert_eq!(result.mask_repeat.unwrap(), parse_longhand!(mask_repeat, "repeat"));
    assert_eq!(result.mask_composite.unwrap(), parse_longhand!(mask_composite, "add"));

    let mut parser = Parser::new("intersect");
    let result = mask::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.mask_composite.unwrap(), parse_longhand!(mask_composite, "intersect"));

    let mut parser = Parser::new("url(\"http://servo/test.png\")");
    let result = mask::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.mask_image.unwrap(), parse_longhand!(mask_image, "url(\"http://servo/test.png\")"));
}

#[test]
fn mask_shorthand_should_parse_position_and_size_correctly() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("7px 4px");
    let result = mask::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.mask_position.unwrap(), parse_longhand!(mask_position, "7px 4px"));

    let mut parser = Parser::new("7px 4px / 30px 20px");
    let result = mask::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.mask_position.unwrap(), parse_longhand!(mask_position, "7px 4px"));
    assert_eq!(result.mask_size.unwrap(), parse_longhand!(mask_size, "30px 20px"));

    let mut parser = Parser::new("/ 30px 20px");
    assert!(mask::parse_value(&context, &mut parser).is_err());

    let mut parser = Parser::new("match-source repeat-x / 30px 20px");
    assert!(mask::parse_value(&context, &mut parser).is_err());
}

#[test]
fn mask_shorthand_should_parse_origin_and_clip_correctly() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("padding-box content-box");
    let result = mask::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.mask_origin.unwrap(), parse_longhand!(mask_origin, "padding-box"));
    assert_eq!(result.mask_clip.unwrap(), parse_longhand!(mask_clip, "content-box"));

    let mut parser = Parser::new("padding-box padding-box");
    let result = mask::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.mask_origin.unwrap(), parse_longhand!(mask_origin, "padding-box"));
    assert_eq!(result.mask_clip.unwrap(), parse_longhand!(mask_clip, "padding-box"));

    let mut parser = Parser::new("padding-box");
    let result = mask::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.mask_origin.unwrap(), parse_longhand!(mask_origin, "padding-box"));
    assert_eq!(result.mask_clip.unwrap(), parse_longhand!(mask_clip, "padding-box"));
}

#[test]
fn mask_shorthand_should_not_parse_when_mode_specified_but_image_not() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("luminance 7px 4px repeat-x padding");
    assert!(mask::parse_value(&context, &mut parser).is_err());

    let mut parser = Parser::new("alpha");
    assert!(mask::parse_value(&context, &mut parser).is_err());
}
