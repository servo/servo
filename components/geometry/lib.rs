/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::f32;

use app_units::{Au, MAX_AU, MIN_AU};
use euclid::default::{Point2D as UntypedPoint2D, Rect as UntypedRect, Size2D as UntypedSize2D};
use euclid::{Box2D, Length, Point2D, Scale, SideOffsets2D, Size2D, Vector2D};
use malloc_size_of_derive::MallocSizeOf;
use webrender_api::units::{
    DeviceIntRect, DeviceIntSize, DevicePixel, FramebufferPixel, LayoutPoint, LayoutRect,
    LayoutSize,
};

// Units for use with euclid::length and euclid::scale_factor.

pub type FramebufferUintLength = Length<u32, FramebufferPixel>;

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

pub type DeviceIndependentIntRect = Box2D<i32, DeviceIndependentPixel>;
pub type DeviceIndependentIntPoint = Point2D<i32, DeviceIndependentPixel>;
pub type DeviceIndependentIntSize = Size2D<i32, DeviceIndependentPixel>;
pub type DeviceIndependentIntLength = Length<i32, DeviceIndependentPixel>;
pub type DeviceIndependentIntSideOffsets = SideOffsets2D<i32, DeviceIndependentPixel>;
pub type DeviceIndependentIntVector2D = Vector2D<i32, DeviceIndependentPixel>;

pub type DeviceIndependentRect = Box2D<f32, DeviceIndependentPixel>;
pub type DeviceIndependentBox2D = Box2D<f32, DeviceIndependentPixel>;
pub type DeviceIndependentPoint = Point2D<f32, DeviceIndependentPixel>;
pub type DeviceIndependentVector2D = Vector2D<f32, DeviceIndependentPixel>;
pub type DeviceIndependentSize = Size2D<f32, DeviceIndependentPixel>;

// An Au is an "App Unit" and represents 1/60th of a CSS pixel.  It was
// originally proposed in 2002 as a standard unit of measure in Gecko.
// See https://bugzilla.mozilla.org/show_bug.cgi?id=177805 for more info.

pub trait MaxRect {
    fn max_rect() -> Self;
}

/// A helper function to convert a Device rect to CSS pixels.
pub fn convert_rect_to_css_pixel(
    rect: DeviceIntRect,
    scale: Scale<f32, DeviceIndependentPixel, DevicePixel>,
) -> DeviceIndependentIntRect {
    (rect.to_f32() / scale).round().to_i32()
}

/// A helper function to convert a Device size to CSS pixels.
pub fn convert_size_to_css_pixel(
    size: DeviceIntSize,
    scale: Scale<f32, DeviceIndependentPixel, DevicePixel>,
) -> DeviceIndependentIntSize {
    (size.to_f32() / scale).round().to_i32()
}

impl MaxRect for UntypedRect<Au> {
    #[inline]
    fn max_rect() -> Self {
        Self::new(
            UntypedPoint2D::new(MIN_AU / 2, MIN_AU / 2),
            UntypedSize2D::new(MAX_AU, MAX_AU),
        )
    }
}

impl MaxRect for LayoutRect {
    #[inline]
    fn max_rect() -> Self {
        Self::from_origin_and_size(
            LayoutPoint::new(f32::MIN / 2.0, f32::MIN / 2.0),
            LayoutSize::new(f32::MAX, f32::MAX),
        )
    }
}

/// A helper function to convert a rect of `f32` pixels to a rect of app units.
pub fn f32_rect_to_au_rect(rect: UntypedRect<f32>) -> UntypedRect<Au> {
    UntypedRect::new(
        Point2D::new(
            Au::from_f32_px(rect.origin.x),
            Au::from_f32_px(rect.origin.y),
        ),
        Size2D::new(
            Au::from_f32_px(rect.size.width),
            Au::from_f32_px(rect.size.height),
        ),
    )
}

/// A helper function to convert a rect of `Au` pixels to a rect of f32 units.
pub fn au_rect_to_f32_rect(rect: UntypedRect<Au>) -> UntypedRect<f32> {
    UntypedRect::new(
        Point2D::new(rect.origin.x.to_f32_px(), rect.origin.y.to_f32_px()),
        Size2D::new(rect.size.width.to_f32_px(), rect.size.height.to_f32_px()),
    )
}
