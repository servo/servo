/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub use crate::unicode_block::{UnicodeBlock, UnicodeBlockMethod};

pub fn is_bidi_control(c: char) -> bool {
    matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200E}' | '\u{200F}' | '\u{061C}')
}

pub fn unicode_plane(codepoint: char) -> u32 {
    (codepoint as u32) >> 16
}

pub fn is_cjk(codepoint: char) -> bool {
    if let Some(
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
        UnicodeBlock::HalfwidthandFullwidthForms,
    ) = codepoint.block()
    {
        return true;
    }

    // https://en.wikipedia.org/wiki/Plane_(Unicode)#Supplementary_Ideographic_Plane
    // https://en.wikipedia.org/wiki/Plane_(Unicode)#Tertiary_Ideographic_Plane
    unicode_plane(codepoint) == 2 || unicode_plane(codepoint) == 3
}

#[test]
fn test_is_cjk() {
    // Test characters from different CJK blocks
    assert_eq!(is_cjk('〇'), true);
    assert_eq!(is_cjk('㐀'), true);
    assert_eq!(is_cjk('あ'), true);
    assert_eq!(is_cjk('ア'), true);
    assert_eq!(is_cjk('㆒'), true);
    assert_eq!(is_cjk('ㆣ'), true);
    assert_eq!(is_cjk('龥'), true);
    assert_eq!(is_cjk('𰾑'), true);
    assert_eq!(is_cjk('𰻝'), true);

    // Test characters from outside CJK blocks
    assert_eq!(is_cjk('a'), false);
    assert_eq!(is_cjk('🙂'), false);
    assert_eq!(is_cjk('©'), false);
}
