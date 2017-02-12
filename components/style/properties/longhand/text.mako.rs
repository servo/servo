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

<%helpers:longhand name="text-overflow" animatable="False" boxed="True"
                   spec="https://drafts.csswg.org/css-ui/#propdef-text-overflow">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    use cssparser;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    #[derive(PartialEq, Eq, Clone, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum Side {
        Clip,
        Ellipsis,
        String(Box<str>),
    }

    #[derive(PartialEq, Eq, Clone, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        pub first: Side,
        pub second: Option<Side>
    }

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        SpecifiedValue {
            first: Side::Clip,
            second: None
        }
    }
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let first = try!(Side::parse(context, input));
        let second = Side::parse(context, input).ok();
        Ok(SpecifiedValue {
            first: first,
            second: second,
        })
    }
    impl Parse for Side {
        fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Side, ()> {
            if let Ok(ident) = input.try(|input| input.expect_ident()) {
                match_ignore_ascii_case! { ident,
                    "clip" => Ok(Side::Clip),
                    "ellipsis" => Ok(Side::Ellipsis),
                    _ => Err(())
                }
            } else {
                Ok(Side::String(try!(input.expect_string()).into_owned().into_boxed_str()))
            }
        }
    }

    impl ToCss for Side {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                Side::Clip => dest.write_str("clip"),
                Side::Ellipsis => dest.write_str("ellipsis"),
                Side::String(ref s) => {
                    cssparser::serialize_string(s, dest)
                }
            }
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.first.to_css(dest));
            if let Some(ref second) = self.second {
                try!(dest.write_str(" "));
                try!(second.to_css(dest));
            }
            Ok(())
        }
    }
</%helpers:longhand>

${helpers.single_keyword("unicode-bidi",
                         "normal embed isolate bidi-override isolate-override plaintext",
                         animatable=False,
                         spec="https://drafts.csswg.org/css-writing-modes/#propdef-unicode-bidi")}

// FIXME: This prop should be animatable.
<%helpers:longhand name="${'text-decoration' if product == 'servo' else 'text-decoration-line'}"
                   custom_cascade="${product == 'servo'}"
                   animatable="False"
                   disable_when_testing="True",
                   spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-line">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    #[derive(PartialEq, Eq, Copy, Clone, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        pub underline: bool,
        pub overline: bool,
        pub line_through: bool,
        pub blink: bool,
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let mut has_any = false;
            macro_rules! write_value {
                ($line:ident => $css:expr) => {
                    if self.$line {
                        if has_any {
                            dest.write_str(" ")?;
                        }
                        dest.write_str($css)?;
                        has_any = true;
                    }
                }
            }
            write_value!(underline => "underline");
            write_value!(overline => "overline");
            write_value!(line_through => "line-through");
            write_value!(blink => "blink");
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
            underline: false, overline: false, line_through: false, blink: false
        };
    }
    #[inline] pub fn get_initial_value() -> computed_value::T {
        computed_value::none
    }
    /// none | [ underline || overline || line-through || blink ]
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut result = SpecifiedValue {
            underline: false, overline: false, line_through: false, blink: false
        };
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(result)
        }
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
                        "blink" => if result.blink { return Err(()) }
                                   else { empty = false; result.blink = true },
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
                         animatable=False,
                         spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-style")}

${helpers.predefined_type(
    "text-decoration-color", "CSSColor",
    "CSSParserColor::RGBA(RGBA::new(0, 0, 0, 255))",
    complex_color=True,
    products="gecko",
    animatable=True,
    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-color")}
