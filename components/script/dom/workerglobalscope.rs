/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{RefCell, RefMut};
use std::default::Default;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use base::cross_process_instant::CrossProcessInstant;
use base::id::{PipelineId, PipelineNamespace};
use constellation_traits::WorkerGlobalScopeInit;
use crossbeam_channel::Receiver;
use devtools_traits::{DevtoolScriptControlMsg, WorkerId};
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use js::jsval::UndefinedValue;
use js::panic::maybe_resume_unwind;
use js::rust::{HandleValue, MutableHandleValue, ParentRuntime};
use net_traits::IpcSend;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{
    CredentialsMode, Destination, InsecureRequestsPolicy, ParserMetadata,
    RequestBuilder as NetRequestInit,
};
use profile_traits::mem::ProcessReports;
use servo_url::{MutableOrigin, ServoUrl};
use timers::TimerScheduler;
use uuid::Uuid;

use super::bindings::codegen::Bindings::MessagePortBinding::StructuredSerializeOptions;
use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding::{
    ImageBitmapOptions, ImageBitmapSource,
};
use crate::dom::bindings::codegen::Bindings::RequestBinding::RequestInit;
use crate::dom::bindings::codegen::Bindings::VoidFunctionBinding::VoidFunction;
use crate::dom::bindings::codegen::Bindings::WorkerBinding::WorkerType;
use crate::dom::bindings::codegen::Bindings::WorkerGlobalScopeBinding::WorkerGlobalScopeMethods;
use crate::dom::bindings::codegen::UnionTypes::{RequestOrUSVString, StringOrFunction};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible, report_pending_exception};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::settings_stack::AutoEntryScript;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::crypto::Crypto;
use crate::dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performance::Performance;
use crate::dom::promise::Promise;
use crate::dom::trustedtypepolicyfactory::TrustedTypePolicyFactory;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::dom::window::{base64_atob, base64_btoa};
use crate::dom::workerlocation::WorkerLocation;
use crate::dom::workernavigator::WorkerNavigator;
use crate::fetch;
use crate::messaging::{CommonScriptMsg, ScriptEventLoopReceiver, ScriptEventLoopSender};
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::{CanGc, JSContext, JSContextHelper, Runtime};
use crate::task::TaskCanceller;
use crate::timers::{IsInterval, TimerCallback};

pub(crate) fn prepare_workerscope_init(
    global: &GlobalScope,
    devtools_sender: Option<IpcSender<DevtoolScriptControlMsg>>,
    worker_id: Option<WorkerId>,
) -> WorkerGlobalScopeInit {
    let init = WorkerGlobalScopeInit {
        resource_threads: global.resource_threads().clone(),
        mem_profiler_chan: global.mem_profiler_chan().clone(),
        to_devtools_sender: global.devtools_chan().cloned(),
        time_profiler_chan: global.time_profiler_chan().clone(),
        from_devtools_sender: devtools_sender,
        script_to_constellation_chan: global.script_to_constellation_chan().clone(),
        worker_id: worker_id.unwrap_or_else(|| WorkerId(Uuid::new_v4())),
        pipeline_id: global.pipeline_id(),
        origin: global.origin().immutable().clone(),
        creation_url: global.creation_url().clone(),
        inherited_secure_context: Some(global.is_secure_context()),
    };

    init
}

// https://html.spec.whatwg.org/multipage/#the-workerglobalscope-common-interface
#[dom_struct]
pub(crate) struct WorkerGlobalScope {
    globalscope: GlobalScope,

    worker_name: DOMString,
    worker_type: WorkerType,

    #[no_trace]
    worker_id: WorkerId,
    #[no_trace]
    worker_url: DomRefCell<ServoUrl>,
    #[ignore_malloc_size_of = "Arc"]
    closing: Arc<AtomicBool>,
    #[ignore_malloc_size_of = "Defined in js"]
    runtime: DomRefCell<Option<Runtime>>,
    location: MutNullableDom<WorkerLocation>,
    navigator: MutNullableDom<WorkerNavigator>,
    #[no_trace]
    /// <https://html.spec.whatwg.org/multipage/#the-workerglobalscope-common-interface:policy-container>
    policy_container: DomRefCell<PolicyContainer>,

    #[ignore_malloc_size_of = "Defined in ipc-channel"]
    #[no_trace]
    /// A `Sender` for sending messages to devtools. This is unused but is stored here to
    /// keep the channel alive.
    _devtools_sender: Option<IpcSender<DevtoolScriptControlMsg>>,

    #[ignore_malloc_size_of = "Defined in crossbeam"]
    #[no_trace]
    /// A `Receiver` for receiving messages from devtools.
    devtools_receiver: Option<Receiver<DevtoolScriptControlMsg>>,

    #[no_trace]
    navigation_start: CrossProcessInstant,
    performance: MutNullableDom<Performance>,
    trusted_types: MutNullableDom<TrustedTypePolicyFactory>,

    /// A [`TimerScheduler`] used to schedule timers for this [`WorkerGlobalScope`].
    /// Timers are handled in the service worker event loop.
    #[no_trace]
    timer_scheduler: RefCell<TimerScheduler>,

    #[no_trace]
    insecure_requests_policy: InsecureRequestsPolicy,
}

impl WorkerGlobalScope {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_inherited(
        init: WorkerGlobalScopeInit,
        worker_name: DOMString,
        worker_type: WorkerType,
        worker_url: ServoUrl,
        runtime: Runtime,
        devtools_receiver: Receiver<DevtoolScriptControlMsg>,
        closing: Arc<AtomicBool>,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
        insecure_requests_policy: InsecureRequestsPolicy,
    ) -> Self {
        // Install a pipeline-namespace in the current thread.
        PipelineNamespace::auto_install();

        let devtools_receiver = match init.from_devtools_sender {
            Some(..) => Some(devtools_receiver),
            None => None,
        };

        Self {
            globalscope: GlobalScope::new_inherited(
                init.pipeline_id,
                init.to_devtools_sender,
                init.mem_profiler_chan,
                init.time_profiler_chan,
                init.script_to_constellation_chan,
                init.resource_threads,
                MutableOrigin::new(init.origin),
                init.creation_url,
                runtime.microtask_queue.clone(),
                #[cfg(feature = "webgpu")]
                gpu_id_hub,
                init.inherited_secure_context,
                false,
            ),
            worker_id: init.worker_id,
            worker_name,
            worker_type,
            worker_url: DomRefCell::new(worker_url),
            closing,
            runtime: DomRefCell::new(Some(runtime)),
            location: Default::default(),
            navigator: Default::default(),
            policy_container: Default::default(),
            devtools_receiver,
            _devtools_sender: init.from_devtools_sender,
            navigation_start: CrossProcessInstant::now(),
            performance: Default::default(),
            timer_scheduler: RefCell::default(),
            insecure_requests_policy,
            trusted_types: Default::default(),
        }
    }

    /// Returns a policy value that should be used by fetches initiated by this worker.
    pub(crate) fn insecure_requests_policy(&self) -> InsecureRequestsPolicy {
        self.insecure_requests_policy
    }

    /// Clear various items when the worker event-loop shuts-down.
    pub(crate) fn clear_js_runtime(&self) {
        self.upcast::<GlobalScope>()
            .remove_web_messaging_and_dedicated_workers_infra();

        // Drop the runtime.
        let runtime = self.runtime.borrow_mut().take();
        drop(runtime);
    }

    pub(crate) fn runtime_handle(&self) -> ParentRuntime {
        self.runtime
            .borrow()
            .as_ref()
            .unwrap()
            .prepare_for_new_child()
    }

    pub(crate) fn devtools_receiver(&self) -> Option<&Receiver<DevtoolScriptControlMsg>> {
        self.devtools_receiver.as_ref()
    }

    #[allow(unsafe_code)]
    pub(crate) fn get_cx(&self) -> JSContext {
        unsafe { JSContext::from_ptr(self.runtime.borrow().as_ref().unwrap().cx()) }
    }

    pub(crate) fn is_closing(&self) -> bool {
        self.closing.load(Ordering::SeqCst)
    }

    pub(crate) fn get_url(&self) -> Ref<ServoUrl> {
        self.worker_url.borrow()
    }

    pub(crate) fn set_url(&self, url: ServoUrl) {
        *self.worker_url.borrow_mut() = url;
    }

    pub(crate) fn get_worker_id(&self) -> WorkerId {
        self.worker_id
    }

    pub(crate) fn pipeline_id(&self) -> PipelineId {
        self.globalscope.pipeline_id()
    }

    pub(crate) fn policy_container(&self) -> Ref<PolicyContainer> {
        self.policy_container.borrow()
    }

    /// Get a mutable reference to the [`TimerScheduler`] for this [`ServiceWorkerGlobalScope`].
    pub(crate) fn timer_scheduler(&self) -> RefMut<TimerScheduler> {
        self.timer_scheduler.borrow_mut()
    }

    /// Return a copy to the shared task canceller that is used to cancel all tasks
    /// when this worker is closing.
    pub(crate) fn shared_task_canceller(&self) -> TaskCanceller {
        TaskCanceller {
            cancelled: self.closing.clone(),
        }
    }
}

impl WorkerGlobalScopeMethods<crate::DomTypeHolder> for WorkerGlobalScope {
    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-self
    fn Self_(&self) -> DomRoot<WorkerGlobalScope> {
        DomRoot::from_ref(self)
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-location
    fn Location(&self) -> DomRoot<WorkerLocation> {
        self.location
            .or_init(|| WorkerLocation::new(self, self.worker_url.borrow().clone(), CanGc::note()))
    }

    // https://html.spec.whatwg.org/multipage/#handler-workerglobalscope-onerror
    error_event_handler!(error, GetOnerror, SetOnerror);

    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-importscripts
    fn ImportScripts(&self, url_strings: Vec<DOMString>, can_gc: CanGc) -> ErrorResult {
        let mut urls = Vec::with_capacity(url_strings.len());
        for url in url_strings {
            let url = self.worker_url.borrow().join(&url);
            match url {
                Ok(url) => urls.push(url),
                Err(_) => return Err(Error::Syntax),
            };
        }

        rooted!(in(self.runtime.borrow().as_ref().unwrap().cx()) let mut rval = UndefinedValue());
        for url in urls {
            let global_scope = self.upcast::<GlobalScope>();
            let request = NetRequestInit::new(
                global_scope.webview_id(),
                url.clone(),
                global_scope.get_referrer(),
            )
            .destination(Destination::Script)
            .credentials_mode(CredentialsMode::Include)
            .parser_metadata(ParserMetadata::NotParserInserted)
            .use_url_credentials(true)
            .origin(global_scope.origin().immutable().clone())
            .insecure_requests_policy(self.insecure_requests_policy())
            .has_trustworthy_ancestor_origin(
                global_scope.has_trustworthy_ancestor_or_current_origin(),
            )
            .pipeline_id(Some(self.upcast::<GlobalScope>().pipeline_id()));

            let (url, source) = match fetch::load_whole_resource(
                request,
                &global_scope.resource_threads().sender(),
                global_scope,
                can_gc,
            ) {
                Err(_) => return Err(Error::Network),
                Ok((metadata, bytes)) => (metadata.final_url, String::from_utf8(bytes).unwrap()),
            };

            let result = self.runtime.borrow().as_ref().unwrap().evaluate_script(
                self.reflector().get_jsobject(),
                &source,
                url.as_str(),
                1,
                rval.handle_mut(),
            );

            maybe_resume_unwind();

            match result {
                Ok(_) => (),
                Err(_) => {
                    if self.is_closing() {
                        // Don't return JSFailed as we might not have
                        // any pending exceptions.
                        println!("evaluate_script failed (terminated)");
                    } else {
                        println!("evaluate_script failed");
                        return Err(Error::JSFailed);
                    }
                },
            }
        }

        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-worker-navigator
    fn Navigator(&self) -> DomRoot<WorkerNavigator> {
        self.navigator
            .or_init(|| WorkerNavigator::new(self, CanGc::note()))
    }

    // https://html.spec.whatwg.org/multipage/#dfn-Crypto
    fn Crypto(&self) -> DomRoot<Crypto> {
        self.upcast::<GlobalScope>().crypto()
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowbase64-btoa
    fn Btoa(&self, btoa: DOMString) -> Fallible<DOMString> {
        base64_btoa(btoa)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowbase64-atob
    fn Atob(&self, atob: DOMString) -> Fallible<DOMString> {
        base64_atob(atob)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-settimeout
    fn SetTimeout(
        &self,
        _cx: JSContext,
        callback: StringOrFunction,
        timeout: i32,
        args: Vec<HandleValue>,
    ) -> i32 {
        let callback = match callback {
            StringOrFunction::String(i) => TimerCallback::StringTimerCallback(i),
            StringOrFunction::Function(i) => TimerCallback::FunctionTimerCallback(i),
        };
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            callback,
            args,
            Duration::from_millis(timeout.max(0) as u64),
            IsInterval::NonInterval,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-cleartimeout
    fn ClearTimeout(&self, handle: i32) {
        self.upcast::<GlobalScope>()
            .clear_timeout_or_interval(handle);
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    fn SetInterval(
        &self,
        _cx: JSContext,
        callback: StringOrFunction,
        timeout: i32,
        args: Vec<HandleValue>,
    ) -> i32 {
        let callback = match callback {
            StringOrFunction::String(i) => TimerCallback::StringTimerCallback(i),
            StringOrFunction::Function(i) => TimerCallback::FunctionTimerCallback(i),
        };
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            callback,
            args,
            Duration::from_millis(timeout.max(0) as u64),
            IsInterval::Interval,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-clearinterval
    fn ClearInterval(&self, handle: i32) {
        self.ClearTimeout(handle);
    }

    // https://html.spec.whatwg.org/multipage/#dom-queuemicrotask
    fn QueueMicrotask(&self, callback: Rc<VoidFunction>) {
        self.upcast::<GlobalScope>()
            .queue_function_as_microtask(callback);
    }

    // https://html.spec.whatwg.org/multipage/#dom-createimagebitmap
    fn CreateImageBitmap(
        &self,
        image: ImageBitmapSource,
        options: &ImageBitmapOptions,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let p = self
            .upcast::<GlobalScope>()
            .create_image_bitmap(image, options, can_gc);
        p
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    // https://fetch.spec.whatwg.org/#fetch-method
    fn Fetch(
        &self,
        input: RequestOrUSVString,
        init: RootedTraceableBox<RequestInit>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        fetch::Fetch(self.upcast(), input, init, comp, can_gc)
    }

    // https://w3c.github.io/hr-time/#the-performance-attribute
    fn Performance(&self) -> DomRoot<Performance> {
        self.performance.or_init(|| {
            let global_scope = self.upcast::<GlobalScope>();
            Performance::new(global_scope, self.navigation_start, CanGc::note())
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-origin
    fn Origin(&self) -> USVString {
        USVString(
            self.upcast::<GlobalScope>()
                .origin()
                .immutable()
                .ascii_serialization(),
        )
    }

    // https://w3c.github.io/webappsec-secure-contexts/#dom-windoworworkerglobalscope-issecurecontext
    fn IsSecureContext(&self) -> bool {
        self.upcast::<GlobalScope>().is_secure_context()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-structuredclone>
    fn StructuredClone(
        &self,
        cx: JSContext,
        value: HandleValue,
        options: RootedTraceableBox<StructuredSerializeOptions>,
        retval: MutableHandleValue,
    ) -> Fallible<()> {
        self.upcast::<GlobalScope>()
            .structured_clone(cx, value, options, retval)
    }

    /// <https://www.w3.org/TR/trusted-types/#dom-windoworworkerglobalscope-trustedtypes>
    fn TrustedTypes(&self, can_gc: CanGc) -> DomRoot<TrustedTypePolicyFactory> {
        self.trusted_types.or_init(|| {
            let global_scope = self.upcast::<GlobalScope>();
            TrustedTypePolicyFactory::new(global_scope, can_gc)
        })
    }
}

impl WorkerGlobalScope {
    #[allow(unsafe_code)]
    pub(crate) fn execute_script(&self, source: DOMString, can_gc: CanGc) {
        let _aes = AutoEntryScript::new(self.upcast());
        let cx = self.runtime.borrow().as_ref().unwrap().cx();
        rooted!(in(cx) let mut rval = UndefinedValue());
        match self.runtime.borrow().as_ref().unwrap().evaluate_script(
            self.reflector().get_jsobject(),
            &source,
            self.worker_url.borrow().as_str(),
            1,
            rval.handle_mut(),
        ) {
            Ok(_) => (),
            Err(_) => {
                if self.is_closing() {
                    println!("evaluate_script failed (terminated)");
                } else {
                    // TODO: An error needs to be dispatched to the parent.
                    // https://github.com/servo/servo/issues/6422
                    println!("evaluate_script failed");
                    unsafe {
                        let ar = enter_realm(self);
                        report_pending_exception(
                            JSContext::from_ptr(cx),
                            true,
                            InRealm::Entered(&ar),
                            can_gc,
                        );
                    }
                }
            },
        }
    }

    pub(crate) fn new_script_pair(&self) -> (ScriptEventLoopSender, ScriptEventLoopReceiver) {
        let dedicated = self.downcast::<DedicatedWorkerGlobalScope>();
        if let Some(dedicated) = dedicated {
            dedicated.new_script_pair()
        } else {
            panic!("need to implement a sender for SharedWorker/ServiceWorker")
        }
    }

    /// Process a single event as if it were the next event
    /// in the queue for this worker event-loop.
    /// Returns a boolean indicating whether further events should be processed.
    #[allow(unsafe_code)]
    pub(crate) fn process_event(&self, msg: CommonScriptMsg) -> bool {
        if self.is_closing() {
            return false;
        }
        match msg {
            CommonScriptMsg::Task(_, task, _, _) => task.run_box(),
            CommonScriptMsg::CollectReports(reports_chan) => {
                let cx = self.get_cx();
                let reports = cx.get_reports(format!("url({})", self.get_url()));
                reports_chan.send(ProcessReports::new(reports));
            },
        }
        true
    }

    pub(crate) fn close(&self) {
        self.closing.store(true, Ordering::SeqCst);
        self.upcast::<GlobalScope>()
            .task_manager()
            .cancel_all_tasks_and_ignore_future_tasks();
    }
}
