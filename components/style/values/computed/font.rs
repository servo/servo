/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed values for font properties

use crate::parser::{Parse, ParserContext};
use crate::values::animated::ToAnimatedValue;
use crate::values::computed::{
    Angle, Context, Integer, Length, NonNegativeLength, NonNegativeNumber, Number, Percentage,
    ToComputedValue,
};
use crate::values::generics::font::{
    FeatureTagValue, FontSettings, TaggedFontValue, VariationValue,
};
use crate::values::generics::{font as generics, NonNegative};
use crate::values::specified::font::{
    self as specified, KeywordInfo, MAX_FONT_WEIGHT, MIN_FONT_WEIGHT,
};
use crate::values::specified::length::{FontBaseSize, NoCalcLength};
use crate::Atom;
use cssparser::{serialize_identifier, CssStringWriter, Parser};
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};

pub use crate::values::computed::Length as MozScriptMinSize;
pub use crate::values::specified::font::MozScriptSizeMultiplier;
pub use crate::values::specified::font::{FontPalette, FontSynthesis};
pub use crate::values::specified::font::{
    FontVariantAlternates, FontVariantEastAsian, FontVariantLigatures, FontVariantNumeric, XLang,
    XTextScale,
};
pub use crate::values::specified::Integer as SpecifiedInteger;
pub use crate::values::specified::Number as SpecifiedNumber;

/// Generic template for font property type classes that use a fixed-point
/// internal representation with `FRACTION_BITS` for the fractional part.
///
/// Values are constructed from and exposed as floating-point, but stored
/// internally as fixed point, so there will be a quantization effect on
/// fractional values, depending on the number of fractional bits used.
///
/// Using (16-bit) fixed-point types rather than floats for these style
/// attributes reduces the memory footprint of gfxFontEntry and gfxFontStyle; it
/// will also tend to reduce the number of distinct font instances that get
/// created, particularly when styles are animated or set to arbitrary values
/// (e.g. by sliders in the UI), which should reduce pressure on graphics
/// resources and improve cache hit rates.
///
/// cbindgen:derive-lt
/// cbindgen:derive-lte
/// cbindgen:derive-gt
/// cbindgen:derive-gte
#[repr(C)]
#[derive(
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    PartialEq,
    PartialOrd,
    ToResolvedValue,
)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub struct FixedPoint<T, const FRACTION_BITS: u16> {
    value: T,
}

impl<T, const FRACTION_BITS: u16> FixedPoint<T, FRACTION_BITS>
where
    T: num_traits::cast::AsPrimitive<f32>,
    f32: num_traits::cast::AsPrimitive<T>,
{
    const SCALE: u16 = 1 << FRACTION_BITS;
    const INVERSE_SCALE: f32 = 1.0 / Self::SCALE as f32;

    /// Returns a fixed-point bit from a floating-point context.
    fn from_float(v: f32) -> Self {
        use num_traits::cast::AsPrimitive;
        Self {
            value: (v * Self::SCALE as f32).round().as_(),
        }
    }

    /// Returns the floating-point representation.
    fn to_float(&self) -> f32 {
        self.value.as_() * Self::INVERSE_SCALE
    }
}

/// font-weight: range 1..1000, fractional values permitted; keywords
/// 'normal', 'bold' aliased to 400, 700 respectively.
///
/// We use an unsigned 10.6 fixed-point value (range 0.0 - 1023.984375)
pub const FONT_WEIGHT_FRACTION_BITS: u16 = 6;

/// This is an alias which is useful mostly as a cbindgen / C++ inference
/// workaround.
pub type FontWeightFixedPoint = FixedPoint<u16, FONT_WEIGHT_FRACTION_BITS>;

/// A value for the font-weight property per:
///
/// https://drafts.csswg.org/css-fonts-4/#propdef-font-weight
///
/// cbindgen:derive-lt
/// cbindgen:derive-lte
/// cbindgen:derive-gt
/// cbindgen:derive-gte
#[derive(
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Hash,
    MallocSizeOf,
    PartialEq,
    PartialOrd,
    ToResolvedValue,
)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct FontWeight(FontWeightFixedPoint);
impl ToAnimatedValue for FontWeight {
    type AnimatedValue = Number;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.value()
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        FontWeight::from_float(animated)
    }
}

impl ToCss for FontWeight {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.value().to_css(dest)
    }
}

impl FontWeight {
    /// The `normal` keyword.
    pub const NORMAL: FontWeight = FontWeight(FontWeightFixedPoint {
        value: 400 << FONT_WEIGHT_FRACTION_BITS,
    });

    /// The `bold` value.
    pub const BOLD: FontWeight = FontWeight(FontWeightFixedPoint {
        value: 700 << FONT_WEIGHT_FRACTION_BITS,
    });

    /// The threshold from which we consider a font bold.
    pub const BOLD_THRESHOLD: FontWeight = FontWeight(FontWeightFixedPoint {
        value: 600 << FONT_WEIGHT_FRACTION_BITS,
    });

    /// Returns the `normal` keyword value.
    pub fn normal() -> Self {
        Self::NORMAL
    }

    /// Weither this weight is bold
    pub fn is_bold(&self) -> bool {
        *self >= Self::BOLD_THRESHOLD
    }

    /// Returns the value as a float.
    pub fn value(&self) -> f32 {
        self.0.to_float()
    }

    /// Construct a valid weight from a float value.
    pub fn from_float(v: f32) -> Self {
        Self(FixedPoint::from_float(
            v.max(MIN_FONT_WEIGHT).min(MAX_FONT_WEIGHT),
        ))
    }

    /// Return the bolder weight.
    ///
    /// See the table in:
    /// https://drafts.csswg.org/css-fonts-4/#font-weight-numeric-values
    pub fn bolder(self) -> Self {
        let value = self.value();
        if value < 350. {
            return Self::NORMAL;
        }
        if value < 550. {
            return Self::BOLD;
        }
        Self::from_float(value.max(900.))
    }

    /// Return the lighter weight.
    ///
    /// See the table in:
    /// https://drafts.csswg.org/css-fonts-4/#font-weight-numeric-values
    pub fn lighter(self) -> Self {
        let value = self.value();
        if value < 550. {
            return Self::from_float(value.min(100.));
        }
        if value < 750. {
            return Self::NORMAL;
        }
        Self::BOLD
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
    /// The computed size, that we use to compute ems etc. This accounts for
    /// e.g., text-zoom.
    pub computed_size: NonNegativeLength,
    /// The actual used size. This is the computed font size, potentially
    /// constrained by other factors like minimum font-size settings and so on.
    #[css(skip)]
    pub used_size: NonNegativeLength,
    /// If derived from a keyword, the keyword and additional transformations applied to it
    #[css(skip)]
    pub keyword_info: KeywordInfo,
}

impl FontSize {
    /// The actual computed font size.
    #[inline]
    pub fn computed_size(&self) -> Length {
        self.computed_size.0
    }

    /// The actual used font size.
    #[inline]
    pub fn used_size(&self) -> Length {
        self.used_size.0
    }

    #[inline]
    /// Get default value of font size.
    pub fn medium() -> Self {
        Self {
            computed_size: NonNegative(Length::new(specified::FONT_MEDIUM_PX)),
            used_size: NonNegative(Length::new(specified::FONT_MEDIUM_PX)),
            keyword_info: KeywordInfo::medium(),
        }
    }
}

impl ToAnimatedValue for FontSize {
    type AnimatedValue = Length;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.computed_size.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        FontSize {
            computed_size: NonNegative(animated.clamp_to_non_negative()),
            used_size: NonNegative(animated.clamp_to_non_negative()),
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
    /// Whether this is the initial font-family that might react to language
    /// changes.
    pub is_initial: bool,
}

macro_rules! static_font_family {
    ($ident:ident, $family:expr) => {
        lazy_static! {
            static ref $ident: FontFamily = FontFamily {
                families: FontFamilyList {
                    #[cfg(feature = "gecko")]
                    list: crate::ArcSlice::from_iter_leaked(std::iter::once($family)),
                    #[cfg(feature = "servo")]
                    list: Box::new([$family]),
                },
                is_system_font: false,
                is_initial: false,
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
    #[cfg(feature = "gecko")]
    pub(crate) fn moz_bullet() -> &'static Self {
        static_font_family!(
            MOZ_BULLET,
            SingleFontFamily::FamilyName(FamilyName {
                name: atom!("-moz-bullet-font"),
                syntax: FontFamilyNameSyntax::Identifiers,
            })
        );

        &*MOZ_BULLET
    }

    /// Returns a font family for a single system font.
    #[cfg(feature = "gecko")]
    pub fn for_system_font(name: &str) -> Self {
        Self {
            families: FontFamilyList {
                list: crate::ArcSlice::from_iter(std::iter::once(SingleFontFamily::FamilyName(
                    FamilyName {
                        name: Atom::from(name),
                        syntax: FontFamilyNameSyntax::Identifiers,
                    },
                ))),
            },
            is_system_font: true,
            is_initial: false,
        }
    }

    /// Returns a generic font family.
    pub fn generic(generic: GenericFontFamily) -> &'static Self {
        macro_rules! generic_font_family {
            ($ident:ident, $family:ident) => {
                static_font_family!(
                    $ident,
                    SingleFontFamily::Generic(GenericFontFamily::$family)
                )
            };
        }

        generic_font_family!(SERIF, Serif);
        generic_font_family!(SANS_SERIF, SansSerif);
        generic_font_family!(MONOSPACE, Monospace);
        generic_font_family!(CURSIVE, Cursive);
        generic_font_family!(FANTASY, Fantasy);
        #[cfg(feature = "gecko")]
        generic_font_family!(MOZ_EMOJI, MozEmoji);
        generic_font_family!(SYSTEM_UI, SystemUi);

        match generic {
            GenericFontFamily::None => {
                debug_assert!(false, "Bogus caller!");
                &*SERIF
            },
            GenericFontFamily::Serif => &*SERIF,
            GenericFontFamily::SansSerif => &*SANS_SERIF,
            GenericFontFamily::Monospace => &*MONOSPACE,
            GenericFontFamily::Cursive => &*CURSIVE,
            GenericFontFamily::Fantasy => &*FANTASY,
            #[cfg(feature = "gecko")]
            GenericFontFamily::MozEmoji => &*MOZ_EMOJI,
            GenericFontFamily::SystemUi => &*SYSTEM_UI,
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
            None => {
                #[cfg(feature = "gecko")]
                return return Ok(());
                #[cfg(feature = "servo")]
                unreachable!();
            },
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

#[cfg(feature = "gecko")]
impl FamilyName {
    fn is_known_icon_font_family(&self) -> bool {
        use crate::gecko_bindings::bindings;
        unsafe { bindings::Gecko_IsKnownIconFontFamily(self.name.as_ptr()) }
    }
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

fn system_ui_enabled(_: &ParserContext) -> bool {
    #[cfg(feature = "gecko")]
    return static_prefs::pref!("layout.css.system-ui.enabled");
    #[cfg(feature = "servo")]
    return false;
}

/// A generic font-family name.
///
/// The order here is important, if you change it make sure that
/// `gfxPlatformFontList.h`s ranged array and `gfxFontFamilyList`'s
/// sSingleGenerics are updated as well.
///
/// NOTE(emilio): Should be u8, but it's a u32 because of ABI issues between GCC
/// and LLVM see https://bugs.llvm.org/show_bug.cgi?id=44228 / bug 1600735 /
/// bug 1726515.
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
#[repr(u32)]
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
    #[parse(condition = "system_ui_enabled")]
    SystemUi,
    /// An internal value for emoji font selection.
    #[css(skip)]
    #[cfg(feature = "gecko")]
    MozEmoji,
}

impl GenericFontFamily {
    /// When we disallow websites to override fonts, we ignore some generic
    /// families that the website might specify, since they're not configured by
    /// the user. See bug 789788 and bug 1730098.
    #[cfg(feature = "gecko")]
    pub(crate) fn valid_for_user_font_prioritization(self) -> bool {
        match self {
            Self::None | Self::Fantasy | Self::Cursive | Self::SystemUi | Self::MozEmoji => false,

            Self::Serif | Self::SansSerif | Self::Monospace => true,
        }
    }
}

impl Parse for SingleFontFamily {
    /// Parse a font-family value.
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(value) = input.try_parse(|i| i.expect_string_cloned()) {
            return Ok(SingleFontFamily::FamilyName(FamilyName {
                name: Atom::from(&*value),
                syntax: FontFamilyNameSyntax::Quoted,
            }));
        }

        if let Ok(generic) = input.try_parse(|i| GenericFontFamily::parse(context, i)) {
            return Ok(SingleFontFamily::Generic(generic));
        }

        let first_ident = input.expect_ident_cloned()?;
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
}

#[cfg(feature = "servo")]
impl SingleFontFamily {
    /// Get the corresponding font-family with Atom
    pub fn from_atom(input: Atom) -> SingleFontFamily {
        match input {
            atom!("serif") => return SingleFontFamily::Generic(GenericFontFamily::Serif),
            atom!("sans-serif") => return SingleFontFamily::Generic(GenericFontFamily::SansSerif),
            atom!("cursive") => return SingleFontFamily::Generic(GenericFontFamily::Cursive),
            atom!("fantasy") => return SingleFontFamily::Generic(GenericFontFamily::Fantasy),
            atom!("monospace") => return SingleFontFamily::Generic(GenericFontFamily::Monospace),
            atom!("system-ui") => return SingleFontFamily::Generic(GenericFontFamily::SystemUi),
            _ => {},
        }

        match_ignore_ascii_case! { &input,
            "serif" => return SingleFontFamily::Generic(GenericFontFamily::Serif),
            "sans-serif" => return SingleFontFamily::Generic(GenericFontFamily::SansSerif),
            "cursive" => return SingleFontFamily::Generic(GenericFontFamily::Cursive),
            "fantasy" => return SingleFontFamily::Generic(GenericFontFamily::Fantasy),
            "monospace" => return SingleFontFamily::Generic(GenericFontFamily::Monospace),
            "system-ui" => return SingleFontFamily::Generic(GenericFontFamily::SystemUi),
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
#[cfg(feature = "gecko")]
#[derive(Clone, Debug, ToComputedValue, ToResolvedValue, ToShmem, PartialEq, Eq)]
#[repr(C)]
pub struct FontFamilyList {
    /// The actual list of font families specified.
    pub list: crate::ArcSlice<SingleFontFamily>,
}

/// A list of font families.
#[cfg(feature = "servo")]
#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    Hash,
    MallocSizeOf,
    PartialEq,
    Serialize,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
pub struct FontFamilyList {
    /// The actual list of font families specified.
    pub list: Box<[SingleFontFamily]>,
}

impl FontFamilyList {
    /// Return iterator of SingleFontFamily
    pub fn iter(&self) -> impl Iterator<Item = &SingleFontFamily> {
        self.list.iter()
    }

    /// If there's a generic font family on the list which is suitable for user
    /// font prioritization, then move it ahead of the other families in the list,
    /// except for any families known to be ligature-based icon fonts, where using a
    /// generic instead of the site's specified font may cause substantial breakage.
    /// If no suitable generic is found in the list, insert the default generic ahead
    /// of all the listed families except for known ligature-based icon fonts.
    #[cfg(feature = "gecko")]
    pub(crate) fn prioritize_first_generic_or_prepend(&mut self, generic: GenericFontFamily) {
        let mut index_of_first_generic = None;
        let mut target_index = None;

        for (i, f) in self.iter().enumerate() {
            match &*f {
                SingleFontFamily::Generic(f) => {
                    if index_of_first_generic.is_none() && f.valid_for_user_font_prioritization() {
                        // If we haven't found a target position, there's nothing to do;
                        // this entry is already ahead of everything except any whitelisted
                        // icon fonts.
                        if target_index.is_none() {
                            return;
                        }
                        index_of_first_generic = Some(i);
                        break;
                    }
                    // A non-prioritized generic (e.g. cursive, fantasy) becomes the target
                    // position for prioritization, just like arbitrary named families.
                    if target_index.is_none() {
                        target_index = Some(i);
                    }
                },
                SingleFontFamily::FamilyName(fam) => {
                    // Target position for the first generic is in front of the first
                    // non-whitelisted icon font family we find.
                    if target_index.is_none() && !fam.is_known_icon_font_family() {
                        target_index = Some(i);
                    }
                },
            }
        }

        let mut new_list = self.list.iter().cloned().collect::<Vec<_>>();
        let first_generic = match index_of_first_generic {
            Some(i) => new_list.remove(i),
            None => SingleFontFamily::Generic(generic),
        };

        if let Some(i) = target_index {
            new_list.insert(i, first_generic);
        } else {
            new_list.push(first_generic);
        }
        self.list = crate::ArcSlice::from_iter(new_list.into_iter());
    }

    /// Returns whether we need to prioritize user fonts.
    #[cfg(feature = "gecko")]
    pub(crate) fn needs_user_font_prioritization(&self) -> bool {
        self.iter().next().map_or(true, |f| match f {
            SingleFontFamily::Generic(f) => !f.valid_for_user_font_prioritization(),
            _ => true,
        })
    }

    /// Return the generic ID if it is a single generic font
    pub fn single_generic(&self) -> Option<GenericFontFamily> {
        let mut iter = self.iter();
        if let Some(SingleFontFamily::Generic(f)) = iter.next() {
            if iter.next().is_none() {
                return Some(*f);
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

/// Use FontSettings as computed type of FontFeatureSettings.
pub type FontFeatureSettings = FontSettings<FeatureTagValue<Integer>>;

/// The computed value for font-variation-settings.
pub type FontVariationSettings = FontSettings<VariationValue<Number>>;

// The computed value of font-{feature,variation}-settings discards values
// with duplicate tags, keeping only the last occurrence of each tag.
fn dedup_font_settings<T>(settings_list: &mut Vec<T>)
where
    T: TaggedFontValue,
{
    if settings_list.len() > 1 {
        settings_list.sort_by_key(|k| k.tag().0);
        // dedup() keeps the first of any duplicates, but we want the last,
        // so we implement it manually here.
        let mut prev_tag = settings_list.last().unwrap().tag();
        for i in (0..settings_list.len() - 1).rev() {
            let cur_tag = settings_list[i].tag();
            if cur_tag == prev_tag {
                settings_list.remove(i);
            }
            prev_tag = cur_tag;
        }
    }
}

impl<T> ToComputedValue for FontSettings<T>
where
    T: ToComputedValue,
    <T as ToComputedValue>::ComputedValue: TaggedFontValue,
{
    type ComputedValue = FontSettings<T::ComputedValue>;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        let mut v = self
            .0
            .iter()
            .map(|item| item.to_computed_value(context))
            .collect::<Vec<_>>();
        dedup_font_settings(&mut v);
        FontSettings(v.into_boxed_slice())
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Self(
            computed
                .0
                .iter()
                .map(T::from_computed_value)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }
}

/// font-language-override can only have a single 1-4 ASCII character
/// OpenType "language system" tag, so we should be able to compute
/// it and store it as a 32-bit integer
/// (see http://www.microsoft.com/typography/otspec/languagetags.htm).
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
#[value_info(other_values = "normal")]
pub struct FontLanguageOverride(pub u32);

impl FontLanguageOverride {
    #[inline]
    /// Get computed default value of `font-language-override` with 0
    pub fn normal() -> FontLanguageOverride {
        FontLanguageOverride(0)
    }

    /// Returns this value as a `&str`, backed by `storage`.
    #[inline]
    pub(crate) fn to_str(self, storage: &mut [u8; 4]) -> &str {
        *storage = u32::to_be_bytes(self.0);
        // Safe because we ensure it's ASCII during parsing
        let slice = if cfg!(debug_assertions) {
            std::str::from_utf8(&storage[..]).unwrap()
        } else {
            unsafe { std::str::from_utf8_unchecked(&storage[..]) }
        };
        slice.trim_end()
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
                value.to_computed_value(base_size.resolve(cx).computed_size())
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

/// - Use a signed 8.8 fixed-point value (representable range -128.0..128)
///
/// Values of <angle> below -90 or above 90 not permitted, so we use out of
/// range values to represent normal | oblique
pub const FONT_STYLE_FRACTION_BITS: u16 = 8;

/// This is an alias which is useful mostly as a cbindgen / C++ inference
/// workaround.
pub type FontStyleFixedPoint = FixedPoint<i16, FONT_STYLE_FRACTION_BITS>;

/// The computed value of `font-style`.
///
/// - Define out of range values min value (-128.0) as meaning 'normal'
/// - Define max value (127.99609375) as 'italic'
/// - Other values represent 'oblique <angle>'
/// - Note that 'oblique 0deg' is distinct from 'normal' (should it be?)
///
/// cbindgen:derive-lt
/// cbindgen:derive-lte
/// cbindgen:derive-gt
/// cbindgen:derive-gte
#[derive(
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    PartialEq,
    PartialOrd,
    ToResolvedValue,
)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[repr(C)]
pub struct FontStyle(FontStyleFixedPoint);

impl FontStyle {
    /// The normal keyword.
    pub const NORMAL: FontStyle = FontStyle(FontStyleFixedPoint {
        value: 100 << FONT_STYLE_FRACTION_BITS,
    });
    /// The italic keyword.
    pub const ITALIC: FontStyle = FontStyle(FontStyleFixedPoint {
        value: 101 << FONT_STYLE_FRACTION_BITS,
    });

    /// The default angle for `font-style: oblique`.
    /// See also https://github.com/w3c/csswg-drafts/issues/2295
    pub const DEFAULT_OBLIQUE_DEGREES: i16 = 14;

    /// The `oblique` keyword with the default degrees.
    pub const OBLIQUE: FontStyle = FontStyle(FontStyleFixedPoint {
        value: Self::DEFAULT_OBLIQUE_DEGREES << FONT_STYLE_FRACTION_BITS,
    });

    /// The `normal` value.
    #[inline]
    pub fn normal() -> Self {
        Self::NORMAL
    }

    /// Returns the oblique angle for this style.
    pub fn oblique(degrees: f32) -> Self {
        Self(FixedPoint::from_float(
            degrees
                .max(specified::FONT_STYLE_OBLIQUE_MIN_ANGLE_DEGREES)
                .min(specified::FONT_STYLE_OBLIQUE_MAX_ANGLE_DEGREES),
        ))
    }

    /// Returns the oblique angle for this style.
    pub fn oblique_degrees(&self) -> f32 {
        debug_assert_ne!(*self, Self::NORMAL);
        debug_assert_ne!(*self, Self::ITALIC);
        self.0.to_float()
    }
}

impl ToCss for FontStyle {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        if *self == Self::NORMAL {
            return dest.write_str("normal");
        }
        if *self == Self::ITALIC {
            return dest.write_str("italic");
        }
        if *self == Self::OBLIQUE {
            return dest.write_str("oblique");
        }
        dest.write_str("oblique ")?;
        let angle = Angle::from_degrees(self.oblique_degrees());
        angle.to_css(dest)?;
        Ok(())
    }
}

impl ToAnimatedValue for FontStyle {
    type AnimatedValue = generics::FontStyle<Angle>;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        if self == Self::NORMAL {
            // This allows us to animate between normal and oblique values. Per spec,
            // https://drafts.csswg.org/css-fonts-4/#font-style-prop:
            //   Animation type: by computed value type; 'normal' animates as 'oblique 0deg'
            return generics::FontStyle::Oblique(Angle::from_degrees(0.0));
        }
        if self == Self::ITALIC {
            return generics::FontStyle::Italic;
        }
        generics::FontStyle::Oblique(Angle::from_degrees(self.oblique_degrees()))
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        match animated {
            generics::FontStyle::Normal => Self::NORMAL,
            generics::FontStyle::Italic => Self::ITALIC,
            generics::FontStyle::Oblique(ref angle) => {
                if angle.degrees() == 0.0 {
                    // Reverse the conversion done in to_animated_value()
                    Self::NORMAL
                } else {
                    Self::oblique(angle.degrees())
                }
            },
        }
    }
}

/// font-stretch is a percentage relative to normal.
///
/// We use an unsigned 10.6 fixed-point value (range 0.0 - 1023.984375)
///
/// We arbitrarily limit here to 1000%. (If that becomes a problem, we could
/// reduce the number of fractional bits and increase the limit.)
pub const FONT_STRETCH_FRACTION_BITS: u16 = 6;

/// This is an alias which is useful mostly as a cbindgen / C++ inference
/// workaround.
pub type FontStretchFixedPoint = FixedPoint<u16, FONT_STRETCH_FRACTION_BITS>;

/// A value for the font-stretch property per:
///
/// https://drafts.csswg.org/css-fonts-4/#propdef-font-stretch
///
/// cbindgen:derive-lt
/// cbindgen:derive-lte
/// cbindgen:derive-gt
/// cbindgen:derive-gte
#[derive(
    Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq, PartialOrd, ToResolvedValue,
)]
#[cfg_attr(feature = "servo", derive(Deserialize, Hash, Serialize))]
#[repr(C)]
pub struct FontStretch(pub FontStretchFixedPoint);

impl FontStretch {
    /// The fraction bits, as an easy-to-access-constant.
    pub const FRACTION_BITS: u16 = FONT_STRETCH_FRACTION_BITS;
    /// 0.5 in our floating point representation.
    pub const HALF: u16 = 1 << (Self::FRACTION_BITS - 1);

    /// The `ultra-condensed` keyword.
    pub const ULTRA_CONDENSED: FontStretch = FontStretch(FontStretchFixedPoint {
        value: 50 << Self::FRACTION_BITS,
    });
    /// The `extra-condensed` keyword.
    pub const EXTRA_CONDENSED: FontStretch = FontStretch(FontStretchFixedPoint {
        value: (62 << Self::FRACTION_BITS) + Self::HALF,
    });
    /// The `condensed` keyword.
    pub const CONDENSED: FontStretch = FontStretch(FontStretchFixedPoint {
        value: 75 << Self::FRACTION_BITS,
    });
    /// The `semi-condensed` keyword.
    pub const SEMI_CONDENSED: FontStretch = FontStretch(FontStretchFixedPoint {
        value: (87 << Self::FRACTION_BITS) + Self::HALF,
    });
    /// The `normal` keyword.
    pub const NORMAL: FontStretch = FontStretch(FontStretchFixedPoint {
        value: 100 << Self::FRACTION_BITS,
    });
    /// The `semi-expanded` keyword.
    pub const SEMI_EXPANDED: FontStretch = FontStretch(FontStretchFixedPoint {
        value: (112 << Self::FRACTION_BITS) + Self::HALF,
    });
    /// The `expanded` keyword.
    pub const EXPANDED: FontStretch = FontStretch(FontStretchFixedPoint {
        value: 125 << Self::FRACTION_BITS,
    });
    /// The `extra-expanded` keyword.
    pub const EXTRA_EXPANDED: FontStretch = FontStretch(FontStretchFixedPoint {
        value: 150 << Self::FRACTION_BITS,
    });
    /// The `ultra-expanded` keyword.
    pub const ULTRA_EXPANDED: FontStretch = FontStretch(FontStretchFixedPoint {
        value: 200 << Self::FRACTION_BITS,
    });

    /// 100%
    pub fn hundred() -> Self {
        Self::NORMAL
    }

    /// Converts to a computed percentage.
    #[inline]
    pub fn to_percentage(&self) -> Percentage {
        Percentage(self.0.to_float() / 100.0)
    }

    /// Converts from a computed percentage value.
    pub fn from_percentage(p: f32) -> Self {
        Self(FixedPoint::from_float((p * 100.).max(0.0).min(1000.0)))
    }

    /// Returns a relevant stretch value from a keyword.
    /// https://drafts.csswg.org/css-fonts-4/#font-stretch-prop
    pub fn from_keyword(kw: specified::FontStretchKeyword) -> Self {
        use specified::FontStretchKeyword::*;
        match kw {
            UltraCondensed => Self::ULTRA_CONDENSED,
            ExtraCondensed => Self::EXTRA_CONDENSED,
            Condensed => Self::CONDENSED,
            SemiCondensed => Self::SEMI_CONDENSED,
            Normal => Self::NORMAL,
            SemiExpanded => Self::SEMI_EXPANDED,
            Expanded => Self::EXPANDED,
            ExtraExpanded => Self::EXTRA_EXPANDED,
            UltraExpanded => Self::ULTRA_EXPANDED,
        }
    }

    /// Returns the stretch keyword if we map to one of the relevant values.
    pub fn as_keyword(&self) -> Option<specified::FontStretchKeyword> {
        use specified::FontStretchKeyword::*;
        // TODO: Can we use match here?
        if *self == Self::ULTRA_CONDENSED {
            return Some(UltraCondensed);
        }
        if *self == Self::EXTRA_CONDENSED {
            return Some(ExtraCondensed);
        }
        if *self == Self::CONDENSED {
            return Some(Condensed);
        }
        if *self == Self::SEMI_CONDENSED {
            return Some(SemiCondensed);
        }
        if *self == Self::NORMAL {
            return Some(Normal);
        }
        if *self == Self::SEMI_EXPANDED {
            return Some(SemiExpanded);
        }
        if *self == Self::EXPANDED {
            return Some(Expanded);
        }
        if *self == Self::EXTRA_EXPANDED {
            return Some(ExtraExpanded);
        }
        if *self == Self::ULTRA_EXPANDED {
            return Some(UltraExpanded);
        }
        None
    }
}

impl ToCss for FontStretch {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.to_percentage().to_css(dest)
    }
}

impl ToAnimatedValue for FontStretch {
    type AnimatedValue = Percentage;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.to_percentage()
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        Self::from_percentage(animated.0)
    }
}
