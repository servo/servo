/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::MutableHandleValue;
use net_traits::indexeddb_thread::{IndexedDBKeyRange, IndexedDBKeyType};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBCursorBinding::{
    IDBCursorDirection, IDBCursorMethods,
};
use crate::dom::bindings::codegen::UnionTypes::IDBObjectStoreOrIDBIndex;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbindex::IDBIndex;
use crate::dom::idbobjectstore::IDBObjectStore;
use crate::dom::idbrequest::IDBRequest;
use crate::dom::idbtransaction::IDBTransaction;
use crate::indexed_db::key_type_to_jsval;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[derive(JSTraceable, MallocSizeOf)]
#[expect(unused)]
pub(crate) enum ObjectStoreOrIndex {
    ObjectStore(DomRoot<IDBObjectStore>),
    Index(DomRoot<IDBIndex>),
}

#[dom_struct]
pub(crate) struct IDBCursor {
    reflector_: Reflector,

    /// <https://www.w3.org/TR/IndexedDB-2/#cursor-transaction>
    transaction: DomRoot<IDBTransaction>,
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
    pub(crate) fn new_inherited(
        transaction: DomRoot<IDBTransaction>,
        direction: IDBCursorDirection,
        got_value: bool,
        source: ObjectStoreOrIndex,
        range: IndexedDBKeyRange,
        key_only: bool,
    ) -> IDBCursor {
        IDBCursor {
            reflector_: Reflector::new(),
            transaction,
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

    #[expect(unused)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        global: &GlobalScope,
        transaction: DomRoot<IDBTransaction>,
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
                IDBObjectStoreOrIDBIndex::IDBObjectStore(source.clone())
            },
            ObjectStoreOrIndex::Index(source) => IDBObjectStoreOrIDBIndex::IDBIndex(source.clone()),
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
