/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use net_traits::indexeddb_thread::{IndexedDBThreadMsg, SyncOperation};
use net_traits::IpcSend;
use profile_traits::ipc;
use servo_atoms::Atom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBDatabaseBinding::{
    IDBDatabaseMethods, IDBObjectStoreParameters, IDBTransactionOptions,
};
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::IDBTransactionMode;
use crate::dom::bindings::codegen::UnionTypes::StringOrStringSequence;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::domstringlist::DOMStringList;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbobjectstore::IDBObjectStore;
use crate::dom::idbtransaction::IDBTransaction;
use crate::dom::idbversionchangeevent::IDBVersionChangeEvent;
use crate::task_source::TaskSource;

#[dom_struct]
pub struct IDBDatabase {
    eventtarget: EventTarget,
    /// <https://w3c.github.io/IndexedDB/#database-name>
    name: DOMString,
    /// <https://w3c.github.io/IndexedDB/#database-version>
    version: Cell<u64>,
    /// <https://w3c.github.io/IndexedDB/#object-store>
    object_store_names: DomRefCell<Vec<DOMString>>,
    /// <https://w3c.github.io/IndexedDB/#database-upgrade-transaction>
    upgrade_transaction: MutNullableDom<IDBTransaction>,

    // Flags
    /// https://w3c.github.io/IndexedDB/#connection-close-pending-flag
    closing: Cell<bool>,
}

impl IDBDatabase {
    pub fn new_inherited(name: DOMString, version: u64) -> IDBDatabase {
        IDBDatabase {
            eventtarget: EventTarget::new_inherited(),
            name,
            version: Cell::new(version),
            object_store_names: Default::default(),

            upgrade_transaction: Default::default(),
            closing: Cell::new(false),
        }
    }

    pub fn new(global: &GlobalScope, name: DOMString, version: u64) -> DomRoot<IDBDatabase> {
        reflect_dom_object(Box::new(IDBDatabase::new_inherited(name, version)), global)
    }

    fn get_idb_thread(&self) -> IpcSender<IndexedDBThreadMsg> {
        self.global().resource_threads().sender()
    }

    pub fn get_name(&self) -> DOMString {
        self.name.clone()
    }

    pub fn object_stores(&self) -> DomRoot<DOMStringList> {
        DOMStringList::new(&self.global(), self.object_store_names.borrow().clone())
    }

    pub fn version(&self) -> u64 {
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let operation = SyncOperation::Version(
            sender,
            self.global().origin().immutable().clone(),
            self.name.to_string(),
        );

        self.get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(operation))
            .unwrap();

        receiver.recv().unwrap()
    }

    pub fn set_transaction(&self, transaction: &IDBTransaction) {
        self.upgrade_transaction.set(Some(transaction));
    }

    #[allow(dead_code)] // This will be used once we allow multiple concurrent connections
    pub fn dispatch_versionchange(&self, old_version: u64, new_version: Option<u64>) {
        let global = self.global();
        let this = Trusted::new(self);
        global
            .database_access_task_source()
            .queue(
                task!(send_versionchange_notification: move || {
                    let this = this.root();
                    let global = this.global();
                    let event = IDBVersionChangeEvent::new(
                        &global,
                        Atom::from("versionchange"),
                        EventBubbles::DoesNotBubble,
                        EventCancelable::NotCancelable,
                        old_version,
                        new_version,
                    );
                    event.upcast::<Event>().fire(this.upcast());
                }),
                global.upcast(),
            )
            .unwrap();
    }
}

impl IDBDatabaseMethods for IDBDatabase {
    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-transaction>
    fn Transaction(
        &self,
        store_names: StringOrStringSequence,
        mode: IDBTransactionMode,
        options: &IDBTransactionOptions,
    ) -> DomRoot<IDBTransaction> {
        // FIXIME:(arihant2math) use options
        // Step 1: Check if upgrade transaction is running
        // FIXME:(rasviitanen)

        // Step 2: if close flag is set, throw error
        // FIXME:(rasviitanen)

        // Step 3
        match store_names {
            StringOrStringSequence::String(name) => IDBTransaction::new(
                &self.global(),
                &self,
                mode,
                DOMStringList::new(&self.global(), vec![name]),
            ),
            StringOrStringSequence::StringSequence(sequence) => {
                // FIXME:(rasviitanen) Remove eventual duplicated names
                // from the sequence
                IDBTransaction::new(
                    &self.global(),
                    &self,
                    mode,
                    DOMStringList::new(&self.global(), sequence),
                )
            },
        }
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-createobjectstore
    fn CreateObjectStore(
        &self,
        name: DOMString,
        options: &IDBObjectStoreParameters,
    ) -> Fallible<DomRoot<IDBObjectStore>> {
        // FIXME:(arihant2math) ^^ Change idl to match above.
        // Step 2
        let upgrade_transaction = match self.upgrade_transaction.get() {
            Some(txn) => txn,
            None => return Err(Error::InvalidState),
        };

        // Step 3
        if !upgrade_transaction.is_active() {
            return Err(Error::TransactionInactive);
        }

        // Step 4
        let key_path = options.keyPath.as_ref();

        // Step 5
        if let Some(ref path) = key_path {
            if !IDBObjectStore::is_valid_key_path(path) {
                return Err(Error::Syntax);
            }
        }

        // Step 6 FIXME:(rasviitanen)
        // If an object store named name already exists in database throw a "ConstraintError" DOMException.

        // Step 7
        let auto_increment = options.autoIncrement;

        // Step 8
        if auto_increment == true {
            match key_path {
                Some(StringOrStringSequence::String(path)) => {
                    if path == "" {
                        return Err(Error::InvalidAccess);
                    }
                },
                Some(StringOrStringSequence::StringSequence(_)) => {
                    return Err(Error::InvalidAccess);
                },
                None => {},
            }
        }

        // Step 9
        let object_store = IDBObjectStore::new(
            &self.global(),
            self.name.clone(),
            name.clone(),
            Some(options),
        );
        object_store.set_transaction(&upgrade_transaction);

        // FIXME:(arihant2math!!!) Move this to constructor
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();

        let operation = SyncOperation::CreateObjectStore(
            sender,
            self.global().origin().immutable().clone(),
            self.name.to_string(),
            name.to_string(),
            auto_increment,
        );

        self.get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(operation))
            .unwrap();

        if receiver
            .recv()
            .expect("Could not receive object store creation status")
            .is_err()
        {
            warn!("Object store creation failed in idb thread");
            return Err(Error::JSFailed);
        };

        self.object_store_names.borrow_mut().push(name);
        Ok(object_store)
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-deleteobjectstore
    fn DeleteObjectStore(&self, name: DOMString) {
        // Steps 1 & 2
        let transaction = self.upgrade_transaction.get();
        let transaction = match transaction {
            Some(transaction) => transaction,
            // FIXME:(arihant2math) Return error
            None => return,
        };

        // Step 3

        if !transaction.is_active() {
            // FIXME:(arihant2math) return error instead
            return;
        }

        // Step 4
        // FIXME:(arihant2math) check if the store is in the database

        // Step 5
        self.object_store_names
            .borrow_mut()
            .retain(|store_name| store_name.to_string() != name.to_string());

        // Step 6
        // FIXME:(arihant2math) Remove from index set ...

        // Step 7
        // FIXME:(arihant2math!!!) Move this to constructor
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();

        let operation = SyncOperation::DeleteObjectStore(
            sender,
            self.global().origin().immutable().clone(),
            self.name.to_string(),
            name.to_string(),
        );

        self.get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(operation))
            .unwrap();

        if receiver
            .recv()
            .expect("Could not receive object store deletion status")
            .is_err()
        {
            warn!("Object store deletion failed in idb thread");
        };
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-name
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-version
    fn Version(&self) -> u64 {
        self.version()
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-objectstorenames
    fn ObjectStoreNames(&self) -> DomRoot<DOMStringList> {
        // FIXME: (arihant2math) Sort the list of names, as per spec
        DOMStringList::new(&self.global(), self.object_store_names.borrow().clone())
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-close
    fn Close(&self) {
        // Step 1: Set the close pending flag of connection.
        self.closing.set(true);

        // Step 2: Handle force flag
        // FIXME:(rasviitanen)

        // Step 3: Wait for all transactions by this db to finish
        // FIXME:(rasviitanen)

        // Step 4: If force flag is set, fire a close event
        // FIXME:(rasviitanen)
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-onabort
    event_handler!(abort, GetOnabort, SetOnabort);

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-onclose
    event_handler!(close, GetOnclose, SetOnclose);

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-onerror
    event_handler!(error, GetOnerror, SetOnerror);

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-onversionchange
    event_handler!(versionchange, GetOnversionchange, SetOnversionchange);
}
