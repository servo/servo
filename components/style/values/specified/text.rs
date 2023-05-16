/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for text properties.

use crate::parser::{Parse, ParserContext};
use crate::properties::longhands::writing_mode::computed_value::T as SpecifiedWritingMode;
use crate::values::computed::text::LineHeight as ComputedLineHeight;
use crate::values::computed::text::TextEmphasisStyle as ComputedTextEmphasisStyle;
use crate::values::computed::text::TextOverflow as ComputedTextOverflow;
use crate::values::computed::{Context, ToComputedValue};
use crate::values::generics::text::InitialLetter as GenericInitialLetter;
use crate::values::generics::text::LineHeight as GenericLineHeight;
use crate::values::generics::text::{GenericTextDecorationLength, Spacing};
use crate::values::specified::length::NonNegativeLengthPercentage;
use crate::values::specified::length::{FontRelativeLength, Length};
use crate::values::specified::length::{LengthPercentage, NoCalcLength};
use crate::values::specified::{AllowQuirks, Integer, NonNegativeNumber, Number};
use cssparser::{Parser, Token};
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Write};
use style_traits::values::SequenceWriter;
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};
use style_traits::{KeywordsCollectFn, SpecifiedValueInfo};
use unicode_segmentation::UnicodeSegmentation;

/// A specified type for the `initial-letter` property.
pub type InitialLetter = GenericInitialLetter<Number, Integer>;

/// A specified value for the `letter-spacing` property.
pub type LetterSpacing = Spacing<Length>;

/// A specified value for the `word-spacing` property.
pub type WordSpacing = Spacing<LengthPercentage>;

/// A specified value for the `line-height` property.
pub type LineHeight = GenericLineHeight<NonNegativeNumber, NonNegativeLengthPercentage>;

impl Parse for InitialLetter {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input
            .try_parse(|i| i.expect_ident_matching("normal"))
            .is_ok()
        {
            return Ok(GenericInitialLetter::Normal);
        }
        let size = Number::parse_at_least_one(context, input)?;
        let sink = input
            .try_parse(|i| Integer::parse_positive(context, i))
            .ok();
        Ok(GenericInitialLetter::Specified(size, sink))
    }
}

impl Parse for LetterSpacing {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Spacing::parse_with(context, input, |c, i| {
            Length::parse_quirky(c, i, AllowQuirks::Yes)
        })
    }
}

impl Parse for WordSpacing {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Spacing::parse_with(context, input, |c, i| {
            LengthPercentage::parse_quirky(c, i, AllowQuirks::Yes)
        })
    }
}

impl ToComputedValue for LineHeight {
    type ComputedValue = ComputedLineHeight;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        use crate::values::specified::length::FontBaseSize;
        match *self {
            GenericLineHeight::Normal => GenericLineHeight::Normal,
            #[cfg(feature = "gecko")]
            GenericLineHeight::MozBlockHeight => GenericLineHeight::MozBlockHeight,
            GenericLineHeight::Number(number) => {
                GenericLineHeight::Number(number.to_computed_value(context))
            },
            GenericLineHeight::Length(ref non_negative_lp) => {
                let result = match non_negative_lp.0 {
                    LengthPercentage::Length(NoCalcLength::Absolute(ref abs)) => {
                        context.maybe_zoom_text(abs.to_computed_value(context))
                    },
                    LengthPercentage::Length(ref length) => length.to_computed_value(context),
                    LengthPercentage::Percentage(ref p) => FontRelativeLength::Em(p.0)
                        .to_computed_value(context, FontBaseSize::CurrentStyle),
                    LengthPercentage::Calc(ref calc) => {
                        let computed_calc =
                            calc.to_computed_value_zoomed(context, FontBaseSize::CurrentStyle);
                        let base = context.style().get_font().clone_font_size().size();
                        computed_calc.resolve(base)
                    },
                };
                GenericLineHeight::Length(result.into())
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            GenericLineHeight::Normal => GenericLineHeight::Normal,
            #[cfg(feature = "gecko")]
            GenericLineHeight::MozBlockHeight => GenericLineHeight::MozBlockHeight,
            GenericLineHeight::Number(ref number) => {
                GenericLineHeight::Number(NonNegativeNumber::from_computed_value(number))
            },
            GenericLineHeight::Length(ref length) => {
                GenericLineHeight::Length(NoCalcLength::from_computed_value(&length.0).into())
            },
        }
    }
}

/// A generic value for the `text-overflow` property.
#[derive(
    Clone,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum TextOverflowSide {
    /// Clip inline content.
    Clip,
    /// Render ellipsis to represent clipped inline content.
    Ellipsis,
    /// Render a given string to represent clipped inline content.
    String(crate::OwnedStr),
}

#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
/// text-overflow. Specifies rendering when inline content overflows its line box edge.
pub struct TextOverflow {
    /// First value. Applies to end line box edge if no second is supplied; line-left edge otherwise.
    pub first: TextOverflowSide,
    /// Second value. Applies to the line-right edge if supplied.
    pub second: Option<TextOverflowSide>,
}

impl Parse for TextOverflow {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<TextOverflow, ParseError<'i>> {
        let first = TextOverflowSide::parse(context, input)?;
        let second = input
            .try_parse(|input| TextOverflowSide::parse(context, input))
            .ok();
        Ok(TextOverflow { first, second })
    }
}

impl ToComputedValue for TextOverflow {
    type ComputedValue = ComputedTextOverflow;

    #[inline]
    fn to_computed_value(&self, _context: &Context) -> Self::ComputedValue {
        if let Some(ref second) = self.second {
            Self::ComputedValue {
                first: self.first.clone(),
                second: second.clone(),
                sides_are_logical: false,
            }
        } else {
            Self::ComputedValue {
                first: TextOverflowSide::Clip,
                second: self.first.clone(),
                sides_are_logical: true,
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        if computed.sides_are_logical {
            assert_eq!(computed.first, TextOverflowSide::Clip);
            TextOverflow {
                first: computed.second.clone(),
                second: None,
            }
        } else {
            TextOverflow {
                first: computed.first.clone(),
                second: Some(computed.second.clone()),
            }
        }
    }
}

bitflags! {
    #[derive(MallocSizeOf, Serialize, SpecifiedValueInfo, ToComputedValue, ToResolvedValue, ToShmem)]
    #[value_info(other_values = "none,underline,overline,line-through,blink")]
    #[repr(C)]
    /// Specified keyword values for the text-decoration-line property.
    pub struct TextDecorationLine: u8 {
        /// No text decoration line is specified.
        const NONE = 0;
        /// underline
        const UNDERLINE = 1 << 0;
        /// overline
        const OVERLINE = 1 << 1;
        /// line-through
        const LINE_THROUGH = 1 << 2;
        /// blink
        const BLINK = 1 << 3;
        /// Only set by presentation attributes
        ///
        /// Setting this will mean that text-decorations use the color
        /// specified by `color` in quirks mode.
        ///
        /// For example, this gives <a href=foo><font color="red">text</font></a>
        /// a red text decoration
        #[cfg(feature = "gecko")]
        const COLOR_OVERRIDE = 0x10;
    }
}

impl Default for TextDecorationLine {
    fn default() -> Self {
        TextDecorationLine::NONE
    }
}

impl Parse for TextDecorationLine {
    /// none | [ underline || overline || line-through || blink ]
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut result = TextDecorationLine::empty();

        // NOTE(emilio): this loop has this weird structure because we run this
        // code to parse the text-decoration shorthand as well, so we need to
        // ensure we don't return an error if we don't consume the whole thing
        // because we find an invalid identifier or other kind of token.
        loop {
            let flag: Result<_, ParseError<'i>> = input.try_parse(|input| {
                let flag = try_match_ident_ignore_ascii_case! { input,
                    "none" if result.is_empty() => TextDecorationLine::NONE,
                    "underline" => TextDecorationLine::UNDERLINE,
                    "overline" => TextDecorationLine::OVERLINE,
                    "line-through" => TextDecorationLine::LINE_THROUGH,
                    "blink" => TextDecorationLine::BLINK,
                };

                Ok(flag)
            });

            let flag = match flag {
                Ok(flag) => flag,
                Err(..) => break,
            };

            if flag.is_empty() {
                return Ok(TextDecorationLine::NONE);
            }

            if result.contains(flag) {
                return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }

            result.insert(flag)
        }

        if !result.is_empty() {
            Ok(result)
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

impl ToCss for TextDecorationLine {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.is_empty() {
            return dest.write_str("none");
        }

        #[cfg(feature = "gecko")]
        {
            if *self == TextDecorationLine::COLOR_OVERRIDE {
                return Ok(());
            }
        }

        let mut writer = SequenceWriter::new(dest, " ");
        let mut any = false;

        macro_rules! maybe_write {
            ($ident:ident => $str:expr) => {
                if self.contains(TextDecorationLine::$ident) {
                    any = true;
                    writer.raw_item($str)?;
                }
            };
        }

        maybe_write!(UNDERLINE => "underline");
        maybe_write!(OVERLINE => "overline");
        maybe_write!(LINE_THROUGH => "line-through");
        maybe_write!(BLINK => "blink");

        debug_assert!(any);

        Ok(())
    }
}

impl TextDecorationLine {
    #[inline]
    /// Returns the initial value of text-decoration-line
    pub fn none() -> Self {
        TextDecorationLine::NONE
    }
}

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
/// Specified value of the text-transform property, stored in two parts:
/// the case-related transforms (mutually exclusive, only one may be in effect), and others (non-exclusive).
pub struct TextTransform {
    /// Case transform, if any.
    pub case_: TextTransformCase,
    /// Non-case transforms.
    pub other_: TextTransformOther,
}

impl TextTransform {
    #[inline]
    /// Returns the initial value of text-transform
    pub fn none() -> Self {
        TextTransform {
            case_: TextTransformCase::None,
            other_: TextTransformOther::empty(),
        }
    }
    #[inline]
    /// Returns whether the value is 'none'
    pub fn is_none(&self) -> bool {
        self.case_ == TextTransformCase::None && self.other_.is_empty()
    }
}

impl Parse for TextTransform {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut result = TextTransform::none();

        // Case keywords are mutually exclusive; other transforms may co-occur.
        loop {
            let location = input.current_source_location();
            let ident = match input.next() {
                Ok(&Token::Ident(ref ident)) => ident,
                Ok(other) => return Err(location.new_unexpected_token_error(other.clone())),
                Err(..) => break,
            };

            match_ignore_ascii_case! { ident,
                "none" if result.is_none() => {
                    return Ok(result);
                },
                "uppercase" if result.case_ == TextTransformCase::None => {
                    result.case_ = TextTransformCase::Uppercase
                },
                "lowercase" if result.case_ == TextTransformCase::None => {
                    result.case_ = TextTransformCase::Lowercase
                },
                "capitalize" if result.case_ == TextTransformCase::None => {
                    result.case_ = TextTransformCase::Capitalize
                },
                "full-width" if !result.other_.intersects(TextTransformOther::FULL_WIDTH) => {
                    result.other_.insert(TextTransformOther::FULL_WIDTH)
                },
                "full-size-kana" if !result.other_.intersects(TextTransformOther::FULL_SIZE_KANA) => {
                    result.other_.insert(TextTransformOther::FULL_SIZE_KANA)
                },
                _ => return Err(location.new_custom_error(
                    SelectorParseErrorKind::UnexpectedIdent(ident.clone())
                )),
            }
        }

        if result.is_none() {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        } else {
            Ok(result)
        }
    }
}

impl ToCss for TextTransform {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.is_none() {
            return dest.write_str("none");
        }

        if self.case_ != TextTransformCase::None {
            self.case_.to_css(dest)?;
            if !self.other_.is_empty() {
                dest.write_str(" ")?;
            }
        }

        self.other_.to_css(dest)
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
/// Specified keyword values for case transforms in the text-transform property. (These are exclusive.)
pub enum TextTransformCase {
    /// No case transform.
    None,
    /// All uppercase.
    Uppercase,
    /// All lowercase.
    Lowercase,
    /// Capitalize each word.
    Capitalize,
}

bitflags! {
    #[derive(MallocSizeOf, SpecifiedValueInfo, ToComputedValue, ToResolvedValue, ToShmem)]
    #[value_info(other_values = "none,full-width,full-size-kana")]
    #[repr(C)]
    /// Specified keyword values for non-case transforms in the text-transform property. (Non-exclusive.)
    pub struct TextTransformOther: u8 {
        /// full-width
        const FULL_WIDTH = 1 << 0;
        /// full-size-kana
        const FULL_SIZE_KANA = 1 << 1;
    }
}

impl ToCss for TextTransformOther {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        let mut writer = SequenceWriter::new(dest, " ");
        let mut any = false;
        macro_rules! maybe_write {
            ($ident:ident => $str:expr) => {
                if self.contains(TextTransformOther::$ident) {
                    writer.raw_item($str)?;
                    any = true;
                }
            };
        }

        maybe_write!(FULL_WIDTH => "full-width");
        maybe_write!(FULL_SIZE_KANA => "full-size-kana");

        debug_assert!(any || self.is_empty());

        Ok(())
    }
}

/// Specified and computed value of text-align-last.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    FromPrimitive,
    Hash,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[allow(missing_docs)]
#[repr(u8)]
pub enum TextAlignLast {
    Auto,
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

/// Specified value of text-align keyword value.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    FromPrimitive,
    Hash,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[allow(missing_docs)]
#[repr(u8)]
pub enum TextAlignKeyword {
    Start,
    Left,
    Right,
    Center,
    #[cfg(any(feature = "gecko", feature = "servo-layout-2013"))]
    Justify,
    #[css(skip)]
    #[cfg(feature = "gecko")]
    Char,
    End,
    #[cfg(feature = "gecko")]
    MozCenter,
    #[cfg(feature = "gecko")]
    MozLeft,
    #[cfg(feature = "gecko")]
    MozRight,
    #[cfg(feature = "servo-layout-2013")]
    ServoCenter,
    #[cfg(feature = "servo-layout-2013")]
    ServoLeft,
    #[cfg(feature = "servo-layout-2013")]
    ServoRight,
}

/// Specified value of text-align property.
#[derive(
    Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem,
)]
pub enum TextAlign {
    /// Keyword value of text-align property.
    Keyword(TextAlignKeyword),
    /// `match-parent` value of text-align property. It has a different handling
    /// unlike other keywords.
    #[cfg(feature = "gecko")]
    MatchParent,
    /// `MozCenterOrInherit` value of text-align property. It cannot be parsed,
    /// only set directly on the elements and it has a different handling
    /// unlike other values.
    #[cfg(feature = "gecko")]
    #[css(skip)]
    MozCenterOrInherit,
}

impl ToComputedValue for TextAlign {
    type ComputedValue = TextAlignKeyword;

    #[inline]
    fn to_computed_value(&self, _context: &Context) -> Self::ComputedValue {
        match *self {
            TextAlign::Keyword(key) => key,
            #[cfg(feature = "gecko")]
            TextAlign::MatchParent => {
                // on the root <html> element we should still respect the dir
                // but the parent dir of that element is LTR even if it's <html dir=rtl>
                // and will only be RTL if certain prefs have been set.
                // In that case, the default behavior here will set it to left,
                // but we want to set it to right -- instead set it to the default (`start`),
                // which will do the right thing in this case (but not the general case)
                if _context.builder.is_root_element {
                    return TextAlignKeyword::Start;
                }
                let parent = _context
                    .builder
                    .get_parent_inherited_text()
                    .clone_text_align();
                let ltr = _context.builder.inherited_writing_mode().is_bidi_ltr();
                match (parent, ltr) {
                    (TextAlignKeyword::Start, true) => TextAlignKeyword::Left,
                    (TextAlignKeyword::Start, false) => TextAlignKeyword::Right,
                    (TextAlignKeyword::End, true) => TextAlignKeyword::Right,
                    (TextAlignKeyword::End, false) => TextAlignKeyword::Left,
                    _ => parent,
                }
            },
            #[cfg(feature = "gecko")]
            TextAlign::MozCenterOrInherit => {
                let parent = _context
                    .builder
                    .get_parent_inherited_text()
                    .clone_text_align();
                if parent == TextAlignKeyword::Start {
                    TextAlignKeyword::Center
                } else {
                    parent
                }
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        TextAlign::Keyword(*computed)
    }
}

fn fill_mode_is_default_and_shape_exists(
    fill: &TextEmphasisFillMode,
    shape: &Option<TextEmphasisShapeKeyword>,
) -> bool {
    shape.is_some() && fill.is_filled()
}

/// Specified value of text-emphasis-style property.
///
/// https://drafts.csswg.org/css-text-decor/#propdef-text-emphasis-style
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
#[allow(missing_docs)]
pub enum TextEmphasisStyle {
    /// [ <fill> || <shape> ]
    Keyword {
        #[css(contextual_skip_if = "fill_mode_is_default_and_shape_exists")]
        fill: TextEmphasisFillMode,
        shape: Option<TextEmphasisShapeKeyword>,
    },
    /// `none`
    None,
    /// `<string>` (of which only the first grapheme cluster will be used).
    String(crate::OwnedStr),
}

/// Fill mode for the text-emphasis-style property
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToCss,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum TextEmphasisFillMode {
    /// `filled`
    Filled,
    /// `open`
    Open,
}

impl TextEmphasisFillMode {
    /// Whether the value is `filled`.
    #[inline]
    pub fn is_filled(&self) -> bool {
        matches!(*self, TextEmphasisFillMode::Filled)
    }
}

/// Shape keyword for the text-emphasis-style property
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToCss,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum TextEmphasisShapeKeyword {
    /// `dot`
    Dot,
    /// `circle`
    Circle,
    /// `double-circle`
    DoubleCircle,
    /// `triangle`
    Triangle,
    /// `sesame`
    Sesame,
}

impl ToComputedValue for TextEmphasisStyle {
    type ComputedValue = ComputedTextEmphasisStyle;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            TextEmphasisStyle::Keyword { fill, shape } => {
                let shape = shape.unwrap_or_else(|| {
                    // FIXME(emilio, bug 1572958): This should set the
                    // rule_cache_conditions properly.
                    //
                    // Also should probably use WritingMode::is_vertical rather
                    // than the computed value of the `writing-mode` property.
                    if context.style().get_inherited_box().clone_writing_mode() ==
                        SpecifiedWritingMode::HorizontalTb
                    {
                        TextEmphasisShapeKeyword::Circle
                    } else {
                        TextEmphasisShapeKeyword::Sesame
                    }
                });
                ComputedTextEmphasisStyle::Keyword { fill, shape }
            },
            TextEmphasisStyle::None => ComputedTextEmphasisStyle::None,
            TextEmphasisStyle::String(ref s) => {
                // Passing `true` to iterate over extended grapheme clusters, following
                // recommendation at http://www.unicode.org/reports/tr29/#Grapheme_Cluster_Boundaries
                //
                // FIXME(emilio): Doing this at computed value time seems wrong.
                // The spec doesn't say that this should be a computed-value
                // time operation. This is observable from getComputedStyle().
                let s = s.graphemes(true).next().unwrap_or("").to_string();
                ComputedTextEmphasisStyle::String(s.into())
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            ComputedTextEmphasisStyle::Keyword { fill, shape } => TextEmphasisStyle::Keyword {
                fill,
                shape: Some(shape),
            },
            ComputedTextEmphasisStyle::None => TextEmphasisStyle::None,
            ComputedTextEmphasisStyle::String(ref string) => {
                TextEmphasisStyle::String(string.clone())
            },
        }
    }
}

impl Parse for TextEmphasisStyle {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input
            .try_parse(|input| input.expect_ident_matching("none"))
            .is_ok()
        {
            return Ok(TextEmphasisStyle::None);
        }

        if let Ok(s) = input.try_parse(|i| i.expect_string().map(|s| s.as_ref().to_owned())) {
            // Handle <string>
            return Ok(TextEmphasisStyle::String(s.into()));
        }

        // Handle a pair of keywords
        let mut shape = input.try_parse(TextEmphasisShapeKeyword::parse).ok();
        let fill = input.try_parse(TextEmphasisFillMode::parse).ok();
        if shape.is_none() {
            shape = input.try_parse(TextEmphasisShapeKeyword::parse).ok();
        }

        if shape.is_none() && fill.is_none() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        // If a shape keyword is specified but neither filled nor open is
        // specified, filled is assumed.
        let fill = fill.unwrap_or(TextEmphasisFillMode::Filled);

        // We cannot do the same because the default `<shape>` depends on the
        // computed writing-mode.
        Ok(TextEmphasisStyle::Keyword { fill, shape })
    }
}

/// The allowed horizontal values for the `text-emphasis-position` property.
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
pub enum TextEmphasisHorizontalWritingModeValue {
    /// Draw marks over the text in horizontal writing mode.
    Over,
    /// Draw marks under the text in horizontal writing mode.
    Under,
}

/// The allowed vertical values for the `text-emphasis-position` property.
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
pub enum TextEmphasisVerticalWritingModeValue {
    /// Draws marks to the right of the text in vertical writing mode.
    Right,
    /// Draw marks to the left of the text in vertical writing mode.
    Left,
}

/// Specified value of `text-emphasis-position` property.
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
pub struct TextEmphasisPosition(
    pub TextEmphasisHorizontalWritingModeValue,
    pub TextEmphasisVerticalWritingModeValue,
);

impl TextEmphasisPosition {
    #[inline]
    /// Returns the initial value of `text-emphasis-position`
    pub fn over_right() -> Self {
        TextEmphasisPosition(
            TextEmphasisHorizontalWritingModeValue::Over,
            TextEmphasisVerticalWritingModeValue::Right,
        )
    }

    #[cfg(feature = "gecko")]
    /// Converts an enumerated value coming from Gecko to a `TextEmphasisPosition`.
    pub fn from_gecko_keyword(kw: u32) -> Self {
        use crate::gecko_bindings::structs;

        let vert = if kw & structs::NS_STYLE_TEXT_EMPHASIS_POSITION_RIGHT != 0 {
            TextEmphasisVerticalWritingModeValue::Right
        } else {
            debug_assert!(kw & structs::NS_STYLE_TEXT_EMPHASIS_POSITION_LEFT != 0);
            TextEmphasisVerticalWritingModeValue::Left
        };
        let horiz = if kw & structs::NS_STYLE_TEXT_EMPHASIS_POSITION_OVER != 0 {
            TextEmphasisHorizontalWritingModeValue::Over
        } else {
            debug_assert!(kw & structs::NS_STYLE_TEXT_EMPHASIS_POSITION_UNDER != 0);
            TextEmphasisHorizontalWritingModeValue::Under
        };
        TextEmphasisPosition(horiz, vert)
    }
}

impl Parse for TextEmphasisPosition {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(horizontal) =
            input.try_parse(|input| TextEmphasisHorizontalWritingModeValue::parse(input))
        {
            let vertical = TextEmphasisVerticalWritingModeValue::parse(input)?;
            Ok(TextEmphasisPosition(horizontal, vertical))
        } else {
            let vertical = TextEmphasisVerticalWritingModeValue::parse(input)?;
            let horizontal = TextEmphasisHorizontalWritingModeValue::parse(input)?;
            Ok(TextEmphasisPosition(horizontal, vertical))
        }
    }
}

#[cfg(feature = "gecko")]
impl From<u8> for TextEmphasisPosition {
    fn from(bits: u8) -> Self {
        TextEmphasisPosition::from_gecko_keyword(bits as u32)
    }
}

#[cfg(feature = "gecko")]
impl From<TextEmphasisPosition> for u8 {
    fn from(v: TextEmphasisPosition) -> u8 {
        use crate::gecko_bindings::structs;

        let mut result = match v.0 {
            TextEmphasisHorizontalWritingModeValue::Over => {
                structs::NS_STYLE_TEXT_EMPHASIS_POSITION_OVER
            },
            TextEmphasisHorizontalWritingModeValue::Under => {
                structs::NS_STYLE_TEXT_EMPHASIS_POSITION_UNDER
            },
        };
        match v.1 {
            TextEmphasisVerticalWritingModeValue::Right => {
                result |= structs::NS_STYLE_TEXT_EMPHASIS_POSITION_RIGHT;
            },
            TextEmphasisVerticalWritingModeValue::Left => {
                result |= structs::NS_STYLE_TEXT_EMPHASIS_POSITION_LEFT;
            },
        };
        result as u8
    }
}

/// Values for the `word-break` property.
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
#[allow(missing_docs)]
pub enum WordBreak {
    Normal,
    BreakAll,
    KeepAll,
    /// The break-word value, needed for compat.
    ///
    /// Specifying `word-break: break-word` makes `overflow-wrap` behave as
    /// `anywhere`, and `word-break` behave like `normal`.
    #[cfg(feature = "gecko")]
    BreakWord,
}

/// Values for the `text-justify` CSS property.
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
#[allow(missing_docs)]
pub enum TextJustify {
    Auto,
    None,
    InterWord,
    // See https://drafts.csswg.org/css-text-3/#valdef-text-justify-distribute
    // and https://github.com/w3c/csswg-drafts/issues/6156 for the alias.
    #[parse(aliases = "distribute")]
    InterCharacter,
}

/// Values for the `-moz-control-character-visibility` CSS property.
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
#[allow(missing_docs)]
pub enum MozControlCharacterVisibility {
    Hidden,
    Visible,
}

impl Default for MozControlCharacterVisibility {
    fn default() -> Self {
        if static_prefs::pref!("layout.css.control-characters.visible") {
            Self::Visible
        } else {
            Self::Hidden
        }
    }
}


/// Values for the `line-break` property.
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
#[allow(missing_docs)]
pub enum LineBreak {
    Auto,
    Loose,
    Normal,
    Strict,
    Anywhere,
}

/// Values for the `overflow-wrap` property.
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
#[allow(missing_docs)]
pub enum OverflowWrap {
    Normal,
    BreakWord,
    Anywhere,
}

/// Implements text-decoration-skip-ink which takes the keywords auto | none | all
///
/// https://drafts.csswg.org/css-text-decor-4/#text-decoration-skip-ink-property
#[repr(u8)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
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
#[allow(missing_docs)]
pub enum TextDecorationSkipInk {
    Auto,
    None,
    All,
}

/// Implements type for `text-decoration-thickness` property
pub type TextDecorationLength = GenericTextDecorationLength<LengthPercentage>;

impl TextDecorationLength {
    /// `Auto` value.
    #[inline]
    pub fn auto() -> Self {
        GenericTextDecorationLength::Auto
    }

    /// Whether this is the `Auto` value.
    #[inline]
    pub fn is_auto(&self) -> bool {
        matches!(*self, GenericTextDecorationLength::Auto)
    }
}

bitflags! {
    #[derive(MallocSizeOf, SpecifiedValueInfo, ToComputedValue, ToResolvedValue, ToShmem)]
    #[value_info(other_values = "auto,from-font,under,left,right")]
    #[repr(C)]
    /// Specified keyword values for the text-underline-position property.
    /// (Non-exclusive, but not all combinations are allowed: the spec grammar gives
    /// `auto | [ from-font | under ] || [ left | right ]`.)
    /// https://drafts.csswg.org/css-text-decor-4/#text-underline-position-property
    pub struct TextUnderlinePosition: u8 {
        /// Use automatic positioning below the alphabetic baseline.
        const AUTO = 0;
        /// Use underline position from the first available font.
        const FROM_FONT = 1 << 0;
        /// Below the glyph box.
        const UNDER = 1 << 1;
        /// In vertical mode, place to the left of the text.
        const LEFT = 1 << 2;
        /// In vertical mode, place to the right of the text.
        const RIGHT = 1 << 3;
    }
}

impl Parse for TextUnderlinePosition {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<TextUnderlinePosition, ParseError<'i>> {
        let mut result = TextUnderlinePosition::empty();

        loop {
            let location = input.current_source_location();
            let ident = match input.next() {
                Ok(&Token::Ident(ref ident)) => ident,
                Ok(other) => return Err(location.new_unexpected_token_error(other.clone())),
                Err(..) => break,
            };

            match_ignore_ascii_case! { ident,
                "auto" if result.is_empty() => {
                    return Ok(result);
                },
                "from-font" if !result.intersects(TextUnderlinePosition::FROM_FONT |
                                                  TextUnderlinePosition::UNDER) => {
                    result.insert(TextUnderlinePosition::FROM_FONT);
                },
                "under" if !result.intersects(TextUnderlinePosition::FROM_FONT |
                                              TextUnderlinePosition::UNDER) => {
                    result.insert(TextUnderlinePosition::UNDER);
                },
                "left" if !result.intersects(TextUnderlinePosition::LEFT |
                                             TextUnderlinePosition::RIGHT) => {
                    result.insert(TextUnderlinePosition::LEFT);
                },
                "right" if !result.intersects(TextUnderlinePosition::LEFT |
                                              TextUnderlinePosition::RIGHT) => {
                    result.insert(TextUnderlinePosition::RIGHT);
                },
                _ => return Err(location.new_custom_error(
                    SelectorParseErrorKind::UnexpectedIdent(ident.clone())
                )),
            }
        }

        if !result.is_empty() {
            Ok(result)
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

impl ToCss for TextUnderlinePosition {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.is_empty() {
            return dest.write_str("auto");
        }

        let mut writer = SequenceWriter::new(dest, " ");
        let mut any = false;

        macro_rules! maybe_write {
            ($ident:ident => $str:expr) => {
                if self.contains(TextUnderlinePosition::$ident) {
                    any = true;
                    writer.raw_item($str)?;
                }
            };
        }

        maybe_write!(FROM_FONT => "from-font");
        maybe_write!(UNDER => "under");
        maybe_write!(LEFT => "left");
        maybe_write!(RIGHT => "right");

        debug_assert!(any);

        Ok(())
    }
}

/// Values for `ruby-position` property
#[repr(u8)]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[allow(missing_docs)]
pub enum RubyPosition {
    AlternateOver,
    AlternateUnder,
    Over,
    Under,
}

impl Parse for RubyPosition {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<RubyPosition, ParseError<'i>> {
        // Parse alternate before
        let alternate = input.try_parse(|i| i.expect_ident_matching("alternate")).is_ok();
        if alternate && input.is_exhausted() {
            return Ok(RubyPosition::AlternateOver);
        }
        // Parse over / under
        let over = try_match_ident_ignore_ascii_case! { input,
            "over" => true,
            "under" => false,
        };
        // Parse alternate after
        let alternate = alternate ||
             input.try_parse(|i| i.expect_ident_matching("alternate")).is_ok();

        Ok(match (over, alternate) {
            (true, true) => RubyPosition::AlternateOver,
            (false, true) => RubyPosition::AlternateUnder,
            (true, false) => RubyPosition::Over,
            (false, false) => RubyPosition::Under,
        })
    }
}

impl ToCss for RubyPosition {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str(match self {
            RubyPosition::AlternateOver => "alternate",
            RubyPosition::AlternateUnder => "alternate under",
            RubyPosition::Over => "over",
            RubyPosition::Under => "under",
        })
    }
}

impl SpecifiedValueInfo for RubyPosition {
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        f(&["alternate", "over", "under"])
    }
}
