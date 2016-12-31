/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`position`][position]s
//!
//! [position]: https://drafts.csswg.org/css-backgrounds-3/#position

use std::fmt;
use style_traits::ToCss;
use values::computed::LengthOrPercentage;

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct Position {
    pub horizontal: LengthOrPercentage,
    pub vertical: LengthOrPercentage,
}

impl ToCss for Position {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(self.horizontal.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.vertical.to_css(dest));
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct HorizontalPosition(pub LengthOrPercentage);

impl ToCss for HorizontalPosition {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct VerticalPosition(pub LengthOrPercentage);

impl ToCss for VerticalPosition {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)
    }
}
