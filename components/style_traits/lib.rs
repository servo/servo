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
#[macro_use]
extern crate cssparser;
extern crate euclid;
#[cfg(feature = "servo")] extern crate heapsize;
#[cfg(feature = "servo")] #[macro_use] extern crate heapsize_derive;
extern crate rustc_serialize;
#[cfg(feature = "servo")] extern crate serde;
#[cfg(feature = "servo")] #[macro_use] extern crate serde_derive;

/// Opaque type stored in type-unsafe work queues for parallel layout.
/// Must be transmutable to and from `TNode`.
pub type UnsafeNode = (usize, usize);

/// One CSS "px" in the coordinate system of the "initial viewport":
/// http://www.w3.org/TR/css-device-adapt/#initial-viewport
///
/// `ViewportPx` is equal to `ScreenPx` times a "page zoom" factor controlled by the user.  This is
/// the desktop-style "full page" zoom that enlarges content but then reflows the layout viewport
/// so it still exactly fits the visible area.
///
/// At the default zoom level of 100%, one `PagePx` is equal to one `ScreenPx`.  However, if the
/// document is zoomed in or out then this scale may be larger or smaller.
#[derive(Clone, Copy, Debug)]
pub enum ViewportPx {}

/// One CSS "px" in the root coordinate system for the content document.
///
/// `PagePx` is equal to `ViewportPx` multiplied by a "viewport zoom" factor controlled by the user.
/// This is the mobile-style "pinch zoom" that enlarges content without reflowing it.  When the
/// viewport zoom is not equal to 1.0, then the layout viewport is no longer the same physical size
/// as the viewable area.
#[derive(Clone, Copy, Debug)]
pub enum PagePx {}

// In summary, the hierarchy of pixel units and the factors to convert from one to the next:
//
// DevicePixel
//   / hidpi_ratio => ScreenPx
//     / desktop_zoom => ViewportPx
//       / pinch_zoom => PagePx

pub mod cursor;
#[macro_use]
pub mod values;
pub mod viewport;

pub use values::ToCss;
