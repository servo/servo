/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dom_struct::dom_struct;
use js::gc::MutableHandleValue;
use script_bindings::codegen::GenericBindings::IDBIndexBinding::IDBIndexMethods;
use script_bindings::conversions::SafeToJSValConvertible;
use script_bindings::str::DOMString;

use crate::dom::bindings::import::base::SafeJSContext;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbobjectstore::KeyPath;
use crate::dom::indexeddb::idbobjectstore::IDBObjectStore;
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
}
