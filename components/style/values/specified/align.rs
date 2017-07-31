/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Values for CSS Box Alignment properties
//!
//! https://drafts.csswg.org/css-align/

use cssparser::Parser;
use gecko_bindings::structs;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseError;
use std::ascii::AsciiExt;
use std::fmt;
use style_traits::{ToCss, ParseError, StyleParseError};

bitflags! {
    /// Constants shared by multiple CSS Box Alignment properties
    ///
    /// These constants match Gecko's `NS_STYLE_ALIGN_*` constants.
    pub flags AlignFlags: u8 {
        // Enumeration stored in the lower 5 bits:
        /// 'auto'
        const ALIGN_AUTO =            structs::NS_STYLE_ALIGN_AUTO as u8,
        /// 'normal'
        const ALIGN_NORMAL =          structs::NS_STYLE_ALIGN_NORMAL as u8,
        /// 'start'
        const ALIGN_START =           structs::NS_STYLE_ALIGN_START as u8,
        /// 'end'
        const ALIGN_END =             structs::NS_STYLE_ALIGN_END as u8,
        /// 'flex-start'
        const ALIGN_FLEX_START =      structs::NS_STYLE_ALIGN_FLEX_START as u8,
        /// 'flex-end'
        const ALIGN_FLEX_END =        structs::NS_STYLE_ALIGN_FLEX_END as u8,
        /// 'center'
        const ALIGN_CENTER =          structs::NS_STYLE_ALIGN_CENTER as u8,
        /// 'left'
        const ALIGN_LEFT =            structs::NS_STYLE_ALIGN_LEFT as u8,
        /// 'right'
        const ALIGN_RIGHT =           structs::NS_STYLE_ALIGN_RIGHT as u8,
        /// 'baseline'
        const ALIGN_BASELINE =        structs::NS_STYLE_ALIGN_BASELINE as u8,
        /// 'last-baseline'
        const ALIGN_LAST_BASELINE =   structs::NS_STYLE_ALIGN_LAST_BASELINE as u8,
        /// 'stretch'
        const ALIGN_STRETCH =         structs::NS_STYLE_ALIGN_STRETCH as u8,
        /// 'self-start'
        const ALIGN_SELF_START =      structs::NS_STYLE_ALIGN_SELF_START as u8,
        /// 'self-end'
        const ALIGN_SELF_END =        structs::NS_STYLE_ALIGN_SELF_END as u8,
        /// 'space-between'
        const ALIGN_SPACE_BETWEEN =   structs::NS_STYLE_ALIGN_SPACE_BETWEEN as u8,
        /// 'space-around'
        const ALIGN_SPACE_AROUND =    structs::NS_STYLE_ALIGN_SPACE_AROUND as u8,
        /// 'space-evenly'
        const ALIGN_SPACE_EVENLY =    structs::NS_STYLE_ALIGN_SPACE_EVENLY as u8,

        // Additional flags stored in the upper bits:
        /// 'legacy' (mutually exclusive w. SAFE & UNSAFE)
        const ALIGN_LEGACY =          structs::NS_STYLE_ALIGN_LEGACY as u8,
        /// 'safe'
        const ALIGN_SAFE =            structs::NS_STYLE_ALIGN_SAFE as u8,
        /// 'unsafe' (mutually exclusive w. SAFE)
        const ALIGN_UNSAFE =          structs::NS_STYLE_ALIGN_UNSAFE as u8,

        /// Mask for the additional flags above.
        const ALIGN_FLAG_BITS =       structs::NS_STYLE_ALIGN_FLAG_BITS as u8,
    }
}

impl ToCss for AlignFlags {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let s = match *self & !ALIGN_FLAG_BITS {
            ALIGN_AUTO => "auto",
            ALIGN_NORMAL => "normal",
            ALIGN_START => "start",
            ALIGN_END => "end",
            ALIGN_FLEX_START => "flex-start",
            ALIGN_FLEX_END => "flex-end",
            ALIGN_CENTER => "center",
            ALIGN_LEFT => "left",
            ALIGN_RIGHT => "right",
            ALIGN_BASELINE => "baseline",
            ALIGN_LAST_BASELINE => "last baseline",
            ALIGN_STRETCH => "stretch",
            ALIGN_SELF_START => "self-start",
            ALIGN_SELF_END => "self-end",
            ALIGN_SPACE_BETWEEN => "space-between",
            ALIGN_SPACE_AROUND => "space-around",
            ALIGN_SPACE_EVENLY => "space-evenly",
            _ => unreachable!()
        };
        dest.write_str(s)?;

        match *self & ALIGN_FLAG_BITS {
            ALIGN_LEGACY => { dest.write_str(" legacy")?; }
            ALIGN_SAFE => { dest.write_str(" safe")?; }
            ALIGN_UNSAFE => { dest.write_str(" unsafe")?; }
            _ => {}
        }
        Ok(())
    }
}

/// Mask for a single AlignFlags value.
const ALIGN_ALL_BITS: u16 = structs::NS_STYLE_ALIGN_ALL_BITS as u16;
/// Number of bits to shift a fallback alignment.
const ALIGN_ALL_SHIFT: u32 = structs::NS_STYLE_ALIGN_ALL_SHIFT;

/// Value of the `align-content` or `justify-content` property.
///
/// https://drafts.csswg.org/css-align/#content-distribution
///
/// The 16-bit field stores the primary value in its lower 8 bits, and the optional fallback value
/// in its upper 8 bits.  This matches the representation of these properties in Gecko.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
pub struct AlignJustifyContent(u16);

impl AlignJustifyContent {
    /// The initial value 'normal'
    #[inline]
    pub fn normal() -> Self {
        Self::new(ALIGN_NORMAL)
    }

    /// Construct a value with no fallback.
    #[inline]
    pub fn new(flags: AlignFlags) -> Self {
        AlignJustifyContent(flags.bits() as u16)
    }

    /// Construct a value including a fallback alignment.
    ///
    /// https://drafts.csswg.org/css-align/#fallback-alignment
    #[inline]
    pub fn with_fallback(flags: AlignFlags, fallback: AlignFlags) -> Self {
        AlignJustifyContent(flags.bits() as u16 | ((fallback.bits() as u16) << ALIGN_ALL_SHIFT))
    }

    /// The primary alignment
    #[inline]
    pub fn primary(self) -> AlignFlags {
        AlignFlags::from_bits((self.0 & ALIGN_ALL_BITS) as u8)
            .expect("AlignJustifyContent must contain valid flags")
    }

    /// The fallback alignment
    #[inline]
    pub fn fallback(self) -> AlignFlags {
        AlignFlags::from_bits((self.0 >> ALIGN_ALL_SHIFT) as u8)
            .expect("AlignJustifyContent must contain valid flags")
    }

    /// Whether this value has extra flags.
    #[inline]
    pub fn has_extra_flags(self) -> bool {
        self.primary().intersects(ALIGN_FLAG_BITS) || self.fallback().intersects(ALIGN_FLAG_BITS)
    }
}

impl ToCss for AlignJustifyContent {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.primary().to_css(dest)?;
        match self.fallback() {
            ALIGN_AUTO => {}
            fallback => {
                dest.write_str(" ")?;
                fallback.to_css(dest)?;
            }
        }
        Ok(())
    }
}

no_viewport_percentage!(AlignJustifyContent);

impl Parse for AlignJustifyContent {
    // normal | <baseline-position> |
    // [ <content-distribution> || [ <overflow-position>? && <content-position> ] ]
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        // normal | <baseline-position>
        if let Ok(value) = input.try(|input| parse_normal_or_baseline(input)) {
            return Ok(AlignJustifyContent::new(value))
        }

        // <content-distribution> followed by optional <*-position>
        if let Ok(value) = input.try(|input| parse_content_distribution(input)) {
            if let Ok(fallback) = input.try(|input| parse_overflow_content_position(input)) {
                return Ok(AlignJustifyContent::with_fallback(value, fallback))
            }
            return Ok(AlignJustifyContent::new(value))
        }

        // <*-position> followed by optional <content-distribution>
        if let Ok(fallback) = input.try(|input| parse_overflow_content_position(input)) {
            if let Ok(value) = input.try(|input| parse_content_distribution(input)) {
                return Ok(AlignJustifyContent::with_fallback(value, fallback))
            }
            return Ok(AlignJustifyContent::new(fallback))
        }
        Err(StyleParseError::UnspecifiedError.into())
    }
}

/// Value of the `align-self` or `justify-self` property.
///
/// https://drafts.csswg.org/css-align/#self-alignment
#[derive(Copy, Clone, Debug, Eq, PartialEq, ToCss)]
pub struct AlignJustifySelf(pub AlignFlags);

impl AlignJustifySelf {
    /// The initial value 'auto'
    #[inline]
    pub fn auto() -> Self {
        AlignJustifySelf(ALIGN_AUTO)
    }

    /// Whether this value has extra flags.
    #[inline]
    pub fn has_extra_flags(self) -> bool {
        self.0.intersects(ALIGN_FLAG_BITS)
    }
}

no_viewport_percentage!(AlignJustifySelf);

impl Parse for AlignJustifySelf {
    // auto | normal | stretch | <baseline-position> |
    // [ <overflow-position>? && <self-position> ]
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        // auto | normal | stretch | <baseline-position>
        if let Ok(value) = input.try(parse_auto_normal_stretch_baseline) {
            return Ok(AlignJustifySelf(value))
        }
        // [ <overflow-position>? && <self-position> ]
        if let Ok(value) = input.try(parse_overflow_self_position) {
            return Ok(AlignJustifySelf(value))
        }
        Err(StyleParseError::UnspecifiedError.into())
    }
}

/// Value of the `align-items` property
///
/// https://drafts.csswg.org/css-align/#self-alignment
#[derive(Copy, Clone, Debug, Eq, PartialEq, ToCss)]
pub struct AlignItems(pub AlignFlags);

impl AlignItems {
    /// The initial value 'normal'
    #[inline]
    pub fn normal() -> Self {
        AlignItems(ALIGN_NORMAL)
    }

    /// Whether this value has extra flags.
    #[inline]
    pub fn has_extra_flags(self) -> bool {
        self.0.intersects(ALIGN_FLAG_BITS)
    }
}

no_viewport_percentage!(AlignItems);

impl Parse for AlignItems {
    // normal | stretch | <baseline-position> |
    // [ <overflow-position>? && <self-position> ]
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        // normal | stretch | <baseline-position>
        if let Ok(value) = input.try(parse_normal_stretch_baseline) {
            return Ok(AlignItems(value))
        }
        // [ <overflow-position>? && <self-position> ]
        if let Ok(value) = input.try(parse_overflow_self_position) {
            return Ok(AlignItems(value))
        }
        Err(StyleParseError::UnspecifiedError.into())
    }
}

/// Value of the `justify-items` property
///
/// https://drafts.csswg.org/css-align/#justify-items-property
#[derive(Copy, Clone, Debug, Eq, PartialEq, ToCss)]
pub struct JustifyItems(pub AlignFlags);

impl JustifyItems {
    /// The initial value 'auto'
    #[inline]
    pub fn auto() -> Self {
        JustifyItems(ALIGN_AUTO)
    }

    /// The value 'normal'
    #[inline]
    pub fn normal() -> Self {
        JustifyItems(ALIGN_NORMAL)
    }

    /// Whether this value has extra flags.
    #[inline]
    pub fn has_extra_flags(self) -> bool {
        self.0.intersects(ALIGN_FLAG_BITS)
    }
}

no_viewport_percentage!(JustifyItems);

impl Parse for JustifyItems {
    // auto | normal | stretch | <baseline-position> |
    // [ <overflow-position>? && <self-position> ]
    // [ legacy && [ left | right | center ] ]
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        // auto | normal | stretch | <baseline-position>
        if let Ok(value) = input.try(parse_auto_normal_stretch_baseline) {
            return Ok(JustifyItems(value))
        }
        // [ legacy && [ left | right | center ] ]
        if let Ok(value) = input.try(parse_legacy) {
            return Ok(JustifyItems(value))
        }
        // [ <overflow-position>? && <self-position> ]
        if let Ok(value) = parse_overflow_self_position(input) {
            return Ok(JustifyItems(value))
        }
        Err(StyleParseError::UnspecifiedError.into())
    }
}

#[cfg(feature = "gecko")]
impl From<u16> for AlignJustifyContent {
    fn from(bits: u16) -> AlignJustifyContent {
        AlignJustifyContent(bits)
    }
}

#[cfg(feature = "gecko")]
impl From<AlignJustifyContent> for u16 {
    fn from(v: AlignJustifyContent) -> u16 {
        v.0
    }
}

// auto | normal | stretch | <baseline-position>
fn parse_auto_normal_stretch_baseline<'i, 't>(input: &mut Parser<'i, 't>)
                                              -> Result<AlignFlags, ParseError<'i>> {
    if let Ok(baseline) = input.try(parse_baseline) {
        return Ok(baseline);
    }

    try_match_ident_ignore_ascii_case! { input.expect_ident()?,
        "auto" => Ok(ALIGN_AUTO),
        "normal" => Ok(ALIGN_NORMAL),
        "stretch" => Ok(ALIGN_STRETCH),
    }
}

// normal | stretch | <baseline-position>
fn parse_normal_stretch_baseline<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    if let Ok(baseline) = input.try(parse_baseline) {
        return Ok(baseline);
    }

    try_match_ident_ignore_ascii_case! { input.expect_ident()?,
        "normal" => Ok(ALIGN_NORMAL),
        "stretch" => Ok(ALIGN_STRETCH),
    }
}

// normal | <baseline-position>
fn parse_normal_or_baseline<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    if let Ok(baseline) = input.try(parse_baseline) {
        return Ok(baseline);
    }

    input.expect_ident_matching("normal")?;
    Ok(ALIGN_NORMAL)
}

// <baseline-position>
fn parse_baseline<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    // FIXME: remove clone() when lifetimes are non-lexical
    try_match_ident_ignore_ascii_case! { input.expect_ident()?.clone(),
        "baseline" => Ok(ALIGN_BASELINE),
        "first" => {
            input.expect_ident_matching("baseline")?;
            Ok(ALIGN_BASELINE)
        }
        "last" => {
            input.expect_ident_matching("baseline")?;
            Ok(ALIGN_LAST_BASELINE)
        }
    }
}

// <content-distribution>
fn parse_content_distribution<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    try_match_ident_ignore_ascii_case! { input.expect_ident()?,
        "stretch" => Ok(ALIGN_STRETCH),
        "space-between" => Ok(ALIGN_SPACE_BETWEEN),
        "space-around" => Ok(ALIGN_SPACE_AROUND),
        "space-evenly" => Ok(ALIGN_SPACE_EVENLY),
    }
}

// [ <overflow-position>? && <content-position> ]
fn parse_overflow_content_position<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    // <content-position> followed by optional <overflow-position>
    if let Ok(mut content) = input.try(parse_content_position) {
        if let Ok(overflow) = input.try(parse_overflow_position) {
            content |= overflow;
        }
        return Ok(content)
    }
    // <overflow-position> followed by required <content-position>
    if let Ok(overflow) = parse_overflow_position(input) {
        if let Ok(content) = parse_content_position(input) {
            return Ok(overflow | content)
        }
    }
    return Err(StyleParseError::UnspecifiedError.into())
}

// <content-position>
fn parse_content_position<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    try_match_ident_ignore_ascii_case! { input.expect_ident()?,
        "start" => Ok(ALIGN_START),
        "end" => Ok(ALIGN_END),
        "flex-start" => Ok(ALIGN_FLEX_START),
        "flex-end" => Ok(ALIGN_FLEX_END),
        "center" => Ok(ALIGN_CENTER),
        "left" => Ok(ALIGN_LEFT),
        "right" => Ok(ALIGN_RIGHT),
    }
}

// <overflow-position>
fn parse_overflow_position<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    try_match_ident_ignore_ascii_case! { input.expect_ident()?,
        "safe" => Ok(ALIGN_SAFE),
        "unsafe" => Ok(ALIGN_UNSAFE),
    }
}

// [ <overflow-position>? && <self-position> ]
fn parse_overflow_self_position<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    // <self-position> followed by optional <overflow-position>
    if let Ok(mut self_position) = input.try(parse_self_position) {
        if let Ok(overflow) = input.try(parse_overflow_position) {
            self_position |= overflow;
        }
        return Ok(self_position)
    }
    // <overflow-position> followed by required <self-position>
    if let Ok(overflow) = parse_overflow_position(input) {
        if let Ok(self_position) = parse_self_position(input) {
            return Ok(overflow | self_position)
        }
    }
    return Err(StyleParseError::UnspecifiedError.into())
}

// <self-position>
fn parse_self_position<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    try_match_ident_ignore_ascii_case! { input.expect_ident()?,
        "start" => Ok(ALIGN_START),
        "end" => Ok(ALIGN_END),
        "flex-start" => Ok(ALIGN_FLEX_START),
        "flex-end" => Ok(ALIGN_FLEX_END),
        "center" => Ok(ALIGN_CENTER),
        "left" => Ok(ALIGN_LEFT),
        "right" => Ok(ALIGN_RIGHT),
        "self-start" => Ok(ALIGN_SELF_START),
        "self-end" => Ok(ALIGN_SELF_END),
    }
}

// [ legacy && [ left | right | center ] ]
fn parse_legacy<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    let a = input.expect_ident()?.clone();
    let b = input.expect_ident()?;
    if a.eq_ignore_ascii_case("legacy") {
        (match_ignore_ascii_case! { &b,
            "left" => Ok(ALIGN_LEGACY | ALIGN_LEFT),
            "right" => Ok(ALIGN_LEGACY | ALIGN_RIGHT),
            "center" => Ok(ALIGN_LEGACY | ALIGN_CENTER),
            _ => Err(())
        }).map_err(|()| SelectorParseError::UnexpectedIdent(b.clone()).into())
    } else if b.eq_ignore_ascii_case("legacy") {
        (match_ignore_ascii_case! { &a,
            "left" => Ok(ALIGN_LEGACY | ALIGN_LEFT),
            "right" => Ok(ALIGN_LEGACY | ALIGN_RIGHT),
            "center" => Ok(ALIGN_LEGACY | ALIGN_CENTER),
            _ => Err(())
        }).map_err(|()| SelectorParseError::UnexpectedIdent(a).into())
    } else {
        Err(StyleParseError::UnspecifiedError.into())
    }
}
