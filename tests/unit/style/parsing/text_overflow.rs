/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use style_traits::ToCss;

#[test]
fn test_text_overflow() {
    use style::properties::longhands::text_overflow;

    assert_roundtrip_with_context!(text_overflow::parse, r#"clip"#);
    assert_roundtrip_with_context!(text_overflow::parse, r#"ellipsis"#);
    assert_roundtrip_with_context!(text_overflow::parse, r#"clip ellipsis"#);
    assert_roundtrip_with_context!(text_overflow::parse, r#""x""#);
    assert_roundtrip_with_context!(text_overflow::parse, r#"'x'"#, r#""x""#);
    assert_roundtrip_with_context!(text_overflow::parse, r#"clip "x""#);
    assert_roundtrip_with_context!(text_overflow::parse, r#""x" clip"#);
    assert_roundtrip_with_context!(text_overflow::parse, r#""x" "y""#);
}

#[test]
fn test_text_overflow_parser_exhaustion() {
    use style::properties::longhands::text_overflow;

    assert_parser_exhausted!(text_overflow::parse, r#"clip rubbish"#, false);
    assert_parser_exhausted!(text_overflow::parse, r#"clip"#, true);
    assert_parser_exhausted!(text_overflow::parse, r#"ellipsis"#, true);
    assert_parser_exhausted!(text_overflow::parse, r#"clip ellipsis"#, true);
}
