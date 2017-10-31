/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TextDecoderBinding;
use dom::bindings::codegen::Bindings::TextDecoderBinding::TextDecoderMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::{DOMString, USVString};
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use encoding_rs::Encoding;
use js::jsapi::{JSContext, JSObject};
use std::ascii::AsciiExt;
use std::borrow::ToOwned;

#[dom_struct]
pub struct TextDecoder {
    reflector_: Reflector,
    encoding: &'static Encoding,
    fatal: bool,
}

impl TextDecoder {
    fn new_inherited(encoding: &'static Encoding, fatal: bool) -> TextDecoder {
        TextDecoder {
            reflector_: Reflector::new(),
            encoding: encoding,
            fatal: fatal,
        }
    }

    fn make_range_error() -> Fallible<DomRoot<TextDecoder>> {
        Err(Error::Range("The given encoding is not supported.".to_owned()))
    }

    pub fn new(global: &GlobalScope, encoding: &'static Encoding, fatal: bool) -> DomRoot<TextDecoder> {
        reflect_dom_object(Box::new(TextDecoder::new_inherited(encoding, fatal)),
                           global,
                           TextDecoderBinding::Wrap)
    }

    /// <https://encoding.spec.whatwg.org/#dom-textdecoder>
    pub fn Constructor(global: &GlobalScope,
                       label: DOMString,
                       options: &TextDecoderBinding::TextDecoderOptions)
                            -> Fallible<DomRoot<TextDecoder>> {
        let encoding = match Encoding::for_label_no_replacement(label.as_bytes()) {
            None => return TextDecoder::make_range_error(),
            Some(enc) => enc
        };
        Ok(TextDecoder::new(global, encoding, options.fatal))
    }
}


impl TextDecoderMethods for TextDecoder {
    // https://encoding.spec.whatwg.org/#dom-textdecoder-encoding
    fn Encoding(&self) -> DOMString {
        DOMString::from(self.encoding.name().to_ascii_lowercase())
    }

    // https://encoding.spec.whatwg.org/#dom-textdecoder-fatal
    fn Fatal(&self) -> bool {
        self.fatal
    }

    #[allow(unsafe_code)]
    // https://encoding.spec.whatwg.org/#dom-textdecoder-decode
    unsafe fn Decode(&self, _cx: *mut JSContext, input: Option<*mut JSObject>)
              -> Fallible<USVString> {
        let input = match input {
            Some(input) => input,
            None => return Ok(USVString("".to_owned())),
        };

        typedarray!(in(_cx) let data_res: ArrayBufferView = input);
        let mut data = match data_res {
            Ok(data) => data,
            Err(_)   => {
                return Err(Error::Type("Argument to TextDecoder.decode is not an ArrayBufferView".to_owned()));
            }
        };

        let s = if self.fatal {
            match self.encoding.decode_without_bom_handling_and_without_replacement(data.as_slice()) {
                Some(s) => s,
                None => return Err(Error::Type("Decoding failed".to_owned())),
            }
        } else {
            let (s, _has_errors) = self.encoding.decode_without_bom_handling(data.as_slice());
            s
        };
        Ok(USVString(s.into_owned()))
    }
}
