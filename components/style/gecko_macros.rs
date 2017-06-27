/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Various macro helpers that depend on Gecko.

/// This macro implemets generic Rect for style coord such LengthOrNumber to converts from/to Servo to/from Gecko.
macro_rules! impl_rect_conversions {
    ($data_type: ident) => {
        impl ::values::generics::rect::Rect<$data_type> {
            pub fn to_gecko_rect(&self, sides: &mut ::gecko_bindings::structs::nsStyleSides) {
                use gecko::values::GeckoStyleCoordConvertible;

                self.0.to_gecko_style_coord(&mut sides.data_at_mut(0));
                self.1.to_gecko_style_coord(&mut sides.data_at_mut(1));
                self.2.to_gecko_style_coord(&mut sides.data_at_mut(2));
                self.3.to_gecko_style_coord(&mut sides.data_at_mut(3));
            }

            pub fn from_gecko_rect(sides: &::gecko_bindings::structs::nsStyleSides)
                                   -> Option<::values::generics::rect::Rect<$data_type>> {
                use gecko::values::GeckoStyleCoordConvertible;

                Some(
                    ::values::generics::rect::Rect::new(
                        $data_type::from_gecko_style_coord(&sides.data_at(0)).expect("coord[${0}] has valid data"),
                        $data_type::from_gecko_style_coord(&sides.data_at(1)).expect("coord[${1}] has valid data"),
                        $data_type::from_gecko_style_coord(&sides.data_at(2)).expect("coord[${2}] has valid data"),
                        $data_type::from_gecko_style_coord(&sides.data_at(3)).expect("coord[${3}] has valid data")
                    )
                )
            }
        }
    }
}
