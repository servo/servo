/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::os::raw::{c_long, c_void};
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use freetype_sys::{
    FT_Add_Default_Modules, FT_Done_Library, FT_Library, FT_Memory, FT_MemoryRec, FT_New_Library,
};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use parking_lot::Mutex;
use servo_allocator::libc_compat::{free, malloc, realloc};
use servo_allocator::usable_size;

static FREETYPE_MEMORY_USAGE: AtomicUsize = AtomicUsize::new(0);
static FREETYPE_LIBRARY_HANDLE: OnceLock<Mutex<FreeTypeLibraryHandle>> = OnceLock::new();

extern "C" fn ft_alloc(_: FT_Memory, req_size: c_long) -> *mut c_void {
    unsafe {
        let pointer = malloc(req_size as usize);
        FREETYPE_MEMORY_USAGE.fetch_add(usable_size(pointer), Ordering::Relaxed);
        pointer
    }
}

extern "C" fn ft_free(_: FT_Memory, pointer: *mut c_void) {
    unsafe {
        FREETYPE_MEMORY_USAGE.fetch_sub(usable_size(pointer), Ordering::Relaxed);
        free(pointer as *mut _);
    }
}

extern "C" fn ft_realloc(
    _: FT_Memory,
    _old_size: c_long,
    new_req_size: c_long,
    old_pointer: *mut c_void,
) -> *mut c_void {
    unsafe {
        FREETYPE_MEMORY_USAGE.fetch_sub(usable_size(old_pointer), Ordering::Relaxed);
        let new_pointer = realloc(old_pointer, new_req_size as usize);
        FREETYPE_MEMORY_USAGE.fetch_add(usable_size(new_pointer), Ordering::Relaxed);
        new_pointer
    }
}

/// A FreeType library handle to be used for creating and dropping FreeType font faces.
/// It is very important that this handle lives as long as the faces themselves, which
/// is why only one of these is created for the entire execution of Servo and never
/// dropped during execution.
#[derive(Clone, Debug)]
pub(crate) struct FreeTypeLibraryHandle {
    pub freetype_library: FT_Library,
    freetype_memory: FT_Memory,
}

unsafe impl Sync for FreeTypeLibraryHandle {}
unsafe impl Send for FreeTypeLibraryHandle {}

impl Drop for FreeTypeLibraryHandle {
    #[allow(unused)]
    fn drop(&mut self) {
        assert!(!self.freetype_library.is_null());
        unsafe {
            FT_Done_Library(self.freetype_library);
            Box::from_raw(self.freetype_memory);
        }
    }
}

impl MallocSizeOf for FreeTypeLibraryHandle {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        unsafe {
            FREETYPE_MEMORY_USAGE.load(Ordering::Relaxed) +
                ops.malloc_size_of(self.freetype_library as *const _) +
                ops.malloc_size_of(self.freetype_memory as *const _)
        }
    }
}

impl FreeTypeLibraryHandle {
    /// Get the shared FreeType library handle. This is protected by a mutex because according to
    /// the FreeType documentation:
    ///
    /// > [Since 2.5.6] In multi-threaded applications it is easiest to use one FT_Library object per
    /// > thread. In case this is too cumbersome, a single FT_Library object across threads is possible
    /// > also, as long as a mutex lock is used around FT_New_Face and FT_Done_Face.
    ///
    /// See <https://freetype.org/freetype2/docs/reference/ft2-library_setup.html>.
    pub(crate) fn get() -> &'static Mutex<FreeTypeLibraryHandle> {
        FREETYPE_LIBRARY_HANDLE.get_or_init(|| {
            let freetype_memory = Box::into_raw(Box::new(FT_MemoryRec {
                user: ptr::null_mut(),
                alloc: ft_alloc,
                free: ft_free,
                realloc: ft_realloc,
            }));
            unsafe {
                let mut freetype_library: FT_Library = ptr::null_mut();
                let result = FT_New_Library(freetype_memory, &mut freetype_library);
                if 0 != result {
                    panic!("Unable to initialize FreeType library");
                }
                FT_Add_Default_Modules(freetype_library);
                Mutex::new(FreeTypeLibraryHandle {
                    freetype_library,
                    freetype_memory,
                })
            }
        })
    }
}
