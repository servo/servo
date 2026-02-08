/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashMap;

use base::generic_channel::{GenericSend, GenericSender};
use dom_struct::dom_struct;
use profile_traits::generic_channel::channel;
use script_bindings::codegen::GenericUnionTypes::StringOrStringSequence;
use storage_traits::indexeddb::{IndexedDBIndex, IndexedDBThreadMsg, KeyPath, SyncOperation};
use stylo_atoms::Atom;
use uuid::Uuid;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DOMStringListBinding::DOMStringListMethods;
use crate::dom::bindings::codegen::Bindings::IDBDatabaseBinding::IDBObjectStoreParameters;
use crate::dom::bindings::codegen::Bindings::IDBObjectStoreBinding::IDBIndexParameters;
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::{
    IDBTransactionMethods, IDBTransactionMode,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::domexception::DOMException;
use crate::dom::domstringlist::DOMStringList;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb::idbdatabase::IDBDatabase;
use crate::dom::indexeddb::idbobjectstore::IDBObjectStore;
use crate::dom::indexeddb::idbrequest::IDBRequest;
use crate::script_runtime::CanGc;

#[dom_struct]
pub struct IDBTransaction {
    eventtarget: EventTarget,
    object_store_names: Dom<DOMStringList>,
    mode: IDBTransactionMode,
    db: Dom<IDBDatabase>,
    error: MutNullableDom<DOMException>,

    store_handles: DomRefCell<HashMap<String, Dom<IDBObjectStore>>>,
    // https://www.w3.org/TR/IndexedDB-2/#transaction-request-list
    requests: DomRefCell<Vec<Dom<IDBRequest>>>,
    // https://www.w3.org/TR/IndexedDB-2/#transaction-active-flag
    active: Cell<bool>,
    // https://www.w3.org/TR/IndexedDB-2/#transaction-finish
    finished: Cell<bool>,
    // Tracks how many IDBRequest instances are still pending for this
    // transaction. The value is incremented when a request is added to the
    // transaction’s request list and decremented once the request has
    // finished.
    pending_request_count: Cell<usize>,

    // An unique identifier, used to commit and revert this transaction
    // FIXME:(rasviitanen) Replace this with a channel
    serial_number: u64,

    /// The id of the associated open request, if any.
    #[no_trace]
    open_request_id: Option<Uuid>,
}

impl IDBTransaction {
    fn new_inherited(
        connection: &IDBDatabase,
        mode: IDBTransactionMode,
        scope: &DOMStringList,
        serial_number: u64,
        open_request_id: Option<Uuid>,
    ) -> IDBTransaction {
        IDBTransaction {
            eventtarget: EventTarget::new_inherited(),
            object_store_names: Dom::from_ref(scope),
            mode,
            db: Dom::from_ref(connection),
            error: Default::default(),

            store_handles: Default::default(),
            requests: Default::default(),
            active: Cell::new(true),
            finished: Cell::new(false),
            pending_request_count: Cell::new(0),
            serial_number,
            open_request_id,
        }
    }

    /// Does a blocking call to get an id from the backend.
    /// TODO: remove in favor of something like `new_with_id` below.
    pub fn new(
        global: &GlobalScope,
        connection: &IDBDatabase,
        mode: IDBTransactionMode,
        scope: &DOMStringList,
        can_gc: CanGc,
    ) -> DomRoot<IDBTransaction> {
        let serial_number = IDBTransaction::register_new(global, connection.get_name());
        reflect_dom_object(
            Box::new(IDBTransaction::new_inherited(
                connection,
                mode,
                scope,
                serial_number,
                None,
            )),
            global,
            can_gc,
        )
    }

    /// Create a new WebIDL object,
    /// based on an existign transaction on the backend.
    /// The two are linked via the `transaction_id`.
    pub(crate) fn new_with_id(
        global: &GlobalScope,
        connection: &IDBDatabase,
        mode: IDBTransactionMode,
        scope: &DOMStringList,
        transaction_id: u64,
        open_request_id: Option<Uuid>,
        can_gc: CanGc,
    ) -> DomRoot<IDBTransaction> {
        reflect_dom_object(
            Box::new(IDBTransaction::new_inherited(
                connection,
                mode,
                scope,
                transaction_id,
                open_request_id,
            )),
            global,
            can_gc,
        )
    }

    // Registers a new transaction in the idb thread, and gets an unique serial number in return.
    // The serial number is used when placing requests against a transaction
    // and allows us to commit/abort transactions running in our idb thread.
    // FIXME:(rasviitanen) We could probably replace this with a channel instead,
    // and queue requests directly to that channel.
    fn register_new(global: &GlobalScope, db_name: DOMString) -> u64 {
        let (sender, receiver) = channel(global.time_profiler_chan().clone()).unwrap();

        global
            .storage_threads()
            .send(IndexedDBThreadMsg::Sync(SyncOperation::RegisterNewTxn(
                sender,
                global.origin().immutable().clone(),
                db_name.to_string(),
            )))
            .unwrap();

        receiver.recv().unwrap()
    }

    pub fn set_active_flag(&self, status: bool) {
        self.active.set(status);
        // When the transaction becomes inactive and no requests are pending,
        // it can transition to the finished state.
        if !status && self.pending_request_count.get() == 0 && !self.finished.get() {
            self.finished.set(true);
            self.dispatch_complete();
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.get()
    }

    pub fn is_finished(&self) -> bool {
        self.finished.get()
    }

    pub fn get_mode(&self) -> IDBTransactionMode {
        self.mode
    }

    pub fn get_db_name(&self) -> DOMString {
        self.db.get_name()
    }

    pub fn get_serial_number(&self) -> u64 {
        self.serial_number
    }

    pub fn add_request(&self, request: &IDBRequest) {
        self.requests.borrow_mut().push(Dom::from_ref(request));
        // Increase the number of outstanding requests so that we can detect when
        // the transaction is allowed to finish.
        self.pending_request_count
            .set(self.pending_request_count.get() + 1);
    }

    /// Must be called by an `IDBRequest` when it finishes (either success or
    /// error). When the last pending request has completed and the transaction
    /// is no longer active, the `"complete"` event is dispatched and any
    /// associated `IDBOpenDBRequest` `"success"` event is fired afterwards.
    pub fn request_finished(&self) {
        if self.pending_request_count.get() == 0 {
            return;
        }
        let remaining = self.pending_request_count.get() - 1;
        self.pending_request_count.set(remaining);

        if remaining == 0 && !self.active.get() && !self.finished.get() {
            self.finished.set(true);
            self.dispatch_complete();
        }
    }

    fn dispatch_complete(&self) {
        let global = self.global();
        let this = Trusted::new(self);
        global.task_manager().database_access_task_source().queue(
            task!(send_complete_notification: move || {
                let this = this.root();
                let global = this.global();
                let event = Event::new(
                    &global,
                    Atom::from("complete"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    CanGc::note()
                );
                event.fire(this.upcast(), CanGc::note());
            }),
        );
    }

    fn get_idb_thread(&self) -> GenericSender<IndexedDBThreadMsg> {
        self.global().storage_threads().sender()
    }

    fn object_store_parameters(
        &self,
        object_store_name: &DOMString,
    ) -> Option<(IDBObjectStoreParameters, Vec<IndexedDBIndex>)> {
        let global = self.global();
        let idb_sender = global.storage_threads().sender();
        let (sender, receiver) =
            channel(global.time_profiler_chan().clone()).expect("failed to create channel");

        let origin = global.origin().immutable().clone();
        let db_name = self.db.get_name().to_string();
        let object_store_name = object_store_name.to_string();

        let operation = SyncOperation::GetObjectStore(
            sender,
            origin.clone(),
            db_name.clone(),
            object_store_name.clone(),
        );

        let _ = idb_sender.send(IndexedDBThreadMsg::Sync(operation));

        // First unwrap for ipc
        // Second unwrap will never happen unless this db gets manually deleted somehow
        let object_store = receiver.recv().ok()?.ok()?;

        // First unwrap for ipc
        // Second unwrap will never happen unless this db gets manually deleted somehow
        let key_path = object_store.key_path.map(|key_path| match key_path {
            KeyPath::String(s) => StringOrStringSequence::String(DOMString::from_string(s)),
            KeyPath::Sequence(seq) => StringOrStringSequence::StringSequence(
                seq.into_iter().map(DOMString::from_string).collect(),
            ),
        });
        Some((
            IDBObjectStoreParameters {
                autoIncrement: object_store.has_key_generator,
                keyPath: key_path,
            },
            object_store.indexes,
        ))
    }
}

impl IDBTransactionMethods<crate::DomTypeHolder> for IDBTransaction {
    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-db>
    fn Db(&self) -> DomRoot<IDBDatabase> {
        DomRoot::from_ref(&*self.db)
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-objectstore>
    fn ObjectStore(&self, name: DOMString, can_gc: CanGc) -> Fallible<DomRoot<IDBObjectStore>> {
        // Step 1: If transaction has finished, throw an "InvalidStateError" DOMException.
        if self.finished.get() {
            return Err(Error::InvalidState(None));
        }

        // Step 2: Check that the object store exists
        if !self.object_store_names.Contains(name.clone()) {
            return Err(Error::NotFound(None));
        }

        // Step 3: Each call to this method on the same
        // IDBTransaction instance with the same name
        // returns the same IDBObjectStore instance.
        if let Some(store) = self.store_handles.borrow().get(&*name.str()) {
            return Ok(DomRoot::from_ref(store));
        }

        let parameters = self.object_store_parameters(&name);
        let store = IDBObjectStore::new(
            &self.global(),
            self.db.get_name(),
            name.clone(),
            parameters.as_ref().map(|(params, _)| params),
            can_gc,
            self,
        );
        if let Some(indexes) = parameters.map(|(_, indexes)| indexes) {
            for index in indexes {
                store.add_index(
                    DOMString::from_string(index.name),
                    &IDBIndexParameters {
                        multiEntry: index.multi_entry,
                        unique: index.unique,
                    },
                    index.key_path.into(),
                    can_gc,
                );
            }
        }
        self.store_handles
            .borrow_mut()
            .insert(name.to_string(), Dom::from_ref(&*store));
        Ok(store)
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#commit-transaction>
    fn Commit(&self) -> Fallible<()> {
        // Step 1
        let (sender, receiver) = channel(self.global().time_profiler_chan().clone()).unwrap();
        let start_operation = SyncOperation::Commit(
            sender,
            self.global().origin().immutable().clone(),
            self.db.get_name().to_string(),
            self.serial_number,
        );

        self.get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(start_operation))
            .unwrap();

        let result = receiver.recv().unwrap();

        // Step 2
        if let Err(_result) = result {
            // FIXME:(rasviitanen) also support Unknown error
            return Err(Error::QuotaExceeded {
                quota: None,
                requested: None,
            });
        }

        // Step 3
        // FIXME:(rasviitanen) https://www.w3.org/TR/IndexedDB-2/#commit-a-transaction

        // Steps 3.1 and 3.3
        self.dispatch_complete();

        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-abort>
    fn Abort(&self) -> Fallible<()> {
        // FIXME:(rasviitanen)
        // This only sets the flags, and does not abort the transaction
        // see https://www.w3.org/TR/IndexedDB-2/#abort-a-transaction
        if self.finished.get() {
            return Err(Error::InvalidState(None));
        }

        self.active.set(false);

        if self.mode == IDBTransactionMode::Versionchange {
            let name = self.db.get_name().to_string();
            let global = self.global();
            let origin = global.origin().immutable().clone();
            let Some(id) = self.open_request_id else {
                debug_assert!(
                    false,
                    "A Versionchange transaction should have an open request id."
                );
                return Err(Error::InvalidState(None));
            };
            if global
                .storage_threads()
                .send(IndexedDBThreadMsg::Sync(
                    SyncOperation::AbortPendingUpgrade { name, id, origin },
                ))
                .is_err()
            {
                error!("Failed to send SyncOperation::AbortPendingUpgrade");
            }
        }

        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-objectstorenames>
    fn ObjectStoreNames(&self) -> DomRoot<DOMStringList> {
        self.object_store_names.as_rooted()
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-mode>
    fn Mode(&self) -> IDBTransactionMode {
        self.mode
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-mode
    // fn Durability(&self) -> IDBTransactionDurability {
    //     // FIXME:(arihant2math) Durability is not implemented at all
    //     unimplemented!();
    // }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-error>
    fn GetError(&self) -> Option<DomRoot<DOMException>> {
        self.error.get()
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-onabort
    event_handler!(abort, GetOnabort, SetOnabort);

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-oncomplete
    event_handler!(complete, GetOncomplete, SetOncomplete);

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-onerror
    event_handler!(error, GetOnerror, SetOnerror);
}
