/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleValue;
use net_traits::IpcSend;
use net_traits::indexeddb_thread::{
    AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, IndexedDBKeyType,
    IndexedDBThreadMsg, SyncOperation,
};
use profile_traits::ipc;
use script_bindings::error::ErrorResult;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBDatabaseBinding::IDBObjectStoreParameters;
use crate::dom::bindings::codegen::Bindings::IDBObjectStoreBinding::IDBObjectStoreMethods;
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::{
    IDBTransactionMethods, IDBTransactionMode,
};
// We need to alias this name, otherwise test-tidy complains at &String reference.
use crate::dom::bindings::codegen::UnionTypes::StringOrStringSequence as StrOrStringSequence;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone;
use crate::dom::domstringlist::DOMStringList;
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbrequest::IDBRequest;
use crate::dom::idbtransaction::IDBTransaction;
use crate::indexed_db::{
    self, ExtractionResult, convert_value_to_key, convert_value_to_key_range, extract_key,
};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[derive(JSTraceable, MallocSizeOf)]
pub enum KeyPath {
    String(DOMString),
    StringSequence(Vec<DOMString>),
}

#[dom_struct]
pub struct IDBObjectStore {
    reflector_: Reflector,
    name: DomRefCell<DOMString>,
    key_path: Option<KeyPath>,
    index_names: DomRoot<DOMStringList>,
    transaction: Dom<IDBTransaction>,

    // We store the db name in the object store to be able to find the correct
    // store in the idb thread when checking if we have a key generator
    db_name: DOMString,
}

impl IDBObjectStore {
    pub fn new_inherited(
        global: &GlobalScope,
        db_name: DOMString,
        name: DOMString,
        options: Option<&IDBObjectStoreParameters>,
        can_gc: CanGc,
        transaction: &IDBTransaction,
    ) -> IDBObjectStore {
        let key_path: Option<KeyPath> = match options {
            Some(options) => options.keyPath.as_ref().map(|path| match path {
                StrOrStringSequence::String(inner) => KeyPath::String(inner.clone()),
                StrOrStringSequence::StringSequence(inner) => {
                    KeyPath::StringSequence(inner.clone())
                },
            }),
            None => None,
        };

        IDBObjectStore {
            reflector_: Reflector::new(),
            name: DomRefCell::new(name),
            key_path,

            index_names: DOMStringList::new(global, Vec::new(), can_gc),
            transaction: Dom::from_ref(transaction),
            db_name,
        }
    }

    pub fn new(
        global: &GlobalScope,
        db_name: DOMString,
        name: DOMString,
        options: Option<&IDBObjectStoreParameters>,
        can_gc: CanGc,
        transaction: &IDBTransaction,
    ) -> DomRoot<IDBObjectStore> {
        reflect_dom_object(
            Box::new(IDBObjectStore::new_inherited(
                global,
                db_name,
                name,
                options,
                can_gc,
                transaction,
            )),
            global,
            can_gc,
        )
    }

    pub fn get_name(&self) -> DOMString {
        self.name.borrow().clone()
    }

    pub fn transaction(&self) -> DomRoot<IDBTransaction> {
        self.transaction.as_rooted()
    }

    fn has_key_generator(&self) -> bool {
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();

        let operation = SyncOperation::HasKeyGenerator(
            sender,
            self.global().origin().immutable().clone(),
            self.db_name.to_string(),
            self.name.borrow().to_string(),
        );

        self.global()
            .resource_threads()
            .send(IndexedDBThreadMsg::Sync(operation))
            .unwrap();

        // First unwrap for ipc
        // Second unwrap will never happen unless this db gets manually deleted somehow
        receiver.recv().unwrap().unwrap()
    }

    // fn get_stored_key_path(&mut self) -> Option<KeyPath> {
    //     let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
    //
    //     let operation = SyncOperation::KeyPath(
    //         sender,
    //         self.global().origin().immutable().clone(),
    //         self.db_name.to_string(),
    //         self.name.borrow().to_string(),
    //     );
    //
    //     self.global()
    //         .resource_threads()
    //         .sender()
    //         .send(IndexedDBThreadMsg::Sync(operation))
    //         .unwrap();
    //
    //     // First unwrap for ipc
    //     // Second unwrap will never happen unless this db gets manually deleted somehow
    //     let key_path = receiver.recv().unwrap().unwrap();
    //     key_path.map(|p| {
    //         // TODO: have separate storage for string sequence of len 1 and signle string
    //         if p.len() == 1 {
    //             KeyPath::String(DOMString::from_string(p[0].clone()))
    //         } else {
    //             let strings: Vec<_> = p.into_iter().map(|s| {
    //                 DOMString::from_string(s)
    //             }).collect();
    //             KeyPath::StringSequence(strings)
    //         }
    //     })
    // }

    // https://www.w3.org/TR/IndexedDB-2/#object-store-in-line-keys
    fn uses_inline_keys(&self) -> bool {
        self.key_path.is_some()
    }

    fn verify_not_deleted(&self) -> ErrorResult {
        let db = self.transaction.Db();
        if !db.object_store_exists(&self.name.borrow()) {
            return Err(Error::InvalidState);
        }
        Ok(())
    }

    /// Checks if the transation is active, throwing a "TransactionInactiveError" DOMException if not.
    fn check_transaction_active(&self) -> Fallible<()> {
        // Let transaction be this object store handle's transaction.
        let transaction = &self.transaction;

        // If transaction is not active, throw a "TransactionInactiveError" DOMException.
        if !transaction.is_active() {
            return Err(Error::TransactionInactive);
        }

        Ok(())
    }

    /// Checks if the transation is active, throwing a "TransactionInactiveError" DOMException if not.
    /// it then checks if the transaction is a read-only transaction, throwing a "ReadOnlyError" DOMException if so.
    fn check_readwrite_transaction_active(&self) -> Fallible<()> {
        // Let transaction be this object store handle's transaction.
        let transaction = &self.transaction;

        // If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        if let IDBTransactionMode::Readonly = transaction.get_mode() {
            return Err(Error::ReadOnly);
        }
        Ok(())
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-put
    fn put(
        &self,
        cx: SafeJSContext,
        value: HandleValue,
        key: HandleValue,
        overwrite: bool,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1. Let transaction be handle’s transaction.
        // Step 2: Let store be this object store handle's object store.
        // This is resolved in the `execute_async` function.
        // Step 3: If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        // Step 5. If transaction is a read-only transaction, throw a "ReadOnlyError" DOMException.
        self.check_readwrite_transaction_active()?;

        // Step 6: If store uses in-line keys and key was given, throw a "DataError" DOMException.
        if !key.is_undefined() && self.uses_inline_keys() {
            return Err(Error::Data);
        }

        // Step 7: If store uses out-of-line keys and has no key generator
        // and key was not given, throw a "DataError" DOMException.
        if !self.uses_inline_keys() && !self.has_key_generator() && key.is_undefined() {
            return Err(Error::Data);
        }

        // Step 8: If key was given, then: convert a value to a key with key
        let serialized_key: Option<IndexedDBKeyType>;

        if !key.is_undefined() {
            serialized_key = Some(convert_value_to_key(cx, key, None)?);
        } else {
            // Step 11: We should use in-line keys instead
            if let Some(Ok(ExtractionResult::Key(kpk))) = self
                .key_path
                .as_ref()
                .map(|p| extract_key(cx, value, p, None))
            {
                serialized_key = Some(kpk);
            } else {
                if !self.has_key_generator() {
                    return Err(Error::Data);
                }
                serialized_key = None;
            }
        }

        // Step 10. Let clone be a clone of value in targetRealm during transaction. Rethrow any exceptions.
        let cloned_value = structuredclone::write(cx, value, None)?;
        let Ok(serialized_value) = bincode::serialize(&cloned_value) else {
            return Err(Error::InvalidState);
        };

        let (sender, receiver) = indexed_db::create_channel(self.global());

        // Step 12. Let operation be an algorithm to run store a record into an object store with store, clone, key, and no-overwrite flag.
        // Step 13. Return the result (an IDBRequest) of running asynchronously execute a request with handle and operation.
        IDBRequest::execute_async(
            self,
            AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                sender,
                key: serialized_key,
                value: serialized_value,
                should_overwrite: overwrite,
            }),
            receiver,
            None,
            can_gc,
        )
    }
}

impl IDBObjectStoreMethods<crate::DomTypeHolder> for IDBObjectStore {
    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-put>
    fn Put(
        &self,
        cx: SafeJSContext,
        value: HandleValue,
        key: HandleValue,
    ) -> Fallible<DomRoot<IDBRequest>> {
        self.put(cx, value, key, true, CanGc::note())
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-add>
    fn Add(
        &self,
        cx: SafeJSContext,
        value: HandleValue,
        key: HandleValue,
    ) -> Fallible<DomRoot<IDBRequest>> {
        self.put(cx, value, key, false, CanGc::note())
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-delete>
    fn Delete(&self, cx: SafeJSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1. Let transaction be this’s transaction.
        // Step 2. Let store be this's object store.
        // Step 3. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        // Step 5. If transaction is a read-only transaction, throw a "ReadOnlyError" DOMException.
        self.check_readwrite_transaction_active()?;

        // Step 6
        // TODO: Convert to key range instead
        let serialized_query = convert_value_to_key(cx, query, None);
        // Step 7. Let operation be an algorithm to run delete records from an object store with store and range.
        // Stpe 8. Return the result (an IDBRequest) of running asynchronously execute a request with this and operation.
        let (sender, receiver) = indexed_db::create_channel(self.global());
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                AsyncOperation::ReadWrite(AsyncReadWriteOperation::RemoveItem { sender, key: q }),
                receiver,
                None,
                CanGc::note(),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-clear>
    fn Clear(&self) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1. Let transaction be this’s transaction.
        // Step 2. Let store be this's object store.
        // Step 3. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        // Step 5. If transaction is a read-only transaction, throw a "ReadOnlyError" DOMException.
        self.check_readwrite_transaction_active()?;

        // Step 6. Let operation be an algorithm to run clear an object store with store.
        // Stpe 7. Return the result (an IDBRequest) of running asynchronously execute a request with this and operation.
        let (sender, receiver) = indexed_db::create_channel(self.global());

        IDBRequest::execute_async(
            self,
            AsyncOperation::ReadWrite(AsyncReadWriteOperation::Clear(sender)),
            receiver,
            None,
            CanGc::note(),
        )
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-get>
    fn Get(&self, cx: SafeJSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1. Let transaction be this’s transaction.
        // Step 2. Let store be this's object store.
        // Step 3. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 5. Let range be the result of converting a value to a key range with query and true. Rethrow any exceptions.
        let serialized_query = convert_value_to_key_range(cx, query, None);

        // Step 6. Let operation be an algorithm to run retrieve a value from an object store with the current Realm record, store, and range.
        // Step 7. Return the result (an IDBRequest) of running asynchronously execute a request with this and operation.
        let (sender, receiver) = indexed_db::create_channel(self.global());
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem {
                    sender,
                    key_range: q,
                }),
                receiver,
                None,
                CanGc::note(),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-getkey>
    fn GetKey(&self, cx: SafeJSContext, query: HandleValue) -> Result<DomRoot<IDBRequest>, Error> {
        // Step 1. Let transaction be this’s transaction.
        // Step 2. Let store be this's object store.
        // Step 3. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 5. Let range be the result of running the steps to convert a value to a key range with query and null disallowed flag set. Rethrow any exceptions.
        let serialized_query = convert_value_to_key_range(cx, query, None);

        // Step 6. Run the steps to asynchronously execute a request and return the IDBRequest created by these steps.
        // The steps are run with this object store handle as source and the steps to retrieve a key from an object
        // store as operation, using store and range.
        let (sender, receiver) = indexed_db::create_channel(self.global());
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetKey {
                    sender,
                    key_range: q,
                }),
                receiver,
                None,
                CanGc::note(),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-getall>
    fn GetAll(
        &self,
        cx: SafeJSContext,
        query: HandleValue,
        count: Option<u32>,
    ) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1. Let transaction be this’s transaction.
        // Step 2. Let store be this's object store.
        // Step 3. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 5. Let range be the result of running the steps to convert a value to a key range with query and null disallowed flag set. Rethrow any exceptions.
        let serialized_query = convert_value_to_key_range(cx, query, None);

        // Step 6. Run the steps to asynchronously execute a request and return the IDBRequest created by these steps.
        // The steps are run with this object store handle as source and the steps to retrieve a key from an object
        // store as operation, using store and range.
        let (sender, receiver) = indexed_db::create_channel(self.global());
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetAllItems {
                    sender,
                    key_range: q,
                    count,
                }),
                receiver,
                None,
                CanGc::note(),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-getallkeys>
    fn GetAllKeys(
        &self,
        cx: SafeJSContext,
        query: HandleValue,
        count: Option<u32>,
    ) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1. Let transaction be this’s transaction.
        // Step 2. Let store be this's object store.
        // Step 3. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 5. Let range be the result of running the steps to convert a value to a key range with query and null disallowed flag set. Rethrow any exceptions.
        let serialized_query = convert_value_to_key_range(cx, query, None);

        // Step 6. Run the steps to asynchronously execute a request and return the IDBRequest created by these steps.
        // The steps are run with this object store handle as source and the steps to retrieve a key from an object
        // store as operation, using store and range.
        let (sender, receiver) = indexed_db::create_channel(self.global());
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetAllKeys {
                    sender,
                    key_range: q,
                    count,
                }),
                receiver,
                None,
                CanGc::note(),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-count>
    fn Count(&self, cx: SafeJSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1. Let transaction be this’s transaction.
        // Step 2. Let store be this's object store.
        // Step 3. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 5. Let range be the result of converting a value to a key range with query. Rethrow any exceptions.
        let serialized_query = convert_value_to_key_range(cx, query, None);

        // Step 6. Let operation be an algorithm to run count the records in a range with store and range.
        // Step 7. Return the result (an IDBRequest) of running asynchronously execute a request with this and operation.
        let (sender, receiver) = indexed_db::create_channel(self.global());
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Count {
                    sender,
                    key_range: q,
                }),
                receiver,
                None,
                CanGc::note(),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-name>
    fn Name(&self) -> DOMString {
        self.name.borrow().clone()
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-setname>
    fn SetName(&self, value: DOMString) -> ErrorResult {
        // Step 2. Let transaction be this’s transaction.
        let transaction = &self.transaction;

        // Step 3. Let store be this's object store.
        // Step 4. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 5. If transaction is not an upgrade transaction, throw an "InvalidStateError" DOMException.
        if transaction.Mode() != IDBTransactionMode::Versionchange {
            return Err(Error::InvalidState);
        }
        // Step 6. If transaction’s state is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        *self.name.borrow_mut() = value;
        Ok(())
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-keypath
    // fn KeyPath(&self, _cx: SafeJSContext, _val: MutableHandleValue) {
    //     unimplemented!();
    // }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-indexnames
    // fn IndexNames(&self) -> DomRoot<DOMStringList> {
    //     unimplemented!();
    // }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-transaction>
    fn Transaction(&self) -> DomRoot<IDBTransaction> {
        self.transaction()
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-autoincrement>
    fn AutoIncrement(&self) -> bool {
        self.has_key_generator()
    }
}
