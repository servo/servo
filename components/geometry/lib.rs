/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate app_units;
extern crate euclid;
extern crate malloc_size_of;
#[macro_use] extern crate malloc_size_of_derive;
extern crate webrender_api;

use app_units::{Au, MAX_AU, MIN_AU};
use euclid::{Point2D, Rect, Size2D};
use std::f32;
use webrender_api::{LayoutPoint, LayoutRect, LayoutSize};

// Units for use with euclid::length and euclid::scale_factor.

/// A normalized "pixel" at the default resolution for the display.
///
/// Like the CSS "px" unit, the exact physical size of this unit may vary between devices, but it
/// should approximate a device-independent reference length.  This unit corresponds to Android's
/// "density-independent pixel" (dip), Mac OS X's "point", and Windows "device-independent pixel."
///
/// The relationship between DevicePixel and DeviceIndependentPixel is defined by the OS.  On most low-dpi
/// screens, one DeviceIndependentPixel is equal to one DevicePixel.  But on high-density screens it can be
/// some larger number.  For example, by default on Apple "retina" displays, one DeviceIndependentPixel equals
/// two DevicePixels.  On Android "MDPI" displays, one DeviceIndependentPixel equals 1.5 device pixels.
///
/// The ratio between DeviceIndependentPixel and DevicePixel for a given display be found by calling
/// `servo::windowing::WindowMethods::hidpi_factor`.
#[derive(Clone, Copy, Debug, MallocSizeOf)]
pub enum DeviceIndependentPixel {}

// An Au is an "App Unit" and represents 1/60th of a CSS pixel.  It was
// originally proposed in 2002 as a standard unit of measure in Gecko.
// See https://bugzilla.mozilla.org/show_bug.cgi?id=177805 for more info.

pub trait MaxRect {
    #[inline(always)]
    fn max_rect() -> Self;
}

impl MaxRect for Rect<Au> {
    fn max_rect() -> Rect<Au> {
        Rect::new(
            Point2D::new(MIN_AU / 2, MIN_AU / 2),
            Size2D::new(MAX_AU, MAX_AU)
        )
    }
}

impl MaxRect for LayoutRect {
    fn max_rect() -> LayoutRect {
        LayoutRect::new(
            LayoutPoint::new(f32::MIN / 2.0, f32::MIN / 2.0),
            LayoutSize::new(f32::MAX, f32::MAX),
        )
    }
}

/// A helper function to convert a rect of `f32` pixels to a rect of app units.
pub fn f32_rect_to_au_rect(rect: Rect<f32>) -> Rect<Au> {
    Rect::new(Point2D::new(Au::from_f32_px(rect.origin.x), Au::from_f32_px(rect.origin.y)),
              Size2D::new(Au::from_f32_px(rect.size.width), Au::from_f32_px(rect.size.height)))
}

/// A helper function to convert a rect of `Au` pixels to a rect of f32 units.
pub fn au_rect_to_f32_rect(rect: Rect<Au>) -> Rect<f32> {
    Rect::new(Point2D::new(rect.origin.x.to_f32_px(), rect.origin.y.to_f32_px()),
              Size2D::new(rect.size.width.to_f32_px(), rect.size.height.to_f32_px()))
}
