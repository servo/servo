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
                 animation_type="none",
                 spec="https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty")}

${helpers.single_keyword("vector-effect", "none non-scaling-stroke",
                         products="gecko", animation_type="none",
                         spec="https://www.w3.org/TR/SVGTiny12/painting.html#VectorEffectProperty")}

// Section 13 - Gradients and Patterns

${helpers.predefined_type(
    "stop-color", "CSSColor",
    "CSSParserColor::RGBA(RGBA::new(0, 0, 0, 255))",
    products="gecko",
    animation_type="none",
    spec="https://www.w3.org/TR/SVGTiny12/painting.html#StopColorProperty")}

${helpers.predefined_type("stop-opacity", "Opacity", "1.0",
                          products="gecko",
                          animation_type="none",
                          spec="https://www.w3.org/TR/SVGTiny12/painting.html#propdef-stop-opacity")}

// Section 15 - Filter Effects

${helpers.predefined_type(
    "flood-color", "CSSColor",
    "CSSParserColor::RGBA(RGBA::new(0, 0, 0, 255))",
    products="gecko",
    animation_type="none",
    spec="https://www.w3.org/TR/SVG/filters.html#FloodColorProperty")}

${helpers.predefined_type("flood-opacity", "Opacity",
                          "1.0", products="gecko", animation_type="none",
                          spec="https://www.w3.org/TR/SVG/filters.html#FloodOpacityProperty")}

${helpers.predefined_type(
    "lighting-color", "CSSColor",
    "CSSParserColor::RGBA(RGBA::new(255, 255, 255, 255))",
    products="gecko",
    animation_type="none",
    spec="https://www.w3.org/TR/SVG/filters.html#LightingColorProperty")}

// CSS Masking Module Level 1
// https://drafts.fxtf.org/css-masking
${helpers.single_keyword("mask-type", "luminance alpha",
                         products="gecko", animation_type="none",
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-type")}

<%helpers:longhand name="clip-path" animation_type="none" products="gecko" boxed="True"
                   flags="CREATES_STACKING_CONTEXT"
                   spec="https://drafts.fxtf.org/css-masking/#propdef-clip-path">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
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

    no_viewport_percentage!(SpecifiedValue);
</%helpers:longhand>

${helpers.single_keyword("mask-mode",
                         "match-source alpha luminance",
                         vector=True,
                         products="gecko",
                         animation_type="none",
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-mode")}

<%helpers:vector_longhand name="mask-repeat" products="gecko" animation_type="none" extra_prefixes="webkit"
                          spec="https://drafts.fxtf.org/css-masking/#propdef-mask-repeat">
    pub use properties::longhands::background_repeat::single_value::parse;
    pub use properties::longhands::background_repeat::single_value::SpecifiedValue;
    pub use properties::longhands::background_repeat::single_value::computed_value;
    pub use properties::longhands::background_repeat::single_value::RepeatKeyword;
    use properties::longhands::background_repeat::single_value;

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(RepeatKeyword::NoRepeat, RepeatKeyword::NoRepeat)
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Other(RepeatKeyword::NoRepeat, None)
    }
</%helpers:vector_longhand>

<%helpers:vector_longhand name="mask-position-x" products="gecko" animation_type="normal" extra_prefixes="webkit"
                          spec="https://drafts.fxtf.org/css-masking/#propdef-mask-position">
    pub use properties::longhands::background_position_x::single_value::get_initial_value;
    pub use properties::longhands::background_position_x::single_value::get_initial_position_value;
    pub use properties::longhands::background_position_x::single_value::get_initial_specified_value;
    pub use properties::longhands::background_position_x::single_value::parse;
    pub use properties::longhands::background_position_x::single_value::SpecifiedValue;
    pub use properties::longhands::background_position_x::single_value::computed_value;
    use properties::animated_properties::{ComputeDistance, Interpolate, RepeatableListInterpolate};
    use properties::longhands::mask_position_x::computed_value::T as MaskPositionX;

    impl Interpolate for MaskPositionX {
        #[inline]
        fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
            Ok(MaskPositionX(try!(self.0.interpolate(&other.0, progress))))
        }
    }

    impl RepeatableListInterpolate for MaskPositionX {}

    impl ComputeDistance for MaskPositionX {
        #[inline]
        fn compute_distance(&self, _other: &Self) -> Result<f64, ()> {
            Err(())
        }
    }
</%helpers:vector_longhand>

<%helpers:vector_longhand name="mask-position-y" products="gecko" animation_type="normal" extra_prefixes="webkit"
                          spec="https://drafts.fxtf.org/css-masking/#propdef-mask-position">
    pub use properties::longhands::background_position_y::single_value::get_initial_value;
    pub use properties::longhands::background_position_y::single_value::get_initial_position_value;
    pub use properties::longhands::background_position_y::single_value::get_initial_specified_value;
    pub use properties::longhands::background_position_y::single_value::parse;
    pub use properties::longhands::background_position_y::single_value::SpecifiedValue;
    pub use properties::longhands::background_position_y::single_value::computed_value;
    use properties::animated_properties::{ComputeDistance, Interpolate, RepeatableListInterpolate};
    use properties::longhands::mask_position_y::computed_value::T as MaskPositionY;

    impl Interpolate for MaskPositionY {
        #[inline]
        fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
            Ok(MaskPositionY(try!(self.0.interpolate(&other.0, progress))))
        }
    }

    impl RepeatableListInterpolate for MaskPositionY {}

    impl ComputeDistance for MaskPositionY {
        #[inline]
        fn compute_distance(&self, _other: &Self) -> Result<f64, ()> {
            Err(())
        }
    }
</%helpers:vector_longhand>

${helpers.single_keyword("mask-clip",
                         "border-box content-box padding-box",
                         extra_gecko_values="fill-box stroke-box view-box no-clip",
                         vector=True,
                         products="gecko",
                         extra_prefixes="webkit",
                         animation_type="none",
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-clip")}

${helpers.single_keyword("mask-origin",
                         "border-box content-box padding-box",
                         extra_gecko_values="fill-box stroke-box view-box",
                         vector=True,
                         products="gecko",
                         extra_prefixes="webkit",
                         animation_type="none",
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-origin")}

<%helpers:longhand name="mask-size" products="gecko" animation_type="normal" extra_prefixes="webkit"
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
                         animation_type="none",
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-composite")}

<%helpers:vector_longhand name="mask-image" products="gecko" animation_type="none" extra_prefixes="webkit"
                          has_uncacheable_values="${product == 'gecko'}"
                          flags="CREATES_STACKING_CONTEXT",
                          spec="https://drafts.fxtf.org/css-masking/#propdef-mask-image">
    use std::fmt;
    use style_traits::ToCss;
    use std::sync::Arc;
    use values::specified::Image;
    use values::specified::url::SpecifiedUrl;
    use values::HasViewportPercentage;

    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;
        use values::computed;
        use values::specified::url::SpecifiedUrl;
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            Image(computed::Image),
            None
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    T::None => dest.write_str("none"),
                    T::Image(ref image) => image.to_css(dest),
                }
            }
        }
    }

    no_viewport_percentage!(SpecifiedValue);

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Image(Image),
        None
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Image(ref image) => image.to_css(dest),
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
                    Ok(SpecifiedValue::Image(Image::Url(url_value)))
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
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T::None => SpecifiedValue::None,
                computed_value::T::Image(ref image) =>
                    SpecifiedValue::Image(ToComputedValue::from_computed_value(image)),
            }
        }
    }
</%helpers:vector_longhand>
