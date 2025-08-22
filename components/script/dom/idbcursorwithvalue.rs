/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::{HandleObject, MutableHandleValue};
use net_traits::indexeddb_thread::IndexedDBKeyRange;

use crate::dom::bindings::codegen::Bindings::IDBCursorBinding::IDBCursorDirection;
use crate::dom::bindings::codegen::Bindings::IDBCursorWithValueBinding::IDBCursorWithValueMethods;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbcursor::{IDBCursor, ObjectStoreOrIndex};
use crate::dom::idbtransaction::IDBTransaction;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[dom_struct]
pub(crate) struct IDBCursorWithValue {
    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbcursorwithvalue-value>
    cursor: IDBCursor,
}

impl IDBCursorWithValue {
    pub(crate) fn new_inherited(
        transaction: DomRoot<IDBTransaction>,
        direction: IDBCursorDirection,
        got_value: bool,
        source: ObjectStoreOrIndex,
        range: IndexedDBKeyRange,
        key_only: bool,
    ) -> IDBCursorWithValue {
        IDBCursorWithValue {
            cursor: IDBCursor::new_inherited(
                transaction,
                direction,
                got_value,
                source,
                range,
                key_only,
            ),
        }
    }

    #[expect(dead_code)]
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
    ) -> DomRoot<IDBCursorWithValue> {
        IDBCursorWithValue::new_with_proto(
            global,
            None,
            transaction,
            direction,
            got_value,
            source,
            range,
            key_only,
            can_gc,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        transaction: DomRoot<IDBTransaction>,
        direction: IDBCursorDirection,
        got_value: bool,
        source: ObjectStoreOrIndex,
        range: IndexedDBKeyRange,
        key_only: bool,
        can_gc: CanGc,
    ) -> DomRoot<IDBCursorWithValue> {
        reflect_dom_object_with_proto(
            Box::new(IDBCursorWithValue::new_inherited(
                transaction,
                direction,
                got_value,
                source,
                range,
                key_only,
            )),
            global,
            proto,
            can_gc,
        )
    }
}

impl IDBCursorWithValueMethods<crate::DomTypeHolder> for IDBCursorWithValue {
    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbcursorwithvalue-value>
    fn Value(&self, _cx: SafeJSContext, mut value: MutableHandleValue) {
        value.set(self.cursor.get_value());
    }
}
