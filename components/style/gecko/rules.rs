/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Bindings for CSS Rule objects

use crate::counter_style::{self, CounterBound};
use crate::gecko_bindings::structs::{self, nsCSSValue};
use crate::gecko_bindings::sugar::ns_css_value::ToNsCssValue;

impl<'a> ToNsCssValue for &'a counter_style::System {
    fn convert(self, nscssvalue: &mut nsCSSValue) {
        use crate::counter_style::System::*;
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
        use crate::counter_style::SpeakAs::*;
        match *self {
            Auto => nscssvalue.set_auto(),
            Bullets => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SPEAKAS_BULLETS as i32),
            Numbers => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SPEAKAS_NUMBERS as i32),
            Words => nscssvalue.set_enum(structs::NS_STYLE_COUNTER_SPEAKAS_WORDS as i32),
            Other(ref other) => nscssvalue.set_atom_ident(other.0.clone()),
        }
    }
}
