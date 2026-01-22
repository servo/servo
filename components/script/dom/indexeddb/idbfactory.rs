/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::HashSet;
use std::rc::Rc;

use base::generic_channel::GenericSend;
use dom_struct::dom_struct;
use js::jsval::UndefinedValue;
use js::rust::HandleValue;
use profile_traits::generic_callback::GenericCallback;
use script_bindings::inheritance::Castable;
use servo_url::origin::ImmutableOrigin;
use storage_traits::indexeddb::{
    BackendError, BackendResult, DatabaseInfo, IndexedDBThreadMsg, OpenDatabaseResult,
    SyncOperation,
};
use stylo_atoms::Atom;
use uuid::Uuid;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBFactoryBinding::{
    IDBDatabaseInfo, IDBFactoryMethods,
};
use crate::dom::bindings::error::{Error, ErrorToJsval, Fallible};
use crate::dom::bindings::import::base::SafeJSContext;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::HashMapTracedValues;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb::idbdatabase::IDBDatabase;
use crate::dom::indexeddb::idbopendbrequest::IDBOpenDBRequest;
use crate::dom::promise::Promise;
use crate::indexeddb::{convert_value_to_key, map_backend_error_to_dom_error};
use crate::script_runtime::CanGc;

/// A non-jstraceable string wrapper for use in `HashMapTracedValues`.
#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
pub(crate) struct DBName(pub(crate) String);

#[dom_struct]
pub struct IDBFactory {
    reflector_: Reflector,
    /// <https://www.w3.org/TR/IndexedDB-2/#connection>
    /// The connections pending #open-a-database-connection.
    pending_connections:
        DomRefCell<HashMapTracedValues<DBName, HashMapTracedValues<Uuid, Dom<IDBOpenDBRequest>>>>,
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

    /// <https://w3c.github.io/IndexedDB/#open-a-database-connection>
    /// The steps that continue on the script-thread.
    fn handle_open_db(
        &self,
        name: String,
        response: OpenDatabaseResult,
        request_id: Uuid,
        can_gc: CanGc,
    ) {
        let name = DBName(name);
        let request = {
            let mut pending = self.pending_connections.borrow_mut();
            let Some(entry) = pending.get_mut(&name) else {
                return debug_assert!(false, "There should be a pending connection for {:?}", name);
            };
            let Some(request) = entry.get_mut(&request_id) else {
                return debug_assert!(
                    false,
                    "There should be a pending connection for {:?}",
                    request_id
                );
            };
            request.as_rooted()
        };
        let global = request.global();
        let finished = match response {
            OpenDatabaseResult::Connection { version, upgraded } => {
                // Step 2.2: Otherwise,
                // set request’s result to result,
                // set request’s done flag,
                // and fire an event named success at request.
                request.dispatch_success(name.0.clone(), version, upgraded, can_gc);
                true
            },
            OpenDatabaseResult::Upgrade {
                version,
                old_version,
                transaction,
            } => {
                // TODO: link with backend connection concept.
                let connection = IDBDatabase::new(
                    &global,
                    DOMString::from_string(name.0.clone()),
                    version,
                    can_gc,
                );
                request.set_connection(&connection);
                request.upgrade_db_version(&connection, old_version, version, transaction, can_gc);
                false
            },
            OpenDatabaseResult::VersionError => {
                // Step 2.1 If result is an error, see dispatch_error().
                self.dispatch_error(name.clone(), request_id, Error::Version(None), can_gc);
                true
            },
            OpenDatabaseResult::AbortError => {
                // Step 2.1 If result is an error, see dispatch_error().
                self.dispatch_error(name.clone(), request_id, Error::Abort(None), can_gc);
                true
            },
        };
        if finished {
            self.note_end_of_open(&name, &request.get_id());
        }
    }

    fn handle_backend_error(
        &self,
        name: String,
        request_id: Uuid,
        backend_error: BackendError,
        can_gc: CanGc,
    ) {
        self.dispatch_error(
            DBName(name),
            request_id,
            map_backend_error_to_dom_error(backend_error),
            can_gc,
        );
    }

    // Step 2.1 If result is an error,
    // set request’s result to undefined,
    // set request’s error to result,
    // set request’s done flag,
    // and fire an event named error at request
    // with its bubbles and cancelable attributes initialized to true.
    fn dispatch_error(&self, name: DBName, request_id: Uuid, dom_exception: Error, can_gc: CanGc) {
        let request = {
            let mut pending = self.pending_connections.borrow_mut();
            let Some(entry) = pending.get_mut(&name) else {
                return debug_assert!(false, "There should be a pending connection for {:?}", name);
            };
            let Some(request) = entry.get_mut(&request_id) else {
                return debug_assert!(
                    false,
                    "There should be a pending connection for {:?}",
                    request_id
                );
            };
            request.as_rooted()
        };
        let global = request.global();
        request.set_result(HandleValue::undefined());
        request.set_error(Some(dom_exception), can_gc);
        let event = Event::new(
            &global,
            Atom::from("error"),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            can_gc,
        );
        event.fire(request.upcast(), can_gc);
    }

    /// <https://w3c.github.io/IndexedDB/#open-a-database-connection>
    pub fn open_database(
        &self,
        name: DOMString,
        version: Option<u64>,
        request: &IDBOpenDBRequest,
    ) -> Result<(), ()> {
        let global = self.global();
        let request_id = request.get_id();

        {
            let mut pending = self.pending_connections.borrow_mut();
            let outer = pending.entry(DBName(name.to_string())).or_default();
            outer.insert(request_id.clone(), Dom::from_ref(request));
        }

        let response_listener = Trusted::new(self);

        let task_source = global
            .task_manager()
            .database_access_task_source()
            .to_sendable();
        let name = name.to_string();
        let name_copy = name.clone();
        let callback = GenericCallback::new(global.time_profiler_chan().clone(), move |message| {
            let response_listener = response_listener.clone();
            let name = name_copy.clone();
            let request_id = request_id.clone();
            let backend_result = match message {
                Ok(inner) => inner,
                Err(err) => Err(BackendError::DbErr(format!("{err:?}"))),
            };
            task_source.queue(task!(set_request_result_to_database: move || {
                let factory = response_listener.root();
                match backend_result {
                    Ok(response) => {
                        factory.handle_open_db(name, response, request_id, CanGc::note())
                    }
                    Err(error) => factory.handle_backend_error(name, request_id, error, CanGc::note()),
                }
            }));
        })
        .expect("Could not create open database callback");

        let open_operation = SyncOperation::OpenDatabase(
            callback,
            global.origin().immutable().clone(),
            name.to_string(),
            version,
            request.get_id(),
        );

        // Note: algo continues in parallel.
        if global
            .storage_threads()
            .send(IndexedDBThreadMsg::Sync(open_operation))
            .is_err()
        {
            return Err(());
        }
        Ok(())
    }

    pub(crate) fn note_end_of_open(&self, name: &DBName, id: &Uuid) {
        let mut pending = self.pending_connections.borrow_mut();
        let empty = {
            let Some(entry) = pending.get_mut(name) else {
                return debug_assert!(false, "There should be a pending connection for {:?}", name);
            };
            entry.remove(id);
            entry.is_empty()
        };
        if empty {
            pending.remove(name);
        }
    }

    pub(crate) fn abort_pending_upgrades(&self) {
        let global = self.global();

        // Note: pending connections removed in `handle_open_db`.
        let pending = self.pending_connections.borrow();
        let pending_upgrades = pending
            .iter()
            .map(|(key, val)| {
                let ids: HashSet<Uuid> = val.iter().map(|(k, _v)| *k).collect();
                (key.0.clone(), ids)
            })
            .collect();
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
        if self.open_database(name, version, &request).is_err() {
            return Err(Error::Operation(None));
        }

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
