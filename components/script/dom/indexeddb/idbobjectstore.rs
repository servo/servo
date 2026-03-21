/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::Cell;
use std::collections::HashMap;

use base::generic_channel::{GenericSend, GenericSender};
use dom_struct::dom_struct;
use js::context::JSContext;
use js::conversions::ToJSValConvertible;
use js::gc::MutableHandleValue;
use js::jsval::NullValue;
use js::rust::HandleValue;
use script_bindings::codegen::GenericBindings::IDBObjectStoreBinding::IDBIndexParameters;
use script_bindings::codegen::GenericUnionTypes::StringOrStringSequence;
use script_bindings::error::ErrorResult;
use storage_traits::indexeddb::{
    self, AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, IndexedDBKeyType,
    IndexedDBThreadMsg, SyncOperation,
};

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
use crate::dom::indexeddb::idbcursor::{IDBCursor, IterationParam, ObjectStoreOrIndex};
use crate::dom::indexeddb::idbcursorwithvalue::IDBCursorWithValue;
use crate::dom::indexeddb::idbindex::IDBIndex;
use crate::dom::indexeddb::idbrequest::IDBRequest;
use crate::dom::indexeddb::idbtransaction::IDBTransaction;
use crate::indexeddb::{
    ExtractionResult, can_inject_key_into_value, convert_value_to_key, convert_value_to_key_range,
    extract_key, inject_key_into_value, is_valid_key_path,
};
use crate::script_runtime::CanGc;

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

impl From<indexeddb::KeyPath> for KeyPath {
    fn from(value: indexeddb::KeyPath) -> Self {
        match value {
            indexeddb::KeyPath::String(string) => KeyPath::String(string.into()),
            indexeddb::KeyPath::Sequence(ss) => {
                KeyPath::StringSequence(ss.into_iter().map(Into::into).collect())
            },
        }
    }
}

impl From<KeyPath> for indexeddb::KeyPath {
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
    index_set: DomRefCell<HashMap<DOMString, Dom<IDBIndex>>>,
    transaction: Dom<IDBTransaction>,
    has_key_generator: bool,
    key_generator_current_number: Cell<Option<i32>>,

    // We store the db name in the object store to address backend operations
    // that are keyed by (origin, database name, object store name).
    db_name: DOMString,
}

impl IDBObjectStore {
    pub fn new_inherited(
        db_name: DOMString,
        name: DOMString,
        options: Option<&IDBObjectStoreParameters>,
        key_generator_current_number: Option<i32>,
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
        let has_key_generator = options.is_some_and(|options| options.autoIncrement);
        let key_generator_current_number = if has_key_generator {
            Some(key_generator_current_number.unwrap_or(1))
        } else {
            None
        };

        IDBObjectStore {
            reflector_: Reflector::new(),
            name: DomRefCell::new(name),
            key_path,
            index_set: DomRefCell::new(HashMap::new()),
            transaction: Dom::from_ref(transaction),
            has_key_generator,
            key_generator_current_number: Cell::new(key_generator_current_number),
            db_name,
        }
    }

    pub fn new(
        global: &GlobalScope,
        db_name: DOMString,
        name: DOMString,
        options: Option<&IDBObjectStoreParameters>,
        key_generator_current_number: Option<i32>,
        can_gc: CanGc,
        transaction: &IDBTransaction,
    ) -> DomRoot<IDBObjectStore> {
        reflect_dom_object(
            Box::new(IDBObjectStore::new_inherited(
                db_name,
                name,
                options,
                key_generator_current_number,
                transaction,
            )),
            global,
            can_gc,
        )
    }

    pub fn get_name(&self) -> DOMString {
        self.name.borrow().clone()
    }

    pub(crate) fn transaction(&self) -> DomRoot<IDBTransaction> {
        self.transaction.as_rooted()
    }

    fn get_idb_thread(&self) -> GenericSender<IndexedDBThreadMsg> {
        self.global().storage_threads().sender()
    }

    fn has_key_generator(&self) -> bool {
        self.has_key_generator
    }

    /// <https://w3c.github.io/IndexedDB/#generate-a-key>
    fn generate_key_for_put(&self) -> Fallible<(IndexedDBKeyType, i32)> {
        // Step 1. Let generator be store's key generator.
        let Some(current_number) = self.key_generator_current_number.get() else {
            return Err(Error::Data(None));
        };
        // Step 2. Let key be generator's current number.
        let key = current_number as f64;
        // Step 3. If key is greater than 2^53 (9007199254740992), then return failure.
        if key > 9_007_199_254_740_992.0 {
            return Err(Error::Constraint(None));
        }
        // Step 4. Increase generator's current number by 1.
        let next_current_number = current_number
            .checked_add(1)
            .ok_or(Error::Constraint(None))?;
        // Step 5. Return key.
        Ok((IndexedDBKeyType::Number(key), next_current_number))
    }

    /// <https://w3c.github.io/IndexedDB/#possibly-update-the-key-generator>
    fn possibly_update_the_key_generator(&self, key: &IndexedDBKeyType) -> Option<i32> {
        // Step 1. If the type of key is not number, abort these steps.
        let IndexedDBKeyType::Number(number) = key else {
            return None;
        };

        // Step 2. Let value be the value of key.
        let mut value = *number;
        // Step 3. Set value to the minimum of value and 2^53 (9007199254740992).
        value = value.min(9_007_199_254_740_992.0);
        // Step 4. Set value to the largest integer not greater than value.
        value = value.floor();
        // Step 5. Let generator be store's key generator.
        let current_number = self.key_generator_current_number.get()?;
        // Step 6. If value is greater than or equal to generator's current number,
        // then set generator's current number to value + 1.
        if value < current_number as f64 {
            return None;
        }

        let next = value + 1.0;
        // Servo currently stores the key generator current number as i32.
        // Saturate to keep "no more generated keys" behavior when this overflows.
        if next >= i32::MAX as f64 {
            return Some(i32::MAX);
        }
        Some(next as i32)
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#object-store-in-line-keys>
    fn uses_inline_keys(&self) -> bool {
        self.key_path.is_some()
    }

    fn verify_not_deleted(&self) -> ErrorResult {
        let db = self.transaction.Db();
        if !db.object_store_exists(&self.name.borrow()) {
            return Err(Error::InvalidState(None));
        }
        Ok(())
    }

    /// Checks if the transaction is active, throwing a "TransactionInactiveError" DOMException if not.
    fn check_transaction_active(&self) -> Fallible<()> {
        // Let transaction be this object store handle's transaction.
        let transaction = &self.transaction;

        // If transaction is not active, throw a "TransactionInactiveError" DOMException.
        // https://w3c.github.io/IndexedDB/#transaction-inactive
        // A transaction is in this state after control returns to the event loop after its creation, and when events are not being dispatched.
        // No requests can be made against the transaction when it is in this state.
        if !transaction.is_active() || !transaction.is_usable() {
            return Err(Error::TransactionInactive(None));
        }

        Ok(())
    }

    /// Checks if the transaction is active, throwing a "TransactionInactiveError" DOMException if not.
    /// it then checks if the transaction is a read-only transaction, throwing a "ReadOnlyError" DOMException if so.
    fn check_readwrite_transaction_active(&self) -> Fallible<()> {
        // Let transaction be this object store handle's transaction.
        let transaction = &self.transaction;

        // If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        if let IDBTransactionMode::Readonly = transaction.get_mode() {
            return Err(Error::ReadOnly(None));
        }
        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-put>
    fn put(
        &self,
        cx: &mut JSContext,
        value: HandleValue,
        key: HandleValue,
        overwrite: bool,
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
            return Err(Error::Data(None));
        }

        // Step 7: If store uses out-of-line keys and has no key generator
        // and key was not given, throw a "DataError" DOMException.
        if !self.uses_inline_keys() && !self.has_key_generator() && key.is_undefined() {
            return Err(Error::Data(None));
        }

        // Step 8. If key was given, then: convert a value to a key with key.
        let serialized_key: Option<IndexedDBKeyType>;
        let maybe_modified_cloned_value;
        let key_generator_current_number_for_put: Option<i32>;

        if !key.is_undefined() {
            let key = convert_value_to_key(cx, key, None)?.into_result()?;
            key_generator_current_number_for_put = self.possibly_update_the_key_generator(&key);
            serialized_key = Some(key);
            maybe_modified_cloned_value = None;
        } else {
            match self.key_path.as_ref() {
                Some(key_path) => {
                    // Step 10. Let clone be a clone of value in targetRealm during transaction.
                    // Rethrow any exceptions.
                    let cloned_value = structuredclone::write(cx.into(), value, None)?;

                    rooted!(&in(cx) let mut cloned_js_value = NullValue());
                    let _ = structuredclone::read(
                        &self.global(),
                        cloned_value,
                        cloned_js_value.handle_mut(),
                        CanGc::from_cx(cx),
                    )?;

                    // TODO: Avoid this deserialize/re-serialize round-trip once extract/inject can
                    // operate directly on the structured clone payload.

                    // Step 11: We should use in-line keys instead
                    // Step 11.1: Let kpk be the result of running the steps to extract a
                    // key from a value using a key path with clone and store’s key path.
                    match extract_key(cx, cloned_js_value.handle(), key_path, None)? {
                        // Step 11.2. If kpk is invalid, throw a "DataError" DOMException.
                        ExtractionResult::Invalid => return Err(Error::Data(None)),
                        // Step 11.3. If kpk is not failure, let key be kpk.
                        ExtractionResult::Key(kpk) => {
                            key_generator_current_number_for_put =
                                self.possibly_update_the_key_generator(&kpk);
                            serialized_key = Some(kpk);
                        },
                        ExtractionResult::Failure => {
                            // Step 11.4. Otherwise:
                            // Step 11.4.1. If store does not have a key generator, throw
                            // a "DataError" DOMException.
                            if !self.has_key_generator() {
                                return Err(Error::Data(None));
                            }
                            let KeyPath::String(key_path) = key_path else {
                                return Err(Error::Data(None));
                            };
                            // Step 11.4.2. If the steps to check that a key could be injected
                            // into a value with clone and store’s key path return false, throw a
                            // "DataError" DOMException.
                            if !can_inject_key_into_value(cx, cloned_js_value.handle(), key_path)? {
                                return Err(Error::Data(None));
                            }
                            // Step 11.4.3. Let key be the result of generating a key for store.
                            let (generated_key, next_current_number) =
                                self.generate_key_for_put()?;
                            // Step 11.4.4. Inject key into value.
                            if !inject_key_into_value(
                                cx,
                                cloned_js_value.handle(),
                                &generated_key,
                                key_path,
                            )? {
                                return Err(Error::Data(None));
                            }
                            serialized_key = Some(generated_key);
                            key_generator_current_number_for_put = Some(next_current_number);
                        },
                    }

                    // Store the clone (possibly with an injected key path), without mutating
                    // the original JS value that was passed to add()/put().
                    maybe_modified_cloned_value = Some(structuredclone::write(
                        cx.into(),
                        cloned_js_value.handle(),
                        None,
                    )?);
                },
                None => {
                    if !self.has_key_generator() {
                        return Err(Error::Data(None));
                    }
                    // Out-of-line key generation happens in the backend as part of executing
                    // the put request, so script does not reserve a key here.
                    serialized_key = None;
                    maybe_modified_cloned_value = None;
                    key_generator_current_number_for_put = None;
                },
            }
        }
        // For paths that did not need key-path extraction/injection above, clone `value` here.
        let cloned_value = match maybe_modified_cloned_value {
            Some(cloned_value) => cloned_value,
            None => structuredclone::write(cx.into(), value, None)?,
        };
        let Ok(serialized_value) = postcard::to_stdvec(&cloned_value) else {
            return Err(Error::InvalidState(None));
        };
        // Step 12. Let operation be an algorithm to run store a record into an object store with store, clone, key, and no-overwrite flag.
        // Step 13. Return the result (an IDBRequest) of running asynchronously execute a request with handle and operation.
        let request = IDBRequest::execute_async(
            self,
            |callback| {
                AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                    callback,
                    key: serialized_key,
                    value: serialized_value,
                    should_overwrite: overwrite,
                    key_generator_current_number: key_generator_current_number_for_put,
                })
            },
            None,
            None,
            CanGc::from_cx(cx),
        )?;
        if let Some(next_key_generator_current_number) = key_generator_current_number_for_put {
            self.key_generator_current_number
                .set(Some(next_key_generator_current_number));
        }
        Ok(request)
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-opencursor>
    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-openkeycursor>
    fn open_cursor(
        &self,
        cx: &mut JSContext,
        query: HandleValue,
        direction: IDBCursorDirection,
        key_only: bool,
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
                CanGc::from_cx(cx),
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
                CanGc::from_cx(cx),
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

        IDBRequest::execute_async(
            self,
            |callback| {
                AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Iterate {
                    callback,
                    key_range: range,
                })
            },
            None,
            Some(iteration_param),
            CanGc::from_cx(cx),
        )
        .inspect(|request| cursor.set_request(request))
    }

    pub(crate) fn add_index(
        &self,
        name: DOMString,
        options: &IDBIndexParameters,
        key_path: KeyPath,
        can_gc: CanGc,
    ) -> DomRoot<IDBIndex> {
        let index = IDBIndex::new(
            &self.global(),
            DomRoot::from_ref(self),
            name.clone(),
            options.multiEntry,
            options.unique,
            key_path,
            can_gc,
        );
        self.index_set
            .borrow_mut()
            .insert(name, Dom::from_ref(&index));
        index
    }
}

impl IDBObjectStoreMethods<crate::DomTypeHolder> for IDBObjectStore {
    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-put>
    fn Put(
        &self,
        cx: &mut JSContext,
        value: HandleValue,
        key: HandleValue,
    ) -> Fallible<DomRoot<IDBRequest>> {
        self.put(cx, value, key, true)
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-add>
    fn Add(
        &self,
        cx: &mut JSContext,
        value: HandleValue,
        key: HandleValue,
    ) -> Fallible<DomRoot<IDBRequest>> {
        self.put(cx, value, key, false)
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-delete>
    fn Delete(&self, cx: &mut JSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1. Let transaction be this’s transaction.
        // Step 2. Let store be this's object store.
        // Step 3. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        // Step 5. If transaction is a read-only transaction, throw a "ReadOnlyError" DOMException.
        self.check_readwrite_transaction_active()?;

        // Step 6. Let range be the result of running the steps to convert a value to a key range with query and null disallowed flag set. Rethrow any exceptions.
        let serialized_query = convert_value_to_key_range(cx, query, Some(true));
        // Step 7. Let operation be an algorithm to run delete records from an object store with store and range.
        // Step 8. Return the result (an IDBRequest) of running asynchronously execute a request with this and operation.
        serialized_query.and_then(|key_range| {
            IDBRequest::execute_async(
                self,
                |callback| {
                    AsyncOperation::ReadWrite(AsyncReadWriteOperation::RemoveItem {
                        callback,
                        key_range,
                    })
                },
                None,
                None,
                CanGc::from_cx(cx),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-clear>
    fn Clear(&self, cx: &mut JSContext) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1. Let transaction be this’s transaction.
        // Step 2. Let store be this's object store.
        // Step 3. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        // Step 5. If transaction is a read-only transaction, throw a "ReadOnlyError" DOMException.
        self.check_readwrite_transaction_active()?;

        // Step 6. Let operation be an algorithm to run clear an object store with store.
        // Step 7. Return the result (an IDBRequest) of running asynchronously execute a request with this and operation.
        IDBRequest::execute_async(
            self,
            |callback| AsyncOperation::ReadWrite(AsyncReadWriteOperation::Clear(callback)),
            None,
            None,
            CanGc::from_cx(cx),
        )
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-get>
    fn Get(&self, cx: &mut JSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
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
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                |callback| {
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem {
                        callback,
                        key_range: q,
                    })
                },
                None,
                None,
                CanGc::from_cx(cx),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-getkey>
    fn GetKey(&self, cx: &mut JSContext, query: HandleValue) -> Result<DomRoot<IDBRequest>, Error> {
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
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                |callback| {
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetKey {
                        callback,
                        key_range: q,
                    })
                },
                None,
                None,
                CanGc::from_cx(cx),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-getall>
    fn GetAll(
        &self,
        cx: &mut JSContext,
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
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                |callback| {
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetAllItems {
                        callback,
                        key_range: q,
                        count,
                    })
                },
                None,
                None,
                CanGc::from_cx(cx),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-getallkeys>
    fn GetAllKeys(
        &self,
        cx: &mut JSContext,
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
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                |callback| {
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetAllKeys {
                        callback,
                        key_range: q,
                        count,
                    })
                },
                None,
                None,
                CanGc::from_cx(cx),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-count>
    fn Count(&self, cx: &mut JSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
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
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                |callback| {
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Count {
                        callback,
                        key_range: q,
                    })
                },
                None,
                None,
                CanGc::from_cx(cx),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-opencursor>
    fn OpenCursor(
        &self,
        cx: &mut JSContext,
        query: HandleValue,
        direction: IDBCursorDirection,
    ) -> Fallible<DomRoot<IDBRequest>> {
        self.open_cursor(cx, query, direction, false)
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-openkeycursor>
    fn OpenKeyCursor(
        &self,
        cx: &mut JSContext,
        query: HandleValue,
        direction: IDBCursorDirection,
    ) -> Fallible<DomRoot<IDBRequest>> {
        self.open_cursor(cx, query, direction, true)
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-name>
    fn Name(&self) -> DOMString {
        self.name.borrow().clone()
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-setname>
    fn SetName(&self, value: DOMString) -> ErrorResult {
        // Step 2. Let transaction be this’s transaction.
        let transaction = &self.transaction;

        // Step 3. Let store be this's object store.
        // Step 4. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 5. If transaction is not an upgrade transaction, throw an "InvalidStateError" DOMException.
        if transaction.Mode() != IDBTransactionMode::Versionchange {
            return Err(Error::InvalidState(None));
        }
        // Step 6. If transaction’s state is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        *self.name.borrow_mut() = value;
        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-keypath>
    fn KeyPath(&self, cx: &mut JSContext, mut ret_val: MutableHandleValue) {
        match &self.key_path {
            Some(KeyPath::String(path)) => path.safe_to_jsval(cx, ret_val),
            Some(KeyPath::StringSequence(paths)) => paths.safe_to_jsval(cx, ret_val),
            None => ret_val.set(NullValue()),
        }
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-indexnames>
    fn IndexNames(&self, can_gc: CanGc) -> DomRoot<DOMStringList> {
        DOMStringList::new_sorted(&self.global(), self.index_set.borrow().keys(), can_gc)
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-transaction>
    fn Transaction(&self) -> DomRoot<IDBTransaction> {
        self.transaction()
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-autoincrement>
    fn AutoIncrement(&self) -> bool {
        self.has_key_generator()
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-createindex>
    fn CreateIndex(
        &self,
        cx: &mut JSContext,
        name: DOMString,
        key_path: StringOrStringSequence,
        options: &IDBIndexParameters,
    ) -> Fallible<DomRoot<IDBIndex>> {
        let key_path: KeyPath = key_path.into();
        // Step 3. If transaction is not an upgrade transaction, throw an "InvalidStateError" DOMException.
        if self.transaction.Mode() != IDBTransactionMode::Versionchange {
            return Err(Error::InvalidState(None));
        }

        // Step 4. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;
        // Step 5. If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 6. If an index named name already exists in store, throw a "ConstraintError" DOMException.
        if self.index_set.borrow().contains_key(&name) {
            return Err(Error::Constraint(None));
        }

        let js_key_path = match key_path.clone() {
            KeyPath::String(s) => StringOrStringSequence::String(s),
            KeyPath::StringSequence(s) => StringOrStringSequence::StringSequence(s),
        };

        // Step 7. If keyPath is not a valid key path, throw a "SyntaxError" DOMException.
        if !is_valid_key_path(cx, &js_key_path)? {
            return Err(Error::Syntax(None));
        }
        // Step 8. Let unique be set if options’s unique member is true, and unset otherwise.
        // Step 9. Let multiEntry be set if options’s multiEntry member is true, and unset otherwise.
        // Step 10. If keyPath is a sequence and multiEntry is set, throw an "InvalidAccessError" DOMException.
        if matches!(key_path, KeyPath::StringSequence(_)) && options.multiEntry {
            return Err(Error::InvalidAccess(None));
        }

        // Step 11. Let index be a new index in store.
        // Set index’s name to name and key path to keyPath. If unique is set, set index’s unique flag.
        // If multiEntry is set, set index’s multiEntry flag.
        let create_index_operation = SyncOperation::CreateIndex(
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
            return Err(Error::Operation(None));
        }

        // Step 12. Add index to this object store handle's index set.
        let index = self.add_index(name, options, key_path, CanGc::from_cx(cx));

        // Step 13. Return a new index handle associated with index and this object store handle.
        Ok(index)
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbobjectstore-deleteindex>
    fn DeleteIndex(&self, name: DOMString) -> Fallible<()> {
        // Step 3. If transaction is not an upgrade transaction, throw an "InvalidStateError" DOMException.
        if self.transaction.Mode() != IDBTransactionMode::Versionchange {
            return Err(Error::InvalidState(None));
        }
        // Step 4. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;
        // Step 5. If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;
        // Step 6. Let index be the index named name in store if one exists,
        // or throw a "NotFoundError" DOMException otherwise.
        if !self.index_set.borrow().contains_key(&name) {
            return Err(Error::NotFound(None));
        }
        // Step 7. Remove index from this object store handle's index set.
        self.index_set.borrow_mut().retain(|n, _| n != &name);
        // Step 8. Destroy index.
        let delete_index_operation = SyncOperation::DeleteIndex(
            self.global().origin().immutable().clone(),
            self.db_name.to_string(),
            self.name.borrow().to_string(),
            name.to_string(),
        );
        self.get_idb_thread()
            .send(IndexedDBThreadMsg::Sync(delete_index_operation))
            .unwrap();
        Ok(())
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-index>
    fn Index(&self, name: DOMString) -> Fallible<DomRoot<IDBIndex>> {
        // Step 3. If store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If the transaction's state is finished, then throw an "InvalidStateError" DOMException.
        if self.transaction.is_finished() {
            return Err(Error::InvalidState(None));
        }

        // Step 5. Let index be the index named name in this’s index set if one exists, or throw a "NotFoundError" DOMException otherwise.
        let index_set = self.index_set.borrow();
        let index = index_set.get(&name).ok_or(Error::NotFound(None))?;

        // Step 6. Return an index handle associated with index and this.
        Ok(index.as_rooted())
    }
}
