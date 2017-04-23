/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use parsing::parse;
use style::context::QuirksMode;
use style::parser::{LengthParsingMode, Parse, ParserContext};
use style::stylesheets::{CssRuleType, Origin};
use style::values::specified::length::{AbsoluteLength, Length, NoCalcLength};
use style_traits::ToCss;

#[test]
fn test_calc() {
    assert!(parse(Length::parse, "calc(1px+ 2px)").is_err());
    assert!(parse(Length::parse, "calc( 1px + 2px )").is_ok());
    assert!(parse(Length::parse, "calc(1px + 2px )").is_ok());
    assert!(parse(Length::parse, "calc( 1px + 2px)").is_ok());
}

#[test]
fn test_length_literals() {
    assert_roundtrip_with_context!(Length::parse, "0.33px", "0.33px");
    assert_roundtrip_with_context!(Length::parse, "0.33in", "0.33in");
    assert_roundtrip_with_context!(Length::parse, "0.33cm", "0.33cm");
    assert_roundtrip_with_context!(Length::parse, "0.33mm", "0.33mm");
    assert_roundtrip_with_context!(Length::parse, "0.33q", "0.33q");
    assert_roundtrip_with_context!(Length::parse, "0.33pt", "0.33pt");
    assert_roundtrip_with_context!(Length::parse, "0.33pc", "0.33pc");
}

#[test]
fn test_length_parsing_modes() {
    // In default length mode, non-zero lengths must have a unit.
    assert!(parse(Length::parse, "1").is_err());

    // In SVG length mode, non-zero lengths are assumed to be px.
    let url = ::servo_url::ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter,
                                     Some(CssRuleType::Style), LengthParsingMode::SVG,
                                     QuirksMode::NoQuirks);
    let mut parser = Parser::new("1");
    let result = Length::parse(&context, &mut parser);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Length::NoCalc(NoCalcLength::Absolute(AbsoluteLength::Px(1.))));
}
