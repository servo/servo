/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::request::{
    CorsSettings, CredentialsMode, Destination, Referrer, Request as NetTraitsRequest,
    RequestBuilder, RequestMode, ServiceWorkersMode,
};
use net_traits::CoreResourceMsg::Fetch as NetTraitsFetch;
use net_traits::{
    CoreResourceMsg, CoreResourceThread, FetchChannels, FetchMetadata, FetchResponseListener,
    FetchResponseMsg, FilteredMetadata, Metadata, NetworkError, ResourceFetchTiming,
    ResourceTimingType,
};
use servo_url::ServoUrl;

use crate::dom::bindings::codegen::Bindings::RequestBinding::{RequestInfo, RequestInit};
use crate::dom::bindings::codegen::Bindings::ResponseBinding::ResponseType as DOMResponseType;
use crate::dom::bindings::codegen::Bindings::ResponseBinding::Response_Binding::ResponseMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::headers::Guard;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::request::Request;
use crate::dom::response::Response;
use crate::dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use crate::network_listener::{
    self, submit_timing_data, NetworkListener, PreInvoke, ResourceTimingListener,
};
use crate::realms::{enter_realm, InRealm};
use crate::task_source::TaskSourceName;

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
pub struct FetchCanceller {
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    cancel_chan: Option<ipc::IpcSender<()>>,
}

impl FetchCanceller {
    /// Create an empty FetchCanceller
    pub fn new() -> Self {
        Default::default()
    }

    /// Obtain an IpcReceiver to send over to Fetch, and initialize
    /// the internal sender
    pub fn initialize(&mut self) -> ipc::IpcReceiver<()> {
        // cancel previous fetch
        self.cancel();
        let (rx, tx) = ipc::channel().unwrap();
        self.cancel_chan = Some(rx);
        tx
    }

    /// Cancel a fetch if it is ongoing
    pub fn cancel(&mut self) {
        if let Some(chan) = self.cancel_chan.take() {
            // stop trying to make fetch happen
            // it's not going to happen

            // The receiver will be destroyed if the request has already completed;
            // so we throw away the error. Cancellation is a courtesy call,
            // we don't actually care if the other side heard.
            let _ = chan.send(());
        }
    }

    /// Use this if you don't want it to send a cancellation request
    /// on drop (e.g. if the fetch completes)
    pub fn ignore(&mut self) {
        let _ = self.cancel_chan.take();
    }
}

impl Drop for FetchCanceller {
    fn drop(&mut self) {
        self.cancel()
    }
}

fn request_init_from_request(request: NetTraitsRequest) -> RequestBuilder {
    RequestBuilder {
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
        redirect_mode: request.redirect_mode,
        integrity_metadata: request.integrity_metadata.clone(),
        url_list: vec![],
        parser_metadata: request.parser_metadata,
        initiator: request.initiator,
        csp_list: None,
        https_state: request.https_state,
        response_tainting: request.response_tainting,
        crash: None,
    }
}

// https://fetch.spec.whatwg.org/#fetch-method
#[allow(crown::unrooted_must_root, non_snake_case)]
pub fn Fetch(
    global: &GlobalScope,
    input: RequestInfo,
    init: RootedTraceableBox<RequestInit>,
    comp: InRealm,
) -> Rc<Promise> {
    let core_resource_thread = global.core_resource_thread();

    // Step 1
    let promise = Promise::new_in_current_realm(comp);
    let response = Response::new(global);

    // Step 2
    let request = match Request::Constructor(global, None, input, init) {
        Err(e) => {
            response.error_stream(e.clone());
            promise.reject_error(e);
            return promise;
        },
        Ok(r) => r.get_request(),
    };
    let timing_type = request.timing_type();

    let mut request_init = request_init_from_request(request);
    request_init.csp_list.clone_from(&global.get_csp_list());

    // Step 3
    if global.downcast::<ServiceWorkerGlobalScope>().is_some() {
        request_init.service_workers_mode = ServiceWorkersMode::None;
    }

    // Step 4
    response.Headers().set_guard(Guard::Immutable);

    // Step 5
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let fetch_context = Arc::new(Mutex::new(FetchContext {
        fetch_promise: Some(TrustedPromise::new(promise.clone())),
        response_object: Trusted::new(&*response),
        resource_timing: ResourceFetchTiming::new(timing_type),
    }));
    let listener = NetworkListener {
        context: fetch_context,
        task_source: global.networking_task_source(),
        canceller: Some(global.task_canceller(TaskSourceName::Networking)),
    };

    ROUTER.add_route(
        action_receiver.to_opaque(),
        Box::new(move |message| {
            listener.notify_fetch(message.to().unwrap());
        }),
    );
    core_resource_thread
        .send(NetTraitsFetch(
            request_init,
            FetchChannels::ResponseMsg(action_sender, None),
        ))
        .unwrap();

    promise
}

impl PreInvoke for FetchContext {}

impl FetchResponseListener for FetchContext {
    fn process_request_body(&mut self) {
        // TODO
    }

    fn process_request_eof(&mut self) {
        // TODO
    }

    #[allow(crown::unrooted_must_root)]
    fn process_response(&mut self, fetch_metadata: Result<FetchMetadata, NetworkError>) {
        let promise = self
            .fetch_promise
            .take()
            .expect("fetch promise is missing")
            .root();

        let _ac = enter_realm(&*promise);
        match fetch_metadata {
            // Step 4.1
            Err(_) => {
                promise.reject_error(Error::Type("Network error occurred".to_string()));
                self.fetch_promise = Some(TrustedPromise::new(promise));
                let response = self.response_object.root();
                response.set_type(DOMResponseType::Error);
                response.error_stream(Error::Type("Network error occurred".to_string()));
                return;
            },
            // Step 4.2
            Ok(metadata) => match metadata {
                FetchMetadata::Unfiltered(m) => {
                    fill_headers_with_metadata(self.response_object.root(), m);
                    self.response_object
                        .root()
                        .set_type(DOMResponseType::Default);
                },
                FetchMetadata::Filtered { filtered, .. } => match filtered {
                    FilteredMetadata::Basic(m) => {
                        fill_headers_with_metadata(self.response_object.root(), m);
                        self.response_object.root().set_type(DOMResponseType::Basic);
                    },
                    FilteredMetadata::Cors(m) => {
                        fill_headers_with_metadata(self.response_object.root(), m);
                        self.response_object.root().set_type(DOMResponseType::Cors);
                    },
                    FilteredMetadata::Opaque => {
                        self.response_object
                            .root()
                            .set_type(DOMResponseType::Opaque);
                    },
                    FilteredMetadata::OpaqueRedirect(url) => {
                        let r = self.response_object.root();
                        r.set_type(DOMResponseType::Opaqueredirect);
                        r.set_final_url(url);
                    },
                },
            },
        }
        // Step 4.3
        promise.resolve_native(&self.response_object.root());
        self.fetch_promise = Some(TrustedPromise::new(promise));
    }

    fn process_response_chunk(&mut self, chunk: Vec<u8>) {
        let response = self.response_object.root();
        response.stream_chunk(chunk);
    }

    fn process_response_eof(&mut self, _response: Result<ResourceFetchTiming, NetworkError>) {
        let response = self.response_object.root();
        let _ac = enter_realm(&*response);
        response.finish();
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
            network_listener::submit_timing(self)
        }
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

fn fill_headers_with_metadata(r: DomRoot<Response>, m: Metadata) {
    r.set_headers(m.headers);
    r.set_raw_status(m.status);
    r.set_final_url(m.final_url);
    r.set_redirected(m.redirected);
}

/// Convenience function for synchronously loading a whole resource.
pub fn load_whole_resource(
    request: RequestBuilder,
    core_resource_thread: &CoreResourceThread,
    global: &GlobalScope,
) -> Result<(Metadata, Vec<u8>), NetworkError> {
    let request = request.https_state(global.get_https_state());
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let url = request.url.clone();
    core_resource_thread
        .send(CoreResourceMsg::Fetch(
            request,
            FetchChannels::ResponseMsg(action_sender, None),
        ))
        .unwrap();

    let mut buf = vec![];
    let mut metadata = None;
    loop {
        match action_receiver.recv().unwrap() {
            FetchResponseMsg::ProcessRequestBody | FetchResponseMsg::ProcessRequestEOF => (),
            FetchResponseMsg::ProcessResponse(Ok(m)) => {
                metadata = Some(match m {
                    FetchMetadata::Unfiltered(m) => m,
                    FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
                })
            },
            FetchResponseMsg::ProcessResponseChunk(data) => buf.extend_from_slice(&data),
            FetchResponseMsg::ProcessResponseEOF(Ok(_)) => {
                let metadata = metadata.unwrap();
                if let Some(timing) = &metadata.timing {
                    submit_timing_data(global, url, InitiatorType::Other, timing);
                }
                return Ok((metadata, buf));
            },
            FetchResponseMsg::ProcessResponse(Err(e)) |
            FetchResponseMsg::ProcessResponseEOF(Err(e)) => return Err(e),
        }
    }
}

/// <https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request>
pub(crate) fn create_a_potential_cors_request(
    url: ServoUrl,
    destination: Destination,
    cors_setting: Option<CorsSettings>,
    same_origin_fallback: Option<bool>,
    referrer: Referrer,
) -> RequestBuilder {
    RequestBuilder::new(url, referrer)
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
}
