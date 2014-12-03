/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DOMStringMapBinding;
use dom::bindings::codegen::Bindings::DOMStringMapBinding::DOMStringMapMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use servo_util::str::DOMString;

use std::collections::HashMap;

#[dom_struct]
pub struct DOMStringMap {
    map: DOMRefCell<HashMap<DOMString, DOMString>>,
    reflector_: Reflector,
}

impl DOMStringMap {
    fn new_inherited() -> DOMStringMap {
        DOMStringMap {
            map: DOMRefCell::new(HashMap::new()),
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: GlobalRef) -> Temporary<DOMStringMap> {
        reflect_dom_object(box DOMStringMap::new_inherited(),
                           global, DOMStringMapBinding::Wrap)
    }
}

impl<'a> DOMStringMapMethods for JSRef<'a, DOMStringMap> {
    fn NamedCreator(self, name: DOMString, value: DOMString) {
        self.map.borrow_mut().insert(name, value);
    }

    fn NamedDeleter(self, name: DOMString) {
        self.map.borrow_mut().remove(&name);
    }

    fn NamedSetter(self, name: DOMString, value: DOMString) {
        self.map.borrow_mut().insert(name, value);
    }

    fn NamedGetter(self, name: DOMString, found: &mut bool) -> DOMString {
        match self.map.borrow().get(&name) {
            Some(value) => {
                *found = true;
                value.clone()
            },
            None => {
                *found = false;
                String::new()
            }
        }
    }
}

impl Reflectable for DOMStringMap {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
