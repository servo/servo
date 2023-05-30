/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified values for font properties

#[cfg(feature = "gecko")]
use crate::context::QuirksMode;
use crate::parser::{Parse, ParserContext};
use crate::values::computed::font::{FamilyName, FontFamilyList, SingleFontFamily};
use crate::values::computed::Percentage as ComputedPercentage;
use crate::values::computed::{font as computed, Length, NonNegativeLength};
use crate::values::computed::{CSSPixelLength, Context, ToComputedValue};
use crate::values::generics::font::VariationValue;
use crate::values::generics::font::{
    self as generics, FeatureTagValue, FontSettings, FontTag, GenericFontSizeAdjust,
};
use crate::values::generics::NonNegative;
use crate::values::specified::length::{FontBaseSize, PX_PER_PT};
use crate::values::specified::{AllowQuirks, Angle, Integer, LengthPercentage};
use crate::values::specified::{NoCalcLength, NonNegativeNumber, NonNegativePercentage, Number};
use crate::values::{serialize_atom_identifier, CustomIdent, SelectorParseErrorKind};
use crate::Atom;
use cssparser::{Parser, Token};
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps, MallocUnconditionalSizeOf};
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

/// System fonts.
#[repr(u8)]
#[derive(
    Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem
)]
#[allow(missing_docs)]
#[cfg(feature = "gecko")]
pub enum SystemFont {
    /// https://drafts.csswg.org/css-fonts/#valdef-font-caption
    Caption,
    /// https://drafts.csswg.org/css-fonts/#valdef-font-icon
    Icon,
    /// https://drafts.csswg.org/css-fonts/#valdef-font-menu
    Menu,
    /// https://drafts.csswg.org/css-fonts/#valdef-font-message-box
    MessageBox,
    /// https://drafts.csswg.org/css-fonts/#valdef-font-small-caption
    SmallCaption,
    /// https://drafts.csswg.org/css-fonts/#valdef-font-status-bar
    StatusBar,
    /// Internal system font, used by the `<menupopup>`s on macOS.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozPullDownMenu,
    /// Internal system font, used for `<button>` elements.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozButton,
    /// Internal font, used by `<select>` elements.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozList,
    /// Internal font, used by `<input>` elements.
    #[parse(condition = "ParserContext::in_ua_or_chrome_sheet")]
    MozField,
    #[css(skip)]
    End, // Just for indexing purposes.
}

// We don't parse system fonts in servo, but in the interest of not
// littering a lot of code with `if engine == "gecko"` conditionals,
// we have a dummy system font module that does nothing

#[derive(
    Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem
)]
#[allow(missing_docs)]
#[cfg(feature = "servo")]
/// void enum for system font, can never exist
pub enum SystemFont {}

#[allow(missing_docs)]
#[cfg(feature = "servo")]
impl SystemFont {
    pub fn parse(_: &mut Parser) -> Result<Self, ()> {
        Err(())
    }
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
#[derive(
    Clone, Copy, Debug, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem,
)]
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
            &computed.value(),
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
            AbsoluteFontWeight::Weight(weight) => computed::FontWeight::from_float(weight.get()),
            AbsoluteFontWeight::Normal => computed::FontWeight::NORMAL,
            AbsoluteFontWeight::Bold => computed::FontWeight::BOLD,
        }
    }
}

impl Parse for AbsoluteFontWeight {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(number) = input.try_parse(|input| Number::parse(context, input)) {
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
                let angle = input.try_parse(|input| Self::parse_angle(context, input))
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
            Self::Normal => computed::FontStyle::NORMAL,
            Self::Italic => computed::FontStyle::ITALIC,
            Self::Oblique(ref angle) => computed::FontStyle::oblique(angle.degrees()),
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        if *computed == computed::FontStyle::NORMAL {
            return Self::Normal;
        }
        if *computed == computed::FontStyle::ITALIC {
            return Self::Italic;
        }
        let degrees = computed.oblique_degrees();
        generics::FontStyle::Oblique(Angle::from_degrees(degrees, /* was_calc = */ false))
    }
}

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
            computed::FontStyle::DEFAULT_OBLIQUE_DEGREES as f32,
            /* was_calc = */ false,
        )
    }
}

/// The specified value of the `font-style` property.
#[derive(
    Clone, Copy, Debug, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem,
)]
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
#[allow(missing_docs)]
#[derive(
    Clone, Copy, Debug, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem,
)]
pub enum FontStretch {
    Stretch(NonNegativePercentage),
    Keyword(FontStretchKeyword),
    #[css(skip)]
    System(SystemFont),
}

/// A keyword value for `font-stretch`.
#[derive(
    Clone, Copy, Debug, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem,
)]
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
    /// Turns the keyword into a computed value.
    pub fn compute(&self) -> computed::FontStretch {
        computed::FontStretch::from_keyword(*self)
    }

    /// Does the opposite operation to `compute`, in order to serialize keywords
    /// if possible.
    pub fn from_percentage(p: f32) -> Option<Self> {
        computed::FontStretch::from_percentage(p).as_keyword()
    }
}

impl FontStretch {
    /// `normal`.
    pub fn normal() -> Self {
        FontStretch::Keyword(FontStretchKeyword::Normal)
    }

    system_font_methods!(FontStretch, font_stretch);
}

impl ToComputedValue for FontStretch {
    type ComputedValue = computed::FontStretch;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            FontStretch::Stretch(ref percentage) => {
                let percentage = percentage.to_computed_value(context).0;
                computed::FontStretch::from_percentage(percentage.0)
            },
            FontStretch::Keyword(ref kw) => kw.compute(),
            FontStretch::System(_) => self.compute_system(context),
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        FontStretch::Stretch(NonNegativePercentage::from_computed_value(&NonNegative(
            computed.to_percentage(),
        )))
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
    Serialize,
    Deserialize,
)]
#[allow(missing_docs)]
#[repr(u8)]
pub enum FontSizeKeyword {
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
    #[css(skip)]
    None,
}

impl FontSizeKeyword {
    /// Convert to an HTML <font size> value
    #[inline]
    pub fn html_size(self) -> u8 {
        self as u8
    }
}

impl Default for FontSizeKeyword {
    fn default() -> Self {
        FontSizeKeyword::Medium
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
#[cfg_attr(feature = "servo", derive(Serialize, Deserialize))]
/// Additional information for keyword-derived font sizes.
pub struct KeywordInfo {
    /// The keyword used
    pub kw: FontSizeKeyword,
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
        Self::new(FontSizeKeyword::Medium)
    }

    /// KeywordInfo value for font-size: none
    pub fn none() -> Self {
        Self::new(FontSizeKeyword::None)
    }

    fn new(kw: FontSizeKeyword) -> Self {
        KeywordInfo {
            kw,
            factor: 1.,
            offset: CSSPixelLength::new(0.),
        }
    }

    /// Computes the final size for this font-size keyword, accounting for
    /// text-zoom.
    fn to_computed_value(&self, context: &Context) -> CSSPixelLength {
        debug_assert_ne!(self.kw, FontSizeKeyword::None);
        let base = context.maybe_zoom_text(self.kw.to_length(context).0);
        base * self.factor + context.maybe_zoom_text(self.offset)
    }

    /// Given a parent keyword info (self), apply an additional factor/offset to
    /// it.
    fn compose(self, factor: f32) -> Self {
        if self.kw == FontSizeKeyword::None {
            return self;
        }
        KeywordInfo {
            kw: self.kw,
            factor: self.factor * factor,
            offset: self.offset * factor,
        }
    }
}

impl SpecifiedValueInfo for KeywordInfo {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        <FontSizeKeyword as SpecifiedValueInfo>::collect_completion_keywords(f);
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
}

impl ToComputedValue for FontFamily {
    type ComputedValue = computed::FontFamily;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            FontFamily::Values(ref list) => computed::FontFamily {
                families: list.clone(),
                is_system_font: false,
                is_initial: false,
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
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match *self {
            FontFamily::Values(ref v) => {
                // Although the family list is refcounted, we always attribute
                // its size to the specified value.
                v.list.unconditional_size_of(ops)
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
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontFamily, ParseError<'i>> {
        let values =
            input.parse_comma_separated(|input| SingleFontFamily::parse(context, input))?;
        Ok(FontFamily::Values(FontFamilyList {
            #[cfg(feature = "gecko")]
            list: crate::ArcSlice::from_iter(values.into_iter()),
            #[cfg(feature = "servo")]
            list: values.into_boxed_slice(),
        }))
    }
}

impl SpecifiedValueInfo for FontFamily {}

/// `FamilyName::parse` is based on `SingleFontFamily::parse` and not the other
/// way around because we want the former to exclude generic family keywords.
impl Parse for FamilyName {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        match SingleFontFamily::parse(context, input) {
            Ok(SingleFontFamily::FamilyName(name)) => Ok(name),
            Ok(SingleFontFamily::Generic(_)) => {
                Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            },
            Err(e) => Err(e),
        }
    }
}

/// Preserve the readability of text when font fallback occurs
pub type FontSizeAdjust = GenericFontSizeAdjust<NonNegativeNumber>;

impl Parse for FontSizeAdjust {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        if let Ok(ident) = input.try_parse(|i| i.expect_ident_cloned()) {
            #[cfg(feature = "gecko")]
            let basis_enabled = static_prefs::pref!("layout.css.font-size-adjust.basis.enabled");
            #[cfg(feature = "servo")]
            let basis_enabled = false;
            let basis = match_ignore_ascii_case! { &ident,
                "none" => return Ok(Self::None),
                // Check for size adjustment basis keywords if enabled.
                "ex-height" if basis_enabled => Self::ExHeight,
                "cap-height" if basis_enabled => Self::CapHeight,
                "ch-width" if basis_enabled => Self::ChWidth,
                "ic-width" if basis_enabled => Self::IcWidth,
                "ic-height" if basis_enabled => Self::IcHeight,
                // Unknown (or disabled) keyword.
                _ => return Err(location.new_custom_error(
                    SelectorParseErrorKind::UnexpectedIdent(ident)
                )),
            };
            let value = NonNegativeNumber::parse(context, input)?;
            return Ok(basis(value));
        }
        // Without a basis keyword, the number refers to the 'ex-height' metric.
        let value = NonNegativeNumber::parse(context, input)?;
        Ok(Self::ExHeight(value))
    }
}

/// This is the ratio applied for font-size: larger
/// and smaller by both Firefox and Chrome
const LARGER_FONT_SIZE_RATIO: f32 = 1.2;

/// The default font size.
pub const FONT_MEDIUM_PX: f32 = 16.0;

impl FontSizeKeyword {
    #[inline]
    #[cfg(feature = "servo")]
    fn to_length(&self, _: &Context) -> NonNegativeLength {
        let medium = Length::new(FONT_MEDIUM_PX);
        // https://drafts.csswg.org/css-fonts-3/#font-size-prop
        NonNegative(match *self {
            FontSizeKeyword::XXSmall => medium * 3.0 / 5.0,
            FontSizeKeyword::XSmall => medium * 3.0 / 4.0,
            FontSizeKeyword::Small => medium * 8.0 / 9.0,
            FontSizeKeyword::Medium => medium,
            FontSizeKeyword::Large => medium * 6.0 / 5.0,
            FontSizeKeyword::XLarge => medium * 3.0 / 2.0,
            FontSizeKeyword::XXLarge => medium * 2.0,
            FontSizeKeyword::XXXLarge => medium * 3.0,
            FontSizeKeyword::None => unreachable!(),
        })
    }

    #[cfg(feature = "gecko")]
    #[inline]
    fn to_length(&self, cx: &Context) -> NonNegativeLength {
        let font = cx.style().get_font();
        let family = &font.mFont.family.families;
        let generic = family
            .single_generic()
            .unwrap_or(computed::GenericFontFamily::None);
        let base_size = unsafe {
            Atom::with(font.mLanguage.mRawPtr, |language| {
                cx.device().base_size_for_generic(language, generic)
            })
        };
        self.to_length_without_context(cx.quirks_mode, base_size)
    }

    /// Resolve a keyword length without any context, with explicit arguments.
    #[cfg(feature = "gecko")]
    #[inline]
    pub fn to_length_without_context(
        &self,
        quirks_mode: QuirksMode,
        base_size: Length,
    ) -> NonNegativeLength {
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
        let base_size_px = base_size.px().round() as i32;
        let html_size = self.html_size() as usize;
        NonNegative(if base_size_px >= 9 && base_size_px <= 16 {
            let mapping = if quirks_mode == QuirksMode::Quirks {
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
            0 | 1 => FontSizeKeyword::XSmall,
            2 => FontSizeKeyword::Small,
            3 => FontSizeKeyword::Medium,
            4 => FontSizeKeyword::Large,
            5 => FontSizeKeyword::XLarge,
            6 => FontSizeKeyword::XXLarge,
            // If value is greater than 7, let it be 7.
            _ => FontSizeKeyword::XXXLarge,
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
                .compose(factor)
        };
        let mut info = KeywordInfo::none();
        let size = match *self {
            FontSize::Length(LengthPercentage::Length(ref l)) => {
                if let NoCalcLength::FontRelative(ref value) = *l {
                    if let FontRelativeLength::Em(em) = *value {
                        // If the parent font was keyword-derived, this is
                        // too. Tack the em unit onto the factor
                        info = compose_keyword(em);
                    }
                }
                let result = l.to_computed_value_with_base_size(context, base_size);
                if l.should_zoom_text() {
                    context.maybe_zoom_text(result)
                } else {
                    result
                }
            },
            FontSize::Length(LengthPercentage::Percentage(pc)) => {
                // If the parent font was keyword-derived, this is too.
                // Tack the % onto the factor
                info = compose_keyword(pc.0);
                (base_size.resolve(context).computed_size() * pc.0).normalized()
            },
            FontSize::Length(LengthPercentage::Calc(ref calc)) => {
                let calc = calc.to_computed_value_zoomed(context, base_size);
                calc.resolve(base_size.resolve(context).computed_size())
            },
            FontSize::Keyword(i) => {
                // As a specified keyword, this is keyword derived
                info = i;
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
                        .computed_size()
                }
            },
        };
        computed::FontSize {
            computed_size: NonNegative(size),
            used_size: NonNegative(size),
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
            ToComputedValue::from_computed_value(&computed.computed_size()),
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
        if let Ok(lp) = input
            .try_parse(|i| LengthPercentage::parse_non_negative_quirky(context, i, allow_quirks))
        {
            return Ok(FontSize::Length(lp));
        }

        if let Ok(kw) = input.try_parse(FontSizeKeyword::parse) {
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
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToCss,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
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
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
/// List of Variant Alternates
pub struct FontVariantAlternates(
    #[css(if_empty = "normal", iterable)] crate::OwnedSlice<VariantAlternates>,
);

impl FontVariantAlternates {
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

impl FontVariantAlternates {
    #[inline]
    /// Get initial specified value with VariantAlternatesList
    pub fn get_initial_specified_value() -> Self {
        Default::default()
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
            .try_parse(|input| input.expect_ident_matching("normal"))
            .is_ok()
        {
            return Ok(Default::default());
        }

        let mut stylistic = None;
        let mut historical = None;
        let mut styleset = None;
        let mut character_variant = None;
        let mut swash = None;
        let mut ornaments = None;
        let mut annotation = None;

        // Parse values for the various alternate types in any order.
        let mut parsed_alternates = VariantAlternatesParsingFlags::empty();
        macro_rules! check_if_parsed(
            ($input:expr, $flag:path) => (
                if parsed_alternates.contains($flag) {
                    return Err($input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                }
                parsed_alternates |= $flag;
            )
        );
        while let Ok(_) = input.try_parse(|input| match *input.next()? {
            Token::Ident(ref value) if value.eq_ignore_ascii_case("historical-forms") => {
                check_if_parsed!(input, VariantAlternatesParsingFlags::HISTORICAL_FORMS);
                historical = Some(VariantAlternates::HistoricalForms);
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
                            swash = Some(VariantAlternates::Swash(ident));
                            Ok(())
                        },
                        "stylistic" => {
                            check_if_parsed!(i, VariantAlternatesParsingFlags::STYLISTIC);
                            let location = i.current_source_location();
                            let ident = CustomIdent::from_ident(location, i.expect_ident()?, &[])?;
                            stylistic = Some(VariantAlternates::Stylistic(ident));
                            Ok(())
                        },
                        "ornaments" => {
                            check_if_parsed!(i, VariantAlternatesParsingFlags::ORNAMENTS);
                            let location = i.current_source_location();
                            let ident = CustomIdent::from_ident(location, i.expect_ident()?, &[])?;
                            ornaments = Some(VariantAlternates::Ornaments(ident));
                            Ok(())
                        },
                        "annotation" => {
                            check_if_parsed!(i, VariantAlternatesParsingFlags::ANNOTATION);
                            let location = i.current_source_location();
                            let ident = CustomIdent::from_ident(location, i.expect_ident()?, &[])?;
                            annotation = Some(VariantAlternates::Annotation(ident));
                            Ok(())
                        },
                        "styleset" => {
                            check_if_parsed!(i, VariantAlternatesParsingFlags::STYLESET);
                            let idents = i.parse_comma_separated(|i| {
                                let location = i.current_source_location();
                                CustomIdent::from_ident(location, i.expect_ident()?, &[])
                            })?;
                            styleset = Some(VariantAlternates::Styleset(idents.into()));
                            Ok(())
                        },
                        "character-variant" => {
                            check_if_parsed!(i, VariantAlternatesParsingFlags::CHARACTER_VARIANT);
                            let idents = i.parse_comma_separated(|i| {
                                let location = i.current_source_location();
                                CustomIdent::from_ident(location, i.expect_ident()?, &[])
                            })?;
                            character_variant = Some(VariantAlternates::CharacterVariant(idents.into()));
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

        // Collect the parsed values in canonical order, so that we'll serialize correctly.
        let mut alternates = Vec::new();
        macro_rules! push_if_some(
            ($value:expr) => (
                if let Some(v) = $value {
                    alternates.push(v);
                }
            )
        );
        push_if_some!(stylistic);
        push_if_some!(historical);
        push_if_some!(styleset);
        push_if_some!(character_variant);
        push_if_some!(swash);
        push_if_some!(ornaments);
        push_if_some!(annotation);

        Ok(FontVariantAlternates(alternates.into()))
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
            #[derive(MallocSizeOf, ToComputedValue, ToResolvedValue, ToShmem)]
            /// Vairants for east asian variant
            pub struct FontVariantEastAsian: u16 {
                /// None of the features
                const NORMAL = 0;
                $(
                    $(#[$($meta)+])*
                    const $ident = $value;
                )+
            }
        }

        impl ToCss for FontVariantEastAsian {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
            where
                W: Write,
            {
                if self.is_empty() {
                    return dest.write_str("normal");
                }

                let mut writer = SequenceWriter::new(dest, " ");
                $(
                    if self.intersects(Self::$ident) {
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
                debug_assert_eq!(structs::$gecko as u16, FontVariantEastAsian::$ident.bits());
            )+
        }

        impl SpecifiedValueInfo for FontVariantEastAsian {
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
impl FontVariantEastAsian {
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
impl_gecko_keyword_conversions!(FontVariantEastAsian, u16);

impl Parse for FontVariantEastAsian {
    /// normal | [ <east-asian-variant-values> || <east-asian-width-values> || ruby ]
    /// <east-asian-variant-values> = [ jis78 | jis83 | jis90 | jis04 | simplified | traditional ]
    /// <east-asian-width-values>   = [ full-width | proportional-width ]
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut result = Self::empty();

        if input
            .try_parse(|input| input.expect_ident_matching("normal"))
            .is_ok()
        {
            return Ok(result);
        }

        while let Ok(flag) = input.try_parse(|input| {
            Ok(
                match_ignore_ascii_case! { &input.expect_ident().map_err(|_| ())?,
                    "jis78" =>
                        exclusive_value!((result, Self::JIS78 | Self::JIS83 |
                                                  Self::JIS90 | Self::JIS04 |
                                                  Self::SIMPLIFIED | Self::TRADITIONAL
                                        ) => Self::JIS78),
                    "jis83" =>
                        exclusive_value!((result, Self::JIS78 | Self::JIS83 |
                                                  Self::JIS90 | Self::JIS04 |
                                                  Self::SIMPLIFIED | Self::TRADITIONAL
                                        ) => Self::JIS83),
                    "jis90" =>
                        exclusive_value!((result, Self::JIS78 | Self::JIS83 |
                                                  Self::JIS90 | Self::JIS04 |
                                                  Self::SIMPLIFIED | Self::TRADITIONAL
                                        ) => Self::JIS90),
                    "jis04" =>
                        exclusive_value!((result, Self::JIS78 | Self::JIS83 |
                                                  Self::JIS90 | Self::JIS04 |
                                                  Self::SIMPLIFIED | Self::TRADITIONAL
                                        ) => Self::JIS04),
                    "simplified" =>
                        exclusive_value!((result, Self::JIS78 | Self::JIS83 |
                                                  Self::JIS90 | Self::JIS04 |
                                                  Self::SIMPLIFIED | Self::TRADITIONAL
                                        ) => Self::SIMPLIFIED),
                    "traditional" =>
                        exclusive_value!((result, Self::JIS78 | Self::JIS83 |
                                                  Self::JIS90 | Self::JIS04 |
                                                  Self::SIMPLIFIED | Self::TRADITIONAL
                                        ) => Self::TRADITIONAL),
                    "full-width" =>
                        exclusive_value!((result, Self::FULL_WIDTH |
                                                  Self::PROPORTIONAL_WIDTH
                                        ) => Self::FULL_WIDTH),
                    "proportional-width" =>
                        exclusive_value!((result, Self::FULL_WIDTH |
                                                  Self::PROPORTIONAL_WIDTH
                                        ) => Self::PROPORTIONAL_WIDTH),
                    "ruby" =>
                        exclusive_value!((result, Self::RUBY) => Self::RUBY),
                    _ => return Err(()),
                },
            )
        }) {
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(result)
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
            #[derive(MallocSizeOf, ToComputedValue, ToResolvedValue, ToShmem)]
            /// Variants of ligatures
            pub struct FontVariantLigatures: u16 {
                /// Specifies that common default features are enabled
                const NORMAL = 0;
                $(
                    $(#[$($meta)+])*
                    const $ident = $value;
                )+
            }
        }

        impl ToCss for FontVariantLigatures {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
            where
                W: Write,
            {
                if self.is_empty() {
                    return dest.write_str("normal");
                }
                if self.contains(FontVariantLigatures::NONE) {
                    return dest.write_str("none");
                }

                let mut writer = SequenceWriter::new(dest, " ");
                $(
                    if self.intersects(FontVariantLigatures::$ident) {
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
                debug_assert_eq!(structs::$gecko as u16, FontVariantLigatures::$ident.bits());
            )+
        }

        impl SpecifiedValueInfo for FontVariantLigatures {
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
impl FontVariantLigatures {
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
impl_gecko_keyword_conversions!(FontVariantLigatures, u16);

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
    ) -> Result<Self, ParseError<'i>> {
        let mut result = Self::empty();
        if input
            .try_parse(|input| input.expect_ident_matching("normal"))
            .is_ok()
        {
            return Ok(result);
        }
        if input
            .try_parse(|input| input.expect_ident_matching("none"))
            .is_ok()
        {
            return Ok(Self::NONE);
        }

        while let Ok(flag) = input.try_parse(|input| {
            Ok(
                match_ignore_ascii_case! { &input.expect_ident().map_err(|_| ())?,
                    "common-ligatures" =>
                        exclusive_value!((result, Self::COMMON_LIGATURES |
                                                  Self::NO_COMMON_LIGATURES
                                        ) => Self::COMMON_LIGATURES),
                    "no-common-ligatures" =>
                        exclusive_value!((result, Self::COMMON_LIGATURES |
                                                  Self::NO_COMMON_LIGATURES
                                        ) => Self::NO_COMMON_LIGATURES),
                    "discretionary-ligatures" =>
                        exclusive_value!((result, Self::DISCRETIONARY_LIGATURES |
                                                  Self::NO_DISCRETIONARY_LIGATURES
                                        ) => Self::DISCRETIONARY_LIGATURES),
                    "no-discretionary-ligatures" =>
                        exclusive_value!((result, Self::DISCRETIONARY_LIGATURES |
                                                  Self::NO_DISCRETIONARY_LIGATURES
                                        ) => Self::NO_DISCRETIONARY_LIGATURES),
                    "historical-ligatures" =>
                        exclusive_value!((result, Self::HISTORICAL_LIGATURES |
                                                  Self::NO_HISTORICAL_LIGATURES
                                        ) => Self::HISTORICAL_LIGATURES),
                    "no-historical-ligatures" =>
                        exclusive_value!((result, Self::HISTORICAL_LIGATURES |
                                                  Self::NO_HISTORICAL_LIGATURES
                                        ) => Self::NO_HISTORICAL_LIGATURES),
                    "contextual" =>
                        exclusive_value!((result, Self::CONTEXTUAL |
                                                  Self::NO_CONTEXTUAL
                                        ) => Self::CONTEXTUAL),
                    "no-contextual" =>
                        exclusive_value!((result, Self::CONTEXTUAL |
                                                  Self::NO_CONTEXTUAL
                                        ) => Self::NO_CONTEXTUAL),
                    _ => return Err(()),
                },
            )
        }) {
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(result)
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
            #[derive(MallocSizeOf, ToComputedValue, ToResolvedValue, ToShmem)]
            /// Vairants of numeric values
            pub struct FontVariantNumeric: u8 {
                /// None of other variants are enabled.
                const NORMAL = 0;
                $(
                    $(#[$($meta)+])*
                    const $ident = $value;
                )+
            }
        }

        impl ToCss for FontVariantNumeric {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
            where
                W: Write,
            {
                if self.is_empty() {
                    return dest.write_str("normal");
                }

                let mut writer = SequenceWriter::new(dest, " ");
                $(
                    if self.intersects(FontVariantNumeric::$ident) {
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
                debug_assert_eq!(structs::$gecko as u8, FontVariantNumeric::$ident.bits());
            )+
        }

        impl SpecifiedValueInfo for FontVariantNumeric {
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
impl FontVariantNumeric {
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
impl_gecko_keyword_conversions!(FontVariantNumeric, u8);

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
    ) -> Result<Self, ParseError<'i>> {
        let mut result = Self::empty();

        if input
            .try_parse(|input| input.expect_ident_matching("normal"))
            .is_ok()
        {
            return Ok(result);
        }

        while let Ok(flag) = input.try_parse(|input| {
            Ok(
                match_ignore_ascii_case! { &input.expect_ident().map_err(|_| ())?,
                    "ordinal" =>
                        exclusive_value!((result, Self::ORDINAL) => Self::ORDINAL),
                    "slashed-zero" =>
                        exclusive_value!((result, Self::SLASHED_ZERO) => Self::SLASHED_ZERO),
                    "lining-nums" =>
                        exclusive_value!((result, Self::LINING_NUMS |
                                                  Self::OLDSTYLE_NUMS
                                        ) => Self::LINING_NUMS),
                    "oldstyle-nums" =>
                        exclusive_value!((result, Self::LINING_NUMS |
                                                  Self::OLDSTYLE_NUMS
                                        ) => Self::OLDSTYLE_NUMS),
                    "proportional-nums" =>
                        exclusive_value!((result, Self::PROPORTIONAL_NUMS |
                                                  Self::TABULAR_NUMS
                                        ) => Self::PROPORTIONAL_NUMS),
                    "tabular-nums" =>
                        exclusive_value!((result, Self::PROPORTIONAL_NUMS |
                                                  Self::TABULAR_NUMS
                                        ) => Self::TABULAR_NUMS),
                    "diagonal-fractions" =>
                        exclusive_value!((result, Self::DIAGONAL_FRACTIONS |
                                                  Self::STACKED_FRACTIONS
                                        ) => Self::DIAGONAL_FRACTIONS),
                    "stacked-fractions" =>
                        exclusive_value!((result, Self::DIAGONAL_FRACTIONS |
                                                  Self::STACKED_FRACTIONS
                                        ) => Self::STACKED_FRACTIONS),
                    _ => return Err(()),
                },
            )
        }) {
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(result)
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

/// This property provides low-level control over OpenType or TrueType font features.
pub type FontFeatureSettings = FontSettings<FeatureTagValue<Integer>>;

/// For font-language-override, use the same representation as the computed value.
pub use crate::values::computed::font::FontLanguageOverride;

impl Parse for FontLanguageOverride {
    /// normal | <string>
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontLanguageOverride, ParseError<'i>> {
        if input
            .try_parse(|input| input.expect_ident_matching("normal"))
            .is_ok()
        {
            return Ok(FontLanguageOverride::normal());
        }

        let string = input.expect_string()?;

        // The OpenType spec requires tags to be 1 to 4 ASCII characters:
        // https://learn.microsoft.com/en-gb/typography/opentype/spec/otff#data-types
        if string.is_empty() || string.len() > 4 || !string.is_ascii() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        let mut bytes = [b' '; 4];
        for (byte, str_byte) in bytes.iter_mut().zip(string.as_bytes()) {
            *byte = *str_byte;
        }

        Ok(FontLanguageOverride(u32::from_be_bytes(bytes)))
    }
}

/// A value for any of the font-synthesis-{weight,style,small-caps} properties.
#[repr(u8)]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum FontSynthesis {
    /// This attribute may be synthesized if not supported by a face.
    Auto,
    /// Do not attempt to synthesis this style attribute.
    None,
}

#[derive(
    Clone,
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
/// Allows authors to choose a palette from those supported by a color font
/// (and potentially @font-palette-values overrides).
pub struct FontPalette(Atom);

#[allow(missing_docs)]
impl FontPalette {
    pub fn normal() -> Self {
        Self(atom!("normal"))
    }
    pub fn light() -> Self {
        Self(atom!("light"))
    }
    pub fn dark() -> Self {
        Self(atom!("dark"))
    }
}

impl Parse for FontPalette {
    /// normal | light | dark | dashed-ident
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<FontPalette, ParseError<'i>> {
        let location = input.current_source_location();
        let ident = input.expect_ident()?;
        match_ignore_ascii_case! { &ident,
            "normal" => Ok(Self::normal()),
            "light" => Ok(Self::light()),
            "dark" => Ok(Self::dark()),
            _ => if ident.starts_with("--") {
                Ok(Self(Atom::from(ident.as_ref())))
            } else {
                Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone())))
            },
        }
    }
}

impl ToCss for FontPalette {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        serialize_atom_identifier(&self.0, dest)
    }
}

/// This property provides low-level control over OpenType or TrueType font
/// variations.
pub type FontVariationSettings = FontSettings<VariationValue<Number>>;

fn parse_one_feature_value<'i, 't>(
    context: &ParserContext,
    input: &mut Parser<'i, 't>,
) -> Result<Integer, ParseError<'i>> {
    if let Ok(integer) = input.try_parse(|i| Integer::parse_non_negative(context, i)) {
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
            .try_parse(|i| parse_one_feature_value(context, i))
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

/// A metrics override value for a @font-face descriptor
///
/// https://drafts.csswg.org/css-fonts/#font-metrics-override-desc
#[derive(
    Clone, Copy, Debug, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem,
)]
pub enum MetricsOverride {
    /// A non-negative `<percentage>` of the computed font size
    Override(NonNegativePercentage),
    /// Normal metrics from the font.
    Normal,
}

impl MetricsOverride {
    #[inline]
    /// Get default value with `normal`
    pub fn normal() -> MetricsOverride {
        MetricsOverride::Normal
    }

    /// The ToComputedValue implementation, used for @font-face descriptors.
    ///
    /// Valid override percentages must be non-negative; we return -1.0 to indicate
    /// the absence of an override (i.e. 'normal').
    #[inline]
    pub fn compute(&self) -> ComputedPercentage {
        match *self {
            MetricsOverride::Normal => ComputedPercentage(-1.0),
            MetricsOverride::Override(percent) => ComputedPercentage(percent.0.get()),
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
/// How to do font-size scaling.
pub enum XTextScale {
    /// Both min-font-size and text zoom are enabled.
    All,
    /// Text-only zoom is enabled, but min-font-size is not honored.
    ZoomOnly,
    /// Neither of them is enabled.
    None,
}

impl XTextScale {
    /// Returns whether text zoom is enabled.
    #[inline]
    pub fn text_zoom_enabled(self) -> bool {
        self != Self::None
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
        Length::new(DEFAULT_SCRIPT_MIN_SIZE_PT as f32 * PX_PER_PT)
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

/// A value for the `math-depth` property.
/// https://mathml-refresh.github.io/mathml-core/#the-math-script-level-property
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub enum MathDepth {
    /// Increment math-depth if math-style is compact.
    AutoAdd,

    /// Add the function's argument to math-depth.
    #[css(function)]
    Add(Integer),

    /// Set math-depth to the specified value.
    Absolute(Integer),
}

impl Parse for MathDepth {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<MathDepth, ParseError<'i>> {
        if input
            .try_parse(|i| i.expect_ident_matching("auto-add"))
            .is_ok()
        {
            return Ok(MathDepth::AutoAdd);
        }
        if let Ok(math_depth_value) = input.try_parse(|input| Integer::parse(context, input)) {
            return Ok(MathDepth::Absolute(math_depth_value));
        }
        input.expect_function_matching("add")?;
        let math_depth_delta_value =
            input.parse_nested_block(|input| Integer::parse(context, input))?;
        Ok(MathDepth::Add(math_depth_delta_value))
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
