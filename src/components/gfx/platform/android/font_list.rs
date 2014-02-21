/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern mod freetype;
extern mod fontconfig;

use fontconfig::fontconfig::{
    FcChar8, FcResultMatch, FcSetSystem, FcPattern,
    FcResultNoMatch, FcMatchPattern, FC_SLANT_ITALIC, FC_WEIGHT_BOLD, FC_SLANT_OBLIQUE
};
use fontconfig::fontconfig::{
    FcConfigGetCurrent, FcConfigGetFonts, FcPatternGetString,
    FcPatternDestroy, FcFontSetDestroy, FcConfigSubstitute,
    FcDefaultSubstitute, FcPatternCreate, FcPatternAddString, FcPatternAddInteger,
    FcFontMatch, FcFontSetList, FcObjectSetCreate, FcObjectSetDestroy,
    FcObjectSetAdd, FcPatternGetInteger
};

use style::computed_values::font_style;


use font::{FontHandleMethods, UsedFontStyle};
use font_list::{FontEntry, FontFamily, FontFamilyMap};
use platform::font::FontHandle;
use platform::font_context::FontContextHandle;

use std::hashmap::HashMap;
use std::libc;
use std::libc::{c_int, c_char};
use std::ptr;
use std::str;

pub struct FontListHandle {
    fctx: FontContextHandle,
}

impl FontListHandle {
    pub fn new(fctx: &FontContextHandle) -> FontListHandle {
        FontListHandle { fctx: fctx.clone() }
    }

    pub fn get_available_families(&self) -> FontFamilyMap {
        let mut family_map : FontFamilyMap = HashMap::new();
        unsafe {
            let config = FcConfigGetCurrent();
            let fontSet = FcConfigGetFonts(config, FcSetSystem);
            for i in range(0, (*fontSet).nfont as int) {
                let font = (*fontSet).fonts.offset(i);
                let family: *FcChar8 = ptr::null();
                let mut v: c_int = 0;
                "family".to_c_str().with_ref(|FC_FAMILY| {
                    while FcPatternGetString(*font, FC_FAMILY, v, &family) == FcResultMatch {
                        let family_name = str::raw::from_c_str(family as *c_char);
                        debug!("Creating new FontFamily for family: {:s}", family_name);
                        let new_family = FontFamily::new(family_name);
                        family_map.insert(family_name, new_family);
                        v += 1;
                    }
                });
            }
        }
        return family_map;
    }

    pub fn load_variations_for_family(&self, family: &mut FontFamily) {
        debug!("getting variations for {:?}", family);
        unsafe {
            let config = FcConfigGetCurrent();
            let font_set = FcConfigGetFonts(config, FcSetSystem);
            let font_set_array_ptr = ptr::to_unsafe_ptr(&font_set);
            let pattern = FcPatternCreate();
            assert!(pattern.is_not_null());
            "family".to_c_str().with_ref(|FC_FAMILY| {
                family.family_name.to_c_str().with_ref(|family_name| {
                    let ok = FcPatternAddString(pattern, FC_FAMILY, family_name as *FcChar8);
                    assert!(ok != 0);
                });
            });

            let object_set = FcObjectSetCreate();
            assert!(object_set.is_not_null());

            "file".to_c_str().with_ref(|FC_FILE| {
                FcObjectSetAdd(object_set, FC_FILE);
            });
            "index".to_c_str().with_ref(|FC_INDEX| {
                FcObjectSetAdd(object_set, FC_INDEX);
            });

            let matches = FcFontSetList(config, font_set_array_ptr, 1, pattern, object_set);

            debug!("found {} variations", (*matches).nfont);

            for i in range(0, (*matches).nfont as int) {
                let font = (*matches).fonts.offset(i);
                let file = "file".to_c_str().with_ref(|FC_FILE| {
                    let file: *FcChar8 = ptr::null();
                    if FcPatternGetString(*font, FC_FILE, 0, &file) == FcResultMatch {
                        str::raw::from_c_str(file as *libc::c_char)
                    } else {
                        fail!();
                    }
                });
                let index = "index".to_c_str().with_ref(|FC_INDEX| {
                    let index: libc::c_int = 0;
                    if FcPatternGetInteger(*font, FC_INDEX, 0, &index) == FcResultMatch {
                        index
                    } else {
                        fail!();
                    }
                });

                debug!("variation file: {}", file);
                debug!("variation index: {}", index);

                let font_handle = FontHandle::new_from_file_unstyled(&self.fctx,
                                                                     file);
                let font_handle = font_handle.unwrap();

                debug!("Creating new FontEntry for face: {:s}", font_handle.face_name());
                let entry = FontEntry::new(font_handle);
                family.entries.push(entry);
            }

            FcFontSetDestroy(matches);
            FcPatternDestroy(pattern);
            FcObjectSetDestroy(object_set);
        }
    }

    pub fn get_last_resort_font_families() -> ~[~str] {
        ~[~"Roboto"]
    }
}

struct AutoPattern {
    pattern: *FcPattern
}

impl Drop for AutoPattern {
    fn drop(&mut self) {
        unsafe {
            FcPatternDestroy(self.pattern);
        }
    }
}

pub fn path_from_identifier(name: ~str, style: &UsedFontStyle) -> Result<~str, ()> {
    unsafe {
        let config = FcConfigGetCurrent();
        let wrapper = AutoPattern { pattern: FcPatternCreate() };
        let pattern = wrapper.pattern;
        let res = "family".to_c_str().with_ref(|FC_FAMILY| {
            name.to_c_str().with_ref(|family| {
                FcPatternAddString(pattern, FC_FAMILY, family as *FcChar8)
            })
        });
        if res != 1 {
            debug!("adding family to pattern failed");
            return Err(());
        }

        match style.style {
            font_style::normal => (),
            font_style::italic => {
                let res = "slant".to_c_str().with_ref(|FC_SLANT| {
                    FcPatternAddInteger(pattern, FC_SLANT, FC_SLANT_ITALIC)
                });
                if res != 1 {
                    debug!("adding slant to pattern failed");
                    return Err(());
                }
            },
            font_style::oblique => {
                let res = "slant".to_c_str().with_ref(|FC_SLANT| {
                    FcPatternAddInteger(pattern, FC_SLANT, FC_SLANT_OBLIQUE)
                });
                if res != 1 {
                    debug!("adding slant(oblique) to pattern failed");
                    return Err(());
                }
            }
        }

        if style.weight.is_bold() {
            let res = "weight".to_c_str().with_ref(|FC_WEIGHT| {
                FcPatternAddInteger(pattern, FC_WEIGHT, FC_WEIGHT_BOLD)
            });
            if res != 1 {
                debug!("adding weight to pattern failed");
                return Err(());
            }
        }

        if FcConfigSubstitute(config, pattern, FcMatchPattern) != 1 {
            debug!("substitution failed");
            return Err(());
        }
        FcDefaultSubstitute(pattern);
        let result = FcResultNoMatch;
        let result_wrapper = AutoPattern { pattern: FcFontMatch(config, pattern, &result) };
        let result_pattern = result_wrapper.pattern;
        if result != FcResultMatch && result_pattern.is_null() {
            debug!("obtaining match to pattern failed");
            return Err(());
        }

        let file: *FcChar8 = ptr::null();
        let res = "file".to_c_str().with_ref(|FC_FILE| {
            FcPatternGetString(result_pattern, FC_FILE, 0, &file)
        });
        if res != FcResultMatch {
            debug!("getting filename for font failed");
            return Err(());
        }
        Ok(str::raw::from_c_str(file as *c_char))
    }
}
