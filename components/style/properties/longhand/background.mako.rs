/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Background", inherited=False) %>

${helpers.predefined_type("background-color", "CSSColor",
    "::cssparser::Color::RGBA(::cssparser::RGBA { red: 0., green: 0., blue: 0., alpha: 0. }) /* transparent */",
    animatable=True)}

<%helpers:vector_longhand gecko_only="True" name="background-image" animatable="False">
    use cssparser::ToCss;
    use std::fmt;
    use values::specified::Image;
    use values::LocalToCss;
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
                Some(computed::Image::Url(ref url, ref _extra_data)) => url.to_css(dest),
                Some(computed::Image::LinearGradient(ref gradient)) =>
                    gradient.to_css(dest)
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
    }
</%helpers:vector_longhand>

<%helpers:longhand name="background-position" animatable="True">
        use cssparser::ToCss;
        use std::fmt;
        use values::LocalToCss;
        use values::HasViewportPercentage;
        use values::specified::position::Position;

        pub mod computed_value {
            use values::computed::position::Position;

            pub type T = Position;
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

        pub fn parse(_context: &ParserContext, input: &mut Parser)
                     -> Result<SpecifiedValue, ()> {
            Ok(try!(Position::parse(input)))
        }
</%helpers:longhand>

${helpers.single_keyword("background-repeat",
                         "repeat repeat-x repeat-y no-repeat",
                         animatable=False)}

${helpers.single_keyword("background-attachment",
                         "scroll fixed" + (" local" if product == "gecko" else ""),
                         animatable=False)}

${helpers.single_keyword("background-clip",
                         "border-box padding-box content-box",
                         animatable=False)}

${helpers.single_keyword("background-origin",
                         "padding-box border-box content-box",
                         animatable=False)}

<%helpers:longhand name="background-size" animatable="True">
    use cssparser::{ToCss, Token};
    use std::ascii::AsciiExt;
    use std::fmt;
    use values::HasViewportPercentage;

    pub mod computed_value {
        use values::computed::LengthOrPercentageOrAuto;

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

    impl HasViewportPercentage for SpecifiedExplicitSize {
        fn has_viewport_percentage(&self) -> bool {
            return self.width.has_viewport_percentage() || self.height.has_viewport_percentage();
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedExplicitSize {
        pub width: specified::LengthOrPercentageOrAuto,
        pub height: specified::LengthOrPercentageOrAuto,
    }

    impl ToCss for SpecifiedExplicitSize {
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
        Explicit(SpecifiedExplicitSize),
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
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::Explicit(computed_value::ExplicitSize {
            width: computed::LengthOrPercentageOrAuto::Auto,
            height: computed::LengthOrPercentageOrAuto::Auto,
        })
    }

    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
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
            width = try!(specified::LengthOrPercentageOrAuto::parse(input))
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
            height = try!(specified::LengthOrPercentageOrAuto::parse(input));
        }

        Ok(SpecifiedValue::Explicit(SpecifiedExplicitSize {
            width: width,
            height: height,
        }))
    }
</%helpers:longhand>
