/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use unicode_properties::{emoji, UnicodeEmoji};

pub use crate::text::shaping::Shaper;

pub mod glyph;
pub mod shaping;
pub mod util;

#[derive(Clone, Copy, Debug)]
pub struct FallbackFontSelectionOptions {
    pub character: char,
    pub prefer_emoji_presentation: bool,
}

impl Default for FallbackFontSelectionOptions {
    fn default() -> Self {
        Self {
            character: ' ',
            prefer_emoji_presentation: false,
        }
    }
}

impl FallbackFontSelectionOptions {
    pub fn new(character: char, next_character: Option<char>) -> Self {
        let prefer_emoji_presentation = match next_character {
            Some(next_character) if emoji::is_emoji_presentation_selector(next_character) => true,
            Some(next_character) if emoji::is_text_presentation_selector(next_character) => false,
            _ => character.is_emoji_char(),
        };
        Self {
            character,
            prefer_emoji_presentation,
        }
    }
}
