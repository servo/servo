/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use net_traits::IpcSend;
use net_traits::indexeddb_thread::{IndexedDBThreadMsg, SyncOperation};
use profile_traits::ipc;
use stylo_atoms::Atom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBDatabaseBinding::{
    IDBDatabaseMethods, IDBObjectStoreParameters, IDBTransactionOptions,
};
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::IDBTransactionMode;
use crate::dom::bindings::codegen::UnionTypes::StringOrStringSequence;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::domstringlist::DOMStringList;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbobjectstore::IDBObjectStore;
use crate::dom::idbtransaction::IDBTransaction;
use crate::dom::idbversionchangeevent::IDBVersionChangeEvent;
use crate::indexed_db::is_valid_key_path;
use crate::script_runtime::CanGc;

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
    /// <https://w3c.github.io/IndexedDB/#connection-close-pending-flag>
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

    pub fn new(
        global: &GlobalScope,
        name: DOMString,
        version: u64,
        can_gc: CanGc,
    ) -> DomRoot<IDBDatabase> {
        reflect_dom_object(
            Box::new(IDBDatabase::new_inherited(name, version)),
            global,
            can_gc,
        )
    }

    fn get_idb_thread(&self) -> IpcSender<IndexedDBThreadMsg> {
        self.global().resource_threads().sender()
    }

    pub fn get_name(&self) -> DOMString {
        self.name.clone()
    }

    pub fn object_stores(&self) -> DomRoot<DOMStringList> {
        DOMStringList::new(
            &self.global(),
            self.object_store_names.borrow().clone(),
            CanGc::note(),
        )
    }

    pub fn version(&self) -> u64 {
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let operation = SyncOperation::Version(
            sender,
            self.global().origin().immutable().clone(),
            self.name.to_string(),
        );

        let _ = self
            .get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(operation));

        receiver.recv().unwrap()
    }

    pub fn set_transaction(&self, transaction: &IDBTransaction) {
        self.upgrade_transaction.set(Some(transaction));
    }

    #[allow(dead_code)] // This will be used once we allow multiple concurrent connections
    pub fn dispatch_versionchange(
        &self,
        old_version: u64,
        new_version: Option<u64>,
        _can_gc: CanGc,
    ) {
        let global = self.global();
        let this = Trusted::new(self);
        global.task_manager().database_access_task_source().queue(
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
                    CanGc::note()
                );
                event.upcast::<Event>().fire(this.upcast(), CanGc::note());
            }),
        );
    }
}

impl IDBDatabaseMethods<crate::DomTypeHolder> for IDBDatabase {
    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-transaction>
    fn Transaction(
        &self,
        store_names: StringOrStringSequence,
        mode: IDBTransactionMode,
        _options: &IDBTransactionOptions,
    ) -> Fallible<DomRoot<IDBTransaction>> {
        // FIXIME:(arihant2math) use options
        // Step 1: Check if upgrade transaction is running
        // FIXME:(rasviitanen)

        // Step 2: if close flag is set, throw error
        if self.closing.get() {
            return Err(Error::InvalidState);
        }

        // Step 3
        Ok(match store_names {
            StringOrStringSequence::String(name) => IDBTransaction::new(
                &self.global(),
                self,
                mode,
                &DOMStringList::new(&self.global(), vec![name], CanGc::note()),
                CanGc::note(),
            ),
            StringOrStringSequence::StringSequence(sequence) => {
                // FIXME:(rasviitanen) Remove eventual duplicated names
                // from the sequence
                IDBTransaction::new(
                    &self.global(),
                    self,
                    mode,
                    &DOMStringList::new(&self.global(), sequence, CanGc::note()),
                    CanGc::note(),
                )
            },
        })
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
        if let Some(path) = key_path {
            if !is_valid_key_path(path) {
                return Err(Error::Syntax);
            }
        }

        // Step 6
        if self
            .object_store_names
            .borrow()
            .iter()
            .any(|store_name| store_name.to_string() == name.to_string())
        {
            return Err(Error::Constraint);
        }

        // Step 7
        let auto_increment = options.autoIncrement;

        // Step 8
        if auto_increment {
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
            CanGc::note(),
        );
        object_store.set_transaction(&upgrade_transaction);

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
            return Err(Error::InvalidState);
        };

        self.object_store_names.borrow_mut().push(name);
        Ok(object_store)
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-deleteobjectstore
    fn DeleteObjectStore(&self, name: DOMString) -> Fallible<()> {
        // Steps 1 & 2
        let transaction = self.upgrade_transaction.get();
        let transaction = match transaction {
            Some(transaction) => transaction,
            None => return Err(Error::InvalidState),
        };

        // Step 3
        if !transaction.is_active() {
            return Err(Error::TransactionInactive);
        }

        // Step 4
        if !self
            .object_store_names
            .borrow()
            .iter()
            .any(|store_name| store_name.to_string() == name.to_string())
        {
            return Err(Error::NotFound);
        }

        // Step 5
        self.object_store_names
            .borrow_mut()
            .retain(|store_name| store_name.to_string() != name.to_string());

        // Step 6
        // FIXME:(arihant2math) Remove from index set ...

        // Step 7
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
            return Err(Error::InvalidState);
        };
        Ok(())
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
        DOMStringList::new(
            &self.global(),
            self.object_store_names.borrow().clone(),
            CanGc::note(),
        )
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-close
    fn Close(&self) {
        // Step 1: Set the close pending flag of connection.
        self.closing.set(true);

        // Step 2: Handle force flag
        // FIXME:(arihant2math)
        // Step 3: Wait for all transactions by this db to finish
        // FIXME:(arihant2math)
        // Step 4: If force flag is set, fire a close event
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let operation = SyncOperation::CloseDatabase(
            sender,
            self.global().origin().immutable().clone(),
            self.name.to_string(),
        );
        let _ = self
            .get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(operation));

        if receiver.recv().is_err() {
            warn!("Database close failed in idb thread");
        };
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
