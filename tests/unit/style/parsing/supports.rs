/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use style::supports::SupportsCondition;
use style_traits::ToCss;

#[test]
fn test_supports_condition() {
    assert_roundtrip!(SupportsCondition::parse, "(margin: 1px)");
    assert_roundtrip!(SupportsCondition::parse, "not (--be: to be)");
    assert_roundtrip!(SupportsCondition::parse, "(color: blue) and future-extension(4)");
}
