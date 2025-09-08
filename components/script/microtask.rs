/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implementation of [microtasks](https://html.spec.whatwg.org/multipage/#microtask) and
//! microtask queues. It is up to implementations of event loops to store a queue and
//! perform checkpoints at appropriate times, as well as enqueue microtasks as required.

use std::cell::Cell;
use std::mem;
use std::rc::Rc;

use base::id::PipelineId;
use js::jsapi::{JSAutoRealm, JobQueueIsEmpty, JobQueueMayNotBeEmpty};

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::PromiseBinding::PromiseJobCallback;
use crate::dom::bindings::codegen::Bindings::VoidFunctionBinding::VoidFunction;
use crate::dom::bindings::root::DomRoot;
use crate::dom::defaultteereadrequest::DefaultTeeReadRequestMicrotask;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlimageelement::ImageElementMicrotask;
use crate::dom::html::htmlmediaelement::MediaElementMicrotask;
use crate::dom::promise::WaitForAllSuccessStepsMicrotask;
use crate::realms::enter_realm;
use crate::script_runtime::{CanGc, JSContext, notify_about_rejected_promises};
use crate::script_thread::ScriptThread;

/// A collection of microtasks in FIFO order.
#[derive(Default, JSTraceable, MallocSizeOf)]
pub(crate) struct MicrotaskQueue {
    /// The list of enqueued microtasks that will be invoked at the next microtask checkpoint.
    microtask_queue: DomRefCell<Vec<Microtask>>,
    /// <https://html.spec.whatwg.org/multipage/#performing-a-microtask-checkpoint>
    performing_a_microtask_checkpoint: Cell<bool>,
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum Microtask {
    Promise(EnqueuedPromiseCallback),
    User(UserMicrotask),
    MediaElement(MediaElementMicrotask),
    ImageElement(ImageElementMicrotask),
    ReadableStreamTeeReadRequest(DefaultTeeReadRequestMicrotask),
    WaitForAllSuccessSteps(WaitForAllSuccessStepsMicrotask),
    CustomElementReaction,
    NotifyMutationObservers,
}

pub(crate) trait MicrotaskRunnable {
    fn handler(&self, _can_gc: CanGc) {}
    fn enter_realm(&self) -> JSAutoRealm;
}

/// A promise callback scheduled to run during the next microtask checkpoint (#4283).
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct EnqueuedPromiseCallback {
    #[conditional_malloc_size_of]
    pub(crate) callback: Rc<PromiseJobCallback>,
    #[no_trace]
    pub(crate) pipeline: PipelineId,
    pub(crate) is_user_interacting: bool,
}

/// A microtask that comes from a queueMicrotask() Javascript call,
/// identical to EnqueuedPromiseCallback once it's on the queue
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct UserMicrotask {
    #[conditional_malloc_size_of]
    pub(crate) callback: Rc<VoidFunction>,
    #[no_trace]
    pub(crate) pipeline: PipelineId,
}

impl MicrotaskQueue {
    /// Add a new microtask to this queue. It will be invoked as part of the next
    /// microtask checkpoint.
    #[allow(unsafe_code)]
    pub(crate) fn enqueue(&self, job: Microtask, cx: JSContext) {
        self.microtask_queue.borrow_mut().push(job);
        unsafe { JobQueueMayNotBeEmpty(*cx) };
    }

    /// <https://html.spec.whatwg.org/multipage/#perform-a-microtask-checkpoint>
    /// Perform a microtask checkpoint, executing all queued microtasks until the queue is empty.
    #[allow(unsafe_code)]
    pub(crate) fn checkpoint<F>(
        &self,
        cx: JSContext,
        target_provider: F,
        globalscopes: Vec<DomRoot<GlobalScope>>,
        can_gc: CanGc,
    ) where
        F: Fn(PipelineId) -> Option<DomRoot<GlobalScope>>,
    {
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
                    unsafe { JobQueueIsEmpty(*cx) };
                }

                match *job {
                    Microtask::Promise(ref job) => {
                        if let Some(target) = target_provider(job.pipeline) {
                            let _guard = ScriptThread::user_interacting_guard();
                            let _realm = enter_realm(&*target);
                            let _ = job
                                .callback
                                .Call_(&*target, ExceptionHandling::Report, can_gc);
                        }
                    },
                    Microtask::User(ref job) => {
                        if let Some(target) = target_provider(job.pipeline) {
                            let _realm = enter_realm(&*target);
                            let _ = job
                                .callback
                                .Call_(&*target, ExceptionHandling::Report, can_gc);
                        }
                    },
                    Microtask::MediaElement(ref task) => {
                        let _realm = task.enter_realm();
                        task.handler(can_gc);
                    },
                    Microtask::ImageElement(ref task) => {
                        let _realm = task.enter_realm();
                        task.handler(can_gc);
                    },
                    Microtask::ReadableStreamTeeReadRequest(ref task) => {
                        let _realm = task.enter_realm();
                        task.handler(can_gc);
                    },
                    Microtask::WaitForAllSuccessSteps(ref task) => {
                        let _realm = task.enter_realm();
                        task.handler(can_gc);
                    },
                    Microtask::CustomElementReaction => {
                        ScriptThread::invoke_backup_element_queue(can_gc);
                    },
                    Microtask::NotifyMutationObservers => {
                        ScriptThread::mutation_observers().notify_mutation_observers(can_gc);
                    },
                }
            }
        }

        // Step 4. For each environment settings object settingsObject whose responsible
        // event loop is this event loop, notify about rejected promises given
        // settingsObject's global object.
        for global in globalscopes.into_iter() {
            notify_about_rejected_promises(&global);
        }

        // TODO: Step 5. Cleanup Indexed Database transactions.

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
