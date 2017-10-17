/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Table", inherited=False) %>

${helpers.single_keyword("table-layout", "auto fixed",
                         gecko_ffi_name="mLayoutStrategy", animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-tables/#propdef-table-layout")}

<%helpers:longhand name="-x-span" products="gecko"
                   spec="Internal-only (for `<col span>` pres attr)"
                   animation_value_type="none"
                   internal="True">
    pub type SpecifiedValue = computed_value::T;
    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;

        #[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
        pub struct T(pub i32);

        impl ToCss for T {
            fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write {
                Ok(())
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(1)
    }

    // never parse it, only set via presentation attribute
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<SpecifiedValue, ParseError<'i>> {
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
</%helpers:longhand>
