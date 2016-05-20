/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery to conditionally expose things.

use util::prefs::get_pref;

/// A container with a condition.
pub struct Guard<T: Clone + Copy> {
    condition: Condition,
    value: T,
}

impl<T: Clone + Copy> Guard<T> {
    /// Construct a new guarded value.
    pub const fn new(condition: Condition, value: T) -> Self {
        Guard {
            condition: condition,
            value: value,
        }
    }

    /// Expose the value if the condition is satisfied.
    pub fn expose(&self) -> Option<T> {
        if self.condition.is_satisfied() {
            Some(self.value)
        } else {
            None
        }
    }
}

/// A condition to expose things.
pub enum Condition {
    /// The condition is satisfied if the preference is set.
    Pref(&'static str),
    /// The condition is always satisfied.
    Satisfied,
}

impl Condition {
    fn is_satisfied(&self) -> bool {
        match *self {
            Condition::Pref(name) => get_pref(name).as_boolean().unwrap_or(false),
            Condition::Satisfied => true,
        }
    }
}
