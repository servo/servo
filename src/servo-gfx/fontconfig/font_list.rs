extern mod freetype;
extern mod fontconfig;

use fc = fontconfig;
use ft = freetype;

use gfx_font::FontHandle;
use gfx_font_list::{FontEntry, FontFamily, FontFamilyMap};
use freetype_impl::font_context::FreeTypeFontContextHandle;
use freetype_impl::font::FreeTypeFontHandle;
use self::fontconfig::fontconfig::{FcConfig, FcFontSet, FcChar8,
                                   FcResultMatch, FcSetSystem, FcPattern,
                                   FcResultNoMatch, FcMatchPattern};
use self::fontconfig::fontconfig::bindgen::{
    FcConfigGetCurrent, FcConfigGetFonts, FcPatternGetString,
    FcInitReinitialize, FcPatternDestroy, FcPatternReference,
    FcFontSetDestroy, FcCharSetDestroy, FcConfigSubstitute,
    FcDefaultSubstitute, FcPatternCreate, FcPatternAddString,
    FcFontMatch, FcFontSetCreate, FcFontSetList, FcPatternPrint,
    FcObjectSetCreate, FcObjectSetDestroy, FcObjectSetAdd,
    FcPatternGetInteger
};

use core::dvec::DVec;
use core::hashmap::linear;
use libc::c_int;
use ptr::Ptr;
use native;

pub struct FontconfigFontListHandle {
    fctx: FreeTypeFontContextHandle,
}

pub impl FontconfigFontListHandle {
    static pub fn new(fctx: &native::FontContextHandle) -> FontconfigFontListHandle {
        FontconfigFontListHandle { fctx: fctx.clone() }
    }

    fn get_available_families() -> FontFamilyMap {
        let mut family_map : FontFamilyMap = linear::LinearMap::new();
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

    fn load_variations_for_family(family: @FontFamily) {
        debug!("getting variations for %?", family);
        let config = FcConfigGetCurrent();
        let font_set = FcConfigGetFonts(config, FcSetSystem);
        let font_set_array_ptr = ptr::to_unsafe_ptr(&font_set);
        unsafe {
            do str::as_c_str("family") |FC_FAMILY| {
                do str::as_c_str(family.family_name) |family_name| {
                    let pattern = FcPatternCreate();
                    assert pattern.is_not_null();
                    let family_name = family_name as *FcChar8;
                    let ok = FcPatternAddString(pattern, FC_FAMILY, family_name);
                    assert ok != 0;

                    let object_set = FcObjectSetCreate();
                    assert object_set.is_not_null();

                    str::as_c_str("file", |FC_FILE| FcObjectSetAdd(object_set, FC_FILE) );
                    str::as_c_str("index", |FC_INDEX| FcObjectSetAdd(object_set, FC_INDEX) );

                    let matches = FcFontSetList(config, font_set_array_ptr, 1, pattern, object_set);

                    debug!("found %? variations", (*matches).nfont);

                    for uint::range(0, (*matches).nfont as uint) |i| {
                        let font = (*matches).fonts.offset(i);
                        let file = do str::as_c_str("file") |FC_FILE| {
                            let file: *FcChar8 = ptr::null();
                            if FcPatternGetString(*font, FC_FILE, 0, &file) == FcResultMatch {
                                str::raw::from_c_str(file as *libc::c_char)
                            } else {
                                fail;
                            }
                        };
                        let index = do str::as_c_str("index") |FC_INDEX| {
                            let index: libc::c_int = 0;
                            if FcPatternGetInteger(*font, FC_INDEX, 0, &index) == FcResultMatch {
                                index
                            } else {
                                fail;
                            }
                        };

                        debug!("variation file: %?", file);
                        debug!("variation index: %?", index);

                        let font_handle = FreeTypeFontHandle::new_from_file_unstyled(&self.fctx, file);
                        let font_handle = font_handle.unwrap();

                        debug!("Creating new FontEntry for face: %s", font_handle.face_name());
                        let entry = @FontEntry::new(family, font_handle);
                        family.entries.push(entry);
                    }

                    FcFontSetDestroy(matches);
                    FcPatternDestroy(pattern);
                    FcObjectSetDestroy(object_set);
                }
            }
        }
    }
}

pub fn path_from_identifier(name: ~str) -> Result<~str, ()> {
    unsafe {
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
}
