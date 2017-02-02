/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::parser::ParserContext;
use style::stylesheets::Origin;
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

}
