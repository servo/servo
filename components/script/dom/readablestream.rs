/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::bindings::codegen::Bindings::ReadableStreamBinding::{ReadableStreamMethods, Wrap};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::js::conversions::ToJSValConvertible;
use crate::script_runtime::JSContext as SafeJSContext;
use dom_struct::dom_struct;
use js::jsapi::{HandleObject, Heap, JSFunction, JSObject, JS_ValueToFunction};
use js::jsapi::{
    IsReadableStream, NewReadableDefaultStreamObject, ReadableStreamCancel, ReadableStreamIsLocked,
};
use js::jsval::{BooleanValue, UndefinedValue};
use js::rust::{Handle, IntoHandle};
use std::ptr;
use std::rc::Rc;

#[dom_struct]
pub struct ReadableStream {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "SM handles JS values"]
    stream: Heap<*mut JSObject>,
}

impl ReadableStream {
    /// <https://streams.spec.whatwg.org/#rs-class>
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

    pub fn Cancel(&self, reason: DOMString) -> Fallible<Rc<Promise>> {
        let cx = self.global().get_cx();
        cancel_readablestream(cx, &self.stream, reason)
    }
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

#[allow(unsafe_code)]
fn cancel_readablestream(
    cx: SafeJSContext,
    stream: &Heap<*mut JSObject>,
    reason: DOMString,
) -> Fallible<Rc<Promise>> {
    unsafe {
        let stream = stream.handle();

        let mut is_locked = false;
        let is_locked_ptr = &mut is_locked as *mut _;
        ReadableStreamIsLocked(*cx, stream, is_locked_ptr);

        let is_stream = IsReadableStream(stream.get());

        if !is_stream || is_locked {
            return Err(Error::Type(
                "The stream you are trying to cancel is not a ReadableStream, or it is locked."
                    .to_string(),
            ));
        }

        rooted!(in(*cx) let mut reason_val = UndefinedValue());
        (*reason).to_jsval(*cx, reason_val.handle_mut());

        rooted!(in(*cx) let raw_cancel_promise = ReadableStreamCancel(*cx, stream, reason_val.handle().into_handle()));

        let cancel_promise = Promise::new_with_js_promise(raw_cancel_promise.handle(), cx);

        Ok(cancel_promise)
    }
}
