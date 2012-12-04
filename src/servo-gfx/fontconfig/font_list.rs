extern mod freetype;
extern mod fontconfig;

use fc = fontconfig;
use ft = freetype;

use gfx_font::FontHandle;
use gfx_font_list::{FontEntry, FontFamily, FontFamilyMap};

use core::dvec::DVec;
use core::send_map::{linear, SendMap};

pub struct FontconfigFontListHandle {
    fctx: (),
}

pub impl FontconfigFontListHandle {
    static pub fn new(_fctx: &native::FontContextHandle) -> FontconfigFontListHandle {
        FontconfigFontListHandle { fctx: () }
    }

    fn get_available_families() -> FontFamilyMap {
        fail;
    }

    fn load_variations_for_family(family: @FontFamily) {
        fail
    }
}
