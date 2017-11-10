/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified values for font properties

#[cfg(feature = "gecko")]
use Atom;
use app_units::Au;
use cssparser::{Parser, Token};
use parser::{Parse, ParserContext};
use properties::longhands::system_font::SystemFont;
use std::fmt;
use style_traits::{ToCss, StyleParseErrorKind, ParseError};
use values::computed::{font as computed, Context, Length, NonNegativeLength, ToComputedValue};
use values::specified::{AllowQuirks, LengthOrPercentage, NoCalcLength, Number};
use values::specified::length::{AU_PER_PT, AU_PER_PX, FontBaseSize};

const DEFAULT_SCRIPT_MIN_SIZE_PT: u32 = 8;

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
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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
                    // Compute it, but shave off the font-relative part (em, %)
                    // This will mean that other font-relative units like ex and ch will be computed against
                    // the old font even when the font changes. There's no particular "right answer" for what
                    // to do here -- Gecko recascades as if the font had changed, we instead track the changes
                    // and reapply, which means that we carry over old computed ex/ch values whilst Gecko
                    // recomputes new ones. This is enough of an edge case to not really matter.
                    let abs = calc.to_computed_value_zoomed(context, FontBaseSize::Custom(Au(0).into()))
                                  .length_component().into();
                    info = parent.keyword_info.map(|i| i.compose(ratio, abs));
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
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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
    fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write {
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
#[derive(Clone, Copy, Debug, PartialEq)]
/// Changes the scriptlevel in effect for the children.
/// Ref: https://wiki.mozilla.org/MathML:mstyle
///
/// The main effect of scriptlevel is to control the font size.
/// https://www.w3.org/TR/MathML3/chapter3.html#presm.scriptlevel
pub enum MozScriptLevel {
    /// Change `font-size` relatively
    Relative(i32),
    /// Change `font-size` absolutely
    Absolute(i32),
    /// Change `font-size` automatically
    Auto
}

impl ToCss for MozScriptLevel {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            MozScriptLevel::Auto => dest.write_str("auto"),
            MozScriptLevel::Relative(rel) => rel.to_css(dest),
            // can only be specified by pres attrs; should not
            // serialize to anything else
            MozScriptLevel::Absolute(_) => Ok(()),
        }
    }
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
