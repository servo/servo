/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::properties::{parse, parse_input};
use crate::stylesheets::block_from;
use style::computed_values::display::T as Display;
use style::properties::declaration_block::PropertyDeclarationBlock;
use style::properties::parse_property_declaration_list;
use style::properties::{Importance, PropertyDeclaration};
use style::values::specified::url::SpecifiedUrl;
use style::values::specified::NoCalcLength;
use style::values::specified::{Length, LengthOrPercentage, LengthOrPercentageOrAuto};
use style_traits::ToCss;

trait ToCssString {
    fn to_css_string(&self) -> String;
}

impl ToCssString for PropertyDeclarationBlock {
    fn to_css_string(&self) -> String {
        let mut css = String::new();
        self.to_css(&mut css).unwrap();
        css
    }
}

#[test]
fn property_declaration_block_should_serialize_correctly() {
    use style::properties::longhands::overflow_x::SpecifiedValue as OverflowValue;

    let declarations = vec![
        (
            PropertyDeclaration::Width(LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(
                70f32,
            ))),
            Importance::Normal,
        ),
        (
            PropertyDeclaration::MinHeight(LengthOrPercentage::Length(NoCalcLength::from_px(
                20f32,
            ))),
            Importance::Normal,
        ),
        (
            PropertyDeclaration::Height(LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(
                20f32,
            ))),
            Importance::Important,
        ),
        (
            PropertyDeclaration::Display(Display::InlineBlock),
            Importance::Normal,
        ),
        (
            PropertyDeclaration::OverflowX(OverflowValue::Auto),
            Importance::Normal,
        ),
        (
            PropertyDeclaration::OverflowY(OverflowValue::Auto),
            Importance::Normal,
        ),
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

    mod list_style {
        use super::*;
        use style::properties::longhands::list_style_position::SpecifiedValue as ListStylePosition;
        use style::properties::longhands::list_style_type::SpecifiedValue as ListStyleType;
        use style::values::generics::url::UrlOrNone as ImageUrlOrNone;

        #[test]
        fn list_style_should_show_all_properties_when_values_are_set() {
            let mut properties = Vec::new();

            let position = ListStylePosition::Inside;
            let image = ImageUrlOrNone::Url(SpecifiedUrl::new_for_testing("http://servo/test.png"));
            let style_type = ListStyleType::Disc;

            properties.push(PropertyDeclaration::ListStylePosition(position));

            #[cfg(feature = "gecko")]
            properties.push(PropertyDeclaration::ListStyleImage(Box::new(image)));
            #[cfg(not(feature = "gecko"))]
            properties.push(PropertyDeclaration::ListStyleImage(image));

            properties.push(PropertyDeclaration::ListStyleType(style_type));

            let serialization = shorthand_properties_to_string(properties);
            assert_eq!(
                serialization,
                "list-style: inside url(\"http://servo/test.png\") disc;"
            );
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
                              background-position-y: bottom 4px; \
                              background-origin: border-box; \
                              background-clip: padding-box;";

            let block =
                parse(|c, i| Ok(parse_property_declaration_list(c, i)), block_text).unwrap();

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

            let block =
                parse(|c, i| Ok(parse_property_declaration_list(c, i)), block_text).unwrap();

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

            let block =
                parse(|c, i| Ok(parse_property_declaration_list(c, i)), block_text).unwrap();

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

            let block =
                parse(|c, i| Ok(parse_property_declaration_list(c, i)), block_text).unwrap();

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
            let block =
                parse(|c, i| Ok(parse_property_declaration_list(c, i)), block_text).unwrap();
            let serialization = block.to_css_string();
            assert_eq!(serialization, "background-position: left 30px bottom 20px;");

            // If there is no longhand consisted of both keyword and position,
            // the shorthand result should be the 2-value format.
            let block_text = "\
                              background-position-x: center;\
                              background-position-y: 20px;";
            let block =
                parse(|c, i| Ok(parse_property_declaration_list(c, i)), block_text).unwrap();
            let serialization = block.to_css_string();
            assert_eq!(serialization, "background-position: center 20px;");
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

            let block =
                parse(|c, i| Ok(parse_property_declaration_list(c, i)), block_text).unwrap();

            let serialization = block.to_css_string();

            assert_eq!(
                serialization,
                "animation: 1s ease-in 0s infinite normal forwards paused bounce;"
            )
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

            let block =
                parse(|c, i| Ok(parse_property_declaration_list(c, i)), block_text).unwrap();

            let serialization = block.to_css_string();

            assert_eq!(
                serialization,
                "animation: 1s ease-in 0s infinite normal forwards paused bounce, \
                 0.2s linear 1s 2 reverse backwards running roll;"
            );
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

            let block =
                parse(|c, i| Ok(parse_property_declaration_list(c, i)), block_text).unwrap();

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

            let block =
                parse(|c, i| Ok(parse_property_declaration_list(c, i)), block_text).unwrap();

            let serialization = block.to_css_string();

            assert_eq!(serialization, block_text);
        }
    }

    mod keywords {
        pub use super::*;
        #[test]
        fn css_wide_keywords_should_be_parsed() {
            let block_text = "--a:inherit;";
            let block =
                parse(|c, i| Ok(parse_property_declaration_list(c, i)), block_text).unwrap();

            let serialization = block.to_css_string();
            assert_eq!(serialization, "--a: inherit;");
        }

        #[test]
        fn non_keyword_custom_property_should_be_unparsed() {
            let block_text = "--main-color: #06c;";
            let block =
                parse(|c, i| Ok(parse_property_declaration_list(c, i)), block_text).unwrap();

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
            let shadow =
                parse(|c, i| Ok(parse_property_declaration_list(c, i)), shadow_css).unwrap();

            assert_eq!(shadow.to_css_string(), shadow_css);
        }
    }
}
