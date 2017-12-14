/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use properties::{parse, parse_input};
use style::computed_values::display::T as Display;
use style::properties::{PropertyDeclaration, Importance};
use style::properties::parse_property_declaration_list;
use style::values::{CustomIdent, RGBA, Auto};
use style::values::generics::flex::FlexBasis;
use style::values::specified::{BorderStyle, BorderSideWidth, Color};
use style::values::specified::{Length, LengthOrPercentage, LengthOrPercentageOrAuto};
use style::values::specified::NoCalcLength;
use style::values::specified::url::SpecifiedUrl;
use style_traits::ToCss;
use stylesheets::block_from;

#[test]
fn property_declaration_block_should_serialize_correctly() {
    use style::properties::longhands::overflow_x::SpecifiedValue as OverflowValue;

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

        (PropertyDeclaration::Display(Display::InlineBlock),
         Importance::Normal),

        (PropertyDeclaration::OverflowX(
            OverflowValue::Auto),
         Importance::Normal),

        (PropertyDeclaration::OverflowY(
            OverflowValue::Auto),
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
        use style::properties::longhands::overflow_x::SpecifiedValue as OverflowValue;

        #[test]
        fn equal_overflow_properties_should_serialize_to_single_value() {
            let mut properties = Vec::new();

            let overflow = OverflowValue::Auto;
            properties.push(PropertyDeclaration::OverflowX(overflow));
            properties.push(PropertyDeclaration::OverflowY(overflow));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "overflow: auto;");
        }

        #[test]
        fn different_overflow_properties_should_serialize_to_two_values() {
            let mut properties = Vec::new();

            let overflow_x = OverflowValue::Scroll;
            properties.push(PropertyDeclaration::OverflowX(overflow_x));

            let overflow_y = OverflowValue::Auto;
            properties.push(PropertyDeclaration::OverflowY(overflow_y));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "overflow-x: scroll; overflow-y: auto;");
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

          let solid = BorderStyle::Solid;

          properties.push(PropertyDeclaration::BorderTopStyle(solid.clone()));
          properties.push(PropertyDeclaration::BorderRightStyle(solid.clone()));
          properties.push(PropertyDeclaration::BorderBottomStyle(solid.clone()));
          properties.push(PropertyDeclaration::BorderLeftStyle(solid.clone()));

          let px_30 = BorderSideWidth::Length(Length::from_px(30f32));
          let px_10 = BorderSideWidth::Length(Length::from_px(10f32));

          properties.push(PropertyDeclaration::BorderTopWidth(px_30.clone()));
          properties.push(PropertyDeclaration::BorderRightWidth(px_30.clone()));
          properties.push(PropertyDeclaration::BorderBottomWidth(px_30.clone()));
          properties.push(PropertyDeclaration::BorderLeftWidth(px_10.clone()));

          let blue = Color::rgba(RGBA::new(0, 0, 255, 255));

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

          let solid = BorderStyle::Solid;

          properties.push(PropertyDeclaration::BorderTopStyle(solid.clone()));
          properties.push(PropertyDeclaration::BorderRightStyle(solid.clone()));
          properties.push(PropertyDeclaration::BorderBottomStyle(solid.clone()));
          properties.push(PropertyDeclaration::BorderLeftStyle(solid.clone()));

          let px_30 = BorderSideWidth::Length(Length::from_px(30f32));

          properties.push(PropertyDeclaration::BorderTopWidth(px_30.clone()));
          properties.push(PropertyDeclaration::BorderRightWidth(px_30.clone()));
          properties.push(PropertyDeclaration::BorderBottomWidth(px_30.clone()));
          properties.push(PropertyDeclaration::BorderLeftWidth(px_30.clone()));

          let blue = Color::rgba(RGBA::new(0, 0, 255, 255));

          properties.push(PropertyDeclaration::BorderTopColor(blue.clone()));
          properties.push(PropertyDeclaration::BorderRightColor(blue.clone()));
          properties.push(PropertyDeclaration::BorderBottomColor(blue.clone()));
          properties.push(PropertyDeclaration::BorderLeftColor(blue.clone()));

          let serialization = shorthand_properties_to_string(properties);
          assert_eq!(serialization, "border-style: solid; border-width: 30px; border-color: rgb(0, 0, 255);");
        }

        #[test]
        fn padding_should_serialize_correctly() {
            use style::values::specified::NonNegativeLengthOrPercentage;

            let mut properties = Vec::new();

            let px_10: NonNegativeLengthOrPercentage = NoCalcLength::from_px(10f32).into();
            let px_15: NonNegativeLengthOrPercentage = NoCalcLength::from_px(15f32).into();
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

            let top_px = BorderSideWidth::Length(Length::from_px(10f32));
            let bottom_px = BorderSideWidth::Length(Length::from_px(10f32));

            let right_px = BorderSideWidth::Length(Length::from_px(15f32));
            let left_px = BorderSideWidth::Length(Length::from_px(15f32));

            properties.push(PropertyDeclaration::BorderTopWidth(top_px));
            properties.push(PropertyDeclaration::BorderRightWidth(right_px));
            properties.push(PropertyDeclaration::BorderBottomWidth(bottom_px));
            properties.push(PropertyDeclaration::BorderLeftWidth(left_px));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-width: 10px 15px;");
        }

        #[test]
        fn border_width_with_keywords_should_serialize_correctly() {
            let mut properties = Vec::new();

            let top_px = BorderSideWidth::Thin;
            let right_px = BorderSideWidth::Medium;
            let bottom_px = BorderSideWidth::Thick;
            let left_px = BorderSideWidth::Length(Length::from_px(15f32));

            properties.push(PropertyDeclaration::BorderTopWidth(top_px));
            properties.push(PropertyDeclaration::BorderRightWidth(right_px));
            properties.push(PropertyDeclaration::BorderBottomWidth(bottom_px));
            properties.push(PropertyDeclaration::BorderLeftWidth(left_px));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-width: thin medium thick 15px;");
        }

        #[test]
        fn border_color_should_serialize_correctly() {
            let mut properties = Vec::new();

            let red = Color::rgba(RGBA::new(255, 0, 0, 255));
            let blue = Color::rgba(RGBA::new(0, 0, 255, 255));

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

            let solid = BorderStyle::Solid;
            let dotted = BorderStyle::Dotted;
            properties.push(PropertyDeclaration::BorderTopStyle(solid.clone()));
            properties.push(PropertyDeclaration::BorderRightStyle(dotted.clone()));
            properties.push(PropertyDeclaration::BorderBottomStyle(solid));
            properties.push(PropertyDeclaration::BorderLeftStyle(dotted));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-style: solid dotted;");
        }

        use style::values::specified::{BorderCornerRadius, Percentage};

        #[test]
        fn border_radius_should_serialize_correctly() {
            let mut properties = Vec::new();
            properties.push(PropertyDeclaration::BorderTopLeftRadius(Box::new(BorderCornerRadius::new(
                Percentage::new(0.01).into(), Percentage::new(0.05).into()
            ))));
            properties.push(PropertyDeclaration::BorderTopRightRadius(Box::new(BorderCornerRadius::new(
                Percentage::new(0.02).into(), Percentage::new(0.06).into()
            ))));
            properties.push(PropertyDeclaration::BorderBottomRightRadius(Box::new(BorderCornerRadius::new(
                Percentage::new(0.03).into(), Percentage::new(0.07).into()
            ))));
            properties.push(PropertyDeclaration::BorderBottomLeftRadius(Box::new(BorderCornerRadius::new(
                Percentage::new(0.04).into(), Percentage::new(0.08).into()
            ))));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-radius: 1% 2% 3% 4% / 5% 6% 7% 8%;");
        }
    }


    mod border_shorthands {
        use super::*;

        #[test]
        fn border_top_and_color() {
            let mut properties = Vec::new();
            properties.push(PropertyDeclaration::BorderTopWidth(BorderSideWidth::Length(Length::from_px(1.))));
            properties.push(PropertyDeclaration::BorderTopStyle(BorderStyle::Solid));
            let c = Color::Numeric {
                parsed: RGBA::new(255, 0, 0, 255),
                authored: Some("green".to_string().into_boxed_str())
            };
            properties.push(PropertyDeclaration::BorderTopColor(c));
            let c = Color::Numeric {
                parsed: RGBA::new(0, 255, 0, 255),
                authored: Some("red".to_string().into_boxed_str())
            };
            properties.push(PropertyDeclaration::BorderTopColor(c.clone()));
            properties.push(PropertyDeclaration::BorderBottomColor(c.clone()));
            properties.push(PropertyDeclaration::BorderLeftColor(c.clone()));
            properties.push(PropertyDeclaration::BorderRightColor(c.clone()));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-top: 1px solid red; border-color: red;");
        }

        #[test]
        fn border_color_and_top() {
            let mut properties = Vec::new();
                let c = Color::Numeric {
                parsed: RGBA::new(0, 255, 0, 255),
                authored: Some("red".to_string().into_boxed_str())
            };
            properties.push(PropertyDeclaration::BorderTopColor(c.clone()));
            properties.push(PropertyDeclaration::BorderBottomColor(c.clone()));
            properties.push(PropertyDeclaration::BorderLeftColor(c.clone()));
            properties.push(PropertyDeclaration::BorderRightColor(c.clone()));

            properties.push(PropertyDeclaration::BorderTopWidth(BorderSideWidth::Length(Length::from_px(1.))));
            properties.push(PropertyDeclaration::BorderTopStyle(BorderStyle::Solid));
            let c = Color::Numeric {
                parsed: RGBA::new(255, 0, 0, 255),
                authored: Some("green".to_string().into_boxed_str())
            };
            properties.push(PropertyDeclaration::BorderTopColor(c));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-color: green red red; border-top: 1px solid green;");
        }

        // we can use border-top as a base to test out the different combinations
        // but afterwards, we only need to to one test per "directional border shorthand"

        #[test]
        fn directional_border_should_show_all_properties_when_values_are_set() {
            let mut properties = Vec::new();

            let width = BorderSideWidth::Length(Length::from_px(4f32));
            let style = BorderStyle::Solid;
            let color = RGBA::new(255, 0, 0, 255).into();

            properties.push(PropertyDeclaration::BorderTopWidth(width));
            properties.push(PropertyDeclaration::BorderTopStyle(style));
            properties.push(PropertyDeclaration::BorderTopColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-top: 4px solid rgb(255, 0, 0);");
        }

        fn get_border_property_values() -> (BorderSideWidth, BorderStyle, Color) {
            (BorderSideWidth::Length(Length::from_px(4f32)),
             BorderStyle::Solid,
             Color::currentcolor())
        }

        #[test]
        fn border_top_should_serialize_correctly() {
            let mut properties = Vec::new();
            let (width, style, color) = get_border_property_values();
            properties.push(PropertyDeclaration::BorderTopWidth(width));
            properties.push(PropertyDeclaration::BorderTopStyle(style));
            properties.push(PropertyDeclaration::BorderTopColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-top: 4px solid;");
        }

        #[test]
        fn border_right_should_serialize_correctly() {
            let mut properties = Vec::new();
            let (width, style, color) = get_border_property_values();
            properties.push(PropertyDeclaration::BorderRightWidth(width));
            properties.push(PropertyDeclaration::BorderRightStyle(style));
            properties.push(PropertyDeclaration::BorderRightColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-right: 4px solid;");
        }

        #[test]
        fn border_bottom_should_serialize_correctly() {
            let mut properties = Vec::new();
            let (width, style, color) = get_border_property_values();
            properties.push(PropertyDeclaration::BorderBottomWidth(width));
            properties.push(PropertyDeclaration::BorderBottomStyle(style));
            properties.push(PropertyDeclaration::BorderBottomColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-bottom: 4px solid;");
        }

        #[test]
        fn border_left_should_serialize_correctly() {
            let mut properties = Vec::new();
            let (width, style, color) = get_border_property_values();
            properties.push(PropertyDeclaration::BorderLeftWidth(width));
            properties.push(PropertyDeclaration::BorderLeftStyle(style));
            properties.push(PropertyDeclaration::BorderLeftColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-left: 4px solid;");
        }

        #[test]
        fn border_should_serialize_correctly() {
            // According to https://drafts.csswg.org/css-backgrounds-3/#the-border-shorthands,
            // the ‘border’ shorthand resets ‘border-image’ to its initial value. To verify the
            // serialization of 'border' shorthand, we need to set 'border-image' as well.
            let block_text = "\
                border-top: 4px solid; \
                border-right: 4px solid; \
                border-bottom: 4px solid; \
                border-left: 4px solid; \
                border-image: none;";

            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

            let serialization = block.to_css_string();

            assert_eq!(serialization, "border: 4px solid;");
        }
    }

    mod list_style {
        use style::properties::longhands::list_style_image::SpecifiedValue as ListStyleImage;
        use style::properties::longhands::list_style_position::SpecifiedValue as ListStylePosition;
        use style::properties::longhands::list_style_type::SpecifiedValue as ListStyleType;
        use style::values::Either;
        use super::*;

        #[test]
        fn list_style_should_show_all_properties_when_values_are_set() {
            let mut properties = Vec::new();

            let position = ListStylePosition::Inside;
            let image =
                ListStyleImage(Either::First(SpecifiedUrl::new_for_testing("http://servo/test.png")));
            let style_type = ListStyleType::Disc;

            properties.push(PropertyDeclaration::ListStylePosition(position));

            #[cfg(feature = "gecko")]
            properties.push(PropertyDeclaration::ListStyleImage(Box::new(image)));
            #[cfg(not(feature = "gecko"))]
            properties.push(PropertyDeclaration::ListStyleImage(image));

            properties.push(PropertyDeclaration::ListStyleType(style_type));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "list-style: inside url(\"http://servo/test.png\") disc;");
        }
    }

    mod outline {
        use style::values::specified::outline::OutlineStyle;
        use super::*;

        #[test]
        fn outline_should_show_all_properties_when_set() {
            let mut properties = Vec::new();

            let width = BorderSideWidth::Length(Length::from_px(4f32));
            let style = OutlineStyle::Other(BorderStyle::Solid);
            let color = RGBA::new(255, 0, 0, 255).into();

            properties.push(PropertyDeclaration::OutlineWidth(width));
            properties.push(PropertyDeclaration::OutlineStyle(style));
            properties.push(PropertyDeclaration::OutlineColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "outline: 4px solid rgb(255, 0, 0);");
        }

        #[test]
        fn outline_should_serialize_correctly_when_style_is_auto() {
            let mut properties = Vec::new();

            let width = BorderSideWidth::Length(Length::from_px(4f32));
            let style = OutlineStyle::Auto;
            let color = RGBA::new(255, 0, 0, 255).into();
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

        properties.push(PropertyDeclaration::ColumnWidth(width));
        properties.push(PropertyDeclaration::ColumnCount(count));

        let serialization = shorthand_properties_to_string(properties);
        assert_eq!(serialization, "columns: auto auto;");
    }

    #[test]
    fn flex_should_serialize_all_available_properties() {
        use style::values::specified::{NonNegativeNumber, Percentage};

        let mut properties = Vec::new();

        let grow = NonNegativeNumber::new(2f32);
        let shrink = NonNegativeNumber::new(3f32);
        let basis =
            FlexBasis::Length(Percentage::new(0.5f32).into());

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

        let direction = FlexDirection::Row;
        let wrap = FlexWrap::Wrap;

        properties.push(PropertyDeclaration::FlexDirection(direction));
        properties.push(PropertyDeclaration::FlexWrap(wrap));

        let serialization = shorthand_properties_to_string(properties);
        assert_eq!(serialization, "flex-flow: row wrap;");
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
                background-position-y: bottom 4px; \
                background-origin: border-box; \
                background-clip: padding-box;";

            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

            let serialization = block.to_css_string();

            assert_eq!(
                serialization,
                "background: rgb(255, 0, 0) url(\"http://servo/test.png\") repeat-x \
                scroll left 7px bottom 4px / 70px 50px border-box padding-box;"
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

            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

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

            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

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
            // Below background-origin only has one value.
            let block_text = "\
                background-color: rgb(0, 0, 255); \
                background-image: url(\"http://servo/test.png\"), none; \
                background-repeat: repeat-x, repeat-y; \
                background-attachment: scroll, scroll; \
                background-size: 70px 50px, 20px 30px; \
                background-position: 7px 4px, 5px 6px; \
                background-origin: border-box; \
                background-clip: padding-box, padding-box;";

            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

            let serialization = block.to_css_string();

            assert_eq!(serialization, block_text);
        }

        #[test]
        fn background_position_should_be_a_valid_form_its_longhands() {
            // If there is any longhand consisted of both keyword and position,
            // the shorthand result should be the 4-value format.
            let block_text = "\
                background-position-x: 30px;\
                background-position-y: bottom 20px;";
            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();
            let serialization = block.to_css_string();
            assert_eq!(serialization, "background-position: left 30px bottom 20px;");

            // If there is no longhand consisted of both keyword and position,
            // the shorthand result should be the 2-value format.
            let block_text = "\
                background-position-x: center;\
                background-position-y: 20px;";
            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();
            let serialization = block.to_css_string();
            assert_eq!(serialization, "background-position: center 20px;");
        }
    }

    mod transform {
        pub use super::*;
        use style::values::generics::transform::TransformOperation;
        use style::values::specified::{Angle, Number};
        use style::values::specified::transform::TransformOperation as SpecifiedOperation;

        #[test]
        fn should_serialize_none_correctly() {
            use style::properties::longhands::transform;

            assert_roundtrip_with_context!(transform::parse, "none");
        }

        #[inline(always)]
        fn validate_serialization(op: &SpecifiedOperation, expected_string: &'static str) {
            let css_string = op.to_css_string();
            assert_eq!(css_string, expected_string);
        }

        #[test]
        fn transform_scale() {
            validate_serialization(&TransformOperation::Scale(Number::new(1.3), None), "scale(1.3)");
            validate_serialization(
                &TransformOperation::Scale(Number::new(2.0), Some(Number::new(2.0))),
                "scale(2, 2)");
            validate_serialization(&TransformOperation::ScaleX(Number::new(42.0)), "scaleX(42)");
            validate_serialization(&TransformOperation::ScaleY(Number::new(0.3)), "scaleY(0.3)");
            validate_serialization(&TransformOperation::ScaleZ(Number::new(1.0)), "scaleZ(1)");
            validate_serialization(
                &TransformOperation::Scale3D(Number::new(4.0), Number::new(5.0), Number::new(6.0)),
                "scale3d(4, 5, 6)");
        }

        #[test]
        fn transform_skew() {
            validate_serialization(
                &TransformOperation::Skew(Angle::from_degrees(42.3, false), None),
                "skew(42.3deg)");
            validate_serialization(
                &TransformOperation::Skew(Angle::from_gradians(-50.0, false), Some(Angle::from_turns(0.73, false))),
                "skew(-50grad, 0.73turn)");
            validate_serialization(
                &TransformOperation::SkewX(Angle::from_radians(0.31, false)), "skewX(0.31rad)");
        }

        #[test]
        fn transform_rotate() {
            validate_serialization(
                &TransformOperation::Rotate(Angle::from_turns(35.0, false)),
                "rotate(35turn)"
            )
        }
    }

    mod quotes {
        pub use super::*;

        #[test]
        fn should_serialize_none_correctly() {
            use style::properties::longhands::quotes;

            assert_roundtrip_with_context!(quotes::parse, "none");
        }
    }

    mod animation {
        pub use super::*;

        #[test]
        fn serialize_single_animation() {
            let block_text = "\
                animation-name: bounce;\
                animation-duration: 1s;\
                animation-timing-function: ease-in;\
                animation-delay: 0s;\
                animation-direction: normal;\
                animation-fill-mode: forwards;\
                animation-iteration-count: infinite;\
                animation-play-state: paused;";

            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

            let serialization = block.to_css_string();

            assert_eq!(serialization, "animation: 1s ease-in 0s infinite normal forwards paused bounce;")
        }

        #[test]
        fn serialize_multiple_animations() {
            let block_text = "\
                animation-name: bounce, roll;\
                animation-duration: 1s, 0.2s;\
                animation-timing-function: ease-in, linear;\
                animation-delay: 0s, 1s;\
                animation-direction: normal, reverse;\
                animation-fill-mode: forwards, backwards;\
                animation-iteration-count: infinite, 2;\
                animation-play-state: paused, running;";

            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

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

            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

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

            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

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

            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

            let serialization = block.to_css_string();

            assert_eq!(serialization, "transition: margin-left 3s cubic-bezier(0.2, 5, 0.5, 2) 4s;");
        }

        #[test]
        fn serialize_multiple_transitions() {
            let block_text = "transition-property: margin-left, width; \
                              transition-duration: 3s, 2s; \
                              transition-delay: 4s, 5s; \
                              transition-timing-function: cubic-bezier(0.2, 5, 0.5, 2), ease;";

            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

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

            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

            let serialization = block.to_css_string();

            assert_eq!(serialization, block_text);
        }

        #[test]
        fn transition_should_serialize_acceptable_step_timing_function() {
            let block_text = "transition-property: margin-left; \
                              transition-duration: 3s; \
                              transition-delay: 4s; \
                              transition-timing-function: steps(2, start);";
            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

            let serialization = block.to_css_string();

            assert_eq!(serialization, "transition: margin-left 3s steps(2, start) 4s;");
        }

        #[test]
        fn transition_should_serialize_acceptable_frames_timing_function() {
            let block_text = "transition-property: margin-left; \
                              transition-duration: 3s; \
                              transition-delay: 4s; \
                              transition-timing-function: frames(2);";
            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

            let serialization = block.to_css_string();

            assert_eq!(serialization, "transition: margin-left 3s frames(2) 4s;");
        }
    }

    mod keywords {
        pub use super::*;
        #[test]
        fn css_wide_keywords_should_be_parsed() {
            let block_text = "--a:inherit;";
            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

            let serialization = block.to_css_string();
            assert_eq!(serialization, "--a: inherit;");
        }

        #[test]
        fn non_keyword_custom_property_should_be_unparsed() {
            let block_text = "--main-color: #06c;";
            let block = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), block_text).unwrap();

            let serialization = block.to_css_string();
            assert_eq!(serialization, block_text);
        }
    }

    mod effects {
        pub use super::*;
        pub use style::properties::longhands::box_shadow::SpecifiedValue as BoxShadowList;
        pub use style::values::specified::effects::{BoxShadow, SimpleShadow};

        #[test]
        fn box_shadow_should_serialize_correctly() {
            use style::values::specified::length::NonNegativeLength;

            let mut properties = Vec::new();
            let shadow_val = BoxShadow {
                base: SimpleShadow {
                    color: None,
                    horizontal: Length::from_px(1f32),
                    vertical: Length::from_px(2f32),
                    blur: Some(NonNegativeLength::from_px(3f32)),
                },
                spread: Some(Length::from_px(4f32)),
                inset: false,
            };
            let shadow_decl = BoxShadowList(vec![shadow_val]);
            properties.push(PropertyDeclaration::BoxShadow(shadow_decl));
            let shadow_css = "box-shadow: 1px 2px 3px 4px;";
            let shadow = parse(|c, e, i| Ok(parse_property_declaration_list(c, e, i)), shadow_css).unwrap();

            assert_eq!(shadow.to_css_string(), shadow_css);
        }
    }

    mod counter_increment {
        pub use super::*;
        pub use style::properties::longhands::counter_increment::SpecifiedValue as CounterIncrement;
        use style::values::specified::Integer;

        #[test]
        fn counter_increment_with_properties_should_serialize_correctly() {
            let mut properties = Vec::new();

            properties.push((CustomIdent("counter1".into()), Integer::new(1)));
            properties.push((CustomIdent("counter2".into()), Integer::new(-4)));

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
