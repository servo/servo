/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for font stuff.

use cssparser::Parser;
use num_traits::One;
use parser::{Parse, ParserContext};
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};
use values::distance::{ComputeSquaredDistance, SquaredDistance};
use values::specified::font::FontTag;

/// https://drafts.csswg.org/css-fonts-4/#feature-tag-value
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct FeatureTagValue<Integer> {
    /// A four-character tag, packed into a u32 (one byte per character).
    pub tag: FontTag,
    /// The actual value.
    pub value: Integer,
}

impl<Integer> ToCss for FeatureTagValue<Integer>
where
    Integer: One + ToCss + PartialEq,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.tag.to_css(dest)?;
        // Don't serialize the default value.
        if self.value != Integer::one() {
            dest.write_char(' ')?;
            self.value.to_css(dest)?;
        }

        Ok(())
    }
}

/// Variation setting for a single feature, see:
///
/// https://drafts.csswg.org/css-fonts-4/#font-variation-settings-def
#[derive(Animate, Clone, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub struct VariationValue<Number> {
    /// A four-character tag, packed into a u32 (one byte per character).
    #[animation(constant)]
    pub tag: FontTag,
    /// The actual value.
    pub value: Number,
}

impl<Number> ComputeSquaredDistance for VariationValue<Number>
where
    Number: ComputeSquaredDistance,
{
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        if self.tag != other.tag {
            return Err(());
        }
        self.value.compute_squared_distance(&other.value)
    }
}

/// A value both for font-variation-settings and font-feature-settings.
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct FontSettings<T>(pub Box<[T]>);

impl<T> FontSettings<T> {
    /// Default value of font settings as `normal`.
    #[inline]
    pub fn normal() -> Self {
        FontSettings(vec![].into_boxed_slice())
    }
}

impl<T: Parse> Parse for FontSettings<T> {
    /// https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-feature-settings
    /// https://drafts.csswg.org/css-fonts-4/#font-variation-settings-def
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("normal")).is_ok() {
            return Ok(Self::normal());
        }

        Ok(FontSettings(
            input.parse_comma_separated(|i| T::parse(context, i))?.into_boxed_slice()
        ))
    }
}

impl<T: ToCss> ToCss for FontSettings<T> {
    /// https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-feature-settings
    /// https://drafts.csswg.org/css-fonts-4/#font-variation-settings-def
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.0.is_empty() {
            return dest.write_str("normal");
        }

        let mut first = true;
        for item in self.0.iter() {
            if !first {
                dest.write_str(", ")?;
            }
            first = false;
            item.to_css(dest)?;
        }

        Ok(())
    }
}
