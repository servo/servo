/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`basic-shape`][basic-shape]s
//!
//! [basic-shape]: https://drafts.csswg.org/css-shapes/#typedef-basic-shape

use values::computed::{Length, LengthOrPercentage};
use values::computed::BorderRadiusSize;
use std::fmt;
use cssparser::{self, Parser, ToCss, Token};

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum BasicShape {
    Inset(InsetRect),
    // Circle(Circle),
    // Ellipse(Ellipse),
    // Polygon(Polygon),
}

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct InsetRect {
    pub top: LengthOrPercentage,
    pub right: LengthOrPercentage,
    pub bottom: LengthOrPercentage,
    pub left: LengthOrPercentage,
    pub round: Option<BorderRadius>,
}

impl ToCss for InsetRect {
    // XXXManishearth again, we should try to reduce the number of values printed here
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("inset("));
        try!(self.top.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.right.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.bottom.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.left.to_css(dest));
        if let Some(ref radius) = self.round {
            try!(dest.write_str(" round "));
            try!(radius.to_css(dest));
        }
        dest.write_str(")")
    }
}

/// https://drafts.csswg.org/css-backgrounds-3/#border-radius
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct BorderRadius {
    pub top_left: BorderRadiusSize,
    pub top_right: BorderRadiusSize,
    pub bottom_left: BorderRadiusSize,
    pub bottom_right: BorderRadiusSize,
}

impl ToCss for BorderRadius {
    // XXXManishearth: We should be producing minimal output:
    // if height=width for all, we should not be printing the part after
    // the slash. For any set of four values,
    // we should try to reduce them to one or two. This probably should be
    // a helper function somewhere, for all the parse_four_sides-like
    // values
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(self.top_left.0.width.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.top_right.0.width.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.bottom_left.0.width.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.bottom_right.0.width.to_css(dest));
        try!(dest.write_str(" / "));
        try!(self.top_left.0.height.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.top_right.0.height.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.bottom_left.0.height.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.bottom_right.0.height.to_css(dest));
        dest.write_str(" ")
    }
}
