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
                if keyword.is_some() || !is_allowed_keyword(k) {
                    return Err(())
                }

                keyword = Some(k);
            }
        }

        if pos.is_some() {
            if let Some(Keyword::Center) = keyword {
                return Err(())      // "center" and <length> is not allowed
            }
        } else if keyword.is_none() {
            return Err(())      // at least one value is necessary
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
