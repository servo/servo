/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use base::generic_channel::{GenericSend, GenericSender};
use dom_struct::dom_struct;
use js::context::JSContext;
use profile_traits::generic_channel::channel;
use storage_traits::indexeddb::{IndexedDBThreadMsg, KeyPath, SyncOperation};
use stylo_atoms::Atom;
use uuid::Uuid;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBDatabaseBinding::{
    IDBDatabaseMethods, IDBObjectStoreParameters, IDBTransactionOptions,
};
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::IDBTransactionMode;
use crate::dom::bindings::codegen::UnionTypes::StringOrStringSequence;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::domstringlist::DOMStringList;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb::idbobjectstore::IDBObjectStore;
use crate::dom::indexeddb::idbtransaction::IDBTransaction;
use crate::dom::indexeddb::idbversionchangeevent::IDBVersionChangeEvent;
use crate::indexeddb::is_valid_key_path;
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

    #[no_trace]
    #[ignore_malloc_size_of = "Uuid"]
    id: Uuid,

    // Flags
    /// <https://w3c.github.io/IndexedDB/#connection-close-pending-flag>
    closing: Cell<bool>,
}

impl IDBDatabase {
    pub fn new_inherited(name: DOMString, id: Uuid, version: u64) -> IDBDatabase {
        IDBDatabase {
            eventtarget: EventTarget::new_inherited(),
            name,
            id,
            version: Cell::new(version),
            object_store_names: Default::default(),

            upgrade_transaction: Default::default(),
            closing: Cell::new(false),
        }
    }

    pub fn new(
        global: &GlobalScope,
        name: DOMString,
        id: Uuid,
        version: u64,
        can_gc: CanGc,
    ) -> DomRoot<IDBDatabase> {
        reflect_dom_object(
            Box::new(IDBDatabase::new_inherited(name, id, version)),
            global,
            can_gc,
        )
    }

    fn get_idb_thread(&self) -> GenericSender<IndexedDBThreadMsg> {
        self.global().storage_threads().sender()
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

    pub(crate) fn object_store_exists(&self, name: &DOMString) -> bool {
        self.object_store_names
            .borrow()
            .iter()
            .any(|store_name| store_name == name)
    }

    pub fn version(&self) -> u64 {
        let (sender, receiver) = channel(self.global().time_profiler_chan().clone()).unwrap();
        let operation = SyncOperation::Version(
            sender,
            self.global().origin().immutable().clone(),
            self.name.to_string(),
        );

        let _ = self
            .get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(operation));

        receiver.recv().unwrap().unwrap_or_else(|e| {
            error!("{e:?}");
            u64::MAX
        })
    }

    pub fn set_transaction(&self, transaction: &IDBTransaction) {
        self.upgrade_transaction.set(Some(transaction));
    }

    /// <https://w3c.github.io/IndexedDB/#eventdef-idbdatabase-versionchange>
    pub fn dispatch_versionchange(
        &self,
        old_version: u64,
        new_version: Option<u64>,
        can_gc: CanGc,
    ) {
        let global = self.global();
        let event = IDBVersionChangeEvent::new(
            &global,
            Atom::from("versionchange"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            old_version,
            new_version,
            can_gc,
        );
        event.upcast::<Event>().fire(self.upcast(), can_gc);
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
        let store_names_len = match &store_names {
            StringOrStringSequence::String(_) => 1,
            StringOrStringSequence::StringSequence(sequence) => sequence.len(),
        };
        println!(
            "[IDBDBG_DB_TXN_CREATE] db={} mode={:?} stores={}",
            self.get_name().to_string(),
            mode,
            store_names_len
        );
        // FIXIME:(arihant2math) use options
        // Step 1: Check if upgrade transaction is running
        // FIXME:(rasviitanen)

        // Step 2: if close flag is set, throw error
        if self.closing.get() {
            return Err(Error::InvalidState(None));
        }

        // Step 3
        let transaction = match store_names {
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
        };
        if mode != IDBTransactionMode::Versionchange {
            // https://w3c.github.io/IndexedDB/#dom-idbdatabase-transaction
            // Step 8: Set transaction’s cleanup event loop to the current event loop.
            // https://w3c.github.io/IndexedDB/#cleanup-indexed-database-transactions
            // “transactions created by a script call to transaction() are deactivated once the task that invoked the script has completed.”
            // https://w3c.github.io/IndexedDB/#transaction-concept
            // “A transaction optionally has a cleanup event loop which is an event loop.”
            // Versionchange transactions can’t be manually created; only script-created transactions()
            // are subject to HTML cleanup deactivation.
            transaction.set_cleanup_event_loop();
            self.global()
                .get_indexeddb()
                .register_indexeddb_transaction(&transaction);
            transaction.set_registered_in_global();
            println!(
                "[IDBDBG_DB_TXN_REGISTER] db={} txn={}",
                self.get_name().to_string(),
                transaction.get_serial_number()
            );
        }
        Ok(transaction)
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-createobjectstore>
    fn CreateObjectStore(
        &self,
        cx: &mut JSContext,
        name: DOMString,
        options: &IDBObjectStoreParameters,
    ) -> Fallible<DomRoot<IDBObjectStore>> {
        // Step 2
        let upgrade_transaction = match self.upgrade_transaction.get() {
            Some(txn) => txn,
            None => return Err(Error::InvalidState(None)),
        };

        // Step 3
        if !upgrade_transaction.is_active() {
            return Err(Error::TransactionInactive(None));
        }

        // Step 4
        let key_path = options.keyPath.as_ref();

        // Step 5
        if let Some(path) = key_path {
            if !is_valid_key_path(cx, path)? {
                return Err(Error::Syntax(None));
            }
        }

        // Step 6
        if self.object_store_names.borrow().contains(&name) {
            return Err(Error::Constraint(None));
        }

        // Step 7
        let auto_increment = options.autoIncrement;

        // Step 8
        if auto_increment {
            match key_path {
                Some(StringOrStringSequence::String(path)) => {
                    if path.is_empty() {
                        return Err(Error::InvalidAccess(None));
                    }
                },
                Some(StringOrStringSequence::StringSequence(_)) => {
                    return Err(Error::InvalidAccess(None));
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
            CanGc::from_cx(cx),
            &upgrade_transaction,
        );

        let (sender, receiver) = channel(self.global().time_profiler_chan().clone()).unwrap();

        let key_paths = key_path.map(|p| match p {
            StringOrStringSequence::String(s) => KeyPath::String(s.to_string()),
            StringOrStringSequence::StringSequence(s) => {
                KeyPath::Sequence(s.iter().map(|s| s.to_string()).collect())
            },
        });
        let operation = SyncOperation::CreateObjectStore(
            sender,
            self.global().origin().immutable().clone(),
            self.name.to_string(),
            name.to_string(),
            key_paths,
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
            return Err(Error::InvalidState(None));
        };

        self.object_store_names.borrow_mut().push(name);
        Ok(object_store)
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-deleteobjectstore>
    fn DeleteObjectStore(&self, name: DOMString) -> Fallible<()> {
        // Steps 1 & 2
        let transaction = self.upgrade_transaction.get();
        let transaction = match transaction {
            Some(transaction) => transaction,
            None => return Err(Error::InvalidState(None)),
        };

        // Step 3
        if !transaction.is_active() {
            return Err(Error::TransactionInactive(None));
        }

        // Step 4
        if !self.object_store_names.borrow().contains(&name) {
            return Err(Error::NotFound(None));
        }

        // Step 5
        self.object_store_names
            .borrow_mut()
            .retain(|store_name| *store_name != name);

        // Step 6
        // FIXME:(arihant2math) Remove from index set ...

        // Step 7
        let (sender, receiver) = channel(self.global().time_profiler_chan().clone()).unwrap();

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
            return Err(Error::InvalidState(None));
        };
        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-name>
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-version>
    fn Version(&self) -> u64 {
        self.version()
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbdatabase-objectstorenames>
    fn ObjectStoreNames(&self, can_gc: CanGc) -> DomRoot<DOMStringList> {
        DOMStringList::new_sorted(&self.global(), &*self.object_store_names.borrow(), can_gc)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-close>
    fn Close(&self) {
        // Step 1: Run close a database connection with this connection.

        // <https://w3c.github.io/IndexedDB/#close-a-database-connection>
        // Step 1: Set connection’s close pending flag to true.
        self.closing.set(true);

        // Note: rest of algo runs in-parallel.
        let operation = SyncOperation::CloseDatabase(
            self.global().origin().immutable().clone(),
            self.id,
            self.name.to_string(),
        );
        let _ = self
            .get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(operation));
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
