/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%helpers:shorthand name="columns" sub_properties="column-count column-width" experimental="True">
    use properties::longhands::{column_count, column_width};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {

        let mut column_count = None;
        let mut column_width = None;
        let mut autos = 0;

        loop {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                // Leave the options to None, 'auto' is the initial value.
                autos += 1;
                continue
            }

            if column_count.is_none() {
                if let Ok(value) = input.try(|input| column_count::parse(context, input)) {
                    column_count = Some(value);
                    continue
                }
            }

            if column_width.is_none() {
                if let Ok(value) = input.try(|input| column_width::parse(context, input)) {
                    column_width = Some(value);
                    continue
                }
            }

            break
        }

        let values = autos + column_count.iter().len() + column_width.iter().len();
        if values == 0 || values > 2 {
            Err(())
        } else {
            Ok(Longhands {
                column_count: column_count,
                column_width: column_width,
            })
        }
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.column_width.to_css(dest));
            try!(write!(dest, " "));

            self.column_count.to_css(dest)
        }
    }
</%helpers:shorthand>
