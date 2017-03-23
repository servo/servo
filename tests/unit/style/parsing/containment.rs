/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::parser::ParserContext;
use style::stylesheets::Origin;

#[test]
fn contain_longhand_should_parse_correctly() {
    use style::properties::longhands::contain;
    use style::properties::longhands::contain::SpecifiedValue;

    let none = parse_longhand!(contain, "none");
    assert_eq!(none, SpecifiedValue::empty());

    let strict = parse_longhand!(contain, "strict");
    assert_eq!(strict, contain::STRICT);

    let style_paint = parse_longhand!(contain, "style paint");
    assert_eq!(style_paint, contain::STYLE | contain::PAINT);

    // Assert that the `2px` is not consumed, which would trigger parsing failure in real use
    assert_parser_exhausted!(contain, "layout 2px", false);
}
