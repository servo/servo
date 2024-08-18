/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::{
    ESClass, GetBuiltinClass, IsArrayBufferObject, JSObject, JS_DeleteUCProperty,
    JS_GetOwnUCPropertyDescriptor, JS_GetStringLength, JS_IsArrayBufferViewObject, ObjectOpResult,
    ObjectOpResult_SpecialCodes, PropertyDescriptor,
};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{HandleValue, MutableHandleValue};
use net_traits::indexeddb_thread::{
    AsyncOperation, IndexedDBKeyType, IndexedDBThreadMsg, SyncOperation,
};
use net_traits::IpcSend;
use profile_traits::ipc;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::IDBDatabaseBinding::IDBObjectStoreParameters;
use crate::dom::bindings::codegen::Bindings::IDBObjectStoreBinding::IDBObjectStoreMethods;
use crate::dom::bindings::codegen::Bindings::IDBTransactionBinding::IDBTransactionMode;
// We need to alias this name, otherwise test-tidy complains at &String reference.
use crate::dom::bindings::codegen::UnionTypes::StringOrStringSequence as StrOrStringSequence;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone;
use crate::dom::domstringlist::DOMStringList;
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbrequest::IDBRequest;
use crate::dom::idbtransaction::IDBTransaction;
use crate::script_runtime::JSContext as SafeJSContext;

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

            index_names: DOMStringList::new(global, Vec::new()),
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
    ) -> DomRoot<IDBObjectStore> {
        reflect_dom_object(
            Box::new(IDBObjectStore::new_inherited(
                global, db_name, name, options,
            )),
            global,
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

    // https://www.w3.org/TR/IndexedDB-2/#valid-key-path
    pub fn is_valid_key_path(key_path: &StrOrStringSequence) -> bool {
        fn is_identifier(s: &str) -> bool {
            // FIXME: (arihant2math)
            true
        }

        let is_valid = |path: &DOMString| {
            let path = path.to_string();
            return if path.is_empty() {
                true
            } else if is_identifier(&path) {
                true
            } else {
                let parts = path.split(".");
                for part in parts {
                    if !is_identifier(part) {
                        return false;
                    }
                }
                true
            };
        };

        match key_path {
            StrOrStringSequence::StringSequence(paths) => {
                if paths.is_empty() {
                    return false;
                }

                for path in paths {
                    if !is_valid(path) {
                        return false;
                    }
                }
                true
            },
            StrOrStringSequence::String(path) => is_valid(path),
        }
    }

    #[allow(unsafe_code)]
    // https://www.w3.org/TR/IndexedDB-2/#convert-value-to-key
    fn convert_value_to_key(
        cx: SafeJSContext,
        input: HandleValue,
        seen: Option<Vec<HandleValue>>,
    ) -> Result<IndexedDBKeyType, Error> {
        // Step 1: If seen was not given, then let seen be a new empty set.
        let _seen = seen.unwrap_or(Vec::new());

        // Step 2: If seen contains input, then return invalid.
        // FIXME:(rasviitanen)
        // Check if we have seen this key
        // Does not currently work with HandleValue,
        // as it does not implement PartialEq

        // Step 3
        // FIXME:(rasviitanen) Accept buffer, array and date as well
        if input.is_number() {
            // FIXME:(rasviitanen) check for NaN
            let key = structuredclone::write(cx, input, None).expect("Could not serialize key");
            return Ok(IndexedDBKeyType::Number(key.serialized.clone()));
        }

        if input.is_string() {
            let key = structuredclone::write(cx, input, None).expect("Could not serialize key");
            return Ok(IndexedDBKeyType::String(key.serialized.clone()));
        }

        if input.is_object() {
            rooted!(in(*cx) let object = input.to_object());
            unsafe {
                let mut built_in_class = ESClass::Other;

                if !GetBuiltinClass(*cx, object.handle().into(), &mut built_in_class) {
                    return Err(Error::Data);
                }

                if let ESClass::Date = built_in_class {
                    // FIXME:(arihant2math) implement it the correct way
                    let key =
                        structuredclone::write(cx, input, None).expect("Could not serialize key");
                    return Ok(IndexedDBKeyType::Date(key.serialized.clone()));
                }

                if IsArrayBufferObject(*object) || JS_IsArrayBufferViewObject(*object) {
                    let key =
                        structuredclone::write(cx, input, None).expect("Could not serialize key");
                    // FIXME:(arihant2math) Return the correct type here
                    // it doesn't really matter at the moment...
                    return Ok(IndexedDBKeyType::Number(key.serialized.clone()));
                }

                if let ESClass::Array = built_in_class {
                    // FIXME:(arihant2math)
                    unimplemented!("Arrays as keys is currently unsupported");
                }
            }
        }

        Err(Error::Data)
    }

    // https://www.w3.org/TR/IndexedDB-2/#evaluate-a-key-path-on-a-value
    #[allow(unsafe_code)]
    fn evaluate_key_path_on_value(
        cx: SafeJSContext,
        value: HandleValue,
        mut return_val: MutableHandleValue,
        key_path: &KeyPath,
    ) {
        // The implementation is translated from gecko:
        // https://github.com/mozilla/gecko-dev/blob/master/dom/indexedDB/KeyPath.cpp
        *return_val = *value;

        rooted!(in(*cx) let mut target_object = ptr::null_mut::<JSObject>());
        rooted!(in(*cx) let mut current_val = *value);
        rooted!(in(*cx) let mut object = ptr::null_mut::<JSObject>());

        let mut target_object_prop_name: Option<String> = None;

        match key_path {
            KeyPath::String(path) => {
                // Step 3
                let path_as_string = path.to_string();
                let mut tokenizer = path_as_string.split('.').into_iter().peekable();

                while let Some(token) = tokenizer.next() {
                    if target_object.get().is_null() {
                        if token == "length" && current_val.is_string() {
                            rooted!(in(*cx) let input_val = current_val.to_string());
                            unsafe {
                                let string_len = JS_GetStringLength(*input_val) as f32;
                                string_len.to_jsval(*cx, return_val);
                            }
                            break;
                        }

                        if !current_val.is_object() {
                            // FIXME:(rasviitanen) Return a proper error
                            return;
                        }

                        *object = current_val.to_object();
                        rooted!(in(*cx) let mut desc = PropertyDescriptor::default());
                        rooted!(in(*cx) let mut intermediate = UndefinedValue());

                        // So rust says that this value is never read, but it is.
                        #[allow(unused)]
                        let mut has_prop = false;

                        unsafe {
                            let prop_name_as_utf16: Vec<u16> = token.encode_utf16().collect();
                            let ok = JS_GetOwnUCPropertyDescriptor(
                                *cx,
                                object.handle().into(),
                                prop_name_as_utf16.as_ptr(),
                                prop_name_as_utf16.len(),
                                desc.handle_mut().into(),
                                &mut false,
                            );

                            if !ok {
                                // FIXME:(arihant2math) Handle this
                                return;
                            }

                            if desc.hasValue_() {
                                *intermediate = desc.handle().value_;
                                has_prop = true;
                            } else {
                                // If we get here it means the object doesn't have the property or the
                                // property is available throuch a getter. We don't want to call any
                                // getters to avoid potential re-entrancy.
                                // The blob object is special since its properties are available
                                // only through getters but we still want to support them for key
                                // extraction. So they need to be handled manually.
                                unimplemented!("Blob tokens are not yet supported");
                            }
                        }

                        if has_prop {
                            // Treat undefined as an error
                            if intermediate.is_undefined() {
                                // FIXME:(rasviitanen) Throw/return error
                                return;
                            }

                            if tokenizer.peek().is_some() {
                                // ...and walk to it if there are more steps...
                                *current_val = *intermediate;
                            } else {
                                // ...otherwise use it as key
                                *return_val = *intermediate;
                            }
                        } else {
                            *target_object = *object;
                            target_object_prop_name = Some(token.to_string());
                        }
                    }

                    if !target_object.get().is_null() {
                        // We have started inserting new objects or are about to just insert
                        // the first one.
                        // FIXME:(rasviitanen) Implement this piece
                        unimplemented!("keyPath tokens that requires insertion are not supported.");
                    }
                } // All tokens processed

                if !target_object.get().is_null() {
                    // If this fails, we lose, and the web page sees a magical property
                    // appear on the object :-(
                    unsafe {
                        let prop_name_as_utf16: Vec<u16> =
                            target_object_prop_name.unwrap().encode_utf16().collect();
                        let mut succeeded = ObjectOpResult {
                            code_: ObjectOpResult_SpecialCodes::Uninitialized as usize,
                        };
                        if !JS_DeleteUCProperty(
                            *cx,
                            target_object.handle().into(),
                            prop_name_as_utf16.as_ptr(),
                            prop_name_as_utf16.len(),
                            &mut succeeded,
                        ) {
                            // FIXME:(rasviitanen) Throw/return error
                            return;
                        }
                    }
                }
            },
            KeyPath::StringSequence(_) => {
                unimplemented!("String sequence keyPath is currently unsupported");
            },
        }
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

    // https://www.w3.org/TR/IndexedDB-2/#extract-a-key-from-a-value-using-a-key-path
    fn extract_key(
        cx: SafeJSContext,
        input: HandleValue,
        key_path: &KeyPath,
        multi_entry: Option<bool>,
    ) -> Result<IndexedDBKeyType, Error> {
        // Step 1: Evaluate key path
        // FIXME:(rasviitanen) Do this propertly
        rooted!(in(*cx) let mut r = UndefinedValue());
        IDBObjectStore::evaluate_key_path_on_value(cx, input, r.handle_mut(), key_path);

        if let Some(_multi_entry) = multi_entry {
            // FIXME:(rasviitanen) handle multi_entry cases
            unimplemented!("multiEntry keys are not yet supported");
        } else {
            IDBObjectStore::convert_value_to_key(cx, r.handle(), None)
        }
    }

    // https://www.w3.org/TR/IndexedDB-2/#object-store-in-line-keys
    fn uses_inline_keys(&self) -> bool {
        self.key_path.is_some()
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-put
    fn put(
        &self,
        cx: SafeJSContext,
        value: HandleValue,
        key: HandleValue,
        overwrite: bool,
    ) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1: Let transaction be this object store handle's transaction.
        let transaction = self
            .transaction
            .get()
            .expect("No transaction in Object Store");

        // Step 2: Let store be this object store handle's object store.
        // This is resolved in the `execute_async` function.

        // Step 3: If store has been deleted, throw an "InvalidStateError" DOMException.
        // FIXME:(rasviitanen)

        // Step 4-5: If transaction is not active, throw a "TransactionInactiveError" DOMException.
        if !transaction.is_active() {
            return Err(Error::TransactionInactive);
        }

        // Step 5: If transaction is a read-only transaction, throw a "ReadOnlyError" DOMException.
        if let IDBTransactionMode::Readonly = transaction.get_mode() {
            return Err(Error::ReadOnly);
        }

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
            serialized_key = IDBObjectStore::convert_value_to_key(cx, key, None)?;
        } else {
            // Step 11: We should use in-line keys instead
            if let Ok(kpk) = IDBObjectStore::extract_key(
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

        let serialized_value =
            structuredclone::write(cx, value, None).expect("Could not serialize value");

        IDBRequest::execute_async(
            &*self,
            AsyncOperation::PutItem(
                serialized_key,
                serialized_value.serialized.clone(),
                overwrite,
            ),
            None,
        )
    }
}

impl IDBObjectStoreMethods for IDBObjectStore {
    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-put
    fn Put(
        &self,
        cx: SafeJSContext,
        value: HandleValue,
        key: HandleValue,
    ) -> Fallible<DomRoot<IDBRequest>> {
        self.put(cx, value, key, true)
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-add
    fn Add(
        &self,
        cx: SafeJSContext,
        value: HandleValue,
        key: HandleValue,
    ) -> Fallible<DomRoot<IDBRequest>> {
        self.put(cx, value, key, false)
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-delete
    fn Delete(&self, cx: SafeJSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        let serialized_query = IDBObjectStore::convert_value_to_key(cx, query, None);
        match serialized_query {
            Ok(q) => IDBRequest::execute_async(&*self, AsyncOperation::RemoveItem(q), None),
            Err(e) => Err(e),
        }
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-clear
    fn Clear(&self) -> Fallible<DomRoot<IDBRequest>> {
        unimplemented!();
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-get
    fn Get(&self, cx: SafeJSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        let serialized_query = IDBObjectStore::convert_value_to_key(cx, query, None);
        match serialized_query {
            Ok(q) => IDBRequest::execute_async(&*self, AsyncOperation::GetItem(q), None),
            Err(e) => Err(e),
        }
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-getkey
    fn GetKey(&self, _cx: SafeJSContext, _query: HandleValue) -> DomRoot<IDBRequest> {
        unimplemented!();
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-getall
    fn GetAll(
        &self,
        _cx: SafeJSContext,
        _query: HandleValue,
        _count: Option<u32>,
    ) -> DomRoot<IDBRequest> {
        unimplemented!();
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-getallkeys
    fn GetAllKeys(
        &self,
        _cx: SafeJSContext,
        _query: HandleValue,
        _count: Option<u32>,
    ) -> DomRoot<IDBRequest> {
        unimplemented!();
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-count
    fn Count(&self, cx: SafeJSContext, query: HandleValue) -> Fallible<DomRoot<IDBRequest>> {
        // Step 1
        let transaction = self.transaction.get().expect("Could not get transaction");

        // Step 2
        // FIXME(arihant2math): investigate further

        // Step 3
        // FIXME(arihant2math): Cannot tell if store has been deleted

        // Step 4
        if !transaction.is_active() {
            return Err(Error::TransactionInactive);
        }

        // Step 5
        let serialized_query = IDBObjectStore::convert_value_to_key(cx, query, None);

        // Step 6
        // match serialized_query {
        //     Ok(q) => IDBRequest::execute_async(&*self, AsyncOperation::Count(q), None),
        //     Err(e) => Err(e),
        // }
        Err(Error::NotSupported)
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-name
    fn Name(&self) -> DOMString {
        self.name.borrow().clone()
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-setname
    fn SetName(&self, value: DOMString) {
        std::mem::replace(&mut *self.name.borrow_mut(), value);
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-keypath
    fn KeyPath(&self, _cx: SafeJSContext) -> JSVal {
        unimplemented!();
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-indexnames
    fn IndexNames(&self) -> DomRoot<DOMStringList> {
        unimplemented!();
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-transaction
    fn Transaction(&self) -> DomRoot<IDBTransaction> {
        unimplemented!();
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbobjectstore-autoincrement
    fn AutoIncrement(&self) -> bool {
        // FIXME(arihant2math): This is wrong
        self.auto_increment
    }
}
