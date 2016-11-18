/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::TRestyleDamage;
use gecko_bindings::bindings;
use gecko_bindings::structs;
use gecko_bindings::structs::{nsChangeHint, nsStyleContext};
use gecko_bindings::sugar::ownership::FFIArcHelpers;
use properties::ComputedValues;
use std::ops::BitOr;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeckoRestyleDamage(nsChangeHint);

impl GeckoRestyleDamage {
    pub fn as_change_hint(&self) -> nsChangeHint {
        self.0
    }
}

impl TRestyleDamage for GeckoRestyleDamage {
    type PreExistingComputedValues = nsStyleContext;

    fn empty() -> Self {
        GeckoRestyleDamage(nsChangeHint(0))
    }

    fn compute(source: &nsStyleContext,
               new_style: &Arc<ComputedValues>) -> Self {
        let context = source as *const nsStyleContext as *mut nsStyleContext;
        let hint = unsafe {
            bindings::Gecko_CalcStyleDifference(context,
                                                new_style.as_borrowed_opt().unwrap())
        };
        GeckoRestyleDamage(hint)
    }

    fn rebuild_and_reflow() -> Self {
        GeckoRestyleDamage(structs::nsChangeHint_nsChangeHint_ReconstructFrame)
    }
}

impl BitOr for GeckoRestyleDamage {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        GeckoRestyleDamage(self.0 | other.0)
    }
}

