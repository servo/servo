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

use libc::{self, c_void, c_long, size_t};

extern fn ft_alloc(_mem: FT_Memory, size: c_long) -> *mut c_void {
    unsafe {
        let ptr = libc::malloc(size as size_t);
        ptr as *mut c_void
    }
}

extern fn ft_free(_mem: FT_Memory, block: *mut c_void) {
    unsafe {
        libc::free(block);
    }
}

extern fn ft_realloc(_mem: FT_Memory, _cur_size: c_long, new_size: c_long, block: *mut c_void) -> *mut c_void {
    unsafe {
        let ptr = libc::realloc(block, new_size as size_t);
        ptr as *mut c_void
    }
}

#[derive(Clone)]
pub struct FreeTypeLibraryHandle {
    pub ctx: FT_Library,
    pub mem: FT_Memory,
}

impl Drop for FreeTypeLibraryHandle {
    fn drop(&mut self) {
        assert!(!self.ctx.is_null());
        unsafe {
            FT_Done_Library(self.ctx);
            Box::from_raw(self.mem);
        }
    }
}

#[derive(Clone)]
pub struct FontContextHandle {
    pub ctx: Rc<FreeTypeLibraryHandle>,
}

impl FontContextHandle {
    pub fn new() -> FontContextHandle {
        let mem = box struct_FT_MemoryRec_ {
            user: ptr::null_mut(),
            alloc: ft_alloc,
            free: ft_free,
            realloc: ft_realloc,
        };
        unsafe {
            let mut ctx: FT_Library = ptr::null_mut();

            let mem = ::std::boxed::into_raw(mem);
            let result = FT_New_Library(mem, &mut ctx);
            if !result.succeeded() { panic!("Unable to initialize FreeType library"); }

            FT_Add_Default_Modules(ctx);

            FontContextHandle {
                ctx: Rc::new(FreeTypeLibraryHandle { ctx: ctx, mem: mem }),
            }
        }
    }
}
