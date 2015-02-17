/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TextEncoderBinding;
use dom::bindings::codegen::Bindings::TextEncoderBinding::TextEncoderMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::error::Fallible;
use dom::bindings::error::Error::IndexSize;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::str::USVString;
use dom::bindings::utils::{Reflector, reflect_dom_object};

use util::str::DOMString;

use std::borrow::ToOwned;
use std::ascii::AsciiExt;
use std::ptr;

use encoding::types::EncodingRef;
use encoding::{Encoding, EncoderTrap};
use encoding::label::encoding_from_whatwg_label;

use libc::uint8_t;
use js::jsapi::{JSContext, JSObject};
use js::jsfriendapi::bindgen::{JS_NewUint8Array, JS_GetUint8ArrayData};

#[dom_struct]
pub struct TextEncoder {
    reflector_: Reflector,
    encoding: DOMString,
    encoder: EncodingRef,
}

impl TextEncoder {
    fn new_inherited(encoding: DOMString, encoder: EncodingRef) -> TextEncoder {
        TextEncoder {
            reflector_: Reflector::new(),
            encoding: encoding,
            encoder: encoder,
        }
    }

    pub fn new(global: GlobalRef, encoding: DOMString, encoder: EncodingRef) -> Temporary<TextEncoder> {
        reflect_dom_object(box TextEncoder::new_inherited(encoding, encoder),
                           global,
                           TextEncoderBinding::Wrap)
    }

    // https://encoding.spec.whatwg.org/#dom-textencoder
    pub fn Constructor(global: GlobalRef,
                       label: DOMString) -> Fallible<Temporary<TextEncoder>> {
        let encoding = match encoding_from_whatwg_label(label.trim().as_slice().to_ascii_lowercase().as_slice()) {
            Some(enc) => enc,
            None => {
                debug!("Encoding Label Not Supported");
                // TODO: should throw RangeError
                return Err(IndexSize)
            }
        };

        match encoding.name() {
            "utf-8" | "utf-16be" | "utf-16le" => {
                Ok(TextEncoder::new(global, encoding.name().to_owned(), encoding))
            }
            _ => {
                debug!("Encoding Not UTF");
                // TODO: should throw RangeError
                Err(IndexSize)
            }
        }
    }
}

impl<'a> TextEncoderMethods for JSRef<'a, TextEncoder> {
    // https://encoding.spec.whatwg.org/#dom-textencoder-encoding
    fn Encoding(self) -> DOMString {
        self.encoding.clone()
    }

    // https://encoding.spec.whatwg.org/#dom-textencoder-encode
    #[allow(unsafe_code)]
    fn Encode(self, cx: *mut JSContext, input: USVString) -> *mut JSObject {
        unsafe {
            let output = self.encoder.encode(input.0.as_slice(), EncoderTrap::Strict).unwrap();
            let length = output.len() as u32;
            let js_object: *mut JSObject = JS_NewUint8Array(cx, length);

            let js_object_data: *mut uint8_t = JS_GetUint8ArrayData(js_object, cx);
            ptr::copy_nonoverlapping(js_object_data, output.as_ptr(), length as usize);
            return js_object;
        }
    }
}
