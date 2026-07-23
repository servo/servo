/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::iter::Sum;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use malloc_size_of_derive::MallocSizeOf;

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

macro_rules! unicode_length_type {
    ($( #[$doc:meta] )+ $type_name:ident) => {
        $( #[$doc] )+
        #[derive(Clone, Copy, Debug, Default, Eq, MallocSizeOf, Ord, PartialEq, PartialOrd)]
        pub struct $type_name(pub usize);

        impl $type_name {
            pub fn zero() -> Self {
                Self(0)
            }

            pub fn one() -> Self {
                Self(1)
            }

            pub fn saturating_sub(self, value: Self) -> Self {
                Self(self.0.saturating_sub(value.0))
            }
        }

        impl From<u32> for $type_name {
            fn from(value: u32) -> Self {
                Self(value as usize)
            }
        }

        impl From<isize> for $type_name {
            fn from(value: isize) -> Self {
                Self(value as usize)
            }
        }

        impl Add for $type_name {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self(self.0 + other.0)
            }
        }

        impl AddAssign for $type_name {
            fn add_assign(&mut self, other: Self) {
                *self = Self(self.0 + other.0)
            }
        }

        impl Sub for $type_name {
            type Output = Self;
            fn sub(self, value: Self) -> Self {
                Self(self.0 - value.0)
            }
        }

        impl SubAssign for $type_name {
            fn sub_assign(&mut self, other: Self) {
                *self = Self(self.0 - other.0)
            }
        }

        impl Sum for $type_name {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::zero(), |a, b| Self(a.0 + b.0))
            }
        }
    };
}

unicode_length_type! {
    /// A length or offset counted in 8-bit code units (bytes) in an UTF-8 string.
    /// This type is used to more reliable work with lengths or offsets in different encodings.
    Utf8CodeUnits
}

unicode_length_type! {
    /// A length or offset counted in 16-bit code units in an UTF-16 string.
    /// This type is used to more reliable work with lengths or offsets in different encodings.
    Utf16CodeUnits
}

unicode_length_type! {
    /// A length or offset counted in 32-bit code units in UTF-32.
    /// This is the same as counting Rust `char`s, Unicode scalar values, or Unicode code points.
    /// This type is used to more reliable work with lengths or offsets in different encodings.
    Utf32CodeUnits
}

impl Utf16CodeUnits {
    pub fn length_of(string: &str) -> Self {
        Self(string.bytes().map(len_utf16_for_utf8_byte).sum())

        // TODO: after upgrading to a Rust version (1.99?) that includes that PR,
        // replace the above with:

        // // `EncodeUtf16::count` is optimized in https://github.com/rust-lang/rust/pull/159467
        // Self(string.encode_utf16().count())
    }

    pub fn to_utf32_code_units_in(self, string: &str) -> Utf32CodeUnits {
        let mut current_utf16_offset = Utf16CodeUnits(0);
        let mut current_utf32_offset = Utf32CodeUnits(0);
        for utf8_byte in string.bytes() {
            if current_utf16_offset >= self {
                break;
            }
            let len_utf16 = len_utf16_for_utf8_byte(utf8_byte);
            current_utf16_offset.0 += len_utf16;
            // `len_utf16 != 0` means this byte is the first byte of the UTF-8 byte sequence
            // for one `char` /  UTF-32 code unit
            current_utf32_offset.0 += (len_utf16 != 0) as usize;
        }
        current_utf32_offset
    }
}

fn len_utf16_for_utf8_byte(byte: u8) -> usize {
    if byte < 0b1000_0000 {
        // 0b0xxx_xxxx: ASCII-compatible U+0000 to U+007F
        1
    } else if byte < 0b1100_0000 {
        // 0b10xx_xxxx: UTF-8 continuation byte, already accounted for by its non-continuation byte
        0
    } else if byte < 0b1111_0000 {
        // 0b110x_xxxx: start of a 2-byte UTF-8 sequence for U+0080 to U+07FF
        // 0b1110_xxxx: start of a 3-byte UTF-8 sequence for U+0800 to U+FFFF
        1
    } else {
        // 0b1111_0xxx: start of a 4-byte UTF-8 sequence for U+010000 to U+10FFFF
        // This is exactly the range encoded as a surrogate pair in UTF-16
        //
        // 0b1111_1xxx: would fall here but never occurs in valid UTF-8
        2
    }
}

impl Utf32CodeUnits {
    pub fn length_of(string: &str) -> Self {
        // `std::str::Chars::count` is optimized in:
        // https://github.com/rust-lang/rust/blob/main/library/core/src/str/count.rs
        Self(string.chars().count())
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

    #[test]
    fn test_utf16_length() {
        assert_eq!(Utf16CodeUnits::length_of(""), Utf16CodeUnits(0));
        assert_eq!(Utf16CodeUnits::length_of("a"), Utf16CodeUnits(1));
        assert_eq!(Utf16CodeUnits::length_of("é"), Utf16CodeUnits(1));
        assert_eq!(Utf16CodeUnits::length_of("字"), Utf16CodeUnits(1));
        assert_eq!(Utf16CodeUnits::length_of("\u{1F4A9}"), Utf16CodeUnits(2));
        assert_eq!(
            Utf16CodeUnits::length_of("\u{1F4A9}字éa"),
            Utf16CodeUnits(5)
        );
    }

    #[test]
    fn test_utf16_to_utf32() {
        let s = "aé字\u{1F4A9}";
        assert_eq!(
            Utf16CodeUnits(0).to_utf32_code_units_in(s),
            Utf32CodeUnits(0)
        );
        assert_eq!(
            Utf16CodeUnits(1).to_utf32_code_units_in(s),
            Utf32CodeUnits(1)
        );
        assert_eq!(
            Utf16CodeUnits(2).to_utf32_code_units_in(s),
            Utf32CodeUnits(2)
        );
        assert_eq!(
            Utf16CodeUnits(3).to_utf32_code_units_in(s),
            Utf32CodeUnits(3)
        );

        // This 16-bit offset splits the would-be surrogate pair. We return the 32-bit position
        // after the whole pair. Should this be an error instead?
        assert_eq!(
            Utf16CodeUnits(4).to_utf32_code_units_in(s),
            Utf32CodeUnits(4)
        );

        assert_eq!(
            Utf16CodeUnits(5).to_utf32_code_units_in(s),
            Utf32CodeUnits(4)
        );

        // This 16-bit offset is out of bounds. We clamp to the nearest valid 32-bit offset,
        // a.k.a the UTF-32 length. Should this be an error instead?
        assert_eq!(
            Utf16CodeUnits(6).to_utf32_code_units_in(s),
            Utf32CodeUnits(4)
        );
        assert_eq!(
            Utf16CodeUnits(7).to_utf32_code_units_in(s),
            Utf32CodeUnits(4)
        );
    }
}
