/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

// CSS Basic User Interface Module Level 1
// https://drafts.csswg.org/css-ui-3/
<% data.new_style_struct("UI", inherited=False, gecko_name="UIReset") %>

// TODO spec says that UAs should not support this
// we should probably remove from gecko (https://bugzilla.mozilla.org/show_bug.cgi?id=1328331)
${helpers.single_keyword("ime-mode", "normal auto active disabled inactive",
                         products="gecko", gecko_ffi_name="mIMEMode",
                         animatable=False,
                         spec="https://drafts.csswg.org/css-ui/#input-method-editor")}

${helpers.single_keyword("-moz-user-select", "auto text none all", products="gecko",
                         alias="-webkit-user-select",
                         gecko_ffi_name="mUserSelect",
                         gecko_enum_prefix="StyleUserSelect",
                         gecko_inexhaustive=True,
                         animatable=False,
                         spec="https://drafts.csswg.org/css-ui-4/#propdef-user-select")}

<%helpers:longhand name="caret-color" animatable="False"
                   spec="https://drafts.csswg.org/css-ui/#caret-color">
    use cssparser::Color;
    use std::fmt;
    use style_traits::ToCss;
    use values::NoViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    impl NoViewportPercentage for SpecifiedValue {}
    impl ComputedValueAsSpecified for SpecifiedValue {}

    pub mod computed_value {
        use cssparser::Color;
        use values::{Auto, Either};

        #[derive(PartialEq, Clone, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T (pub Either<Color, Auto>);
    }
    
    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
           
           match *self {
               computed_value::T(Either::First(color)) => {
                    color.to_css(dest)?;
               },
               computed_value::T(Either::Second(Auto)) => {
                    dest.write_str("auto")?;
               }
           };
           
           Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T { 
        computed_value::T(Either::Second(Auto))
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {

        if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
            return Ok(computed_value::T(Either::Second(Auto)));
        }
        
        if let Ok(color) = Color::parse(input) {
            return Ok(computed_value::T(Either::First(color)));
        }

        Err(())
    }
</%helpers:longhand>