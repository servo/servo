/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TextDecoderBinding;
use dom::bindings::codegen::Bindings::TextDecoderBinding::TextDecoderMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::str::USVString;
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{Reflector, reflect_dom_object};

use util::str::DOMString;

use encoding::Encoding;
use encoding::types::{EncodingRef, DecoderTrap};
use encoding::label::encoding_from_whatwg_label;
use js::jsapi::{JSContext, JSObject};
use js::jsapi::JS_GetObjectAsArrayBufferView;

use std::borrow::ToOwned;
use std::ptr;
use std::slice;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct TextDecoder {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Defined in rust-encoding"]
    encoding: EncodingRef,
    fatal: bool,
}

impl TextDecoder {
    fn new_inherited(encoding: EncodingRef, fatal: bool) -> TextDecoder {
        TextDecoder {
            reflector_: Reflector::new(),
            encoding: encoding,
            fatal: fatal,
        }
    }

    fn make_range_error() -> Fallible<Root<TextDecoder>> {
        Err(Error::Range("The given encoding is not supported.".to_owned()))
    }

    pub fn new(global: GlobalRef, encoding: EncodingRef, fatal: bool) -> Root<TextDecoder> {
        reflect_dom_object(box TextDecoder::new_inherited(encoding, fatal),
                           global,
                           TextDecoderBinding::Wrap)
    }

    /// https://encoding.spec.whatwg.org/#dom-textdecoder
    pub fn Constructor(global: GlobalRef,
                       label: DOMString,
                       options: &TextDecoderBinding::TextDecoderOptions)
                            -> Fallible<Root<TextDecoder>> {
        let encoding = match encoding_from_whatwg_label(&label) {
            None => return TextDecoder::make_range_error(),
            Some(enc) => enc
        };
        // The rust-encoding crate has WHATWG compatibility, so we are
        // guaranteed to have a whatwg_name because we successfully got
        // the encoding from encoding_from_whatwg_label.
        // Use match + panic! instead of unwrap for better error message
        match encoding.whatwg_name() {
            None => panic!("Label {} fits valid encoding without valid name", label),
            Some("replacement") => return TextDecoder::make_range_error(),
            _ => ()
        };
        Ok(TextDecoder::new(global, encoding, options.fatal))
    }
}


impl<'a> TextDecoderMethods for &'a TextDecoder {
    // https://encoding.spec.whatwg.org/#dom-textdecoder-encoding
    fn Encoding(self) -> DOMString {
        self.encoding.whatwg_name().unwrap().to_owned()
    }

    // https://encoding.spec.whatwg.org/#dom-textdecoder-fatal
    fn Fatal(self) -> bool {
        self.fatal
    }

    #[allow(unsafe_code)]
    // https://encoding.spec.whatwg.org/#dom-textdecoder-decode
    fn Decode(self, _cx: *mut JSContext, input: Option<*mut JSObject>)
              -> Fallible<USVString> {
        let input = match input {
            Some(input) => input,
            None => return Ok(USVString("".to_owned())),
        };

        let mut length = 0;
        let mut data = ptr::null_mut();
        if unsafe { JS_GetObjectAsArrayBufferView(input, &mut length, &mut data).is_null() } {
            return Err(Error::Type("Argument to TextDecoder.decode is not an ArrayBufferView".to_owned()));
        }

        let buffer = unsafe {
            slice::from_raw_parts(data as *const _, length as usize)
        };
        let trap = if self.fatal {
            DecoderTrap::Strict
        } else {
            DecoderTrap::Replace
        };
        match self.encoding.decode(buffer, trap) {
            Ok(s) => Ok(USVString(s)),
            Err(_) => Err(Error::Type("Decoding failed".to_owned())),
        }
    }
}
