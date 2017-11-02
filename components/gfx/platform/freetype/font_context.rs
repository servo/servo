/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use freetype::freetype::FT_Add_Default_Modules;
use freetype::freetype::FT_Done_Library;
use freetype::freetype::FT_Library;
use freetype::freetype::FT_Memory;
use freetype::freetype::FT_MemoryRec_;
use freetype::freetype::FT_New_Library;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use servo_allocator::libc_compat::{malloc, realloc, free};
use servo_allocator::usable_size;
use std::os::raw::{c_long, c_void};
use std::ptr;
use std::rc::Rc;

// We pass a |User| struct -- via an opaque |void*| -- to FreeType each time a new instance is
// created. FreeType passes it back to the ft_alloc/ft_realloc/ft_free callbacks. We use it to
// record the memory usage of each FreeType instance.
pub struct User {
    size: usize,
}

extern fn ft_alloc(mem: FT_Memory, req_size: c_long) -> *mut c_void {
    unsafe {
        let ptr = malloc(req_size as usize);
        let ptr = ptr as *mut c_void;  // libc::c_void vs std::os::raw::c_void
        let actual_size = usable_size(ptr);
        let user = (*mem).user as *mut User;
        (*user).size += actual_size;
        ptr
    }
}

extern fn ft_free(mem: FT_Memory, ptr: *mut c_void) {
    unsafe {
        let actual_size = usable_size(ptr);
        let user = (*mem).user as *mut User;
        (*user).size -= actual_size;
        free(ptr as *mut _);
    }
}

extern fn ft_realloc(mem: FT_Memory, _old_size: c_long, new_req_size: c_long,
                     old_ptr: *mut c_void) -> *mut c_void {
    unsafe {
        let old_actual_size = usable_size(old_ptr);
        let new_ptr = realloc(old_ptr as *mut _, new_req_size as usize);
        let new_ptr = new_ptr as *mut c_void;
        let new_actual_size = usable_size(new_ptr);
        let user = (*mem).user as *mut User;
        (*user).size += new_actual_size;
        (*user).size -= old_actual_size;
        new_ptr
    }
}

// A |*mut User| field in a struct triggers a "use of `#[derive]` with a raw pointer" warning from
// rustc. But using a typedef avoids this, so...
pub type UserPtr = *mut User;

// WARNING: We need to be careful how we use this struct. See the comment about Rc<> in
// FontContextHandle.
#[derive(Clone, Debug)]
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

impl MallocSizeOf for FreeTypeLibraryHandle {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        unsafe {
            (*self.user).size +
                ops.malloc_size_of(self.ctx as *const _) +
                ops.malloc_size_of(self.mem as *const _) +
                ops.malloc_size_of(self.user as *const _)
        }
    }
}

#[derive(Clone, Debug)]
pub struct FontContextHandle {
    // WARNING: FreeTypeLibraryHandle contains raw pointers, is clonable, and also implements
    // `Drop`. This field needs to be Rc<> to make sure that the `drop` function is only called
    // once, otherwise we'll get crashes. Yuk.
    pub ctx: Rc<FreeTypeLibraryHandle>,
}

impl MallocSizeOf for FontContextHandle {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.ctx.size_of(ops)
    }
}

impl FontContextHandle {
    pub fn new() -> FontContextHandle {
        let user = Box::into_raw(Box::new(User {
            size: 0,
        }));
        let mem = Box::into_raw(Box::new(FT_MemoryRec_ {
            user: user as *mut c_void,
            alloc: Some(ft_alloc),
            free: Some(ft_free),
            realloc: Some(ft_realloc),
        }));
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
