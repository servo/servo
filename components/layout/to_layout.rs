/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style::computed_values::mix_blend_mode;
use style::values::computed;
use webrender_traits::{BorderStyle, MixBlendMode};

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

impl ToLayout for mix_blend_mode::T {
    type Type = MixBlendMode;
    fn to_layout(&self) -> MixBlendMode {
        use webrender_traits::MixBlendMode::*;
        match *self {
            mix_blend_mode::T::normal => Normal,
            mix_blend_mode::T::multiply => Multiply,
            mix_blend_mode::T::screen => Screen,
            mix_blend_mode::T::overlay => Overlay,
            mix_blend_mode::T::darken => Darken,
            mix_blend_mode::T::lighten => Lighten,
            mix_blend_mode::T::color_dodge => ColorDodge,
            mix_blend_mode::T::color_burn => ColorBurn,
            mix_blend_mode::T::hard_light => HardLight,
            mix_blend_mode::T::soft_light => SoftLight,
            mix_blend_mode::T::difference => Difference,
            mix_blend_mode::T::exclusion => Exclusion,
            mix_blend_mode::T::hue => Hue,
            mix_blend_mode::T::saturation => Saturation,
            mix_blend_mode::T::color => Color,
            mix_blend_mode::T::luminosity => Luminosity,
        }
    }
}
