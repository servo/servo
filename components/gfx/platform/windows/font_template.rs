/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gdi32;
use std::ffi::OsString;
use std::io::Error;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use string_cache::Atom;
use webrender_traits::NativeFontHandle;
use winapi::{DWORD, LF_FACESIZE, LOGFONTW, OUT_TT_ONLY_PRECIS, WCHAR};

const GDI_ERROR: DWORD = 0xffffffff;

#[derive(Deserialize, Serialize, Debug)]
pub struct FontTemplateData {
    pub bytes: Vec<u8>,
    pub identifier: Atom,
}

impl FontTemplateData {
    pub fn new(identifier: Atom,
               font_data: Option<Vec<u8>>) -> Result<FontTemplateData, Error> {
        let bytes = match font_data {
            Some(bytes) => {
                bytes
            },
            None => {
                assert!(identifier.len() < LF_FACESIZE);
                let name = OsString::from(identifier.as_ref());
                let buffer: Vec<WCHAR> = name.encode_wide().collect();
                let mut string: [WCHAR; LF_FACESIZE] = [0; LF_FACESIZE];

                for (src, dest) in buffer.iter().zip(string.iter_mut()) {
                    *dest = *src;
                }

                let config = LOGFONTW {
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
                    lfFaceName: string,
                };

                unsafe {
                    let hdc = gdi32::CreateCompatibleDC(ptr::null_mut());
                    let hfont = gdi32::CreateFontIndirectW(&config as *const _);
                    gdi32::SelectObject(hdc, hfont as *mut _);
                    let size = gdi32::GetFontData(hdc, 0, 0, ptr::null_mut(), 0);
                    assert!(size != GDI_ERROR);
                    let mut buffer: Vec<u8> = vec![0; size as usize];
                    let actual_size = gdi32::GetFontData(hdc, 0, 0, buffer.as_mut_ptr() as *mut _, size);
                    assert!(actual_size == size);
                    gdi32::DeleteDC(hdc);
                    gdi32::DeleteObject(hfont as *mut _);
                    buffer
                }
            }
        };

        Ok(FontTemplateData {
            bytes: bytes,
            identifier: identifier,
        })
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    pub fn bytes_if_in_memory(&self) -> Option<Vec<u8>> {
        Some(self.bytes())
    }

    pub fn native_font(&self) -> Option<NativeFontHandle> {
        None
    }
}
