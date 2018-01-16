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
#[allow(unused_imports)] use std::ascii::AsciiExt;
use std::fmt;
use style_traits::{ToCss, ParseError, StyleParseErrorKind};

bitflags! {
    /// Constants shared by multiple CSS Box Alignment properties
    ///
    /// These constants match Gecko's `NS_STYLE_ALIGN_*` constants.
    #[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
    #[derive(ToComputedValue)]
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

impl ToCss for AlignFlags {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let s = match *self & !AlignFlags::FLAG_BITS {
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
        };
        dest.write_str(s)?;

        match *self & AlignFlags::FLAG_BITS {
            AlignFlags::LEGACY => { dest.write_str(" legacy")?; }
            AlignFlags::SAFE => { dest.write_str(" safe")?; }
            AlignFlags::UNSAFE => { dest.write_str(" unsafe")?; }
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
/// <https://drafts.csswg.org/css-align/#content-distribution>
///
/// The 16-bit field stores the primary value in its lower 8 bits, and the optional fallback value
/// in its upper 8 bits.  This matches the representation of these properties in Gecko.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
pub struct AlignJustifyContent(u16);

/// Whether fallback is allowed in align-content / justify-content parsing.
///
/// This is used for the place-content shorthand, until the resolutions from [1]
/// are specified.
///
/// [1]: https://github.com/w3c/csswg-drafts/issues/1002
#[derive(Clone, Copy, PartialEq)]
pub enum FallbackAllowed {
    /// Allow fallback alignment.
    Yes,
    /// Don't allow fallback alignment.
    No,
}


impl AlignJustifyContent {
    /// The initial value 'normal'
    #[inline]
    pub fn normal() -> Self {
        Self::new(AlignFlags::NORMAL)
    }

    /// Construct a value with no fallback.
    #[inline]
    pub fn new(flags: AlignFlags) -> Self {
        AlignJustifyContent(flags.bits() as u16)
    }

    /// Construct a value including a fallback alignment.
    ///
    /// <https://drafts.csswg.org/css-align/#fallback-alignment>
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
        self.primary().intersects(AlignFlags::FLAG_BITS) || self.fallback().intersects(AlignFlags::FLAG_BITS)
    }

    /// Parse a value for align-content / justify-content, optionally allowing
    /// fallback.
    pub fn parse_with_fallback<'i, 't>(
        input: &mut Parser<'i, 't>,
        fallback_allowed: FallbackAllowed,
    ) -> Result<Self, ParseError<'i>> {
        // normal | <baseline-position>
        if let Ok(value) = input.try(|input| parse_normal_or_baseline(input)) {
            return Ok(AlignJustifyContent::new(value))
        }

        // <content-distribution> followed by optional <*-position>
        if let Ok(value) = input.try(|input| parse_content_distribution(input)) {
            if fallback_allowed == FallbackAllowed::Yes {
                if let Ok(fallback) = input.try(|input| parse_overflow_content_position(input)) {
                    return Ok(AlignJustifyContent::with_fallback(value, fallback))
                }
            }
            return Ok(AlignJustifyContent::new(value))
        }

        // <*-position> followed by optional <content-distribution>
        let fallback = parse_overflow_content_position(input)?;
        if fallback_allowed == FallbackAllowed::Yes {
            if let Ok(value) = input.try(|input| parse_content_distribution(input)) {
                return Ok(AlignJustifyContent::with_fallback(value, fallback))
            }
        }

        Ok(AlignJustifyContent::new(fallback))
    }
}

impl ToCss for AlignJustifyContent {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.primary().to_css(dest)?;
        match self.fallback() {
            AlignFlags::AUTO => {}
            fallback => {
                dest.write_str(" ")?;
                fallback.to_css(dest)?;
            }
        }
        Ok(())
    }
}


impl Parse for AlignJustifyContent {
    // normal | <baseline-position> |
    // [ <content-distribution> || [ <overflow-position>? && <content-position> ] ]
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Self::parse_with_fallback(input, FallbackAllowed::Yes)
    }
}

/// Value of the `align-self` or `justify-self` property.
///
/// <https://drafts.csswg.org/css-align/#self-alignment>
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ToComputedValue, ToCss)]
pub struct AlignJustifySelf(pub AlignFlags);

impl AlignJustifySelf {
    /// The initial value 'auto'
    #[inline]
    pub fn auto() -> Self {
        AlignJustifySelf(AlignFlags::AUTO)
    }

    /// Whether this value has extra flags.
    #[inline]
    pub fn has_extra_flags(self) -> bool {
        self.0.intersects(AlignFlags::FLAG_BITS)
    }
}


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
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

/// Value of the `align-items` property
///
/// <https://drafts.csswg.org/css-align/#self-alignment>
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ToComputedValue, ToCss)]
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
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

/// Value of the `justify-items` property
///
/// <https://drafts.csswg.org/css-align/#justify-items-property>
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ToCss)]
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
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
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

    try_match_ident_ignore_ascii_case! { input,
        "auto" => Ok(AlignFlags::AUTO),
        "normal" => Ok(AlignFlags::NORMAL),
        "stretch" => Ok(AlignFlags::STRETCH),
    }
}

// normal | stretch | <baseline-position>
fn parse_normal_stretch_baseline<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    if let Ok(baseline) = input.try(parse_baseline) {
        return Ok(baseline);
    }

    try_match_ident_ignore_ascii_case! { input,
        "normal" => Ok(AlignFlags::NORMAL),
        "stretch" => Ok(AlignFlags::STRETCH),
    }
}

// normal | <baseline-position>
fn parse_normal_or_baseline<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    if let Ok(baseline) = input.try(parse_baseline) {
        return Ok(baseline);
    }

    input.expect_ident_matching("normal")?;
    Ok(AlignFlags::NORMAL)
}

// <baseline-position>
fn parse_baseline<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    // FIXME: remove clone() when lifetimes are non-lexical
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
    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
}

// <content-position>
fn parse_content_position<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    try_match_ident_ignore_ascii_case! { input,
        "start" => Ok(AlignFlags::START),
        "end" => Ok(AlignFlags::END),
        "flex-start" => Ok(AlignFlags::FLEX_START),
        "flex-end" => Ok(AlignFlags::FLEX_END),
        "center" => Ok(AlignFlags::CENTER),
        "left" => Ok(AlignFlags::LEFT),
        "right" => Ok(AlignFlags::RIGHT),
    }
}

// <overflow-position>
fn parse_overflow_position<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    try_match_ident_ignore_ascii_case! { input,
        "safe" => Ok(AlignFlags::SAFE),
        "unsafe" => Ok(AlignFlags::UNSAFE),
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
    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
}

// <self-position>
fn parse_self_position<'i, 't>(input: &mut Parser<'i, 't>) -> Result<AlignFlags, ParseError<'i>> {
    try_match_ident_ignore_ascii_case! { input,
        "start" => Ok(AlignFlags::START),
        "end" => Ok(AlignFlags::END),
        "flex-start" => Ok(AlignFlags::FLEX_START),
        "flex-end" => Ok(AlignFlags::FLEX_END),
        "center" => Ok(AlignFlags::CENTER),
        "left" => Ok(AlignFlags::LEFT),
        "right" => Ok(AlignFlags::RIGHT),
        "self-start" => Ok(AlignFlags::SELF_START),
        "self-end" => Ok(AlignFlags::SELF_END),
    }
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
