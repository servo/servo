/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;
use std::thread;

use js::jsapi::{HideScriptedCaller, UnhideScriptedCaller};
use js::rust::Runtime;

use crate::DomTypes;
use crate::interfaces::{DomHelpers, GlobalScopeHelpers};
use crate::root::{Dom, DomRoot};
use crate::script_runtime::CanGc;

#[derive(Debug, Eq, JSTraceable, PartialEq)]
pub enum StackEntryKind {
    Incumbent,
    Entry,
}

#[cfg_attr(crown, allow(crown::unrooted_must_root))]
#[derive(JSTraceable)]
pub struct StackEntry<D: DomTypes> {
    pub global: Dom<D::GlobalScope>,
    pub kind: StackEntryKind,
}

/// RAII struct that pushes and pops entries from the script settings stack.
pub struct GenericAutoEntryScript<D: DomTypes> {
    global: DomRoot<D::GlobalScope>,
    #[cfg(feature = "tracing")]
    #[allow(dead_code)]
    span: tracing::span::EnteredSpan,
}

impl<D: DomTypes> GenericAutoEntryScript<D> {
    /// <https://html.spec.whatwg.org/multipage/#prepare-to-run-script>
    pub fn new(global: &D::GlobalScope) -> Self {
        let settings_stack = <D as DomHelpers<D>>::settings_stack();
        settings_stack.with(|stack| {
            trace!("Prepare to run script with {:p}", global);
            let mut stack = stack.borrow_mut();
            stack.push(StackEntry {
                global: Dom::from_ref(global),
                kind: StackEntryKind::Entry,
            });
            Self {
                global: DomRoot::from_ref(global),
                #[cfg(feature = "tracing")]
                span: tracing::info_span!(
                    "ScriptEvaluate",
                    servo_profiling = true,
                    url = global.get_url().to_string(),
                )
                .entered(),
            }
        })
    }
}

impl<D: DomTypes> Drop for GenericAutoEntryScript<D> {
    /// <https://html.spec.whatwg.org/multipage/#clean-up-after-running-script>
    fn drop(&mut self) {
        let settings_stack = <D as DomHelpers<D>>::settings_stack();
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
        if !thread::panicking() && D::GlobalScope::incumbent().is_none() {
            self.global.perform_a_microtask_checkpoint(CanGc::note());
        }
    }
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
