/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use crossbeam_channel::Sender;

use crate::dom::dedicatedworkerglobalscope::DedicatedWorkerControlMsg;
use crate::runtime::script_runtime::ThreadSafeJSContext;

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct AutoCloseWorker {
    /// <https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-closing>
    #[conditional_malloc_size_of]
    pub(super) closing: Arc<AtomicBool>,
    #[conditional_malloc_size_of]
    pub(super) animation_frame_provider_supported: Arc<AtomicBool>,
    /// A handle to join on the worker thread.
    #[ignore_malloc_size_of = "JoinHandle"]
    pub(super) join_handle: Option<JoinHandle<()>>,
    /// A sender of control messages.
    #[no_trace]
    pub(super) control_sender: Sender<DedicatedWorkerControlMsg>,
    /// The context to request an interrupt on the worker thread.
    #[ignore_malloc_size_of = "mozjs"]
    #[no_trace]
    pub(super) context: ThreadSafeJSContext,
}

impl Drop for AutoCloseWorker {
    /// <https://html.spec.whatwg.org/multipage/#terminate-a-worker>
    fn drop(&mut self) {
        // Step 1. Set the worker's `WorkerGlobalScope` object's closing flag to true.
        self.closing.store(true, Ordering::SeqCst);

        if self
            .control_sender
            .send(DedicatedWorkerControlMsg::Exit)
            .is_err()
        {
            warn!("Couldn't send an exit message to a dedicated worker.");
        }

        self.context.request_interrupt_callback();

        // Step 2. If there are any tasks queued in the `WorkerGlobalScope` object's relevant agent's event loop's task queues, discard them without processing them.
        // Step 3. Abort the script currently running in the worker.
        // Step 4. If the worker's WorkerGlobalScope object is actually a DedicatedWorkerGlobalScope object (i.e. the worker is a dedicated worker), then empty the port message queue of the port that the worker's implicit port is entangled with.
        // TODO Steps 2-4.
        if self
            .join_handle
            .take()
            .expect("No handle to join on worker.")
            .join()
            .is_err()
        {
            warn!("Failed to join on dedicated worker thread.");
        }
    }
}
