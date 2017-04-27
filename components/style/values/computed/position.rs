/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`position`][position] values.
//!
//! [position]: https://drafts.csswg.org/css-backgrounds-3/#position

use std::fmt;
use style_traits::ToCss;
use values::computed::LengthOrPercentage;
use values::generics::position::{Position as GenericPosition, PositionWithKeyword};
use values::generics::position::HorizontalPosition as GenericHorizontalPosition;
use values::generics::position::VerticalPosition as GenericVerticalPosition;

/// The computed value of a CSS `<position>`
pub type Position = PositionWithKeyword<LengthOrPercentage>;

impl Copy for Position {}

/// The computed value for `<position>` values without a keyword.
pub type OriginPosition = GenericPosition<LengthOrPercentage, LengthOrPercentage>;

impl Copy for OriginPosition {}

impl OriginPosition {
    #[inline]
    /// The initial value for `perspective-origin`
    pub fn center() -> OriginPosition {
        GenericPosition {
            horizontal: LengthOrPercentage::Percentage(0.5),
            vertical: LengthOrPercentage::Percentage(0.5),
        }
    }
}

impl Position {
    #[inline]
    /// Construct a position at (0, 0)
    pub fn zero() -> Self {
        Position {
            horizontal: GenericHorizontalPosition(LengthOrPercentage::zero()),
            vertical: GenericVerticalPosition(LengthOrPercentage::zero()),
        }
    }
}

impl ToCss for Position {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.horizontal.to_css(dest)?;
        dest.write_str(" ")?;
        self.vertical.to_css(dest)
    }
}

/// The computed value of a horizontal `<position>`
pub type HorizontalPosition = GenericHorizontalPosition<LengthOrPercentage>;

impl Copy for HorizontalPosition {}

impl HorizontalPosition {
    #[inline]
    /// Create a zero position value.
    pub fn zero() -> HorizontalPosition {
        GenericHorizontalPosition(LengthOrPercentage::Percentage(0.0))
    }
}

/// The computed value of a vertical `<position>`
pub type VerticalPosition = GenericVerticalPosition<LengthOrPercentage>;

impl Copy for VerticalPosition {}

impl VerticalPosition {
    #[inline]
    /// Create a zero position value.
    pub fn zero() -> VerticalPosition {
        GenericVerticalPosition(LengthOrPercentage::Percentage(0.0))
    }
}
