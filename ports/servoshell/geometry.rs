/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::{Point2D, Size2D};
use servo::style_traits::DevicePixel;
use winit::dpi::{PhysicalPosition, PhysicalSize};

pub fn winit_size_to_euclid_size<T>(size: PhysicalSize<T>) -> Size2D<T, DevicePixel> {
    Size2D::new(size.width, size.height)
}

pub fn winit_position_to_euclid_point<T>(position: PhysicalPosition<T>) -> Point2D<T, DevicePixel> {
    Point2D::new(position.x, position.y)
}
