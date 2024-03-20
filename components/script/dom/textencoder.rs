/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use dom_struct::dom_struct;
use js::jsapi::JSObject;
use js::rust::HandleObject;
use js::typedarray::Uint8Array;

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::TextEncoderBinding::TextEncoderMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;

#[dom_struct]
pub struct TextEncoder {
    reflector_: Reflector,
}

impl TextEncoder {
    fn new_inherited() -> TextEncoder {
        TextEncoder {
            reflector_: Reflector::new(),
        }
    }

    fn new(global: &GlobalScope, proto: Option<HandleObject>) -> DomRoot<TextEncoder> {
        reflect_dom_object_with_proto(Box::new(TextEncoder::new_inherited()), global, proto)
    }

    // https://encoding.spec.whatwg.org/#dom-textencoder
    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<TextEncoder>> {
        Ok(TextEncoder::new(global, proto))
    }
}

impl TextEncoderMethods for TextEncoder {
    // https://encoding.spec.whatwg.org/#dom-textencoder-encoding
    fn Encoding(&self) -> DOMString {
        DOMString::from("utf-8")
    }

    // https://encoding.spec.whatwg.org/#dom-textencoder-encode
    fn Encode(&self, cx: JSContext, input: USVString) -> Uint8Array {
        let encoded = input.0.as_bytes();

        rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
        create_buffer_source(cx, encoded, js_object.handle_mut())
            .expect("Converting input to uint8 array should never fail")
    }
}
