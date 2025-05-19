/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An implementation of Houdini worklets.
//!
//! The goal of this implementation is to maximize responsiveness of worklets,
//! and in particular to ensure that the thread performing worklet tasks
//! is never busy GCing or loading worklet code. We do this by providing a custom
//! thread pool implementation, which only performs GC or code loading on
//! a backup thread, not on the primary worklet thread.

use std::cell::OnceCell;
use std::cmp::max;
use std::collections::{HashMap, hash_map};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::thread;

use base::id::PipelineId;
use crossbeam_channel::{Receiver, Sender, unbounded};
use dom_struct::dom_struct;
use js::jsapi::{GCReason, JS_GC, JS_GetGCParameter, JSGCParamKey, JSTracer};
use malloc_size_of::malloc_size_of_is_0;
use net_traits::IpcSend;
use net_traits::request::{Destination, RequestBuilder, RequestMode};
use servo_url::{ImmutableOrigin, ServoUrl};
use style::thread_state::{self, ThreadState};
use swapper::{Swapper, swapper};
use uuid::Uuid;

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::RequestBinding::RequestCredentials;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::WorkletBinding::{WorkletMethods, WorkletOptions};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::TrustedPromise;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, RootCollection, ThreadLocalStackRoots};
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::trace::{CustomTraceable, JSTraceable, RootedTraceableBox};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
#[cfg(feature = "testbinding")]
use crate::dom::testworkletglobalscope::TestWorkletTask;
use crate::dom::window::Window;
use crate::dom::workletglobalscope::{
    WorkletGlobalScope, WorkletGlobalScopeInit, WorkletGlobalScopeType, WorkletTask,
};
use crate::fetch::load_whole_resource;
use crate::messaging::{CommonScriptMsg, MainThreadScriptMsg};
use crate::realms::InRealm;
use crate::script_runtime::{CanGc, Runtime, ScriptThreadEventCategory};
use crate::script_thread::ScriptThread;
use crate::task::TaskBox;
use crate::task_source::TaskSourceName;

// Magic numbers
const WORKLET_THREAD_POOL_SIZE: u32 = 3;
const MIN_GC_THRESHOLD: u32 = 1_000_000;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableField {
    worklet_id: WorkletId,
    /// The cached version of the script thread's WorkletThreadPool. We keep this cached
    /// because we may need to access it after the script thread has terminated.
    #[ignore_malloc_size_of = "Difficult to measure memory usage of Rc<...> types"]
    thread_pool: OnceCell<Rc<WorkletThreadPool>>,
}

impl Drop for DroppableField {
    fn drop(&mut self) {
        let worklet_id = self.worklet_id;
        if let Some(thread_pool) = self.thread_pool.get_mut() {
            thread_pool.exit_worklet(worklet_id);
        }
    }
}

#[dom_struct]
/// <https://drafts.css-houdini.org/worklets/#worklet>
pub(crate) struct Worklet {
    reflector: Reflector,
    window: Dom<Window>,
    global_type: WorkletGlobalScopeType,
    droppable_field: DroppableField,
}

impl Worklet {
    fn new_inherited(window: &Window, global_type: WorkletGlobalScopeType) -> Worklet {
        Worklet {
            reflector: Reflector::new(),
            window: Dom::from_ref(window),
            global_type,
            droppable_field: DroppableField {
                worklet_id: WorkletId::new(),
                thread_pool: OnceCell::new(),
            },
        }
    }

    pub(crate) fn new(
        window: &Window,
        global_type: WorkletGlobalScopeType,
        can_gc: CanGc,
    ) -> DomRoot<Worklet> {
        debug!("Creating worklet {:?}.", global_type);
        reflect_dom_object(
            Box::new(Worklet::new_inherited(window, global_type)),
            window,
            can_gc,
        )
    }

    pub(crate) fn worklet_id(&self) -> WorkletId {
        self.droppable_field.worklet_id
    }

    #[allow(dead_code)]
    pub(crate) fn worklet_global_scope_type(&self) -> WorkletGlobalScopeType {
        self.global_type
    }
}

impl WorkletMethods<crate::DomTypeHolder> for Worklet {
    /// <https://drafts.css-houdini.org/worklets/#dom-worklet-addmodule>
    fn AddModule(
        &self,
        module_url: USVString,
        options: &WorkletOptions,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 1.
        let promise = Promise::new_in_current_realm(comp, can_gc);

        // Step 3.
        let module_url_record = match self.window.Document().base_url().join(&module_url.0) {
            Ok(url) => url,
            Err(err) => {
                // Step 4.
                debug!("URL {:?} parse error {:?}.", module_url.0, err);
                promise.reject_error(Error::Syntax, can_gc);
                return promise;
            },
        };
        debug!("Adding Worklet module {}.", module_url_record);

        // Steps 6-12 in parallel.
        let pending_tasks_struct = PendingTasksStruct::new();

        self.droppable_field
            .thread_pool
            .get_or_init(ScriptThread::worklet_thread_pool)
            .fetch_and_invoke_a_worklet_script(
                self.window.pipeline_id(),
                self.droppable_field.worklet_id,
                self.global_type,
                self.window.origin().immutable().clone(),
                self.window.as_global_scope().api_base_url(),
                module_url_record,
                options.credentials,
                pending_tasks_struct,
                &promise,
            );

        // Step 5.
        debug!("Returning promise.");
        promise
    }
}

/// A guid for worklets.
#[derive(Clone, Copy, Debug, Eq, Hash, JSTraceable, PartialEq)]
pub(crate) struct WorkletId(#[no_trace] Uuid);

malloc_size_of_is_0!(WorkletId);

impl WorkletId {
    fn new() -> WorkletId {
        WorkletId(servo_rand::random_uuid())
    }
}

/// <https://drafts.css-houdini.org/worklets/#pending-tasks-struct>
#[derive(Clone, Debug)]
struct PendingTasksStruct(Arc<AtomicIsize>);

impl PendingTasksStruct {
    fn new() -> PendingTasksStruct {
        PendingTasksStruct(Arc::new(AtomicIsize::new(
            WORKLET_THREAD_POOL_SIZE as isize,
        )))
    }

    fn set_counter_to(&self, value: isize) -> isize {
        self.0.swap(value, Ordering::AcqRel)
    }

    fn decrement_counter_by(&self, offset: isize) -> isize {
        self.0.fetch_sub(offset, Ordering::AcqRel)
    }
}

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
pub(crate) struct WorkletThreadPool {
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

impl Drop for WorkletThreadPool {
    fn drop(&mut self) {
        let _ = self.cold_backup_sender.send(WorkletData::Quit);
        let _ = self.hot_backup_sender.send(WorkletData::Quit);
        let _ = self.primary_sender.send(WorkletData::Quit);
    }
}

impl WorkletThreadPool {
    /// Create a new thread pool and spawn the threads.
    /// When the thread pool is dropped, the threads will be asked to quit.
    pub(crate) fn spawn(global_init: WorkletGlobalScopeInit) -> WorkletThreadPool {
        let primary_role = WorkletThreadRole::new(false, false);
        let hot_backup_role = WorkletThreadRole::new(true, false);
        let cold_backup_role = WorkletThreadRole::new(false, true);
        let primary_sender = primary_role.sender.clone();
        let hot_backup_sender = hot_backup_role.sender.clone();
        let cold_backup_sender = cold_backup_role.sender.clone();
        let init = WorkletThreadInit {
            primary_sender: primary_sender.clone(),
            hot_backup_sender: hot_backup_sender.clone(),
            cold_backup_sender: cold_backup_sender.clone(),
            global_init,
        };
        WorkletThreadPool {
            primary_sender,
            hot_backup_sender,
            cold_backup_sender,
            control_sender_0: WorkletThread::spawn(primary_role, init.clone(), 0),
            control_sender_1: WorkletThread::spawn(hot_backup_role, init.clone(), 1),
            control_sender_2: WorkletThread::spawn(cold_backup_role, init, 2),
        }
    }

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
        credentials: RequestCredentials,
        pending_tasks_struct: PendingTasksStruct,
        promise: &Rc<Promise>,
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
                credentials,
                pending_tasks_struct: pending_tasks_struct.clone(),
                promise: TrustedPromise::new(promise.clone()),
            });
        }
        self.wake_threads();
    }

    pub(crate) fn exit_worklet(&self, worklet_id: WorkletId) {
        for sender in &[
            &self.control_sender_0,
            &self.control_sender_1,
            &self.control_sender_2,
        ] {
            let _ = sender.send(WorkletControl::ExitWorklet(worklet_id));
        }
        self.wake_threads();
    }

    /// For testing.
    #[cfg(feature = "testbinding")]
    pub(crate) fn test_worklet_lookup(&self, id: WorkletId, key: String) -> Option<String> {
        let (sender, receiver) = unbounded();
        let msg = WorkletData::Task(id, WorkletTask::Test(TestWorkletTask::Lookup(key, sender)));
        let _ = self.primary_sender.send(msg);
        receiver.recv().expect("Test worklet has died?")
    }

    fn wake_threads(&self) {
        // If any of the threads are blocked waiting on data, wake them up.
        let _ = self.cold_backup_sender.send(WorkletData::WakeUp);
        let _ = self.hot_backup_sender.send(WorkletData::WakeUp);
        let _ = self.primary_sender.send(WorkletData::WakeUp);
    }
}

/// The data messages sent to worklet threads
enum WorkletData {
    Task(WorkletId, WorkletTask),
    StartSwapRoles(Sender<WorkletData>),
    FinishSwapRoles(Swapper<WorkletThreadRole>),
    WakeUp,
    Quit,
}

/// The control message sent to worklet threads
enum WorkletControl {
    ExitWorklet(WorkletId),
    FetchAndInvokeAWorkletScript {
        pipeline_id: PipelineId,
        worklet_id: WorkletId,
        global_type: WorkletGlobalScopeType,
        origin: ImmutableOrigin,
        base_url: ServoUrl,
        script_url: ServoUrl,
        credentials: RequestCredentials,
        pending_tasks_struct: PendingTasksStruct,
        promise: TrustedPromise,
    },
}

/// A role that a worklet thread can be playing.
///
/// These roles are used as tokens or capabilities, we track unique
/// ownership using Rust's types, and use atomic swapping to exchange
/// them between worklet threads. This ensures that each thread pool has
/// exactly one primary, one hot backup and one cold backup.
struct WorkletThreadRole {
    receiver: Receiver<WorkletData>,
    sender: Sender<WorkletData>,
    is_hot_backup: bool,
    is_cold_backup: bool,
}

impl WorkletThreadRole {
    fn new(is_hot_backup: bool, is_cold_backup: bool) -> WorkletThreadRole {
        let (sender, receiver) = unbounded();
        WorkletThreadRole {
            sender,
            receiver,
            is_hot_backup,
            is_cold_backup,
        }
    }
}

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

/// A thread for executing worklets.
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct WorkletThread {
    /// Which role the thread is currently playing
    role: WorkletThreadRole,

    /// The thread's receiver for control messages
    control_receiver: Receiver<WorkletControl>,

    /// Senders
    primary_sender: Sender<WorkletData>,
    hot_backup_sender: Sender<WorkletData>,
    cold_backup_sender: Sender<WorkletData>,

    /// Data for initializing new worklet global scopes
    global_init: WorkletGlobalScopeInit,

    /// The global scopes created by this thread
    global_scopes: HashMap<WorkletId, Dom<WorkletGlobalScope>>,

    /// A one-place buffer for control messages
    control_buffer: Option<WorkletControl>,

    /// The JS runtime
    runtime: Runtime,
    should_gc: bool,
    gc_threshold: u32,
}

#[allow(unsafe_code)]
unsafe impl JSTraceable for WorkletThread {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        debug!("Tracing worklet thread.");
        self.global_scopes.trace(trc);
    }
}

impl WorkletThread {
    /// Spawn a new worklet thread, returning the channel to send it control messages.
    #[allow(unsafe_code)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn spawn(
        role: WorkletThreadRole,
        init: WorkletThreadInit,
        thread_index: u8,
    ) -> Sender<WorkletControl> {
        let (control_sender, control_receiver) = unbounded();
        let _ = thread::Builder::new()
            .name(format!("Worklet#{thread_index}"))
            .spawn(move || {
                // TODO: add a new IN_WORKLET thread state?
                // TODO: set interrupt handler?
                // TODO: configure the JS runtime (e.g. discourage GC, encourage agressive JIT)
                debug!("Initializing worklet thread.");
                thread_state::initialize(ThreadState::SCRIPT | ThreadState::IN_WORKER);
                let roots = RootCollection::new();
                let _stack_roots = ThreadLocalStackRoots::new(&roots);
                let mut thread = RootedTraceableBox::new(WorkletThread {
                    role,
                    control_receiver,
                    primary_sender: init.primary_sender,
                    hot_backup_sender: init.hot_backup_sender,
                    cold_backup_sender: init.cold_backup_sender,
                    global_init: init.global_init,
                    global_scopes: HashMap::new(),
                    control_buffer: None,
                    runtime: Runtime::new(None),
                    should_gc: false,
                    gc_threshold: MIN_GC_THRESHOLD,
                });
                thread.run(CanGc::note());
            })
            .expect("Couldn't start worklet thread");
        control_sender
    }

    /// The main event loop for a worklet thread
    fn run(&mut self, can_gc: CanGc) {
        loop {
            // The handler for data messages
            let message = self.role.receiver.recv().unwrap();
            match message {
                // The whole point of this thread pool is to perform tasks!
                WorkletData::Task(id, task) => {
                    self.perform_a_worklet_task(id, task);
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
            if self.role.is_cold_backup {
                if let Some(control) = self.control_buffer.take() {
                    self.process_control(control, can_gc);
                }
                while let Ok(control) = self.control_receiver.try_recv() {
                    self.process_control(control, can_gc);
                }
                self.gc();
            } else if self.control_buffer.is_none() {
                if let Ok(control) = self.control_receiver.try_recv() {
                    self.control_buffer = Some(control);
                    let msg = WorkletData::StartSwapRoles(self.role.sender.clone());
                    let _ = self.cold_backup_sender.send(msg);
                }
            }
            // If we are tight on memory, and we're a backup then perform a gc.
            // If we are tight on memory, and we're the primary then try to become the hot backup.
            // Hopefully this happens soon!
            if self.current_memory_usage() > self.gc_threshold {
                if self.role.is_hot_backup || self.role.is_cold_backup {
                    self.should_gc = false;
                    self.gc();
                } else if !self.should_gc {
                    self.should_gc = true;
                    let msg = WorkletData::StartSwapRoles(self.role.sender.clone());
                    let _ = self.hot_backup_sender.send(msg);
                }
            }
        }
    }

    /// The current memory usage of the thread
    #[allow(unsafe_code)]
    fn current_memory_usage(&self) -> u32 {
        unsafe { JS_GetGCParameter(self.runtime.cx(), JSGCParamKey::JSGC_BYTES) }
    }

    /// Perform a GC.
    #[allow(unsafe_code)]
    fn gc(&mut self) {
        debug!(
            "BEGIN GC (usage = {}, threshold = {}).",
            self.current_memory_usage(),
            self.gc_threshold
        );
        unsafe { JS_GC(self.runtime.cx(), GCReason::API) };
        self.gc_threshold = max(MIN_GC_THRESHOLD, self.current_memory_usage() * 2);
        debug!(
            "END GC (usage = {}, threshold = {}).",
            self.current_memory_usage(),
            self.gc_threshold
        );
    }

    /// Get the worklet global scope for a given worklet.
    /// Creates the worklet global scope if it doesn't exist.
    fn get_worklet_global_scope(
        &mut self,
        pipeline_id: PipelineId,
        worklet_id: WorkletId,
        global_type: WorkletGlobalScopeType,
        base_url: ServoUrl,
    ) -> DomRoot<WorkletGlobalScope> {
        match self.global_scopes.entry(worklet_id) {
            hash_map::Entry::Occupied(entry) => DomRoot::from_ref(entry.get()),
            hash_map::Entry::Vacant(entry) => {
                debug!("Creating new worklet global scope.");
                let executor = WorkletExecutor::new(worklet_id, self.primary_sender.clone());
                let result = WorkletGlobalScope::new(
                    global_type,
                    &self.runtime,
                    pipeline_id,
                    base_url,
                    executor,
                    &self.global_init,
                );
                entry.insert(Dom::from_ref(&*result));
                result
            },
        }
    }

    /// Fetch and invoke a worklet script.
    /// <https://drafts.css-houdini.org/worklets/#fetch-and-invoke-a-worklet-script>
    #[allow(clippy::too_many_arguments)]
    fn fetch_and_invoke_a_worklet_script(
        &self,
        global_scope: &WorkletGlobalScope,
        pipeline_id: PipelineId,
        origin: ImmutableOrigin,
        script_url: ServoUrl,
        credentials: RequestCredentials,
        pending_tasks_struct: PendingTasksStruct,
        promise: TrustedPromise,
        can_gc: CanGc,
    ) {
        debug!("Fetching from {}.", script_url);
        // Step 1.
        // TODO: Settings object?

        // Step 2.
        // TODO: Fetch a module graph, not just a single script.
        // TODO: Fetch the script asynchronously?
        // TODO: Caching.
        let resource_fetcher = self.global_init.resource_threads.sender();
        let request = RequestBuilder::new(
            None,
            script_url,
            global_scope.upcast::<GlobalScope>().get_referrer(),
        )
        .destination(Destination::Script)
        .mode(RequestMode::CorsMode)
        .credentials_mode(credentials.convert())
        .origin(origin);

        let script = load_whole_resource(
            request,
            &resource_fetcher,
            global_scope.upcast::<GlobalScope>(),
            can_gc,
        )
        .ok()
        .and_then(|(_, bytes)| String::from_utf8(bytes).ok());

        // Step 4.
        // NOTE: the spec parses and executes the script in separate steps,
        // but our JS API doesn't separate these, so we do the steps out of order.
        // Also, the spec currently doesn't allow exceptions to be propagated
        // to the main script thread.
        // https://github.com/w3c/css-houdini-drafts/issues/407
        let ok = script
            .map(|script| global_scope.evaluate_js(&script, can_gc))
            .unwrap_or(false);

        if !ok {
            // Step 3.
            debug!("Failed to load script.");
            let old_counter = pending_tasks_struct.set_counter_to(-1);
            if old_counter > 0 {
                self.run_in_script_thread(promise.reject_task(Error::Abort));
            }
        } else {
            // Step 5.
            debug!("Finished adding script.");
            let old_counter = pending_tasks_struct.decrement_counter_by(1);
            if old_counter == 1 {
                debug!("Resolving promise.");
                let msg = MainThreadScriptMsg::WorkletLoaded(pipeline_id);
                self.global_init
                    .to_script_thread_sender
                    .send(msg)
                    .expect("Worklet thread outlived script thread.");
                self.run_in_script_thread(promise.resolve_task(()));
            }
        }
    }

    /// Perform a task.
    fn perform_a_worklet_task(&self, worklet_id: WorkletId, task: WorkletTask) {
        match self.global_scopes.get(&worklet_id) {
            Some(global) => global.perform_a_worklet_task(task),
            None => warn!("No such worklet as {:?}.", worklet_id),
        }
    }

    /// Process a control message.
    fn process_control(&mut self, control: WorkletControl, can_gc: CanGc) {
        match control {
            WorkletControl::ExitWorklet(worklet_id) => {
                self.global_scopes.remove(&worklet_id);
            },
            WorkletControl::FetchAndInvokeAWorkletScript {
                pipeline_id,
                worklet_id,
                global_type,
                origin,
                base_url,
                script_url,
                credentials,
                pending_tasks_struct,
                promise,
            } => {
                let global =
                    self.get_worklet_global_scope(pipeline_id, worklet_id, global_type, base_url);
                self.fetch_and_invoke_a_worklet_script(
                    &global,
                    pipeline_id,
                    origin,
                    script_url,
                    credentials,
                    pending_tasks_struct,
                    promise,
                    can_gc,
                )
            },
        }
    }

    /// Run a task in the main script thread.
    fn run_in_script_thread<T>(&self, task: T)
    where
        T: TaskBox + 'static,
    {
        // NOTE: It's unclear which task source should be used here:
        // https://drafts.css-houdini.org/worklets/#dom-worklet-addmodule
        let msg = CommonScriptMsg::Task(
            ScriptThreadEventCategory::WorkletEvent,
            Box::new(task),
            None,
            TaskSourceName::DOMManipulation,
        );
        let msg = MainThreadScriptMsg::Common(msg);
        self.global_init
            .to_script_thread_sender
            .send(msg)
            .expect("Worklet thread outlived script thread.");
    }
}

/// An executor of worklet tasks
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) struct WorkletExecutor {
    worklet_id: WorkletId,
    #[no_trace]
    primary_sender: Sender<WorkletData>,
}

impl WorkletExecutor {
    fn new(worklet_id: WorkletId, primary_sender: Sender<WorkletData>) -> WorkletExecutor {
        WorkletExecutor {
            worklet_id,
            primary_sender,
        }
    }

    /// Schedule a worklet task to be peformed by the worklet thread pool.
    pub(crate) fn schedule_a_worklet_task(&self, task: WorkletTask) {
        let _ = self
            .primary_sender
            .send(WorkletData::Task(self.worklet_id, task));
    }
}
