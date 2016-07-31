/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// Per CSS-TEXT 6.2, "for legacy reasons, UAs must treat `word-wrap` as an alternate name for
// the `overflow-wrap` property, as if it were a shorthand of `overflow-wrap`."
<%helpers:shorthand name="word-wrap" sub_properties="overflow-wrap">
    use properties::longhands::overflow_wrap;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        Ok(Longhands {
            overflow_wrap: Some(try!(overflow_wrap::parse(context, input))),
        })
    }

    pub fn serialize<'a, W, I>(dest: &mut W, declarations: I) -> fmt::Result
        where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

        let mut overflow_wrap = None;

        for decl in declarations {
            match *decl {
                PropertyDeclaration::OverflowWrap(ref value) => { overflow_wrap = Some(value); },
                _ => return Err(fmt::Error)
            }
        }

        match overflow_wrap {
            Some(overflow_wrap) => overflow_wrap.to_css(dest),
            None => return Err(fmt::Error)
        }
    }
</%helpers:shorthand>
