/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::ToCss;

use euclid::length::Length;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::size::Size2D;
use euclid::num::Zero;

use std::default::Default;
use std::i32;
use std::fmt;
use std::ops::{Add, Sub, Neg, Mul, Div, Rem};

use rustc_serialize::{Encoder, Encodable};

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
//
// FIXME: Implement Au using Length and ScaleFactor instead of a custom type.
#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
pub struct Au(pub i32);

impl Default for Au {
    #[inline]
    fn default() -> Au {
        Au(0)
    }
}

impl Zero for Au {
    #[inline]
    fn zero() -> Au {
        Au(0)
    }
}

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

pub const MIN_AU: Au = Au(i32::MIN);
pub const MAX_AU: Au = Au(i32::MAX);

impl Encodable for Au {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_f64(self.to_f64_px())
    }
}

impl fmt::Debug for Au {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}px", self.to_f64_px())
    }
}

impl ToCss for Au {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        write!(dest, "{}px", self.to_f64_px())
    }
}

impl Add for Au {
    type Output = Au;

    #[inline]
    fn add(self, other: Au) -> Au {
        Au(self.0.wrapping_add(other.0))
    }
}

impl Sub for Au {
    type Output = Au;

    #[inline]
    fn sub(self, other: Au) -> Au {
        Au(self.0.wrapping_sub(other.0))
    }

}

impl Mul<i32> for Au {
    type Output = Au;

    #[inline]
    fn mul(self, other: i32) -> Au {
        Au(self.0.wrapping_mul(other))
    }
}

impl Div<i32> for Au {
    type Output = Au;

    #[inline]
    fn div(self, other: i32) -> Au {
        Au(self.0 / other)
    }
}

impl Rem<i32> for Au {
    type Output = Au;

    #[inline]
    fn rem(self, other: i32) -> Au {
        Au(self.0 % other)
    }
}

impl Neg for Au {
    type Output = Au;

    #[inline]
    fn neg(self) -> Au {
        Au(-self.0)
    }
}

impl Au {
    /// FIXME(pcwalton): Workaround for lack of cross crate inlining of newtype structs!
    #[inline]
    pub fn new(value: i32) -> Au {
        Au(value)
    }

    #[inline]
    pub fn scale_by(self, factor: f32) -> Au {
        Au(((self.0 as f32) * factor) as i32)
    }

    #[inline]
    pub fn from_px(px: i32) -> Au {
        Au((px * 60) as i32)
    }

    #[inline]
    pub fn from_page_px(px: Length<PagePx, f32>) -> Au {
        Au((px.get() * 60f32) as i32)
    }

    /// Rounds this app unit down to the pixel towards zero and returns it.
    #[inline]
    pub fn to_px(self) -> i32 {
        self.0 / 60
    }

    /// Rounds this app unit down to the previous (left or top) pixel and returns it.
    #[inline]
    pub fn to_prev_px(self) -> i32 {
        ((self.0 as f64) / 60f64).floor() as i32
    }

    /// Rounds this app unit up to the next (right or bottom) pixel and returns it.
    #[inline]
    pub fn to_next_px(self) -> i32 {
        ((self.0 as f64) / 60f64).ceil() as i32
    }

    #[inline]
    pub fn to_nearest_px(self) -> i32 {
        ((self.0 as f64) / 60f64).round() as i32
    }

    #[inline]
    pub fn to_f32_px(self) -> f32 {
        (self.0 as f32) / 60f32
    }

    #[inline]
    pub fn to_f64_px(self) -> f64 {
        (self.0 as f64) / 60f64
    }

    #[inline]
    pub fn to_snapped(self) -> Au {
        let res = self.0 % 60i32;
        return if res >= 30i32 { return Au(self.0 - res + 60i32) }
                       else { return Au(self.0 - res) };
    }

    #[inline]
    pub fn from_f32_px(px: f32) -> Au {
        Au((px * 60f32) as i32)
    }

    #[inline]
    pub fn from_pt(pt: f64) -> Au {
        Au::from_f64_px(pt_to_px(pt))
    }

    #[inline]
    pub fn from_f64_px(px: f64) -> Au {
        Au((px * 60.) as i32)
    }
}

// assumes 72 points per inch, and 96 px per inch
pub fn pt_to_px(pt: f64) -> f64 {
    pt / 72. * 96.
}

// assumes 72 points per inch, and 96 px per inch
pub fn px_to_pt(px: f64) -> f64 {
    px / 96. * 72.
}

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
