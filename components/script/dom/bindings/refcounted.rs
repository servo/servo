/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::trace::trace_object;
use dom::bindings::utils::Reflectable;

use js::jsapi::{JSTracer, JSObject};

use libc;
use std::cell::RefCell;
use std::collections::hashmap::HashMap;

local_data_key!(pub LiveReferences: LiveDOMReferences)

pub struct LiveDOMReferences {
    table: RefCell<HashMap<*mut JSObject, uint>>
}

impl LiveDOMReferences {
    pub fn initialize() {
        LiveReferences.replace(Some(LiveDOMReferences {
            table: RefCell::new(HashMap::new()),
        }));
    }

    pub fn addref<T: Reflectable>(&self, obj: &T) {
        let reflector = obj.reflector().get_jsobject();
        let mut table = self.table.borrow_mut();
        let entry = table.find_or_insert(reflector, 0);
        *entry += 1;
    }

    pub fn release<T: Reflectable>(&self, obj: &T) {
        let reflector = obj.reflector().get_jsobject();
        let refcount = {
            let mut table = self.table.borrow_mut();
            let entry = table.get_mut(&reflector);
            assert!(*entry != 0);
            *entry -= 1;
            *entry
        };
        if refcount == 0 {
            assert!(self.table.borrow_mut().remove(&reflector));
        }
    }
}

impl Drop for LiveDOMReferences {
    fn drop(&mut self) {
        assert!(self.table.borrow().keys().count() == 0);
    }
}

pub extern fn trace_refcounted_objects(tracer: *mut JSTracer, data: *mut libc::c_void) {
    let table = data as *const RefCell<HashMap<*mut JSObject, uint>>;
    let table = unsafe { (*table).borrow() };
    for reflector in table.keys() {
        trace_object(tracer, "", *reflector);
    }
}

