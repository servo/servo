/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::{JS, Root};
use dom::bindings::trace::JSTraceable;
use dom::globalscope::GlobalScope;
use js::jsapi::JSTracer;
use std::cell::RefCell;

thread_local!(static STACK: RefCell<Vec<StackEntry>> = RefCell::new(Vec::new()));

#[allow(unrooted_must_root)]
#[derive(JSTraceable)]
struct StackEntry {
    global: JS<GlobalScope>,
}

/// Traces the script settings stack.
pub unsafe fn trace(tracer: *mut JSTracer) {
    STACK.with(|stack| {
        stack.borrow().trace(tracer);
    })
}

/// RAII struct that pushes and pops entries from the script settings stack.
pub struct AutoEntryScript {
    global: *const GlobalScope,
}

impl AutoEntryScript {
    /// https://html.spec.whatwg.org/multipage/#prepare-to-run-script
    pub fn new(global: &GlobalScope) -> Self {
        STACK.with(|stack| {
            trace!("Prepare to run script with {:p}", global);
            let mut stack = stack.borrow_mut();
            stack.push(StackEntry {
                global: JS::from_ref(global),
            });
            AutoEntryScript {
                global: global as *const _,
            }
        })
    }
}

impl Drop for AutoEntryScript {
    /// https://html.spec.whatwg.org/multipage/#clean-up-after-running-script
    fn drop(&mut self) {
        STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            let entry = stack.pop().unwrap();
            assert_eq!(&*entry.global as *const GlobalScope,
                       self.global,
                       "Dropped AutoEntryScript out of order.");
            trace!("Clean up after running script with {:p}", self.global);
        })
    }
}

/// Returns the ["entry"] global object.
///
/// ["entry"]: https://html.spec.whatwg.org/multipage/#entry
pub fn entry_global() -> Root<GlobalScope> {
    STACK.with(|stack| {
        stack.borrow()
             .last()
             .map(|entry| Root::from_ref(&*entry.global))
    }).unwrap()
}
