/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Selecting the default global allocator for Servo

#[global_allocator]
static ALLOC: Allocator = Allocator;

pub use crate::platform::*;

#[cfg(not(any(
    windows,
    target_os = "android",
    feature = "use-system-allocator",
    target_env = "ohos"
)))]
mod platform {
    use std::os::raw::c_void;

    pub use jemallocator::Jemalloc as Allocator;

    /// Get the size of a heap block.
    ///
    /// # Safety
    ///
    /// Passing a non-heap allocated pointer to this function results in undefined behavior.
    pub unsafe extern "C" fn usable_size(ptr: *const c_void) -> usize {
        jemallocator::usable_size(ptr)
    }

    /// Memory allocation APIs compatible with libc
    pub mod libc_compat {
        pub use jemalloc_sys::{free, malloc, realloc};
    }
}

#[cfg(all(
    not(windows),
    any(
        target_os = "android",
        feature = "use-system-allocator",
        target_env = "ohos"
    )
))]
mod platform {
    pub use std::alloc::System as Allocator;
    use std::os::raw::c_void;

    /// Get the size of a heap block.
    ///
    /// # Safety
    ///
    /// Passing a non-heap allocated pointer to this function results in undefined behavior.
    pub unsafe extern "C" fn usable_size(ptr: *const c_void) -> usize {
        #[cfg(target_vendor = "apple")]
        return libc::malloc_size(ptr);

        #[cfg(not(target_vendor = "apple"))]
        return libc::malloc_usable_size(ptr as *mut _);
    }

    pub mod libc_compat {
        pub use libc::{free, malloc, realloc};
    }
}

#[cfg(windows)]
mod platform {
    pub use std::alloc::System as Allocator;
    use std::os::raw::c_void;

    use winapi::um::heapapi::{GetProcessHeap, HeapSize, HeapValidate};

    /// Get the size of a heap block.
    ///
    /// # Safety
    ///
    /// Passing a non-heap allocated pointer to this function results in undefined behavior.
    pub unsafe extern "C" fn usable_size(mut ptr: *const c_void) -> usize {
        let heap = GetProcessHeap();

        if HeapValidate(heap, 0, ptr) == 0 {
            ptr = *(ptr as *const *const c_void).offset(-1);
        }

        HeapSize(heap, 0, ptr) as usize
    }
}
