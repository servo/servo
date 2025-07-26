/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleValue;
use net_traits::IpcSend;
use net_traits::indexeddb_thread::{
    AsyncOperation, AsyncReadOnlyOperation, AsyncReadWriteOperation, IndexedDBKeyType,
    IndexedDBThreadMsg, SyncOperation,
};
use profile_traits::ipc;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBDatabaseBinding::IDBObjectStoreParameters;
use crate::dom::bindings::codegen::Bindings::IDBObjectStoreBinding::IDBObjectStoreMethods;
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::IDBTransactionMode;
// We need to alias this name, otherwise test-tidy complains at &String reference.
use crate::dom::bindings::codegen::UnionTypes::StringOrStringSequence as StrOrStringSequence;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone;
use crate::dom::domstringlist::DOMStringList;
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbrequest::IDBRequest;
use crate::dom::idbtransaction::IDBTransaction;
use crate::indexed_db::{convert_value_to_key, extract_key};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[derive(JSTraceable, MallocSizeOf)]
pub enum KeyPath {
    String(DOMString),
    StringSequence(Vec<DOMString>),
}

#[dom_struct]
pub struct IDBObjectStore {
    reflector_: Reflector,
    name: DomRefCell<DOMString>,
    key_path: Option<KeyPath>,
    index_names: DomRoot<DOMStringList>,
    transaction: MutNullableDom<IDBTransaction>,
    auto_increment: bool,

    // We store the db name in the object store to be able to find the correct
    // store in the idb thread when checking if we have a key generator
    db_name: DOMString,
}

impl IDBObjectStore {
    pub fn new_inherited(
        global: &GlobalScope,
        db_name: DOMString,
        name: DOMString,
        options: Option<&IDBObjectStoreParameters>,
        can_gc: CanGc,
    ) -> IDBObjectStore {
        let key_path: Option<KeyPath> = match options {
            Some(options) => options.keyPath.as_ref().map(|path| match path {
                StrOrStringSequence::String(inner) => KeyPath::String(inner.clone()),
                StrOrStringSequence::StringSequence(inner) => {
                    KeyPath::StringSequence(inner.clone())
                },
            }),
            None => None,
        };

        IDBObjectStore {
            reflector_: Reflector::new(),
            name: DomRefCell::new(name),
            key_path,

            index_names: DOMStringList::new(global, Vec::new(), can_gc),
            transaction: Default::default(),
            // FIXME:(arihant2math)
            auto_increment: false,

            db_name,
        }
    }

    pub fn new(
        global: &GlobalScope,
        db_name: DOMString,
        name: DOMString,
        options: Option<&IDBObjectStoreParameters>,
        can_gc: CanGc,
    ) -> DomRoot<IDBObjectStore> {
        reflect_dom_object(
            Box::new(IDBObjectStore::new_inherited(
                global, db_name, name, options, can_gc,
            )),
            global,
            can_gc,
        )
    }

    pub fn get_name(&self) -> DOMString {
        self.name.borrow().clone()
    }

    pub fn set_transaction(&self, transaction: &IDBTransaction) {
        self.transaction.set(Some(transaction));
    }

    pub fn transaction(&self) -> Option<DomRoot<IDBTransaction>> {
        self.transaction.get()
    }

    fn has_key_generator(&self) -> bool {
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();

        let operation = SyncOperation::HasKeyGenerator(
            sender,
            self.global().origin().immutable().clone(),
            self.db_name.to_string(),
            self.name.borrow().to_string(),
        );

        self.global()
            .resource_threads()
            .sender()
            .send(IndexedDBThreadMsg::Sync(operation))
            .unwrap();

        receiver.recv().unwrap()
    }

    // https://www.w3.org/TR/IndexedDB-2/#object-store-in-line-keys
    fn uses_inline_keys(&self) -> bool {
        self.key_path.is_some()
    }

    /// Checks if the transation is active, throwing a "TransactionInactiveError" DOMException if not.
    fn check_transaction_active(&self) -> Fallible<()> {
        // Let transaction be this object store handle's transaction.
        let transaction = self.transaction.get().ok_or(Error::TransactionInactive)?;

        // If transaction is not active, throw a "TransactionInactiveError" DOMException.
        if !transaction.is_active() {
            return Err(Error::TransactionInactive);
        }

        Ok(())
    }

    /// Checks if the transation is active, throwing a "TransactionInactiveError" DOMException if not.
    /// it then checks if the transaction is a read-only transaction, throwing a "ReadOnlyError" DOMException if so.
    fn check_readwrite_transaction_active(&self) -> Fallible<()> {
        // Let transaction be this object store handle's transaction.
        let transaction = self.transaction.get().ok_or(Error::TransactionInactive)?;

        // If transaction is not active, throw a "TransactionInactiveError" DOMException.
        if !transaction.is_active() {
            return Err(Error::TransactionInactive);
        }

        if let IDBTransactionMode::Readonly = transaction.get_mode() {
            return Err(Error::ReadOnly);
        }
        Ok(())
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-put
    fn put(
        &self,
        cx: SafeJSContext,
        value: HandleValue,
        key: HandleValue,
        overwrite: bool,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1: Unneeded, handled by self.check_readwrite_transaction_active()
        // Step 2: Let store be this object store handle's object store.
        // This is resolved in the `execute_async` function.

        // Step 3: If store has been deleted, throw an "InvalidStateError" DOMException.
        // FIXME:(rasviitanen)

        // Steps 4-5
        self.check_readwrite_transaction_active()?;

        // Step 6: If store uses in-line keys and key was given, throw a "DataError" DOMException.
        if !key.is_undefined() && self.uses_inline_keys() {
            return Err(Error::Data);
        }

        // Step 7: If store uses out-of-line keys and has no key generator
        // and key was not given, throw a "DataError" DOMException.
        if !self.uses_inline_keys() && !self.has_key_generator() && key.is_undefined() {
            return Err(Error::Data);
        }

        // Step 8: If key was given, then: convert a value to a key with key
        let serialized_key: IndexedDBKeyType;

        if !key.is_undefined() {
            serialized_key = convert_value_to_key(cx, key, None)?;
        } else {
            // Step 11: We should use in-line keys instead
            if let Ok(kpk) = extract_key(
                cx,
                value,
                self.key_path.as_ref().expect("No key path"),
                None,
            ) {
                serialized_key = kpk;
            } else {
                // FIXME:(rasviitanen)
                // Check if store has a key generator
                // Check if we can inject a key
                return Err(Error::Data);
            }
        }

        let serialized_value = structuredclone::write(cx, value, None)?;

        IDBRequest::execute_async(
            self,
            AsyncOperation::ReadWrite(AsyncReadWriteOperation::PutItem(
                serialized_key,
                serialized_value.serialized,
                overwrite,
            )),
            None,
            can_gc,
        )
    }
}

impl IDBObjectStoreMethods<crate::DomTypeHolder> for IDBObjectStore {
    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-put
    fn Put(
        &self,
        cx: SafeJSContext,
        value: HandleValue,
        key: HandleValue,
    ) -> Fallible<DomRoot<IDBRequest>> {
        self.put(cx, value, key, true, CanGc::note())
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-add
    fn Add(
        &self,
        cx: SafeJSContext,
        value: HandleValue,
        key: HandleValue,
    ) -> Fallible<DomRoot<IDBRequest>> {
        self.put(cx, value, key, false, CanGc::note())
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-delete
    fn Delete(&self, cx: SafeJSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1: Unneeded, handled by self.check_readwrite_transaction_active()
        // TODO: Step 2
        // TODO: Step 3
        // Steps 4-5
        self.check_readwrite_transaction_active()?;
        // Step 6
        // TODO: Convert to key range instead
        let serialized_query = convert_value_to_key(cx, query, None);
        // Step 7
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                AsyncOperation::ReadWrite(AsyncReadWriteOperation::RemoveItem(q)),
                None,
                CanGc::note(),
            )
        })
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-clear
    fn Clear(&self) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1: Unneeded, handled by self.check_readwrite_transaction_active()
        // TODO: Step 2
        // TODO: Step 3
        // Steps 4-5
        self.check_readwrite_transaction_active()?;
        IDBRequest::execute_async(
            self,
            AsyncOperation::ReadWrite(AsyncReadWriteOperation::Clear),
            None,
            CanGc::note(),
        )
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-get
    fn Get(&self, cx: SafeJSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1: Unneeded, handled by self.check_transaction_active()
        // TODO: Step 2
        // TODO: Step 3
        // Step 4
        self.check_transaction_active()?;
        // Step 5
        // TODO: Convert to key range instead
        let serialized_query = convert_value_to_key(cx, query, None);
        // Step 6
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                AsyncOperation::ReadOnly(AsyncReadOnlyOperation::GetItem(q)),
                None,
                CanGc::note(),
            )
        })
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-getkey
    // fn GetKey(&self, _cx: SafeJSContext, _query: HandleValue) -> DomRoot<IDBRequest> {
    //     // Step 1: Unneeded, handled by self.check_transaction_active()
    //     // TODO: Step 2
    //     // TODO: Step 3
    //     // Step 4
    //     self.check_transaction_active()?;
    //     // Step 5
    //     // TODO: Convert to key range instead
    //     let serialized_query = IDBObjectStore::convert_value_to_key(cx, query, None);
    //     unimplemented!();
    // }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-getall
    // fn GetAll(
    //     &self,
    //     _cx: SafeJSContext,
    //     _query: HandleValue,
    //     _count: Option<u32>,
    // ) -> DomRoot<IDBRequest> {
    //     unimplemented!();
    // }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-getallkeys
    // fn GetAllKeys(
    //     &self,
    //     _cx: SafeJSContext,
    //     _query: HandleValue,
    //     _count: Option<u32>,
    // ) -> DomRoot<IDBRequest> {
    //     unimplemented!();
    // }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-count
    fn Count(&self, cx: SafeJSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1: Unneeded, handled by self.check_transaction_active()
        // TODO: Step 2
        // TODO: Step 3
        // Steps 4
        self.check_transaction_active()?;

        // Step 5
        let serialized_query = convert_value_to_key(cx, query, None);

        // Step 6
        serialized_query.and_then(|q| {
            IDBRequest::execute_async(
                self,
                AsyncOperation::ReadOnly(AsyncReadOnlyOperation::Count(q)),
                None,
                CanGc::note(),
            )
        })
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-name
    fn Name(&self) -> DOMString {
        self.name.borrow().clone()
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-setname
    fn SetName(&self, value: DOMString) {
        *self.name.borrow_mut() = value;
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-keypath
    // fn KeyPath(&self, _cx: SafeJSContext, _val: MutableHandleValue) {
    //     unimplemented!();
    // }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-indexnames
    // fn IndexNames(&self) -> DomRoot<DOMStringList> {
    //     unimplemented!();
    // }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-transaction
    // fn Transaction(&self) -> DomRoot<IDBTransaction> {
    //     unimplemented!();
    // }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-autoincrement
    fn AutoIncrement(&self) -> bool {
        // FIXME(arihant2math): This is wrong
        self.auto_increment
    }
}
