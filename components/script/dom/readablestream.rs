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
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::js::conversions::ToJSValConvertible;
use crate::script_runtime::JSContext as SafeJSContext;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject, JS_ValueToFunction};
use js::jsapi::{
    IsReadableStream, NewReadableDefaultStreamObject, ReadableStreamCancel,
    ReadableStreamGetReader, ReadableStreamIsLocked,
    ReadableStreamReaderMode as JSReadableStreamReaderMode, ReadableStreamTee,
};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{Handle, IntoHandle, IntoMutableHandle, MutableHandle};
use std::ptr::NonNull;
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

        ReadableStream::new(global, stream)
    }

    fn new_inherited(stream: Heap<*mut JSObject>) -> ReadableStream {
        ReadableStream {
            reflector_: Reflector::new(),
            stream,
        }
    }

    fn new(global: &GlobalScope, stream: Heap<*mut JSObject>) -> DomRoot<ReadableStream> {
        reflect_dom_object(
            Box::new(ReadableStream::new_inherited(stream)),
            global,
            Wrap,
        )
    }
}

impl ReadableStreamMethods for ReadableStream {
    fn Cancel(&self, reason: DOMString) -> Fallible<Rc<Promise>> {
        let cx = self.global().get_cx();
        cancel_readablestream(cx, &self.stream, reason)
    }

    fn GetReader(&self, cx: SafeJSContext) -> Fallible<NonNull<JSObject>> {
        get_stream_reader(cx, &self.stream)
    }

    fn Locked(&self) -> bool {
        stream_is_locked(self.global().get_cx(), &self.stream)
    }

    fn Tee(&self, cx: SafeJSContext) -> Fallible<JSVal> {
        tee_stream(cx, &self.stream)
    }
}

#[allow(unsafe_code)]
fn tee_stream(cx: SafeJSContext, stream: &Heap<*mut JSObject>) -> Fallible<JSVal> {
    unsafe {
        rooted!(in(*cx) let mut branch1_stream = UndefinedValue());
        rooted!(in(*cx) let mut branch2_stream = UndefinedValue());

        if !ReadableStreamTee(
            *cx,
            stream.handle(),
            MutableHandle::new(&mut branch1_stream.to_object()).into_handle_mut(),
            MutableHandle::new(&mut branch2_stream.to_object()).into_handle_mut(),
        ) {
            return Err(Error::Type(
                "The stream you are trying to tee is not a ReadableStream, or it is locked."
                    .to_string(),
            ));
        }

        Ok(to_frozen_array(&[*branch1_stream, *branch2_stream], cx))
    }
}

#[allow(unsafe_code)]
fn get_stream_reader(
    cx: SafeJSContext,
    stream: &Heap<*mut JSObject>,
) -> Fallible<NonNull<JSObject>> {
    unsafe {
        let stream = stream.handle();
        if !IsReadableStream(stream.get()) {
            return Err(Error::Type(
                "The stream you are trying to create a reader for is not a ReadableStream."
                    .to_string(),
            ));
        }

        // TODO: support ReadableStreamBYOBReader,
        // SM currently seems to only support default mode,
        // see https://github.com/servo/mozjs/blob/
        // bc4935b668171863e537d3fb0d911001a6742013/mozjs/js/public/Stream.h#L274
        Ok(NonNull::new_unchecked(ReadableStreamGetReader(
            *cx,
            stream,
            JSReadableStreamReaderMode::Default,
        )))
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
fn stream_is_locked(cx: SafeJSContext, stream: &Heap<*mut JSObject>) -> bool {
    let mut is_locked = false;
    unsafe {
        let stream = stream.handle();
        let is_locked_ptr = &mut is_locked as *mut _;
        ReadableStreamIsLocked(*cx, stream, is_locked_ptr);
    }
    is_locked
}

#[allow(unsafe_code)]
fn cancel_readablestream(
    cx: SafeJSContext,
    stream: &Heap<*mut JSObject>,
    reason: DOMString,
) -> Fallible<Rc<Promise>> {
    unsafe {
        let is_stream = IsReadableStream(stream.get());

        let is_locked = if is_stream {
            stream_is_locked(cx.clone(), stream)
        } else {
            false
        };

        if !is_stream || is_locked {
            return Err(Error::Type(
                "The stream you are trying to cancel is not a ReadableStream, or it is locked."
                    .to_string(),
            ));
        }

        rooted!(in(*cx) let mut reason_val = UndefinedValue());
        (*reason).to_jsval(*cx, reason_val.handle_mut());

        rooted!(in(*cx) let raw_cancel_promise = ReadableStreamCancel(*cx, stream.handle(), reason_val.handle().into_handle()));

        let cancel_promise = Promise::new_with_js_promise(raw_cancel_promise.handle(), cx);

        Ok(cancel_promise)
    }
}
