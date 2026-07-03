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
use servo_url::ImmutableOrigin;
use storage_traits::cache_storage::{CacheStorageThreadMessage, CacheStorageThreadResponse};
use storage_traits::client_storage::{StorageIdentifier, StorageProxyMap, StorageType};

use crate::dom::Promise;
use crate::dom::bindings::codegen::Bindings::CacheStorageBinding::CacheStorageMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;


/// <https://w3c.github.io/ServiceWorker/#cachestorage-interface>
#[dom_struct]
pub(crate) struct CacheStorage {
    reflector_: Reflector,

    #[no_trace]
    #[ignore_malloc_size_of = "GenericCallback"]
    callback: RefCell<Option<GenericCallback<CacheStorageThreadResponse>>>,

    // Dequeue of pending promises for backend operations.
    #[ignore_malloc_size_of = "Rc"]
    pending_promises: RefCell<VecDeque<Rc<Promise>>>,
}

impl CacheStorage {
    fn new_inherited() -> CacheStorage {
        CacheStorage {
            reflector_: Reflector::new(),
            callback: RefCell::new(None),
            pending_promises: RefCell::new(VecDeque::new()),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<CacheStorage> {
        reflect_dom_object_with_cx(Box::new(CacheStorage::new_inherited()), global, cx)
    }

    /// Setup the callback to the backend service, if this hasn't been done already.
    fn get_or_setup_callback(&self) -> GenericCallback<CacheStorageThreadResponse> {
        if let Some(cb) = self.callback.borrow().as_ref() {
            return cb.clone();
        }

        let global = self.global();
        let response_listener = Trusted::new(self);

        let task_source = global
            .task_manager()
            .database_access_task_source()
            .to_sendable();
        let callback = GenericCallback::new(move |message| {
            let response_listener = response_listener.clone();
            let response = match message {
                Ok(inner) => inner,
                Err(err) => return error!("Error in IndexedDB factory callback {:?}.", err),
            };
            // Step 5.3: Queue a database task to run these steps:
            task_source.queue(task!(set_request_result_to_database: move |cx| {
                let cache_storage = response_listener.root();
                cache_storage.handle_response(cx, response)
            }));
        })
        .expect("Could not create open database callback");

        *self.callback.borrow_mut() = Some(callback.clone());

        callback
    }

    fn handle_response(&self, cx: &mut JSContext, response: CacheStorageThreadResponse) {
        match response {
            CacheStorageThreadResponse::HasCacheResult(result) => {
                let promise = self.pending_promises.borrow_mut().pop_front();
                if let Some(promise) = promise {
                    match result {
                        Ok(has_cache) => promise.resolve_native(cx, &has_cache),
                        Err(err) => promise.reject_error(cx, Error::Operation(Some(err))),
                    }
                } else {
                    error!("No pending promise for HasCacheResult response.");
                }
            },
        }
    }
}

/// <https://w3c.github.io/ServiceWorker/#relevant-name-to-cache-map>
fn relevant_name_to_cache_map(
    global: &GlobalScope,
    origin: ImmutableOrigin,
) -> Result<StorageProxyMap, Error> {
    // The relevant name to cache map for a CacheStorage object
    // is the name to cache map associated with the result of
    // running obtain a local storage bottle map with
    // the object’s relevant settings object and "caches".
    let handle = global.storage_threads().client_storage_handle();
    let message = handle
        .obtain_a_storage_bottle_map(
            StorageType::Local,
            global.webview_id(),
            StorageIdentifier::Caches,
            origin,
        )
        .recv();
    let Ok(response) = message else {
        return Err(Error::Operation(None));
    };
    let Ok(proxy_map) = response else {
        return Err(Error::Operation(None));
    };
    Ok(proxy_map)
}

impl CacheStorageMethods<crate::DomTypeHolder> for CacheStorage {
    /// <https://w3c.github.io/ServiceWorker/#relevant-name-to-cache-map>
    fn Has(&self, cx: &mut JSContext, cache_name: DOMString) -> Rc<Promise> {
        let global = self.global();

        // Step 1: Let promise be a new promise.
        let promise = Promise::new(cx, &global);

        // Step 2: Run the following substeps in parallel:
        let callback = self.get_or_setup_callback();
        let origin = global.origin().immutable().clone();
        let proxy_map = match relevant_name_to_cache_map(&global, origin.clone()) {
            Ok(proxy_map) => proxy_map,
            Err(err) => {
                promise.reject_error(cx, err);
                return promise;
            },
        };
        if global
            .storage_threads()
            .send(CacheStorageThreadMessage::HasCache {
                cache_name: cache_name.to_string(),
                callback,
                proxy: proxy_map,
                origin,
            })
            .is_err()
        {
            promise.reject_error(cx, Error::Operation(None));
            return promise;
        }

        self.pending_promises
            .borrow_mut()
            .push_back(promise.clone());

        promise
    }
}
