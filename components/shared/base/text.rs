/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Range, Sub, SubAssign};

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

/// Equivalent to either `Range`, `RangeTo`, `RangeFrom`, or `RangeFull`
#[derive(Clone, Copy, Eq, PartialEq, MallocSizeOf)]
pub struct RangeAny<T>(RangeAnyInner<T>);

#[derive(Clone, Copy, Eq, PartialEq, MallocSizeOf)]
enum RangeAnyInner<T> {
    Range { start: T, end: T },
    RangeFrom { start: T },
    RangeTo { end: T },
    RangeFull,
}

size_of_test!(RangeAny<u32>, 12);
size_of_test!(Option<RangeAny<u32>>, 12);

impl<T: fmt::Debug> fmt::Debug for RangeAny<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            RangeAnyInner::Range { start, end } => write!(f, "{start:?}..{end:?}"),
            RangeAnyInner::RangeFrom { start } => write!(f, "{start:?}.."),
            RangeAnyInner::RangeTo { end, .. } => write!(f, "..{end:?}"),
            RangeAnyInner::RangeFull => write!(f, ".."),
        }
    }
}

impl<T> RangeAny<T> {
    pub fn new(start: Option<T>, end: Option<T>) -> Self {
        Self(match (start, end) {
            (Some(start), Some(end)) => RangeAnyInner::Range { start, end },
            (Some(start), None) => RangeAnyInner::RangeFrom { start },
            (None, Some(end)) => RangeAnyInner::RangeTo { end },
            (None, None) => RangeAnyInner::RangeFull,
        })
    }

    /// Returns a `RangeAny` that represents the range from the start to the given end.
    pub fn from_start_to(end: T) -> Self {
        Self(RangeAnyInner::RangeTo { end })
    }

    /// Returns a `RangeAny` that represents the full range: both bounds unset
    pub fn full() -> Self {
        Self(RangeAnyInner::RangeFull)
    }

    // Note: for a fully-generic general purpose container we’d return `Option<&T>`
    // and remove the `Copy` bound, but Servo only uses `RangeAny` with `Utf*CodeUnits` types
    // that implement `Copy`, so relying on `Copy` makes callers less verbose.
    pub fn start(&self) -> Option<T>
    where
        T: Copy,
    {
        match self.0 {
            RangeAnyInner::Range { start, .. } | RangeAnyInner::RangeFrom { start } => Some(start),
            RangeAnyInner::RangeTo { .. } | RangeAnyInner::RangeFull => None,
        }
    }

    pub fn end(&self) -> Option<T>
    where
        T: Copy,
    {
        match self.0 {
            RangeAnyInner::Range { end, .. } | RangeAnyInner::RangeTo { end, .. } => Some(end),
            RangeAnyInner::RangeFrom { .. } | RangeAnyInner::RangeFull => None,
        }
    }

    /// Apply `Option::map` to each bound of this range
    pub fn map<U>(&self, f: impl Fn(T) -> U + Copy) -> RangeAny<U>
    where
        T: Copy,
    {
        RangeAny::new(self.start().map(f), self.end().map(f))
    }

    /// Returns the intersection of two ranges, if it is non-empty
    pub fn intersect(&self, other: Self) -> Option<Self>
    where
        T: Copy + Ord,
    {
        // TODO: https://github.com/rust-lang/rust/issues/144273
        // let start = a.start.reduce(b.start, std::cmp::max);
        // let end = a.end.reduce(b.end, std::cmp::min);
        let start = match (self.start(), other.start()) {
            (None, None) => None,
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (Some(a), Some(b)) => Some(a.max(b)),
        };
        let end = match (self.end(), other.end()) {
            (None, None) => None,
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (Some(a), Some(b)) => Some(a.min(b)),
        };
        if start
            .as_ref()
            .is_none_or(|start| end.as_ref().is_none_or(|end| start < end))
        {
            Some(Self::new(start, end))
        } else {
            // `max()..min()` producing a "backwards" range means the intersection is empty
            None
        }
    }
}

impl<T> From<Range<T>> for RangeAny<T> {
    fn from(value: Range<T>) -> Self {
        Self::new(Some(value.start), Some(value.end))
    }
}

/// A marker to make callers acknowledge that a method computes 32-bit offsets or lengths,
/// and trying to compute past `u32::MAX` code units may result in integer overflow.
///
/// The default Rust behavior for integer overflow is panic on debug mode,
/// and silent wrapping (which for offsets or lengths returns a wrong value) in release mode.
pub struct AssumeUnder4GB;

fn infallible_u32_to_usize(value: u32) -> usize {
    const _: () = assert!(usize::BITS >= u32::BITS, "16-bit targets are not supported");
    value as usize
}

macro_rules! unicode_length_type {
    ($( #[$doc:meta] )+ $type_name:ident) => {
        $( #[$doc] )+
        #[derive(Clone, Copy, Default, Eq, MallocSizeOf, Ord, PartialEq, PartialOrd)]
        pub struct $type_name(pub u32);

        impl $type_name {
            const ZERO: Self = Self(0);

            #[inline]
            pub fn saturating_sub(self, value: Self) -> Self {
                Self(self.0.saturating_sub(value.0))
            }

            #[inline]
            pub fn to_usize_range(range: &Range<Self>) -> Range<usize> {
                usize::from(range.start)..usize::from(range.end)
            }
        }

        impl From<u32> for $type_name {
            #[inline]
            fn from(value: u32) -> Self {
                Self(value)
            }
        }

        impl From<$type_name> for usize {
            #[inline]
            fn from(value: $type_name) -> usize {
                infallible_u32_to_usize(value.0)
            }
        }

        impl Add for $type_name {
            type Output = Self;

            #[inline]
            fn add(self, other: Self) -> Self {
                Self(self.0 + other.0)
            }
        }

        impl AddAssign for $type_name {
            #[inline]
            fn add_assign(&mut self, other: Self) {
                *self = Self(self.0 + other.0)
            }
        }

        impl Sub for $type_name {
            type Output = Self;

            #[inline]
            fn sub(self, value: Self) -> Self {
                Self(self.0 - value.0)
            }
        }

        impl SubAssign for $type_name {
            #[inline]
            fn sub_assign(&mut self, other: Self) {
                *self = Self(self.0 - other.0)
            }
        }

        impl Sum for $type_name {
            #[inline]
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::ZERO, |a, b| Self(a.0 + b.0))
            }
        }

        /// Use compact formatting regardless of `Formatter::alternate`
        impl fmt::Debug for $type_name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($type_name), "({:?})"), self.0)
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

impl Utf8CodeUnits {
    /// Returns the length of `string` in UTF-8 code units (bytes)
    pub fn length_of(_: AssumeUnder4GB, string: &str) -> Self {
        Self(string.len() as u32)
    }

    pub fn length_of_char(char: char) -> Self {
        // Never overflows, the value is always in 1..=4
        Self(char.len_utf8() as u32)
    }
}

impl Utf16CodeUnits {
    /// Returns the length of `string` in UTF-16 code units
    pub fn length_of(_: AssumeUnder4GB, string: &str) -> Self {
        Self(string.bytes().map(len_utf16_for_utf8_byte).sum())

        // TODO: after upgrading to a Rust version (1.99?) that includes that PR,
        // replace the above with:

        // // `EncodeUtf16::count` is optimized in https://github.com/rust-lang/rust/pull/159467
        // Self(string.encode_utf16().count())
    }

    pub fn length_of_char(char: char) -> Self {
        // Never overflows, the value is always in 1 or 2
        Self(char.len_utf16() as u32)
    }

    /// Convert this UTF-16 offset in `string` to an UTF-8 (byte) offset
    pub fn to_utf8_code_units_in(self, _: AssumeUnder4GB, string: &str) -> Utf8CodeUnits {
        self.to_utf8_code_units_in_iter(AssumeUnder4GB, std::iter::once(string))
    }

    /// Convert this UTF-16 offset in an iterator of strings, to an UTF-8 (byte) offset
    ///
    /// Note: this silently wraps and returns an incorrect value for results larger than
    /// `u32::MAX` bytes (4 GiB).
    pub fn to_utf8_code_units_in_iter<S>(
        self,
        _: AssumeUnder4GB,
        iter: impl IntoIterator<Item = S>,
    ) -> Utf8CodeUnits
    where
        S: AsRef<str>,
    {
        let mut current_utf16_offset = Utf16CodeUnits(0);
        let mut current_utf8_offset = Utf8CodeUnits(0);
        for string in iter {
            for utf8_byte in string.as_ref().bytes() {
                current_utf16_offset.0 += len_utf16_for_utf8_byte(utf8_byte);
                if current_utf16_offset > self {
                    return current_utf8_offset;
                }
                current_utf8_offset.0 += len_utf8_for_utf8_byte(utf8_byte);
            }
        }
        current_utf8_offset
    }

    /// Convert this UTF-16 offset in `string` to an UTF-32 offset
    ///
    /// Note: this never overflows since the return value is always less than or equal to self as
    /// one UTF-32 code unit corresponds to one or two UTF-16 code units.
    pub fn to_utf32_code_units_in(self, string: &str) -> Utf32CodeUnits {
        let mut current_utf16_offset = Utf16CodeUnits(0);
        let mut current_utf32_offset = Utf32CodeUnits(0);
        for utf8_byte in string.bytes() {
            let len_utf16 = len_utf16_for_utf8_byte(utf8_byte);
            current_utf16_offset.0 += len_utf16;
            if current_utf16_offset > self {
                break;
            }
            current_utf32_offset.0 += len_utf16_to_len_utf32_for_utf8_byte(len_utf16);
        }
        current_utf32_offset
    }
}

fn len_utf16_for_utf8_byte(byte: u8) -> u32 {
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

fn len_utf8_for_utf8_byte(byte: u8) -> u32 {
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

fn len_utf16_to_len_utf32_for_utf8_byte(len_utf16: u32) -> u32 {
    if len_utf16 != 0 {
        // First byte of the UTF-8 byte sequence for one code point / UTF-32 code unit
        1
    } else {
        // UTF-8 continuation byte
        0
    }
}

impl Utf32CodeUnits {
    /// Returns the length of `string` in UTF-32 code units (`char` count)
    pub fn length_of(_: AssumeUnder4GB, string: &str) -> Self {
        // `std::str::Chars::count` is optimized in:
        // https://github.com/rust-lang/rust/blob/main/library/core/src/str/count.rs
        Self(string.chars().count() as u32)
    }

    /// Convert this UTF-32 (`char`) offset in `string` to an UTF-8 (byte) offset
    pub fn to_utf8_code_units_in(self, _: AssumeUnder4GB, string: &str) -> Utf8CodeUnits {
        let mut current_utf32_offset = Utf32CodeUnits(0);
        for (current_utf8_offset, utf8_byte) in string.bytes().enumerate() {
            if (utf8_byte & 0b1100_0000) == 0b1000_0000 {
                // UTF-8 continuation byte
                continue;
            }
            if current_utf32_offset >= self {
                return Utf8CodeUnits(current_utf8_offset as u32);
            }
            current_utf32_offset.0 += 1;
        }
        Utf8CodeUnits(string.len() as u32)
    }

    /// Convert this UTF-32 (`char`) offset in `string` to an UTF-16 offset
    pub fn to_utf16_code_units_in(self, _: AssumeUnder4GB, string: &str) -> Utf16CodeUnits {
        let mut current_utf32_offset = Utf32CodeUnits(0);
        let mut current_utf16_offset = Utf16CodeUnits(0);
        for utf8_byte in string.bytes() {
            if current_utf32_offset >= self {
                break;
            }
            let len_utf16 = len_utf16_for_utf8_byte(utf8_byte);
            current_utf16_offset.0 += len_utf16;
            current_utf32_offset.0 += len_utf16_to_len_utf32_for_utf8_byte(len_utf16);
        }
        current_utf16_offset
    }
}

impl Utf32CodeUnitsOrNodeOffset {
    /// Convert this UTF-32 (`char`) offset in `string` to an UTF-16 offset
    pub fn to_utf16_code_units_in(self, _: AssumeUnder4GB, string: &str) -> Utf16CodeUnits {
        Utf32CodeUnits(self.0).to_utf16_code_units_in(AssumeUnder4GB, string)
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
            Utf16CodeUnits::length_of(AssumeUnder4GB, ""),
            Utf16CodeUnits(0)
        );
        assert_eq!(
            Utf16CodeUnits::length_of(AssumeUnder4GB, "a"),
            Utf16CodeUnits(1)
        );
        assert_eq!(
            Utf16CodeUnits::length_of(AssumeUnder4GB, "é"),
            Utf16CodeUnits(1)
        );
        assert_eq!(
            Utf16CodeUnits::length_of(AssumeUnder4GB, "字"),
            Utf16CodeUnits(1)
        );
        assert_eq!(
            Utf16CodeUnits::length_of(AssumeUnder4GB, "\u{1F4A9}"),
            Utf16CodeUnits(2)
        );
        assert_eq!(
            Utf16CodeUnits::length_of(AssumeUnder4GB, "\u{1F4A9}字éa"),
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
        // before the whole pair. Should this be an error instead?
        assert_eq!(
            Utf16CodeUnits(4).to_utf32_code_units_in(s),
            Utf32CodeUnits(3)
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

    #[test]
    fn test_utf32_to_utf16() {
        let string = "aé字\u{1F4A9}";
        assert_eq!(
            Utf32CodeUnits(0).to_utf16_code_units_in(AssumeUnder4GB, string),
            Utf16CodeUnits(0),
        );
        assert_eq!(
            Utf32CodeUnits(1).to_utf16_code_units_in(AssumeUnder4GB, string),
            Utf16CodeUnits(1),
        );
        assert_eq!(
            Utf32CodeUnits(2).to_utf16_code_units_in(AssumeUnder4GB, string),
            Utf16CodeUnits(2),
        );
        assert_eq!(
            Utf32CodeUnits(3).to_utf16_code_units_in(AssumeUnder4GB, string),
            Utf16CodeUnits(3),
        );

        assert_eq!(
            Utf32CodeUnits(4).to_utf16_code_units_in(AssumeUnder4GB, string),
            Utf16CodeUnits(5),
        );

        // This 32-bit offset is out of bounds. We clamp to the nearest valid 16-bit offset,
        // a.k.a the UTF-16 length. Should this be an error instead?
        assert_eq!(
            Utf32CodeUnits(6).to_utf16_code_units_in(AssumeUnder4GB, string),
            Utf16CodeUnits(5),
        );
        assert_eq!(
            Utf32CodeUnits(1000).to_utf16_code_units_in(AssumeUnder4GB, string),
            Utf16CodeUnits(5),
        );
    }
}
