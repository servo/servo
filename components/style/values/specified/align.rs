/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Values for CSS Box Alignment properties
//!
//! https://drafts.csswg.org/css-align/

use cssparser::Parser;
use gecko_bindings::structs;
use parser::{Parse, ParserContext};
use std::ascii::AsciiExt;
use std::fmt;
use style_traits::ToCss;
use values::HasViewportPercentage;

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
        /// 'left'
        const ALIGN_RIGHT =           structs::NS_STYLE_ALIGN_RIGHT as u8,
        /// 'right'
        const ALIGN_BASELINE =        structs::NS_STYLE_ALIGN_BASELINE as u8,
        /// 'baseline'
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
            ALIGN_RIGHT => "left",
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

    /// The combined 16-bit flags, for copying into a Gecko style struct.
    #[inline]
    pub fn bits(self) -> u16 { self.0 }

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
    fn parse(_: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
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
        Err(())
    }
}

/// Value of the `align-self` or `justify-self` property.
///
/// https://drafts.csswg.org/css-align/#self-alignment
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

impl ToCss for AlignJustifySelf {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)
    }
}

impl Parse for AlignJustifySelf {
    // auto | normal | stretch | <baseline-position> |
    // [ <overflow-position>? && <self-position> ]
    fn parse(_: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        // auto | normal | stretch | <baseline-position>
        if let Ok(value) = input.try(parse_auto_normal_stretch_baseline) {
            return Ok(AlignJustifySelf(value))
        }
        // [ <overflow-position>? && <self-position> ]
        if let Ok(value) = input.try(parse_overflow_self_position) {
            return Ok(AlignJustifySelf(value))
        }
        Err(())
    }
}

/// Value of the `align-items` property
///
/// https://drafts.csswg.org/css-align/#self-alignment
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

impl ToCss for AlignItems {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)
    }
}

impl Parse for AlignItems {
    // normal | stretch | <baseline-position> |
    // [ <overflow-position>? && <self-position> ]
    fn parse(_: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        // normal | stretch | <baseline-position>
        if let Ok(value) = input.try(parse_normal_stretch_baseline) {
            return Ok(AlignItems(value))
        }
        // [ <overflow-position>? && <self-position> ]
        if let Ok(value) = input.try(parse_overflow_self_position) {
            return Ok(AlignItems(value))
        }
        Err(())
    }
}

/// Value of the `justify-items` property
///
/// https://drafts.csswg.org/css-align/#justify-items-property
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct JustifyItems(pub AlignFlags);

impl JustifyItems {
    /// The initial value 'auto'
    #[inline]
    pub fn auto() -> Self {
        JustifyItems(ALIGN_AUTO)
    }

    /// Whether this value has extra flags.
    #[inline]
    pub fn has_extra_flags(self) -> bool {
        self.0.intersects(ALIGN_FLAG_BITS)
    }
}

no_viewport_percentage!(JustifyItems);

impl ToCss for JustifyItems {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)
    }
}

impl Parse for JustifyItems {
    // auto | normal | stretch | <baseline-position> |
    // [ <overflow-position>? && <self-position> ]
    // [ legacy && [ left | right | center ] ]
    fn parse(_: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        // auto | normal | stretch | <baseline-position>
        if let Ok(value) = input.try(parse_auto_normal_stretch_baseline) {
            return Ok(JustifyItems(value))
        }
        // [ <overflow-position>? && <self-position> ]
        if let Ok(value) = input.try(parse_overflow_self_position) {
            return Ok(JustifyItems(value))
        }
        // [ legacy && [ left | right | center ] ]
        if let Ok(value) = input.try(parse_legacy) {
            return Ok(JustifyItems(value))
        }
        Err(())
    }
}

// auto | normal | stretch | <baseline-position>
fn parse_auto_normal_stretch_baseline(input: &mut Parser) -> Result<AlignFlags, ()> {
    if let Ok(baseline) = input.try(|input| parse_baseline(input)) {
        return Ok(baseline);
    }

    let ident = input.expect_ident()?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(ALIGN_AUTO),
        "normal" => Ok(ALIGN_NORMAL),
        "stretch" => Ok(ALIGN_STRETCH),
        _ => Err(())
    }
}

// normal | stretch | <baseline-position>
fn parse_normal_stretch_baseline(input: &mut Parser) -> Result<AlignFlags, ()> {
    if let Ok(baseline) = input.try(|input| parse_baseline(input)) {
        return Ok(baseline);
    }

    let ident = input.expect_ident()?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(ALIGN_NORMAL),
        "stretch" => Ok(ALIGN_STRETCH),
        _ => Err(())
    }
}

// normal | <baseline-position>
fn parse_normal_or_baseline(input: &mut Parser) -> Result<AlignFlags, ()> {
    if let Ok(baseline) = input.try(|input| parse_baseline(input)) {
        return Ok(baseline);
    }

    let ident = input.expect_ident()?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(ALIGN_NORMAL),
        _ => Err(())
    }
}

// <baseline-position>
fn parse_baseline(input: &mut Parser) -> Result<AlignFlags, ()> {
    let ident = input.expect_ident()?;
    match_ignore_ascii_case! { &ident,
        "baseline" => Ok(ALIGN_BASELINE),
        "first" => {
            if input.try(|input| input.expect_ident_matching("baseline")).is_ok() {
                return Ok(ALIGN_BASELINE);
            }
            Err(())
        },
        "last" => {
            if input.try(|input| input.expect_ident_matching("baseline")).is_ok() {
                return Ok(ALIGN_LAST_BASELINE);
            }
            Err(())
        },
        _ => Err(())
    }
}

// <content-distribution>
fn parse_content_distribution(input: &mut Parser) -> Result<AlignFlags, ()> {
    let ident = input.expect_ident()?;
    match_ignore_ascii_case! { &ident,
      "stretch" => Ok(ALIGN_STRETCH),
      "space-between" => Ok(ALIGN_SPACE_BETWEEN),
      "space-around" => Ok(ALIGN_SPACE_AROUND),
      "space-evenly" => Ok(ALIGN_SPACE_EVENLY),
      _ => Err(())
    }
}

// [ <overflow-position>? && <content-position> ]
fn parse_overflow_content_position(input: &mut Parser) -> Result<AlignFlags, ()> {
    // <content-position> followed by optional <overflow-position>
    if let Ok(mut content) = input.try(|input| parse_content_position(input)) {
        if let Ok(overflow) = input.try(|input| parse_overflow_position(input)) {
            content |= overflow;
        }
        return Ok(content)
    }
    // <overflow-position> followed by required <content-position>
    if let Ok(overflow) = input.try(|input| parse_overflow_position(input)) {
        if let Ok(content) = input.try(|input| parse_content_position(input)) {
            return Ok(overflow | content)
        }
    }
    return Err(())
}

// <content-position>
fn parse_content_position(input: &mut Parser) -> Result<AlignFlags, ()> {
    let ident = input.expect_ident()?;
    match_ignore_ascii_case! { &ident,
        "start" => Ok(ALIGN_START),
        "end" => Ok(ALIGN_END),
        "flex-start" => Ok(ALIGN_FLEX_START),
        "flex-end" => Ok(ALIGN_FLEX_END),
        "center" => Ok(ALIGN_CENTER),
        "left" => Ok(ALIGN_LEFT),
        "right" => Ok(ALIGN_RIGHT),
        _ => Err(())
    }
}

// <overflow-position>
fn parse_overflow_position(input: &mut Parser) -> Result<AlignFlags, ()> {
    let ident = input.expect_ident()?;
    match_ignore_ascii_case! { &ident,
        "safe" => Ok(ALIGN_SAFE),
        "unsafe" => Ok(ALIGN_UNSAFE),
        _ => Err(())
    }
}

// [ <overflow-position>? && <self-position> ]
fn parse_overflow_self_position(input: &mut Parser) -> Result<AlignFlags, ()> {
    // <self-position> followed by optional <overflow-position>
    if let Ok(mut self_position) = input.try(|input| parse_self_position(input)) {
        if let Ok(overflow) = input.try(|input| parse_overflow_position(input)) {
            self_position |= overflow;
        }
        return Ok(self_position)
    }
    // <overflow-position> followed by required <self-position>
    if let Ok(overflow) = input.try(|input| parse_overflow_position(input)) {
        if let Ok(self_position) = input.try(|input| parse_self_position(input)) {
            return Ok(overflow | self_position)
        }
    }
    return Err(())
}

// <self-position>
fn parse_self_position(input: &mut Parser) -> Result<AlignFlags, ()> {
    let ident = input.expect_ident()?;
    match_ignore_ascii_case! { &ident,
        "start" => Ok(ALIGN_START),
        "end" => Ok(ALIGN_END),
        "flex-start" => Ok(ALIGN_FLEX_START),
        "flex-end" => Ok(ALIGN_FLEX_END),
        "center" => Ok(ALIGN_CENTER),
        "left" => Ok(ALIGN_LEFT),
        "right" => Ok(ALIGN_RIGHT),
        "self-start" => Ok(ALIGN_SELF_START),
        "self-end" => Ok(ALIGN_SELF_END),
        _ => Err(())
    }
}

// [ legacy && [ left | right | center ] ]
fn parse_legacy(input: &mut Parser) -> Result<AlignFlags, ()> {
    let a = input.expect_ident()?;
    let b = input.expect_ident()?;
    if a.eq_ignore_ascii_case("legacy") {
        match_ignore_ascii_case! { &b,
            "left" => Ok(ALIGN_LEGACY | ALIGN_LEFT),
            "right" => Ok(ALIGN_LEGACY | ALIGN_RIGHT),
            "center" => Ok(ALIGN_LEGACY | ALIGN_CENTER),
            _ => Err(())
        }
    } else if b.eq_ignore_ascii_case("legacy") {
        match_ignore_ascii_case! { &a,
            "left" => Ok(ALIGN_LEGACY | ALIGN_LEFT),
            "right" => Ok(ALIGN_LEGACY | ALIGN_RIGHT),
            "center" => Ok(ALIGN_LEGACY | ALIGN_CENTER),
            _ => Err(())
        }
    } else {
        Err(())
    }
}
