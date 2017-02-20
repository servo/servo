/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="marker" products="gecko"
    sub_properties="marker-start marker-end marker-mid"
    spec="https://www.w3.org/TR/SVG2/painting.html#MarkerShorthand">
    use values::specified::UrlOrNone;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        use parser::Parse;
        let url = UrlOrNone::parse(context, input)?;

        Ok(Longhands {
            marker_start: Some(url.clone()),
            marker_mid: Some(url.clone()),
            marker_end: Some(url),
        })
    }

    impl<'a> LonghandsToSerialize<'a>  {
        fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if let DeclaredValue::Value(ref start) = *self.marker_start {
                if let DeclaredValue::Value(ref mid) = *self.marker_mid {
                    if let DeclaredValue::Value(ref end) = *self.marker_end {
                        if start == mid && mid == end {
                            start.to_css(dest)?;
                        }
                    }
                }
            }
            Ok(())
        }
    }
</%helpers:shorthand>
