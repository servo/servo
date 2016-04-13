<%page args="helpers"/>

% for side in ["top", "right", "bottom", "left"]:
    <%helpers:longhand name="border-${side}-width">
        use app_units::Au;
        use cssparser::ToCss;
        use std::fmt;

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                self.0.to_css(dest)
            }
        }

        #[inline]
        pub fn parse(_context: &ParserContext, input: &mut Parser)
                               -> Result<SpecifiedValue, ()> {
            specified::parse_border_width(input).map(SpecifiedValue)
        }
        #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
        pub struct SpecifiedValue(pub specified::Length);
        pub mod computed_value {
            use app_units::Au;
            pub type T = Au;
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            Au::from_px(3)  // medium
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
                self.0.to_computed_value(context)
            }
        }
    </%helpers:longhand>
% endfor
