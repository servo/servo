/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Selecting the default global allocator for Servo, and exposing common
//! allocator introspection APIs for memory profiling.

use std::os::raw::c_void;

#[cfg(not(feature = "allocation-tracking"))]
#[global_allocator]
static ALLOC: Allocator = Allocator;

#[cfg(feature = "allocation-tracking")]
#[global_allocator]
static ALLOC: crate::tracking::AccountingAlloc<Allocator> =
    crate::tracking::AccountingAlloc::with_allocator(Allocator);

#[cfg(feature = "allocation-tracking")]
mod tracking;

pub fn is_tracking_unmeasured() -> bool {
    cfg!(feature = "allocation-tracking")
}

pub fn dump_unmeasured(_writer: impl std::io::Write) {
    #[cfg(feature = "allocation-tracking")]
    ALLOC.dump_unmeasured_allocations(_writer);
}

pub struct HeapReport {
    pub path: &'static str,
    pub size: Option<usize>,
}

pub use crate::platform::*;

type EnclosingSizeFn = unsafe extern "C" fn(*const c_void) -> usize;

/// # Safety
/// No restrictions. The passed pointer is never dereferenced.
/// This function is only marked unsafe because the MallocSizeOfOps APIs
/// requires an unsafe function pointer.
#[cfg(feature = "allocation-tracking")]
unsafe extern "C" fn enclosing_size_impl(ptr: *const c_void) -> usize {
    let (adjusted, size) = crate::ALLOC.enclosing_size(ptr);
    if size != 0 {
        crate::ALLOC.note_allocation(adjusted, size);
    }
    size
}

#[expect(non_upper_case_globals)]
#[cfg(feature = "allocation-tracking")]
pub static enclosing_size: Option<EnclosingSizeFn> = Some(crate::enclosing_size_impl);

#[expect(non_upper_case_globals)]
#[cfg(not(feature = "allocation-tracking"))]
pub static enclosing_size: Option<EnclosingSizeFn> = None;

#[cfg(not(any(windows, feature = "use-system-allocator", target_env = "ohos")))]
mod platform {
    use std::ffi::CString;
    use std::mem::size_of;
    use std::os::raw::c_void;
    use std::ptr::null_mut;

    use tikv_jemalloc_sys::mallctl;
    pub use tikv_jemallocator::Jemalloc as Allocator;

    pub fn heap_reports() -> Vec<crate::HeapReport> {
        vec![
            crate::HeapReport {
                path: "jemalloc-heap-allocated",
                size: jemalloc_stat("stats.allocated"),
            },
            crate::HeapReport {
                path: "jemalloc-heap-active",
                size: jemalloc_stat("stats.active"),
            },
            crate::HeapReport {
                path: "jemalloc-heap-mapped",
                size: jemalloc_stat("stats.mapped"),
            },
        ]
    }

    fn jemalloc_stat(value_name: &str) -> Option<usize> {
        // Before we request the measurement of interest, we first send an "epoch"
        // request. Without that jemalloc gives cached statistics(!) which can be
        // highly inaccurate.
        let epoch_c_name = CString::new("epoch").unwrap();
        let mut epoch: u64 = 0;
        let epoch_ptr = &mut epoch as *mut _ as *mut c_void;
        let mut epoch_len = size_of::<u64>();

        let value_c_name = CString::new(value_name).unwrap();
        let mut value: usize = 0;
        let value_ptr = &mut value as *mut _ as *mut c_void;
        let mut value_len = size_of::<usize>();

        // Using the same values for the `old` and `new` parameters is enough
        // to get the statistics updated.
        let rv = unsafe {
            mallctl(
                epoch_c_name.as_ptr(),
                epoch_ptr,
                &mut epoch_len,
                epoch_ptr,
                epoch_len,
            )
        };
        if rv != 0 {
            return None;
        }

        let rv = unsafe {
            mallctl(
                value_c_name.as_ptr(),
                value_ptr,
                &mut value_len,
                null_mut(),
                0,
            )
        };
        if rv != 0 {
            return None;
        }

        Some(value as usize)
    }

    /// Get the size of a heap block.
    ///
    /// # Safety
    ///
    /// Passing a non-heap allocated pointer to this function results in undefined behavior.
    pub unsafe extern "C" fn usable_size(ptr: *const c_void) -> usize {
        let size = unsafe { tikv_jemallocator::usable_size(ptr) };
        #[cfg(feature = "allocation-tracking")]
        crate::ALLOC.note_allocation(ptr, size);
        size
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
            let size = libc::malloc_size(ptr);
            #[cfg(feature = "allocation-tracking")]
            crate::ALLOC.note_allocation(ptr, size);
            size
        }

        #[cfg(not(target_vendor = "apple"))]
        unsafe {
            let size = libc::malloc_usable_size(ptr as *mut _);
            #[cfg(feature = "allocation-tracking")]
            crate::ALLOC.note_allocation(ptr, size);
            size
        }
    }

    pub mod libc_compat {
        pub use libc::{free, malloc, realloc};
    }

    pub fn heap_reports() -> Vec<crate::HeapReport> {
        Vec::new()
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

            let size = HeapSize(heap, 0, ptr) as usize;
            #[cfg(feature = "allocation-tracking")]
            crate::ALLOC.note_allocation(ptr, size);
            size
        }
    }

    pub fn heap_reports() -> Vec<crate::HeapReport> {
        Vec::new()
    }
}
