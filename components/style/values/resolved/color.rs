/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Resolved color values.

use super::{Context, ToResolvedValue};

use crate::color::AbsoluteColor;
use crate::values::computed::color as computed;
use crate::values::generics::color as generics;

impl ToResolvedValue for computed::Color {
    // A resolved color value is an rgba color, with currentcolor resolved.
    type ResolvedValue = AbsoluteColor;

    #[inline]
    fn to_resolved_value(self, context: &Context) -> Self::ResolvedValue {
        context.style.resolve_color(self)
    }

    #[inline]
    fn from_resolved_value(resolved: Self::ResolvedValue) -> Self {
        generics::Color::Absolute(resolved)
    }
}

impl ToResolvedValue for computed::CaretColor {
    // A resolved caret-color value is an rgba color, with auto resolving to
    // currentcolor.
    type ResolvedValue = AbsoluteColor;

    #[inline]
    fn to_resolved_value(self, context: &Context) -> Self::ResolvedValue {
        let color = match self.0 {
            generics::ColorOrAuto::Color(color) => color,
            generics::ColorOrAuto::Auto => generics::Color::currentcolor(),
        };
        color.to_resolved_value(context)
    }

    #[inline]
    fn from_resolved_value(resolved: Self::ResolvedValue) -> Self {
        generics::CaretColor(generics::ColorOrAuto::Color(
            computed::Color::from_resolved_value(resolved),
        ))
    }
}
