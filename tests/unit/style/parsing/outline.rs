/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::parser::ParserContext;
use style::stylesheets::{CssRuleType, Origin};
use style_traits::ToCss;

#[test]
fn test_outline_style() {
    use style::properties::longhands::outline_style;

    assert_roundtrip_with_context!(outline_style::parse, r#"auto"#);
    assert_roundtrip_with_context!(outline_style::parse, r#"none"#);
    assert_roundtrip_with_context!(outline_style::parse, r#"solid"#);
    assert_roundtrip_with_context!(outline_style::parse, r#"double"#);
    assert_roundtrip_with_context!(outline_style::parse, r#"dotted"#);
    assert_roundtrip_with_context!(outline_style::parse, r#"dashed"#);
    assert_roundtrip_with_context!(outline_style::parse, r#"groove"#);
    assert_roundtrip_with_context!(outline_style::parse, r#"ridge"#);
    assert_roundtrip_with_context!(outline_style::parse, r#"inset"#);
    assert_roundtrip_with_context!(outline_style::parse, r#"outset"#);

    {
        // The outline-style property accepts the same values as border-style,
        // except that 'hidden' is not a legal outline style.

        let url = ::servo_url::ServoUrl::parse("http://localhost").unwrap();
        let reporter = CSSErrorReporterTest;
        let context = ParserContext::new(Origin::Author, &url, &reporter, Some(CssRuleType::Style));
        let mut parser = Parser::new(r#"hidden"#);
        let parsed = outline_style::parse(&context, &mut parser);
        assert!(parsed.is_err());
    };

}
