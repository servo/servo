/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Color", inherited=True) %>

<%helpers:raw_longhand name="color" need_clone="True" animatable="True"
                       spec="https://drafts.csswg.org/css-color/#color">
    use cssparser::Color as CSSParserColor;
    use cssparser::RGBA;
    use values::specified::{CSSColor, CSSRGBA};

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, _context: &Context) -> computed_value::T {
            self.parsed
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            CSSRGBA {
                parsed: *computed,
                authored: None,
            }
        }
    }

    pub type SpecifiedValue = CSSRGBA;
    pub mod computed_value {
        use cssparser;
        pub type T = cssparser::RGBA;
    }
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        RGBA { red: 0., green: 0., blue: 0., alpha: 1. }  /* black */
    }
    pub fn parse_specified(context: &ParserContext, input: &mut Parser)
                           -> Result<DeclaredValue<SpecifiedValue>, ()> {
        let value = try!(CSSColor::parse(context, input));
        let rgba = match value.parsed {
            CSSParserColor::RGBA(rgba) => rgba,
            CSSParserColor::CurrentColor => return Ok(DeclaredValue::Inherit)
        };
        Ok(DeclaredValue::Value(CSSRGBA {
            parsed: rgba,
            authored: value.authored,
        }))
    }
</%helpers:raw_longhand>
