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

#[deriving(Clone)]
struct FreeTypeLibraryHandle {
    ctx: FT_Library,
}

// FIXME(ksh8281) this value have to use atomic operation for counting ref
static mut font_context_ref_count: uint = 0;
static mut ft_pointer: Option<FT_Library> = None;
pub struct FontContextHandle {
    ctx: FreeTypeLibraryHandle,
}

impl Drop for FontContextHandle {
    #[fixed_stack_segment]
    fn drop(&mut self) {
        assert!(self.ctx.ctx.is_not_null());
        unsafe {
            assert!(font_context_ref_count >= 1);
            font_context_ref_count = font_context_ref_count - 1;
            if font_context_ref_count == 0 {
                FT_Done_FreeType(self.ctx.ctx);
            }
        }
    }
}

impl FontContextHandle {
    #[fixed_stack_segment]
    pub fn new() -> FontContextHandle {
        unsafe {
            match ft_pointer {
                Some(ref ctx) => {
                    font_context_ref_count = font_context_ref_count + 1;
                    FontContextHandle {
                        ctx: FreeTypeLibraryHandle { ctx: ctx.clone() },
                    }
                },
                None => {
                    let ctx: FT_Library = ptr::null();
                    let result = FT_Init_FreeType(ptr::to_unsafe_ptr(&ctx));
                    if !result.succeeded() { fail!(); }
                    ft_pointer = Some(ctx);
                    font_context_ref_count = font_context_ref_count + 1;
                    FontContextHandle {
                        ctx: FreeTypeLibraryHandle { ctx: ctx },
                    }
                }
            }
        }
    }
}

impl FontContextHandleMethods for FontContextHandle {
    fn clone(&self) -> FontContextHandle {
        unsafe {
            font_context_ref_count = font_context_ref_count + 1;
            FontContextHandle{
                ctx: self.ctx.clone()
            }
        }
    }

    fn create_font_from_identifier(&self, name: ~str, style: UsedFontStyle)
                                -> Result<FontHandle, ()> {
        debug!("Creating font handle for {:s}", name);
        do path_from_identifier(name, &style).and_then |file_name| {
            debug!("Opening font face {:s}", file_name);
            FontHandle::new_from_file(self, file_name.to_owned(), &style)
        }
    }
}

