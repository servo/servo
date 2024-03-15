/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CString;
use std::ptr;

use fontconfig_sys::{
    FcChar8, FcConfigGetCurrent, FcConfigGetFonts, FcConfigSubstitute, FcDefaultSubstitute,
    FcFontMatch, FcFontSetDestroy, FcFontSetList, FcMatchPattern, FcNameParse, FcObjectSetAdd,
    FcObjectSetCreate, FcObjectSetDestroy, FcPatternAddString, FcPatternCreate, FcPatternDestroy,
    FcPatternGetInteger, FcPatternGetString, FcResultMatch, FcSetSystem,
};
use libc::{c_char, c_int};
use log::debug;
use serde::{Deserialize, Serialize};
use style::Atom;

use super::c_str_to_string;
use crate::text::util::is_cjk;

static FC_FAMILY: &[u8] = b"family\0";
static FC_FILE: &[u8] = b"file\0";
static FC_INDEX: &[u8] = b"index\0";
static FC_FONTFORMAT: &[u8] = b"fontformat\0";

/// An identifier for a local font on systems using Freetype.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LocalFontIdentifier {
    /// The path to the font.
    pub path: Atom,
    /// The variation index within the font.
    pub variation_index: i32,
}

pub fn for_each_available_family<F>(mut callback: F)
where
    F: FnMut(String),
{
    unsafe {
        let config = FcConfigGetCurrent();
        let font_set = FcConfigGetFonts(config, FcSetSystem);
        for i in 0..((*font_set).nfont as isize) {
            let font = (*font_set).fonts.offset(i);
            let mut family: *mut FcChar8 = ptr::null_mut();
            let mut format: *mut FcChar8 = ptr::null_mut();
            let mut v: c_int = 0;
            if FcPatternGetString(*font, FC_FONTFORMAT.as_ptr() as *mut c_char, v, &mut format) !=
                FcResultMatch
            {
                continue;
            }

            // Skip bitmap fonts. They aren't supported by FreeType.
            let fontformat = c_str_to_string(format as *const c_char);
            if fontformat != "TrueType" && fontformat != "CFF" && fontformat != "Type 1" {
                continue;
            }

            while FcPatternGetString(*font, FC_FAMILY.as_ptr() as *mut c_char, v, &mut family) ==
                FcResultMatch
            {
                let family_name = c_str_to_string(family as *const c_char);
                callback(family_name);
                v += 1;
            }
        }
    }
}

pub fn for_each_variation<F>(family_name: &str, mut callback: F)
where
    F: FnMut(LocalFontIdentifier),
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
        let ok = FcPatternAddString(
            pattern,
            FC_FAMILY.as_ptr() as *mut c_char,
            family_name as *mut FcChar8,
        );
        assert_ne!(ok, 0);

        let object_set = FcObjectSetCreate();
        assert!(!object_set.is_null());

        FcObjectSetAdd(object_set, FC_FILE.as_ptr() as *mut c_char);
        FcObjectSetAdd(object_set, FC_INDEX.as_ptr() as *mut c_char);

        let matches = FcFontSetList(config, font_set_array_ptr, 1, pattern, object_set);
        debug!("found {} variations", (*matches).nfont);

        for i in 0..((*matches).nfont as isize) {
            let font = (*matches).fonts.offset(i);

            let mut path: *mut FcChar8 = ptr::null_mut();
            let result = FcPatternGetString(*font, FC_FILE.as_ptr() as *mut c_char, 0, &mut path);
            assert_eq!(result, FcResultMatch);

            let mut index: libc::c_int = 0;
            let result =
                FcPatternGetInteger(*font, FC_INDEX.as_ptr() as *mut c_char, 0, &mut index);
            assert_eq!(result, FcResultMatch);

            callback(LocalFontIdentifier {
                path: Atom::from(c_str_to_string(path as *const c_char)),
                variation_index: index as i32,
            });
        }

        FcFontSetDestroy(matches);
        FcPatternDestroy(pattern);
        FcObjectSetDestroy(object_set);
    }
}

pub fn system_default_family(generic_name: &str) -> Option<String> {
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
            FcPatternGetString(
                family_match,
                FC_FAMILY.as_ptr() as *mut c_char,
                0,
                &mut match_string,
            );
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

pub static SANS_SERIF_FONT_FAMILY: &str = "DejaVu Sans";

// Based on gfxPlatformGtk::GetCommonFallbackFonts() in Gecko
pub fn fallback_font_families(codepoint: Option<char>) -> Vec<&'static str> {
    let mut families = vec!["DejaVu Serif", "FreeSerif", "DejaVu Sans", "FreeSans"];

    if let Some(codepoint) = codepoint {
        if is_cjk(codepoint) {
            families.push("TakaoPGothic");
            families.push("Droid Sans Fallback");
            families.push("WenQuanYi Micro Hei");
            families.push("NanumGothic");
            families.push("Noto Sans CJK HK");
            families.push("Noto Sans CJK JP");
            families.push("Noto Sans CJK KR");
            families.push("Noto Sans CJK SC");
            families.push("Noto Sans CJK TC");
            families.push("Noto Sans HK");
            families.push("Noto Sans JP");
            families.push("Noto Sans KR");
            families.push("Noto Sans SC");
            families.push("Noto Sans TC");
        }
    }

    families
}
