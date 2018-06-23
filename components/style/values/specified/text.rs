/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for text properties.

use cssparser::{Parser, Token};
use parser::{Parse, ParserContext};
use properties::longhands::writing_mode::computed_value::T as SpecifiedWritingMode;
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Write};
use style_traits::{CssWriter, KeywordsCollectFn, ParseError};
use style_traits::{SpecifiedValueInfo, StyleParseErrorKind, ToCss};
use style_traits::values::SequenceWriter;
use unicode_segmentation::UnicodeSegmentation;
use values::computed::{Context, ToComputedValue};
use values::computed::text::LineHeight as ComputedLineHeight;
use values::computed::text::TextEmphasisKeywordValue as ComputedTextEmphasisKeywordValue;
use values::computed::text::TextEmphasisStyle as ComputedTextEmphasisStyle;
use values::computed::text::TextOverflow as ComputedTextOverflow;
use values::generics::text::InitialLetter as GenericInitialLetter;
use values::generics::text::LineHeight as GenericLineHeight;
use values::generics::text::MozTabSize as GenericMozTabSize;
use values::generics::text::Spacing;
use values::specified::{AllowQuirks, Integer, NonNegativeNumber, Number};
use values::specified::length::{FontRelativeLength, Length, LengthOrPercentage, NoCalcLength};
use values::specified::length::{NonNegativeLength, NonNegativeLengthOrPercentage};

/// A specified type for the `initial-letter` property.
pub type InitialLetter = GenericInitialLetter<Number, Integer>;

/// A specified value for the `letter-spacing` property.
pub type LetterSpacing = Spacing<Length>;

/// A specified value for the `word-spacing` property.
pub type WordSpacing = Spacing<LengthOrPercentage>;

/// A specified value for the `line-height` property.
pub type LineHeight = GenericLineHeight<NonNegativeNumber, NonNegativeLengthOrPercentage>;

impl Parse for InitialLetter {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("normal")).is_ok() {
            return Ok(GenericInitialLetter::Normal);
        }
        let size = Number::parse_at_least_one(context, input)?;
        let sink = input.try(|i| Integer::parse_positive(context, i)).ok();
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
            LengthOrPercentage::parse_quirky(c, i, AllowQuirks::Yes)
        })
    }
}

impl Parse for LineHeight {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(number) = input.try(|i| NonNegativeNumber::parse(context, i)) {
            return Ok(GenericLineHeight::Number(number));
        }
        if let Ok(nlop) = input.try(|i| NonNegativeLengthOrPercentage::parse(context, i)) {
            return Ok(GenericLineHeight::Length(nlop));
        }
        let location = input.current_source_location();
        let ident = input.expect_ident()?;
        match ident {
            ref ident if ident.eq_ignore_ascii_case("normal") => Ok(GenericLineHeight::Normal),
            #[cfg(feature = "gecko")]
            ref ident if ident.eq_ignore_ascii_case("-moz-block-height") =>
            {
                Ok(GenericLineHeight::MozBlockHeight)
            },
            ident => Err(location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone()))),
        }
    }
}

impl ToComputedValue for LineHeight {
    type ComputedValue = ComputedLineHeight;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        use values::computed::Length as ComputedLength;
        use values::specified::length::FontBaseSize;
        match *self {
            GenericLineHeight::Normal => GenericLineHeight::Normal,
            #[cfg(feature = "gecko")]
            GenericLineHeight::MozBlockHeight => GenericLineHeight::MozBlockHeight,
            GenericLineHeight::Number(number) => {
                GenericLineHeight::Number(number.to_computed_value(context))
            },
            GenericLineHeight::Length(ref non_negative_lop) => {
                let result = match non_negative_lop.0 {
                    LengthOrPercentage::Length(NoCalcLength::Absolute(ref abs)) => {
                        context
                            .maybe_zoom_text(abs.to_computed_value(context).into())
                            .0
                    },
                    LengthOrPercentage::Length(ref length) => length.to_computed_value(context),
                    LengthOrPercentage::Percentage(ref p) => FontRelativeLength::Em(p.0)
                        .to_computed_value(context, FontBaseSize::CurrentStyle),
                    LengthOrPercentage::Calc(ref calc) => {
                        let computed_calc =
                            calc.to_computed_value_zoomed(context, FontBaseSize::CurrentStyle);
                        let font_relative_length =
                            FontRelativeLength::Em(computed_calc.percentage())
                                .to_computed_value(context, FontBaseSize::CurrentStyle)
                                .px();

                        let absolute_length = computed_calc.unclamped_length().px();
                        let pixel = computed_calc
                            .clamping_mode
                            .clamp(absolute_length + font_relative_length);
                        ComputedLength::new(pixel)
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
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss)]
pub enum TextOverflowSide {
    /// Clip inline content.
    Clip,
    /// Render ellipsis to represent clipped inline content.
    Ellipsis,
    /// Render a given string to represent clipped inline content.
    String(Box<str>),
}

impl Parse for TextOverflowSide {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<TextOverflowSide, ParseError<'i>> {
        let location = input.current_source_location();
        match *input.next()? {
            Token::Ident(ref ident) => {
                match_ignore_ascii_case! { ident,
                    "clip" => Ok(TextOverflowSide::Clip),
                    "ellipsis" => Ok(TextOverflowSide::Ellipsis),
                    _ => Err(location.new_custom_error(
                        SelectorParseErrorKind::UnexpectedIdent(ident.clone())
                    ))
                }
            },
            Token::QuotedString(ref v) => Ok(TextOverflowSide::String(
                v.as_ref().to_owned().into_boxed_str(),
            )),
            ref t => Err(location.new_unexpected_token_error(t.clone())),
        }
    }
}

#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss)]
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
            .try(|input| TextOverflowSide::parse(context, input))
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

macro_rules! impl_text_decoration_line {
    {
        $(
            $(#[$($meta:tt)+])*
            $ident:ident / $css:expr => $value:expr,
        )+
    } => {
        bitflags! {
            #[derive(MallocSizeOf, ToComputedValue)]
            /// Specified keyword values for the text-decoration-line property.
            pub struct TextDecorationLine: u8 {
                /// No text decoration line is specified
                const NONE = 0;
                $(
                    $(#[$($meta)+])*
                    const $ident = $value;
                )+
                #[cfg(feature = "gecko")]
                /// Only set by presentation attributes
                ///
                /// Setting this will mean that text-decorations use the color
                /// specified by `color` in quirks mode.
                ///
                /// For example, this gives <a href=foo><font color="red">text</font></a>
                /// a red text decoration
                const COLOR_OVERRIDE = 0x10;
            }
        }

        impl Parse for TextDecorationLine {
            /// none | [ underline || overline || line-through || blink ]
            fn parse<'i, 't>(
                _context: &ParserContext,
                input: &mut Parser<'i, 't>,
            ) -> Result<TextDecorationLine, ParseError<'i>> {
                let mut result = TextDecorationLine::NONE;
                if input
                    .try(|input| input.expect_ident_matching("none"))
                    .is_ok()
                {
                    return Ok(result);
                }

                loop {
                    let result = input.try(|input| {
                        let ident = input.expect_ident().map_err(|_| ())?;
                        match_ignore_ascii_case! { ident,
                            $(
                                $css => {
                                    if result.contains(TextDecorationLine::$ident) {
                                        Err(())
                                    } else {
                                        result.insert(TextDecorationLine::$ident);
                                        Ok(())
                                    }
                                }
                            )+
                            _ => Err(()),
                        }
                    });
                    if result.is_err() {
                        break;
                    }
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

                let mut writer = SequenceWriter::new(dest, " ");
                $(
                    if self.contains(TextDecorationLine::$ident) {
                        writer.raw_item($css)?;
                    }
                )+
                Ok(())
            }
        }

        impl SpecifiedValueInfo for TextDecorationLine {
            fn collect_completion_keywords(f: KeywordsCollectFn) {
                f(&["none", $($css,)+]);
            }
        }
    }
}

impl_text_decoration_line! {
    /// Underline
    UNDERLINE / "underline" => 1 << 0,
    /// Overline
    OVERLINE / "overline" => 1 << 1,
    /// Line through
    LINE_THROUGH / "line-through" => 1 << 2,
    /// Blink
    BLINK / "blink" => 1 << 3,
}

#[cfg(feature = "gecko")]
impl_bitflags_conversions!(TextDecorationLine);

impl TextDecorationLine {
    #[inline]
    /// Returns the initial value of text-decoration-line
    pub fn none() -> Self {
        TextDecorationLine::NONE
    }
}

macro_rules! define_text_align_keyword {
    ($(
        $(#[$($meta:tt)+])*
        $name: ident => $discriminant: expr,
    )+) => {
        /// Specified value of text-align keyword value.
        #[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Parse, PartialEq,
                 SpecifiedValueInfo, ToComputedValue, ToCss)]
        #[allow(missing_docs)]
        pub enum TextAlignKeyword {
            $(
                $(#[$($meta)+])*
                $name = $discriminant,
            )+
        }

        impl TextAlignKeyword {
            /// Construct a TextAlignKeyword from u32.
            pub fn from_u32(discriminant: u32) -> Option<TextAlignKeyword> {
                match discriminant {
                    $(
                        $discriminant => Some(TextAlignKeyword::$name),
                    )+
                    _ => None
                }
            }
        }
    }
}

// FIXME(emilio): Why reinventing the world?
#[cfg(feature = "gecko")]
define_text_align_keyword! {
    Start => 0,
    End => 1,
    Left => 2,
    Right => 3,
    Center => 4,
    Justify => 5,
    MozCenter => 6,
    MozLeft => 7,
    MozRight => 8,
    #[css(skip)]
    Char => 10,
}

#[cfg(feature = "servo")]
define_text_align_keyword! {
    Start => 0,
    End => 1,
    Left => 2,
    Right => 3,
    Center => 4,
    Justify => 5,
    ServoCenter => 6,
    ServoLeft => 7,
    ServoRight => 8,
}

impl TextAlignKeyword {
    /// Return the initial value of TextAlignKeyword.
    #[inline]
    pub fn start() -> TextAlignKeyword {
        TextAlignKeyword::Start
    }
}

/// Specified value of text-align property.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SpecifiedValueInfo, ToCss)]
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

impl Parse for TextAlign {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // MozCenterOrInherit cannot be parsed, only set directly on the elements
        if let Ok(key) = input.try(TextAlignKeyword::parse) {
            return Ok(TextAlign::Keyword(key));
        }
        #[cfg(feature = "gecko")]
        {
            input.expect_ident_matching("match-parent")?;
            return Ok(TextAlign::MatchParent);
        }
        #[cfg(feature = "servo")]
        {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
    }
}

impl TextAlign {
    /// Convert an enumerated value coming from Gecko to a `TextAlign`.
    #[cfg(feature = "gecko")]
    pub fn from_gecko_keyword(kw: u32) -> Self {
        use gecko_bindings::structs::NS_STYLE_TEXT_ALIGN_MATCH_PARENT;
        if kw == NS_STYLE_TEXT_ALIGN_MATCH_PARENT {
            TextAlign::MatchParent
        } else {
            TextAlign::Keyword(TextAlignKeyword::from_gecko_keyword(kw))
        }
    }
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
                if _context.is_root_element {
                    return TextAlignKeyword::start();
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

/// Specified value of text-emphasis-style property.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss)]
pub enum TextEmphasisStyle {
    /// <fill> <shape>
    Keyword(TextEmphasisKeywordValue),
    /// `none`
    None,
    /// String (will be used only first grapheme cluster) for the text-emphasis-style property
    String(String),
}

/// Keyword value for the text-emphasis-style property
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss)]
pub enum TextEmphasisKeywordValue {
    /// <fill>
    Fill(TextEmphasisFillMode),
    /// <shape>
    Shape(TextEmphasisShapeKeyword),
    /// <fill> <shape>
    FillAndShape(TextEmphasisFillMode, TextEmphasisShapeKeyword),
}

impl TextEmphasisKeywordValue {
    fn fill(&self) -> Option<TextEmphasisFillMode> {
        match *self {
            TextEmphasisKeywordValue::Fill(fill) |
            TextEmphasisKeywordValue::FillAndShape(fill, _) => Some(fill),
            _ => None,
        }
    }

    fn shape(&self) -> Option<TextEmphasisShapeKeyword> {
        match *self {
            TextEmphasisKeywordValue::Shape(shape) |
            TextEmphasisKeywordValue::FillAndShape(_, shape) => Some(shape),
            _ => None,
        }
    }
}

/// Fill mode for the text-emphasis-style property
#[derive(Clone, Copy, Debug, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo,
         ToCss)]
pub enum TextEmphasisFillMode {
    /// `filled`
    Filled,
    /// `open`
    Open,
}

/// Shape keyword for the text-emphasis-style property
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq,
         SpecifiedValueInfo, ToCss)]
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

impl TextEmphasisShapeKeyword {
    /// converts fill mode to a unicode char
    pub fn char(&self, fill: TextEmphasisFillMode) -> &str {
        let fill = fill == TextEmphasisFillMode::Filled;
        match *self {
            TextEmphasisShapeKeyword::Dot => if fill {
                "\u{2022}"
            } else {
                "\u{25e6}"
            },
            TextEmphasisShapeKeyword::Circle => if fill {
                "\u{25cf}"
            } else {
                "\u{25cb}"
            },
            TextEmphasisShapeKeyword::DoubleCircle => if fill {
                "\u{25c9}"
            } else {
                "\u{25ce}"
            },
            TextEmphasisShapeKeyword::Triangle => if fill {
                "\u{25b2}"
            } else {
                "\u{25b3}"
            },
            TextEmphasisShapeKeyword::Sesame => if fill {
                "\u{fe45}"
            } else {
                "\u{fe46}"
            },
        }
    }
}

impl ToComputedValue for TextEmphasisStyle {
    type ComputedValue = ComputedTextEmphasisStyle;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            TextEmphasisStyle::Keyword(ref keyword) => {
                let default_shape = if context.style().get_inherited_box().clone_writing_mode() ==
                    SpecifiedWritingMode::HorizontalTb
                {
                    TextEmphasisShapeKeyword::Circle
                } else {
                    TextEmphasisShapeKeyword::Sesame
                };
                ComputedTextEmphasisStyle::Keyword(ComputedTextEmphasisKeywordValue {
                    fill: keyword.fill().unwrap_or(TextEmphasisFillMode::Filled),
                    shape: keyword.shape().unwrap_or(default_shape),
                })
            },
            TextEmphasisStyle::None => ComputedTextEmphasisStyle::None,
            TextEmphasisStyle::String(ref s) => {
                // Passing `true` to iterate over extended grapheme clusters, following
                // recommendation at http://www.unicode.org/reports/tr29/#Grapheme_Cluster_Boundaries
                let string = s.graphemes(true).next().unwrap_or("").to_string();
                ComputedTextEmphasisStyle::String(string)
            },
        }
    }
    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            ComputedTextEmphasisStyle::Keyword(ref keyword) => TextEmphasisStyle::Keyword(
                TextEmphasisKeywordValue::FillAndShape(keyword.fill, keyword.shape),
            ),
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
            .try(|input| input.expect_ident_matching("none"))
            .is_ok()
        {
            return Ok(TextEmphasisStyle::None);
        }

        if let Ok(s) = input.try(|i| i.expect_string().map(|s| s.as_ref().to_owned())) {
            // Handle <string>
            return Ok(TextEmphasisStyle::String(s));
        }

        // Handle a pair of keywords
        let mut shape = input.try(TextEmphasisShapeKeyword::parse).ok();
        let fill = input.try(TextEmphasisFillMode::parse).ok();
        if shape.is_none() {
            shape = input.try(TextEmphasisShapeKeyword::parse).ok();
        }

        // At least one of shape or fill must be handled
        let keyword_value = match (fill, shape) {
            (Some(fill), Some(shape)) => TextEmphasisKeywordValue::FillAndShape(fill, shape),
            (Some(fill), None) => TextEmphasisKeywordValue::Fill(fill),
            (None, Some(shape)) => TextEmphasisKeywordValue::Shape(shape),
            _ => return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
        };
        Ok(TextEmphasisStyle::Keyword(keyword_value))
    }
}

/// The allowed horizontal values for the `text-emphasis-position` property.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq,
         SpecifiedValueInfo, ToComputedValue, ToCss)]
pub enum TextEmphasisHorizontalWritingModeValue {
    /// Draw marks over the text in horizontal writing mode.
    Over,
    /// Draw marks under the text in horizontal writing mode.
    Under,
}

/// The allowed vertical values for the `text-emphasis-position` property.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq,
         SpecifiedValueInfo, ToComputedValue, ToCss)]
pub enum TextEmphasisVerticalWritingModeValue {
    /// Draws marks to the right of the text in vertical writing mode.
    Right,
    /// Draw marks to the left of the text in vertical writing mode.
    Left,
}

/// Specified value of `text-emphasis-position` property.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue, ToCss)]
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
        use gecko_bindings::structs;

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
            input.try(|input| TextEmphasisHorizontalWritingModeValue::parse(input))
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
        use gecko_bindings::structs;

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

/// A specified value for the `-moz-tab-size` property.
pub type MozTabSize = GenericMozTabSize<NonNegativeNumber, NonNegativeLength>;

impl Parse for MozTabSize {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(number) = input.try(|i| NonNegativeNumber::parse(context, i)) {
            // Numbers need to be parsed first because `0` must be recognised
            // as the number `0` and not the length `0px`.
            return Ok(GenericMozTabSize::Number(number));
        }
        Ok(GenericMozTabSize::Length(NonNegativeLength::parse(
            context,
            input,
        )?))
    }
}
