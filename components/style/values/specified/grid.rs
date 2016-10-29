/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ToCss};
use std::fmt;
use values::NoViewportPercentage;
use values::computed::ComputedValueAsSpecified;

#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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

        let mut vec = Vec::new();
        if self.is_span {
            vec.push("span".to_owned());
        }

        if let Some(i) = self.integer {
            vec.push(format!("{}", i));
        }

        if let Some(ref s) = self.ident {
            vec.push(s.to_owned());
        }

        write!(dest, "{}", vec.join(" "))
    }
}

impl GridLine {
    pub fn parse(input: &mut Parser) -> Result<GridLine, ()> {
        let mut grid_line = Default::default();
        if let Ok(_) = input.try(|i| i.expect_ident_matching("auto")) {
            return Ok(grid_line)
        }

        for _ in 0..3 {     // Maximum possible entities for <grid-line>
            if let Ok(_) = input.try(|i| i.expect_ident_matching("span")) {
                if grid_line.is_span {
                    return Err(())
                } else {
                    grid_line.is_span = true;
                }
            } else if let Ok(i) = input.try(|i| i.expect_integer()) {
                if i == 0 || grid_line.integer.is_some() {
                    return Err(())
                } else {
                    grid_line.integer = Some(i);
                }
            } else if let Ok(name) = input.try(|i| i.expect_ident()) {
                if grid_line.ident.is_some() {
                    return Err(())
                } else {
                    grid_line.ident = Some(name.into_owned());
                }
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
