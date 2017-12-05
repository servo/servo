/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="text-decoration"
                    sub_properties="text-decoration-line
                    ${' text-decoration-style text-decoration-color' if product == 'gecko' else ''}"
                    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration">

    % if product == "gecko":
        use values::specified;
        use properties::longhands::{text_decoration_line, text_decoration_style, text_decoration_color};
    % else:
        use properties::longhands::text_decoration_line;
    % endif

    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
        % if product == "gecko":
            let (mut line, mut style, mut color, mut any) = (None, None, None, false);
        % else:
            let (mut line, mut any) = (None, false);
        % endif

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

            parse_component!(line, text_decoration_line);

            % if product == "gecko":
                parse_component!(style, text_decoration_style);
                parse_component!(color, text_decoration_color);
            % endif

            break;
        }

        if !any {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(expanded! {
            text_decoration_line: unwrap_or_initial!(text_decoration_line, line),

            % if product == "gecko":
                text_decoration_style: unwrap_or_initial!(text_decoration_style, style),
                text_decoration_color: unwrap_or_initial!(text_decoration_color, color),
            % endif
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.text_decoration_line.to_css(dest)?;

            % if product == "gecko":
                if *self.text_decoration_style != text_decoration_style::SpecifiedValue::Solid {
                    dest.write_str(" ")?;
                    self.text_decoration_style.to_css(dest)?;
                }

                if *self.text_decoration_color != specified::Color::CurrentColor {
                    dest.write_str(" ")?;
                    self.text_decoration_color.to_css(dest)?;
                }
            % endif

            Ok(())
        }
    }
</%helpers:shorthand>
