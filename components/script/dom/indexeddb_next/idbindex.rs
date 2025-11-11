/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::IDBIndexBinding::IDBIndexMethods;
use script_bindings::str::DOMString;

use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb_next::idbobjectstore::IDBObjectStore;
use crate::script_runtime::CanGc;

/// An "object" implementing the spec’s IDBIndex interface:
/// <https://w3c.github.io/IndexedDB/#index-interface>.
///
/// The IDBIndex interface represents an index handle:
/// <https://w3c.github.io/IndexedDB/#index-handle-construct>.
///
/// The index handle can be used to interact with the data stored in the
/// object store of the associated index, using properties of the stored
/// values.
#[dom_struct]
pub(crate) struct IDBIndex {
    reflector_: Reflector,

    /// <https://w3c.github.io/IndexedDB/#index-unique-flag>
    unique_flag: bool,
    /// <https://w3c.github.io/IndexedDB/#index-multientry-flag>
    multi_entry_flag: bool,

    /// <https://w3c.github.io/IndexedDB/#index-handle-object-store-handle>
    object_store_handle: DomRoot<IDBObjectStore>,
    /// <https://w3c.github.io/IndexedDB/#index-handle-name>
    name: DOMString,
}

impl IDBIndex {
    pub fn new_inherited(
        unique_flag: bool,
        multi_entry_flag: bool,
        object_store_handle: DomRoot<IDBObjectStore>,
        name: DOMString,
    ) -> IDBIndex {
        IDBIndex {
            reflector_: Reflector::new(),
            unique_flag,
            multi_entry_flag,
            object_store_handle,
            name,
        }
    }

    #[expect(dead_code)]
    pub fn new(
        global: &GlobalScope,
        unique_flag: bool,
        multi_entry_flag: bool,
        object_store_handle: DomRoot<IDBObjectStore>,
        name: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<IDBIndex> {
        reflect_dom_object(
            Box::new(IDBIndex::new_inherited(
                unique_flag,
                multi_entry_flag,
                object_store_handle,
                name,
            )),
            global,
            can_gc,
        )
    }
}

impl IDBIndexMethods<crate::DomTypeHolder> for IDBIndex {
    /// <https://w3c.github.io/IndexedDB/#dom-idbindex-objectstore>
    fn ObjectStore(&self) -> DomRoot<IDBObjectStore> {
        self.object_store_handle.clone()
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbindex-multientry>
    fn MultiEntry(&self) -> bool {
        self.multi_entry_flag
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbindex-unique>
    fn Unique(&self) -> bool {
        self.unique_flag
    }
}
