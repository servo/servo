/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified values for font properties

use Atom;
use app_units::Au;
use byteorder::{BigEndian, ByteOrder};
use cssparser::{Parser, Token};
#[cfg(feature = "gecko")]
use gecko_bindings::bindings;
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use parser::{Parse, ParserContext};
use properties::longhands::system_font::SystemFont;
#[allow(unused_imports)]
use std::ascii::AsciiExt;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};
use values::CustomIdent;
use values::computed::{font as computed, Context, Length, NonNegativeLength, ToComputedValue};
use values::computed::font::{SingleFontFamily, FontFamilyList, FamilyName};
use values::generics::{FontSettings, FontSettingTagFloat};
use values::specified::{AllowQuirks, LengthOrPercentage, NoCalcLength, Number};
use values::specified::length::{AU_PER_PT, AU_PER_PX, FontBaseSize};

const DEFAULT_SCRIPT_MIN_SIZE_PT: u32 = 8;
const DEFAULT_SCRIPT_SIZE_MULTIPLIER: f64 = 0.71;

#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToCss)]
/// A specified font-weight value
pub enum FontWeight {
    /// Normal variant
    Normal,
    /// Bold variant
    Bold,
    /// Bolder variant
    Bolder,
    /// Lighter variant
    Lighter,
    /// Computed weight variant
    Weight(computed::FontWeight),
    /// System font varaint
    System(SystemFont),
}

impl FontWeight {
    /// Get a specified FontWeight from a gecko keyword
    pub fn from_gecko_keyword(kw: u32) -> Self {
        computed::FontWeight::from_int(kw as i32).map(FontWeight::Weight)
            .expect("Found unexpected value in style struct for font-weight property")
    }

    /// Get a specified FontWeight from a SystemFont
    pub fn system_font(f: SystemFont) -> Self {
        FontWeight::System(f)
    }

    /// Retreive a SystemFont from FontWeight
    pub fn get_system(&self) -> Option<SystemFont> {
        if let FontWeight::System(s) = *self {
            Some(s)
        } else {
            None
        }
    }
}

impl Parse for FontWeight {
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<FontWeight, ParseError<'i>> {
        let result = match *input.next()? {
            Token::Ident(ref ident) => {
                match_ignore_ascii_case! { ident,
                    "normal" => Ok(FontWeight::Normal),
                    "bold" => Ok(FontWeight::Bold),
                    "bolder" => Ok(FontWeight::Bolder),
                    "lighter" => Ok(FontWeight::Lighter),
                    _ => Err(()),
                }
            }
            Token::Number { int_value: Some(value), .. } => {
                computed::FontWeight::from_int(value).map(FontWeight::Weight)
            },
            _ => Err(()),
        };

        result.map_err(|_| input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

impl ToComputedValue for FontWeight {
    type ComputedValue = computed::FontWeight;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            FontWeight::Weight(weight) => weight,
            FontWeight::Normal => computed::FontWeight::normal(),
            FontWeight::Bold => computed::FontWeight::bold(),
            FontWeight::Bolder => {
                context.builder.get_parent_font().clone_font_weight().bolder()
            },
            FontWeight::Lighter => {
                context.builder.get_parent_font().clone_font_weight().lighter()
            },
            #[cfg(feature = "gecko")]
            FontWeight::System(_) => {
                context.cached_system_font.as_ref().unwrap().font_weight.clone()
            },
            #[cfg(not(feature = "gecko"))]
            FontWeight::System(_) => unreachable!(),
        }
    }

    #[inline]
    fn from_computed_value(computed: &computed::FontWeight) -> Self {
        FontWeight::Weight(*computed)
    }
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
/// A specified font-size value
pub enum FontSize {
    /// A length; e.g. 10px.
    Length(LengthOrPercentage),
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
    Keyword(computed::KeywordInfo),
    /// font-size: smaller
    Smaller,
    /// font-size: larger
    Larger,
    /// Derived from a specified system font.
    System(SystemFont)
}

impl ToCss for FontSize {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            FontSize::Length(ref lop) => lop.to_css(dest),
            FontSize::Keyword(info) => info.kw.to_css(dest),
            FontSize::Smaller => dest.write_str("smaller"),
            FontSize::Larger => dest.write_str("larger"),
            FontSize::System(sys) => sys.to_css(dest),
        }
    }
}

impl From<LengthOrPercentage> for FontSize {
    fn from(other: LengthOrPercentage) -> Self {
        FontSize::Length(other)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
/// Specifies a prioritized list of font family names or generic family names
pub enum FontFamily {
    /// List of `font-family`
    Values(FontFamilyList),
    /// System font
    System(SystemFont),
}

impl FontFamily {
    /// Get `font-family` with system font
    pub fn system_font(f: SystemFont) -> Self {
        FontFamily::System(f)
    }

    /// Get system font
    pub fn get_system(&self) -> Option<SystemFont> {
        if let FontFamily::System(s) = *self {
            Some(s)
        } else {
            None
        }
    }

    /// Parse a specified font-family value
    pub fn parse_specified<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        input.parse_comma_separated(|input| SingleFontFamily::parse(input)).map(|v| {
            FontFamily::Values(FontFamilyList::new(v.into_boxed_slice()))
        })
    }

    #[cfg(feature = "gecko")]
    /// Return the generic ID if it is a single generic font
    pub fn single_generic(&self) -> Option<u8> {
        match *self {
            FontFamily::Values(ref values) => values.single_generic(),
            _ => None,
        }
    }
}

impl ToComputedValue for FontFamily {
    type ComputedValue = computed::FontFamily;

    fn to_computed_value(&self, _cx: &Context) -> Self::ComputedValue {
        match *self {
            FontFamily::Values(ref v) => computed::FontFamily(v.clone()),
            FontFamily::System(_) => {
                #[cfg(feature = "gecko")] {
                    _cx.cached_system_font.as_ref().unwrap().font_family.clone()
                }
                #[cfg(feature = "servo")] {
                    unreachable!()
                }
            }
        }
    }

    fn from_computed_value(other: &computed::FontFamily) -> Self {
        FontFamily::Values(other.0.clone())
    }
}

#[cfg(feature = "gecko")]
impl MallocSizeOf for FontFamily {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        match *self {
            FontFamily::Values(ref v) => {
                // Although a SharedFontList object is refcounted, we always
                // attribute its size to the specified value.
                unsafe {
                    bindings::Gecko_SharedFontList_SizeOfIncludingThis(
                        v.0.get()
                    )
                }
            }
            FontFamily::System(_) => 0,
        }
    }
}

impl ToCss for FontFamily {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            FontFamily::Values(ref v) => {
                let mut iter = v.iter();
                iter.next().unwrap().to_css(dest)?;
                for family in iter {
                    dest.write_str(", ")?;
                    family.to_css(dest)?;
                }
                Ok(())
            }
            FontFamily::System(sys) => sys.to_css(dest),
        }
    }
}

impl Parse for FontFamily {
    /// <family-name>#
    /// <family-name> = <string> | [ <ident>+ ]
    /// TODO: <generic-family>
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<FontFamily, ParseError<'i>> {
        FontFamily::parse_specified(input)
    }
}

/// `FamilyName::parse` is based on `SingleFontFamily::parse` and not the other way around
/// because we want the former to exclude generic family keywords.
impl Parse for FamilyName {
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        match SingleFontFamily::parse(input) {
            Ok(SingleFontFamily::FamilyName(name)) => Ok(name),
            Ok(SingleFontFamily::Generic(_)) => Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
            Err(e) => Err(e)
        }
    }
}

#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss)]
/// Preserve the readability of text when font fallback occurs
pub enum FontSizeAdjust {
    /// None variant
    None,
    /// Number variant
    Number(Number),
    /// system font
    System(SystemFont),
}

impl FontSizeAdjust {
    #[inline]
    /// Default value of font-size-adjust
    pub fn none() -> Self {
        FontSizeAdjust::None
    }

    /// Get font-size-adjust with SystemFont
    pub fn system_font(f: SystemFont) -> Self {
        FontSizeAdjust::System(f)
    }

    /// Get SystemFont variant
    pub fn get_system(&self) -> Option<SystemFont> {
        if let FontSizeAdjust::System(s) = *self {
            Some(s)
        } else {
            None
        }
    }
}

impl ToComputedValue for FontSizeAdjust {
    type ComputedValue = computed::FontSizeAdjust;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            FontSizeAdjust::None => computed::FontSizeAdjust::None,
            FontSizeAdjust::Number(ref n) => computed::FontSizeAdjust::Number(n.to_computed_value(context)),
            FontSizeAdjust::System(_) => {
                #[cfg(feature = "gecko")] {
                    context.cached_system_font.as_ref().unwrap().font_size_adjust
                }
                #[cfg(feature = "servo")] {
                    unreachable!()
                }
            }
        }
    }

    fn from_computed_value(computed: &computed::FontSizeAdjust) -> Self {
        match *computed {
            computed::FontSizeAdjust::None => FontSizeAdjust::None,
            computed::FontSizeAdjust::Number(ref v) => FontSizeAdjust::Number(Number::from_computed_value(v)),
        }
    }
}

impl Parse for FontSizeAdjust {
    /// none | <number>
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<FontSizeAdjust, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(FontSizeAdjust::None);
        }

        Ok(FontSizeAdjust::Number(Number::parse_non_negative(context, input)?))
    }
}

/// CSS font keywords
#[derive(Animate, ComputeSquaredDistance, MallocSizeOf, ToAnimatedValue, ToAnimatedZero)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum KeywordSize {
    XXSmall = 1, // This is to enable the NonZero optimization
                 // which simplifies the representation of Option<KeywordSize>
                 // in bindgen
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
    XXLarge,
    // This is not a real font keyword and will not parse
    // HTML font-size 7 corresponds to this value
    XXXLarge,
}

impl KeywordSize {
    /// Parse a keyword size
    pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        try_match_ident_ignore_ascii_case! { input,
            "xx-small" => Ok(KeywordSize::XXSmall),
            "x-small" => Ok(KeywordSize::XSmall),
            "small" => Ok(KeywordSize::Small),
            "medium" => Ok(KeywordSize::Medium),
            "large" => Ok(KeywordSize::Large),
            "x-large" => Ok(KeywordSize::XLarge),
            "xx-large" => Ok(KeywordSize::XXLarge),
        }
    }

    /// Convert to an HTML <font size> value
    pub fn html_size(&self) -> u8 {
        match *self {
            KeywordSize::XXSmall => 0,
            KeywordSize::XSmall => 1,
            KeywordSize::Small => 2,
            KeywordSize::Medium => 3,
            KeywordSize::Large => 4,
            KeywordSize::XLarge => 5,
            KeywordSize::XXLarge => 6,
            KeywordSize::XXXLarge => 7,
        }
    }
}

impl Default for KeywordSize {
    fn default() -> Self {
        KeywordSize::Medium
    }
}

impl ToCss for KeywordSize {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str(match *self {
            KeywordSize::XXSmall => "xx-small",
            KeywordSize::XSmall => "x-small",
            KeywordSize::Small => "small",
            KeywordSize::Medium => "medium",
            KeywordSize::Large => "large",
            KeywordSize::XLarge => "x-large",
            KeywordSize::XXLarge => "xx-large",
            KeywordSize::XXXLarge => unreachable!("We should never serialize \
                                      specified values set via
                                      HTML presentation attributes"),
        })
    }
}

/// This is the ratio applied for font-size: larger
/// and smaller by both Firefox and Chrome
const LARGER_FONT_SIZE_RATIO: f32 = 1.2;

/// The default font size.
pub const FONT_MEDIUM_PX: i32 = 16;

#[cfg(feature = "servo")]
impl ToComputedValue for KeywordSize {
    type ComputedValue = NonNegativeLength;
    #[inline]
    fn to_computed_value(&self, _: &Context) -> NonNegativeLength {
        // https://drafts.csswg.org/css-fonts-3/#font-size-prop
        match *self {
            KeywordSize::XXSmall => Au::from_px(FONT_MEDIUM_PX) * 3 / 5,
            KeywordSize::XSmall => Au::from_px(FONT_MEDIUM_PX) * 3 / 4,
            KeywordSize::Small => Au::from_px(FONT_MEDIUM_PX) * 8 / 9,
            KeywordSize::Medium => Au::from_px(FONT_MEDIUM_PX),
            KeywordSize::Large => Au::from_px(FONT_MEDIUM_PX) * 6 / 5,
            KeywordSize::XLarge => Au::from_px(FONT_MEDIUM_PX) * 3 / 2,
            KeywordSize::XXLarge => Au::from_px(FONT_MEDIUM_PX) * 2,
            KeywordSize::XXXLarge => Au::from_px(FONT_MEDIUM_PX) * 3,
        }.into()
    }

    #[inline]
    fn from_computed_value(_: &NonNegativeLength) -> Self {
        unreachable!()
    }
}

#[cfg(feature = "gecko")]
impl ToComputedValue for KeywordSize {
    type ComputedValue = NonNegativeLength;
    #[inline]
    fn to_computed_value(&self, cx: &Context) -> NonNegativeLength {
        use context::QuirksMode;
        use values::specified::length::au_to_int_px;
        // Data from nsRuleNode.cpp in Gecko
        // Mapping from base size and HTML size to pixels
        // The first index is (base_size - 9), the second is the
        // HTML size. "0" is CSS keyword xx-small, not HTML size 0,
        // since HTML size 0 is the same as 1.
        //
        //  xxs   xs      s      m     l      xl     xxl   -
        //  -     0/1     2      3     4      5      6     7
        static FONT_SIZE_MAPPING: [[i32; 8]; 8] = [
            [9,    9,     9,     9,    11,    14,    18,    27],
            [9,    9,     9,    10,    12,    15,    20,    30],
            [9,    9,    10,    11,    13,    17,    22,    33],
            [9,    9,    10,    12,    14,    18,    24,    36],
            [9,   10,    12,    13,    16,    20,    26,    39],
            [9,   10,    12,    14,    17,    21,    28,    42],
            [9,   10,    13,    15,    18,    23,    30,    45],
            [9,   10,    13,    16,    18,    24,    32,    48]
        ];

        // Data from nsRuleNode.cpp in Gecko
        // (https://dxr.mozilla.org/mozilla-central/rev/35fbf14b9/layout/style/nsRuleNode.cpp#3303)
        //
        // This table gives us compatibility with WinNav4 for the default fonts only.
        // In WinNav4, the default fonts were:
        //
        //     Times/12pt ==   Times/16px at 96ppi
        //   Courier/10pt == Courier/13px at 96ppi
        //
        // xxs   xs     s      m      l     xl     xxl    -
        // -     1      2      3      4     5      6      7
        static QUIRKS_FONT_SIZE_MAPPING: [[i32; 8]; 8] = [
            [9,    9,     9,     9,    11,    14,    18,    28],
            [9,    9,     9,    10,    12,    15,    20,    31],
            [9,    9,     9,    11,    13,    17,    22,    34],
            [9,    9,    10,    12,    14,    18,    24,    37],
            [9,    9,    10,    13,    16,    20,    26,    40],
            [9,    9,    11,    14,    17,    21,    28,    42],
            [9,   10,    12,    15,    17,    23,    30,    45],
            [9,   10,    13,    16,    18,    24,    32,    48]
        ];

        static FONT_SIZE_FACTORS: [i32; 8] = [60, 75, 89, 100, 120, 150, 200, 300];

        let ref gecko_font = cx.style().get_font().gecko();
        let base_size = unsafe { Atom::with(gecko_font.mLanguage.mRawPtr, |atom| {
            cx.font_metrics_provider.get_size(atom, gecko_font.mGenericID).0
        }) };

        let base_size_px = au_to_int_px(base_size as f32);
        let html_size = self.html_size() as usize;
        if base_size_px >= 9 && base_size_px <= 16 {
            let mapping = if cx.quirks_mode == QuirksMode::Quirks {
                QUIRKS_FONT_SIZE_MAPPING
            } else {
                FONT_SIZE_MAPPING
            };
            Au::from_px(mapping[(base_size_px - 9) as usize][html_size]).into()
        } else {
            Au(FONT_SIZE_FACTORS[html_size] * base_size / 100).into()
        }
    }

    #[inline]
    fn from_computed_value(_: &NonNegativeLength) -> Self {
        unreachable!()
    }
}

impl FontSize {
    /// <https://html.spec.whatwg.org/multipage/#rules-for-parsing-a-legacy-font-size>
    pub fn from_html_size(size: u8) -> Self {
        FontSize::Keyword(match size {
            // If value is less than 1, let it be 1.
            0 | 1 => KeywordSize::XSmall,
            2 => KeywordSize::Small,
            3 => KeywordSize::Medium,
            4 => KeywordSize::Large,
            5 => KeywordSize::XLarge,
            6 => KeywordSize::XXLarge,
            // If value is greater than 7, let it be 7.
            _ => KeywordSize::XXXLarge,
        }.into())
    }

    /// Compute it against a given base font size
    pub fn to_computed_value_against(
        &self,
        context: &Context,
        base_size: FontBaseSize,
    ) -> computed::FontSize {
        use values::specified::length::FontRelativeLength;

        let compose_keyword = |factor| {
            context.style().get_parent_font()
                   .clone_font_size().keyword_info
                   .map(|i| i.compose(factor, Au(0).into()))
        };
        let mut info = None;
        let size = match *self {
            FontSize::Length(LengthOrPercentage::Length(
                    NoCalcLength::FontRelative(value))) => {
                if let FontRelativeLength::Em(em) = value {
                    // If the parent font was keyword-derived, this is too.
                    // Tack the em unit onto the factor
                    info = compose_keyword(em);
                }
                value.to_computed_value(context, base_size).into()
            }
            FontSize::Length(LengthOrPercentage::Length(
                    NoCalcLength::ServoCharacterWidth(value))) => {
                value.to_computed_value(base_size.resolve(context)).into()
            }
            FontSize::Length(LengthOrPercentage::Length(
                    NoCalcLength::Absolute(ref l))) => {
                context.maybe_zoom_text(l.to_computed_value(context).into())
            }
            FontSize::Length(LengthOrPercentage::Length(ref l)) => {
                l.to_computed_value(context).into()
            }
            FontSize::Length(LengthOrPercentage::Percentage(pc)) => {
                // If the parent font was keyword-derived, this is too.
                // Tack the % onto the factor
                info = compose_keyword(pc.0);
                base_size.resolve(context).scale_by(pc.0).into()
            }
            FontSize::Length(LengthOrPercentage::Calc(ref calc)) => {
                let parent = context.style().get_parent_font().clone_font_size();
                // if we contain em/% units and the parent was keyword derived, this is too
                // Extract the ratio/offset and compose it
                if (calc.em.is_some() || calc.percentage.is_some()) && parent.keyword_info.is_some() {
                    let ratio = calc.em.unwrap_or(0.) + calc.percentage.map_or(0., |pc| pc.0);
                    // Compute it, but shave off the font-relative part (em, %).
                    //
                    // This will mean that other font-relative units like ex and
                    // ch will be computed against the old parent font even when
                    // the font changes.
                    //
                    // There's no particular "right answer" for what to do here,
                    // Gecko recascades as if the font had changed, we instead
                    // track the changes and reapply, which means that we carry
                    // over old computed ex/ch values whilst Gecko recomputes
                    // new ones.
                    //
                    // This is enough of an edge case to not really matter.
                    let abs = calc.to_computed_value_zoomed(
                        context,
                        FontBaseSize::InheritedStyleButStripEmUnits,
                    ).length_component();

                    info = parent.keyword_info.map(|i| i.compose(ratio, abs.into()));
                }
                let calc = calc.to_computed_value_zoomed(context, base_size);
                calc.to_used_value(Some(base_size.resolve(context))).unwrap().into()
            }
            FontSize::Keyword(i) => {
                // As a specified keyword, this is keyword derived
                info = Some(i);
                i.to_computed_value(context)
            }
            FontSize::Smaller => {
                info = compose_keyword(1. / LARGER_FONT_SIZE_RATIO);
                FontRelativeLength::Em(1. / LARGER_FONT_SIZE_RATIO)
                    .to_computed_value(context, base_size).into()
            }
            FontSize::Larger => {
                info = compose_keyword(LARGER_FONT_SIZE_RATIO);
                FontRelativeLength::Em(LARGER_FONT_SIZE_RATIO)
                    .to_computed_value(context, base_size).into()
            }

            FontSize::System(_) => {
                #[cfg(feature = "servo")] {
                    unreachable!()
                }
                #[cfg(feature = "gecko")] {
                    context.cached_system_font.as_ref().unwrap().font_size.size
                }
            }
        };
        computed::FontSize {
            size: size,
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
        FontSize::Length(LengthOrPercentage::Length(
            ToComputedValue::from_computed_value(&computed.size.0)
        ))
    }
}

impl FontSize {
    /// Construct a system font value.
    pub fn system_font(f: SystemFont) -> Self {
        FontSize::System(f)
    }

    /// Obtain the system font, if any
    pub fn get_system(&self) -> Option<SystemFont> {
        if let FontSize::System(s) = *self {
            Some(s)
        } else {
            None
        }
    }

    #[inline]
    /// Get initial value for specified font size.
    pub fn medium() -> Self {
        FontSize::Keyword(computed::KeywordInfo::medium())
    }

    /// Parses a font-size, with quirks.
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks
    ) -> Result<FontSize, ParseError<'i>> {
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse_non_negative_quirky(context, i, allow_quirks)) {
            return Ok(FontSize::Length(lop))
        }

        if let Ok(kw) = input.try(KeywordSize::parse) {
            return Ok(FontSize::Keyword(kw.into()))
        }

        try_match_ident_ignore_ascii_case! { input,
            "smaller" => Ok(FontSize::Smaller),
            "larger" => Ok(FontSize::Larger),
        }
    }

    #[allow(unused_mut)]
    /// Cascade `font-size` with specified value
    pub fn cascade_specified_font_size(
        context: &mut Context,
        specified_value: &FontSize,
        mut computed: computed::FontSize
    ) {
        // we could use clone_language and clone_font_family() here but that's
        // expensive. Do it only in gecko mode for now.
        #[cfg(feature = "gecko")] {
            // if the language or generic changed, we need to recalculate
            // the font size from the stored font-size origin information.
            if context.builder.get_font().gecko().mLanguage.mRawPtr !=
               context.builder.get_parent_font().gecko().mLanguage.mRawPtr ||
               context.builder.get_font().gecko().mGenericID !=
               context.builder.get_parent_font().gecko().mGenericID {
                if let Some(info) = computed.keyword_info {
                    computed.size = info.to_computed_value(context);
                }
            }
        }

        let device = context.builder.device;
        let mut font = context.builder.take_font();
        let parent_unconstrained = {
            let parent_font = context.builder.get_parent_font();
            font.apply_font_size(computed, parent_font, device)
        };
        context.builder.put_font(font);

        if let Some(parent) = parent_unconstrained {
            let new_unconstrained =
                specified_value.to_computed_value_against(context, FontBaseSize::Custom(Au::from(parent)));
            context.builder
                   .mutate_font()
                   .apply_unconstrained_font_size(new_unconstrained.size);
        }
    }
}

impl Parse for FontSize {
    /// <length> | <percentage> | <absolute-size> | <relative-size>
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<FontSize, ParseError<'i>> {
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

#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
/// Set of variant alternates
pub enum VariantAlternates {
    /// Enables display of stylistic alternates
    #[css(function)]
    Stylistic(CustomIdent),
    /// Enables display with stylistic sets
    #[css(comma, function, iterable)]
    Styleset(Box<[CustomIdent]>),
    /// Enables display of specific character variants
    #[css(comma, function, iterable)]
    CharacterVariant(Box<[CustomIdent]>),
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

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
/// List of Variant Alternates
pub struct VariantAlternatesList(pub Box<[VariantAlternates]>);

impl VariantAlternatesList {
    /// Returns the length of all variant alternates.
    pub fn len(&self) -> usize {
        self.0.iter().fold(0, |acc, alternate| {
            match *alternate {
                VariantAlternates::Swash(_) | VariantAlternates::Stylistic(_) |
                VariantAlternates::Ornaments(_) | VariantAlternates::Annotation(_) => {
                    acc + 1
                },
                VariantAlternates::Styleset(ref slice) |
                VariantAlternates::CharacterVariant(ref slice) => {
                    acc + slice.len()
                },
                _ => acc,
            }
        })
    }
}

impl ToCss for VariantAlternatesList {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.0.is_empty() {
            return dest.write_str("normal");
        }

        let mut iter = self.0.iter();
        iter.next().unwrap().to_css(dest)?;
        for alternate in iter {
            dest.write_str(" ")?;
            alternate.to_css(dest)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
/// Control over the selection of these alternate glyphs
pub enum FontVariantAlternates {
    /// Use alternative glyph from value
    Value(VariantAlternatesList),
    /// Use system font glyph
    System(SystemFont)
}

impl FontVariantAlternates {
    #[inline]
    /// Get initial specified value with VariantAlternatesList
    pub fn get_initial_specified_value() -> Self {
        FontVariantAlternates::Value(VariantAlternatesList(vec![].into_boxed_slice()))
    }

    /// Get FontVariantAlternates with system font
    pub fn system_font(f: SystemFont) -> Self {
        FontVariantAlternates::System(f)
    }

    /// Get SystemFont of FontVariantAlternates
    pub fn get_system(&self) -> Option<SystemFont> {
        if let FontVariantAlternates::System(s) = *self {
            Some(s)
        } else {
            None
        }
    }
}

impl ToComputedValue for FontVariantAlternates {
    type ComputedValue = computed::FontVariantAlternates;

    fn to_computed_value(&self, _context: &Context) -> computed::FontVariantAlternates {
        match *self {
            FontVariantAlternates::Value(ref v) => v.clone(),
            FontVariantAlternates::System(_) => {
                #[cfg(feature = "gecko")] {
                    _context.cached_system_font.as_ref().unwrap().font_variant_alternates.clone()
                }
                #[cfg(feature = "servo")] {
                    unreachable!()
                }
            }
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
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<FontVariantAlternates, ParseError<'i>> {
        let mut alternates = Vec::new();
        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(FontVariantAlternates::Value(VariantAlternatesList(alternates.into_boxed_slice())));
        }

        let mut parsed_alternates = VariantAlternatesParsingFlags::empty();
        macro_rules! check_if_parsed(
            ($input:expr, $flag:path) => (
                if parsed_alternates.contains($flag) {
                    return Err($input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                }
                parsed_alternates |= $flag;
            )
        );
        while let Ok(_) = input.try(|input| {
            // FIXME: remove clone() when lifetimes are non-lexical
            match input.next()?.clone() {
                Token::Ident(ref value) if value.eq_ignore_ascii_case("historical-forms") => {
                    check_if_parsed!(input, VariantAlternatesParsingFlags::HISTORICAL_FORMS);
                    alternates.push(VariantAlternates::HistoricalForms);
                    Ok(())
                },
                Token::Function(ref name) => {
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
                                alternates.push(VariantAlternates::Styleset(idents.into_boxed_slice()));
                                Ok(())
                            },
                            "character-variant" => {
                                check_if_parsed!(i, VariantAlternatesParsingFlags::CHARACTER_VARIANT);
                                let idents = i.parse_comma_separated(|i| {
                                    let location = i.current_source_location();
                                    CustomIdent::from_ident(location, i.expect_ident()?, &[])
                                })?;
                                alternates.push(VariantAlternates::CharacterVariant(idents.into_boxed_slice()));
                                Ok(())
                            },
                            _ => return Err(i.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
                        }
                    })
                },
                _ => Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
            }
        }) { }

        if parsed_alternates.is_empty() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        Ok(FontVariantAlternates::Value(VariantAlternatesList(alternates.into_boxed_slice())))
    }
}

bitflags! {
    #[derive(MallocSizeOf)]
    /// Vairants for east asian variant
    pub struct VariantEastAsian: u16 {
        /// None of the features
        const NORMAL = 0;
        /// Enables rendering of JIS78 forms (OpenType feature: jp78)
        const JIS78 = 0x01;
        /// Enables rendering of JIS83 forms (OpenType feature: jp83).
        const JIS83 = 0x02;
        /// Enables rendering of JIS90 forms (OpenType feature: jp90).
        const JIS90 = 0x04;
        /// Enables rendering of JIS2004 forms (OpenType feature: jp04).
        const JIS04 = 0x08;
        /// Enables rendering of simplified forms (OpenType feature: smpl).
        const SIMPLIFIED = 0x10;
        /// Enables rendering of traditional forms (OpenType feature: trad).
        const TRADITIONAL = 0x20;
        /// Enables rendering of full-width variants (OpenType feature: fwid).
        const FULL_WIDTH = 0x40;
        /// Enables rendering of proportionally-spaced variants (OpenType feature: pwid).
        const PROPORTIONAL_WIDTH = 0x80;
        /// Enables display of ruby variant glyphs (OpenType feature: ruby).
        const RUBY = 0x100;
    }
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

impl ToCss for VariantEastAsian {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.is_empty() {
            return dest.write_str("normal")
        }

        let mut has_any = false;

        macro_rules! write_value {
            ($ident:path => $str:expr) => {
                if self.intersects($ident) {
                    if has_any {
                        dest.write_str(" ")?;
                    }
                    has_any = true;
                    dest.write_str($str)?;
                }
            }
        }

        write_value!(VariantEastAsian::JIS78 => "jis78");
        write_value!(VariantEastAsian::JIS83 => "jis83");
        write_value!(VariantEastAsian::JIS90 => "jis90");
        write_value!(VariantEastAsian::JIS04 => "jis04");
        write_value!(VariantEastAsian::SIMPLIFIED => "simplified");
        write_value!(VariantEastAsian::TRADITIONAL => "traditional");
        write_value!(VariantEastAsian::FULL_WIDTH => "full-width");
        write_value!(VariantEastAsian::PROPORTIONAL_WIDTH => "proportional-width");
        write_value!(VariantEastAsian::RUBY => "ruby");

        debug_assert!(has_any);
        Ok(())
    }
}

#[cfg(feature = "gecko")]
impl_gecko_keyword_conversions!(VariantEastAsian, u16);

/// Asserts that all variant-east-asian matches its NS_FONT_VARIANT_EAST_ASIAN_* value.
#[cfg(feature = "gecko")]
#[inline]
pub fn assert_variant_east_asian_matches() {
    use gecko_bindings::structs;

    macro_rules! check_variant_east_asian {
        ( $( $a:ident => $b:path),*, ) => {
            if cfg!(debug_assertions) {
                $(
                    assert_eq!(structs::$a as u16, $b.bits());
                )*
            }
        }
    }

    check_variant_east_asian! {
        NS_FONT_VARIANT_EAST_ASIAN_FULL_WIDTH => VariantEastAsian::FULL_WIDTH,
        NS_FONT_VARIANT_EAST_ASIAN_JIS04 => VariantEastAsian::JIS04,
        NS_FONT_VARIANT_EAST_ASIAN_JIS78 => VariantEastAsian::JIS78,
        NS_FONT_VARIANT_EAST_ASIAN_JIS83 => VariantEastAsian::JIS83,
        NS_FONT_VARIANT_EAST_ASIAN_JIS90 => VariantEastAsian::JIS90,
        NS_FONT_VARIANT_EAST_ASIAN_PROP_WIDTH => VariantEastAsian::PROPORTIONAL_WIDTH,
        NS_FONT_VARIANT_EAST_ASIAN_RUBY => VariantEastAsian::RUBY,
        NS_FONT_VARIANT_EAST_ASIAN_SIMPLIFIED => VariantEastAsian::SIMPLIFIED,
        NS_FONT_VARIANT_EAST_ASIAN_TRADITIONAL => VariantEastAsian::TRADITIONAL,
    }
}

#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq, ToCss)]
/// Allows control of glyph substitution and sizing in East Asian text.
pub enum FontVariantEastAsian {
    /// Value variant with `variant-east-asian`
    Value(VariantEastAsian),
    /// System font variant
    System(SystemFont)
}

impl FontVariantEastAsian {
    #[inline]
    /// Get default `font-variant-east-asian` with `empty` variant
    pub fn empty() -> Self {
        FontVariantEastAsian::Value(VariantEastAsian::empty())
    }

    /// Get `font-variant-east-asian` with system font
    pub fn system_font(f: SystemFont) -> Self {
        FontVariantEastAsian::System(f)
    }

    /// Get system font
    pub fn get_system(&self) -> Option<SystemFont> {
        if let FontVariantEastAsian::System(s) = *self {
            Some(s)
        } else {
            None
        }
    }
}

impl ToComputedValue for FontVariantEastAsian {
    type ComputedValue = computed::FontVariantEastAsian;

    fn to_computed_value(&self, _context: &Context) -> computed::FontVariantEastAsian {
        match *self {
            FontVariantEastAsian::Value(ref v) => v.clone(),
            FontVariantEastAsian::System(_) => {
                #[cfg(feature = "gecko")] {
                    _context.cached_system_font.as_ref().unwrap().font_variant_east_asian.clone()
                }
                #[cfg(feature = "servo")] {
                    unreachable!()
                }
            }
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
        input: &mut Parser<'i, 't>
    ) -> Result<FontVariantEastAsian, ParseError<'i>> {
        let mut result = VariantEastAsian::empty();

        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(FontVariantEastAsian::Value(result))
        }

        while let Ok(flag) = input.try(|input| {
            Ok(match_ignore_ascii_case! { &input.expect_ident().map_err(|_| ())?,
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
            })
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

bitflags! {
    #[derive(MallocSizeOf)]
    /// Variants of ligatures
    pub struct VariantLigatures: u16 {
        /// Specifies that common default features are enabled
        const NORMAL = 0;
        /// Specifies that all types of ligatures and contextual forms
        /// covered by this property are explicitly disabled
        const NONE = 0x01;
        /// Enables display of common ligatures
        const COMMON_LIGATURES = 0x02;
        /// Disables display of common ligatures
        const NO_COMMON_LIGATURES = 0x04;
        /// Enables display of discretionary ligatures
        const DISCRETIONARY_LIGATURES = 0x08;
        /// Disables display of discretionary ligatures
        const NO_DISCRETIONARY_LIGATURES = 0x10;
        /// Enables display of historical ligatures
        const HISTORICAL_LIGATURES = 0x20;
        /// Disables display of historical ligatures
        const NO_HISTORICAL_LIGATURES = 0x40;
        /// Enables display of contextual alternates
        const CONTEXTUAL = 0x80;
        /// Disables display of contextual alternates
        const NO_CONTEXTUAL = 0x100;
    }
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

impl ToCss for VariantLigatures {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.is_empty() {
            return dest.write_str("normal")
        }
        if self.contains(VariantLigatures::NONE) {
            return dest.write_str("none")
        }

        let mut has_any = false;

        macro_rules! write_value {
            ($ident:path => $str:expr) => {
                if self.intersects($ident) {
                    if has_any {
                        dest.write_str(" ")?;
                    }
                    has_any = true;
                    dest.write_str($str)?;
                }
            }
        }

        write_value!(VariantLigatures::COMMON_LIGATURES => "common-ligatures");
        write_value!(VariantLigatures::NO_COMMON_LIGATURES => "no-common-ligatures");
        write_value!(VariantLigatures::DISCRETIONARY_LIGATURES => "discretionary-ligatures");
        write_value!(VariantLigatures::NO_DISCRETIONARY_LIGATURES => "no-discretionary-ligatures");
        write_value!(VariantLigatures::HISTORICAL_LIGATURES => "historical-ligatures");
        write_value!(VariantLigatures::NO_HISTORICAL_LIGATURES => "no-historical-ligatures");
        write_value!(VariantLigatures::CONTEXTUAL => "contextual");
        write_value!(VariantLigatures::NO_CONTEXTUAL => "no-contextual");

        debug_assert!(has_any);
        Ok(())
    }
}

#[cfg(feature = "gecko")]
impl_gecko_keyword_conversions!(VariantLigatures, u16);

/// Asserts that all variant-east-asian matches its NS_FONT_VARIANT_EAST_ASIAN_* value.
#[cfg(feature = "gecko")]
#[inline]
pub fn assert_variant_ligatures_matches() {
    use gecko_bindings::structs;

    macro_rules! check_variant_ligatures {
        ( $( $a:ident => $b:path),*, ) => {
            if cfg!(debug_assertions) {
                $(
                    assert_eq!(structs::$a as u16, $b.bits());
                )*
            }
        }
    }

    check_variant_ligatures! {
        NS_FONT_VARIANT_LIGATURES_NONE => VariantLigatures::NONE,
        NS_FONT_VARIANT_LIGATURES_COMMON => VariantLigatures::COMMON_LIGATURES,
        NS_FONT_VARIANT_LIGATURES_NO_COMMON => VariantLigatures::NO_COMMON_LIGATURES,
        NS_FONT_VARIANT_LIGATURES_DISCRETIONARY => VariantLigatures::DISCRETIONARY_LIGATURES,
        NS_FONT_VARIANT_LIGATURES_NO_DISCRETIONARY => VariantLigatures::NO_DISCRETIONARY_LIGATURES,
        NS_FONT_VARIANT_LIGATURES_HISTORICAL => VariantLigatures::HISTORICAL_LIGATURES,
        NS_FONT_VARIANT_LIGATURES_NO_HISTORICAL => VariantLigatures::NO_HISTORICAL_LIGATURES,
        NS_FONT_VARIANT_LIGATURES_CONTEXTUAL => VariantLigatures::CONTEXTUAL,
        NS_FONT_VARIANT_LIGATURES_NO_CONTEXTUAL => VariantLigatures::NO_CONTEXTUAL,
    }
}

#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq, ToCss)]
/// Ligatures and contextual forms are ways of combining glyphs
/// to produce more harmonized forms
pub enum FontVariantLigatures {
    /// Value variant with `variant-ligatures`
    Value(VariantLigatures),
    /// System font variant
    System(SystemFont)
}

impl FontVariantLigatures {
    /// Get `font-variant-ligatures` with system font
    pub fn system_font(f: SystemFont) -> Self {
        FontVariantLigatures::System(f)
    }

    /// Get system font
    pub fn get_system(&self) -> Option<SystemFont> {
        if let FontVariantLigatures::System(s) = *self {
            Some(s)
        } else {
            None
        }
    }

    #[inline]
    /// Default value of `font-variant-ligatures` as `empty`
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

    fn to_computed_value(&self, _context: &Context) -> computed::FontVariantLigatures {
        match *self {
            FontVariantLigatures::Value(ref v) => v.clone(),
            FontVariantLigatures::System(_) => {
                #[cfg(feature = "gecko")] {
                    _context.cached_system_font.as_ref().unwrap().font_variant_ligatures.clone()
                }
                #[cfg(feature = "servo")] {
                    unreachable!()
                }
            }
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
    fn parse<'i, 't> (
        _context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<FontVariantLigatures, ParseError<'i>> {
        let mut result = VariantLigatures::empty();

        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(FontVariantLigatures::Value(result))
        }
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(FontVariantLigatures::Value(VariantLigatures::NONE))
        }

        while let Ok(flag) = input.try(|input| {
            Ok(match_ignore_ascii_case! { &input.expect_ident().map_err(|_| ())?,
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
            })
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

bitflags! {
    #[derive(MallocSizeOf)]
    /// Vairants of numeric values
    pub struct VariantNumeric: u8 {
        /// None of other variants are enabled.
        const NORMAL = 0;
        /// Enables display of lining numerals.
        const LINING_NUMS = 0x01;
        /// Enables display of old-style numerals.
        const OLDSTYLE_NUMS = 0x02;
        /// Enables display of proportional numerals.
        const PROPORTIONAL_NUMS = 0x04;
        /// Enables display of tabular numerals.
        const TABULAR_NUMS = 0x08;
        /// Enables display of lining diagonal fractions.
        const DIAGONAL_FRACTIONS = 0x10;
        /// Enables display of lining stacked fractions.
        const STACKED_FRACTIONS = 0x20;
        /// Enables display of letter forms used with ordinal numbers.
        const ORDINAL = 0x80;
        /// Enables display of slashed zeros.
        const SLASHED_ZERO = 0x40;
    }
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

impl ToCss for VariantNumeric {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.is_empty() {
            return dest.write_str("normal")
        }

        let mut has_any = false;

        macro_rules! write_value {
            ($ident:path => $str:expr) => {
                if self.intersects($ident) {
                    if has_any {
                        dest.write_str(" ")?;
                    }
                    has_any = true;
                    dest.write_str($str)?;
                }
            }
        }

        write_value!(VariantNumeric::LINING_NUMS => "lining-nums");
        write_value!(VariantNumeric::OLDSTYLE_NUMS => "oldstyle-nums");
        write_value!(VariantNumeric::PROPORTIONAL_NUMS => "proportional-nums");
        write_value!(VariantNumeric::TABULAR_NUMS => "tabular-nums");
        write_value!(VariantNumeric::DIAGONAL_FRACTIONS => "diagonal-fractions");
        write_value!(VariantNumeric::STACKED_FRACTIONS => "stacked-fractions");
        write_value!(VariantNumeric::SLASHED_ZERO => "slashed-zero");
        write_value!(VariantNumeric::ORDINAL => "ordinal");

        debug_assert!(has_any);
        Ok(())
    }
}

#[cfg(feature = "gecko")]
impl_gecko_keyword_conversions!(VariantNumeric, u8);

/// Asserts that all variant-east-asian matches its NS_FONT_VARIANT_EAST_ASIAN_* value.
#[cfg(feature = "gecko")]
#[inline]
pub fn assert_variant_numeric_matches() {
    use gecko_bindings::structs;

    macro_rules! check_variant_numeric {
        ( $( $a:ident => $b:path),*, ) => {
            if cfg!(debug_assertions) {
                $(
                    assert_eq!(structs::$a as u8, $b.bits());
                )*
            }
        }
    }

    check_variant_numeric! {
        NS_FONT_VARIANT_NUMERIC_LINING => VariantNumeric::LINING_NUMS,
        NS_FONT_VARIANT_NUMERIC_OLDSTYLE => VariantNumeric::OLDSTYLE_NUMS,
        NS_FONT_VARIANT_NUMERIC_PROPORTIONAL => VariantNumeric::PROPORTIONAL_NUMS,
        NS_FONT_VARIANT_NUMERIC_TABULAR => VariantNumeric::TABULAR_NUMS,
        NS_FONT_VARIANT_NUMERIC_DIAGONAL_FRACTIONS => VariantNumeric::DIAGONAL_FRACTIONS,
        NS_FONT_VARIANT_NUMERIC_STACKED_FRACTIONS => VariantNumeric::STACKED_FRACTIONS,
        NS_FONT_VARIANT_NUMERIC_SLASHZERO => VariantNumeric::SLASHED_ZERO,
        NS_FONT_VARIANT_NUMERIC_ORDINAL => VariantNumeric::ORDINAL,
    }
}

#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq, ToCss)]
/// Specifies control over numerical forms.
pub enum FontVariantNumeric {
    /// Value variant with `variant-numeric`
    Value(VariantNumeric),
    /// System font
    System(SystemFont)
}

impl FontVariantNumeric {
    #[inline]
    /// Default value of `font-variant-numeric` as `empty`
    pub fn empty() -> FontVariantNumeric {
        FontVariantNumeric::Value(VariantNumeric::empty())
    }

    /// Get `font-variant-numeric` with system font
    pub fn system_font(f: SystemFont) -> Self {
        FontVariantNumeric::System(f)
    }

    /// Get system font
    pub fn get_system(&self) -> Option<SystemFont> {
        if let FontVariantNumeric::System(s) = *self {
            Some(s)
        } else {
            None
        }
    }
}

impl ToComputedValue for FontVariantNumeric {
    type ComputedValue = computed::FontVariantNumeric;

    fn to_computed_value(&self, _context: &Context) -> computed::FontVariantNumeric {
        match *self {
            FontVariantNumeric::Value(ref v) => v.clone(),
            FontVariantNumeric::System(_) => {
                #[cfg(feature = "gecko")] {
                    _context.cached_system_font.as_ref().unwrap().font_variant_numeric.clone()
                }
                #[cfg(feature = "servo")] {
                    unreachable!()
                }
            }
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
        input: &mut Parser<'i, 't>
    ) -> Result<FontVariantNumeric, ParseError<'i>> {
        let mut result = VariantNumeric::empty();

        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(FontVariantNumeric::Value(result))
        }

        while let Ok(flag) = input.try(|input| {
            Ok(match_ignore_ascii_case! { &input.expect_ident().map_err(|_| ())?,
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
            })
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

#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq, ToCss)]
/// Define initial settings that apply when the font defined
/// by an @font-face rule is rendered.
pub enum FontFeatureSettings {
    /// Value of `FontSettings`
    Value(computed::FontFeatureSettings),
    /// System font
    System(SystemFont)
}

impl FontFeatureSettings {
    #[inline]
    /// Get default value of `font-feature-settings` as normal
    pub fn normal() -> FontFeatureSettings {
        FontFeatureSettings::Value(FontSettings::Normal)
    }

    /// Get `font-feature-settings` with system font
    pub fn system_font(f: SystemFont) -> Self {
        FontFeatureSettings::System(f)
    }

    /// Get system font
    pub fn get_system(&self) -> Option<SystemFont> {
        if let FontFeatureSettings::System(s) = *self {
            Some(s)
        } else {
            None
        }
    }
}

impl ToComputedValue for FontFeatureSettings {
    type ComputedValue = computed::FontFeatureSettings;

    fn to_computed_value(&self, _context: &Context) -> computed::FontFeatureSettings {
        match *self {
            FontFeatureSettings::Value(ref v) => v.clone(),
            FontFeatureSettings::System(_) => {
                #[cfg(feature = "gecko")] {
                    _context.cached_system_font.as_ref().unwrap().font_feature_settings.clone()
                }
                #[cfg(feature = "servo")] {
                    unreachable!()
                }
            }
        }
    }

    fn from_computed_value(other: &computed::FontFeatureSettings) -> Self {
        FontFeatureSettings::Value(other.clone())
    }
}

impl Parse for FontFeatureSettings {
    /// normal | <feature-tag-value>#
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<FontFeatureSettings, ParseError<'i>> {
        computed::FontFeatureSettings::parse(context, input).map(FontFeatureSettings::Value)
    }
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
/// Whether user agents are allowed to synthesize bold or oblique font faces
/// when a font family lacks bold or italic faces
pub struct FontSynthesis {
    /// If a `font-weight` is requested that the font family does not contain,
    /// the user agent may synthesize the requested weight from the weights
    /// that do exist in the font family.
    pub weight: bool,
    /// If a font-style is requested that the font family does not contain,
    /// the user agent may synthesize the requested style from the normal face in the font family.
    pub style: bool,
}

impl FontSynthesis {
    #[inline]
    /// Get the default value of font-synthesis
    pub fn get_initial_value() -> Self {
        FontSynthesis {
            weight: true,
            style: true
        }
    }
}

impl Parse for FontSynthesis {
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<FontSynthesis, ParseError<'i>> {
        let mut result = FontSynthesis { weight: false, style: false };
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
        use gecko_bindings::structs;

        FontSynthesis {
            weight: bits & structs::NS_FONT_SYNTHESIS_WEIGHT as u8 != 0,
            style: bits & structs::NS_FONT_SYNTHESIS_STYLE as u8 != 0
        }
    }
}

#[cfg(feature = "gecko")]
impl From<FontSynthesis> for u8 {
    fn from(v: FontSynthesis) -> u8 {
        use gecko_bindings::structs;

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

#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, ToCss)]
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
    System(SystemFont)
}

impl FontLanguageOverride {
    #[inline]
    /// Get default value with `normal`
    pub fn normal() -> FontLanguageOverride {
        FontLanguageOverride::Normal
    }

    /// Get `font-language-override` with `system font`
    pub fn system_font(f: SystemFont) -> Self {
        FontLanguageOverride::System(f)
    }

    /// Get system font
    pub fn get_system(&self) -> Option<SystemFont> {
        if let FontLanguageOverride::System(s) = *self {
            Some(s)
        } else {
            None
        }
    }
}

impl ToComputedValue for FontLanguageOverride {
    type ComputedValue = computed::FontLanguageOverride;

    #[inline]
    fn to_computed_value(&self, _context: &Context) -> computed::FontLanguageOverride {
        #[allow(unused_imports)] use std::ascii::AsciiExt;
        match *self {
            FontLanguageOverride::Normal => computed::FontLanguageOverride(0),
            FontLanguageOverride::Override(ref lang) => {
                if lang.is_empty() || lang.len() > 4 || !lang.is_ascii() {
                    return computed::FontLanguageOverride(0)
                }
                let mut computed_lang = lang.to_string();
                while computed_lang.len() < 4 {
                    computed_lang.push(' ');
                }
                let bytes = computed_lang.into_bytes();
                computed::FontLanguageOverride(BigEndian::read_u32(&bytes))
            }
            FontLanguageOverride::System(_) => {
                #[cfg(feature = "gecko")] {
                    _context.cached_system_font.as_ref().unwrap().font_language_override
                }
                #[cfg(feature = "servo")] {
                    unreachable!()
                }
            }
        }
    }
    #[inline]
    fn from_computed_value(computed: &computed::FontLanguageOverride) -> Self {
        if computed.0 == 0 {
            return FontLanguageOverride::Normal
        }
        let mut buf = [0; 4];
        BigEndian::write_u32(&mut buf, computed.0);
        FontLanguageOverride::Override(
            if cfg!(debug_assertions) {
                String::from_utf8(buf.to_vec()).unwrap()
            } else {
                unsafe { String::from_utf8_unchecked(buf.to_vec()) }
            }.into_boxed_str()
        )
    }
}

impl Parse for FontLanguageOverride {
    /// normal | <string>
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<FontLanguageOverride, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(FontLanguageOverride::Normal)
        }

        let string = input.expect_string()?;
        Ok(FontLanguageOverride::Override(string.as_ref().to_owned().into_boxed_str()))
    }
}

/// This property provides low-level control over OpenType or TrueType font variations.
pub type FontVariantSettings = FontSettings<FontSettingTagFloat>;

#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
/// text-zoom. Enable if true, disable if false
pub struct XTextZoom(pub bool);

impl Parse for XTextZoom {
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<XTextZoom, ParseError<'i>> {
        debug_assert!(false, "Should be set directly by presentation attributes only.");
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

impl ToCss for XTextZoom {
    fn to_css<W>(&self, _: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        Ok(())
    }
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
/// Internal property that reflects the lang attribute
pub struct XLang(pub Atom);

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
        input: &mut Parser<'i, 't>
    ) -> Result<XLang, ParseError<'i>> {
        debug_assert!(false, "Should be set directly by presentation attributes only.");
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

impl ToCss for XLang {
    fn to_css<W>(&self, _: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        Ok(())
    }
}

#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq, ToCss)]
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
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<MozScriptMinSize, ParseError<'i>> {
        debug_assert!(false, "Should be set directly by presentation attributes only.");
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, ToCss)]
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
    Auto
}

impl Parse for MozScriptLevel {
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<MozScriptLevel, ParseError<'i>> {
        if let Ok(i) = input.try(|i| i.expect_integer()) {
            return Ok(MozScriptLevel::Relative(i))
        }
        input.expect_ident_matching("auto")?;
        Ok(MozScriptLevel::Auto)
    }
}

#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, ToComputedValue, ToCss)]
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
        input: &mut Parser<'i, 't>
    ) -> Result<MozScriptSizeMultiplier, ParseError<'i>> {
        debug_assert!(false, "Should be set directly by presentation attributes only.");
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
