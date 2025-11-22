/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use base::id::WebViewId;
use ipc_channel::ipc;
use js::jsapi::{ExceptionStackBehavior, JS_IsExceptionPending};
use js::jsval::UndefinedValue;
use js::rust::HandleValue;
use js::rust::wrappers::JS_SetPendingException;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{
    CorsSettings, CredentialsMode, Destination, InsecureRequestsPolicy, Referrer,
    Request as NetTraitsRequest, RequestBuilder, RequestId, RequestMode, ServiceWorkersMode,
};
use net_traits::{
    CoreResourceMsg, CoreResourceThread, FetchChannels, FetchMetadata, FetchResponseMsg,
    FilteredMetadata, Metadata, NetworkError, ResourceFetchTiming, ResourceTimingType,
    cancel_async_fetch,
};
use parking_lot::Mutex;
use servo_url::ServoUrl;
use timers::TimerEventRequest;

use crate::body::BodyMixin;
use crate::dom::abortsignal::AbortAlgorithm;
use crate::dom::bindings::codegen::Bindings::AbortSignalBinding::AbortSignalMethods;
use crate::dom::bindings::codegen::Bindings::RequestBinding::{
    RequestInfo, RequestInit, RequestMethods,
};
use crate::dom::bindings::codegen::Bindings::ResponseBinding::Response_Binding::ResponseMethods;
use crate::dom::bindings::codegen::Bindings::ResponseBinding::ResponseType as DOMResponseType;
use crate::dom::bindings::codegen::Bindings::WindowBinding::{DeferredRequestInit, WindowMethods};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::import::module::SafeJSContext;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::fetchlaterresult::FetchLaterResult;
use crate::dom::globalscope::GlobalScope;
use crate::dom::headers::Guard;
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::request::Request;
use crate::dom::response::Response;
use crate::dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use crate::dom::window::Window;
use crate::network_listener::{
    self, FetchResponseListener, NetworkListener, ResourceTimingListener, submit_timing_data,
};
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::CanGc;

/// RAII fetch canceller object.
/// By default initialized to having a
/// request associated with it, which can be manually cancelled with `cancel`,
/// or automatically cancelled on drop.
/// Calling `ignore` will sever the relationship with the request,
/// meaning it cannot be cancelled through this canceller from that point on.
#[derive(Default, JSTraceable, MallocSizeOf)]
pub(crate) struct FetchCanceller {
    #[no_trace]
    request_id: Option<RequestId>,
    #[no_trace]
    core_resource_thread: Option<CoreResourceThread>,
}

impl FetchCanceller {
    /// Create a FetchCanceller associated with a request,
    // and a particular(public vs private) resource thread.
    pub(crate) fn new(request_id: RequestId, core_resource_thread: CoreResourceThread) -> Self {
        Self {
            request_id: Some(request_id),
            core_resource_thread: Some(core_resource_thread),
        }
    }

    /// Cancel a fetch if it is ongoing
    pub(crate) fn cancel(&mut self) {
        if let Some(request_id) = self.request_id.take() {
            // stop trying to make fetch happen
            // it's not going to happen

            if let Some(ref core_resource_thread) = self.core_resource_thread {
                // No error handling here. Cancellation is a courtesy call,
                // we don't actually care if the other side heard.
                cancel_async_fetch(vec![request_id], core_resource_thread);
            }
        }
    }

    /// Use this if you don't want it to send a cancellation request
    /// on drop (e.g. if the fetch completes)
    pub(crate) fn ignore(&mut self) {
        let _ = self.request_id.take();
    }
}

impl Drop for FetchCanceller {
    fn drop(&mut self) {
        self.cancel()
    }
}

fn request_init_from_request(request: NetTraitsRequest, global: &GlobalScope) -> RequestBuilder {
    let mut builder =
        RequestBuilder::new(request.target_webview_id, request.url(), request.referrer)
            .method(request.method)
            .headers(request.headers)
            .unsafe_request(request.unsafe_request)
            .body(request.body)
            .destination(request.destination)
            .synchronous(request.synchronous)
            .mode(request.mode)
            .cache_mode(request.cache_mode)
            .use_cors_preflight(request.use_cors_preflight)
            .credentials_mode(request.credentials_mode)
            .use_url_credentials(request.use_url_credentials)
            .referrer_policy(request.referrer_policy)
            .pipeline_id(request.pipeline_id)
            .redirect_mode(request.redirect_mode)
            .integrity_metadata(request.integrity_metadata)
            .cryptographic_nonce_metadata(request.cryptographic_nonce_metadata)
            .parser_metadata(request.parser_metadata)
            .initiator(request.initiator)
            .client(global.request_client())
            .insecure_requests_policy(request.insecure_requests_policy)
            .has_trustworthy_ancestor_origin(request.has_trustworthy_ancestor_origin)
            .https_state(request.https_state)
            .response_tainting(request.response_tainting);
    builder.id = request.id;
    builder
}

/// <https://fetch.spec.whatwg.org/#abort-fetch>
fn abort_fetch_call(
    promise: Rc<Promise>,
    request: &Request,
    response_object: Option<&Response>,
    abort_reason: HandleValue,
    global: &GlobalScope,
    cx: SafeJSContext,
    can_gc: CanGc,
) {
    // Step 1. Reject promise with error.
    promise.reject(cx, abort_reason, can_gc);
    // Step 2. If request’s body is non-null and is readable, then cancel request’s body with error.
    if let Some(body) = request.body() {
        if body.is_readable() {
            body.cancel(cx, global, abort_reason, can_gc);
        }
    }
    // Step 3. If responseObject is null, then return.
    // Step 4. Let response be responseObject’s response.
    let Some(response) = response_object else {
        return;
    };
    // Step 5. If response’s body is non-null and is readable, then error response’s body with error.
    if let Some(body) = response.body() {
        if body.is_readable() {
            body.error(abort_reason, can_gc);
        }
    }
}

/// <https://fetch.spec.whatwg.org/#dom-global-fetch>
#[allow(non_snake_case)]
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
pub(crate) fn Fetch(
    global: &GlobalScope,
    input: RequestInfo,
    init: RootedTraceableBox<RequestInit>,
    comp: InRealm,
    can_gc: CanGc,
) -> Rc<Promise> {
    // Step 1. Let p be a new promise.
    let promise = Promise::new_in_current_realm(comp, can_gc);
    let cx = GlobalScope::get_cx();

    // Step 7. Let responseObject be null.
    // NOTE: We do initialize the object earlier earlier so we can use it to track errors
    let response = Response::new(global, can_gc);
    response.Headers(can_gc).set_guard(Guard::Immutable);

    // Step 2. Let requestObject be the result of invoking the initial value of Request as constructor
    //         with input and init as arguments. If this throws an exception, reject p with it and return p.
    let request_object = match Request::Constructor(global, None, can_gc, input, init) {
        Err(e) => {
            response.error_stream(e.clone(), can_gc);
            promise.reject_error(e, can_gc);
            return promise;
        },
        Ok(r) => r,
    };
    // Step 3. Let request be requestObject’s request.
    let request = request_object.get_request();
    let request_id = request.id;

    // Step 4. If requestObject’s signal is aborted, then:
    let signal = request_object.Signal();
    if signal.aborted() {
        // Step 4.1. Abort the fetch() call with p, request, null, and requestObject’s signal’s abort reason.
        rooted!(in(*cx) let mut abort_reason = UndefinedValue());
        signal.Reason(cx, abort_reason.handle_mut());
        abort_fetch_call(
            promise.clone(),
            &request_object,
            None,
            abort_reason.handle(),
            global,
            cx,
            can_gc,
        );
        // Step 4.2. Return p.
        return promise;
    }

    // Step 5. Let globalObject be request’s client’s global object.
    // NOTE:   We already get the global object as an argument
    let mut request_init = request_init_from_request(request, global);

    // Step 6. If globalObject is a ServiceWorkerGlobalScope object, then set request’s
    //         service-workers mode to "none".
    if global.is::<ServiceWorkerGlobalScope>() {
        request_init.service_workers_mode = ServiceWorkersMode::None;
    }

    // Step 8. Let relevantRealm be this’s relevant realm.
    //
    // Is `comp` as argument

    // Step 9. Let locallyAborted be false.
    // Step 10. Let controller be null.
    let fetch_context = FetchContext {
        fetch_promise: Some(TrustedPromise::new(promise.clone())),
        response_object: Trusted::new(&*response),
        request: Trusted::new(&*request_object),
        global: Trusted::new(global),
        locally_aborted: false,
        canceller: FetchCanceller::new(request_id, global.core_resource_thread()),
    };
    let network_listener = NetworkListener::new(
        fetch_context,
        global.task_manager().networking_task_source().to_sendable(),
    );
    let fetch_context = network_listener.context.clone();

    // Step 11. Add the following abort steps to requestObject’s signal:
    signal.add(&AbortAlgorithm::Fetch(fetch_context));

    // Step 12. Set controller to the result of calling fetch given request and
    // processResponse given response being these steps:
    global.fetch_with_network_listener(request_init, network_listener);

    // Step 13. Return p.
    promise
}

/// <https://fetch.spec.whatwg.org/#queue-a-deferred-fetch>
fn queue_deferred_fetch(
    request: NetTraitsRequest,
    activate_after: Finite<f64>,
    global: &GlobalScope,
) -> Arc<Mutex<DeferredFetchRecord>> {
    let trusted_global = Trusted::new(global);
    // Step 1. Populate request from client given request.
    // TODO
    // Step 2. Set request’s service-workers mode to "none".
    // TODO
    // Step 3. Set request’s keepalive to true.
    // TODO
    // Step 4. Let deferredRecord be a new deferred fetch record whose request is request, and whose notify invoked is onActivatedWithoutTermination.
    let deferred_record = Arc::new(Mutex::new(DeferredFetchRecord {
        request,
        global: trusted_global.clone(),
        invoke_state: Cell::new(DeferredFetchRecordInvokeState::Pending),
        activated: Cell::new(false),
    }));
    // Step 5. Append deferredRecord to request’s client’s fetch group’s deferred fetch records.
    // TODO
    // Step 6. If activateAfter is non-null, then run the following steps in parallel:
    let deferred_record_clone = deferred_record.clone();
    global.schedule_timer(TimerEventRequest {
        callback: Box::new(move || {
            // Step 6.2. Process deferredRecord.
            deferred_record_clone.lock().process();

            // Last step of https://fetch.spec.whatwg.org/#process-a-deferred-fetch
            //
            // Step 4. Queue a global task on the deferred fetch task source with
            // deferredRecord’s request’s client’s global object to run deferredRecord’s notify invoked.
            let deferred_record_clone = deferred_record_clone.clone();
            trusted_global
                .root()
                .task_manager()
                .deferred_fetch_task_source()
                .queue(task!(notify_deferred_record: move || {
                    deferred_record_clone.lock().activate();
                }));
        }),
        // Step 6.1. The user agent should wait until any of the following conditions is met:
        duration: Duration::from_millis(*activate_after as u64),
    });
    // Step 7. Return deferredRecord.
    deferred_record
}

/// <https://fetch.spec.whatwg.org/#dom-window-fetchlater>
#[allow(non_snake_case, unsafe_code)]
pub(crate) fn FetchLater(
    window: &Window,
    input: RequestInfo,
    init: RootedTraceableBox<DeferredRequestInit>,
    can_gc: CanGc,
) -> Fallible<DomRoot<FetchLaterResult>> {
    let global_scope = window.upcast();
    // Step 1. Let requestObject be the result of invoking the initial value
    // of Request as constructor with input and init as arguments.
    let request_object = Request::constructor(global_scope, None, can_gc, input, &init.parent)?;
    // Step 2. If requestObject’s signal is aborted, then throw signal’s abort reason.
    let signal = request_object.Signal();
    if signal.aborted() {
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut abort_reason = UndefinedValue());
        signal.Reason(cx, abort_reason.handle_mut());
        unsafe {
            assert!(!JS_IsExceptionPending(*cx));
            JS_SetPendingException(*cx, abort_reason.handle(), ExceptionStackBehavior::Capture);
        }
        return Err(Error::JSFailed);
    }
    // Step 3. Let request be requestObject’s request.
    let request = request_object.get_request();
    // Step 4. Let activateAfter be null.
    let mut activate_after = Finite::wrap(0_f64);
    // Step 5. If init is given and init["activateAfter"] exists, then set
    // activateAfter to init["activateAfter"].
    if let Some(init_activate_after) = init.activateAfter.as_ref() {
        activate_after = *init_activate_after;
    }
    // Step 6. If activateAfter is less than 0, then throw a RangeError.
    if *activate_after < 0.0 {
        return Err(Error::Range("activateAfter must be at least 0".to_owned()));
    }
    // Step 7. If this’s relevant global object’s associated document is not fully active, then throw a TypeError.
    if !window.Document().is_fully_active() {
        return Err(Error::Type("Document is not fully active".to_owned()));
    }
    let url = request.url();
    // Step 8. If request’s URL’s scheme is not an HTTP(S) scheme, then throw a TypeError.
    if !matches!(url.scheme(), "http" | "https") {
        return Err(Error::Type("URL is not http(s)".to_owned()));
    }
    // Step 9. If request’s URL is not a potentially trustworthy URL, then throw a SecurityError.
    if !url.is_potentially_trustworthy() {
        return Err(Error::Type("URL is not trustworthy".to_owned()));
    }
    // Step 10. If request’s body is not null, and request’s body length is null, then throw a TypeError.
    if let Some(body) = request.body.as_ref() {
        if body.len().is_none() {
            return Err(Error::Type("Body is null".to_owned()));
        }
    }
    // Step 11. If the available deferred-fetch quota given request’s client and request’s URL’s
    // origin is less than request’s total request length, then throw a "QuotaExceededError" DOMException.
    // TODO
    // Step 12. Let activated be false.
    // Step 13. Let deferredRecord be the result of calling queue a deferred fetch given request,
    // activateAfter, and the following step: set activated to true.
    let deferred_record = queue_deferred_fetch(request, activate_after, global_scope);
    // Step 14. Add the following abort steps to requestObject’s signal: Set deferredRecord’s invoke state to "aborted".
    signal.add(&AbortAlgorithm::FetchLater(deferred_record.clone()));
    // Step 15. Return a new FetchLaterResult whose activated getter steps are to return activated.
    Ok(FetchLaterResult::new(window, deferred_record, can_gc))
}

/// <https://fetch.spec.whatwg.org/#deferred-fetch-record-invoke-state>
#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
enum DeferredFetchRecordInvokeState {
    Pending,
    Sent,
    Aborted,
}

/// <https://fetch.spec.whatwg.org/#deferred-fetch-record>
#[derive(MallocSizeOf)]
pub(crate) struct DeferredFetchRecord {
    /// <https://fetch.spec.whatwg.org/#deferred-fetch-record-request>
    request: NetTraitsRequest,
    /// <https://fetch.spec.whatwg.org/#deferred-fetch-record-invoke-state>
    invoke_state: Cell<DeferredFetchRecordInvokeState>,
    global: Trusted<GlobalScope>,
    activated: Cell<bool>,
}

impl DeferredFetchRecord {
    /// Part of step 13 of <https://fetch.spec.whatwg.org/#dom-window-fetchlater>
    fn activate(&self) {
        // and the following step: set activated to true.
        self.activated.set(true);
    }
    /// Part of step 14 of <https://fetch.spec.whatwg.org/#dom-window-fetchlater>
    pub(crate) fn abort(&self) {
        // Set deferredRecord’s invoke state to "aborted".
        self.invoke_state
            .set(DeferredFetchRecordInvokeState::Aborted);
    }
    /// Part of step 15 of <https://fetch.spec.whatwg.org/#dom-window-fetchlater>
    pub(crate) fn activated_getter_steps(&self) -> bool {
        // whose activated getter steps are to return activated.
        self.activated.get()
    }
    /// <https://fetch.spec.whatwg.org/#process-a-deferred-fetch>
    fn process(&self) {
        // Step 1. If deferredRecord’s invoke state is not "pending", then return.
        if self.invoke_state.get() != DeferredFetchRecordInvokeState::Pending {
            return;
        }
        // Step 2. Set deferredRecord’s invoke state to "sent".
        self.invoke_state.set(DeferredFetchRecordInvokeState::Sent);
        // Step 3. Fetch deferredRecord’s request.
        let url = self.request.url().clone();
        let fetch_later_listener = FetchLaterListener {
            url,
            global: self.global.clone(),
        };
        let global = self.global.root();
        let request_init = request_init_from_request(self.request.clone(), &global);
        global.fetch(
            request_init,
            fetch_later_listener,
            global.task_manager().networking_task_source().to_sendable(),
        );
        // Step 4 is handled by caller
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct FetchContext {
    #[ignore_malloc_size_of = "unclear ownership semantics"]
    fetch_promise: Option<TrustedPromise>,
    response_object: Trusted<Response>,
    request: Trusted<Request>,
    global: Trusted<GlobalScope>,
    locally_aborted: bool,
    canceller: FetchCanceller,
}

impl FetchContext {
    /// Step 11 of <https://fetch.spec.whatwg.org/#dom-global-fetch>
    pub(crate) fn abort_fetch(
        &mut self,
        abort_reason: HandleValue,
        cx: SafeJSContext,
        can_gc: CanGc,
    ) {
        // Step 11.1. Set locallyAborted to true.
        self.locally_aborted = true;
        // Step 11.2. Assert: controller is non-null.
        //
        // N/a, that's self

        // Step 11.3. Abort controller with requestObject’s signal’s abort reason.
        self.canceller.cancel();

        // Step 11.4. Abort the fetch() call with p, request, responseObject,
        // and requestObject’s signal’s abort reason.
        let promise = self
            .fetch_promise
            .take()
            .expect("fetch promise is missing")
            .root();
        abort_fetch_call(
            promise,
            &self.request.root(),
            Some(&self.response_object.root()),
            abort_reason,
            &self.global.root(),
            cx,
            can_gc,
        );
    }
}

/// Step 12 of <https://fetch.spec.whatwg.org/#dom-global-fetch>
impl FetchResponseListener for FetchContext {
    fn process_request_body(&mut self, _: RequestId) {
        // TODO
    }

    fn process_request_eof(&mut self, _: RequestId) {
        // TODO
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn process_response(
        &mut self,
        _: RequestId,
        fetch_metadata: Result<FetchMetadata, NetworkError>,
    ) {
        // Step 12.1. If locallyAborted is true, then abort these steps.
        if self.locally_aborted {
            return;
        }
        let promise = self
            .fetch_promise
            .take()
            .expect("fetch promise is missing")
            .root();

        let _ac = enter_realm(&*promise);
        match fetch_metadata {
            // Step 12.3. If response is a network error, then reject
            // p with a TypeError and abort these steps.
            Err(_) => {
                promise.reject_error(
                    Error::Type("Network error occurred".to_string()),
                    CanGc::note(),
                );
                self.fetch_promise = Some(TrustedPromise::new(promise));
                let response = self.response_object.root();
                response.set_type(DOMResponseType::Error, CanGc::note());
                response.error_stream(
                    Error::Type("Network error occurred".to_string()),
                    CanGc::note(),
                );
                return;
            },
            // Step 12.4. Set responseObject to the result of creating a Response object,
            // given response, "immutable", and relevantRealm.
            Ok(metadata) => match metadata {
                FetchMetadata::Unfiltered(m) => {
                    fill_headers_with_metadata(self.response_object.root(), m, CanGc::note());
                    self.response_object
                        .root()
                        .set_type(DOMResponseType::Default, CanGc::note());
                },
                FetchMetadata::Filtered { filtered, .. } => match filtered {
                    FilteredMetadata::Basic(m) => {
                        fill_headers_with_metadata(self.response_object.root(), m, CanGc::note());
                        self.response_object
                            .root()
                            .set_type(DOMResponseType::Basic, CanGc::note());
                    },
                    FilteredMetadata::Cors(m) => {
                        fill_headers_with_metadata(self.response_object.root(), m, CanGc::note());
                        self.response_object
                            .root()
                            .set_type(DOMResponseType::Cors, CanGc::note());
                    },
                    FilteredMetadata::Opaque => {
                        self.response_object
                            .root()
                            .set_type(DOMResponseType::Opaque, CanGc::note());
                    },
                    FilteredMetadata::OpaqueRedirect(url) => {
                        let r = self.response_object.root();
                        r.set_type(DOMResponseType::Opaqueredirect, CanGc::note());
                        r.set_final_url(url);
                    },
                },
            },
        }

        // Step 12.5. Resolve p with responseObject.
        promise.resolve_native(&self.response_object.root(), CanGc::note());
        self.fetch_promise = Some(TrustedPromise::new(promise));
    }

    fn process_response_chunk(&mut self, _: RequestId, chunk: Vec<u8>) {
        let response = self.response_object.root();
        response.stream_chunk(chunk, CanGc::note());
    }

    fn process_response_eof(
        self,
        _: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        let response_object = self.response_object.root();
        let _ac = enter_realm(&*response_object);
        response_object.finish(CanGc::note());
        // TODO
        // ... trailerObject is not supported in Servo yet.

        // navigation submission is handled in servoparser/mod.rs
        if let Ok(response) = response {
            if response.timing_type == ResourceTimingType::Resource {
                network_listener::submit_timing(&self, &response, CanGc::note());
            }
        }
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None, None);
    }
}

impl ResourceTimingListener for FetchContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (
            InitiatorType::Fetch,
            self.resource_timing_global().get_url().clone(),
        )
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.response_object.root().global()
    }
}

struct FetchLaterListener {
    /// URL of this request.
    url: ServoUrl,
    /// The global object fetching the report uri violation
    global: Trusted<GlobalScope>,
}

impl FetchResponseListener for FetchLaterListener {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
        _: RequestId,
        fetch_metadata: Result<FetchMetadata, NetworkError>,
    ) {
        _ = fetch_metadata;
    }

    fn process_response_chunk(&mut self, _: RequestId, chunk: Vec<u8>) {
        _ = chunk;
    }

    fn process_response_eof(
        self,
        _: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        if let Ok(response) = response {
            network_listener::submit_timing(&self, &response, CanGc::note());
        }
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = self.resource_timing_global();
        global.report_csp_violations(violations, None, None);
    }
}

impl ResourceTimingListener for FetchLaterListener {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (InitiatorType::Fetch, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.global.root()
    }
}

fn fill_headers_with_metadata(r: DomRoot<Response>, m: Metadata, can_gc: CanGc) {
    r.set_headers(m.headers, can_gc);
    r.set_status(&m.status);
    r.set_final_url(m.final_url);
    r.set_redirected(m.redirected);
}

pub(crate) trait CspViolationsProcessor {
    fn process_csp_violations(&self, violations: Vec<Violation>);
}

/// Convenience function for synchronously loading a whole resource.
pub(crate) fn load_whole_resource(
    request: RequestBuilder,
    core_resource_thread: &CoreResourceThread,
    global: &GlobalScope,
    csp_violations_processor: &dyn CspViolationsProcessor,
    can_gc: CanGc,
) -> Result<(Metadata, Vec<u8>), NetworkError> {
    let request = request.https_state(global.get_https_state());
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let url = request.url.clone();
    core_resource_thread
        .send(CoreResourceMsg::Fetch(
            request,
            FetchChannels::ResponseMsg(action_sender),
        ))
        .unwrap();

    let mut buf = vec![];
    let mut metadata = None;
    loop {
        match action_receiver.recv().unwrap() {
            FetchResponseMsg::ProcessRequestBody(..) | FetchResponseMsg::ProcessRequestEOF(..) => {
            },
            FetchResponseMsg::ProcessResponse(_, Ok(m)) => {
                metadata = Some(match m {
                    FetchMetadata::Unfiltered(m) => m,
                    FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
                })
            },
            FetchResponseMsg::ProcessResponseChunk(_, data) => buf.extend_from_slice(&data),
            FetchResponseMsg::ProcessResponseEOF(_, Ok(_)) => {
                let metadata = metadata.unwrap();
                if let Some(timing) = &metadata.timing {
                    submit_timing_data(global, url, InitiatorType::Other, timing, can_gc);
                }
                return Ok((metadata, buf));
            },
            FetchResponseMsg::ProcessResponse(_, Err(e)) |
            FetchResponseMsg::ProcessResponseEOF(_, Err(e)) => return Err(e),
            FetchResponseMsg::ProcessCspViolations(_, violations) => {
                csp_violations_processor.process_csp_violations(violations);
            },
        }
    }
}

/// <https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request>
#[allow(clippy::too_many_arguments)]
pub(crate) fn create_a_potential_cors_request(
    webview_id: Option<WebViewId>,
    url: ServoUrl,
    destination: Destination,
    cors_setting: Option<CorsSettings>,
    same_origin_fallback: Option<bool>,
    referrer: Referrer,
    insecure_requests_policy: InsecureRequestsPolicy,
    has_trustworthy_ancestor_origin: bool,
    policy_container: PolicyContainer,
) -> RequestBuilder {
    RequestBuilder::new(webview_id, url, referrer)
        // https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request
        // Step 1
        .mode(match cors_setting {
            Some(_) => RequestMode::CorsMode,
            None if same_origin_fallback == Some(true) => RequestMode::SameOrigin,
            None => RequestMode::NoCors,
        })
        // https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request
        // Step 3-4
        .credentials_mode(match cors_setting {
            Some(CorsSettings::Anonymous) => CredentialsMode::CredentialsSameOrigin,
            _ => CredentialsMode::Include,
        })
        // Step 5
        .destination(destination)
        .use_url_credentials(true)
        .insecure_requests_policy(insecure_requests_policy)
        .has_trustworthy_ancestor_origin(has_trustworthy_ancestor_origin)
        .policy_container(policy_container)
}
