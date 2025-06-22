/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use base::id::WebViewId;
use ipc_channel::ipc;
use net_traits::policy_container::{PolicyContainer, RequestPolicyContainer};
use net_traits::request::{
    CorsSettings, CredentialsMode, Destination, InsecureRequestsPolicy, Referrer,
    Request as NetTraitsRequest, RequestBuilder, RequestId, RequestMode, ServiceWorkersMode,
};
use net_traits::{
    CoreResourceMsg, CoreResourceThread, FetchChannels, FetchMetadata, FetchResponseListener,
    FetchResponseMsg, FilteredMetadata, Metadata, NetworkError, ResourceFetchTiming,
    ResourceTimingType, cancel_async_fetch,
};
use servo_url::ServoUrl;

use crate::dom::bindings::codegen::Bindings::RequestBinding::{
    RequestInfo, RequestInit, RequestMethods,
};
use crate::dom::bindings::codegen::Bindings::ResponseBinding::Response_Binding::ResponseMethods;
use crate::dom::bindings::codegen::Bindings::ResponseBinding::ResponseType as DOMResponseType;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::globalscope::GlobalScope;
use crate::dom::headers::Guard;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::request::Request;
use crate::dom::response::Response;
use crate::dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use crate::network_listener::{self, PreInvoke, ResourceTimingListener, submit_timing_data};
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::CanGc;

struct FetchContext {
    fetch_promise: Option<TrustedPromise>,
    response_object: Trusted<Response>,
    resource_timing: ResourceFetchTiming,
}

/// RAII fetch canceller object. By default initialized to not having a canceller
/// in it, however you can ask it for a cancellation receiver to send to Fetch
/// in which case it will store the sender. You can manually cancel it
/// or let it cancel on Drop in that case.
#[derive(Default, JSTraceable, MallocSizeOf)]
pub(crate) struct FetchCanceller {
    #[no_trace]
    request_id: Option<RequestId>,
}

impl FetchCanceller {
    /// Create an empty FetchCanceller
    pub(crate) fn new(request_id: RequestId) -> Self {
        Self {
            request_id: Some(request_id),
        }
    }

    /// Cancel a fetch if it is ongoing
    pub(crate) fn cancel(&mut self) {
        if let Some(request_id) = self.request_id.take() {
            // stop trying to make fetch happen
            // it's not going to happen

            // No error handling here. Cancellation is a courtesy call,
            // we don't actually care if the other side heard.
            cancel_async_fetch(vec![request_id]);
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

fn request_init_from_request(request: NetTraitsRequest) -> RequestBuilder {
    RequestBuilder {
        id: request.id,
        method: request.method.clone(),
        url: request.url(),
        headers: request.headers.clone(),
        unsafe_request: request.unsafe_request,
        body: request.body.clone(),
        service_workers_mode: ServiceWorkersMode::All,
        destination: request.destination,
        synchronous: request.synchronous,
        mode: request.mode.clone(),
        cache_mode: request.cache_mode,
        use_cors_preflight: request.use_cors_preflight,
        credentials_mode: request.credentials_mode,
        use_url_credentials: request.use_url_credentials,
        origin: GlobalScope::current()
            .expect("No current global object")
            .origin()
            .immutable()
            .clone(),
        referrer: request.referrer.clone(),
        referrer_policy: request.referrer_policy,
        pipeline_id: request.pipeline_id,
        target_webview_id: request.target_webview_id,
        redirect_mode: request.redirect_mode,
        integrity_metadata: request.integrity_metadata.clone(),
        cryptographic_nonce_metadata: request.cryptographic_nonce_metadata.clone(),
        url_list: vec![],
        parser_metadata: request.parser_metadata,
        initiator: request.initiator,
        policy_container: request.policy_container,
        insecure_requests_policy: request.insecure_requests_policy,
        has_trustworthy_ancestor_origin: request.has_trustworthy_ancestor_origin,
        https_state: request.https_state,
        response_tainting: request.response_tainting,
        crash: None,
    }
}

/// <https://fetch.spec.whatwg.org/#fetch-method>
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

    // Step 7. Let responseObject be null.
    // NOTE: We do initialize the object earlier earlier so we can use it to track errors
    let response = Response::new(global, can_gc);
    response.Headers(can_gc).set_guard(Guard::Immutable);

    // Step 2. Let requestObject be the result of invoking the initial value of Request as constructor
    //         with input and init as arguments. If this throws an exception, reject p with it and return p.
    let request = match Request::Constructor(global, None, can_gc, input, init) {
        Err(e) => {
            response.error_stream(e.clone(), can_gc);
            promise.reject_error(e, can_gc);
            return promise;
        },
        Ok(r) => {
            // Step 3. Let request be requestObject’s request.
            r.get_request()
        },
    };
    let timing_type = request.timing_type();

    let mut request_init = request_init_from_request(request);
    request_init.policy_container =
        RequestPolicyContainer::PolicyContainer(global.policy_container());

    // TODO: Step 4. If requestObject’s signal is aborted, then: [..]

    // Step 5. Let globalObject be request’s client’s global object.
    // NOTE:   We already get the global object as an argument

    // Step 6. If globalObject is a ServiceWorkerGlobalScope object, then set request’s
    //         service-workers mode to "none".
    if global.is::<ServiceWorkerGlobalScope>() {
        request_init.service_workers_mode = ServiceWorkersMode::None;
    }

    // TODO: Steps 8-11, abortcontroller stuff

    // Step 12. Set controller to the result of calling fetch given request and
    //           processResponse given response being these steps: [..]
    let fetch_context = Arc::new(Mutex::new(FetchContext {
        fetch_promise: Some(TrustedPromise::new(promise.clone())),
        response_object: Trusted::new(&*response),
        resource_timing: ResourceFetchTiming::new(timing_type),
    }));

    global.fetch(
        request_init,
        fetch_context,
        global.task_manager().networking_task_source().to_sendable(),
    );

    // Step 13. Return p.
    promise
}

impl PreInvoke for FetchContext {}

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
        let promise = self
            .fetch_promise
            .take()
            .expect("fetch promise is missing")
            .root();

        let _ac = enter_realm(&*promise);
        match fetch_metadata {
            // Step 4.1
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
            // Step 4.2
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

        // Step 4.3
        promise.resolve_native(&self.response_object.root(), CanGc::note());
        self.fetch_promise = Some(TrustedPromise::new(promise));
    }

    fn process_response_chunk(&mut self, _: RequestId, chunk: Vec<u8>) {
        let response = self.response_object.root();
        response.stream_chunk(chunk, CanGc::note());
    }

    fn process_response_eof(
        &mut self,
        _: RequestId,
        _response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        let response = self.response_object.root();
        let _ac = enter_realm(&*response);
        response.finish(CanGc::note());
        // TODO
        // ... trailerObject is not supported in Servo yet.
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        // navigation submission is handled in servoparser/mod.rs
        if self.resource_timing.timing_type == ResourceTimingType::Resource {
            network_listener::submit_timing(self, CanGc::note())
        }
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None);
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

fn fill_headers_with_metadata(r: DomRoot<Response>, m: Metadata, can_gc: CanGc) {
    r.set_headers(m.headers, can_gc);
    r.set_status(&m.status);
    r.set_final_url(m.final_url);
    r.set_redirected(m.redirected);
}

/// Convenience function for synchronously loading a whole resource.
pub(crate) fn load_whole_resource(
    request: RequestBuilder,
    core_resource_thread: &CoreResourceThread,
    global: &GlobalScope,
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
            FetchResponseMsg::ProcessRequestBody(..) |
            FetchResponseMsg::ProcessRequestEOF(..) |
            FetchResponseMsg::ProcessCspViolations(..) => {},
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
