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
use encoding_rs::{Decoder, DecoderResult, Encoding};
use std::borrow::ToOwned;
use std::cell::{Cell, RefCell};

#[dom_struct]
pub struct TextDecoder {
    reflector_: Reflector,
    encoding: &'static Encoding,
    fatal: bool,
    ignoreBOM: bool,
    #[ignore_malloc_size_of = "defined in encoding_rs"]
    decoder: RefCell<Decoder>,
    in_stream: RefCell<Vec<u8>>,
    do_not_flush: Cell<bool>,
}

impl TextDecoder {
    fn new_inherited(encoding: &'static Encoding, fatal: bool, ignoreBOM: bool) -> TextDecoder {
        TextDecoder {
            reflector_: Reflector::new(),
            encoding: encoding,
            fatal: fatal,
            ignoreBOM: ignoreBOM,
            decoder: RefCell::new(
                if ignoreBOM { encoding.new_decoder() } else { encoding.new_decoder_without_bom_handling() }
            ),
            in_stream: RefCell::new(Vec::new()),
            do_not_flush: Cell::new(false),
        }
    }

    fn make_range_error() -> Fallible<DomRoot<TextDecoder>> {
        Err(Error::Range("The given encoding is not supported.".to_owned()))
    }

    pub fn new(global: &GlobalScope, encoding: &'static Encoding, fatal: bool, ignoreBOM: bool)
            -> DomRoot<TextDecoder> {
        reflect_dom_object(Box::new(TextDecoder::new_inherited(encoding, fatal, ignoreBOM)),
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
        Ok(TextDecoder::new(global, encoding, options.fatal, options.ignoreBOM))
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

    // https://encoding.spec.whatwg.org/#dom-textdecoder-ignorebom
    fn IgnoreBOM(&self) -> bool {
        self.ignoreBOM
    }

    // https://encoding.spec.whatwg.org/#dom-textdecoder-decode
    fn Decode(
        &self,
        input: Option<ArrayBufferViewOrArrayBuffer>,
        options: &TextDecodeOptions
    ) -> Fallible<USVString> {
        // Step 1.
        if !self.do_not_flush.get() {
            if self.ignoreBOM {
                self.decoder.replace(self.encoding.new_decoder_without_bom_handling());
            } else {
                self.decoder.replace(self.encoding.new_decoder());
            }
            self.in_stream.replace(Vec::new());
        }

        // Step 2.
        self.do_not_flush.set(options.stream);

        // Step 3.
        match input {
            Some(ArrayBufferViewOrArrayBuffer::ArrayBufferView(ref a)) => {
                self.in_stream.borrow_mut().extend_from_slice(&a.to_vec());
            },
            Some(ArrayBufferViewOrArrayBuffer::ArrayBuffer(ref a)) => {
                self.in_stream.borrow_mut().extend_from_slice(&a.to_vec());
            },
            None => {},
        };

        let mut decoder = self.decoder.borrow_mut();
        let (remaining, s) = {
            let mut in_stream = self.in_stream.borrow_mut();

            let (remaining, s) = if self.fatal {
                // Step 4.
                let mut out_stream = String::with_capacity(
                    decoder.max_utf8_buffer_length_without_replacement(in_stream.len()).unwrap()
                );
                // Step 5: Implemented by encoding_rs::Decoder.
                match decoder.decode_to_string_without_replacement(&in_stream, &mut out_stream, !options.stream) {
                    (DecoderResult::InputEmpty, read) => {
                        (in_stream.split_off(read), out_stream)
                    },
                    // Step 5.3.3.
                    _ => return Err(Error::Type("Decoding failed".to_owned())),
                }
            } else {
                // Step 4.
                let mut out_stream = String::with_capacity(decoder.max_utf8_buffer_length(in_stream.len()).unwrap());
                // Step 5: Implemented by encoding_rs::Decoder.
                let (_result, read, _replaced) = decoder.decode_to_string(&in_stream, &mut out_stream, !options.stream);
                (in_stream.split_off(read), out_stream)
            };
            (remaining, s)
        };
        self.in_stream.replace(remaining);
        Ok(USVString(s))
    }
}
