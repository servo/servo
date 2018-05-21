/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Selecting the default global allocator for Servo

#![cfg_attr(feature = "unstable", feature(global_allocator, allocator_api, alloc_system))]

#[cfg(feature = "unstable")]
#[global_allocator]
static ALLOC: Allocator = Allocator;

pub use platform::*;


#[cfg(all(feature = "unstable", not(windows)))]
mod platform {
    extern crate jemalloc_sys as ffi;

    use std::alloc::{GlobalAlloc, Layout, Opaque, System};
    use std::os::raw::{c_int, c_void};

    /// Get the size of a heap block.
    pub unsafe extern "C" fn usable_size(ptr: *const c_void) -> usize {
        ffi::malloc_usable_size(ptr as *const _)
    }

    /// Memory allocation APIs compatible with libc
    pub mod libc_compat {
        pub use super::ffi::{malloc, realloc, free};
    }

    pub struct Allocator;

    // The minimum alignment guaranteed by the architecture. This value is used to
    // add fast paths for low alignment values.
    #[cfg(all(any(target_arch = "arm",
                  target_arch = "mips",
                  target_arch = "mipsel",
                  target_arch = "powerpc")))]
    const MIN_ALIGN: usize = 8;
    #[cfg(all(any(target_arch = "x86",
                  target_arch = "x86_64",
                  target_arch = "aarch64",
                  target_arch = "powerpc64",
                  target_arch = "powerpc64le",
                  target_arch = "mips64",
                  target_arch = "s390x",
                  target_arch = "sparc64")))]
    const MIN_ALIGN: usize = 16;

    fn layout_to_flags(align: usize, size: usize) -> c_int {
        // If our alignment is less than the minimum alignment they we may not
        // have to pass special flags asking for a higher alignment. If the
        // alignment is greater than the size, however, then this hits a sort of odd
        // case where we still need to ask for a custom alignment. See #25 for more
        // info.
        if align <= MIN_ALIGN && align <= size {
            0
        } else {
            // Equivalent to the MALLOCX_ALIGN(a) macro.
            align.trailing_zeros() as _
        }
    }

    unsafe impl GlobalAlloc for Allocator {
        #[inline]
        unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
            let flags = layout_to_flags(layout.align(), layout.size());
            ffi::mallocx(layout.size(), flags)  as *mut Opaque
        }

        #[inline]
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut Opaque {
            if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() {
                ffi::calloc(1, layout.size()) as *mut Opaque
            } else {
                let flags = layout_to_flags(layout.align(), layout.size()) | ffi::MALLOCX_ZERO;
                ffi::mallocx(layout.size(), flags) as *mut Opaque
            }
        }

        #[inline]
        unsafe fn dealloc(&self, ptr: *mut Opaque, layout: Layout) {
            let flags = layout_to_flags(layout.align(), layout.size());
            ffi::sdallocx(ptr as *mut _, layout.size(), flags)
        }

        #[inline]
        unsafe fn realloc(&self,
                          ptr: *mut Opaque,
                          layout: Layout,
                          new_size: usize) -> *mut Opaque {
            let flags = layout_to_flags(layout.align(), new_size);
            ffi::rallocx(ptr as *mut _, new_size, flags) as *mut Opaque
        }

        #[inline]
        fn oom(&self) -> ! {
            System.oom()
        }
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
