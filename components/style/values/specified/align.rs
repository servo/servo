/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Values for CSS Box Alignment properties
//!
//! https://drafts.csswg.org/css-align/

use cssparser::Parser;
use gecko_bindings::structs;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

bitflags! {
    /// Constants shared by multiple CSS Box Alignment properties
    ///
    /// These constants match Gecko's `NS_STYLE_ALIGN_*` constants.
    #[derive(MallocSizeOf, ToComputedValue)]
    pub struct AlignFlags: u8 {
        // Enumeration stored in the lower 5 bits:
        /// 'auto'
        const AUTO =            structs::NS_STYLE_ALIGN_AUTO as u8;
        /// 'normal'
        const NORMAL =          structs::NS_STYLE_ALIGN_NORMAL as u8;
        /// 'start'
        const START =           structs::NS_STYLE_ALIGN_START as u8;
        /// 'end'
        const END =             structs::NS_STYLE_ALIGN_END as u8;
        /// 'flex-start'
        const FLEX_START =      structs::NS_STYLE_ALIGN_FLEX_START as u8;
        /// 'flex-end'
        const FLEX_END =        structs::NS_STYLE_ALIGN_FLEX_END as u8;
        /// 'center'
        const CENTER =          structs::NS_STYLE_ALIGN_CENTER as u8;
        /// 'left'
        const LEFT =            structs::NS_STYLE_ALIGN_LEFT as u8;
        /// 'right'
        const RIGHT =           structs::NS_STYLE_ALIGN_RIGHT as u8;
        /// 'baseline'
        const BASELINE =        structs::NS_STYLE_ALIGN_BASELINE as u8;
        /// 'last-baseline'
        const LAST_BASELINE =   structs::NS_STYLE_ALIGN_LAST_BASELINE as u8;
        /// 'stretch'
        const STRETCH =         structs::NS_STYLE_ALIGN_STRETCH as u8;
        /// 'self-start'
        const SELF_START =      structs::NS_STYLE_ALIGN_SELF_START as u8;
        /// 'self-end'
        const SELF_END =        structs::NS_STYLE_ALIGN_SELF_END as u8;
        /// 'space-between'
        const SPACE_BETWEEN =   structs::NS_STYLE_ALIGN_SPACE_BETWEEN as u8;
        /// 'space-around'
        const SPACE_AROUND =    structs::NS_STYLE_ALIGN_SPACE_AROUND as u8;
        /// 'space-evenly'
        const SPACE_EVENLY =    structs::NS_STYLE_ALIGN_SPACE_EVENLY as u8;

        // Additional flags stored in the upper bits:
        /// 'legacy' (mutually exclusive w. SAFE & UNSAFE)
        const LEGACY =          structs::NS_STYLE_ALIGN_LEGACY as u8;
        /// 'safe'
        const SAFE =            structs::NS_STYLE_ALIGN_SAFE as u8;
        /// 'unsafe' (mutually exclusive w. SAFE)
        const UNSAFE =          structs::NS_STYLE_ALIGN_UNSAFE as u8;

        /// Mask for the additional flags above.
        const FLAG_BITS =       structs::NS_STYLE_ALIGN_FLAG_BITS as u8;
    }
}

impl AlignFlags {
    /// Returns the enumeration value stored in the lower 5 bits.
    #[inline]
    fn value(&self) -> Self {
        *self & !AlignFlags::FLAG_BITS
    }
}

impl ToCss for AlignFlags {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self & AlignFlags::FLAG_BITS {
            AlignFlags::LEGACY => dest.write_str("legacy ")?,
            AlignFlags::SAFE => dest.write_str("safe ")?,
            // Don't serialize "unsafe", since it's the default.
            _ => {}
        }

        dest.write_str(match self.value() {
            AlignFlags::AUTO => "auto",
            AlignFlags::NORMAL => "normal",
            AlignFlags::START => "start",
            AlignFlags::END => "end",
            AlignFlags::FLEX_START => "flex-start",
            AlignFlags::FLEX_END => "flex-end",
            AlignFlags::CENTER => "center",
            AlignFlags::LEFT => "left",
            AlignFlags::RIGHT => "right",
            AlignFlags::BASELINE => "baseline",
            AlignFlags::LAST_BASELINE => "last baseline",
            AlignFlags::STRETCH => "stretch",
            AlignFlags::SELF_START => "self-start",
            AlignFlags::SELF_END => "self-end",
            AlignFlags::SPACE_BETWEEN => "space-between",
            AlignFlags::SPACE_AROUND => "space-around",
            AlignFlags::SPACE_EVENLY => "space-evenly",
            _ => unreachable!()
        })
    }
}

/// An axis direction, either inline (for the `justify` properties) or block,
/// (for the `align` properties).
#[derive(Clone, Copy, PartialEq)]
pub enum AxisDirection {
    /// Block direction.
    Block,
    /// Inline direction.
    Inline,
}

/// Shared value for the `align-content` and `justify-content` properties.
///
/// <https://drafts.csswg.org/css-align/#content-distribution>
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub struct ContentDistribution {
    primary: AlignFlags,
    // FIXME(https://github.com/w3c/csswg-drafts/issues/1002): This will need to
    // accept fallback alignment, eventually.
}

impl ContentDistribution {
    /// The initial value 'normal'
    #[inline]
    pub fn normal() -> Self {
        Self::new(AlignFlags::NORMAL)
    }

    /// The initial value 'normal'
    #[inline]
    pub fn new(primary: AlignFlags) -> Self {
        Self { primary }
    }

    fn from_bits(bits: u16) -> Self {
        Self {
            primary: AlignFlags::from_bits_truncate(bits as u8)
        }
    }

    fn as_bits(&self) -> u16 {
        self.primary.bits() as u16
    }

    /// Returns whether this value is valid for both axis directions.
    pub fn is_valid_on_both_axes(&self) -> bool {
        match self.primary.value() {
            // <baseline-position> is only allowed on the block axis.
            AlignFlags::BASELINE |
            AlignFlags::LAST_BASELINE => false,

            // left | right are only allowed on the inline axis.
            AlignFlags::LEFT |
            AlignFlags::RIGHT => false,

            _ => true,
        }
    }

    /// The primary alignment
    #[inline]
    pub fn primary(self) -> AlignFlags {
        self.primary
    }

    /// Whether this value has extra flags.
    #[inline]
    pub fn has_extra_flags(self) -> bool {
        self.primary().intersects(AlignFlags::FLAG_BITS)
    }

    /// Parse a value for align-content / justify-content.
    pub fn parse<'i, 't>(
        input: &mut Parser<'i, 't>,
        axis: AxisDirection,
    ) -> Result<Self, ParseError<'i>> {
        // Try to parse normal first
        if input.try(|i| i.expect_ident_matching("normal")).is_ok() {
            return Ok(ContentDistribution::normal());
        }

        // Parse <baseline-position>, but only on the block axis.
        if axis == AxisDirection::Block {
            if let Ok(value) = input.try(parse_baseline) {
                return Ok(ContentDistribution::new(value));
            }
        }

        // <content-distribution>
        if let Ok(value) = input.try(parse_content_distribution) {
            return Ok(ContentDistribution::new(value));
        }

        // <overflow-position>? <content-position>
        let overflow_position =
            input.try(parse_overflow_position)
            .unwrap_or(AlignFlags::empty());

        let content_position = try_match_ident_ignore_ascii_case! { input,
            "start" => AlignFlags::START,
            "end" => AlignFlags::END,
            "flex-start" => AlignFlags::FLEX_START,
            "flex-end" => AlignFlags::FLEX_END,
            "center" => AlignFlags::CENTER,
            "left" if axis == AxisDirection::Inline => AlignFlags::LEFT,
            "right" if axis == AxisDirection::Inline => AlignFlags::RIGHT,
        };

        Ok(ContentDistribution::new(content_position | overflow_position))
    }
}

/// Value for the `align-content` property.
///
/// <https://drafts.csswg.org/css-align/#propdef-align-content>
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub struct AlignContent(pub ContentDistribution);

impl Parse for AlignContent {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Ok(AlignContent(ContentDistribution::parse(
            input,
            AxisDirection::Block,
        )?))
    }
}

#[cfg(feature = "gecko")]
impl From<u16> for AlignContent {
    fn from(bits: u16) -> Self {
        AlignContent(ContentDistribution::from_bits(bits))
    }
}

#[cfg(feature = "gecko")]
impl From<AlignContent> for u16 {
    fn from(v: AlignContent) -> u16 {
        v.0.as_bits()
    }
}

/// Value for the `justify-content` property.
///
/// <https://drafts.csswg.org/css-align/#propdef-align-content>
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub struct JustifyContent(pub ContentDistribution);

impl Parse for JustifyContent {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Ok(JustifyContent(ContentDistribution::parse(
            input,
            AxisDirection::Inline,
        )?))
    }
}

#[cfg(feature = "gecko")]
impl From<u16> for JustifyContent {
    fn from(bits: u16) -> Self {
        JustifyContent(ContentDistribution::from_bits(bits))
    }
}

#[cfg(feature = "gecko")]
impl From<JustifyContent> for u16 {
    fn from(v: JustifyContent) -> u16 {
        v.0.as_bits()
    }
}

/// <https://drafts.csswg.org/css-align/#self-alignment>
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub struct SelfAlignment(pub AlignFlags);

impl SelfAlignment {
    /// The initial value 'auto'
    #[inline]
    pub fn auto() -> Self {
        SelfAlignment(AlignFlags::AUTO)
    }

    /// Returns whether this value is valid for both axis directions.
    pub fn is_valid_on_both_axes(&self) -> bool {
        match self.0.value() {
            // left | right are only allowed on the inline axis.
            AlignFlags::LEFT |
            AlignFlags::RIGHT => false,

            _ => true,
        }
    }

    /// Whether this value has extra flags.
    #[inline]
    pub fn has_extra_flags(self) -> bool {
        self.0.intersects(AlignFlags::FLAG_BITS)
    }

    /// Parse a self-alignment value on one of the axis.
    pub fn parse<'i, 't>(
        input: &mut Parser<'i, 't>,
        axis: AxisDirection,
    ) -> Result<Self, ParseError<'i>> {
        // <baseline-position>
        //
        // It's weird that this accepts <baseline-position>, but not
        // justify-content...
        if let Ok(value) = input.try(parse_baseline) {
            return Ok(SelfAlignment(value));
        }

        // auto | normal | stretch
        if let Ok(value) = input.try(parse_auto_normal_stretch) {
            return Ok(SelfAlignment(value))
        }

        // <overflow-position>? <self-position>
        let overflow_position =
            input.try(parse_overflow_position)
            .unwrap_or(AlignFlags::empty());
        let self_position = parse_self_position(input, axis)?;
        Ok(SelfAlignment(overflow_position | self_position))
    }
}

/// The specified value of the align-self property.
///
/// <https://drafts.csswg.org/css-align/#propdef-align-self>
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub struct AlignSelf(pub SelfAlignment);

impl Parse for AlignSelf {
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Ok(AlignSelf(SelfAlignment::parse(input, AxisDirection::Block)?))
    }
}

impl From<u8> for AlignSelf {
    fn from(bits: u8) -> Self {
        AlignSelf(SelfAlignment(AlignFlags::from_bits_truncate(bits)))
    }
}

impl From<AlignSelf> for u8 {
    fn from(align: AlignSelf) -> u8 {
        (align.0).0.bits()
    }
}

/// The specified value of the justify-self property.
///
/// <https://drafts.csswg.org/css-align/#propdef-justify-self>
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub struct JustifySelf(pub SelfAlignment);

impl Parse for JustifySelf {
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Ok(JustifySelf(SelfAlignment::parse(input, AxisDirection::Inline)?))
    }
}

impl From<u8> for JustifySelf {
    fn from(bits: u8) -> Self {
        JustifySelf(SelfAlignment(AlignFlags::from_bits_truncate(bits)))
    }
}

impl From<JustifySelf> for u8 {
    fn from(justify: JustifySelf) -> u8 {
        (justify.0).0.bits()
    }
}

/// Value of the `align-items` property
///
/// <https://drafts.csswg.org/css-align/#self-alignment>
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub struct AlignItems(pub AlignFlags);

impl AlignItems {
    /// The initial value 'normal'
    #[inline]
    pub fn normal() -> Self {
        AlignItems(AlignFlags::NORMAL)
    }

    /// Whether this value has extra flags.
    #[inline]
    pub fn has_extra_flags(self) -> bool {
        self.0.intersects(AlignFlags::FLAG_BITS)
    }
}


impl Parse for AlignItems {
    // normal | stretch | <baseline-position> |
    // <overflow-position>? <self-position>
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        // <baseline-position>
        if let Ok(baseline) = input.try(parse_baseline) {
            return Ok(AlignItems(baseline));
        }

        // normal | stretch
        if let Ok(value) = input.try(parse_normal_stretch) {
            return Ok(AlignItems(value))
        }
        // <overflow-position>? <self-position>
        let overflow = input.try(parse_overflow_position).unwrap_or(AlignFlags::empty());
        let self_position = parse_self_position(input, AxisDirection::Block)?;
        Ok(AlignItems(self_position | overflow))
    }
}

/// Value of the `justify-items` property
///
/// <https://drafts.csswg.org/css-align/#justify-items-property>
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToCss)]
pub struct JustifyItems(pub AlignFlags);

impl JustifyItems {
    /// The initial value 'auto'
    #[inline]
    pub fn auto() -> Self {
        JustifyItems(AlignFlags::AUTO)
    }

    /// The value 'normal'
    #[inline]
    pub fn normal() -> Self {
        JustifyItems(AlignFlags::NORMAL)
    }

    /// Whether this value has extra flags.
    #[inline]
    pub fn has_extra_flags(self) -> bool {
        self.0.intersects(AlignFlags::FLAG_BITS)
    }
}

impl Parse for JustifyItems {
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        // <baseline-position>
        //
        // It's weird that this accepts <baseline-position>, but not
        // justify-content...
        if let Ok(baseline) = input.try(parse_baseline) {
            return Ok(JustifyItems(baseline));
        }

        // auto | normal | stretch
        //
        // FIXME(emilio): auto is no longer a keyword in the current spec, and
        // has been renamed to legacy, but that needs different changes because
        // right now it's the initial value for both style systems, and has that
        // weird behavior of "inheriting" into descendants.
        //
        // Fix this in both.
        //
        // See also:
        //   https://bugs.webkit.org/show_bug.cgi?id=172711
        //   https://bugs.chromium.org/p/chromium/issues/detail?id=726148
        //
        if let Ok(value) = input.try(parse_auto_normal_stretch) {
            return Ok(JustifyItems(value))
        }

        // [ legacy || [ left | right | center ] ]
        if let Ok(value) = input.try(parse_legacy) {
            return Ok(JustifyItems(value))
        }

        // <overflow-position>? <self-position>
        let overflow = input.try(parse_overflow_position).unwrap_or(AlignFlags::empty());
        let self_position = parse_self_position(input, AxisDirection::Inline)?;
        Ok(JustifyItems(overflow | self_position))
    }
}

// auto | normal | stretch
fn parse_auto_normal_stretch<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<AlignFlags, ParseError<'i>> {
    try_match_ident_ignore_ascii_case! { input,
        "auto" => Ok(AlignFlags::AUTO),
        "normal" => Ok(AlignFlags::NORMAL),
        "stretch" => Ok(AlignFlags::STRETCH),
    }
}

// normal | stretch
fn parse_normal_stretch<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    try_match_ident_ignore_ascii_case! { input,
        "normal" => Ok(AlignFlags::NORMAL),
        "stretch" => Ok(AlignFlags::STRETCH),
    }
}

// <baseline-position>
fn parse_baseline<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    try_match_ident_ignore_ascii_case! { input,
        "baseline" => Ok(AlignFlags::BASELINE),
        "first" => {
            input.expect_ident_matching("baseline")?;
            Ok(AlignFlags::BASELINE)
        }
        "last" => {
            input.expect_ident_matching("baseline")?;
            Ok(AlignFlags::LAST_BASELINE)
        }
    }
}

// <content-distribution>
fn parse_content_distribution<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    try_match_ident_ignore_ascii_case! { input,
        "stretch" => Ok(AlignFlags::STRETCH),
        "space-between" => Ok(AlignFlags::SPACE_BETWEEN),
        "space-around" => Ok(AlignFlags::SPACE_AROUND),
        "space-evenly" => Ok(AlignFlags::SPACE_EVENLY),
    }
}

// <overflow-position>
fn parse_overflow_position<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    try_match_ident_ignore_ascii_case! { input,
        "safe" => Ok(AlignFlags::SAFE),
        "unsafe" => Ok(AlignFlags::UNSAFE),
    }
}

// <self-position> | left | right in the inline axis.
fn parse_self_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    axis: AxisDirection,
) -> Result<AlignFlags, ParseError<'i>> {
    Ok(try_match_ident_ignore_ascii_case! { input,
        "start" => AlignFlags::START,
        "end" => AlignFlags::END,
        "flex-start" => AlignFlags::FLEX_START,
        "flex-end" => AlignFlags::FLEX_END,
        "center" => AlignFlags::CENTER,
        "self-start" => AlignFlags::SELF_START,
        "self-end" => AlignFlags::SELF_END,
        "left" if axis == AxisDirection::Inline => AlignFlags::LEFT,
        "right" if axis == AxisDirection::Inline => AlignFlags::RIGHT,
    })
}

// [ legacy && [ left | right | center ] ]
fn parse_legacy<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    let a_location = input.current_source_location();
    let a = input.expect_ident()?.clone();
    let b_location = input.current_source_location();
    let b = input.expect_ident()?;
    if a.eq_ignore_ascii_case("legacy") {
        (match_ignore_ascii_case! { &b,
            "left" => Ok(AlignFlags::LEGACY | AlignFlags::LEFT),
            "right" => Ok(AlignFlags::LEGACY | AlignFlags::RIGHT),
            "center" => Ok(AlignFlags::LEGACY | AlignFlags::CENTER),
            _ => Err(())
        }).map_err(|()| b_location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(b.clone())))
    } else if b.eq_ignore_ascii_case("legacy") {
        (match_ignore_ascii_case! { &a,
            "left" => Ok(AlignFlags::LEGACY | AlignFlags::LEFT),
            "right" => Ok(AlignFlags::LEGACY | AlignFlags::RIGHT),
            "center" => Ok(AlignFlags::LEGACY | AlignFlags::CENTER),
            _ => Err(())
        }).map_err(|()| a_location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(a)))
    } else {
        Err(a_location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}
