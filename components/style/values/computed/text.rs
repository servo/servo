/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for text properties.

use std::fmt;
use style_traits::ToCss;
use values::{CSSInteger, CSSFloat};
use values::animated::ToAnimatedZero;
use values::computed::{NonNegativeLength, NonNegativeNumber};
use values::computed::length::{Length, LengthOrPercentage};
use values::generics::text::InitialLetter as GenericInitialLetter;
use values::generics::text::LineHeight as GenericLineHeight;
use values::generics::text::Spacing;
use values::specified::text::{TextOverflowSide, TextDecorationLine};

pub use values::specified::TextAlignKeyword as TextAlign;

/// A computed value for the `initial-letter` property.
pub type InitialLetter = GenericInitialLetter<CSSFloat, CSSInteger>;

/// A computed value for the `letter-spacing` property.
pub type LetterSpacing = Spacing<Length>;

/// A computed value for the `word-spacing` property.
pub type WordSpacing = Spacing<LengthOrPercentage>;

/// A computed value for the `line-height` property.
pub type LineHeight = GenericLineHeight<NonNegativeNumber, NonNegativeLength>;

impl ToAnimatedZero for LineHeight {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> { Err(()) }
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
/// text-overflow.
/// When the specified value only has one side, that's the "second"
/// side, and the sides are logical, so "second" means "end".  The
/// start side is Clip in that case.
///
/// When the specified value has two sides, those are our "first"
/// and "second" sides, and they are physical sides ("left" and
/// "right").
pub struct TextOverflow {
    /// First side
    pub first: TextOverflowSide,
    /// Second side
    pub second: TextOverflowSide,
    /// True if the specified value only has one side.
    pub sides_are_logical: bool,
}

impl TextOverflow {
    /// Returns the initial `text-overflow` value
    pub fn get_initial_value() -> TextOverflow {
        TextOverflow {
            first: TextOverflowSide::Clip,
            second: TextOverflowSide::Clip,
            sides_are_logical: true,
        }
    }
}

impl ToCss for TextOverflow {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if self.sides_are_logical {
            debug_assert!(self.first == TextOverflowSide::Clip);
            self.second.to_css(dest)?;
        } else {
            self.first.to_css(dest)?;
            dest.write_str(" ")?;
            self.second.to_css(dest)?;
        }
        Ok(())
    }
}

impl ToCss for TextDecorationLine {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut has_any = false;

        macro_rules! write_value {
            ($line:path => $css:expr) => {
                if self.contains($line) {
                    if has_any {
                        dest.write_str(" ")?;
                    }
                    dest.write_str($css)?;
                    has_any = true;
                }
            }
        }
        write_value!(TextDecorationLine::UNDERLINE => "underline");
        write_value!(TextDecorationLine::OVERLINE => "overline");
        write_value!(TextDecorationLine::LINE_THROUGH => "line-through");
        write_value!(TextDecorationLine::BLINK => "blink");
        if !has_any {
            dest.write_str("none")?;
        }

        Ok(())
    }
}
