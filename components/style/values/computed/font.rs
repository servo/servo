/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values for font properties

use app_units::Au;
use std::fmt;
use style_traits::ToCss;
use values::animated::ToAnimatedValue;
use values::computed::{Context, NonNegativeLength, ToComputedValue};
use values::specified::font as specified;
use values::specified::length::{FontBaseSize, NoCalcLength};

pub use values::computed::Length as MozScriptMinSize;
pub use values::specified::font::XTextZoom;

/// As of CSS Fonts Module Level 3, only the following values are
/// valid: 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900
///
/// However, system fonts may provide other values. Pango
/// may provide 350, 380, and 1000 (on top of the existing values), for example.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq, ToCss)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub struct FontWeight(pub u16);

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
    /// Computes the final size for this font-size keyword, accounting for
    /// text-zoom.
    pub fn to_computed_value(&self, context: &Context) -> NonNegativeLength {
        let base = context.maybe_zoom_text(self.kw.to_computed_value(context));
        base.scale_by(self.factor) + context.maybe_zoom_text(self.offset)
    }

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

impl FontWeight {
    /// Value for normal
    pub fn normal() -> Self {
        FontWeight(400)
    }

    /// Value for bold
    pub fn bold() -> Self {
        FontWeight(700)
    }

    /// Convert from an integer to Weight
    pub fn from_int(n: i32) -> Result<Self, ()> {
        if n >= 100 && n <= 900 && n % 100 == 0 {
            Ok(FontWeight(n as u16))
        } else {
            Err(())
        }
    }

    /// Convert from an Gecko weight
    pub fn from_gecko_weight(weight: u16) -> Self {
        // we allow a wider range of weights than is parseable
        // because system fonts may provide custom values
        FontWeight(weight)
    }

    /// Weither this weight is bold
    pub fn is_bold(&self) -> bool {
        self.0 > 500
    }

    /// Return the bolder weight
    pub fn bolder(self) -> Self {
        if self.0 < 400 {
            FontWeight(400)
        } else if self.0 < 600 {
            FontWeight(700)
        } else {
            FontWeight(900)
        }
    }

    /// Returns the lighter weight
    pub fn lighter(self) -> Self {
        if self.0 < 600 {
            FontWeight(100)
        } else if self.0 < 800 {
            FontWeight(400)
        } else {
            FontWeight(700)
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

impl ToComputedValue for specified::MozScriptMinSize {
    type ComputedValue = MozScriptMinSize;

    fn to_computed_value(&self, cx: &Context) -> MozScriptMinSize {
        // this value is used in the computation of font-size, so
        // we use the parent size
        let base_size = FontBaseSize::InheritedStyle;
        match self.0 {
            NoCalcLength::FontRelative(value) => {
                value.to_computed_value(cx, base_size)
            }
            NoCalcLength::ServoCharacterWidth(value) => {
                value.to_computed_value(base_size.resolve(cx))
            }
            ref l => {
                l.to_computed_value(cx)
            }
        }
    }

    fn from_computed_value(other: &MozScriptMinSize) -> Self {
        specified::MozScriptMinSize(ToComputedValue::from_computed_value(other))
    }
}

/// The computed value of the -moz-script-level property.
pub type MozScriptLevel = i8;

#[cfg(feature = "gecko")]
impl ToComputedValue for specified::MozScriptLevel {
    type ComputedValue = MozScriptLevel;

    fn to_computed_value(&self, cx: &Context) -> i8 {
        use properties::longhands::_moz_math_display::SpecifiedValue as DisplayValue;
        use std::{cmp, i8};

        let int = match *self {
            specified::MozScriptLevel::Auto => {
                let parent = cx.builder.get_parent_font().clone__moz_script_level() as i32;
                let display = cx.builder.get_parent_font().clone__moz_math_display();
                if display == DisplayValue::inline {
                    parent + 1
                } else {
                    parent
                }
            }
            specified::MozScriptLevel::Relative(rel) => {
                let parent = cx.builder.get_parent_font().clone__moz_script_level();
                parent as i32 + rel
            }
            specified::MozScriptLevel::Absolute(abs) => abs,
        };
        cmp::min(int, i8::MAX as i32) as i8
    }

    fn from_computed_value(other: &i8) -> Self {
        specified::MozScriptLevel::Absolute(*other as i32)
    }
}
