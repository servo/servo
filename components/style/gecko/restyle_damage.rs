/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Gecko's restyle damage computation (aka change hints, aka `nsChangeHint`).

use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs;
use crate::gecko_bindings::structs::nsChangeHint;
use crate::matching::{StyleChange, StyleDifference};
use crate::properties::ComputedValues;
use std::ops::{BitAnd, BitOr, BitOrAssign, Not};

/// The representation of Gecko's restyle damage is just a wrapper over
/// `nsChangeHint`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeckoRestyleDamage(nsChangeHint);

impl GeckoRestyleDamage {
    /// Trivially construct a new `GeckoRestyleDamage`.
    #[inline]
    pub fn new(raw: nsChangeHint) -> Self {
        GeckoRestyleDamage(raw)
    }

    /// Get the inner change hint for this damage.
    #[inline]
    pub fn as_change_hint(&self) -> nsChangeHint {
        self.0
    }

    /// Get an empty change hint, that is (`nsChangeHint(0)`).
    #[inline]
    pub fn empty() -> Self {
        GeckoRestyleDamage(nsChangeHint(0))
    }

    /// Returns whether this restyle damage represents the empty damage.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == nsChangeHint(0)
    }

    /// Computes the `StyleDifference` (including the appropriate change hint)
    /// given an old and a new style.
    pub fn compute_style_difference(
        old_style: &ComputedValues,
        new_style: &ComputedValues,
    ) -> StyleDifference {
        let mut any_style_changed = false;
        let mut reset_only = false;
        let hint = unsafe {
            bindings::Gecko_CalcStyleDifference(
                old_style.as_gecko_computed_style(),
                new_style.as_gecko_computed_style(),
                &mut any_style_changed,
                &mut reset_only,
            )
        };
        if reset_only && old_style.custom_properties() != new_style.custom_properties() {
            // The Gecko_CalcStyleDifference call only checks the non-custom
            // property structs, so we check the custom properties here. Since
            // they generate no damage themselves, we can skip this check if we
            // already know we had some inherited (regular) property
            // differences.
            any_style_changed = true;
            reset_only = false;
        }
        let change = if any_style_changed {
            StyleChange::Changed { reset_only }
        } else {
            StyleChange::Unchanged
        };
        let damage = GeckoRestyleDamage(nsChangeHint(hint));
        StyleDifference { damage, change }
    }

    /// Returns true if this restyle damage contains all the damage of |other|.
    pub fn contains(self, other: Self) -> bool {
        self & other == other
    }

    /// Gets restyle damage to reconstruct the entire frame, subsuming all
    /// other damage.
    pub fn reconstruct() -> Self {
        GeckoRestyleDamage(structs::nsChangeHint::nsChangeHint_ReconstructFrame)
    }
}

impl Default for GeckoRestyleDamage {
    fn default() -> Self {
        Self::empty()
    }
}

impl BitOr for GeckoRestyleDamage {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        GeckoRestyleDamage(self.0 | other.0)
    }
}

impl BitOrAssign for GeckoRestyleDamage {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

impl BitAnd for GeckoRestyleDamage {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        GeckoRestyleDamage(nsChangeHint((self.0).0 & (other.0).0))
    }
}

impl Not for GeckoRestyleDamage {
    type Output = Self;
    fn not(self) -> Self {
        GeckoRestyleDamage(nsChangeHint(!(self.0).0))
    }
}
