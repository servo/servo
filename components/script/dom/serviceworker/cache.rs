/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::DomRoot;
use servo_base::generic_channel::{GenericCallback, GenericSend};
use servo_url::ServoUrl;
use storage_traits::cache_storage::{CacheStorageThreadMessage, CacheStorageThreadResponse};

use crate::dom::Promise;
use crate::dom::bindings::codegen::Bindings::CacheBinding::CacheMethods;
use crate::dom::bindings::codegen::GenericBindings::CacheBinding::CacheQueryOptions;
use crate::dom::bindings::codegen::UnionTypes::RequestOrUSVString;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::str::DOMString;
use crate::dom::fetch::request::Request;
use crate::dom::globalscope::GlobalScope;

/// <https://w3c.github.io/ServiceWorker/#cache>
#[dom_struct]
pub(crate) struct Cache {
    reflector_: Reflector,

    /// The name used to identify
    /// <https://w3c.github.io/ServiceWorker/#dfn-relevant-request-response-list>
    name: DOMString,

    #[no_trace]
    #[ignore_malloc_size_of = "GenericCallback"]
    callback: RefCell<Option<GenericCallback<CacheStorageThreadResponse>>>,

    // Dequeue of pending promises for backend operations.
    #[conditional_malloc_size_of]
    pending_promises: RefCell<VecDeque<Rc<Promise>>>,
}

impl Cache {
    fn new_inherited(name: DOMString) -> Cache {
        Cache {
            reflector_: Reflector::new(),
            name,
            callback: Default::default(),
            pending_promises: Default::default(),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope, name: DOMString) -> DomRoot<Cache> {
        reflect_dom_object_with_cx(Box::new(Cache::new_inherited(name)), global, cx)
    }

    /// Setup the callback to the backend service, if this hasn't been done already.
    pub(crate) fn get_or_setup_callback(&self) -> GenericCallback<CacheStorageThreadResponse> {
        if let Some(cb) = self.callback.borrow().as_ref() {
            return cb.clone();
        }

        let global = self.global();
        let response_listener = Trusted::new(self);

        let task_source = global
            .task_manager()
            .dom_manipulation_task_source()
            .to_sendable();
        let callback = GenericCallback::new(move |message| {
            let response_listener = response_listener.clone();
            let response = match message {
                Ok(inner) => Some(inner),
                Err(err) => {
                    error!("Error in Cache callback {:?}.", err);
                    None
                },
            };
            task_source.queue(task!(set_request_result_to_database: move |cx| {
                let cache = response_listener.root();
                cache.handle_response(cx, response)
            }));
        })
        .expect("Could not create Cache callback");

        *self.callback.borrow_mut() = Some(callback.clone());

        callback
    }

    fn handle_response(&self, cx: &mut JSContext, response: Option<CacheStorageThreadResponse>) {
        let response = match response {
            Some(response) => response,
            None => {
                let Some(promise) = self.pending_promises.borrow_mut().pop_front() else {
                    error!("No pending promise for Cache response.");
                    return;
                };
                promise.reject_error(
                    cx,
                    Error::Operation(Some("No response from Cache backend.".to_string())),
                );
                return;
            },
        };
        match response {
            // <https://w3c.github.io/ServiceWorker/#cache-keys>
            CacheStorageThreadResponse::KeysResult(results) => {
                // Step 5.4: Queue a task,
                // on promise’s relevant settings object’s responsible event loop using the DOM manipulation task source,
                // to perform the following steps:
                // Note: we are inside the task.

                // Step 5.4.1: Let requestList be a list.
                let mut request_list: Vec<DomRoot<Request>> = Vec::new();

                let Some(promise) = self.pending_promises.borrow_mut().pop_front() else {
                    debug_assert!(false, "No pending promise for Cache KeysResult response.");
                    return;
                };

                // Step 5.4.2: For each request of requests:
                // Step 5.4.2.1: Add a new Request object associated with request
                // and a new associated Headers object whose guard is "immutable" to requestList.
                for url in results.unwrap_or_default() {
                    let Ok(url) = ServoUrl::parse(&url) else {
                        promise.reject_error(cx, Error::Type(c"Invalid URL".to_owned()));
                        return;
                    };
                    let request = Request::new(cx, &self.global(), None, url);
                    // TODO: associate request with a header.
                    request_list.push(request);
                }

                // Step 5.4.3: Resolve promise with a frozen array created from requestList, in realm.
                promise.resolve_native(cx, &request_list);
            },
            CacheStorageThreadResponse::DeleteCacheResult(_result) => debug_assert!(
                false,
                "Unexpected DeleteCacheResult response in Cache handle_response."
            ),
            CacheStorageThreadResponse::HasCacheResult(_result) => debug_assert!(
                false,
                "Unexpected HasCacheResult response in Cache handle_response."
            ),
            CacheStorageThreadResponse::OpenCacheResult { .. } => debug_assert!(
                false,
                "Unexpected OpenCacheResult response in Cache handle_response."
            ),
        }
    }
}

impl CacheMethods<crate::DomTypeHolder> for Cache {
    /// <https://w3c.github.io/ServiceWorker/#dom-cache-keys>
    fn Keys(
        &self,
        cx: &mut JSContext,
        request: Option<RequestOrUSVString>,
        _options: &CacheQueryOptions,
    ) -> Rc<Promise> {
        // Step 1: Let r be null.
        let mut r: Option<DomRoot<Request>> = None;

        // Step 3: Let realm be this’s relevant realm.
        // Note: the global is used as the realm; step re-ordered to make it available in Step 2.
        let global = self.global();

        // Step 4: Let promise be a new promise.
        // Note: step re-ordered to make it available in Step 2.
        let promise = Promise::new(cx, &global);

        // Step 2: If the optional argument request is not omitted, then:
        if let Some(request) = request {
            // Step 2.1: If request is a Request object, then:
            if let RequestOrUSVString::Request(request) = request {
                // Step 2.1.1: Set r to request.
                r = Some(request);
            // Step 2.2: Else if request is a string, then:
            } else if let RequestOrUSVString::USVString(request_string) = request {
                // Step 2.2.1: Set r to the associated request of the result of
                // invoking the initial value of Request as
                // constructor with request as its argument.
                let Ok(url) = ServoUrl::parse(&request_string) else {
                    // If this throws an exception, return a promise rejected with that exception.
                    // Note: only the url parse can error.
                    promise.reject_error(cx, Error::Type(c"Invalid URL".to_owned()));
                    return promise;
                };
                let request = Request::new(cx, &global, None, url);
                r = Some(request);
            }
        }

        // TODO: use r in the backend.
        let _ = r;

        // Step 5: Run these substeps in parallel:
        let callback = self.get_or_setup_callback();
        if global
            .storage_threads()
            .send(CacheStorageThreadMessage::Keys {
                cache_name: self.name.to_string(),
                callback,
                origin: global.origin().immutable().clone(),
            })
            .is_err()
        {
            promise.reject_error(
                cx,
                Error::Operation(Some("Could not run the parallel steps.".to_string())),
            );
            return promise;
        }

        self.pending_promises
            .borrow_mut()
            .push_back(promise.clone());

        promise
    }
}
