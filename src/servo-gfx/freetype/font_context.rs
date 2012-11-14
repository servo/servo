extern mod freetype;

use freetype::{
    FTErrorMethods,
    FT_Error,
    FT_Library,
};
use freetype::bindgen::{
    FT_Init_FreeType, 
    FT_Done_FreeType
};

use gfx_font::{
    FontHandle,
    UsedFontStyle,
};
use gfx_font_context::FontContextHandleMethods;

pub struct FreeTypeFontContextHandle {
    ctx: FT_Library,

    drop {
        assert self.ctx.is_not_null();
        FT_Done_FreeType(self.ctx);
    }
}

pub impl FreeTypeFontContextHandle {
    static pub fn new() -> FreeTypeFontContextHandle {
        let ctx: FT_Library = ptr::null();
        let result = FT_Init_FreeType(ptr::to_unsafe_ptr(&ctx));
        if !result.succeeded() { fail; }

        FreeTypeFontContextHandle { 
            ctx: ctx,
        }
    }
}

pub impl FreeTypeFontContextHandle : FontContextHandleMethods {
    fn create_font_from_identifier(_identifier: ~str, _style: UsedFontStyle)
        -> Result<FontHandle, ()> {

        fail;
    }
}

