/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::gc::HandleValue;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::MutableHandleValue;
use storage_traits::indexeddb_thread::{
    AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, IndexedDBKeyRange,
    IndexedDBKeyType, IndexedDBRecord,
};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBCursorBinding::{
    IDBCursorDirection, IDBCursorMethods,
};
use crate::dom::bindings::codegen::Bindings::IDBIndexBinding::IDBIndexMethods;
use crate::dom::bindings::codegen::Bindings::IDBRequestBinding::IDBRequestReadyState;
use crate::dom::bindings::codegen::UnionTypes::IDBObjectStoreOrIDBIndex;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::structuredclone;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb::idbindex::IDBIndex;
use crate::dom::indexeddb::idbobjectstore::IDBObjectStore;
use crate::dom::indexeddb::idbrequest::IDBRequest;
use crate::dom::indexeddb::idbtransaction::IDBTransaction;
use crate::indexed_db::{
    self, ExtractionResult, convert_value_to_key, extract_key, key_type_to_jsval,
};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[derive(JSTraceable, MallocSizeOf)]
#[expect(unused)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum ObjectStoreOrIndex {
    ObjectStore(Dom<IDBObjectStore>),
    Index(Dom<IDBIndex>),
}

#[dom_struct]
pub(crate) struct IDBCursor {
    reflector_: Reflector,

    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-transaction>
    transaction: Dom<IDBTransaction>,
    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-range>
    #[no_trace]
    range: IndexedDBKeyRange,
    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-source>
    source: ObjectStoreOrIndex,
    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-direction>
    direction: IDBCursorDirection,
    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-position>
    #[no_trace]
    position: DomRefCell<Option<IndexedDBKeyType>>,
    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-key>
    #[no_trace]
    key: DomRefCell<Option<IndexedDBKeyType>>,
    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-value>
    #[ignore_malloc_size_of = "mozjs"]
    value: Heap<JSVal>,
    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-got-value-flag>
    got_value: Cell<bool>,
    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-object-store-position>
    #[no_trace]
    object_store_position: DomRefCell<Option<IndexedDBKeyType>>,
    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-key-only-flag>
    key_only: bool,

    /// <https://w3c.github.io/IndexedDB/#cursor-request>
    request: MutNullableDom<IDBRequest>,
}

impl IDBCursor {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(
        transaction: &IDBTransaction,
        direction: IDBCursorDirection,
        got_value: bool,
        source: ObjectStoreOrIndex,
        range: IndexedDBKeyRange,
        key_only: bool,
    ) -> IDBCursor {
        IDBCursor {
            reflector_: Reflector::new(),
            transaction: Dom::from_ref(transaction),
            range,
            source,
            direction,
            position: DomRefCell::new(None),
            key: DomRefCell::new(None),
            value: Heap::default(),
            got_value: Cell::new(got_value),
            object_store_position: DomRefCell::new(None),
            key_only,
            request: Default::default(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        global: &GlobalScope,
        transaction: &IDBTransaction,
        direction: IDBCursorDirection,
        got_value: bool,
        source: ObjectStoreOrIndex,
        range: IndexedDBKeyRange,
        key_only: bool,
        can_gc: CanGc,
    ) -> DomRoot<IDBCursor> {
        reflect_dom_object(
            Box::new(IDBCursor::new_inherited(
                transaction,
                direction,
                got_value,
                source,
                range,
                key_only,
            )),
            global,
            can_gc,
        )
    }

    fn set_position(&self, position: Option<IndexedDBKeyType>) {
        *self.position.borrow_mut() = position;
    }

    fn set_key(&self, key: Option<IndexedDBKeyType>) {
        *self.key.borrow_mut() = key;
    }

    fn set_object_store_position(&self, object_store_position: Option<IndexedDBKeyType>) {
        *self.object_store_position.borrow_mut() = object_store_position;
    }

    pub(crate) fn set_request(&self, request: &IDBRequest) {
        self.request.set(Some(request));
    }

    pub(crate) fn value(&self, mut out: MutableHandleValue) {
        out.set(self.value.get());
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-effective-object-store>
    fn effective_object_store(&self) -> DomRoot<IDBObjectStore> {
        match &self.source {
            ObjectStoreOrIndex::ObjectStore(object_store) => DomRoot::from_ref(object_store),
            ObjectStoreOrIndex::Index(index) => index.ObjectStore(),
        }
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-effective-key>
    fn effective_key(&self) -> Option<IndexedDBKeyType> {
        match &self.source {
            ObjectStoreOrIndex::ObjectStore(_) => self.position.borrow().clone(),
            ObjectStoreOrIndex::Index(_) => self.object_store_position.borrow().clone(),
        }
    }

    /// If this cursor's transaction is not active, throw a "TransactionInactiveError" DOMException.
    fn check_transaction_active(&self) -> Fallible<()> {
        if !self.transaction.is_active() {
            return Err(Error::TransactionInactive);
        }
        Ok(())
    }

    /// If this cursor's transaction is a read-only transaction, throw a "ReadOnlyError" DOMException.
    fn check_transaction_readwrite(&self) -> Fallible<()> {
        if !self.transaction.is_active() {
            return Err(Error::TransactionInactive);
        }
        Ok(())
    }

    /// If the cursor's source has been deleted, throw an "InvalidStateError" DOMException.
    fn verify_not_deleted(&self) -> ErrorResult {
        match &self.source {
            ObjectStoreOrIndex::ObjectStore(object_store) => object_store.verify_not_deleted(),
            ObjectStoreOrIndex::Index(index) => index.verify_not_deleted(),
        }
    }
}

impl IDBCursorMethods<crate::DomTypeHolder> for IDBCursor {
    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbcursor-source>
    fn Source(&self) -> IDBObjectStoreOrIDBIndex {
        match &self.source {
            ObjectStoreOrIndex::ObjectStore(source) => {
                IDBObjectStoreOrIDBIndex::IDBObjectStore(source.as_rooted())
            },
            ObjectStoreOrIndex::Index(source) => {
                IDBObjectStoreOrIDBIndex::IDBIndex(source.as_rooted())
            },
        }
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbcursor-direction>
    fn Direction(&self) -> IDBCursorDirection {
        self.direction
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbcursor-key>
    fn Key(&self, cx: SafeJSContext, can_gc: CanGc, mut value: MutableHandleValue) {
        match self.key.borrow().as_ref() {
            Some(key) => key_type_to_jsval(cx, key, value, can_gc),
            None => value.set(UndefinedValue()),
        }
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbcursor-primarykey>
    fn PrimaryKey(&self, cx: SafeJSContext, can_gc: CanGc, mut value: MutableHandleValue) {
        match self.effective_key() {
            Some(effective_key) => key_type_to_jsval(cx, &effective_key, value, can_gc),
            None => value.set(UndefinedValue()),
        }
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbcursor-request>
    fn Request(&self) -> DomRoot<IDBRequest> {
        self.request
            .get()
            .expect("IDBCursor.request should be set when cursor is opened")
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbcursor-advance>
    fn Advance(&self, count: u32) -> Fallible<()> {
        // Step 1. If count is 0 (zero), throw a TypeError.
        if count == 0 {
            return Err(Error::Type("count is 0".to_string()));
        }

        // Step 2. Let transaction be this cursor's transaction.
        // Step 3. If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 4. If the cursor’s source or effective object store has been deleted, throw an
        // "InvalidStateError" DOMException.
        self.verify_not_deleted()?;
        self.effective_object_store().verify_not_deleted()?;

        // Step 5. If this cursor’s got value flag is unset, indicating that the cursor is being
        // iterated or has iterated past its end, throw an "InvalidStateError" DOMException.
        if !self.got_value.get() {
            return Err(Error::InvalidState(None));
        }

        // Step 6. Unset the got value flag on the cursor.
        self.got_value.set(false);

        // Step 7. Let request be the request created when this cursor was created.
        let request = self
            .request
            .get()
            .expect("IDBCursor.request should be set when cursor is opened");

        // Step 8. Unset the done flag on request.
        request.set_ready_state(IDBRequestReadyState::Pending);

        // Step 9. Run the steps to asynchronously execute a request with the cursor’s source as
        // source, the steps to iterate a cursor as operation and request, using the current Realm
        // as targetRealm, this cursor and count.
        Err(Error::NotSupported)
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbcursor-continue>
    fn Continue(&self, cx: SafeJSContext, key: HandleValue) -> Fallible<()> {
        // Step 1. Let transaction be this cursor's transaction.
        // Step 2. If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 3. If the cursor’s source or effective object store has been deleted, throw an
        // "InvalidStateError" DOMException.
        self.verify_not_deleted()?;
        self.effective_object_store().verify_not_deleted()?;

        // Step 4. If this cursor’s got value flag is unset, indicating that the cursor is being
        // iterated or has iterated past its end, throw an "InvalidStateError" DOMException.
        if !self.got_value.get() {
            return Err(Error::InvalidState(None));
        }

        // Step 5. If key is given, then:
        let key = if !key.is_null_or_undefined() {
            // Step 5.1. Let r be the result of running the steps to convert a value to a key with
            // key. Rethrow any exceptions.
            let r = convert_value_to_key(cx, key, None)?;

            // Step 5.2. If r is invalid, throw a "DataError" DOMException.
            // Step 5.3. Let key be r.
            let key = r.into_result()?;

            // Step 5.4. If key is less than or equal to this cursor’s position and this cursor’s
            // direction is "next" or "nextunique", throw a "DataError" DOMException.
            if self
                .position
                .borrow()
                .as_ref()
                .is_some_and(|cursor_position| &key <= cursor_position) &&
                matches!(
                    self.direction,
                    IDBCursorDirection::Next | IDBCursorDirection::Nextunique
                )
            {
                return Err(Error::Data);
            }

            // Step 5.5. If key is greater than or equal to this cursor’s position and this
            // cursor’s direction is "prev" or "prevunique", throw a "DataError" DOMException.
            if self
                .position
                .borrow()
                .as_ref()
                .is_some_and(|cursor_position| &key >= cursor_position) &&
                matches!(
                    self.direction,
                    IDBCursorDirection::Prev | IDBCursorDirection::Prevunique
                )
            {
                return Err(Error::Data);
            }

            Some(key)
        } else {
            None
        };

        // Step 6. Unset the got value flag on the cursor.
        self.got_value.set(false);

        // Step 7. Let request be the request created when this cursor was created.
        let request = self
            .request
            .get()
            .expect("IDBCursor.request should be set when cursor is opened");

        // Step 8. Unset the done flag on request.
        request.set_ready_state(IDBRequestReadyState::Pending);

        // Step 9. Run the steps to asynchronously execute a request with the cursor’s source as
        // source, the steps to iterate a cursor as operation and request, using the current Realm
        // as targetRealm, this cursor and key (if given).
        let iteration_param = IterationParam {
            cursor: Trusted::new(self),
            key,
            primary_key: None,
            count: None,
        };
        let (sender, receiver) = indexed_db::create_channel(self.global());
        match &self.source {
            ObjectStoreOrIndex::ObjectStore(object_store) => {
                IDBRequest::execute_async(
                    object_store,
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Iterate {
                        sender,
                        key_range: self.range.clone(),
                    }),
                    receiver,
                    None,
                    Some(iteration_param),
                    CanGc::note(),
                )?;
            },
            ObjectStoreOrIndex::Index(_index) => {
                // TODO: IDBRequest::execute_async currently does not accept using index as source.
                return Err(Error::NotSupported);
            },
        };

        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbcursor-continueprimarykey>
    fn ContinuePrimaryKey(
        &self,
        cx: SafeJSContext,
        key: HandleValue,
        primary_key: HandleValue,
    ) -> Fallible<()> {
        // Step 1. Let transaction be this cursor's transaction.
        // Step 2. If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 3. If the cursor’s source or effective object store has been deleted, throw an
        // "InvalidStateError" DOMException.
        self.verify_not_deleted()?;
        self.effective_object_store().verify_not_deleted()?;

        // Step 4. If this cursor’s source is not an index throw an "InvalidAccessError" DOMException.
        if !matches!(self.source, ObjectStoreOrIndex::Index(..)) {
            return Err(Error::InvalidAccess);
        }

        // Step 5. If this cursor’s direction is not "next" or "prev", throw an
        // "InvalidAccessError" DOMException.
        if !matches!(
            self.direction,
            IDBCursorDirection::Next | IDBCursorDirection::Prev
        ) {
            return Err(Error::InvalidAccess);
        }

        // Step 6. If this cursor’s got value flag is unset, indicating that the cursor is being
        // iterated or has iterated past its end, throw an "InvalidStateError" DOMException.
        if !self.got_value.get() {
            return Err(Error::InvalidState(None));
        }

        // Step 7. Let r be the result of running the steps to convert a value to a key with key.
        // Rethrow any exceptions.
        let r = convert_value_to_key(cx, key, None)?;

        // Step 8. If r is invalid, throw a "DataError" DOMException.
        // Step 9. Let key be r.
        let key = r.into_result()?;

        // Step 10. Let r be the result of running the steps to convert a value to a key with
        // primaryKey. Rethrow any exceptions.
        let r = convert_value_to_key(cx, primary_key, None)?;

        // Step 11. If r is invalid, throw a "DataError" DOMException.
        // Step 12. Let primaryKey be r.
        let primary_key = r.into_result()?;

        // Step 13. If key is less than this cursor’s position and this cursor’s direction is
        // "next", throw a "DataError" DOMException.
        if self
            .position
            .borrow()
            .as_ref()
            .is_some_and(|cursor_position| &key < cursor_position) &&
            self.direction == IDBCursorDirection::Next
        {
            return Err(Error::Data);
        }

        // Step 14. If key is greater than this cursor’s position and this cursor’s direction is
        // "prev", throw a "DataError" DOMException.
        if self
            .position
            .borrow()
            .as_ref()
            .is_some_and(|cursor_position| &key > cursor_position) &&
            self.direction == IDBCursorDirection::Prev
        {
            return Err(Error::Data);
        }

        // Step 15. If key is equal to this cursor’s position and primaryKey is less than or equal
        // to this cursor’s object store position and this cursor’s direction is "next", throw a
        // "DataError" DOMException.
        if self
            .position
            .borrow()
            .as_ref()
            .is_some_and(|cursor_position| &key == cursor_position) &&
            self.object_store_position.borrow().as_ref().is_some_and(
                |cursor_object_store_position| &primary_key <= cursor_object_store_position,
            ) &&
            self.direction == IDBCursorDirection::Next
        {
            return Err(Error::Data);
        }

        // Step 16. If key is equal to this cursor’s position and primaryKey is greater than or
        // equal to this cursor’s object store position and this cursor’s direction is "prev",
        // throw a "DataError" DOMException.
        if self
            .position
            .borrow()
            .as_ref()
            .is_some_and(|cursor_position| &key == cursor_position) &&
            self.object_store_position.borrow().as_ref().is_some_and(
                |cursor_object_store_position| &primary_key >= cursor_object_store_position,
            ) &&
            self.direction == IDBCursorDirection::Prev
        {
            return Err(Error::Data);
        }

        // Step 17. Unset the got value flag on the cursor.
        self.got_value.set(false);

        // Step 18. Let request be the request created when this cursor was created.
        let request = self
            .request
            .get()
            .expect("IDBCursor.request should be set when cursor is opened");

        // Step 19. Unset the done flag on request.
        request.set_ready_state(IDBRequestReadyState::Pending);

        // Step 20. Run the steps to asynchronously execute a request with the cursor’s source as
        // source, the steps to iterate a cursor as operation and request, using the current Realm
        // as targetRealm, this cursor, key and primaryKey.
        let iteration_param = IterationParam {
            cursor: Trusted::new(self),
            key: Some(key),
            primary_key: Some(primary_key),
            count: None,
        };
        let (sender, receiver) = indexed_db::create_channel(self.global());
        match &self.source {
            ObjectStoreOrIndex::ObjectStore(object_store) => {
                IDBRequest::execute_async(
                    object_store,
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Iterate {
                        sender,
                        key_range: self.range.clone(),
                    }),
                    receiver,
                    None,
                    Some(iteration_param),
                    CanGc::note(),
                )?;
            },
            ObjectStoreOrIndex::Index(_index) => {
                // TODO: IDBRequest::execute_async currently does not accept using index as source.
                return Err(Error::NotSupported);
            },
        };

        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbcursor-update>
    fn Update(&self, cx: SafeJSContext, value: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1. Let transaction be this cursor's transaction.
        // Step 2. If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 3. If transaction is a read-only transaction, throw a "ReadOnlyError" DOMException.
        self.check_transaction_readwrite()?;

        // Step 4. If the cursor’s source or effective object store has been deleted, throw an
        // "InvalidStateError" DOMException.
        self.verify_not_deleted()?;
        self.effective_object_store().verify_not_deleted()?;

        // Step 5. If this cursor’s got value flag is unset, indicating that the cursor is being
        // iterated or has iterated past its end, throw an "InvalidStateError" DOMException.
        if !self.got_value.get() {
            return Err(Error::InvalidState(None));
        }

        // Step 6. If this cursor’s key only flag is set, throw an "InvalidStateError" DOMException.
        if self.key_only {
            return Err(Error::InvalidState(None));
        }

        // Step 7. Let targetRealm be a user-agent defined Realm.
        // Step 8. Let clone be a clone of value in targetRealm. Rethrow any exceptions.
        let cloned_value = structuredclone::write(cx, value, None)?;
        let Ok(serialized_value) = bincode::serialize(&cloned_value) else {
            return Err(Error::InvalidState(None));
        };
        rooted!(in(*cx) let mut clone = UndefinedValue());
        let _ = structuredclone::read(
            &self.global(),
            cloned_value,
            clone.handle_mut(),
            CanGc::note(),
        )?;

        // Step 9. If the effective object store of this cursor uses in-line keys, then:
        if self.effective_object_store().uses_inline_keys() {
            // Step 9.1. Let kpk be the result of running the steps to extract a key from a value
            // using a key path with clone and the key path of the effective object store. Rethrow
            // any exceptions.
            let kpk = extract_key(
                cx,
                clone.handle(),
                self.effective_object_store()
                    .key_path()
                    .expect("uses_inline_keys() being retur guarantees it has a key path"),
                None,
            )?;

            // Step 9..2 If kpk is failure, invalid, or not equal to the cursor’s effective key,
            // throw a "DataError" DOMException.
            match &kpk {
                ExtractionResult::Failure | ExtractionResult::Invalid => {
                    return Err(Error::Data);
                },
                ExtractionResult::Key(kpk) => {
                    if kpk != &self.effective_key().ok_or(Error::InvalidState(None))? {
                        return Err(Error::Data);
                    }
                },
            }
        }

        // Step 10. Run the steps to asynchronously execute a request and return the IDBRequest
        // created by these steps. The steps are run with this cursor as source and the steps to
        // store a record into an object store as operation, using this cursor’s effective object
        // store as store, the clone as value, this cursor’s effective key as key, and with the
        // no-overwrite flag unset.
        let (sender, receiver) = indexed_db::create_channel(self.global());
        IDBRequest::execute_async(
            // FIXME: IDBRequest::execute_async currently does not accept using cursor as source.
            // We use the cursor's effective object store instead, for now.
            &self.effective_object_store(),
            AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem {
                sender,
                key: Some(self.effective_key().ok_or(Error::InvalidState(None))?),
                value: serialized_value,
                should_overwrite: true,
            }),
            receiver,
            None,
            None,
            CanGc::note(),
        )
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbcursor-delete>
    fn Delete(&self) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1. Let transaction be this cursor's transaction.
        // Step 2. If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 3. If transaction is a read-only transaction, throw a "ReadOnlyError" DOMException.
        self.check_transaction_readwrite()?;

        // Step 4. If the cursor’s source or effective object store has been deleted, throw an
        // "InvalidStateError" DOMException.
        self.verify_not_deleted()?;
        self.effective_object_store().verify_not_deleted()?;

        // Step 5. If this cursor’s got value flag is unset, indicating that the cursor is being
        // iterated or has iterated past its end, throw an "InvalidStateError" DOMException.
        if !self.got_value.get() {
            return Err(Error::InvalidState(None));
        }

        // Step 6. If this cursor’s key only flag is set, throw an "InvalidStateError" DOMException.
        if self.key_only {
            return Err(Error::InvalidState(None));
        }

        // Step 7. Run the steps to asynchronously execute a request and return the IDBRequest
        // created by these steps. The steps are run with this cursor as source and the steps to
        // delete records from an object store as operation, using this cursor’s effective object
        // store and effective key as store and key respectively.
        let (sender, receiver) = indexed_db::create_channel(self.global());
        IDBRequest::execute_async(
            // FIXME: IDBRequest::execute_async currently does not accept using cursor as source.
            // We use the cursor's effective object store instead, for now.
            &self.effective_object_store(),
            AsyncOperation::ReadWrite(AsyncReadWriteOperation::RemoveItem {
                sender,
                key: self.effective_key().ok_or(Error::InvalidState(None))?,
            }),
            receiver,
            None,
            None,
            CanGc::note(),
        )
    }
}

/// A struct containing parameters for
/// <https://www.w3.org/TR/IndexedDB-2/#iterate-a-cursor>
#[derive(Clone)]
pub(crate) struct IterationParam {
    pub(crate) cursor: Trusted<IDBCursor>,
    pub(crate) key: Option<IndexedDBKeyType>,
    pub(crate) primary_key: Option<IndexedDBKeyType>,
    pub(crate) count: Option<u32>,
}

/// <https://www.w3.org/TR/IndexedDB-2/#iterate-a-cursor>
///
/// NOTE: Be cautious: this part of the specification seems to assume the cursor’s source is an
/// index. Therefore,
///   "record’s key" means the key of the record,
///   "record’s value" means the primary key of the record, and
///   "record’s referenced value" means the value of the record.
pub(crate) fn iterate_cursor(
    global: &GlobalScope,
    cx: SafeJSContext,
    param: &IterationParam,
    records: Vec<IndexedDBRecord>,
    can_gc: CanGc,
) -> Result<Option<DomRoot<IDBCursor>>, Error> {
    // Unpack IterationParam
    let cursor = param.cursor.root();
    let key = param.key.clone();
    let primary_key = param.primary_key.clone();
    let count = param.count;

    // Step 1. Let source be cursor’s source.
    let source = &cursor.source;

    // Step 2. Let direction be cursor’s direction.
    let direction = cursor.direction;

    // Step 3. Assert: if primaryKey is given, source is an index and direction is "next" or "prev".
    if primary_key.is_some() {
        assert!(matches!(source, ObjectStoreOrIndex::Index(..)));
        assert!(matches!(
            direction,
            IDBCursorDirection::Next | IDBCursorDirection::Prev
        ));
    }

    // Step 4. Let records be the list of records in source.
    // NOTE: It is given as a function parameter.

    // Step 5. Let range be cursor’s range.
    let range = &cursor.range;

    // Step 6. Let position be cursor’s position.
    let mut position = cursor.position.borrow().clone();

    // Step 7. Let object store position be cursor’s object store position.
    let object_store_position = cursor.object_store_position.borrow().clone();

    // Step 8. If count is not given, let count be 1.
    let mut count = count.unwrap_or(1);

    let mut found_record: Option<&IndexedDBRecord> = None;

    // Step 9. While count is greater than 0:
    while count > 0 {
        // Step 9.1. Switch on direction:
        found_record = match direction {
            // "next"
            IDBCursorDirection::Next => records.iter().find(|record| {
                // Let found record be the first record in records which satisfy all of the
                // following requirements:

                // If key is defined, the record’s key is greater than or equal to key.
                let requirement1 = || match &key {
                    Some(key) => &record.key >= key,
                    None => true,
                };

                // If primaryKey is defined, the record’s key is equal to key and the record’s
                // value is greater than or equal to primaryKey, or the record’s key is greater
                // than key.
                let requirement2 = || match &primary_key {
                    Some(primary_key) => key.as_ref().is_some_and(|key| {
                        (&record.key == key && &record.primary_key >= primary_key) ||
                            &record.key > key
                    }),
                    _ => true,
                };

                // If position is defined, and source is an object store, the record’s key is
                // greater than position.
                let requirement3 = || match (&position, source) {
                    (Some(position), ObjectStoreOrIndex::ObjectStore(_)) => &record.key > position,
                    _ => true,
                };

                // If position is defined, and source is an index, the record’s key is equal to
                // position and the record’s value is greater than object store position or the
                // record’s key is greater than position.
                let requirement4 = || match (&position, source) {
                    (Some(position), ObjectStoreOrIndex::Index(_)) => {
                        (&record.key == position &&
                            object_store_position.as_ref().is_some_and(
                                |object_store_position| &record.primary_key > object_store_position,
                            )) ||
                            &record.key > position
                    },
                    _ => true,
                };

                // The record’s key is in range.
                let requirement5 = || range.contains(&record.key);

                // NOTE: Use closures here for lazy computation on requirements.
                requirement1() &&
                    requirement2() &&
                    requirement3() &&
                    requirement4() &&
                    requirement5()
            }),
            // "nextunique"
            IDBCursorDirection::Nextunique => records.iter().find(|record| {
                // Let found record be the first record in records which satisfy all of the
                // following requirements:

                // If key is defined, the record’s key is greater than or equal to key.
                let requirement1 = || match &key {
                    Some(key) => &record.key >= key,
                    None => true,
                };

                // If position is defined, the record’s key is greater than position.
                let requirement2 = || match &position {
                    Some(position) => &record.key > position,
                    None => true,
                };

                // The record’s key is in range.
                let requirement3 = || range.contains(&record.key);

                // NOTE: Use closures here for lazy computation on requirements.
                requirement1() && requirement2() && requirement3()
            }),
            // "prev"
            IDBCursorDirection::Prev => {
                records.iter().rev().find(|&record| {
                    // Let found record be the last record in records which satisfy all of the
                    // following requirements:

                    // If key is defined, the record’s key is less than or equal to key.
                    let requirement1 = || match &key {
                        Some(key) => &record.key <= key,
                        None => true,
                    };

                    // If primaryKey is defined, the record’s key is equal to key and the record’s
                    // value is less than or equal to primaryKey, or the record’s key is less than
                    // key.
                    let requirement2 = || match &primary_key {
                        Some(primary_key) => key.as_ref().is_some_and(|key| {
                            (&record.key == key && &record.primary_key <= primary_key) ||
                                &record.key < key
                        }),
                        _ => true,
                    };

                    // If position is defined, and source is an object store, the record’s key is
                    // less than position.
                    let requirement3 = || match (&position, source) {
                        (Some(position), ObjectStoreOrIndex::ObjectStore(_)) => {
                            &record.key < position
                        },
                        _ => true,
                    };

                    // If position is defined, and source is an index, the record’s key is equal to
                    // position and the record’s value is less than object store position or the
                    // record’s key is less than position.
                    let requirement4 = || match (&position, source) {
                        (Some(position), ObjectStoreOrIndex::Index(_)) => {
                            (&record.key == position &&
                                object_store_position.as_ref().is_some_and(
                                    |object_store_position| {
                                        &record.primary_key < object_store_position
                                    },
                                )) ||
                                &record.key < position
                        },
                        _ => true,
                    };

                    // The record’s key is in range.
                    let requirement5 = || range.contains(&record.key);

                    // NOTE: Use closures here for lazy computation on requirements.
                    requirement1() &&
                        requirement2() &&
                        requirement3() &&
                        requirement4() &&
                        requirement5()
                })
            },
            // "prevunique"
            IDBCursorDirection::Prevunique => records
                .iter()
                .rev()
                .find(|&record| {
                    // Let temp record be the last record in records which satisfy all of the
                    // following requirements:

                    // If key is defined, the record’s key is less than or equal to key.
                    let requirement1 = || match &key {
                        Some(key) => &record.key <= key,
                        None => true,
                    };

                    // If position is defined, the record’s key is less than position.
                    let requirement2 = || match &position {
                        Some(position) => &record.key < position,
                        None => true,
                    };

                    // The record’s key is in range.
                    let requirement3 = || range.contains(&record.key);

                    // NOTE: Use closures here for lazy computation on requirements.
                    requirement1() && requirement2() && requirement3()
                })
                // If temp record is defined, let found record be the first record in records
                // whose key is equal to temp record’s key.
                .map(|temp_record| {
                    records
                        .iter()
                        .find(|&record| record.key == temp_record.key)
                        .expect(
                            "Record with key equal to temp record's key should exist in records",
                        )
                }),
        };

        match found_record {
            // Step 9.2. If found record is not defined, then:
            None => {
                // Step 9.2.1. Set cursor’s key to undefined.
                cursor.set_key(None);

                // Step 9.2.2. If source is an index, set cursor’s object store position to undefined.
                if matches!(source, ObjectStoreOrIndex::Index(_)) {
                    cursor.set_object_store_position(None);
                }

                // Step 9.2.3. If cursor’s key only flag is unset, set cursor’s value to undefined.
                if !cursor.key_only {
                    cursor.value.set(UndefinedValue());
                }

                // Step 9.2.4. Return null.
                return Ok(None);
            },
            Some(found_record) => {
                // Step 9.3. Let position be found record’s key.
                position = Some(found_record.key.clone());

                // Step 9.4. If source is an index, let object store position be found record’s value.
                if matches!(source, ObjectStoreOrIndex::Index(_)) {
                    cursor.set_object_store_position(Some(found_record.primary_key.clone()));
                }

                // Step 9.5. Decrease count by 1.
                count -= 1;
            },
        }
    }
    let found_record =
        found_record.expect("The while loop above guarantees found_record is defined");

    // Step 10. Set cursor’s position to position.
    cursor.set_position(position);

    // Step 11. If source is an index, set cursor’s object store position to object store position.
    if let ObjectStoreOrIndex::Index(_) = source {
        cursor.set_object_store_position(object_store_position);
    }

    // Step 12. Set cursor’s key to found record’s key.
    cursor.set_key(Some(found_record.key.clone()));

    // Step 13. If cursor’s key only flag is unset, then:
    if !cursor.key_only {
        // Step 13.1. Let serialized be found record’s referenced value.
        // Step 13.2. Set cursor’s value to ! StructuredDeserialize(serialized, targetRealm)
        rooted!(in(*cx) let mut new_cursor_value = UndefinedValue());
        bincode::deserialize(&found_record.value)
            .map_err(|_| Error::Data)
            .and_then(|data| {
                structuredclone::read(global, data, new_cursor_value.handle_mut(), can_gc)
            })?;
        cursor.value.set(new_cursor_value.get());
    }

    // Step 14. Set cursor’s got value flag.
    cursor.got_value.set(true);

    // Step 15. Return cursor.
    Ok(Some(cursor))
}
