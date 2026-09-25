/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, allow(crown::jscontext_first_arg))]

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{HandleValue, MutableHandleValue};
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object};
use storage_traits::indexeddb::{
    AsyncOperation, AsyncReadOnlyOperation, IndexedDBKeyRange, IndexedDBKeyType, IndexedDBRecord,
};

use crate::dom::bindings::codegen::Bindings::IDBCursorBinding::{
    IDBCursorDirection, IDBCursorMethods,
};
use crate::dom::bindings::codegen::UnionTypes::IDBObjectStoreOrIDBIndex;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::structuredclone;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb::idbindex::IDBIndex;
use crate::dom::indexeddb::idbobjectstore::IDBObjectStore;
use crate::dom::indexeddb::idbrequest::IDBRequest;
use crate::dom::indexeddb::idbtransaction::IDBTransaction;
use crate::dom::indexeddb::key::{convert_value_to_key, key_type_to_jsval};

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum ObjectStoreOrIndex {
    ObjectStore(Dom<IDBObjectStore>),
    Index(Dom<IDBIndex>),
}

#[dom_struct]
pub(crate) struct IDBCursor {
    reflector_: Reflector,

    /// <https://www.w3.org/TR/IndexedDB-3/#cursor-transaction>
    transaction: Dom<IDBTransaction>,
    /// <https://www.w3.org/TR/IndexedDB-3/#cursor-range>
    #[no_trace]
    range: IndexedDBKeyRange,
    /// <https://www.w3.org/TR/IndexedDB-3/#cursor-source>
    source: ObjectStoreOrIndex,
    /// <https://www.w3.org/TR/IndexedDB-3/#cursor-direction>
    direction: IDBCursorDirection,
    /// <https://www.w3.org/TR/IndexedDB-3/#cursor-position>
    #[no_trace]
    position: DomRefCell<Option<IndexedDBKeyType>>,
    /// <https://www.w3.org/TR/IndexedDB-3/#cursor-key>
    #[no_trace]
    key: DomRefCell<Option<IndexedDBKeyType>>,
    #[ignore_malloc_size_of = "mozjs"]
    cached_key: DomRefCell<Option<Heap<JSVal>>>,
    #[ignore_malloc_size_of = "mozjs"]
    cached_primary_key: DomRefCell<Option<Heap<JSVal>>>,
    /// <https://www.w3.org/TR/IndexedDB-3/#cursor-value>
    #[ignore_malloc_size_of = "mozjs"]
    value: Heap<JSVal>,
    /// <https://www.w3.org/TR/IndexedDB-3/#cursor-got-value-flag>
    got_value: Cell<bool>,
    /// <https://www.w3.org/TR/IndexedDB-3/#cursor-object-store-position>
    #[no_trace]
    object_store_position: DomRefCell<Option<IndexedDBKeyType>>,
    /// <https://www.w3.org/TR/IndexedDB-3/#cursor-key-only-flag>
    key_only: bool,

    /// <https://w3c.github.io/IndexedDB/#cursor-request>
    request: MutNullableDom<IDBRequest>,
}

impl IDBCursor {
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
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
            cached_key: DomRefCell::new(None),
            cached_primary_key: DomRefCell::new(None),
            value: Heap::default(),
            got_value: Cell::new(got_value),
            object_store_position: DomRefCell::new(None),
            key_only,
            request: Default::default(),
        }
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        transaction: &IDBTransaction,
        direction: IDBCursorDirection,
        got_value: bool,
        source: ObjectStoreOrIndex,
        range: IndexedDBKeyRange,
        key_only: bool,
    ) -> DomRoot<IDBCursor> {
        reflect_dom_object(
            cx,
            Box::new(IDBCursor::new_inherited(
                transaction,
                direction,
                got_value,
                source,
                range,
                key_only,
            )),
            global,
        )
    }

    fn set_position(&self, position: Option<IndexedDBKeyType>) {
        let changed = *self.position.borrow() != position;
        *self.position.borrow_mut() = position;
        if changed {
            *self.cached_primary_key.borrow_mut() = None;
        }
    }

    fn set_key(&self, key: Option<IndexedDBKeyType>) {
        let key_changed = {
            let current_key = self.key.borrow();
            current_key.as_ref() != key.as_ref()
        };
        *self.key.borrow_mut() = key;
        if key_changed {
            *self.cached_key.borrow_mut() = None;
        }
    }

    fn set_object_store_position(&self, object_store_position: Option<IndexedDBKeyType>) {
        let changed = *self.object_store_position.borrow() != object_store_position;
        *self.object_store_position.borrow_mut() = object_store_position;
        if changed {
            *self.cached_primary_key.borrow_mut() = None;
        }
    }

    pub(crate) fn set_request(&self, request: &IDBRequest) {
        self.request.set(Some(request));
    }

    pub(crate) fn value(&self, mut out: MutableHandleValue) {
        out.set(self.value.get());
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#cursor-effective-key>
    pub(crate) fn effective_key(&self) -> Option<IndexedDBKeyType> {
        match &self.source {
            ObjectStoreOrIndex::ObjectStore(_) => self.position.borrow().clone(),
            ObjectStoreOrIndex::Index(_) => self.object_store_position.borrow().clone(),
        }
    }

    pub(crate) fn effective_object_store(&self) -> DomRoot<IDBObjectStore> {
        match &self.source {
            ObjectStoreOrIndex::ObjectStore(store) => store.as_rooted(),
            ObjectStoreOrIndex::Index(index) => index.object_store(),
        }
    }

    pub(crate) fn verify_not_deleted(&self) -> ErrorResult {
        match &self.source {
            ObjectStoreOrIndex::ObjectStore(store) => store.verify_not_deleted(),
            ObjectStoreOrIndex::Index(index) => index.verify_not_deleted(),
        }
    }

    pub(crate) fn check_transaction_active(&self) -> ErrorResult {
        if !self.transaction.is_active() || !self.transaction.is_usable() {
            Err(Error::TransactionInactive(Some(
                "Transaction is not active".to_owned(),
            )))
        } else {
            Ok(())
        }
    }
}

impl IDBCursorMethods<crate::DomTypeHolder> for IDBCursor {
    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbcursor-source>
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

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbcursor-direction>
    fn Direction(&self) -> IDBCursorDirection {
        self.direction
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbcursor-key>
    fn Key(&self, cx: &mut JSContext, mut value: MutableHandleValue) {
        // The key getter steps are to return the result of converting a key to a value with the cursor’s current key.
        //
        // NOTE: If key returns an object (e.g. a Date or Array), it returns the
        // same object instance every time it is inspected, until the cursor’s key is changed.
        // This means that if the object is modified, those modifications will be seen by
        // anyone inspecting the value of the cursor. However modifying such an object does not
        // modify the contents of the database.
        if let Some(cached) = &*self.cached_key.borrow() {
            value.set(cached.get());
            return;
        }

        match self.key.borrow().as_ref() {
            Some(key) => key_type_to_jsval(cx, key, value.reborrow()),
            None => value.set(UndefinedValue()),
        }

        *self.cached_key.borrow_mut() = Some(Heap::default());
        self.cached_key.borrow().as_ref().unwrap().set(value.get());
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbcursor-primarykey>
    fn PrimaryKey(&self, cx: &mut JSContext, mut value: MutableHandleValue) {
        // NOTE: If primaryKey returns an object (e.g. a Date or Array),
        // it returns the same object instance every time it is inspected,
        // until the cursor’s effective key is changed. This means that if the object is modified,
        // those modifications will be seen by anyone inspecting the value of the cursor.
        // However modifying such an object does not modify the contents of the database.
        if let Some(cached) = &*self.cached_primary_key.borrow() {
            value.set(cached.get());
            return;
        }

        match self.effective_key() {
            Some(effective_key) => key_type_to_jsval(cx, &effective_key, value.reborrow()),
            None => value.set(UndefinedValue()),
        }

        *self.cached_primary_key.borrow_mut() = Some(Heap::default());
        self.cached_primary_key
            .borrow()
            .as_ref()
            .unwrap()
            .set(value.get());
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbcursor-request>
    fn Request(&self) -> DomRoot<IDBRequest> {
        self.request
            .get()
            .expect("IDBCursor.request should be set when cursor is opened")
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbcursor-advance>
    fn Advance(&self, cx: &mut JSContext, count: u32) -> ErrorResult {
        // Step 1: If count is 0 (zero), throw a TypeError.
        if count == 0 {
            return Err(Error::Type(c"Count cannot be zero".to_owned()));
        }

        // Step 2: Let transaction be this cursor's transaction.
        // Step 3: If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 4: If the cursor's source or effective object store has been deleted, throw an
        // "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 5: If this cursor's got value flag is unset, indicating that the cursor is being
        // iterated or has iterated past its end, throw an "InvalidStateError" DOMException.
        if !self.got_value.get() {
            return Err(Error::InvalidState(Some(
                "The cursor is being iterated or has iterated past its end".to_owned(),
            )));
        }

        // Step 7: Unset the got value flag on the cursor.
        self.got_value.set(false);

        // Step 8: Let request be the request created when this cursor was created.
        let request = self.Request();

        // Step 9: Unset the request's done flag on request.
        request.reset();

        // Step 10: Run the steps to asynchronously execute a request with the cursor's source as
        // source, the steps to iterate a cursor as operation and request, using the current Realm
        // as targetRealm, this cursor and count.
        let iteration_param = IterationParam {
            cursor: Trusted::new(self),
            key: None,
            primary_key: None,
            count: Some(count),
        };

        let range = self.range.clone();
        IDBRequest::execute_async(
            cx,
            &self.effective_object_store(),
            |callback| {
                AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Iterate {
                    callback,
                    key_range: range,
                })
            },
            Some(request),
            Some(iteration_param),
        )?;

        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbcursor-continue>
    fn Continue(&self, cx: &mut JSContext, key: HandleValue) -> ErrorResult {
        // Step 1: Let transaction be this cursor's transaction.
        // Step 2: If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 3: If the cursor's source or effective object store has been deleted, throw an
        // "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4: If this cursor's got value flag is unset, indicating that the cursor is being
        // iterated or has iterated past its end, throw an "InvalidStateError" DOMException.
        if !self.got_value.get() {
            return Err(Error::InvalidState(Some(
                "The cursor is being iterated or has iterated past its end".to_owned(),
            )));
        }

        // Step 5: If key is given, then:
        let converted_key = if !key.is_undefined() {
            // Step 5.1: Let r be the result of running the steps to convert a value to a key with
            // key. Rethrow any exceptions.
            // Step 5.2: If r is invalid, throw a "DataError" DOMException.
            let k = convert_value_to_key(cx, key, None)?.into_result()?;

            // Step 5.4: If key is less than or equal to this cursor's position and this cursor's
            // direction is "next" or "nextunique", throw a "DataError" DOMException.
            if let Some(position) = self.position.borrow().as_ref() {
                match self.direction {
                    IDBCursorDirection::Next | IDBCursorDirection::Nextunique => {
                        if &k <= position {
                            return Err(Error::Data(Some(
                                "Key is less than or equal to this cursor's position".to_owned(),
                            )));
                        }
                    },
                    // Step 5.5: If key is greater than or equal to this cursor's position
                    // and this cursor's direction is "prev" or "prevunique", throw a
                    // "DataError" DOMException.
                    IDBCursorDirection::Prev | IDBCursorDirection::Prevunique => {
                        if &k >= position {
                            return Err(Error::Data(Some(
                                "Key is greater than or equal to this cursor's position".to_owned(),
                            )));
                        }
                    },
                }
            }

            Some(k)
        } else {
            None
        };

        // Step 6: Unset the got value flag on the cursor.
        self.got_value.set(false);

        // Step 7: Let request be the request created when this cursor was created.
        let request = self.Request();

        // Step 8: Unset the request's done flag on request.
        request.reset();

        // Step 9: Run the steps to asynchronously execute a request with the cursor's source as
        // source, the steps to iterate a cursor as operation and request, using the current Realm
        // as targetRealm, this cursor and key (if given).
        let iteration_param = IterationParam {
            cursor: Trusted::new(self),
            key: converted_key,
            primary_key: None,
            count: None,
        };

        let range = self.range.clone();
        IDBRequest::execute_async(
            cx,
            &self.effective_object_store(),
            |callback| {
                AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Iterate {
                    callback,
                    key_range: range,
                })
            },
            Some(request),
            Some(iteration_param),
        )?;

        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB-3/#dom-idbcursor-continueprimarykey>
    fn ContinuePrimaryKey(
        &self,
        cx: &mut JSContext,
        key: HandleValue,
        primary_key: HandleValue,
    ) -> ErrorResult {
        // Step 1: Let transaction be this cursor's transaction.
        // Step 2: If transaction is not active, throw a "TransactionInactiveError" DOMException.
        self.check_transaction_active()?;

        // Step 3: If the cursor's source or effective object store has been deleted, throw an
        // "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4: If this cursor's source is not an index, throw an "InvalidAccessError"
        // DOMException.
        if !matches!(self.source, ObjectStoreOrIndex::Index(_)) {
            return Err(Error::InvalidAccess(Some(
                "The cursor's source is not an index".to_owned(),
            )));
        }

        // Step 5: If this cursor's direction is not "next" or "prev", throw an "InvalidAccessError"
        // DOMException.
        if !matches!(
            self.direction,
            IDBCursorDirection::Next | IDBCursorDirection::Prev
        ) {
            return Err(Error::InvalidAccess(Some(
                "The cursor's direction is not 'next' or 'prev'".to_owned(),
            )));
        }

        // Step 6: If this cursor's got value flag is unset, indicating that the cursor is being
        // iterated or has iterated past its end, throw an "InvalidStateError" DOMException.
        if !self.got_value.get() {
            return Err(Error::InvalidState(Some(
                "The cursor is being iterated or has iterated past its end".to_owned(),
            )));
        }

        // Step 7: Let r be the result of running the steps to convert a value to a key with key.
        // Step 8: If r is invalid, throw a "DataError" DOMException.
        let key = convert_value_to_key(cx, key, None)?.into_result()?;

        // Step 10: Let r be the result of running the steps to convert a value to a key with
        // primaryKey.
        // Step 11: If r is invalid, throw a "DataError" DOMException.
        let primary_key = convert_value_to_key(cx, primary_key, None)?.into_result()?;

        let position = self.position.borrow().clone();
        let object_store_position = self.object_store_position.borrow().clone();

        match self.direction {
            IDBCursorDirection::Next => {
                // Step 13: If key is less than this cursor's position and this cursor's direction
                // is "next", throw a "DataError" DOMException.
                if let Some(pos) = &position {
                    if &key < pos {
                        return Err(Error::Data(Some(
                            "Key is less than this cursor's position".to_owned(),
                        )));
                    }
                    // Step 15: If key is equal to this cursor's position and primaryKey is less
                    // than or equal to this cursor's object store position and this cursor's
                    // direction is "next", throw a "DataError" DOMException.
                    if &key == pos &&
                        object_store_position
                            .as_ref()
                            .is_some_and(|osp| &primary_key <= osp)
                    {
                        return Err(Error::Data(Some(
                            "Primary key is less than or equal to object store position".to_owned(),
                        )));
                    }
                }
            },
            IDBCursorDirection::Prev => {
                // Step 14: If key is greater than this cursor's position and this cursor's
                // direction is "prev", throw a "DataError" DOMException.
                if let Some(pos) = &position {
                    if &key > pos {
                        return Err(Error::Data(Some(
                            "Key is greater than this cursor's position".to_owned(),
                        )));
                    }
                    // Step 16: If key is equal to this cursor's position and primaryKey is greater
                    // than or equal to this cursor's object store position and this cursor's
                    // direction is "prev", throw a "DataError" DOMException.
                    if &key == pos &&
                        object_store_position
                            .as_ref()
                            .is_some_and(|osp| &primary_key >= osp)
                    {
                        return Err(Error::Data(Some(
                            "Primary key is greater than or equal to object store position"
                                .to_owned(),
                        )));
                    }
                }
            },
            _ => unreachable!(),
        }

        // Step 17: Unset the got value flag on the cursor.
        self.got_value.set(false);

        // Step 18: Let request be the request created when this cursor was created.
        let request = self.Request();

        // Step 19: Unset the request's done flag on request.
        request.reset();

        // Step 20: Run the steps to asynchronously execute a request with the cursor's source as
        // source, the steps to iterate a cursor as operation and request, using the current Realm
        // as targetRealm, this cursor, key and primaryKey.
        let iteration_param = IterationParam {
            cursor: Trusted::new(self),
            key: Some(key),
            primary_key: Some(primary_key),
            count: None,
        };

        let range = self.range.clone();
        IDBRequest::execute_async(
            cx,
            &self.effective_object_store(),
            |callback| {
                AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Iterate {
                    callback,
                    key_range: range,
                })
            },
            Some(request),
            Some(iteration_param),
        )?;

        Ok(())
    }
}

/// A struct containing parameters for
/// <https://www.w3.org/TR/IndexedDB-3/#iterate-a-cursor>
#[derive(Clone)]
pub(crate) struct IterationParam {
    pub(crate) cursor: Trusted<IDBCursor>,
    pub(crate) key: Option<IndexedDBKeyType>,
    pub(crate) primary_key: Option<IndexedDBKeyType>,
    pub(crate) count: Option<u32>,
}

/// <https://www.w3.org/TR/IndexedDB-3/#iterate-a-cursor>
///
/// NOTE: Be cautious: this part of the specification seems to assume the cursor’s source is an
/// index. Therefore,
///   "record’s key" means the key of the record,
///   "record’s value" means the primary key of the record, and
///   "record’s referenced value" means the value of the record.
pub(crate) fn iterate_cursor(
    global: &GlobalScope,
    cx: &mut JSContext,
    param: &IterationParam,
    records: Vec<IndexedDBRecord>,
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
    let mut object_store_position = cursor.object_store_position.borrow().clone();

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
                    object_store_position = Some(found_record.primary_key.clone());
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
        rooted!(&in(cx) let mut new_cursor_value = UndefinedValue());
        postcard::from_bytes(&found_record.value)
            .map_err(|_| Error::Data(None))
            .and_then(|data| {
                structuredclone::read(cx, global, data, new_cursor_value.handle_mut())
            })?;
        cursor.value.set(new_cursor_value.get());
    }

    // Step 14. Set cursor’s got value flag.
    cursor.got_value.set(true);

    // Step 15. Return cursor.
    Ok(Some(cursor))
}
