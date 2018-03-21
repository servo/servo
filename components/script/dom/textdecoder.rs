/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TextDecoderBinding;
use dom::bindings::codegen::Bindings::TextDecoderBinding::{TextDecoderMethods, TextDecodeOptions};
use dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::{DOMString, USVString};
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use encoding_rs::Encoding;
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

    // https://encoding.spec.whatwg.org/#dom-textdecoder-decode
    fn Decode(
        &self,
        input: Option<ArrayBufferViewOrArrayBuffer>,
        _options: &TextDecodeOptions
    ) -> Fallible<USVString> {
        match input {
            Some(arr) => {
                let vec: Vec<u8> = match arr {
                    ArrayBufferViewOrArrayBuffer::ArrayBufferView(ref a) => a.to_vec(),
                    ArrayBufferViewOrArrayBuffer::ArrayBuffer(ref a) => a.to_vec()
                };
                let s = if self.fatal {
                    match self.encoding.decode_without_bom_handling_and_without_replacement(&vec) {
                        Some(s) => s,
                        None => return Err(Error::Type("Decoding failed".to_owned())),
                    }
                } else {
                    let (s, _has_errors) = self.encoding.decode_without_bom_handling(&vec);
                    s
                };
                Ok(USVString(s.into_owned()))
            }
            None => Ok(USVString("".to_owned()))
        }
    }
}
