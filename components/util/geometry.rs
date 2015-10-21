/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::{Au, MAX_AU};
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::size::Size2D;
use std::i32;
use std::ops::Add;

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
#[derive(Debug, Copy, Clone)]
pub enum ScreenPx {}

/// One CSS "px" in the coordinate system of the "initial viewport":
/// http://www.w3.org/TR/css-device-adapt/#initial-viewport
///
/// ViewportPx is equal to ScreenPx times a "page zoom" factor controlled by the user.  This is
/// the desktop-style "full page" zoom that enlarges content but then reflows the layout viewport
/// so it still exactly fits the visible area.
///
/// At the default zoom level of 100%, one PagePx is equal to one ScreenPx.  However, if the
/// document is zoomed in or out then this scale may be larger or smaller.
#[derive(RustcEncodable, Debug, Copy, Clone)]
pub enum ViewportPx {}

/// One CSS "px" in the root coordinate system for the content document.
///
/// PagePx is equal to ViewportPx multiplied by a "viewport zoom" factor controlled by the user.
/// This is the mobile-style "pinch zoom" that enlarges content without reflowing it.  When the
/// viewport zoom is not equal to 1.0, then the layout viewport is no longer the same physical size
/// as the viewable area.
#[derive(RustcEncodable, Debug, Copy, Clone)]
pub enum PagePx {}

// In summary, the hierarchy of pixel units and the factors to convert from one to the next:
//
// DevicePixel
//   / hidpi_ratio => ScreenPx
//     / desktop_zoom => ViewportPx
//       / pinch_zoom => PagePx

// An Au is an "App Unit" and represents 1/60th of a CSS pixel.  It was
// originally proposed in 2002 as a standard unit of measure in Gecko.
// See https://bugzilla.mozilla.org/show_bug.cgi?id=177805 for more info.

pub static ZERO_POINT: Point2D<Au> = Point2D {
    x: Au(0),
    y: Au(0),
};

pub static ZERO_RECT: Rect<Au> = Rect {
    origin: Point2D {
        x: Au(0),
        y: Au(0),
    },
    size: Size2D {
        width: Au(0),
        height: Au(0),
    }
};

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

/// Returns true if the rect contains the given point. Points on the top or left sides of the rect
/// are considered inside the rectangle, while points on the right or bottom sides of the rect are
/// not considered inside the rectangle.
pub fn rect_contains_point<T: PartialOrd + Add<T, Output=T>>(rect: Rect<T>, point: Point2D<T>) -> bool {
    point.x >= rect.origin.x && point.x < rect.origin.x + rect.size.width &&
        point.y >= rect.origin.y && point.y < rect.origin.y + rect.size.height
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
