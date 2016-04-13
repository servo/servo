<%page args="helpers"/>

<%helpers:longhand name="font-size">
    use app_units::Au;
    use cssparser::ToCss;
    use std::fmt;
    use values::FONT_MEDIUM_PX;
    use values::specified::{LengthOrPercentage, Length, Percentage};

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }

    #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
    pub struct SpecifiedValue(pub specified::LengthOrPercentage);
    pub mod computed_value {
        use app_units::Au;
        pub type T = Au;
    }
    #[inline] pub fn get_initial_value() -> computed_value::T {
        Au::from_px(FONT_MEDIUM_PX)
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
            match self.0 {
                LengthOrPercentage::Length(Length::FontRelative(value)) => {
                    value.to_computed_value(context.inherited_style().get_font().clone_font_size(),
                                            context.style().root_font_size())
                }
                LengthOrPercentage::Length(Length::ServoCharacterWidth(value)) => {
                    value.to_computed_value(context.inherited_style().get_font().clone_font_size())
                }
                LengthOrPercentage::Length(l) => {
                    l.to_computed_value(context)
                }
                LengthOrPercentage::Percentage(Percentage(value)) => {
                    context.inherited_style().get_font().clone_font_size().scale_by(value)
                }
                LengthOrPercentage::Calc(calc) => {
                    let calc = calc.to_computed_value(context);
                    calc.length() + context.inherited_style().get_font().clone_font_size()
                                           .scale_by(calc.percentage())
                }
            }
        }
    }
    /// <length> | <percentage> | <absolute-size> | <relative-size>
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use values::specified::{Length, LengthOrPercentage};

        input.try(specified::LengthOrPercentage::parse_non_negative)
        .or_else(|()| {
            let ident = try!(input.expect_ident());
            specified::Length::from_str(&ident as &str)
                .ok_or(())
                .map(specified::LengthOrPercentage::Length)
        })
        .map(SpecifiedValue)
    }
</%helpers:longhand>
