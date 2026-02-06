/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dom_struct::dom_struct;
use js::context::JSContext;
use js::gc::{HandleValue, MutableHandleValue};
use script_bindings::codegen::GenericBindings::IDBIndexBinding::IDBIndexMethods;
use script_bindings::conversions::SafeToJSValConvertible;
use script_bindings::error::{Error, ErrorResult, Fallible};
use script_bindings::str::DOMString;
use storage_traits::indexeddb::{AsyncOperation, AsyncReadOnlyOperation};

use crate::dom::bindings::import::base::SafeJSContext;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbobjectstore::KeyPath;
use crate::dom::idbrequest::IDBRequest;
use crate::dom::indexeddb::idbobjectstore::IDBObjectStore;
use crate::indexeddb::convert_value_to_key_range;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct IDBIndex {
    reflector_: Reflector,
    object_store: DomRoot<IDBObjectStore>,
    name: DOMString,
    multi_entry: bool,
    unique: bool,
    key_path: KeyPath,
}

impl IDBIndex {
    pub fn new_inherited(
        object_store: DomRoot<IDBObjectStore>,
        name: DOMString,
        multi_entry: bool,
        unique: bool,
        key_path: KeyPath,
    ) -> IDBIndex {
        IDBIndex {
            reflector_: Reflector::new(),
            object_store,
            name,
            multi_entry,
            unique,
            key_path,
        }
    }

    pub fn new(
        global: &GlobalScope,
        object_store: DomRoot<IDBObjectStore>,
        name: DOMString,
        multi_entry: bool,
        unique: bool,
        key_path: KeyPath,
        can_gc: CanGc,
    ) -> DomRoot<IDBIndex> {
        reflect_dom_object(
            Box::new(IDBIndex::new_inherited(
                object_store,
                name,
                multi_entry,
                unique,
                key_path,
            )),
            global,
            can_gc,
        )
    }

    pub(crate) fn name(&self) -> String {
        self.name.to_string()
    }

    pub(crate) fn key_path(&self) -> &KeyPath {
        &self.key_path
    }

    pub(crate) fn multi_entry(&self) -> bool {
        self.multi_entry
    }

    pub(crate) fn unique(&self) -> bool {
        self.unique
    }

    pub(crate) fn object_store(&self) -> &DomRoot<IDBObjectStore> {
        &self.object_store
    }

    fn verify_not_deleted(&self) -> ErrorResult {
        if !self.object_store.index_exists(&self.name) {
            return Err(Error::InvalidState(None));
        }
        self.object_store.verify_not_deleted()?;
        Ok(())
    }
}

impl IDBIndexMethods<crate::DomTypeHolder> for IDBIndex {
    /// <https://www.w3.org/TR/IndexedDB/#dom-idbindex-objectstore>
    fn ObjectStore(&self) -> DomRoot<IDBObjectStore> {
        self.object_store.clone()
    }

    /// <https://www.w3.org/TR/IndexedDB/#dom-idbindex-multientry>
    fn MultiEntry(&self) -> bool {
        self.multi_entry
    }

    /// <https://www.w3.org/TR/IndexedDB/#dom-idbindex-unique>
    fn Unique(&self) -> bool {
        self.unique
    }

    /// <https://www.w3.org/TR/IndexedDB/#dom-idbindex-keypath>
    fn KeyPath(&self, cx: SafeJSContext, can_gc: CanGc, retval: MutableHandleValue) {
        match &self.key_path {
            KeyPath::String(string) => {
                string.safe_to_jsval(cx, retval, can_gc);
            },
            KeyPath::StringSequence(sequence) => {
                sequence.safe_to_jsval(cx, retval, can_gc);
            },
        }
    }

    /// <https://www.w3.org/TR/IndexedDB/#dom-idbindex-get>
    fn Get(&self, cx: &mut JSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        // Step 3. If index or index’s object store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        self.object_store.check_transaction_active()?;

        // Step 5. Let range be the result of converting a value to a key range with query and true. Rethrow any exceptions.
        let range = convert_value_to_key_range(cx, query, None);

        // Step 6. Let operation be an algorithm to run retrieve a referenced value from an index with the current Realm record, index, and range.
        // Step 7. Return the result (an IDBRequest) of running asynchronously execute a request with this and operation.
        range.and_then(|q| {
            IDBRequest::execute_async(
                self,
                |callback| {
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::IndexGetItem {
                        callback,
                        index_name: self.name.to_string(),
                        key_range: q,
                    })
                },
                None,
                None,
                CanGc::from_cx(cx),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB/#dom-idbindex-getkey>
    fn GetKey(
        &self,
        cx: &mut JSContext,
        query_or_options: HandleValue,
    ) -> Fallible<DomRoot<IDBRequest>> {
        // Step 3. If index or index’s object store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        self.object_store.check_transaction_active()?;

        // Step 5. Let range be the result of converting a value to a key range with query and true. Rethrow any exceptions.
        let range = convert_value_to_key_range(cx, query_or_options, None);
        range.and_then(|q| {
            IDBRequest::execute_async(
                self,
                |callback| {
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::IndexGetKey {
                        callback,
                        index_name: self.name.to_string(),
                        key_range: q,
                    })
                },
                None,
                None,
                CanGc::from_cx(cx),
            )
        })
    }

    /// <https://www.w3.org/TR/IndexedDB/#dom-idbindex-count>
    fn Count(&self, cx: &mut JSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        // Step 3. If index or index’s object store has been deleted, throw an "InvalidStateError" DOMException.
        self.verify_not_deleted()?;

        // Step 4. If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        self.object_store.check_transaction_active()?;

        // Step 5. Let range be the result of converting a value to a key range with query. Rethrow any exceptions.
        let range = convert_value_to_key_range(cx, query, None);
        range.and_then(|q| {
            IDBRequest::execute_async(
                self,
                |callback| {
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::IndexCount {
                        callback,
                        index_name: self.name.to_string(),
                        key_range: q,
                    })
                },
                None,
                None,
                CanGc::from_cx(cx),
            )
        })
    }
}
