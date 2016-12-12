/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::{Au, MAX_AU};
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::side_offsets::SideOffsets2D;
use euclid::size::Size2D;
use std::i32;

// Units for use with euclid::length and euclid::scale_factor.

/// A normalized "pixel" at the default resolution for the display.
///
/// Like the CSS "px" unit, the exact physical size of this unit may vary between devices, but it
/// should approximate a device-independent reference length.  This unit corresponds to Android's
/// "density-independent pixel" (dip), Mac OS X's "point", and Windows "device-independent pixel."
///
/// The relationship between DevicePixel and ScreenPx is defined by the OS.  On most low-dpi
/// screens, one ScreenPx is equal to one DevicePixel.  But on high-density screens it can be
/// some larger number.  For example, by default on Apple "retina" displays, one ScreenPx equals
/// two DevicePixels.  On Android "MDPI" displays, one ScreenPx equals 1.5 device pixels.
///
/// The ratio between ScreenPx and DevicePixel for a given display be found by calling
/// `servo::windowing::WindowMethods::hidpi_factor`.
#[derive(Clone, Copy, Debug)]
pub enum ScreenPx {}

known_heap_size!(0, ScreenPx);

// An Au is an "App Unit" and represents 1/60th of a CSS pixel.  It was
// originally proposed in 2002 as a standard unit of measure in Gecko.
// See https://bugzilla.mozilla.org/show_bug.cgi?id=177805 for more info.

#[inline(always)]
pub fn max_rect() -> Rect<Au> {
    Rect::new(Point2D::new(Au(i32::MIN / 2), Au(i32::MIN / 2)), Size2D::new(MAX_AU, MAX_AU))
}

/// A helper function to convert a point of `Au` pixels to a point of f32 units.
#[inline]
pub fn au_point_to_f32_point(point: &Point2D<Au>) -> Point2D<f32> {
    Point2D::new(point.x.to_f32_px(), point.y.to_f32_px())
}

/// A helper function to convert a point of f32 pixels to a point of `Au` units.
#[inline]
pub fn f32_point_to_au_point(point: &Point2D<f32>) -> Point2D<Au> {
    Point2D::new(Au::from_f32_px(point.x), Au::from_f32_px(point.y))
}

/// A helper function to convert a point of f32 pixels to a rect of `Au` units.
#[inline]
pub fn f32_size_to_au_size(size: &Size2D<f32>) -> Size2D<Au> {
    Size2D::new(Au::from_f32_px(size.width), Au::from_f32_px(size.height))
}

/// A helper function to convert a point of `Au` pixels to a rect of f32 units.
#[inline]
pub fn au_size_to_f32_size(size: &Size2D<Au>) -> Size2D<f32> {
    Size2D::new(size.width.to_f32_px(), size.height.to_f32_px())
}

/// A helper function to convert a rect of `f32` pixels to a rect of app units.
#[inline]
pub fn f32_rect_to_au_rect(rect: Rect<f32>) -> Rect<Au> {
    Rect::new(Point2D::new(Au::from_f32_px(rect.origin.x), Au::from_f32_px(rect.origin.y)),
              Size2D::new(Au::from_f32_px(rect.size.width), Au::from_f32_px(rect.size.height)))
}

/// A helper function to convert a rect of `Au` pixels to a rect of f32 units.
#[inline]
pub fn au_rect_to_f32_rect(rect: Rect<Au>) -> Rect<f32> {
    Rect::new(au_point_to_f32_point(&rect.origin), au_size_to_f32_size(&rect.size))
}

/// A helper function to convert a set of side offsets of `Au` pixels to one of f32 units.
#[inline]
pub fn au_side_offsets_to_f32_side_offsets(side_offsets: &SideOffsets2D<Au>)
                                           -> SideOffsets2D<f32> {
    SideOffsets2D::new(side_offsets.top.to_f32_px(),
                       side_offsets.right.to_f32_px(),
                       side_offsets.bottom.to_f32_px(),
                       side_offsets.left.to_f32_px())
}
