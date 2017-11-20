/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values for font properties

use app_units::Au;
use byteorder::{BigEndian, ByteOrder};
use std::fmt;
use style_traits::ToCss;
use values::CSSFloat;
use values::animated::{ToAnimatedValue, ToAnimatedZero};
use values::computed::{Context, NonNegativeLength, ToComputedValue};
use values::generics::{FontSettings, FontSettingTagInt};
use values::specified::font as specified;
use values::specified::length::{FontBaseSize, NoCalcLength};

pub use values::computed::Length as MozScriptMinSize;
pub use values::specified::font::{XTextZoom, FontSynthesis, FontVariantSettings};

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

    #[inline]
    /// Get default value of font size.
    pub fn medium() -> Self {
        Self {
            size: Au::from_px(specified::FONT_MEDIUM_PX).into(),
            keyword_info: Some(KeywordInfo::medium())
        }
    }

    /// FIXME(emilio): This is very complex. Also, it should move to
    /// StyleBuilder.
    pub fn cascade_inherit_font_size(context: &mut Context) {
        // If inheriting, we must recompute font-size in case of language
        // changes using the font_size_keyword. We also need to do this to
        // handle mathml scriptlevel changes
        let kw_inherited_size = context.builder.get_parent_font()
                                       .clone_font_size()
                                       .keyword_info.map(|info| {
            specified::FontSize::Keyword(info).to_computed_value(context).size
        });
        let mut font = context.builder.take_font();
        font.inherit_font_size_from(context.builder.get_parent_font(),
                                    kw_inherited_size,
                                    context.builder.device);
        context.builder.put_font(font);
    }

    /// Cascade the initial value for the `font-size` property.
    ///
    /// FIXME(emilio): This is the only function that is outside of the
    /// `StyleBuilder`, and should really move inside!
    ///
    /// Can we move the font stuff there?
    pub fn cascade_initial_font_size(context: &mut Context) {
        // font-size's default ("medium") does not always
        // compute to the same value and depends on the font
        let computed = specified::FontSize::medium().to_computed_value(context);
        context.builder.mutate_font().set_font_size(computed);
        #[cfg(feature = "gecko")] {
            let device = context.builder.device;
            context.builder.mutate_font().fixup_font_min_size(device);
        }
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

#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq, ToCss)]
/// Preserve the readability of text when font fallback occurs
pub enum FontSizeAdjust {
    #[animation(error)]
    /// None variant
    None,
    /// Number variant
    Number(CSSFloat),
}

impl FontSizeAdjust {
    #[inline]
    /// Default value of font-size-adjust
    pub fn none() -> Self {
        FontSizeAdjust::None
    }

    /// Get font-size-adjust with float number
    pub fn from_gecko_adjust(gecko: f32) -> Self {
        if gecko == -1.0 {
            FontSizeAdjust::None
        } else {
            FontSizeAdjust::Number(gecko)
        }
    }
}

impl ToAnimatedZero for FontSizeAdjust {
    #[inline]
    // FIXME(emilio): why?
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Err(())
    }
}

impl ToAnimatedValue for FontSizeAdjust {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        match animated {
            FontSizeAdjust::Number(number) => FontSizeAdjust::Number(number.max(0.)),
            _ => animated
        }
    }
}

/// Use VariantAlternatesList as computed type of FontVariantAlternates
pub type FontVariantAlternates = specified::VariantAlternatesList;

impl FontVariantAlternates {
    #[inline]
    /// Get initial value with VariantAlternatesList
    pub fn get_initial_value() -> Self {
        specified::VariantAlternatesList(vec![].into_boxed_slice())
    }
}

/// Use VariantEastAsian as computed type of FontVariantEastAsian
pub type FontVariantEastAsian = specified::VariantEastAsian;

/// Use VariantLigatures as computed type of FontVariantLigatures
pub type FontVariantLigatures = specified::VariantLigatures;

/// Use VariantNumeric as computed type of FontVariantNumeric
pub type FontVariantNumeric = specified::VariantNumeric;

/// Use FontSettings as computed type of FontFeatureSettings
pub type FontFeatureSettings = FontSettings<FontSettingTagInt>;

impl FontFeatureSettings {
    #[inline]
    /// Default value of `font-feature-settings` as `normal`
    pub fn normal() -> FontFeatureSettings {
        FontSettings::Normal
    }
}

/// font-language-override can only have a single three-letter
/// OpenType "language system" tag, so we should be able to compute
/// it and store it as a 32-bit integer
/// (see http://www.microsoft.com/typography/otspec/languagetags.htm).
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq)]
pub struct FontLanguageOverride(pub u32);

impl FontLanguageOverride {
    #[inline]
    /// Get computed default value of `font-language-override` with 0
    pub fn zero() -> FontLanguageOverride {
        FontLanguageOverride(0)
    }
}

impl ToCss for FontLanguageOverride {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use std::str;

        if self.0 == 0 {
            return dest.write_str("normal")
        }
        let mut buf = [0; 4];
        BigEndian::write_u32(&mut buf, self.0);
        // Safe because we ensure it's ASCII during computing
        let slice = if cfg!(debug_assertions) {
            str::from_utf8(&buf).unwrap()
        } else {
            unsafe { str::from_utf8_unchecked(&buf) }
        };
        slice.trim_right().to_css(dest)
    }
}

#[cfg(feature = "gecko")]
impl From<u32> for FontLanguageOverride {
    fn from(bits: u32) -> FontLanguageOverride {
        FontLanguageOverride(bits)
    }
}

#[cfg(feature = "gecko")]
impl From<FontLanguageOverride> for u32 {
    fn from(v: FontLanguageOverride) -> u32 {
        v.0
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
            specified::MozScriptLevel::MozAbsolute(abs) => abs,
        };
        cmp::min(int, i8::MAX as i32) as i8
    }

    fn from_computed_value(other: &i8) -> Self {
        specified::MozScriptLevel::MozAbsolute(*other as i32)
    }
}
