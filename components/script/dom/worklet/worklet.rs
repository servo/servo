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
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

use crossbeam_channel::{Receiver, SendError, Sender, unbounded};
use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use malloc_size_of::malloc_size_of_is_0;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{Destination, Origin, PreloadedResources, RequestClient};
use rustc_hash::FxHashMap;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use servo_base::id::PipelineId;
use servo_url::{ImmutableOrigin, ServoUrl};
use swapper::Swapper;
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
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
#[cfg(feature = "testbinding")]
use crate::dom::window::Window;
use crate::dom::workletglobalscope::{
    WorkletGlobalScope, WorkletGlobalScopeInit, WorkletGlobalScopeType,
};
use crate::messaging::{CommonScriptMsg, MainThreadScriptMsg, ScriptEventLoopSender};
use crate::microtask::MicrotaskQueue;
use crate::modules::script_module::fetch_a_module_script_graph;
use crate::realms::enter_auto_realm;
use crate::script_runtime::{IntroductionType, Runtime, ScriptThreadEventCategory};
use crate::tasks::task_source::TaskSourceName;
use crate::url::ensure_blob_referenced_by_url_is_kept_alive;

// Magic numbers
const WORKLET_THREAD_POOL_SIZE: u32 = 3;
pub(crate) const MIN_GC_THRESHOLD: u32 = 1_000_000;

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
    fn exit_worklet(&self, worklet_id: WorkletId);
    fn wake_threads(&self);
    /// Send a `WorkletTask` to a Worklet thread to execute.
    fn run_task(&self, worklet_id: WorkletId, worklet_task: WorkletTask);
}

// A boxed closure sent to the "Primary Worklet Thread" to execute Worklet tasks.
pub(crate) type WorkletTask = Box<dyn FnOnce(&mut JSContext, &WorkletGlobalScope) + Send>;

/// The data messages sent to worklet threads
pub(crate) enum WorkletData {
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
pub(crate) struct WorkletThreadRole {
    receiver: Receiver<WorkletData>,
    sender: Sender<WorkletData>,
    is_hot_backup: bool,
    is_cold_backup: bool,
}

impl WorkletThreadRole {
    pub(crate) fn new(is_hot_backup: bool, is_cold_backup: bool) -> WorkletThreadRole {
        let (sender, receiver) = unbounded();
        WorkletThreadRole {
            sender,
            receiver,
            is_hot_backup,
            is_cold_backup,
        }
    }

    pub(crate) fn sender(&self) -> Sender<WorkletData> {
        self.sender.clone()
    }

    pub(crate) fn receiver(&self) -> Receiver<WorkletData> {
        self.receiver.clone()
    }

    pub(crate) fn is_hot_backup(&self) -> bool {
        self.is_hot_backup
    }

    pub(crate) fn is_cold_backup(&self) -> bool {
        self.is_cold_backup
    }
}

/// WorkletThread contains the common Worklet infrastructure used by both the Stateless and
/// Stateful Worklets
/// <https://html.spec.whatwg.org/multipage/#worklets-worklet>
pub(crate) struct WorkletThread {
    /// Data for initializing new worklet global scopes
    pub(crate) global_init: WorkletGlobalScopeInit,

    /// The global scopes created by this thread
    pub(crate) global_scopes: FxHashMap<WorkletId, Dom<WorkletGlobalScope>>,

    /// The thread's receiver for control messages
    control_receiver: Receiver<WorkletControl>,
    /// The sender for sending control messages to this thread's event loop
    pub(crate) control_sender: Sender<WorkletControl>,

    /// A one-place buffer for control messages
    pub(crate) control_buffer: Option<WorkletControl>,

    /// A flag that is set when a `WorkletThread` begins shutting down.
    pub(crate) closing: Arc<AtomicBool>,

    /// The JS runtime
    runtime: Runtime,
    pub(crate) should_gc: bool,
    pub(crate) gc_threshold: u32,
}

impl WorkletThread {
    pub(crate) fn new(
        global_init: WorkletGlobalScopeInit,
        control_receiver: Receiver<WorkletControl>,
        control_sender: Sender<WorkletControl>,
        runtime: Runtime,
    ) -> Self {
        WorkletThread {
            global_init,
            global_scopes: FxHashMap::default(),
            control_receiver,
            control_sender,
            control_buffer: None,
            closing: Arc::new(AtomicBool::new(false)),
            runtime,
            should_gc: false,
            gc_threshold: MIN_GC_THRESHOLD,
        }
    }

    /// Fetch and invoke a worklet script.
    /// <https://html.spec.whatwg.org/multipage/#fetch-a-worklet-script-graph>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fetch_and_invoke_a_worklet_script(
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

    pub(crate) fn control_receiver(&self) -> Receiver<WorkletControl> {
        self.control_receiver.clone()
    }

    pub(crate) fn runtime_cx_no_gc(&self) -> &JSContext {
        self.runtime.cx_no_gc()
    }

    pub(crate) fn microtask_queue(&self) -> Rc<MicrotaskQueue> {
        self.runtime.microtask_queue.clone()
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
    pub(crate) fn new(
        worklet_id: WorkletId,
        primary_sender: Sender<WorkletData>,
        hot_backup_sender: Sender<WorkletData>,
        cold_backup_sender: Sender<WorkletData>,
        control_sender: Sender<WorkletControl>,
    ) -> Self {
        WorkletExecutor {
            worklet_id,
            primary_sender: primary_sender,
            hot_backup_sender: hot_backup_sender,
            cold_backup_sender: cold_backup_sender,
            control_sender: control_sender,
        }
    }

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
