/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use style::values::generics::text::Spacing;

#[test]
fn negative_letter_spacing_should_parse_properly() {
    use style::properties::longhands::letter_spacing;
    use style::values::specified::length::{Length, NoCalcLength, FontRelativeLength};

    let negative_value = parse_longhand!(letter_spacing, "-0.5em");
    let expected = Spacing::Value(Length::NoCalc(NoCalcLength::FontRelative(FontRelativeLength::Em(-0.5))));
    assert_eq!(negative_value, expected);
}

#[test]
fn negative_word_spacing_should_parse_properly() {
    use style::properties::longhands::word_spacing;
    use style::values::specified::length::{NoCalcLength, LengthOrPercentage, FontRelativeLength};

    let negative_value = parse_longhand!(word_spacing, "-0.5em");
    let expected = Spacing::Value(LengthOrPercentage::Length(
        NoCalcLength::FontRelative(FontRelativeLength::Em(-0.5))
    ));
    assert_eq!(negative_value, expected);
}

#[test]
fn text_emphasis_style_longhand_should_parse_properly() {
    use style::properties::longhands::text_emphasis_style;
    use style::properties::longhands::text_emphasis_style::{ShapeKeyword, SpecifiedValue, KeywordValue};

    let none = parse_longhand!(text_emphasis_style, "none");
    assert_eq!(none, SpecifiedValue::None);

    let fill = parse_longhand!(text_emphasis_style, "open");
    let fill_struct = SpecifiedValue::Keyword(KeywordValue::Fill(false));
    assert_eq!(fill, fill_struct);

    let shape = parse_longhand!(text_emphasis_style, "triangle");
    let shape_struct = SpecifiedValue::Keyword(KeywordValue::Shape(ShapeKeyword::Triangle));
    assert_eq!(shape, shape_struct);

    let fill_shape = parse_longhand!(text_emphasis_style, "filled dot");
    let fill_shape_struct = SpecifiedValue::Keyword(KeywordValue::FillAndShape(true, ShapeKeyword::Dot));
    assert_eq!(fill_shape, fill_shape_struct);

    let shape_fill = parse_longhand!(text_emphasis_style, "dot filled");
    let shape_fill_struct = SpecifiedValue::Keyword(KeywordValue::FillAndShape(true, ShapeKeyword::Dot));
    assert_eq!(shape_fill, shape_fill_struct);

    let a_string = parse_longhand!(text_emphasis_style, "\"a\"");
    let a_string_struct = SpecifiedValue::String("a".to_string());
    assert_eq!(a_string, a_string_struct);

    let chinese_string = parse_longhand!(text_emphasis_style, "\"点\"");
    let chinese_string_struct = SpecifiedValue::String("点".to_string());
    assert_eq!(chinese_string, chinese_string_struct);

    let unicode_string = parse_longhand!(text_emphasis_style, "\"\\25B2\"");
    let unicode_string_struct = SpecifiedValue::String("▲".to_string());
    assert_eq!(unicode_string, unicode_string_struct);

    let devanagari_string = parse_longhand!(text_emphasis_style, "\"षि\"");
    let devanagari_string_struct = SpecifiedValue::String("षि".to_string());
    assert_eq!(devanagari_string, devanagari_string_struct);
}

#[test]
fn test_text_emphasis_position() {
    use style::properties::longhands::text_emphasis_position;
    use style::properties::longhands::text_emphasis_position::{HorizontalWritingModeValue, VerticalWritingModeValue};
    use style::properties::longhands::text_emphasis_position::SpecifiedValue;

    let over_right = parse_longhand!(text_emphasis_position, "over right");
    assert_eq!(over_right, SpecifiedValue(HorizontalWritingModeValue::Over, VerticalWritingModeValue::Right));

    let over_left = parse_longhand!(text_emphasis_position, "over left");
    assert_eq!(over_left, SpecifiedValue(HorizontalWritingModeValue::Over, VerticalWritingModeValue::Left));

    let under_right = parse_longhand!(text_emphasis_position, "under right");
    assert_eq!(under_right, SpecifiedValue(HorizontalWritingModeValue::Under, VerticalWritingModeValue::Right));

    let under_left = parse_longhand!(text_emphasis_position, "under left");
    assert_eq!(under_left, SpecifiedValue(HorizontalWritingModeValue::Under, VerticalWritingModeValue::Left));

    let right_over = parse_longhand!(text_emphasis_position, "right over");
    assert_eq!(right_over, SpecifiedValue(HorizontalWritingModeValue::Over, VerticalWritingModeValue::Right));

    let left_over = parse_longhand!(text_emphasis_position, "left over");
    assert_eq!(left_over, SpecifiedValue(HorizontalWritingModeValue::Over, VerticalWritingModeValue::Left));

    let right_under = parse_longhand!(text_emphasis_position, "right under");
    assert_eq!(right_under, SpecifiedValue(HorizontalWritingModeValue::Under, VerticalWritingModeValue::Right));

    let left_under = parse_longhand!(text_emphasis_position, "left under");
    assert_eq!(left_under, SpecifiedValue(HorizontalWritingModeValue::Under, VerticalWritingModeValue::Left));
}

#[test]
fn webkit_text_stroke_shorthand_should_parse_properly() {
    use style::properties::longhands::_webkit_text_stroke_color;
    use style::properties::longhands::_webkit_text_stroke_width;
    use style::properties::shorthands::_webkit_text_stroke;

    let result = parse(_webkit_text_stroke::parse_value, "thin red").unwrap();
    assert_eq!(result._webkit_text_stroke_color, parse_longhand!(_webkit_text_stroke_color, "red"));
    assert_eq!(result._webkit_text_stroke_width, parse_longhand!(_webkit_text_stroke_width, "thin"));

    // ensure its no longer sensitive to order
    let result = parse(_webkit_text_stroke::parse_value, "red thin").unwrap();
    assert_eq!(result._webkit_text_stroke_color, parse_longhand!(_webkit_text_stroke_color, "red"));
    assert_eq!(result._webkit_text_stroke_width, parse_longhand!(_webkit_text_stroke_width, "thin"));
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
