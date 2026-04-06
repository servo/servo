/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use dom_struct::dom_struct;
use servo_base::generic_channel::{GenericCallback, GenericSend};
use storage_traits::client_storage::ClientStorageThreadMessage;

use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::{
    PermissionName, PermissionState,
};
use crate::dom::bindings::codegen::Bindings::StorageManagerBinding::{
    StorageEstimate, StorageManagerMethods,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::permissions::request_permission_to_use;
use crate::dom::promise::Promise;
use crate::realms::InRealm;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct StorageManager {
    reflector_: Reflector,
}

impl StorageManager {
    fn new_inherited() -> StorageManager {
        StorageManager {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<StorageManager> {
        reflect_dom_object(Box::new(StorageManager::new_inherited()), global, can_gc)
    }

    fn origin_cannot_obtain_local_storage_shelf(&self) -> bool {
        !self.global().origin().is_tuple()
    }

    fn reject_with_type_error(
        promise_slot: &Arc<Mutex<Option<TrustedPromise>>>,
        error: Error,
        can_gc: CanGc,
    ) {
        if let Some(trusted_promise) = promise_slot.lock().expect("poisoned").take() {
            trusted_promise.root().reject_error(error, can_gc);
        }
    }

    fn type_error_from_string(message: String) -> Error {
        let message = std::ffi::CString::new(message)
            .unwrap_or_else(|_| c"Storage operation failed".to_owned());
        Error::Type(message)
    }

    fn resolve_boolean_task(
        promise_slot: Arc<Mutex<Option<TrustedPromise>>>,
        result: Result<bool, String>,
    ) -> impl crate::task::TaskOnce {
        task!(storage_manager_boolean_response: move |cx| {
            let Some(trusted_promise) = promise_slot.lock().expect("poisoned").take() else {
                error!("StorageManager callback called twice.");
                return;
            };

            let promise = trusted_promise.root();
            match result {
                Ok(value) => promise.resolve_native(&value, CanGc::from_cx(cx)),
                Err(message) => promise.reject_error(
                    StorageManager::type_error_from_string(message),
                    CanGc::from_cx(cx),
                ),
            }
        })
    }

    fn resolve_estimate_task(
        promise_slot: Arc<Mutex<Option<TrustedPromise>>>,
        result: Result<(u64, u64), String>,
    ) -> impl crate::task::TaskOnce {
        task!(storage_manager_estimate_response: move |cx| {
            let Some(trusted_promise) = promise_slot.lock().expect("poisoned").take() else {
                error!("StorageManager callback called twice.");
                return;
            };

            let promise = trusted_promise.root();
            match result {
                Ok((usage, quota)) => {
                    let mut estimate = StorageEstimate::empty();
                    estimate.usage = Some(usage);
                    estimate.quota = Some(quota);
                    promise.resolve_native(&estimate, CanGc::from_cx(cx));
                },
                Err(message) => {
                    promise.reject_error(
                        StorageManager::type_error_from_string(message),
                        CanGc::from_cx(cx),
                    );
                },
            }
        })
    }
}

impl StorageManagerMethods<crate::DomTypeHolder> for StorageManager {
    /// <https://storage.spec.whatwg.org/#dom-storagemanager-persisted>
    fn Persisted(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        // Step 1. Let promise be a new promise.
        let promise = Promise::new_in_current_realm(comp, can_gc);
        // Step 2. Let global be this’s relevant global object.
        let global = self.global();

        // Step 3. Let shelf be the result of running obtain a local storage shelf with this’s relevant
        // settings object.
        // Step 4. If shelf is failure, then reject promise with a TypeError.
        if self.origin_cannot_obtain_local_storage_shelf() {
            promise.reject_error(
                Error::Type(c"Storage is unavailable for opaque origins".to_owned()),
                can_gc,
            );
            return promise;
        }

        // Step 5. Otherwise, run these steps in parallel:
        // Step 5.1. Let persisted be true if shelf’s bucket map["default"]'s mode is "persistent";
        // otherwise false.
        // It will be false when there’s an internal error.
        // Step 5.2. Queue a storage task with global to resolve promise with persisted.
        let promise_slot = Arc::new(Mutex::new(Some(TrustedPromise::new(promise.clone()))));
        let callback_promise_slot = promise_slot.clone();
        let task_source = global
            .task_manager()
            .database_access_task_source()
            .to_sendable();
        let callback = GenericCallback::new(move |message| {
            let result = message.unwrap_or_else(|error| Err(error.to_string()));
            task_source.queue(StorageManager::resolve_boolean_task(
                callback_promise_slot.clone(),
                result,
            ));
        })
        .expect("Could not create StorageManager persisted callback");

        if global
            .storage_threads()
            .send(ClientStorageThreadMessage::Persisted {
                origin: global.origin().immutable().clone(),
                sender: callback,
            })
            .is_err()
        {
            StorageManager::reject_with_type_error(
                &promise_slot,
                Error::Type(c"Failed to queue storage task".to_owned()),
                can_gc,
            );
        }

        // Step 6. Return promise.
        promise
    }

    /// <https://storage.spec.whatwg.org/#dom-storagemanager-persist>
    fn Persist(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        // Step 1. Let promise be a new promise.
        let promise = Promise::new_in_current_realm(comp, can_gc);
        // Step 2. Let global be this’s relevant global object.
        let global = self.global();

        // Step 3. Let shelf be the result of running obtain a local storage shelf with this’s relevant
        // settings object.
        // Step 4. If shelf is failure, then reject promise with a TypeError.
        if self.origin_cannot_obtain_local_storage_shelf() {
            promise.reject_error(
                Error::Type(c"Storage is unavailable for opaque origins".to_owned()),
                can_gc,
            );
            return promise;
        }

        // Step 5. Otherwise, run these steps in parallel:
        let promise_slot = Arc::new(Mutex::new(Some(TrustedPromise::new(promise.clone()))));
        let response_promise_slot = promise_slot.clone();
        let request_promise_slot = promise_slot.clone();
        let response_task_source = global
            .task_manager()
            .database_access_task_source()
            .to_sendable();
        let request_task_source = global.task_manager().database_access_task_source();
        let trusted_manager = Trusted::new(self);

        request_task_source.queue(task!(storage_manager_persist_request: move |cx| {
            let manager = trusted_manager.root();

            // Step 5.1. Let permission be the result of requesting permission to use
            // "persistent-storage".
            let permission = request_permission_to_use(
                PermissionName::Persistent_storage,
                &manager.global(),
            );

            // Step 5.2. Let bucket be shelf’s bucket map["default"].
            // Step 5.3. Let persisted be true if bucket’s mode is "persistent"; otherwise false.
            // It will be false when there’s an internal error.
            // Step 5.4. If persisted is false and permission is "granted", then:
            // Step 5.4.1. Set bucket’s mode to "persistent".
            // Step 5.4.2. If there was no internal error, then set persisted to true.
            // Step 5.5. Queue a storage task with global to resolve promise with persisted.
            let callback = GenericCallback::new(move |message| {
                let result = message.unwrap_or_else(|error| Err(error.to_string()));
                response_task_source.queue(StorageManager::resolve_boolean_task(
                    response_promise_slot.clone(),
                    result,
                ));
            })
            .expect("Could not create StorageManager persist callback");

            if manager
                .global()
                .storage_threads()
                .send(ClientStorageThreadMessage::Persist {
                    origin: manager.global().origin().immutable().clone(),
                    permission_granted: permission == PermissionState::Granted,
                    sender: callback,
                })
                .is_err()
            {
                StorageManager::reject_with_type_error(
                    &request_promise_slot,
                    Error::Type(c"Failed to queue storage task".to_owned()),
                    CanGc::from_cx(cx),
                );
            }
        }));

        // Step 6. Return promise.
        promise
    }

    /// <https://storage.spec.whatwg.org/#dom-storagemanager-estimate>
    fn Estimate(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        // Step 1. Let promise be a new promise.
        let promise = Promise::new_in_current_realm(comp, can_gc);
        // Step 2. Let global be this’s relevant global object.
        let global = self.global();

        // Step 3. Let shelf be the result of running obtain a local storage shelf with this’s relevant
        // settings object.
        // Step 4. If shelf is failure, then reject promise with a TypeError.
        if self.origin_cannot_obtain_local_storage_shelf() {
            promise.reject_error(
                Error::Type(c"Storage is unavailable for opaque origins".to_owned()),
                can_gc,
            );
            return promise;
        }

        // Step 5. Otherwise, run these steps in parallel:
        // Step 5.1. Let usage be storage usage for shelf.
        // Step 5.2. Let quota be storage quota for shelf.
        // Step 5.3. Let dictionary be a new StorageEstimate dictionary whose usage member is usage and quota
        // member is quota.
        // Step 5.4. If there was an internal error while obtaining usage and quota, then queue a storage
        // task with global to reject promise with a TypeError.
        // Step 5.5. Otherwise, queue a storage task with global to resolve promise with dictionary.
        let promise_slot = Arc::new(Mutex::new(Some(TrustedPromise::new(promise.clone()))));
        let callback_promise_slot = promise_slot.clone();
        let task_source = global
            .task_manager()
            .database_access_task_source()
            .to_sendable();
        let callback = GenericCallback::new(move |message| {
            let result = message.unwrap_or_else(|error| Err(error.to_string()));
            task_source.queue(StorageManager::resolve_estimate_task(
                callback_promise_slot.clone(),
                result,
            ));
        })
        .expect("Could not create StorageManager estimate callback");

        if global
            .storage_threads()
            .send(ClientStorageThreadMessage::Estimate {
                origin: global.origin().immutable().clone(),
                sender: callback,
            })
            .is_err()
        {
            StorageManager::reject_with_type_error(
                &promise_slot,
                Error::Type(c"Failed to queue storage task".to_owned()),
                can_gc,
            );
        }

        // Step 6. Return promise.
        promise
    }
}
