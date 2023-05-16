/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for text properties.

#[cfg(feature = "servo")]
use crate::properties::StyleBuilder;
use crate::values::computed::length::{Length, LengthPercentage};
use crate::values::computed::{Context, NonNegativeLength, NonNegativeNumber, ToComputedValue};
use crate::values::generics::text::InitialLetter as GenericInitialLetter;
use crate::values::generics::text::LineHeight as GenericLineHeight;
use crate::values::generics::text::{GenericTextDecorationLength, Spacing};
use crate::values::specified::text::{self as specified, TextOverflowSide};
use crate::values::specified::text::{TextEmphasisFillMode, TextEmphasisShapeKeyword};
use crate::values::{CSSFloat, CSSInteger};
use crate::Zero;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

pub use crate::values::specified::text::{TextAlignLast, TextUnderlinePosition, MozControlCharacterVisibility};
pub use crate::values::specified::{LineBreak, OverflowWrap, RubyPosition, WordBreak};
pub use crate::values::specified::{TextDecorationLine, TextEmphasisPosition};
pub use crate::values::specified::{TextDecorationSkipInk, TextJustify, TextTransform};

/// A computed value for the `initial-letter` property.
pub type InitialLetter = GenericInitialLetter<CSSFloat, CSSInteger>;

/// Implements type for `text-decoration-thickness` property.
pub type TextDecorationLength = GenericTextDecorationLength<LengthPercentage>;

/// The computed value of `text-align`.
pub type TextAlign = specified::TextAlignKeyword;

/// A computed value for the `letter-spacing` property.
#[repr(transparent)]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    ToAnimatedValue,
    ToAnimatedZero,
    ToResolvedValue,
)]
pub struct LetterSpacing(pub Length);

impl LetterSpacing {
    /// Return the `normal` computed value, which is just zero.
    #[inline]
    pub fn normal() -> Self {
        LetterSpacing(Length::zero())
    }
}

impl ToCss for LetterSpacing {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        // https://drafts.csswg.org/css-text/#propdef-letter-spacing
        //
        // For legacy reasons, a computed letter-spacing of zero yields a
        // resolved value (getComputedStyle() return value) of normal.
        if self.0.is_zero() {
            return dest.write_str("normal");
        }
        self.0.to_css(dest)
    }
}

impl ToComputedValue for specified::LetterSpacing {
    type ComputedValue = LetterSpacing;
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            Spacing::Normal => LetterSpacing(Length::zero()),
            Spacing::Value(ref v) => LetterSpacing(v.to_computed_value(context)),
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        if computed.0.is_zero() {
            return Spacing::Normal;
        }
        Spacing::Value(ToComputedValue::from_computed_value(&computed.0))
    }
}

/// A computed value for the `word-spacing` property.
pub type WordSpacing = LengthPercentage;

impl ToComputedValue for specified::WordSpacing {
    type ComputedValue = WordSpacing;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            Spacing::Normal => LengthPercentage::zero(),
            Spacing::Value(ref v) => v.to_computed_value(context),
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Spacing::Value(ToComputedValue::from_computed_value(computed))
    }
}

/// A computed value for the `line-height` property.
pub type LineHeight = GenericLineHeight<NonNegativeNumber, NonNegativeLength>;

#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToResolvedValue)]
#[repr(C)]
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
#[derive(Clone, Copy, Debug, Default, MallocSizeOf, PartialEq, ToResolvedValue)]
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
        // Start with no declarations if this is an atomic inline-level box;
        // otherwise, start with the declarations in effect and add in the text
        // decorations that this block specifies.
        let mut result = if style.get_box().clone_display().is_atomic_inline_level() {
            Self::default()
        } else {
            style
                .get_parent_inherited_text()
                .text_decorations_in_effect
                .clone()
        };

        let line = style.get_text().clone_text_decoration_line();

        result.underline |= line.contains(TextDecorationLine::UNDERLINE);
        result.overline |= line.contains(TextDecorationLine::OVERLINE);
        result.line_through |= line.contains(TextDecorationLine::LINE_THROUGH);

        result
    }
}

/// Computed value for the text-emphasis-style property
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss, ToResolvedValue)]
#[allow(missing_docs)]
#[repr(C, u8)]
pub enum TextEmphasisStyle {
    /// [ <fill> || <shape> ]
    Keyword {
        #[css(skip_if = "TextEmphasisFillMode::is_filled")]
        fill: TextEmphasisFillMode,
        shape: TextEmphasisShapeKeyword,
    },
    /// `none`
    None,
    /// `<string>` (of which only the first grapheme cluster will be used).
    String(crate::OwnedStr),
}
