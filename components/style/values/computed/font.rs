/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed values for font properties

#[cfg(feature = "gecko")]
use crate::gecko_bindings::{bindings, structs};
use crate::values::animated::ToAnimatedValue;
use crate::values::computed::{
    Angle, Context, Integer, Length, NonNegativeLength, NonNegativeNumber, NonNegativePercentage,
};
use crate::values::computed::{Number, Percentage, ToComputedValue};
use crate::values::generics::font::{FeatureTagValue, FontSettings, VariationValue};
use crate::values::generics::{font as generics, NonNegative};
use crate::values::specified::font::{
    self as specified, KeywordInfo, MAX_FONT_WEIGHT, MIN_FONT_WEIGHT,
};
use crate::values::specified::length::{FontBaseSize, NoCalcLength};
use crate::values::CSSFloat;
use crate::Atom;
use cssparser::{serialize_identifier, CssStringWriter, Parser};
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use std::fmt::{self, Write};
use std::hash::{Hash, Hasher};
use style_traits::{CssWriter, ParseError, ToCss};

pub use crate::values::computed::Length as MozScriptMinSize;
pub use crate::values::specified::font::{FontSynthesis, MozScriptSizeMultiplier};
pub use crate::values::specified::font::{XLang, XTextZoom};
pub use crate::values::specified::Integer as SpecifiedInteger;

/// A value for the font-weight property per:
///
/// https://drafts.csswg.org/css-fonts-4/#propdef-font-weight
///
/// This is effectively just a `Number`.
#[derive(
    Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq, ToCss, ToResolvedValue,
)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub struct FontWeight(pub Number);

impl Hash for FontWeight {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_u64((self.0 * 10000.).trunc() as u64);
    }
}

impl ToAnimatedValue for FontWeight {
    type AnimatedValue = Number;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        FontWeight(animated.max(MIN_FONT_WEIGHT).min(MAX_FONT_WEIGHT))
    }
}

#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    ToAnimatedZero,
    ToCss,
    ToResolvedValue,
)]
#[cfg_attr(feature = "servo", derive(Serialize, Deserialize))]
/// The computed value of font-size
pub struct FontSize {
    /// The size.
    pub size: NonNegativeLength,
    /// If derived from a keyword, the keyword and additional transformations applied to it
    #[css(skip)]
    pub keyword_info: KeywordInfo,
}

impl FontWeight {
    /// Value for normal
    pub fn normal() -> Self {
        FontWeight(400.)
    }

    /// Value for bold
    pub fn bold() -> Self {
        FontWeight(700.)
    }

    /// Convert from an Gecko weight
    #[cfg(feature = "gecko")]
    pub fn from_gecko_weight(weight: structs::FontWeight) -> Self {
        // we allow a wider range of weights than is parseable
        // because system fonts may provide custom values
        let weight = unsafe { bindings::Gecko_FontWeight_ToFloat(weight) };
        FontWeight(weight)
    }

    /// Weither this weight is bold
    pub fn is_bold(&self) -> bool {
        self.0 > 500.
    }

    /// Return the bolder weight.
    ///
    /// See the table in:
    /// https://drafts.csswg.org/css-fonts-4/#font-weight-numeric-values
    pub fn bolder(self) -> Self {
        if self.0 < 350. {
            FontWeight(400.)
        } else if self.0 < 550. {
            FontWeight(700.)
        } else {
            FontWeight(self.0.max(900.))
        }
    }

    /// Return the lighter weight.
    ///
    /// See the table in:
    /// https://drafts.csswg.org/css-fonts-4/#font-weight-numeric-values
    pub fn lighter(self) -> Self {
        if self.0 < 550. {
            FontWeight(self.0.min(100.))
        } else if self.0 < 750. {
            FontWeight(400.)
        } else {
            FontWeight(700.)
        }
    }
}

impl FontSize {
    /// The actual computed font size.
    #[inline]
    pub fn size(&self) -> Length {
        self.size.0
    }

    #[inline]
    /// Get default value of font size.
    pub fn medium() -> Self {
        Self {
            size: NonNegative(Length::new(specified::FONT_MEDIUM_PX)),
            keyword_info: KeywordInfo::medium(),
        }
    }
}

impl ToAnimatedValue for FontSize {
    type AnimatedValue = Length;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.size.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        FontSize {
            size: NonNegative(animated.clamp_to_non_negative()),
            keyword_info: KeywordInfo::none(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, ToComputedValue, ToResolvedValue)]
#[cfg_attr(feature = "servo", derive(Hash, MallocSizeOf, Serialize, Deserialize))]
/// Specifies a prioritized list of font family names or generic family names.
#[repr(C)]
pub struct FontFamily {
    /// The actual list of family names.
    pub families: FontFamilyList,
    /// Whether this font-family came from a specified system-font.
    pub is_system_font: bool,
}

macro_rules! static_font_family {
    ($ident:ident, $family:expr) => {
        lazy_static! {
            static ref $ident: FontFamily = FontFamily {
                families: FontFamilyList {
                    list: crate::ArcSlice::from_iter_leaked(std::iter::once($family)),
                    fallback: GenericFontFamily::None,
                },
                is_system_font: false,
            };
        }
    };
}


impl FontFamily {
    #[inline]
    /// Get default font family as `serif` which is a generic font-family
    pub fn serif() -> Self {
        Self::generic(GenericFontFamily::Serif).clone()
    }

    /// Returns the font family for `-moz-bullet-font`.
    pub(crate) fn moz_bullet() -> &'static Self {
        static_font_family!(MOZ_BULLET, SingleFontFamily::FamilyName(FamilyName {
            name: atom!("-moz-bullet-font"),
            syntax: FontFamilyNameSyntax::Identifiers,
        }));

        &*MOZ_BULLET
    }

    /// Returns a font family for a single system font.
    pub fn for_system_font(name: &str) -> Self {
        Self {
            families: FontFamilyList {
                list: crate::ArcSlice::from_iter(std::iter::once(SingleFontFamily::FamilyName(FamilyName {
                    name: Atom::from(name),
                    syntax: FontFamilyNameSyntax::Identifiers,
                }))),
                fallback: GenericFontFamily::None,
            },
            is_system_font: true,
        }
    }

    /// Returns a generic font family.
    pub fn generic(generic: GenericFontFamily) -> &'static Self {
        macro_rules! generic_font_family {
            ($ident:ident, $family:ident) => {
                static_font_family!($ident, SingleFontFamily::Generic(GenericFontFamily::$family))
            }
        }

        generic_font_family!(SERIF, Serif);
        generic_font_family!(SANS_SERIF, SansSerif);
        generic_font_family!(MONOSPACE, Monospace);
        generic_font_family!(CURSIVE, Cursive);
        generic_font_family!(FANTASY, Fantasy);
        generic_font_family!(MOZ_EMOJI, MozEmoji);

        match generic {
            GenericFontFamily::None => {
                debug_assert!(false, "Bogus caller!");
                &*SERIF
            }
            GenericFontFamily::Serif => &*SERIF,
            GenericFontFamily::SansSerif => &*SANS_SERIF,
            GenericFontFamily::Monospace => &*MONOSPACE,
            GenericFontFamily::Cursive => &*CURSIVE,
            GenericFontFamily::Fantasy => &*FANTASY,
            GenericFontFamily::MozEmoji => &*MOZ_EMOJI,
        }
    }
}

#[cfg(feature = "gecko")]
impl MallocSizeOf for FontFamily {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        use malloc_size_of::MallocUnconditionalSizeOf;
        // SharedFontList objects are generally measured from the pointer stored
        // in the specified value. So only count this if the SharedFontList is
        // unshared.
        let shared_font_list = &self.families.list;
        if shared_font_list.is_unique() {
            shared_font_list.unconditional_size_of(ops)
        } else {
            0
        }
    }
}

impl ToCss for FontFamily {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        let mut iter = self.families.iter();
        match iter.next() {
            Some(f) => f.to_css(dest)?,
            None => return self.families.fallback.to_css(dest),
        }
        for family in iter {
            dest.write_str(", ")?;
            family.to_css(dest)?;
        }
        Ok(())
    }
}

/// The name of a font family of choice.
#[derive(
    Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq, ToComputedValue, ToResolvedValue, ToShmem,
)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct FamilyName {
    /// Name of the font family.
    pub name: Atom,
    /// Syntax of the font family.
    pub syntax: FontFamilyNameSyntax,
}

impl ToCss for FamilyName {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        match self.syntax {
            FontFamilyNameSyntax::Quoted => {
                dest.write_char('"')?;
                write!(CssStringWriter::new(dest), "{}", self.name)?;
                dest.write_char('"')
            },
            FontFamilyNameSyntax::Identifiers => {
                let mut first = true;
                for ident in self.name.to_string().split(' ') {
                    if first {
                        first = false;
                    } else {
                        dest.write_char(' ')?;
                    }
                    debug_assert!(
                        !ident.is_empty(),
                        "Family name with leading, \
                         trailing, or consecutive white spaces should \
                         have been marked quoted by the parser"
                    );
                    serialize_identifier(ident, dest)?;
                }
                Ok(())
            },
        }
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq, ToComputedValue, ToResolvedValue, ToShmem,
)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
/// Font family names must either be given quoted as strings,
/// or unquoted as a sequence of one or more identifiers.
#[repr(u8)]
pub enum FontFamilyNameSyntax {
    /// The family name was specified in a quoted form, e.g. "Font Name"
    /// or 'Font Name'.
    Quoted,

    /// The family name was specified in an unquoted form as a sequence of
    /// identifiers.
    Identifiers,
}

/// A set of faces that vary in weight, width or slope.
/// cbindgen:derive-mut-casts=true
#[derive(
    Clone, Debug, Eq, MallocSizeOf, PartialEq, ToCss, ToComputedValue, ToResolvedValue, ToShmem,
)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize, Hash))]
#[repr(u8)]
pub enum SingleFontFamily {
    /// The name of a font family of choice.
    FamilyName(FamilyName),
    /// Generic family name.
    Generic(GenericFontFamily),
}

/// A generic font-family name.
///
/// The order here is important, if you change it make sure that
/// `gfxPlatformFontList.h`s ranged array and `gfxFontFamilyList`'s
/// sSingleGenerics are updated as well.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    PartialEq,
    Parse,
    ToCss,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum GenericFontFamily {
    /// No generic family specified, only for internal usage.
    ///
    /// NOTE(emilio): Gecko code relies on this variant being zero.
    #[css(skip)]
    None = 0,
    Serif,
    SansSerif,
    #[parse(aliases = "-moz-fixed")]
    Monospace,
    Cursive,
    Fantasy,
    /// An internal value for emoji font selection.
    #[css(skip)]
    #[cfg(feature = "gecko")]
    MozEmoji,
}

impl SingleFontFamily {
    /// Parse a font-family value.
    pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(value) = input.try_parse(|i| i.expect_string_cloned()) {
            return Ok(SingleFontFamily::FamilyName(FamilyName {
                name: Atom::from(&*value),
                syntax: FontFamilyNameSyntax::Quoted,
            }));
        }

        let first_ident = input.expect_ident_cloned()?;
        if let Ok(generic) = GenericFontFamily::from_ident(&first_ident) {
            return Ok(SingleFontFamily::Generic(generic));
        }

        let reserved = match_ignore_ascii_case! { &first_ident,
            // https://drafts.csswg.org/css-fonts/#propdef-font-family
            // "Font family names that happen to be the same as a keyword value
            //  (`inherit`, `serif`, `sans-serif`, `monospace`, `fantasy`, and `cursive`)
            //  must be quoted to prevent confusion with the keywords with the same names.
            //  The keywords ‘initial’ and ‘default’ are reserved for future use
            //  and must also be quoted when used as font names.
            //  UAs must not consider these keywords as matching the <family-name> type."
            "inherit" | "initial" | "unset" | "revert" | "default" => true,
            _ => false,
        };

        let mut value = first_ident.as_ref().to_owned();
        let mut serialize_quoted = value.contains(' ');

        // These keywords are not allowed by themselves.
        // The only way this value can be valid with with another keyword.
        if reserved {
            let ident = input.expect_ident()?;
            serialize_quoted = serialize_quoted || ident.contains(' ');
            value.push(' ');
            value.push_str(&ident);
        }
        while let Ok(ident) = input.try_parse(|i| i.expect_ident_cloned()) {
            serialize_quoted = serialize_quoted || ident.contains(' ');
            value.push(' ');
            value.push_str(&ident);
        }
        let syntax = if serialize_quoted {
            // For font family names which contains special white spaces, e.g.
            // `font-family: \ a\ \ b\ \ c\ ;`, it is tricky to serialize them
            // as identifiers correctly. Just mark them quoted so we don't need
            // to worry about them in serialization code.
            FontFamilyNameSyntax::Quoted
        } else {
            FontFamilyNameSyntax::Identifiers
        };
        Ok(SingleFontFamily::FamilyName(FamilyName {
            name: Atom::from(value),
            syntax,
        }))
    }

    #[cfg(feature = "servo")]
    /// Get the corresponding font-family with Atom
    pub fn from_atom(input: Atom) -> SingleFontFamily {
        match input {
            atom!("serif") => return SingleFontFamily::Generic(GenericFontFamily::Serif),
            atom!("sans-serif") => return SingleFontFamily::Generic(GenericFontFamily::SansSerif),
            atom!("cursive") => return SingleFontFamily::Generic(GenericFontFamily::Cursive),
            atom!("fantasy") => return SingleFontFamily::Generic(GenericFontFamily::Fantasy),
            atom!("monospace") => return SingleFontFamily::Generic(GenericFontFamily::Monospace),
            _ => {},
        }

        match_ignore_ascii_case! { &input,
            "serif" => return SingleFontFamily::Generic(GenericFontFamily::Serif),
            "sans-serif" => return SingleFontFamily::Generic(GenericFontFamily::SansSerif),
            "cursive" => return SingleFontFamily::Generic(GenericFontFamily::Cursive),
            "fantasy" => return SingleFontFamily::Generic(GenericFontFamily::Fantasy),
            "monospace" => return SingleFontFamily::Generic(GenericFontFamily::Monospace),
            _ => {}
        }

        // We don't know if it's quoted or not. So we set it to
        // quoted by default.
        SingleFontFamily::FamilyName(FamilyName {
            name: input,
            syntax: FontFamilyNameSyntax::Quoted,
        })
    }
}

/// A list of font families.
#[derive(Clone, Debug, ToComputedValue, ToResolvedValue, ToShmem, PartialEq, Eq)]
#[repr(C)]
pub struct FontFamilyList {
    /// The actual list of font families specified.
    pub list: crate::ArcSlice<SingleFontFamily>,
    /// A fallback font type (none, serif, or sans-serif, generally).
    pub fallback: GenericFontFamily,
}

impl FontFamilyList {
    /// Return iterator of SingleFontFamily
    pub fn iter(&self) -> impl Iterator<Item = &SingleFontFamily> {
        self.list.iter()
    }

    /// Puts the fallback in the list if needed.
    pub fn normalize(&mut self) {
        if self.fallback == GenericFontFamily::None {
            return;
        }
        let mut new_list = self.list.iter().cloned().collect::<Vec<_>>();
        new_list.push(SingleFontFamily::Generic(self.fallback));
        self.list = crate::ArcSlice::from_iter(new_list.into_iter());
    }

    /// If there's a generic font family on the list (which isn't cursive or
    /// fantasy), then move it to the front of the list. Otherwise, prepend the
    /// default generic.
    pub (crate) fn prioritize_first_generic_or_prepend(&mut self, generic: GenericFontFamily) {
        let index_of_first_generic = self.iter().position(|f| {
            match *f {
                SingleFontFamily::Generic(f) => f != GenericFontFamily::Cursive && f != GenericFontFamily::Fantasy,
                _ => false,
            }
        });

        if let Some(0) = index_of_first_generic {
            return; // Already first
        }

        let mut new_list = self.list.iter().cloned().collect::<Vec<_>>();
        let element_to_prepend = match index_of_first_generic {
            Some(i) => new_list.remove(i),
            None => SingleFontFamily::Generic(generic),
        };

        new_list.insert(0, element_to_prepend);
        self.list = crate::ArcSlice::from_iter(new_list.into_iter());
    }

    /// Return the generic ID if it is a single generic font
    pub fn single_generic(&self) -> Option<GenericFontFamily> {
        let mut iter = self.iter();
        if let Some(SingleFontFamily::Generic(f)) = iter.next() {
            if iter.next().is_none() {
                return Some(f.clone());
            }
        }
        None
    }
}

/// Preserve the readability of text when font fallback occurs
pub type FontSizeAdjust = generics::GenericFontSizeAdjust<NonNegativeNumber>;

impl FontSizeAdjust {
    #[inline]
    /// Default value of font-size-adjust
    pub fn none() -> Self {
        FontSizeAdjust::None
    }
}

/// Use VariantAlternatesList as computed type of FontVariantAlternates
pub type FontVariantAlternates = specified::VariantAlternatesList;

impl FontVariantAlternates {
    /// Get initial value with VariantAlternatesList
    #[inline]
    pub fn get_initial_value() -> Self {
        Self::default()
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
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToResolvedValue)]
#[repr(C)]
pub struct FontLanguageOverride(pub u32);

impl FontLanguageOverride {
    #[inline]
    /// Get computed default value of `font-language-override` with 0
    pub fn zero() -> FontLanguageOverride {
        FontLanguageOverride(0)
    }

    /// Returns this value as a `&str`, backed by `storage`.
    #[inline]
    pub(crate) fn to_str(self, storage: &mut [u8; 4]) -> &str {
        *storage = u32::to_be_bytes(self.0);
        // Safe because we ensure it's ASCII during computing
        let slice = if cfg!(debug_assertions) {
            std::str::from_utf8(&storage[..]).unwrap()
        } else {
            unsafe { std::str::from_utf8_unchecked(&storage[..]) }
        };
        slice.trim_end()
    }

    /// Parses a str, return `Self::zero()` if the input isn't a valid OpenType
    /// "language system" tag.
    #[inline]
    pub fn from_str(lang: &str) -> Self {
        if lang.is_empty() || lang.len() > 4 {
            return Self::zero();
        }
        let mut bytes = [b' '; 4];
        for (byte, lang_byte) in bytes.iter_mut().zip(lang.as_bytes()) {
            if !lang_byte.is_ascii() {
                return Self::zero();
            }
            *byte = *lang_byte;
        }
        Self(u32::from_be_bytes(bytes))
    }

    /// Unsafe because `Self::to_str` requires the value to represent a UTF-8
    /// string.
    #[inline]
    pub unsafe fn from_u32(value: u32) -> Self {
        Self(value)
    }
}

impl ToCss for FontLanguageOverride {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        if self.0 == 0 {
            return dest.write_str("normal");
        }
        self.to_str(&mut [0; 4]).to_css(dest)
    }
}

// FIXME(emilio): Make Gecko use the cbindgen'd fontLanguageOverride, then
// remove this.
#[cfg(feature = "gecko")]
impl From<u32> for FontLanguageOverride {
    fn from(v: u32) -> Self {
        unsafe { Self::from_u32(v) }
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
            NoCalcLength::FontRelative(value) => value.to_computed_value(cx, base_size),
            NoCalcLength::ServoCharacterWidth(value) => {
                value.to_computed_value(base_size.resolve(cx))
            },
            ref l => l.to_computed_value(cx),
        }
    }

    fn from_computed_value(other: &MozScriptMinSize) -> Self {
        specified::MozScriptMinSize(ToComputedValue::from_computed_value(other))
    }
}

/// The computed value of the math-depth property.
pub type MathDepth = i8;

#[cfg(feature = "gecko")]
impl ToComputedValue for specified::MathDepth {
    type ComputedValue = MathDepth;

    fn to_computed_value(&self, cx: &Context) -> i8 {
        use crate::properties::longhands::math_style::SpecifiedValue as MathStyleValue;
        use std::{cmp, i8};

        let int = match *self {
            specified::MathDepth::AutoAdd => {
                let parent = cx.builder.get_parent_font().clone_math_depth() as i32;
                let style = cx.builder.get_parent_font().clone_math_style();
                if style == MathStyleValue::Compact {
                    parent.saturating_add(1)
                } else {
                    parent
                }
            },
            specified::MathDepth::Add(rel) => {
                let parent = cx.builder.get_parent_font().clone_math_depth();
                (parent as i32).saturating_add(rel.to_computed_value(cx))
            },
            specified::MathDepth::Absolute(abs) => abs.to_computed_value(cx),
        };
        cmp::min(int, i8::MAX as i32) as i8
    }

    fn from_computed_value(other: &i8) -> Self {
        let computed_value = *other as i32;
        specified::MathDepth::Absolute(SpecifiedInteger::from_computed_value(&computed_value))
    }
}

/// A wrapper over an `Angle`, that handles clamping to the appropriate range
/// for `font-style` animation.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss, ToResolvedValue)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub struct FontStyleAngle(pub Angle);

impl ToAnimatedValue for FontStyleAngle {
    type AnimatedValue = Angle;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        FontStyleAngle(Angle::from_degrees(
            animated
                .degrees()
                .min(specified::FONT_STYLE_OBLIQUE_MAX_ANGLE_DEGREES)
                .max(specified::FONT_STYLE_OBLIQUE_MIN_ANGLE_DEGREES),
        ))
    }
}

impl Hash for FontStyleAngle {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_u64((self.0.degrees() * 10000.).trunc() as u64);
    }
}

/// The computed value of `font-style`.
///
/// FIXME(emilio): Angle should be a custom type to handle clamping during
/// animation.
pub type FontStyle = generics::FontStyle<FontStyleAngle>;

impl FontStyle {
    /// The `normal` value.
    #[inline]
    pub fn normal() -> Self {
        generics::FontStyle::Normal
    }

    /// The default angle for font-style: oblique. This is 20deg per spec:
    ///
    /// https://drafts.csswg.org/css-fonts-4/#valdef-font-style-oblique-angle
    #[inline]
    pub fn default_angle() -> FontStyleAngle {
        FontStyleAngle(Angle::from_degrees(
            specified::DEFAULT_FONT_STYLE_OBLIQUE_ANGLE_DEGREES,
        ))
    }

    /// Get the font style from Gecko's nsFont struct.
    #[cfg(feature = "gecko")]
    pub fn from_gecko(style: structs::FontSlantStyle) -> Self {
        let mut angle = 0.;
        let mut italic = false;
        let mut normal = false;
        unsafe {
            bindings::Gecko_FontSlantStyle_Get(style, &mut normal, &mut italic, &mut angle);
        }
        if normal {
            return generics::FontStyle::Normal;
        }
        if italic {
            return generics::FontStyle::Italic;
        }
        generics::FontStyle::Oblique(FontStyleAngle(Angle::from_degrees(angle)))
    }
}

impl ToCss for FontStyle {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            generics::FontStyle::Normal => dest.write_str("normal"),
            generics::FontStyle::Italic => dest.write_str("italic"),
            generics::FontStyle::Oblique(ref angle) => {
                dest.write_str("oblique")?;
                // Use `degrees` instead of just comparing Angle because
                // `degrees` can return slightly different values due to
                // floating point conversions.
                if angle.0.degrees() != Self::default_angle().0.degrees() {
                    dest.write_char(' ')?;
                    angle.to_css(dest)?;
                }
                Ok(())
            },
        }
    }
}

/// A value for the font-stretch property per:
///
/// https://drafts.csswg.org/css-fonts-4/#propdef-font-stretch
#[derive(
    Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq, ToCss, ToResolvedValue,
)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub struct FontStretch(pub NonNegativePercentage);

impl FontStretch {
    /// 100%
    pub fn hundred() -> Self {
        FontStretch(NonNegativePercentage::hundred())
    }

    /// The float value of the percentage
    #[inline]
    pub fn value(&self) -> CSSFloat {
        ((self.0).0).0
    }
}

impl ToAnimatedValue for FontStretch {
    type AnimatedValue = Percentage;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0.to_animated_value()
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        FontStretch(NonNegativePercentage::from_animated_value(animated))
    }
}

impl Hash for FontStretch {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_u64((self.value() * 10000.).trunc() as u64);
    }
}
