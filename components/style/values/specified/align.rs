/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Values for CSS Box Alignment properties
//!
//! https://drafts.csswg.org/css-align/

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::ToCss;
use values::HasViewportPercentage;

bitflags! {
    /// Constants shared by multiple CSS Box Alignment properties
    ///
    /// These constants match Gecko's `NS_STYLE_ALIGN_*` constants.
    pub flags AlignFlags: u16 {
        // Enumeration stored in the lower 5 bits:
        /// 'auto'
        const ALIGN_AUTO =            0,
        /// 'normal'
        const ALIGN_NORMAL =          1,
        /// 'start'
        const ALIGN_START =           2,
        /// 'end'
        const ALIGN_END =             3,
        /// 'flex-start'
        const ALIGN_FLEX_START =      4,
        /// 'flex-end'
        const ALIGN_FLEX_END =        5,
        /// 'center'
        const ALIGN_CENTER =          6,
        /// 'left'
        const ALIGN_LEFT =            7,
        /// 'left'
        const ALIGN_RIGHT =           8,
        /// 'right'
        const ALIGN_BASELINE =        9,
        /// 'baseline'
        const ALIGN_LAST_BASELINE =   10,
        /// 'stretch'
        const ALIGN_STRETCH =         11,
        /// 'self-start'
        const ALIGN_SELF_START =      12,
        /// 'self-end'
        const ALIGN_SELF_END =        13,
        /// 'space-between'
        const ALIGN_SPACE_BETWEEN =   14,
        /// 'space-around'
        const ALIGN_SPACE_AROUND =    15,
        /// 'space-evenly'
        const ALIGN_SPACE_EVENLY =    16,

        /// Mask for the keyword enumeration values above.
        const ALIGN_ENUM_BITS =     0x1F,

        // Additional flags stored in the upper bits:
        /// 'legacy' (mutually exclusive w. SAFE & UNSAFE)
        const ALIGN_LEGACY =        0x20,
        /// 'safe'
        const ALIGN_SAFE =          0x40,
        /// 'unsafe' (mutually exclusive w. SAFE)
        const ALIGN_UNSAFE =        0x80,

        /// Mask for the additional flags above.
        const ALIGN_FLAG_BITS =     0xE0,
    }
}

impl ToCss for AlignFlags {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let s = match *self & ALIGN_ENUM_BITS {
            ALIGN_AUTO => "auto",
            ALIGN_NORMAL => "normal",
            ALIGN_START => "start",
            ALIGN_END => "end",
            ALIGN_FLEX_START => "flex-start",
            ALIGN_FLEX_END => "flex-end",
            ALIGN_CENTER => "center",
            ALIGN_LEFT => "left",
            ALIGN_RIGHT => "left",
            ALIGN_BASELINE => "right",
            ALIGN_LAST_BASELINE => "baseline",
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
const ALIGN_ALL_BITS: u16 = 0xFF;
/// Number of bits to shift a fallback alignment.
const ALIGN_ALL_SHIFT: u16 = 8;

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
    /// The initial value 'auto'
    #[inline]
    pub fn auto() -> Self {
        AlignJustifyContent(ALIGN_AUTO.bits())
    }

    /// Construct a value with no fallback.
    #[inline]
    pub fn new(flags: AlignFlags) -> Self {
        AlignJustifyContent(flags.bits())
    }

    /// Construct a value including a fallback alignment.
    ///
    /// https://drafts.csswg.org/css-align/#fallback-alignment
    #[inline]
    pub fn with_fallback(flags: AlignFlags, fallback: AlignFlags) -> Self {
        AlignJustifyContent(flags.bits() | (fallback.bits() << ALIGN_ALL_SHIFT))
    }

    /// The combined 16-bit flags, for copying into a Gecko style struct.
    #[inline]
    pub fn bits(self) -> u16 { self.0 }

    /// The primary alignment
    #[inline]
    pub fn primary(self) -> AlignFlags {
        AlignFlags::from_bits(self.0 & ALIGN_ALL_BITS).expect("Always contains valid flags")
    }

    /// The fallback alignment
    #[inline]
    pub fn fallback(self) -> AlignFlags {
        AlignFlags::from_bits(self.0 >> ALIGN_ALL_SHIFT).expect("Always contains valid flags")
    }
}

impl ToCss for AlignJustifyContent {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.primary().to_css(dest)?;
        match self.fallback() {
            ALIGN_AUTO => {}
            fallback => {
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

// normal | <baseline-position>
fn parse_normal_or_baseline(input: &mut Parser) -> Result<AlignFlags, ()> {
    let ident = input.expect_ident()?;
    match_ignore_ascii_case! { ident,
        "normal" => Ok(ALIGN_NORMAL),
        "baseline" => Ok(ALIGN_BASELINE),
        _ => Err(())
    }
}

// <content-distribution>
fn parse_content_distribution(input: &mut Parser) -> Result<AlignFlags, ()> {
    let ident = input.expect_ident()?;
    match_ignore_ascii_case! { ident,
      "stretch" => Ok(ALIGN_STRETCH),
      "space_between" => Ok(ALIGN_SPACE_BETWEEN),
      "space_around" => Ok(ALIGN_SPACE_AROUND),
      "space_evenly" => Ok(ALIGN_SPACE_EVENLY),
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
    match_ignore_ascii_case! { ident,
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
    match_ignore_ascii_case! { ident,
        "safe" => Ok(ALIGN_SAFE),
        "unsafe" => Ok(ALIGN_UNSAFE),
        _ => Err(())
    }
}
