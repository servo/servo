/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values for font properties

use app_units::Au;
use std::fmt;
use style_traits::ToCss;
use values::animated::ToAnimatedValue;
use values::computed::NonNegativeLength;
use values::specified::font as specified;

#[derive(Animate, ComputeSquaredDistance, MallocSizeOf, ToAnimatedZero)]
#[derive(Clone, Copy, Debug, PartialEq)]
/// The computed value of font-size
pub struct FontSize {
    /// The size.
    pub size: NonNegativeLength,
    /// If derived from a keyword, the keyword and additional transformations applied to it
    pub keyword_info: Option<KeywordInfo>,
}

#[derive(Animate, ComputeSquaredDistance, MallocSizeOf, ToAnimatedValue, ToAnimatedZero)]
#[derive(Clone, Copy, Debug, PartialEq)]
/// Additional information for keyword-derived font sizes.
pub struct KeywordInfo {
    /// The keyword used
    pub kw: specified::KeywordSize,
    /// A factor to be multiplied by the computed size of the keyword
    pub factor: f32,
    /// An additional Au offset to add to the kw*factor in the case of calcs
    pub offset: NonNegativeLength,
}

impl KeywordInfo {
    /// Given a parent keyword info (self), apply an additional factor/offset to it
    pub fn compose(self, factor: f32, offset: NonNegativeLength) -> Self {
        KeywordInfo {
            kw: self.kw,
            factor: self.factor * factor,
            offset: self.offset.scale_by(factor) + offset,
        }
    }

    /// KeywordInfo value for font-size: medium
    pub fn medium() -> Self {
        specified::KeywordSize::Medium.into()
    }
}

impl From<specified::KeywordSize> for KeywordInfo {
    fn from(x: specified::KeywordSize) -> Self {
        KeywordInfo {
            kw: x,
            factor: 1.,
            offset: Au(0).into(),
        }
    }
}

impl FontSize {
    /// The actual computed font size.
    pub fn size(self) -> Au {
        self.size.into()
    }
}

impl ToCss for FontSize {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.size.to_css(dest)
    }
}

/// XXXManishearth it might be better to
/// animate this as computed, however this complicates
/// clamping and might not be the right thing to do.
/// We should figure it out.
impl ToAnimatedValue for FontSize {
    type AnimatedValue = NonNegativeLength;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.size
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        FontSize {
            size: animated.clamp(),
            keyword_info: None,
        }
    }
}
