/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery to conditionally expose things.

use js::jsapi::{HandleObject, JSContext};
use servo_config::prefs::PREFS;

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
    ///
    /// The passed handle is the object on which the value may be exposed.
    pub unsafe fn expose(&self, cx: *mut JSContext, obj: HandleObject) -> Option<T> {
        if self.condition.is_satisfied(cx, obj) {
            Some(self.value)
        } else {
            None
        }
    }
}

/// A condition to expose things.
pub enum Condition {
    /// The condition is satisfied if the function returns true.
    Func(unsafe fn(*mut JSContext, HandleObject) -> bool),
    /// The condition is satisfied if the preference is set.
    Pref(&'static str),
    /// The condition is always satisfied.
    Satisfied,
}

impl Condition {
    unsafe fn is_satisfied(&self, cx: *mut JSContext, obj: HandleObject) -> bool {
        match *self {
            Condition::Pref(name) => PREFS.get(name).as_boolean().unwrap_or(false),
            Condition::Func(f) => f(cx, obj),
            Condition::Satisfied => true,
        }
    }
}
