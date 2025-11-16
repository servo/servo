/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::MutableHandleValue;

use crate::dom::bindings::codegen::Bindings::IDBCursorBinding::{
    IDBCursorDirection, IDBCursorMethods,
};
use crate::dom::bindings::codegen::UnionTypes::IDBObjectStoreOrIDBIndex;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb_next::idbindex::IDBIndex;
use crate::dom::indexeddb_next::idbobjectstore::IDBObjectStore;
use crate::dom::indexeddb_next::idbrequest::IDBRequest;
use crate::dom::indexeddb_next::idbtransaction::IDBTransaction;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[derive(JSTraceable, MallocSizeOf)]
#[expect(unused)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum ObjectStoreOrIndexHandle {
    ObjectStoreHandle(Dom<IDBObjectStore>),
    IndexHandle(Dom<IDBIndex>),
}

/// An "object" implementing the spec’s IDBCursor interface:
/// <https://w3c.github.io/IndexedDB/#cursor-interface>.
///
/// The IDBCursor interface represents a cursor:
/// <https://w3c.github.io/IndexedDB/#cursor-construct>.
///
/// The cursor can be used to iterate over data stored in the associated
/// source (an object store or an index).
///
/// The IDBCursor struct has a remote counterpart in the backend, which
/// performs some of the steps defined by the corresponding spec algorithms.
#[dom_struct]
pub(crate) struct IDBCursor {
    reflector_: Reflector,

    /// <https://w3c.github.io/IndexedDB/#cursor-transaction>
    transaction: Dom<IDBTransaction>,
    /// <https://w3c.github.io/IndexedDB/#cursor-source>
    source: ObjectStoreOrIndexHandle,
    /// <https://w3c.github.io/IndexedDB/#cursor-direction>
    direction: IDBCursorDirection,
    /// <https://w3c.github.io/IndexedDB/#cursor-value>
    #[ignore_malloc_size_of = "mozjs"]
    value: Heap<JSVal>,
    /// <https://w3c.github.io/IndexedDB/#cursor-got-value-flag>
    got_value_flag: Cell<bool>,
    /// <https://w3c.github.io/IndexedDB/#cursor-request>
    request: MutNullableDom<IDBRequest>,
    /// <https://w3c.github.io/IndexedDB/#cursor-key-only-flag>
    key_only_flag: bool,
}

impl IDBCursor {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn _new_inherited(
        transaction: &IDBTransaction,
        source: ObjectStoreOrIndexHandle,
        direction: IDBCursorDirection,
        key_only_flag: bool,
    ) -> IDBCursor {
        IDBCursor {
            reflector_: Reflector::new(),
            transaction: Dom::from_ref(transaction),
            source,
            direction,
            value: Heap::default(),
            got_value_flag: Cell::new(false),
            request: Default::default(),
            key_only_flag,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn _new(
        global: &GlobalScope,
        transaction: &IDBTransaction,
        source: ObjectStoreOrIndexHandle,
        direction: IDBCursorDirection,
        key_only: bool,
        can_gc: CanGc,
    ) -> DomRoot<IDBCursor> {
        reflect_dom_object(
            Box::new(IDBCursor::_new_inherited(
                transaction,
                source,
                direction,
                key_only,
            )),
            global,
            can_gc,
        )
    }

    pub(crate) fn value(&self, mut out: MutableHandleValue) {
        out.set(self.value.get());
    }
}

impl IDBCursorMethods<crate::DomTypeHolder> for IDBCursor {
    /// <https://w3c.github.io/IndexedDB/#dom-idbcursor-source>
    fn Source(&self) -> IDBObjectStoreOrIDBIndex {
        match &self.source {
            ObjectStoreOrIndexHandle::ObjectStoreHandle(object_store_handle) => {
                IDBObjectStoreOrIDBIndex::IDBObjectStore(object_store_handle.as_rooted())
            },
            ObjectStoreOrIndexHandle::IndexHandle(index_handle) => {
                IDBObjectStoreOrIDBIndex::IDBIndex(index_handle.as_rooted())
            },
        }
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbcursor-direction>
    fn Direction(&self) -> IDBCursorDirection {
        self.direction
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbcursor-key>
    fn Key(&self, _cx: SafeJSContext, _can_gc: CanGc, mut value: MutableHandleValue) {
        value.set(UndefinedValue());
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbcursor-primarykey>
    fn PrimaryKey(&self, _cx: SafeJSContext, _can_gc: CanGc, mut value: MutableHandleValue) {
        value.set(UndefinedValue());
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbcursor-request>
    fn Request(&self) -> DomRoot<IDBRequest> {
        self.request
            .get()
            .expect("IDBCursor.request should be set when cursor is opened")
    }
}
