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
            let family: *FcChar8 = ptr::null();
            let mut v: c_int = 0;
            "family".to_c_str().with_ref(|FC_FAMILY| {
                while FcPatternGetString(*font, FC_FAMILY, v, &family) == FcResultMatch {
                    let family_name = str::raw::from_c_str(family as *c_char);
                    callback(family_name);
                    v += 1;
                }
            });
        }
    }
}

pub fn get_variations_for_family(family_name: &str, callback: |String|) {
    debug!("getting variations for {}", family_name);
    unsafe {
        let config = FcConfigGetCurrent();
        let font_set = FcConfigGetFonts(config, FcSetSystem);
        let font_set_array_ptr = &font_set;
        let pattern = FcPatternCreate();
        assert!(pattern.is_not_null());
        "family".to_c_str().with_ref(|FC_FAMILY| {
            family_name.to_c_str().with_ref(|family_name| {
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
