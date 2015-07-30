/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use freetype::freetype::FTErrorMethods;
use freetype::freetype::FT_Add_Default_Modules;
use freetype::freetype::FT_Done_Library;
use freetype::freetype::FT_Library;
use freetype::freetype::FT_Memory;
use freetype::freetype::FT_New_Library;
use freetype::freetype::struct_FT_MemoryRec_;

use std::ptr;
use std::rc::Rc;
use std::rt::heap;
use util::mem::{HeapSizeOf, heap_size_of};

use libc::{c_void, c_long};

// We pass a |User| struct -- via an opaque |void*| -- to FreeType each time a new instance is
// created. FreeType passes it back to the ft_alloc/ft_realloc/ft_free callbacks. We use it to
// record the memory usage of each FreeType instance.
struct User {
    size: usize,
}

// FreeType doesn't require any particular alignment for allocations.
const FT_ALIGNMENT: usize = 1;

extern fn ft_alloc(mem: FT_Memory, req_size: c_long) -> *mut c_void {
    unsafe {
        let ptr = heap::allocate(req_size as usize, FT_ALIGNMENT) as *mut c_void;
        let actual_size = heap_size_of(ptr);

        let user = (*mem).user as *mut User;
        (*user).size += actual_size;

        ptr
    }
}

extern fn ft_free(mem: FT_Memory, ptr: *mut c_void) {
    unsafe {
        let actual_size = heap_size_of(ptr);

        let user = (*mem).user as *mut User;
        (*user).size -= actual_size;

        heap::deallocate(ptr as *mut u8, actual_size, FT_ALIGNMENT);
    }
}

extern fn ft_realloc(mem: FT_Memory, _cur_size: c_long, new_req_size: c_long,
                     old_ptr: *mut c_void) -> *mut c_void {
    unsafe {
        let old_actual_size = heap_size_of(old_ptr);
        let new_ptr = heap::reallocate(old_ptr as *mut u8, old_actual_size,
                                       new_req_size as usize, FT_ALIGNMENT) as *mut c_void;
        let new_actual_size = heap_size_of(new_ptr);

        let user = (*mem).user as *mut User;
        (*user).size += new_actual_size - old_actual_size;

        new_ptr
    }
}

// A |*mut User| field in a struct triggers a "use of `#[derive]` with a raw pointer" warning from
// rustc. But using a typedef avoids this, so...
pub type UserPtr = *mut User;

// WARNING: We need to be careful how we use this struct. See the comment about Rc<> in
// FontContextHandle.
#[derive(Clone)]
pub struct FreeTypeLibraryHandle {
    pub ctx: FT_Library,
    mem: FT_Memory,
    user: UserPtr,
}

impl Drop for FreeTypeLibraryHandle {
    fn drop(&mut self) {
        assert!(!self.ctx.is_null());
        unsafe {
            FT_Done_Library(self.ctx);
            Box::from_raw(self.mem);
            Box::from_raw(self.user);
        }
    }
}

impl HeapSizeOf for FreeTypeLibraryHandle {
    fn heap_size_of_children(&self) -> usize {
        let ft_size = unsafe { (*self.user).size };
        ft_size +
            heap_size_of(self.ctx as *const c_void) +
            heap_size_of(self.mem as *const c_void) +
            heap_size_of(self.user as *const c_void)
    }
}

#[derive(Clone, HeapSizeOf)]
pub struct FontContextHandle {
    // WARNING: FreeTypeLibraryHandle contains raw pointers, is clonable, and also implements
    // `Drop`. This field needs to be Rc<> to make sure that the `drop` function is only called
    // once, otherwise we'll get crashes. Yuk.
    pub ctx: Rc<FreeTypeLibraryHandle>,
}

impl FontContextHandle {
    pub fn new() -> FontContextHandle {
        let user = Box::into_raw(box User {
            size: 0,
        });
        let mem = Box::into_raw(box struct_FT_MemoryRec_ {
            user: user as *mut c_void,
            alloc: ft_alloc,
            free: ft_free,
            realloc: ft_realloc,
        });
        unsafe {
            let mut ctx: FT_Library = ptr::null_mut();

            let result = FT_New_Library(mem, &mut ctx);
            if !result.succeeded() { panic!("Unable to initialize FreeType library"); }

            FT_Add_Default_Modules(ctx);

            FontContextHandle {
                ctx: Rc::new(FreeTypeLibraryHandle { ctx: ctx, mem: mem, user: user }),
            }
        }
    }
}
