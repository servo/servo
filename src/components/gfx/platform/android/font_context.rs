/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::UsedFontStyle;
use platform::font::FontHandle;
use font_context::FontContextHandleMethods;
use platform::font_list::path_from_identifier;

use freetype::freetype::{FTErrorMethods, FT_Library};
use freetype::freetype::{FT_Done_FreeType, FT_Init_FreeType};

use std::ptr;

struct FreeTypeLibraryHandle {
    ctx: FT_Library,
}

impl Drop for FreeTypeLibraryHandle {
    #[fixed_stack_segment]
    fn drop(&self) {
        assert!(self.ctx.is_not_null());
        unsafe {
            FT_Done_FreeType(self.ctx);
        }
    }
}

pub struct FontContextHandle {
    ctx: @FreeTypeLibraryHandle,
}

impl FontContextHandle {
    #[fixed_stack_segment]
    pub fn new() -> FontContextHandle {
        unsafe {
            let ctx: FT_Library = ptr::null();
            let result = FT_Init_FreeType(ptr::to_unsafe_ptr(&ctx));
            if !result.succeeded() { fail!(); }

            FontContextHandle { 
                ctx: @FreeTypeLibraryHandle { ctx: ctx },
            }
        }
    }
}

impl FontContextHandleMethods for FontContextHandle {
    fn clone(&self) -> FontContextHandle {
        FontContextHandle { ctx: self.ctx }
    }

    fn create_font_from_identifier(&self, name: ~str, style: UsedFontStyle)
                                -> Result<FontHandle, ()> {
        debug!("Creating font handle for %s", name);
        do path_from_identifier(name, &style).chain |file_name| {
            debug!("Opening font face %s", file_name);
            FontHandle::new_from_file(self, file_name, &style)
        }
    }
}

