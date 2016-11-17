/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::codegen::Bindings::TextEncoderBinding;
use dom::bindings::codegen::Bindings::TextEncoderBinding::TextEncoderMethods;
use dom::bindings::error::Fallible;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::globalscope::GlobalScope;
use encoding::EncoderTrap;
use encoding::Encoding;
use encoding::all::UTF_8;
use js::jsapi::{JSContext, JSObject};
use js::jsapi::{JS_GetUint8ArrayData, JS_NewUint8Array};
use libc::uint8_t;
use std::ptr;

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

    pub fn new(global: &GlobalScope) -> Root<TextEncoder> {
        reflect_dom_object(box TextEncoder::new_inherited(),
                           global,
                           TextEncoderBinding::Wrap)
    }

    // https://encoding.spec.whatwg.org/#dom-textencoder
    pub fn Constructor(global: &GlobalScope) -> Fallible<Root<TextEncoder>> {
        Ok(TextEncoder::new(global))
    }
}

impl TextEncoderMethods for TextEncoder {
    // https://encoding.spec.whatwg.org/#dom-textencoder-encoding
    fn Encoding(&self) -> DOMString {
        DOMString::from(UTF_8.name())
    }

    #[allow(unsafe_code)]
    // https://encoding.spec.whatwg.org/#dom-textencoder-encode
    unsafe fn Encode(&self, cx: *mut JSContext, input: USVString) -> NonZero<*mut JSObject> {
        let encoded = UTF_8.encode(&input.0, EncoderTrap::Strict).unwrap();
        let length = encoded.len() as u32;
        rooted!(in(cx) let js_object = JS_NewUint8Array(cx, length));
        assert!(!js_object.is_null());
        let mut is_shared = false;
        let js_object_data: *mut uint8_t = JS_GetUint8ArrayData(js_object.get(), &mut is_shared, ptr::null());
        assert!(!is_shared);
        ptr::copy_nonoverlapping(encoded.as_ptr(), js_object_data, length as usize);
        NonZero::new(js_object.get())
    }
}
