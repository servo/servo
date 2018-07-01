/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust helpers for Gecko's `nsCSSShadowItem`.

use app_units::Au;
use gecko_bindings::structs::nsCSSShadowItem;
use values::computed::effects::{BoxShadow, SimpleShadow};

impl nsCSSShadowItem {
    /// Sets this item from the given box shadow.
    #[inline]
    pub fn set_from_box_shadow(&mut self, shadow: BoxShadow) {
        self.set_from_simple_shadow(shadow.base);
        self.mSpread = shadow.spread.to_i32_au();
        self.mInset = shadow.inset;
    }

    /// Returns this item as a box shadow.
    #[inline]
    pub fn to_box_shadow(&self) -> BoxShadow {
        BoxShadow {
            base: self.extract_simple_shadow(),
            spread: Au(self.mSpread).into(),
            inset: self.mInset,
        }
    }

    /// Sets this item from the given simple shadow.
    #[inline]
    pub fn set_from_simple_shadow(&mut self, shadow: SimpleShadow) {
        self.mXOffset = shadow.horizontal.to_i32_au();
        self.mYOffset = shadow.vertical.to_i32_au();
        self.mRadius = shadow.blur.0.to_i32_au();
        self.mSpread = 0;
        self.mInset = false;
        self.mColor = shadow.color.into();
    }

    /// Gets a simple shadow from this item.
    #[inline]
    fn extract_simple_shadow(&self) -> SimpleShadow {
        SimpleShadow {
            color: self.mColor.into(),
            horizontal: Au(self.mXOffset).into(),
            vertical: Au(self.mYOffset).into(),
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
