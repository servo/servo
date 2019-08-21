/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed values for font properties

#[cfg(feature = "gecko")]
use crate::gecko_bindings::sugar::refptr::RefPtr;
#[cfg(feature = "gecko")]
use crate::gecko_bindings::{bindings, structs};
use crate::values::animated::{ToAnimatedValue, ToAnimatedZero};
use crate::values::computed::{Angle, Context, Integer, Length, NonNegativeLength, NonNegativePercentage};
use crate::values::computed::{Number, Percentage, ToComputedValue};
use crate::values::generics::{font as generics, NonNegative};
use crate::values::generics::font::{FeatureTagValue, FontSettings, VariationValue};
use crate::values::specified::font::{self as specified, KeywordInfo, MAX_FONT_WEIGHT, MIN_FONT_WEIGHT};
use crate::values::specified::length::{FontBaseSize, NoCalcLength};
use crate::values::CSSFloat;
use crate::Atom;
use app_units::Au;
use byteorder::{BigEndian, ByteOrder};
use cssparser::{serialize_identifier, CssStringWriter, Parser};
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use std::fmt::{self, Write};
use std::hash::{Hash, Hasher};
#[cfg(feature = "gecko")]
use std::mem::{self, ManuallyDrop};
#[cfg(feature = "servo")]
use std::slice;
use style_traits::{CssWriter, ParseError, ToCss};
#[cfg(feature = "gecko")]
use to_shmem::{SharedMemoryBuilder, ToShmem};

pub use crate::values::computed::Length as MozScriptMinSize;
pub use crate::values::specified::font::{FontSynthesis, MozScriptSizeMultiplier};
pub use crate::values::specified::font::{XLang, XTextZoom};

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
/// The computed value of font-size
pub struct FontSize {
    /// The size.
    pub size: NonNegativeLength,
    /// If derived from a keyword, the keyword and additional transformations applied to it
    #[css(skip)]
    pub keyword_info: Option<KeywordInfo>,
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
    pub fn size(self) -> Au {
        self.size.into()
    }

    #[inline]
    /// Get default value of font size.
    pub fn medium() -> Self {
        Self {
            size: Au::from_px(specified::FONT_MEDIUM_PX).into(),
            keyword_info: Some(KeywordInfo::medium()),
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
            keyword_info: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, ToResolvedValue)]
#[cfg_attr(feature = "servo", derive(Hash, MallocSizeOf))]
/// Specifies a prioritized list of font family names or generic family names.
pub struct FontFamily {
    /// The actual list of family names.
    pub families: FontFamilyList,
    /// Whether this font-family came from a specified system-font.
    pub is_system_font: bool,
}

impl FontFamily {
    #[inline]
    /// Get default font family as `serif` which is a generic font-family
    pub fn serif() -> Self {
        FontFamily {
            families: FontFamilyList::new(Box::new([SingleFontFamily::Generic(
                GenericFontFamily::Serif,
            )])),
            is_system_font: false,
        }
    }
}

#[cfg(feature = "gecko")]
impl MallocSizeOf for FontFamily {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // SharedFontList objects are generally shared from the pointer
        // stored in the specified value. So only count this if the
        // SharedFontList is unshared.
        let shared_font_list = self.families.shared_font_list().get();
        unsafe { bindings::Gecko_SharedFontList_SizeOfIncludingThisIfUnshared(shared_font_list) }
    }
}

impl ToCss for FontFamily {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        let mut iter = self.families.iter();
        iter.next().unwrap().to_css(dest)?;
        for family in iter {
            dest.write_str(", ")?;
            family.to_css(dest)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq, ToResolvedValue, ToShmem)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
/// The name of a font family of choice
pub struct FamilyName {
    /// Name of the font family
    pub name: Atom,
    /// Syntax of the font family
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

#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq, ToResolvedValue, ToShmem)]
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

#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, ToCss, ToResolvedValue, ToShmem)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize, Hash))]
/// A set of faces that vary in weight, width or slope.
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
    Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq, Parse, ToCss, ToResolvedValue, ToShmem,
)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum GenericFontFamily {
    /// No generic family specified, only for internal usage.
    #[css(skip)]
    None,
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
        if let Ok(value) = input.try(|i| i.expect_string_cloned()) {
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

        // These keywords are not allowed by themselves.
        // The only way this value can be valid with with another keyword.
        if reserved {
            let ident = input.expect_ident()?;
            value.push(' ');
            value.push_str(&ident);
        }
        while let Ok(ident) = input.try(|i| i.expect_ident_cloned()) {
            value.push(' ');
            value.push_str(&ident);
        }
        let syntax = if value.starts_with(' ') || value.ends_with(' ') || value.contains("  ") {
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

    #[cfg(feature = "gecko")]
    /// Get the corresponding font-family with family name
    fn from_font_family_name(family: &structs::FontFamilyName) -> SingleFontFamily {
        if family.mName.mRawPtr.is_null() {
            debug_assert_ne!(family.mGeneric, GenericFontFamily::None);
            return SingleFontFamily::Generic(family.mGeneric);
        }
        let name = unsafe { Atom::from_raw(family.mName.mRawPtr) };
        SingleFontFamily::FamilyName(FamilyName {
            name,
            syntax: family.mSyntax,
        })
    }
}

#[cfg(feature = "servo")]
#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq, ToResolvedValue, ToShmem)]
/// A list of SingleFontFamily
pub struct FontFamilyList(Box<[SingleFontFamily]>);

#[cfg(feature = "gecko")]
#[derive(Clone, Debug)]
/// A list of SingleFontFamily
pub enum FontFamilyList {
    /// A strong reference to a Gecko SharedFontList object.
    SharedFontList(RefPtr<structs::SharedFontList>),
    /// A font-family generic ID.
    Generic(GenericFontFamily),
}

#[cfg(feature = "gecko")]
impl ToShmem for FontFamilyList {
    fn to_shmem(&self, _builder: &mut SharedMemoryBuilder) -> ManuallyDrop<Self> {
        // In practice, the only SharedFontList objects we create from shared
        // style sheets are ones with a single generic entry.
        ManuallyDrop::new(match *self {
            FontFamilyList::SharedFontList(ref r) => {
                assert!(
                    r.mNames.len() == 1 && r.mNames[0].mName.mRawPtr.is_null(),
                    "ToShmem failed for FontFamilyList: cannot handle non-generic families",
                );
                FontFamilyList::Generic(r.mNames[0].mGeneric)
            },
            FontFamilyList::Generic(t) => FontFamilyList::Generic(t),
        })
    }
}

#[cfg(feature = "gecko")]
impl PartialEq for FontFamilyList {
    fn eq(&self, other: &FontFamilyList) -> bool {
        let self_list = self.shared_font_list();
        let other_list = other.shared_font_list();

        if self_list.mNames.len() != other_list.mNames.len() {
            return false;
        }
        for (a, b) in self_list.mNames.iter().zip(other_list.mNames.iter()) {
            if a.mSyntax != b.mSyntax ||
                a.mName.mRawPtr != b.mName.mRawPtr ||
                a.mGeneric != b.mGeneric
            {
                return false;
            }
        }
        true
    }
}

#[cfg(feature = "gecko")]
impl Eq for FontFamilyList {}

impl FontFamilyList {
    /// Return FontFamilyList with a vector of SingleFontFamily
    #[cfg(feature = "servo")]
    pub fn new(families: Box<[SingleFontFamily]>) -> FontFamilyList {
        FontFamilyList(families)
    }

    /// Return FontFamilyList with a vector of SingleFontFamily
    #[cfg(feature = "gecko")]
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
                SingleFontFamily::FamilyName(ref f) => unsafe {
                    bindings::Gecko_nsTArray_FontFamilyName_AppendNamed(
                        names,
                        f.name.as_ptr(),
                        f.syntax,
                    );
                },
                SingleFontFamily::Generic(family) => unsafe {
                    bindings::Gecko_nsTArray_FontFamilyName_AppendGeneric(names, family);
                },
            }
        }

        FontFamilyList::SharedFontList(unsafe { RefPtr::from_addrefed(fontlist) })
    }

    /// Return iterator of SingleFontFamily
    #[cfg(feature = "servo")]
    pub fn iter(&self) -> slice::Iter<SingleFontFamily> {
        self.0.iter()
    }

    /// Return iterator of SingleFontFamily
    #[cfg(feature = "gecko")]
    pub fn iter(&self) -> FontFamilyNameIter {
        FontFamilyNameIter {
            names: &self.shared_font_list().mNames,
            cur: 0,
        }
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

    /// Return a reference to the Gecko SharedFontList.
    #[cfg(feature = "gecko")]
    pub fn shared_font_list(&self) -> &RefPtr<structs::SharedFontList> {
        match *self {
            FontFamilyList::SharedFontList(ref r) => r,
            FontFamilyList::Generic(t) => {
                unsafe {
                    // TODO(heycam): Should really add StaticRefPtr sugar.
                    let index = t as usize;
                    mem::transmute::<
                        &structs::StaticRefPtr<structs::SharedFontList>,
                        &RefPtr<structs::SharedFontList>,
                    >(&structs::SharedFontList_sSingleGenerics[index])
                }
            },
        }
    }
}

/// Iterator of FontFamily
#[cfg(feature = "gecko")]
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

/// Preserve the readability of text when font fallback occurs
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    ToCss,
    ToResolvedValue,
)]
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
            _ => animated,
        }
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
}

impl ToCss for FontLanguageOverride {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        use std::str;

        if self.0 == 0 {
            return dest.write_str("normal");
        }
        let mut buf = [0; 4];
        BigEndian::write_u32(&mut buf, self.0);
        // Safe because we ensure it's ASCII during computing
        let slice = if cfg!(debug_assertions) {
            str::from_utf8(&buf).unwrap()
        } else {
            unsafe { str::from_utf8_unchecked(&buf) }
        };
        slice.trim_end().to_css(dest)
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

/// The computed value of the -moz-script-level property.
pub type MozScriptLevel = i8;

#[cfg(feature = "gecko")]
impl ToComputedValue for specified::MozScriptLevel {
    type ComputedValue = MozScriptLevel;

    fn to_computed_value(&self, cx: &Context) -> i8 {
        use crate::properties::longhands::_moz_math_display::SpecifiedValue as DisplayValue;
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
            },
            specified::MozScriptLevel::Relative(rel) => {
                let parent = cx.builder.get_parent_font().clone__moz_script_level();
                parent as i32 + rel
            },
            specified::MozScriptLevel::MozAbsolute(abs) => abs,
        };
        cmp::min(int, i8::MAX as i32) as i8
    }

    fn from_computed_value(other: &i8) -> Self {
        specified::MozScriptLevel::MozAbsolute(*other as i32)
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
