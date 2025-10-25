/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::{RefCell, RefMut};
use std::default::Default;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use base::IpcSend;
use base::cross_process_instant::CrossProcessInstant;
use base::id::{PipelineId, PipelineNamespace};
use constellation_traits::WorkerGlobalScopeInit;
use content_security_policy::CspList;
use crossbeam_channel::Receiver;
use devtools_traits::{DevtoolScriptControlMsg, WorkerId};
use dom_struct::dom_struct;
use encoding_rs::UTF_8;
use fonts::FontContext;
use headers::{HeaderMapExt, ReferrerPolicy as ReferrerPolicyHeader};
use ipc_channel::ipc::IpcSender;
use js::jsapi::JS_AddInterruptCallback;
use js::jsval::UndefinedValue;
use js::panic::maybe_resume_unwind;
use js::rust::{HandleValue, MutableHandleValue, ParentRuntime};
use mime::Mime;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{
    CredentialsMode, Destination, InsecureRequestsPolicy, ParserMetadata,
    RequestBuilder as NetRequestInit, RequestId,
};
use net_traits::{
    FetchMetadata, FetchResponseListener, Metadata, NetworkError, ReferrerPolicy,
    ResourceFetchTiming, ResourceTimingType,
};
use profile_traits::mem::{ProcessReports, perform_memory_report};
use servo_url::{MutableOrigin, ServoUrl};
use timers::TimerScheduler;
use uuid::Uuid;

use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding::{
    ImageBitmapOptions, ImageBitmapSource,
};
use crate::dom::bindings::codegen::Bindings::MessagePortBinding::StructuredSerializeOptions;
use crate::dom::bindings::codegen::Bindings::ReportingObserverBinding::Report;
use crate::dom::bindings::codegen::Bindings::RequestBinding::RequestInit;
use crate::dom::bindings::codegen::Bindings::VoidFunctionBinding::VoidFunction;
use crate::dom::bindings::codegen::Bindings::WorkerBinding::WorkerType;
use crate::dom::bindings::codegen::Bindings::WorkerGlobalScopeBinding::WorkerGlobalScopeMethods;
use crate::dom::bindings::codegen::UnionTypes::{
    RequestOrUSVString, TrustedScriptOrString, TrustedScriptOrStringOrFunction,
    TrustedScriptURLOrUSVString,
};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::utils::define_all_exposed_interfaces;
use crate::dom::crypto::Crypto;
use crate::dom::csp::{GlobalCspReporting, Violation, parse_csp_list_from_metadata};
use crate::dom::dedicatedworkerglobalscope::{
    AutoWorkerReset, DedicatedWorkerGlobalScope, interrupt_callback,
};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlscriptelement::{SCRIPT_JS_MIMES, ScriptOrigin, ScriptType};
use crate::dom::indexeddb::idbfactory::IDBFactory;
use crate::dom::performance::Performance;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::reportingendpoint::{ReportingEndpoint, SendReportsToEndpoints};
use crate::dom::reportingobserver::ReportingObserver;
use crate::dom::trustedscripturl::TrustedScriptURL;
use crate::dom::trustedtypepolicyfactory::TrustedTypePolicyFactory;
use crate::dom::types::ImageBitmap;
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::dom::window::{base64_atob, base64_btoa};
use crate::dom::worker::TrustedWorkerAddress;
use crate::dom::workerlocation::WorkerLocation;
use crate::dom::workernavigator::WorkerNavigator;
use crate::fetch::{CspViolationsProcessor, Fetch, load_whole_resource};
use crate::messaging::{CommonScriptMsg, ScriptEventLoopReceiver, ScriptEventLoopSender};
use crate::network_listener::{PreInvoke, ResourceTimingListener, submit_timing};
use crate::realms::{InRealm, enter_realm};
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::{CanGc, IntroductionType, JSContext, JSContextHelper, Runtime};
use crate::task::TaskCanceller;
use crate::timers::{IsInterval, TimerCallback};
use crate::unminify::unminify_js;

pub(crate) fn prepare_workerscope_init(
    global: &GlobalScope,
    devtools_sender: Option<IpcSender<DevtoolScriptControlMsg>>,
    worker_id: Option<WorkerId>,
) -> WorkerGlobalScopeInit {
    WorkerGlobalScopeInit {
        resource_threads: global.resource_threads().clone(),
        storage_threads: global.storage_threads().clone(),
        mem_profiler_chan: global.mem_profiler_chan().clone(),
        to_devtools_sender: global.devtools_chan().cloned(),
        time_profiler_chan: global.time_profiler_chan().clone(),
        from_devtools_sender: devtools_sender,
        script_to_constellation_chan: global.script_to_constellation_chan().clone(),
        script_to_embedder_chan: global.script_to_embedder_chan().clone(),
        worker_id: worker_id.unwrap_or_else(|| WorkerId(Uuid::new_v4())),
        pipeline_id: global.pipeline_id(),
        origin: global.origin().immutable().clone(),
        creation_url: global.creation_url().clone(),
        inherited_secure_context: Some(global.is_secure_context()),
        unminify_js: global.unminify_js(),
    }
}

pub(crate) struct ScriptFetchContext {
    scope: Trusted<WorkerGlobalScope>,
    resource_timing: ResourceFetchTiming,
    response: Option<Metadata>,
    body_bytes: Vec<u8>,
    url: ServoUrl,
    worker: TrustedWorkerAddress,
    policy_container: PolicyContainer,
}

impl ScriptFetchContext {
    pub(crate) fn new(
        scope: Trusted<WorkerGlobalScope>,
        url: ServoUrl,
        worker: TrustedWorkerAddress,
        policy_container: PolicyContainer,
    ) -> ScriptFetchContext {
        ScriptFetchContext {
            scope,
            response: None,
            body_bytes: Vec::new(),
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
            url,
            worker,
            policy_container,
        }
    }
}

impl FetchResponseListener for ScriptFetchContext {
    fn process_request_body(&mut self, _request_id: RequestId) {}

    fn process_request_eof(&mut self, _request_id: RequestId) {}

    fn process_response(
        &mut self,
        _request_id: RequestId,
        metadata: Result<FetchMetadata, NetworkError>,
    ) {
        self.response = metadata.ok().map(|m| match m {
            FetchMetadata::Unfiltered(m) => m,
            FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
        });
    }

    fn process_response_chunk(&mut self, _request_id: RequestId, mut chunk: Vec<u8>) {
        self.body_bytes.append(&mut chunk);
    }

    fn process_response_eof(
        &mut self,
        _request_id: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        let scope = self.scope.root();

        if response
            .inspect_err(|e| error!("error loading script {} ({:?})", self.url, e))
            .is_err() ||
            self.response.is_none()
        {
            scope.on_complete(None, self.worker.clone(), CanGc::note());
            return;
        }
        let metadata = self.response.take().unwrap();

        // The processResponseConsumeBody steps defined inside
        // [run a worker](https://html.spec.whatwg.org/multipage/#run-a-worker)

        // Step 1 Set worker global scope's url to response's url.
        scope.set_url(metadata.final_url.clone());

        // Step 2 Initialize worker global scope's policy container given worker global scope, response, and inside settings.
        scope
            .initialize_policy_container_for_worker_global_scope(&metadata, &self.policy_container);
        scope.set_endpoints_list(ReportingEndpoint::parse_reporting_endpoints_header(
            &metadata.final_url.clone(),
            &metadata.headers,
        ));
        scope
            .upcast::<GlobalScope>()
            .set_https_state(metadata.https_state);

        // The processResponseConsumeBody steps defined inside
        // [fetch a classic worker script](https://html.spec.whatwg.org/multipage/#fetch-a-classic-worker-script)

        // Step 1 Set response to response's unsafe response. Done in process_response

        // Step 2 If any of the following are true: bodyBytes is null or failure; or response's status is not an ok status,
        if !metadata.status.is_success() {
            // then run onComplete given null, and abort these steps.
            scope.on_complete(None, self.worker.clone(), CanGc::note());
            return;
        }

        // Step 3 If all of the following are true:
        // response's URL's scheme is an HTTP(S) scheme;
        let is_http_scheme = matches!(metadata.final_url.scheme(), "http" | "https");
        // and the result of extracting a MIME type from response's header list is not a JavaScript MIME type,
        let not_a_javascript_mime_type = !metadata.content_type.clone().is_some_and(|ct| {
            let mime: Mime = ct.into_inner().into();
            SCRIPT_JS_MIMES.contains(&mime.essence_str())
        });

        if is_http_scheme && not_a_javascript_mime_type {
            // then run onComplete given null, and abort these steps.
            scope.on_complete(None, self.worker.clone(), CanGc::note());
            return;
        }

        // Step 4 Let sourceText be the result of UTF-8 decoding bodyBytes.
        let (source, _, _) = UTF_8.decode(&self.body_bytes);

        // Step 5 Let script be the result of creating a classic script using
        // sourceText, settingsObject, response's URL, and the default script fetch options.

        // Step 6 Run onComplete given script.
        scope.on_complete(Some(source), self.worker.clone(), CanGc::note());
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        submit_timing(self, CanGc::note());
    }

    fn process_csp_violations(
        &mut self,
        _request_id: RequestId,
        violations: Vec<content_security_policy::Violation>,
    ) {
        let scope = self.scope.root();

        if let Some(worker_scope) = scope.downcast::<DedicatedWorkerGlobalScope>() {
            worker_scope.report_csp_violations(violations);
        }
    }
}

impl ResourceTimingListener for ScriptFetchContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (InitiatorType::Other, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.scope.root().global()
    }
}

impl PreInvoke for ScriptFetchContext {}

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
    #[conditional_malloc_size_of]
    closing: Arc<AtomicBool>,
    execution_ready: AtomicBool,
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
    indexeddb: MutNullableDom<IDBFactory>,
    trusted_types: MutNullableDom<TrustedTypePolicyFactory>,

    /// A [`TimerScheduler`] used to schedule timers for this [`WorkerGlobalScope`].
    /// Timers are handled in the service worker event loop.
    #[no_trace]
    timer_scheduler: RefCell<TimerScheduler>,

    #[no_trace]
    insecure_requests_policy: InsecureRequestsPolicy,

    /// <https://w3c.github.io/reporting/#windoworworkerglobalscope-registered-reporting-observer-list>
    reporting_observer_list: DomRefCell<Vec<DomRoot<ReportingObserver>>>,

    /// <https://w3c.github.io/reporting/#windoworworkerglobalscope-reports>
    report_list: DomRefCell<Vec<Report>>,

    /// <https://w3c.github.io/reporting/#windoworworkerglobalscope-endpoints>
    #[no_trace]
    endpoints_list: DomRefCell<Vec<ReportingEndpoint>>,
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
        font_context: Option<Arc<FontContext>>,
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
                init.script_to_embedder_chan,
                init.resource_threads,
                init.storage_threads,
                MutableOrigin::new(init.origin),
                init.creation_url,
                None,
                runtime.microtask_queue.clone(),
                #[cfg(feature = "webgpu")]
                gpu_id_hub,
                init.inherited_secure_context,
                init.unminify_js,
                font_context,
            ),
            worker_id: init.worker_id,
            worker_name,
            worker_type,
            worker_url: DomRefCell::new(worker_url),
            closing,
            execution_ready: AtomicBool::new(false),
            runtime: DomRefCell::new(Some(runtime)),
            location: Default::default(),
            navigator: Default::default(),
            policy_container: Default::default(),
            devtools_receiver,
            _devtools_sender: init.from_devtools_sender,
            navigation_start: CrossProcessInstant::now(),
            performance: Default::default(),
            indexeddb: Default::default(),
            timer_scheduler: RefCell::default(),
            insecure_requests_policy,
            trusted_types: Default::default(),
            reporting_observer_list: Default::default(),
            report_list: Default::default(),
            endpoints_list: Default::default(),
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

    pub(crate) fn is_execution_ready(&self) -> bool {
        self.execution_ready.load(Ordering::Relaxed)
    }

    pub(crate) fn get_url(&self) -> Ref<'_, ServoUrl> {
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

    pub(crate) fn policy_container(&self) -> Ref<'_, PolicyContainer> {
        self.policy_container.borrow()
    }

    pub(crate) fn set_csp_list(&self, csp_list: Option<CspList>) {
        self.policy_container.borrow_mut().set_csp_list(csp_list);
    }

    pub(crate) fn set_referrer_policy(&self, referrer_policy: ReferrerPolicy) {
        self.policy_container
            .borrow_mut()
            .set_referrer_policy(referrer_policy);
    }

    pub(crate) fn append_reporting_observer(&self, reporting_observer: DomRoot<ReportingObserver>) {
        self.reporting_observer_list
            .borrow_mut()
            .push(reporting_observer);
    }

    pub(crate) fn remove_reporting_observer(&self, reporting_observer: &ReportingObserver) {
        if let Some(index) = self
            .reporting_observer_list
            .borrow()
            .iter()
            .position(|observer| &**observer == reporting_observer)
        {
            self.reporting_observer_list.borrow_mut().remove(index);
        }
    }

    pub(crate) fn registered_reporting_observers(&self) -> Vec<DomRoot<ReportingObserver>> {
        self.reporting_observer_list.borrow().clone()
    }

    pub(crate) fn append_report(&self, report: Report) {
        self.report_list.borrow_mut().push(report);
        let trusted_worker = Trusted::new(self);
        self.upcast::<GlobalScope>()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(send_to_reporting_endpoints: move || {
                let worker = trusted_worker.root();
                let reports = std::mem::take(&mut *worker.report_list.borrow_mut());
                worker.upcast::<GlobalScope>().send_reports_to_endpoints(
                    reports,
                    worker.endpoints_list.borrow().clone(),
                );
            }));
    }

    pub(crate) fn buffered_reports(&self) -> Vec<Report> {
        self.report_list.borrow().clone()
    }

    pub(crate) fn set_endpoints_list(&self, endpoints: Option<Vec<ReportingEndpoint>>) {
        if let Some(endpoints) = endpoints {
            *self.endpoints_list.borrow_mut() = endpoints;
        }
    }

    /// Get a mutable reference to the [`TimerScheduler`] for this [`ServiceWorkerGlobalScope`].
    pub(crate) fn timer_scheduler(&self) -> RefMut<'_, TimerScheduler> {
        self.timer_scheduler.borrow_mut()
    }

    /// Return a copy to the shared task canceller that is used to cancel all tasks
    /// when this worker is closing.
    pub(crate) fn shared_task_canceller(&self) -> TaskCanceller {
        TaskCanceller {
            cancelled: self.closing.clone(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#initialize-worker-policy-container> and
    /// <https://html.spec.whatwg.org/multipage/#creating-a-policy-container-from-a-fetch-response>
    fn initialize_policy_container_for_worker_global_scope(
        &self,
        metadata: &Metadata,
        parent_policy_container: &PolicyContainer,
    ) {
        // Step 1. If workerGlobalScope's url is local but its scheme is not "blob":
        //
        // Note that we also allow for blob here, as the parent_policy_container is in both cases
        // the container that we need to clone.
        if metadata.final_url.is_local_scheme() {
            // Step 1.2. Set workerGlobalScope's policy container to a clone of workerGlobalScope's
            // owner set[0]'s relevant settings object's policy container.
            //
            // Step 1. If response's URL's scheme is "blob", then return a clone of response's URL's
            // blob URL entry's environment's policy container.
            self.set_csp_list(parent_policy_container.csp_list.clone());
            self.set_referrer_policy(parent_policy_container.get_referrer_policy());
            return;
        }
        // Step 3. Set result's CSP list to the result of parsing a response's Content Security Policies given response.
        self.set_csp_list(parse_csp_list_from_metadata(&metadata.headers));
        // Step 5. Set result's referrer policy to the result of parsing the `Referrer-Policy`
        // header given response. [REFERRERPOLICY]
        let referrer_policy = metadata
            .headers
            .as_ref()
            .and_then(|headers| headers.typed_get::<ReferrerPolicyHeader>())
            .into();
        self.set_referrer_policy(referrer_policy);
    }

    /// onComplete algorithm defined inside <https://html.spec.whatwg.org/multipage/#run-a-worker>
    #[allow(unsafe_code)]
    fn on_complete(
        &self,
        script: Option<Cow<'_, str>>,
        worker: TrustedWorkerAddress,
        can_gc: CanGc,
    ) {
        let dedicated_worker_scope = self
            .downcast::<DedicatedWorkerGlobalScope>()
            .expect("Only DedicatedWorkerGlobalScope is supported for now");

        let Some(script) = script else {
            // Step 1 Queue a global task on the DOM manipulation task source given
            // worker's relevant global object to fire an event named error at worker.
            dedicated_worker_scope.forward_simple_error_at_worker(worker.clone());

            // Step 2 TODO Run the environment discarding steps for inside settings.
            return;
        };

        unsafe {
            // Handle interrupt requests
            JS_AddInterruptCallback(*self.get_cx(), Some(interrupt_callback));
        }

        if self.is_closing() {
            return;
        }

        {
            let _ar = AutoWorkerReset::new(dedicated_worker_scope, worker);
            let realm = enter_realm(self);
            define_all_exposed_interfaces(
                dedicated_worker_scope.upcast(),
                InRealm::entered(&realm),
                can_gc,
            );
            self.execution_ready.store(true, Ordering::Relaxed);
            self.execute_script(DOMString::from(script), can_gc);
            dedicated_worker_scope.fire_queued_messages(can_gc);
        }
    }
}

impl WorkerGlobalScopeMethods<crate::DomTypeHolder> for WorkerGlobalScope {
    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-self
    fn Self_(&self) -> DomRoot<WorkerGlobalScope> {
        DomRoot::from_ref(self)
    }

    // https://w3c.github.io/IndexedDB/#factory-interface
    fn IndexedDB(&self) -> DomRoot<IDBFactory> {
        self.indexeddb.or_init(|| {
            let global_scope = self.upcast::<GlobalScope>();
            IDBFactory::new(global_scope, CanGc::note())
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-location
    fn Location(&self) -> DomRoot<WorkerLocation> {
        self.location
            .or_init(|| WorkerLocation::new(self, self.worker_url.borrow().clone(), CanGc::note()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-importscripts
    fn ImportScripts(
        &self,
        url_strings: Vec<TrustedScriptURLOrUSVString>,
        can_gc: CanGc,
    ) -> ErrorResult {
        // https://html.spec.whatwg.org/multipage/#import-scripts-into-worker-global-scope
        // Step 1: If worker global scope's type is "module", throw a TypeError exception.
        if self.worker_type == WorkerType::Module {
            return Err(Error::Type(
                "importScripts() is not allowed in module workers".to_string(),
            ));
        }

        // Step 4: Let urlStrings be « ».
        let mut urls = Vec::with_capacity(url_strings.len());
        // Step 5: For each url of urls:
        for url in url_strings {
            // Step 3: Append the result of invoking the Get Trusted Type compliant string algorithm
            // with TrustedScriptURL, this's relevant global object, url, "WorkerGlobalScope importScripts",
            // and "script" to urlStrings.
            let url = TrustedScriptURL::get_trusted_script_url_compliant_string(
                self.upcast::<GlobalScope>(),
                url,
                "WorkerGlobalScope",
                "importScripts",
                can_gc,
            )?;
            let url = self.worker_url.borrow().join(&url.str());
            match url {
                Ok(url) => urls.push(url),
                Err(_) => return Err(Error::Syntax(None)),
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
            .policy_container(global_scope.policy_container())
            .has_trustworthy_ancestor_origin(
                global_scope.has_trustworthy_ancestor_or_current_origin(),
            )
            .pipeline_id(Some(self.upcast::<GlobalScope>().pipeline_id()));

            let (url, source) = match load_whole_resource(
                request,
                &global_scope.resource_threads().sender(),
                global_scope,
                &WorkerCspProcessor {
                    global_scope: DomRoot::from_ref(global_scope),
                },
                can_gc,
            ) {
                Err(_) => return Err(Error::Network),
                Ok((metadata, bytes)) => {
                    // https://html.spec.whatwg.org/multipage/#fetch-a-classic-worker-imported-script
                    // Step 7: Check if response status is not an ok status
                    if !metadata.status.is_success() {
                        return Err(Error::Network);
                    }

                    // Step 7: Check if the MIME type is not a JavaScript MIME type
                    let not_a_javascript_mime_type =
                        !metadata.content_type.clone().is_some_and(|ct| {
                            let mime: Mime = ct.into_inner().into();
                            SCRIPT_JS_MIMES.contains(&mime.essence_str())
                        });
                    if not_a_javascript_mime_type {
                        return Err(Error::Network);
                    }

                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                },
            };

            let options = self
                .runtime
                .borrow()
                .as_ref()
                .unwrap()
                .new_compile_options(url.as_str(), 1);
            let result = self.runtime.borrow().as_ref().unwrap().evaluate_script(
                self.reflector().get_jsobject(),
                &source,
                rval.handle_mut(),
                options,
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

    // https://html.spec.whatwg.org/multipage/#handler-workerglobalscope-onerror
    error_event_handler!(error, GetOnerror, SetOnerror);

    // https://html.spec.whatwg.org/multipage/#handler-workerglobalscope-onlanguagechange
    event_handler!(languagechange, GetOnlanguagechange, SetOnlanguagechange);

    // https://html.spec.whatwg.org/multipage/#handler-workerglobalscope-onoffline
    event_handler!(offline, GetOnoffline, SetOnoffline);

    // https://html.spec.whatwg.org/multipage/#handler-workerglobalscope-ononline
    event_handler!(online, GetOnonline, SetOnonline);

    // https://html.spec.whatwg.org/multipage/#handler-workerglobalscope-onrejectionhandled
    event_handler!(
        rejectionhandled,
        GetOnrejectionhandled,
        SetOnrejectionhandled
    );

    // https://html.spec.whatwg.org/multipage/#handler-workerglobalscope-onunhandledrejection
    event_handler!(
        unhandledrejection,
        GetOnunhandledrejection,
        SetOnunhandledrejection
    );

    // https://html.spec.whatwg.org/multipage/#dom-worker-navigator
    fn Navigator(&self) -> DomRoot<WorkerNavigator> {
        self.navigator
            .or_init(|| WorkerNavigator::new(self, CanGc::note()))
    }

    // https://html.spec.whatwg.org/multipage/#dfn-Crypto
    fn Crypto(&self) -> DomRoot<Crypto> {
        self.upcast::<GlobalScope>().crypto(CanGc::note())
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
        callback: TrustedScriptOrStringOrFunction,
        timeout: i32,
        args: Vec<HandleValue>,
        can_gc: CanGc,
    ) -> Fallible<i32> {
        let callback = match callback {
            TrustedScriptOrStringOrFunction::String(i) => {
                TimerCallback::StringTimerCallback(TrustedScriptOrString::String(i))
            },
            TrustedScriptOrStringOrFunction::TrustedScript(i) => {
                TimerCallback::StringTimerCallback(TrustedScriptOrString::TrustedScript(i))
            },
            TrustedScriptOrStringOrFunction::Function(i) => TimerCallback::FunctionTimerCallback(i),
        };
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            callback,
            args,
            Duration::from_millis(timeout.max(0) as u64),
            IsInterval::NonInterval,
            can_gc,
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
        callback: TrustedScriptOrStringOrFunction,
        timeout: i32,
        args: Vec<HandleValue>,
        can_gc: CanGc,
    ) -> Fallible<i32> {
        let callback = match callback {
            TrustedScriptOrStringOrFunction::String(i) => {
                TimerCallback::StringTimerCallback(TrustedScriptOrString::String(i))
            },
            TrustedScriptOrStringOrFunction::TrustedScript(i) => {
                TimerCallback::StringTimerCallback(TrustedScriptOrString::TrustedScript(i))
            },
            TrustedScriptOrStringOrFunction::Function(i) => TimerCallback::FunctionTimerCallback(i),
        };
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            callback,
            args,
            Duration::from_millis(timeout.max(0) as u64),
            IsInterval::Interval,
            can_gc,
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

    /// <https://html.spec.whatwg.org/multipage/#dom-createimagebitmap>
    fn CreateImageBitmap(
        &self,
        image: ImageBitmapSource,
        options: &ImageBitmapOptions,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        ImageBitmap::create_image_bitmap(self.upcast(), image, 0, 0, None, None, options, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-createimagebitmap>
    fn CreateImageBitmap_(
        &self,
        image: ImageBitmapSource,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        options: &ImageBitmapOptions,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        ImageBitmap::create_image_bitmap(
            self.upcast(),
            image,
            sx,
            sy,
            Some(sw),
            Some(sh),
            options,
            can_gc,
        )
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    /// <https://fetch.spec.whatwg.org/#dom-global-fetch>
    fn Fetch(
        &self,
        input: RequestOrUSVString,
        init: RootedTraceableBox<RequestInit>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        Fetch(self.upcast(), input, init, comp, can_gc)
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
        can_gc: CanGc,
        retval: MutableHandleValue,
    ) -> Fallible<()> {
        self.upcast::<GlobalScope>()
            .structured_clone(cx, value, options, retval, can_gc)
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
    pub(crate) fn execute_script(&self, source: DOMString, can_gc: CanGc) {
        let global = self.upcast::<GlobalScope>();
        let mut script = ScriptOrigin::external(
            Rc::new(source),
            self.worker_url.borrow().clone(),
            ScriptFetchOptions::default_classic_script(global),
            ScriptType::Classic,
            global.unminified_js_dir(),
        );
        unminify_js(&mut script);

        global.run_a_classic_script(&script, 1, Some(IntroductionType::WORKER), can_gc);
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
                perform_memory_report(|ops| {
                    let reports = cx.get_reports(format!("url({})", self.get_url()), ops);
                    reports_chan.send(ProcessReports::new(reports));
                });
            },
            CommonScriptMsg::ReportCspViolations(_, violations) => {
                self.upcast::<GlobalScope>()
                    .report_csp_violations(violations, None, None);
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

struct WorkerCspProcessor {
    global_scope: DomRoot<GlobalScope>,
}

impl CspViolationsProcessor for WorkerCspProcessor {
    fn process_csp_violations(&self, violations: Vec<Violation>) {
        self.global_scope
            .report_csp_violations(violations, None, None);
    }
}
