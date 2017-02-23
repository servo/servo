/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use servo_url::ServoUrl;
use style::parser::ParserContext;
use style::stylesheets::Origin;
use style_traits::ToCss;

#[test]
fn test_moz_user_select() {
    use style::properties::longhands::_moz_user_select;

    assert_roundtrip_with_context!(_moz_user_select::parse, "auto");
    assert_roundtrip_with_context!(_moz_user_select::parse, "text");
    assert_roundtrip_with_context!(_moz_user_select::parse, "none");
    assert_roundtrip_with_context!(_moz_user_select::parse, "element");
    assert_roundtrip_with_context!(_moz_user_select::parse, "elements");
    assert_roundtrip_with_context!(_moz_user_select::parse, "toggle");
    assert_roundtrip_with_context!(_moz_user_select::parse, "tri_state");
    assert_roundtrip_with_context!(_moz_user_select::parse, "-moz-all");
    assert_roundtrip_with_context!(_moz_user_select::parse, "-moz-none");
    assert_roundtrip_with_context!(_moz_user_select::parse, "-moz-text");

    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));

    let mut negative = Parser::new("potato");
    assert!(_moz_user_select::parse(&context, &mut negative).is_err());
}
