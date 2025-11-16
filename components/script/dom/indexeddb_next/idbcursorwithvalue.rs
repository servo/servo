/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::MutableHandleValue;

use crate::dom::bindings::codegen::Bindings::IDBCursorBinding::IDBCursorDirection;
use crate::dom::bindings::codegen::Bindings::IDBCursorWithValueBinding::IDBCursorWithValueMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb_next::idbcursor::{IDBCursor, ObjectStoreOrIndexHandle};
use crate::dom::indexeddb_next::idbtransaction::IDBTransaction;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// An "object" implementing the spec’s IDBCursorWithValue interface:
/// <https://w3c.github.io/IndexedDB/#idbcursorwithvalue>.
///
/// The IDBCursorWithValue interface extends IDBCursor and allows getting
/// values for cursors whose key only flag is set to false.
///
/// The IDBCursorWithValueis struct has a remote counterpart in the backend,
/// which performs some of the steps defined by the corresponding spec
/// algorithms.
#[dom_struct]
pub(crate) struct IDBCursorWithValue {
    cursor: IDBCursor,
}

impl IDBCursorWithValue {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn _new_inherited(
        transaction: &IDBTransaction,
        source: ObjectStoreOrIndexHandle,
        direction: IDBCursorDirection,
        key_only: bool,
    ) -> IDBCursorWithValue {
        IDBCursorWithValue {
            cursor: IDBCursor::_new_inherited(transaction, source, direction, key_only),
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
    ) -> DomRoot<IDBCursorWithValue> {
        reflect_dom_object(
            Box::new(IDBCursorWithValue::_new_inherited(
                transaction,
                source,
                direction,
                key_only,
            )),
            global,
            can_gc,
        )
    }
}

impl IDBCursorWithValueMethods<crate::DomTypeHolder> for IDBCursorWithValue {
    /// <https://w3c.github.io/IndexedDB/#dom-idbcursorwithvalue-value>
    fn Value(&self, _cx: SafeJSContext, value: MutableHandleValue) {
        self.cursor.value(value);
    }
}
