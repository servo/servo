/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use style_traits::ToCss;

#[test]
fn initial_letter_should_be_parsed_correctly() {
    use style::properties::longhands::initial_letter;

    assert_roundtrip_with_context!(initial_letter::parse, "1.5");
    assert_roundtrip_with_context!(initial_letter::parse, "1.5 3");
    assert_roundtrip_with_context!(initial_letter::parse, "normal");
}

#[test]
fn initial_letter_doesnt_parse_invalid_input() {
    use style::properties::longhands::initial_letter;

    assert!(parse(initial_letter::parse, "1.5x 5").is_err());
}
