/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::values::generics::text::Spacing;

use crate::parsing::parse;

#[test]
fn negative_letter_spacing_should_parse_properly() {
    use style::properties::longhands::letter_spacing;
    use style::values::specified::length::{FontRelativeLength, Length, NoCalcLength};

    let negative_value = parse_longhand!(letter_spacing, "-0.5em");
    let expected = Spacing::Value(Length::NoCalc(NoCalcLength::FontRelative(
        FontRelativeLength::Em(-0.5),
    )));
    assert_eq!(negative_value, expected);
}

#[test]
fn negative_word_spacing_should_parse_properly() {
    use style::properties::longhands::word_spacing;
    use style::values::specified::length::{FontRelativeLength, LengthPercentage, NoCalcLength};

    let negative_value = parse_longhand!(word_spacing, "-0.5em");
    let expected = Spacing::Value(LengthPercentage::Length(NoCalcLength::FontRelative(
        FontRelativeLength::Em(-0.5),
    )));
    assert_eq!(negative_value, expected);
}

#[test]
fn line_height_should_return_number_on_plain_zero() {
    use style::properties::longhands::line_height;

    let result = parse(line_height::parse, "0").unwrap();
    assert_eq!(result, parse_longhand!(line_height, "0"));
}

#[test]
fn line_height_should_return_length_on_length_zero() {
    use style::properties::longhands::line_height;

    let result = parse(line_height::parse, "0px").unwrap();
    assert_eq!(result, parse_longhand!(line_height, "0px"));
}
