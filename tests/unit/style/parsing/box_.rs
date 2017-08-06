/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use style_traits::ToCss;

#[test]
fn test_transform_translate() {
    use style::properties::longhands::transform;
    assert_roundtrip_with_context!(transform::parse, "translate(2px)");
    assert_roundtrip_with_context!(transform::parse, "translate(2px, 5px)");
    assert!(parse(transform::parse, "translate(2px foo)").is_err());
    assert!(parse(transform::parse, "perspective(-10px)").is_err());
}

#[test]
fn test_unexhausted_transform() {
    use style::properties::longhands::transform;
    assert_parser_exhausted!(transform::parse, "rotate(70deg)foo", false);
    assert_parser_exhausted!(transform::parse, "rotate(70deg) foo", false);
}
