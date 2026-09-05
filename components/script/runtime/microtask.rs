/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implementation of [microtasks](https://html.spec.whatwg.org/multipage/#microtask) and
//! microtask queues. It is up to implementations of event loops to store a queue and
//! perform checkpoints at appropriate times, as well as enqueue microtasks as required.

use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::NonNull;
use std::rc::Rc;

use js::context::JSContext;
use js::jsapi::{
    GetExecutionGlobalFromJSMicroTask, GetPromiseUserInputEventHandlingState, IsJSMicroTask,
    JSTracer, MaybeGetPromiseFromJSMicroTask, PromiseUserInputEventHandlingState,
    ToMaybeWrappedJSMicroTask,
};
use js::jsval::{JSVal, PrivateValue};
use js::panic::wrap_panic;
use js::realm::AutoRealm;
use js::rust::wrappers2::{
    EnqueueMicroTask, HasAnyMicroTasks, JS_DequeueNextMicroTask,
    MaybeGetHostDefinedDataFromJSMicroTask, RunJSMicroTask,
};
use malloc_size_of::MallocSizeOf;
use script_bindings::root::Dom;
use script_bindings::settings_stack::{run_a_callback, run_a_script};

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::VoidFunctionBinding::VoidFunction;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::event_loop::script_thread::ScriptThread;
use crate::realms::enter_auto_realm;
use crate::runtime::script_runtime::notify_about_rejected_promises;
use crate::{DomTypeHolder, JSTraceable};

#[derive(Default, JSTraceable, MallocSizeOf)]
pub(crate) struct MicrotaskQueue {
    /// <https://html.spec.whatwg.org/multipage/#performing-a-microtask-checkpoint>
    performing_a_microtask_checkpoint: Cell<bool>,
    // microtasks are not accounted for in the size of the queue
    // as they are floating around in the memory and only referenced by the queue
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct NotifyMutationObserversMicrotask;

impl NotifyMutationObserversMicrotask {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl MicrotaskRunnable for NotifyMutationObserversMicrotask {
    fn handler(&self, cx: &mut JSContext) {
        ScriptThread::mutation_observers().notify_mutation_observers(cx);
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct CustomElementReactionMicrotask;

impl CustomElementReactionMicrotask {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl MicrotaskRunnable for CustomElementReactionMicrotask {
    fn handler(&self, cx: &mut JSContext) {
        ScriptThread::invoke_backup_element_queue(cx);
    }
}

pub(crate) trait MicrotaskRunnable: JSTraceable + MallocSizeOf {
    // must also take care of entering the realm
    fn handler(&self, _cx: &mut JSContext) {}
}

/// A microtask that comes from a queueMicrotask() Javascript call
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct UserMicrotask {
    #[conditional_malloc_size_of]
    pub(crate) callback: Rc<VoidFunction>,
    pub(crate) global: Dom<GlobalScope>,
}

impl MicrotaskRunnable for UserMicrotask {
    fn handler(&self, cx: &mut JSContext) {
        let mut realm = enter_auto_realm(cx, &*self.global);
        let cx = &mut realm;
        let _ = self
            .callback
            .Call_(cx, &*self.global, ExceptionHandling::Report);
    }
}

fn microtask_from_jsval(val: JSVal) -> *mut Box<dyn MicrotaskRunnable> {
    val.to_private() as *const Box<dyn MicrotaskRunnable> as *mut Box<dyn MicrotaskRunnable>
}

impl MicrotaskQueue {
    /// Add a new microtask to this queue. It will be invoked as part of the next
    /// microtask checkpoint.
    #[expect(unsafe_code)]
    pub(crate) fn enqueue(&self, cx: &JSContext, task: Box<dyn MicrotaskRunnable>) {
        let task = Box::new(task);
        let raw = Box::into_raw(task);
        unsafe {
            EnqueueMicroTask(cx, &PrivateValue(raw as *const c_void));
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#perform-a-microtask-checkpoint>
    /// Perform a microtask checkpoint, executing all queued microtasks until the queue is empty.
    #[expect(unsafe_code)]
    pub(crate) fn checkpoint(&self, cx: &mut JSContext, globalscopes: Vec<DomRoot<GlobalScope>>) {
        // Step 1. If the event loop's performing a microtask checkpoint is true, then return.
        if self.performing_a_microtask_checkpoint.get() {
            return;
        }

        // Step 2. Set the event loop's performing a microtask checkpoint to true.
        self.performing_a_microtask_checkpoint.set(true);

        debug!("Now performing a microtask checkpoint");

        rooted!(&in(cx) let mut generic_task: js::jsapi::GenericMicroTask);
        rooted!(&in(cx) let mut js_micro_task: *mut js::jsapi::JSMicroTask);
        rooted!(&in(cx) let mut execution_global: *mut js::jsapi::JSObject);
        rooted!(&in(cx) let mut incumbent_global: *mut js::jsapi::JSObject);
        rooted!(&in(cx) let mut data: *mut js::jsapi::JSObject);

        // Step 3. While the event loop's microtask queue is not empty:
        // based on https://spidermonkey.dev/blog/2026/01/15/job-responsibility.html#running-micro-tasks
        // and https://searchfox.org/firefox-main/rev/7ae92e67d094086cd3e09918ec94b6278a948535/xpcom/base/CycleCollectedJSContext.cpp#1176
        // and its helper functions
        while unsafe { HasAnyMicroTasks(cx) } {
            unsafe { JS_DequeueNextMicroTask(cx, generic_task.handle_mut()) };

            // https://searchfox.org/firefox-main/rev/50691777d300fffc7d1f7844b59769109bc76f3e/xpcom/base/CycleCollectedJSContext.cpp#916
            if !unsafe { IsJSMicroTask(generic_task.as_ptr()) } {
                rooted!(&in(cx) let task = unsafe {
                    Box::from_raw(
                        microtask_from_jsval(*generic_task),
                    )
                });
                task.handler(cx);
                continue;
            }

            js_micro_task.set(unsafe { ToMaybeWrappedJSMicroTask(generic_task.as_ptr()) });
            execution_global.set(unsafe { GetExecutionGlobalFromJSMicroTask(js_micro_task.get()) });
            if !unsafe {
                MaybeGetHostDefinedDataFromJSMicroTask(
                    js_micro_task.get(),
                    incumbent_global.handle_mut(),
                    data.handle_mut(),
                )
            } {
                continue;
            }

            let interaction = if let Some(promise) =
                NonNull::new(unsafe { MaybeGetPromiseFromJSMicroTask(js_micro_task.get()) })
            {
                unsafe { GetPromiseUserInputEventHandlingState(promise.as_ptr()) }
            } else {
                PromiseUserInputEventHandlingState::DontCare
            };
            let _maybe_user_interacting_guard = if interaction ==
                PromiseUserInputEventHandlingState::HadUserInteractionAtCreation
            {
                Some(ScriptThread::user_interacting_guard())
            } else {
                None
            };
            let global_scope = unsafe { GlobalScope::from_object(execution_global.get()) };
            run_a_script::<DomTypeHolder, _, _>(cx, &global_scope, |cx| {
                let mut r = || {
                    let mut realm = AutoRealm::new_from_handle(cx, execution_global.handle());
                    let _ = unsafe { RunJSMicroTask(&mut realm, js_micro_task.handle()) };
                };
                if incumbent_global.get().is_null() {
                    r();
                } else {
                    let global_scope = unsafe { GlobalScope::from_object(incumbent_global.get()) };
                    run_a_callback::<DomTypeHolder, _>(&global_scope, r);
                }
            });
        }

        // Step 4. For each environment settings object settingsObject whose responsible
        // event loop is this event loop, notify about rejected promises given
        // settingsObject's global object.
        for global in globalscopes.clone().into_iter() {
            notify_about_rejected_promises(cx, &global);
        }

        // https://html.spec.whatwg.org/multipage/#perform-a-microtask-checkpoint
        // Step 5. Cleanup Indexed Database transactions.
        // https://w3c.github.io/IndexedDB/#cleanup-indexed-database-transactions
        // “These steps are invoked by [HTML]. They ensure that transactions created by a script call
        // to transaction() are deactivated once the task that invoked the script has completed.”
        for global in globalscopes.iter() {
            if let Some(factory) = global.indexeddb_factory() {
                let _ = factory.cleanup_indexeddb_transactions(cx);
            }
        }

        // TODO: Step 6. Perform ClearKeptObjects().

        // Step 7. Set the event loop's performing a microtask checkpoint to false.
        self.performing_a_microtask_checkpoint.set(false);
        // TODO: Step 8. Record timing info for microtask checkpoint.
    }

    #[expect(unsafe_code)]
    pub(crate) fn clear(&self, cx: &JSContext) {
        rooted!(&in(cx) let mut generic_task: js::jsapi::GenericMicroTask);
        while unsafe { HasAnyMicroTasks(cx) } {
            unsafe { JS_DequeueNextMicroTask(cx, generic_task.handle_mut()) };
            if !unsafe { IsJSMicroTask(generic_task.as_ptr()) } {
                let task = unsafe { Box::from_raw(microtask_from_jsval(*generic_task)) };
                drop(task);
            }
        }
    }
}

#[expect(unsafe_code)]
pub(crate) unsafe extern "C" fn trace_non_gc_things_micro_task(
    trc: *mut JSTracer,
    val: *mut JSVal,
) {
    wrap_panic(&mut || {
        let task = microtask_from_jsval(unsafe { *val });
        unsafe { (**task).trace(trc) };
    })
}
