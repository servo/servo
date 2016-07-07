/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::{Au, MAX_AU};
use euclid::point::Point2D;
use euclid::rect::Rect;
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

pub static MAX_RECT: Rect<Au> = Rect {
    origin: Point2D {
        x: Au(i32::MIN / 2),
        y: Au(i32::MIN / 2),
    },
    size: Size2D {
        width: MAX_AU,
        height: MAX_AU,
    }
};

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

pub trait ExpandToPixelBoundaries {
    fn expand_to_px_boundaries(&self) -> Self;
}

impl ExpandToPixelBoundaries for Rect<Au> {
    fn expand_to_px_boundaries(&self) -> Rect<Au> {
        let bottom_right = self.bottom_right();
        let bottom_right = Point2D::new(Au::from_px(bottom_right.x.ceil_to_px()),
                                       Au::from_px(bottom_right.y.ceil_to_px()));
        let new_origin = Point2D::new(Au::from_px(self.origin.x.to_px()),
                                      Au::from_px(self.origin.y.to_px()));
        Rect::new(new_origin,
                  Size2D::new(bottom_right.x - new_origin.x,
                              bottom_right.y - new_origin.y))
    }
}
