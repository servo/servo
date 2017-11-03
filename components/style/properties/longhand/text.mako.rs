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

${helpers.predefined_type("text-overflow",
                          "TextOverflow",
                          "computed::TextOverflow::get_initial_value()",
                          animation_value_type="discrete",
                          boxed=True,
                          flags="APPLIES_TO_PLACEHOLDER",
                          spec="https://drafts.csswg.org/css-ui/#propdef-text-overflow")}

${helpers.single_keyword("unicode-bidi",
                         "normal embed isolate bidi-override isolate-override plaintext",
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-writing-modes/#propdef-unicode-bidi")}

<%helpers:longhand name="text-decoration-line"
                   custom_cascade="${product == 'servo'}"
                   animation_value_type="discrete"
                   flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                   spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-line">
    use std::fmt;
    use style_traits::ToCss;

    bitflags! {
        #[derive(MallocSizeOf, ToComputedValue)]
        pub struct SpecifiedValue: u8 {
            const NONE = 0;
            const UNDERLINE = 0x01;
            const OVERLINE = 0x02;
            const LINE_THROUGH = 0x04;
            const BLINK = 0x08;
        % if product == "gecko":
            /// Only set by presentation attributes
            ///
            /// Setting this will mean that text-decorations use the color
            /// specified by `color` in quirks mode.
            ///
            /// For example, this gives <a href=foo><font color="red">text</font></a>
            /// a red text decoration
            const COLOR_OVERRIDE = 0x10;
        % endif
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let mut has_any = false;

            macro_rules! write_value {
                ($line:path => $css:expr) => {
                    if self.contains($line) {
                        if has_any {
                            dest.write_str(" ")?;
                        }
                        dest.write_str($css)?;
                        has_any = true;
                    }
                }
            }
            write_value!(SpecifiedValue::UNDERLINE => "underline");
            write_value!(SpecifiedValue::OVERLINE => "overline");
            write_value!(SpecifiedValue::LINE_THROUGH => "line-through");
            write_value!(SpecifiedValue::BLINK => "blink");
            if !has_any {
                dest.write_str("none")?;
            }

            Ok(())
        }
    }
    pub mod computed_value {
        pub type T = super::SpecifiedValue;
        #[allow(non_upper_case_globals)]
        pub const none: T = super::SpecifiedValue {
            bits: 0
        };
    }
    #[inline] pub fn get_initial_value() -> computed_value::T {
        computed_value::none
    }
    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::empty()
    }
    /// none | [ underline || overline || line-through || blink ]
    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        let mut result = SpecifiedValue::empty();
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(result)
        }
        let mut empty = true;

        loop {
            let result: Result<_, ParseError> = input.try(|input| {
                let location = input.current_source_location();
                match input.expect_ident() {
                    Ok(ident) => {
                        (match_ignore_ascii_case! { &ident,
                            "underline" => if result.contains(SpecifiedValue::UNDERLINE) { Err(()) }
                                           else { empty = false; result.insert(SpecifiedValue::UNDERLINE); Ok(()) },
                            "overline" => if result.contains(SpecifiedValue::OVERLINE) { Err(()) }
                                          else { empty = false; result.insert(SpecifiedValue::OVERLINE); Ok(()) },
                            "line-through" => if result.contains(SpecifiedValue::LINE_THROUGH) { Err(()) }
                                              else {
                                                  empty = false;
                                                  result.insert(SpecifiedValue::LINE_THROUGH); Ok(())
                                              },
                            "blink" => if result.contains(SpecifiedValue::BLINK) { Err(()) }
                                       else { empty = false; result.insert(SpecifiedValue::BLINK); Ok(()) },
                            _ => Err(())
                        }).map_err(|()| {
                            location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone()))
                        })
                    }
                    Err(e) => return Err(e.into())
                }
            });
            if result.is_err() {
                break;
            }
        }

        if !empty { Ok(result) } else { Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)) }
    }

    % if product == "servo":
        fn cascade_property_custom(_declaration: &PropertyDeclaration,
                                   context: &mut computed::Context) {
            longhands::_servo_text_decorations_in_effect::derive_from_text_decoration(context);
        }
    % endif

    #[cfg(feature = "gecko")]
    impl_bitflags_conversions!(SpecifiedValue);
</%helpers:longhand>

${helpers.single_keyword("text-decoration-style",
                         "solid double dotted dashed wavy -moz-none",
                         products="gecko",
                         animation_value_type="discrete",
                         flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                         spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-style")}

${helpers.predefined_type(
    "text-decoration-color",
    "Color",
    "computed_value::T::currentcolor()",
    initial_specified_value="specified::Color::currentcolor()",
    products="gecko",
    animation_value_type="AnimatedColor",
    ignored_when_colors_disabled=True,
    flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-color",
)}

${helpers.predefined_type(
    "initial-letter",
    "InitialLetter",
    "computed::InitialLetter::normal()",
    initial_specified_value="specified::InitialLetter::normal()",
    animation_value_type="discrete",
    products="gecko",
    flags="APPLIES_TO_FIRST_LETTER",
    spec="https://drafts.csswg.org/css-inline/#sizing-drop-initials")}
