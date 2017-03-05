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
            text_decoration_color: unwrap_or_initial!(text_decoration_color, color),
            text_decoration_line: unwrap_or_initial!(text_decoration_line, line),
            text_decoration_style: unwrap_or_initial!(text_decoration_style, style),
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.text_decoration_line.to_css(dest)?;
            dest.write_str(" ")?;
            self.text_decoration_style.to_css(dest)?;
            if self.text_decoration_color.parsed != CSSParserColor::CurrentColor {
                dest.write_str(" ")?;
                self.text_decoration_color.to_css(dest)?;
            }
            Ok(())
        }
    }
</%helpers:shorthand>
