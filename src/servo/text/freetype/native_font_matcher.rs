extern mod freetype;

use freetype::{
    FT_Error,
    FT_Library,
};
use freetype::bindgen::{
    FT_Init_FreeType, 
    FT_Done_FreeType
};


pub struct FreeTypeNativeFontMatcher {
    ft_lib: FT_Library,

    drop {
        assert self.ft_lib.is_not_null();
        FT_Done_FreeType(self.ft_lib);
    }
}

pub impl FreeTypeNativeFontMatcher {
    static pub fn new() -> FreeTypeNativeFontMatcher {
        let lib: FT_Library = ptr::null();
        let res = FT_Init_FreeType(ptr::addr_of(&lib));
        // FIXME: error handling
        assert res == 0 as FT_Error;

        FreeTypeNativeFontMatcher { 
            ft_lib: lib,
        }
    }
}