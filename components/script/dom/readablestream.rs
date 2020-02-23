/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::bindings::codegen::Bindings::ReadableStreamBinding::Wrap;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::js::conversions::ToJSValConvertible;
use crate::script_runtime::JSContext as SafeJSContext;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSFunction, JSObject, JS_ValueToFunction};
use js::jsapi::{IsReadableStream, NewReadableDefaultStreamObject};
use js::jsval::UndefinedValue;
use js::rust::{Handle, IntoHandle};
use std::ptr;
use std::rc::Rc;

#[dom_struct]
pub struct ReadableStream {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "SM handles JS values"]
    stream: Heap<*mut JSObject>,
}

#[allow(unsafe_code)]
fn construct_default_readablestream(
    cx: SafeJSContext,
    underlying_source: *mut JSObject,
    size: Rc<Function>,
    high_watermark: Finite<f64>,
    proto: *mut JSObject,
) -> Heap<*mut JSObject> {
    let heap = Heap::default();
    let source = Handle::new(&underlying_source);
    let proto = Handle::new(&proto);

    unsafe {
        rooted!(in(*cx) let mut size_val = UndefinedValue());
        size.to_jsval(*cx, size_val.handle_mut());

        let func = JS_ValueToFunction(*cx, size_val.handle().into_handle());
        let size_func = Handle::new(&func);

        rooted!(in(*cx)
            let stream = NewReadableDefaultStreamObject(
                *cx,
                source.into_handle(),
                size_func.into_handle(),
                *high_watermark,
                proto.into_handle()
            )
        );

        assert!(IsReadableStream(stream.get()));

        heap.set(stream.get());
    }

    heap
}

impl ReadableStream {
    /// <https://html.spec.whatwg.org/multipage/#dom-messagechannel>
    #[allow(non_snake_case)]
    pub fn Constructor(
        cx: SafeJSContext,
        global: &GlobalScope,
        underlying_source: *mut JSObject,
        size: Rc<Function>,
        high_watermark: Finite<f64>,
        proto: *mut JSObject,
    ) -> DomRoot<ReadableStream> {
        let stream =
            construct_default_readablestream(cx, underlying_source, size, high_watermark, proto);

        reflect_dom_object(
            Box::new(ReadableStream {
                reflector_: Reflector::new(),
                stream,
            }),
            global,
            Wrap,
        )
    }
}
