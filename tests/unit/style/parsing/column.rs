/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use servo_url::ServoUrl;
use style::parser::ParserContext;
use style::stylesheets::{CssRuleType, Origin};
use style_traits::ToCss;

#[test]
fn test_column_width() {
    use style::properties::longhands::column_width;

    assert_roundtrip_with_context!(column_width::parse, "auto");
    assert_roundtrip_with_context!(column_width::parse, "6px");
    assert_roundtrip_with_context!(column_width::parse, "2.5em");
    assert_roundtrip_with_context!(column_width::parse, "0.3vw");

    let url = ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter, Some(CssRuleType::Style));

    let mut negative = Parser::new("-6px");
    assert!(column_width::parse(&context, &mut negative).is_err());
}

#[test]
fn test_column_gap() {
    use style::properties::longhands::column_gap;

    assert_roundtrip_with_context!(column_gap::parse, "normal");
    assert_roundtrip_with_context!(column_gap::parse, "6px");
    assert_roundtrip_with_context!(column_gap::parse, "2.5em");
    assert_roundtrip_with_context!(column_gap::parse, "0.3vw");

    let url = ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter, Some(CssRuleType::Style));

    let mut negative = Parser::new("-6px");
    assert!(column_gap::parse(&context, &mut negative).is_err());
}
