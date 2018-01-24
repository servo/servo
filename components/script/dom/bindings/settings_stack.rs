/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::trace::JSTraceable;
use dom::globalscope::GlobalScope;
use js::jsapi::GetScriptedCallerGlobal;
use js::jsapi::HideScriptedCaller;
use js::jsapi::JSTracer;
use js::jsapi::UnhideScriptedCaller;
use js::rust::Runtime;
use std::cell::RefCell;
use std::thread;

thread_local!(static STACK: RefCell<Vec<StackEntry>> = RefCell::new(Vec::new()));

#[derive(Debug, Eq, JSTraceable, PartialEq)]
enum StackEntryKind {
    Incumbent,
    Entry,
}

#[allow(unrooted_must_root)]
#[derive(JSTraceable)]
struct StackEntry {
    global: Dom<GlobalScope>,
    kind: StackEntryKind,
}

/// Traces the script settings stack.
pub unsafe fn trace(tracer: *mut JSTracer) {
    STACK.with(|stack| {
        stack.borrow().trace(tracer);
    })
}

pub fn is_execution_stack_empty() -> bool {
    STACK.with(|stack| {
        stack.borrow().is_empty()
    })
}

/// RAII struct that pushes and pops entries from the script settings stack.
pub struct AutoEntryScript {
    global: DomRoot<GlobalScope>,
}

impl AutoEntryScript {
    /// <https://html.spec.whatwg.org/multipage/#prepare-to-run-script>
    pub fn new(global: &GlobalScope) -> Self {
        STACK.with(|stack| {
            trace!("Prepare to run script with {:p}", global);
            let mut stack = stack.borrow_mut();
            stack.push(StackEntry {
                global: Dom::from_ref(global),
                kind: StackEntryKind::Entry,
            });
            AutoEntryScript {
                global: DomRoot::from_ref(global),
            }
        })
    }
}

impl Drop for AutoEntryScript {
    /// <https://html.spec.whatwg.org/multipage/#clean-up-after-running-script>
    fn drop(&mut self) {
        STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            let entry = stack.pop().unwrap();
            assert_eq!(&*entry.global as *const GlobalScope,
                       &*self.global as *const GlobalScope,
                       "Dropped AutoEntryScript out of order.");
            assert_eq!(entry.kind, StackEntryKind::Entry);
            trace!("Clean up after running script with {:p}", &*entry.global);
        });

        // Step 5
        if !thread::panicking() && incumbent_global().is_none() {
            self.global.perform_a_microtask_checkpoint();
        }
    }
}

/// Returns the ["entry"] global object.
///
/// ["entry"]: https://html.spec.whatwg.org/multipage/#entry
pub fn entry_global() -> DomRoot<GlobalScope> {
    STACK.with(|stack| {
        stack.borrow()
             .iter()
             .rev()
             .find(|entry| entry.kind == StackEntryKind::Entry)
             .map(|entry| DomRoot::from_ref(&*entry.global))
    }).unwrap()
}

/// RAII struct that pushes and pops entries from the script settings stack.
pub struct AutoIncumbentScript {
    global: usize,
}

impl AutoIncumbentScript {
    /// <https://html.spec.whatwg.org/multipage/#prepare-to-run-a-callback>
    pub fn new(global: &GlobalScope) -> Self {
        // Step 2-3.
        unsafe {
            let cx = Runtime::get();
            assert!(!cx.is_null());
            HideScriptedCaller(cx);
        }
        STACK.with(|stack| {
            trace!("Prepare to run a callback with {:p}", global);
            // Step 1.
            let mut stack = stack.borrow_mut();
            stack.push(StackEntry {
                global: Dom::from_ref(global),
                kind: StackEntryKind::Incumbent,
            });
            AutoIncumbentScript {
                global: global as *const _ as usize,
            }
        })
    }
}

impl Drop for AutoIncumbentScript {
    /// <https://html.spec.whatwg.org/multipage/#clean-up-after-running-a-callback>
    fn drop(&mut self) {
        STACK.with(|stack| {
            // Step 4.
            let mut stack = stack.borrow_mut();
            let entry = stack.pop().unwrap();
            // Step 3.
            assert_eq!(&*entry.global as *const GlobalScope as usize,
                       self.global,
                       "Dropped AutoIncumbentScript out of order.");
            assert_eq!(entry.kind, StackEntryKind::Incumbent);
            trace!("Clean up after running a callback with {:p}", &*entry.global);
        });
        unsafe {
            // Step 1-2.
            let cx = Runtime::get();
            assert!(!cx.is_null());
            UnhideScriptedCaller(cx);
        }
    }
}

/// Returns the ["incumbent"] global object.
///
/// ["incumbent"]: https://html.spec.whatwg.org/multipage/#incumbent
pub fn incumbent_global() -> Option<DomRoot<GlobalScope>> {
    // https://html.spec.whatwg.org/multipage/#incumbent-settings-object

    // Step 1, 3: See what the JS engine has to say. If we've got a scripted
    // caller override in place, the JS engine will lie to us and pretend that
    // there's nothing on the JS stack, which will cause us to check the
    // incumbent script stack below.
    unsafe {
        let cx = Runtime::get();
        assert!(!cx.is_null());
        let global = GetScriptedCallerGlobal(cx);
        if !global.is_null() {
            return Some(GlobalScope::from_object(global));
        }
    }

    // Step 2: nothing from the JS engine. Let's use whatever's on the explicit stack.
    STACK.with(|stack| {
        stack.borrow()
             .last()
             .map(|entry| DomRoot::from_ref(&*entry.global))
    })
}
