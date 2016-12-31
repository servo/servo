/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A grid line type.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::ToCss;
use values::NoViewportPercentage;
use values::computed::ComputedValueAsSpecified;

#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-grid/#typedef-grid-row-start-grid-line
#[allow(missing_docs)]
pub struct GridLine {
    pub is_span: bool,
    pub ident: Option<String>,
    pub integer: Option<i32>,
}

impl Default for GridLine {
    fn default() -> Self {
        GridLine {
            is_span: false,
            ident: None,
            integer: None,
        }
    }
}

impl ToCss for GridLine {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if !self.is_span && self.ident.is_none() && self.integer.is_none() {
            return dest.write_str("auto")
        }

        if self.is_span {
            try!(dest.write_str("span"));
        }

        if let Some(i) = self.integer {
            try!(write!(dest, " {}", i));
        }

        if let Some(ref s) = self.ident {
            try!(write!(dest, " {}", s));
        }

        Ok(())
    }
}

impl Parse for GridLine {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let mut grid_line = Default::default();
        if input.try(|i| i.expect_ident_matching("auto")).is_ok() {
            return Ok(grid_line)
        }

        for _ in 0..3 {     // Maximum possible entities for <grid-line>
            if input.try(|i| i.expect_ident_matching("span")).is_ok() {
                if grid_line.is_span {
                    return Err(())
                }
                grid_line.is_span = true;
            } else if let Ok(i) = input.try(|i| i.expect_integer()) {
                if i == 0 || grid_line.integer.is_some() {
                    return Err(())
                }
                grid_line.integer = Some(i);
            } else if let Ok(name) = input.try(|i| i.expect_ident()) {
                if grid_line.ident.is_some() {
                    return Err(())
                }
                grid_line.ident = Some(name.into_owned());
            } else {
                break
            }
        }

        if grid_line.is_span {
            if let Some(i) = grid_line.integer {
                if i < 0 {      // disallow negative integers for grid spans
                    return Err(())
                }
            } else {
                grid_line.integer = Some(1);
            }
        }

        Ok(grid_line)
    }
}

impl ComputedValueAsSpecified for GridLine {}
impl NoViewportPercentage for GridLine {}
