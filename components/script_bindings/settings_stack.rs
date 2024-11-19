/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::thread;

use js::jsapi::{GetScriptedCallerGlobal, HideScriptedCaller, JSTracer, UnhideScriptedCaller};
use js::rust::Runtime;

use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::trace::JSTraceable;
use crate::script_runtime::CanGc;
use crate::{DomHelpers, DomTypes};

#[derive(Debug, Eq, JSTraceable, PartialEq)]
pub enum StackEntryKind {
    Incumbent,
    Entry,
}

#[allow(crown::unrooted_must_root)]
#[derive(JSTraceable)]
pub struct StackEntry<D: DomTypes> {
    pub global: Dom<D::GlobalScope>,
    pub kind: StackEntryKind,
}

/// RAII struct that pushes and pops entries from the script settings stack.
pub struct AutoEntryScript<D: DomTypes> {
    global: DomRoot<D::GlobalScope>,
}

impl<D: DomTypes> AutoEntryScript<D> {
    /// <https://html.spec.whatwg.org/multipage/#prepare-to-run-script>
    pub fn new(global: &D::GlobalScope) -> Self {
        let settings_stack = <D as crate::DomHelpers<D>>::settings_stack();
        settings_stack.with(|stack| {
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

impl<D: DomTypes> Drop for AutoEntryScript<D> {
    /// <https://html.spec.whatwg.org/multipage/#clean-up-after-running-script>
    fn drop(&mut self) {
        let settings_stack = <D as crate::DomHelpers<D>>::settings_stack();
        settings_stack.with(|stack| {
            let mut stack = stack.borrow_mut();
            let entry = stack.pop().unwrap();
            assert_eq!(
                &*entry.global as *const D::GlobalScope, &*self.global as *const D::GlobalScope,
                "Dropped AutoEntryScript out of order."
            );
            assert_eq!(entry.kind, StackEntryKind::Entry);
            trace!("Clean up after running script with {:p}", &*entry.global);
        });

        // Step 5
        if !thread::panicking() && incumbent_global::<D>().is_none() {
            <D as crate::DomHelpers<D>>::perform_a_microtask_checkpoint(
                &self.global,
                CanGc::note(),
            );
        }
    }
}

/// RAII struct that pushes and pops entries from the script settings stack.
pub struct AutoIncumbentScript<D: DomTypes> {
    global: usize,
    _marker: std::marker::PhantomData<D>,
}

impl<D: DomTypes> AutoIncumbentScript<D> {
    /// <https://html.spec.whatwg.org/multipage/#prepare-to-run-a-callback>
    pub fn new(global: &D::GlobalScope) -> Self {
        // Step 2-3.
        unsafe {
            let cx = Runtime::get();
            assert!(!cx.is_null());
            HideScriptedCaller(cx);
        }
        let settings_stack = <D as crate::DomHelpers<D>>::settings_stack();
        settings_stack.with(|stack| {
            trace!("Prepare to run a callback with {:p}", global);
            // Step 1.
            let mut stack = stack.borrow_mut();
            stack.push(StackEntry {
                global: Dom::from_ref(global),
                kind: StackEntryKind::Incumbent,
            });
            AutoIncumbentScript {
                global: global as *const _ as usize,
                _marker: std::marker::PhantomData,
            }
        })
    }
}

impl<D: DomTypes> Drop for AutoIncumbentScript<D> {
    /// <https://html.spec.whatwg.org/multipage/#clean-up-after-running-a-callback>
    fn drop(&mut self) {
        let settings_stack = <D as crate::DomHelpers<D>>::settings_stack();
        settings_stack.with(|stack| {
            // Step 4.
            let mut stack = stack.borrow_mut();
            let entry = stack.pop().unwrap();
            // Step 3.
            assert_eq!(
                &*entry.global as *const D::GlobalScope as usize, self.global,
                "Dropped AutoIncumbentScript out of order."
            );
            assert_eq!(entry.kind, StackEntryKind::Incumbent);
            trace!(
                "Clean up after running a callback with {:p}",
                &*entry.global
            );
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
pub fn incumbent_global<D: DomTypes>() -> Option<DomRoot<D::GlobalScope>> {
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
            return Some(<D as DomHelpers<D>>::global_scope_from_object(global));
        }
    }

    // Step 2: nothing from the JS engine. Let's use whatever's on the explicit stack.
    let settings_stack = <D as crate::DomHelpers<D>>::settings_stack();
    settings_stack.with(|stack| {
        stack
            .borrow()
            .last()
            .map(|entry| DomRoot::from_ref(&*entry.global))
    })
}
