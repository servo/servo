/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use crossbeam_channel::{Receiver, Sender, TrySendError, bounded, unbounded};
#[cfg(feature = "devtools")]
use devtools_traits::{DevtoolsPageInfo, ScriptToDevtoolsControlMsg};
use fonts::FontContext;
use net_traits::image_cache::ImageCache;
use servo_base::id::{ScriptEventLoopId, WorkerId};
use servo_constellation_traits::SharedWorkerCreationData;

use crate::dom::sharedworkerglobalscope::{
    SharedWorkerControlMsg, SharedWorkerGlobalScope, SharedWorkerScriptMsg,
};
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::event_loop::timers::OneshotTimers;
use crate::runtime::script_runtime::ThreadSafeJSContext;
use crate::sharedworker_manager::SharedWorkerEvent;

pub(crate) struct SharedWorkerThreadControl {
    closing: Arc<AtomicBool>,
    execution: Mutex<ExecutionState>,
    resumed: Condvar,
    sender: Sender<SharedWorkerControlMsg>,
}

struct ExecutionState {
    suspended: bool,
    contexts: HashMap<WorkerId, (ThreadSafeJSContext, Sender<()>)>,
}

pub(crate) struct SharedWorkerContext {
    pub(crate) control: Arc<SharedWorkerThreadControl>,
    pub(crate) wakeups: Receiver<()>,
    worker_id: WorkerId,
}

impl Drop for SharedWorkerContext {
    fn drop(&mut self) {
        self.control
            .execution
            .lock()
            .expect("SharedWorker execution lock poisoned")
            .contexts
            .remove(&self.worker_id);
    }
}

impl SharedWorkerThreadControl {
    pub(crate) fn closing(&self) -> Arc<AtomicBool> {
        self.closing.clone()
    }

    pub(crate) fn register_context(
        self: &Arc<Self>,
        worker_id: WorkerId,
        context: ThreadSafeJSContext,
    ) -> SharedWorkerContext {
        let (sender, wakeups) = bounded(1);
        let mut execution = self
            .execution
            .lock()
            .expect("SharedWorker execution lock poisoned");
        assert!(
            !execution.contexts.contains_key(&worker_id),
            "Worker context registered twice"
        );
        execution.contexts.insert(worker_id, (context, sender));
        SharedWorkerContext {
            control: self.clone(),
            wakeups,
            worker_id,
        }
    }

    fn wake_contexts(execution: &ExecutionState, interrupt: bool) {
        for (context, sender) in execution.contexts.values() {
            if interrupt {
                context.request_interrupt_callback();
            }
            match sender.try_send(()) {
                Ok(()) | Err(TrySendError::Full(())) => {},
                Err(TrySendError::Disconnected(())) => {
                    warn!("Worker lifetime receiver closed before its context was removed");
                },
            }
        }
    }

    pub(crate) fn wake_waiters(&self) {
        let execution = self
            .execution
            .lock()
            .expect("SharedWorker execution lock poisoned");
        Self::wake_contexts(&execution, false);
        self.resumed.notify_all();
    }

    /// <https://html.spec.whatwg.org/multipage/#run-a-worker>
    fn set_suspended(&self, suspended: bool) {
        let mut execution = self
            .execution
            .lock()
            .expect("SharedWorker execution lock poisoned");
        if self.closing.load(Ordering::SeqCst) || execution.suspended == suspended {
            return;
        }
        // Step 8 of onComplete. Suspending workers: Start monitoring worker global scope,
        // such that whenever worker global scope's closing flag is false and it is
        // suspendable, the user agent suspends execution of script in worker global scope
        // until such time as either the closing flag switches to true or worker global
        // scope stops being suspendable.
        execution.suspended = suspended;
        Self::wake_contexts(&execution, suspended);
        self.resumed.notify_all();
    }

    /// <https://html.spec.whatwg.org/multipage/#run-a-worker>
    pub(crate) fn wait_until_resumed(
        &self,
        closing: &AtomicBool,
        timers: Option<&OneshotTimers>,
    ) -> bool {
        let mut execution = self
            .execution
            .lock()
            .expect("SharedWorker execution lock poisoned");
        let is_closing = || closing.load(Ordering::SeqCst) || self.closing.load(Ordering::SeqCst);
        if execution.suspended && !is_closing() {
            // <https://html.spec.whatwg.org/multipage/#run-steps-after-a-timeout>
            // Step 4.1. Otherwise, global is a WorkerGlobalScope object; wait until
            // milliseconds milliseconds have passed with the worker not suspended
            // (not necessarily consecutively).
            if let Some(timers) = timers {
                timers.suspend();
            }
            while execution.suspended && !is_closing() {
                execution = self
                    .resumed
                    .wait(execution)
                    .expect("SharedWorker execution lock poisoned");
            }
            if !is_closing() &&
                let Some(timers) = timers
            {
                timers.resume();
            }
        }
        !is_closing()
    }

    /// <https://html.spec.whatwg.org/multipage/#terminate-a-worker>
    fn terminate(&self) {
        // Step 1. Set the worker's WorkerGlobalScope object's closing flag to true.
        let already_closing = self.closing.swap(true, Ordering::SeqCst);
        {
            let mut execution = self
                .execution
                .lock()
                .expect("SharedWorker execution lock poisoned");
            execution.suspended = false;
            // Step 3. Abort the script currently running in the worker.
            Self::wake_contexts(&execution, true);
            self.resumed.notify_all();
        }
        if self.sender.send(SharedWorkerControlMsg::Exit).is_err() && !already_closing {
            warn!("SharedWorker control channel closed during termination");
        }
    }
}

pub(crate) struct SharedWorkerStart {
    pub(crate) creation: SharedWorkerCreationData,
    pub(crate) event_loop_id: ScriptEventLoopId,
    pub(crate) events: Sender<(WorkerId, SharedWorkerEvent)>,
}

pub(crate) struct SharedWorkerThread {
    control: Arc<SharedWorkerThreadControl>,
    sender: Sender<SharedWorkerScriptMsg>,
    join_handle: Option<JoinHandle<()>>,
}

struct ThreadFinished {
    worker_id: WorkerId,
    events: Sender<(WorkerId, SharedWorkerEvent)>,
    control: Arc<SharedWorkerThreadControl>,
}

impl Drop for ThreadFinished {
    fn drop(&mut self) {
        self.control.closing.store(true, Ordering::SeqCst);
        self.control.wake_waiters();
        let _ = self.events.send((
            self.worker_id,
            SharedWorkerEvent::Exited {
                panicked: thread::panicking(),
            },
        ));
    }
}

impl SharedWorkerThread {
    pub(crate) fn spawn(
        start: SharedWorkerStart,
        image_cache: Arc<dyn ImageCache>,
        font_context: Arc<FontContext>,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
    ) -> io::Result<Self> {
        #[cfg(feature = "devtools")]
        let mut start = start;
        let worker_id = start.creation.init.worker_id;
        let events = start.events.clone();
        let (sender, receiver) = unbounded();
        let (control_sender, control_receiver) = unbounded();
        let control = Arc::new(SharedWorkerThreadControl {
            closing: Arc::new(AtomicBool::new(false)),
            execution: Mutex::new(ExecutionState {
                suspended: true,
                contexts: HashMap::new(),
            }),
            resumed: Condvar::new(),
            sender: control_sender,
        });
        #[cfg(feature = "devtools")]
        let devtools_receiver = {
            let (devtools_sender, devtools_receiver) = servo_base::generic_channel::channel()
                .ok_or_else(|| {
                    io::Error::other("Could not create SharedWorker devtools channel")
                })?;
            start.creation.init.from_devtools_sender = Some(devtools_sender.clone());
            if let Some(channel) = &start.creation.init.to_devtools_sender &&
                let Err(error) = channel.send(ScriptToDevtoolsControlMsg::NewGlobal(
                    (
                        start.creation.browsing_context_id,
                        start.creation.init.pipeline_id,
                        Some(worker_id),
                        start.creation.webview_id,
                    ),
                    devtools_sender,
                    DevtoolsPageInfo {
                        title: format!("SharedWorker for {}", start.creation.url.url()),
                        url: start.creation.url.url(),
                        is_top_level_global: false,
                        is_service_worker: false,
                    },
                ))
            {
                warn!("Failed to register SharedWorker with devtools: {error}");
            }
            devtools_receiver
        };

        let thread_control = control.clone();
        let thread_sender = sender.clone();
        let join_handle = thread::Builder::new()
            .name(format!("SWW:{}", start.creation.url.debug_compact()))
            .spawn(move || {
                let _finished = ThreadFinished {
                    worker_id,
                    events,
                    control: thread_control.clone(),
                };
                SharedWorkerGlobalScope::run_shared_worker_scope(
                    start,
                    thread_sender,
                    receiver,
                    thread_control,
                    control_receiver,
                    image_cache,
                    font_context,
                    #[cfg(feature = "webgpu")]
                    gpu_id_hub,
                    #[cfg(feature = "devtools")]
                    devtools_receiver,
                );
            })?;
        Ok(Self {
            control,
            join_handle: Some(join_handle),
            sender,
        })
    }

    pub(crate) fn terminate(&self) {
        self.control.terminate();
    }

    pub(crate) fn is_closing(&self) -> bool {
        self.control.closing.load(Ordering::SeqCst)
    }

    pub(crate) fn connect(
        &self,
        connection_id: servo_base::id::MessagePortId,
        port: servo_constellation_traits::MessagePortImpl,
    ) -> bool {
        self.sender
            .send(SharedWorkerScriptMsg::Connect {
                connection_id,
                port,
            })
            .is_ok()
    }

    pub(crate) fn set_suspended(&self, suspended: bool) {
        self.control.set_suspended(suspended);
    }

    pub(crate) fn set_online(&self, online: bool) {
        let _ = self
            .control
            .sender
            .send(SharedWorkerControlMsg::SetNetworkOnlineState(online));
    }
}

impl Drop for SharedWorkerThread {
    fn drop(&mut self) {
        self.control.terminate();
        if let Some(handle) = self.join_handle.take() &&
            handle.join().is_err()
        {
            error!("SharedWorker thread panicked");
        }
    }
}
