/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for text properties.

use cssparser::{Parser, Token};
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseErrorKind;
#[allow(unused_imports)] use std::ascii::AsciiExt;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};
use values::computed::{Context, ToComputedValue};
use values::computed::text::LineHeight as ComputedLineHeight;
use values::computed::text::TextOverflow as ComputedTextOverflow;
use values::generics::text::InitialLetter as GenericInitialLetter;
use values::generics::text::LineHeight as GenericLineHeight;
use values::generics::text::Spacing;
use values::specified::{AllowQuirks, Integer, NonNegativeNumber, Number};
use values::specified::length::{FontRelativeLength, Length, LengthOrPercentage, NoCalcLength};
use values::specified::length::NonNegativeLengthOrPercentage;

/// A specified type for the `initial-letter` property.
pub type InitialLetter = GenericInitialLetter<Number, Integer>;

/// A specified value for the `letter-spacing` property.
pub type LetterSpacing = Spacing<Length>;

/// A specified value for the `word-spacing` property.
pub type WordSpacing = Spacing<LengthOrPercentage>;

/// A specified value for the `line-height` property.
pub type LineHeight = GenericLineHeight<NonNegativeNumber, NonNegativeLengthOrPercentage>;

impl Parse for InitialLetter {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("normal")).is_ok() {
            return Ok(GenericInitialLetter::Normal);
        }
        let size = Number::parse_at_least_one(context, input)?;
        let sink = input.try(|i| Integer::parse_positive(context, i)).ok();
        Ok(GenericInitialLetter::Specified(size, sink))
    }
}

impl Parse for LetterSpacing {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Spacing::parse_with(context, input, |c, i| {
            Length::parse_quirky(c, i, AllowQuirks::Yes)
        })
    }
}

impl Parse for WordSpacing {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Spacing::parse_with(context, input, |c, i| {
            LengthOrPercentage::parse_quirky(c, i, AllowQuirks::Yes)
        })
    }
}

impl Parse for LineHeight {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(number) = input.try(|i| NonNegativeNumber::parse(context, i)) {
            return Ok(GenericLineHeight::Number(number))
        }
        if let Ok(nlop) = input.try(|i| NonNegativeLengthOrPercentage::parse(context, i)) {
            return Ok(GenericLineHeight::Length(nlop))
        }
        let location = input.current_source_location();
        let ident = input.expect_ident()?;
        match ident {
            ref ident if ident.eq_ignore_ascii_case("normal") => {
                Ok(GenericLineHeight::Normal)
            },
            #[cfg(feature = "gecko")]
            ref ident if ident.eq_ignore_ascii_case("-moz-block-height") => {
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
            GenericLineHeight::Normal => {
                GenericLineHeight::Normal
            },
            #[cfg(feature = "gecko")]
            GenericLineHeight::MozBlockHeight => {
                GenericLineHeight::MozBlockHeight
            },
            GenericLineHeight::Number(number) => {
                GenericLineHeight::Number(number.to_computed_value(context))
            },
            GenericLineHeight::Length(ref non_negative_lop) => {
                let result = match non_negative_lop.0 {
                    LengthOrPercentage::Length(NoCalcLength::Absolute(ref abs)) => {
                        context.maybe_zoom_text(abs.to_computed_value(context).into()).0
                    }
                    LengthOrPercentage::Length(ref length) => {
                        length.to_computed_value(context)
                    },
                    LengthOrPercentage::Percentage(ref p) => {
                        FontRelativeLength::Em(p.0)
                            .to_computed_value(
                                context,
                                FontBaseSize::CurrentStyle,
                            )
                    }
                    LengthOrPercentage::Calc(ref calc) => {
                        let computed_calc =
                            calc.to_computed_value_zoomed(context, FontBaseSize::CurrentStyle);
                        let font_relative_length =
                            FontRelativeLength::Em(computed_calc.percentage())
                                .to_computed_value(
                                    context,
                                    FontBaseSize::CurrentStyle,
                                ).px();

                        let absolute_length = computed_calc.unclamped_length().px();
                        let pixel = computed_calc
                            .clamping_mode
                            .clamp(absolute_length + font_relative_length);
                        ComputedLength::new(pixel)
                    }
                };
                GenericLineHeight::Length(result.into())
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            GenericLineHeight::Normal => {
                GenericLineHeight::Normal
            },
            #[cfg(feature = "gecko")]
            GenericLineHeight::MozBlockHeight => {
                GenericLineHeight::MozBlockHeight
            },
            GenericLineHeight::Number(ref number) => {
                GenericLineHeight::Number(NonNegativeNumber::from_computed_value(number))
            },
            GenericLineHeight::Length(ref length) => {
                GenericLineHeight::Length(NoCalcLength::from_computed_value(&length.0).into())
            }
        }
    }
}

/// A generic value for the `text-overflow` property.
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, ToCss)]
pub enum TextOverflowSide {
    /// Clip inline content.
    Clip,
    /// Render ellipsis to represent clipped inline content.
    Ellipsis,
    /// Render a given string to represent clipped inline content.
    String(Box<str>),
}

impl Parse for TextOverflowSide {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                        -> Result<TextOverflowSide, ParseError<'i>> {
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
            }
            Token::QuotedString(ref v) => {
                Ok(TextOverflowSide::String(v.as_ref().to_owned().into_boxed_str()))
            }
            ref t => Err(location.new_unexpected_token_error(t.clone())),
        }
    }
}

#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, ToCss)]
/// text-overflow. Specifies rendering when inline content overflows its line box edge.
pub struct TextOverflow {
    /// First value. Applies to end line box edge if no second is supplied; line-left edge otherwise.
    pub first: TextOverflowSide,
    /// Second value. Applies to the line-right edge if supplied.
    pub second: Option<TextOverflowSide>,
}

impl Parse for TextOverflow {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<TextOverflow, ParseError<'i>> {
        let first = TextOverflowSide::parse(context, input)?;
        let second = input.try(|input| TextOverflowSide::parse(context, input)).ok();
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
            assert!(computed.first == TextOverflowSide::Clip);
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
    #[derive(MallocSizeOf, ToComputedValue)]
    /// Specified keyword values for the text-decoration-line property.
    pub struct TextDecorationLine: u8 {
        /// No text decoration line is specified
        const NONE = 0;
        /// Underline
        const UNDERLINE = 0x01;
        /// Overline
        const OVERLINE = 0x02;
        /// Line through
        const LINE_THROUGH = 0x04;
        /// Blink
        const BLINK = 0x08;
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

#[cfg(feature = "gecko")]
impl_bitflags_conversions!(TextDecorationLine);

impl TextDecorationLine {
    #[inline]
    /// Returns the initial value of text-decoration-line
    pub fn none() -> Self {
        TextDecorationLine::NONE
    }
}

impl Parse for TextDecorationLine {
    /// none | [ underline || overline || line-through || blink ]
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<TextDecorationLine, ParseError<'i>> {
        let mut result = TextDecorationLine::NONE;
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(result)
        }

        loop {
            let result: Result<_, ParseError> = input.try(|input| {
                try_match_ident_ignore_ascii_case! { input,
                    "underline" => {
                        if result.contains(TextDecorationLine::UNDERLINE) {
                            Err(())
                        } else {
                            result.insert(TextDecorationLine::UNDERLINE);
                            Ok(())
                        }
                    }
                    "overline" => {
                        if result.contains(TextDecorationLine::OVERLINE) {
                            Err(())
                        } else {
                            result.insert(TextDecorationLine::OVERLINE);
                            Ok(())
                        }
                    }
                    "line-through" => {
                        if result.contains(TextDecorationLine::LINE_THROUGH) {
                            Err(())
                        } else {
                            result.insert(TextDecorationLine::LINE_THROUGH);
                            Ok(())
                        }
                    }
                    "blink" => {
                        if result.contains(TextDecorationLine::BLINK) {
                            Err(())
                        } else {
                            result.insert(TextDecorationLine::BLINK);
                            Ok(())
                        }
                    }
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

macro_rules! define_text_align_keyword {
    ($($name: ident => $discriminant: expr,)+) => {
        /// Specified value of text-align keyword value.
        #[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Parse, PartialEq, ToComputedValue, ToCss)]
        #[allow(missing_docs)]
        pub enum TextAlignKeyword {
            $(
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
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
    MozCenterOrInherit,

}

impl Parse for TextAlign {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
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

impl ToCss for TextAlign {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            TextAlign::Keyword(key) => key.to_css(dest),
            #[cfg(feature = "gecko")]
            TextAlign::MatchParent => dest.write_str("match-parent"),
            #[cfg(feature = "gecko")]
            TextAlign::MozCenterOrInherit => Ok(()),
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
                let parent = _context.builder.get_parent_inheritedtext().clone_text_align();
                let ltr = _context.builder.inherited_writing_mode().is_bidi_ltr();
                match (parent, ltr) {
                    (TextAlignKeyword::Start, true) => TextAlignKeyword::Left,
                    (TextAlignKeyword::Start, false) => TextAlignKeyword::Right,
                    (TextAlignKeyword::End, true) => TextAlignKeyword::Right,
                    (TextAlignKeyword::End, false) => TextAlignKeyword::Left,
                    _ => parent
                }
            },
            #[cfg(feature = "gecko")]
            TextAlign::MozCenterOrInherit => {
                let parent = _context.builder.get_parent_inheritedtext().clone_text_align();
                if parent == TextAlignKeyword::Start {
                    TextAlignKeyword::Center
                } else {
                    parent
                }
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        TextAlign::Keyword(*computed)
    }
}
