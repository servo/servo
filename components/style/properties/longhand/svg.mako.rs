/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("SVG", inherited=False, gecko_name="SVGReset") %>

// TODO: Which of these should be animatable properties?
${helpers.single_keyword("dominant-baseline",
                 """auto use-script no-change reset-size ideographic alphabetic hanging
                    mathematical central middle text-after-edge text-before-edge""",
                 products="gecko",
                 animatable=False,
                 spec="https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty")}

${helpers.single_keyword("vector-effect", "none non-scaling-stroke",
                         products="gecko", animatable=False,
                         spec="https://www.w3.org/TR/SVGTiny12/painting.html#VectorEffectProperty")}

// Section 13 - Gradients and Patterns

${helpers.predefined_type(
    "stop-color", "CSSColor",
    "CSSParserColor::RGBA(RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 })",
    products="gecko",
    animatable=False,
    spec="https://www.w3.org/TR/SVGTiny12/painting.html#StopColorProperty")}

${helpers.predefined_type("stop-opacity", "Opacity", "1.0",
                          products="gecko",
                          animatable=False,
                          spec="https://www.w3.org/TR/SVGTiny12/painting.html#propdef-stop-opacity")}

// Section 15 - Filter Effects

${helpers.predefined_type(
    "flood-color", "CSSColor",
    "CSSParserColor::RGBA(RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 })",
    products="gecko",
    animatable=False,
    spec="https://www.w3.org/TR/SVG/filters.html#FloodColorProperty")}

${helpers.predefined_type("flood-opacity", "Opacity",
                          "1.0", products="gecko", animatable=False,
                          spec="https://www.w3.org/TR/SVG/filters.html#FloodOpacityProperty")}

${helpers.predefined_type(
    "lighting-color", "CSSColor",
    "CSSParserColor::RGBA(RGBA { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 })",
    products="gecko",
    animatable=False,
    spec="https://www.w3.org/TR/SVG/filters.html#LightingColorProperty")}

// CSS Masking Module Level 1
// https://drafts.fxtf.org/css-masking
${helpers.single_keyword("mask-type", "luminance alpha",
                         products="gecko", animatable=False,
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-type")}

<%helpers:longhand name="clip-path" animatable="False" products="gecko"
                   spec="https://drafts.fxtf.org/css-masking/#propdef-clip-path">
    use std::fmt;
    use style_traits::ToCss;
    use values::NoViewportPercentage;
    use values::specified::basic_shape::{ShapeSource, GeometryBox};

    pub mod computed_value {
        use app_units::Au;
        use values::computed::basic_shape::{ShapeSource, GeometryBox};

        pub type T = ShapeSource<GeometryBox>;
    }

    pub type SpecifiedValue = ShapeSource<GeometryBox>;

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        Default::default()
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        ShapeSource::parse(context, input)
    }

    impl NoViewportPercentage for SpecifiedValue {}
</%helpers:longhand>

${helpers.single_keyword("mask-mode",
                         "alpha luminance match-source",
                         vector=True,
                         products="gecko",
                         animatable=False,
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-mode")}

// TODO implement all of repeat-style for background and mask
// https://drafts.csswg.org/css-backgrounds-3/#repeat-style
${helpers.single_keyword("mask-repeat",
                         "repeat repeat-x repeat-y space round no-repeat",
                         vector=True,
                         products="gecko",
                         extra_prefixes="webkit",
                         animatable=False,
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-repeat")}

<%helpers:vector_longhand name="mask-position" products="gecko" animatable="True" extra_prefixes="webkit"
                          spec="https://drafts.fxtf.org/css-masking/#propdef-mask-position">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::specified::position::Position;

    pub mod computed_value {
        use values::computed::position::Position;
        use properties::animated_properties::{Interpolate, RepeatableListInterpolate};
        use properties::longhands::mask_position::computed_value::T as MaskPosition;

        pub type T = Position;

        impl RepeatableListInterpolate for MaskPosition {}

        impl Interpolate for MaskPosition {
            fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
                Ok(MaskPosition(try!(self.0.interpolate(&other.0, progress))))
            }
        }
    }

    pub type SpecifiedValue = Position;

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        use values::computed::position::Position;
        Position {
            horizontal: computed::LengthOrPercentage::Percentage(0.0),
            vertical: computed::LengthOrPercentage::Percentage(0.0),
        }
    }
    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        use values::specified::Percentage;
        use values::specified::position::{HorizontalPosition, VerticalPosition};
        Position {
            horizontal: HorizontalPosition {
                keyword: None,
                position: Some(specified::LengthOrPercentage::Percentage(Percentage(0.0))),
            },
            vertical: VerticalPosition {
                keyword: None,
                position: Some(specified::LengthOrPercentage::Percentage(Percentage(0.0))),
            },
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser)
                 -> Result<SpecifiedValue, ()> {
        Position::parse(context, input)
    }
</%helpers:vector_longhand>

${helpers.single_keyword("mask-clip",
                         "content-box padding-box border-box",
                         extra_gecko_values="fill-box stroke-box view-box no-clip",
                         vector=True,
                         products="gecko",
                         extra_prefixes="webkit",
                         animatable=False,
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-clip")}

${helpers.single_keyword("mask-origin",
                         "content-box padding-box border-box",
                         extra_gecko_values="fill-box stroke-box view-box",
                         vector=True,
                         products="gecko",
                         extra_prefixes="webkit",
                         animatable=False,
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-origin")}

<%helpers:longhand name="mask-size" products="gecko" animatable="True" extra_prefixes="webkit"
                   spec="https://drafts.fxtf.org/css-masking/#propdef-mask-size">
    use properties::longhands::background_size;
    pub use ::properties::longhands::background_size::SpecifiedValue;
    pub use ::properties::longhands::background_size::single_value as single_value;
    pub use ::properties::longhands::background_size::computed_value as computed_value;

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        background_size::get_initial_value()
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        background_size::parse(context, input)
    }
</%helpers:longhand>

${helpers.single_keyword("mask-composite",
                         "add subtract intersect exclude",
                         vector=True,
                         products="gecko",
                         extra_prefixes="webkit",
                         animatable=False,
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-composite")}

<%helpers:vector_longhand name="mask-image" products="gecko" animatable="False" extra_prefixes="webkit"
                          has_uncacheable_values="${product == 'gecko'}",
                          spec="https://drafts.fxtf.org/css-masking/#propdef-mask-image">
    use std::fmt;
    use style_traits::ToCss;
    use std::sync::Arc;
    use values::specified::Image;
    use values::specified::url::SpecifiedUrl;
    use values::NoViewportPercentage;

    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;
        use values::computed;
        use values::specified::url::SpecifiedUrl;
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            Image(computed::Image),
            Url(SpecifiedUrl),
            None
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    T::None => dest.write_str("none"),
                    T::Image(ref image) => image.to_css(dest),
                    T::Url(ref url) => url.to_css(dest),
                }
            }
        }
    }

    impl NoViewportPercentage for SpecifiedValue {}

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Image(Image),
        Url(SpecifiedUrl),
        None
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Image(ref image) => image.to_css(dest),
                SpecifiedValue::Url(ref url) => url.to_css(dest),
                SpecifiedValue::None => dest.write_str("none"),
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::None
    }
    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::None
    }
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            Ok(SpecifiedValue::None)
        } else {
            let image = try!(Image::parse(context, input));
            match image {
                Image::Url(url_value) => {
                    let has_valid_url = match url_value.url() {
                        Some(url) => url.fragment().is_some(),
                        None => false,
                    };

                    if has_valid_url {
                        Ok(SpecifiedValue::Url(url_value))
                    } else {
                        Ok(SpecifiedValue::Image(Image::Url(url_value)))
                    }
                }
                image => Ok(SpecifiedValue::Image(image))
            }
        }
    }
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            match *self {
                SpecifiedValue::None => computed_value::T::None,
                SpecifiedValue::Image(ref image) =>
                    computed_value::T::Image(image.to_computed_value(context)),
                SpecifiedValue::Url(ref url) =>
                    computed_value::T::Url(url.clone()),
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T::None => SpecifiedValue::None,
                computed_value::T::Image(ref image) =>
                    SpecifiedValue::Image(ToComputedValue::from_computed_value(image)),
                computed_value::T::Url(ref url) =>
                    SpecifiedValue::Url(url.clone()),
            }
        }
    }
</%helpers:vector_longhand>
