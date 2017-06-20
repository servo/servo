/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to effects.

use std::fmt;
use style_traits::ToCss;
#[cfg(feature = "gecko")]
use values::specified::url::SpecifiedUrl;

/// A generic value for the `filter` property.
///
/// Keyword `none` is represented by an empty slice.
#[cfg_attr(feature = "servo", derive(Deserialize, HeapSizeOf, Serialize))]
#[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToComputedValue)]
pub struct FilterList<Filter>(pub Box<[Filter]>);

/// A generic value for a single `filter`.
#[cfg_attr(feature = "servo", derive(Deserialize, HeapSizeOf, Serialize))]
#[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToComputedValue, ToCss)]
pub enum Filter<Angle, Factor, Length, DropShadow> {
    /// `blur(<length>)`
    #[css(function)]
    Blur(Length),
    /// `brightness(<factor>)`
    #[css(function)]
    Brightness(Factor),
    /// `contrast(<factor>)`
    #[css(function)]
    Contrast(Factor),
    /// `grayscale(<factor>)`
    #[css(function)]
    Grayscale(Factor),
    /// `hue-rotate(<angle>)`
    #[css(function)]
    HueRotate(Angle),
    /// `invert(<factor>)`
    #[css(function)]
    Invert(Factor),
    /// `opacity(<factor>)`
    #[css(function)]
    Opacity(Factor),
    /// `saturate(<factor>)`
    #[css(function)]
    Saturate(Factor),
    /// `sepia(<factor>)`
    #[css(function)]
    Sepia(Factor),
    /// `drop-shadow(...)`
    #[css(function)]
    DropShadow(DropShadow),
    /// `<url>`
    #[cfg(feature = "gecko")]
    Url(SpecifiedUrl),
}

impl<F> FilterList<F> {
    /// Returns `none`.
    #[inline]
    pub fn none() -> Self {
        FilterList(vec![].into_boxed_slice())
    }
}

impl<F> From<Vec<F>> for FilterList<F> {
    #[inline]
    fn from(vec: Vec<F>) -> Self {
        FilterList(vec.into_boxed_slice())
    }
}

impl<F> ToCss for FilterList<F>
where
    F: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write
    {
        if let Some((first, rest)) = self.0.split_first() {
            first.to_css(dest)?;
            for filter in rest {
                dest.write_str(" ")?;
                filter.to_css(dest)?;
            }
            Ok(())
        } else {
            dest.write_str("none")
        }
    }
}
