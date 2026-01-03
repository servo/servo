/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::MutableHandleValue;
use storage_traits::indexeddb::IndexedDBKeyRange;

use crate::dom::bindings::codegen::Bindings::IDBCursorBinding::IDBCursorDirection;
use crate::dom::bindings::codegen::Bindings::IDBCursorWithValueBinding::IDBCursorWithValueMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb::idbcursor::{IDBCursor, ObjectStoreOrIndex};
use crate::dom::indexeddb::idbtransaction::IDBTransaction;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[dom_struct]
pub(crate) struct IDBCursorWithValue {
    cursor: IDBCursor,
}

impl IDBCursorWithValue {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(
        transaction: &IDBTransaction,
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

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new(
        global: &GlobalScope,
        transaction: &IDBTransaction,
        direction: IDBCursorDirection,
        got_value: bool,
        source: ObjectStoreOrIndex,
        range: IndexedDBKeyRange,
        key_only: bool,
        can_gc: CanGc,
    ) -> DomRoot<IDBCursorWithValue> {
        reflect_dom_object(
            Box::new(IDBCursorWithValue::new_inherited(
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
}

impl IDBCursorWithValueMethods<crate::DomTypeHolder> for IDBCursorWithValue {
    /// <https://www.w3.org/TR/IndexedDB-2/#dom-idbcursorwithvalue-value>
    fn Value(&self, _cx: SafeJSContext, value: MutableHandleValue) {
        self.cursor.value(value);
    }
}
