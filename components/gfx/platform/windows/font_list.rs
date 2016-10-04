/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gdi32;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use winapi::{LOGFONTW, LPARAM, OUT_TT_ONLY_PRECIS, VOID};
use winapi::{c_int, DWORD, LF_FACESIZE};

pub static SANS_SERIF_FONT_FAMILY: &'static str = "Arial";

pub fn system_default_family(_: &str) -> Option<String> {
    None
}

pub fn last_resort_font_families() -> Vec<String> {
    vec!("Arial".to_owned())
}

unsafe extern "system" fn enum_font_callback(lpelfe: *const LOGFONTW,
                                             _: *const VOID,
                                             _: DWORD,
                                             lparam: LPARAM) -> c_int {
    let name = (*lpelfe).lfFaceName;
    let term_pos = name.iter().position(|c| *c == 0).unwrap();
    let name = OsString::from_wide(&name[0..term_pos]).into_string().unwrap();

    let fonts = lparam as *mut Vec<String>;
    let fonts = &mut *fonts;
    fonts.push(name);

    1
}

pub fn for_each_available_family<F>(mut callback: F) where F: FnMut(String) {
    let mut fonts = Vec::new();

    let mut config = LOGFONTW {
        lfHeight: 0,
        lfWidth: 0,
        lfEscapement: 0,
        lfOrientation: 0,
        lfWeight: 0,
        lfItalic: 0,
        lfUnderline: 0,
        lfStrikeOut: 0,
        lfCharSet: 0,
        lfOutPrecision: OUT_TT_ONLY_PRECIS as u8,
        lfClipPrecision: 0,
        lfQuality: 0,
        lfPitchAndFamily: 0,
        lfFaceName: [0; LF_FACESIZE],
    };

    unsafe {
        let hdc = gdi32::CreateCompatibleDC(ptr::null_mut());
        gdi32::EnumFontFamiliesExW(hdc,
                                   &mut config,
                                   Some(enum_font_callback),
                                   &mut fonts as *mut Vec<String> as LPARAM,
                                   0);
        gdi32::DeleteDC(hdc);
    }

    for family in fonts {
        callback(family);
    }
}

pub fn for_each_variation<F>(family_name: &str, mut callback: F) where F: FnMut(String) {
    callback(family_name.to_owned());
}
