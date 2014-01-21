/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{Reflectable, Traceable, trace_reflector};
use dom::types::*;

use std::cast;
use std::libc;
use std::ptr;
use js::jsapi::{JSTracer, JSTRACE_OBJECT, JS_CallTracer};

impl Traceable for Node {
    fn trace(&self, tracer: *mut JSTracer) {
        fn trace_node(tracer: *mut JSTracer, node: Option<JSManaged<Node>>, name: &str) {
            if node.is_none() {
                return;
            }
            debug!("tracing {:s}", name);
            let node = node.unwrap();
            let obj = node.reflector().get_jsobject();
            assert!(obj.is_not_null());
            unsafe {
                (*tracer).debugPrinter = ptr::null();
                (*tracer).debugPrintIndex = -1;
                name.to_c_str().with_ref(|name| {
                    (*tracer).debugPrintArg = name as *libc::c_void;
                    JS_CallTracer(cast::transmute(tracer), obj, JSTRACE_OBJECT as u32);
                });
            }
        }
        debug!("tracing {:p}?:", self.reflector().get_jsobject());
        trace_node(tracer, self.parent_node, "parent");
        trace_node(tracer, self.first_child, "first child");
        trace_node(tracer, self.last_child, "last child");
        trace_node(tracer, self.next_sibling, "next sibling");
        trace_node(tracer, self.prev_sibling, "prev sibling");
        let owner_doc = self.owner_doc();
        trace_reflector(tracer, "document", owner_doc.reflector());
    }
}
