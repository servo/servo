// Copyright 2019 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub trait AsEndMutPtr<T> {
    fn as_end_mut_ptr(self) -> *mut T;
}

impl<'a> AsEndMutPtr<u8> for &'a mut [u8] {
    fn as_end_mut_ptr(self) -> *mut u8 {
        unsafe { self.as_mut_ptr().add(self.len()) }
    }
}
