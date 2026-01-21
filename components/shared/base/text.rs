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
    ($type_name:ident) => {
        /// A length in code units of the given text encoding. For instance, `Utf8CodeUnitLength`
        /// is a length in UTF-8 code units (one byte each). `Utf16CodeUnitLength` is a length in
        /// UTF-16 code units (two bytes each). This type is used to more reliable work with
        /// lengths in different encodings.
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

unicode_length_type!(Utf8CodeUnitLength);
unicode_length_type!(Utf16CodeUnitLength);

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
}
