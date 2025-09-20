/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use js::gc::MutableHandleValue;
use js::jsval::NullValue;
use js::rust::HandleValue;
use net_traits::IpcSend;
use net_traits::indexeddb_thread::{
    AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, IndexedDBKeyType,
    IndexedDBThreadMsg, SyncOperation,
};
use profile_traits::ipc;
use script_bindings::codegen::GenericBindings::IDBObjectStoreBinding::IDBIndexParameters;
use script_bindings::codegen::GenericUnionTypes::StringOrStringSequence;
use script_bindings::conversions::SafeToJSValConvertible;
use script_bindings::error::ErrorResult;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBCursorBinding::IDBCursorDirection;
use crate::dom::bindings::codegen::Bindings::IDBDatabaseBinding::IDBObjectStoreParameters;
use crate::dom::bindings::codegen::Bindings::IDBObjectStoreBinding::IDBObjectStoreMethods;
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::{
    IDBTransactionMethods, IDBTransactionMode,
};
// We need to alias this name, otherwise test-tidy complains at &String reference.
use crate::dom::bindings::codegen::UnionTypes::StringOrStringSequence as StrOrStringSequence;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone;
use crate::dom::domstringlist::DOMStringList;
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbcursor::{IDBCursor, IterationParam, ObjectStoreOrIndex};
use crate::dom::idbcursorwithvalue::IDBCursorWithValue;
use crate::dom::idbindex::IDBIndex;
use crate::dom::idbrequest::IDBRequest;
use crate::dom::idbtransaction::IDBTransaction;
use crate::indexed_db::{
    self, ExtractionResult, convert_value_to_key, convert_value_to_key_range, extract_key,
};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub enum KeyPath {
    String(DOMString),
    StringSequence(Vec<DOMString>),
}

impl From<StringOrStringSequence> for KeyPath {
    fn from(value: StringOrStringSequence) -> Self {
        match value {
            StringOrStringSequence::String(s) => KeyPath::String(s),
            StringOrStringSequence::StringSequence(ss) => KeyPath::StringSequence(ss),
        }
    }
}

impl From<net_traits::indexeddb_thread::KeyPath> for KeyPath {
    fn from(value: net_traits::indexeddb_thread::KeyPath) -> Self {
        match value {
            net_traits::indexeddb_thread::KeyPath::String(s) => {
                KeyPath::String(DOMString::from_string(s))
            },
            net_traits::indexeddb_thread::KeyPath::Sequence(ss) => {
                KeyPath::StringSequence(ss.into_iter().map(DOMString::from_string).collect())
            },
        }
    }
}

impl From<KeyPath> for net_traits::indexeddb_thread::KeyPath {
    fn from(item: KeyPath) -> Self {
        match item {
            KeyPath::String(s) => Self::String(s.to_string()),
            KeyPath::StringSequence(ss) => {
                Self::Sequence(ss.into_iter().map(|s| s.to_string()).collect())
            },
        }
    }
}

#[dom_struct]
pub struct IDBObjectStore {
    reflector_: Reflector,
    name: DomRefCell<DOMString>,
    key_path: Option<KeyPath>,
    index_names: DomRefCell<Vec<DOMString>>,
    transaction: Dom<IDBTransaction>,

    // We store the db name in the object store to be able to find the correct
    // store in the idb thread when checking if we have a key generator
    db_name: DOMString,
}

impl IDBObjectStore {
    pub fn new_inherited(
        _global: &GlobalScope,
        db_name: DOMString,
        name: DOMString,
        options: Option<&IDBObjectStoreParameters>,
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
            index_names: DomRefCell::new(vec![]),
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

    fn get_idb_thread(&self) -> IpcSender<IndexedDBThreadMsg> {
        self.global().resource_threads().sender()
    }

    fn has_key_generator(&self) -> bool {
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();

        let operation = SyncOperation::HasKeyGenerator(
            sender,
            self.global().origin().immutable().clone(),
            self.db_name.to_string(),
            self.name.borrow().to_string(),
        );

        self.get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(operation))
            .unwrap();

        // First unwrap for ipc
        // Second unwrap will never happen unless this db gets manually deleted somehow
        receiver.recv().unwrap().unwrap()
    }

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
            serialized_key = Some(convert_value_to_key(cx, key, None)?.into_result()?);
        } else {
            // Step 11: We should use in-line keys instead
            // Step 11.1: Let kpk be the result of running the steps to extract a
            // key from a value using a key path with clone and store’s key path.
            let extraction_result = self
                .key_path
                .as_ref()
                .map(|p| extract_key(cx, value, p, None));

            match extraction_result {
                Some(Ok(ExtractionResult::Failure)) | None => {
                    // Step 11.4. Otherwise:
                    // Step 11.4.1. If store does not have a key generator, throw
                    // a "DataError" DOMException.
                    if !self.has_key_generator() {
                        return Err(Error::Data);
                    }
                    // Stept 11.4.2. Otherwise, if the steps to check that a key could
                    // be injected into a value with clone and store’s key path return
                    // false, throw a "DataError" DOMException.
                    // TODO
                    serialized_key = None;
                },
                // Step 11.1. Rethrow any exceptions.
                Some(extraction_result) => match extraction_result? {
                    // Step 11.2. If kpk is invalid, throw a "DataError" DOMException.
                    ExtractionResult::Invalid => return Err(Error::Data),
                    // Step 11.3. If kpk is not failure, let key be kpk.
                    ExtractionResult::Key(kpk) => serialized_key = Some(kpk),
                    ExtractionResult::Failure => unreachable!(),
                },
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
            None,
            can_gc,
        )
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-opencursor>
    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-openkeycursor>
    fn open_cursor(
        &self,
        cx: SafeJSContext,
        query: HandleValue,
        direction: IDBCursorDirection,
        key_only: bool,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1. Let transaction be this object store handle's transaction.
        // Step 2. Let store be this object store handle's object store.

        // Step 3. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 5. Let range be the result of running the steps to convert a value to a key range
        // with query. Rethrow any exceptions.
        //
        // The query parameter may be a key or an IDBKeyRange to use as the cursor's range. If null
        // or not given, an unbounded key range is used.
        let range = convert_value_to_key_range(cx, query, Some(false))?;

        // Step 6. Let cursor be a new cursor with transaction set to transaction, an undefined
        // position, direction set to direction, got value flag unset, and undefined key and value.
        // The source of cursor is store. The range of cursor is range.
        //
        // NOTE: A cursor that has the key only flag unset implements the IDBCursorWithValue
        // interface as well.
        let cursor = if key_only {
            IDBCursor::new(
                &self.global(),
                &self.transaction,
                direction,
                false,
                ObjectStoreOrIndex::ObjectStore(Dom::from_ref(self)),
                range.clone(),
                key_only,
                can_gc,
            )
        } else {
            DomRoot::upcast(IDBCursorWithValue::new(
                &self.global(),
                &self.transaction,
                direction,
                false,
                ObjectStoreOrIndex::ObjectStore(Dom::from_ref(self)),
                range.clone(),
                key_only,
                can_gc,
            ))
        };

        // Step 7. Run the steps to asynchronously execute a request and return the IDBRequest
        // created by these steps. The steps are run with this object store handle as source and
        // the steps to iterate a cursor as operation, using the current Realm as targetRealm, and
        // cursor.
        let iteration_param = IterationParam {
            cursor: Trusted::new(&cursor),
            key: None,
            primary_key: None,
            count: None,
        };
        let (sender, receiver) = indexed_db::create_channel(self.global());
        IDBRequest::execute_async(
            self,
            AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Iterate {
                sender,
                key_range: range,
            }),
            receiver,
            None,
            Some(iteration_param),
            can_gc,
        )
        .inspect(|request| cursor.set_request(request))
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
        let serialized_query = convert_value_to_key(cx, query, None)?.into_result();
        // Step 7. Let operation be an algorithm to run delete records from an object store with store and range.
        // Stpe 8. Return the result (an IDBRequest) of running asynchronously execute a request with this and operation.
        let (sender, receiver) = indexed_db::create_channel(self.global());
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                AsyncOperation::ReadWrite(AsyncReadWriteOperation::RemoveItem { sender, key: q }),
                receiver,
                None,
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
                None,
                CanGc::note(),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-opencursor>
    fn OpenCursor(
        &self,
        cx: SafeJSContext,
        query: HandleValue,
        direction: IDBCursorDirection,
    ) -> Fallible<DomRoot<IDBRequest>> {
        self.open_cursor(cx, query, direction, false, CanGc::note())
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-openkeycursor>
    fn OpenKeyCursor(
        &self,
        cx: SafeJSContext,
        query: HandleValue,
        direction: IDBCursorDirection,
    ) -> Fallible<DomRoot<IDBRequest>> {
        self.open_cursor(cx, query, direction, true, CanGc::note())
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
    fn KeyPath(&self, cx: SafeJSContext, mut ret_val: MutableHandleValue) {
        match &self.key_path {
            Some(KeyPath::String(path)) => path.safe_to_jsval(cx, ret_val),
            Some(KeyPath::StringSequence(paths)) => paths.safe_to_jsval(cx, ret_val),
            None => ret_val.set(NullValue()),
        }
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-indexnames>
    fn IndexNames(&self) -> DomRoot<DOMStringList> {
        DOMStringList::new(&self.global(), self.index_names.borrow().clone(), CanGc::note())
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-transaction>
    fn Transaction(&self) -> DomRoot<IDBTransaction> {
        self.transaction()
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-autoincrement>
    fn AutoIncrement(&self) -> bool {
        self.has_key_generator()
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-createindex>
    fn CreateIndex(
        &self,
        name: DOMString,
        key_path: StringOrStringSequence,
        options: &IDBIndexParameters,
    ) -> Fallible<DomRoot<IDBIndex>> {
        let key_path: KeyPath = key_path.into();
        // Step 3. If transaction is not an upgrade transaction, throw an "InvalidStateError" DOMException.
        if self.transaction.Mode() != IDBTransactionMode::Versionchange {
            return Err(Error::InvalidState);
        }
        // TODO: Step 4. If store has been deleted, throw an "InvalidStateError" DOMException.
        // Step 5. If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;
        // Step 6. If an index named name already exists in store, throw a "ConstraintError" DOMException.
        if self.index_names.borrow().contains(&name) {
            return Err(Error::Constraint);
        }
        // TODO: Step 7. If keyPath is not a valid key path, throw a "SyntaxError" DOMException.
        // TODO: Step 8. Let unique be set if options’s unique member is true, and unset otherwise.
        // TODO: Step 9. Let multiEntry be set if options’s multiEntry member is true, and unset otherwise.
        // Step 10. If keyPath is a sequence and multiEntry is set, throw an "InvalidAccessError" DOMException.
        if matches!(key_path, KeyPath::StringSequence(_)) && options.multiEntry {
            return Err(Error::InvalidAccess);
        }
        // Step 11. Let index be a new index in store.
        // Set index’s name to name and key path to keyPath. If unique is set, set index’s unique flag.
        // If multiEntry is set, set index’s multiEntry flag.
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let create_index_operation = SyncOperation::CreateIndex(
            sender,
            self.global().origin().immutable().clone(),
            self.db_name.to_string(),
            self.name.borrow().to_string(),
            name.to_string(),
            key_path.clone().into(),
            options.unique,
            options.multiEntry,
        );
        if self
            .get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(create_index_operation))
            .is_err()
        {
            return Err(Error::Operation);
        }
        let result = receiver.recv().unwrap();
        if result.is_err() {
            return Err(Error::Operation);
        }
        // Step 12. Add index to this object store handle's index set.
        self.index_names.borrow_mut().push(name.clone());
        // Step 13. Return a new index handle associated with index and this object store handle.
        Ok(IDBIndex::new(
            &self.global(),
            DomRoot::from_ref(self),
            name,
            options.multiEntry,
            options.unique,
            CanGc::note(),
        ))
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-deleteindex>
    fn DeleteIndex(&self, name: DOMString) -> Fallible<()> {
        // Step 3. If transaction is not an upgrade transaction, throw an "InvalidStateError" DOMException.
        if self.transaction.Mode() != IDBTransactionMode::Versionchange {
            return Err(Error::InvalidState);
        }
        // TODO: Step 4. If store has been deleted, throw an "InvalidStateError" DOMException.
        // Step 5. If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;
        // Step 6. Let index be the index named name in store if one exists,
        // or throw a "NotFoundError" DOMException otherwise.
        if !self.index_names.borrow().contains(&name) {
            return Err(Error::NotFound);
        }
        // Step 7. Remove index from this object store handle's index set.
        self.index_names.borrow_mut().retain(|n| n != &name);
        // Step 8. Destroy index.
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let delete_index_operation = SyncOperation::DeleteIndex(
            sender,
            self.global().origin().immutable().clone(),
            self.db_name.to_string(),
            self.name.borrow().to_string(),
            name.to_string(),
        );
        self.get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(delete_index_operation))
            .unwrap();
        let result = receiver.recv().unwrap();
        if result.is_err() {
            return Err(Error::Operation);
        }
        Ok(())
    }
}
