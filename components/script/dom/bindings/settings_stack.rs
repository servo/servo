/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use js::jsapi::{GetScriptedCallerGlobal, JSTracer};
use js::rust::Runtime;
use script_bindings::settings_stack::*;

//use script_bindings::interfaces::{DomHelpers, GlobalScopeHelpers};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::globalscope::GlobalScope;

thread_local!(pub(super) static STACK: RefCell<Vec<StackEntry<crate::DomTypeHolder>>> = const {
    RefCell::new(Vec::new())
});

/// Traces the script settings stack.
pub(crate) unsafe fn trace(tracer: *mut JSTracer) {
    STACK.with(|stack| {
        stack.borrow().trace(tracer);
    })
}

pub(crate) fn is_execution_stack_empty() -> bool {
    STACK.with(|stack| stack.borrow().is_empty())
}

pub(crate) type AutoEntryScript = GenericAutoEntryScript<crate::DomTypeHolder>;

/// Returns the ["entry"] global object.
///
/// ["entry"]: https://html.spec.whatwg.org/multipage/#entry
pub(crate) fn entry_global() -> DomRoot<GlobalScope> {
    STACK
        .with(|stack| {
            stack
                .borrow()
                .iter()
                .rev()
                .find(|entry| entry.kind == StackEntryKind::Entry)
                .map(|entry| DomRoot::from_ref(&*entry.global))
        })
        .unwrap()
}

pub type AutoIncumbentScript = GenericAutoIncumbentScript<crate::DomTypeHolder>;

/// Returns the ["incumbent"] global object.
///
/// ["incumbent"]: https://html.spec.whatwg.org/multipage/#incumbent
pub(crate) fn incumbent_global() -> Option<DomRoot<GlobalScope>> {
    // https://html.spec.whatwg.org/multipage/#incumbent-settings-object

    // Step 1, 3: See what the JS engine has to say. If we've got a scripted
    // caller override in place, the JS engine will lie to us and pretend that
    // there's nothing on the JS stack, which will cause us to check the
    // incumbent script stack below.
    unsafe {
        let Some(cx) = Runtime::get() else {
            // It's not meaningful to return a global object if the runtime
            // no longer exists.
            return None;
        };
        let global = GetScriptedCallerGlobal(cx.as_ptr());
        if !global.is_null() {
            return Some(GlobalScope::from_object(global));
        }
    }

    // Step 2: nothing from the JS engine. Let's use whatever's on the explicit stack.
    STACK.with(|stack| {
        stack
            .borrow()
            .last()
            .map(|entry| DomRoot::from_ref(&*entry.global))
    })
}
