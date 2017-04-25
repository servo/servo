/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::RequestBinding::RequestInfo;
use dom::bindings::codegen::Bindings::RequestBinding::RequestInit;
use dom::bindings::codegen::Bindings::ResponseBinding::ResponseBinding::ResponseMethods;
use dom::bindings::codegen::Bindings::ResponseBinding::ResponseType as DOMResponseType;
use dom::bindings::error::Error;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::DomObject;
use dom::bindings::trace::RootedTraceableBox;
use dom::globalscope::GlobalScope;
use dom::headers::Guard;
use dom::promise::Promise;
use dom::request::Request;
use dom::response::Response;
use dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::jsapi::JSAutoCompartment;
use net_traits::{FetchResponseListener, NetworkError};
use net_traits::{FilteredMetadata, FetchMetadata, Metadata};
use net_traits::CoreResourceMsg::Fetch as NetTraitsFetch;
use net_traits::request::{Request as NetTraitsRequest, ServiceWorkersMode};
use net_traits::request::RequestInit as NetTraitsRequestInit;
use network_listener::{NetworkListener, PreInvoke};
use servo_url::ServoUrl;
use std::mem;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

struct FetchContext {
    fetch_promise: Option<TrustedPromise>,
    response_object: Trusted<Response>,
    body: Vec<u8>,
}

fn from_referrer_to_referrer_url(request: &NetTraitsRequest) -> Option<ServoUrl> {
    request.referrer.to_url().map(|url| url.clone())
}

fn request_init_from_request(request: NetTraitsRequest) -> NetTraitsRequestInit {
    NetTraitsRequestInit {
        method: request.method.clone(),
        url: request.url(),
        headers: request.headers.clone(),
        unsafe_request: request.unsafe_request,
        body: request.body.clone(),
        type_: request.type_,
        destination: request.destination,
        synchronous: request.synchronous,
        mode: request.mode,
        use_cors_preflight: request.use_cors_preflight,
        credentials_mode: request.credentials_mode,
        use_url_credentials: request.use_url_credentials,
        origin: GlobalScope::current().expect("No current global object").origin().immutable().clone(),
        referrer_url: from_referrer_to_referrer_url(&request),
        referrer_policy: request.referrer_policy,
        pipeline_id: request.pipeline_id,
        redirect_mode: request.redirect_mode,
        ..NetTraitsRequestInit::default()
    }
}

// https://fetch.spec.whatwg.org/#fetch-method
#[allow(unrooted_must_root)]
pub fn Fetch(global: &GlobalScope, input: RequestInfo, init: RootedTraceableBox<RequestInit>) -> Rc<Promise> {
    let core_resource_thread = global.core_resource_thread();

    // Step 1
    let promise = Promise::new(global);
    let response = Response::new(global);

    // Step 2
    let request = match Request::Constructor(global, input, init) {
        Err(e) => {
            promise.reject_error(promise.global().get_cx(), e);
            return promise;
        },
        Ok(r) => r.get_request(),
    };
    let mut request_init = request_init_from_request(request);

    // Step 3
    if global.downcast::<ServiceWorkerGlobalScope>().is_some() {
        request_init.service_workers_mode = ServiceWorkersMode::Foreign;
    }

    // Step 4
    response.Headers().set_guard(Guard::Immutable);

    // Step 5
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let fetch_context = Arc::new(Mutex::new(FetchContext {
        fetch_promise: Some(TrustedPromise::new(promise.clone())),
        response_object: Trusted::new(&*response),
        body: vec![],
    }));
    let listener = NetworkListener {
        context: fetch_context,
        task_source: global.networking_task_source(),
        wrapper: Some(global.get_runnable_wrapper())
    };

    ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
        listener.notify_fetch(message.to().unwrap());
    });
    core_resource_thread.send(NetTraitsFetch(request_init, action_sender)).unwrap();

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
        let promise = self.fetch_promise.take().expect("fetch promise is missing").root();

        // JSAutoCompartment needs to be manually made.
        // Otherwise, Servo will crash.
        let promise_cx = promise.global().get_cx();
        let _ac = JSAutoCompartment::new(promise_cx, promise.reflector().get_jsobject().get());
        match fetch_metadata {
            // Step 4.1
            Err(_) => {
                promise.reject_error(
                    promise.global().get_cx(),
                    Error::Type("Network error occurred".to_string()));
                self.fetch_promise = Some(TrustedPromise::new(promise));
                self.response_object.root().set_type(DOMResponseType::Error);
                return;
            },
            // Step 4.2
            Ok(metadata) => {
                match metadata {
                    FetchMetadata::Unfiltered(m) => {
                        fill_headers_with_metadata(self.response_object.root(), m);
                        self.response_object.root().set_type(DOMResponseType::Default);
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
                        FilteredMetadata::Opaque =>
                            self.response_object.root().set_type(DOMResponseType::Opaque),
                        FilteredMetadata::OpaqueRedirect =>
                            self.response_object.root().set_type(DOMResponseType::Opaqueredirect)
                    }
                }
            }
        }
        // Step 4.3
        promise.resolve_native(
            promise_cx,
            &self.response_object.root());
        self.fetch_promise = Some(TrustedPromise::new(promise));
    }

    fn process_response_chunk(&mut self, mut chunk: Vec<u8>) {
        self.body.append(&mut chunk);
    }

    fn process_response_eof(&mut self, _response: Result<(), NetworkError>) {
        let response = self.response_object.root();
        let global = response.global();
        let cx = global.get_cx();
        let _ac = JSAutoCompartment::new(cx, global.reflector().get_jsobject().get());
        response.finish(mem::replace(&mut self.body, vec![]));
        // TODO
        // ... trailerObject is not supported in Servo yet.
    }
}

fn fill_headers_with_metadata(r: Root<Response>, m: Metadata) {
    r.set_headers(m.headers);
    r.set_raw_status(m.status);
    r.set_final_url(m.final_url);
}
