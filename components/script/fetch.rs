/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::RequestBinding::RequestInit;
use dom::bindings::codegen::Bindings::ResponseBinding::ResponseBinding::ResponseMethods;
use dom::bindings::codegen::UnionTypes::RequestOrUSVString;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::Reflectable;
use dom::promise::Promise;
use dom::request::Request;
use dom::response::Response;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::jsapi::JSAutoCompartment;
use net_traits::{FetchResponseListener, Metadata, NetworkError};
use net_traits::CoreResourceMsg::Fetch as NetTraitsFetch;
use net_traits::request::{Request as NetTraitsRequest, RequestInit as NetTraitsRequestInit};
use network_listener::{NetworkListener, PreInvoke};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use url::Url;

#[allow(unrooted_must_root)]
struct FetchContext {
    fetch_promise: Option<TrustedPromise>,
    response_object: Trusted<Response>,
}

fn from_referer_to_referer_url(request: &NetTraitsRequest) -> Option<Url> {
    let referer = request.referrer.borrow();
    match referer.to_url() {
        Some(url) => Some(url.clone()),
        None => None,
    }
}

fn request_init_from_request(request: NetTraitsRequest) -> NetTraitsRequestInit {
    NetTraitsRequestInit {
        method: request.method.borrow().clone(),
        url: request.url(),
        headers: request.headers.borrow().clone(),
        unsafe_request: request.unsafe_request,
        same_origin_data: request.same_origin_data.get(),
        body: request.body.borrow().clone(),
        destination: request.destination,
        synchronous: request.synchronous,
        mode: request.mode,
        use_cors_preflight: request.use_cors_preflight,
        credentials_mode: request.credentials_mode,
        use_url_credentials: request.use_url_credentials,
        // TODO: NetTraitsRequestInit and NetTraitsRequest have different "origin"
        // ... NetTraitsRequestInit.origin: Url
        // ... NetTraitsRequest.origin: RefCell<Origin>
        origin: request.url(),
        referrer_url: from_referer_to_referer_url(&request),
        referrer_policy: request.referrer_policy.get(),
        pipeline_id: request.pipeline_id.get(),
    }
}

// https://fetch.spec.whatwg.org/#fetch-method
#[allow(unrooted_must_root)]
pub fn Fetch(global: GlobalRef, input: RequestOrUSVString, init: &RequestInit) -> Fallible<Rc<Promise>> {
    let core_resource_thread = global.core_resource_thread();

    // Step 1
    let promise = Promise::new(global);
    let response = Response::new(global);

    // Step 3
    response.Headers().set_guard_immutable();

    // Step 2
    let request = match Request::Constructor(global, input, init) {
        Err(e) => {
            promise.maybe_reject_error(promise.global().r().get_cx(),
                                       e);
            return Ok(promise);
        }
        Ok(r) => r.get_request(),
    };

    let request_init = request_init_from_request(request);

    // Step 4
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let fetch_context = Arc::new(Mutex::new(FetchContext {
        fetch_promise: Some(TrustedPromise::new(promise.clone())),
        response_object: Trusted::new(response.r()),
    }));
    let listener = NetworkListener {
        context: fetch_context.clone(),
        // TODO: double check how to get script_chan
        script_chan: global.networking_task_source(),
        wrapper: None,
    };

    ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
        listener.notify_fetch(message.to().unwrap());
    });
    core_resource_thread.send(NetTraitsFetch(request_init, action_sender)).unwrap();

    Ok(promise)
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
    fn process_response(&mut self, metadata: Result<Metadata, NetworkError>) {
        let promise = match self.fetch_promise.take() {
            Some(p) => p.root(),
            None => {
                return;
            }
        };

        // JSAutoCompartment needs to be manually made. Otherwise,
        // Servo will crash.
        let promise_cx = promise.global().r().get_cx();
        let _ac = JSAutoCompartment::new(promise_cx, promise.reflector().get_jsobject().get());
        match metadata {
            // Step 4.1
            Err(_) => {
                promise.maybe_reject_error(
                    promise.global().r().get_cx(),
                    Error::Type("Network error occurred".to_string()));
                self.fetch_promise = Some(TrustedPromise::new(promise));
                return;
            },
            // Step 4.2
            Ok(meta) => {
                self.response_object.root().set_headers(meta.headers);
                self.response_object.root().set_raw_status(meta.status);
                self.response_object.root().set_final_url(meta.final_url);
            },
        }
        // Step 4.3
        promise.maybe_resolve_native(
            promise_cx,
            &self.response_object.root());
        self.fetch_promise = Some(TrustedPromise::new(promise));
    }

    fn process_response_chunk(&mut self, mut chunk: Vec<u8>) {
        // TODO when body is implemented
        // ... this will append the chunk to Response's body.
    }

    fn process_response_eof(&mut self, response: Result<(), NetworkError>) {
        // TODO
        // ... trailerObject is not supported in Servo yet.
    }
}
