/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ffi::CString;

use constellation_traits::MessagePortImpl;
use embedder_traits::JSValue;
use js::jsapi::{Heap, JS_NewPlainObject, JSPROP_ENUMERATE};
use js::jsval::{JSVal, NullValue};
use js::rust::MutableHandleValue;
use js::rust::wrappers::JS_DefineProperty;
use script_bindings::conversions::SafeToJSValConvertible;
use script_bindings::realms::InRealm;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::{CanGc, JSContext};

use crate::dom::bindings::transferable::Transferable;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;

#[expect(unsafe_code)]
pub(crate) fn jsvalue_to_jsval(
    cx: JSContext,
    global: &GlobalScope,
    data: &JSValue,
    mut out: MutableHandleValue,
    in_realm: InRealm,
    can_gc: CanGc,
) -> Vec<DomRoot<MessagePort>> {
    let mut ports = vec![];
    match data {
        JSValue::Undefined => ().safe_to_jsval(cx, out, can_gc),
        JSValue::Null => out.set(NullValue()),
        JSValue::Boolean(b) => b.safe_to_jsval(cx, out, can_gc),
        JSValue::Number(n) => n.safe_to_jsval(cx, out, can_gc),
        JSValue::String(s) => s.safe_to_jsval(cx, out, can_gc),
        JSValue::Element(..) |
        JSValue::ShadowRoot(..) |
        JSValue::Frame(..) |
        JSValue::Window(..) => todo!(),
        JSValue::Array(a) => {
            rooted_vec!(let mut values);
            for v in a {
                rooted!(in(*cx) let mut value: JSVal);
                let inner_ports =
                    jsvalue_to_jsval(cx, global, v, value.handle_mut(), in_realm, can_gc);
                values.push(Heap::boxed(value.get()));
                ports.extend(inner_ports);
            }
            values.safe_to_jsval(cx, out, can_gc);
        },
        JSValue::Object(o) => {
            rooted!(in(*cx) let mut obj = unsafe { JS_NewPlainObject(*cx) });
            assert!(!obj.handle().is_null());
            for (key, value) in o {
                let Ok(key) = CString::new(key.as_bytes()) else {
                    continue;
                };
                rooted!(in(*cx) let mut js_value: JSVal);
                let inner_ports =
                    jsvalue_to_jsval(cx, global, value, js_value.handle_mut(), in_realm, can_gc);
                unsafe {
                    assert!(JS_DefineProperty(
                        *cx,
                        obj.handle(),
                        key.as_ptr(),
                        js_value.handle(),
                        JSPROP_ENUMERATE as _,
                    ));
                }
                ports.extend(inner_ports);
            }
        },
        JSValue::MessagePort { id, entangled, .. } => {
            let mut port_impl = MessagePortImpl::new(*id, true);
            port_impl.entangle(*entangled);
            port_impl.start();
            port_impl.set_has_been_shipped();
            let port = MessagePort::transfer_receive(global, *id, port_impl).unwrap();
            port.safe_to_jsval(cx, out, can_gc);
            ports.push(port);
        },
    }
    ports
}
