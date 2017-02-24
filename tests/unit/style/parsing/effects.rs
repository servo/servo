/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use servo_url::ServoUrl;
use style::parser::ParserContext;
use style::stylesheets::Origin;
use style_traits::ToCss;
use style::properties::longhands;

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
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));

    let mut parser = Parser::new("1px 2px rubbish");
    let parsed = longhands::parse_origin(&context, &mut parser);
    assert_eq!(parsed.is_ok(), true);
    assert_eq!(parser.is_exhausted(), false);

    let mut parser = Parser::new("1px 2px");
    let parsed = longhands::parse_origin(&context, &mut parser);
    assert_eq!(parsed.is_ok(), true);
    assert_eq!(parser.is_exhausted(), true);

    let mut parser = Parser::new("1px");
    let parsed = longhands::parse_origin(&context, &mut parser);
    assert_eq!(parsed.is_ok(), true);
    assert_eq!(parser.is_exhausted(), true);
}
