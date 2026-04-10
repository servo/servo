/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use profile_traits::generic_channel::channel;
use servo_base::generic_channel::{GenericSend, GenericSender};
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
    close_pending: Cell<bool>,
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
            close_pending: Cell::new(false),
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
            CanGc::deprecated_note(),
        )
    }

    pub(crate) fn object_store_names_snapshot(&self) -> Vec<DOMString> {
        // https://w3c.github.io/IndexedDB/#abort-upgrade-transaction
        // Step 4. Set connection’s object store set to the set of object stores in database if database previously existed,
        // or the empty set if database was newly created.
        self.object_store_names.borrow().clone()
    }

    pub(crate) fn set_object_store_names_from_backend(&self, names: Vec<String>) {
        // https://w3c.github.io/IndexedDB/#abort-upgrade-transaction
        // Step 4. NOTE: This reverts the value of objectStoreNames returned by the IDBDatabase object.
        *self.object_store_names.borrow_mut() = names.into_iter().map(Into::into).collect();
    }

    pub(crate) fn restore_object_store_names(&self, names: Vec<DOMString>) {
        // https://w3c.github.io/IndexedDB/#abort-upgrade-transaction
        // Step 4. NOTE: This reverts the value of objectStoreNames returned by the IDBDatabase object.
        *self.object_store_names.borrow_mut() = names;
    }

    pub(crate) fn object_store_exists(&self, name: &DOMString) -> bool {
        self.object_store_names
            .borrow()
            .iter()
            .any(|store_name| store_name == name)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-version>
    pub(crate) fn version(&self) -> u64 {
        // The version getter steps are to return this’s version.
        self.version.get()
    }

    pub(crate) fn set_version(&self, version: u64) {
        self.version.set(version);
    }

    pub fn set_transaction(&self, transaction: &IDBTransaction) {
        self.upgrade_transaction.set(Some(transaction));
    }

    pub(crate) fn clear_upgrade_transaction(&self, transaction: &IDBTransaction) {
        let current = self
            .upgrade_transaction
            .get()
            .expect("clear_upgrade_transaction called but no upgrade transaction is set");

        debug_assert!(
            &*current == transaction,
            "clear_upgrade_transaction called with non-current transaction"
        );

        self.upgrade_transaction.set(None);
    }

    /// <https://w3c.github.io/IndexedDB/#eventdef-idbdatabase-versionchange>
    pub fn dispatch_versionchange(
        &self,
        old_version: u64,
        new_version: Option<u64>,
        can_gc: CanGc,
    ) {
        let global = self.global();
        let _ = IDBVersionChangeEvent::fire_version_change_event(
            &global,
            self.upcast(),
            Atom::from("versionchange"),
            old_version,
            new_version,
            can_gc,
        );
    }
}

impl IDBDatabaseMethods<crate::DomTypeHolder> for IDBDatabase {
    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-transaction>
    fn Transaction(
        &self,
        store_names: StringOrStringSequence,
        mode: IDBTransactionMode,
        options: &IDBTransactionOptions,
    ) -> Fallible<DomRoot<IDBTransaction>> {
        // Step 1. If a live upgrade transaction is associated with the connection,
        // throw an "InvalidStateError" DOMException.
        if self.upgrade_transaction.get().is_some() {
            return Err(Error::InvalidState(None));
        }

        // Step 2. If this’s close pending flag is true, then throw an
        // "InvalidStateError" DOMException.
        if self.close_pending.get() {
            return Err(Error::InvalidState(None));
        }

        // Step 3. Let scope be the set of unique strings in storeNames if it is
        // a sequence, or a set containing one string equal to storeNames otherwise.
        let mut scope = match store_names {
            StringOrStringSequence::String(name) => vec![name],
            StringOrStringSequence::StringSequence(sequence) => sequence,
        };
        scope.sort_unstable_by(|left, right| {
            left.str().encode_utf16().cmp(right.str().encode_utf16())
        });
        scope.dedup();

        // Step 4. If any string in scope is not the name of an object store in
        // the connected database, throw a "NotFoundError" DOMException.
        if scope.iter().any(|name| !self.object_store_exists(name)) {
            return Err(Error::NotFound(None));
        }

        // Step 5. If scope is empty, throw an "InvalidAccessError" DOMException.
        if scope.is_empty() {
            return Err(Error::InvalidAccess(None));
        }

        // Step 6. If mode is not "readonly" or "readwrite", throw a TypeError.
        if mode != IDBTransactionMode::Readonly && mode != IDBTransactionMode::Readwrite {
            return Err(Error::Type(c"Invalid transaction mode".to_owned()));
        }

        // Step 7. Let transaction be a newly created transaction with this
        // connection, mode, options’ durability member, and the set of object
        // stores named in scope.
        let durability = options.durability;
        let scope = DOMStringList::new(&self.global(), scope, CanGc::deprecated_note());
        let transaction = IDBTransaction::new(
            &self.global(),
            self,
            mode,
            durability,
            &scope,
            CanGc::deprecated_note(),
        );

        // Step 8. Set transaction’s cleanup event loop to the current event loop.
        transaction.set_cleanup_event_loop();
        // https://w3c.github.io/IndexedDB/#cleanup-indexed-database-transactions
        // NOTE: These steps are invoked by [HTML]. They ensure that transactions created
        // by a script call to transaction() are deactivated once the task that invoked
        // the script has completed. The steps are run at most once for each transaction.
        // https://w3c.github.io/IndexedDB/#transaction-concept
        // A transaction optionally has a cleanup event loop which is an event loop.
        self.global()
            .get_indexeddb()
            .register_indexeddb_transaction(&transaction);

        // Step 9. Return an IDBTransaction object representing transaction.
        Ok(transaction)
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbdatabase-createobjectstore>
    fn CreateObjectStore(
        &self,
        cx: &mut JSContext,
        name: DOMString,
        options: &IDBObjectStoreParameters,
    ) -> Fallible<DomRoot<IDBObjectStore>> {
        // Step 1. Let database be this’s associated database.

        // Step 2. Let transaction be database’s upgrade transaction if it is not null,
        // or throw an "InvalidStateError" DOMException otherwise.
        let transaction = match self.upgrade_transaction.get() {
            Some(txn) => txn,
            None => return Err(Error::InvalidState(None)),
        };

        // Step 3. If transaction’s state is not active, then throw a
        // "TransactionInactiveError" DOMException.
        if !transaction.is_active() {
            return Err(Error::TransactionInactive(None));
        }

        // Step 4. Let keyPath be options’s keyPath member if it is not undefined
        // or null, or null otherwise.
        let key_path = options.keyPath.as_ref();

        // Step 5. If keyPath is not null and is not a valid key path, throw a
        // "SyntaxError" DOMException.
        if let Some(path) = key_path {
            if !is_valid_key_path(cx, path)? {
                return Err(Error::Syntax(None));
            }
        }

        // Step 6. If an object store named name already exists in database throw
        // a "ConstraintError" DOMException.
        if self.object_store_names.borrow().contains(&name) {
            return Err(Error::Constraint(None));
        }

        // Step 7. Let autoIncrement be options’s autoIncrement member.
        let auto_increment = options.autoIncrement;

        // Step 8. If autoIncrement is true and keyPath is an empty string or any
        // sequence (empty or otherwise), throw an "InvalidAccessError" DOMException.
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

        // Step 9. Let store be a new object store in database. Set the created
        // object store’s name to name. If autoIncrement is true, then the
        // created object store uses a key generator. If keyPath is not null,
        // set the created object store’s key path to keyPath.
        let object_store = IDBObjectStore::new(
            &self.global(),
            self.name.clone(),
            name.clone(),
            Some(options),
            if auto_increment { Some(1) } else { None },
            CanGc::from_cx(cx),
            &transaction,
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

        // Step 10. Return a new object store handle associated with store and transaction.
        Ok(object_store)
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbdatabase-deleteobjectstore>
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

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbdatabase-name>
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbdatabase-version>
    fn Version(&self) -> u64 {
        self.version()
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbdatabase-objectstorenames>
    fn ObjectStoreNames(&self, can_gc: CanGc) -> DomRoot<DOMStringList> {
        DOMStringList::new_sorted(&self.global(), &*self.object_store_names.borrow(), can_gc)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbdatabase-close>
    fn Close(&self) {
        // Step 1: Run close a database connection with this connection.

        // <https://w3c.github.io/IndexedDB/#close-a-database-connection>
        // Step 1: Set connection’s close pending flag to true.
        self.close_pending.set(true);

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

    // https://www.w3.org/TR/IndexedDB-3/#dom-idbdatabase-onabort
    event_handler!(abort, GetOnabort, SetOnabort);

    // https://www.w3.org/TR/IndexedDB-3/#dom-idbdatabase-onclose
    event_handler!(close, GetOnclose, SetOnclose);

    // https://www.w3.org/TR/IndexedDB-3/#dom-idbdatabase-onerror
    event_handler!(error, GetOnerror, SetOnerror);

    // https://www.w3.org/TR/IndexedDB-3/#dom-idbdatabase-onversionchange
    event_handler!(versionchange, GetOnversionchange, SetOnversionchange);
}
