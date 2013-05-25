/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::UsedFontStyle;
use platform::font::FontHandle;
use font_context::FontContextHandleMethods;
use platform::font_list::path_from_identifier;

use freetype::freetype::{FTErrorMethods, FT_Library};
use freetype::freetype::bindgen::{FT_Done_FreeType, FT_Init_FreeType};


struct FreeTypeLibraryHandle {
    ctx: FT_Library,
}

impl Drop for FreeTypeLibraryHandle {
    fn finalize(&self) {
        assert!(self.ctx.is_not_null());
        FT_Done_FreeType(self.ctx);
    }
}

pub struct FontContextHandle {
    ctx: @FreeTypeLibraryHandle,
}

pub impl FontContextHandle {
    pub fn new() -> FontContextHandle {
        let ctx: FT_Library = ptr::null();
        let result = FT_Init_FreeType(ptr::to_unsafe_ptr(&ctx));
        if !result.succeeded() { fail!(); }

        FontContextHandle { 
            ctx: @FreeTypeLibraryHandle { ctx: ctx },
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

