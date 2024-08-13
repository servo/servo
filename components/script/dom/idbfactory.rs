/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::IDBFactoryBinding;
use crate::dom::bindings::codegen::Bindings::IDBFactoryBinding::IDBFactoryMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbopendbrequest::IDBOpenDBRequest;
use crate::dom::promise::Promise;
use crate::script_runtime::JSContext as SafeJSContext;

use dom_struct::dom_struct;
use js::rust::HandleValue;
use servo_url::origin::ImmutableOrigin;
use std::rc::Rc;

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

    pub fn new(global: &GlobalScope) -> DomRoot<IDBFactory> {
        reflect_dom_object(
            Box::new(IDBFactory::new_inherited()),
            global,
            IDBFactoryBinding::Wrap,
        )
    }
}

impl IDBFactoryMethods for IDBFactory {
    // https://www.w3.org/TR/IndexedDB-2/#dom-idbfactory-open
    #[allow(unsafe_code)]
    fn Open(&self, name: DOMString, version: Option<u64>) -> Fallible<DomRoot<IDBOpenDBRequest>> {
        // Step 1: If version is 0 (zero), throw a TypeError.
        if version == Some(0) {
            return Err(Error::Type(
                "The version must be an integer >= 1".to_owned(),
            ));
        };

        // Step 2: Let origin be the origin of the global scope used to
        // access this IDBFactory.
        let global = self.global();
        let origin = global.origin();

        // Step 3: if origin is an opaque origin,
        // throw a "SecurityError" DOMException and abort these steps.
        if let ImmutableOrigin::Opaque(_) = origin.immutable() {
            return Err(Error::Security);
        }

        // Step 4: Let request be a new open request.
        let request = IDBOpenDBRequest::new(&self.global());

        // Step 5: Runs in parallel
        request.open_database(name, version);

        // Step 6
        Ok(request)
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbfactory-deletedatabase
    fn DeleteDatabase(&self, name: DOMString) -> Fallible<DomRoot<IDBOpenDBRequest>> {
        // Step 1: Let origin be the origin of the global scope used to
        // access this IDBFactory.
        let global = self.global();
        let origin = global.origin();

        // Step 2: if origin is an opaque origin,
        // throw a "SecurityError" DOMException and abort these steps.
        if let ImmutableOrigin::Opaque(_) = origin.immutable() {
            return Err(Error::Security);
        }

        // Step 3: Let request be a new open request
        let request = IDBOpenDBRequest::new(&self.global());

        // Step 4: Runs in parallel
        request.delete_database(name.to_string());

        // Step 5: Return request
        Ok(request)
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbfactory-databases
    fn Databases(&self) -> Rc<Promise> {
        unimplemented!();
    }

    // https://www.w3.org/TR/IndexedDB-2/#dom-idbfactory-cmp
    fn Cmp(&self, _cx: SafeJSContext, _first: HandleValue, _second: HandleValue) -> i16 {
        unimplemented!();
    }
}
