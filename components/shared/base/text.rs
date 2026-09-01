/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::mem::size_of;
use std::ops::Range;

use nonmax::NonMaxU32;

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

/// Equivalent to either `Range`, `RangeTo`, `RangeFrom`, or `RangeFull`
#[derive(Clone, Copy)]
pub struct RangeAny<T> {
    /// `None` means zero
    pub start: Option<T>,
    /// `None` means the full available length
    pub end: Option<T>,
}

impl<T> RangeAny<T> {
    /// Apply `Option::map` to each bound of this range
    pub fn map<U>(self, f: impl Fn(T) -> U + Copy) -> RangeAny<U> {
        let Self { start, end } = self;
        RangeAny {
            start: start.map(f),
            end: end.map(f),
        }
    }

    /// Returns the intersection of two ranges, if it is non-empty
    pub fn intersect(self, other: Self) -> Option<Self>
    where
        T: Ord,
    {
        // TODO: https://github.com/rust-lang/rust/issues/144273
        // let start = a.start.reduce(b.start, std::cmp::max);
        // let end = a.end.reduce(b.end, std::cmp::min);
        let start = match (self.start, other.start) {
            (None, None) => None,
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (Some(a), Some(b)) => Some(a.max(b)),
        };
        let end = match (self.end, other.end) {
            (None, None) => None,
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (Some(a), Some(b)) => Some(a.min(b)),
        };
        if start
            .as_ref()
            .is_none_or(|start| end.as_ref().is_none_or(|end| start < end))
        {
            Some(RangeAny { start, end })
        } else {
            // `max()..min()` producing a "backwards" range means the intersection is empty
            None
        }
    }
}

impl<T> From<Range<T>> for RangeAny<T> {
    fn from(value: Range<T>) -> Self {
        Self {
            start: Some(value.start),
            end: Some(value.end),
        }
    }
}

#[allow(unexpected_cfgs)] // for `target_pointer_width = "128"`
pub(crate) fn infalliable_u32_to_usize(value: u32) -> usize {
    cfg_if::cfg_if! {
        if #[cfg(any(
            target_pointer_width = "32",
            target_pointer_width = "64",
            // Rust 1.98 supports no 128-bit target but maybe some future version will
            // Some folks bother to write down RV128I after all
            target_pointer_width = "128",
        ))] {
            value as usize
        } else if #[cfg(target_pointer_width = "16")] {
            const _: () = panic!("16-bit targets are not supported");
        } else {
            const _: () = panic!("This target exceeds the author’s wildest expectations");
        }
    }
}

/// Tried to create string offset or length value larger than supported, or negative
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CodeUnitsOverflowError;

macro_rules! impl_tryfrom_int {
    ($( $Int:ident ),+ => $type_name:ident) => {
        $(
            impl TryFrom<$Int> for $type_name {
                type Error = CodeUnitsOverflowError;
                #[inline]
                fn try_from(value: $Int) -> Result<Self, Self::Error> {
                    u32::try_from(value).map_err(|_| CodeUnitsOverflowError).and_then(Self::new)
                }
            }
        )+
    };
}

macro_rules! unicode_length_type {
    ($( #[$doc:meta] )+ $type_name:ident) => {
        $( #[$doc] )+
        #[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $type_name(NonMaxU32);

        impl $type_name {
            pub const ZERO: Self = Self(NonMaxU32::ZERO);

            pub const MAX_VALUE: u32 = NonMaxU32::MAX.get();

            // Note: the convention from `NonZeroU32::new` and `u32::checked_add`
            // is to return an `Option`, but `TryFrom` requires returning a `Result`.
            // Sticking to one convention avoids `option.ok_or(CodeUnitsOverflowError)` at many
            // call sites that use both.

            #[inline]
            pub fn new(value: u32) -> Result<Self, CodeUnitsOverflowError> {
                NonMaxU32::new(value).map(Self).ok_or(CodeUnitsOverflowError)
            }

            /// # Safety
            ///
            /// `value` must not be greater than [`Self::MAX_VALUE`],
            /// which is currently `u32::MAX - 1`
            #[allow(unsafe_code)]
            #[inline]
            pub unsafe fn new_unchecked(value: u32) -> Self {
                unsafe {
                    Self(NonMaxU32::new_unchecked(value))
                }
            }

            #[inline]
            pub fn get(self) -> u32 {
                self.0.get()
            }

            #[inline]
            pub fn checked_add(self, other: Self) -> Result<Self, CodeUnitsOverflowError> {
                self.get().checked_add(other.get()).ok_or(CodeUnitsOverflowError).and_then(Self::new)
            }

            #[inline]
            pub fn sum(iter: impl IntoIterator<Item = Self>) -> Result<Self, CodeUnitsOverflowError> {
                iter.into_iter().try_fold(Self::ZERO, |acc, item| acc.checked_add(item))
            }

            #[inline]
            pub fn sum_results(iter: impl IntoIterator<Item = Result<Self, CodeUnitsOverflowError>>) -> Result<Self, CodeUnitsOverflowError> {
                iter.into_iter().try_fold(Self::ZERO, |acc, item| acc.checked_add(item?))
            }

            #[inline]
            pub fn checked_sub(self, other: Self) -> Result<Self, CodeUnitsOverflowError> {
                self.get().checked_sub(other.get()).ok_or(CodeUnitsOverflowError).and_then(Self::new)
            }

            #[inline]
            pub fn saturating_sub(self, value: Self) -> Self {
                Self::new(self.get().saturating_sub(value.0.get())).unwrap()
            }
        }

        impl TryFrom<u32> for $type_name {
            type Error = CodeUnitsOverflowError;

            #[inline]
            fn try_from(value: u32) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl_tryfrom_int!(i32, usize, isize => $type_name);

        impl From<$type_name> for u32 {
            #[inline]
            fn from(value: $type_name) -> u32 {
                value.get()
            }
        }

        impl From<$type_name> for usize {
            #[inline]
            fn from(value: $type_name) -> usize {
                infalliable_u32_to_usize(value.get())
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

unicode_length_type! {
    /// A length or offset counted in 32-bit code units in UTF-32 or a node offset in a container
    /// node counted in previous siblings.
    Utf32CodeUnitsOrNodeOffset
}

const _: () = assert!(size_of::<Utf8CodeUnits>() == 4);
const _: () = assert!(size_of::<Option<Utf8CodeUnits>>() == 4);
const _: () = assert!(size_of::<RangeAny<Utf8CodeUnits>>() == 8);

/// Improving this further may require generic pattern types:
/// <https://github.com/rust-lang/rust/issues/136574>
const _: () = assert!(size_of::<Option<RangeAny<Utf8CodeUnits>>>() == 12);

impl Utf16CodeUnits {
    pub fn length_of(string: &str) -> Result<Self, CodeUnitsOverflowError> {
        Self::length_of_as_usize(string).try_into()
    }

    pub fn length_of_iter<S: AsRef<str>>(
        iter: impl IntoIterator<Item = S>,
    ) -> Result<Self, CodeUnitsOverflowError> {
        iter.into_iter()
            .map(|string| Self::length_of_as_usize(string.as_ref()))
            .sum::<usize>()
            .try_into()
    }

    fn length_of_as_usize(string: &str) -> usize {
        string.bytes().map(len_utf16_for_utf8_byte).sum::<usize>()

        // TODO: after upgrading to a Rust version (1.99?) that includes that PR,
        // replace the above with:

        // // `EncodeUtf16::count` is optimized in https://github.com/rust-lang/rust/pull/159467
        // string.encode_utf16().count()
    }

    pub fn to_utf8_code_units_in(
        self,
        string: &str,
    ) -> Result<Utf8CodeUnits, CodeUnitsOverflowError> {
        self.to_utf8_code_units_in_iter(Some(string))
    }

    pub fn to_utf8_code_units_in_iter<S: AsRef<str>>(
        self,
        iter: impl IntoIterator<Item = S>,
    ) -> Result<Utf8CodeUnits, CodeUnitsOverflowError> {
        let expected_utf16_offset = usize::from(self);
        let mut current_utf16_offset = 0;
        let mut current_utf8_offset = 0;
        for string in iter {
            for utf8_byte in string.as_ref().bytes() {
                if current_utf16_offset >= expected_utf16_offset {
                    break;
                }
                current_utf16_offset += len_utf16_for_utf8_byte(utf8_byte);
                current_utf8_offset += len_utf8_for_utf8_byte(utf8_byte);
            }
        }
        current_utf8_offset.try_into()
    }

    pub fn to_utf32_code_units_in(
        self,
        string: &str,
    ) -> Result<Utf32CodeUnits, CodeUnitsOverflowError> {
        let expected_utf16_offset = usize::from(self);
        let mut current_utf16_offset = 0;
        let mut current_utf32_offset = 0;
        for utf8_byte in string.bytes() {
            if current_utf16_offset >= expected_utf16_offset {
                break;
            }
            increment_offsets_for_utf8_byte(
                utf8_byte,
                &mut current_utf16_offset,
                &mut current_utf32_offset,
            );
        }
        current_utf32_offset.try_into()
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

fn len_utf8_for_utf8_byte(byte: u8) -> usize {
    if byte < 0b1000_0000 {
        // 0b0xxx_xxxx: ASCII-compatible U+0000 to U+007F
        1
    } else if byte < 0b1100_0000 {
        // 0b10xx_xxxx: UTF-8 continuation byte, already accounted for by its non-continuation byte
        0
    } else if byte < 0b1110_0000 {
        // 0b110x_xxxx: start of a 2-byte UTF-8 sequence for U+0080 to U+07FF
        2
    } else if byte < 0b1111_0000 {
        // 0b1110_xxxx: start of a 3-byte UTF-8 sequence for U+0800 to U+FFFF
        3
    } else {
        // 0b1111_0xxx: start of a 4-byte UTF-8 sequence for U+010000 to U+10FFFF
        4
    }
}

fn increment_offsets_for_utf8_byte(
    utf8_byte: u8,
    utf16_offset: &mut usize,
    utf32_offset: &mut usize,
) {
    let len_utf16 = len_utf16_for_utf8_byte(utf8_byte);
    *utf16_offset += len_utf16;
    // `len_utf16 != 0` means this byte is the first byte of the UTF-8 byte sequence
    // for one `char` /  UTF-32 code unit
    *utf32_offset += (len_utf16 != 0) as usize;
}

impl Utf32CodeUnits {
    pub fn length_of(string: &str) -> Result<Self, CodeUnitsOverflowError> {
        // `std::str::Chars::count` is optimized in:
        // https://github.com/rust-lang/rust/blob/main/library/core/src/str/count.rs
        string.chars().count().try_into()
    }

    pub fn length_of_iter<S: AsRef<str>>(
        iter: impl IntoIterator<Item = S>,
    ) -> Result<Self, CodeUnitsOverflowError> {
        iter.into_iter()
            .map(|string| string.as_ref().chars().count())
            .sum::<usize>()
            .try_into()
    }

    pub fn to_utf8_code_units_in(
        self,
        string: &str,
    ) -> Result<Utf8CodeUnits, CodeUnitsOverflowError> {
        let expected_utf32_offset = usize::from(self);
        let mut current_utf32_offset = 0;
        for (current_utf8_offset, utf8_byte) in string.bytes().enumerate() {
            if (utf8_byte & 0b1100_0000) == 0b1000_0000 {
                // UTF-8 continuation byte
                continue;
            }
            if current_utf32_offset >= expected_utf32_offset {
                return current_utf8_offset.try_into();
            }
            current_utf32_offset += 1;
        }
        string.len().try_into()
    }

    pub fn to_utf16_code_units_in(
        self,
        string: &str,
    ) -> Result<Utf16CodeUnits, CodeUnitsOverflowError> {
        let expected_utf32_offset = usize::from(self);
        let mut current_utf32_offset = 0;
        let mut current_utf16_offset = 0;
        for utf8_byte in string.bytes() {
            if current_utf32_offset >= expected_utf32_offset {
                break;
            }
            increment_offsets_for_utf8_byte(
                utf8_byte,
                &mut current_utf16_offset,
                &mut current_utf32_offset,
            );
        }
        current_utf16_offset.try_into()
    }
}

impl Utf32CodeUnitsOrNodeOffset {
    pub fn to_utf16_code_units_in(
        self,
        string: &str,
    ) -> Result<Utf16CodeUnits, CodeUnitsOverflowError> {
        Utf32CodeUnits(self.0).to_utf16_code_units_in(string)
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
        assert_eq!(
            Utf16CodeUnits::length_of(""),
            Ok(Utf16CodeUnits::new(0).unwrap())
        );
        assert_eq!(
            Utf16CodeUnits::length_of("a"),
            Ok(Utf16CodeUnits::new(1).unwrap())
        );
        assert_eq!(
            Utf16CodeUnits::length_of("é"),
            Ok(Utf16CodeUnits::new(1).unwrap())
        );
        assert_eq!(
            Utf16CodeUnits::length_of("字"),
            Ok(Utf16CodeUnits::new(1).unwrap())
        );
        assert_eq!(
            Utf16CodeUnits::length_of("\u{1F4A9}"),
            Ok(Utf16CodeUnits::new(2).unwrap())
        );
        assert_eq!(
            Utf16CodeUnits::length_of("\u{1F4A9}字éa"),
            Ok(Utf16CodeUnits::new(5).unwrap())
        );
    }

    #[test]
    fn test_utf16_to_utf32() {
        let s = "aé字\u{1F4A9}";
        assert_eq!(
            Utf16CodeUnits::new(0).unwrap().to_utf32_code_units_in(s),
            Ok(Utf32CodeUnits::new(0).unwrap())
        );
        assert_eq!(
            Utf16CodeUnits::new(1).unwrap().to_utf32_code_units_in(s),
            Ok(Utf32CodeUnits::new(1).unwrap())
        );
        assert_eq!(
            Utf16CodeUnits::new(2).unwrap().to_utf32_code_units_in(s),
            Ok(Utf32CodeUnits::new(2).unwrap())
        );
        assert_eq!(
            Utf16CodeUnits::new(3).unwrap().to_utf32_code_units_in(s),
            Ok(Utf32CodeUnits::new(3).unwrap())
        );

        // This 16-bit offset splits the would-be surrogate pair. We return the 32-bit position
        // after the whole pair. Should this be an error instead?
        assert_eq!(
            Utf16CodeUnits::new(4).unwrap().to_utf32_code_units_in(s),
            Ok(Utf32CodeUnits::new(4).unwrap())
        );

        assert_eq!(
            Utf16CodeUnits::new(5).unwrap().to_utf32_code_units_in(s),
            Ok(Utf32CodeUnits::new(4).unwrap())
        );

        // This 16-bit offset is out of bounds. We clamp to the nearest valid 32-bit offset,
        // a.k.a the UTF-32 length. Should this be an error instead?
        assert_eq!(
            Utf16CodeUnits::new(6).unwrap().to_utf32_code_units_in(s),
            Ok(Utf32CodeUnits::new(4).unwrap())
        );
        assert_eq!(
            Utf16CodeUnits::new(7).unwrap().to_utf32_code_units_in(s),
            Ok(Utf32CodeUnits::new(4).unwrap())
        );
    }

    #[test]
    fn test_utf32_to_utf16() {
        let string = "aé字\u{1F4A9}";
        assert_eq!(
            Utf32CodeUnits::new(0)
                .unwrap()
                .to_utf16_code_units_in(string),
            Ok(Utf16CodeUnits::new(0).unwrap()),
        );
        assert_eq!(
            Utf32CodeUnits::new(1)
                .unwrap()
                .to_utf16_code_units_in(string),
            Ok(Utf16CodeUnits::new(1).unwrap()),
        );
        assert_eq!(
            Utf32CodeUnits::new(2)
                .unwrap()
                .to_utf16_code_units_in(string),
            Ok(Utf16CodeUnits::new(2).unwrap()),
        );
        assert_eq!(
            Utf32CodeUnits::new(3)
                .unwrap()
                .to_utf16_code_units_in(string),
            Ok(Utf16CodeUnits::new(3).unwrap()),
        );

        assert_eq!(
            Utf32CodeUnits::new(4)
                .unwrap()
                .to_utf16_code_units_in(string),
            Ok(Utf16CodeUnits::new(5).unwrap()),
        );

        // This 32-bit offset is out of bounds. We clamp to the nearest valid 16-bit offset,
        // a.k.a the UTF-16 length. Should this be an error instead?
        assert_eq!(
            Utf32CodeUnits::new(6)
                .unwrap()
                .to_utf16_code_units_in(string),
            Ok(Utf16CodeUnits::new(5).unwrap()),
        );
        assert_eq!(
            Utf32CodeUnits::new(1000)
                .unwrap()
                .to_utf16_code_units_in(string),
            Ok(Utf16CodeUnits::new(5).unwrap()),
        );
    }
}
