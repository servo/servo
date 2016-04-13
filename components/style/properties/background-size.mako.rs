<%page args="helpers"/>

<%helpers:longhand name="background-size">
    use cssparser::{ToCss, Token};
    use std::ascii::AsciiExt;
    use std::fmt;

    pub mod computed_value {
        use values::computed::LengthOrPercentageOrAuto;

        #[derive(PartialEq, Clone, Debug, HeapSizeOf)]
        pub struct ExplicitSize {
            pub width: LengthOrPercentageOrAuto,
            pub height: LengthOrPercentageOrAuto,
        }

        #[derive(PartialEq, Clone, Debug, HeapSizeOf)]
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

    #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
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


    #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
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
        fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
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
