/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::RangeInclusive;

// File bellow constructed with help of:
// <https://freetype.org/freetype2/docs/reference/ft2-truetype_tables.html#tt_ucr_xxx>

/// TrueType Unicode Range 1 masks (ulUnicodeRange1) as defined in FreeType2.
///
/// Each constant represents a bit in the 32-bit bitfield.
/// The comments indicate the Unicode codepoint ranges associated
/// with each mask.

const TT_UCR_BASIC_LATIN: u32 = 1 << 0;
// Bit  0: Basic Latin (U+0020-U+007E)

const TT_UCR_LATIN1_SUPPLEMENT: u32 = 1 << 1;
// Bit  1: C1 Controls and Latin-1 Supplement (U+0080-U+00FF)

const TT_UCR_LATIN_EXTENDED_A: u32 = 1 << 2;
// Bit  2: Latin Extended-A (U+0100-U+017F)

const TT_UCR_LATIN_EXTENDED_B: u32 = 1 << 3;
// Bit  3: Latin Extended-B (U+0180-U+024F)

const TT_UCR_IPA_EXTENSIONS: u32 = 1 << 4;
/* Bit  4: IPA Extensions
Phonetic Extensions
Phonetic Extensions Supplement
Codepoint ranges: U+0250-U+02AF, U+1D00-U+1D7F, U+1D80-U+1DBF */

const TT_UCR_SPACING_MODIFIER: u32 = 1 << 5;
/* Bit  5: Spacing Modifier Letters
Modifier Tone Letters
Codepoint ranges: U+02B0-U+02FF, U+A700-U+A71F */

const TT_UCR_COMBINING_DIACRITICAL_MARKS: u32 = 1 << 6;
/* Bit  6: Combining Diacritical Marks
Combining Diacritical Marks Supplement
Codepoint ranges: U+0300-U+036F, U+1DC0-U+1DFF */

const TT_UCR_GREEK: u32 = 1 << 7;
// Bit  7: Greek and Coptic (U+0370-U+03FF)

const TT_UCR_COPTIC: u32 = 1 << 8;
// Bit  8: Coptic (U+2C80-U+2CFF)

const TT_UCR_CYRILLIC: u32 = 1 << 9;
/* Bit  9: Cyrillic
Cyrillic Supplement
Cyrillic Extended-A
Cyrillic Extended-B
Codepoint ranges: U+0400-U+04FF, U+0500-U+052F, U+2DE0-U+2DFF, U+A640-U+A69F */

const TT_UCR_ARMENIAN: u32 = 1 << 10;
// Bit 10: Armenian (U+0530-U+058F)

const TT_UCR_HEBREW: u32 = 1 << 11;
// Bit 11: Hebrew (U+0590-U+05FF)

const TT_UCR_VAI: u32 = 1 << 12;
// Bit 12: Vai (U+A500-U+A63F)

const TT_UCR_ARABIC: u32 = 1 << 13;
/* Bit 13: Arabic
Arabic Supplement
Codepoint ranges: U+0600-U+06FF, U+0750-U+077F */

const TT_UCR_NKO: u32 = 1 << 14;
// Bit 14: NKo (U+07C0-U+07FF)

const TT_UCR_DEVANAGARI: u32 = 1 << 15;
// Bit 15: Devanagari (U+0900-U+097F)

const TT_UCR_BENGALI: u32 = 1 << 16;
// Bit 16: Bengali (U+0980-U+09FF)

const TT_UCR_GURMUKHI: u32 = 1 << 17;
// Bit 17: Gurmukhi (U+0A00-U+0A7F)

const TT_UCR_GUJARATI: u32 = 1 << 18;
// Bit 18: Gujarati (U+0A80-U+0AFF)

const TT_UCR_ORIYA: u32 = 1 << 19;
// Bit 19: Oriya (U+0B00-U+0B7F)

const TT_UCR_TAMIL: u32 = 1 << 20;
// Bit 20: Tamil (U+0B80-U+0BFF)

const TT_UCR_TELUGU: u32 = 1 << 21;
// Bit 21: Telugu (U+0C00-U+0C7F)

const TT_UCR_KANNADA: u32 = 1 << 22;
// Bit 22: Kannada (U+0C80-U+0CFF)

const TT_UCR_MALAYALAM: u32 = 1 << 23;
// Bit 23: Malayalam (U+0D00-U+0D7F)

const TT_UCR_THAI: u32 = 1 << 24;
// Bit 24: Thai (U+0E00-U+0E7F)

const TT_UCR_LAO: u32 = 1 << 25;
// Bit 25: Lao (U+0E80-U+0EFF)

const TT_UCR_GEORGIAN: u32 = 1 << 26;
/* Bit 26: Georgian
Georgian Supplement
Codepoint ranges: U+10A0-U+10FF, U+2D00-U+2D2F */

const TT_UCR_BALINESE: u32 = 1 << 27;
// Bit 27: Balinese (U+1B00-U+1B7F)

const TT_UCR_HANGUL_JAMO: u32 = 1 << 28;
// Bit 28: Hangul Jamo (U+1100-U+11FF)

const TT_UCR_LATIN_EXTENDED_ADDITIONAL: u32 = 1 << 29;
/* Bit 29: Latin Extended Additional
Latin Extended-C
Latin Extended-D
Codepoint ranges: U+1E00-U+1EFF, U+2C60-U+2C7F, U+A720-U+A7FF */

const TT_UCR_GREEK_EXTENDED: u32 = 1 << 30;
// Bit 30: Greek Extended (U+1F00-U+1FFF)

const TT_UCR_GENERAL_PUNCTUATION: u32 = 1 << 31;
/* Bit 31: General Punctuation
Supplemental Punctuation
Codepoint ranges: U+2000-U+206F, U+2E00-U+2E7F */

/// TrueType Unicode Range 2 masks (ulUnicodeRange2) as defined in FreeType2.
///
/// Each constant represents the mask for a specified Unicode range
/// (in the second 32-bit field). Note that although the overall Unicode
/// bit indices continue from 32 onward, here the shifts start at 0.

const TT_UCR_SUPERSCRIPTS_SUBSCRIPTS: u32 = 1 << 0;
// Bit 32: Superscripts And Subscripts (U+2070-U+209F)

const TT_UCR_CURRENCY_SYMBOLS: u32 = 1 << 1;
// Bit 33: Currency Symbols (U+20A0-U+20CF)

const TT_UCR_COMBINING_DIACRITICAL_MARKS_SYMB: u32 = 1 << 2;
/* Bit 34: Combining Diacritical Marks For Symbols (U+20D0-U+20FF) */

const TT_UCR_LETTERLIKE_SYMBOLS: u32 = 1 << 3;
// Bit 35: Letterlike Symbols (U+2100-U+214F)

const TT_UCR_NUMBER_FORMS: u32 = 1 << 4;
// Bit 36: Number Forms (U+2150-U+218F)

const TT_UCR_ARROWS: u32 = 1 << 5;
/* Bit 37: Arrows, Supplemental Arrows-A, Supplemental Arrows-B,
   and Miscellaneous Symbols and Arrows.
   Codepoint ranges:
     U+2190-U+21FF
     U+27F0-U+27FF
     U+2900-U+297F
     U+2B00-U+2BFF
*/

const TT_UCR_MATHEMATICAL_OPERATORS: u32 = 1 << 6;
/* Bit 38: Mathematical Operators, Supplemental Mathematical Operators,
   Miscellaneous Mathematical Symbols-A, and Miscellaneous Mathematical Symbols-B.
   Codepoint ranges:
     U+2200-U+22FF
     U+2A00-U+2AFF
     U+27C0-U+27EF
     U+2980-U+29FF
*/

const TT_UCR_MISCELLANEOUS_TECHNICAL: u32 = 1 << 7;
// Bit 39: Miscellaneous Technical (U+2300-U+23FF)

const TT_UCR_CONTROL_PICTURES: u32 = 1 << 8;
// Bit 40: Control Pictures (U+2400-U+243F)

const TT_UCR_OCR: u32 = 1 << 9;
// Bit 41: Optical Character Recognition (U+2440-U+245F)

const TT_UCR_ENCLOSED_ALPHANUMERICS: u32 = 1 << 10;
// Bit 42: Enclosed Alphanumerics (U+2460-U+24FF)

const TT_UCR_BOX_DRAWING: u32 = 1 << 11;
// Bit 43: Box Drawing (U+2500-U+257F)

const TT_UCR_BLOCK_ELEMENTS: u32 = 1 << 12;
// Bit 44: Block Elements (U+2580-U+259F)

const TT_UCR_GEOMETRIC_SHAPES: u32 = 1 << 13;
// Bit 45: Geometric Shapes (U+25A0-U+25FF)

const TT_UCR_MISCELLANEOUS_SYMBOLS: u32 = 1 << 14;
// Bit 46: Miscellaneous Symbols (U+2600-U+26FF)

const TT_UCR_DINGBATS: u32 = 1 << 15;
// Bit 47: Dingbats (U+2700-U+27BF)

const TT_UCR_CJK_SYMBOLS: u32 = 1 << 16;
// Bit 48: CJK Symbols and Punctuation (U+3000-U+303F)

const TT_UCR_HIRAGANA: u32 = 1 << 17;
// Bit 49: Hiragana (U+3040-U+309F)

const TT_UCR_KATAKANA: u32 = 1 << 18;
/* Bit 50: Katakana and Katakana Phonetic Extensions.
   Codepoint ranges:
     U+30A0-U+30FF
     U+31F0-U+31FF
*/

const TT_UCR_BOPOMOFO: u32 = 1 << 19;
/* Bit 51: Bopomofo and Bopomofo Extended.
   Codepoint ranges:
     U+3100-U+312F
     U+31A0-U+31BF
*/

const TT_UCR_HANGUL_COMPATIBILITY_JAMO: u32 = 1 << 20;
// Bit 52: Hangul Compatibility Jamo (U+3130-U+318F)

const TT_UCR_CJK_MISC: u32 = 1 << 21;
// Bit 53: Phags-Pa (also used for CJK Miscellaneous)
// Deprecated alias:
const TT_UCR_KANBUN: u32 = TT_UCR_CJK_MISC;
// TT_UCR_PHAGSPA is provided without an assignment in the original table.
const TT_UCR_PHAGSPA: u32 = 0; // Undefined / Placeholder

const TT_UCR_ENCLOSED_CJK_LETTERS_MONTHS: u32 = 1 << 22;
// Bit 54: Enclosed CJK Letters and Months (U+3200-U+32FF)

const TT_UCR_CJK_COMPATIBILITY: u32 = 1 << 23;
// Bit 55: CJK Compatibility (U+3300-U+33FF)

const TT_UCR_HANGUL: u32 = 1 << 24;
// Bit 56: Hangul Syllables (U+AC00-U+D7A3)

const TT_UCR_SURROGATES: u32 = 1 << 25;
/* Bit 57: High Surrogates, High Private Use Surrogates, and Low Surrogates.
   According to OpenType specs v1.3+, setting bit 57 implies support for at least
   one codepoint beyond the Basic Multilingual Plane (i.e., >= U+10000).
   Codepoint ranges:
     U+D800-U+DB7F
     U+DB80-U+DBFF
     U+DC00-U+DFFF
*/
const TT_UCR_NON_PLANE_0: u32 = TT_UCR_SURROGATES; // Alias

const TT_UCR_PHOENICIAN: u32 = 1 << 26;
// Bit 58: Phoenician (U+10900-U+1091F)

const TT_UCR_CJK_UNIFIED_IDEOGRAPHS: u32 = 1 << 27;
/* Bit 59: CJK Unified Ideographs, along with:
   - CJK Radicals Supplement
   - Kangxi Radicals
   - Ideographic Description Characters
   - CJK Unified Ideographs Extension A
   - CJK Unified Ideographs Extension B
   - Kanbun
   Codepoint ranges include:
     U+4E00-U+9FFF,
     U+2E80-U+2EFF,
     U+2F00-U+2FDF,
     U+2FF0-U+2FFF,
     U+3400-U+4DB5,
     U+20000-U+2A6DF,
     U+3190-U+319F
*/

const TT_UCR_PRIVATE_USE: u32 = 1 << 28;
// Bit 60: Private Use (U+E000-U+F8FF)

const TT_UCR_CJK_COMPATIBILITY_IDEOGRAPHS: u32 = 1 << 29;
/* Bit 61: CJK Strokes, CJK Compatibility Ideographs, and
   CJK Compatibility Ideographs Supplement.
   Codepoint ranges:
     U+31C0-U+31EF,
     U+F900-U+FAFF,
     U+2F800-U+2FA1F
*/

const TT_UCR_ALPHABETIC_PRESENTATION_FORMS: u32 = 1 << 30;
// Bit 62: Alphabetic Presentation Forms (U+FB00-U+FB4F)

const TT_UCR_ARABIC_PRESENTATION_FORMS_A: u32 = 1 << 31;
// Bit 63: Arabic Presentation Forms-A (U+FB50-U+FDFF)

/// TrueType Unicode Range 3 masks (ulUnicodeRange3) as defined in FreeType2.
///
/// Each constant represents a bit in the 32-bit bitfield (offset from bit 64).
/// The comments indicate the overall bit number and the corresponding Unicode codepoint ranges.

const TT_UCR_COMBINING_HALF_MARKS: u32 = 1 << 0;
// Bit 64: Combining Half Marks (U+FE20-U+FE2F)

const TT_UCR_CJK_COMPATIBILITY_FORMS: u32 = 1 << 1;
/* Bit 65: Vertical Forms, CJK Compatibility Forms
   Codepoint ranges:
     U+FE10-U+FE1F
     U+FE30-U+FE4F
*/

const TT_UCR_SMALL_FORM_VARIANTS: u32 = 1 << 2;
// Bit 66: Small Form Variants (U+FE50-U+FE6F)

const TT_UCR_ARABIC_PRESENTATION_FORMS_B: u32 = 1 << 3;
// Bit 67: Arabic Presentation Forms-B (U+FE70-U+FEFE)

const TT_UCR_HALFWIDTH_FULLWIDTH_FORMS: u32 = 1 << 4;
// Bit 68: Halfwidth and Fullwidth Forms (U+FF00-U+FFEF)

const TT_UCR_SPECIALS: u32 = 1 << 5;
// Bit 69: Specials (U+FFF0-U+FFFD)

const TT_UCR_TIBETAN: u32 = 1 << 6;
// Bit 70: Tibetan (U+0F00-U+0FFF)

const TT_UCR_SYRIAC: u32 = 1 << 7;
// Bit 71: Syriac (U+0700-U+074F)

const TT_UCR_THAANA: u32 = 1 << 8;
// Bit 72: Thaana (U+0780-U+07BF)

const TT_UCR_SINHALA: u32 = 1 << 9;
// Bit 73: Sinhala (U+0D80-U+0DFF)

const TT_UCR_MYANMAR: u32 = 1 << 10;
// Bit 74: Myanmar (U+1000-U+109F)

const TT_UCR_ETHIOPIC: u32 = 1 << 11;
/* Bit 75: Ethiopic, Ethiopic Supplement, and Ethiopic Extended
   Codepoint ranges:
     U+1200-U+137F
     U+1380-U+139F
     U+2D80-U+2DDF
*/

const TT_UCR_CHEROKEE: u32 = 1 << 12;
// Bit 76: Cherokee (U+13A0-U+13FF)

const TT_UCR_CANADIAN_ABORIGINAL_SYLLABICS: u32 = 1 << 13;
// Bit 77: Unified Canadian Aboriginal Syllabics (U+1400-U+167F)

const TT_UCR_OGHAM: u32 = 1 << 14;
// Bit 78: Ogham (U+1680-U+169F)

const TT_UCR_RUNIC: u32 = 1 << 15;
// Bit 79: Runic (U+16A0-U+16FF)

const TT_UCR_KHMER: u32 = 1 << 16;
/* Bit 80: Khmer and Khmer Symbols
   Codepoint ranges:
     U+1780-U+17FF
     U+19E0-U+19FF
*/

const TT_UCR_MONGOLIAN: u32 = 1 << 17;
// Bit 81: Mongolian (U+1800-U+18AF)

const TT_UCR_BRAILLE: u32 = 1 << 18;
// Bit 82: Braille Patterns (U+2800-U+28FF)

const TT_UCR_YI: u32 = 1 << 19;
/* Bit 83: Yi Syllables and Yi Radicals
   Codepoint ranges:
     U+A000-U+A48F
     U+A490-U+A4CF
*/

const TT_UCR_PHILIPPINE: u32 = 1 << 20;
/* Bit 84: Tagalog, Hanunoo, Buhid, and Tagbanwa
   Codepoint ranges:
     U+1700-U+171F
     U+1720-U+173F
     U+1740-U+175F
     U+1760-U+177F
*/

const TT_UCR_OLD_ITALIC: u32 = 1 << 21;
// Bit 85: Old Italic (U+10300-U+1032F)

const TT_UCR_GOTHIC: u32 = 1 << 22;
// Bit 86: Gothic (U+10330-U+1034F)

const TT_UCR_DESERET: u32 = 1 << 23;
// Bit 87: Deseret (U+10400-U+1044F)

const TT_UCR_MUSICAL_SYMBOLS: u32 = 1 << 24;
/* Bit 88: Byzantine Musical Symbols, Musical Symbols, and Ancient Greek Musical Notation
   Codepoint ranges:
     U+1D000-U+1D0FF
     U+1D100-U+1D1FF
     U+1D200-U+1D24F
*/

const TT_UCR_MATH_ALPHANUMERIC_SYMBOLS: u32 = 1 << 25;
// Bit 89: Mathematical Alphanumeric Symbols (U+1D400-U+1D7FF)

const TT_UCR_PRIVATE_USE_SUPPLEMENTARY: u32 = 1 << 26;
/* Bit 90: Private Use (plane 15) and Private Use (plane 16)
   Codepoint ranges:
     U+F0000-U+FFFFD
     U+100000-U+10FFFD
*/

const TT_UCR_VARIATION_SELECTORS: u32 = 1 << 27;
/* Bit 91: Variation Selectors and Variation Selectors Supplement
   Codepoint ranges:
     U+FE00-U+FE0F
     U+E0100-U+E01EF
*/

const TT_UCR_TAGS: u32 = 1 << 28;
// Bit 92: Tags (U+E0000-U+E007F)

const TT_UCR_LIMBU: u32 = 1 << 29;
// Bit 93: Limbu (U+1900-U+194F)

const TT_UCR_TAI_LE: u32 = 1 << 30;
// Bit 94: Tai Le (U+1950-U+197F)

const TT_UCR_NEW_TAI_LUE: u32 = 1 << 31;
// Bit 95: New Tai Lue (U+1980-U+19DF)

/// TrueType Unicode Range 4 masks (ulUnicodeRange4) as defined in FreeType2.
///
/// Each constant represents a bit in the 32-bit bitfield (offset from overall bit 96).
/// The comments indicate the overall bit number and the associated Unicode codepoint ranges.

const TT_UCR_BUGINESE: u32 = 1 << 0;
// Bit 96: Buginese (U+1A00-U+1A1F)

const TT_UCR_GLAGOLITIC: u32 = 1 << 1;
// Bit 97: Glagolitic (U+2C00-U+2C5F)

const TT_UCR_TIFINAGH: u32 = 1 << 2;
// Bit 98: Tifinagh (U+2D30-U+2D7F)

const TT_UCR_YIJING: u32 = 1 << 3;
// Bit 99: Yijing Hexagram Symbols (U+4DC0-U+4DFF)

const TT_UCR_SYLOTI_NAGRI: u32 = 1 << 4;
// Bit 100: Syloti Nagri (U+A800-U+A82F)

const TT_UCR_LINEAR_B: u32 = 1 << 5;
/* Bit 101: Linear B Syllabary, Linear B Ideograms, Aegean Numbers
   Codepoint ranges:
     U+10000-U+1007F
     U+10080-U+100FF
     U+10100-U+1013F
*/

const TT_UCR_ANCIENT_GREEK_NUMBERS: u32 = 1 << 6;
// Bit 102: Ancient Greek Numbers (U+10140-U+1018F)

const TT_UCR_UGARITIC: u32 = 1 << 7;
// Bit 103: Ugaritic (U+10380-U+1039F)

const TT_UCR_OLD_PERSIAN: u32 = 1 << 8;
// Bit 104: Old Persian (U+103A0-U+103DF)

const TT_UCR_SHAVIAN: u32 = 1 << 9;
// Bit 105: Shavian (U+10450-U+1047F)

const TT_UCR_OSMANYA: u32 = 1 << 10;
// Bit 106: Osmanya (U+10480-U+104AF)

const TT_UCR_CYPRIOT_SYLLABARY: u32 = 1 << 11;
// Bit 107: Cypriot Syllabary (U+10800-U+1083F)

const TT_UCR_KHAROSHTHI: u32 = 1 << 12;
// Bit 108: Kharoshthi (U+10A00-U+10A5F)

const TT_UCR_TAI_XUAN_JING: u32 = 1 << 13;
// Bit 109: Tai Xuan Jing Symbols (U+1D300-U+1D35F)

const TT_UCR_CUNEIFORM: u32 = 1 << 14;
/* Bit 110: Cuneiform and Cuneiform Numbers and Punctuation
   Codepoint ranges:
     U+12000-U+123FF
     U+12400-U+1247F
*/

const TT_UCR_COUNTING_ROD_NUMERALS: u32 = 1 << 15;
// Bit 111: Counting Rod Numerals (U+1D360-U+1D37F)

const TT_UCR_SUNDANESE: u32 = 1 << 16;
// Bit 112: Sundanese (U+1B80-U+1BBF)

const TT_UCR_LEPCHA: u32 = 1 << 17;
// Bit 113: Lepcha (U+1C00-U+1C4F)

const TT_UCR_OL_CHIKI: u32 = 1 << 18;
// Bit 114: Ol Chiki (U+1C50-U+1C7F)

const TT_UCR_SAURASHTRA: u32 = 1 << 19;
// Bit 115: Saurashtra (U+A880-U+A8DF)

const TT_UCR_KAYAH_LI: u32 = 1 << 20;
// Bit 116: Kayah Li (U+A900-U+A92F)

const TT_UCR_REJANG: u32 = 1 << 21;
// Bit 117: Rejang (U+A930-U+A95F)

const TT_UCR_CHAM: u32 = 1 << 22;
// Bit 118: Cham (U+AA00-U+AA5F)

const TT_UCR_ANCIENT_SYMBOLS: u32 = 1 << 23;
// Bit 119: Ancient Symbols (U+10190-U+101CF)

const TT_UCR_PHAISTOS_DISC: u32 = 1 << 24;
// Bit 120: Phaistos Disc (U+101D0-U+101FF)

const TT_UCR_OLD_ANATOLIAN: u32 = 1 << 25;
/* Bit 121: Old Anatolian (Carian, Lycian, Lydian)
   Codepoint ranges:
     U+102A0-U+102DF
     U+10280-U+1029F
     U+10920-U+1093F
*/

const TT_UCR_GAME_TILES: u32 = 1 << 26;
/* Bit 122: Domino Tiles and Mahjong Tiles
   Codepoint ranges:
     U+1F030-U+1F09F
     U+1F000-U+1F02F
*/

// Bits 123-127 are reserved for process-internal usage.

/// A utility structure to represent Unicode ranges associated with a given mask.
/// Multiple codepoint ranges are stored in the `range` vector.
struct UnicodeRange {
    mask: u64,
    range: Vec<RangeInclusive<u32>>,
}

/// Converts the `ulUnicodeRange1` bitfield into a vector of codepoint ranges.
/// Each active mask leads to all of its associated codepoint ranges being included
/// in the result (using a flat mapping over the vector of ranges).
fn convert_unicode_range1(input: u64) -> Vec<RangeInclusive<u32>> {
    let ranges = vec![
        UnicodeRange {
            mask: TT_UCR_BASIC_LATIN as u64,
            range: vec![0x0020..=0x007E],
        },
        UnicodeRange {
            mask: TT_UCR_LATIN1_SUPPLEMENT as u64,
            range: vec![0x0080..=0x00FF],
        },
        UnicodeRange {
            mask: TT_UCR_LATIN_EXTENDED_A as u64,
            range: vec![0x0100..=0x017F],
        },
        UnicodeRange {
            mask: TT_UCR_LATIN_EXTENDED_B as u64,
            range: vec![0x0180..=0x024F],
        },
        UnicodeRange {
            mask: TT_UCR_IPA_EXTENSIONS as u64,
            range: vec![0x0250..=0x02AF, 0x1D00..=0x1D7F, 0x1D80..=0x1DBF],
        },
        UnicodeRange {
            mask: TT_UCR_SPACING_MODIFIER as u64,
            range: vec![0x02B0..=0x02FF, 0xA700..=0xA71F],
        },
        UnicodeRange {
            mask: TT_UCR_COMBINING_DIACRITICAL_MARKS as u64,
            range: vec![0x0300..=0x036F, 0x1DC0..=0x1DFF],
        },
        UnicodeRange {
            mask: TT_UCR_GREEK as u64,
            range: vec![0x0370..=0x03FF],
        },
        UnicodeRange {
            mask: TT_UCR_COPTIC as u64,
            range: vec![0x2C80..=0x2CFF],
        },
        UnicodeRange {
            mask: TT_UCR_CYRILLIC as u64,
            range: vec![
                0x0400..=0x04FF,
                0x0500..=0x052F,
                0x2DE0..=0x2DFF,
                0xA640..=0xA69F,
            ],
        },
        UnicodeRange {
            mask: TT_UCR_ARMENIAN as u64,
            range: vec![0x0530..=0x058F],
        },
        UnicodeRange {
            mask: TT_UCR_HEBREW as u64,
            range: vec![0x0590..=0x05FF],
        },
        UnicodeRange {
            mask: TT_UCR_VAI as u64,
            range: vec![0xA500..=0xA63F],
        },
        UnicodeRange {
            mask: TT_UCR_ARABIC as u64,
            range: vec![0x0600..=0x06FF, 0x0750..=0x077F],
        },
        UnicodeRange {
            mask: TT_UCR_NKO as u64,
            range: vec![0x07C0..=0x07FF],
        },
        UnicodeRange {
            mask: TT_UCR_DEVANAGARI as u64,
            range: vec![0x0900..=0x097F],
        },
        UnicodeRange {
            mask: TT_UCR_BENGALI as u64,
            range: vec![0x0980..=0x09FF],
        },
        UnicodeRange {
            mask: TT_UCR_GURMUKHI as u64,
            range: vec![0x0A00..=0x0A7F],
        },
        UnicodeRange {
            mask: TT_UCR_GUJARATI as u64,
            range: vec![0x0A80..=0x0AFF],
        },
        UnicodeRange {
            mask: TT_UCR_ORIYA as u64,
            range: vec![0x0B00..=0x0B7F],
        },
        UnicodeRange {
            mask: TT_UCR_TAMIL as u64,
            range: vec![0x0B80..=0x0BFF],
        },
        UnicodeRange {
            mask: TT_UCR_TELUGU as u64,
            range: vec![0x0C00..=0x0C7F],
        },
        UnicodeRange {
            mask: TT_UCR_KANNADA as u64,
            range: vec![0x0C80..=0x0CFF],
        },
        UnicodeRange {
            mask: TT_UCR_MALAYALAM as u64,
            range: vec![0x0D00..=0x0D7F],
        },
        UnicodeRange {
            mask: TT_UCR_THAI as u64,
            range: vec![0x0E00..=0x0E7F],
        },
        UnicodeRange {
            mask: TT_UCR_LAO as u64,
            range: vec![0x0E80..=0x0EFF],
        },
        UnicodeRange {
            mask: TT_UCR_GEORGIAN as u64,
            range: vec![0x10A0..=0x10FF, 0x2D00..=0x2D2F],
        },
        UnicodeRange {
            mask: TT_UCR_BALINESE as u64,
            range: vec![0x1B00..=0x1B7F],
        },
        UnicodeRange {
            mask: TT_UCR_HANGUL_JAMO as u64,
            range: vec![0x1100..=0x11FF],
        },
        UnicodeRange {
            mask: TT_UCR_LATIN_EXTENDED_ADDITIONAL as u64,
            range: vec![0x1E00..=0x1EFF, 0x2C60..=0x2C7F, 0xA720..=0xA7FF],
        },
        UnicodeRange {
            mask: TT_UCR_GREEK_EXTENDED as u64,
            range: vec![0x1F00..=0x1FFF],
        },
        UnicodeRange {
            mask: TT_UCR_GENERAL_PUNCTUATION as u64,
            range: vec![0x2000..=0x206F, 0x2E00..=0x2E7F],
        },
    ];

    ranges
        .iter()
        .filter(|r| input & r.mask != 0)
        .flat_map(|r| r.range.clone())
        .collect()
}

/// Converts the `ulUnicodeRange2` bitfield into a vector of codepoint ranges.
/// For each active bit (mask) in the input, all associated ranges are included (flattened).
fn convert_unicode_range2(input: u64) -> Vec<RangeInclusive<u32>> {
    let ranges = vec![
        UnicodeRange {
            mask: TT_UCR_SUPERSCRIPTS_SUBSCRIPTS as u64,
            range: vec![0x2070..=0x209F],
        },
        UnicodeRange {
            mask: TT_UCR_CURRENCY_SYMBOLS as u64,
            range: vec![0x20A0..=0x20CF],
        },
        UnicodeRange {
            mask: TT_UCR_COMBINING_DIACRITICAL_MARKS_SYMB as u64,
            range: vec![0x20D0..=0x20FF],
        },
        UnicodeRange {
            mask: TT_UCR_LETTERLIKE_SYMBOLS as u64,
            range: vec![0x2100..=0x214F],
        },
        UnicodeRange {
            mask: TT_UCR_NUMBER_FORMS as u64,
            range: vec![0x2150..=0x218F],
        },
        UnicodeRange {
            mask: TT_UCR_ARROWS as u64,
            range: vec![
                0x2190..=0x21FF,
                0x27F0..=0x27FF,
                0x2900..=0x297F,
                0x2B00..=0x2BFF,
            ],
        },
        UnicodeRange {
            mask: TT_UCR_MATHEMATICAL_OPERATORS as u64,
            range: vec![
                0x2200..=0x22FF,
                0x2A00..=0x2AFF,
                0x27C0..=0x27EF,
                0x2980..=0x29FF,
            ],
        },
        UnicodeRange {
            mask: TT_UCR_MISCELLANEOUS_TECHNICAL as u64,
            range: vec![0x2300..=0x23FF],
        },
        UnicodeRange {
            mask: TT_UCR_CONTROL_PICTURES as u64,
            range: vec![0x2400..=0x243F],
        },
        UnicodeRange {
            mask: TT_UCR_OCR as u64,
            range: vec![0x2440..=0x245F],
        },
        UnicodeRange {
            mask: TT_UCR_ENCLOSED_ALPHANUMERICS as u64,
            range: vec![0x2460..=0x24FF],
        },
        UnicodeRange {
            mask: TT_UCR_BOX_DRAWING as u64,
            range: vec![0x2500..=0x257F],
        },
        UnicodeRange {
            mask: TT_UCR_BLOCK_ELEMENTS as u64,
            range: vec![0x2580..=0x259F],
        },
        UnicodeRange {
            mask: TT_UCR_GEOMETRIC_SHAPES as u64,
            range: vec![0x25A0..=0x25FF],
        },
        UnicodeRange {
            mask: TT_UCR_MISCELLANEOUS_SYMBOLS as u64,
            range: vec![0x2600..=0x26FF],
        },
        UnicodeRange {
            mask: TT_UCR_DINGBATS as u64,
            range: vec![0x2700..=0x27BF],
        },
        UnicodeRange {
            mask: TT_UCR_CJK_SYMBOLS as u64,
            range: vec![0x3000..=0x303F],
        },
        UnicodeRange {
            mask: TT_UCR_HIRAGANA as u64,
            range: vec![0x3040..=0x309F],
        },
        UnicodeRange {
            mask: TT_UCR_KATAKANA as u64,
            range: vec![0x30A0..=0x30FF, 0x31F0..=0x31FF],
        },
        UnicodeRange {
            mask: TT_UCR_BOPOMOFO as u64,
            range: vec![0x3100..=0x312F, 0x31A0..=0x31BF],
        },
        UnicodeRange {
            mask: TT_UCR_HANGUL_COMPATIBILITY_JAMO as u64,
            range: vec![0x3130..=0x318F],
        },
        UnicodeRange {
            mask: TT_UCR_CJK_MISC as u64,
            range: vec![0xA840..=0xA87F],
        },
        UnicodeRange {
            mask: TT_UCR_ENCLOSED_CJK_LETTERS_MONTHS as u64,
            range: vec![0x3200..=0x32FF],
        },
        UnicodeRange {
            mask: TT_UCR_CJK_COMPATIBILITY as u64,
            range: vec![0x3300..=0x33FF],
        },
        UnicodeRange {
            mask: TT_UCR_HANGUL as u64,
            range: vec![0xAC00..=0xD7A3],
        },
        UnicodeRange {
            mask: TT_UCR_SURROGATES as u64,
            range: vec![0xD800..=0xDB7F, 0xDB80..=0xDBFF, 0xDC00..=0xDFFF],
        },
        UnicodeRange {
            mask: TT_UCR_PHOENICIAN as u64,
            range: vec![0x10900..=0x1091F],
        },
        UnicodeRange {
            mask: TT_UCR_CJK_UNIFIED_IDEOGRAPHS as u64,
            range: vec![
                0x4E00..=0x9FFF,
                0x2E80..=0x2EFF,
                0x2F00..=0x2FDF,
                0x2FF0..=0x2FFF,
                0x3400..=0x4DB5,
                0x20000..=0x2A6DF,
                0x3190..=0x319F,
            ],
        },
        UnicodeRange {
            mask: TT_UCR_PRIVATE_USE as u64,
            range: vec![0xE000..=0xF8FF],
        },
        UnicodeRange {
            mask: TT_UCR_CJK_COMPATIBILITY_IDEOGRAPHS as u64,
            range: vec![0x31C0..=0x31EF, 0xF900..=0xFAFF, 0x2F800..=0x2FA1F],
        },
        UnicodeRange {
            mask: TT_UCR_ALPHABETIC_PRESENTATION_FORMS as u64,
            range: vec![0xFB00..=0xFB4F],
        },
        UnicodeRange {
            mask: TT_UCR_ARABIC_PRESENTATION_FORMS_A as u64,
            range: vec![0xFB50..=0xFDFF],
        },
    ];

    ranges
        .iter()
        .filter(|r| input & r.mask != 0)
        .flat_map(|r| r.range.clone())
        .collect()
}

/// Converts the `ulUnicodeRange3` bitfield into a vector of codepoint ranges.
/// For each active mask in the input, all associated codepoint ranges (flattened)
/// are included in the output.
fn convert_unicode_range3(input: u64) -> Vec<RangeInclusive<u32>> {
    let ranges = vec![
        UnicodeRange {
            mask: TT_UCR_COMBINING_HALF_MARKS as u64,
            range: vec![0xFE20..=0xFE2F],
        },
        UnicodeRange {
            mask: TT_UCR_CJK_COMPATIBILITY_FORMS as u64,
            range: vec![0xFE10..=0xFE1F, 0xFE30..=0xFE4F],
        },
        UnicodeRange {
            mask: TT_UCR_SMALL_FORM_VARIANTS as u64,
            range: vec![0xFE50..=0xFE6F],
        },
        UnicodeRange {
            mask: TT_UCR_ARABIC_PRESENTATION_FORMS_B as u64,
            range: vec![0xFE70..=0xFEFE],
        },
        UnicodeRange {
            mask: TT_UCR_HALFWIDTH_FULLWIDTH_FORMS as u64,
            range: vec![0xFF00..=0xFFEF],
        },
        UnicodeRange {
            mask: TT_UCR_SPECIALS as u64,
            range: vec![0xFFF0..=0xFFFD],
        },
        UnicodeRange {
            mask: TT_UCR_TIBETAN as u64,
            range: vec![0x0F00..=0x0FFF],
        },
        UnicodeRange {
            mask: TT_UCR_SYRIAC as u64,
            range: vec![0x0700..=0x074F],
        },
        UnicodeRange {
            mask: TT_UCR_THAANA as u64,
            range: vec![0x0780..=0x07BF],
        },
        UnicodeRange {
            mask: TT_UCR_SINHALA as u64,
            range: vec![0x0D80..=0x0DFF],
        },
        UnicodeRange {
            mask: TT_UCR_MYANMAR as u64,
            range: vec![0x1000..=0x109F],
        },
        UnicodeRange {
            mask: TT_UCR_ETHIOPIC as u64,
            range: vec![0x1200..=0x137F, 0x1380..=0x139F, 0x2D80..=0x2DDF],
        },
        UnicodeRange {
            mask: TT_UCR_CHEROKEE as u64,
            range: vec![0x13A0..=0x13FF],
        },
        UnicodeRange {
            mask: TT_UCR_CANADIAN_ABORIGINAL_SYLLABICS as u64,
            range: vec![0x1400..=0x167F],
        },
        UnicodeRange {
            mask: TT_UCR_OGHAM as u64,
            range: vec![0x1680..=0x169F],
        },
        UnicodeRange {
            mask: TT_UCR_RUNIC as u64,
            range: vec![0x16A0..=0x16FF],
        },
        UnicodeRange {
            mask: TT_UCR_KHMER as u64,
            range: vec![0x1780..=0x17FF, 0x19E0..=0x19FF],
        },
        UnicodeRange {
            mask: TT_UCR_MONGOLIAN as u64,
            range: vec![0x1800..=0x18AF],
        },
        UnicodeRange {
            mask: TT_UCR_BRAILLE as u64,
            range: vec![0x2800..=0x28FF],
        },
        UnicodeRange {
            mask: TT_UCR_YI as u64,
            range: vec![0xA000..=0xA48F, 0xA490..=0xA4CF],
        },
        UnicodeRange {
            mask: TT_UCR_PHILIPPINE as u64,
            range: vec![
                0x1700..=0x171F,
                0x1720..=0x173F,
                0x1740..=0x175F,
                0x1760..=0x177F,
            ],
        },
        UnicodeRange {
            mask: TT_UCR_OLD_ITALIC as u64,
            range: vec![0x10300..=0x1032F],
        },
        UnicodeRange {
            mask: TT_UCR_GOTHIC as u64,
            range: vec![0x10330..=0x1034F],
        },
        UnicodeRange {
            mask: TT_UCR_DESERET as u64,
            range: vec![0x10400..=0x1044F],
        },
        UnicodeRange {
            mask: TT_UCR_MUSICAL_SYMBOLS as u64,
            range: vec![0x1D000..=0x1D0FF, 0x1D100..=0x1D1FF, 0x1D200..=0x1D24F],
        },
        UnicodeRange {
            mask: TT_UCR_MATH_ALPHANUMERIC_SYMBOLS as u64,
            range: vec![0x1D400..=0x1D7FF],
        },
        UnicodeRange {
            mask: TT_UCR_PRIVATE_USE_SUPPLEMENTARY as u64,
            range: vec![0xF0000..=0xFFFFD, 0x100000..=0x10FFFD],
        },
        UnicodeRange {
            mask: TT_UCR_VARIATION_SELECTORS as u64,
            range: vec![0xFE00..=0xFE0F, 0xE0100..=0xE01EF],
        },
        UnicodeRange {
            mask: TT_UCR_TAGS as u64,
            range: vec![0xE0000..=0xE007F],
        },
        UnicodeRange {
            mask: TT_UCR_LIMBU as u64,
            range: vec![0x1900..=0x194F],
        },
        UnicodeRange {
            mask: TT_UCR_TAI_LE as u64,
            range: vec![0x1950..=0x197F],
        },
        UnicodeRange {
            mask: TT_UCR_NEW_TAI_LUE as u64,
            range: vec![0x1980..=0x19DF],
        },
    ];

    ranges
        .iter()
        .filter(|r| input & r.mask != 0)
        .flat_map(|r| r.range.clone())
        .collect()
}

/// Converts the `ulUnicodeRange4` bitfield into a vector of codepoint ranges.
/// For each active bit in the input, all associated ranges from the defined
/// Unicode constants are flattened into the result.
fn convert_unicode_range4(input: u64) -> Vec<RangeInclusive<u32>> {
    let ranges = vec![
        UnicodeRange {
            mask: TT_UCR_BUGINESE as u64,
            range: vec![0x1A00..=0x1A1F],
        },
        UnicodeRange {
            mask: TT_UCR_GLAGOLITIC as u64,
            range: vec![0x2C00..=0x2C5F],
        },
        UnicodeRange {
            mask: TT_UCR_TIFINAGH as u64,
            range: vec![0x2D30..=0x2D7F],
        },
        UnicodeRange {
            mask: TT_UCR_YIJING as u64,
            range: vec![0x4DC0..=0x4DFF],
        },
        UnicodeRange {
            mask: TT_UCR_SYLOTI_NAGRI as u64,
            range: vec![0xA800..=0xA82F],
        },
        UnicodeRange {
            mask: TT_UCR_LINEAR_B as u64,
            range: vec![0x10000..=0x1007F, 0x10080..=0x100FF, 0x10100..=0x1013F],
        },
        UnicodeRange {
            mask: TT_UCR_ANCIENT_GREEK_NUMBERS as u64,
            range: vec![0x10140..=0x1018F],
        },
        UnicodeRange {
            mask: TT_UCR_UGARITIC as u64,
            range: vec![0x10380..=0x1039F],
        },
        UnicodeRange {
            mask: TT_UCR_OLD_PERSIAN as u64,
            range: vec![0x103A0..=0x103DF],
        },
        UnicodeRange {
            mask: TT_UCR_SHAVIAN as u64,
            range: vec![0x10450..=0x1047F],
        },
        UnicodeRange {
            mask: TT_UCR_OSMANYA as u64,
            range: vec![0x10480..=0x104AF],
        },
        UnicodeRange {
            mask: TT_UCR_CYPRIOT_SYLLABARY as u64,
            range: vec![0x10800..=0x1083F],
        },
        UnicodeRange {
            mask: TT_UCR_KHAROSHTHI as u64,
            range: vec![0x10A00..=0x10A5F],
        },
        UnicodeRange {
            mask: TT_UCR_TAI_XUAN_JING as u64,
            range: vec![0x1D300..=0x1D35F],
        },
        UnicodeRange {
            mask: TT_UCR_CUNEIFORM as u64,
            range: vec![0x12000..=0x123FF, 0x12400..=0x1247F],
        },
        UnicodeRange {
            mask: TT_UCR_COUNTING_ROD_NUMERALS as u64,
            range: vec![0x1D360..=0x1D37F],
        },
        UnicodeRange {
            mask: TT_UCR_SUNDANESE as u64,
            range: vec![0x1B80..=0x1BBF],
        },
        UnicodeRange {
            mask: TT_UCR_LEPCHA as u64,
            range: vec![0x1C00..=0x1C4F],
        },
        UnicodeRange {
            mask: TT_UCR_OL_CHIKI as u64,
            range: vec![0x1C50..=0x1C7F],
        },
        UnicodeRange {
            mask: TT_UCR_SAURASHTRA as u64,
            range: vec![0xA880..=0xA8DF],
        },
        UnicodeRange {
            mask: TT_UCR_KAYAH_LI as u64,
            range: vec![0xA900..=0xA92F],
        },
        UnicodeRange {
            mask: TT_UCR_REJANG as u64,
            range: vec![0xA930..=0xA95F],
        },
        UnicodeRange {
            mask: TT_UCR_CHAM as u64,
            range: vec![0xAA00..=0xAA5F],
        },
        UnicodeRange {
            mask: TT_UCR_ANCIENT_SYMBOLS as u64,
            range: vec![0x10190..=0x101CF],
        },
        UnicodeRange {
            mask: TT_UCR_PHAISTOS_DISC as u64,
            range: vec![0x101D0..=0x101FF],
        },
        UnicodeRange {
            mask: TT_UCR_OLD_ANATOLIAN as u64,
            range: vec![0x102A0..=0x102DF, 0x10280..=0x1029F, 0x10920..=0x1093F],
        },
        UnicodeRange {
            mask: TT_UCR_GAME_TILES as u64,
            range: vec![0x1F030..=0x1F09F, 0x1F000..=0x1F02F],
        },
        // Bits 123-127 are reserved for process-internal usage and are not included.
    ];

    ranges
        .iter()
        .filter(|r| input & r.mask != 0)
        .flat_map(|r| r.range.clone())
        .collect()
}

pub fn convert_unicode_ranges(
    unicode_range1: u64,
    unicode_range2: u64,
    unicode_range3: u64,
    unicode_range4: u64,
) -> Vec<RangeInclusive<u32>> {
    let mut result = Vec::<RangeInclusive<u32>>::new();
    result.extend(convert_unicode_range1(unicode_range1));
    result.extend(convert_unicode_range2(unicode_range2));
    result.extend(convert_unicode_range3(unicode_range3));
    result.extend(convert_unicode_range4(unicode_range4));
    result
}
