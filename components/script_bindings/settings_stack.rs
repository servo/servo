/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;
use std::thread;

use js::jsapi::{HideScriptedCaller, UnhideScriptedCaller};
use js::rust::Runtime;

use crate::DomTypes;
use crate::interfaces::{DomHelpers, GlobalScopeHelpers};
use crate::root::Dom;
use crate::script_runtime::temp_cx;

#[derive(Debug, Eq, JSTraceable, PartialEq)]
pub enum StackEntryKind {
    Incumbent,
    Entry,
}

#[cfg_attr(crown, expect(crown::unrooted_must_root))]
#[derive(JSTraceable)]
pub struct StackEntry<D: DomTypes> {
    pub global: Dom<D::GlobalScope>,
    pub kind: StackEntryKind,
}

/// Wrapper that pushes and pops entries from the script settings stack.
///
/// <https://html.spec.whatwg.org/multipage/#prepare-to-run-script>
/// <https://html.spec.whatwg.org/multipage/#clean-up-after-running-script>
pub fn run_a_script<D: DomTypes, R>(global: &D::GlobalScope, f: impl FnOnce() -> R) -> R {
    let settings_stack = <D as DomHelpers<D>>::settings_stack();
    let _span = settings_stack.with(|stack| {
        trace!("Prepare to run script with {:p}", global);
        let mut stack = stack.borrow_mut();
        stack.push(StackEntry {
            global: Dom::from_ref(global),
            kind: StackEntryKind::Entry,
        });
        profile_traits::info_span!("ScriptEvaluate", url = global.get_url().to_string()).entered()
    });
    let r = f();
    let stack_is_empty = settings_stack.with(|stack| {
        let mut stack = stack.borrow_mut();
        let entry = stack.pop().unwrap();
        assert_eq!(
            &*entry.global as *const D::GlobalScope, global as *const D::GlobalScope,
            "Dropped AutoEntryScript out of order."
        );
        assert_eq!(entry.kind, StackEntryKind::Entry);
        trace!("Clean up after running script with {:p}", &*entry.global);
        stack.is_empty()
    });

    // Step 5
    if !thread::panicking() && stack_is_empty {
        let mut cx = unsafe { temp_cx() };
        global.perform_a_microtask_checkpoint(&mut cx);
    }
    r
}

/// RAII struct that pushes and pops entries from the script settings stack.
pub struct GenericAutoIncumbentScript<D: DomTypes> {
    global: usize,
    _marker: PhantomData<D>,
}

impl<D: DomTypes> GenericAutoIncumbentScript<D> {
    /// <https://html.spec.whatwg.org/multipage/#prepare-to-run-a-callback>
    pub fn new(global: &D::GlobalScope) -> Self {
        // Step 2-3.
        unsafe {
            let cx =
                Runtime::get().expect("Creating a new incumbent script after runtime shutdown");
            HideScriptedCaller(cx.as_ptr());
        }
        let settings_stack = <D as DomHelpers<D>>::settings_stack();
        settings_stack.with(|stack| {
            trace!("Prepare to run a callback with {:p}", global);
            // Step 1.
            let mut stack = stack.borrow_mut();
            stack.push(StackEntry {
                global: Dom::from_ref(global),
                kind: StackEntryKind::Incumbent,
            });
            Self {
                global: global as *const _ as usize,
                _marker: PhantomData,
            }
        })
    }
}

impl<D: DomTypes> Drop for GenericAutoIncumbentScript<D> {
    /// <https://html.spec.whatwg.org/multipage/#clean-up-after-running-a-callback>
    fn drop(&mut self) {
        let settings_stack = <D as DomHelpers<D>>::settings_stack();
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
            if let Some(cx) = Runtime::get() {
                UnhideScriptedCaller(cx.as_ptr());
            }
        }
    }
}
