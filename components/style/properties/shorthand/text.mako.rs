/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="text-decoration"
                    sub_properties="text-decoration-color
                                    text-decoration-line
                                    text-decoration-style"
                    products="gecko"
                    disable_when_testing="True"
                    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration">
    use cssparser::Color as CSSParserColor;
    use properties::longhands::{text_decoration_color, text_decoration_line, text_decoration_style};
    use values::specified::CSSColor;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let (mut color, mut line, mut style, mut any) = (None, None, None, false);
        loop {
            macro_rules! parse_component {
                ($value:ident, $module:ident) => (
                    if $value.is_none() {
                        if let Ok(value) = input.try(|input| $module::parse(context, input)) {
                            $value = Some(value);
                            any = true;
                            continue;
                        }
                    }
                )
            }

            parse_component!(color, text_decoration_color);
            parse_component!(line, text_decoration_line);
            parse_component!(style, text_decoration_style);
            break;
        }

        if !any {
            return Err(());
        }

        Ok(Longhands {
            text_decoration_color: color.or(Some(CSSColor { parsed: CSSParserColor::CurrentColor,
                                                            authored: None })),
            text_decoration_line: line.or(Some(text_decoration_line::computed_value::none)),
            text_decoration_style: style.or(Some(text_decoration_style::computed_value::T::solid)),
        })
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self.text_decoration_line {
                DeclaredValue::Value(ref line) => {
                    try!(line.to_css(dest));
                },
                _ => {
                    try!(write!(dest, "none"));
                }
            };

            if let DeclaredValue::Value(ref style) = *self.text_decoration_style {
                try!(write!(dest, " "));
                try!(style.to_css(dest));
            }

            if let DeclaredValue::Value(ref color) = *self.text_decoration_color {
                try!(write!(dest, " "));
                try!(color.to_css(dest));
            }

            Ok(())
        }
    }
</%helpers:shorthand>
