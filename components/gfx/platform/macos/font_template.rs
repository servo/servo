/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core_graphics::data_provider::CGDataProvider;
use core_graphics::font::CGFont;
use core_text::font::CTFont;
use core_text;

use string_cache::Atom;

/// Platform specific font representation for mac.
/// The identifier is a PostScript font name. The
/// CTFont object is cached here for use by the
/// paint functions that create CGFont references.
pub struct FontTemplateData {
    pub ctfont: Option<CTFont>,
    pub identifier: Atom,
    pub font_data: Option<Vec<u8>>
}

unsafe impl Send for FontTemplateData {}
unsafe impl Sync for FontTemplateData {}

impl FontTemplateData {
    pub fn new(identifier: Atom, font_data: Option<Vec<u8>>) -> FontTemplateData {
        let ctfont = match font_data {
            Some(ref bytes) => {
                let fontprov = CGDataProvider::from_buffer(bytes);
                let cgfont_result = CGFont::from_data_provider(fontprov);
                match cgfont_result {
                    Ok(cgfont) => Some(core_text::font::new_from_CGFont(&cgfont, 0.0)),
                    Err(_) => None
                }
            },
            None => {
                Some(core_text::font::new_from_name(identifier.as_slice(), 0.0).unwrap())
            }
        };

        FontTemplateData {
            ctfont: ctfont,
            identifier: identifier.to_owned(),
            font_data: font_data
        }
    }
}
