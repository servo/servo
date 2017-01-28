/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use servo_url::ServoUrl;
use style::parser::ParserContext;
use style::properties::longhands::{border_image_outset, border_image_repeat, border_image_slice};
use style::properties::longhands::{border_image_source, border_image_width};
use style::properties::shorthands::border_image;
use style::stylesheets::Origin;

#[test]
fn border_image_shorthand_should_parse_when_all_properties_specified() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("linear-gradient(red, blue) 30 30% 45 fill / 20px 40px / 10px \
                                 round stretch");
    let result = border_image::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.border_image_source.unwrap(),
               parse_longhand!(border_image_source, "linear-gradient(red, blue)"));
    assert_eq!(result.border_image_slice.unwrap(), parse_longhand!(border_image_slice, "30 30% 45 fill"));
    assert_eq!(result.border_image_width.unwrap(), parse_longhand!(border_image_width, "20px 40px"));
    assert_eq!(result.border_image_outset.unwrap(), parse_longhand!(border_image_outset, "10px"));
    assert_eq!(result.border_image_repeat.unwrap(), parse_longhand!(border_image_repeat, "round stretch"));
}

#[test]
fn border_image_shorthand_should_parse_without_width() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("linear-gradient(red, blue) 30 30% 45 fill / / 10px round stretch");
    let result = border_image::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.border_image_source.unwrap(),
               parse_longhand!(border_image_source, "linear-gradient(red, blue)"));
    assert_eq!(result.border_image_slice.unwrap(), parse_longhand!(border_image_slice, "30 30% 45 fill"));
    assert_eq!(result.border_image_outset.unwrap(), parse_longhand!(border_image_outset, "10px"));
    assert_eq!(result.border_image_repeat.unwrap(), parse_longhand!(border_image_repeat, "round stretch"));
    assert_eq!(result.border_image_width.unwrap(), border_image_width::get_initial_specified_value());
}

#[test]
fn border_image_shorthand_should_parse_without_outset() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("linear-gradient(red, blue) 30 30% 45 fill / 20px 40px round");
    let result = border_image::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.border_image_source.unwrap(),
               parse_longhand!(border_image_source, "linear-gradient(red, blue)"));
    assert_eq!(result.border_image_slice.unwrap(), parse_longhand!(border_image_slice, "30 30% 45 fill"));
    assert_eq!(result.border_image_width.unwrap(), parse_longhand!(border_image_width, "20px 40px"));
    assert_eq!(result.border_image_repeat.unwrap(), parse_longhand!(border_image_repeat, "round"));
    assert_eq!(result.border_image_outset.unwrap(), border_image_outset::get_initial_specified_value());
}

#[test]
fn border_image_shorthand_should_parse_without_width_or_outset() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("linear-gradient(red, blue) 30 30% 45 fill round");
    let result = border_image::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.border_image_source.unwrap(),
               parse_longhand!(border_image_source, "linear-gradient(red, blue)"));
    assert_eq!(result.border_image_slice.unwrap(), parse_longhand!(border_image_slice, "30 30% 45 fill"));
    assert_eq!(result.border_image_repeat.unwrap(), parse_longhand!(border_image_repeat, "round"));
    assert_eq!(result.border_image_width.unwrap(), border_image_width::get_initial_specified_value());
    assert_eq!(result.border_image_outset.unwrap(), border_image_outset::get_initial_specified_value());
}

#[test]
fn border_image_shorthand_should_parse_with_just_source() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("linear-gradient(red, blue)");
    let result = border_image::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.border_image_source.unwrap(),
               parse_longhand!(border_image_source, "linear-gradient(red, blue)"));
    assert_eq!(result.border_image_slice.unwrap(), border_image_slice::get_initial_specified_value());
    assert_eq!(result.border_image_width.unwrap(), border_image_width::get_initial_specified_value());
    assert_eq!(result.border_image_outset.unwrap(), border_image_outset::get_initial_specified_value());
    assert_eq!(result.border_image_repeat.unwrap(), border_image_repeat::get_initial_specified_value());
}

#[test]
fn border_image_outset_should_error_on_negative_length() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("-1em");
    let result = border_image_outset::parse(&context, &mut parser);
    assert_eq!(result, Err(()));
}

#[test]
fn border_image_outset_should_error_on_negative_number() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("-15");
    let result = border_image_outset::parse(&context, &mut parser);
    assert_eq!(result, Err(()));
}

#[test]
fn border_image_outset_should_return_number_on_plain_zero() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("0");
    let result = border_image_outset::parse(&context, &mut parser);
    assert_eq!(result.unwrap(), parse_longhand!(border_image_outset, "0"));
}

#[test]
fn border_image_outset_should_return_length_on_length_zero() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("0em");
    let result = border_image_outset::parse(&context, &mut parser);
    assert_eq!(result.unwrap(), parse_longhand!(border_image_outset, "0em"));
}
