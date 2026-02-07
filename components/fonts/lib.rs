/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

mod font;
mod font_context;
mod font_store;
mod glyph;
#[expect(unsafe_code)]
pub mod platform; // Public because integration tests need this
mod shapers;
mod system_font_service;

pub(crate) use font::*;
// These items are not meant to be part of the public API but are used for integration tests
pub use font::{Font, FontFamilyDescriptor, FontSearchScope, PlatformFontMethods};
pub use font::{
    FontBaseline, FontGroup, FontMetrics, FontRef, LAST_RESORT_GLYPH_ADVANCE, ShapingFlags,
    ShapingOptions,
};
pub use font_context::{
    CspViolationHandler, FontContext, FontContextWebFontMethods, NetworkTimingHandler,
    WebFontDocumentContext,
};
pub use font_store::FontTemplates;
pub use fonts_traits::*;
pub(crate) use glyph::*;
pub use glyph::{GlyphInfo, GlyphStore};
pub use platform::font_list::fallback_font_families;
pub(crate) use shapers::*;
use style::values::computed::XLang;
pub use system_font_service::SystemFontService;
use unicode_properties::{EmojiStatus, UnicodeEmoji, emoji};

/// Whether or not font fallback selection prefers the emoji or text representation
/// of a character. If `None` then either presentation is acceptable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum EmojiPresentationPreference {
    None,
    Text,
    Emoji,
}

#[derive(Clone, Debug)]
pub struct FallbackFontSelectionOptions {
    pub(crate) character: char,
    pub(crate) presentation_preference: EmojiPresentationPreference,
    pub(crate) lang: XLang,
}

impl Default for FallbackFontSelectionOptions {
    fn default() -> Self {
        Self {
            character: ' ',
            presentation_preference: EmojiPresentationPreference::None,
            lang: XLang("".into()),
        }
    }
}

impl FallbackFontSelectionOptions {
    pub(crate) fn new(character: char, next_character: Option<char>, lang: XLang) -> Self {
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
            lang,
        }
    }
}

pub(crate) fn float_to_fixed(before: usize, f: f64) -> i32 {
    ((1i32 << before) as f64 * f) as i32
}

pub(crate) fn fixed_to_float(before: usize, f: i32) -> f64 {
    f as f64 * 1.0f64 / ((1i32 << before) as f64)
}
