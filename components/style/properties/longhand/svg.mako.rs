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
                 animatable=False)}

${helpers.single_keyword("vector-effect", "none non-scaling-stroke",
                         products="gecko", animatable=False)}

// Section 13 - Gradients and Patterns

${helpers.predefined_type(
    "stop-color", "CSSColor",
    "CSSParserColor::RGBA(RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 })",
    products="gecko",
    animatable=False)}

${helpers.predefined_type("stop-opacity", "Opacity", "1.0",
                          products="gecko",
                          animatable=False)}

// Section 15 - Filter Effects

${helpers.predefined_type(
    "flood-color", "CSSColor",
    "CSSParserColor::RGBA(RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 })",
    products="gecko",
    animatable=False)}

${helpers.predefined_type("flood-opacity", "Opacity",
                          "1.0", products="gecko", animatable=False)}

${helpers.predefined_type(
    "lighting-color", "CSSColor",
    "CSSParserColor::RGBA(RGBA { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 })",
    products="gecko",
    animatable=False)}

// CSS Masking Module Level 1
// https://www.w3.org/TR/css-masking-1/
${helpers.single_keyword("mask-type", "luminance alpha",
                         products="gecko", animatable=False)}

<%helpers:longhand name="clip-path" animatable="False" products="gecko">
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
                         animatable=False)}

// TODO implement all of repeat-style for background and mask
// https://drafts.csswg.org/css-backgrounds-3/#repeat-style
${helpers.single_keyword("mask-repeat",
                         "repeat repeat-x repeat-y space round no-repeat",
                         vector=True,
                         products="gecko",
                         animatable=False)}

<%helpers:longhand name="mask-position" products="gecko" animatable="True">
    use properties::longhands::background_position;
    pub use ::properties::longhands::background_position::SpecifiedValue;
    pub use ::properties::longhands::background_position::single_value as single_value;
    pub use ::properties::longhands::background_position::computed_value as computed_value;

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        background_position::get_initial_value()
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        background_position::parse(context, input)
    }
</%helpers:longhand>

// missing: margin-box fill-box stroke-box view-box no-clip
// (gecko doesn't implement these)
${helpers.single_keyword("mask-clip",
                         "content-box padding-box border-box",
                         vector=True,
                         products="gecko",
                         animatable=False)}

// missing: margin-box fill-box stroke-box view-box
// (gecko doesn't implement these)
${helpers.single_keyword("mask-origin",
                         "content-box padding-box border-box",
                         vector=True,
                         products="gecko",
                         animatable=False)}

<%helpers:longhand name="mask-size" products="gecko" animatable="True">
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
                         animatable=False)}

<%helpers:vector_longhand name="mask-image" products="gecko" animatable="False"
                          has_uncacheable_values="${product == 'gecko'}">
    use std::fmt;
    use style_traits::ToCss;
    use url::Url;
    use values::specified::{Image, UrlExtraData};
    use values::NoViewportPercentage;

    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;
        use url::Url;
        use values::computed;
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            Image(computed::Image),
            Url(Url, computed::UrlExtraData),
            None
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    T::None => dest.write_str("none"),
                    T::Image(ref image) => image.to_css(dest),
                    T::Url(ref url, _) => url.to_css(dest),
                }
            }
        }
    }

    impl NoViewportPercentage for SpecifiedValue {}

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Image(Image),
        Url(Url, UrlExtraData),
        None
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Image(ref image) => image.to_css(dest),
                SpecifiedValue::Url(ref url, _) => url.to_css(dest),
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
                Image::Url(url, data) => {
                    if url.fragment().is_some() {
                        Ok(SpecifiedValue::Url(url, data))
                    } else {
                        Ok(SpecifiedValue::Image(Image::Url(url, data)))
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
                SpecifiedValue::Url(ref url, ref data) =>
                    computed_value::T::Url(url.clone(), data.clone()),
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T::None => SpecifiedValue::None,
                computed_value::T::Image(ref image) =>
                    SpecifiedValue::Image(ToComputedValue::from_computed_value(image)),
                computed_value::T::Url(ref url, ref data) =>
                    SpecifiedValue::Url(url.clone(), data.clone()),
            }
        }
    }
</%helpers:vector_longhand>
