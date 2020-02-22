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
use js::jsapi::{Heap, JSFunction, JSObject};
use js::jsapi::NewReadableDefaultStreamObject;
use js::jsval::UndefinedValue;
use dom_struct::dom_struct;
use std::rc::Rc;
use std::ptr;

#[dom_struct]
pub struct ReadableStream {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "SM handles JS values"]
    stream: Heap<*mut JSObject>,
}

impl ReadableStream {
    /// <https://html.spec.whatwg.org/multipage/#dom-messagechannel>
    #[allow(non_snake_case, unsafe_code)]
    pub fn Constructor(
        cx: SafeJSContext,
        global: &GlobalScope,
        underlying_source: *mut JSObject,
        _size: Rc<Function>,
        high_watermark: Finite<f64>,
        proto: *mut JSObject,
    ) -> DomRoot<ReadableStream> {
        let heap = Heap::default();

        unsafe {
            let source = Heap::boxed(underlying_source);
            let proto = Heap::boxed(proto);
            rooted!(in(*cx) let mut size_handler = ptr::null_mut::<JSFunction>());
            let size = Heap::boxed(size_handler.get());

            rooted!(in(*cx) let stream = NewReadableDefaultStreamObject(*cx, source.handle(), size.handle(), *high_watermark, proto.handle()));
            heap.set(stream.get());
        }

        reflect_dom_object(
            Box::new(ReadableStream {
                reflector_: Reflector::new(),
                stream: heap,
            }),
            global,
            Wrap,
        )
    }
}