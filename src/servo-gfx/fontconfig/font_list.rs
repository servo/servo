extern mod freetype;
extern mod fontconfig;

use fc = fontconfig;
use ft = freetype;

use gfx_font::FontHandle;
use gfx_font_list::{FontEntry, FontFamily, FontFamilyMap};
use freetype_impl::font_context::FreeTypeFontContextHandle;
use self::fontconfig::fontconfig::{FcConfig, FcFontSet, FcChar8,
                                   FcResultMatch, FcSetSystem, FcPattern,
                                   FcResultNoMatch, FcMatchPattern};
use self::fontconfig::fontconfig::bindgen::{
    FcConfigGetCurrent, FcConfigGetFonts, FcPatternGetString,
    FcInitReinitialize, FcPatternDestroy, FcPatternReference,
    FcFontSetDestroy, FcCharSetDestroy, FcConfigSubstitute,
    FcDefaultSubstitute, FcPatternCreate, FcPatternAddString,
    FcFontMatch,
};

use core::dvec::DVec;
use core::send_map::{linear, SendMap};
use libc::c_int;
use ptr::Ptr;

pub struct FontconfigFontListHandle {
    fctx: FreeTypeFontContextHandle,
}

pub impl FontconfigFontListHandle {
    static pub fn new(fctx: &native::FontContextHandle) -> FontconfigFontListHandle {
        FontconfigFontListHandle { fctx: fctx.clone() }
    }

    fn get_available_families() -> FontFamilyMap {
        let mut family_map : FontFamilyMap = linear::LinearMap();
        unsafe {
            let config = FcConfigGetCurrent();
            let fontSet = FcConfigGetFonts(config, FcSetSystem);
            for uint::range(0, (*fontSet).nfont as uint) |i| {
                let font = (*fontSet).fonts.offset(i);
                let family: *FcChar8 = ptr::null();
                let mut v: c_int = 0;
                do str::as_c_str("family") |FC_FAMILY| {
                    while FcPatternGetString(*font, FC_FAMILY, v, &family) == FcResultMatch {
                        let family_name = str::raw::from_buf(family as *u8);
                        debug!("Creating new FontFamily for family: %s", family_name);
                        let new_family = @FontFamily::new(family_name);
                        family_map.insert(family_name, new_family);
                        v += 1;
                    }
                }
            }
        }
        return family_map;
    }

    fn load_variations_for_family(_family: @FontFamily) {
        fail
    }
}

pub fn path_from_identifier(name: ~str) -> Result<~str, ()> unsafe {
    let config = FcConfigGetCurrent();
    let pattern = FcPatternCreate();
    let res = do str::as_c_str("family") |FC_FAMILY| {
        do str::as_c_str(name) |family| {
            FcPatternAddString(pattern, FC_FAMILY, family as *FcChar8)
        }
    };
    if res != 1 {
        debug!("adding family to pattern failed");
        return Err(());
    }

    if FcConfigSubstitute(config, pattern, FcMatchPattern) != 1 {
        debug!("substitution failed");
        return Err(());
    }
    FcDefaultSubstitute(pattern);
    let result = FcResultNoMatch;
    let result_pattern = FcFontMatch(config, pattern, &result);
    if result != FcResultMatch && result_pattern.is_null() {
        debug!("obtaining match to pattern failed");
        return Err(());
    }

    let file: *FcChar8 = ptr::null();
    let res = do str::as_c_str("file") |FC_FILE| {
        FcPatternGetString(result_pattern, FC_FILE, 0, &file)
    };
    if res != FcResultMatch {
        debug!("getting filename for font failed");
        return Err(());
    }
    Ok(str::raw::from_buf(file as *u8))
}