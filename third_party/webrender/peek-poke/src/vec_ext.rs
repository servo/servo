// Copyright 2019 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::vec::Vec;

pub trait VecExt  {
    type Item;
    unsafe fn set_end_ptr(&mut self, end: *const Self::Item);
}

impl<T> VecExt for Vec<T> {
    type Item = T;
    unsafe fn set_end_ptr(&mut self, end: *const T) {
        assert!(end as usize >= self.as_ptr() as usize);
        let new_len = end as usize - self.as_ptr() as usize;
        assert!(new_len <= self.capacity());
        self.set_len(new_len);
    }
}
