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

