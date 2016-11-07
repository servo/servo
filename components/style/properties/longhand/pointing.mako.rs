/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Pointing", inherited=True, gecko_name="UserInterface") %>

<%helpers:longhand name="cursor" animatable="False">
    pub use self::computed_value::T as SpecifiedValue;
    use values::NoViewportPercentage;
    use values::computed::ComputedValueAsSpecified;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}

    pub mod computed_value {
        use std::fmt;
        use style_traits::cursor::Cursor;
        use style_traits::ToCss;

        #[derive(Clone, PartialEq, Eq, Copy, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            AutoCursor,
            SpecifiedCursor(Cursor),
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    T::AutoCursor => dest.write_str("auto"),
                    T::SpecifiedCursor(c) => c.to_css(dest),
                }
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::AutoCursor
    }
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use std::ascii::AsciiExt;
        use style_traits::cursor::Cursor;
        let ident = try!(input.expect_ident());
        if ident.eq_ignore_ascii_case("auto") {
            Ok(SpecifiedValue::AutoCursor)
        } else {
            Cursor::from_css_keyword(&ident)
            .map(SpecifiedValue::SpecifiedCursor)
        }
    }
</%helpers:longhand>

// NB: `pointer-events: auto` (and use of `pointer-events` in anything that isn't SVG, in fact)
// is nonstandard, slated for CSS4-UI.
// TODO(pcwalton): SVG-only values.
${helpers.single_keyword("pointer-events", "auto none", animatable=False)}

${helpers.single_keyword("-moz-user-input", "none enabled disabled",
                         products="gecko", gecko_ffi_name="mUserInput",
                         gecko_enum_prefix="StyleUserInput",
                         animatable=False)}

${helpers.single_keyword("-moz-user-modify", "read-only read-write write-only",
                         products="gecko", gecko_ffi_name="mUserModify",
                         gecko_enum_prefix="StyleUserModify",
                         animatable=False)}

${helpers.single_keyword("-moz-user-focus",
                         "ignore normal select-after select-before select-menu select-same select-all none",
                         products="gecko", gecko_ffi_name="mUserFocus",
                         gecko_enum_prefix="StyleUserFocus",
                         animatable=False)}
