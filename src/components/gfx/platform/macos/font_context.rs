/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::UsedFontStyle;
use font_context::FontContextHandleMethods;
use platform::macos::font::FontHandle;

use core_text;

#[deriving(Clone)]
pub struct FontContextHandle {
    ctx: ()
}

#[deriving(Clone)]
impl FontContextHandle {
    // this is a placeholder until NSFontManager or whatever is bound in here.
    pub fn new() -> FontContextHandle {
        FontContextHandle { ctx: () }
    }
}

impl FontContextHandleMethods for FontContextHandle {
    fn create_font_from_identifier(&self,
                                   name: ~str,
                                   style: UsedFontStyle)
                                -> Result<FontHandle, ()> {
        let ctfont_result = core_text::font::new_from_name(name, style.pt_size);
        ctfont_result.and_then(|ctfont| {
            FontHandle::new_from_CTFont(self, ctfont)
        })
    }
}
