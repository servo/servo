/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implementation of [microtasks](https://html.spec.whatwg.org/multipage/#microtask) and
//! microtask queues. It is up to implementations of event loops to store a queue and
//! perform checkpoints at appropriate times, as well as enqueue microtasks as required.

use std::cell::Cell;
use std::mem;
use std::rc::Rc;

use js::context::JSContext;
use js::rust::wrappers2::JobQueueMayNotBeEmpty;
use malloc_size_of::MallocSizeOf;
use script_bindings::cell::DomRefCell;
use script_bindings::root::Dom;

use crate::JSTraceable;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::PromiseBinding::PromiseJobCallback;
use crate::dom::bindings::codegen::Bindings::VoidFunctionBinding::VoidFunction;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::realms::enter_auto_realm;
use crate::script_runtime::notify_about_rejected_promises;
use crate::script_thread::ScriptThread;

/// A collection of microtasks in FIFO order.
#[derive(Default, JSTraceable, MallocSizeOf)]
pub(crate) struct MicrotaskQueue {
    /// The list of enqueued microtasks that will be invoked at the next microtask checkpoint.
    microtask_queue: DomRefCell<Vec<Box<dyn MicrotaskRunnable>>>,
    /// <https://html.spec.whatwg.org/multipage/#performing-a-microtask-checkpoint>
    performing_a_microtask_checkpoint: Cell<bool>,
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

/// A promise callback scheduled to run during the next microtask checkpoint (#4283).
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct EnqueuedPromiseCallback {
    #[conditional_malloc_size_of]
    pub(crate) callback: Rc<PromiseJobCallback>,
    pub(crate) global: Dom<GlobalScope>,
    pub(crate) is_user_interacting: bool,
}

impl MicrotaskRunnable for EnqueuedPromiseCallback {
    fn handler(&self, cx: &mut JSContext) {
        let _guard = ScriptThread::user_interacting_guard();
        let mut realm = enter_auto_realm(cx, &*self.global);
        let cx = &mut realm;
        let _ = self
            .callback
            .Call_(cx, &*self.global, ExceptionHandling::Report);
    }
}

/// A microtask that comes from a queueMicrotask() Javascript call,
/// identical to EnqueuedPromiseCallback once it's on the queue
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

impl MicrotaskQueue {
    /// Add a new microtask to this queue. It will be invoked as part of the next
    /// microtask checkpoint.
    #[expect(unsafe_code)]
    pub(crate) fn enqueue(&self, cx: &JSContext, task: Box<dyn MicrotaskRunnable>) {
        self.microtask_queue.borrow_mut().push(task);
        unsafe { JobQueueMayNotBeEmpty(cx) };
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

        // Step 3. While the event loop's microtask queue is not empty:
        while !self.microtask_queue.borrow().is_empty() {
            rooted_vec!(let mut pending_queue);
            mem::swap(&mut *pending_queue, &mut *self.microtask_queue.borrow_mut());

            for (idx, job) in pending_queue.iter().enumerate() {
                if idx == pending_queue.len() - 1 && self.microtask_queue.borrow().is_empty() {
                    unsafe { js::rust::wrappers2::JobQueueIsEmpty(cx) };
                }

                job.handler(cx);
            }
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
            let _ = global.get_indexeddb(cx).cleanup_indexeddb_transactions(cx);
        }

        // TODO: Step 6. Perform ClearKeptObjects().

        // Step 7. Set the event loop's performing a microtask checkpoint to false.
        self.performing_a_microtask_checkpoint.set(false);
        // TODO: Step 8. Record timing info for microtask checkpoint.
    }

    pub(crate) fn empty(&self) -> bool {
        self.microtask_queue.borrow().is_empty()
    }

    pub(crate) fn clear(&self) {
        self.microtask_queue.borrow_mut().clear();
    }
}
