/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::HashSet;
use std::rc::Rc;

use base::generic_channel::GenericSend;
use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsval::UndefinedValue;
use js::rust::HandleValue;
use profile_traits::generic_callback::GenericCallback;
use script_bindings::inheritance::Castable;
use servo_url::origin::ImmutableOrigin;
use storage_traits::indexeddb::{
    BackendResult, ConnectionMsg, DatabaseInfo, IndexedDBThreadMsg, SyncOperation,
};
use stylo_atoms::Atom;
use uuid::Uuid;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBFactoryBinding::{
    IDBDatabaseInfo, IDBFactoryMethods,
};
use crate::dom::bindings::error::{Error, ErrorToJsval, Fallible};
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::HashMapTracedValues;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb::idbopendbrequest::IDBOpenDBRequest;
use crate::dom::promise::Promise;
use crate::dom::types::IDBTransaction;
use crate::indexeddb::{convert_value_to_key, map_backend_error_to_dom_error};
use crate::script_runtime::CanGc;

/// A non-jstraceable string wrapper for use in `HashMapTracedValues`.
#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
pub(crate) struct DBName(pub(crate) String);

#[dom_struct]
pub struct IDBFactory {
    reflector_: Reflector,
    /// <https://www.w3.org/TR/IndexedDB-2/#connection>
    /// The connections opened through this factory.
    /// We store the open request, which contains the connection.
    /// TODO: remove when we are sure they are not needed anymore.
    connections:
        DomRefCell<HashMapTracedValues<DBName, HashMapTracedValues<Uuid, Dom<IDBOpenDBRequest>>>>,

    /// <https://www.w3.org/TR/IndexedDB-2/#transaction>
    /// Active transactions associated with this factory's global.
    indexeddb_transactions: DomRefCell<Vec<Dom<IDBTransaction>>>,

    #[no_trace]
    callback: DomRefCell<Option<GenericCallback<ConnectionMsg>>>,
}

impl IDBFactory {
    pub fn new_inherited() -> IDBFactory {
        IDBFactory {
            reflector_: Reflector::new(),
            connections: Default::default(),
            indexeddb_transactions: Default::default(),
            callback: Default::default(),
        }
    }

    pub(crate) fn register_indexeddb_transaction(&self, txn: &IDBTransaction) {
        let mut v = self.indexeddb_transactions.borrow_mut();
        if v.iter()
            .any(|entry| std::ptr::eq::<IDBTransaction>(&**entry, txn))
        {
            return;
        }
        v.push(Dom::from_ref(txn));
    }

    pub(crate) fn unregister_indexeddb_transaction(&self, txn: &IDBTransaction) {
        self.indexeddb_transactions
            .borrow_mut()
            .retain(|entry| !std::ptr::eq::<IDBTransaction>(&**entry, txn));
    }

    pub(crate) fn cleanup_indexeddb_transactions(&self) -> bool {
        // We implement the HTML-triggered deactivation effect by tracking script-created
        // transactions on the global and deactivating them at the microtask checkpoint.
        let snapshot: Vec<DomRoot<IDBTransaction>> = {
            let mut transactions = self.indexeddb_transactions.borrow_mut();
            transactions.retain(|txn| !txn.is_finished());

            transactions
                .iter()
                .map(|txn| DomRoot::from_ref(&**txn))
                .collect()
        };
        // https://html.spec.whatwg.org/multipage/#perform-a-microtask-checkpoint
        // https://w3c.github.io/IndexedDB/#cleanup-indexed-database-transactions
        // To cleanup Indexed Database transactions, run the following steps.
        // They will return true if any transactions were cleaned up, or false otherwise.
        // If there are no transactions with cleanup event loop matching the current event loop, return false.
        // For each transaction transaction with cleanup event loop matching the current event loop:
        // Set transaction’s state to inactive.
        // Clear transaction’s cleanup event loop.
        // Return true.
        let any_matching = snapshot
            .iter()
            .any(|txn| txn.cleanup_event_loop_matches_current());
        if !any_matching {
            return false;
        }
        for txn in snapshot {
            if txn.cleanup_event_loop_matches_current() {
                txn.set_active_flag(false);
                txn.clear_cleanup_event_loop();
                if txn.is_usable() {
                    txn.maybe_commit();
                }
            }
        }
        self.indexeddb_transactions
            .borrow_mut()
            .retain(|txn| !txn.is_finished());
        true
    }

    pub fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<IDBFactory> {
        reflect_dom_object(Box::new(IDBFactory::new_inherited()), global, can_gc)
    }

    /// Setup the callback to the backend service, if this hasn't been done already.
    fn get_or_setup_callback(&self) -> GenericCallback<ConnectionMsg> {
        if let Some(cb) = self.callback.borrow().as_ref() {
            return cb.clone();
        }

        let global = self.global();
        let response_listener = Trusted::new(self);

        let task_source = global
            .task_manager()
            .database_access_task_source()
            .to_sendable();
        let callback = GenericCallback::new(global.time_profiler_chan().clone(), move |message| {
            let response_listener = response_listener.clone();
            let response = match message {
                Ok(inner) => inner,
                Err(err) => return error!("Error in IndexedDB factory callback {:?}.", err),
            };
            task_source.queue(task!(set_request_result_to_database: move || {
                let factory = response_listener.root();
                factory.handle_connection_message(response, CanGc::note())
            }));
        })
        .expect("Could not create open database callback");

        *self.callback.borrow_mut() = Some(callback.clone());

        callback
    }

    fn get_request(&self, name: String, request_id: &Uuid) -> Option<DomRoot<IDBOpenDBRequest>> {
        let name = DBName(name);
        let mut pending = self.connections.borrow_mut();
        let Some(entry) = pending.get_mut(&name) else {
            debug_assert!(false, "There should be a pending connection for {:?}", name);
            return None;
        };
        let Some(request) = entry.get_mut(request_id) else {
            debug_assert!(
                false,
                "There should be a pending connection for {:?}",
                request_id
            );
            return None;
        };
        Some(request.as_rooted())
    }

    /// <https://w3c.github.io/IndexedDB/#open-a-database-connection>
    /// The steps that continue on the script-thread.
    /// This covers interacting with the current open request,
    /// as well as with other open connections preventing the request from making progress.
    fn handle_connection_message(&self, response: ConnectionMsg, can_gc: CanGc) {
        match response {
            ConnectionMsg::Connection {
                name,
                id,
                version,
                upgraded,
            } => {
                let Some(request) = self.get_request(name.clone(), &id) else {
                    return debug_assert!(
                        false,
                        "There should be a request to handle ConnectionMsg::Connection."
                    );
                };

                // Step 2.2: Otherwise,
                // set request’s result to result,
                // set request’s done flag,
                // and fire an event named success at request.
                request.dispatch_success(name, version, upgraded, can_gc);
            },
            ConnectionMsg::Upgrade {
                name,
                id,
                version,
                old_version,
                transaction,
            } => {
                let global = self.global();

                let Some(request) = self.get_request(name.clone(), &id) else {
                    return debug_assert!(
                        false,
                        "There should be a request to handle ConnectionMsg::Upgrade."
                    );
                };

                let connection =
                    request.get_or_init_connection(&global, name, version, false, can_gc);
                request.upgrade_db_version(&connection, old_version, version, transaction, can_gc);
            },
            ConnectionMsg::VersionError { name, id } => {
                // Step 2.1 If result is an error, see dispatch_error().
                self.dispatch_error(name, id, Error::Version(None), can_gc);
            },
            ConnectionMsg::AbortError { name, id } => {
                // Step 2.1 If result is an error, see dispatch_error().
                self.dispatch_error(name, id, Error::Abort(None), can_gc);
            },
            ConnectionMsg::DatabaseError { name, id, error } => {
                // Step 2.1 If result is an error, see dispatch_error().
                self.dispatch_error(name, id, map_backend_error_to_dom_error(error), can_gc);
            },
            ConnectionMsg::VersionChange {
                name,
                id,
                version,
                old_version,
            } => {
                let global = self.global();
                let Some(request) = self.get_request(name.clone(), &id) else {
                    return debug_assert!(
                        false,
                        "There should be a request to handle ConnectionMsg::VersionChange."
                    );
                };
                let connection =
                    request.get_or_init_connection(&global, name.clone(), version, false, can_gc);

                // Step 10.2: fire a version change event named versionchange at entry with db’s version and version.
                connection.dispatch_versionchange(old_version, Some(version), can_gc);

                // Step 10.3: Wait for all of the events to be fired.
                // Note: backend is at this step; sending a message to continue algo there.
                let operation = SyncOperation::NotifyEndOfVersionChange {
                    id,
                    name,
                    old_version,
                    origin: global.origin().immutable().clone(),
                };
                if global
                    .storage_threads()
                    .send(IndexedDBThreadMsg::Sync(operation))
                    .is_err()
                {
                    error!("Failed to send SyncOperation::NotifyEndOfVersionChange.");
                }
            },
            ConnectionMsg::Blocked {
                name,
                id,
                version,
                old_version,
            } => {
                let Some(request) = self.get_request(name, &id) else {
                    return debug_assert!(
                        false,
                        "There should be a request to handle ConnectionMsg::VersionChange."
                    );
                };

                // Step 10.4: fire a version change event named blocked at request with db’s version and version.
                request.dispatch_blocked(old_version, Some(version), can_gc);
            },
        }
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbfactory-open>
    /// The error dispatching part from within a task part.
    fn dispatch_error(&self, name: String, request_id: Uuid, dom_exception: Error, can_gc: CanGc) {
        let name = DBName(name);

        // Step 5.3.1: If result is an error, then:
        let request = {
            let mut pending = self.connections.borrow_mut();
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

        // Step 5.3.1.1: Set request’s result to undefined.
        request.set_result(HandleValue::undefined());

        // Step 5.3.1.2: Set request’s error to result.
        request.set_error(Some(dom_exception), can_gc);

        // Step 5.3.1.3: Set request’s done flag to true.
        // TODO.

        // Step 5.3.1.4: Fire an event named error at request
        // with its bubbles
        // and cancelable attributes initialized to true.
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
    fn open_database(
        &self,
        name: DOMString,
        version: Option<u64>,
        request: &IDBOpenDBRequest,
    ) -> Result<(), ()> {
        let global = self.global();
        let request_id = request.get_id();

        {
            let mut pending = self.connections.borrow_mut();
            let outer = pending.entry(DBName(name.to_string())).or_default();
            outer.insert(request_id, Dom::from_ref(request));
        }

        let callback = self.get_or_setup_callback();

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

    pub(crate) fn abort_pending_upgrades(&self) {
        let global = self.global();
        let pending = self.connections.borrow();
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
    fn Databases(&self, cx: &mut JSContext) -> Rc<Promise> {
        // Step 1: Let environment be this’s relevant settings object
        let global = self.global();

        // Step 2: Let storageKey be the result of running obtain a storage key given environment.
        // If failure is returned, then return a promise rejected with a "SecurityError" DOMException
        // TODO: implement storage keys.

        // Step 3: Let p be a new promise.
        let p = Promise::new(&global, CanGc::from_cx(cx));

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
            task_source.queue(task!(set_request_result_to_database: move |cx| {
                let promise = trusted_promise.root();
                match result {
                    Err(err) => {
                        let error = map_backend_error_to_dom_error(err);
                        rooted!(&in(cx) let mut rval = UndefinedValue());
                        error
                            .clone()
                            .to_jsval(cx.into(), &promise.global(), rval.handle_mut(), CanGc::from_cx(cx));
                        promise.reject_native(&rval.handle(), CanGc::from_cx(cx));
                    },
                    Ok(info_list) => {
                        let info_list: Vec<IDBDatabaseInfo> = info_list
                            .into_iter()
                            .map(|info| IDBDatabaseInfo {
                                name: Some(DOMString::from(info.name)),
                                version: Some(info.version),
                        })
                        .collect();
                        promise.resolve_native(&info_list, CanGc::from_cx(cx));
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
    fn Cmp(&self, cx: &mut JSContext, first: HandleValue, second: HandleValue) -> Fallible<i16> {
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
