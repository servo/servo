/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::MutableHandleValue;
use storage_traits::indexeddb_thread::{IndexedDBKeyRange, IndexedDBKeyType, IndexedDBRecord};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBCursorBinding::{
    IDBCursorDirection, IDBCursorMethods,
};
use crate::dom::bindings::codegen::UnionTypes::IDBObjectStoreOrIDBIndex;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::structuredclone;
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbindex::IDBIndex;
use crate::dom::idbobjectstore::IDBObjectStore;
use crate::dom::idbrequest::IDBRequest;
use crate::dom::idbtransaction::IDBTransaction;
use crate::indexed_db::key_type_to_jsval;
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

    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-effective-key>
    pub(crate) fn effective_key(&self) -> Option<IndexedDBKeyType> {
        match &self.source {
            ObjectStoreOrIndex::ObjectStore(_) => self.position.borrow().clone(),
            ObjectStoreOrIndex::Index(_) => self.object_store_position.borrow().clone(),
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
    fn Key(&self, cx: SafeJSContext, mut value: MutableHandleValue) {
        match self.key.borrow().as_ref() {
            Some(key) => key_type_to_jsval(cx, key, value),
            None => value.set(UndefinedValue()),
        }
    }

    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbcursor-primarykey>
    fn PrimaryKey(&self, cx: SafeJSContext, mut value: MutableHandleValue) {
        match self.effective_key() {
            Some(effective_key) => key_type_to_jsval(cx, &effective_key, value),
            None => value.set(UndefinedValue()),
        }
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbcursor-request>
    fn Request(&self) -> DomRoot<IDBRequest> {
        self.request
            .get()
            .expect("IDBCursor.request should be set when cursor is opened")
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
            .and_then(|data| structuredclone::read(global, data, new_cursor_value.handle_mut()))?;
        cursor.value.set(new_cursor_value.get());
    }

    // Step 14. Set cursor’s got value flag.
    cursor.got_value.set(true);

    // Step 15. Return cursor.
    Ok(Some(cursor))
}
