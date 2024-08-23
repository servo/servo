/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::thread;

use js::jsapi::{GetScriptedCallerGlobal, HideScriptedCaller, JSTracer, UnhideScriptedCaller};
use js::rust::Runtime;
pub use script_bindings::settings_stack::*;

use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

/// Returns the ["entry"] global object.
///
/// ["entry"]: https://html.spec.whatwg.org/multipage/#entry
pub fn entry_global() -> DomRoot<GlobalScope> {
    STACK
        .with(|stack| {
            stack
                .borrow()
                .iter()
                .rev()
                .find(|entry| entry.kind == StackEntryKind::Entry)
                .map(|entry| DomRoot::from_ref(unsafe { &*(entry.global as *const GlobalScope) }))
        })
        .unwrap()
}
