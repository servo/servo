/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::io::{self, Write};
use std::ptr;

use brotli::DecompressorWriter as BrotliDecoder;
use dom_struct::dom_struct;
use flate2::write::{DeflateDecoder, GzDecoder, ZlibDecoder};
use js::jsapi::JSObject;
use js::jsval::UndefinedValue;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};
use js::typedarray::Uint8;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::CompressionStreamBinding::CompressionFormat;
use crate::dom::bindings::codegen::Bindings::DecompressionStreamBinding::DecompressionStreamMethods;
use crate::dom::bindings::conversions::SafeToJSValConvertible;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::compressionstream::convert_chunk_to_vec;
use crate::dom::transformstreamdefaultcontroller::TransformerType;
use crate::dom::types::{
    GlobalScope, ReadableStream, TransformStream, TransformStreamDefaultController, WritableStream,
};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// A wrapper to blend ZlibDecoder<Vec<u8>>, DeflateDecoder<Vec<u8>> and GzDecoder<Vec<u8>>
/// together as a single type.
enum Decompressor {
    Deflate(ZlibDecoder<Vec<u8>>),
    DeflateRaw(DeflateDecoder<Vec<u8>>),
    Gzip(GzDecoder<Vec<u8>>),
    Brotli(Box<BrotliDecoder<Vec<u8>>>),
}

/// Expose methods of the inner decoder.
impl Decompressor {
    fn new(format: CompressionFormat) -> Decompressor {
        match format {
            CompressionFormat::Deflate => Decompressor::Deflate(ZlibDecoder::new(Vec::new())),
            CompressionFormat::Deflate_raw => {
                Decompressor::DeflateRaw(DeflateDecoder::new(Vec::new()))
            },
            CompressionFormat::Gzip => Decompressor::Gzip(GzDecoder::new(Vec::new())),
            CompressionFormat::Brotli => {
                Decompressor::Brotli(Box::new(BrotliDecoder::new(Vec::new(), 4096)))
            },
        }
    }

    fn get_ref(&self) -> &Vec<u8> {
        match self {
            Decompressor::Deflate(zlib_decoder) => zlib_decoder.get_ref(),
            Decompressor::DeflateRaw(deflate_decoder) => deflate_decoder.get_ref(),
            Decompressor::Gzip(gz_decoder) => gz_decoder.get_ref(),
            Decompressor::Brotli(brotli_decoder) => brotli_decoder.get_ref(),
        }
    }

    fn get_mut(&mut self) -> &mut Vec<u8> {
        match self {
            Decompressor::Deflate(zlib_decoder) => zlib_decoder.get_mut(),
            Decompressor::DeflateRaw(deflate_decoder) => deflate_decoder.get_mut(),
            Decompressor::Gzip(gz_decoder) => gz_decoder.get_mut(),
            Decompressor::Brotli(brotli_decoder) => brotli_decoder.get_mut(),
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        match self {
            Decompressor::Deflate(zlib_decoder) => zlib_decoder.write(buf),
            Decompressor::DeflateRaw(deflate_decoder) => deflate_decoder.write(buf),
            Decompressor::Gzip(gz_decoder) => gz_decoder.write(buf),
            Decompressor::Brotli(brotli_decoder) => brotli_decoder.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Decompressor::Deflate(zlib_decoder) => zlib_decoder.flush(),
            Decompressor::DeflateRaw(deflate_decoder) => deflate_decoder.flush(),
            Decompressor::Gzip(gz_decoder) => gz_decoder.flush(),
            Decompressor::Brotli(brotli_decoder) => brotli_decoder.flush(),
        }
    }

    fn try_finish(&mut self) -> io::Result<()> {
        match self {
            Decompressor::Deflate(zlib_decoder) => zlib_decoder.try_finish(),
            Decompressor::DeflateRaw(deflate_decoder) => deflate_decoder.try_finish(),
            Decompressor::Gzip(gz_decoder) => gz_decoder.try_finish(),
            Decompressor::Brotli(brotli_decoder) => brotli_decoder.flush(),
        }
    }
}

impl MallocSizeOf for Decompressor {
    #[expect(unsafe_code)]
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match self {
            Decompressor::Deflate(zlib_decoder) => zlib_decoder.size_of(ops),
            Decompressor::DeflateRaw(deflate_decoder) => deflate_decoder.size_of(ops),
            Decompressor::Gzip(gz_decoder) => gz_decoder.size_of(ops),
            Decompressor::Brotli(brotli_decoder) => unsafe {
                ops.malloc_size_of(&**brotli_decoder)
            },
        }
    }
}

/// <https://compression.spec.whatwg.org/#decompressionstream>
#[dom_struct]
pub(crate) struct DecompressionStream {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#generictransformstream>
    transform: Dom<TransformStream>,

    /// <https://compression.spec.whatwg.org/#decompressionstream-format>
    format: CompressionFormat,

    // <https://compression.spec.whatwg.org/#decompressionstream-context>
    #[no_trace]
    context: RefCell<Decompressor>,
}

impl DecompressionStream {
    fn new_inherited(
        transform: &TransformStream,
        format: CompressionFormat,
    ) -> DecompressionStream {
        DecompressionStream {
            reflector_: Reflector::new(),
            transform: Dom::from_ref(transform),
            format,
            context: RefCell::new(Decompressor::new(format)),
        }
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        transform: &TransformStream,
        format: CompressionFormat,
        can_gc: CanGc,
    ) -> DomRoot<DecompressionStream> {
        reflect_dom_object_with_proto(
            Box::new(DecompressionStream::new_inherited(transform, format)),
            global,
            proto,
            can_gc,
        )
    }
}

impl DecompressionStreamMethods<crate::DomTypeHolder> for DecompressionStream {
    /// <https://compression.spec.whatwg.org/#dom-decompressionstream-decompressionstream>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        format: CompressionFormat,
    ) -> Fallible<DomRoot<DecompressionStream>> {
        // Step 1. If format is unsupported in DecompressionStream, then throw a TypeError.
        // NOTE: All of "deflate", "deflate-raw", "gzip" and "br" are supported.

        // Step 2. Set this’s format to format.
        // Step 5. Set this’s transform to a new TransformStream.
        let transform = TransformStream::new_with_proto(global, None, can_gc);
        let decompression_stream =
            DecompressionStream::new_with_proto(global, proto, &transform, format, can_gc);

        // Step 3. Let transformAlgorithm be an algorithm which takes a chunk argument and runs the
        // decompress and enqueue a chunk algorithm with this and chunk.
        // Step 4. Let flushAlgorithm be an algorithm which takes no argument and runs the
        // decompress flush and enqueue algorithm with this.
        let transformer_type = TransformerType::Decompressor(decompression_stream.clone());

        // Step 6. Set up this’s transform with transformAlgorithm set to transformAlgorithm and
        // flushAlgorithm set to flushAlgorithm.
        let cx = GlobalScope::get_cx();
        transform.set_up(cx, global, transformer_type, can_gc)?;

        Ok(decompression_stream)
    }

    /// <https://streams.spec.whatwg.org/#dom-generictransformstream-readable>
    fn Readable(&self) -> DomRoot<ReadableStream> {
        // The readable getter steps are to return this’s transform.[[readable]].
        self.transform.get_readable()
    }

    /// <https://streams.spec.whatwg.org/#dom-generictransformstream-writable>
    fn Writable(&self) -> DomRoot<WritableStream> {
        // The writable getter steps are to return this’s transform.[[writable]].
        self.transform.get_writable()
    }
}

/// <https://compression.spec.whatwg.org/#decompress-and-enqueue-a-chunk>
pub(crate) fn decompress_and_enqueue_a_chunk(
    cx: SafeJSContext,
    global: &GlobalScope,
    ds: &DecompressionStream,
    chunk: SafeHandleValue,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc,
) -> Fallible<()> {
    // Step 1. If chunk is not a BufferSource type, then throw a TypeError.
    let chunk = convert_chunk_to_vec(cx, chunk, can_gc)?;

    // Step 2. Let buffer be the result of decompressing chunk with ds’s format and context. If
    // this results in an error, then throw a TypeError.
    // NOTE: In our implementation, the enum type of context already indicates the format.
    let mut decompressor = ds.context.borrow_mut();
    let mut offset = 0;
    let mut written = 1;
    while offset < chunk.len() && written > 0 {
        written = decompressor
            .write(&chunk[offset..])
            .map_err(|_| Error::Type("DecompressionStream: write() failed".to_string()))?;
        offset += written;
    }
    decompressor
        .flush()
        .map_err(|_| Error::Type("DecompressionStream: flush() failed".to_string()))?;
    let buffer = decompressor.get_ref();

    // Step 3. If buffer is empty, return.
    if buffer.is_empty() {
        return Ok(());
    }

    // Step 4. Let arrays be the result of splitting buffer into one or more non-empty pieces and
    // converting them into Uint8Arrays.
    // Step 5. For each Uint8Array array of arrays, enqueue array in ds’s transform.
    // NOTE: We process the result in a single Uint8Array.
    rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
    let array = create_buffer_source::<Uint8>(cx, buffer, js_object.handle_mut(), can_gc)
        .map_err(|_| Error::Type("Cannot convert byte sequence to Uint8Array".to_owned()))?;
    rooted!(in(*cx) let mut rval = UndefinedValue());
    array.safe_to_jsval(cx, rval.handle_mut(), can_gc);
    controller.enqueue(cx, global, rval.handle(), can_gc)?;

    // NOTE: We don't need to keep result that has been copied to Uint8Array. Clear the inner
    // buffer of decompressor to save memory.
    decompressor.get_mut().clear();

    // Step 6. If the end of the compressed input has been reached, and ds’s context has not fully
    // consumed chunk, then throw a TypeError.
    if offset < chunk.len() {
        return Err(Error::Type(
            "The end of the compressed input has been reached".to_string(),
        ));
    }

    Ok(())
}

/// <https://compression.spec.whatwg.org/#decompress-flush-and-enqueue>
pub(crate) fn decompress_flush_and_enqueue(
    cx: SafeJSContext,
    global: &GlobalScope,
    ds: &DecompressionStream,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc,
) -> Fallible<()> {
    // Step 1. Let buffer be the result of decompressing an empty input with ds’s format and
    // context, with the finish flag.
    // NOTE: In our implementation, the enum type of context already indicates the format.
    let mut decompressor = ds.context.borrow_mut();
    let offset = decompressor.get_ref().len();
    let is_ended = decompressor
        .write(&[0])
        .map_err(|_| Error::Type("DecompressionStream: write() failed".to_string()))? ==
        0;
    decompressor
        .try_finish()
        .map_err(|_| Error::Type("DecompressionStream: try_finish() failed".to_string()))?;
    let buffer = &decompressor.get_ref()[offset..];

    // Step 2. If buffer is empty, return.
    if !buffer.is_empty() {
        // Step 2.1. Let arrays be the result of splitting buffer into one or more non-empty pieces
        // and converting them into Uint8Arrays.
        // Step 2.2. For each Uint8Array array of arrays, enqueue array in ds’s transform.
        // NOTE: We process the result in a single Uint8Array.
        rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
        let array = create_buffer_source::<Uint8>(cx, buffer, js_object.handle_mut(), can_gc)
            .map_err(|_| Error::Type("Cannot convert byte sequence to Uint8Array".to_owned()))?;
        rooted!(in(*cx) let mut rval = UndefinedValue());
        array.safe_to_jsval(cx, rval.handle_mut(), can_gc);
        controller.enqueue(cx, global, rval.handle(), can_gc)?;
    }

    // NOTE: We don't need to keep result that has been copied to Uint8Array. Clear the inner
    // buffer of decompressor to save memory.
    decompressor.get_mut().clear();

    // Step 3. If the end of the compressed input has not been reached, then throw a TypeError.
    //
    // NOTE: If the end of the compressed input has not been reached, flate2::write::DeflateDecoder
    // and flate2::write::GzDecoder can detect it and throw an error on `try_finish` in Step 1.
    // However, flate2::write::ZlibDecoder does not. We need to test it by ourselves.
    //
    // To test it, we write one more byte to the decoder. If it accepts the extra byte, this
    // indicates the end has not been reached. Otherwise, the end has been reached. This test has
    // to been done before calling `try_finish`, so we execute it in Step 1, and store the result
    // in `is_ended`.
    if !is_ended {
        return Err(Error::Type(
            "The end of the compressed input has not been reached".to_string(),
        ));
    }

    Ok(())
}
