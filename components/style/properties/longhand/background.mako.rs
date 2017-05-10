/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Background", inherited=False) %>

${helpers.predefined_type("background-color", "CSSColor",
    "::cssparser::Color::RGBA(::cssparser::RGBA::transparent())",
    initial_specified_value="SpecifiedValue::transparent()",
    spec="https://drafts.csswg.org/css-backgrounds/#background-color",
    animation_value_type="IntermediateColor", complex_color=True)}

${helpers.predefined_type("background-image", "LayerImage",
    initial_value="computed_value::T(None)",
    initial_specified_value="SpecifiedValue(None)",
    spec="https://drafts.csswg.org/css-backgrounds/#the-background-image",
    vector="True",
    animation_value_type="none",
    has_uncacheable_values="True" if product == "gecko" else "False")}

<%helpers:predefined_type name="background-position-x" type="position::HorizontalPosition"
                          initial_value="computed::position::HorizontalPosition::zero()"
                          initial_specified_value="specified::position::HorizontalPosition::left()"
                          spec="https://drafts.csswg.org/css-backgrounds-4/#propdef-background-position-x"
                          animation_value_type="ComputedValue" vector="True" delegate_animate="True">
    #[inline]
    /// Get the initial value for horizontal position.
    pub fn get_initial_position_value() -> SpecifiedValue {
        use values::generics::position::{HorizontalPosition, PositionValue};
        use values::specified::{LengthOrPercentage, Percentage};
        HorizontalPosition(PositionValue {
            keyword: None,
            position: Some(LengthOrPercentage::Percentage(Percentage(0.0))),
        })
    }
</%helpers:predefined_type>

<%helpers:predefined_type name="background-position-y" type="position::VerticalPosition"
                          initial_value="computed::position::VerticalPosition::zero()"
                          initial_specified_value="specified::position::VerticalPosition::top()"
                          spec="https://drafts.csswg.org/css-backgrounds-4/#propdef-background-position-y"
                          animation_value_type="ComputedValue" vector="True" delegate_animate="True">
    /// Get the initial value for vertical position.
    pub fn get_initial_position_value() -> SpecifiedValue {
        use values::generics::position::{VerticalPosition, PositionValue};
        use values::specified::{LengthOrPercentage, Percentage};
        VerticalPosition(PositionValue {
            keyword: None,
            position: Some(LengthOrPercentage::Percentage(Percentage(0.0))),
        })
    }
</%helpers:predefined_type>

<%helpers:vector_longhand name="background-repeat" animation_value_type="none"
                          spec="https://drafts.csswg.org/css-backgrounds/#the-background-repeat">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    define_css_keyword_enum!(RepeatKeyword:
                             "repeat" => Repeat,
                             "space" => Space,
                             "round" => Round,
                             "no-repeat" => NoRepeat);

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        RepeatX,
        RepeatY,
        Other(RepeatKeyword, Option<RepeatKeyword>),
    }

    pub mod computed_value {
        pub use super::RepeatKeyword;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub RepeatKeyword, pub RepeatKeyword);
    }

    no_viewport_percentage!(SpecifiedValue);

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match (self.0, self.1) {
                (RepeatKeyword::Repeat, RepeatKeyword::NoRepeat) => dest.write_str("repeat-x"),
                (RepeatKeyword::NoRepeat, RepeatKeyword::Repeat) => dest.write_str("repeat-y"),
                (horizontal, vertical) => {
                    try!(horizontal.to_css(dest));
                    if horizontal != vertical {
                        try!(dest.write_str(" "));
                        try!(vertical.to_css(dest));
                    }
                    Ok(())
                },
            }
        }
    }
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::RepeatX => dest.write_str("repeat-x"),
                SpecifiedValue::RepeatY => dest.write_str("repeat-y"),
                SpecifiedValue::Other(horizontal, vertical) => {
                    try!(horizontal.to_css(dest));
                    if let Some(vertical) = vertical {
                        try!(dest.write_str(" "));
                        try!(vertical.to_css(dest));
                    }
                    Ok(())
                }
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(RepeatKeyword::Repeat, RepeatKeyword::Repeat)
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Other(RepeatKeyword::Repeat, None)
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, _context: &Context) -> computed_value::T {
            match *self {
                SpecifiedValue::RepeatX =>
                    computed_value::T(RepeatKeyword::Repeat, RepeatKeyword::NoRepeat),
                SpecifiedValue::RepeatY =>
                    computed_value::T(RepeatKeyword::NoRepeat, RepeatKeyword::Repeat),
                SpecifiedValue::Other(horizontal, vertical) =>
                    computed_value::T(horizontal, vertical.unwrap_or(horizontal))
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match (computed.0, computed.1) {
                (RepeatKeyword::Repeat, RepeatKeyword::NoRepeat) => SpecifiedValue::RepeatX,
                (RepeatKeyword::NoRepeat, RepeatKeyword::Repeat) => SpecifiedValue::RepeatY,
                (horizontal, vertical) => SpecifiedValue::Other(horizontal, Some(vertical)),
            }
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let ident = input.expect_ident()?;
        match_ignore_ascii_case! { &ident,
            "repeat-x" => Ok(SpecifiedValue::RepeatX),
            "repeat-y" => Ok(SpecifiedValue::RepeatY),
            _ => {
                let horizontal = try!(RepeatKeyword::from_ident(&ident));
                let vertical = input.try(RepeatKeyword::parse).ok();
                Ok(SpecifiedValue::Other(horizontal, vertical))
            }
        }
    }
</%helpers:vector_longhand>

${helpers.single_keyword("background-attachment",
                         "scroll fixed" + (" local" if product == "gecko" else ""),
                         vector=True,
                         spec="https://drafts.csswg.org/css-backgrounds/#the-background-attachment",
                         animation_value_type="none")}

${helpers.single_keyword("background-clip",
                         "border-box padding-box content-box",
                         extra_gecko_values="text",
                         vector=True, extra_prefixes="webkit",
                         spec="https://drafts.csswg.org/css-backgrounds/#the-background-clip",
                         animation_value_type="none")}

${helpers.single_keyword("background-origin",
                         "padding-box border-box content-box",
                         vector=True, extra_prefixes="webkit",
                         spec="https://drafts.csswg.org/css-backgrounds/#the-background-origin",
                         animation_value_type="none")}

<%helpers:vector_longhand name="background-size" animation_value_type="ComputedValue" extra_prefixes="webkit"
                          spec="https://drafts.csswg.org/css-backgrounds/#the-background-size">
    use cssparser::Token;
    use std::ascii::AsciiExt;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    #[allow(missing_docs)]
    pub mod computed_value {
        use values::computed::LengthOrPercentageOrAuto;
        use properties::animated_properties::{Animatable, RepeatableListAnimatable};

        #[derive(PartialEq, Clone, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct ExplicitSize {
            pub width: LengthOrPercentageOrAuto,
            pub height: LengthOrPercentageOrAuto,
        }

        #[derive(PartialEq, Clone, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            Explicit(ExplicitSize),
            Cover,
            Contain,
        }

        impl RepeatableListAnimatable for T {}

        impl Animatable for T {
            fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
                use properties::longhands::background_size::single_value::computed_value::ExplicitSize;
                match (self, other) {
                    (&T::Explicit(ref me), &T::Explicit(ref other)) => {
                        Ok(T::Explicit(ExplicitSize {
                            width: try!(me.width.interpolate(&other.width, time)),
                            height: try!(me.height.interpolate(&other.height, time)),
                        }))
                    }
                    _ => Err(()),
                }
            }

            #[inline]
            fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
                self.compute_squared_distance(other).map(|sd| sd.sqrt())
            }

            #[inline]
            fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
                match (self, other) {
                    (&T::Explicit(ref me), &T::Explicit(ref other)) => {
                        Ok(try!(me.width.compute_squared_distance(&other.width)) +
                           try!(me.height.compute_squared_distance(&other.height)))
                    },
                    _ => Err(())
                }
            }
        }
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                computed_value::T::Explicit(ref size) => size.to_css(dest),
                computed_value::T::Cover => dest.write_str("cover"),
                computed_value::T::Contain => dest.write_str("contain"),
            }
        }
    }

    impl HasViewportPercentage for ExplicitSize {
        fn has_viewport_percentage(&self) -> bool {
            return self.width.has_viewport_percentage() || self.height.has_viewport_percentage();
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    #[allow(missing_docs)]
    pub struct ExplicitSize {
        pub width: specified::LengthOrPercentageOrAuto,
        pub height: specified::LengthOrPercentageOrAuto,
    }

    impl ToCss for ExplicitSize {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.width.to_css(dest));
            try!(dest.write_str(" "));
            self.height.to_css(dest)
        }
    }

    impl ToCss for computed_value::ExplicitSize {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.width.to_css(dest));
            try!(dest.write_str(" "));
            self.height.to_css(dest)
        }
    }

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                SpecifiedValue::Explicit(ref explicit_size) => explicit_size.has_viewport_percentage(),
                _ => false
            }
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Explicit(ExplicitSize),
        Cover,
        Contain,
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Explicit(ref size) => size.to_css(dest),
                SpecifiedValue::Cover => dest.write_str("cover"),
                SpecifiedValue::Contain => dest.write_str("contain"),
            }
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            match *self {
                SpecifiedValue::Explicit(ref size) => {
                    computed_value::T::Explicit(computed_value::ExplicitSize {
                        width: size.width.to_computed_value(context),
                        height: size.height.to_computed_value(context),
                    })
                }
                SpecifiedValue::Cover => computed_value::T::Cover,
                SpecifiedValue::Contain => computed_value::T::Contain,
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T::Explicit(ref size) => {
                    SpecifiedValue::Explicit(ExplicitSize {
                        width: ToComputedValue::from_computed_value(&size.width),
                        height: ToComputedValue::from_computed_value(&size.height),
                    })
                }
                computed_value::T::Cover => SpecifiedValue::Cover,
                computed_value::T::Contain => SpecifiedValue::Contain,
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::Explicit(computed_value::ExplicitSize {
            width: computed::LengthOrPercentageOrAuto::Auto,
            height: computed::LengthOrPercentageOrAuto::Auto,
        })
    }
    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Explicit(ExplicitSize {
            width: specified::LengthOrPercentageOrAuto::Auto,
            height: specified::LengthOrPercentageOrAuto::Auto,
        })
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        if input.try(|input| input.expect_ident_matching("cover")).is_ok() {
            return Ok(SpecifiedValue::Cover);
        }

        if input.try(|input| input.expect_ident_matching("contain")).is_ok() {
            return Ok(SpecifiedValue::Contain);
        }

        let width =
            try!(specified::LengthOrPercentageOrAuto::parse_non_negative(context, input));

        let height = input.try(|input| {
            specified::LengthOrPercentageOrAuto::parse_non_negative(context, input)
        }).unwrap_or(specified::LengthOrPercentageOrAuto::Auto);

        Ok(SpecifiedValue::Explicit(ExplicitSize {
            width: width,
            height: height,
        }))
    }
</%helpers:vector_longhand>

// https://drafts.fxtf.org/compositing/#background-blend-mode
${helpers.single_keyword("background-blend-mode",
                         """normal multiply screen overlay darken lighten color-dodge
                            color-burn hard-light soft-light difference exclusion hue
                            saturation color luminosity""",
                         vector=True, products="gecko", animation_value_type="none",
                         spec="https://drafts.fxtf.org/compositing/#background-blend-mode")}
