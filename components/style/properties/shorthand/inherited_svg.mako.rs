/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="marker" products="gecko"
    sub_properties="marker-start marker-end marker-mid"
    spec="https://www.w3.org/TR/SVG2/painting.html#MarkerShorthand">
    use values::specified::UrlOrNone;

    pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                               -> Result<Longhands, ParseError<'i>> {
        use parser::Parse;
        let url = UrlOrNone::parse(context, input)?;

        Ok(expanded! {
            marker_start: url.clone(),
            marker_mid: url.clone(),
            marker_end: url,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            if self.marker_start == self.marker_mid && self.marker_mid == self.marker_end {
                self.marker_start.to_css(dest)
            } else {
                Ok(())
            }
        }
    }
</%helpers:shorthand>
