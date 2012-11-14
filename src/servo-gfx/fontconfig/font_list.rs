extern mod freetype;
extern mod fontconfig;

use fc = fontconfig;
use ft = freetype;

use gfx_font::FontHandle;
use gfx_font_list::{FontEntry, FontFamily};

use core::dvec::DVec;
use core::send_map::{linear, SendMap};

pub struct FontconfigFontListHandle {
    fctx: (),
}

pub impl FontconfigFontListHandle {
    static pub fn new(_fctx: &native::FontContextHandle) -> FontconfigFontListHandle {
        FontconfigFontListHandle { fctx: () }
    }

    fn get_available_families(&const self,
                              _fctx: &native::FontContextHandle)
                           -> linear::LinearMap<~str, @FontFamily> {
        fail;
    }
}
