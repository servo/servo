/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ucd::{Codepoint, UnicodeBlock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompressionMode {
    CompressNone,
    CompressWhitespace,
    CompressWhitespaceNewline,
    DiscardNewline
}

// ported from Gecko's nsTextFrameUtils::TransformText.
//
// High level TODOs:
//
// * Issue #113: consider incoming text state (arabic, etc)
//               and propagate outgoing text state (dual of above)
//
// * Issue #114: record skipped and kept chars for mapping original to new text
//
// * Untracked: various edge cases for bidi, CJK, etc.
pub fn transform_text(text: &str,
                      mode: CompressionMode,
                      incoming_whitespace: bool,
                      output_text: &mut String)
                      -> bool {
    let out_whitespace = match mode {
        CompressionMode::CompressNone | CompressionMode::DiscardNewline => {
            for ch in text.chars() {
                if is_discardable_char(ch, mode) {
                    // TODO: record skipped char
                } else {
                    // TODO: record kept char
                    if ch == '\t' {
                        // TODO: set "has tab" flag
                    }
                    output_text.push(ch);
                }
            }
            false
        },

        CompressionMode::CompressWhitespace | CompressionMode::CompressWhitespaceNewline => {
            let mut in_whitespace: bool = incoming_whitespace;
            for ch in text.chars() {
                // TODO: discard newlines between CJK chars
                let mut next_in_whitespace: bool = is_in_whitespace(ch, mode);

                if !next_in_whitespace {
                    if is_always_discardable_char(ch) {
                        // revert whitespace setting, since this char was discarded
                        next_in_whitespace = in_whitespace;
                        // TODO: record skipped char
                    } else {
                        // TODO: record kept char
                        output_text.push(ch);
                    }
                } else { /* next_in_whitespace; possibly add a space char */
                    if in_whitespace {
                        // TODO: record skipped char
                    } else {
                        // TODO: record kept char
                        output_text.push(' ');
                    }
                }
                // save whitespace context for next char
                in_whitespace = next_in_whitespace;
            } /* /for str::each_char */
            in_whitespace
        }
    };

    return out_whitespace;

    fn is_in_whitespace(ch: char, mode: CompressionMode) -> bool {
        match (ch, mode) {
            (' ', _)  => true,
            ('\t', _) => true,
            ('\n', CompressionMode::CompressWhitespaceNewline) => true,
            (_, _)    => false
        }
    }

    fn is_discardable_char(ch: char, mode: CompressionMode) -> bool {
        if is_always_discardable_char(ch) {
            return true;
        }
        match mode {
            CompressionMode::DiscardNewline | CompressionMode::CompressWhitespaceNewline => ch == '\n',
            _ => false
        }
    }

    fn is_always_discardable_char(ch: char) -> bool {
        // TODO: check for soft hyphens.
        is_bidi_control(ch)
    }
}

pub fn float_to_fixed(before: usize, f: f64) -> i32 {
    ((1i32 << before) as f64 * f) as i32
}

pub fn fixed_to_float(before: usize, f: i32) -> f64 {
    f as f64 * 1.0f64 / ((1i32 << before) as f64)
}

pub fn is_bidi_control(c: char) -> bool {
    match c {
        '\u{202A}'...'\u{202E}' => true,
        '\u{2066}'...'\u{2069}' => true,
        '\u{200E}' | '\u{200F}' | '\u{061C}' => true,
        _ => false
    }
}

pub fn unicode_plane(codepoint: char) -> u32 {
    (codepoint as u32) >> 16
}

pub fn is_cjk(codepoint: char) -> bool {
    if let Some(block) = codepoint.block() {
        match block {
            UnicodeBlock::CJKRadicalsSupplement |
            UnicodeBlock::KangxiRadicals |
            UnicodeBlock::IdeographicDescriptionCharacters |
            UnicodeBlock::CJKSymbolsandPunctuation |
            UnicodeBlock::Hiragana |
            UnicodeBlock::Katakana |
            UnicodeBlock::Bopomofo |
            UnicodeBlock::HangulCompatibilityJamo |
            UnicodeBlock::Kanbun |
            UnicodeBlock::BopomofoExtended |
            UnicodeBlock::CJKStrokes |
            UnicodeBlock::KatakanaPhoneticExtensions |
            UnicodeBlock::EnclosedCJKLettersandMonths |
            UnicodeBlock::CJKCompatibility |
            UnicodeBlock::CJKUnifiedIdeographsExtensionA |
            UnicodeBlock::YijingHexagramSymbols |
            UnicodeBlock::CJKUnifiedIdeographs |
            UnicodeBlock::CJKCompatibilityIdeographs |
            UnicodeBlock::CJKCompatibilityForms |
            UnicodeBlock::HalfwidthandFullwidthForms => {
                return true
            }

            _ => {}
        }
    }


    // https://en.wikipedia.org/wiki/Plane_(Unicode)#Supplementary_Ideographic_Plane
    unicode_plane(codepoint) == 2
}
