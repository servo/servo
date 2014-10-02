/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use geom::length::Length;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

use serialize::{Encodable, Encoder};
use std::default::Default;
use std::i32;
use std::num::{NumCast, Zero};
use std::fmt;

// Units for use with geom::length and geom::scale_factor.

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
#[deriving(Encodable)]
pub enum ViewportPx {}

/// One CSS "px" in the root coordinate system for the content document.
///
/// PagePx is equal to ViewportPx multiplied by a "viewport zoom" factor controlled by the user.
/// This is the mobile-style "pinch zoom" that enlarges content without reflowing it.  When the
/// viewport zoom is not equal to 1.0, then the layout viewport is no longer the same physical size
/// as the viewable area.
#[deriving(Encodable)]
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
#[deriving(Clone, PartialEq, PartialOrd, Eq, Ord, Zero)]
pub struct Au(pub i32);

impl Default for Au {
    #[inline]
    fn default() -> Au {
        Au(0)
    }
}

pub static MAX_AU: Au = Au(i32::MAX);

impl<E, S: Encoder<E>> Encodable<S, E> for Au {
    fn encode(&self, e: &mut S) -> Result<(), E> {
        e.emit_f64(to_frac_px(*self))
    }
}

impl fmt::Show for Au {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}px", to_frac_px(*self))
    }}

impl Add<Au,Au> for Au {
    #[inline]
    fn add(&self, other: &Au) -> Au {
        let Au(s) = *self;
        let Au(o) = *other;
        Au(s + o)
    }
}

impl Sub<Au,Au> for Au {
    #[inline]
    fn sub(&self, other: &Au) -> Au {
        let Au(s) = *self;
        let Au(o) = *other;
        Au(s - o)
    }

}

impl Mul<i32, Au> for Au {
    #[inline]
    fn mul(&self, other: &i32) -> Au {
        let Au(s) = *self;
        Au(s * *other)
    }
}

impl Div<i32, Au> for Au {
    #[inline]
    fn div(&self, other: &i32) -> Au {
        let Au(s) = *self;
        Au(s / *other)
    }
}

impl Rem<i32, Au> for Au {
    #[inline]
    fn rem(&self, other: &i32) -> Au {
        let Au(s) = *self;
        Au(s % *other)
    }
}

impl Neg<Au> for Au {
    #[inline]
    fn neg(&self) -> Au {
        let Au(s) = *self;
        Au(-s)
    }
}


impl NumCast for Au {
    #[inline]
    fn from<T:ToPrimitive>(n: T) -> Option<Au> {
        Some(Au(n.to_i32().unwrap()))
    }
}

impl ToPrimitive for Au {
    #[inline]
    fn to_i64(&self) -> Option<i64> {
        let Au(s) = *self;
        Some(s as i64)
    }

    #[inline]
    fn to_u64(&self) -> Option<u64> {
        let Au(s) = *self;
        Some(s as u64)
    }

    #[inline]
    fn to_f32(&self) -> Option<f32> {
        let Au(s) = *self;
        s.to_f32()
    }

    #[inline]
    fn to_f64(&self) -> Option<f64> {
        let Au(s) = *self;
        s.to_f64()
    }
}

impl Au {
    /// FIXME(pcwalton): Workaround for lack of cross crate inlining of newtype structs!
    #[inline]
    pub fn new(value: i32) -> Au {
        Au(value)
    }

    #[inline]
    pub fn scale_by(self, factor: f64) -> Au {
        let Au(s) = self;
        Au(((s as f64) * factor) as i32)
    }

    #[inline]
    pub fn from_px(px: int) -> Au {
        NumCast::from(px * 60).unwrap()
    }

    #[inline]
    pub fn from_page_px(px: Length<PagePx, f32>) -> Au {
        NumCast::from(px.get() * 60f32).unwrap()
    }

    #[inline]
    pub fn to_nearest_px(&self) -> int {
        let Au(s) = *self;
        ((s as f64) / 60f64).round() as int
    }

    #[inline]
    pub fn to_subpx(&self) -> f64 {
        let Au(s) = *self;
        (s as f64) / 60f64
    }

    #[inline]
    pub fn to_snapped(&self) -> Au {
        let Au(s) = *self;
        let res = s % 60i32;
        return if res >= 30i32 { return Au(s - res + 60i32) }
                       else { return Au(s - res) };
    }

    #[inline]
    pub fn from_frac32_px(px: f32) -> Au {
        Au((px * 60f32) as i32)
    }

    #[inline]
    pub fn from_pt(pt: f64) -> Au {
        from_frac_px(pt_to_px(pt))
    }

    #[inline]
    pub fn from_frac_px(px: f64) -> Au {
        Au((px * 60f64) as i32)
    }

    #[inline]
    pub fn min(x: Au, y: Au) -> Au {
        let Au(xi) = x;
        let Au(yi) = y;
        if xi < yi { x } else { y }
    }

    #[inline]
    pub fn max(x: Au, y: Au) -> Au {
        let Au(xi) = x;
        let Au(yi) = y;
        if xi > yi { x } else { y }
    }
}

// assumes 72 points per inch, and 96 px per inch
pub fn pt_to_px(pt: f64) -> f64 {
    pt / 72f64 * 96f64
}

// assumes 72 points per inch, and 96 px per inch
pub fn px_to_pt(px: f64) -> f64 {
    px / 96f64 * 72f64
}

pub fn from_frac_px(px: f64) -> Au {
    Au((px * 60f64) as i32)
}

pub fn from_px(px: int) -> Au {
    NumCast::from(px * 60).unwrap()
}

pub fn to_px(au: Au) -> int {
    let Au(a) = au;
    (a / 60) as int
}

pub fn to_frac_px(au: Au) -> f64 {
    let Au(a) = au;
    (a as f64) / 60f64
}

// assumes 72 points per inch, and 96 px per inch
pub fn from_pt(pt: f64) -> Au {
    from_px((pt / 72f64 * 96f64) as int)
}

// assumes 72 points per inch, and 96 px per inch
pub fn to_pt(au: Au) -> f64 {
    let Au(a) = au;
    (a as f64) / 60f64 * 72f64 / 96f64
}

/// Returns true if the rect contains the given point. Points on the top or left sides of the rect
/// are considered inside the rectangle, while points on the right or bottom sides of the rect are
/// not considered inside the rectangle.
pub fn rect_contains_point<T:PartialOrd + Add<T,T>>(rect: Rect<T>, point: Point2D<T>) -> bool {
    point.x >= rect.origin.x && point.x < rect.origin.x + rect.size.width &&
        point.y >= rect.origin.y && point.y < rect.origin.y + rect.size.height
}

/// A helper function to convert a rect of `f32` pixels to a rect of app units.
pub fn f32_rect_to_au_rect(rect: Rect<f32>) -> Rect<Au> {
    Rect(Point2D(Au::from_frac32_px(rect.origin.x), Au::from_frac32_px(rect.origin.y)),
         Size2D(Au::from_frac32_px(rect.size.width), Au::from_frac32_px(rect.size.height)))
}

