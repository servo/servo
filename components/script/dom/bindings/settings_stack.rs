/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::marker::PhantomData;
use std::thread;

use js::jsapi::{GetScriptedCallerGlobal, HideScriptedCaller, JSTracer, UnhideScriptedCaller};
use js::rust::Runtime;

use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::bindings::utils::DomHelpers;
use crate::dom::globalscope::{GlobalScope, GlobalScopeHelpers};
use crate::script_runtime::CanGc;
use crate::DomTypes;

thread_local!(pub(super) static STACK: RefCell<Vec<StackEntry<crate::DomTypeHolder>>> = const {
    RefCell::new(Vec::new())
});

#[derive(Debug, Eq, JSTraceable, PartialEq)]
enum StackEntryKind {
    Incumbent,
    Entry,
}

#[cfg_attr(crown, allow(crown::unrooted_must_root))]
#[derive(JSTraceable)]
pub(crate) struct StackEntry<D: DomTypes> {
    global: Dom<D::GlobalScope>,
    kind: StackEntryKind,
}

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

/// RAII struct that pushes and pops entries from the script settings stack.
pub(crate) struct GenericAutoEntryScript<D: DomTypes> {
    global: DomRoot<D::GlobalScope>,
    #[cfg(feature = "tracing")]
    #[allow(dead_code)]
    span: tracing::span::EnteredSpan,
}

impl<D: DomTypes> GenericAutoEntryScript<D> {
    /// <https://html.spec.whatwg.org/multipage/#prepare-to-run-script>
    pub(crate) fn new(global: &D::GlobalScope) -> Self {
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
        if !thread::panicking() && incumbent_global().is_none() {
            self.global.perform_a_microtask_checkpoint(CanGc::note());
        }
    }
}

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

/// RAII struct that pushes and pops entries from the script settings stack.
pub(crate) struct GenericAutoIncumbentScript<D: DomTypes> {
    global: usize,
    _marker: PhantomData<D>,
}

pub(crate) type AutoIncumbentScript = GenericAutoIncumbentScript<crate::DomTypeHolder>;

impl<D: DomTypes> GenericAutoIncumbentScript<D> {
    /// <https://html.spec.whatwg.org/multipage/#prepare-to-run-a-callback>
    pub(crate) fn new(global: &D::GlobalScope) -> Self {
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
