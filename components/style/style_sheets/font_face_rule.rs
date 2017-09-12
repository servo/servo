/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// TODO(emilio): unify this, components/style/font_face.rs, and
// components/style/gecko/rules.rs
#![allow(missing_docs)]

#[cfg(feature = "servo")]
pub use font_face::FontFaceRuleData as FontFaceRule;
#[cfg(feature = "gecko")]
pub use gecko::rules::FontFaceRule;

impl FontFaceRule {
    #[cfg(feature = "servo")]
    pub fn clone_conditionally_gecko_or_servo(&self) -> FontFaceRule {
        self.clone()
    }

    #[cfg(feature = "gecko")]
    pub fn clone_conditionally_gecko_or_servo(&self) -> FontFaceRule {
        self.deep_clone_from_gecko()
    }
}
