/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified values for font properties

#[cfg(feature = "gecko")]
use crate::gecko_bindings::bindings;
use crate::parser::{Parse, ParserContext};
use crate::properties::longhands::system_font::SystemFont;
use crate::values::computed::font::{FamilyName, FontFamilyList, FontStyleAngle, SingleFontFamily};
use crate::values::computed::{font as computed, Length, NonNegativeLength};
use crate::values::computed::{Angle as ComputedAngle, Percentage as ComputedPercentage};
use crate::values::computed::{CSSPixelLength, Context, ToComputedValue};
use crate::values::generics::font::VariationValue;
use crate::values::generics::font::{self as generics, FeatureTagValue, FontSettings, FontTag};
use crate::values::generics::NonNegative;
use crate::values::specified::length::{FontBaseSize, AU_PER_PT, AU_PER_PX};
use crate::values::specified::{AllowQuirks, Angle, Integer, LengthPercentage};
use crate::values::specified::{NoCalcLength, NonNegativeNumber, Number, Percentage};
use crate::values::CustomIdent;
use crate::Atom;
use cssparser::{Parser, Token};
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use std::fmt::{self, Write};
use style_traits::values::SequenceWriter;
use style_traits::{CssWriter, KeywordsCollectFn, ParseError};
use style_traits::{SpecifiedValueInfo, StyleParseErrorKind, ToCss};

// FIXME(emilio): The system font code is copy-pasta, and should be cleaned up.
macro_rules! system_font_methods {
    ($ty:ident, $field:ident) => {
        system_font_methods!($ty);

        fn compute_system(&self, _context: &Context) -> <$ty as ToComputedValue>::ComputedValue {
            debug_assert!(matches!(*self, $ty::System(..)));
            #[cfg(feature = "gecko")]
            {
                _context.cached_system_font.as_ref().unwrap().$field.clone()
            }
            #[cfg(feature = "servo")]
            {
                unreachable!()
            }
        }
    };

    ($ty:ident) => {
        /// Get a specified value that represents a system font.
        pub fn system_font(f: SystemFont) -> Self {
            $ty::System(f)
        }

        /// Retreive a SystemFont from the specified value.
        pub fn get_system(&self) -> Option<SystemFont> {
            if let $ty::System(s) = *self {
                Some(s)
            } else {
                None
            }
        }
    };
}

const DEFAULT_SCRIPT_MIN_SIZE_PT: u32 = 8;
const DEFAULT_SCRIPT_SIZE_MULTIPLIER: f64 = 0.71;

/// The minimum font-weight value per:
///
/// https://drafts.csswg.org/css-fonts-4/#font-weight-numeric-values
pub const MIN_FONT_WEIGHT: f32 = 1.;

/// The maximum font-weight value per:
///
/// https://drafts.csswg.org/css-fonts-4/#font-weight-numeric-values
pub const MAX_FONT_WEIGHT: f32 = 1000.;

/// A specified font-weight value.
///
/// https://drafts.csswg.org/css-fonts-4/#propdef-font-weight
#[derive(Clone, Copy, Debug, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub enum FontWeight {
    /// `<font-weight-absolute>`
    Absolute(AbsoluteFontWeight),
    /// Bolder variant
    Bolder,
    /// Lighter variant
    Lighter,
    /// System font variant.
    #[css(skip)]
    System(SystemFont),
}

impl FontWeight {
    system_font_methods!(FontWeight, font_weight);

    /// `normal`
    #[inline]
    pub fn normal() -> Self {
        FontWeight::Absolute(AbsoluteFontWeight::Normal)
    }

    /// Get a specified FontWeight from a gecko keyword
    pub fn from_gecko_keyword(kw: u32) -> Self {
        debug_assert!(kw % 100 == 0);
        debug_assert!(kw as f32 <= MAX_FONT_WEIGHT);
        FontWeight::Absolute(AbsoluteFontWeight::Weight(Number::new(kw as f32)))
    }
}

impl ToComputedValue for FontWeight {
    type ComputedValue = computed::FontWeight;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            FontWeight::Absolute(ref abs) => abs.compute(),
            FontWeight::Bolder => context
                .builder
                .get_parent_font()
                .clone_font_weight()
                .bolder(),
            FontWeight::Lighter => context
                .builder
                .get_parent_font()
                .clone_font_weight()
                .lighter(),
            FontWeight::System(_) => self.compute_system(context),
        }
    }

    #[inline]
    fn from_computed_value(computed: &computed::FontWeight) -> Self {
        FontWeight::Absolute(AbsoluteFontWeight::Weight(Number::from_computed_value(
            &computed.0,
        )))
    }
}

/// An absolute font-weight value for a @font-face rule.
///
/// https://drafts.csswg.org/css-fonts-4/#font-weight-absolute-values
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub enum AbsoluteFontWeight {
    /// A `<number>`, with the additional constraints specified in:
    ///
    ///   https://drafts.csswg.org/css-fonts-4/#font-weight-numeric-values
    Weight(Number),
    /// Normal font weight. Same as 400.
    Normal,
    /// Bold font weight. Same as 700.
    Bold,
}

impl AbsoluteFontWeight {
    /// Returns the computed value for this absolute font weight.
    pub fn compute(&self) -> computed::FontWeight {
        match *self {
            AbsoluteFontWeight::Weight(weight) => {
                computed::FontWeight(weight.get().max(MIN_FONT_WEIGHT).min(MAX_FONT_WEIGHT))
            },
            AbsoluteFontWeight::Normal => computed::FontWeight::normal(),
            AbsoluteFontWeight::Bold => computed::FontWeight::bold(),
        }
    }
}

impl Parse for AbsoluteFontWeight {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(number) = input.try(|input| Number::parse(context, input)) {
            // We could add another AllowedNumericType value, but it doesn't
            // seem worth it just for a single property with such a weird range,
            // so we do the clamping here manually.
            if !number.was_calc() &&
                (number.get() < MIN_FONT_WEIGHT || number.get() > MAX_FONT_WEIGHT)
            {
                return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }
            return Ok(AbsoluteFontWeight::Weight(number));
        }

        Ok(try_match_ident_ignore_ascii_case! { input,
            "normal" => AbsoluteFontWeight::Normal,
            "bold" => AbsoluteFontWeight::Bold,
        })
    }
}

/// The specified value of the `font-style` property, without the system font
/// crap.
pub type SpecifiedFontStyle = generics::FontStyle<Angle>;

impl ToCss for SpecifiedFontStyle {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            generics::FontStyle::Normal => dest.write_str("normal"),
            generics::FontStyle::Italic => dest.write_str("italic"),
            generics::FontStyle::Oblique(ref angle) => {
                dest.write_str("oblique")?;
                if *angle != Self::default_angle() {
                    dest.write_char(' ')?;
                    angle.to_css(dest)?;
                }
                Ok(())
            },
        }
    }
}

impl Parse for SpecifiedFontStyle {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Ok(try_match_ident_ignore_ascii_case! { input,
            "normal" => generics::FontStyle::Normal,
            "italic" => generics::FontStyle::Italic,
            "oblique" => {
                let angle = input.try(|input| Self::parse_angle(context, input))
                    .unwrap_or_else(|_| Self::default_angle());

                generics::FontStyle::Oblique(angle)
            },
        })
    }
}

impl ToComputedValue for SpecifiedFontStyle {
    type ComputedValue = computed::FontStyle;

    fn to_computed_value(&self, _: &Context) -> Self::ComputedValue {
        match *self {
            generics::FontStyle::Normal => generics::FontStyle::Normal,
            generics::FontStyle::Italic => generics::FontStyle::Italic,
            generics::FontStyle::Oblique(ref angle) => {
                generics::FontStyle::Oblique(FontStyleAngle(Self::compute_angle(angle)))
            },
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            generics::FontStyle::Normal => generics::FontStyle::Normal,
            generics::FontStyle::Italic => generics::FontStyle::Italic,
            generics::FontStyle::Oblique(ref angle) => {
                generics::FontStyle::Oblique(Angle::from_computed_value(&angle.0))
            },
        }
    }
}

/// The default angle for `font-style: oblique`.
///
/// NOTE(emilio): As of right now this diverges from the spec, which specifies
/// 20, because it's not updated yet to account for the resolution in:
///
///   https://github.com/w3c/csswg-drafts/issues/2295
pub const DEFAULT_FONT_STYLE_OBLIQUE_ANGLE_DEGREES: f32 = 14.;

/// From https://drafts.csswg.org/css-fonts-4/#valdef-font-style-oblique-angle:
///
///     Values less than -90deg or values greater than 90deg are
///     invalid and are treated as parse errors.
///
/// The maximum angle value that `font-style: oblique` should compute to.
pub const FONT_STYLE_OBLIQUE_MAX_ANGLE_DEGREES: f32 = 90.;

/// The minimum angle value that `font-style: oblique` should compute to.
pub const FONT_STYLE_OBLIQUE_MIN_ANGLE_DEGREES: f32 = -90.;

impl SpecifiedFontStyle {
    /// Gets a clamped angle in degrees from a specified Angle.
    pub fn compute_angle_degrees(angle: &Angle) -> f32 {
        angle
            .degrees()
            .max(FONT_STYLE_OBLIQUE_MIN_ANGLE_DEGREES)
            .min(FONT_STYLE_OBLIQUE_MAX_ANGLE_DEGREES)
    }

    fn compute_angle(angle: &Angle) -> ComputedAngle {
        ComputedAngle::from_degrees(Self::compute_angle_degrees(angle))
    }

    /// Parse a suitable angle for font-style: oblique.
    pub fn parse_angle<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Angle, ParseError<'i>> {
        let angle = Angle::parse(context, input)?;
        if angle.was_calc() {
            return Ok(angle);
        }

        let degrees = angle.degrees();
        if degrees < FONT_STYLE_OBLIQUE_MIN_ANGLE_DEGREES ||
            degrees > FONT_STYLE_OBLIQUE_MAX_ANGLE_DEGREES
        {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        return Ok(angle);
    }

    /// The default angle for `font-style: oblique`.
    pub fn default_angle() -> Angle {
        Angle::from_degrees(
            DEFAULT_FONT_STYLE_OBLIQUE_ANGLE_DEGREES,
            /* was_calc = */ false,
        )
    }
}

/// The specified value of the `font-style` property.
#[derive(Clone, Copy, Debug, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
#[allow(missing_docs)]
pub enum FontStyle {
    Specified(SpecifiedFontStyle),
    #[css(skip)]
    System(SystemFont),
}

impl FontStyle {
    /// Return the `normal` value.
    #[inline]
    pub fn normal() -> Self {
        FontStyle::Specified(generics::FontStyle::Normal)
    }

    system_font_methods!(FontStyle, font_style);
}

impl ToComputedValue for FontStyle {
    type ComputedValue = computed::FontStyle;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            FontStyle::Specified(ref specified) => specified.to_computed_value(context),
            FontStyle::System(..) => self.compute_system(context),
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        FontStyle::Specified(SpecifiedFontStyle::from_computed_value(computed))
    }
}

/// A value for the `font-stretch` property.
///
/// https://drafts.csswg.org/css-fonts-4/#font-stretch-prop
///
/// TODO(emilio): We could derive Parse if we had NonNegativePercentage.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
#[repr(u8)]
pub enum FontStretch {
    Stretch(Percentage),
    Keyword(FontStretchKeyword),
    #[css(skip)]
    System(SystemFont),
}

/// A keyword value for `font-stretch`.
#[derive(Clone, Copy, Debug, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
#[allow(missing_docs)]
pub enum FontStretchKeyword {
    Normal,
    Condensed,
    UltraCondensed,
    ExtraCondensed,
    SemiCondensed,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl FontStretchKeyword {
    /// Resolves the value of the keyword as specified in:
    ///
    /// https://drafts.csswg.org/css-fonts-4/#font-stretch-prop
    pub fn compute(&self) -> ComputedPercentage {
        use self::FontStretchKeyword::*;
        ComputedPercentage(match *self {
            UltraCondensed => 0.5,
            ExtraCondensed => 0.625,
            Condensed => 0.75,
            SemiCondensed => 0.875,
            Normal => 1.,
            SemiExpanded => 1.125,
            Expanded => 1.25,
            ExtraExpanded => 1.5,
            UltraExpanded => 2.,
        })
    }

    /// Does the opposite operation to `compute`, in order to serialize keywords
    /// if possible.
    pub fn from_percentage(percentage: f32) -> Option<Self> {
        use self::FontStretchKeyword::*;
        // NOTE(emilio): Can't use `match` because of rust-lang/rust#41620.
        if percentage == 0.5 {
            return Some(UltraCondensed);
        }
        if percentage == 0.625 {
            return Some(ExtraCondensed);
        }
        if percentage == 0.75 {
            return Some(Condensed);
        }
        if percentage == 0.875 {
            return Some(SemiCondensed);
        }
        if percentage == 1. {
            return Some(Normal);
        }
        if percentage == 1.125 {
            return Some(SemiExpanded);
        }
        if percentage == 1.25 {
            return Some(Expanded);
        }
        if percentage == 1.5 {
            return Some(ExtraExpanded);
        }
        if percentage == 2. {
            return Some(UltraExpanded);
        }
        None
    }
}

impl FontStretch {
    /// `normal`.
    pub fn normal() -> Self {
        FontStretch::Keyword(FontStretchKeyword::Normal)
    }

    system_font_methods!(FontStretch, font_stretch);
}

impl Parse for FontStretch {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // From https://drafts.csswg.org/css-fonts-4/#font-stretch-prop:
        //
        //    Values less than 0% are not allowed and are treated as parse
        //    errors.
        if let Ok(percentage) = input.try(|input| Percentage::parse_non_negative(context, input)) {
            return Ok(FontStretch::Stretch(percentage));
        }

        Ok(FontStretch::Keyword(FontStretchKeyword::parse(input)?))
    }
}

impl ToComputedValue for FontStretch {
    type ComputedValue = computed::FontStretch;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            FontStretch::Stretch(ref percentage) => {
                computed::FontStretch(NonNegative(percentage.to_computed_value(context)))
            },
            FontStretch::Keyword(ref kw) => computed::FontStretch(NonNegative(kw.compute())),
            FontStretch::System(_) => self.compute_system(context),
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        FontStretch::Stretch(Percentage::from_computed_value(&(computed.0).0))
    }
}

/// CSS font keywords
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[allow(missing_docs)]
pub enum KeywordSize {
    #[css(keyword = "xx-small")]
    XXSmall,
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
    #[css(keyword = "xx-large")]
    XXLarge,
    #[css(keyword = "xxx-large")]
    XXXLarge,
}

impl KeywordSize {
    /// Convert to an HTML <font size> value
    #[inline]
    pub fn html_size(self) -> u8 {
        self as u8
    }
}

impl Default for KeywordSize {
    fn default() -> Self {
        KeywordSize::Medium
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
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
/// Additional information for keyword-derived font sizes.
pub struct KeywordInfo {
    /// The keyword used
    pub kw: KeywordSize,
    /// A factor to be multiplied by the computed size of the keyword
    #[css(skip)]
    pub factor: f32,
    /// An additional fixed offset to add to the kw * factor in the case of
    /// `calc()`.
    #[css(skip)]
    pub offset: CSSPixelLength,
}

impl KeywordInfo {
    /// KeywordInfo value for font-size: medium
    pub fn medium() -> Self {
        Self::new(KeywordSize::Medium)
    }

    fn new(kw: KeywordSize) -> Self {
        KeywordInfo {
            kw,
            factor: 1.,
            offset: CSSPixelLength::new(0.),
        }
    }

    /// Computes the final size for this font-size keyword, accounting for
    /// text-zoom.
    fn to_computed_value(&self, context: &Context) -> CSSPixelLength {
        let base = context.maybe_zoom_text(self.kw.to_length(context).0);
        base * self.factor + context.maybe_zoom_text(self.offset)
    }

    /// Given a parent keyword info (self), apply an additional factor/offset to
    /// it.
    fn compose(self, factor: f32) -> Self {
        KeywordInfo {
            kw: self.kw,
            factor: self.factor * factor,
            offset: self.offset * factor,
        }
    }
}

impl SpecifiedValueInfo for KeywordInfo {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        <KeywordSize as SpecifiedValueInfo>::collect_completion_keywords(f);
    }
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
/// A specified font-size value
pub enum FontSize {
    /// A length; e.g. 10px.
    Length(LengthPercentage),
    /// A keyword value, along with a ratio and absolute offset.
    /// The ratio in any specified keyword value
    /// will be 1 (with offset 0), but we cascade keywordness even
    /// after font-relative (percent and em) values
    /// have been applied, which is where the ratio
    /// comes in. The offset comes in if we cascaded a calc value,
    /// where the font-relative portion (em and percentage) will
    /// go into the ratio, and the remaining units all computed together
    /// will go into the offset.
    /// See bug 1355707.
    Keyword(KeywordInfo),
    /// font-size: smaller
    Smaller,
    /// font-size: larger
    Larger,
    /// Derived from a specified system font.
    #[css(skip)]
    System(SystemFont),
}

/// Specifies a prioritized list of font family names or generic family names.
#[derive(Clone, Debug, Eq, PartialEq, ToCss, ToShmem)]
#[cfg_attr(feature = "servo", derive(Hash))]
pub enum FontFamily {
    /// List of `font-family`
    #[css(comma)]
    Values(#[css(iterable)] FontFamilyList),
    /// System font
    #[css(skip)]
    System(SystemFont),
}

impl FontFamily {
    system_font_methods!(FontFamily, font_family);

    /// Parse a specified font-family value
    pub fn parse_specified<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        let values = input.parse_comma_separated(SingleFontFamily::parse)?;
        Ok(FontFamily::Values(FontFamilyList::new(
            values.into_boxed_slice(),
        )))
    }
}

impl ToComputedValue for FontFamily {
    type ComputedValue = computed::FontFamily;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            FontFamily::Values(ref v) => computed::FontFamily {
                families: v.clone(),
                is_system_font: false,
            },
            FontFamily::System(_) => self.compute_system(context),
        }
    }

    fn from_computed_value(other: &computed::FontFamily) -> Self {
        FontFamily::Values(other.families.clone())
    }
}

#[cfg(feature = "gecko")]
impl MallocSizeOf for FontFamily {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        match *self {
            FontFamily::Values(ref v) => {
                // Although a SharedFontList object is refcounted, we always
                // attribute its size to the specified value, as long as it's
                // not a value in SharedFontList::sSingleGenerics.
                if matches!(v, FontFamilyList::SharedFontList(_)) {
                    let ptr = v.shared_font_list().get();
                    unsafe { bindings::Gecko_SharedFontList_SizeOfIncludingThis(ptr) }
                } else {
                    0
                }
            },
            FontFamily::System(_) => 0,
        }
    }
}

impl Parse for FontFamily {
    /// <family-name>#
    /// <family-name> = <string> | [ <ident>+ ]
    /// TODO: <generic-family>
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontFamily, ParseError<'i>> {
        FontFamily::parse_specified(input)
    }
}

impl SpecifiedValueInfo for FontFamily {}

/// `FamilyName::parse` is based on `SingleFontFamily::parse` and not the other way around
/// because we want the former to exclude generic family keywords.
impl Parse for FamilyName {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        match SingleFontFamily::parse(input) {
            Ok(SingleFontFamily::FamilyName(name)) => Ok(name),
            Ok(SingleFontFamily::Generic(_)) => {
                Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            },
            Err(e) => Err(e),
        }
    }
}

#[derive(Clone, Copy, Debug, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
/// Preserve the readability of text when font fallback occurs
pub enum FontSizeAdjust {
    /// None variant
    None,
    /// Number variant
    Number(NonNegativeNumber),
    /// system font
    #[css(skip)]
    System(SystemFont),
}

impl FontSizeAdjust {
    #[inline]
    /// Default value of font-size-adjust
    pub fn none() -> Self {
        FontSizeAdjust::None
    }

    system_font_methods!(FontSizeAdjust, font_size_adjust);
}

impl ToComputedValue for FontSizeAdjust {
    type ComputedValue = computed::FontSizeAdjust;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            FontSizeAdjust::None => computed::FontSizeAdjust::None,
            FontSizeAdjust::Number(ref n) => {
                // The computed version handles clamping of animated values
                // itself.
                computed::FontSizeAdjust::Number(n.to_computed_value(context).0)
            },
            FontSizeAdjust::System(_) => self.compute_system(context),
        }
    }

    fn from_computed_value(computed: &computed::FontSizeAdjust) -> Self {
        match *computed {
            computed::FontSizeAdjust::None => FontSizeAdjust::None,
            computed::FontSizeAdjust::Number(v) => {
                FontSizeAdjust::Number(NonNegativeNumber::from_computed_value(&v.into()))
            },
        }
    }
}

/// This is the ratio applied for font-size: larger
/// and smaller by both Firefox and Chrome
const LARGER_FONT_SIZE_RATIO: f32 = 1.2;

/// The default font size.
pub const FONT_MEDIUM_PX: i32 = 16;

impl KeywordSize {
    #[inline]
    #[cfg(feature = "servo")]
    fn to_length(&self, _: &Context) -> NonNegativeLength {
        let medium = Length::new(FONT_MEDIUM_PX as f32);
        // https://drafts.csswg.org/css-fonts-3/#font-size-prop
        NonNegative(match *self {
            KeywordSize::XXSmall => medium * 3.0 / 5.0,
            KeywordSize::XSmall => medium * 3.0 / 4.0,
            KeywordSize::Small => medium * 8.0 / 9.0,
            KeywordSize::Medium => medium,
            KeywordSize::Large => medium * 6.0 / 5.0,
            KeywordSize::XLarge => medium * 3.0 / 2.0,
            KeywordSize::XXLarge => medium * 2.0,
            KeywordSize::XXXLarge => medium * 3.0,
        })
    }

    #[cfg(feature = "gecko")]
    #[inline]
    fn to_length(&self, cx: &Context) -> NonNegativeLength {
        use crate::context::QuirksMode;

        // The tables in this function are originally from
        // nsRuleNode::CalcFontPointSize in Gecko:
        //
        // https://searchfox.org/mozilla-central/rev/c05d9d61188d32b8/layout/style/nsRuleNode.cpp#3150
        //
        // Mapping from base size and HTML size to pixels
        // The first index is (base_size - 9), the second is the
        // HTML size. "0" is CSS keyword xx-small, not HTML size 0,
        // since HTML size 0 is the same as 1.
        //
        //  xxs   xs      s      m     l      xl     xxl   -
        //  -     0/1     2      3     4      5      6     7
        static FONT_SIZE_MAPPING: [[i32; 8]; 8] = [
            [9, 9, 9, 9, 11, 14, 18, 27],
            [9, 9, 9, 10, 12, 15, 20, 30],
            [9, 9, 10, 11, 13, 17, 22, 33],
            [9, 9, 10, 12, 14, 18, 24, 36],
            [9, 10, 12, 13, 16, 20, 26, 39],
            [9, 10, 12, 14, 17, 21, 28, 42],
            [9, 10, 13, 15, 18, 23, 30, 45],
            [9, 10, 13, 16, 18, 24, 32, 48],
        ];

        // This table gives us compatibility with WinNav4 for the default fonts only.
        // In WinNav4, the default fonts were:
        //
        //     Times/12pt ==   Times/16px at 96ppi
        //   Courier/10pt == Courier/13px at 96ppi
        //
        // xxs   xs     s      m      l     xl     xxl    -
        // -     1      2      3      4     5      6      7
        static QUIRKS_FONT_SIZE_MAPPING: [[i32; 8]; 8] = [
            [9, 9, 9, 9, 11, 14, 18, 28],
            [9, 9, 9, 10, 12, 15, 20, 31],
            [9, 9, 9, 11, 13, 17, 22, 34],
            [9, 9, 10, 12, 14, 18, 24, 37],
            [9, 9, 10, 13, 16, 20, 26, 40],
            [9, 9, 11, 14, 17, 21, 28, 42],
            [9, 10, 12, 15, 17, 23, 30, 45],
            [9, 10, 13, 16, 18, 24, 32, 48],
        ];

        static FONT_SIZE_FACTORS: [i32; 8] = [60, 75, 89, 100, 120, 150, 200, 300];

        let ref gecko_font = cx.style().get_font().gecko();
        let base_size = unsafe {
            Atom::with(gecko_font.mLanguage.mRawPtr, |atom| {
                cx.font_metrics_provider
                    .get_size(atom, gecko_font.mGenericID)
            })
        };

        let base_size_px = base_size.px().round() as i32;
        let html_size = self.html_size() as usize;
        NonNegative(if base_size_px >= 9 && base_size_px <= 16 {
            let mapping = if cx.quirks_mode == QuirksMode::Quirks {
                QUIRKS_FONT_SIZE_MAPPING
            } else {
                FONT_SIZE_MAPPING
            };
            Length::new(mapping[(base_size_px - 9) as usize][html_size] as f32)
        } else {
            base_size * FONT_SIZE_FACTORS[html_size] as f32 / 100.0
        })
    }
}

impl FontSize {
    /// <https://html.spec.whatwg.org/multipage/#rules-for-parsing-a-legacy-font-size>
    pub fn from_html_size(size: u8) -> Self {
        FontSize::Keyword(KeywordInfo::new(match size {
            // If value is less than 1, let it be 1.
            0 | 1 => KeywordSize::XSmall,
            2 => KeywordSize::Small,
            3 => KeywordSize::Medium,
            4 => KeywordSize::Large,
            5 => KeywordSize::XLarge,
            6 => KeywordSize::XXLarge,
            // If value is greater than 7, let it be 7.
            _ => KeywordSize::XXXLarge,
        }))
    }

    /// Compute it against a given base font size
    pub fn to_computed_value_against(
        &self,
        context: &Context,
        base_size: FontBaseSize,
    ) -> computed::FontSize {
        use crate::values::specified::length::FontRelativeLength;

        let compose_keyword = |factor| {
            context
                .style()
                .get_parent_font()
                .clone_font_size()
                .keyword_info
                .map(|i| i.compose(factor))
        };
        let mut info = None;
        let size = match *self {
            FontSize::Length(LengthPercentage::Length(NoCalcLength::FontRelative(value))) => {
                if let FontRelativeLength::Em(em) = value {
                    // If the parent font was keyword-derived, this is too.
                    // Tack the em unit onto the factor
                    info = compose_keyword(em);
                }
                value.to_computed_value(context, base_size)
            },
            FontSize::Length(LengthPercentage::Length(NoCalcLength::ServoCharacterWidth(
                value,
            ))) => value.to_computed_value(base_size.resolve(context)),
            FontSize::Length(LengthPercentage::Length(NoCalcLength::Absolute(ref l))) => {
                context.maybe_zoom_text(l.to_computed_value(context))
            },
            FontSize::Length(LengthPercentage::Length(ref l)) => l.to_computed_value(context),
            FontSize::Length(LengthPercentage::Percentage(pc)) => {
                // If the parent font was keyword-derived, this is too.
                // Tack the % onto the factor
                info = compose_keyword(pc.0);
                base_size.resolve(context) * pc.0
            },
            FontSize::Length(LengthPercentage::Calc(ref calc)) => {
                let calc = calc.to_computed_value_zoomed(context, base_size);
                calc.resolve(base_size.resolve(context))
            },
            FontSize::Keyword(i) => {
                // As a specified keyword, this is keyword derived
                info = Some(i);
                i.to_computed_value(context).clamp_to_non_negative()
            },
            FontSize::Smaller => {
                info = compose_keyword(1. / LARGER_FONT_SIZE_RATIO);
                FontRelativeLength::Em(1. / LARGER_FONT_SIZE_RATIO)
                    .to_computed_value(context, base_size)
            },
            FontSize::Larger => {
                info = compose_keyword(LARGER_FONT_SIZE_RATIO);
                FontRelativeLength::Em(LARGER_FONT_SIZE_RATIO).to_computed_value(context, base_size)
            },

            FontSize::System(_) => {
                #[cfg(feature = "servo")]
                {
                    unreachable!()
                }
                #[cfg(feature = "gecko")]
                {
                    context
                        .cached_system_font
                        .as_ref()
                        .unwrap()
                        .font_size
                        .size
                        .0
                }
            },
        };
        computed::FontSize {
            size: NonNegative(size),
            keyword_info: info,
        }
    }
}

impl ToComputedValue for FontSize {
    type ComputedValue = computed::FontSize;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> computed::FontSize {
        self.to_computed_value_against(context, FontBaseSize::InheritedStyle)
    }

    #[inline]
    fn from_computed_value(computed: &computed::FontSize) -> Self {
        FontSize::Length(LengthPercentage::Length(
            ToComputedValue::from_computed_value(&computed.size.0),
        ))
    }
}

impl FontSize {
    system_font_methods!(FontSize);

    /// Get initial value for specified font size.
    #[inline]
    pub fn medium() -> Self {
        FontSize::Keyword(KeywordInfo::medium())
    }

    /// Parses a font-size, with quirks.
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<FontSize, ParseError<'i>> {
        if let Ok(lp) =
            input.try(|i| LengthPercentage::parse_non_negative_quirky(context, i, allow_quirks))
        {
            return Ok(FontSize::Length(lp));
        }

        if let Ok(kw) = input.try(KeywordSize::parse) {
            return Ok(FontSize::Keyword(KeywordInfo::new(kw)));
        }

        try_match_ident_ignore_ascii_case! { input,
            "smaller" => Ok(FontSize::Smaller),
            "larger" => Ok(FontSize::Larger),
        }
    }
}

impl Parse for FontSize {
    /// <length> | <percentage> | <absolute-size> | <relative-size>
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontSize, ParseError<'i>> {
        FontSize::parse_quirky(context, input, AllowQuirks::No)
    }
}

bitflags! {
    #[cfg_attr(feature = "servo", derive(MallocSizeOf))]
    /// Flags of variant alternates in bit
    struct VariantAlternatesParsingFlags: u8 {
        /// None of variant alternates enabled
        const NORMAL = 0;
        /// Historical forms
        const HISTORICAL_FORMS = 0x01;
        /// Stylistic Alternates
        const STYLISTIC = 0x02;
        /// Stylistic Sets
        const STYLESET = 0x04;
        /// Character Variant
        const CHARACTER_VARIANT = 0x08;
        /// Swash glyphs
        const SWASH = 0x10;
        /// Ornaments glyphs
        const ORNAMENTS = 0x20;
        /// Annotation forms
        const ANNOTATION = 0x40;
    }
}

#[derive(
    Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToResolvedValue, ToShmem,
)]
#[repr(C, u8)]
/// Set of variant alternates
pub enum VariantAlternates {
    /// Enables display of stylistic alternates
    #[css(function)]
    Stylistic(CustomIdent),
    /// Enables display with stylistic sets
    #[css(comma, function)]
    Styleset(#[css(iterable)] crate::OwnedSlice<CustomIdent>),
    /// Enables display of specific character variants
    #[css(comma, function)]
    CharacterVariant(#[css(iterable)] crate::OwnedSlice<CustomIdent>),
    /// Enables display of swash glyphs
    #[css(function)]
    Swash(CustomIdent),
    /// Enables replacement of default glyphs with ornaments
    #[css(function)]
    Ornaments(CustomIdent),
    /// Enables display of alternate annotation forms
    #[css(function)]
    Annotation(CustomIdent),
    /// Enables display of historical forms
    HistoricalForms,
}

#[derive(
    Clone,
    Debug,
    Default,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
/// List of Variant Alternates
pub struct VariantAlternatesList(
    #[css(if_empty = "normal", iterable)] crate::OwnedSlice<VariantAlternates>,
);

impl VariantAlternatesList {
    /// Returns the length of all variant alternates.
    pub fn len(&self) -> usize {
        self.0.iter().fold(0, |acc, alternate| match *alternate {
            VariantAlternates::Swash(_) |
            VariantAlternates::Stylistic(_) |
            VariantAlternates::Ornaments(_) |
            VariantAlternates::Annotation(_) => acc + 1,
            VariantAlternates::Styleset(ref slice) |
            VariantAlternates::CharacterVariant(ref slice) => acc + slice.len(),
            _ => acc,
        })
    }
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
/// Control over the selection of these alternate glyphs
pub enum FontVariantAlternates {
    /// Use alternative glyph from value
    Value(VariantAlternatesList),
    /// Use system font glyph
    #[css(skip)]
    System(SystemFont),
}

impl FontVariantAlternates {
    #[inline]
    /// Get initial specified value with VariantAlternatesList
    pub fn get_initial_specified_value() -> Self {
        FontVariantAlternates::Value(Default::default())
    }

    system_font_methods!(FontVariantAlternates, font_variant_alternates);
}

impl ToComputedValue for FontVariantAlternates {
    type ComputedValue = computed::FontVariantAlternates;

    fn to_computed_value(&self, context: &Context) -> computed::FontVariantAlternates {
        match *self {
            FontVariantAlternates::Value(ref v) => v.clone(),
            FontVariantAlternates::System(_) => self.compute_system(context),
        }
    }

    fn from_computed_value(other: &computed::FontVariantAlternates) -> Self {
        FontVariantAlternates::Value(other.clone())
    }
}

impl Parse for FontVariantAlternates {
    /// normal |
    ///  [ stylistic(<feature-value-name>)           ||
    ///    historical-forms                          ||
    ///    styleset(<feature-value-name> #)          ||
    ///    character-variant(<feature-value-name> #) ||
    ///    swash(<feature-value-name>)               ||
    ///    ornaments(<feature-value-name>)           ||
    ///    annotation(<feature-value-name>) ]
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontVariantAlternates, ParseError<'i>> {
        if input
            .try(|input| input.expect_ident_matching("normal"))
            .is_ok()
        {
            return Ok(FontVariantAlternates::Value(Default::default()));
        }

        let mut alternates = Vec::new();
        let mut parsed_alternates = VariantAlternatesParsingFlags::empty();
        macro_rules! check_if_parsed(
            ($input:expr, $flag:path) => (
                if parsed_alternates.contains($flag) {
                    return Err($input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                }
                parsed_alternates |= $flag;
            )
        );
        while let Ok(_) = input.try(|input| match *input.next()? {
            Token::Ident(ref value) if value.eq_ignore_ascii_case("historical-forms") => {
                check_if_parsed!(input, VariantAlternatesParsingFlags::HISTORICAL_FORMS);
                alternates.push(VariantAlternates::HistoricalForms);
                Ok(())
            },
            Token::Function(ref name) => {
                let name = name.clone();
                input.parse_nested_block(|i| {
                    match_ignore_ascii_case! { &name,
                        "swash" => {
                            check_if_parsed!(i, VariantAlternatesParsingFlags::SWASH);
                            let location = i.current_source_location();
                            let ident = CustomIdent::from_ident(location, i.expect_ident()?, &[])?;
                            alternates.push(VariantAlternates::Swash(ident));
                            Ok(())
                        },
                        "stylistic" => {
                            check_if_parsed!(i, VariantAlternatesParsingFlags::STYLISTIC);
                            let location = i.current_source_location();
                            let ident = CustomIdent::from_ident(location, i.expect_ident()?, &[])?;
                            alternates.push(VariantAlternates::Stylistic(ident));
                            Ok(())
                        },
                        "ornaments" => {
                            check_if_parsed!(i, VariantAlternatesParsingFlags::ORNAMENTS);
                            let location = i.current_source_location();
                            let ident = CustomIdent::from_ident(location, i.expect_ident()?, &[])?;
                            alternates.push(VariantAlternates::Ornaments(ident));
                            Ok(())
                        },
                        "annotation" => {
                            check_if_parsed!(i, VariantAlternatesParsingFlags::ANNOTATION);
                            let location = i.current_source_location();
                            let ident = CustomIdent::from_ident(location, i.expect_ident()?, &[])?;
                            alternates.push(VariantAlternates::Annotation(ident));
                            Ok(())
                        },
                        "styleset" => {
                            check_if_parsed!(i, VariantAlternatesParsingFlags::STYLESET);
                            let idents = i.parse_comma_separated(|i| {
                                let location = i.current_source_location();
                                CustomIdent::from_ident(location, i.expect_ident()?, &[])
                            })?;
                            alternates.push(VariantAlternates::Styleset(idents.into()));
                            Ok(())
                        },
                        "character-variant" => {
                            check_if_parsed!(i, VariantAlternatesParsingFlags::CHARACTER_VARIANT);
                            let idents = i.parse_comma_separated(|i| {
                                let location = i.current_source_location();
                                CustomIdent::from_ident(location, i.expect_ident()?, &[])
                            })?;
                            alternates.push(VariantAlternates::CharacterVariant(idents.into()));
                            Ok(())
                        },
                        _ => return Err(i.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
                    }
                })
            },
            _ => Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
        }) {}

        if parsed_alternates.is_empty() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        Ok(FontVariantAlternates::Value(VariantAlternatesList(
            alternates.into(),
        )))
    }
}

macro_rules! impl_variant_east_asian {
    {
        $(
            $(#[$($meta:tt)+])*
            $ident:ident / $css:expr => $gecko:ident = $value:expr,
        )+
    } => {
        bitflags! {
            #[derive(MallocSizeOf, ToResolvedValue, ToShmem)]
            /// Vairants for east asian variant
            pub struct VariantEastAsian: u16 {
                /// None of the features
                const NORMAL = 0;
                $(
                    $(#[$($meta)+])*
                    const $ident = $value;
                )+
            }
        }

        impl ToCss for VariantEastAsian {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
            where
                W: Write,
            {
                if self.is_empty() {
                    return dest.write_str("normal");
                }

                let mut writer = SequenceWriter::new(dest, " ");
                $(
                    if self.intersects(VariantEastAsian::$ident) {
                        writer.raw_item($css)?;
                    }
                )+
                Ok(())
            }
        }

        /// Asserts that all variant-east-asian matches its NS_FONT_VARIANT_EAST_ASIAN_* value.
        #[cfg(feature = "gecko")]
        #[inline]
        pub fn assert_variant_east_asian_matches() {
            use crate::gecko_bindings::structs;
            $(
                debug_assert_eq!(structs::$gecko as u16, VariantEastAsian::$ident.bits());
            )+
        }

        impl SpecifiedValueInfo for VariantEastAsian {
            fn collect_completion_keywords(f: KeywordsCollectFn) {
                f(&["normal", $($css,)+]);
            }
        }
    }
}

impl_variant_east_asian! {
    /// Enables rendering of JIS78 forms (OpenType feature: jp78)
    JIS78 / "jis78" => NS_FONT_VARIANT_EAST_ASIAN_JIS78 = 0x01,
    /// Enables rendering of JIS83 forms (OpenType feature: jp83).
    JIS83 / "jis83" => NS_FONT_VARIANT_EAST_ASIAN_JIS83 = 0x02,
    /// Enables rendering of JIS90 forms (OpenType feature: jp90).
    JIS90 / "jis90" => NS_FONT_VARIANT_EAST_ASIAN_JIS90 = 0x04,
    /// Enables rendering of JIS2004 forms (OpenType feature: jp04).
    JIS04 / "jis04" => NS_FONT_VARIANT_EAST_ASIAN_JIS04 = 0x08,
    /// Enables rendering of simplified forms (OpenType feature: smpl).
    SIMPLIFIED / "simplified" => NS_FONT_VARIANT_EAST_ASIAN_SIMPLIFIED = 0x10,
    /// Enables rendering of traditional forms (OpenType feature: trad).
    TRADITIONAL / "traditional" => NS_FONT_VARIANT_EAST_ASIAN_TRADITIONAL = 0x20,
    /// Enables rendering of full-width variants (OpenType feature: fwid).
    FULL_WIDTH / "full-width" => NS_FONT_VARIANT_EAST_ASIAN_FULL_WIDTH = 0x40,
    /// Enables rendering of proportionally-spaced variants (OpenType feature: pwid).
    PROPORTIONAL_WIDTH / "proportional-width" => NS_FONT_VARIANT_EAST_ASIAN_PROP_WIDTH = 0x80,
    /// Enables display of ruby variant glyphs (OpenType feature: ruby).
    RUBY / "ruby" => NS_FONT_VARIANT_EAST_ASIAN_RUBY = 0x100,
}

#[cfg(feature = "gecko")]
impl VariantEastAsian {
    /// Obtain a specified value from a Gecko keyword value
    ///
    /// Intended for use with presentation attributes, not style structs
    pub fn from_gecko_keyword(kw: u16) -> Self {
        Self::from_bits_truncate(kw)
    }

    /// Transform into gecko keyword
    pub fn to_gecko_keyword(self) -> u16 {
        self.bits()
    }
}

#[cfg(feature = "gecko")]
impl_gecko_keyword_conversions!(VariantEastAsian, u16);

#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
/// Allows control of glyph substitution and sizing in East Asian text.
pub enum FontVariantEastAsian {
    /// Value variant with `variant-east-asian`
    Value(VariantEastAsian),
    /// System font variant
    #[css(skip)]
    System(SystemFont),
}

impl FontVariantEastAsian {
    #[inline]
    /// Get default `font-variant-east-asian` with `empty` variant
    pub fn empty() -> Self {
        FontVariantEastAsian::Value(VariantEastAsian::empty())
    }

    system_font_methods!(FontVariantEastAsian, font_variant_east_asian);
}

impl ToComputedValue for FontVariantEastAsian {
    type ComputedValue = computed::FontVariantEastAsian;

    fn to_computed_value(&self, context: &Context) -> computed::FontVariantEastAsian {
        match *self {
            FontVariantEastAsian::Value(ref v) => v.clone(),
            FontVariantEastAsian::System(_) => self.compute_system(context),
        }
    }

    fn from_computed_value(other: &computed::FontVariantEastAsian) -> Self {
        FontVariantEastAsian::Value(other.clone())
    }
}

impl Parse for FontVariantEastAsian {
    /// normal | [ <east-asian-variant-values> || <east-asian-width-values> || ruby ]
    /// <east-asian-variant-values> = [ jis78 | jis83 | jis90 | jis04 | simplified | traditional ]
    /// <east-asian-width-values>   = [ full-width | proportional-width ]
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontVariantEastAsian, ParseError<'i>> {
        let mut result = VariantEastAsian::empty();

        if input
            .try(|input| input.expect_ident_matching("normal"))
            .is_ok()
        {
            return Ok(FontVariantEastAsian::Value(result));
        }

        while let Ok(flag) = input.try(|input| {
            Ok(
                match_ignore_ascii_case! { &input.expect_ident().map_err(|_| ())?,
                    "jis78" =>
                        exclusive_value!((result, VariantEastAsian::JIS78 | VariantEastAsian::JIS83 |
                                                  VariantEastAsian::JIS90 | VariantEastAsian::JIS04 |
                                                  VariantEastAsian::SIMPLIFIED | VariantEastAsian::TRADITIONAL
                                        ) => VariantEastAsian::JIS78),
                    "jis83" =>
                        exclusive_value!((result, VariantEastAsian::JIS78 | VariantEastAsian::JIS83 |
                                                  VariantEastAsian::JIS90 | VariantEastAsian::JIS04 |
                                                  VariantEastAsian::SIMPLIFIED | VariantEastAsian::TRADITIONAL
                                        ) => VariantEastAsian::JIS83),
                    "jis90" =>
                        exclusive_value!((result, VariantEastAsian::JIS78 | VariantEastAsian::JIS83 |
                                                  VariantEastAsian::JIS90 | VariantEastAsian::JIS04 |
                                                  VariantEastAsian::SIMPLIFIED | VariantEastAsian::TRADITIONAL
                                        ) => VariantEastAsian::JIS90),
                    "jis04" =>
                        exclusive_value!((result, VariantEastAsian::JIS78 | VariantEastAsian::JIS83 |
                                                  VariantEastAsian::JIS90 | VariantEastAsian::JIS04 |
                                                  VariantEastAsian::SIMPLIFIED | VariantEastAsian::TRADITIONAL
                                        ) => VariantEastAsian::JIS04),
                    "simplified" =>
                        exclusive_value!((result, VariantEastAsian::JIS78 | VariantEastAsian::JIS83 |
                                                  VariantEastAsian::JIS90 | VariantEastAsian::JIS04 |
                                                  VariantEastAsian::SIMPLIFIED | VariantEastAsian::TRADITIONAL
                                        ) => VariantEastAsian::SIMPLIFIED),
                    "traditional" =>
                        exclusive_value!((result, VariantEastAsian::JIS78 | VariantEastAsian::JIS83 |
                                                  VariantEastAsian::JIS90 | VariantEastAsian::JIS04 |
                                                  VariantEastAsian::SIMPLIFIED | VariantEastAsian::TRADITIONAL
                                        ) => VariantEastAsian::TRADITIONAL),
                    "full-width" =>
                        exclusive_value!((result, VariantEastAsian::FULL_WIDTH |
                                                  VariantEastAsian::PROPORTIONAL_WIDTH
                                        ) => VariantEastAsian::FULL_WIDTH),
                    "proportional-width" =>
                        exclusive_value!((result, VariantEastAsian::FULL_WIDTH |
                                                  VariantEastAsian::PROPORTIONAL_WIDTH
                                        ) => VariantEastAsian::PROPORTIONAL_WIDTH),
                    "ruby" =>
                        exclusive_value!((result, VariantEastAsian::RUBY) => VariantEastAsian::RUBY),
                    _ => return Err(()),
                },
            )
        }) {
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(FontVariantEastAsian::Value(result))
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

macro_rules! impl_variant_ligatures {
    {
        $(
            $(#[$($meta:tt)+])*
            $ident:ident / $css:expr => $gecko:ident = $value:expr,
        )+
    } => {
        bitflags! {
            #[derive(MallocSizeOf, ToResolvedValue, ToShmem)]
            /// Variants of ligatures
            pub struct VariantLigatures: u16 {
                /// Specifies that common default features are enabled
                const NORMAL = 0;
                $(
                    $(#[$($meta)+])*
                    const $ident = $value;
                )+
            }
        }

        impl ToCss for VariantLigatures {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
            where
                W: Write,
            {
                if self.is_empty() {
                    return dest.write_str("normal");
                }
                if self.contains(VariantLigatures::NONE) {
                    return dest.write_str("none");
                }

                let mut writer = SequenceWriter::new(dest, " ");
                $(
                    if self.intersects(VariantLigatures::$ident) {
                        writer.raw_item($css)?;
                    }
                )+
                Ok(())
            }
        }

        /// Asserts that all variant-east-asian matches its NS_FONT_VARIANT_EAST_ASIAN_* value.
        #[cfg(feature = "gecko")]
        #[inline]
        pub fn assert_variant_ligatures_matches() {
            use crate::gecko_bindings::structs;
            $(
                debug_assert_eq!(structs::$gecko as u16, VariantLigatures::$ident.bits());
            )+
        }

        impl SpecifiedValueInfo for VariantLigatures {
            fn collect_completion_keywords(f: KeywordsCollectFn) {
                f(&["normal", $($css,)+]);
            }
        }
    }
}

impl_variant_ligatures! {
    /// Specifies that all types of ligatures and contextual forms
    /// covered by this property are explicitly disabled
    NONE / "none" => NS_FONT_VARIANT_LIGATURES_NONE = 0x01,
    /// Enables display of common ligatures
    COMMON_LIGATURES / "common-ligatures" => NS_FONT_VARIANT_LIGATURES_COMMON = 0x02,
    /// Disables display of common ligatures
    NO_COMMON_LIGATURES / "no-common-ligatures" => NS_FONT_VARIANT_LIGATURES_NO_COMMON = 0x04,
    /// Enables display of discretionary ligatures
    DISCRETIONARY_LIGATURES / "discretionary-ligatures" => NS_FONT_VARIANT_LIGATURES_DISCRETIONARY = 0x08,
    /// Disables display of discretionary ligatures
    NO_DISCRETIONARY_LIGATURES / "no-discretionary-ligatures" => NS_FONT_VARIANT_LIGATURES_NO_DISCRETIONARY = 0x10,
    /// Enables display of historical ligatures
    HISTORICAL_LIGATURES / "historical-ligatures" => NS_FONT_VARIANT_LIGATURES_HISTORICAL = 0x20,
    /// Disables display of historical ligatures
    NO_HISTORICAL_LIGATURES / "no-historical-ligatures" => NS_FONT_VARIANT_LIGATURES_NO_HISTORICAL = 0x40,
    /// Enables display of contextual alternates
    CONTEXTUAL / "contextual" => NS_FONT_VARIANT_LIGATURES_CONTEXTUAL = 0x80,
    /// Disables display of contextual alternates
    NO_CONTEXTUAL / "no-contextual" => NS_FONT_VARIANT_LIGATURES_NO_CONTEXTUAL = 0x100,
}

#[cfg(feature = "gecko")]
impl VariantLigatures {
    /// Obtain a specified value from a Gecko keyword value
    ///
    /// Intended for use with presentation attributes, not style structs
    pub fn from_gecko_keyword(kw: u16) -> Self {
        Self::from_bits_truncate(kw)
    }

    /// Transform into gecko keyword
    pub fn to_gecko_keyword(self) -> u16 {
        self.bits()
    }
}

#[cfg(feature = "gecko")]
impl_gecko_keyword_conversions!(VariantLigatures, u16);

#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
/// Ligatures and contextual forms are ways of combining glyphs
/// to produce more harmonized forms
pub enum FontVariantLigatures {
    /// Value variant with `variant-ligatures`
    Value(VariantLigatures),
    /// System font variant
    #[css(skip)]
    System(SystemFont),
}

impl FontVariantLigatures {
    system_font_methods!(FontVariantLigatures, font_variant_ligatures);

    /// Default value of `font-variant-ligatures` as `empty`
    #[inline]
    pub fn empty() -> FontVariantLigatures {
        FontVariantLigatures::Value(VariantLigatures::empty())
    }

    #[inline]
    /// Get `none` variant of `font-variant-ligatures`
    pub fn none() -> FontVariantLigatures {
        FontVariantLigatures::Value(VariantLigatures::NONE)
    }
}

impl ToComputedValue for FontVariantLigatures {
    type ComputedValue = computed::FontVariantLigatures;

    fn to_computed_value(&self, context: &Context) -> computed::FontVariantLigatures {
        match *self {
            FontVariantLigatures::Value(ref v) => v.clone(),
            FontVariantLigatures::System(_) => self.compute_system(context),
        }
    }

    fn from_computed_value(other: &computed::FontVariantLigatures) -> Self {
        FontVariantLigatures::Value(other.clone())
    }
}

impl Parse for FontVariantLigatures {
    /// normal | none |
    /// [ <common-lig-values> ||
    ///   <discretionary-lig-values> ||
    ///   <historical-lig-values> ||
    ///   <contextual-alt-values> ]
    /// <common-lig-values>        = [ common-ligatures | no-common-ligatures ]
    /// <discretionary-lig-values> = [ discretionary-ligatures | no-discretionary-ligatures ]
    /// <historical-lig-values>    = [ historical-ligatures | no-historical-ligatures ]
    /// <contextual-alt-values>    = [ contextual | no-contextual ]
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontVariantLigatures, ParseError<'i>> {
        let mut result = VariantLigatures::empty();

        if input
            .try(|input| input.expect_ident_matching("normal"))
            .is_ok()
        {
            return Ok(FontVariantLigatures::Value(result));
        }
        if input
            .try(|input| input.expect_ident_matching("none"))
            .is_ok()
        {
            return Ok(FontVariantLigatures::Value(VariantLigatures::NONE));
        }

        while let Ok(flag) = input.try(|input| {
            Ok(
                match_ignore_ascii_case! { &input.expect_ident().map_err(|_| ())?,
                    "common-ligatures" =>
                        exclusive_value!((result, VariantLigatures::COMMON_LIGATURES |
                                                  VariantLigatures::NO_COMMON_LIGATURES
                                        ) => VariantLigatures::COMMON_LIGATURES),
                    "no-common-ligatures" =>
                        exclusive_value!((result, VariantLigatures::COMMON_LIGATURES |
                                                  VariantLigatures::NO_COMMON_LIGATURES
                                        ) => VariantLigatures::NO_COMMON_LIGATURES),
                    "discretionary-ligatures" =>
                        exclusive_value!((result, VariantLigatures::DISCRETIONARY_LIGATURES |
                                                  VariantLigatures::NO_DISCRETIONARY_LIGATURES
                                        ) => VariantLigatures::DISCRETIONARY_LIGATURES),
                    "no-discretionary-ligatures" =>
                        exclusive_value!((result, VariantLigatures::DISCRETIONARY_LIGATURES |
                                                  VariantLigatures::NO_DISCRETIONARY_LIGATURES
                                        ) => VariantLigatures::NO_DISCRETIONARY_LIGATURES),
                    "historical-ligatures" =>
                        exclusive_value!((result, VariantLigatures::HISTORICAL_LIGATURES |
                                                  VariantLigatures::NO_HISTORICAL_LIGATURES
                                        ) => VariantLigatures::HISTORICAL_LIGATURES),
                    "no-historical-ligatures" =>
                        exclusive_value!((result, VariantLigatures::HISTORICAL_LIGATURES |
                                                  VariantLigatures::NO_HISTORICAL_LIGATURES
                                        ) => VariantLigatures::NO_HISTORICAL_LIGATURES),
                    "contextual" =>
                        exclusive_value!((result, VariantLigatures::CONTEXTUAL |
                                                  VariantLigatures::NO_CONTEXTUAL
                                        ) => VariantLigatures::CONTEXTUAL),
                    "no-contextual" =>
                        exclusive_value!((result, VariantLigatures::CONTEXTUAL |
                                                  VariantLigatures::NO_CONTEXTUAL
                                        ) => VariantLigatures::NO_CONTEXTUAL),
                    _ => return Err(()),
                },
            )
        }) {
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(FontVariantLigatures::Value(result))
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

macro_rules! impl_variant_numeric {
    {
        $(
            $(#[$($meta:tt)+])*
            $ident:ident / $css:expr => $gecko:ident = $value:expr,
        )+
    } => {
        bitflags! {
            #[derive(MallocSizeOf, ToResolvedValue, ToShmem)]
            /// Vairants of numeric values
            pub struct VariantNumeric: u8 {
                /// None of other variants are enabled.
                const NORMAL = 0;
                $(
                    $(#[$($meta)+])*
                    const $ident = $value;
                )+
            }
        }

        impl ToCss for VariantNumeric {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
            where
                W: Write,
            {
                if self.is_empty() {
                    return dest.write_str("normal");
                }

                let mut writer = SequenceWriter::new(dest, " ");
                $(
                    if self.intersects(VariantNumeric::$ident) {
                        writer.raw_item($css)?;
                    }
                )+
                Ok(())
            }
        }

        /// Asserts that all variant-east-asian matches its NS_FONT_VARIANT_EAST_ASIAN_* value.
        #[cfg(feature = "gecko")]
        #[inline]
        pub fn assert_variant_numeric_matches() {
            use crate::gecko_bindings::structs;
            $(
                debug_assert_eq!(structs::$gecko as u8, VariantNumeric::$ident.bits());
            )+
        }

        impl SpecifiedValueInfo for VariantNumeric {
            fn collect_completion_keywords(f: KeywordsCollectFn) {
                f(&["normal", $($css,)+]);
            }
        }
    }
}

impl_variant_numeric! {
    /// Enables display of lining numerals.
    LINING_NUMS / "lining-nums" => NS_FONT_VARIANT_NUMERIC_LINING = 0x01,
    /// Enables display of old-style numerals.
    OLDSTYLE_NUMS / "oldstyle-nums" => NS_FONT_VARIANT_NUMERIC_OLDSTYLE = 0x02,
    /// Enables display of proportional numerals.
    PROPORTIONAL_NUMS / "proportional-nums" => NS_FONT_VARIANT_NUMERIC_PROPORTIONAL = 0x04,
    /// Enables display of tabular numerals.
    TABULAR_NUMS / "tabular-nums" => NS_FONT_VARIANT_NUMERIC_TABULAR = 0x08,
    /// Enables display of lining diagonal fractions.
    DIAGONAL_FRACTIONS / "diagonal-fractions" => NS_FONT_VARIANT_NUMERIC_DIAGONAL_FRACTIONS = 0x10,
    /// Enables display of lining stacked fractions.
    STACKED_FRACTIONS / "stacked-fractions" => NS_FONT_VARIANT_NUMERIC_STACKED_FRACTIONS = 0x20,
    /// Enables display of letter forms used with ordinal numbers.
    ORDINAL / "ordinal" => NS_FONT_VARIANT_NUMERIC_ORDINAL = 0x80,
    /// Enables display of slashed zeros.
    SLASHED_ZERO / "slashed-zero" => NS_FONT_VARIANT_NUMERIC_SLASHZERO = 0x40,
}

#[cfg(feature = "gecko")]
impl VariantNumeric {
    /// Obtain a specified value from a Gecko keyword value
    ///
    /// Intended for use with presentation attributes, not style structs
    pub fn from_gecko_keyword(kw: u8) -> Self {
        Self::from_bits_truncate(kw)
    }

    /// Transform into gecko keyword
    pub fn to_gecko_keyword(self) -> u8 {
        self.bits()
    }
}

#[cfg(feature = "gecko")]
impl_gecko_keyword_conversions!(VariantNumeric, u8);

#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
/// Specifies control over numerical forms.
pub enum FontVariantNumeric {
    /// Value variant with `variant-numeric`
    Value(VariantNumeric),
    /// System font
    #[css(skip)]
    System(SystemFont),
}

impl FontVariantNumeric {
    #[inline]
    /// Default value of `font-variant-numeric` as `empty`
    pub fn empty() -> FontVariantNumeric {
        FontVariantNumeric::Value(VariantNumeric::empty())
    }

    system_font_methods!(FontVariantNumeric, font_variant_numeric);
}

impl ToComputedValue for FontVariantNumeric {
    type ComputedValue = computed::FontVariantNumeric;

    fn to_computed_value(&self, context: &Context) -> computed::FontVariantNumeric {
        match *self {
            FontVariantNumeric::Value(ref v) => v.clone(),
            FontVariantNumeric::System(_) => self.compute_system(context),
        }
    }

    fn from_computed_value(other: &computed::FontVariantNumeric) -> Self {
        FontVariantNumeric::Value(other.clone())
    }
}

impl Parse for FontVariantNumeric {
    /// normal |
    ///  [ <numeric-figure-values>   ||
    ///    <numeric-spacing-values>  ||
    ///    <numeric-fraction-values> ||
    ///    ordinal                   ||
    ///    slashed-zero ]
    /// <numeric-figure-values>   = [ lining-nums | oldstyle-nums ]
    /// <numeric-spacing-values>  = [ proportional-nums | tabular-nums ]
    /// <numeric-fraction-values> = [ diagonal-fractions | stacked-fractions ]
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontVariantNumeric, ParseError<'i>> {
        let mut result = VariantNumeric::empty();

        if input
            .try(|input| input.expect_ident_matching("normal"))
            .is_ok()
        {
            return Ok(FontVariantNumeric::Value(result));
        }

        while let Ok(flag) = input.try(|input| {
            Ok(
                match_ignore_ascii_case! { &input.expect_ident().map_err(|_| ())?,
                    "ordinal" =>
                        exclusive_value!((result, VariantNumeric::ORDINAL) => VariantNumeric::ORDINAL),
                    "slashed-zero" =>
                        exclusive_value!((result, VariantNumeric::SLASHED_ZERO) => VariantNumeric::SLASHED_ZERO),
                    "lining-nums" =>
                        exclusive_value!((result, VariantNumeric::LINING_NUMS |
                                                  VariantNumeric::OLDSTYLE_NUMS
                                        ) => VariantNumeric::LINING_NUMS),
                    "oldstyle-nums" =>
                        exclusive_value!((result, VariantNumeric::LINING_NUMS |
                                                  VariantNumeric::OLDSTYLE_NUMS
                                        ) => VariantNumeric::OLDSTYLE_NUMS),
                    "proportional-nums" =>
                        exclusive_value!((result, VariantNumeric::PROPORTIONAL_NUMS |
                                                  VariantNumeric::TABULAR_NUMS
                                        ) => VariantNumeric::PROPORTIONAL_NUMS),
                    "tabular-nums" =>
                        exclusive_value!((result, VariantNumeric::PROPORTIONAL_NUMS |
                                                  VariantNumeric::TABULAR_NUMS
                                        ) => VariantNumeric::TABULAR_NUMS),
                    "diagonal-fractions" =>
                        exclusive_value!((result, VariantNumeric::DIAGONAL_FRACTIONS |
                                                  VariantNumeric::STACKED_FRACTIONS
                                        ) => VariantNumeric::DIAGONAL_FRACTIONS),
                    "stacked-fractions" =>
                        exclusive_value!((result, VariantNumeric::DIAGONAL_FRACTIONS |
                                                  VariantNumeric::STACKED_FRACTIONS
                                        ) => VariantNumeric::STACKED_FRACTIONS),
                    _ => return Err(()),
                },
            )
        }) {
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(FontVariantNumeric::Value(result))
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

/// This property provides low-level control over OpenType or TrueType font features.
pub type SpecifiedFontFeatureSettings = FontSettings<FeatureTagValue<Integer>>;

/// Define initial settings that apply when the font defined by an @font-face
/// rule is rendered.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub enum FontFeatureSettings {
    /// Value of `FontSettings`
    Value(SpecifiedFontFeatureSettings),
    /// System font
    #[css(skip)]
    System(SystemFont),
}

impl FontFeatureSettings {
    #[inline]
    /// Get default value of `font-feature-settings` as normal
    pub fn normal() -> FontFeatureSettings {
        FontFeatureSettings::Value(FontSettings::normal())
    }

    system_font_methods!(FontFeatureSettings, font_feature_settings);
}

impl ToComputedValue for FontFeatureSettings {
    type ComputedValue = computed::FontFeatureSettings;

    fn to_computed_value(&self, context: &Context) -> computed::FontFeatureSettings {
        match *self {
            FontFeatureSettings::Value(ref v) => v.to_computed_value(context),
            FontFeatureSettings::System(_) => self.compute_system(context),
        }
    }

    fn from_computed_value(other: &computed::FontFeatureSettings) -> Self {
        FontFeatureSettings::Value(ToComputedValue::from_computed_value(other))
    }
}

impl Parse for FontFeatureSettings {
    /// normal | <feature-tag-value>#
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontFeatureSettings, ParseError<'i>> {
        SpecifiedFontFeatureSettings::parse(context, input).map(FontFeatureSettings::Value)
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
/// Whether user agents are allowed to synthesize bold or oblique font faces
/// when a font family lacks bold or italic faces
pub struct FontSynthesis {
    /// If a `font-weight` is requested that the font family does not contain,
    /// the user agent may synthesize the requested weight from the weights
    /// that do exist in the font family.
    #[css(represents_keyword)]
    pub weight: bool,
    /// If a font-style is requested that the font family does not contain,
    /// the user agent may synthesize the requested style from the normal face in the font family.
    #[css(represents_keyword)]
    pub style: bool,
}

impl FontSynthesis {
    #[inline]
    /// Get the default value of font-synthesis
    pub fn get_initial_value() -> Self {
        FontSynthesis {
            weight: true,
            style: true,
        }
    }
}

impl Parse for FontSynthesis {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontSynthesis, ParseError<'i>> {
        let mut result = FontSynthesis {
            weight: false,
            style: false,
        };
        try_match_ident_ignore_ascii_case! { input,
            "none" => Ok(result),
            "weight" => {
                result.weight = true;
                if input.try(|input| input.expect_ident_matching("style")).is_ok() {
                    result.style = true;
                }
                Ok(result)
            },
            "style" => {
                result.style = true;
                if input.try(|input| input.expect_ident_matching("weight")).is_ok() {
                    result.weight = true;
                }
                Ok(result)
            },
        }
    }
}

impl ToCss for FontSynthesis {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.weight && self.style {
            dest.write_str("weight style")
        } else if self.style {
            dest.write_str("style")
        } else if self.weight {
            dest.write_str("weight")
        } else {
            dest.write_str("none")
        }
    }
}

#[cfg(feature = "gecko")]
impl From<u8> for FontSynthesis {
    fn from(bits: u8) -> FontSynthesis {
        use crate::gecko_bindings::structs;

        FontSynthesis {
            weight: bits & structs::NS_FONT_SYNTHESIS_WEIGHT as u8 != 0,
            style: bits & structs::NS_FONT_SYNTHESIS_STYLE as u8 != 0,
        }
    }
}

#[cfg(feature = "gecko")]
impl From<FontSynthesis> for u8 {
    fn from(v: FontSynthesis) -> u8 {
        use crate::gecko_bindings::structs;

        let mut bits: u8 = 0;
        if v.weight {
            bits |= structs::NS_FONT_SYNTHESIS_WEIGHT as u8;
        }
        if v.style {
            bits |= structs::NS_FONT_SYNTHESIS_STYLE as u8;
        }
        bits
    }
}

#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
/// Allows authors to explicitly specify the language system of the font,
/// overriding the language system implied by the content language
pub enum FontLanguageOverride {
    /// When rendering with OpenType fonts,
    /// the content language of the element is
    /// used to infer the OpenType language system
    Normal,
    /// Single three-letter case-sensitive OpenType language system tag,
    /// specifies the OpenType language system to be used instead of
    /// the language system implied by the language of the element
    Override(Box<str>),
    /// Use system font
    #[css(skip)]
    System(SystemFont),
}

impl FontLanguageOverride {
    #[inline]
    /// Get default value with `normal`
    pub fn normal() -> FontLanguageOverride {
        FontLanguageOverride::Normal
    }

    /// The ToComputedValue implementation for non-system-font
    /// FontLanguageOverride, used for @font-face descriptors.
    #[inline]
    pub fn compute_non_system(&self) -> computed::FontLanguageOverride {
        match *self {
            FontLanguageOverride::Normal => computed::FontLanguageOverride::zero(),
            FontLanguageOverride::Override(ref lang) => {
                computed::FontLanguageOverride::from_str(lang)
            },
            FontLanguageOverride::System(..) => unreachable!(),
        }
    }

    system_font_methods!(FontLanguageOverride, font_language_override);
}

impl ToComputedValue for FontLanguageOverride {
    type ComputedValue = computed::FontLanguageOverride;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> computed::FontLanguageOverride {
        match *self {
            FontLanguageOverride::System(_) => self.compute_system(context),
            _ => self.compute_non_system(),
        }
    }
    #[inline]
    fn from_computed_value(computed: &computed::FontLanguageOverride) -> Self {
        if *computed == computed::FontLanguageOverride::zero() {
            return FontLanguageOverride::Normal;
        }
        FontLanguageOverride::Override(computed.to_str(&mut [0; 4]).into())
    }
}

impl Parse for FontLanguageOverride {
    /// normal | <string>
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontLanguageOverride, ParseError<'i>> {
        if input
            .try(|input| input.expect_ident_matching("normal"))
            .is_ok()
        {
            return Ok(FontLanguageOverride::Normal);
        }

        let string = input.expect_string()?;
        Ok(FontLanguageOverride::Override(
            string.as_ref().to_owned().into_boxed_str(),
        ))
    }
}

/// This property provides low-level control over OpenType or TrueType font
/// variations.
pub type SpecifiedFontVariationSettings = FontSettings<VariationValue<Number>>;

/// Define initial settings that apply when the font defined by an @font-face
/// rule is rendered.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub enum FontVariationSettings {
    /// Value of `FontSettings`
    Value(SpecifiedFontVariationSettings),
    /// System font
    #[css(skip)]
    System(SystemFont),
}

impl FontVariationSettings {
    #[inline]
    /// Get default value of `font-variation-settings` as normal
    pub fn normal() -> FontVariationSettings {
        FontVariationSettings::Value(FontSettings::normal())
    }

    system_font_methods!(FontVariationSettings, font_variation_settings);
}

impl ToComputedValue for FontVariationSettings {
    type ComputedValue = computed::FontVariationSettings;

    fn to_computed_value(&self, context: &Context) -> computed::FontVariationSettings {
        match *self {
            FontVariationSettings::Value(ref v) => v.to_computed_value(context),
            FontVariationSettings::System(_) => self.compute_system(context),
        }
    }

    fn from_computed_value(other: &computed::FontVariationSettings) -> Self {
        FontVariationSettings::Value(ToComputedValue::from_computed_value(other))
    }
}

impl Parse for FontVariationSettings {
    /// normal | <variation-tag-value>#
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontVariationSettings, ParseError<'i>> {
        SpecifiedFontVariationSettings::parse(context, input).map(FontVariationSettings::Value)
    }
}

fn parse_one_feature_value<'i, 't>(
    context: &ParserContext,
    input: &mut Parser<'i, 't>,
) -> Result<Integer, ParseError<'i>> {
    if let Ok(integer) = input.try(|i| Integer::parse_non_negative(context, i)) {
        return Ok(integer);
    }

    try_match_ident_ignore_ascii_case! { input,
        "on" => Ok(Integer::new(1)),
        "off" => Ok(Integer::new(0)),
    }
}

impl Parse for FeatureTagValue<Integer> {
    /// https://drafts.csswg.org/css-fonts-4/#feature-tag-value
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let tag = FontTag::parse(context, input)?;
        let value = input
            .try(|i| parse_one_feature_value(context, i))
            .unwrap_or_else(|_| Integer::new(1));

        Ok(Self { tag, value })
    }
}

impl Parse for VariationValue<Number> {
    /// This is the `<string> <number>` part of the font-variation-settings
    /// syntax.
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let tag = FontTag::parse(context, input)?;
        let value = Number::parse(context, input)?;
        Ok(Self { tag, value })
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
/// text-zoom. Enable if true, disable if false
pub struct XTextZoom(#[css(skip)] pub bool);

impl Parse for XTextZoom {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<XTextZoom, ParseError<'i>> {
        debug_assert!(
            false,
            "Should be set directly by presentation attributes only."
        );
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
/// Internal property that reflects the lang attribute
pub struct XLang(#[css(skip)] pub Atom);

impl XLang {
    #[inline]
    /// Get default value for `-x-lang`
    pub fn get_initial_value() -> XLang {
        XLang(atom!(""))
    }
}

impl Parse for XLang {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<XLang, ParseError<'i>> {
        debug_assert!(
            false,
            "Should be set directly by presentation attributes only."
        );
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
/// Specifies the minimum font size allowed due to changes in scriptlevel.
/// Ref: https://wiki.mozilla.org/MathML:mstyle
pub struct MozScriptMinSize(pub NoCalcLength);

impl MozScriptMinSize {
    #[inline]
    /// Calculate initial value of -moz-script-min-size.
    pub fn get_initial_value() -> Length {
        Length::new(DEFAULT_SCRIPT_MIN_SIZE_PT as f32 * (AU_PER_PT / AU_PER_PX))
    }
}

impl Parse for MozScriptMinSize {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<MozScriptMinSize, ParseError<'i>> {
        debug_assert!(
            false,
            "Should be set directly by presentation attributes only."
        );
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
/// Changes the scriptlevel in effect for the children.
/// Ref: https://wiki.mozilla.org/MathML:mstyle
///
/// The main effect of scriptlevel is to control the font size.
/// https://www.w3.org/TR/MathML3/chapter3.html#presm.scriptlevel
pub enum MozScriptLevel {
    /// Change `font-size` relatively.
    Relative(i32),
    /// Change `font-size` absolutely.
    ///
    /// Should only be serialized by presentation attributes, so even though
    /// serialization for this would look the same as for the `Relative`
    /// variant, it is unexposed, so no big deal.
    #[css(function)]
    MozAbsolute(i32),
    /// Change `font-size` automatically.
    Auto,
}

impl Parse for MozScriptLevel {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<MozScriptLevel, ParseError<'i>> {
        // We don't bother to handle calc here.
        if let Ok(i) = input.try(|i| i.expect_integer()) {
            return Ok(MozScriptLevel::Relative(i));
        }
        input.expect_ident_matching("auto")?;
        Ok(MozScriptLevel::Auto)
    }
}

#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
/// Specifies the multiplier to be used to adjust font size
/// due to changes in scriptlevel.
///
/// Ref: https://www.w3.org/TR/MathML3/chapter3.html#presm.mstyle.attrs
pub struct MozScriptSizeMultiplier(pub f32);

impl MozScriptSizeMultiplier {
    #[inline]
    /// Get default value of `-moz-script-size-multiplier`
    pub fn get_initial_value() -> MozScriptSizeMultiplier {
        MozScriptSizeMultiplier(DEFAULT_SCRIPT_SIZE_MULTIPLIER as f32)
    }
}

impl Parse for MozScriptSizeMultiplier {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<MozScriptSizeMultiplier, ParseError<'i>> {
        debug_assert!(
            false,
            "Should be set directly by presentation attributes only."
        );
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

impl From<f32> for MozScriptSizeMultiplier {
    fn from(v: f32) -> Self {
        MozScriptSizeMultiplier(v)
    }
}

impl From<MozScriptSizeMultiplier> for f32 {
    fn from(v: MozScriptSizeMultiplier) -> f32 {
        v.0
    }
}
