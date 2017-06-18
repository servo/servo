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
${helpers.single_keyword("ime-mode", "auto normal active disabled inactive",
                         products="gecko", gecko_ffi_name="mIMEMode",
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-ui/#input-method-editor")}

${helpers.single_keyword("-moz-user-select", "auto text none all element elements" +
                            " toggle tri-state -moz-all -moz-none -moz-text",
                         products="gecko",
                         alias="-webkit-user-select",
                         gecko_ffi_name="mUserSelect",
                         gecko_enum_prefix="StyleUserSelect",
                         animation_value_type="none",
                         spec="https://drafts.csswg.org/css-ui-4/#propdef-user-select")}

${helpers.single_keyword("-moz-window-dragging", "default drag no-drag", products="gecko",
                         gecko_ffi_name="mWindowDragging",
                         gecko_enum_prefix="StyleWindowDragging",
                         gecko_inexhaustive=True,
                         animation_value_type="discrete",
                         spec="None (Nonstandard Firefox-only property)")}

${helpers.single_keyword("-moz-window-shadow", "none default menu tooltip sheet", products="gecko",
                         gecko_ffi_name="mWindowShadow",
                         gecko_constant_prefix="NS_STYLE_WINDOW_SHADOW",
                         gecko_inexhaustive=True,
                         animation_value_type="discrete",
                         internal=True,
                         spec="None (Nonstandard internal property)")}

<%helpers:longhand name="-moz-force-broken-image-icon"
                   products="gecko"
                   animation_value_type="discrete"
                   spec="None (Nonstandard Firefox-only property)">
    use std::fmt;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;

    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        #[derive(Debug, Clone, Copy, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub bool);
    }

    pub use self::computed_value::T as SpecifiedValue;

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            dest.write_str(if self.0 { "1" } else { "0" })
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(false)
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        computed_value::T(false)
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}

    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        match input.expect_integer()? {
            0 => Ok(computed_value::T(false)),
            1 => Ok(computed_value::T(true)),
            _ => Err(StyleParseError::UnspecifiedError.into()),
        }
    }

    impl From<u8> for SpecifiedValue {
        fn from(bits: u8) -> SpecifiedValue {
            SpecifiedValue(bits == 1)
        }
    }

    impl From<SpecifiedValue> for u8 {
        fn from(v: SpecifiedValue) -> u8 {
            match v.0 {
                true => 1u8,
                false => 0u8,
            }
        }
    }
</%helpers:longhand>
