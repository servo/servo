/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values for font properties

use app_units::Au;
use byteorder::{BigEndian, ByteOrder};
#[cfg(feature = "servo")]
use computed_values::font_stretch::T as ComputedFontStretch;
#[cfg(feature = "servo")]
use computed_values::font_variant_caps::T as ComputedFontVariantCaps;
#[cfg(feature = "gecko")]
use gecko_bindings::{bindings, structs};
#[cfg(feature = "gecko")]
use gecko_bindings::sugar::refptr::RefPtr;
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use std::fmt::{self, Write};
#[cfg(feature = "gecko")]
use std::hash::{Hash, Hasher};
#[cfg(feature = "servo")]
use std::slice;
use style_traits::{CssWriter, ToCss};
#[cfg(feature = "servo")]
use style_traits::values::font::{FontStretch, FontVariantCaps};
use values::CSSFloat;
use values::animated::{ToAnimatedValue, ToAnimatedZero};
use values::computed::{Context, NonNegativeLength, ToComputedValue, Integer, Number};
use values::distance::{ComputeSquaredDistance, SquaredDistance};
use values::generics::font::{FontSettings, FeatureTagValue, VariationValue};
use values::specified::font as specified;
use values::specified::length::{FontBaseSize, NoCalcLength};

pub use values::computed::Length as MozScriptMinSize;
pub use values::specified::font::{XTextZoom, XLang, MozScriptSizeMultiplier, FontSynthesis};

pub use style_traits::values::font::{FamilyName, FontWeight, SingleFontFamily};

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
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
/// Specifies a prioritized list of font family names or generic family names.
pub struct FontFamily(pub FontFamilyList);

impl FontFamily {
    #[inline]
    /// Get default font family as `serif` which is a generic font-family
    pub fn serif() -> Self {
        FontFamily(
            FontFamilyList::new(Box::new([SingleFontFamily::Generic(atom!("serif"))]))
        )
    }
}

#[cfg(feature = "gecko")]
impl MallocSizeOf for FontFamily {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // SharedFontList objects are generally shared from the pointer
        // stored in the specified value. So only count this if the
        // SharedFontList is unshared.
        unsafe {
            bindings::Gecko_SharedFontList_SizeOfIncludingThisIfUnshared(
                (self.0).0.get()
            )
        }
    }
}

impl ToCss for FontFamily {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
        let mut iter = self.0.iter();
        iter.next().unwrap().to_css(dest)?;
        for family in iter {
            dest.write_str(", ")?;
            family.to_css(dest)?;
        }
        Ok(())
    }
}
#[cfg(feature = "servo")]
#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
/// A list of SingleFontFamily
pub struct FontFamilyList(Box<[SingleFontFamily]>);

#[cfg(feature = "gecko")]
#[derive(Clone, Debug)]
/// A list of SingleFontFamily
pub struct FontFamilyList(pub RefPtr<structs::SharedFontList>);

#[cfg(feature = "gecko")]
impl Hash for FontFamilyList {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        for name in self.0.mNames.iter() {
            name.mType.hash(state);
            name.mName.hash(state);
        }
    }
}

#[cfg(feature = "gecko")]
impl PartialEq for FontFamilyList {
    fn eq(&self, other: &FontFamilyList) -> bool {
        if self.0.mNames.len() != other.0.mNames.len() {
            return false;
        }
        for (a, b) in self.0.mNames.iter().zip(other.0.mNames.iter()) {
            if a.mType != b.mType || &*a.mName != &*b.mName {
                return false;
            }
        }
        true
    }
}

#[cfg(feature = "gecko")]
impl Eq for FontFamilyList {}

impl FontFamilyList {
    #[cfg(feature = "servo")]
    /// Return FontFamilyList with a vector of SingleFontFamily
    pub fn new(families: Box<[SingleFontFamily]>) -> FontFamilyList {
        FontFamilyList(families)
    }

    #[cfg(feature = "gecko")]
    /// Return FontFamilyList with a vector of SingleFontFamily
    pub fn new(families: Box<[SingleFontFamily]>) -> FontFamilyList {
        let fontlist;
        let names;
        unsafe {
            fontlist = bindings::Gecko_SharedFontList_Create();
            names = &mut (*fontlist).mNames;
            names.ensure_capacity(families.len());
        };

        for family in families.iter() {
            match *family {
                SingleFontFamily::FamilyName(ref f) => {
                    let quoted = matches!(f.syntax, FamilyNameSyntax::Quoted);
                    unsafe {
                        bindings::Gecko_nsTArray_FontFamilyName_AppendNamed(
                            names,
                            f.name.as_ptr(),
                            quoted
                        );
                    }
                }
                SingleFontFamily::Generic(ref name) => {
                    let (family_type, _generic) = SingleFontFamily::generic(name);
                    unsafe {
                        bindings::Gecko_nsTArray_FontFamilyName_AppendGeneric(
                            names,
                            family_type
                        );
                    }
                }
            }
        }

        FontFamilyList(unsafe { RefPtr::from_addrefed(fontlist) })
    }

    #[cfg(feature = "servo")]
    /// Return iterator of SingleFontFamily
    pub fn iter(&self) -> slice::Iter<SingleFontFamily> {
        self.0.iter()
    }

    #[cfg(feature = "gecko")]
    /// Return iterator of SingleFontFamily
    pub fn iter(&self) -> FontFamilyNameIter {
        FontFamilyNameIter {
            names: &self.0.mNames,
            cur: 0,
        }
    }

    #[cfg(feature = "gecko")]
    /// Return the generic ID if it is a single generic font
    pub fn single_generic(&self) -> Option<u8> {
        let mut iter = self.iter();
        if let Some(SingleFontFamily::Generic(ref name)) = iter.next() {
            if iter.next().is_none() {
                return Some(SingleFontFamily::generic(name).1);
            }
        }
        None
    }
}

#[cfg(feature = "gecko")]
/// Iterator of FontFamily
pub struct FontFamilyNameIter<'a> {
    names: &'a structs::nsTArray<structs::FontFamilyName>,
    cur: usize,
}

#[cfg(feature = "gecko")]
impl<'a> Iterator for FontFamilyNameIter<'a> {
    type Item = SingleFontFamily;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur < self.names.len() {
            let item = SingleFontFamily::from_font_family_name(&self.names[self.cur]);
            self.cur += 1;
            Some(item)
        } else {
            None
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

/// Use FontSettings as computed type of FontFeatureSettings.
pub type FontFeatureSettings = FontSettings<FeatureTagValue<Integer>>;

/// The computed value for font-variation-settings.
pub type FontVariationSettings = FontSettings<VariationValue<Number>>;

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
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
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
                if display == DisplayValue::Inline {
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
#[cfg(feature = "servo")]
impl ComputeSquaredDistance for FontWeight {
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        Ok(SquaredDistance::Sqrt((self.0 as f64 - other.0 as f64).abs() as f64))
    }
}

#[cfg(feature = "servo")]
impl From<ComputedFontStretch> for FontStretch {
    fn from(value: ComputedFontStretch) -> FontStretch {
        match value {
            ComputedFontStretch::Normal => FontStretch::Normal,
            ComputedFontStretch::UltraCondensed => FontStretch::UltraCondensed,
            ComputedFontStretch::ExtraCondensed => FontStretch::ExtraCondensed,
            ComputedFontStretch::Condensed => FontStretch::Condensed,
            ComputedFontStretch::SemiCondensed => FontStretch::SemiCondensed,
            ComputedFontStretch::SemiExpanded => FontStretch::SemiExpanded,
            ComputedFontStretch::Expanded => FontStretch::Expanded,
            ComputedFontStretch::ExtraExpanded => FontStretch::ExtraExpanded,
            ComputedFontStretch::UltraExpanded => FontStretch::UltraExpanded,
        }
    }
}

#[cfg(feature = "servo")]
impl From<ComputedFontVariantCaps> for FontVariantCaps {
    fn from(value: ComputedFontVariantCaps) -> FontVariantCaps {
        match value {
            ComputedFontVariantCaps::Normal => FontVariantCaps::Normal,
            ComputedFontVariantCaps::SmallCaps => FontVariantCaps::SmallCaps,
        }
    }
}
