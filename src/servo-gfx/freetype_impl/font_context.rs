extern mod freetype;
extern mod fontconfig;

use self::freetype::freetype::{
    FTErrorMethods,
    FT_Error,
    FT_Library,
};
use self::freetype::freetype::bindgen::{
    FT_Init_FreeType, 
    FT_Done_FreeType
};
use fontconfig::font_list::path_from_identifier;

use gfx_font::{
    FontHandle,
    UsedFontStyle,
};
use font_context::FontContextHandleMethods;
use freetype_impl::font::FreeTypeFontHandle;

struct FreeTypeLibraryHandle {
    ctx: FT_Library,

    drop {
        assert self.ctx.is_not_null();
        FT_Done_FreeType(self.ctx);
    }
}

pub struct FreeTypeFontContextHandle {
    ctx: @FreeTypeLibraryHandle,
}

pub impl FreeTypeFontContextHandle {
    static pub fn new() -> FreeTypeFontContextHandle {
        let ctx: FT_Library = ptr::null();
        let result = FT_Init_FreeType(ptr::to_unsafe_ptr(&ctx));
        if !result.succeeded() { fail; }

        FreeTypeFontContextHandle { 
            ctx: @FreeTypeLibraryHandle { ctx: ctx },
        }
    }
}

pub impl FreeTypeFontContextHandle : FontContextHandleMethods {
    pure fn clone(&const self) -> FreeTypeFontContextHandle {
        FreeTypeFontContextHandle { ctx: self.ctx }
    }

    fn create_font_from_identifier(name: ~str, style: UsedFontStyle)
        -> Result<FontHandle, ()> unsafe {
        debug!("Creating font handle for %s", name);
        do path_from_identifier(name).chain |file_name| {
            debug!("Opening font face %s", file_name);
            FreeTypeFontHandle::new_from_file(&self, file_name, &style)
        }
    }
}

