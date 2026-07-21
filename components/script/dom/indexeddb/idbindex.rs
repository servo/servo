/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dom_struct::dom_struct;
use js::context::JSContext;
use js::gc::{HandleValue, MutableHandleValue};
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::IDBIndexBinding::IDBIndexMethods;
use script_bindings::codegen::GenericBindings::IDBTransactionBinding::IDBTransactionMode;
use script_bindings::conversions::SafeToJSValConvertible;
use script_bindings::error::{Error, ErrorResult, Fallible};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::str::DOMString;
use storage_traits::indexeddb::{AsyncOperation, AsyncReadOnlyOperation};

use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbobjectstore::KeyPath;
use crate::dom::idbrequest::IDBRequest;
use crate::dom::indexeddb::idbobjectstore::IDBObjectStore;
use crate::indexeddb::convert_value_to_key_range;

#[dom_struct]
pub(crate) struct IDBIndex {
    reflector_: Reflector,
    object_store: Dom<IDBObjectStore>,
    name: DomRefCell<DOMString>,
    multi_entry: bool,
    unique: bool,
    key_path: KeyPath,
}

impl IDBIndex {
    pub fn new_inherited(
        object_store: &IDBObjectStore,
        name: DOMString,
        multi_entry: bool,
        unique: bool,
        key_path: KeyPath,
    ) -> IDBIndex {
        IDBIndex {
            reflector_: Reflector::new(),
            object_store: Dom::from_ref(object_store),
            name: DomRefCell::new(name),
            multi_entry,
            unique,
            key_path,
        }
    }

    pub fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        object_store: &IDBObjectStore,
        name: DOMString,
        multi_entry: bool,
        unique: bool,
        key_path: KeyPath,
    ) -> DomRoot<IDBIndex> {
        reflect_dom_object_with_cx(
            Box::new(IDBIndex::new_inherited(
                object_store,
                name,
                multi_entry,
                unique,
                key_path,
            )),
            global,
            cx,
        )
    }

    pub(crate) fn name(&self) -> String {
        self.name.borrow().to_string()
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

    pub(crate) fn object_store(&self) -> DomRoot<IDBObjectStore> {
        self.object_store.as_rooted()
    }

    fn verify_not_deleted(&self) -> ErrorResult {
        if !self.object_store.index_exists(&self.name.borrow()) {
            return Err(Error::InvalidState(None));
        }
        self.object_store.verify_not_deleted()?;
        Ok(())
    }
}

impl IDBIndexMethods<crate::DomTypeHolder> for IDBIndex {
    /// <https://www.w3.org/TR/IndexedDB/#dom-idbindex-name>
    fn Name(&self) -> DOMString {
        self.name.borrow().clone()
    }

    /// <https://www.w3.org/TR/IndexedDB/#ref-for-dom-idbindex-name%E2%91%A2>
    fn SetName(&self, name: DOMString) -> ErrorResult {
        // Step 1: Let name be the given value.
        // Step 2: Let transaction be this’s transaction.
        let transaction = self.object_store.transaction();

        // Step 3: Let index be this’s index.
        // We do not have an explicit object representing the underlying index.

        // Step 4: If transaction is not an upgrade transaction, throw an "InvalidStateError" DOMException.
        if transaction.get_mode() != IDBTransactionMode::Versionchange {
            return Err(Error::InvalidState(Some(
                "Transaction is not an upgrade transaction".to_owned(),
            )));
        }

        // Step 5: If transaction’s state is not active, then throw a "TransactionInactiveError" DOMException.
        if !transaction.is_active() {
            return Err(Error::TransactionInactive(Some(
                "Transaction is not active while updating index name".to_owned(),
            )));
        }

        // Step 6: If index or index’s object store has been deleted, throw an "InvalidStateError" DOMException.
        let mut stored_name = self.name.borrow_mut();
        if !self.object_store.has_index(&stored_name) ||
            !transaction
                .get_db()
                .object_store_exists(&self.object_store.get_name())
        {
            return Err(Error::InvalidState(Some(
                "Index or its object store has been deleted".to_owned(),
            )));
        }

        // Step 7: If index’s name is equal to name, terminate these steps.
        if *stored_name == name {
            return Ok(());
        }

        // Step 8: If an index named name already exists in index’s object store, throw a "ConstraintError" DOMException.
        if self.object_store.has_index(&name) {
            return Err(Error::Constraint(Some(
                "An index with the given name already exists".to_owned(),
            )));
        }

        // Step 9: Set index’s name to name.
        self.object_store.rename_index(&stored_name, &name);

        // Step 10: Set this’s name to name.
        *stored_name = name;
        Ok(())
    }

    /// <https://www.w3.org/TR/IndexedDB/#dom-idbindex-objectstore>
    fn ObjectStore(&self) -> DomRoot<IDBObjectStore> {
        self.object_store.as_rooted()
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
    fn KeyPath(&self, cx: &mut JSContext, retval: MutableHandleValue) {
        match &self.key_path {
            KeyPath::String(string) => {
                string.safe_to_jsval(cx, retval);
            },
            KeyPath::StringSequence(sequence) => {
                sequence.safe_to_jsval(cx, retval);
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
                cx,
                self,
                |callback| {
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::IndexGetItem {
                        callback,
                        index_name: self.name.borrow().to_string(),
                        key_range: q,
                    })
                },
                None,
                None,
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
                cx,
                self,
                |callback| {
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::IndexGetKey {
                        callback,
                        index_name: self.name.borrow().to_string(),
                        key_range: q,
                    })
                },
                None,
                None,
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
                cx,
                self,
                |callback| {
                    AsyncOperation::ReadOnly(AsyncReadOnlyOperation::IndexCount {
                        callback,
                        index_name: self.name.borrow().to_string(),
                        key_range: q,
                    })
                },
                None,
                None,
            )
        })
    }
}
