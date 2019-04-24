/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::RequestBinding::RequestInfo;
use crate::dom::bindings::codegen::Bindings::RequestBinding::RequestInit;
use crate::dom::bindings::codegen::Bindings::ResponseBinding::ResponseBinding::ResponseMethods;
use crate::dom::bindings::codegen::Bindings::ResponseBinding::ResponseType as DOMResponseType;
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
use crate::network_listener::{self, NetworkListener, PreInvoke, ResourceTimingListener};
use crate::task_source::TaskSourceName;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::jsapi::JSAutoCompartment;
use net_traits::request::RequestBuilder;
use net_traits::request::{Request as NetTraitsRequest, ServiceWorkersMode};
use net_traits::CoreResourceMsg::Fetch as NetTraitsFetch;
use net_traits::{FetchChannels, FetchResponseListener, NetworkError};
use net_traits::{FetchMetadata, FilteredMetadata, Metadata};
use net_traits::{ResourceFetchTiming, ResourceTimingType};
use servo_url::ServoUrl;
use std::mem;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

struct FetchContext {
    fetch_promise: Option<TrustedPromise>,
    response_object: Trusted<Response>,
    body: Vec<u8>,
    resource_timing: ResourceFetchTiming,
}

/// RAII fetch canceller object. By default initialized to not having a canceller
/// in it, however you can ask it for a cancellation receiver to send to Fetch
/// in which case it will store the sender. You can manually cancel it
/// or let it cancel on Drop in that case.
#[derive(Default, JSTraceable, MallocSizeOf)]
pub struct FetchCanceller {
    #[ignore_malloc_size_of = "channels are hard"]
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

fn from_referrer_to_referrer_url(request: &NetTraitsRequest) -> Option<ServoUrl> {
    request.referrer.to_url().map(|url| url.clone())
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
        referrer_url: from_referrer_to_referrer_url(&request),
        referrer_policy: request.referrer_policy,
        pipeline_id: request.pipeline_id,
        redirect_mode: request.redirect_mode,
        integrity_metadata: "".to_owned(),
        url_list: vec![],
    }
}

// https://fetch.spec.whatwg.org/#fetch-method
#[allow(unrooted_must_root)]
#[allow(unsafe_code)]
pub fn Fetch(
    global: &GlobalScope,
    input: RequestInfo,
    init: RootedTraceableBox<RequestInit>,
) -> Rc<Promise> {
    let core_resource_thread = global.core_resource_thread();

    // Step 1
    let promise = unsafe { Promise::new_in_current_compartment(global) };
    let response = Response::new(global);

    // Step 2
    let request = match Request::Constructor(global, input, init) {
        Err(e) => {
            promise.reject_error(e);
            return promise;
        },
        Ok(r) => r.get_request(),
    };
    let timing_type = request.timing_type();

    let mut request_init = request_init_from_request(request);

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
        body: vec![],
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

    #[allow(unrooted_must_root)]
    fn process_response(&mut self, fetch_metadata: Result<FetchMetadata, NetworkError>) {
        let promise = self
            .fetch_promise
            .take()
            .expect("fetch promise is missing")
            .root();

        // JSAutoCompartment needs to be manually made.
        // Otherwise, Servo will crash.
        let promise_cx = promise.global().get_cx();
        let _ac = JSAutoCompartment::new(promise_cx, promise.reflector().get_jsobject().get());
        match fetch_metadata {
            // Step 4.1
            Err(_) => {
                promise.reject_error(Error::Type("Network error occurred".to_string()));
                self.fetch_promise = Some(TrustedPromise::new(promise));
                self.response_object.root().set_type(DOMResponseType::Error);
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
                    FilteredMetadata::Opaque => self
                        .response_object
                        .root()
                        .set_type(DOMResponseType::Opaque),
                    FilteredMetadata::OpaqueRedirect => self
                        .response_object
                        .root()
                        .set_type(DOMResponseType::Opaqueredirect),
                },
            },
        }
        // Step 4.3
        promise.resolve_native(&self.response_object.root());
        self.fetch_promise = Some(TrustedPromise::new(promise));
    }

    fn process_response_chunk(&mut self, mut chunk: Vec<u8>) {
        self.body.append(&mut chunk);
    }

    fn process_response_eof(&mut self, _response: Result<ResourceFetchTiming, NetworkError>) {
        let response = self.response_object.root();
        let global = response.global();
        let cx = global.get_cx();
        let _ac = JSAutoCompartment::new(cx, global.reflector().get_jsobject().get());
        response.finish(mem::replace(&mut self.body, vec![]));
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
        match self.resource_timing.timing_type {
            ResourceTimingType::Resource => network_listener::submit_timing(self),
            _ => {},
        };
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
}
