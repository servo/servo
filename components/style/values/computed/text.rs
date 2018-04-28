/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for text properties.

#[cfg(feature = "servo")]
use properties::StyleBuilder;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};
use values::{CSSFloat, CSSInteger};
use values::computed::{NonNegativeLength, NonNegativeNumber};
use values::computed::length::{Length, LengthOrPercentage};
use values::generics::text::InitialLetter as GenericInitialLetter;
use values::generics::text::LineHeight as GenericLineHeight;
use values::generics::text::MozTabSize as GenericMozTabSize;
use values::generics::text::Spacing;
use values::specified::text::{TextEmphasisFillMode, TextEmphasisShapeKeyword, TextOverflowSide};

pub use values::specified::TextAlignKeyword as TextAlign;
pub use values::specified::TextEmphasisPosition;

/// A computed value for the `initial-letter` property.
pub type InitialLetter = GenericInitialLetter<CSSFloat, CSSInteger>;

/// A computed value for the `letter-spacing` property.
pub type LetterSpacing = Spacing<Length>;

/// A computed value for the `word-spacing` property.
pub type WordSpacing = Spacing<LengthOrPercentage>;

/// A computed value for the `line-height` property.
pub type LineHeight = GenericLineHeight<NonNegativeNumber, NonNegativeLength>;

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
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.sides_are_logical {
            debug_assert_eq!(self.first, TextOverflowSide::Clip);
            self.second.to_css(dest)?;
        } else {
            self.first.to_css(dest)?;
            dest.write_str(" ")?;
            self.second.to_css(dest)?;
        }
        Ok(())
    }
}

/// A struct that represents the _used_ value of the text-decoration property.
///
/// FIXME(emilio): This is done at style resolution time, though probably should
/// be done at layout time, otherwise we need to account for display: contents
/// and similar stuff when we implement it.
///
/// FIXME(emilio): Also, should be just a bitfield instead of three bytes.
#[derive(Clone, Copy, Debug, Default, MallocSizeOf, PartialEq)]
pub struct TextDecorationsInEffect {
    /// Whether an underline is in effect.
    pub underline: bool,
    /// Whether an overline decoration is in effect.
    pub overline: bool,
    /// Whether a line-through style is in effect.
    pub line_through: bool,
}

impl TextDecorationsInEffect {
    /// Computes the text-decorations in effect for a given style.
    #[cfg(feature = "servo")]
    pub fn from_style(style: &StyleBuilder) -> Self {
        use values::computed::Display;

        // Start with no declarations if this is an atomic inline-level box;
        // otherwise, start with the declarations in effect and add in the text
        // decorations that this block specifies.
        let mut result = match style.get_box().clone_display() {
            Display::InlineBlock | Display::InlineTable => Self::default(),
            _ => style
                .get_parent_inheritedtext()
                .text_decorations_in_effect
                .clone(),
        };

        let text_style = style.get_text();

        result.underline |= text_style.has_underline();
        result.overline |= text_style.has_overline();
        result.line_through |= text_style.has_line_through();

        result
    }
}

/// A specified value for the `-moz-tab-size` property.
pub type MozTabSize = GenericMozTabSize<NonNegativeNumber, NonNegativeLength>;

/// computed value for the text-emphasis-style property
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
pub enum TextEmphasisStyle {
    /// Keyword value for the text-emphasis-style property (`filled` `open`)
    Keyword(TextEmphasisKeywordValue),
    /// `none`
    None,
    /// String (will be used only first grapheme cluster) for the text-emphasis-style property
    String(String),
}

/// Keyword value for the text-emphasis-style property
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
pub struct TextEmphasisKeywordValue {
    /// fill for the text-emphasis-style property
    pub fill: TextEmphasisFillMode,
    /// shape for the text-emphasis-style property
    pub shape: TextEmphasisShapeKeyword,
}
