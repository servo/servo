/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Gecko's restyle damage computation (aka change hints, aka `nsChangeHint`).

use gecko_bindings::bindings;
use gecko_bindings::structs;
use gecko_bindings::structs::{nsChangeHint, nsStyleContext};
use gecko_bindings::sugar::ownership::FFIArcHelpers;
use properties::ComputedValues;
use std::ops::{BitOr, BitOrAssign};
use std::sync::Arc;

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

    /// Computes a change hint given an old style (in the form of a
    /// `nsStyleContext`, and a new style (in the form of `ComputedValues`).
    ///
    /// Note that we could in theory just get two `ComputedValues` here and diff
    /// them, but Gecko has an interesting optimization when they mark accessed
    /// structs, so they effectively only diff structs that have ever been
    /// accessed from layout.
    pub fn compute(source: &nsStyleContext,
                   new_style: &Arc<ComputedValues>) -> Self {
        // TODO(emilio): Const-ify this?
        let context = source as *const nsStyleContext as *mut nsStyleContext;
        let hint = unsafe {
            bindings::Gecko_CalcStyleDifference(context,
                                                new_style.as_borrowed_opt().unwrap())
        };
        GeckoRestyleDamage(hint)
    }

    /// Get a restyle damage that represents the maximum action to be taken
    /// (rebuild and reflow).
    pub fn rebuild_and_reflow() -> Self {
        GeckoRestyleDamage(structs::nsChangeHint_nsChangeHint_ReconstructFrame)
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
