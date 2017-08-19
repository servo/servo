/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust helpers for Gecko's `nsCSSShadowItem`.

use app_units::Au;
use gecko::values::{convert_rgba_to_nscolor, convert_nscolor_to_rgba};
use gecko_bindings::structs::nsCSSShadowItem;
use values::computed::RGBAColor;
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
            base: self.extract_simple_shadow(),
            spread: Au(self.mSpread),
            inset: self.mInset,
        }
    }

    /// Sets this item from the given simple shadow.
    #[inline]
    pub fn set_from_simple_shadow(&mut self, shadow: SimpleShadow) {
        self.mXOffset = shadow.horizontal.0;
        self.mYOffset = shadow.vertical.0;
        self.mRadius = shadow.blur.value();
        self.mSpread = 0;
        self.mInset = false;
        if let Some(color) = shadow.color {
            self.mHasColor = true;
            self.mColor = convert_rgba_to_nscolor(&color);
        } else {
            // TODO handle currentColor
            // https://bugzilla.mozilla.org/show_bug.cgi?id=760345
            self.mHasColor = false;
            self.mColor = 0;
        }
    }

    #[inline]
    fn extract_color(&self) -> Option<RGBAColor> {
        if self.mHasColor {
            Some(convert_nscolor_to_rgba(self.mColor))
        } else {
            None
        }
    }

    /// Gets a simple shadow from this item.
    #[inline]
    fn extract_simple_shadow(&self) -> SimpleShadow {
        SimpleShadow {
            color: self.extract_color(),
            horizontal: Au(self.mXOffset),
            vertical: Au(self.mYOffset),
            blur: Au(self.mRadius).into(),
        }
    }

    /// Returns this item as a simple shadow.
    #[inline]
    pub fn to_simple_shadow(&self) -> SimpleShadow {
        debug_assert_eq!(self.mSpread, 0);
        debug_assert_eq!(self.mInset, false);
        self.extract_simple_shadow()
    }
}
