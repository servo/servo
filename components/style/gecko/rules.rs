/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Bindings for CSS Rule objects

use byteorder::{BigEndian, WriteBytesExt};
use computed_values::{font_feature_settings, font_stretch, font_style, font_weight};
use computed_values::font_family::FamilyName;
use counter_style;
use cssparser::UnicodeRange;
use font_face::{FontFaceRuleData, Source, FontDisplay};
use gecko_bindings::bindings;
use gecko_bindings::structs::{self, nsCSSFontFaceRule, nsCSSValue};
use gecko_bindings::structs::{nsCSSCounterDesc, nsCSSCounterStyleRule};
use gecko_bindings::sugar::ns_css_value::ToNsCssValue;
use gecko_bindings::sugar::refptr::{RefPtr, UniqueRefPtr};
use shared_lock::{ToCssWithGuard, SharedRwLockReadGuard};
use std::{fmt, str};
use values::generics::FontSettings;

/// A @font-face rule
pub type FontFaceRule = RefPtr<nsCSSFontFaceRule>;

impl ToNsCssValue for FamilyName {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_string_from_atom(&self.name)
    }
}

impl ToNsCssValue for font_weight::T {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_integer(self as i32)
    }
}

impl ToNsCssValue for font_feature_settings::T {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        match self {
            FontSettings::Normal => nscssvalue.set_normal(),
            FontSettings::Tag(tags) => {
                nscssvalue.set_pair_list(tags.into_iter().map(|entry| {
                    let mut feature = nsCSSValue::null();
                    let mut raw = [0u8; 4];
                    (&mut raw[..]).write_u32::<BigEndian>(entry.tag).unwrap();
                    feature.set_string(str::from_utf8(&raw).unwrap());

                    let mut index = nsCSSValue::null();
                    index.set_integer(entry.value.0 as i32);

                    (feature, index)
                }))
            }
        }
    }
}

macro_rules! map_enum {
    (
        $(
            $prop:ident {
                $($servo:ident => $gecko:ident,)+
            }
        )+
    ) => {
        $(
            impl ToNsCssValue for $prop::T {
                fn convert(self, nscssvalue: &mut nsCSSValue) {
                    nscssvalue.set_enum(match self {
                        $( $prop::T::$servo => structs::$gecko as i32, )+
                    })
                }
            }
        )+
    }
}

map_enum! {
    font_style {
        normal => NS_FONT_STYLE_NORMAL,
        italic => NS_FONT_STYLE_ITALIC,
        oblique => NS_FONT_STYLE_OBLIQUE,
    }

    font_stretch {
        normal          => NS_FONT_STRETCH_NORMAL,
        ultra_condensed => NS_FONT_STRETCH_ULTRA_CONDENSED,
        extra_condensed => NS_FONT_STRETCH_EXTRA_CONDENSED,
        condensed       => NS_FONT_STRETCH_CONDENSED,
        semi_condensed  => NS_FONT_STRETCH_SEMI_CONDENSED,
        semi_expanded   => NS_FONT_STRETCH_SEMI_EXPANDED,
        expanded        => NS_FONT_STRETCH_EXPANDED,
        extra_expanded  => NS_FONT_STRETCH_EXTRA_EXPANDED,
        ultra_expanded  => NS_FONT_STRETCH_ULTRA_EXPANDED,
    }
}

impl ToNsCssValue for Vec<Source> {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        let src_len = self.iter().fold(0, |acc, src| {
            acc + match *src {
                // Each format hint takes one position in the array of mSrc.
                Source::Url(ref url) => url.format_hints.len() + 1,
                Source::Local(_) => 1,
            }
        });
        let mut target_srcs =
            nscssvalue.set_array(src_len as i32).as_mut_slice().iter_mut();
        macro_rules! next { () => {
            target_srcs.next().expect("Length of target_srcs should be enough")
        } }
        for src in self.into_iter() {
            match src {
                Source::Url(url) => {
                    next!().set_url(&url.url);
                    for hint in url.format_hints.iter() {
                        next!().set_font_format(&hint);
                    }
                }
                Source::Local(family) => {
                    next!().set_local_font(&family.name);
                }
            }
        }
        debug_assert!(target_srcs.next().is_none(), "Should have filled all slots");
    }
}

impl ToNsCssValue for Vec<UnicodeRange> {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        let target_ranges = nscssvalue
            .set_array((self.len() * 2) as i32)
            .as_mut_slice().chunks_mut(2);
        for (range, target) in self.iter().zip(target_ranges) {
            target[0].set_integer(range.start as i32);
            target[1].set_integer(range.end as i32);
        }
    }
}

impl ToNsCssValue for FontDisplay {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_enum(match self {
            FontDisplay::Auto => structs::NS_FONT_DISPLAY_AUTO,
            FontDisplay::Block => structs::NS_FONT_DISPLAY_BLOCK,
            FontDisplay::Swap => structs::NS_FONT_DISPLAY_SWAP,
            FontDisplay::Fallback => structs::NS_FONT_DISPLAY_FALLBACK,
            FontDisplay::Optional => structs::NS_FONT_DISPLAY_OPTIONAL,
        } as i32)
    }
}

impl FontFaceRule {
    /// Ask Gecko to deep clone the nsCSSFontFaceRule, and then construct
    /// a FontFaceRule object from it.
    pub fn deep_clone_from_gecko(&self) -> FontFaceRule {
        let result = unsafe {
            UniqueRefPtr::from_addrefed(
                bindings::Gecko_CSSFontFaceRule_Clone(self.get()))
        };
        result.get()
    }
}

impl From<FontFaceRuleData> for FontFaceRule {
    fn from(data: FontFaceRuleData) -> FontFaceRule {
        let mut result = unsafe {
            UniqueRefPtr::from_addrefed(bindings::Gecko_CSSFontFaceRule_Create(
                data.source_location.line as u32, data.source_location.column as u32
            ))
        };
        data.set_descriptors(&mut result.mDecl.mDescriptors);
        result.get()
    }
}

impl ToCssWithGuard for FontFaceRule {
    fn to_css<W>(&self, _guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        ns_auto_string!(css_text);
        unsafe {
            bindings::Gecko_CSSFontFaceRule_GetCssText(self.get(), &mut *css_text);
        }
        write!(dest, "{}", css_text)
    }
}

/// A @counter-style rule
pub type CounterStyleRule = RefPtr<nsCSSCounterStyleRule>;

impl CounterStyleRule {
    /// Ask Gecko to deep clone the nsCSSCounterStyleRule, and then construct
    /// a CounterStyleRule object from it.
    pub fn deep_clone_from_gecko(&self) -> CounterStyleRule {
        let result = unsafe {
            UniqueRefPtr::from_addrefed(
                bindings::Gecko_CSSCounterStyle_Clone(self.get()))
        };
        result.get()
    }
}

impl From<counter_style::CounterStyleRuleData> for CounterStyleRule {
    fn from(data: counter_style::CounterStyleRuleData) -> CounterStyleRule {
        let mut result = unsafe {
            UniqueRefPtr::from_addrefed(
                bindings::Gecko_CSSCounterStyle_Create(data.name().0.as_ptr()))
        };
        data.set_descriptors(&mut result.mValues);
        result.get()
    }
}

impl ToCssWithGuard for CounterStyleRule {
    fn to_css<W>(&self, _guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        ns_auto_string!(css_text);
        unsafe {
            bindings::Gecko_CSSCounterStyle_GetCssText(self.get(), &mut *css_text);
        }
        write!(dest, "{}", css_text)
    }
}

/// The type of nsCSSCounterStyleRule::mValues
pub type CounterStyleDescriptors = [nsCSSValue; nsCSSCounterDesc::eCSSCounterDesc_COUNT as usize];

impl ToNsCssValue for counter_style::System {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        use counter_style::System::*;
        match self {
            Cyclic => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_CYCLIC as i32),
            Numeric => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_NUMERIC as i32),
            Alphabetic => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_ALPHABETIC as i32),
            Symbolic => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_SYMBOLIC as i32),
            Additive => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_ADDITIVE as i32),
            Fixed { first_symbol_value } => {
                let mut a = nsCSSValue::null();
                let mut b = nsCSSValue::null();
                a.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_FIXED as i32);
                b.set_integer(first_symbol_value.unwrap_or(1));
                nscssvalue.set_pair(&a, &b);
            }
            Extends(other) => {
                let mut a = nsCSSValue::null();
                let mut b = nsCSSValue::null();
                a.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_EXTENDS as i32);
                b.set_atom_ident(other.0);
                nscssvalue.set_pair(&a, &b);
            }
        }
    }
}

impl ToNsCssValue for counter_style::Negative {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        if let Some(second) = self.1 {
            let mut a = nsCSSValue::null();
            let mut b = nsCSSValue::null();
            a.set_from(self.0);
            b.set_from(second);
            nscssvalue.set_pair(&a, &b);
        } else {
            nscssvalue.set_from(self.0)
        }
    }
}

impl ToNsCssValue for counter_style::Symbol {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        match self {
            counter_style::Symbol::String(s) => nscssvalue.set_string(&s),
            counter_style::Symbol::Ident(s) => nscssvalue.set_ident(&s),
        }
    }
}

impl ToNsCssValue for counter_style::Ranges {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        if self.0.is_empty() {
            nscssvalue.set_auto();
        } else {
            nscssvalue.set_pair_list(self.0.into_iter().map(|range| {
                fn set_bound(bound: Option<i32>, nscssvalue: &mut nsCSSValue) {
                    if let Some(finite) = bound {
                        nscssvalue.set_integer(finite)
                    } else {
                        nscssvalue.set_enum(structs::NS_STYLE_COUNTER_RANGE_INFINITE as i32)
                    }
                }
                let mut start = nsCSSValue::null();
                let mut end = nsCSSValue::null();
                set_bound(range.start, &mut start);
                set_bound(range.end, &mut end);
                (start, end)
            }));
        }
    }
}

impl ToNsCssValue for counter_style::Pad {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        let mut min_length = nsCSSValue::null();
        let mut pad_with = nsCSSValue::null();
        min_length.set_integer(self.0 as i32);
        pad_with.set_from(self.1);
        nscssvalue.set_pair(&min_length, &pad_with);
    }
}

impl ToNsCssValue for counter_style::Fallback {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_atom_ident(self.0 .0)
    }
}

impl ToNsCssValue for counter_style::Symbols {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_list(self.0.into_iter().map(|item| {
            let mut value = nsCSSValue::null();
            value.set_from(item);
            value
        }));
    }
}

impl ToNsCssValue for counter_style::AdditiveSymbols {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_pair_list(self.0.into_iter().map(|tuple| {
            let mut weight = nsCSSValue::null();
            let mut symbol = nsCSSValue::null();
            weight.set_integer(tuple.weight as i32);
            symbol.set_from(tuple.symbol);
            (weight, symbol)
        }));
    }
}

impl ToNsCssValue for counter_style::SpeakAs {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        use counter_style::SpeakAs::*;
        match self {
            Auto => nscssvalue.set_auto(),
            Bullets => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SPEAKAS_BULLETS as i32),
            Numbers => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SPEAKAS_NUMBERS as i32),
            Words => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SPEAKAS_WORDS as i32),
            Other(other) => nscssvalue.set_atom_ident(other.0),
        }
    }
}
