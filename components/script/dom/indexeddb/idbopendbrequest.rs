/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::GenericSend;
use dom_struct::dom_struct;
use js::jsval::UndefinedValue;
use js::rust::HandleValue;
use profile_traits::generic_callback::GenericCallback;
use script_bindings::conversions::SafeToJSValConvertible;
use storage_traits::indexeddb::{
    BackendError, BackendResult, IndexedDBThreadMsg, OpenDatabaseResult, SyncOperation,
};
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::IDBOpenDBRequestBinding::IDBOpenDBRequestMethods;
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::IDBTransactionMode;
use crate::dom::bindings::error::{Error, ErrorToJsval};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb::idbdatabase::IDBDatabase;
use crate::dom::indexeddb::idbfactory::DBName;
use crate::dom::indexeddb::idbrequest::IDBRequest;
use crate::dom::indexeddb::idbtransaction::IDBTransaction;
use crate::dom::indexeddb::idbversionchangeevent::IDBVersionChangeEvent;
use crate::indexeddb::map_backend_error_to_dom_error;
use crate::realms::enter_realm;
use crate::script_runtime::CanGc;

#[derive(Clone)]
struct OpenRequestListener {
    open_request: Trusted<IDBOpenDBRequest>,
}

impl OpenRequestListener {
    /// <https://w3c.github.io/IndexedDB/#open-a-database-connection>
    /// The steps that continue on the script-thread.
    fn handle_open_db(&self, name: String, response: OpenDatabaseResult, can_gc: CanGc) {
        let request = self.open_request.root();
        let global = request.global();
        let idb_factory = global.get_indexeddb();
        let name = DBName(name);
        match response {
            OpenDatabaseResult::Connection { version, upgraded } => {
                // Step 2.2: Otherwise,
                // set request’s result to result,
                // set request’s done flag,
                // and fire an event named success at request.
                let connection = idb_factory.get_connection(&name).unwrap_or_else(|| {
                    if upgraded {
                        unreachable!("A connection should exist for the upgraded db.");
                    }
                    let connection = IDBDatabase::new(
                        &global,
                        DOMString::from_string(name.0.clone()),
                        version,
                        can_gc,
                    );
                    idb_factory.note_connection(name.clone(), &connection);
                    connection
                });
                request.dispatch_success(&connection);
            },
            OpenDatabaseResult::Upgrade {
                version,
                old_version,
                transaction,
                created_db: _,
            } => {
                // TODO: link with backend connection concept.
                let connection = idb_factory.get_connection(&name).unwrap_or_else(|| {
                    let connection = IDBDatabase::new(
                        &global,
                        DOMString::from_string(name.0.clone()),
                        version,
                        can_gc,
                    );
                    idb_factory.note_connection(name.clone(), &connection);
                    connection
                });
                request.upgrade_db_version(&connection, old_version, version, transaction, can_gc);
            },
            OpenDatabaseResult::VersionError => {
                // Step 2.1 If result is an error, see dispatch_error().
                self.dispatch_error(Error::Version(None), can_gc);
            },
            OpenDatabaseResult::AbortError => {
                // Step 2.1 If result is an error, see dispatch_error().
                self.dispatch_error(Error::Abort(None), can_gc);
            },
        }
    }

    fn handle_backend_error(&self, backend_error: BackendError, can_gc: CanGc) {
        self.dispatch_error(map_backend_error_to_dom_error(backend_error), can_gc);
    }

    // Step 2.1 If result is an error,
    // set request’s result to undefined,
    // set request’s error to result,
    // set request’s done flag,
    // and fire an event named error at request
    // with its bubbles and cancelable attributes initialized to true.
    fn dispatch_error(&self, dom_exception: Error, can_gc: CanGc) {
        let request = self.open_request.root();
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

    /// The contionuation of the parallel steps of
    /// <https://www.w3.org/TR/IndexedDB/#dom-idbfactory-deletedatabase>
    fn handle_delete_db(&self, result: BackendResult<u64>, can_gc: CanGc) {
        // Step 4.1: Let result be the result of deleting a database, with storageKey, name, and request.
        // Note: done with the `result` argument.

        // Step 4.2: Set request’s processed flag to true.
        // TODO: implemen the flag.
        // Note: the flag may be need to be set on the backend(as well as here?).

        // Step 3: Queue a database task to run these steps:
        // Note: we are in the queued task.

        let open_request = self.open_request.root();
        let global = open_request.global();

        // Note: setting the done flag here as it is done in both branches below.
        open_request.idbrequest.set_ready_state_done();

        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut rval = UndefinedValue());

        let _ac = enter_realm(&*open_request);

        match result {
            Ok(version) => {
                // Step 4.3.2: Otherwise,
                // set request’s result to undefined,
                // set request’s done flag to true,
                // and fire a version change event named success at request with result and null.
                open_request.set_result(rval.handle());
                let event = IDBVersionChangeEvent::new(
                    &global,
                    Atom::from("success"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    version,
                    None,
                    CanGc::note(),
                );
                event.upcast::<Event>().fire(open_request.upcast(), can_gc);
            },
            Err(err) => {
                // Step 4.3.1:
                // If result is an error,
                // set request’s error to result,
                // set request’s done flag to true,
                // and fire an event named error at request
                // with its bubbles and cancelable attributes initialized to true.

                // TODO: transform backend error into jsval.
                let error = map_backend_error_to_dom_error(err);
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                error
                    .clone()
                    .to_jsval(cx, &global, rval.handle_mut(), can_gc);
                open_request.set_result(rval.handle());
                let event = Event::new(
                    &global,
                    Atom::from("error"),
                    EventBubbles::Bubbles,
                    EventCancelable::Cancelable,
                    can_gc,
                );
                event.fire(open_request.upcast(), can_gc);
            },
        }
    }
}

#[dom_struct]
pub struct IDBOpenDBRequest {
    idbrequest: IDBRequest,
}

impl IDBOpenDBRequest {
    pub fn new_inherited() -> IDBOpenDBRequest {
        IDBOpenDBRequest {
            idbrequest: IDBRequest::new_inherited(),
        }
    }

    pub fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<IDBOpenDBRequest> {
        reflect_dom_object(Box::new(IDBOpenDBRequest::new_inherited()), global, can_gc)
    }

    /// <https://w3c.github.io/IndexedDB/#upgrade-a-database>
    /// Step 10: Queue a database task to run these steps:
    /// The below are the steps in the task.
    fn upgrade_db_version(
        &self,
        connection: &IDBDatabase,
        old_version: u64,
        version: u64,
        transaction: u64,
        can_gc: CanGc,
    ) {
        let global = self.global();
        let cx = GlobalScope::get_cx();

        // Note: the transaction should link wiht one created on the backend.
        // Steps here are meant to create the corresponding webidl object.
        let transaction = IDBTransaction::new_with_id(
            &global,
            connection,
            IDBTransactionMode::Versionchange,
            &connection.object_stores(),
            transaction,
            can_gc,
        );
        transaction.set_upgrade_metadata(old_version);
        connection.set_transaction(&transaction);

        rooted!(in(*cx) let mut connection_val = UndefinedValue());
        connection.safe_to_jsval(cx, connection_val.handle_mut(), can_gc);

        // Step 10.1: Set request’s result to connection.
        self.idbrequest.set_result(connection_val.handle());

        // Step 10.2: Set request’s transaction to transaction.
        self.idbrequest.set_transaction(&transaction);

        // Step 10.3: Set request’s done flag to true.
        self.idbrequest.set_ready_state_done();

        // Step 10.4: Set transaction’s state to active.
        // TODO: message to set state of backend transaction.
        transaction.set_active_flag(true);

        // Step 10.5: Let didThrow be the result of
        // firing a version change event named upgradeneeded
        // at request with old version and version.
        let event = IDBVersionChangeEvent::new(
            &global,
            Atom::from("upgradeneeded"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            old_version,
            Some(version),
            CanGc::note(),
        );

        // TODO: use as part of step 10.6.2
        let _did_throw = event.upcast::<Event>().fire(self.upcast(), can_gc);

        // Step 10.6: If transaction’s state is active, then:
        if transaction.is_active() {
            // Step 10.6.1: Set transaction’s state to inactive.
            transaction.set_active_flag(false);

            // Step 10.6.2: If didThrow is true,
            // run abort a transaction with transaction
            // and a newly created "AbortError" DOMException.
            // TODO: implement.
        }

        let aborted = transaction.is_aborted();
        if global
            .storage_threads()
            .send(IndexedDBThreadMsg::OpenTransactionInactive {
                name: connection.get_name().to_string(),
                transaction: transaction.get_serial_number(),
                aborted,
            })
            .is_err()
        {
            error!("Failed to send OpenTransactionInactive.");
        }
    }

    pub fn set_result(&self, result: HandleValue) {
        self.idbrequest.set_result(result);
    }

    pub fn set_error(&self, error: Option<Error>, can_gc: CanGc) {
        self.idbrequest.set_error(error, can_gc);
    }

    /// <https://w3c.github.io/IndexedDB/#open-a-database-connection>
    pub fn open_database(&self, name: DOMString, version: Option<u64>) -> Result<(), ()> {
        let global = self.global();

        let response_listener = OpenRequestListener {
            open_request: Trusted::new(self),
        };

        let task_source = global
            .task_manager()
            .database_access_task_source()
            .to_sendable();
        let name = name.to_string();
        let name_copy = name.clone();
        let callback = GenericCallback::new(global.time_profiler_chan().clone(), move |message| {
            let response_listener = response_listener.clone();
            let name = name_copy.clone();
            let backend_result = match message {
                Ok(inner) => inner,
                Err(err) => Err(BackendError::DbErr(format!("{err:?}"))),
            };
            task_source.queue(task!(set_request_result_to_database: move || {
                match backend_result {
                    Ok(response) => {
                        response_listener.handle_open_db(name, response, CanGc::note())
                    }
                    Err(error) => response_listener.handle_backend_error(error, CanGc::note()),
                }
            }));
        })
        .expect("Could not create open database callback");

        let open_operation = SyncOperation::OpenDatabase(
            callback,
            global.origin().immutable().clone(),
            name.to_string(),
            version,
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

    pub fn delete_database(&self, name: String) -> Result<(), ()> {
        let global = self.global();

        let task_source = global
            .task_manager()
            .database_access_task_source()
            .to_sendable();
        let response_listener = OpenRequestListener {
            open_request: Trusted::new(self),
        };
        let callback = GenericCallback::new(global.time_profiler_chan().clone(), move |message| {
            let response_listener = response_listener.clone();
            task_source.queue(task!(request_callback: move || {
                response_listener.handle_delete_db(message.unwrap(), CanGc::note());
            }))
        })
        .expect("Could not create delete database callback");

        let delete_operation =
            SyncOperation::DeleteDatabase(callback, global.origin().immutable().clone(), name);

        if global
            .storage_threads()
            .send(IndexedDBThreadMsg::Sync(delete_operation))
            .is_err()
        {
            return Err(());
        }
        Ok(())
    }

    pub fn dispatch_success(&self, result: &IDBDatabase) {
        let global = self.global();
        let this = Trusted::new(self);
        let result = Trusted::new(result);

        global.task_manager().database_access_task_source().queue(
            task!(send_success_notification: move || {
                let this = this.root();
                let result = result.root();
                this.idbrequest.set_ready_state_done();
                let global = this.global();
                let cx = GlobalScope::get_cx();

                let _ac = enter_realm(&*result);
                rooted!(in(*cx) let mut result_val = UndefinedValue());
                result.safe_to_jsval(cx, result_val.handle_mut(), CanGc::note());
                this.set_result(result_val.handle());

                let event = Event::new(
                    &global,
                    Atom::from("success"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    CanGc::note()
                );
                event.fire(this.upcast(), CanGc::note());
            }),
        );
    }
}

impl IDBOpenDBRequestMethods<crate::DomTypeHolder> for IDBOpenDBRequest {
    // https://www.w3.org/TR/IndexedDB-2/#dom-idbopendbrequest-onblocked
    event_handler!(blocked, GetOnblocked, SetOnblocked);

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbopendbrequest-onupgradeneeded
    event_handler!(upgradeneeded, GetOnupgradeneeded, SetOnupgradeneeded);
}
