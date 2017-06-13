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
extern crate selectors;
#[cfg(feature = "servo")] #[macro_use] extern crate serde_derive;

use selectors::parser::SelectorParseError;
use std::borrow::Cow;

/// Opaque type stored in type-unsafe work queues for parallel layout.
/// Must be transmutable to and from `TNode`.
pub type UnsafeNode = (usize, usize);

/// Represents a mobile style pinch zoom factor.
/// TODO(gw): Once WR supports pinch zoom, use a type directly from webrender_traits.
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

pub use values::{ToCss, OneOrMoreCommaSeparated};
pub use viewport::HasViewportPercentage;

/// The error type for all CSS parsing routines.
pub type ParseError<'i> = cssparser::ParseError<'i, SelectorParseError<'i, StyleParseError<'i>>>;

#[derive(Clone, Debug, PartialEq)]
/// Errors that can be encountered while parsing CSS values.
pub enum StyleParseError<'i> {
    /// A bad URL token in a DVB.
    BadUrlInDeclarationValueBlock,
    /// A bad string token in a DVB.
    BadStringInDeclarationValueBlock,
    /// Unexpected closing parenthesis in a DVB.
    UnbalancedCloseParenthesisInDeclarationValueBlock,
    /// Unexpected closing bracket in a DVB.
    UnbalancedCloseSquareBracketInDeclarationValueBlock,
    /// Unexpected closing curly bracket in a DVB.
    UnbalancedCloseCurlyBracketInDeclarationValueBlock,
    /// A property declaration parsing error.
    PropertyDeclaration(PropertyDeclarationParseError),
    /// A property declaration value had input remaining after successfully parsing.
    PropertyDeclarationValueNotExhausted,
    /// An unexpected dimension token was encountered.
    UnexpectedDimension(Cow<'i, str>),
    /// A media query using a ranged expression with no value was encountered.
    RangedExpressionWithNoValue,
    /// A function was encountered that was not expected.
    UnexpectedFunction(Cow<'i, str>),
    /// @namespace must be before any rule but @charset and @import
    UnexpectedNamespaceRule,
    /// @import must be before any rule but @charset
    UnexpectedImportRule,
    /// Unexpected @charset rule encountered.
    UnexpectedCharsetRule,
    /// Unsupported @ rule
    UnsupportedAtRule(Cow<'i, str>),
    /// A placeholder for many sources of errors that require more specific variants.
    UnspecifiedError,
}

/// The result of parsing a property declaration.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum PropertyDeclarationParseError {
    /// The property declaration was for an unknown property.
    UnknownProperty,
    /// The property declaration was for a disabled experimental property.
    ExperimentalProperty,
    /// The property declaration contained an invalid value.
    InvalidValue,
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

impl<'a> From<PropertyDeclarationParseError> for ParseError<'a> {
    fn from(this: PropertyDeclarationParseError) -> Self {
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

