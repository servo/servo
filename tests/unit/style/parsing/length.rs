/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use parsing::parse;
use style::parser::{Parse, ParserContext};
use style::stylesheets::{CssRuleType, Origin};
use style::values::specified::length::Length;
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
