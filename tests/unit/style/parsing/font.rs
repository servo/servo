/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use style::properties::longhands::font_weight;

#[test]
fn font_weight_keyword_should_preserve_keyword() {
    use style::properties::longhands::font_weight::SpecifiedValue;

    let result = parse(font_weight::parse, "normal").unwrap();
    assert_eq!(result, SpecifiedValue::Normal);

    let result = parse(font_weight::parse, "bold").unwrap();
    assert_eq!(result, SpecifiedValue::Bold);
}
