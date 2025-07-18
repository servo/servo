/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use ipc_channel::router::ROUTER;
use js::jsval::UndefinedValue;
use js::rust::HandleValue;
use net_traits::IpcSend;
use net_traits::indexeddb_thread::{IndexedDBThreadMsg, SyncOperation};
use profile_traits::ipc;
use script_bindings::conversions::SafeToJSValConvertible;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::IDBOpenDBRequestBinding::IDBOpenDBRequestMethods;
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::IDBTransactionMode;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbdatabase::IDBDatabase;
use crate::dom::idbrequest::IDBRequest;
use crate::dom::idbtransaction::IDBTransaction;
use crate::dom::idbversionchangeevent::IDBVersionChangeEvent;
use crate::realms::enter_realm;
use crate::script_runtime::CanGc;

#[derive(Clone)]
struct OpenRequestListener {
    open_request: Trusted<IDBOpenDBRequest>,
}

impl OpenRequestListener {
    // https://www.w3.org/TR/IndexedDB-2/#open-a-database
    fn handle_open_db(
        &self,
        name: String,
        request_version: Option<u64>,
        db_version: u64,
        can_gc: CanGc,
    ) -> (Fallible<DomRoot<IDBDatabase>>, bool) {
        // Step 5-6
        let request_version = match request_version {
            Some(v) => v,
            None => {
                if db_version == 0 {
                    1
                } else {
                    db_version
                }
            },
        };

        // Step 7
        if request_version < db_version {
            return (Err(Error::Version), false);
        }

        // Step 8-9
        let open_request = self.open_request.root();
        let global = open_request.global();
        let connection = IDBDatabase::new(
            &global,
            DOMString::from_string(name.clone()),
            request_version,
            can_gc,
        );

        // Step 10
        if request_version > db_version {
            // FIXME:(rasviitanen) Do step 10.1-10.5
            // connection.dispatch_versionchange(db_version, Some(request_version));
            // Step 10.6
            open_request.upgrade_db_version(&connection, request_version, CanGc::note());
            // Step 11
            (Ok(connection), true)
        } else {
            // Step 11
            (Ok(connection), false)
        }
    }

    fn handle_delete_db(&self, result: Result<(), ()>, can_gc: CanGc) {
        let open_request = self.open_request.root();
        let global = open_request.global();
        open_request.idbrequest.set_ready_state_done();

        match result {
            Ok(_) => {
                let _ac = enter_realm(&*open_request);
                #[allow(unsafe_code)]
                unsafe {
                    open_request
                        .set_result(js::gc::Handle::from_raw(js::jsapi::UndefinedHandleValue));
                }

                let event = Event::new(
                    &global,
                    Atom::from("success"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    can_gc,
                );
                event.fire(open_request.upcast(), can_gc);
            },
            Err(_e) => {
                // FIXME(rasviitanen) Set the error of request to the
                // appropriate error

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

    // https://www.w3.org/TR/IndexedDB-2/#run-an-upgrade-transaction
    fn upgrade_db_version(&self, connection: &IDBDatabase, version: u64, can_gc: CanGc) {
        let global = self.global();
        // Step 2
        let transaction = IDBTransaction::new(
            &global,
            connection,
            IDBTransactionMode::Versionchange,
            &connection.object_stores(),
            can_gc,
        );

        // Step 3
        connection.set_transaction(&transaction);

        // Step 4
        transaction.set_active_flag(false);

        // Step 5-7
        let old_version = connection.version();
        transaction.upgrade_db_version(version);

        // Step 8
        let this = Trusted::new(self);
        let connection = Trusted::new(connection);
        let trusted_transaction = Trusted::new(&*transaction);
        global.task_manager().database_access_task_source().queue(
            task!(send_upgradeneeded_notification: move || {
                let this = this.root();
                let txn = trusted_transaction.root();
                let conn = connection.root();
                let global = this.global();
                let cx = GlobalScope::get_cx();

                // Step 8.1
                let _ac = enter_realm(&*conn);
                rooted!(in(*cx) let mut connection_val = UndefinedValue());
                conn.safe_to_jsval(cx, connection_val.handle_mut());
                this.idbrequest.set_result(connection_val.handle());

                // Step 8.2
                this.idbrequest.set_transaction(&txn);

                // Step 8.3
                this.idbrequest.set_ready_state_done();

                let event = IDBVersionChangeEvent::new(
                    &global,
                    Atom::from("upgradeneeded"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    old_version,
                    Some(version),
                    CanGc::note(),
                );

                // Step 8.4
                txn.set_active_flag(true);
                // Step 8.5
                let _did_throw = event.upcast::<Event>().fire(this.upcast(), CanGc::note());
                // FIXME:(rasviitanen) Handle throw (Step 8.5)
                // https://www.w3.org/TR/IndexedDB-2/#run-an-upgrade-transaction
                // Step 8.6
                txn.set_active_flag(false);

                // Implementation specific: we fire the success on db here
                // to make sure the success event occurs after the upgrade event.
                txn.wait();
                this.dispatch_success(&conn);
            }),
        );

        // Step 9: Starts and waits for the transaction to finish
        transaction.wait();
    }

    pub fn set_result(&self, result: HandleValue) {
        self.idbrequest.set_result(result);
    }

    pub fn set_error(&self, error: Error, can_gc: CanGc) {
        self.idbrequest.set_error(error, can_gc);
    }

    pub fn open_database(&self, name: DOMString, version: Option<u64>) {
        let global = self.global();

        let (sender, receiver) = ipc::channel(global.time_profiler_chan().clone()).unwrap();
        let response_listener = OpenRequestListener {
            open_request: Trusted::new(self),
        };

        let open_operation = SyncOperation::OpenDatabase(
            sender,
            global.origin().immutable().clone(),
            name.to_string(),
            version,
        );

        let task_source = global
            .task_manager()
            .database_access_task_source()
            .to_sendable();

        let trusted_request = Trusted::new(self);
        let name = name.to_string();
        ROUTER.add_typed_route(
            receiver.to_ipc_receiver(),
            Box::new(move |message| {
                let trusted_request = trusted_request.clone();
                let response_listener = response_listener.clone();
                let name = name.clone();

                task_source.queue(
                    task!(set_request_result_to_database: move || {
                        let (result, did_upgrade) =
                            response_listener.handle_open_db(name, version, message.unwrap(), CanGc::note());
                        // If an upgrade event was created, it will be responsible for
                        // dispatching the success event
                        if did_upgrade {
                            return;
                        }
                        let request = trusted_request.root();
                        let global = request.global();
                        match result {
                            Ok(db) => {
                                request.dispatch_success(&db);
                            },
                            Err(dom_exception) => {
                                request.set_result(HandleValue::undefined());
                                request.set_error(dom_exception, CanGc::note());
                                let event = Event::new(
                                    &global,
                                    Atom::from("error"),
                                    EventBubbles::Bubbles,
                                    EventCancelable::Cancelable,
                                    CanGc::note()
                                );
                                event.fire(request.upcast(), CanGc::note());
                            }
                        }
                    }),
                );
            }),
        );

        global
            .resource_threads()
            .sender()
            .send(IndexedDBThreadMsg::Sync(open_operation))
            .unwrap();
    }

    pub fn delete_database(&self, name: String) {
        let global = self.global();

        let (sender, receiver) = ipc::channel(global.time_profiler_chan().clone()).unwrap();
        let task_source = global
            .task_manager()
            .database_access_task_source()
            .to_sendable();
        let response_listener = OpenRequestListener {
            open_request: Trusted::new(self),
        };

        let delete_operation =
            SyncOperation::DeleteDatabase(sender, global.origin().immutable().clone(), name);

        ROUTER.add_typed_route(
            receiver.to_ipc_receiver(),
            Box::new(move |message| {
                let response_listener = response_listener.clone();
                task_source.queue(task!(request_callback: move || {
                    response_listener.handle_delete_db(message.unwrap(), CanGc::note());
                }));
            }),
        );

        global
            .resource_threads()
            .sender()
            .send(IndexedDBThreadMsg::Sync(delete_operation))
            .unwrap();
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
                result.safe_to_jsval(cx, result_val.handle_mut());
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
