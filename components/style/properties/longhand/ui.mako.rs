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

${helpers.single_keyword("-moz-window-dragging", "default drag no-drag", products="gecko",
                         gecko_ffi_name="mWindowDragging",
                         gecko_enum_prefix="StyleWindowDragging",
                         animatable=False,
                         spec="None (Nonstandard Firefox-only property)")}

<%helpers:longhand name="caret-color" animatable="False"
                   spec="https://drafts.csswg.org/css-ui/#caret-color">
    use cssparser::Color;
    use std::fmt;
    use style_traits::ToCss;
    use values::NoViewportPercentage;
    use values::computed::ComputedValueAsSpecified;

    impl NoViewportPercentage for SpecifiedValue {}
    impl ComputedValueAsSpecified for SpecifiedValue {}

    pub type SpecifiedValue = computed_value::T;

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
        use cssparser::RGBA;
        
        if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
            Ok(computed_value::T(Either::Second(Auto)))
        }
        else {
            Ok(computed_value::T(Either::First(Color::RGBA(RGBA {red: 0.5, green: 0.5, blue: 0.5, alpha: 0.5}))))

        }
    }
</%helpers:longhand>
