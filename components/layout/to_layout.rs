/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style::values::computed;
use webrender_traits::BorderStyle;

pub trait ToLayout {
    type Type;
    fn to_layout(&self) -> Self::Type;
}

impl ToLayout for computed::BorderStyle {
    type Type = BorderStyle;
    fn to_layout(&self) -> Self::Type {
        use webrender_traits::BorderStyle::*;
        match *self {
            computed::BorderStyle::none => None,
            computed::BorderStyle::solid => Solid,
            computed::BorderStyle::double => Double,
            computed::BorderStyle::dotted => Dotted,
            computed::BorderStyle::dashed => Dashed,
            computed::BorderStyle::hidden => Hidden,
            computed::BorderStyle::groove => Groove,
            computed::BorderStyle::ridge => Ridge,
            computed::BorderStyle::inset => Inset,
            computed::BorderStyle::outset => Outset,
        }
    }
}