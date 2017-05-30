/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implementation of [microtasks](https://html.spec.whatwg.org/multipage/#microtask) and
//! microtask queues. It is up to implementations of event loops to store a queue and
//! perform checkpoints at appropriate times, as well as enqueue microtasks as required.

use dom::bindings::callback::ExceptionHandling;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::PromiseBinding::PromiseJobCallback;
use dom::bindings::js::Root;
use dom::globalscope::GlobalScope;
use dom::htmlimageelement::ImageElementMicrotask;
use dom::htmlmediaelement::MediaElementMicrotask;
use dom::mutationobserver::MutationObserver;
use msg::constellation_msg::PipelineId;
use std::cell::Cell;
use std::mem;
use std::rc::Rc;

/// A collection of microtasks in FIFO order.
#[derive(JSTraceable, HeapSizeOf, Default)]
pub struct MicrotaskQueue {
    /// The list of enqueued microtasks that will be invoked at the next microtask checkpoint.
    microtask_queue: DOMRefCell<Vec<Microtask>>,
    /// https://html.spec.whatwg.org/multipage/#performing-a-microtask-checkpoint
    performing_a_microtask_checkpoint: Cell<bool>,
}

#[derive(JSTraceable, HeapSizeOf)]
pub enum Microtask {
    Promise(EnqueuedPromiseCallback),
    MediaElement(MediaElementMicrotask),
    ImageElement(ImageElementMicrotask),
    NotifyMutationObservers,
}

pub trait MicrotaskRunnable {
    fn handler(&self) {}
}

/// A promise callback scheduled to run during the next microtask checkpoint (#4283).
#[derive(JSTraceable, HeapSizeOf)]
pub struct EnqueuedPromiseCallback {
    #[ignore_heap_size_of = "Rc has unclear ownership"]
    pub callback: Rc<PromiseJobCallback>,
    pub pipeline: PipelineId,
}

impl MicrotaskQueue {
    /// Add a new microtask to this queue. It will be invoked as part of the next
    /// microtask checkpoint.
    pub fn enqueue(&self, job: Microtask) {
        self.microtask_queue.borrow_mut().push(job);
    }

    /// https://html.spec.whatwg.org/multipage/#perform-a-microtask-checkpoint
    /// Perform a microtask checkpoint, executing all queued microtasks until the queue is empty.
    pub fn checkpoint<F>(&self, target_provider: F)
        where F: Fn(PipelineId) -> Option<Root<GlobalScope>>
    {
        if self.performing_a_microtask_checkpoint.get() {
            return;
        }

        // Step 1
        self.performing_a_microtask_checkpoint.set(true);

        // Steps 2-7
        while !self.microtask_queue.borrow().is_empty() {
            rooted_vec!(let mut pending_queue);
            mem::swap(
                &mut *pending_queue,
                &mut *self.microtask_queue.borrow_mut());

            for job in pending_queue.iter() {
                match *job {
                    Microtask::Promise(ref job) => {
                        if let Some(target) = target_provider(job.pipeline) {
                            let _ = job.callback.Call_(&*target, ExceptionHandling::Report);
                        }
                    },
                    Microtask::MediaElement(ref task) => {
                        task.handler();
                    },
                    Microtask::ImageElement(ref task) => {
                        task.handler();
                    },
                    Microtask::NotifyMutationObservers => {
                        MutationObserver::notify_mutation_observers();
                    }
                }
            }
        }

        //TODO: Step 8 - notify about rejected promises

        // Step 9
        self.performing_a_microtask_checkpoint.set(false);
    }
}
