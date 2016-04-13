<%page args="helpers"/>

<%helpers:longhand name="line-height">
    use cssparser::ToCss;
    use std::fmt;
    use values::AuExtensionMethods;
    use values::CSSFloat;

    #[derive(Debug, Clone, PartialEq, Copy, HeapSizeOf)]
    pub enum SpecifiedValue {
        Normal,
        Number(CSSFloat),
        LengthOrPercentage(specified::LengthOrPercentage),
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Normal => dest.write_str("normal"),
                SpecifiedValue::LengthOrPercentage(value) => value.to_css(dest),
                SpecifiedValue::Number(number) => write!(dest, "{}", number),
            }
        }
    }
    /// normal | <number> | <length> | <percentage>
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use cssparser::Token;
        use std::ascii::AsciiExt;
        input.try(specified::LengthOrPercentage::parse_non_negative)
        .map(SpecifiedValue::LengthOrPercentage)
        .or_else(|()| {
            match try!(input.next()) {
                Token::Number(ref value) if value.value >= 0. => {
                    Ok(SpecifiedValue::Number(value.value))
                }
                Token::Ident(ref value) if value.eq_ignore_ascii_case("normal") => {
                    Ok(SpecifiedValue::Normal)
                }
                _ => Err(()),
            }
        })
    }
    pub mod computed_value {
        use app_units::Au;
        use std::fmt;
        use values::CSSFloat;
        #[derive(PartialEq, Copy, Clone, HeapSizeOf, Debug)]
        pub enum T {
            Normal,
            Length(Au),
            Number(CSSFloat),
        }
    }
    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                computed_value::T::Normal => dest.write_str("normal"),
                computed_value::T::Length(length) => length.to_css(dest),
                computed_value::T::Number(number) => write!(dest, "{}", number),
            }
        }
    }
     #[inline]
    pub fn get_initial_value() -> computed_value::T { computed_value::T::Normal }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
            match *self {
                SpecifiedValue::Normal => computed_value::T::Normal,
                SpecifiedValue::Number(value) => computed_value::T::Number(value),
                SpecifiedValue::LengthOrPercentage(value) => {
                    match value {
                        specified::LengthOrPercentage::Length(value) =>
                            computed_value::T::Length(value.to_computed_value(context)),
                        specified::LengthOrPercentage::Percentage(specified::Percentage(value)) => {
                            let fr = specified::Length::FontRelative(specified::FontRelativeLength::Em(value));
                            computed_value::T::Length(fr.to_computed_value(context))
                        },
                        specified::LengthOrPercentage::Calc(calc) => {
                            let calc = calc.to_computed_value(context);
                            let fr = specified::FontRelativeLength::Em(calc.percentage());
                            let fr = specified::Length::FontRelative(fr);
                            computed_value::T::Length(calc.length() + fr.to_computed_value(context))
                        }
                    }
                }
            }
        }
    }
</%helpers:longhand>
