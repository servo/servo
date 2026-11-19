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
use servo_base::generic_channel::GenericCallback;
use servo_base::id::WorkerId;
use servo_constellation_traits::{SharedWorkerCommand, SharedWorkerHostEvent, SharedWorkerStart};

use crate::dom::sharedworkerglobalscope::{
    SharedWorkerControlMsg, SharedWorkerGlobalScope, SharedWorkerScriptMsg,
};
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::event_loop::timers::OneshotTimers;
use crate::messaging::MainThreadScriptMsg;
use crate::runtime::script_runtime::ThreadSafeJSContext;

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

    /// <https://html.spec.whatwg.org/multipage/workers.html#run-a-worker>
    fn set_suspended(&self, suspended: bool) {
        let mut execution = self
            .execution
            .lock()
            .expect("SharedWorker execution lock poisoned");
        if self.closing.load(Ordering::SeqCst) || execution.suspended == suspended {
            return;
        }
        // Suspending workers: "the user agent suspends execution of script in
        // worker global scope until such time as either the closing flag switches
        // to true or worker global scope stops being suspendable."
        execution.suspended = suspended;
        Self::wake_contexts(&execution, suspended);
        self.resumed.notify_all();
    }

    /// <https://html.spec.whatwg.org/multipage/workers.html#run-a-worker>
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
            // <https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#run-steps-after-a-timeout>
            // "wait until milliseconds milliseconds have passed with the worker
            // not suspended (not necessarily consecutively)."
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

    /// <https://html.spec.whatwg.org/multipage/workers.html#terminate-a-worker>
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

pub(crate) struct SharedWorkerThread {
    control: Arc<SharedWorkerThreadControl>,
    join_handle: Option<JoinHandle<()>>,
    events: GenericCallback<SharedWorkerHostEvent>,
}

struct ThreadFinished {
    worker_id: WorkerId,
    host_sender: Sender<MainThreadScriptMsg>,
    control: Arc<SharedWorkerThreadControl>,
}

impl Drop for ThreadFinished {
    fn drop(&mut self) {
        self.control.closing.store(true, Ordering::SeqCst);
        self.control.wake_waiters();
        if self
            .host_sender
            .send(MainThreadScriptMsg::SharedWorkerExited(self.worker_id))
            .is_err()
        {
            warn!("SharedWorker script host exited before its worker");
        }
    }
}

impl SharedWorkerThread {
    pub(crate) fn spawn(
        start: SharedWorkerStart,
        host_sender: Sender<MainThreadScriptMsg>,
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
        let command_control = control.clone();
        let command_sender = sender.clone();
        let commands = GenericCallback::new(move |message| match message {
            Ok(SharedWorkerCommand::Connect {
                connection_id,
                port,
            }) => {
                if command_sender
                    .send(SharedWorkerScriptMsg::Connect {
                        connection_id,
                        port,
                    })
                    .is_err()
                {
                    warn!("SharedWorker exited before accepting connection {connection_id:?}");
                }
            },
            Ok(SharedWorkerCommand::SetNetworkOnlineState(online)) => {
                if command_control
                    .sender
                    .send(SharedWorkerControlMsg::SetNetworkOnlineState(online))
                    .is_err()
                {
                    warn!("SharedWorker exited before receiving network state");
                }
            },
            Ok(SharedWorkerCommand::SetSuspended(suspended)) => {
                command_control.set_suspended(suspended);
            },
            Ok(SharedWorkerCommand::Terminate) => command_control.terminate(),
            Err(error) => {
                error!("Invalid SharedWorker host command: {error}");
                command_control.terminate();
            },
        })
        .map_err(|error| io::Error::other(error.to_string()))?;

        #[cfg(feature = "devtools")]
        let devtools_receiver = {
            let (devtools_sender, devtools_receiver) = servo_base::generic_channel::channel()
                .ok_or_else(|| {
                    io::Error::other("Could not create SharedWorker devtools channel")
                })?;
            start.creation.init.from_devtools_sender = Some(devtools_sender.clone());
            if let Some(channel) = &start.creation.init.to_devtools_sender {
                if let Err(error) = channel.send(ScriptToDevtoolsControlMsg::NewGlobal(
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
                )) {
                    warn!("Failed to register SharedWorker with devtools: {error}");
                }
            }
            devtools_receiver
        };

        let thread_control = control.clone();
        let join_handle = thread::Builder::new()
            .name(format!("SWW:{}", start.creation.url.debug_compact()))
            .spawn(move || {
                let _finished = ThreadFinished {
                    worker_id,
                    host_sender,
                    control: thread_control.clone(),
                };
                SharedWorkerGlobalScope::run_shared_worker_scope(
                    start,
                    sender,
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
        if let Err(error) = events.send(SharedWorkerHostEvent::Started(commands)) {
            warn!("SharedWorker manager disappeared during startup: {error}");
            control.terminate();
        }
        Ok(Self {
            control,
            join_handle: Some(join_handle),
            events,
        })
    }
}

impl Drop for SharedWorkerThread {
    fn drop(&mut self) {
        self.control.terminate();
        if let Some(handle) = self.join_handle.take() {
            if handle.join().is_err() {
                error!("SharedWorker thread panicked");
            }
        }
        if let Err(error) = self.events.send(SharedWorkerHostEvent::Exited) {
            warn!("Failed to report SharedWorker exit: {error}");
        }
    }
}
