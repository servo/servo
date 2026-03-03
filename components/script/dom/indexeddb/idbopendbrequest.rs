/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::GenericSend;
use dom_struct::dom_struct;
use js::jsval::UndefinedValue;
use js::rust::HandleValue;
use profile_traits::generic_callback::GenericCallback;
use script_bindings::conversions::SafeToJSValConvertible;
use storage_traits::indexeddb::{BackendResult, IndexedDBThreadMsg, SyncOperation};
use stylo_atoms::Atom;
use uuid::Uuid;

use crate::dom::bindings::codegen::Bindings::IDBOpenDBRequestBinding::IDBOpenDBRequestMethods;
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::IDBTransactionMode;
use crate::dom::bindings::error::{Error, ErrorToJsval};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb::idbdatabase::IDBDatabase;
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
    /// The continuation of the parallel steps of
    /// <https://www.w3.org/TR/IndexedDB/#dom-idbfactory-deletedatabase>
    fn handle_delete_db(&self, result: BackendResult<u64>, can_gc: CanGc) {
        // Step 4.1: Let result be the result of deleting a database, with storageKey, name, and request.
        // Note: done with the `result` argument.

        // Step 4.2: Set request’s processed flag to true.
        // The backend tracks this flag for connection queue processing.

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
                let _ = IDBVersionChangeEvent::fire_version_change_event(
                    &global,
                    open_request.upcast(),
                    Atom::from("success"),
                    version,
                    None,
                    can_gc,
                );
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
    pending_connection: MutNullableDom<IDBDatabase>,

    /// The id used both for the request and the related connection.
    #[no_trace]
    id: Uuid,
}

impl IDBOpenDBRequest {
    pub fn new_inherited() -> IDBOpenDBRequest {
        IDBOpenDBRequest {
            idbrequest: IDBRequest::new_inherited(),
            pending_connection: Default::default(),
            id: Uuid::new_v4(),
        }
    }

    pub fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<IDBOpenDBRequest> {
        reflect_dom_object(Box::new(IDBOpenDBRequest::new_inherited()), global, can_gc)
    }

    pub(crate) fn get_id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn get_or_init_connection(
        &self,
        global: &GlobalScope,
        name: String,
        version: u64,
        upgraded: bool,
        can_gc: CanGc,
    ) -> DomRoot<IDBDatabase> {
        self.pending_connection.or_init(|| {
            debug_assert!(!upgraded, "A connection should exist for the upgraded db.");
            IDBDatabase::new(
                global,
                DOMString::from_string(name.clone()),
                self.get_id(),
                version,
                can_gc,
            )
        })
    }

    /// <https://w3c.github.io/IndexedDB/#upgrade-a-database>
    /// Step 10: Queue a database task to run these steps:
    /// The below are the steps in the task.
    pub(crate) fn upgrade_db_version(
        &self,
        connection: &IDBDatabase,
        old_version: u64,
        version: u64,
        transaction: u64,
        can_gc: CanGc,
    ) {
        let global = self.global();
        let cx = GlobalScope::get_cx();

        let transaction = IDBTransaction::new_with_serial(
            &global,
            connection,
            IDBTransactionMode::Versionchange,
            &connection.object_stores(),
            transaction,
            can_gc,
        );
        transaction.set_versionchange_old_version(old_version);
        connection.set_transaction(&transaction);
        // This task runs Step 10.4 later, so keep the transaction inactive until then.
        transaction.set_active_flag(false);

        rooted!(in(*cx) let mut connection_val = UndefinedValue());
        connection.safe_to_jsval(cx, connection_val.handle_mut(), can_gc);

        // Step 10.1: Set request’s result to connection.
        self.idbrequest.set_result(connection_val.handle());

        // Step 10.2: Set request’s transaction to transaction.
        self.idbrequest.set_transaction(&transaction);

        // Step 10.3: Set request’s done flag to true.
        self.idbrequest.set_ready_state_done();

        // Step 10.4: Set transaction’s state to active.
        transaction.set_active_flag(true);

        // Step 10.5: Let didThrow be the result of firing a version change event
        // named upgradeneeded at request with old version and version.
        let did_throw = IDBVersionChangeEvent::fire_version_change_event(
            &global,
            self.upcast(),
            Atom::from("upgradeneeded"),
            old_version,
            Some(version),
            can_gc,
        );

        // Step 10.6: If transaction’s state is active, then:
        if transaction.is_active() {
            // Step 10.6.1: Set transaction’s state to inactive.
            transaction.set_active_flag(false);

            // Step 10.6.2: If didThrow is true, run abort a transaction with
            // transaction and a newly created "AbortError" DOMException.
            if did_throw {
                transaction.initiate_abort(Error::Abort(None), can_gc);
                transaction.request_backend_abort();
            } else {
                // The upgrade transaction auto-commits once inactive and quiescent.
                transaction.maybe_commit();
            }
        }
    }

    pub(crate) fn delete_database(&self, name: String) -> Result<(), ()> {
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

        let delete_operation = SyncOperation::DeleteDatabase(
            callback,
            global.origin().immutable().clone(),
            name,
            self.get_id(),
        );

        if global
            .storage_threads()
            .send(IndexedDBThreadMsg::Sync(delete_operation))
            .is_err()
        {
            return Err(());
        }
        Ok(())
    }

    pub fn set_result(&self, result: HandleValue) {
        self.idbrequest.set_result(result);
    }

    pub fn set_ready_state_done(&self) {
        self.idbrequest.set_ready_state_done();
    }

    pub fn set_error(&self, error: Option<Error>, can_gc: CanGc) {
        self.idbrequest.set_error(error, can_gc);
    }

    pub fn clear_transaction(&self) {
        self.idbrequest.clear_transaction();
    }

    pub(crate) fn clear_transaction_if_matches(&self, transaction: &IDBTransaction) -> bool {
        let matches = self
            .idbrequest
            .transaction()
            .is_some_and(|current| &*current == transaction);
        if matches {
            self.idbrequest.clear_transaction();
        }
        matches
    }

    pub fn dispatch_success(&self, name: String, version: u64, upgraded: bool, can_gc: CanGc) {
        let global = self.global();
        let result = self.get_or_init_connection(&global, name, version, upgraded, can_gc);
        self.idbrequest.set_ready_state_done();
        let cx = GlobalScope::get_cx();

        let _ac = enter_realm(&*result);
        rooted!(in(*cx) let mut result_val = UndefinedValue());
        result.safe_to_jsval(cx, result_val.handle_mut(), CanGc::note());
        self.set_result(result_val.handle());

        let event = Event::new(
            &global,
            Atom::from("success"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            CanGc::note(),
        );
        event.fire(self.upcast(), CanGc::note());
    }

    /// <https://w3c.github.io/IndexedDB/#eventdef-idbopendbrequest-blocked>
    pub fn dispatch_blocked(&self, old_version: u64, new_version: Option<u64>, can_gc: CanGc) {
        let global = self.global();
        let _ = IDBVersionChangeEvent::fire_version_change_event(
            &global,
            self.upcast(),
            Atom::from("blocked"),
            old_version,
            new_version,
            can_gc,
        );
    }
}

impl IDBOpenDBRequestMethods<crate::DomTypeHolder> for IDBOpenDBRequest {
    // https://www.w3.org/TR/IndexedDB-3/#dom-idbopendbrequest-onblocked
    event_handler!(blocked, GetOnblocked, SetOnblocked);

    // https://www.w3.org/TR/IndexedDB-3/#dom-idbopendbrequest-onupgradeneeded
    event_handler!(upgradeneeded, GetOnupgradeneeded, SetOnupgradeneeded);
}
