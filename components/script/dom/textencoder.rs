/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TextEncoderBinding;
use dom::bindings::codegen::Bindings::TextEncoderBinding::TextEncoderMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::USVString;
use encoding::label::encoding_from_whatwg_label;
use encoding::types::EncodingRef;
use encoding::{EncoderTrap, Encoding};
use js::jsapi::{JSContext, JSObject};
use js::jsapi::{JS_GetUint8ArrayData, JS_NewUint8Array};
use libc::uint8_t;
use std::borrow::ToOwned;
use std::ptr;
use util::str::DOMString;

#[dom_struct]
pub struct TextEncoder {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Defined in rust-encoding"]
    encoder: EncodingRef,
}

impl TextEncoder {
    fn new_inherited(encoder: EncodingRef) -> TextEncoder {
        TextEncoder {
            reflector_: Reflector::new(),
            encoder: encoder,
        }
    }

    pub fn new(global: GlobalRef, encoder: EncodingRef) -> Root<TextEncoder> {
        reflect_dom_object(box TextEncoder::new_inherited(encoder),
                           global,
                           TextEncoderBinding::Wrap)
    }

    // https://encoding.spec.whatwg.org/#dom-textencoder
    pub fn Constructor(global: GlobalRef,
                       label: DOMString) -> Fallible<Root<TextEncoder>> {
        let encoding = match encoding_from_whatwg_label(&label) {
            Some(enc) => enc,
            None => {
                debug!("Encoding Label Not Supported");
                return Err(Error::Range("The given encoding is not supported.".to_owned()))
            }
        };

        match encoding.name() {
            "utf-8" | "utf-16be" | "utf-16le" => {
                Ok(TextEncoder::new(global, encoding))
            }
            _ => {
                debug!("Encoding Not UTF");
                Err(Error::Range("The encoding must be utf-8, utf-16le, or utf-16be.".to_owned()))
            }
        }
    }
}

impl TextEncoderMethods for TextEncoder {
    // https://encoding.spec.whatwg.org/#dom-textencoder-encoding
    fn Encoding(&self) -> DOMString {
        DOMString::from(self.encoder.name())
    }

    #[allow(unsafe_code)]
    // https://encoding.spec.whatwg.org/#dom-textencoder-encode
    fn Encode(&self, cx: *mut JSContext, input: USVString) -> *mut JSObject {
        unsafe {
            let encoded = self.encoder.encode(&input.0, EncoderTrap::Strict).unwrap();
            let length = encoded.len() as u32;
            let js_object: *mut JSObject = JS_NewUint8Array(cx, length);

            let js_object_data: *mut uint8_t = JS_GetUint8ArrayData(js_object, &mut false, ptr::null());
            ptr::copy_nonoverlapping(encoded.as_ptr(), js_object_data, length as usize);
            js_object
        }
    }
}
