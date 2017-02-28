/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Color", inherited=True) %>

<%helpers:longhand name="color" need_clone="True" animatable="True"
                   spec="https://drafts.csswg.org/css-color/#color">
    use cssparser::Color as CSSParserColor;
    use cssparser::RGBA;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::specified::{CSSColor, CSSRGBA};

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            match self.0.parsed {
                CSSParserColor::RGBA(rgba) => rgba,
                CSSParserColor::CurrentColor => context.inherited_style.get_color().clone_color(),
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue(CSSColor {
                parsed: CSSParserColor::RGBA(*computed),
                authored: None,
            })
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub CSSColor);
    no_viewport_percentage!(SpecifiedValue);

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }

    pub mod computed_value {
        use cssparser;
        pub type T = cssparser::RGBA;
    }
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        RGBA::new(0, 0, 0, 255) // black
    }
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        CSSColor::parse(context, input).map(SpecifiedValue)
    }
</%helpers:longhand>
