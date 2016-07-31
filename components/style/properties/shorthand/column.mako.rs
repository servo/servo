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

    pub fn serialize<'a, W, I>(dest: &mut W, declarations: I) -> fmt::Result
        where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

        let mut column_width = None;
        let mut column_count = None;

        for decl in declarations {
            match *decl {
                PropertyDeclaration::ColumnWidth(ref value) => { column_width = Some(value); },
                PropertyDeclaration::ColumnCount(ref value) => { column_count = Some(value); },
                _ => return Err(fmt::Error)
            }
        }

        let (column_width, column_count) = try_unwrap_longhands!(column_width, column_count);

        try!(column_width.to_css(dest));
        try!(write!(dest, " "));

        column_count.to_css(dest)
    }
</%helpers:shorthand>
