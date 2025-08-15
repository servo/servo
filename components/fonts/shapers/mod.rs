/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod harfbuzz;
pub use harfbuzz::{ShapedGlyphData, ShapedGlyphEntry, Shaper};

pub fn unicode_script_to_iso15924_tag(script: unicode_script::Script) -> u32 {
    let bytes: [u8; 4] = match script {
        unicode_script::Script::Unknown => *b"Zzzz",
        _ => {
            let short_name = script.short_name();
            short_name.as_bytes().try_into().unwrap()
        },
    };

    u32::from_be_bytes(bytes)
}
