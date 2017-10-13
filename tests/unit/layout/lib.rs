/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(raw)]

extern crate layout;
#[macro_use] extern crate size_of_test;

#[cfg(all(test, target_pointer_width = "64"))] mod size_of;

use std::mem;
use std::ptr;
use std::raw;

#[test]
fn test_trait_object_layout() {
    assert_eq!(mem::size_of::<raw::TraitObject>(), mem::size_of::<layout::TraitObject>());
    let null: *mut () = ptr::null_mut();
    let a = raw::TraitObject {
        data: null,
        vtable: null,
    };
    let b = layout::TraitObject {
        data: null,
        vtable: null,
    };

    fn offset<T, U>(struct_: &T, field: &U) -> usize {
        let addr_struct = struct_ as *const T as usize;
        let addr_field = field as *const U as usize;
        addr_field - addr_struct
    }

    assert_eq!(offset(&a, &a.data), offset(&b, &b.data));
    assert_eq!(offset(&a, &a.vtable), offset(&b, &b.vtable));
}
