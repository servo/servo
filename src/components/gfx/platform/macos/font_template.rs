/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core_text::font::CTFont;
use core_text;

/// Platform specific font representation for mac.
/// The identifier is a PostScript font name. The
/// CTFont object is cached here for use by the
/// render functions that create CGFont references.
pub struct FontTemplateData {
    pub ctfont: CTFont,
    pub identifier: String,
}

impl FontTemplateData {
    pub fn new(identifier: &str) -> FontTemplateData {
        let ctfont_result = core_text::font::new_from_name(identifier.as_slice(), 0.0);
        FontTemplateData {
            ctfont: ctfont_result.unwrap(),
            identifier: identifier.to_string(),
        }
    }
}
