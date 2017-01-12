/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ExceptionHandling;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::PromiseBinding::PromiseJobCallback;
use dom::bindings::js::Root;
use dom::globalscope::GlobalScope;
use msg::constellation_msg::PipelineId;
use std::cell::Cell;
use std::rc::Rc;

/// A promise callback scheduled to run during the next microtask checkpoint (#4283).
#[derive(JSTraceable, HeapSizeOf)]
pub struct EnqueuedPromiseCallback {
    #[ignore_heap_size_of = "Rc has unclear ownership"]
    pub callback: Rc<PromiseJobCallback>,
    pub pipeline: PipelineId,
}

#[derive(JSTraceable, HeapSizeOf)]
pub enum Microtask {
    Promise(EnqueuedPromiseCallback),
}

/// A collection of microtasks in FIFO order.
#[derive(JSTraceable, HeapSizeOf)]
pub struct MicrotaskQueue {
    /// A snapshot of `microtask_queue` that was taken at the start of the microtask checkpoint.
    /// Used to work around mutability errors when appending new microtasks while performing
    /// a microtask checkpoint.
    flushing_queue: DOMRefCell<Vec<Microtask>>,
    /// The list of enqueued microtasks that will be invoked at the next microtask checkpoint.
    microtask_queue: DOMRefCell<Vec<Microtask>>,
    /// https://html.spec.whatwg.org/multipage/#performing-a-microtask-checkpoint
    performing_a_microtask_checkpoint: Cell<bool>,
}

impl MicrotaskQueue {
    /// Create a new PromiseJobQueue instance.
    pub fn new() -> MicrotaskQueue {
        MicrotaskQueue {
            microtask_queue: DOMRefCell::new(vec![]),
            flushing_queue: DOMRefCell::new(vec![]),
            performing_a_microtask_checkpoint: Cell::new(false),
        }
    }

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
            {
                let mut pending_queue = self.microtask_queue.borrow_mut();
                *self.flushing_queue.borrow_mut() = pending_queue.drain(..).collect();
            }
            // N.B. borrowing this vector is safe w.r.t. mutability, since any promise job that
            // is enqueued while invoking these callbacks will be placed in `pending_queue`;
            // `flushing_queue` is a static snapshot during this checkpoint.
            for job in &*self.flushing_queue.borrow() {
                match *job {
                    Microtask::Promise(ref job) => {
                        if let Some(target) = target_provider(job.pipeline) {
                            let _ = job.callback.Call_(&*target, ExceptionHandling::Report);
                        }
                    }
                }
            }
            self.flushing_queue.borrow_mut().clear();
        }

        //TODO: Step 8 - notify about rejected promises

        // Step 9
        self.performing_a_microtask_checkpoint.set(false);
    }
}
