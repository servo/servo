/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gecko_bindings::bindings;
use gecko_bindings::structs;
use gecko_bindings::structs::{nsChangeHint, nsStyleContext};
use gecko_bindings::sugar::ownership::FFIArcHelpers;
use properties::ComputedValues;
use std::ops::{BitOr, BitOrAssign};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeckoRestyleDamage(nsChangeHint);

impl GeckoRestyleDamage {
    pub fn new(raw: nsChangeHint) -> Self {
        GeckoRestyleDamage(raw)
    }

    pub fn as_change_hint(&self) -> nsChangeHint {
        self.0
    }

    pub fn empty() -> Self {
        GeckoRestyleDamage(nsChangeHint(0))
    }

    pub fn is_empty(&self) -> bool {
        self.0 == nsChangeHint(0)
    }

    pub fn compute(source: &nsStyleContext,
                   new_style: &Arc<ComputedValues>) -> Self {
        let context = source as *const nsStyleContext as *mut nsStyleContext;
        let hint = unsafe {
            bindings::Gecko_CalcStyleDifference(context,
                                                new_style.as_borrowed_opt().unwrap())
        };
        GeckoRestyleDamage(hint)
    }

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
