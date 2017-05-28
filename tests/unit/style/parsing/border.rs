/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use style::parser::Parse;
use style::properties::MaybeBoxed;
use style::properties::longhands::{border_image_outset, border_image_repeat, border_image_slice};
use style::properties::longhands::{border_image_source, border_image_width};
use style::properties::shorthands::border_image;
use style::values::specified::BorderRadius;
use style_traits::ToCss;

macro_rules! assert_longhand {
    ($parsed_shorthand: expr, $prop: ident, $value_string: expr) => {
        assert_eq!($parsed_shorthand.$prop, parse_longhand!($prop, $value_string).maybe_boxed())
    }
}

macro_rules! assert_initial {
    ($parsed_shorthand: expr, $prop: ident) => {
        assert_eq!($parsed_shorthand.$prop, $prop::get_initial_specified_value().maybe_boxed())
    }
}

macro_rules! assert_border_radius_values {
    ($input:expr; $tlw:expr, $trw:expr, $brw:expr, $blw:expr ;
                  $tlh:expr, $trh:expr, $brh:expr, $blh:expr) => {
        let input = parse(BorderRadius::parse, $input)
                          .expect(&format!("Failed parsing {} as border radius",
                                  $input));
        assert_eq!(::style_traits::ToCss::to_css_string(&input.top_left.0.width), $tlw);
        assert_eq!(::style_traits::ToCss::to_css_string(&input.top_right.0.width), $trw);
        assert_eq!(::style_traits::ToCss::to_css_string(&input.bottom_right.0.width), $brw);
        assert_eq!(::style_traits::ToCss::to_css_string(&input.bottom_left.0.width), $blw);
        assert_eq!(::style_traits::ToCss::to_css_string(&input.top_left.0.height), $tlh);
        assert_eq!(::style_traits::ToCss::to_css_string(&input.top_right.0.height), $trh);
        assert_eq!(::style_traits::ToCss::to_css_string(&input.bottom_right.0.height), $brh);
        assert_eq!(::style_traits::ToCss::to_css_string(&input.bottom_left.0.height), $blh);
    }
}

#[test]
fn test_border_radius() {
    assert_border_radius_values!("10px";
                                 "10px", "10px", "10px", "10px" ;
                                 "10px", "10px", "10px", "10px");
    assert_border_radius_values!("10px 20px";
                                 "10px", "20px", "10px", "20px" ;
                                 "10px", "20px", "10px", "20px");
    assert_border_radius_values!("10px 20px 30px";
                                 "10px", "20px", "30px", "20px" ;
                                 "10px", "20px", "30px", "20px");
    assert_border_radius_values!("10px 20px 30px 40px";
                                 "10px", "20px", "30px", "40px" ;
                                 "10px", "20px", "30px", "40px");
    assert_border_radius_values!("10% / 20px";
                                 "10%", "10%", "10%", "10%" ;
                                 "20px", "20px", "20px", "20px");
    assert_border_radius_values!("10px / 20px 30px";
                                 "10px", "10px", "10px", "10px" ;
                                 "20px", "30px", "20px", "30px");
    assert_border_radius_values!("10px 20px 30px 40px / 1px 2px 3px 4px";
                                 "10px", "20px", "30px", "40px" ;
                                 "1px", "2px", "3px", "4px");
    assert_border_radius_values!("10px 20px 30px 40px / 1px 2px 3px 4px";
                                 "10px", "20px", "30px", "40px" ;
                                 "1px", "2px", "3px", "4px");
    assert_border_radius_values!("10px 20px 30px 40px / 1px 2px 3px 4px";
                                 "10px", "20px", "30px", "40px" ;
                                 "1px", "2px", "3px", "4px");
    assert_border_radius_values!("10px -20px 30px 40px";
                                 "10px", "10px", "10px", "10px";
                                 "10px", "10px", "10px", "10px");
    assert_border_radius_values!("10px 20px -30px 40px";
                                 "10px", "20px", "10px", "20px";
                                 "10px", "20px", "10px", "20px");
    assert_border_radius_values!("10px 20px 30px -40px";
                                 "10px", "20px", "30px", "20px";
                                 "10px", "20px", "30px", "20px");
    assert!(parse(BorderRadius::parse, "-10px 20px 30px 40px").is_err());
}

#[test]
fn border_image_shorthand_should_parse_when_all_properties_specified() {
    let input = "linear-gradient(red, blue) 30 30% 45 fill / 20px 40px / 10px round stretch";
    let result = parse(border_image::parse_value, input).unwrap();

    assert_longhand!(result, border_image_source, "linear-gradient(red, blue)");
    assert_longhand!(result, border_image_slice, "30 30% 45 fill");
    assert_longhand!(result, border_image_width, "20px 40px");
    assert_longhand!(result, border_image_outset, "10px");
    assert_longhand!(result, border_image_repeat, "round stretch");
}

#[test]
fn border_image_shorthand_should_parse_without_width() {
    let input = "linear-gradient(red, blue) 30 30% 45 fill / / 10px round stretch";
    let result = parse(border_image::parse_value, input).unwrap();

    assert_longhand!(result, border_image_source, "linear-gradient(red, blue)");
    assert_longhand!(result, border_image_slice, "30 30% 45 fill");
    assert_longhand!(result, border_image_outset, "10px");
    assert_longhand!(result, border_image_repeat, "round stretch");
    assert_initial!(result, border_image_width);
}

#[test]
fn border_image_shorthand_should_parse_without_outset() {
    let input = "linear-gradient(red, blue) 30 30% 45 fill / 20px 40px round";
    let result = parse(border_image::parse_value, input).unwrap();

    assert_longhand!(result, border_image_source, "linear-gradient(red, blue)");
    assert_longhand!(result, border_image_slice, "30 30% 45 fill");
    assert_longhand!(result, border_image_width, "20px 40px");
    assert_longhand!(result, border_image_repeat, "round");
    assert_initial!(result, border_image_outset);
}

#[test]
fn border_image_shorthand_should_parse_without_width_or_outset() {
    let input = "linear-gradient(red, blue) 30 30% 45 fill round";
    let result = parse(border_image::parse_value, input).unwrap();

    assert_longhand!(result, border_image_source, "linear-gradient(red, blue)");
    assert_longhand!(result, border_image_slice, "30 30% 45 fill");
    assert_longhand!(result, border_image_repeat, "round");
    assert_initial!(result, border_image_width);
    assert_initial!(result, border_image_outset);
}

#[test]
fn border_image_shorthand_should_parse_with_just_source() {
    let result = parse(border_image::parse_value, "linear-gradient(red, blue)").unwrap();

    assert_longhand!(result, border_image_source, "linear-gradient(red, blue)");
    assert_initial!(result, border_image_slice);
    assert_initial!(result, border_image_width);
    assert_initial!(result, border_image_outset);
    assert_initial!(result, border_image_repeat);
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
