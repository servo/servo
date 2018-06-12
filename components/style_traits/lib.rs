/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains shared types and messages for use by devtools/script.
//! The traits are here instead of in script so that the devtools crate can be
//! modified independently of the rest of Servo.

#![crate_name = "style_traits"]
#![crate_type = "rlib"]

#![deny(unsafe_code, missing_docs)]

extern crate app_units;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate cssparser;
extern crate euclid;
extern crate malloc_size_of;
#[macro_use] extern crate malloc_size_of_derive;
extern crate selectors;
#[cfg(feature = "servo")] #[macro_use] extern crate serde;
#[cfg(feature = "servo")] extern crate webrender_api;
extern crate servo_arc;
#[cfg(feature = "servo")] extern crate servo_atoms;
#[cfg(feature = "servo")] extern crate servo_url;

#[cfg(feature = "servo")] pub use webrender_api::DevicePixel;

use cssparser::{CowRcStr, Token};
use selectors::parser::SelectorParseErrorKind;
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
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize, MallocSizeOf))]
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
/// <http://www.w3.org/TR/css-device-adapt/#initial-viewport>
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
pub mod specified_value_info;
#[macro_use]
pub mod values;
#[macro_use]
pub mod viewport;

pub use specified_value_info::{CssType, KeywordsCollectFn, SpecifiedValueInfo};
pub use values::{Comma, CommaWithSpace, CssWriter, OneOrMoreSeparated, Separator, Space, ToCss};

/// The error type for all CSS parsing routines.
pub type ParseError<'i> = cssparser::ParseError<'i, StyleParseErrorKind<'i>>;

/// Error in property value parsing
pub type ValueParseError<'i> = cssparser::ParseError<'i, ValueParseErrorKind<'i>>;

#[derive(Clone, Debug, PartialEq)]
/// Errors that can be encountered while parsing CSS values.
pub enum StyleParseErrorKind<'i> {
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
    ValueError(ValueParseErrorKind<'i>),
    /// An error was encountered while parsing a selector
    SelectorError(SelectorParseErrorKind<'i>),

    /// The property declaration was for an unknown property.
    UnknownProperty(CowRcStr<'i>),
    /// An unknown vendor-specific identifier was encountered.
    UnknownVendorProperty,
    /// The property declaration was for a disabled experimental property.
    ExperimentalProperty,
    /// The property declaration contained an invalid color value.
    InvalidColor(CowRcStr<'i>, Token<'i>),
    /// The property declaration contained an invalid filter value.
    InvalidFilter(CowRcStr<'i>, Token<'i>),
    /// The property declaration contained an invalid value.
    OtherInvalidValue(CowRcStr<'i>),
    /// The declaration contained an animation property, and we were parsing
    /// this as a keyframe block (so that property should be ignored).
    ///
    /// See: https://drafts.csswg.org/css-animations/#keyframes
    AnimationPropertyInKeyframeBlock,
    /// The property is not allowed within a page rule.
    NotAllowedInPageRule,
}

impl<'i> From<ValueParseErrorKind<'i>> for StyleParseErrorKind<'i> {
    fn from(this: ValueParseErrorKind<'i>) -> Self {
        StyleParseErrorKind::ValueError(this)
    }
}

impl<'i> From<SelectorParseErrorKind<'i>> for StyleParseErrorKind<'i> {
    fn from(this: SelectorParseErrorKind<'i>) -> Self {
        StyleParseErrorKind::SelectorError(this)
    }
}

/// Specific errors that can be encountered while parsing property values.
#[derive(Clone, Debug, PartialEq)]
pub enum ValueParseErrorKind<'i> {
    /// An invalid token was encountered while parsing a color value.
    InvalidColor(Token<'i>),
    /// An invalid filter value was encountered.
    InvalidFilter(Token<'i>),
}

impl<'i> StyleParseErrorKind<'i> {
    /// Create an InvalidValue parse error
    pub fn new_invalid<S>(name: S, value_error: ParseError<'i>) -> ParseError<'i>
    where
        S: Into<CowRcStr<'i>>,
    {
        let name = name.into();
        let variant = match value_error.kind {
            cssparser::ParseErrorKind::Custom(StyleParseErrorKind::ValueError(e)) => {
                match e {
                    ValueParseErrorKind::InvalidColor(token) => {
                        StyleParseErrorKind::InvalidColor(name, token)
                    }
                    ValueParseErrorKind::InvalidFilter(token) => {
                        StyleParseErrorKind::InvalidFilter(name, token)
                    }
                }
            }
            _ => StyleParseErrorKind::OtherInvalidValue(name),
        };
        cssparser::ParseError {
            kind: cssparser::ParseErrorKind::Custom(variant),
            location: value_error.location,
        }
    }
}

bitflags! {
    /// The mode to use when parsing values.
    pub struct ParsingMode: u8 {
        /// In CSS; lengths must have units, except for zero values, where the unit can be omitted.
        /// <https://www.w3.org/TR/css3-values/#lengths>
        const DEFAULT = 0x00;
        /// In SVG; a coordinate or length value without a unit identifier (e.g., "25") is assumed
        /// to be in user units (px).
        /// <https://www.w3.org/TR/SVG/coords.html#Units>
        const ALLOW_UNITLESS_LENGTH = 0x01;
        /// In SVG; out-of-range values are not treated as an error in parsing.
        /// <https://www.w3.org/TR/SVG/implnote.html#RangeClamping>
        const ALLOW_ALL_NUMERIC_VALUES = 0x02;
    }
}

impl ParsingMode {
    /// Whether the parsing mode allows unitless lengths for non-zero values to be intpreted as px.
    #[inline]
    pub fn allows_unitless_lengths(&self) -> bool {
        self.intersects(ParsingMode::ALLOW_UNITLESS_LENGTH)
    }

    /// Whether the parsing mode allows all numeric values.
    #[inline]
    pub fn allows_all_numeric_values(&self) -> bool {
        self.intersects(ParsingMode::ALLOW_ALL_NUMERIC_VALUES)
    }
}

#[cfg(feature = "servo")]
/// Speculatively execute paint code in the worklet thread pool.
pub trait SpeculativePainter: Send + Sync {
    /// <https://drafts.css-houdini.org/css-paint-api/#draw-a-paint-image>
    fn speculatively_draw_a_paint_image(&self, properties: Vec<(Atom, String)>, arguments: Vec<String>);
}
