/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Background", inherited=False) %>

${helpers.predefined_type("background-color", "CSSColor",
    "::cssparser::Color::RGBA(::cssparser::RGBA { red: 0., green: 0., blue: 0., alpha: 0. }) /* transparent */",
    animatable=True)}

<%helpers:vector_longhand name="background-image" animatable="False"
                          has_uncacheable_values="${product == 'gecko'}">
    use std::fmt;
    use style_traits::ToCss;
    use values::specified::Image;
    use values::NoViewportPercentage;

    pub mod computed_value {
        use values::computed;
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Option<computed::Image>);
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self.0 {
                None => dest.write_str("none"),
                Some(ref image) => image.to_css(dest),
            }
        }
    }

    impl NoViewportPercentage for SpecifiedValue {}

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub Option<Image>);

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue(Some(ref image)) => image.to_css(dest),
                SpecifiedValue(None) => dest.write_str("none"),
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(None)
    }
    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue(None)
    }
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            Ok(SpecifiedValue(None))
        } else {
            Ok(SpecifiedValue(Some(try!(Image::parse(context, input)))))
        }
    }
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            match *self {
                SpecifiedValue(None) => computed_value::T(None),
                SpecifiedValue(Some(ref image)) =>
                    computed_value::T(Some(image.to_computed_value(context))),
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T(None) => SpecifiedValue(None),
                computed_value::T(Some(ref image)) =>
                    SpecifiedValue(Some(ToComputedValue::from_computed_value(image))),
            }
        }
    }
</%helpers:vector_longhand>

<%helpers:vector_longhand name="background-position-x" animatable="True">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::specified::position::HorizontalPosition;

    #[allow(missing_docs)]
    pub mod computed_value {
        use values::computed::position::HorizontalPosition;
        use properties::animated_properties::{Interpolate, RepeatableListInterpolate};

        pub type T = HorizontalPosition;
    }

    #[allow(missing_docs)]
    pub type SpecifiedValue = HorizontalPosition;

    #[inline]
    #[allow(missing_docs)]
    pub fn get_initial_value() -> computed_value::T {
        use values::computed::position::HorizontalPosition;
        HorizontalPosition(computed::LengthOrPercentage::Percentage(0.0))
    }
    #[inline]
    #[allow(missing_docs)]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        use values::specified::position::Keyword;
        HorizontalPosition {
            keyword: Some(Keyword::Left),
            position: None,
        }
    }
    #[inline]
    #[allow(missing_docs)]
    pub fn get_initial_position_value() -> SpecifiedValue {
        use values::specified::{LengthOrPercentage, Percentage};
        HorizontalPosition {
            keyword: None,
            position: Some(LengthOrPercentage::Percentage(Percentage(0.0))),
        }
    }

    #[allow(missing_docs)]
    pub fn parse(context: &ParserContext, input: &mut Parser)
                 -> Result<SpecifiedValue, ()> {
        HorizontalPosition::parse(context, input)
    }
</%helpers:vector_longhand>

<%helpers:vector_longhand name="background-position-y" animatable="True">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::specified::position::VerticalPosition;

    #[allow(missing_docs)]
    pub mod computed_value {
        use values::computed::position::VerticalPosition;
        use properties::animated_properties::{Interpolate, RepeatableListInterpolate};

        pub type T = VerticalPosition;
    }

    #[allow(missing_docs)]
    pub type SpecifiedValue = VerticalPosition;

    #[inline]
    #[allow(missing_docs)]
    pub fn get_initial_value() -> computed_value::T {
        use values::computed::position::VerticalPosition;
        VerticalPosition(computed::LengthOrPercentage::Percentage(0.0))
    }
    #[inline]
    #[allow(missing_docs)]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        use values::specified::position::Keyword;
        VerticalPosition {
            keyword: Some(Keyword::Top),
            position: None,
        }
    }
    #[inline]
    #[allow(missing_docs)]
    pub fn get_initial_position_value() -> SpecifiedValue {
        use values::specified::{LengthOrPercentage, Percentage};
        VerticalPosition {
            keyword: None,
            position: Some(LengthOrPercentage::Percentage(Percentage(0.0))),
        }
    }

    #[inline]
    #[allow(missing_docs)]
    pub fn parse(context: &ParserContext, input: &mut Parser)
                 -> Result<SpecifiedValue, ()> {
        VerticalPosition::parse(context, input)
    }
</%helpers:vector_longhand>

${helpers.single_keyword("background-repeat",
                         "repeat repeat-x repeat-y space round no-repeat",
                         vector=True,
                         animatable=False)}

${helpers.single_keyword("background-attachment",
                         "scroll fixed" + (" local" if product == "gecko" else ""),
                         vector=True,
                         animatable=False)}

${helpers.single_keyword("background-clip",
                         "border-box padding-box content-box",
                         vector=True,
                         animatable=False)}

${helpers.single_keyword("background-origin",
                         "padding-box border-box content-box",
                         vector=True,
                         animatable=False)}

<%helpers:vector_longhand name="background-size" animatable="True">
    use cssparser::Token;
    use std::ascii::AsciiExt;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    #[allow(missing_docs)]
    pub mod computed_value {
        use values::computed::LengthOrPercentageOrAuto;
        use properties::animated_properties::{Interpolate, RepeatableListInterpolate};

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

        impl RepeatableListInterpolate for T {}

        impl Interpolate for T {
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
        let width;
        if let Ok(value) = input.try(|input| {
            match input.next() {
                Err(_) => Err(()),
                Ok(Token::Ident(ref ident)) if ident.eq_ignore_ascii_case("cover") => {
                    Ok(SpecifiedValue::Cover)
                }
                Ok(Token::Ident(ref ident)) if ident.eq_ignore_ascii_case("contain") => {
                    Ok(SpecifiedValue::Contain)
                }
                Ok(_) => Err(()),
            }
        }) {
            return Ok(value)
        } else {
            width = try!(specified::LengthOrPercentageOrAuto::parse(context, input))
        }

        let height;
        if let Ok(value) = input.try(|input| {
            match input.next() {
                Err(_) => Ok(specified::LengthOrPercentageOrAuto::Auto),
                Ok(_) => Err(()),
            }
        }) {
            height = value
        } else {
            height = try!(specified::LengthOrPercentageOrAuto::parse(context, input));
        }

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
                         vector="true", products="gecko", animatable=False)}
