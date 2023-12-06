/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

//! Different kind of helpers to interact with Gecko values.

use crate::color::{AbsoluteColor, ColorSpace};
use crate::counter_style::{Symbol, Symbols};
use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs::CounterStylePtr;
use crate::values::generics::CounterStyle;
use crate::values::Either;
use crate::Atom;

/// Convert a color value to `nscolor`.
pub fn convert_absolute_color_to_nscolor(color: &AbsoluteColor) -> u32 {
    let srgb = color.to_color_space(ColorSpace::Srgb);
    u32::from_le_bytes([
        (srgb.components.0 * 255.0).round() as u8,
        (srgb.components.1 * 255.0).round() as u8,
        (srgb.components.2 * 255.0).round() as u8,
        (srgb.alpha * 255.0).round() as u8,
    ])
}

/// Convert a given `nscolor` to a Servo AbsoluteColor value.
pub fn convert_nscolor_to_absolute_color(color: u32) -> AbsoluteColor {
    let [r, g, b, a] = color.to_le_bytes();
    AbsoluteColor::srgb(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    )
}

impl CounterStyle {
    /// Convert this counter style to a Gecko CounterStylePtr.
    #[inline]
    pub fn to_gecko_value(&self, gecko_value: &mut CounterStylePtr) {
        unsafe { bindings::Gecko_CounterStyle_ToPtr(self, gecko_value) }
    }

    /// Convert Gecko CounterStylePtr to CounterStyle or String.
    pub fn from_gecko_value(gecko_value: &CounterStylePtr) -> Either<Self, String> {
        use crate::values::CustomIdent;

        let name = unsafe { bindings::Gecko_CounterStyle_GetName(gecko_value) };
        if !name.is_null() {
            let name = unsafe { Atom::from_raw(name) };
            debug_assert_ne!(name, atom!("none"));
            Either::First(CounterStyle::Name(CustomIdent(name)))
        } else {
            let anonymous =
                unsafe { bindings::Gecko_CounterStyle_GetAnonymous(gecko_value).as_ref() }.unwrap();
            let symbols = &anonymous.mSymbols;
            if anonymous.mSingleString {
                debug_assert_eq!(symbols.len(), 1);
                Either::Second(symbols[0].to_string())
            } else {
                let symbol_type = anonymous.mSymbolsType;
                let symbols = symbols
                    .iter()
                    .map(|gecko_symbol| Symbol::String(gecko_symbol.to_string().into()))
                    .collect();
                Either::First(CounterStyle::Symbols(symbol_type, Symbols(symbols)))
            }
        }
    }
}
