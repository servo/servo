/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Bindings for CSS Rule objects

use byteorder::{BigEndian, WriteBytesExt};
use computed_values::{font_stretch, font_style};
use counter_style::{self, CounterBound};
use cssparser::UnicodeRange;
use font_face::{FontDisplay, FontWeight, Source};
use gecko_bindings::structs::{self, nsCSSValue};
use gecko_bindings::sugar::ns_css_value::ToNsCssValue;
use properties::longhands::font_language_override;
use std::str;
use values::computed::font::FamilyName;
use values::generics::font::FontTag;
use values::specified::font::AbsoluteFontWeight;
use values::specified::font::{SpecifiedFontFeatureSettings, SpecifiedFontVariationSettings};

impl<'a> ToNsCssValue for &'a FamilyName {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_string_from_atom(&self.name)
    }
}

impl<'a> ToNsCssValue for &'a AbsoluteFontWeight {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_font_weight(self.compute().0)
    }
}

impl ToNsCssValue for FontTag {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        let mut raw = [0u8; 4];
        (&mut raw[..]).write_u32::<BigEndian>(self.0).unwrap();
        nscssvalue.set_string(str::from_utf8(&raw).unwrap());
    }
}

impl<'a> ToNsCssValue for &'a SpecifiedFontFeatureSettings {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        if self.0.is_empty() {
            nscssvalue.set_normal();
            return;
        }

        nscssvalue.set_pair_list(self.0.iter().map(|entry| {
            let mut index = nsCSSValue::null();
            index.set_integer(entry.value.value());
            (entry.tag.into(), index)
        }))
    }
}

impl<'a> ToNsCssValue for &'a SpecifiedFontVariationSettings {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        if self.0.is_empty() {
            nscssvalue.set_normal();
            return;
        }

        nscssvalue.set_pair_list(self.0.iter().map(|entry| {
            let mut value = nsCSSValue::null();
            value.set_number(entry.value.into());
            (entry.tag.into(), value)
        }))
    }
}

impl<'a> ToNsCssValue for &'a FontWeight {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        let FontWeight(ref first, ref second) = *self;

        let second = match *second {
            None => {
                nscssvalue.set_from(first);
                return;
            }
            Some(ref second) => second,
        };

        let mut a = nsCSSValue::null();
        let mut b = nsCSSValue::null();

        a.set_from(first);
        b.set_from(second);

        nscssvalue.set_pair(&a, &b);
    }
}

impl<'a> ToNsCssValue for &'a font_language_override::SpecifiedValue {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        match *self {
            font_language_override::SpecifiedValue::Normal => nscssvalue.set_normal(),
            font_language_override::SpecifiedValue::Override(ref lang) => {
                nscssvalue.set_string(&*lang)
            },
            // This path is unreachable because the descriptor is only specified by the user.
            font_language_override::SpecifiedValue::System(_) => unreachable!(),
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
            impl<'a> ToNsCssValue for &'a $prop::T {
                fn convert(self, nscssvalue: &mut nsCSSValue) {
                    nscssvalue.set_enum(match *self {
                        $( $prop::T::$servo => structs::$gecko as i32, )+
                    })
                }
            }
        )+
    }
}

map_enum! {
    font_style {
        Normal => NS_FONT_STYLE_NORMAL,
        Italic => NS_FONT_STYLE_ITALIC,
        Oblique => NS_FONT_STYLE_OBLIQUE,
    }

    font_stretch {
        Normal          => NS_FONT_STRETCH_NORMAL,
        UltraCondensed  => NS_FONT_STRETCH_ULTRA_CONDENSED,
        ExtraCondensed  => NS_FONT_STRETCH_EXTRA_CONDENSED,
        Condensed       => NS_FONT_STRETCH_CONDENSED,
        SemiCondensed   => NS_FONT_STRETCH_SEMI_CONDENSED,
        SemiExpanded    => NS_FONT_STRETCH_SEMI_EXPANDED,
        Expanded        => NS_FONT_STRETCH_EXPANDED,
        ExtraExpanded   => NS_FONT_STRETCH_EXTRA_EXPANDED,
        UltraExpanded   => NS_FONT_STRETCH_ULTRA_EXPANDED,
    }
}

impl<'a> ToNsCssValue for &'a Vec<Source> {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        let src_len = self.iter().fold(0, |acc, src| {
            acc + match *src {
                // Each format hint takes one position in the array of mSrc.
                Source::Url(ref url) => url.format_hints.len() + 1,
                Source::Local(_) => 1,
            }
        });
        let mut target_srcs = nscssvalue
            .set_array(src_len as i32)
            .as_mut_slice()
            .iter_mut();
        macro_rules! next {
            () => {
                target_srcs
                    .next()
                    .expect("Length of target_srcs should be enough")
            };
        }
        for src in self.iter() {
            match *src {
                Source::Url(ref url) => {
                    next!().set_url(&url.url);
                    for hint in url.format_hints.iter() {
                        next!().set_font_format(&hint);
                    }
                },
                Source::Local(ref family) => {
                    next!().set_local_font(&family.name);
                },
            }
        }
        debug_assert!(target_srcs.next().is_none(), "Should have filled all slots");
    }
}

impl<'a> ToNsCssValue for &'a Vec<UnicodeRange> {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        let target_ranges = nscssvalue
            .set_array((self.len() * 2) as i32)
            .as_mut_slice()
            .chunks_mut(2);
        for (range, target) in self.iter().zip(target_ranges) {
            target[0].set_integer(range.start as i32);
            target[1].set_integer(range.end as i32);
        }
    }
}

impl<'a> ToNsCssValue for &'a FontDisplay {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_enum(match *self {
            FontDisplay::Auto => structs::NS_FONT_DISPLAY_AUTO,
            FontDisplay::Block => structs::NS_FONT_DISPLAY_BLOCK,
            FontDisplay::Swap => structs::NS_FONT_DISPLAY_SWAP,
            FontDisplay::Fallback => structs::NS_FONT_DISPLAY_FALLBACK,
            FontDisplay::Optional => structs::NS_FONT_DISPLAY_OPTIONAL,
        } as i32)
    }
}

impl<'a> ToNsCssValue for &'a counter_style::System {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        use counter_style::System::*;
        match *self {
            Cyclic => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_CYCLIC as i32),
            Numeric => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_NUMERIC as i32),
            Alphabetic => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_ALPHABETIC as i32),
            Symbolic => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_SYMBOLIC as i32),
            Additive => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_ADDITIVE as i32),
            Fixed {
                ref first_symbol_value,
            } => {
                let mut a = nsCSSValue::null();
                let mut b = nsCSSValue::null();
                a.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_FIXED as i32);
                b.set_integer(first_symbol_value.map_or(1, |v| v.value()));
                nscssvalue.set_pair(&a, &b);
            },
            Extends(ref other) => {
                let mut a = nsCSSValue::null();
                let mut b = nsCSSValue::null();
                a.set_enum(structs::NS_STYLE_COUNTER_SYSTEM_EXTENDS as i32);
                b.set_atom_ident(other.0.clone());
                nscssvalue.set_pair(&a, &b);
            },
        }
    }
}

impl<'a> ToNsCssValue for &'a counter_style::Negative {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        if let Some(ref second) = self.1 {
            let mut a = nsCSSValue::null();
            let mut b = nsCSSValue::null();
            a.set_from(&self.0);
            b.set_from(second);
            nscssvalue.set_pair(&a, &b);
        } else {
            nscssvalue.set_from(&self.0)
        }
    }
}

impl<'a> ToNsCssValue for &'a counter_style::Symbol {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        match *self {
            counter_style::Symbol::String(ref s) => nscssvalue.set_string(s),
            counter_style::Symbol::Ident(ref s) => nscssvalue.set_ident_from_atom(&s.0),
        }
    }
}

impl<'a> ToNsCssValue for &'a counter_style::Ranges {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        if self.0.is_empty() {
            nscssvalue.set_auto();
        } else {
            nscssvalue.set_pair_list(self.0.iter().map(|range| {
                fn set_bound(bound: CounterBound, nscssvalue: &mut nsCSSValue) {
                    if let CounterBound::Integer(finite) = bound {
                        nscssvalue.set_integer(finite.value())
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

impl<'a> ToNsCssValue for &'a counter_style::Pad {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        let mut min_length = nsCSSValue::null();
        let mut pad_with = nsCSSValue::null();
        min_length.set_integer(self.0.value());
        pad_with.set_from(&self.1);
        nscssvalue.set_pair(&min_length, &pad_with);
    }
}

impl<'a> ToNsCssValue for &'a counter_style::Fallback {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_atom_ident(self.0 .0.clone())
    }
}

impl<'a> ToNsCssValue for &'a counter_style::Symbols {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_list(self.0.iter().map(|item| {
            let mut value = nsCSSValue::null();
            value.set_from(item);
            value
        }));
    }
}

impl<'a> ToNsCssValue for &'a counter_style::AdditiveSymbols {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_pair_list(self.0.iter().map(|tuple| {
            let mut weight = nsCSSValue::null();
            let mut symbol = nsCSSValue::null();
            weight.set_integer(tuple.weight.value());
            symbol.set_from(&tuple.symbol);
            (weight, symbol)
        }));
    }
}

impl<'a> ToNsCssValue for &'a counter_style::SpeakAs {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        use counter_style::SpeakAs::*;
        match *self {
            Auto => nscssvalue.set_auto(),
            Bullets => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SPEAKAS_BULLETS as i32),
            Numbers => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SPEAKAS_NUMBERS as i32),
            Words => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SPEAKAS_WORDS as i32),
            Other(ref other) => nscssvalue.set_atom_ident(other.0.clone()),
        }
    }
}
