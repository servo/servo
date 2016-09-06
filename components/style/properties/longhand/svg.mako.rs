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
    use cssparser::ToCss;
    use std::fmt;
    use values::LocalToCss;
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
                         "repeat repeat-x repeat-y no-repeat",
                         vector=True,
                         products="gecko",
                         animatable=False)}

<%helpers:longhand name="mask-position" products="gecko" animatable="True">
    use properties::longhands::background_position;
    pub mod computed_value {
        pub type T = ::properties::longhands::background_position::computed_value::T;
    }
    pub type SpecifiedValue = background_position::SpecifiedValue;

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
    pub mod computed_value {
        pub type T = ::properties::longhands::background_size::computed_value::T;
    }
    pub type SpecifiedValue = background_size::SpecifiedValue;

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
