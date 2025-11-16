/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleValue;

use crate::dom::bindings::codegen::Bindings::IDBFactoryBinding::IDBFactoryMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::import::base::SafeJSContext;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::indexeddb_next::idbopendbrequest::IDBOpenDBRequest;
use crate::script_runtime::CanGc;

/// An "object" implementing the spec's IDBFactory interface:
/// <https://w3c.github.io/IndexedDB/#factory-interface>.
///
/// In the spec, this represents the global `indexedDB` entry point.
///
/// The IDBFactory struct has a remote counterpart in the backend, which
/// performs some of the steps defined by the corresponding spec algorithms.
#[dom_struct]
pub struct IDBFactory {
    reflector_: Reflector,
}

impl IDBFactory {
    pub fn new_inherited() -> IDBFactory {
        IDBFactory {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<IDBFactory> {
        reflect_dom_object(Box::new(IDBFactory::new_inherited()), global, can_gc)
    }
}

impl IDBFactoryMethods<crate::DomTypeHolder> for IDBFactory {
    /// <https://w3c.github.io/IndexedDB/#dom-idbfactory-open>
    fn Open(&self, _name: DOMString, _version: Option<u64>) -> Fallible<DomRoot<IDBOpenDBRequest>> {
        Err(Error::NotSupported)
    }

    /// <https://w3c.github.io/IndexedDB/#dom-idbfactory-deletedatabase>
    fn DeleteDatabase(&self, _name: DOMString) -> Fallible<DomRoot<IDBOpenDBRequest>> {
        Err(Error::NotSupported)
    }

    // /// <https://w3c.github.io/IndexedDB/#dom-idbfactory-databases>
    // fn Databases(&self) -> Rc<Promise> {
    //     unimplemented!();
    // }

    /// <https://w3c.github.io/IndexedDB/#dom-idbfactory-cmp>
    fn Cmp(&self, _cx: SafeJSContext, _first: HandleValue, _second: HandleValue) -> Fallible<i16> {
        Err(Error::NotSupported)
    }
}
