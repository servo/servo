/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use servo_url::ServoUrl;
use style::computed_values::display::T::inline_block;
use style::parser::ParserContext;
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock, Importance, PropertyId};
use style::properties::longhands::outline_color::computed_value::T as ComputedColor;
use style::properties::parse_property_declaration_list;
use style::stylesheets::Origin;
use style::values::{RGBA, Auto};
use style::values::specified::{BorderStyle, BorderWidth, CSSColor, Length, NoCalcLength};
use style::values::specified::{LengthOrPercentage, LengthOrPercentageOrAuto, LengthOrPercentageOrAutoOrContent};
use style::values::specified::url::SpecifiedUrl;
use style_traits::ToCss;
use stylesheets::block_from;

fn parse_declaration_block(css_properties: &str) -> PropertyDeclarationBlock {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter);
    let mut parser = Parser::new(css_properties);
    parse_property_declaration_list(&context, &mut parser)
}

#[test]
fn property_declaration_block_should_serialize_correctly() {
    use style::properties::longhands::overflow_x::SpecifiedValue as OverflowXValue;
    use style::properties::longhands::overflow_y::SpecifiedValue as OverflowYContainer;

    let declarations = vec![
        (PropertyDeclaration::Width(
            LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(70f32))),
         Importance::Normal),

        (PropertyDeclaration::MinHeight(
            LengthOrPercentage::Length(NoCalcLength::from_px(20f32))),
         Importance::Normal),

        (PropertyDeclaration::Height(
            LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32))),
         Importance::Important),

        (PropertyDeclaration::Display(
            inline_block),
         Importance::Normal),

        (PropertyDeclaration::OverflowX(
            OverflowXValue::auto),
         Importance::Normal),

        (PropertyDeclaration::OverflowY(
            OverflowYContainer(OverflowXValue::auto)),
         Importance::Normal),
    ];

    let block = block_from(declarations);

    let css_string = block.to_css_string();

    assert_eq!(
        css_string,
        "width: 70px; min-height: 20px; height: 20px !important; display: inline-block; overflow: auto;"
    );
}

mod shorthand_serialization {
    pub use super::*;

    pub fn shorthand_properties_to_string(properties: Vec<PropertyDeclaration>) -> String {
        let block = block_from(properties.into_iter().map(|d| (d, Importance::Normal)));

        block.to_css_string()
    }

    // Add Test to show error if a longhand property is missing!!!!!!

    mod overflow {
        pub use super::*;
        use style::properties::longhands::overflow_x::SpecifiedValue as OverflowXValue;
        use style::properties::longhands::overflow_y::SpecifiedValue as OverflowYContainer;

        #[test]
        fn equal_overflow_properties_should_serialize_to_single_value() {
            let mut properties = Vec::new();

            let overflow_x = OverflowXValue::auto;
            properties.push(PropertyDeclaration::OverflowX(overflow_x));

            let overflow_y = OverflowYContainer(OverflowXValue::auto);
            properties.push(PropertyDeclaration::OverflowY(overflow_y));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "overflow: auto;");
        }

        #[test]
        fn different_overflow_properties_should_serialize_to_two_values() {
            let mut properties = Vec::new();

            let overflow_x = OverflowXValue::scroll;
            properties.push(PropertyDeclaration::OverflowX(overflow_x));

            let overflow_y = OverflowYContainer(OverflowXValue::auto);
            properties.push(PropertyDeclaration::OverflowY(overflow_y));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "overflow-x: scroll; overflow-y: auto;");
        }
    }

    mod text {
        use style::properties::longhands::text_decoration_line as TextDecorationLine;
        use style::properties::longhands::text_decoration_style::SpecifiedValue as TextDecorationStyle;
        use super::*;

        #[test]
        fn text_decoration_should_show_all_properties_when_set() {
            let mut properties = Vec::new();

            let line = TextDecorationLine::OVERLINE;
            let style = TextDecorationStyle::dotted;
            let color = CSSColor {
                parsed: ComputedColor::RGBA(RGBA::new(128, 0, 128, 255)),
                authored: None
            };

            properties.push(PropertyDeclaration::TextDecorationLine(line));
            properties.push(PropertyDeclaration::TextDecorationStyle(style));
            properties.push(PropertyDeclaration::TextDecorationColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "text-decoration: overline dotted rgb(128, 0, 128);");
        }

        #[test]
        fn text_decoration_should_not_serialize_initial_style_value() {
            let mut properties = Vec::new();

            let line = TextDecorationLine::UNDERLINE;
            let style = TextDecorationStyle::solid;
            let color = CSSColor::currentcolor();

            properties.push(PropertyDeclaration::TextDecorationLine(line));
            properties.push(PropertyDeclaration::TextDecorationStyle(style));
            properties.push(PropertyDeclaration::TextDecorationColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "text-decoration: underline;");
        }
    }

    mod four_sides_shorthands {
        pub use super::*;

        // we can use margin as a base to test out the different combinations
        // but afterwards, we only need to to one test per "four sides shorthand"
        #[test]
        fn all_equal_properties_should_serialize_to_one_value() {
            let mut properties = Vec::new();

            let px_70 = LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(70f32));
            properties.push(PropertyDeclaration::MarginTop(px_70.clone()));
            properties.push(PropertyDeclaration::MarginRight(px_70.clone()));
            properties.push(PropertyDeclaration::MarginBottom(px_70.clone()));
            properties.push(PropertyDeclaration::MarginLeft(px_70));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "margin: 70px;");
        }

        #[test]
        fn equal_vertical_and_equal_horizontal_properties_should_serialize_to_two_value() {
            let mut properties = Vec::new();

            let vertical_px = LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(10f32));
            let horizontal_px = LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(5f32));

            properties.push(PropertyDeclaration::MarginTop(vertical_px.clone()));
            properties.push(PropertyDeclaration::MarginRight(horizontal_px.clone()));
            properties.push(PropertyDeclaration::MarginBottom(vertical_px));
            properties.push(PropertyDeclaration::MarginLeft(horizontal_px));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "margin: 10px 5px;");
        }

        #[test]
        fn different_vertical_and_equal_horizontal_properties_should_serialize_to_three_values() {
            let mut properties = Vec::new();

            let top_px = LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(8f32));
            let bottom_px = LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(10f32));
            let horizontal_px = LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(5f32));

            properties.push(PropertyDeclaration::MarginTop(top_px));
            properties.push(PropertyDeclaration::MarginRight(horizontal_px.clone()));
            properties.push(PropertyDeclaration::MarginBottom(bottom_px));
            properties.push(PropertyDeclaration::MarginLeft(horizontal_px));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "margin: 8px 5px 10px;");
        }

        #[test]
        fn different_properties_should_serialize_to_four_values() {
            let mut properties = Vec::new();

            let top_px = LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(8f32));
            let right_px = LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(12f32));
            let bottom_px = LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(10f32));
            let left_px = LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(14f32));

            properties.push(PropertyDeclaration::MarginTop(top_px));
            properties.push(PropertyDeclaration::MarginRight(right_px));
            properties.push(PropertyDeclaration::MarginBottom(bottom_px));
            properties.push(PropertyDeclaration::MarginLeft(left_px));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "margin: 8px 12px 10px 14px;");
        }

        #[test]
        fn different_longhands_should_serialize_to_long_form() {
          let mut properties = Vec::new();

          let solid = BorderStyle::solid;

          properties.push(PropertyDeclaration::BorderTopStyle(solid.clone()));
          properties.push(PropertyDeclaration::BorderRightStyle(solid.clone()));
          properties.push(PropertyDeclaration::BorderBottomStyle(solid.clone()));
          properties.push(PropertyDeclaration::BorderLeftStyle(solid.clone()));

          let px_30 = BorderWidth::from_length(Length::from_px(30f32));
          let px_10 = BorderWidth::from_length(Length::from_px(10f32));

          properties.push(PropertyDeclaration::BorderTopWidth(Box::new(px_30.clone())));
          properties.push(PropertyDeclaration::BorderRightWidth(Box::new(px_30.clone())));
          properties.push(PropertyDeclaration::BorderBottomWidth(Box::new(px_30.clone())));
          properties.push(PropertyDeclaration::BorderLeftWidth(Box::new(px_10.clone())));

          let blue = CSSColor {
              parsed: ComputedColor::RGBA(RGBA::new(0, 0, 255, 255)),
              authored: None
          };

          properties.push(PropertyDeclaration::BorderTopColor(blue.clone()));
          properties.push(PropertyDeclaration::BorderRightColor(blue.clone()));
          properties.push(PropertyDeclaration::BorderBottomColor(blue.clone()));
          properties.push(PropertyDeclaration::BorderLeftColor(blue.clone()));

          let serialization = shorthand_properties_to_string(properties);
          assert_eq!(serialization,
          "border-style: solid; border-width: 30px 30px 30px 10px; border-color: rgb(0, 0, 255);");
        }

        #[test]
        fn same_longhands_should_serialize_correctly() {
          let mut properties = Vec::new();

          let solid = BorderStyle::solid;

          properties.push(PropertyDeclaration::BorderTopStyle(solid.clone()));
          properties.push(PropertyDeclaration::BorderRightStyle(solid.clone()));
          properties.push(PropertyDeclaration::BorderBottomStyle(solid.clone()));
          properties.push(PropertyDeclaration::BorderLeftStyle(solid.clone()));

          let px_30 = BorderWidth::from_length(Length::from_px(30f32));

          properties.push(PropertyDeclaration::BorderTopWidth(Box::new(px_30.clone())));
          properties.push(PropertyDeclaration::BorderRightWidth(Box::new(px_30.clone())));
          properties.push(PropertyDeclaration::BorderBottomWidth(Box::new(px_30.clone())));
          properties.push(PropertyDeclaration::BorderLeftWidth(Box::new(px_30.clone())));

          let blue = CSSColor {
              parsed: ComputedColor::RGBA(RGBA::new(0, 0, 255, 255)),
              authored: None
          };

          properties.push(PropertyDeclaration::BorderTopColor(blue.clone()));
          properties.push(PropertyDeclaration::BorderRightColor(blue.clone()));
          properties.push(PropertyDeclaration::BorderBottomColor(blue.clone()));
          properties.push(PropertyDeclaration::BorderLeftColor(blue.clone()));

          let serialization = shorthand_properties_to_string(properties);
          assert_eq!(serialization, "border: 30px solid rgb(0, 0, 255);");
        }

        #[test]
        fn padding_should_serialize_correctly() {
            let mut properties = Vec::new();

            let px_10 = LengthOrPercentage::Length(NoCalcLength::from_px(10f32));
            let px_15 = LengthOrPercentage::Length(NoCalcLength::from_px(15f32));
            properties.push(PropertyDeclaration::PaddingTop(px_10.clone()));
            properties.push(PropertyDeclaration::PaddingRight(px_15.clone()));
            properties.push(PropertyDeclaration::PaddingBottom(px_10));
            properties.push(PropertyDeclaration::PaddingLeft(px_15));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "padding: 10px 15px;");
        }

        #[test]
        fn border_width_should_serialize_correctly() {
            let mut properties = Vec::new();

            let top_px = BorderWidth::from_length(Length::from_px(10f32));
            let bottom_px = BorderWidth::from_length(Length::from_px(10f32));

            let right_px = BorderWidth::from_length(Length::from_px(15f32));
            let left_px = BorderWidth::from_length(Length::from_px(15f32));

            properties.push(PropertyDeclaration::BorderTopWidth(Box::new(top_px)));
            properties.push(PropertyDeclaration::BorderRightWidth(Box::new(right_px)));
            properties.push(PropertyDeclaration::BorderBottomWidth(Box::new(bottom_px)));
            properties.push(PropertyDeclaration::BorderLeftWidth(Box::new(left_px)));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-width: 10px 15px;");
        }

        #[test]
        fn border_width_with_keywords_should_serialize_correctly() {
            let mut properties = Vec::new();

            let top_px = BorderWidth::Thin;
            let right_px = BorderWidth::Medium;
            let bottom_px = BorderWidth::Thick;
            let left_px = BorderWidth::from_length(Length::from_px(15f32));

            properties.push(PropertyDeclaration::BorderTopWidth(Box::new(top_px)));
            properties.push(PropertyDeclaration::BorderRightWidth(Box::new(right_px)));
            properties.push(PropertyDeclaration::BorderBottomWidth(Box::new(bottom_px)));
            properties.push(PropertyDeclaration::BorderLeftWidth(Box::new(left_px)));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-width: thin medium thick 15px;");
        }

        #[test]
        fn border_color_should_serialize_correctly() {
            let mut properties = Vec::new();

            let red = CSSColor {
                parsed: ComputedColor::RGBA(RGBA::new(255, 0, 0, 255)),
                authored: None
            };

            let blue = CSSColor {
                parsed: ComputedColor::RGBA(RGBA::new(0, 0, 255, 255)),
                authored: None
            };

            properties.push(PropertyDeclaration::BorderTopColor(blue.clone()));
            properties.push(PropertyDeclaration::BorderRightColor(red.clone()));
            properties.push(PropertyDeclaration::BorderBottomColor(blue));
            properties.push(PropertyDeclaration::BorderLeftColor(red));

            let serialization = shorthand_properties_to_string(properties);

            // TODO: Make the rgb test show border-color as blue red instead of below tuples
            assert_eq!(serialization, "border-color: rgb(0, 0, 255) rgb(255, 0, 0);");
        }

        #[test]
        fn border_style_should_serialize_correctly() {
            let mut properties = Vec::new();

            let solid = BorderStyle::solid;
            let dotted = BorderStyle::dotted;
            properties.push(PropertyDeclaration::BorderTopStyle(solid.clone()));
            properties.push(PropertyDeclaration::BorderRightStyle(dotted.clone()));
            properties.push(PropertyDeclaration::BorderBottomStyle(solid));
            properties.push(PropertyDeclaration::BorderLeftStyle(dotted));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-style: solid dotted;");
        }
    }


    mod border_shorthands {
        use super::*;

        // we can use border-top as a base to test out the different combinations
        // but afterwards, we only need to to one test per "directional border shorthand"

        #[test]
        fn directional_border_should_show_all_properties_when_values_are_set() {
            let mut properties = Vec::new();

            let width = BorderWidth::from_length(Length::from_px(4f32));
            let style = BorderStyle::solid;
            let color = CSSColor {
                parsed: ComputedColor::RGBA(RGBA::new(255, 0, 0, 255)),
                authored: None
            };

            properties.push(PropertyDeclaration::BorderTopWidth(Box::new(width)));
            properties.push(PropertyDeclaration::BorderTopStyle(style));
            properties.push(PropertyDeclaration::BorderTopColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-top: 4px solid rgb(255, 0, 0);");
        }

        fn get_border_property_values() -> (BorderWidth, BorderStyle, CSSColor) {
            (BorderWidth::from_length(Length::from_px(4f32)),
             BorderStyle::solid,
             CSSColor::currentcolor())
        }

        #[test]
        fn border_top_should_serialize_correctly() {
            let mut properties = Vec::new();
            let (width, style, color) = get_border_property_values();
            properties.push(PropertyDeclaration::BorderTopWidth(Box::new(width)));
            properties.push(PropertyDeclaration::BorderTopStyle(style));
            properties.push(PropertyDeclaration::BorderTopColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-top: 4px solid;");
        }

        #[test]
        fn border_right_should_serialize_correctly() {
            let mut properties = Vec::new();
            let (width, style, color) = get_border_property_values();
            properties.push(PropertyDeclaration::BorderRightWidth(Box::new(width)));
            properties.push(PropertyDeclaration::BorderRightStyle(style));
            properties.push(PropertyDeclaration::BorderRightColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-right: 4px solid;");
        }

        #[test]
        fn border_bottom_should_serialize_correctly() {
            let mut properties = Vec::new();
            let (width, style, color) = get_border_property_values();
            properties.push(PropertyDeclaration::BorderBottomWidth(Box::new(width)));
            properties.push(PropertyDeclaration::BorderBottomStyle(style));
            properties.push(PropertyDeclaration::BorderBottomColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-bottom: 4px solid;");
        }

        #[test]
        fn border_left_should_serialize_correctly() {
            let mut properties = Vec::new();
            let (width, style, color) = get_border_property_values();
            properties.push(PropertyDeclaration::BorderLeftWidth(Box::new(width)));
            properties.push(PropertyDeclaration::BorderLeftStyle(style));
            properties.push(PropertyDeclaration::BorderLeftColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-left: 4px solid;");
        }

        #[test]
        fn border_should_serialize_correctly() {
            let mut properties = Vec::new();
            let (width, style, color) = get_border_property_values();

            properties.push(PropertyDeclaration::BorderTopWidth(Box::new(width.clone())));
            properties.push(PropertyDeclaration::BorderTopStyle(style.clone()));
            properties.push(PropertyDeclaration::BorderTopColor(color.clone()));

            properties.push(PropertyDeclaration::BorderRightWidth(Box::new(width.clone())));
            properties.push(PropertyDeclaration::BorderRightStyle(style.clone()));
            properties.push(PropertyDeclaration::BorderRightColor(color.clone()));

            properties.push(PropertyDeclaration::BorderBottomWidth(Box::new(width.clone())));
            properties.push(PropertyDeclaration::BorderBottomStyle(style.clone()));
            properties.push(PropertyDeclaration::BorderBottomColor(color.clone()));

            properties.push(PropertyDeclaration::BorderLeftWidth(Box::new(width.clone())));
            properties.push(PropertyDeclaration::BorderLeftStyle(style.clone()));
            properties.push(PropertyDeclaration::BorderLeftColor(color.clone()));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border: 4px solid;");
        }
    }

    mod list_style {
        use style::properties::longhands::list_style_position::SpecifiedValue as ListStylePosition;
        use style::properties::longhands::list_style_type::SpecifiedValue as ListStyleType;
        use style::values::Either;
        use super::*;

        #[test]
        fn list_style_should_show_all_properties_when_values_are_set() {
            let mut properties = Vec::new();

            let position = ListStylePosition::inside;
            let image = Either::First(
                SpecifiedUrl::new_for_testing("http://servo/test.png"));
            let style_type = ListStyleType::disc;

            properties.push(PropertyDeclaration::ListStylePosition(position));
            properties.push(PropertyDeclaration::ListStyleImage(image));
            properties.push(PropertyDeclaration::ListStyleType(style_type));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "list-style: inside url(\"http://servo/test.png\") disc;");
        }
    }

    mod outline {
        use style::properties::longhands::outline_width::SpecifiedValue as WidthContainer;
        use style::values::Either;
        use super::*;

        #[test]
        fn outline_should_show_all_properties_when_set() {
            let mut properties = Vec::new();

            let width = WidthContainer(Length::from_px(4f32));
            let style = Either::Second(BorderStyle::solid);
            let color = CSSColor {
                parsed: ComputedColor::RGBA(RGBA::new(255, 0, 0, 255)),
                authored: None
            };

            properties.push(PropertyDeclaration::OutlineWidth(width));
            properties.push(PropertyDeclaration::OutlineStyle(style));
            properties.push(PropertyDeclaration::OutlineColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "outline: 4px solid rgb(255, 0, 0);");
        }

        #[test]
        fn outline_should_serialize_correctly_when_style_is_auto() {
            let mut properties = Vec::new();

            let width = WidthContainer(Length::from_px(4f32));
            let style = Either::First(Auto);
            let color = CSSColor {
                parsed: ComputedColor::RGBA(RGBA::new(255, 0, 0, 255)),
                authored: None
            };
            properties.push(PropertyDeclaration::OutlineWidth(width));
            properties.push(PropertyDeclaration::OutlineStyle(style));
            properties.push(PropertyDeclaration::OutlineColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "outline: 4px auto rgb(255, 0, 0);");
        }
    }

    #[test]
    fn columns_should_serialize_correctly() {
        use style::values::{Auto, Either};

        let mut properties = Vec::new();

        let width = Either::Second(Auto);
        let count = Either::Second(Auto);

        properties.push(PropertyDeclaration::ColumnWidth(Box::new(width)));
        properties.push(PropertyDeclaration::ColumnCount(count));

        let serialization = shorthand_properties_to_string(properties);
        assert_eq!(serialization, "columns: auto auto;");
    }

    #[test]
    fn flex_should_serialize_all_available_properties() {
        use style::values::specified::Number as NumberContainer;
        use style::values::specified::Percentage as PercentageContainer;

        let mut properties = Vec::new();

        let grow = NumberContainer(2f32);
        let shrink = NumberContainer(3f32);
        let basis =
            LengthOrPercentageOrAutoOrContent::Percentage(PercentageContainer(0.5f32));

        properties.push(PropertyDeclaration::FlexGrow(grow));
        properties.push(PropertyDeclaration::FlexShrink(shrink));
        properties.push(PropertyDeclaration::FlexBasis(basis));

        let serialization = shorthand_properties_to_string(properties);
        assert_eq!(serialization, "flex: 2 3 50%;");
    }

    #[test]
    fn flex_flow_should_serialize_all_available_properties() {
        use style::properties::longhands::flex_direction::SpecifiedValue as FlexDirection;
        use style::properties::longhands::flex_wrap::SpecifiedValue as FlexWrap;

        let mut properties = Vec::new();

        let direction = FlexDirection::row;
        let wrap = FlexWrap::wrap;

        properties.push(PropertyDeclaration::FlexDirection(direction));
        properties.push(PropertyDeclaration::FlexWrap(wrap));

        let serialization = shorthand_properties_to_string(properties);
        assert_eq!(serialization, "flex-flow: row wrap;");
    }

    mod font {
        use super::*;

        #[test]
        fn font_should_serialize_to_empty_if_there_are_nondefault_subproperties() {
            // Test with non-default font-kerning value
            let block_text = "font-style: italic; \
                              font-variant: normal; \
                              font-weight: bolder; \
                              font-stretch: expanded; \
                              font-size: 4px; \
                              line-height: 3; \
                              font-family: serif; \
                              font-size-adjust: none; \
                              font-variant-caps: normal; \
                              font-variant-position: normal; \
                              font-language-override: normal; \
                              font-kerning: none";

            let block = parse_declaration_block(block_text);

            let mut s = String::new();
            let id = PropertyId::parse("font".into()).unwrap();
            let x = block.property_value_to_css(&id, &mut s);

            assert_eq!(x.is_ok(), true);
            assert_eq!(s, "");
        }

        #[test]
        fn font_should_serialize_all_available_properties() {
            let block_text = "font-style: italic; \
                              font-variant: normal; \
                              font-weight: bolder; \
                              font-stretch: expanded; \
                              font-size: 4px; \
                              line-height: 3; \
                              font-family: serif; \
                              font-size-adjust: none; \
                              font-kerning: auto; \
                              font-variant-caps: normal; \
                              font-variant-position: normal; \
                              font-language-override: normal;";

            let block = parse_declaration_block(block_text);

            let serialization = block.to_css_string();

            assert_eq!(serialization, "font: italic normal bolder expanded 4px/3 serif;");
        }

    }

    mod background {
        use super::*;

        #[test]
        fn background_should_serialize_all_available_properties_when_specified() {
            let block_text = "\
                background-color: rgb(255, 0, 0); \
                background-image: url(\"http://servo/test.png\"); \
                background-repeat: repeat-x; \
                background-attachment: scroll; \
                background-size: 70px 50px; \
                background-position-x: 7px; \
                background-position-y: 4px; \
                background-origin: border-box; \
                background-clip: padding-box;";
            let block = parse_declaration_block(block_text);

            let serialization = block.to_css_string();

            assert_eq!(
                serialization,
                "background: rgb(255, 0, 0) url(\"http://servo/test.png\") repeat-x \
                scroll 7px 4px / 70px 50px border-box padding-box;"
            );
        }

        #[test]
        fn background_should_combine_origin_and_clip_properties_when_equal() {
            let block_text = "\
                background-color: rgb(255, 0, 0); \
                background-image: url(\"http://servo/test.png\"); \
                background-repeat: repeat-x; \
                background-attachment: scroll; \
                background-size: 70px 50px; \
                background-position-x: 7px; \
                background-position-y: 4px; \
                background-origin: padding-box; \
                background-clip: padding-box;";
            let block = parse_declaration_block(block_text);

            let serialization = block.to_css_string();

            assert_eq!(
                serialization,
                "background: rgb(255, 0, 0) url(\"http://servo/test.png\") repeat-x \
                scroll 7px 4px / 70px 50px padding-box;"
            );
        }

        #[test]
        fn serialize_multiple_backgrounds() {
            let block_text = "\
                background-color: rgb(0, 0, 255); \
                background-image: url(\"http://servo/test.png\"), none; \
                background-repeat: repeat-x, repeat-y; \
                background-attachment: scroll, scroll; \
                background-size: 70px 50px, 20px 30px; \
                background-position-x: 7px, 70px; \
                background-position-y: 4px, 40px; \
                background-origin: border-box, padding-box; \
                background-clip: padding-box, padding-box;";
            let block = parse_declaration_block(block_text);

            let serialization = block.to_css_string();

            assert_eq!(
                serialization, "background: \
                url(\"http://servo/test.png\") repeat-x scroll 7px 4px / 70px 50px border-box padding-box, \
                rgb(0, 0, 255) none repeat-y scroll 70px 40px / 20px 30px padding-box;"
            );
        }

        #[test]
        fn serialize_multiple_backgrounds_unequal_property_lists() {
            // When the lengths of property values are different, the shorthand serialization
            // should not be used. Previously the implementation cycled values if the lists were
            // uneven. This is incorrect, in that we should serialize to a shorthand only when the
            // lists have the same length (this affects background, transition and animation).
            // https://github.com/servo/servo/issues/15398 )
            // With background, the color is one exception as it should only appear once for
            // multiple backgrounds.
            // Below, background-position and background-origin only have one value.
            let block_text = "\
                background-color: rgb(0, 0, 255); \
                background-image: url(\"http://servo/test.png\"), none; \
                background-repeat: repeat-x, repeat-y; \
                background-attachment: scroll, scroll; \
                background-size: 70px 50px, 20px 30px; \
                background-position: 7px 4px; \
                background-origin: border-box; \
                background-clip: padding-box, padding-box;";
            let block = parse_declaration_block(block_text);

            let serialization = block.to_css_string();

            assert_eq!(serialization, block_text);
        }
    }

    mod mask {
        use style::properties::longhands::mask_clip as clip;
        use style::properties::longhands::mask_composite as composite;
        use style::properties::longhands::mask_image as image;
        use style::properties::longhands::mask_mode as mode;
        use style::properties::longhands::mask_origin as origin;
        use style::properties::longhands::mask_position_x as position_x;
        use style::properties::longhands::mask_position_y as position_y;
        use style::properties::longhands::mask_repeat as repeat;
        use style::properties::longhands::mask_size as size;
        use style::values::specified::Image;
        use style::values::specified::position::{HorizontalPosition, VerticalPosition};
        use super::*;

        macro_rules! single_vec_value_typedef {
            ($name:ident, $path:expr) => {
                $name::SpecifiedValue(
                    vec![$path]
                )
            };
        }
        macro_rules! single_vec_keyword_value {
            ($name:ident, $kw:ident) => {
                $name::SpecifiedValue(
                    vec![$name::single_value::SpecifiedValue::$kw]
                )
            };
        }
        macro_rules! single_vec_variant_value {
            ($name:ident, $variant:expr) => {
                $name::SpecifiedValue(
                        vec![$variant]
                )
            };
        }

        #[test]
        fn mask_should_serialize_all_available_properties_when_specified() {
            let mut properties = Vec::new();

            let image = single_vec_value_typedef!(image,
                image::single_value::SpecifiedValue::Image(
                    Image::Url(SpecifiedUrl::new_for_testing("http://servo/test.png"))));

            let mode = single_vec_keyword_value!(mode, luminance);

            let position_x = single_vec_value_typedef!(position_x,
                HorizontalPosition {
                    keyword: None,
                    position: Some(LengthOrPercentage::Length(NoCalcLength::from_px(7f32))),
                }
            );
            let position_y = single_vec_value_typedef!(position_y,
                VerticalPosition {
                    keyword: None,
                    position: Some(LengthOrPercentage::Length(NoCalcLength::from_px(4f32))),
                }
            );

            let size = single_vec_variant_value!(size,
                size::single_value::SpecifiedValue::Explicit(
                    size::single_value::ExplicitSize {
                        width: LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(70f32)),
                        height: LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(50f32))
                    }
                )
            );

            let repeat = single_vec_keyword_value!(repeat, repeat_x);
            let origin = single_vec_keyword_value!(origin, padding_box);
            let clip = single_vec_keyword_value!(clip, border_box);
            let composite = single_vec_keyword_value!(composite, subtract);

            properties.push(PropertyDeclaration::MaskImage(image));
            properties.push(PropertyDeclaration::MaskMode(mode));
            properties.push(PropertyDeclaration::MaskPositionX(position_x));
            properties.push(PropertyDeclaration::MaskPositionY(position_y));
            properties.push(PropertyDeclaration::MaskSize(size));
            properties.push(PropertyDeclaration::MaskRepeat(repeat));
            properties.push(PropertyDeclaration::MaskOrigin(origin));
            properties.push(PropertyDeclaration::MaskClip(clip));
            properties.push(PropertyDeclaration::MaskComposite(composite));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(
                serialization,
                "mask: url(\"http://servo/test.png\") luminance 7px 4px / 70px 50px \
                repeat-x padding-box border-box subtract;"
            );
        }

        #[test]
        fn mask_should_combine_origin_and_clip_properties_when_equal() {
            let mut properties = Vec::new();

            let image = single_vec_value_typedef!(image,
                image::single_value::SpecifiedValue::Image(
                    Image::Url(SpecifiedUrl::new_for_testing("http://servo/test.png"))));

            let mode = single_vec_keyword_value!(mode, luminance);

            let position_x = single_vec_value_typedef!(position_x,
                HorizontalPosition {
                    keyword: None,
                    position: Some(LengthOrPercentage::Length(NoCalcLength::from_px(7f32))),
                }
            );

            let position_y = single_vec_value_typedef!(position_y,
                VerticalPosition {
                    keyword: None,
                    position: Some(LengthOrPercentage::Length(NoCalcLength::from_px(4f32))),
                }
            );

            let size = single_vec_variant_value!(size,
                size::single_value::SpecifiedValue::Explicit(
                    size::single_value::ExplicitSize {
                        width: LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(70f32)),
                        height: LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(50f32))
                    }
                )
            );

            let repeat = single_vec_keyword_value!(repeat, repeat_x);
            let origin = single_vec_keyword_value!(origin, padding_box);
            let clip = single_vec_keyword_value!(clip, padding_box);
            let composite = single_vec_keyword_value!(composite, subtract);

            properties.push(PropertyDeclaration::MaskImage(image));
            properties.push(PropertyDeclaration::MaskMode(mode));
            properties.push(PropertyDeclaration::MaskPositionX(position_x));
            properties.push(PropertyDeclaration::MaskPositionY(position_y));
            properties.push(PropertyDeclaration::MaskSize(size));
            properties.push(PropertyDeclaration::MaskRepeat(repeat));
            properties.push(PropertyDeclaration::MaskOrigin(origin));
            properties.push(PropertyDeclaration::MaskClip(clip));
            properties.push(PropertyDeclaration::MaskComposite(composite));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(
                serialization,
                "mask: url(\"http://servo/test.png\") luminance 7px 4px / 70px 50px \
                repeat-x padding-box subtract;"
            );
        }
    }

    mod scroll_snap_type {
        pub use super::*;
        use style::properties::longhands::scroll_snap_type_x::SpecifiedValue as ScrollSnapTypeXValue;

        #[test]
        fn should_serialize_to_empty_string_if_sub_types_not_equal() {
            let declarations = vec![
                (PropertyDeclaration::ScrollSnapTypeX(ScrollSnapTypeXValue::mandatory),
                Importance::Normal),
                (PropertyDeclaration::ScrollSnapTypeY(ScrollSnapTypeXValue::none),
                Importance::Normal)
            ];

            let block = block_from(declarations);

            let mut s = String::new();

            let id = PropertyId::parse("scroll-snap-type".into()).unwrap();
            let x = block.single_value_to_css(&id, &mut s);

            assert_eq!(x.is_ok(), true);
            assert_eq!(s, "");
        }

        #[test]
        fn should_serialize_to_single_value_if_sub_types_are_equal() {
            let declarations = vec![
                (PropertyDeclaration::ScrollSnapTypeX(ScrollSnapTypeXValue::mandatory),
                Importance::Normal),
                (PropertyDeclaration::ScrollSnapTypeY(ScrollSnapTypeXValue::mandatory),
                Importance::Normal)
            ];

            let block = block_from(declarations);

            let mut s = String::new();

            let id = PropertyId::parse("scroll-snap-type".into()).unwrap();
            let x = block.single_value_to_css(&id, &mut s);

            assert_eq!(x.is_ok(), true);
            assert_eq!(s, "mandatory");
        }
    }

    mod transform {
        pub use super::*;

        #[test]
        fn should_serialize_none_correctly() {
            use cssparser::Parser;
            use media_queries::CSSErrorReporterTest;
            use style::parser::ParserContext;
            use style::properties::longhands::transform;
            use style::stylesheets::Origin;

            let mut s = String::new();
            let url = ::servo_url::ServoUrl::parse("http://localhost").unwrap();
            let reporter = CSSErrorReporterTest;
            let context = ParserContext::new(Origin::Author, &url, &reporter);

            let parsed = transform::parse(&context, &mut Parser::new("none")).unwrap();
            let try_serialize = parsed.to_css(&mut s);

            assert_eq!(try_serialize.is_ok(), true);
            assert_eq!(s, "none");
        }
    }

    mod quotes {
        pub use super::*;

        #[test]
        fn should_serialize_none_correctly() {
            use cssparser::Parser;
            use media_queries::CSSErrorReporterTest;
            use style::parser::ParserContext;
            use style::properties::longhands::quotes;
            use style::stylesheets::Origin;

            let mut s = String::new();
            let url = ::servo_url::ServoUrl::parse("http://localhost").unwrap();
            let reporter = CSSErrorReporterTest;
            let context = ParserContext::new(Origin::Author, &url, &reporter);

            let parsed = quotes::parse(&context, &mut Parser::new("none")).unwrap();
            let try_serialize = parsed.to_css(&mut s);

            assert_eq!(try_serialize.is_ok(), true);
            assert_eq!(s, "none");
        }
    }

    mod animation {
        pub use super::*;

        #[test]
        fn serialize_single_animation() {
            let block = parse_declaration_block("\
                animation-name: bounce;\
                animation-duration: 1s;\
                animation-timing-function: ease-in;\
                animation-delay: 0s;\
                animation-direction: normal;\
                animation-fill-mode: forwards;\
                animation-iteration-count: infinite;\
                animation-play-state: paused;");

            let serialization = block.to_css_string();

            assert_eq!(serialization, "animation: 1s ease-in 0s infinite normal forwards paused bounce;")
        }

        #[test]
        fn serialize_multiple_animations() {
            let block = parse_declaration_block("\
                animation-name: bounce, roll;\
                animation-duration: 1s, 0.2s;\
                animation-timing-function: ease-in, linear;\
                animation-delay: 0s, 1s;\
                animation-direction: normal, reverse;\
                animation-fill-mode: forwards, backwards;\
                animation-iteration-count: infinite, 2;\
                animation-play-state: paused, running;");

            let serialization = block.to_css_string();

            assert_eq!(serialization,
                       "animation: 1s ease-in 0s infinite normal forwards paused bounce, \
                                   0.2s linear 1s 2 reverse backwards running roll;");
        }

        #[test]
        fn serialize_multiple_animations_unequal_property_lists() {
            // When the lengths of property values are different, the shorthand serialization
            // should not be used. Previously the implementation cycled values if the lists were
            // uneven. This is incorrect, in that we should serialize to a shorthand only when the
            // lists have the same length (this affects background, transition and animation).
            // https://github.com/servo/servo/issues/15398 )
            let block_text = "\
                animation-name: bounce, roll, flip, jump; \
                animation-duration: 1s, 0.2s; \
                animation-timing-function: ease-in, linear; \
                animation-delay: 0s, 1s, 0.5s; \
                animation-direction: normal; \
                animation-fill-mode: forwards, backwards; \
                animation-iteration-count: infinite, 2; \
                animation-play-state: paused, running;";
            let block = parse_declaration_block(block_text);

            let serialization = block.to_css_string();

            assert_eq!(serialization, block_text);
        }

        #[test]
        fn serialize_multiple_without_all_properties_returns_longhand() {
            // timing function and direction are missing, so no shorthand is returned.
            let block_text = "animation-name: bounce, roll; \
                              animation-duration: 1s, 0.2s; \
                              animation-delay: 0s, 1s; \
                              animation-fill-mode: forwards, backwards; \
                              animation-iteration-count: infinite, 2; \
                              animation-play-state: paused, running;";
            let block = parse_declaration_block(block_text);

            let serialization = block.to_css_string();

            assert_eq!(serialization, block_text);
        }
    }

    mod transition {
        pub use super::*;

        #[test]
        fn transition_should_serialize_all_available_properties() {
            let block_text = "transition-property: margin-left; \
                              transition-duration: 3s; \
                              transition-delay: 4s; \
                              transition-timing-function: cubic-bezier(0.2, 5, 0.5, 2);";
            let block = parse_declaration_block(block_text);

            let serialization = block.to_css_string();

            assert_eq!(serialization, "transition: margin-left 3s cubic-bezier(0.2, 5, 0.5, 2) 4s;");
        }

        #[test]
        fn serialize_multiple_transitions() {
            let block_text = "transition-property: margin-left, width; \
                              transition-duration: 3s, 2s; \
                              transition-delay: 4s, 5s; \
                              transition-timing-function: cubic-bezier(0.2, 5, 0.5, 2), ease;";
            let block = parse_declaration_block(block_text);

            let serialization = block.to_css_string();

            assert_eq!(serialization, "transition: \
                margin-left 3s cubic-bezier(0.2, 5, 0.5, 2) 4s, \
                width 2s ease 5s;");
        }

        #[test]
        fn serialize_multiple_transitions_unequal_property_lists() {
            // When the lengths of property values are different, the shorthand serialization
            // should not be used. Previously the implementation cycled values if the lists were
            // uneven. This is incorrect, in that we should serialize to a shorthand only when the
            // lists have the same length (this affects background, transition and animation).
            // https://github.com/servo/servo/issues/15398 )
            // The duration below has 1 extra value.
            let block_text = "transition-property: margin-left, width; \
                              transition-duration: 3s, 2s, 4s; \
                              transition-delay: 4s, 5s; \
                              transition-timing-function: cubic-bezier(0.2, 5, 0.5, 2), ease;";
            let block = parse_declaration_block(block_text);

            let serialization = block.to_css_string();

            assert_eq!(serialization, block_text);
        }
    }

    mod keywords {
        pub use super::*;
        #[test]
        fn css_wide_keywords_should_be_parsed() {
            let block_text = "--a:inherit;";
            let block = parse_declaration_block(block_text);

            let serialization = block.to_css_string();
            assert_eq!(serialization, "--a: inherit;");
        }

        #[test]
        fn non_keyword_custom_property_should_be_unparsed() {
            let block_text = "--main-color: #06c;";
            let block = parse_declaration_block(block_text);

            let serialization = block.to_css_string();
            assert_eq!(serialization, block_text);
        }
    }

    mod effects {
        pub use super::*;
        pub use style::properties::longhands::box_shadow::SpecifiedValue as BoxShadow;
        pub use style::values::specified::Shadow;

        #[test]
        fn box_shadow_should_serialize_correctly() {
            let mut properties = Vec::new();
            let shadow_val = Shadow { offset_x: Length::from_px(1f32), offset_y: Length::from_px(2f32),
            blur_radius: Length::from_px(3f32), spread_radius: Length::from_px(4f32), color: None, inset: false };
            let shadow_decl = BoxShadow(vec![shadow_val]);
            properties.push(PropertyDeclaration:: BoxShadow(shadow_decl));
            let shadow_css = "box-shadow: 1px 2px 3px 4px;";
            let shadow  =  parse_declaration_block(shadow_css);

            assert_eq!(shadow.to_css_string(), shadow_css);
        }
    }

    mod counter_increment {
        pub use super::*;
        pub use style::properties::longhands::counter_increment::SpecifiedValue as CounterIncrement;

        #[test]
        fn counter_increment_with_properties_should_serialize_correctly() {
            let mut properties = Vec::new();

            properties.push(("counter1".to_owned(), 1));
            properties.push(("counter2".to_owned(), -4));

            let counter_increment = CounterIncrement(properties);
            let counter_increment_css = "counter1 1 counter2 -4";

            assert_eq!(counter_increment.to_css_string(), counter_increment_css);
        }

        #[test]
        fn counter_increment_without_properties_should_serialize_correctly() {
            let counter_increment = CounterIncrement(Vec::new());
            let counter_increment_css = "none";

            assert_eq!(counter_increment.to_css_string(), counter_increment_css);
        }
    }
}
