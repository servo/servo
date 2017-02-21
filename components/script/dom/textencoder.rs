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
use dom_struct::dom_struct;
use encoding::EncoderTrap;
use encoding::Encoding;
use encoding::all::UTF_8;
use js::jsapi::{JSContext, JSObject};
use js::typedarray::{Uint8Array, CreateWith};
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

        rooted!(in(cx) let mut js_object = ptr::null_mut());
        assert!(Uint8Array::create(cx, CreateWith::Slice(&encoded), js_object.handle_mut()).is_ok());

        NonZero::new(js_object.get())
    }
}
