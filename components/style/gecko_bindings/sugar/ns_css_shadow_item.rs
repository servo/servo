/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust helpers for Gecko's `nsCSSShadowItem`.

use app_units::Au;
use gecko::values::{convert_rgba_to_nscolor, convert_nscolor_to_rgba};
use gecko_bindings::structs::nsCSSShadowItem;
use values::computed::Color;
use values::computed::effects::{BoxShadow, SimpleShadow};

impl nsCSSShadowItem {
    /// Sets this item from the given box shadow.
    #[inline]
    pub fn set_from_box_shadow(&mut self, shadow: BoxShadow) {
        self.set_from_simple_shadow(shadow.base);
        self.mSpread = shadow.spread.0;
        self.mInset = shadow.inset;
    }

    /// Returns this item as a box shadow.
    #[inline]
    pub fn to_box_shadow(&self) -> BoxShadow {
        BoxShadow {
            base: SimpleShadow {
                color: Color::rgba(convert_nscolor_to_rgba(self.mColor)),
                horizontal: Au(self.mXOffset),
                vertical: Au(self.mYOffset),
                blur: Au(self.mRadius),
            },
            spread: Au(self.mSpread),
            inset: self.mInset,
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
