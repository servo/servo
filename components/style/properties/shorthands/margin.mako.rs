/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

${helpers.four_sides_shorthand("margin", "margin-%s", "specified::LengthPercentageOrAuto::parse",
                               spec="https://drafts.csswg.org/css-box/#propdef-margin",
                               allowed_in_page_rule=True,
                               allow_quirks=True)}

% for axis in ["block", "inline"]:
    <%
        spec = "https://drafts.csswg.org/css-logical/#propdef-margin-%s" % axis
    %>
    <%helpers:shorthand
        name="margin-${axis}"
        sub_properties="${' '.join(
            'margin-%s-%s' % (axis, side)
            for side in ['start', 'end']
        )}"
        spec="${spec}">

        use crate::parser::Parse;
        use crate::values::specified::length::LengthPercentageOrAuto;
        pub fn parse_value<'i, 't>(
            context: &ParserContext,
            input: &mut Parser<'i, 't>,
        ) -> Result<Longhands, ParseError<'i>> {
            let start_value = LengthPercentageOrAuto::parse(context, input)?;
            let end_value =
                input.try(|input| LengthPercentageOrAuto::parse(context, input)).unwrap_or_else(|_| start_value.clone());

            Ok(expanded! {
                margin_${axis}_start: start_value,
                margin_${axis}_end: end_value,
            })
        }

        impl<'a> ToCss for LonghandsToSerialize<'a>  {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
                self.margin_${axis}_start.to_css(dest)?;

                if self.margin_${axis}_end != self.margin_${axis}_start {
                    dest.write_str(" ")?;
                    self.margin_${axis}_end.to_css(dest)?;
                }

                Ok(())
            }
        }
    </%helpers:shorthand>
% endfor
