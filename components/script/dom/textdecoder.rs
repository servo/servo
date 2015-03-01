/* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TextDecoderBinding;
use dom::bindings::codegen::Bindings::TextDecoderBinding::TextDecoderMethods;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::trace::JSTraceable;
use dom::bindings::global::GlobalRef;
use dom::bindings::error::Fallible;
use dom::bindings::error::Error::Syntax;
use dom::bindings::utils::{Reflector, reflect_dom_object};

use encoding::types::EncodingRef;
use encoding::{Encoding, DecoderTrap};
use encoding::label::encoding_from_whatwg_label;

use js::jsfriendapi::bindgen::{JS_GetUint8ArrayData, JS_GetArrayBufferByteLength};
use js::jsapi::JSTracer;

use util::str::DOMString;

use std::borrow::ToOwned;
use std::slice::from_raw_parts;

#[dom_struct]
pub struct TextDecoder {
    reflector_: Reflector,
    encoding: EncodingRef,
    fatal: bool
}

no_jsmanaged_fields!(EncodingRef);

impl TextDecoder {
    fn new_inherited(encoding: EncodingRef, fatal: bool) -> TextDecoder {
        TextDecoder {
            reflector_: Reflector::new(),
            encoding: encoding,
            fatal: fatal
        }
    }

    pub fn new(global: GlobalRef, encoding: EncodingRef, fatal: bool) -> Temporary<TextDecoder> {
        reflect_dom_object(box TextDecoder::new_inherited(encoding, fatal),
                           global,
                           TextDecoderBinding::Wrap)
    }

    // Spec: https://encoding.spec.whatwg.org/#dom-textdecoder
    pub fn Constructor(global: GlobalRef,
                       label: DOMString,
                       options: &TextDecoderBinding::TextDecoderOptions)
                            -> Fallible<Temporary<TextDecoder>> {
        let encoding = match encoding_from_whatwg_label(label.as_slice()) {
            Some(enc) => enc,
            None      => return Err(Syntax) // FIXME: Should throw a RangeError as per spec
        };
        Ok(TextDecoder::new(global, encoding, options.fatal))
    }
}

impl<'a> TextDecoderMethods for JSRef<'a, TextDecoder> {
    pub fn Decode(self, cx: *mut JSContext, input: *mut JSObject) -> Fallible<DOMString> {
        let length: usize = JS_GetArrayBufferByteLength(input, cx) as usize;
        let stream: *const uint8_t = JS_GetUint8ArrayData(input, cx) as *const uint8_t;
        let trap = if self.fatal { DecoderTrap::Strict } else { DecoderTrap::Replace };
        unsafe { self.encoding.decode(from_raw_parts(stream, length), trap) }
    }

    fn Encoding(self) -> DOMString {
        match self.encoding.whatwg_name() {
            Some(enc) => enc.to_owned(),
            None      => "Unknown"
        }
    }

    fn Fatal(self) -> bool {
        self.fatal
    }

    fn IgnoreBOM(self) -> bool {
        false
    }
}
