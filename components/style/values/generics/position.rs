/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS handling of specified and computed values of
//! [`position`](https://drafts.csswg.org/css-backgrounds-3/#position)

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::ToCss;
use values::HasViewportPercentage;
use values::computed::{ComputedValueAsSpecified, Context, ToComputedValue};
use values::specified::{LengthOrPercentage, Percentage};

define_css_keyword_enum!{ Keyword:
    "center" => Center,
    "left" => Left,
    "right" => Right,
    "top" => Top,
    "bottom" => Bottom,
    "x-start" => XStart,
    "x-end" => XEnd,
    "y-start" => YStart,
    "y-end" => YEnd
}

impl From<Keyword> for LengthOrPercentage {
    fn from(val: Keyword) -> LengthOrPercentage {
        match val {
            Keyword::Center => LengthOrPercentage::Percentage(Percentage(0.5)),
            Keyword::Left | Keyword::Top => LengthOrPercentage::Percentage(Percentage(0.0)),
            Keyword::Right | Keyword::Bottom => LengthOrPercentage::Percentage(Percentage(1.0)),
            // FIXME(canaltinova): Support logical keywords
            Keyword::XStart | Keyword::YStart => LengthOrPercentage::Percentage(Percentage(0.0)),
            Keyword::XEnd | Keyword::YEnd => LengthOrPercentage::Percentage(Percentage(1.0)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A generic type for representing horizontal or vertical `<position>` value.
pub struct PositionValue<L> {
    /// Even though this is generic, it's always a `<length-percentage>` value.
    pub position: Option<L>,
    /// A position keyword.
    pub keyword: Option<Keyword>,
}

impl<L: HasViewportPercentage> HasViewportPercentage for PositionValue<L> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.position.as_ref().map_or(false, |pos| pos.has_viewport_percentage())
    }
}

impl<L: Parse> PositionValue<L> {
    /// Internal parsing function which (after parsing) checks the keyword with the
    /// given function.
    pub fn parse_internal<F>(context: &ParserContext, input: &mut Parser,
                             mut is_allowed_keyword: F) -> Result<PositionValue<L>, ()>
        where F: FnMut(Keyword) -> bool
    {
        let (mut pos, mut keyword) = (None, None);
        for _ in 0..2 {
            if let Ok(l) = input.try(|i| L::parse(context, i)) {
                if pos.is_some() {
                    return Err(())
                }

                pos = Some(l);
            }

            if let Ok(k) = input.try(Keyword::parse) {
                if keyword.is_some() || (k == Keyword::Center && pos.is_some()) || !is_allowed_keyword(k) {
                    return Err(())
                }

                keyword = Some(k);
            }
        }

        if pos.is_none() && keyword.is_none() {
            return Err(())
        }

        Ok(PositionValue {
            position: pos,
            keyword: keyword,
        })
    }
}

impl<L: ToCss> ToCss for PositionValue<L> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if let Some(keyword) = self.keyword {
            keyword.to_css(dest)?;
        }

        if let Some(ref position) = self.position {
            if self.keyword.is_some() {
                dest.write_str(" ")?;
            }

            position.to_css(dest)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A generic type for representing horizontal `<position>`
pub struct HorizontalPosition<L>(pub L);

impl<L: ToCss> ToCss for HorizontalPosition<L> {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)
    }
}

impl<L: HasViewportPercentage> HasViewportPercentage for HorizontalPosition<L> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.0.has_viewport_percentage()
    }
}

impl<L: Parse> Parse for HorizontalPosition<PositionValue<L>> {
    #[inline]
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        PositionValue::parse_internal(context, input, |keyword| {
            match keyword {
                Keyword::Left | Keyword::Right | Keyword::Center |
                Keyword::XStart | Keyword::XEnd => true,
                _ => false,
            }
        }).map(HorizontalPosition)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A generic type for representing vertical `<position>`
pub struct VerticalPosition<L>(pub L);

impl<L: ToCss> ToCss for VerticalPosition<L> {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)
    }
}

impl<L: HasViewportPercentage> HasViewportPercentage for VerticalPosition<L> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.0.has_viewport_percentage()
    }
}

impl<L: Parse> Parse for VerticalPosition<PositionValue<L>> {
    #[inline]
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        PositionValue::parse_internal(context, input, |keyword| {
            match keyword {
                Keyword::Top | Keyword::Bottom | Keyword::Center |
                Keyword::YStart | Keyword::YEnd => true,
                _ => false,
            }
        }).map(VerticalPosition)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A generic type for representing a CSS [position](https://drafts.csswg.org/css-values/#position).
///
/// Note that the horizontal and vertical positions aren't really different types.
/// They're just unit struct wrappers over `LengthOrPercentage`. They should be different
/// because they allow different keywords (for e.g., vertical position doesn't allow
/// `right` or `left` keywords and vice versa).
pub struct Position<H, V> {
    /// The horizontal component of position.
    pub horizontal: H,
    /// The vertical component of position.
    pub vertical: V,
}

/// A generic type for representing positions with keywords.
pub type PositionWithKeyword<L> = Position<HorizontalPosition<L>, VerticalPosition<L>>;

impl<H: HasViewportPercentage, V: HasViewportPercentage> HasViewportPercentage for Position<H, V> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.horizontal.has_viewport_percentage() || self.vertical.has_viewport_percentage()
    }
}

impl<L: ToCss> ToCss for Position<L, L> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.horizontal.to_css(dest)?;
        dest.write_str(" ")?;
        self.vertical.to_css(dest)
    }
}

impl<L: ToCss> ToCss for PositionWithKeyword<PositionValue<L>> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        macro_rules! to_css_with_keyword {
            ($pos:expr, $default:expr) => {
                $pos.keyword.unwrap_or($default).to_css(dest)?;
                if let Some(ref position) = $pos.position {
                    dest.write_str(" ")?;
                    position.to_css(dest)?;
                }
            }
        }

        if (self.horizontal.0.keyword.is_some() && self.horizontal.0.position.is_some()) ||
           (self.vertical.0.keyword.is_some() && self.vertical.0.position.is_some()) {
            to_css_with_keyword!(self.horizontal.0, Keyword::Left);
            dest.write_str(" ")?;
            to_css_with_keyword!(self.vertical.0, Keyword::Top);
            return Ok(())
        }

        self.horizontal.to_css(dest)?;
        dest.write_str(" ")?;
        self.vertical.to_css(dest)
    }
}

impl<H: ToComputedValue, V: ToComputedValue> ToComputedValue for Position<H, V> {
    type ComputedValue = Position<H::ComputedValue, V::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        Position {
            horizontal: self.horizontal.to_computed_value(context),
            vertical: self.vertical.to_computed_value(context),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Position {
            horizontal: ToComputedValue::from_computed_value(&computed.horizontal),
            vertical: ToComputedValue::from_computed_value(&computed.vertical),
        }
    }
}
