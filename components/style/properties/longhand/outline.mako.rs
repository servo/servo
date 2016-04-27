/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Outline",
                         inherited=False,
                         additional_methods=[Method("outline_has_nonzero_width", "bool")]) %>

// TODO(pcwalton): `invert`
${helpers.predefined_type("outline-color", "CSSColor", "::cssparser::Color::CurrentColor")}

<%helpers:longhand name="outline-style" need_clone="True">
    pub use values::specified::BorderStyle as SpecifiedValue;
    pub fn get_initial_value() -> SpecifiedValue { SpecifiedValue::none }
    pub mod computed_value {
        pub use values::specified::BorderStyle as T;
    }
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        match SpecifiedValue::parse(input) {
            Ok(SpecifiedValue::hidden) => Err(()),
            result => result
        }
    }
</%helpers:longhand>

<%helpers:longhand name="outline-width">
    use app_units::Au;
    use cssparser::ToCss;
    use std::fmt;
    use values::AuExtensionMethods;

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        specified::parse_border_width(input).map(SpecifiedValue)
    }
    #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
    pub struct SpecifiedValue(pub specified::Length);
    pub mod computed_value {
        use app_units::Au;
        pub type T = Au;
    }
    pub use super::border_top_width::get_initial_value;
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
            self.0.to_computed_value(context)
        }
    }
</%helpers:longhand>

${helpers.predefined_type("outline-offset", "Length", "Au(0)")}
