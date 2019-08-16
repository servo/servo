/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="text-decoration"
                    engines="gecko servo-2013"
                    flags="SHORTHAND_IN_GETCS"
                    sub_properties="text-decoration-line
                    ${' text-decoration-style text-decoration-color text-decoration-thickness' if engine == 'gecko' else ''}"
                    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration">

    % if engine == "gecko":
        use crate::values::specified;
        use crate::properties::longhands::{text_decoration_style, text_decoration_color, text_decoration_thickness};
        use crate::properties::{PropertyId, LonghandId};
    % endif
    use crate::properties::longhands::text_decoration_line;

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        % if engine == "gecko":
            let text_decoration_thickness_enabled =
                PropertyId::Longhand(LonghandId::TextDecorationThickness).enabled_for_all_content();

            let (mut line, mut style, mut color, mut thickness, mut any) = (None, None, None, None, false);
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

            % if engine == "gecko":
                parse_component!(style, text_decoration_style);
                parse_component!(color, text_decoration_color);
                if text_decoration_thickness_enabled {
                    parse_component!(thickness, text_decoration_thickness);
                }
            % endif

            break;
        }

        if !any {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(expanded! {
            text_decoration_line: unwrap_or_initial!(text_decoration_line, line),

            % if engine == "gecko":
                text_decoration_style: unwrap_or_initial!(text_decoration_style, style),
                text_decoration_color: unwrap_or_initial!(text_decoration_color, color),
                text_decoration_thickness: unwrap_or_initial!(text_decoration_thickness, thickness),
            % endif
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            self.text_decoration_line.to_css(dest)?;

            % if engine == "gecko":
                if *self.text_decoration_style != text_decoration_style::SpecifiedValue::Solid {
                    dest.write_str(" ")?;
                    self.text_decoration_style.to_css(dest)?;
                }

                if *self.text_decoration_color != specified::Color::CurrentColor {
                    dest.write_str(" ")?;
                    self.text_decoration_color.to_css(dest)?;
                }

                if let Some(text_decoration_thickness) = self.text_decoration_thickness {
                    if !text_decoration_thickness.is_auto() {
                        dest.write_str(" ")?;
                        self.text_decoration_thickness.to_css(dest)?;
                    }
                }
            % endif

            Ok(())
        }
    }
</%helpers:shorthand>
