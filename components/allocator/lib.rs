/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Selecting the default global allocator for Servo

#![cfg_attr(all(feature = "unstable", windows), feature(alloc_system, allocator_api))]
#![cfg_attr(feature = "unstable", feature(global_allocator))]

#[cfg(feature = "unstable")]
#[global_allocator]
static ALLOC: Allocator = Allocator;

pub use platform::*;


#[cfg(all(feature = "unstable", not(windows)))]
mod platform {
    extern crate jemallocator;

    pub use self::jemallocator::Jemalloc as Allocator;
    use std::os::raw::c_void;

    /// Get the size of a heap block.
    pub unsafe extern "C" fn usable_size(ptr: *const c_void) -> usize {
        jemallocator::usable_size(ptr)
    }

    /// Memory allocation APIs compatible with libc
    pub mod libc_compat {
        pub use super::jemallocator::ffi::{malloc, realloc, free};
    }
}

#[cfg(all(feature = "unstable", windows))]
mod platform {
    extern crate alloc_system;
    extern crate kernel32;

    pub use self::alloc_system::System as Allocator;
    use self::kernel32::{GetProcessHeap, HeapSize, HeapValidate};
    use std::os::raw::c_void;

    /// Get the size of a heap block.
    pub unsafe extern "C" fn usable_size(mut ptr: *const c_void) -> usize {
        let heap = GetProcessHeap();

        if HeapValidate(heap, 0, ptr) == 0 {
            ptr = *(ptr as *const *const c_void).offset(-1);
        }

        HeapSize(heap, 0, ptr) as usize
    }
}

#[cfg(not(feature = "unstable"))]
mod platform {
    use std::os::raw::c_void;

    /// Without `#[global_allocator]` we cannot be certain of what allocator is used
    /// or how it is linked. We therefore disable memory reporting. (Return zero.)
    pub unsafe extern "C" fn usable_size(_ptr: *const c_void) -> usize {
        0
    }

    /// Memory allocation APIs compatible with libc
    pub mod libc_compat {
        extern crate libc;
        pub use self::libc::{malloc, realloc, free};
    }
}
