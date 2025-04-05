/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Selecting the default global allocator for Servo

#[global_allocator]
static ALLOC: Allocator = Allocator;

pub use crate::platform::*;

#[cfg(not(any(windows, feature = "use-system-allocator", target_env = "ohos")))]
mod platform {
    use std::os::raw::c_void;

    pub use tikv_jemallocator::Jemalloc as Allocator;

    /// Get the size of a heap block.
    ///
    /// # Safety
    ///
    /// Passing a non-heap allocated pointer to this function results in undefined behavior.
    pub unsafe extern "C" fn usable_size(ptr: *const c_void) -> usize {
        unsafe { tikv_jemallocator::usable_size(ptr) }
    }

    /// Memory allocation APIs compatible with libc
    pub mod libc_compat {
        pub use tikv_jemalloc_sys::{free, malloc, realloc};
    }
}

#[cfg(all(
    not(windows),
    any(feature = "use-system-allocator", target_env = "ohos")
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
        unsafe {
            return libc::malloc_size(ptr);
        }

        #[cfg(not(target_vendor = "apple"))]
        unsafe {
            return libc::malloc_usable_size(ptr as *mut _);
        }
    }

    pub mod libc_compat {
        pub use libc::{free, malloc, realloc};
    }
}

#[cfg(windows)]
mod platform {
    pub use std::alloc::System as Allocator;
    use std::os::raw::c_void;

    use windows_sys::Win32::Foundation::FALSE;
    use windows_sys::Win32::System::Memory::{GetProcessHeap, HeapSize, HeapValidate};

    /// Get the size of a heap block.
    ///
    /// # Safety
    ///
    /// Passing a non-heap allocated pointer to this function results in undefined behavior.
    pub unsafe extern "C" fn usable_size(mut ptr: *const c_void) -> usize {
        unsafe {
            let heap = GetProcessHeap();

            if HeapValidate(heap, 0, ptr) == FALSE {
                ptr = *(ptr as *const *const c_void).offset(-1)
            }

            HeapSize(heap, 0, ptr) as usize
        }
    }
}
