/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(uppercase_variables)]

extern crate freetype;
extern crate fontconfig;

use fontconfig::fontconfig::{FcChar8, FcResultMatch, FcSetSystem};
use fontconfig::fontconfig::{
    FcConfigGetCurrent, FcConfigGetFonts, FcPatternGetString,
    FcPatternDestroy, FcFontSetDestroy,
    FcPatternCreate, FcPatternAddString,
    FcFontSetList, FcObjectSetCreate, FcObjectSetDestroy,
    FcObjectSetAdd, FcPatternGetInteger
};

use libc;
use libc::{c_int, c_char};
use std::ptr;
use std::str;

pub fn get_available_families(callback: |String|) {
    unsafe {
        let config = FcConfigGetCurrent();
        let fontSet = FcConfigGetFonts(config, FcSetSystem);
        for i in range(0, (*fontSet).nfont as int) {
            let font = (*fontSet).fonts.offset(i);
            let mut family: *mut FcChar8 = ptr::mut_null();
            let mut v: c_int = 0;
            let mut FC_FAMILY_C = "family".to_c_str();
            let FC_FAMILY = FC_FAMILY_C.as_mut_ptr();
            while FcPatternGetString(*font, FC_FAMILY, v, &mut family) == FcResultMatch {
                let family_name = str::raw::from_c_str(family as *const c_char);
                callback(family_name);
                v += 1;
            }
        }
    }
}

pub fn get_variations_for_family(family_name: &str, callback: |String|) {
    debug!("getting variations for {}", family_name);
    unsafe {
        let config = FcConfigGetCurrent();
        let mut font_set = FcConfigGetFonts(config, FcSetSystem);
        let font_set_array_ptr = &mut font_set;
        let pattern = FcPatternCreate();
        assert!(pattern.is_not_null());
        let mut FC_FAMILY_C = "family".to_c_str();
        let FC_FAMILY = FC_FAMILY_C.as_mut_ptr();
        let mut family_name_c = family_name.to_c_str();
        let family_name = family_name_c.as_mut_ptr();
        let ok = FcPatternAddString(pattern, FC_FAMILY, family_name as *mut FcChar8);
        assert!(ok != 0);

        let object_set = FcObjectSetCreate();
        assert!(object_set.is_not_null());

        let mut FC_FILE_C = "file".to_c_str();
        let FC_FILE = FC_FILE_C.as_mut_ptr();
        FcObjectSetAdd(object_set, FC_FILE);
        let mut FC_INDEX_C = "index".to_c_str();
        let FC_INDEX = FC_INDEX_C.as_mut_ptr();
        FcObjectSetAdd(object_set, FC_INDEX);

        let matches = FcFontSetList(config, font_set_array_ptr, 1, pattern, object_set);

        debug!("found {} variations", (*matches).nfont);

        for i in range(0, (*matches).nfont as int) {
            let font = (*matches).fonts.offset(i);
            let mut FC_FILE_C = "file".to_c_str();
            let FC_FILE = FC_FILE_C.as_mut_ptr();
            let mut file: *mut FcChar8 = ptr::mut_null();
            let file = if FcPatternGetString(*font, FC_FILE, 0, &mut file) == FcResultMatch {
                str::raw::from_c_str(file as *const libc::c_char)
            } else {
                fail!();
            };
            let mut FC_INDEX_C = "index".to_c_str();
            let FC_INDEX = FC_INDEX_C.as_mut_ptr();
            let mut index: libc::c_int = 0;
            let index = if FcPatternGetInteger(*font, FC_INDEX, 0, &mut index) == FcResultMatch {
                index
            } else {
                fail!();
            };

            debug!("variation file: {}", file);
            debug!("variation index: {}", index);

            callback(file);
        }

        FcFontSetDestroy(matches);
        FcPatternDestroy(pattern);
        FcObjectSetDestroy(object_set);
    }
}

pub fn get_last_resort_font_families() -> Vec<String> {
    vec!(
        "Fira Sans".to_string(),
        "DejaVu Sans".to_string(),
        "Arial".to_string()
    )
}
