// Copyright 2019 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{Peek, Poke};
use euclid::{Point2D, Rect, SideOffsets2D, Size2D, Transform3D, Vector2D};

unsafe impl<T: Poke, U> Poke for Point2D<T, U> {
    #[inline(always)]
    fn max_size() -> usize {
        2 * T::max_size()
    }
    #[inline(always)]
    unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
        let bytes = self.x.poke_into(bytes);
        let bytes = self.y.poke_into(bytes);
        bytes
    }
}
impl<T: Peek, U> Peek for Point2D<T, U> {
    #[inline(always)]
    unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
        let bytes = T::peek_from(bytes, &mut (*output).x);
        let bytes = T::peek_from(bytes, &mut (*output).y);
        bytes
    }
}

unsafe impl<T: Poke, U> Poke for Rect<T, U> {
    #[inline(always)]
    fn max_size() -> usize {
        Point2D::<T, U>::max_size() + Size2D::<T, U>::max_size()
    }
    #[inline(always)]
    unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
        let bytes = self.origin.poke_into(bytes);
        let bytes = self.size.poke_into(bytes);
        bytes
    }
}
impl<T: Peek, U> Peek for Rect<T, U> {
    #[inline(always)]
    unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
        let bytes = Point2D::<T, U>::peek_from(bytes, &mut (*output).origin);
        let bytes = Size2D::<T, U>::peek_from(bytes, &mut (*output).size);
        bytes
    }
}

unsafe impl<T: Poke, U> Poke for SideOffsets2D<T, U> {
    #[inline(always)]
    fn max_size() -> usize {
        4 * T::max_size()
    }
    #[inline(always)]
    unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
        let bytes = self.top.poke_into(bytes);
        let bytes = self.right.poke_into(bytes);
        let bytes = self.bottom.poke_into(bytes);
        let bytes = self.left.poke_into(bytes);
        bytes
    }
}
impl<T: Peek, U> Peek for SideOffsets2D<T, U> {
    #[inline(always)]
    unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
        let bytes = T::peek_from(bytes, &mut (*output).top);
        let bytes = T::peek_from(bytes, &mut (*output).right);
        let bytes = T::peek_from(bytes, &mut (*output).bottom);
        let bytes = T::peek_from(bytes, &mut (*output).left);
        bytes
    }
}

unsafe impl<T: Poke, U> Poke for Size2D<T, U> {
    #[inline(always)]
    fn max_size() -> usize {
        2 * T::max_size()
    }
    #[inline(always)]
    unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
        let bytes = self.width.poke_into(bytes);
        let bytes = self.height.poke_into(bytes);
        bytes
    }
}
impl<T: Peek, U> Peek for Size2D<T, U> {
    #[inline(always)]
    unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
        let bytes = T::peek_from(bytes, &mut (*output).width);
        let bytes = T::peek_from(bytes, &mut (*output).height);
        bytes
    }
}

unsafe impl<T: Poke, S, D> Poke for Transform3D<T, S, D> {
    #[inline(always)]
    fn max_size() -> usize {
        16 * T::max_size()
    }
    #[inline(always)]
    unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
        let bytes = self.m11.poke_into(bytes);
        let bytes = self.m12.poke_into(bytes);
        let bytes = self.m13.poke_into(bytes);
        let bytes = self.m14.poke_into(bytes);
        let bytes = self.m21.poke_into(bytes);
        let bytes = self.m22.poke_into(bytes);
        let bytes = self.m23.poke_into(bytes);
        let bytes = self.m24.poke_into(bytes);
        let bytes = self.m31.poke_into(bytes);
        let bytes = self.m32.poke_into(bytes);
        let bytes = self.m33.poke_into(bytes);
        let bytes = self.m34.poke_into(bytes);
        let bytes = self.m41.poke_into(bytes);
        let bytes = self.m42.poke_into(bytes);
        let bytes = self.m43.poke_into(bytes);
        let bytes = self.m44.poke_into(bytes);
        bytes
    }
}
impl<T: Peek, S, D> Peek for Transform3D<T, S, D> {
    #[inline(always)]
    unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
        let bytes = T::peek_from(bytes, &mut (*output).m11);
        let bytes = T::peek_from(bytes, &mut (*output).m12);
        let bytes = T::peek_from(bytes, &mut (*output).m13);
        let bytes = T::peek_from(bytes, &mut (*output).m14);
        let bytes = T::peek_from(bytes, &mut (*output).m21);
        let bytes = T::peek_from(bytes, &mut (*output).m22);
        let bytes = T::peek_from(bytes, &mut (*output).m23);
        let bytes = T::peek_from(bytes, &mut (*output).m24);
        let bytes = T::peek_from(bytes, &mut (*output).m31);
        let bytes = T::peek_from(bytes, &mut (*output).m32);
        let bytes = T::peek_from(bytes, &mut (*output).m33);
        let bytes = T::peek_from(bytes, &mut (*output).m34);
        let bytes = T::peek_from(bytes, &mut (*output).m41);
        let bytes = T::peek_from(bytes, &mut (*output).m42);
        let bytes = T::peek_from(bytes, &mut (*output).m43);
        let bytes = T::peek_from(bytes, &mut (*output).m44);
        bytes
    }
}

unsafe impl<T: Poke, U> Poke for Vector2D<T, U> {
    #[inline(always)]
    fn max_size() -> usize {
        2 * T::max_size()
    }
    #[inline(always)]
    unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
        let bytes = self.x.poke_into(bytes);
        let bytes = self.y.poke_into(bytes);
        bytes
    }
}
impl<T: Peek, U> Peek for Vector2D<T, U> {
    #[inline(always)]
    unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
        let bytes = T::peek_from(bytes, &mut (*output).x);
        let bytes = T::peek_from(bytes, &mut (*output).y);
        bytes
    }
}
