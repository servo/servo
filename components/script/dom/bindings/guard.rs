/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Machinery to conditionally expose things.

use crate::dom::bindings::codegen::InterfaceObjectMap;
use crate::dom::bindings::interface::is_exposed_in;
use crate::script_runtime::JSContext;
use js::rust::HandleObject;
use servo_config::prefs;

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
    pub fn expose(&self, cx: JSContext, obj: HandleObject, global: HandleObject) -> Option<T> {
        if self.condition.is_satisfied(cx, obj, global) {
            Some(self.value)
        } else {
            None
        }
    }
}

/// A condition to expose things.
pub enum Condition {
    /// The condition is satisfied if the function returns true.
    Func(fn(JSContext, HandleObject) -> bool),
    /// The condition is satisfied if the preference is set.
    Pref(&'static str),
    // The condition is satisfied if the interface is exposed in the global.
    Exposed(InterfaceObjectMap::Globals),
    /// The condition is always satisfied.
    Satisfied,
}

impl Condition {
    fn is_satisfied(&self, cx: JSContext, obj: HandleObject, global: HandleObject) -> bool {
        match *self {
            Condition::Pref(name) => prefs::pref_map().get(name).as_bool().unwrap_or(false),
            Condition::Func(f) => f(cx, obj),
            Condition::Exposed(globals) => is_exposed_in(global, globals),
            Condition::Satisfied => true,
        }
    }
}
