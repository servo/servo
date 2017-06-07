/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust helpers for Gecko's `nsCSSShadowItem`.

use app_units::Au;
use cssparser::Color;
use gecko::values::{convert_rgba_to_nscolor, convert_nscolor_to_rgba};
use gecko_bindings::structs::nsCSSShadowItem;
use values::computed::Shadow;

impl nsCSSShadowItem {
    /// Set this item to the given shadow value.
    pub fn set_from_shadow(&mut self, other: Shadow) {
        self.mXOffset = other.offset_x.0;
        self.mYOffset = other.offset_y.0;
        self.mRadius = other.blur_radius.0;
        self.mSpread = other.spread_radius.0;
        self.mInset = other.inset;
        self.mColor = match other.color {
            Color::RGBA(rgba) => {
                self.mHasColor = true;
                convert_rgba_to_nscolor(&rgba)
            },
            // TODO handle currentColor
            // https://bugzilla.mozilla.org/show_bug.cgi?id=760345
            Color::CurrentColor => 0,
        }
    }

    /// Generate shadow value from this shadow item.
    pub fn to_shadow(&self) -> Shadow {
        Shadow {
            offset_x: Au(self.mXOffset),
            offset_y: Au(self.mYOffset),
            blur_radius: Au(self.mRadius),
            spread_radius: Au(self.mSpread),
            inset: self.mInset,
            color: Color::RGBA(convert_nscolor_to_rgba(self.mColor)),
        }
    }
}
