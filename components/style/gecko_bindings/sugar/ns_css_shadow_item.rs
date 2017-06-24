/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust helpers for Gecko's `nsCSSShadowItem`.

use app_units::Au;
use gecko::values::{convert_rgba_to_nscolor, convert_nscolor_to_rgba};
use gecko_bindings::structs::nsCSSShadowItem;
use values::computed::{Color, Shadow};
use values::computed::effects::SimpleShadow;

impl nsCSSShadowItem {
    /// Set this item to the given shadow value.
    pub fn set_from_shadow(&mut self, other: Shadow) {
        self.mXOffset = other.offset_x.0;
        self.mYOffset = other.offset_y.0;
        self.mRadius = other.blur_radius.0;
        self.mSpread = other.spread_radius.0;
        self.mInset = other.inset;
        if other.color.is_currentcolor() {
            // TODO handle currentColor
            // https://bugzilla.mozilla.org/show_bug.cgi?id=760345
            self.mHasColor = false;
            self.mColor = 0;
        } else {
            self.mHasColor = true;
            self.mColor = convert_rgba_to_nscolor(&other.color.color);
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
            color: Color::rgba(convert_nscolor_to_rgba(self.mColor)),
        }
    }

    /// Sets this item from the given simple shadow.
    #[inline]
    pub fn set_from_simple_shadow(&mut self, shadow: SimpleShadow) {
        self.mXOffset = shadow.horizontal.0;
        self.mYOffset = shadow.vertical.0;
        self.mRadius = shadow.blur.0;
        self.mSpread = 0;
        self.mInset = false;
        if shadow.color.is_currentcolor() {
            // TODO handle currentColor
            // https://bugzilla.mozilla.org/show_bug.cgi?id=760345
            self.mHasColor = false;
            self.mColor = 0;
        } else {
            self.mHasColor = true;
            self.mColor = convert_rgba_to_nscolor(&shadow.color.color);
        }
    }

    /// Returns this item as a simple shadow.
    #[inline]
    pub fn to_simple_shadow(&self) -> SimpleShadow {
        debug_assert_eq!(self.mSpread, 0);
        debug_assert_eq!(self.mInset, false);
        SimpleShadow {
            color: Color::rgba(convert_nscolor_to_rgba(self.mColor)),
            horizontal: Au(self.mXOffset),
            vertical: Au(self.mYOffset),
            blur: Au(self.mRadius),
        }
    }
}
