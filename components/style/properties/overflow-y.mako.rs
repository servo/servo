<%page args="helpers"/>

// FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
<%helpers:longhand name="overflow-y">
    use super::overflow_x;

    use cssparser::ToCss;
    use std::fmt;

    pub use self::computed_value::T as SpecifiedValue;

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }

    pub mod computed_value {
        #[derive(Debug, Clone, Copy, PartialEq, HeapSizeOf)]
        pub struct T(pub super::super::overflow_x::computed_value::T);
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
            computed_value::T(self.0.to_computed_value(context))
        }
    }

    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(overflow_x::get_initial_value())
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        overflow_x::parse(context, input).map(SpecifiedValue)
    }
</%helpers:longhand>
