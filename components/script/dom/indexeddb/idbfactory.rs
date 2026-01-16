/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use base::generic_channel::GenericSend;
use dom_struct::dom_struct;
use js::jsval::UndefinedValue;
use js::rust::HandleValue;
use profile_traits::generic_callback::GenericCallback;
use servo_url::origin::ImmutableOrigin;
use storage_traits::indexeddb::{BackendResult, DatabaseInfo, IndexedDBThreadMsg, SyncOperation};
use uuid::Uuid;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBFactoryBinding::{
    IDBDatabaseInfo, IDBFactoryMethods,
};
use crate::dom::bindings::error::{Error, ErrorToJsval, Fallible};
use crate::dom::bindings::import::base::SafeJSContext;
use crate::dom::bindings::refcounted::TrustedPromise;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb::idbopendbrequest::IDBOpenDBRequest;
use crate::dom::promise::Promise;
use crate::indexeddb::{convert_value_to_key, map_backend_error_to_dom_error};
use crate::script_runtime::CanGc;

/// A non-jstraceable string wrapper for use in `HashMapTracedValues`.
#[derive(Clone, Eq, Hash, MallocSizeOf, PartialEq)]
pub(crate) struct DBName(pub(crate) String);

#[dom_struct]
pub struct IDBFactory {
    reflector_: Reflector,
    /// <https://www.w3.org/TR/IndexedDB-2/#connection>
    /// The connections pending #open-a-database-connection.
    #[no_trace]
    pending_connections: DomRefCell<HashMap<DBName, HashSet<Uuid>>>,
}

impl IDBFactory {
    pub fn new_inherited() -> IDBFactory {
        IDBFactory {
            reflector_: Reflector::new(),
            pending_connections: Default::default(),
        }
    }

    pub fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<IDBFactory> {
        reflect_dom_object(Box::new(IDBFactory::new_inherited()), global, can_gc)
    }

    fn note_start_of_open(&self, name: DBName, id: Uuid) {
        let mut pending = self.pending_connections.borrow_mut();
        let entry = pending.entry(name).or_default();
        entry.insert(id);
    }

    pub(crate) fn note_end_of_open(&self, name: &DBName, id: &Uuid) {
        let mut pending = self.pending_connections.borrow_mut();
        let Some(entry) = pending.get_mut(name) else {
            return debug_assert!(
                false,
                "There should be a pending connection for {:?}",
                name.0
            );
        };
        entry.remove(id);
    }

    pub(crate) fn abort_pending_upgrades(&self) {
        let global = self.global();
        let mut pending = self.pending_connections.borrow_mut();
        let pending_upgrades = pending.drain().map(|(key, val)| (key.0, val)).collect();
        let origin = global.origin().immutable().clone();
        if global
            .storage_threads()
            .send(IndexedDBThreadMsg::Sync(
                SyncOperation::AbortPendingUpgrades {
                    pending_upgrades,
                    origin,
                },
            ))
            .is_err()
        {
            error!("Failed to send SyncOperation::AbortPendingUpgrade");
        }
    }
}

impl IDBFactoryMethods<crate::DomTypeHolder> for IDBFactory {
    /// <https://w3c.github.io/IndexedDB/#dom-idbfactory-open>
    fn Open(&self, name: DOMString, version: Option<u64>) -> Fallible<DomRoot<IDBOpenDBRequest>> {
        // Step 1: If version is 0 (zero), throw a TypeError.
        if version == Some(0) {
            return Err(Error::Type(
                "The version must be an integer >= 1".to_owned(),
            ));
        };

        // Step 2: Let origin be the origin of the global scope used to
        // access this IDBFactory.
        // TODO: update to 3.0 spec.
        // Let environment be this’s relevant settings object.
        let global = self.global();
        let origin = global.origin();

        // Step 3: if origin is an opaque origin,
        // throw a "SecurityError" DOMException and abort these steps.
        // TODO: update to 3.0 spec.
        // Let storageKey be the result of running obtain a storage key given environment.
        if let ImmutableOrigin::Opaque(_) = origin.immutable() {
            return Err(Error::Security(None));
        }

        // Step 4: Let request be a new open request.
        let request = IDBOpenDBRequest::new(&self.global(), CanGc::note());

        // Step 5: Runs in parallel
        if request.open_database(name.clone(), version).is_err() {
            return Err(Error::Operation(None));
        }

        self.note_start_of_open(DBName(name.to_string()), request.get_id());

        // Step 6: Return a new IDBOpenDBRequest object for request.
        Ok(request)
    }

    /// <https://www.w3.org/TR/IndexedDB/#dom-idbfactory-deletedatabase>
    fn DeleteDatabase(&self, name: DOMString) -> Fallible<DomRoot<IDBOpenDBRequest>> {
        // Step 1: Let environment be this’s relevant settings object.
        let global = self.global();

        // Step 2: Let storageKey be the result of running obtain a storage key given environment.
        // If failure is returned, then throw a "SecurityError" DOMException and abort these steps.
        // TODO: use a storage key.
        let origin = global.origin();

        // Legacy step 2: if origin is an opaque origin,
        // throw a "SecurityError" DOMException and abort these steps.
        // TODO: remove when a storage key is used.
        if let ImmutableOrigin::Opaque(_) = origin.immutable() {
            return Err(Error::Security(None));
        }

        // Step 3: Let request be a new open request
        let request = IDBOpenDBRequest::new(&self.global(), CanGc::note());

        // Step 4: Runs in parallel
        if request.delete_database(name.to_string()).is_err() {
            return Err(Error::Operation(None));
        }

        // Step 5: Return request
        Ok(request)
    }

    /// <https://www.w3.org/TR/IndexedDB/#dom-idbfactory-databases>
    fn Databases(&self, can_gc: CanGc) -> Rc<Promise> {
        // Step 1: Let environment be this’s relevant settings object
        let global = self.global();

        // Step 2: Let storageKey be the result of running obtain a storage key given environment.
        // If failure is returned, then return a promise rejected with a "SecurityError" DOMException
        // TODO: implement storage keys.

        // Step 3: Let p be a new promise.
        let p = Promise::new(&global, can_gc);

        // Note: the option is required to pass the promise to a task from within the generic callback,
        // see #41356
        let mut trusted_promise: Option<TrustedPromise> = Some(TrustedPromise::new(p.clone()));

        // Step 4: Run these steps in parallel:
        // Note implementing by communicating with the backend.
        let task_source = global
            .task_manager()
            .database_access_task_source()
            .to_sendable();
        let callback = GenericCallback::new(global.time_profiler_chan().clone(), move |message| {
            let result: BackendResult<Vec<DatabaseInfo>> = message.unwrap();
            let Some(trusted_promise) = trusted_promise.take() else {
                return error!("Callback for `DataBases` called twice.");
            };

            // Step 3.5: Queue a database task to resolve p with result.
            task_source.queue(task!(set_request_result_to_database: move || {
                let can_gc = CanGc::note();
                let promise = trusted_promise.root();
                match result {
                    Err(err) => {
                        let error = map_backend_error_to_dom_error(err);
                        let cx = GlobalScope::get_cx();
                        rooted!(in(*cx) let mut rval = UndefinedValue());
                        error
                            .clone()
                            .to_jsval(cx, &promise.global(), rval.handle_mut(), can_gc);
                        promise.reject_native(&rval.handle(), can_gc);
                    },
                    Ok(info_list) => {
                        let info_list: Vec<IDBDatabaseInfo> = info_list
                            .into_iter()
                            .map(|info| IDBDatabaseInfo {
                                name: Some(DOMString::from(info.name)),
                                version: Some(info.version),
                        })
                        .collect();
                        promise.resolve_native(&info_list, can_gc);
                },
            }
            }));
        })
        .expect("Could not create delete database callback");

        let get_operation =
            SyncOperation::GetDatabases(callback, global.origin().immutable().clone());
        if global
            .storage_threads()
            .send(IndexedDBThreadMsg::Sync(get_operation))
            .is_err()
        {
            error!("Failed to send SyncOperation::GetDatabases");
        }

        // Step 5: Return p.
        p
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbfactory-cmp>
    fn Cmp(&self, cx: SafeJSContext, first: HandleValue, second: HandleValue) -> Fallible<i16> {
        let first_key = convert_value_to_key(cx, first, None)?.into_result()?;
        let second_key = convert_value_to_key(cx, second, None)?.into_result()?;
        let cmp = first_key.partial_cmp(&second_key);
        if let Some(cmp) = cmp {
            match cmp {
                std::cmp::Ordering::Less => Ok(-1),
                std::cmp::Ordering::Equal => Ok(0),
                std::cmp::Ordering::Greater => Ok(1),
            }
        } else {
            Ok(i16::MAX)
        }
    }
}
