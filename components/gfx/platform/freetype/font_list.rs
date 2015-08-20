/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(non_snake_case)]

extern crate freetype;
extern crate fontconfig;

use fontconfig::fontconfig::{FcChar8, FcResultMatch, FcSetSystem};
use fontconfig::fontconfig::{FcConfigGetCurrent, FcConfigGetFonts, FcConfigSubstitute};
use fontconfig::fontconfig::{FcDefaultSubstitute, FcFontMatch, FcNameParse, FcPatternGetString};
use fontconfig::fontconfig::{FcObjectSetAdd, FcPatternGetInteger};
use fontconfig::fontconfig::{FcPatternAddString, FcFontSetList, FcObjectSetCreate, FcObjectSetDestroy};
use fontconfig::fontconfig::{FcPatternDestroy, FcFontSetDestroy, FcMatchPattern, FcPatternCreate};

use util::str::c_str_to_string;

use libc;
use libc::{c_int, c_char};
use std::borrow::ToOwned;
use std::ffi::CString;
use std::ptr;

static FC_FAMILY: &'static [u8] = b"family\0";
static FC_FILE: &'static [u8] = b"file\0";
static FC_INDEX: &'static [u8] = b"index\0";
static FC_FONTFORMAT: &'static [u8] = b"fontformat\0";

pub fn get_available_families<F>(mut callback: F) where F: FnMut(String) {
    unsafe {
        let config = FcConfigGetCurrent();
        let fontSet = FcConfigGetFonts(config, FcSetSystem);
        for i in 0..((*fontSet).nfont as isize) {
            let font = (*fontSet).fonts.offset(i);
            let mut family: *mut FcChar8 = ptr::null_mut();
            let mut format: *mut FcChar8 = ptr::null_mut();
            let mut v: c_int = 0;
            if FcPatternGetString(*font, FC_FONTFORMAT.as_ptr() as *mut c_char, v, &mut format) != FcResultMatch {
                continue;
            }

            // Skip bitmap fonts. They aren't supported by FreeType.
            let fontformat = c_str_to_string(format as *const c_char);
            if fontformat != "TrueType" &&
               fontformat != "CFF" &&
               fontformat != "Type 1" {
                continue;
            }

            while FcPatternGetString(*font, FC_FAMILY.as_ptr() as *mut c_char, v, &mut family) == FcResultMatch {
                let family_name = c_str_to_string(family as *const c_char);
                callback(family_name);
                v += 1;
            }
        }
    }
}

pub fn get_variations_for_family<F>(family_name: &str, mut callback: F)
    where F: FnMut(String)
{
    debug!("getting variations for {}", family_name);
    unsafe {
        let config = FcConfigGetCurrent();
        let mut font_set = FcConfigGetFonts(config, FcSetSystem);
        let font_set_array_ptr = &mut font_set;
        let pattern = FcPatternCreate();
        assert!(!pattern.is_null());
        let family_name_c = CString::new(family_name).unwrap();
        let family_name = family_name_c.as_ptr();
        let ok = FcPatternAddString(pattern, FC_FAMILY.as_ptr() as *mut c_char, family_name as *mut FcChar8);
        assert!(ok != 0);

        let object_set = FcObjectSetCreate();
        assert!(!object_set.is_null());

        FcObjectSetAdd(object_set, FC_FILE.as_ptr() as *mut c_char);
        FcObjectSetAdd(object_set, FC_INDEX.as_ptr() as *mut c_char);

        let matches = FcFontSetList(config, font_set_array_ptr, 1, pattern, object_set);

        debug!("found {} variations", (*matches).nfont);

        for i in 0..((*matches).nfont as isize) {
            let font = (*matches).fonts.offset(i);
            let mut file: *mut FcChar8 = ptr::null_mut();
            let result = FcPatternGetString(*font, FC_FILE.as_ptr() as *mut c_char, 0, &mut file);
            let file = if result == FcResultMatch {
                c_str_to_string(file as *const c_char)
            } else {
                panic!();
            };
            let mut index: libc::c_int = 0;
            let result = FcPatternGetInteger(*font, FC_INDEX.as_ptr() as *mut c_char, 0, &mut index);
            let index = if result == FcResultMatch {
                index
            } else {
                panic!();
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

pub fn get_system_default_family(generic_name: &str) -> Option<String> {
    let generic_name_c = CString::new(generic_name).unwrap();
    let generic_name_ptr = generic_name_c.as_ptr();

    unsafe {
        let pattern = FcNameParse(generic_name_ptr as *mut FcChar8);

        FcConfigSubstitute(ptr::null_mut(), pattern, FcMatchPattern);
        FcDefaultSubstitute(pattern);

        let mut result = 0;
        let family_match = FcFontMatch(ptr::null_mut(), pattern, &mut result);

        let family_name = if result == FcResultMatch {
            let mut match_string: *mut FcChar8 = ptr::null_mut();
            FcPatternGetString(family_match, FC_FAMILY.as_ptr() as *mut c_char, 0, &mut match_string);
            let result = c_str_to_string(match_string as *const c_char);
            FcPatternDestroy(family_match);
            Some(result)
        } else {
            None
        };

        FcPatternDestroy(pattern);
        family_name
    }
}

#[cfg(target_os="linux")]
pub fn get_last_resort_font_families() -> Vec<String> {
    vec!(
        "Fira Sans".to_owned(),
        "DejaVu Sans".to_owned(),
        "Arial".to_owned()
    )
}

#[cfg(target_os="android")]
pub fn get_last_resort_font_families() -> Vec<String> {
    vec!("Roboto".to_owned())
}
