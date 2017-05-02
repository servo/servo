/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use style::parser::Parse;
use style::properties::longhands::{border_image_outset, border_image_repeat, border_image_slice};
use style::properties::longhands::{border_image_source, border_image_width};
use style::properties::shorthands::border_image;
use style_traits::ToCss;

#[test]
fn border_image_shorthand_should_parse_when_all_properties_specified() {
    let input = "linear-gradient(red, blue) 30 30% 45 fill / 20px 40px / 10px round stretch";
    let result = parse(border_image::parse_value, input).unwrap();

    assert_eq!(result.border_image_source,
               parse_longhand!(border_image_source, "linear-gradient(red, blue)"));
    assert_eq!(result.border_image_slice, parse_longhand!(border_image_slice, "30 30% 45 fill"));
    assert_eq!(result.border_image_width, parse_longhand!(border_image_width, "20px 40px"));
    assert_eq!(result.border_image_outset, parse_longhand!(border_image_outset, "10px"));
    assert_eq!(result.border_image_repeat, parse_longhand!(border_image_repeat, "round stretch"));
}

#[test]
fn border_image_shorthand_should_parse_without_width() {
    let input = "linear-gradient(red, blue) 30 30% 45 fill / / 10px round stretch";
    let result = parse(border_image::parse_value, input).unwrap();

    assert_eq!(result.border_image_source,
               parse_longhand!(border_image_source, "linear-gradient(red, blue)"));
    assert_eq!(result.border_image_slice, parse_longhand!(border_image_slice, "30 30% 45 fill"));
    assert_eq!(result.border_image_outset, parse_longhand!(border_image_outset, "10px"));
    assert_eq!(result.border_image_repeat, parse_longhand!(border_image_repeat, "round stretch"));
    assert_eq!(result.border_image_width, border_image_width::get_initial_specified_value());
}

#[test]
fn border_image_shorthand_should_parse_without_outset() {
    let input = "linear-gradient(red, blue) 30 30% 45 fill / 20px 40px round";
    let result = parse(border_image::parse_value, input).unwrap();

    assert_eq!(result.border_image_source,
               parse_longhand!(border_image_source, "linear-gradient(red, blue)"));
    assert_eq!(result.border_image_slice, parse_longhand!(border_image_slice, "30 30% 45 fill"));
    assert_eq!(result.border_image_width, parse_longhand!(border_image_width, "20px 40px"));
    assert_eq!(result.border_image_repeat, parse_longhand!(border_image_repeat, "round"));
    assert_eq!(result.border_image_outset, border_image_outset::get_initial_specified_value());
}

#[test]
fn border_image_shorthand_should_parse_without_width_or_outset() {
    let input = "linear-gradient(red, blue) 30 30% 45 fill round";
    let result = parse(border_image::parse_value, input).unwrap();

    assert_eq!(result.border_image_source,
               parse_longhand!(border_image_source, "linear-gradient(red, blue)"));
    assert_eq!(result.border_image_slice, parse_longhand!(border_image_slice, "30 30% 45 fill"));
    assert_eq!(result.border_image_repeat, parse_longhand!(border_image_repeat, "round"));
    assert_eq!(result.border_image_width, border_image_width::get_initial_specified_value());
    assert_eq!(result.border_image_outset, border_image_outset::get_initial_specified_value());
}

#[test]
fn border_image_shorthand_should_parse_with_just_source() {
    let result = parse(border_image::parse_value, "linear-gradient(red, blue)").unwrap();

    assert_eq!(result.border_image_source,
               parse_longhand!(border_image_source, "linear-gradient(red, blue)"));
    assert_eq!(result.border_image_slice, border_image_slice::get_initial_specified_value());
    assert_eq!(result.border_image_width, border_image_width::get_initial_specified_value());
    assert_eq!(result.border_image_outset, border_image_outset::get_initial_specified_value());
    assert_eq!(result.border_image_repeat, border_image_repeat::get_initial_specified_value());
}

#[test]
fn border_image_outset_should_error_on_negative_length() {
    let result = parse(border_image_outset::parse, "-1em");
    assert_eq!(result, Err(()));
}

#[test]
fn border_image_outset_should_error_on_negative_number() {
    let result = parse(border_image_outset::parse, "-15");
    assert_eq!(result, Err(()));
}

#[test]
fn border_image_outset_should_return_number_on_plain_zero() {
    let result = parse(border_image_outset::parse, "0");
    assert_eq!(result.unwrap(), parse_longhand!(border_image_outset, "0"));
}

#[test]
fn border_image_outset_should_return_length_on_length_zero() {
    let result = parse(border_image_outset::parse, "0em");
    assert_eq!(result.unwrap(), parse_longhand!(border_image_outset, "0em"));
}

#[test]
fn test_border_style() {
    use style::values::specified::BorderStyle;

    assert_roundtrip_with_context!(BorderStyle::parse, r#"none"#);
    assert_roundtrip_with_context!(BorderStyle::parse, r#"hidden"#);
    assert_roundtrip_with_context!(BorderStyle::parse, r#"solid"#);
    assert_roundtrip_with_context!(BorderStyle::parse, r#"double"#);
    assert_roundtrip_with_context!(BorderStyle::parse, r#"dotted"#);
    assert_roundtrip_with_context!(BorderStyle::parse, r#"dashed"#);
    assert_roundtrip_with_context!(BorderStyle::parse, r#"groove"#);
    assert_roundtrip_with_context!(BorderStyle::parse, r#"ridge"#);
    assert_roundtrip_with_context!(BorderStyle::parse, r#"inset"#);
    assert_roundtrip_with_context!(BorderStyle::parse, r#"outset"#);
}

#[test]
fn test_border_spacing() {
    use style::properties::longhands::border_spacing;

    assert_parser_exhausted!(border_spacing::parse, "1px rubbish", false);
    assert_parser_exhausted!(border_spacing::parse, "1px", true);
    assert_parser_exhausted!(border_spacing::parse, "1px 2px", true);
}
