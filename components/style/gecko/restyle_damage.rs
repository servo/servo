/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Gecko's restyle damage computation (aka change hints, aka `nsChangeHint`).

use gecko_bindings::bindings;
use gecko_bindings::structs;
use gecko_bindings::structs::{nsChangeHint, nsStyleContext};
use matching::{StyleChange, StyleDifference};
use properties::ComputedValues;
use servo_arc::Arc;
use std::ops::{BitAnd, BitOr, BitOrAssign, Not};

/// The representation of Gecko's restyle damage is just a wrapper over
/// `nsChangeHint`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeckoRestyleDamage(nsChangeHint);

impl GeckoRestyleDamage {
    /// Trivially construct a new `GeckoRestyleDamage`.
    pub fn new(raw: nsChangeHint) -> Self {
        GeckoRestyleDamage(raw)
    }

    /// Get the inner change hint for this damage.
    pub fn as_change_hint(&self) -> nsChangeHint {
        self.0
    }

    /// Get an empty change hint, that is (`nsChangeHint(0)`).
    pub fn empty() -> Self {
        GeckoRestyleDamage(nsChangeHint(0))
    }

    /// Returns whether this restyle damage represents the empty damage.
    pub fn is_empty(&self) -> bool {
        self.0 == nsChangeHint(0)
    }

    /// Computes the `StyleDifference` (including the appropriate change hint)
    /// given an old style (in the form of a `nsStyleContext`, and a new style
    /// (in the form of `ComputedValues`).
    ///
    /// Note that we could in theory just get two `ComputedValues` here and diff
    /// them, but Gecko has an interesting optimization when they mark accessed
    /// structs, so they effectively only diff structs that have ever been
    /// accessed from layout.
    pub fn compute_style_difference(
        source: &nsStyleContext,
        old_style: &ComputedValues,
        new_style: &Arc<ComputedValues>,
    ) -> StyleDifference {
        let mut any_style_changed: bool = false;
        let hint = unsafe {
            bindings::Gecko_CalcStyleDifference(old_style.as_style_context(),
                                                new_style.as_style_context(),
                                                source.mBits,
                                                &mut any_style_changed)
        };
        let change = if any_style_changed { StyleChange::Changed } else { StyleChange::Unchanged };
        StyleDifference::new(GeckoRestyleDamage(hint), change)
    }

    /// Returns true if this restyle damage contains all the damage of |other|.
    pub fn contains(self, other: Self) -> bool {
        self & other == other
    }

    /// Gets restyle damage to reconstruct the entire frame, subsuming all
    /// other damage.
    pub fn reconstruct() -> Self {
        GeckoRestyleDamage(structs::nsChangeHint_nsChangeHint_ReconstructFrame)
    }

    /// Assuming |self| is applied to an element, returns the set of damage that
    /// would be superfluous to apply for descendants.
    pub fn handled_for_descendants(self) -> Self {
        let hint = unsafe {
            bindings::Gecko_HintsHandledForDescendants(self.0)
        };
        GeckoRestyleDamage(hint)
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
