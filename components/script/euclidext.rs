/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::default::{Rect, Size2D};

pub trait Size2DExt {
    fn to_u64(&self) -> Size2D<u64>;
}

impl Size2DExt for Size2D<f32> {
    fn to_u64(&self) -> Size2D<u64> {
        self.cast()
    }
}

impl Size2DExt for Size2D<f64> {
    fn to_u64(&self) -> Size2D<u64> {
        self.cast()
    }
}

impl Size2DExt for Size2D<u32> {
    fn to_u64(&self) -> Size2D<u64> {
        self.cast()
    }
}

pub trait RectExt {
    fn to_u64(&self) -> Rect<u64>;
}

impl RectExt for Rect<f64> {
    fn to_u64(&self) -> Rect<u64> {
        self.cast()
    }
}

impl RectExt for Rect<u32> {
    fn to_u64(&self) -> Rect<u64> {
        self.cast()
    }
}
