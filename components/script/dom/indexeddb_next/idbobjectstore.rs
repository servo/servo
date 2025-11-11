/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::gc::MutableHandleValue;
use js::rust::HandleValue;
use script_bindings::error::ErrorResult;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBCursorBinding::IDBCursorDirection;
use crate::dom::bindings::codegen::Bindings::IDBDatabaseBinding::IDBObjectStoreParameters;
use crate::dom::bindings::codegen::Bindings::IDBObjectStoreBinding::IDBObjectStoreMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::domstringlist::DOMStringList;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb_next::idbrequest::IDBRequest;
use crate::dom::indexeddb_next::idbtransaction::IDBTransaction;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// An "object" implementing the spec’s IDBObjectStore interface:
/// <https://w3c.github.io/IndexedDB/#object-store-interface>.
///
/// The IDBObjectStore interface represents an object store handle:
/// <https://w3c.github.io/IndexedDB/#object-store-handle-construct>.
///
/// The object store handle can be used to interact with the data stored in
/// the associated object store using object store keys.
#[dom_struct]
pub struct IDBObjectStore {
    reflector_: Reflector,

    /// <https://w3c.github.io/IndexedDB/#object-store-handle-transaction>
    transaction: Dom<IDBTransaction>,
    /// <https://w3c.github.io/IndexedDB/#object-store-handle-name>
    name: DomRefCell<DOMString>,

    index_names: DomRefCell<Vec<DOMString>>,
}

impl IDBObjectStore {
    pub fn _new_inherited(
        transaction: &IDBTransaction,
        name: DOMString,
        _options: Option<&IDBObjectStoreParameters>,
    ) -> IDBObjectStore {
        IDBObjectStore {
            reflector_: Reflector::new(),
            transaction: Dom::from_ref(transaction),
            name: DomRefCell::new(name),
            index_names: Default::default(),
        }
    }

    pub fn _new(
        global: &GlobalScope,
        transaction: &IDBTransaction,
        name: DOMString,
        options: Option<&IDBObjectStoreParameters>,
        can_gc: CanGc,
    ) -> DomRoot<IDBObjectStore> {
        reflect_dom_object(
            Box::new(IDBObjectStore::_new_inherited(transaction, name, options)),
            global,
            can_gc,
        )
    }
}

impl IDBObjectStoreMethods<crate::DomTypeHolder> for IDBObjectStore {
    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-name>
    fn Name(&self) -> DOMString {
        self.name.borrow().clone()
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-name>
    fn SetName(&self, _value: DOMString) -> ErrorResult {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-keypath>
    fn KeyPath(&self, _cx: SafeJSContext, mut _ret_val: MutableHandleValue) {}

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-indexnames>
    fn IndexNames(&self) -> DomRoot<DOMStringList> {
        DOMStringList::new(
            &self.global(),
            self.index_names.borrow().clone(),
            CanGc::note(),
        )
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-transaction>
    fn Transaction(&self) -> DomRoot<IDBTransaction> {
        self.transaction.as_rooted()
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-autoincrement>
    fn AutoIncrement(&self) -> bool {
        //        self.has_key_generator()
        false
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-put>
    fn Put(
        &self,
        _cx: SafeJSContext,
        _value: HandleValue,
        _key: HandleValue,
    ) -> Fallible<DomRoot<IDBRequest>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-add>
    fn Add(
        &self,
        _cx: SafeJSContext,
        _value: HandleValue,
        _key: HandleValue,
    ) -> Fallible<DomRoot<IDBRequest>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-delete>
    fn Delete(&self, _cx: SafeJSContext, _query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-clear>
    fn Clear(&self) -> Fallible<DomRoot<IDBRequest>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-get>
    fn Get(&self, _cx: SafeJSContext, _query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-getkey>
    fn GetKey(
        &self,
        _cx: SafeJSContext,
        _query: HandleValue,
    ) -> Result<DomRoot<IDBRequest>, Error> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-getall>
    fn GetAll(
        &self,
        _cx: SafeJSContext,
        _query: HandleValue,
        _count: Option<u32>,
    ) -> Fallible<DomRoot<IDBRequest>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-getallkeys>
    fn GetAllKeys(
        &self,
        _cx: SafeJSContext,
        _query: HandleValue,
        _count: Option<u32>,
    ) -> Fallible<DomRoot<IDBRequest>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-count>
    fn Count(&self, _cx: SafeJSContext, _query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-opencursor>
    fn OpenCursor(
        &self,
        _cx: SafeJSContext,
        _query: HandleValue,
        _direction: IDBCursorDirection,
    ) -> Fallible<DomRoot<IDBRequest>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbobjectstore-openkeycursor>
    fn OpenKeyCursor(
        &self,
        _cx: SafeJSContext,
        _query: HandleValue,
        _direction: IDBCursorDirection,
    ) -> Fallible<DomRoot<IDBRequest>> {
        Err(Error::NotSupported)
    }
}
