/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};

use encoding_rs::{Decoder, DecoderResult, Encoding};
use script_bindings::str::DOMString;

use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use crate::dom::bindings::error::{Error, Fallible};

/// The shared part of `TextDecoder` and `TextDecoderStream`
///
/// Note that this does NOT correspond to the `TextDecoderCommon`
/// interface defined in the WebIDL
#[allow(non_snake_case)]
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct TextDecoderCommon {
    #[no_trace]
    encoding: &'static Encoding,
    fatal: bool,
    ignoreBOM: bool,
    #[ignore_malloc_size_of = "defined in encoding_rs"]
    #[no_trace]
    decoder: RefCell<Decoder>,
    in_stream: RefCell<Vec<u8>>,
    do_not_flush: Cell<bool>,
}

#[allow(non_snake_case)]
impl TextDecoderCommon {
    pub(crate) fn new_inherited(
        encoding: &'static Encoding,
        fatal: bool,
        ignoreBOM: bool,
    ) -> TextDecoderCommon {
        let decoder = if ignoreBOM {
            encoding.new_decoder()
        } else {
            encoding.new_decoder_without_bom_handling()
        };

        TextDecoderCommon {
            encoding,
            fatal,
            ignoreBOM,
            decoder: RefCell::new(decoder),
            in_stream: RefCell::new(Vec::new()),
            do_not_flush: Cell::new(false),
        }
    }

    pub(crate) fn encoding(&self) -> DOMString {
        DOMString::from(self.encoding.name().to_ascii_lowercase())
    }

    pub(crate) fn fatal(&self) -> bool {
        self.fatal
    }

    pub(crate) fn ignore_bom(&self) -> bool {
        self.ignoreBOM
    }

    #[allow(unsafe_code)]
    pub(crate) fn decode(
        &self,
        input: Option<&ArrayBufferViewOrArrayBuffer>,
        do_not_flush: bool,
    ) -> Fallible<String> {
        // Step 1. If this’s do not flush is false, then set this’s decoder to a
        // new instance of this’s encoding’s decoder, this’s I/O queue to the
        // I/O queue of bytes « end-of-queue », and this’s BOM seen to false.
        if !self.do_not_flush.get() {
            if self.ignoreBOM {
                self.decoder
                    .replace(self.encoding.new_decoder_without_bom_handling());
            } else {
                self.decoder.replace(self.encoding.new_decoder());
            }
            self.in_stream.replace(Vec::new());
        }

        // Step 2. Set this’s do not flush to options["stream"].
        self.do_not_flush.set(do_not_flush);

        // Step 3. If input is given, then push a copy of input to this’s I/O queue.
        match input {
            Some(ArrayBufferViewOrArrayBuffer::ArrayBufferView(a)) => {
                self.in_stream
                    .borrow_mut()
                    .extend_from_slice(unsafe { a.as_slice() });
            },
            Some(ArrayBufferViewOrArrayBuffer::ArrayBuffer(a)) => {
                self.in_stream
                    .borrow_mut()
                    .extend_from_slice(unsafe { a.as_slice() });
            },
            None => {},
        };

        let mut decoder = self.decoder.borrow_mut();
        let (remaining, s) = {
            let mut in_stream = self.in_stream.borrow_mut();

            let (remaining, s) = if self.fatal {
                // Step 4. Let output be the I/O queue of scalar values « end-of-queue ».
                let mut out_stream = String::with_capacity(
                    decoder
                        .max_utf8_buffer_length_without_replacement(in_stream.len())
                        .unwrap(),
                );
                // Step 5: Implemented by encoding_rs::Decoder.
                match decoder.decode_to_string_without_replacement(
                    &in_stream,
                    &mut out_stream,
                    !do_not_flush,
                ) {
                    (DecoderResult::InputEmpty, read) => (in_stream.split_off(read), out_stream),
                    // Step 5.3.3.
                    _ => return Err(Error::Type("Decoding failed".to_owned())),
                }
            } else {
                // Step 4. Let output be the I/O queue of scalar values « end-of-queue ».
                let mut out_stream =
                    String::with_capacity(decoder.max_utf8_buffer_length(in_stream.len()).unwrap());
                // Step 5: Implemented by encoding_rs::Decoder.
                let (_result, read, _replaced) =
                    decoder.decode_to_string(&in_stream, &mut out_stream, !do_not_flush);
                (in_stream.split_off(read), out_stream)
            };
            (remaining, s)
        };
        self.in_stream.replace(remaining);
        Ok(s)
    }
}
