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

use std::cell::{self, Cell, RefCell, RefMut};
use std::cmp::max;
use std::collections::hash_map;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::thread;

use crossbeam_channel::{Receiver, SendError, Sender, unbounded};
use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsapi::{GCReason, JSGCParamKey, JSTracer};
use js::realm::CurrentRealm;
use js::rust::wrappers2::{JS_GC, JS_GetGCParameter};
use malloc_size_of::malloc_size_of_is_0;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{Destination, Origin, PreloadedResources, RequestClient};
use rustc_hash::FxHashMap;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use servo_base::id::PipelineId;
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
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::trace::{JSTraceable, RootedTraceableBox};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::dom::workletglobalscope::{
    WorkletGlobalScope, WorkletGlobalScopeInit, WorkletGlobalScopeType,
};
use crate::messaging::{CommonScriptMsg, MainThreadScriptMsg, ScriptEventLoopSender};
use crate::modules::script_module::fetch_a_module_script_graph;
use crate::realms::enter_auto_realm;
use crate::runtime::microtask::MicrotaskQueue;
use crate::runtime::script_runtime::{IntroductionType, Runtime, ScriptThreadEventCategory};
use crate::tasks::task_source::TaskSourceName;
use crate::url::ensure_blob_referenced_by_url_is_kept_alive;

// Magic numbers
const WORKLET_THREAD_POOL_SIZE: u32 = 3;
const MIN_GC_THRESHOLD: u32 = 1_000_000;

type LazyCellWithBoxedInitializer<T> = cell::LazyCell<T, Box<dyn FnOnce() -> T>>;

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableField {
    worklet_id: WorkletId,
    /// The cached version of the script thread's WorkletThreadPool. We keep this cached
    /// because we may need to access it after the script thread has terminated.
    /// NOTE: Do not access the `thread_pool` field directly, instead use the
    /// `Worklet::worklet_thread_pool` method to access the Thread Pool.
    #[ignore_malloc_size_of = "Difficult to measure memory usage of Rc<...> types"]
    thread_pool: LazyCellWithBoxedInitializer<Rc<dyn WorkletThreadPool>>,

    /// NOTE: The `is_thread_pool_initialized` field is a temporary workaround because
    /// using the `LazyCell::get()` method requires Rust version >1.94.0 and is not
    /// supported by the current MSRV (Minimum Supported Rust Version).
    is_thread_pool_initialized: Cell<bool>,
}

impl Drop for DroppableField {
    fn drop(&mut self) {
        let worklet_id = self.worklet_id;
        if self.is_thread_pool_initialized.get() {
            self.thread_pool.exit_worklet(worklet_id);
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
    fn new_inherited(
        window: &Window,
        global_type: WorkletGlobalScopeType,
        thread_pool_constructor: Box<dyn FnOnce() -> Rc<dyn WorkletThreadPool>>,
    ) -> Worklet {
        Worklet {
            reflector: Reflector::new(),
            window: Dom::from_ref(window),
            global_type,
            droppable_field: DroppableField {
                worklet_id: WorkletId::new(),
                thread_pool: LazyCellWithBoxedInitializer::new(thread_pool_constructor),
                is_thread_pool_initialized: Cell::new(false),
            },
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        global_type: WorkletGlobalScopeType,
        thread_pool_constructor: Box<dyn FnOnce() -> Rc<dyn WorkletThreadPool>>,
    ) -> DomRoot<Worklet> {
        debug!("Creating worklet {:?}.", global_type);
        reflect_dom_object_with_cx(
            Box::new(Worklet::new_inherited(
                window,
                global_type,
                thread_pool_constructor,
            )),
            window,
            cx,
        )
    }

    pub(crate) fn worklet_thread_pool(&self) -> Rc<dyn WorkletThreadPool> {
        self.droppable_field.is_thread_pool_initialized.set(true);
        self.droppable_field.thread_pool.clone()
    }

    #[cfg(feature = "testbinding")]
    pub(crate) fn worklet_id(&self) -> WorkletId {
        self.droppable_field.worklet_id
    }

    #[expect(dead_code)]
    pub(crate) fn worklet_global_scope_type(&self) -> WorkletGlobalScopeType {
        self.global_type
    }
}

impl WorkletMethods<crate::DomTypeHolder> for Worklet {
    /// <https://html.spec.whatwg.org/multipage/#dom-worklet-addmodule>
    fn AddModule(
        &self,
        realm: &mut CurrentRealm,
        module_url: USVString,
        options: &WorkletOptions,
    ) -> Rc<Promise> {
        let promise = Promise::new_in_realm(realm);

        // Step 1. Let outsideSettings be the relevant settings object of this.
        // Step 2. Let moduleURLRecord be the result of encoding-parsing a URL given moduleURL, relative to outsideSettings.
        let module_url_record = match self.window.Document().base_url().join(&module_url.0) {
            Ok(url) => url,
            Err(err) => {
                // Step 3. If moduleURLRecord is failure, then return a promise rejected with a "SyntaxError" DOMException.
                debug!("URL {:?} parse error {:?}.", module_url.0, err);
                promise.reject_error(realm, Error::Syntax(None));

                return promise;
            },
        };
        debug!("Adding Worklet module {}.", module_url_record);

        let global_scope = self.window.as_global_scope();

        let pending_tasks_struct = PendingTasksStruct::new();

        // NOTE: The following steps are split between `WorkletThread::get_worklet_global_scope` and `WorkledThread::fetch_and_invoke_a_worklet_script` methods:
        // Step 5. Let workletInstance be this.
        // Step 6. Run the following steps in parallel:
        // Step 6.1. If workletInstance's global scopes is empty:
        // Step 6.1.1. Create a worklet global scope given workletInstance.
        // Step 6.1.2. Optionally, create additional global scope instances given workletInstance, depending on the specific worklet in question and its specification.
        // Step 6.1.3. Wait for all steps of the creation process(es) — including those taking place within the worklet agents — to complete, before moving on.
        // Step 6.2. Let pendingTasks be workletInstance's global scopes's size.

        // Step 6.3. Let addedSuccessfully be false.
        // NOTE: We skip step 6.3 because we do not implement the `added modules list` yet
        // <https://html.spec.whatwg.org/multipage/#concept-worklet-added-modules-list>

        self.worklet_thread_pool()
            .fetch_and_invoke_a_worklet_script(
                self.window.pipeline_id(),
                self.droppable_field.worklet_id,
                self.global_type,
                self.window.origin().immutable().clone(),
                global_scope.api_base_url(),
                module_url_record,
                global_scope.policy_container(),
                options.credentials,
                pending_tasks_struct,
                &promise,
                global_scope.inherited_secure_context(),
            );

        // Step 7. Return promise
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
        WorkletId(Uuid::new_v4())
    }
}

/// <https://drafts.css-houdini.org/worklets/#pending-tasks-struct>
#[derive(Clone, Debug)]
pub(crate) struct PendingTasksStruct(Arc<AtomicIsize>);

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

pub trait WorkletThreadPool: JSTraceable {
    /// Loads a worklet module into every thread in this thread pool.
    /// If all of the threads load successfully, the promise is resolved.
    /// If any of the threads fails to load, the promise is rejected.
    /// NOTE: The method implements the Step 6 of AddModule
    /// <https://html.spec.whatwg.org/multipage/#dom-worklet-addmodule>
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
    );
    /// Request that the [`WorkletGlobalScope`] associated with the [`WorkletId`]
    /// be removed from all the threads in the thread pool.
    fn exit_worklet(&self, worklet_id: WorkletId);
    /// Signal all the threads in the pool that there may be control messages to
    /// process.
    fn wake_threads(&self);
    /// Queue a [`WorkletTask`] for execution on this [`WorkletThreadPool`].
    /// The task will be executed in the context of the [`WorkletGlobalScope`]
    /// represented by the [`WorketId`].
    fn perform_a_worklet_task(&self, worklet_id: WorkletId, worklet_task: WorkletTask);
}

/// The `StatelessWorkletThreadPool` executes the associated
/// [`WorkletTask`]s in a dedicated thread pool with the assumption that
/// the tasks are idempotent. This is useful for the paint worklet, for
/// example.
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

impl StatelessWorkletThreadPool {
    /// Create a new thread pool and spawn the threads.
    /// When the thread pool is dropped, the threads will be asked to quit.
    pub(crate) fn spawn(global_init: WorkletGlobalScopeInit) -> StatelessWorkletThreadPool {
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
        StatelessWorkletThreadPool {
            primary_sender,
            hot_backup_sender,
            cold_backup_sender,
            control_sender_0: WorkletThread::spawn(primary_role, init.clone(), 0),
            control_sender_1: WorkletThread::spawn(hot_backup_role, init.clone(), 1),
            control_sender_2: WorkletThread::spawn(cold_backup_role, init, 2),
        }
    }
}

impl WorkletThreadPool for StatelessWorkletThreadPool {
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

    /// Queue the [`WorkletTask`] for execution on the primary thread.
    fn perform_a_worklet_task(&self, worklet_id: WorkletId, worklet_task: WorkletTask) {
        let msg = WorkletData::Task(worklet_id, worklet_task);
        let _ = self.primary_sender.send(msg);
    }
}

/// A task which can be performed in the context of a [`WorkletGlobalScope`].
type WorkletTask = Box<dyn FnOnce(&mut JSContext, &WorkletGlobalScope) + Send>;

/// The data messages sent to worklet threads
enum WorkletData {
    Task(WorkletId, WorkletTask),
    StartSwapRoles(Sender<WorkletData>),
    FinishSwapRoles(Swapper<WorkletThreadRole>),
    WakeUp,
    Quit,
}

/// The control message sent to worklet threads
pub(crate) enum WorkletControl {
    ExitWorklet(WorkletId),
    FetchAndInvokeAWorkletScript {
        pipeline_id: PipelineId,
        worklet_id: WorkletId,
        global_type: WorkletGlobalScopeType,
        origin: ImmutableOrigin,
        base_url: ServoUrl,
        script_url: ServoUrl,
        policy_container: PolicyContainer,
        credentials: RequestCredentials,
        pending_tasks_struct: PendingTasksStruct,
        promise: TrustedPromise,
        inherited_secure_context: Option<bool>,
    },
    Common(CommonScriptMsg),
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
    /// The sender for sending control messages to this thread's event loop
    control_sender: Sender<WorkletControl>,

    /// Senders
    primary_sender: Sender<WorkletData>,
    hot_backup_sender: Sender<WorkletData>,
    cold_backup_sender: Sender<WorkletData>,

    /// Data for initializing new worklet global scopes
    global_init: WorkletGlobalScopeInit,

    /// The global scopes created by this thread
    global_scopes: FxHashMap<WorkletId, Dom<WorkletGlobalScope>>,

    /// A one-place buffer for control messages
    control_buffer: Option<WorkletControl>,

    /// A flag that is set when a `WorkletThread` begins shutting down.
    closing: Arc<AtomicBool>,

    /// The JS runtime
    runtime: Runtime,
    should_gc: bool,
    gc_threshold: u32,
}

#[expect(unsafe_code)]
unsafe impl JSTraceable for WorkletThread {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        debug!("Tracing worklet thread.");
        unsafe { self.global_scopes.trace(trc) };
    }
}

impl WorkletThread {
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
                let mut thread = RootedTraceableBox::new(WorkletThread {
                    role,
                    control_receiver,
                    control_sender: control_sender_clone,
                    primary_sender: init.primary_sender,
                    hot_backup_sender: init.hot_backup_sender,
                    cold_backup_sender: init.cold_backup_sender,
                    global_init: init.global_init,
                    global_scopes: FxHashMap::default(),
                    control_buffer: None,
                    runtime,
                    should_gc: false,
                    closing: Arc::new(AtomicBool::new(false)),
                    gc_threshold: MIN_GC_THRESHOLD,
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
            let message = self.role.receiver.recv().unwrap();
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
            if self.role.is_cold_backup {
                if let Some(control) = self.control_buffer.take() {
                    self.process_control(control, cx);
                }
                while let Ok(control) = self.control_receiver.try_recv() {
                    self.process_control(control, cx);
                }

                for worklet_global_scope in self.global_scopes.values() {
                    worklet_global_scope.perform_a_microtask_checkpoint(cx);
                }

                self.gc(cx);
            } else if self.control_buffer.is_none() &&
                let Ok(control) = self.control_receiver.try_recv()
            {
                self.control_buffer = Some(control);
                let msg = WorkletData::StartSwapRoles(self.role.sender.clone());
                let _ = self.cold_backup_sender.send(msg);
            }
            // If we are tight on memory, and we're a backup then perform a gc.
            // If we are tight on memory, and we're the primary then try to become the hot backup.
            // Hopefully this happens soon!
            if self.current_memory_usage() > self.gc_threshold {
                if self.role.is_hot_backup || self.role.is_cold_backup {
                    self.should_gc = false;
                    self.gc(cx);
                } else if !self.should_gc {
                    self.should_gc = true;
                    let msg = WorkletData::StartSwapRoles(self.role.sender.clone());
                    let _ = self.hot_backup_sender.send(msg);
                }
            }
        }
    }

    /// The current memory usage of the thread
    #[expect(unsafe_code)]
    fn current_memory_usage(&self) -> u32 {
        unsafe { JS_GetGCParameter(self.runtime.cx_no_gc(), JSGCParamKey::JSGC_BYTES) }
    }

    /// Perform a GC.
    #[expect(unsafe_code)]
    fn gc(&mut self, cx: &mut JSContext) {
        debug!(
            "BEGIN GC (usage = {}, threshold = {}).",
            self.current_memory_usage(),
            self.gc_threshold
        );
        unsafe { JS_GC(cx, GCReason::API) };
        self.gc_threshold = max(MIN_GC_THRESHOLD, self.current_memory_usage() * 2);
        debug!(
            "END GC (usage = {}, threshold = {}).",
            self.current_memory_usage(),
            self.gc_threshold
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
        match self.global_scopes.entry(worklet_id) {
            hash_map::Entry::Occupied(entry) => DomRoot::from_ref(entry.get()),

            // Step 6.1. If workletInstance's global scopes is empty:
            hash_map::Entry::Vacant(entry) => {
                debug!("Creating new worklet global scope.");

                // Step 6.1.1. Create a worklet global scope given workletInstance.
                let executor = WorkletExecutor {
                    worklet_id,
                    primary_sender: self.primary_sender.clone(),
                    hot_backup_sender: self.hot_backup_sender.clone(),
                    cold_backup_sender: self.cold_backup_sender.clone(),
                    control_sender: self.control_sender.clone(),
                };

                let result = WorkletGlobalScope::new(
                    global_type,
                    pipeline_id,
                    base_url,
                    inherited_secure_context,
                    executor,
                    &self.global_init,
                    cx,
                    self.closing.clone(),
                    microtask_queue,
                );
                entry.insert(Dom::from_ref(&*result));
                result
            },
        }
    }

    /// Fetch and invoke a worklet script.
    /// <https://html.spec.whatwg.org/multipage/#fetch-a-worklet-script-graph>
    #[allow(clippy::too_many_arguments)]
    fn fetch_and_invoke_a_worklet_script(
        &self,
        global_scope: &WorkletGlobalScope,
        pipeline_id: PipelineId,
        origin: ImmutableOrigin,
        script_url: ServoUrl,
        policy_container: PolicyContainer,
        credentials: RequestCredentials,
        pending_tasks_struct: PendingTasksStruct,
        promise: TrustedPromise,
        cx: &mut JSContext,
    ) {
        debug!("Fetching from {}.", script_url);
        // TODO: Settings object?

        // TODO: Fetch the script asynchronously?
        // TODO: Caching.
        let global = global_scope.upcast::<GlobalScope>();

        // Step 1. Let requestURL be request's URL.
        let request_client = RequestClient {
            preloaded_resources: PreloadedResources::default(),
            policy_container,
            origin: Origin::Origin(origin),
            is_nested_browsing_context: global.is_nested_browsing_context(),
            insecure_requests_policy: global.insecure_requests_policy(),
            has_trustworthy_ancestor_origin: global.has_trustworthy_ancestor_origin(),
        };

        // Step 2. If moduleResponsesMap[requestURL] is "fetching", wait in parallel until that entry's value changes, then queue a task on the networking task source to proceed with running the following steps.
        // NOTE: We do not perform the Step 2 because Worklet currently does not implement a `module responses map`
        // <https://html.spec.whatwg.org/multipage/#concept-worklet-module-responses-map>

        // `fetch_a_module_script_graph` requires the `on_complete` closure to be cloneable
        // therefore, we wrap the TrustedPromise in an Rc to make it cloneable and RefCell allows calling `reject_task` and `resolve_task`
        let promise_task = Rc::new(RefCell::new(Some(promise)));
        let script_thread_sender = self.global_init.to_script_thread_sender.clone();
        let rooted_global = DomRoot::from_ref(global);
        let script_url = ensure_blob_referenced_by_url_is_kept_alive(global, script_url);

        // NOTE: We implement the rest of the steps in AddModule here
        // <https://html.spec.whatwg.org/multipage/#dom-worklet-addmodule>
        // Step 6.4. For each workletGlobalScope of workletInstance's global scopes,
        // queue a global task on the networking task source given workletGlobalScope to fetch a worklet script graph given moduleURLRecord,
        // outsideSettings, workletInstance's worklet destination type, options["credentials"], workletGlobalScope's relevant settings object,
        // workletInstance's module responses map, and the following steps given script:
        fetch_a_module_script_graph(
            cx,
            global,
            script_url,
            request_client,
            Destination::PaintWorklet,
            global.get_referrer(),
            credentials.convert(),
            Some(IntroductionType::WORKLET),
            move |cx, module_tree| {
                match module_tree {
                    // Step 6.4.1. If script is null:
                    None => {
                        debug!("Failed to load script.");

                        reject_promise(
                            &pending_tasks_struct,
                            promise_task.borrow_mut(),
                            script_thread_sender.clone(),
                        );
                    },
                    Some(script) => {
                        let mut realm = enter_auto_realm(cx, &*rooted_global);
                        let cx = &mut realm.current_realm();

                        // Step 6.4.2. If script's error to rethrow is not null:
                        // NOTE: The `AddModule` specification in the Step 6.4.2.1.1.2. requires the promise to be rejected with the script's "rethrow error".
                        // However, the `JSVal` from `get_rethrow_error` cannot be used with the `promise_task` here because they are from different runtimes.
                        // So we throw an AbortError instead.
                        if script.get_rethrow_error().take().is_some() {
                            // Step 6.4.2.1. and its substeps are handled by `reject_promise` function
                            reject_promise(
                                &pending_tasks_struct,
                                promise_task.borrow_mut(),
                                script_thread_sender.clone(),
                            );

                            // Step 6.4.2.2. Abort these steps.
                            return;
                        }

                        // Step 6.4.4. Run a module script given script.
                        rooted_global.run_a_module_script(cx, script, false);

                        // NOTE: we are treating all negative values as -1
                        // Step 6.4.5.1. If pendingTasks is not −1:
                        // Step 6.4.5.1.1. Set pendingTasks to pendingTasks − 1.
                        let old_counter = pending_tasks_struct.decrement_counter_by(1);
                        // Step Step 6.4.5.1.2. If pendingTasks is 0 then, resolve promise.
                        if old_counter == 1 {
                            debug!("Resolving promise.");

                            let msg = MainThreadScriptMsg::WorkletLoaded(pipeline_id);
                            script_thread_sender
                                .send(msg)
                                .expect("Worklet thread outlived script thread.");

                            let task = promise_task
                                .borrow_mut()
                                .take()
                                .expect("promise_task must be consumed exactly once")
                                .resolve_task(());

                            let msg = CommonScriptMsg::Task(
                                ScriptThreadEventCategory::WorkletEvent,
                                Box::new(task),
                                None,
                                TaskSourceName::Networking,
                            );

                            // Step 6.4.5. Queue a global task on the networking task source given workletInstance's relevant global object to perform the following steps:
                            let msg = MainThreadScriptMsg::Common(msg);
                            script_thread_sender
                                .send(msg)
                                .expect("Worklet thread outlived script thread.");
                        }
                    },
                }
            },
        );
    }

    /// Run the steps for the `WorkletTask` for a given Worklet.
    fn perform_a_worklet_task(
        &self,
        cx: &mut JSContext,
        worklet_id: WorkletId,
        worklet_task: WorkletTask,
    ) {
        match self.global_scopes.get(&worklet_id) {
            Some(global) => worklet_task(cx, global),
            None => warn!("No such worklet as {:?}.", worklet_id),
        }
    }

    /// Process a control message.
    fn process_control(&mut self, control: WorkletControl, cx: &mut js::context::JSContext) {
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
                    self.runtime.microtask_queue.clone(),
                );
                self.fetch_and_invoke_a_worklet_script(
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

/// This function is an abstraction of steps 6.4.1.1 and 6.4.2.1 of the `AddModule` spec
/// <https://html.spec.whatwg.org/multipage/#dom-worklet-addmodule>
pub(crate) fn reject_promise(
    pending_tasks_struct: &PendingTasksStruct,
    mut promise_task: RefMut<'_, Option<TrustedPromise>>,
    script_thread_sender: Sender<MainThreadScriptMsg>,
) {
    // Step 6.4.1.1.1.1. Set pendingTasks to −1
    let old_counter = pending_tasks_struct.set_counter_to(-1);

    // 6.4.1.1.1. If pendingTasks is not −1:
    if old_counter > 0 {
        // 6.4.1.1.1.2. Reject promise with an "AbortError" DOMException
        let task = promise_task
            .take()
            .expect("promise_task must be consumed exactly once")
            .reject_task(Error::Abort(None));

        let msg = CommonScriptMsg::Task(
            ScriptThreadEventCategory::WorkletEvent,
            Box::new(task),
            None,
            TaskSourceName::Networking,
        );

        // Step 6.4.1.1. Queue a global task on the networking task source given workletInstance's relevant global object to perform the following steps:
        let msg = MainThreadScriptMsg::Common(msg);
        script_thread_sender
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
    #[no_trace]
    hot_backup_sender: Sender<WorkletData>,
    #[no_trace]
    cold_backup_sender: Sender<WorkletData>,
    #[no_trace]
    control_sender: Sender<WorkletControl>,
}

impl WorkletExecutor {
    /// If any of the threads are blocked waiting on data, wake them up.
    pub(crate) fn wake_threads(&self) -> Result<(), SendError<()>> {
        self.cold_backup_sender
            .send(WorkletData::WakeUp)
            .map_err(|_| SendError(()))?;
        self.hot_backup_sender
            .send(WorkletData::WakeUp)
            .map_err(|_| SendError(()))?;
        self.primary_sender
            .send(WorkletData::WakeUp)
            .map_err(|_| SendError(()))
    }

    /// Schedule a worklet task to be peformed by the worklet thread pool.
    pub(crate) fn schedule_a_worklet_task(&self, task: WorkletTask) {
        let _ = self
            .primary_sender
            .send(WorkletData::Task(self.worklet_id, task));
    }

    pub(crate) fn send_control_message(
        &self,
        control_message: WorkletControl,
    ) -> Result<(), SendError<()>> {
        self.control_sender
            .send(control_message)
            .map_err(|_| SendError(()))?;
        self.wake_threads()
    }

    pub(crate) fn event_loop_sender(&self) -> ScriptEventLoopSender {
        ScriptEventLoopSender::Worklet(self.clone())
    }
}
