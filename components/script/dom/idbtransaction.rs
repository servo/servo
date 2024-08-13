/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::DOMStringListBinding::DOMStringListBinding::DOMStringListMethods;
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding;
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::IDBTransactionMethods;
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::IDBTransactionMode;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;
use crate::task_source::TaskSource;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::domexception::DOMException;
use crate::dom::domstringlist::DOMStringList;
use crate::dom::eventtarget::EventTarget;
use crate::dom::idbdatabase::IDBDatabase;
use crate::dom::idbobjectstore::IDBObjectStore;
use crate::dom::idbrequest::IDBRequest;

use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use net_traits::indexeddb_thread::{IndexedDBThreadMsg, IndexedDBThreadReturnType, SyncOperation};
use net_traits::IpcSend;
use profile_traits::ipc;

use servo_atoms::Atom;

use std::cell::Cell;
use std::collections::HashMap;

#[dom_struct]
pub struct IDBTransaction {
    eventtarget: EventTarget,
    object_store_names: DomRoot<DOMStringList>,
    mode: IDBTransactionMode,
    db: Dom<IDBDatabase>,
    error: MutNullableDom<DOMException>,

    // Not specified in WebIDL below this line
    store_handles: DomRefCell<HashMap<String, Dom<IDBObjectStore>>>,
    // https://www.w3.org/TR/IndexedDB-2/#transaction-request-list
    requests: DomRefCell<Vec<Dom<IDBRequest>>>,
    // https://www.w3.org/TR/IndexedDB-2/#transaction-active-flag
    active: Cell<bool>,
    // https://www.w3.org/TR/IndexedDB-2/#transaction-finish
    finished: Cell<bool>,
    // An unique identifier, used to commit and revert this transaction
    // FIXME:(rasviitanen) Replace this with a channel
    serial_number: u64,
}

impl IDBTransaction {
    fn new_inherited(
        connection: &IDBDatabase,
        mode: IDBTransactionMode,
        scope: DomRoot<DOMStringList>,
        serial_number: u64,
    ) -> IDBTransaction {
        IDBTransaction {
            eventtarget: EventTarget::new_inherited(),
            object_store_names: scope,
            mode: mode,
            db: Dom::from_ref(connection),
            error: Default::default(),

            store_handles: Default::default(),
            requests: Default::default(),
            active: Cell::new(true),
            finished: Cell::new(false),
            serial_number: serial_number,
        }
    }

    pub fn new(
        global: &GlobalScope,
        connection: &IDBDatabase,
        mode: IDBTransactionMode,
        scope: DomRoot<DOMStringList>,
    ) -> DomRoot<IDBTransaction> {
        let serial_number = IDBTransaction::register_new(&global, connection.get_name());
        reflect_dom_object(
            Box::new(IDBTransaction::new_inherited(
                connection,
                mode,
                scope,
                serial_number,
            )),
            global,
            IDBTransactionBinding::Wrap,
        )
    }

    // Registers a new transaction in the idb thread, and gets an unique serial number in return.
    // The serial number is used when placing requests against a transaction
    // and allows us to commit/abort transactions running in our idb thread.
    // FIXME:(rasviitanen) We could probably replace this with a channel instead,
    // and queue requests directly to that channel.
    fn register_new(global: &GlobalScope, db_name: DOMString) -> u64 {
        let (sender, receiver) = ipc::channel(global.time_profiler_chan().clone()).unwrap();

        global
            .resource_threads()
            .sender()
            .send(IndexedDBThreadMsg::Sync(SyncOperation::RegisterNewTxn(
                sender,
                global.origin().immutable().clone(),
                db_name.to_string(),
            )))
            .unwrap();

        receiver.recv().unwrap()
    }

    // Runs the transaction and waits for it to finish
    pub fn wait(&self) {
        // Start the transaction
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();

        let start_operation = SyncOperation::StartTransaction(
            sender,
            self.global().origin().immutable().clone(),
            self.db.get_name().to_string(),
            self.serial_number,
        );

        self.get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(start_operation))
            .unwrap();

        // Wait for transaction to complete
        if receiver.recv().is_err() {
            warn!("IDBtransaction failed to run");
        };
    }

    pub fn set_active_flag(&self, status: bool) {
        self.active.set(status)
    }

    pub fn is_active(&self) -> bool {
        self.active.get()
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
        receiver.recv().unwrap();
    }

    fn dispatch_complete(&self) {
        let global = self.global();
        let this = Trusted::new(self);
        global
            .database_access_task_source()
            .queue(
                task!(send_complete_notification: move || {
                    let this = this.root();
                    let global = this.global();
                    let event = Event::new(
                        &global,
                        Atom::from("complete"),
                        EventBubbles::DoesNotBubble,
                        EventCancelable::NotCancelable,
                    );
                    event.upcast::<Event>().fire(this.upcast());
                }),
                global.upcast(),
            )
            .unwrap();
    }

    fn get_idb_thread(&self) -> IpcSender<IndexedDBThreadMsg> {
        self.global().resource_threads().sender()
    }
}

impl IDBTransactionMethods for IDBTransaction {
    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-db
    fn Db(&self) -> DomRoot<IDBDatabase> {
        DomRoot::from_ref(&*self.db)
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-objectstore
    fn ObjectStore(&self, name: DOMString) -> Fallible<DomRoot<IDBObjectStore>> {
        // Step 1: Handle the case where transaction has finised
        if self.finished.get() {
            return Err(Error::InvalidState);
        }

        // Step 2: Check that the object store exists
        if !self.object_store_names.Contains(name.clone()) {
            return Err(Error::NotFound);
        }

        // Step 3: Each call to this method on the same
        // IDBTransaction instance with the same name
        // returns the same IDBObjectStore instance.
        let mut store_handles = self.store_handles.borrow_mut();
        let store = store_handles.entry(name.to_string()).or_insert({
            let store = IDBObjectStore::new(&self.global(), self.db.get_name(), name, None);
            store.set_transaction(&self);
            Dom::from_ref(&*store)
        });

        Ok(DomRoot::from_ref(&*store))
    }

    // https://www.w3.org/TR/IndexedDB-2/#commit-transaction
    fn Commit(&self) -> Fallible<()> {
        // Step 1
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
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
        if let IndexedDBThreadReturnType::Commit(Err(_result)) = result {
            // FIXME:(rasviitanen) also support Unknown error
            return Err(Error::QuotaExceeded);
        }

        // Step 3
        // FIXME:(rasviitanen) https://www.w3.org/TR/IndexedDB-2/#commit-a-transaction

        // Steps 3.1 and 3.3
        self.dispatch_complete();

        Ok(())
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-abort
    fn Abort(&self) -> Fallible<()> {
        // FIXME:(rasviitanen)
        // This only sets the flags, and does not abort the transaction
        // see https://www.w3.org/TR/IndexedDB-2/#abort-a-transaction
        if self.finished.get() {
            return Err(Error::InvalidState);
        }

        self.active.set(false);

        Ok(())
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-objectstorenames
    fn ObjectStoreNames(&self) -> DomRoot<DOMStringList> {
        self.object_store_names.clone()
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-mode
    fn Mode(&self) -> IDBTransactionMode {
        self.mode
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-error
    fn Error(&self) -> DomRoot<DOMException> {
        // FIXME:(rasviitanen) ???
        // It's weird that the WebIDL specifies that this isn't returning an Option.
        // "The error attribute’s getter must return this transaction's error, or null if none."
        unimplemented!();
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-onabort
    event_handler!(abort, GetOnabort, SetOnabort);

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-oncomplete
    event_handler!(complete, GetOncomplete, SetOncomplete);

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbtransaction-onerror
    event_handler!(error, GetOnerror, SetOnerror);
}
