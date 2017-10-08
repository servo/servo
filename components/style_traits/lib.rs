/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains shared types and messages for use by devtools/script.
//! The traits are here instead of in script so that the devtools crate can be
//! modified independently of the rest of Servo.

#![crate_name = "style_traits"]
#![crate_type = "rlib"]

#![deny(unsafe_code, missing_docs)]

#![cfg_attr(feature = "servo", feature(plugin))]

extern crate app_units;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate cssparser;
extern crate euclid;
#[cfg(feature = "servo")] extern crate heapsize;
#[cfg(feature = "servo")] #[macro_use] extern crate heapsize_derive;
#[cfg(feature = "gecko")] extern crate malloc_size_of;
#[cfg(feature = "gecko")] #[macro_use] extern crate malloc_size_of_derive;
extern crate selectors;
#[cfg(feature = "servo")] #[macro_use] extern crate serde;
#[cfg(feature = "servo")] extern crate webrender_api;
extern crate servo_arc;
#[cfg(feature = "servo")] extern crate servo_atoms;

#[cfg(feature = "servo")] pub use webrender_api::DevicePixel;

use cssparser::{CowRcStr, Token};
use selectors::parser::SelectorParseError;
#[cfg(feature = "servo")] use servo_atoms::Atom;

/// One hardware pixel.
///
/// This unit corresponds to the smallest addressable element of the display hardware.
#[cfg(not(feature = "servo"))]
#[derive(Clone, Copy, Debug)]
pub enum DevicePixel {}

/// Represents a mobile style pinch zoom factor.
/// TODO(gw): Once WR supports pinch zoom, use a type directly from webrender_api.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize, HeapSizeOf))]
pub struct PinchZoomFactor(f32);

impl PinchZoomFactor {
    /// Construct a new pinch zoom factor.
    pub fn new(scale: f32) -> PinchZoomFactor {
        PinchZoomFactor(scale)
    }

    /// Get the pinch zoom factor as an untyped float.
    pub fn get(&self) -> f32 {
        self.0
    }
}

/// One CSS "px" in the coordinate system of the "initial viewport":
/// http://www.w3.org/TR/css-device-adapt/#initial-viewport
///
/// `CSSPixel` is equal to `DeviceIndependentPixel` times a "page zoom" factor controlled by the user.  This is
/// the desktop-style "full page" zoom that enlarges content but then reflows the layout viewport
/// so it still exactly fits the visible area.
///
/// At the default zoom level of 100%, one `CSSPixel` is equal to one `DeviceIndependentPixel`.  However, if the
/// document is zoomed in or out then this scale may be larger or smaller.
#[derive(Clone, Copy, Debug)]
pub enum CSSPixel {}

// In summary, the hierarchy of pixel units and the factors to convert from one to the next:
//
// DevicePixel
//   / hidpi_ratio => DeviceIndependentPixel
//     / desktop_zoom => CSSPixel

pub mod cursor;
#[macro_use]
pub mod values;
#[macro_use]
pub mod viewport;

pub use values::{Comma, CommaWithSpace, OneOrMoreSeparated, Separator, Space, ToCss};

/// The error type for all CSS parsing routines.
pub type ParseError<'i> = cssparser::ParseError<'i, SelectorParseError<'i, StyleParseError<'i>>>;

#[derive(Clone, Debug, PartialEq)]
/// Errors that can be encountered while parsing CSS values.
pub enum StyleParseError<'i> {
    /// A bad URL token in a DVB.
    BadUrlInDeclarationValueBlock(CowRcStr<'i>),
    /// A bad string token in a DVB.
    BadStringInDeclarationValueBlock(CowRcStr<'i>),
    /// Unexpected closing parenthesis in a DVB.
    UnbalancedCloseParenthesisInDeclarationValueBlock,
    /// Unexpected closing bracket in a DVB.
    UnbalancedCloseSquareBracketInDeclarationValueBlock,
    /// Unexpected closing curly bracket in a DVB.
    UnbalancedCloseCurlyBracketInDeclarationValueBlock,
    /// A property declaration parsing error.
    PropertyDeclaration(PropertyDeclarationParseError<'i>),
    /// A property declaration value had input remaining after successfully parsing.
    PropertyDeclarationValueNotExhausted,
    /// An unexpected dimension token was encountered.
    UnexpectedDimension(CowRcStr<'i>),
    /// Expected identifier not found.
    ExpectedIdentifier(Token<'i>),
    /// Missing or invalid media feature name.
    MediaQueryExpectedFeatureName(CowRcStr<'i>),
    /// Missing or invalid media feature value.
    MediaQueryExpectedFeatureValue,
    /// min- or max- properties must have a value.
    RangedExpressionWithNoValue,
    /// A function was encountered that was not expected.
    UnexpectedFunction(CowRcStr<'i>),
    /// @namespace must be before any rule but @charset and @import
    UnexpectedNamespaceRule,
    /// @import must be before any rule but @charset
    UnexpectedImportRule,
    /// Unexpected @charset rule encountered.
    UnexpectedCharsetRule,
    /// Unsupported @ rule
    UnsupportedAtRule(CowRcStr<'i>),
    /// A placeholder for many sources of errors that require more specific variants.
    UnspecifiedError,
    /// An unexpected token was found within a namespace rule.
    UnexpectedTokenWithinNamespace(Token<'i>),
    /// An error was encountered while parsing a property value.
    ValueError(ValueParseError<'i>),
}

/// Specific errors that can be encountered while parsing property values.
#[derive(Clone, Debug, PartialEq)]
pub enum ValueParseError<'i> {
    /// An invalid token was encountered while parsing a color value.
    InvalidColor(Token<'i>),
    /// An invalid filter value was encountered.
    InvalidFilter(Token<'i>),
}

impl<'a> From<ValueParseError<'a>> for ParseError<'a> {
    fn from(this: ValueParseError<'a>) -> Self {
        StyleParseError::ValueError(this).into()
    }
}

impl<'i> ValueParseError<'i> {
    /// Attempt to extract a ValueParseError value from a ParseError.
    pub fn from_parse_error(this: ParseError<'i>) -> Option<ValueParseError<'i>> {
        match this {
            cssparser::ParseError::Custom(
                SelectorParseError::Custom(
                    StyleParseError::ValueError(e))) => Some(e),
            _ => None,
        }
    }
}

/// The result of parsing a property declaration.
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyDeclarationParseError<'i> {
    /// The property declaration was for an unknown property.
    UnknownProperty(CowRcStr<'i>),
    /// An unknown vendor-specific identifier was encountered.
    UnknownVendorProperty,
    /// The property declaration was for a disabled experimental property.
    ExperimentalProperty,
    /// The property declaration contained an invalid value.
    InvalidValue(CowRcStr<'i>, Option<ValueParseError<'i>>),
    /// The declaration contained an animation property, and we were parsing
    /// this as a keyframe block (so that property should be ignored).
    ///
    /// See: https://drafts.csswg.org/css-animations/#keyframes
    AnimationPropertyInKeyframeBlock,
    /// The property is not allowed within a page rule.
    NotAllowedInPageRule,
}

impl<'a> From<StyleParseError<'a>> for ParseError<'a> {
    fn from(this: StyleParseError<'a>) -> Self {
        cssparser::ParseError::Custom(SelectorParseError::Custom(this))
    }
}

impl<'a> From<PropertyDeclarationParseError<'a>> for ParseError<'a> {
    fn from(this: PropertyDeclarationParseError<'a>) -> Self {
        cssparser::ParseError::Custom(SelectorParseError::Custom(StyleParseError::PropertyDeclaration(this)))
    }
}

bitflags! {
    /// The mode to use when parsing values.
    pub flags ParsingMode: u8 {
        /// In CSS, lengths must have units, except for zero values, where the unit can be omitted.
        /// https://www.w3.org/TR/css3-values/#lengths
        const PARSING_MODE_DEFAULT = 0x00,
        /// In SVG, a coordinate or length value without a unit identifier (e.g., "25") is assumed
        /// to be in user units (px).
        /// https://www.w3.org/TR/SVG/coords.html#Units
        const PARSING_MODE_ALLOW_UNITLESS_LENGTH = 0x01,
        /// In SVG, out-of-range values are not treated as an error in parsing.
        /// https://www.w3.org/TR/SVG/implnote.html#RangeClamping
        const PARSING_MODE_ALLOW_ALL_NUMERIC_VALUES = 0x02,
    }
}

impl ParsingMode {
    /// Whether the parsing mode allows unitless lengths for non-zero values to be intpreted as px.
    pub fn allows_unitless_lengths(&self) -> bool {
        self.intersects(PARSING_MODE_ALLOW_UNITLESS_LENGTH)
    }

    /// Whether the parsing mode allows all numeric values.
    pub fn allows_all_numeric_values(&self) -> bool {
        self.intersects(PARSING_MODE_ALLOW_ALL_NUMERIC_VALUES)
    }
}

#[cfg(feature = "servo")]
/// Speculatively execute paint code in the worklet thread pool.
pub trait SpeculativePainter: Send + Sync {
    /// https://drafts.css-houdini.org/css-paint-api/#draw-a-paint-image
    fn speculatively_draw_a_paint_image(&self, properties: Vec<(Atom, String)>, arguments: Vec<String>);
}
