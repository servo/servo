/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Outline",
                         inherited=False,
                         additional_methods=[Method("outline_has_nonzero_width", "bool")]) %>

// TODO(pcwalton): `invert`
${helpers.predefined_type("outline-color", "CSSColor", "::cssparser::Color::CurrentColor",
                          animatable=True, complex_color=True, need_clone=True)}

<%helpers:longhand name="outline-style" need_clone="True" animatable="False">
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

<%helpers:longhand name="outline-width" animatable="True">
    use app_units::Au;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        specified::parse_border_width(input).map(SpecifiedValue)
    }

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            let &SpecifiedValue(length) = self;
            length.has_viewport_percentage()
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub specified::Length);
    pub mod computed_value {
        use app_units::Au;
        pub type T = Au;
    }
    pub use super::border_top_width::get_initial_value;
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            self.0.to_computed_value(context)
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue(ToComputedValue::from_computed_value(computed))
        }
    }
</%helpers:longhand>

// The -moz-outline-radius-* properties are non-standard and not on a standards track.
// TODO: Should they animate?
% for corner in ["topleft", "topright", "bottomright", "bottomleft"]:
    ${helpers.predefined_type("-moz-outline-radius-" + corner, "BorderRadiusSize",
                              "computed::BorderRadiusSize::zero()",
                              "parse", products="gecko",
                              animatable=False)}
% endfor

${helpers.predefined_type("outline-offset", "Length", "Au(0)", products="servo", animatable=True)}
