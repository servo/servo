/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Text",
                         inherited=False,
                         gecko_name="TextReset",
                         additional_methods=[Method("has_underline", "bool"),
                                             Method("has_overline", "bool"),
                                             Method("has_line_through", "bool")]) %>

${helpers.single_keyword("text-overflow", "clip ellipsis", animatable=False)}

${helpers.single_keyword("unicode-bidi",
                         "normal embed isolate bidi-override isolate-override plaintext",
                         animatable=False)}

// FIXME: This prop should be animatable.
<%helpers:longhand name="${'text-decoration' if product == 'servo' else 'text-decoration-line'}"
                   custom_cascade="${product == 'servo'}"
                   animatable="False">
    use cssparser::ToCss;
    use std::fmt;
    use values::computed::ComputedValueAsSpecified;
    use values::NoViewportPercentage;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}

    #[derive(PartialEq, Eq, Copy, Clone, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        pub underline: bool,
        pub overline: bool,
        pub line_through: bool,
        // 'blink' is accepted in the parser but ignored.
        // Just not blinking the text is a conforming implementation per CSS 2.1.
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let mut space = false;
            if self.underline {
                try!(dest.write_str("underline"));
                space = true;
            }
            if self.overline {
                if space {
                    try!(dest.write_str(" "));
                }
                try!(dest.write_str("overline"));
                space = true;
            }
            if self.line_through {
                if space {
                    try!(dest.write_str(" "));
                }
                try!(dest.write_str("line-through"));
            }
            Ok(())
        }
    }
    pub mod computed_value {
        pub type T = super::SpecifiedValue;
        #[allow(non_upper_case_globals)]
        pub const none: T = super::SpecifiedValue {
            underline: false, overline: false, line_through: false
        };
    }
    #[inline] pub fn get_initial_value() -> computed_value::T {
        computed_value::none
    }
    /// none | [ underline || overline || line-through || blink ]
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut result = SpecifiedValue {
            underline: false, overline: false, line_through: false,
        };
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(result)
        }
        let mut blink = false;
        let mut empty = true;

        while input.try(|input| {
                if let Ok(ident) = input.expect_ident() {
                    match_ignore_ascii_case! { ident,
                        "underline" => if result.underline { return Err(()) }
                                       else { empty = false; result.underline = true },
                        "overline" => if result.overline { return Err(()) }
                                      else { empty = false; result.overline = true },
                        "line-through" => if result.line_through { return Err(()) }
                                          else { empty = false; result.line_through = true },
                        "blink" => if blink { return Err(()) }
                                   else { empty = false; blink = true },
                        _ => return Err(())
                    }
                } else {
                    return Err(());
                }
                Ok(())
            }).is_ok() {
        }

        if !empty { Ok(result) } else { Err(()) }
    }

    % if product == "servo":
        fn cascade_property_custom(_declaration: &PropertyDeclaration,
                                   _inherited_style: &ComputedValues,
                                   context: &mut computed::Context,
                                   _seen: &mut PropertyBitField,
                                   _cacheable: &mut bool,
                                   _error_reporter: &mut StdBox<ParseErrorReporter + Send>) {
                longhands::_servo_text_decorations_in_effect::derive_from_text_decoration(context);
        }
    % endif
</%helpers:longhand>

${helpers.single_keyword("text-decoration-style",
                         "solid double dotted dashed wavy -moz-none",
                         products="gecko",
                         animatable=False)}

${helpers.predefined_type(
    "text-decoration-color", "CSSColor",
    "CSSParserColor::RGBA(RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 })",
    products="gecko",
    animatable=True)}
