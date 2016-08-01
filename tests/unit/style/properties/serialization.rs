/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use cssparser::ToCss;
pub use std::sync::Arc;
pub use style::computed_values::display::T::inline_block;
pub use style::properties::{DeclaredValue, PropertyDeclaration, PropertyDeclarationBlock};
pub use style::values::specified::{BorderStyle, CSSColor, Length};
pub use style::values::specified::{LengthOrPercentage, LengthOrPercentageOrAuto, LengthOrPercentageOrAutoOrContent};
pub use style::properties::longhands::outline_color::computed_value::T as ComputedColor;
pub use style::values::RGBA;

#[test]
fn property_declaration_block_should_serialize_correctly() {
    use style::properties::longhands::overflow_x::computed_value::T as OverflowXValue;
    use style::properties::longhands::overflow_y::computed_value::T as OverflowYContainer;

    let mut normal = Vec::new();
    let mut important = Vec::new();

    let length = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(Length::from_px(70f32)));
    normal.push(PropertyDeclaration::Width(length));

    let min_height = DeclaredValue::Value(LengthOrPercentage::Length(Length::from_px(20f32)));
    normal.push(PropertyDeclaration::MinHeight(min_height));

    let value = DeclaredValue::Value(inline_block);
    normal.push(PropertyDeclaration::Display(value));

    let overflow_x = DeclaredValue::Value(OverflowXValue::auto);
    normal.push(PropertyDeclaration::OverflowX(overflow_x));

    let overflow_y = DeclaredValue::Value(OverflowYContainer(OverflowXValue::auto));
    normal.push(PropertyDeclaration::OverflowY(overflow_y));

    let height = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(Length::from_px(20f32)));
    important.push(PropertyDeclaration::Height(height));

    normal.reverse();
    important.reverse();
    let block = PropertyDeclarationBlock {
        normal: Arc::new(normal),
        important: Arc::new(important)
    };

    let css_string = block.to_css_string();

    assert_eq!(
        css_string,
        "width: 70px; min-height: 20px; display: inline-block; overflow: auto; height: 20px !important;"
    );
}

mod shorthand_serialization {
    pub use super::*;

    pub fn shorthand_properties_to_string(properties: Vec<PropertyDeclaration>) -> String {
        let block = PropertyDeclarationBlock {
            normal: Arc::new(properties),
            important: Arc::new(Vec::new())
        };

        block.to_css_string()
    }

    // Add Test to show error if a longhand property is missing!!!!!!

    mod overflow {
        pub use super::*;
        use style::properties::longhands::overflow_x::computed_value::T as OverflowXValue;
        use style::properties::longhands::overflow_y::computed_value::T as OverflowYContainer;

        #[test]
        fn equal_overflow_properties_should_serialize_to_single_value() {
            let mut properties = Vec::new();

            let overflow_x = DeclaredValue::Value(OverflowXValue::auto);
            properties.push(PropertyDeclaration::OverflowX(overflow_x));

            let overflow_y = DeclaredValue::Value(OverflowYContainer(OverflowXValue::auto));
            properties.push(PropertyDeclaration::OverflowY(overflow_y));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "overflow: auto;");
        }

        #[test]
        fn different_overflow_properties_should_serialize_to_two_values() {
            let mut properties = Vec::new();

            let overflow_x = DeclaredValue::Value(OverflowXValue::scroll);
            properties.push(PropertyDeclaration::OverflowX(overflow_x));

            let overflow_y = DeclaredValue::Value(OverflowYContainer(OverflowXValue::auto));
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

            let px_70 = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(Length::from_px(70f32)));
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

            let vertical_px = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(Length::from_px(10f32)));
            let horizontal_px = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(Length::from_px(5f32)));

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

            let top_px = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(Length::from_px(8f32)));
            let bottom_px = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(Length::from_px(10f32)));
            let horizontal_px = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(Length::from_px(5f32)));

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

            let top_px = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(Length::from_px(8f32)));
            let right_px = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(Length::from_px(12f32)));
            let bottom_px = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(Length::from_px(10f32)));
            let left_px = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(Length::from_px(14f32)));

            properties.push(PropertyDeclaration::MarginTop(top_px));
            properties.push(PropertyDeclaration::MarginRight(right_px));
            properties.push(PropertyDeclaration::MarginBottom(bottom_px));
            properties.push(PropertyDeclaration::MarginLeft(left_px));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "margin: 8px 12px 10px 14px;");
        }

        #[test]
        fn padding_should_serialize_correctly() {
            let mut properties = Vec::new();

            let px_10 = DeclaredValue::Value(LengthOrPercentage::Length(Length::from_px(10f32)));
            let px_15 = DeclaredValue::Value(LengthOrPercentage::Length(Length::from_px(15f32)));
            properties.push(PropertyDeclaration::PaddingTop(px_10.clone()));
            properties.push(PropertyDeclaration::PaddingRight(px_15.clone()));
            properties.push(PropertyDeclaration::PaddingBottom(px_10));
            properties.push(PropertyDeclaration::PaddingLeft(px_15));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "padding: 10px 15px;");
        }

        #[test]
        fn border_width_should_serialize_correctly() {
            use style::properties::longhands::border_bottom_width::SpecifiedValue as BottomContainer;
            use style::properties::longhands::border_left_width::SpecifiedValue as LeftContainer;
            use style::properties::longhands::border_right_width::SpecifiedValue as RightContainer;
            use style::properties::longhands::border_top_width::SpecifiedValue as TopContainer;

            let mut properties = Vec::new();

            let top_px = DeclaredValue::Value(TopContainer(Length::from_px(10f32)));
            let bottom_px = DeclaredValue::Value(BottomContainer(Length::from_px(10f32)));

            let right_px = DeclaredValue::Value(RightContainer(Length::from_px(15f32)));
            let left_px = DeclaredValue::Value(LeftContainer(Length::from_px(15f32)));

            properties.push(PropertyDeclaration::BorderTopWidth(top_px));
            properties.push(PropertyDeclaration::BorderRightWidth(right_px));
            properties.push(PropertyDeclaration::BorderBottomWidth(bottom_px));
            properties.push(PropertyDeclaration::BorderLeftWidth(left_px));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-width: 10px 15px;");
        }

        #[test]
        fn border_color_should_serialize_correctly() {
            let mut properties = Vec::new();

            let red = DeclaredValue::Value(CSSColor {
                parsed: ComputedColor::RGBA(RGBA { red: 1f32, green: 0f32, blue: 0f32, alpha: 1f32 }),
                authored: None
            });

            let blue = DeclaredValue::Value(CSSColor {
                parsed: ComputedColor::RGBA(RGBA { red: 0f32, green: 0f32, blue: 1f32, alpha: 1f32 }),
                authored: None
            });

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

            let solid = DeclaredValue::Value(BorderStyle::solid);
            let dotted = DeclaredValue::Value(BorderStyle::dotted);
            properties.push(PropertyDeclaration::BorderTopStyle(solid.clone()));
            properties.push(PropertyDeclaration::BorderRightStyle(dotted.clone()));
            properties.push(PropertyDeclaration::BorderBottomStyle(solid));
            properties.push(PropertyDeclaration::BorderLeftStyle(dotted));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-style: solid dotted;");
        }
    }


    mod border_shorthands {
        use style::properties::longhands::border_bottom_width::SpecifiedValue as BottomContainer;
        use style::properties::longhands::border_left_width::SpecifiedValue as LeftContainer;
        use style::properties::longhands::border_right_width::SpecifiedValue as RightContainer;
        use style::properties::longhands::border_top_width::SpecifiedValue as TopContainer;
        use super::*;

        // we can use border-top as a base to test out the different combinations
        // but afterwards, we only need to to one test per "directional border shorthand"

        #[test]
        fn directional_border_should_show_all_properties_when_values_are_set() {
            let mut properties = Vec::new();

            let width = DeclaredValue::Value(TopContainer(Length::from_px(4f32)));
            let style = DeclaredValue::Value(BorderStyle::solid);
            let color = DeclaredValue::Value(CSSColor {
                parsed: ComputedColor::RGBA(RGBA { red: 1f32, green: 0f32, blue: 0f32, alpha: 1f32 }),
                authored: None
            });

            properties.push(PropertyDeclaration::BorderTopWidth(width));
            properties.push(PropertyDeclaration::BorderTopStyle(style));
            properties.push(PropertyDeclaration::BorderTopColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-top: 4px solid rgb(255, 0, 0);");
        }

        #[test]
        fn directional_border_with_no_specified_style_will_show_style_as_none() {
            let mut properties = Vec::new();

            let width = DeclaredValue::Value(TopContainer(Length::from_px(4f32)));
            let style = DeclaredValue::Initial;
            let color = DeclaredValue::Value(CSSColor {
                parsed: ComputedColor::RGBA(RGBA { red: 1f32, green: 0f32, blue: 0f32, alpha: 1f32 }),
                authored: None
            });

            properties.push(PropertyDeclaration::BorderTopWidth(width));
            properties.push(PropertyDeclaration::BorderTopStyle(style));
            properties.push(PropertyDeclaration::BorderTopColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-top: 4px none rgb(255, 0, 0);");
        }

        #[test]
        fn directional_border_with_no_specified_color_will_not_show_color() {
            let mut properties = Vec::new();

            let width = DeclaredValue::Value(TopContainer(Length::from_px(4f32)));
            let style = DeclaredValue::Value(BorderStyle::solid);
            let color = DeclaredValue::Initial;

            properties.push(PropertyDeclaration::BorderTopWidth(width));
            properties.push(PropertyDeclaration::BorderTopStyle(style));
            properties.push(PropertyDeclaration::BorderTopColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-top: 4px solid;");
        }

        #[test]
        fn border_right_should_serialize_correctly() {
            let mut properties = Vec::new();

            let width = DeclaredValue::Value(RightContainer(Length::from_px(4f32)));
            let style = DeclaredValue::Value(BorderStyle::solid);
            let color = DeclaredValue::Initial;

            properties.push(PropertyDeclaration::BorderRightWidth(width));
            properties.push(PropertyDeclaration::BorderRightStyle(style));
            properties.push(PropertyDeclaration::BorderRightColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-right: 4px solid;");
        }

        #[test]
        fn border_bottom_should_serialize_correctly() {
            let mut properties = Vec::new();

            let width = DeclaredValue::Value(BottomContainer(Length::from_px(4f32)));
            let style = DeclaredValue::Value(BorderStyle::solid);
            let color = DeclaredValue::Initial;

            properties.push(PropertyDeclaration::BorderBottomWidth(width));
            properties.push(PropertyDeclaration::BorderBottomStyle(style));
            properties.push(PropertyDeclaration::BorderBottomColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-bottom: 4px solid;");
        }

        #[test]
        fn border_left_should_serialize_correctly() {
            let mut properties = Vec::new();

            let width = DeclaredValue::Value(LeftContainer(Length::from_px(4f32)));
            let style = DeclaredValue::Value(BorderStyle::solid);
            let color = DeclaredValue::Initial;

            properties.push(PropertyDeclaration::BorderLeftWidth(width));
            properties.push(PropertyDeclaration::BorderLeftStyle(style));
            properties.push(PropertyDeclaration::BorderLeftColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border-left: 4px solid;");
        }

        #[test]
        fn border_should_serialize_correctly() {
            let mut properties = Vec::new();

            let top_width = DeclaredValue::Value(TopContainer(Length::from_px(4f32)));
            let top_style = DeclaredValue::Value(BorderStyle::solid);
            let top_color = DeclaredValue::Initial;

            properties.push(PropertyDeclaration::BorderTopWidth(top_width));
            properties.push(PropertyDeclaration::BorderTopStyle(top_style));
            properties.push(PropertyDeclaration::BorderTopColor(top_color));

            let right_width = DeclaredValue::Value(RightContainer(Length::from_px(4f32)));
            let right_style = DeclaredValue::Value(BorderStyle::solid);
            let right_color = DeclaredValue::Initial;

            properties.push(PropertyDeclaration::BorderRightWidth(right_width));
            properties.push(PropertyDeclaration::BorderRightStyle(right_style));
            properties.push(PropertyDeclaration::BorderRightColor(right_color));

            let bottom_width = DeclaredValue::Value(BottomContainer(Length::from_px(4f32)));
            let bottom_style = DeclaredValue::Value(BorderStyle::solid);
            let bottom_color = DeclaredValue::Initial;

            properties.push(PropertyDeclaration::BorderBottomWidth(bottom_width));
            properties.push(PropertyDeclaration::BorderBottomStyle(bottom_style));
            properties.push(PropertyDeclaration::BorderBottomColor(bottom_color));

            let left_width = DeclaredValue::Value(LeftContainer(Length::from_px(4f32)));
            let left_style = DeclaredValue::Value(BorderStyle::solid);
            let left_color = DeclaredValue::Initial;

            properties.push(PropertyDeclaration::BorderLeftWidth(left_width));
            properties.push(PropertyDeclaration::BorderLeftStyle(left_style));
            properties.push(PropertyDeclaration::BorderLeftColor(left_color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "border: 4px solid;");
        }
    }

    mod list_style {
        use style::properties::longhands::list_style_image::SpecifiedValue as ListStyleImage;
        use style::properties::longhands::list_style_position::computed_value::T as ListStylePosition;
        use style::properties::longhands::list_style_type::computed_value::T as ListStyleType;
        use super::*;
        use url::Url;

        #[test]
        fn list_style_should_show_all_properties_when_values_are_set() {
            let mut properties = Vec::new();

            let position = DeclaredValue::Value(ListStylePosition::inside);
            let image = DeclaredValue::Value(ListStyleImage::Url(
                Url::parse("http://servo/test.png").unwrap()
            ));
            let style_type = DeclaredValue::Value(ListStyleType::disc);

            properties.push(PropertyDeclaration::ListStylePosition(position));
            properties.push(PropertyDeclaration::ListStyleImage(image));
            properties.push(PropertyDeclaration::ListStyleType(style_type));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "list-style: inside url(\"http://servo/test.png\") disc;");
        }

        #[test]
        fn list_style_should_show_all_properties_even_if_only_one_is_set() {
            let mut properties = Vec::new();

            let position = DeclaredValue::Initial;
            let image = DeclaredValue::Initial;
            let style_type = DeclaredValue::Value(ListStyleType::disc);

            properties.push(PropertyDeclaration::ListStylePosition(position));
            properties.push(PropertyDeclaration::ListStyleImage(image));
            properties.push(PropertyDeclaration::ListStyleType(style_type));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "list-style: outside none disc;");
        }
    }

    #[test]
    fn overflow_wrap_should_only_serialize_with_a_single_property() {
        use style::properties::longhands::overflow_wrap::computed_value::T as OverflowWrap;

        let value = DeclaredValue::Value(OverflowWrap::break_word);

        let properties = vec![
            PropertyDeclaration::OverflowWrap(value)
        ];

        let serialization = shorthand_properties_to_string(properties);

        // word-wrap is considered an outdated alternative to overflow-wrap, but it is currently
        // what servo is using in its naming conventions:
        // https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-wrap
        assert_eq!(serialization, "word-wrap: break-word;");
    }

    mod outline {
        use style::properties::longhands::outline_width::SpecifiedValue as WidthContainer;
        use super::*;

        #[test]
        fn outline_should_show_all_properties_when_set() {
            let mut properties = Vec::new();

            let width = DeclaredValue::Value(WidthContainer(Length::from_px(4f32)));
            let style = DeclaredValue::Value(BorderStyle::solid);
            let color = DeclaredValue::Value(CSSColor {
                parsed: ComputedColor::RGBA(RGBA { red: 1f32, green: 0f32, blue: 0f32, alpha: 1f32 }),
                authored: None
            });

            properties.push(PropertyDeclaration::OutlineWidth(width));
            properties.push(PropertyDeclaration::OutlineStyle(style));
            properties.push(PropertyDeclaration::OutlineColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "outline: 4px solid rgb(255, 0, 0);");
        }

        #[test]
        fn outline_should_not_show_color_if_not_set() {
            let mut properties = Vec::new();

            let width = DeclaredValue::Value(WidthContainer(Length::from_px(4f32)));
            let style = DeclaredValue::Value(BorderStyle::solid);
            let color = DeclaredValue::Initial;

            properties.push(PropertyDeclaration::OutlineWidth(width));
            properties.push(PropertyDeclaration::OutlineStyle(style));
            properties.push(PropertyDeclaration::OutlineColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "outline: 4px solid;");
        }

        #[test]
        fn outline_should_serialize_correctly_when_style_is_not_set() {
            let mut properties = Vec::new();

            let width = DeclaredValue::Value(WidthContainer(Length::from_px(4f32)));
            let style = DeclaredValue::Initial;
            let color = DeclaredValue::Value(CSSColor {
                parsed: ComputedColor::RGBA(RGBA { red: 1f32, green: 0f32, blue: 0f32, alpha: 1f32 }),
                authored: None
            });
            properties.push(PropertyDeclaration::OutlineWidth(width));
            properties.push(PropertyDeclaration::OutlineStyle(style));
            properties.push(PropertyDeclaration::OutlineColor(color));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "outline: 4px none rgb(255, 0, 0);");
        }
    }

    #[test]
    fn columns_should_serialize_correctly() {
        use style::properties::longhands::column_count::SpecifiedValue as ColumnCount;
        use style::properties::longhands::column_width::SpecifiedValue as ColumnWidth;

        let mut properties = Vec::new();

        let width = DeclaredValue::Value(ColumnWidth::Auto);
        let count = DeclaredValue::Value(ColumnCount::Auto);

        properties.push(PropertyDeclaration::ColumnWidth(width));
        properties.push(PropertyDeclaration::ColumnCount(count));

        let serialization = shorthand_properties_to_string(properties);
        assert_eq!(serialization, "columns: auto auto;");
    }

    #[test]
    fn transition_should_serialize_all_available_properties() {
        use euclid::point::Point2D;
        use style::properties::animated_properties::TransitionProperty;
        use style::properties::longhands::transition_duration::computed_value::T as DurationContainer;
        use style::properties::longhands::transition_property::computed_value::T as PropertyContainer;
        use style::properties::longhands::transition_timing_function::computed_value::T as TimingContainer;
        use style::properties::longhands::transition_timing_function::computed_value::TransitionTimingFunction;
        use style::values::specified::Time as TimeContainer;

        let property_name = DeclaredValue::Value(
            PropertyContainer(vec![TransitionProperty::MarginLeft])
        );

        let duration = DeclaredValue::Value(
            DurationContainer(vec![TimeContainer(3f32)])
        );

        let delay = DeclaredValue::Value(
            DurationContainer(vec![TimeContainer(4f32)])
        );

        let timing_function = DeclaredValue::Value(
            TimingContainer(vec![
                TransitionTimingFunction::CubicBezier(Point2D::new(0f32, 5f32), Point2D::new(5f32, 10f32))
            ])
        );

        let mut properties = Vec::new();

        properties.push(PropertyDeclaration::TransitionProperty(property_name));
        properties.push(PropertyDeclaration::TransitionDelay(delay));
        properties.push(PropertyDeclaration::TransitionDuration(duration));
        properties.push(PropertyDeclaration::TransitionTimingFunction(timing_function));

        let serialization = shorthand_properties_to_string(properties);
        assert_eq!(serialization, "transition: margin-left 3s cubic-bezier(0, 5, 5, 10) 4s;");
    }

    #[test]
    fn flex_should_serialize_all_available_properties() {
        use style::values::specified::Number as NumberContainer;
        use style::values::specified::Percentage as PercentageContainer;

        let mut properties = Vec::new();

        let grow = DeclaredValue::Value(NumberContainer(2f32));
        let shrink = DeclaredValue::Value(NumberContainer(3f32));
        let basis = DeclaredValue::Value(
            LengthOrPercentageOrAutoOrContent::Percentage(PercentageContainer(0.5f32))
        );

        properties.push(PropertyDeclaration::FlexGrow(grow));
        properties.push(PropertyDeclaration::FlexShrink(shrink));
        properties.push(PropertyDeclaration::FlexBasis(basis));

        let serialization = shorthand_properties_to_string(properties);
        assert_eq!(serialization, "flex: 2 3 50%;");
    }

    #[test]
    fn flex_flow_should_serialize_all_available_properties() {
        use style::properties::longhands::flex_direction::computed_value::T as FlexDirection;
        use style::properties::longhands::flex_wrap::computed_value::T as FlexWrap;

        let mut properties = Vec::new();

        let direction = DeclaredValue::Value(FlexDirection::row);
        let wrap = DeclaredValue::Value(FlexWrap::wrap);

        properties.push(PropertyDeclaration::FlexDirection(direction));
        properties.push(PropertyDeclaration::FlexWrap(wrap));

        let serialization = shorthand_properties_to_string(properties);
        assert_eq!(serialization, "flex-flow: row wrap;");
    }

    // TODO: Populate Atom Cache for testing so that the font shorthand can be tested
    /*
    mod font {
        use super::*;
        use style::properties::longhands::font_family::computed_value::T as FamilyContainer;
        use style::properties::longhands::font_family::computed_value::FontFamily;
        use style::properties::longhands::font_style::computed_value::T as FontStyle;
        use style::properties::longhands::font_variant::computed_value::T as FontVariant;
        use style::properties::longhands::font_weight::SpecifiedValue as FontWeight;
        use style::properties::longhands::font_size::SpecifiedValue as FontSizeContainer;
        use style::properties::longhands::font_stretch::computed_value::T as FontStretch;
        use style::properties::longhands::line_height::SpecifiedValue as LineHeight;

        #[test]
        fn font_should_serialize_all_available_properties() {
            let mut properties = Vec::new();


            let font_family = DeclaredValue::Value(
                FamilyContainer(vec![FontFamily::Generic(atom!("serif"))])
            );


            let font_style = DeclaredValue::Value(FontStyle::italic);
            let font_variant = DeclaredValue::Value(FontVariant::normal);
            let font_weight = DeclaredValue::Value(FontWeight::Bolder);
            let font_size = DeclaredValue::Value(FontSizeContainer(
                LengthOrPercentage::Length(Length::from_px(4f32)))
            );
            let font_stretch = DeclaredValue::Value(FontStretch::expanded);
            let line_height = DeclaredValue::Value(LineHeight::Number(3f32));

            properties.push(PropertyDeclaration::FontFamily(font_family));
            properties.push(PropertyDeclaration::FontStyle(font_style));
            properties.push(PropertyDeclaration::FontVariant(font_variant));
            properties.push(PropertyDeclaration::FontWeight(font_weight));
            properties.push(PropertyDeclaration::FontSize(font_size));
            properties.push(PropertyDeclaration::FontStretch(font_stretch));
            properties.push(PropertyDeclaration::LineHeight(line_height));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "font:;");
        }
    }
    */

    // TODO: Populate Atom Cache for testing so that the animation shorthand can be tested
    /*
    #[test]
    fn animation_should_serialize_all_available_properties() {
        let mut properties = Vec::new();

        assert_eq!(serialization, "animation;");
    }
    */

    mod background {
        use style::properties::longhands::background_attachment::computed_value::T as Attachment;
        use style::properties::longhands::background_clip::computed_value::T as Clip;
        use style::properties::longhands::background_image::SpecifiedValue as ImageContainer;
        use style::properties::longhands::background_origin::computed_value::T as Origin;
        use style::properties::longhands::background_position::SpecifiedValue as Position;
        use style::properties::longhands::background_repeat::computed_value::T as Repeat;
        use style::properties::longhands::background_size::SpecifiedExplicitSize;
        use style::properties::longhands::background_size::SpecifiedValue as Size;
        use style::values::specified::Image;
        use super::*;
        use url::Url;


        #[test]
        fn background_should_serialize_all_available_properties_when_specified() {
            let mut properties = Vec::new();

            let color = DeclaredValue::Value(CSSColor {
                parsed: ComputedColor::RGBA(RGBA { red: 1f32, green: 0f32, blue: 0f32, alpha: 1f32 }),
                authored: None
            });

            let position = DeclaredValue::Value(
                Position {
                    horizontal: LengthOrPercentage::Length(Length::from_px(7f32)),
                    vertical: LengthOrPercentage::Length(Length::from_px(4f32))
                }
            );

            let repeat = DeclaredValue::Value(Repeat::repeat_x);
            let attachment = DeclaredValue::Value(Attachment::scroll);

            let image = DeclaredValue::Value(ImageContainer(
                Some(Image::Url(Url::parse("http://servo/test.png").unwrap()))
            ));

            let size = DeclaredValue::Value(
                Size::Explicit(SpecifiedExplicitSize {
                    width: LengthOrPercentageOrAuto::Length(Length::from_px(70f32)),
                    height: LengthOrPercentageOrAuto::Length(Length::from_px(50f32))
                }
            ));

            let origin = DeclaredValue::Value(Origin::border_box);
            let clip = DeclaredValue::Value(Clip::padding_box);

            properties.push(PropertyDeclaration::BackgroundColor(color));
            properties.push(PropertyDeclaration::BackgroundPosition(position));
            properties.push(PropertyDeclaration::BackgroundRepeat(repeat));
            properties.push(PropertyDeclaration::BackgroundAttachment(attachment));
            properties.push(PropertyDeclaration::BackgroundImage(image));
            properties.push(PropertyDeclaration::BackgroundSize(size));
            properties.push(PropertyDeclaration::BackgroundOrigin(origin));
            properties.push(PropertyDeclaration::BackgroundClip(clip));

            let serialization = shorthand_properties_to_string(properties);

            assert_eq!(
                serialization,
                "background: rgb(255, 0, 0) url(\"http://servo/test.png\") repeat-x \
                scroll 7px 4px / 70px 50px border-box padding-box;"
            );
        }

        #[test]
        fn background_should_combine_origin_and_clip_properties_when_equal() {
            let mut properties = Vec::new();

            let color = DeclaredValue::Value(CSSColor {
                parsed: ComputedColor::RGBA(RGBA { red: 1f32, green: 0f32, blue: 0f32, alpha: 1f32 }),
                authored: None
            });

            let position = DeclaredValue::Value(
                Position {
                    horizontal: LengthOrPercentage::Length(Length::from_px(7f32)),
                    vertical: LengthOrPercentage::Length(Length::from_px(4f32))
                }
            );

            let repeat = DeclaredValue::Value(Repeat::repeat_x);
            let attachment = DeclaredValue::Value(Attachment::scroll);

            let image = DeclaredValue::Value(ImageContainer(
                Some(Image::Url(Url::parse("http://servo/test.png").unwrap()))
            ));

            let size = DeclaredValue::Value(
                Size::Explicit(SpecifiedExplicitSize {
                    width: LengthOrPercentageOrAuto::Length(Length::from_px(70f32)),
                    height: LengthOrPercentageOrAuto::Length(Length::from_px(50f32))
                })
            );

            let origin = DeclaredValue::Value(Origin::padding_box);
            let clip = DeclaredValue::Value(Clip::padding_box);

            properties.push(PropertyDeclaration::BackgroundColor(color));
            properties.push(PropertyDeclaration::BackgroundPosition(position));
            properties.push(PropertyDeclaration::BackgroundRepeat(repeat));
            properties.push(PropertyDeclaration::BackgroundAttachment(attachment));
            properties.push(PropertyDeclaration::BackgroundImage(image));
            properties.push(PropertyDeclaration::BackgroundSize(size));
            properties.push(PropertyDeclaration::BackgroundOrigin(origin));
            properties.push(PropertyDeclaration::BackgroundClip(clip));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(
                serialization,
                "background: rgb(255, 0, 0) url(\"http://servo/test.png\") repeat-x \
                scroll 7px 4px / 70px 50px padding-box;"
            );
        }

        #[test]
        fn background_should_always_print_color_and_url_and_repeat_and_attachment_and_position() {
            let mut properties = Vec::new();

            let color = DeclaredValue::Value(CSSColor {
                parsed: ComputedColor::RGBA(RGBA { red: 1f32, green: 0f32, blue: 0f32, alpha: 1f32 }),
                authored: None
            });

            let position = DeclaredValue::Value(
                Position {
                    horizontal: LengthOrPercentage::Length(Length::from_px(0f32)),
                    vertical: LengthOrPercentage::Length(Length::from_px(0f32))
                }
            );

            let repeat = DeclaredValue::Value(Repeat::repeat_x);
            let attachment = DeclaredValue::Value(Attachment::scroll);

            let image = DeclaredValue::Value(ImageContainer(None));

            let size = DeclaredValue::Initial;

            let origin = DeclaredValue::Initial;
            let clip = DeclaredValue::Initial;

            properties.push(PropertyDeclaration::BackgroundColor(color));
            properties.push(PropertyDeclaration::BackgroundPosition(position));
            properties.push(PropertyDeclaration::BackgroundRepeat(repeat));
            properties.push(PropertyDeclaration::BackgroundAttachment(attachment));
            properties.push(PropertyDeclaration::BackgroundImage(image));
            properties.push(PropertyDeclaration::BackgroundSize(size));
            properties.push(PropertyDeclaration::BackgroundOrigin(origin));
            properties.push(PropertyDeclaration::BackgroundClip(clip));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(serialization, "background: rgb(255, 0, 0) none repeat-x scroll 0px 0px;");
        }
    }
}
