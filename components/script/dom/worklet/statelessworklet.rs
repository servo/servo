/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cmp::max;
use std::collections::hash_map;
use std::rc::Rc;
use std::thread;

use crossbeam_channel::{Sender, unbounded};
use js::context::JSContext;
use js::jsapi::{GCReason, JSGCParamKey, JSTracer};
use js::rust::wrappers2::{JS_GC, JS_GetGCParameter};
use net_traits::policy_container::PolicyContainer;
use servo_base::id::PipelineId;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::thread_state::{self, ThreadState};
use swapper::swapper;

use crate::dom::bindings::codegen::Bindings::RequestBinding::RequestCredentials;
use crate::dom::bindings::refcounted::TrustedPromise;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::trace::{JSTraceable, RootedTraceableBox};
use crate::dom::promise::Promise;
use crate::dom::worklet::{
    MIN_GC_THRESHOLD, PendingTasksStruct, WorkletControl, WorkletData, WorkletExecutor, WorkletId,
    WorkletTask, WorkletThread, WorkletThreadPool, WorkletThreadRole,
};
use crate::dom::workletglobalscope::{
    WorkletGlobalScope, WorkletGlobalScopeInit, WorkletGlobalScopeType,
};
use crate::messaging::CommonScriptMsg;
use crate::microtask::MicrotaskQueue;
use crate::script_runtime::Runtime;

/// Worklets execute in a dedicated thread pool.
///
/// The goal is to ensure that there is a primary worklet thread,
/// which is able to responsively execute worklet code. In particular,
/// worklet execution should not be delayed by GC, or by script
/// loading.
///
/// To achieve this, we implement a three-thread pool, with the
/// threads cycling between three thread roles:
///
///  * The primary worklet thread is the one available to execute
///    worklet code.
///
///  * The hot backup thread may peform GC, but otherwise is expected
///    to take over the primary role.
///
///  * The cold backup thread may peform script loading and other
///    long-running tasks.
///
/// In the implementation, we use two kinds of messages:
///
///  * Data messages are expected to be processed quickly, and include
///    the worklet tasks to be performed by the primary thread, as
///    well as requests to change role or quit execution.
///
///  * Control messages are expected to be processed more slowly, and
///    include script loading.
///
/// Data messages are targeted at a role, for example, task execution
/// is expected to be performed by whichever thread is currently
/// primary. Control messages are targeted at a thread, for example
/// adding a module is performed in every thread, even if they change roles
/// in the middle of module loading.
///
/// The thread pool lives in the script thread, and is initialized
/// when a worklet adds a module. It is dropped when the script thread
/// is dropped, and asks each of the worklet threads to quit.
///
/// Layout can end up blocking on the primary worklet thread
/// (e.g. when invoking a paint callback), so it is important to avoid
/// deadlock by making sure the primary worklet thread doesn't end up
/// blocking waiting on layout. In particular, since the constellation
/// can block waiting on layout, this means the primary worklet thread
/// can't block waiting on the constellation. In general, the primary
/// worklet thread shouldn't perform any blocking operations. If a worklet
/// thread needs to do anything blocking, it should send a control
/// message, to make sure that the blocking operation is performed
/// by a backup thread, not by the primary thread.

#[derive(Clone, JSTraceable)]
pub(crate) struct StatelessWorkletThreadPool {
    // Channels to send data messages to the three roles.
    #[no_trace]
    primary_sender: Sender<WorkletData>,
    #[no_trace]
    hot_backup_sender: Sender<WorkletData>,
    #[no_trace]
    cold_backup_sender: Sender<WorkletData>,
    // Channels to send control messages to the three threads.
    #[no_trace]
    control_sender_0: Sender<WorkletControl>,
    #[no_trace]
    control_sender_1: Sender<WorkletControl>,
    #[no_trace]
    control_sender_2: Sender<WorkletControl>,
}

impl Drop for StatelessWorkletThreadPool {
    fn drop(&mut self) {
        let _ = self.cold_backup_sender.send(WorkletData::Quit);
        let _ = self.hot_backup_sender.send(WorkletData::Quit);
        let _ = self.primary_sender.send(WorkletData::Quit);
    }
}

// TODO: rename to StatelessWorkletThreadInit
/// Data to initialize a worklet thread.
#[derive(Clone)]
struct WorkletThreadInit {
    /// Senders
    primary_sender: Sender<WorkletData>,
    hot_backup_sender: Sender<WorkletData>,
    cold_backup_sender: Sender<WorkletData>,

    /// Data for initializing new worklet global scopes
    global_init: WorkletGlobalScopeInit,
}

impl StatelessWorkletThreadPool {
    /// Create a new thread pool and spawn the threads.
    /// When the thread pool is dropped, the threads will be asked to quit.
    pub(crate) fn spawn(global_init: WorkletGlobalScopeInit) -> StatelessWorkletThreadPool {
        let primary_role = WorkletThreadRole::new(false, false);
        let hot_backup_role = WorkletThreadRole::new(true, false);
        let cold_backup_role = WorkletThreadRole::new(false, true);
        let primary_sender = primary_role.sender();
        let hot_backup_sender = hot_backup_role.sender();
        let cold_backup_sender = cold_backup_role.sender();
        let init = WorkletThreadInit {
            primary_sender: primary_sender.clone(),
            hot_backup_sender: hot_backup_sender.clone(),
            cold_backup_sender: cold_backup_sender.clone(),
            global_init,
        };
        StatelessWorkletThreadPool {
            primary_sender,
            hot_backup_sender,
            cold_backup_sender,
            control_sender_0: StatelessWorkletThread::spawn(primary_role, init.clone(), 0),
            control_sender_1: StatelessWorkletThread::spawn(hot_backup_role, init.clone(), 1),
            control_sender_2: StatelessWorkletThread::spawn(cold_backup_role, init, 2),
        }
    }
}

impl WorkletThreadPool for StatelessWorkletThreadPool {
    /// Loads a worklet module into every worklet thread.
    /// If all of the threads load successfully, the promise is resolved.
    /// If any of the threads fails to load, the promise is rejected.
    /// <https://drafts.css-houdini.org/worklets/#fetch-and-invoke-a-worklet-script>
    #[allow(clippy::too_many_arguments)]
    fn fetch_and_invoke_a_worklet_script(
        &self,
        pipeline_id: PipelineId,
        worklet_id: WorkletId,
        global_type: WorkletGlobalScopeType,
        origin: ImmutableOrigin,
        base_url: ServoUrl,
        script_url: ServoUrl,
        policy_container: PolicyContainer,
        credentials: RequestCredentials,
        pending_tasks_struct: PendingTasksStruct,
        promise: &Rc<Promise>,
        inherited_secure_context: Option<bool>,
    ) {
        // Send each thread a control message asking it to load the script.
        for sender in &[
            &self.control_sender_0,
            &self.control_sender_1,
            &self.control_sender_2,
        ] {
            let _ = sender.send(WorkletControl::FetchAndInvokeAWorkletScript {
                pipeline_id,
                worklet_id,
                global_type,
                origin: origin.clone(),
                base_url: base_url.clone(),
                script_url: script_url.clone(),
                policy_container: policy_container.clone(),
                credentials,
                pending_tasks_struct: pending_tasks_struct.clone(),
                promise: TrustedPromise::new(promise.clone()),
                inherited_secure_context,
            });
        }
        self.wake_threads();
    }

    fn exit_worklet(&self, worklet_id: WorkletId) {
        for sender in &[
            &self.control_sender_0,
            &self.control_sender_1,
            &self.control_sender_2,
        ] {
            let _ = sender.send(WorkletControl::ExitWorklet(worklet_id));
        }
        self.wake_threads();
    }

    fn wake_threads(&self) {
        // If any of the threads are blocked waiting on data, wake them up.
        let _ = self.cold_backup_sender.send(WorkletData::WakeUp);
        let _ = self.hot_backup_sender.send(WorkletData::WakeUp);
        let _ = self.primary_sender.send(WorkletData::WakeUp);
    }

    /// Send a `WorkletTask` to the "Primary Worklet Thread" to execute.
    fn run_task(&self, worklet_id: WorkletId, worklet_task: WorkletTask) {
        let msg = WorkletData::Task(worklet_id, worklet_task);
        let _ = self.primary_sender.send(msg);
    }
}

/// A thread for executing stateless worklets.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct StatelessWorkletThread {
    worklet_thread: WorkletThread,

    /// Which role the thread is currently playing
    role: WorkletThreadRole,

    /// Senders
    primary_sender: Sender<WorkletData>,
    hot_backup_sender: Sender<WorkletData>,
    cold_backup_sender: Sender<WorkletData>,
}

#[expect(unsafe_code)]
unsafe impl JSTraceable for StatelessWorkletThread {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        debug!("Tracing worklet thread.");
        unsafe { self.worklet_thread.global_scopes.trace(trc) };
    }
}

impl StatelessWorkletThread {
    #[allow(unsafe_code)]
    /// Spawn a new worklet thread, returning the channel to send it control messages.
    fn spawn(
        role: WorkletThreadRole,
        init: WorkletThreadInit,
        thread_index: u8,
    ) -> Sender<WorkletControl> {
        let (control_sender, control_receiver) = unbounded();
        let control_sender_clone = control_sender.clone();
        let _ = thread::Builder::new()
            .name(format!("Worklet#{thread_index}"))
            .spawn(move || {
                // TODO: add a new IN_WORKLET thread state?
                // TODO: set interrupt handler?
                // TODO: configure the JS runtime (e.g. discourage GC, encourage agressive JIT)
                debug!("Initializing worklet thread.");
                thread_state::initialize(ThreadState::SCRIPT | ThreadState::IN_WORKER);
                let runtime = Runtime::new(None);
                let mut cx = unsafe { runtime.cx() };
                let worklet_thread = WorkletThread::new(
                    init.global_init,
                    control_receiver,
                    control_sender_clone,
                    runtime,
                );
                let mut thread = RootedTraceableBox::new(StatelessWorkletThread {
                    worklet_thread,
                    role,
                    primary_sender: init.primary_sender,
                    hot_backup_sender: init.hot_backup_sender,
                    cold_backup_sender: init.cold_backup_sender,
                });
                thread.run(&mut cx);
            })
            .expect("Couldn't start worklet thread");
        control_sender
    }

    /// The main event loop for a worklet thread
    fn run(&mut self, cx: &mut JSContext) {
        loop {
            // The handler for data messages
            let message = self.role.receiver().recv().unwrap();
            match message {
                // The whole point of this thread pool is to perform tasks!
                WorkletData::Task(id, task) => {
                    self.perform_a_worklet_task(cx, id, task);
                },
                // To start swapping roles, get ready to perform an atomic swap,
                // and block waiting for the other end to finish it.
                // NOTE: the cold backup can block on the primary or the hot backup;
                //       the hot backup can block on the primary;
                //       the primary can block on nothing;
                //       this total ordering on thread roles is what guarantees deadlock-freedom.
                WorkletData::StartSwapRoles(sender) => {
                    let (our_swapper, their_swapper) = swapper();
                    match sender.send(WorkletData::FinishSwapRoles(their_swapper)) {
                        Ok(_) => {},
                        Err(_) => {
                            // This might happen if the script thread shuts down while
                            // waiting for the worklet to finish.
                            return;
                        },
                    };
                    let _ = our_swapper.swap(&mut self.role);
                },
                // To finish swapping roles, perform the atomic swap.
                // The other end should have already started the swap, so this shouldn't block.
                WorkletData::FinishSwapRoles(swapper) => {
                    let _ = swapper.swap(&mut self.role);
                },
                // Wake up! There may be control messages to process.
                WorkletData::WakeUp => {},
                // Quit!
                WorkletData::Quit => {
                    return;
                },
            }

            // Only process control messages if we're the cold backup,
            // otherwise if there are outstanding control messages,
            // try to become the cold backup.
            if self.role.is_cold_backup() {
                if let Some(control) = self.worklet_thread.control_buffer.take() {
                    self.process_control(control, cx);
                }
                while let Ok(control) = self.worklet_thread.control_receiver().try_recv() {
                    self.process_control(control, cx);
                }

                for worklet_global_scope in self.worklet_thread.global_scopes.values() {
                    worklet_global_scope.perform_a_microtask_checkpoint(cx);
                }

                self.gc(cx);
            } else if self.worklet_thread.control_buffer.is_none() &&
                let Ok(control) = self.worklet_thread.control_receiver().try_recv()
            {
                self.worklet_thread.control_buffer = Some(control);
                let msg = WorkletData::StartSwapRoles(self.role.sender().clone());
                let _ = self.cold_backup_sender.send(msg);
            }
            // If we are tight on memory, and we're a backup then perform a gc.
            // If we are tight on memory, and we're the primary then try to become the hot backup.
            // Hopefully this happens soon!
            if self.current_memory_usage() > self.worklet_thread.gc_threshold {
                if self.role.is_hot_backup() || self.role.is_cold_backup() {
                    self.worklet_thread.should_gc = false;
                    self.gc(cx);
                } else if !self.worklet_thread.should_gc {
                    self.worklet_thread.should_gc = true;
                    let msg = WorkletData::StartSwapRoles(self.role.sender().clone());
                    let _ = self.hot_backup_sender.send(msg);
                }
            }
        }
    }

    /// The current memory usage of the thread
    #[expect(unsafe_code)]
    fn current_memory_usage(&self) -> u32 {
        unsafe {
            JS_GetGCParameter(
                self.worklet_thread.runtime_cx_no_gc(),
                JSGCParamKey::JSGC_BYTES,
            )
        }
    }

    /// Perform a GC.
    #[expect(unsafe_code)]
    fn gc(&mut self, cx: &mut JSContext) {
        debug!(
            "BEGIN GC (usage = {}, threshold = {}).",
            self.current_memory_usage(),
            self.worklet_thread.gc_threshold
        );
        unsafe { JS_GC(cx, GCReason::API) };
        self.worklet_thread.gc_threshold = max(MIN_GC_THRESHOLD, self.current_memory_usage() * 2);
        debug!(
            "END GC (usage = {}, threshold = {}).",
            self.current_memory_usage(),
            self.worklet_thread.gc_threshold
        );
    }

    /// Get the worklet global scope for a given worklet.
    /// Creates the worklet global scope if it doesn't exist.
    #[expect(clippy::too_many_arguments)]
    fn get_worklet_global_scope(
        &mut self,
        cx: &mut JSContext,
        pipeline_id: PipelineId,
        worklet_id: WorkletId,
        inherited_secure_context: Option<bool>,
        global_type: WorkletGlobalScopeType,
        base_url: ServoUrl,
        microtask_queue: Rc<MicrotaskQueue>,
    ) -> DomRoot<WorkletGlobalScope> {
        match self.worklet_thread.global_scopes.entry(worklet_id) {
            hash_map::Entry::Occupied(entry) => DomRoot::from_ref(entry.get()),

            // Step 6.1. If workletInstance's global scopes is empty:
            hash_map::Entry::Vacant(entry) => {
                debug!("Creating new worklet global scope.");

                // Step 6.1.1. Create a worklet global scope given workletInstance.
                let executor = WorkletExecutor::new(
                    worklet_id,
                    self.primary_sender.clone(),
                    self.hot_backup_sender.clone(),
                    self.cold_backup_sender.clone(),
                    self.worklet_thread.control_sender.clone(),
                );

                let result = WorkletGlobalScope::new(
                    global_type,
                    pipeline_id,
                    base_url,
                    inherited_secure_context,
                    executor,
                    &self.worklet_thread.global_init,
                    cx,
                    self.worklet_thread.closing.clone(),
                    microtask_queue,
                );
                entry.insert(Dom::from_ref(&*result));
                result
            },
        }
    }

    /// Execute a `WorkletTask`.
    fn perform_a_worklet_task(
        &self,
        cx: &mut JSContext,
        worklet_id: WorkletId,
        worklet_task: WorkletTask,
    ) {
        match self.worklet_thread.global_scopes.get(&worklet_id) {
            Some(global) => worklet_task(cx, global),
            None => warn!("No such worklet as {:?}.", worklet_id),
        }
    }

    /// Process a control message.
    fn process_control(&mut self, control: WorkletControl, cx: &mut js::context::JSContext) {
        match control {
            WorkletControl::ExitWorklet(worklet_id) => {
                self.worklet_thread.global_scopes.remove(&worklet_id);
            },
            WorkletControl::FetchAndInvokeAWorkletScript {
                pipeline_id,
                worklet_id,
                global_type,
                origin,
                base_url,
                script_url,
                policy_container,
                credentials,
                pending_tasks_struct,
                promise,
                inherited_secure_context,
            } => {
                // A worklet global scope is created here as part of the AddModule specs.
                // <https://html.spec.whatwg.org/multipage/#dom-worklet-addmodule>
                // 6.1.3. Wait for all steps of the creation process(es) — including those taking place within the worklet agents — to complete, before moving on.
                let global = self.get_worklet_global_scope(
                    cx,
                    pipeline_id,
                    worklet_id,
                    inherited_secure_context,
                    global_type,
                    base_url,
                    self.worklet_thread.microtask_queue(),
                );
                self.worklet_thread.fetch_and_invoke_a_worklet_script(
                    &global,
                    pipeline_id,
                    origin,
                    script_url,
                    policy_container,
                    credentials,
                    pending_tasks_struct,
                    promise,
                    cx,
                )
            },
            WorkletControl::Common(script_msg) => {
                if let CommonScriptMsg::Task(_, task, _, _) = script_msg {
                    task.run_box(cx);
                }
            },
        }
    }
}
