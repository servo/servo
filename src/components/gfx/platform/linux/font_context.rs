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
use std::rc::Rc;

#[deriving(Clone)]
struct FreeTypeLibraryHandle {
    ctx: FT_Library,
}

#[deriving(Clone)]
pub struct FontContextHandle {
    ctx: Rc<FreeTypeLibraryHandle>,
}

impl Drop for FreeTypeLibraryHandle {
    fn drop(&mut self) {
        assert!(self.ctx.is_not_null());
        unsafe { FT_Done_FreeType(self.ctx) };
    }
}

impl FontContextHandle {
    pub fn new() -> FontContextHandle {
        unsafe {
            let ctx: FT_Library = ptr::null();
            let result = FT_Init_FreeType(&ctx);
            if !result.succeeded() { fail!("Unable to initialize FreeType library"); }
            FontContextHandle {
                ctx: Rc::new(FreeTypeLibraryHandle { ctx: ctx }),
            }
        }
    }
}

impl FontContextHandleMethods for FontContextHandle {
    fn create_font_from_identifier(&self, name: ~str, style: UsedFontStyle)
                                -> Result<FontHandle, ()> {
        debug!("Creating font handle for {:s}", name);
        path_from_identifier(name, &style).and_then(|file_name| {
            debug!("Opening font face {:s}", file_name);
            FontHandle::new_from_file(self, file_name.to_owned(), &style)
        })
    }
}

