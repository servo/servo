/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use unicode_properties::{emoji, EmojiStatus, UnicodeEmoji};

pub use crate::text::shaping::Shaper;

pub mod glyph;
pub mod shaping;
pub mod util;

/// Whether or not font fallback selection prefers the emoji or text representation
/// of a character. If `None` then either presentation is acceptable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EmojiPresentationPreference {
    None,
    Text,
    Emoji,
}

#[derive(Clone, Copy, Debug)]
pub struct FallbackFontSelectionOptions {
    pub character: char,
    pub presentation_preference: EmojiPresentationPreference,
}

impl Default for FallbackFontSelectionOptions {
    fn default() -> Self {
        Self {
            character: ' ',
            presentation_preference: EmojiPresentationPreference::None,
        }
    }
}

impl FallbackFontSelectionOptions {
    pub fn new(character: char, next_character: Option<char>) -> Self {
        let presentation_preference = match next_character {
            Some(next_character) if emoji::is_emoji_presentation_selector(next_character) => {
                EmojiPresentationPreference::Emoji
            },
            Some(next_character) if emoji::is_text_presentation_selector(next_character) => {
                EmojiPresentationPreference::Text
            },
            // We don't want to select emoji prsentation for any possible character that might be an emoji, because
            // that includes characters such as '0' that are also used outside of emoji clusters. Instead, only
            // select the emoji font for characters that explicitly have an emoji presentation (in the absence
            // of the emoji presentation selectors above).
            _ if matches!(
                character.emoji_status(),
                EmojiStatus::EmojiPresentation |
                    EmojiStatus::EmojiPresentationAndModifierBase |
                    EmojiStatus::EmojiPresentationAndEmojiComponent |
                    EmojiStatus::EmojiPresentationAndModifierAndEmojiComponent
            ) =>
            {
                EmojiPresentationPreference::Emoji
            },
            _ if character.is_emoji_char() => EmojiPresentationPreference::Text,
            _ => EmojiPresentationPreference::None,
        };
        Self {
            character,
            presentation_preference,
        }
    }
}
