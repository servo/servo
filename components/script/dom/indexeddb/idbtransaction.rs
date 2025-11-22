/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use base::IpcSend;
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use profile_traits::ipc;
use script_bindings::codegen::GenericUnionTypes::StringOrStringSequence;
use storage_traits::indexeddb_thread::{
    IndexedDBThreadMsg, IndexedDBTransaction, IndexedDBTxnMode, KeyPath, KvsOperation,
    SyncOperation, TransactionState,
};
use stylo_atoms::Atom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBDatabaseBinding::IDBObjectStoreParameters;
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
use crate::dom::indexeddb::idbopendbrequest::IDBOpenDBRequest;
use crate::dom::indexeddb::idbrequest::IDBRequest;
use crate::script_runtime::CanGc;

#[dom_struct]
pub struct IDBTransaction {
    eventtarget: EventTarget,
    db: Dom<IDBDatabase>,
    error: MutNullableDom<DOMException>,

    store_handles: DomRefCell<HashMap<String, Dom<IDBObjectStore>>>,
    // https://www.w3.org/TR/IndexedDB-2/#transaction-request-list
    requests: DomRefCell<Vec<Dom<IDBRequest>>>,
    // https://www.w3.org/TR/IndexedDB-2/#transaction-active-flag
    active: Cell<bool>,
    // Tracks how many IDBRequest instances are still pending for this
    // transaction. The value is incremented when a request is added to the
    // transaction’s request list and decremented once the request has
    // finished.
    pending_request_count: Cell<usize>,
    // When the transaction belongs to a database open request (i.e. during an
    // upgrade), the corresponding IDBOpenDBRequest is stored so we can fire its
    // "success" event after the transaction is fully finished.
    open_request: MutNullableDom<IDBOpenDBRequest>,

    // An unique identifier, used to commit and revert this transaction
    // FIXME:(rasviitanen) Replace this with a channel
    serial_number: u64,

    #[ignore_malloc_size_of = "Arcs are hard"]
    #[no_trace]
    shared_object: Arc<IndexedDBTransaction>,
    #[ignore_malloc_size_of = "Channels are hard"]
    #[no_trace]
    sender: IpcSender<KvsOperation>,
}

impl IDBTransaction {
    fn new_inherited(
        connection: &IDBDatabase,
        serial_number: u64,
        shared_object: Arc<IndexedDBTransaction>,
        sender: IpcSender<KvsOperation>,
    ) -> IDBTransaction {
        IDBTransaction {
            eventtarget: EventTarget::new_inherited(),
            db: Dom::from_ref(connection),
            error: Default::default(),

            store_handles: Default::default(),
            requests: Default::default(),
            active: Cell::new(true),
            pending_request_count: Cell::new(0),
            open_request: Default::default(),
            serial_number,
            shared_object,
            sender,
        }
    }

    pub fn new(
        global: &GlobalScope,
        connection: &IDBDatabase,
        mode: IDBTransactionMode,
        scope: &DOMStringList,
        can_gc: CanGc,
    ) -> DomRoot<IDBTransaction> {
        let shared_object = Arc::new(IndexedDBTransaction {
            mode: match mode {
                IDBTransactionMode::Readonly => IndexedDBTxnMode::Readonly,
                IDBTransactionMode::Readwrite => IndexedDBTxnMode::Readwrite,
                IDBTransactionMode::Versionchange => IndexedDBTxnMode::Versionchange,
            },
            object_stores: scope.strings().iter().map(|s| s.to_string()).collect(),
            state: Mutex::new(TransactionState::InProgress),
        });
        let (serial_number, sender) =
            IDBTransaction::register_new(global, connection.get_name(), shared_object.clone());
        reflect_dom_object(
            Box::new(IDBTransaction::new_inherited(
                connection,
                serial_number,
                shared_object,
                sender,
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
    fn register_new(
        global: &GlobalScope,
        db_name: DOMString,
        object: Arc<IndexedDBTransaction>,
    ) -> (u64, IpcSender<KvsOperation>) {
        let (sender, receiver) = ipc::channel(global.time_profiler_chan().clone()).unwrap();

        global
            .storage_threads()
            .send(IndexedDBThreadMsg::Sync(SyncOperation::RegisterNewTxn(
                sender,
                global.origin().immutable().clone(),
                db_name.to_string(),
                object,
            )))
            .unwrap();

        receiver.recv().unwrap()
    }

    // Waits for transaction to finish
    pub fn wait(&self) {
        if !matches!(
            *self.shared_object.state.lock().unwrap(),
            TransactionState::InProgress
        ) {
            return;
        }
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        if self.sender.send(KvsOperation::Wait(sender)).is_ok() {
            let _ = receiver.recv();
        }
    }

    pub fn set_active_flag(&self, status: bool) {
        self.active.set(status);
        // When the transaction becomes inactive and no requests are pending,
        // it can transition to the finished state.
        if !status &&
            self.pending_request_count.get() == 0 &&
            matches!(
                *self.shared_object.state.lock().unwrap(),
                TransactionState::InProgress
            )
        {
            let (sender, receiver) =
                ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
            if self.sender.send(KvsOperation::Commit(sender)).is_ok() {
                let _ = receiver.recv();
            }
            self.dispatch_complete();
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.get() &&
            matches!(
                *self.shared_object.state.lock().unwrap(),
                TransactionState::InProgress
            )
    }

    pub fn get_mode(&self) -> IDBTransactionMode {
        match self.shared_object.mode {
            IndexedDBTxnMode::Readonly => IDBTransactionMode::Readonly,
            IndexedDBTxnMode::Readwrite => IDBTransactionMode::Readwrite,
            IndexedDBTxnMode::Versionchange => IDBTransactionMode::Versionchange,
        }
    }

    pub fn get_db_name(&self) -> DOMString {
        self.db.get_name()
    }

    pub fn get_serial_number(&self) -> u64 {
        self.serial_number
    }

    /// Associate an `IDBOpenDBRequest` with this transaction so that its
    /// "success" event is dispatched only once the transaction has truly
    /// finished (after the "complete" event).
    pub fn set_open_request(&self, request: &IDBOpenDBRequest) {
        self.open_request.set(Some(request));
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

        if remaining == 0 &&
            !self.active.get() &&
            matches!(
                *self.shared_object.state.lock().unwrap(),
                TransactionState::InProgress
            )
        {
            let (sender, receiver) =
                ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
            if self.sender.send(KvsOperation::Commit(sender)).is_ok() {
                let _ = receiver.recv();
            }
            self.dispatch_complete();
        }
    }

    pub fn upgrade_db_version(&self, version: u64) {
        // Runs the previous request and waits for them to finish
        self.wait();
        // Queue a request to upgrade the db version
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let upgrade_version_operation = SyncOperation::UpgradeVersion(
            sender,
            self.global().origin().immutable().clone(),
            self.db.get_name().to_string(),
            self.serial_number,
            version,
        );
        self.get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(upgrade_version_operation))
            .unwrap();
        // Wait for the version to be updated
        // TODO(jdm): This returns a Result; what do we do with an error?
        let _ = receiver.recv().unwrap();
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

                // If this transaction was created as part of an IDBOpenDBRequest,
                // fire the "success" event for that
                // request after the complete event to respect spec ordering.
                if let Some(open_req) = this.open_request.get() {
                    open_req.dispatch_success(&this.db);
                    this.open_request.set(None);
                }
            }),
        );
    }

    fn get_idb_thread(&self) -> IpcSender<IndexedDBThreadMsg> {
        self.global().storage_threads().sender()
    }

    fn object_store_parameters(
        &self,
        object_store_name: &DOMString,
    ) -> Option<IDBObjectStoreParameters> {
        let global = self.global();
        let idb_sender = global.storage_threads().sender();
        let (sender, receiver) =
            ipc::channel(global.time_profiler_chan().clone()).expect("failed to create channel");

        let origin = global.origin().immutable().clone();
        let db_name = self.db.get_name().to_string();
        let object_store_name = object_store_name.to_string();

        let operation = SyncOperation::HasKeyGenerator(
            sender,
            origin.clone(),
            db_name.clone(),
            object_store_name.clone(),
        );

        let _ = idb_sender.send(IndexedDBThreadMsg::Sync(operation));

        // First unwrap for ipc
        // Second unwrap will never happen unless this db gets manually deleted somehow
        let auto_increment = receiver.recv().ok()?.ok()?;

        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).ok()?;
        let operation = SyncOperation::KeyPath(sender, origin, db_name, object_store_name);

        let _ = idb_sender.send(IndexedDBThreadMsg::Sync(operation));

        // First unwrap for ipc
        // Second unwrap will never happen unless this db gets manually deleted somehow
        let key_path = receiver.recv().unwrap().ok()?;
        let key_path = key_path.map(|key_path| match key_path {
            KeyPath::String(s) => StringOrStringSequence::String(DOMString::from_string(s)),
            KeyPath::Sequence(seq) => StringOrStringSequence::StringSequence(
                seq.into_iter().map(DOMString::from_string).collect(),
            ),
        });
        Some(IDBObjectStoreParameters {
            autoIncrement: auto_increment,
            keyPath: key_path,
        })
    }

    pub(crate) fn queue_operation(&self, operation: KvsOperation) {
        // The send can legitimately fail if the backend side of the channel
        // has already been closed (for example, when the IndexedDB manager
        // shuts down because of an earlier error).  Dropping the error here
        // avoids panicking in those cases while still allowing the higher-level
        // logic to notice that the transaction did not complete successfully.
        //
        // NOTE: callers that care about the result should check the returned
        // state of the shared transaction object instead of relying on this
        // send operation to succeed.
        let _ = self.sender.send(operation);
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
        if !matches!(
            *self.shared_object.state.lock().unwrap(),
            TransactionState::InProgress
        ) {
            return Err(Error::InvalidState(None));
        }

        // Step 2: Check that the object store exists
        if !self.shared_object.object_stores.contains(&name.to_string()) {
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
            parameters.as_ref(),
            can_gc,
            self,
        );
        self.store_handles
            .borrow_mut()
            .insert(name.to_string(), Dom::from_ref(&*store));
        Ok(store)
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#commit-transaction>
    fn Commit(&self) -> Fallible<()> {
        // Step 1
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let _ = self.sender.send(KvsOperation::Commit(sender));
        let _ = receiver.recv();

        // Step 2
        // if let Err(_result) = result {
        //     // FIXME:(rasviitanen) also support Unknown error
        //     return Err(Error::QuotaExceeded {
        //         quota: None,
        //         requested: None,
        //     });
        // }

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
        if !matches!(
            *self.shared_object.state.lock().unwrap(),
            TransactionState::InProgress
        ) {
            return Err(Error::InvalidState(None));
        }

        self.active.set(false);

        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-objectstorenames>
    fn ObjectStoreNames(&self) -> DomRoot<DOMStringList> {
        let object_store_names: Vec<_> = self
            .shared_object
            .object_stores
            .iter()
            .map(|s| DOMString::from_string(s.clone()))
            .collect();
        let dom_string_list = DOMStringList::new(&self.global(), object_store_names, CanGc::note());
        dom_string_list
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-mode>
    fn Mode(&self) -> IDBTransactionMode {
        match self.shared_object.mode {
            IndexedDBTxnMode::Readonly => IDBTransactionMode::Readonly,
            IndexedDBTxnMode::Readwrite => IDBTransactionMode::Readwrite,
            IndexedDBTxnMode::Versionchange => IDBTransactionMode::Versionchange,
        }
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
