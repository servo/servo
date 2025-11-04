/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use encoding_rs::{Decoder, DecoderResult, Encoding};

use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use crate::dom::bindings::error::{Error, Fallible};

/// The shared part of `TextDecoder` and `TextDecoderStream`
///
/// Note that other than the three attributes defined in the `TextDecoderCommon`
/// interface in the WebIDL, this also performs decoding.
///
/// <https://encoding.spec.whatwg.org/#textdecodercommon>
#[expect(non_snake_case)]
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct TextDecoderCommon {
    /// <https://encoding.spec.whatwg.org/#dom-textdecoder-encoding>
    #[no_trace]
    encoding: &'static Encoding,

    /// <https://encoding.spec.whatwg.org/#dom-textdecoder-fatal>
    fatal: bool,

    /// <https://encoding.spec.whatwg.org/#dom-textdecoder-ignorebom>
    ignoreBOM: bool,

    /// The native decoder that is used to perform decoding
    ///
    /// <https://encoding.spec.whatwg.org/#textdecodercommon-decoder>
    #[ignore_malloc_size_of = "defined in encoding_rs"]
    #[no_trace]
    decoder: RefCell<Decoder>,

    /// <https://encoding.spec.whatwg.org/#textdecodercommon-i-o-queue>
    io_queue: RefCell<Vec<u8>>,
}

#[expect(non_snake_case)]
impl TextDecoderCommon {
    pub(crate) fn new_inherited(
        encoding: &'static Encoding,
        fatal: bool,
        ignoreBOM: bool,
    ) -> TextDecoderCommon {
        let decoder = if ignoreBOM {
            encoding.new_decoder_without_bom_handling()
        } else {
            encoding.new_decoder_with_bom_removal()
        };

        TextDecoderCommon {
            encoding,
            fatal,
            ignoreBOM,
            decoder: RefCell::new(decoder),
            io_queue: RefCell::new(Vec::new()),
        }
    }

    /// <https://encoding.spec.whatwg.org/#textdecoder-encoding>
    pub(crate) fn encoding(&self) -> &'static Encoding {
        self.encoding
    }

    /// <https://encoding.spec.whatwg.org/#textdecodercommon-decoder>
    pub(crate) fn decoder(&self) -> &RefCell<Decoder> {
        &self.decoder
    }

    /// <https://encoding.spec.whatwg.org/#textdecodercommon-i-o-queue>
    pub(crate) fn io_queue(&self) -> &RefCell<Vec<u8>> {
        &self.io_queue
    }

    /// <https://encoding.spec.whatwg.org/#textdecoder-error-mode>
    pub(crate) fn fatal(&self) -> bool {
        self.fatal
    }

    /// <https://encoding.spec.whatwg.org/#textdecoder-ignore-bom-flag>
    pub(crate) fn ignore_bom(&self) -> bool {
        self.ignoreBOM
    }

    /// Shared by `TextDecoder` and `TextDecoderStream`
    ///
    /// <https://encoding.spec.whatwg.org/#dom-textdecoder-decode>
    /// <https://encoding.spec.whatwg.org/#decode-and-enqueue-a-chunk>
    #[allow(unsafe_code)]
    pub(crate) fn decode(
        &self,
        input: Option<&ArrayBufferViewOrArrayBuffer>,
        last: bool,
    ) -> Fallible<String> {
        // <https://encoding.spec.whatwg.org/#dom-textdecoder-decode>
        // Step 3. If input is given, then push a copy of input to this’s I/O queue.
        //
        // <https://encoding.spec.whatwg.org/#decode-and-enqueue-a-chunk>
        // Step 2. Push a copy of bufferSource to decoder’s I/O queue.
        //
        // NOTE: try to avoid this copy unless there are bytes left
        let mut io_queue = self.io_queue.borrow_mut();
        let input = match &input {
            Some(ArrayBufferViewOrArrayBuffer::ArrayBufferView(a)) => unsafe {
                if io_queue.is_empty() {
                    a.as_slice()
                } else {
                    io_queue.extend_from_slice(a.as_slice());
                    &io_queue[..]
                }
            },
            Some(ArrayBufferViewOrArrayBuffer::ArrayBuffer(a)) => unsafe {
                if io_queue.is_empty() {
                    a.as_slice()
                } else {
                    io_queue.extend_from_slice(a.as_slice());
                    &io_queue[..]
                }
            },
            None => &io_queue[..],
        };

        let mut decoder = self.decoder.borrow_mut();
        let (output, read) = if self.fatal {
            // <https://encoding.spec.whatwg.org/#dom-textdecoder-decode>
            // Step 4. Let output be the I/O queue of scalar values « end-of-queue ».
            //
            // <https://encoding.spec.whatwg.org/#decode-and-enqueue-a-chunk>
            // Step 3. Let output be the I/O queue of scalar values « end-of-queue ».
            let mut output = String::with_capacity(
                decoder
                    .max_utf8_buffer_length_without_replacement(input.len())
                    .ok_or_else(|| {
                        Error::Type("Expected UTF8 buffer length would overflow".to_owned())
                    })?,
            );

            // Note: The two algorithms below are implemented in
            // `encoding_rs::Decoder::decode_to_string_without_replacement`
            //
            // <https://encoding.spec.whatwg.org/#dom-textdecoder-decode>
            // Step 5. While true:
            // Step 5.1 Let item be the result of reading from this’s I/O queue.
            // Step 5.2 If item is end-of-queue and this’s do not flush is true,
            //      then return the result of running serialize I/O queue with this and output.
            // Step 5.3 Otherwise:
            // Step 5.3.1 Let result be the result of processing an item with item, this’s decoder,
            //      this’s I/O queue, output, and this’s error mode.
            //
            // <https://encoding.spec.whatwg.org/#decode-and-enqueue-a-chunk>
            // Step 4. While true:
            // Step 4.1 Let item be the result of reading from decoder’s I/O queue.
            // Step 4.2 If item is end-of-queue:
            // Step 4.2.1 Let outputChunk be the result of running serialize I/O queue with decoder and output.
            // Step 4.2.2 If outputChunk is not the empty string, then enqueue outputChunk in decoder’s transform.
            // Step 4.2.3 Return.
            // Step 4.3 Let result be the result of processing an item with item, decoder’s decoder,
            //      decoder’s I/O queue, output, and decoder’s error mode.
            // Step 4.4 If result is error, then throw a TypeError.
            let (result, read) =
                decoder.decode_to_string_without_replacement(input, &mut output, last);
            match result {
                // <https://encoding.spec.whatwg.org/#dom-textdecoder-decode>
                // Step 5.3.2 If result is finished, then return the result of running serialize I/O
                //      queue with this and output.
                DecoderResult::InputEmpty => (output, read),
                // <https://encoding.spec.whatwg.org/#dom-textdecoder-decode>
                // Step 5.3.3 Otherwise, if result is error, throw a TypeError.
                DecoderResult::Malformed(_, _) => {
                    return Err(Error::Type("Decoding failed".to_owned()));
                },
                DecoderResult::OutputFull => {
                    unreachable!("output is allocated with sufficient capacity")
                },
            }
        } else {
            // <https://encoding.spec.whatwg.org/#dom-textdecoder-decode>
            // Step 4. Let output be the I/O queue of scalar values « end-of-queue ».
            let mut output =
                String::with_capacity(decoder.max_utf8_buffer_length(input.len()).ok_or_else(
                    || Error::Type("Expected UTF8 buffer length would overflow".to_owned()),
                )?);

            // Note: The two algorithms below are implemented in
            // `encoding_rs::Decoder::decode_to_string`
            //
            // <https://encoding.spec.whatwg.org/#dom-textdecoder-decode>
            // Step 5. While true:
            // Step 5.1 Let item be the result of reading from this’s I/O queue.
            // Step 5.2 If item is end-of-queue and this’s do not flush is true,
            //      then return the result of running serialize I/O queue with this and output.
            // Step 5.3 Otherwise:
            // Step 5.3.1 Let result be the result of processing an item with item, this’s decoder,
            //      this’s I/O queue, output, and this’s error mode.
            //
            // <https://encoding.spec.whatwg.org/#decode-and-enqueue-a-chunk>
            // Step 4. While true:
            // Step 4.1 Let item be the result of reading from decoder’s I/O queue.
            // Step 4.2 If item is end-of-queue:
            // Step 4.2.1 Let outputChunk be the result of running serialize I/O queue with decoder and output.
            // Step 4.2.2 If outputChunk is not the empty string, then enqueue outputChunk in decoder’s transform.
            // Step 4.2.3 Return.
            // Step 4.3 Let result be the result of processing an item with item, decoder’s decoder,
            //      decoder’s I/O queue, output, and decoder’s error mode.
            // Step 4.4 If result is error, then throw a TypeError.
            let (result, read, _replaced) = decoder.decode_to_string(input, &mut output, last);
            match result {
                // <https://encoding.spec.whatwg.org/#dom-textdecoder-decode>
                // Step 5.3.2 If result is finished, then return the result of running serialize I/O
                //      queue with this and output.
                encoding_rs::CoderResult::InputEmpty => (output, read),
                encoding_rs::CoderResult::OutputFull => {
                    unreachable!("output is allocated with sufficient capacity")
                },
            }
        };

        let (_consumed, remaining) = input.split_at(read);
        *io_queue = remaining.to_vec();

        Ok(output)
    }
}
