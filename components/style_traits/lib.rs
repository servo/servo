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
#[macro_use] extern crate cssparser;
extern crate euclid;
#[cfg(feature = "servo")] extern crate heapsize;
#[cfg(feature = "servo")] #[macro_use] extern crate heapsize_derive;
#[cfg(feature = "servo")] #[macro_use] extern crate serde_derive;

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
