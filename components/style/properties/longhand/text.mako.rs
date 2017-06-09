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

<%helpers:longhand name="text-overflow" animation_value_type="none" boxed="True"
                   spec="https://drafts.csswg.org/css-ui/#propdef-text-overflow">
    use std::fmt;
    use style_traits::ToCss;

    no_viewport_percentage!(SpecifiedValue);

    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    #[derive(Clone, Debug, Eq, PartialEq, ToCss)]
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
        pub use super::Side;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T {
            // When the specified value only has one side, that's the "second"
            // side, and the sides are logical, so "second" means "end".  The
            // start side is Clip in that case.
            //
            // When the specified value has two sides, those are our "first"
            // and "second" sides, and they are physical sides ("left" and
            // "right").
            pub first: Side,
            pub second: Side,
            pub sides_are_logical: bool
        }
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.sides_are_logical {
                assert!(self.first == Side::Clip);
                try!(self.second.to_css(dest));
            } else {
                try!(self.first.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.second.to_css(dest));
            }
            Ok(())
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, _context: &Context) -> Self::ComputedValue {
            if let Some(ref second) = self.second {
                Self::ComputedValue { first: self.first.clone(),
                                      second: second.clone(),
                                      sides_are_logical: false }
            } else {
                Self::ComputedValue { first: Side::Clip,
                                      second: self.first.clone(),
                                      sides_are_logical: true }
            }
        }

        #[inline]
        fn from_computed_value(computed: &Self::ComputedValue) -> Self {
            if computed.sides_are_logical {
                assert!(computed.first == Side::Clip);
                SpecifiedValue { first: computed.second.clone(),
                                 second: None }
            } else {
                SpecifiedValue { first: computed.first.clone(),
                                 second: Some(computed.second.clone()) }
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T {
            first: Side::Clip,
            second: Side::Clip,
            sides_are_logical: true,
        }
    }
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let first = try!(Side::parse(context, input));
        let second = input.try(|input| Side::parse(context, input)).ok();
        Ok(SpecifiedValue {
            first: first,
            second: second,
        })
    }
    impl Parse for Side {
        fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Side, ()> {
            if let Ok(ident) = input.try(|input| input.expect_ident()) {
                match_ignore_ascii_case! { &ident,
                    "clip" => Ok(Side::Clip),
                    "ellipsis" => Ok(Side::Ellipsis),
                    _ => Err(())
                }
            } else {
                Ok(Side::String(try!(input.expect_string()).into_owned().into_boxed_str()))
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
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-writing-modes/#propdef-unicode-bidi")}

<%helpers:longhand name="text-decoration-line"
                   custom_cascade="${product == 'servo'}"
                   animation_value_type="discrete"
                   spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-line">
    use std::fmt;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    bitflags! {
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub flags SpecifiedValue: u8 {
            const NONE = 0,
            const UNDERLINE = 0x01,
            const OVERLINE = 0x02,
            const LINE_THROUGH = 0x04,
            const BLINK = 0x08,
        % if product == "gecko":
            /// Only set by presentation attributes
            ///
            /// Setting this will mean that text-decorations use the color
            /// specified by `color` in quirks mode.
            ///
            /// For example, this gives <a href=foo><font color="red">text</font></a>
            /// a red text decoration
            const COLOR_OVERRIDE = 0x10,
        % endif
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let mut has_any = false;

            macro_rules! write_value {
                ($line:ident => $css:expr) => {
                    if self.contains($line) {
                        if has_any {
                            dest.write_str(" ")?;
                        }
                        dest.write_str($css)?;
                        has_any = true;
                    }
                }
            }
            write_value!(UNDERLINE => "underline");
            write_value!(OVERLINE => "overline");
            write_value!(LINE_THROUGH => "line-through");
            write_value!(BLINK => "blink");
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
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut result = SpecifiedValue::empty();
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(result)
        }
        let mut empty = true;

        while input.try(|input| {
                if let Ok(ident) = input.expect_ident() {
                    match_ignore_ascii_case! { &ident,
                        "underline" => if result.contains(UNDERLINE) { return Err(()) }
                                       else { empty = false; result.insert(UNDERLINE) },
                        "overline" => if result.contains(OVERLINE) { return Err(()) }
                                      else { empty = false; result.insert(OVERLINE) },
                        "line-through" => if result.contains(LINE_THROUGH) { return Err(()) }
                                          else { empty = false; result.insert(LINE_THROUGH) },
                        "blink" => if result.contains(BLINK) { return Err(()) }
                                   else { empty = false; result.insert(BLINK) },
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
                                   _cacheable: &mut bool,
                                   _error_reporter: &ParseErrorReporter) {
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
                         spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-style")}

${helpers.predefined_type(
    "text-decoration-color", "Color",
    "computed_value::T::currentcolor()",
    initial_specified_value="specified::Color::currentcolor()",
    products="gecko",
    animation_value_type="IntermediateColor",
    ignored_when_colors_disabled=True,
    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-color")}

${helpers.predefined_type(
    "initial-letter",
    "InitialLetter",
    "computed::InitialLetter::normal()",
    initial_specified_value="specified::InitialLetter::normal()",
    animation_value_type="discrete",
    products="gecko",
    spec="https://drafts.csswg.org/css-inline/#sizing-drop-initials")}
