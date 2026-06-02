/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Machinery to conditionally expose things.

use js::rust::HandleObject;
use servo_config::prefs::get;

use crate::DomTypes;
use crate::codegen::Globals::Globals;
use crate::interface::is_exposed_in;
use crate::interfaces::GlobalScopeHelpers;
use crate::realms::{AlreadyInRealm, InRealm};
use crate::script_runtime::JSContext;

/// A container with a list of conditions.
pub(crate) struct Guard<T: Clone + Copy> {
    conditions: &'static [Condition],
    value: T,
}

impl<T: Clone + Copy> Guard<T> {
    /// Construct a new guarded value.
    pub(crate) const fn new(conditions: &'static [Condition], value: T) -> Self {
        Guard { conditions, value }
    }

    /// Expose the value if the conditions are satisfied.
    ///
    /// The passed handle is the object on which the value may be exposed.
    pub(crate) fn expose<D: DomTypes>(
        &self,
        cx: JSContext,
        obj: HandleObject,
        global: HandleObject,
    ) -> Option<T> {
        let mut exposed_on_global = false;
        let conditions_satisfied = self.conditions.iter().all(|c| match c {
            Condition::Satisfied => {
                exposed_on_global = true;
                true
            },
            // If there are multiple Exposed conditions, we just need one of them to be true
            Condition::Exposed(globals) => {
                exposed_on_global |= is_exposed_in(global, *globals);
                true
            },
            _ => c.is_satisfied::<D>(cx, obj, global),
        });

        if conditions_satisfied && exposed_on_global {
            Some(self.value)
        } else {
            None
        }
    }
}

/// A condition to expose things.
#[derive(Clone, Copy)]
pub(crate) enum Condition {
    /// The condition is satisfied if the function returns true.
    Func(fn(JSContext, HandleObject) -> bool),
    /// The condition is satisfied if the preference is set.
    Pref(&'static str),
    // The condition is satisfied if the interface is exposed in the global.
    Exposed(Globals),
    SecureContext(),
    /// The condition is always satisfied.
    Satisfied,
}

fn is_secure_context<D: DomTypes>(cx: JSContext) -> bool {
    unsafe {
        let in_realm_proof = AlreadyInRealm::assert_for_cx(JSContext::from_ptr(*cx));
        D::GlobalScope::from_context(*cx, InRealm::Already(&in_realm_proof)).is_secure_context()
    }
}

impl Condition {
    pub(crate) fn is_satisfied<D: DomTypes>(
        &self,
        cx: JSContext,
        obj: HandleObject,
        global: HandleObject,
    ) -> bool {
        match *self {
            Condition::Pref(name) => get().get_value(name).try_into().unwrap_or(false),
            Condition::Func(f) => f(cx, obj),
            Condition::Exposed(globals) => is_exposed_in(global, globals),
            Condition::SecureContext() => is_secure_context::<D>(cx),
            Condition::Satisfied => true,
        }
    }
}
