/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::io::{self, Write};
use std::ptr;

use brotli::CompressorWriter as BrotliEncoder;
use dom_struct::dom_struct;
use flate2::Compression;
use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};
use js::jsapi::JSObject;
use js::jsval::UndefinedValue;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};
use js::typedarray::Uint8;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::CompressionStreamBinding::{
    CompressionFormat, CompressionStreamMethods,
};
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use crate::dom::bindings::conversions::{SafeFromJSValConvertible, SafeToJSValConvertible};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::transformstreamdefaultcontroller::TransformerType;
use crate::dom::types::{
    GlobalScope, ReadableStream, TransformStream, TransformStreamDefaultController, WritableStream,
};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// A wrapper to blend ZlibEncoder<Vec<u8>>, DeflateEncoder<Vec<u8>> and GzEncoder<Vec<u8>>
/// together as a single type.
enum Compressor {
    Deflate(ZlibEncoder<Vec<u8>>),
    DeflateRaw(DeflateEncoder<Vec<u8>>),
    Gzip(GzEncoder<Vec<u8>>),
    Brotli(Box<BrotliEncoder<Vec<u8>>>),
}

/// Expose methods of the inner encoder.
impl Compressor {
    fn new(format: CompressionFormat) -> Compressor {
        match format {
            CompressionFormat::Deflate => {
                Compressor::Deflate(ZlibEncoder::new(Vec::new(), Compression::default()))
            },
            CompressionFormat::Deflate_raw => {
                Compressor::DeflateRaw(DeflateEncoder::new(Vec::new(), Compression::default()))
            },
            CompressionFormat::Gzip => {
                Compressor::Gzip(GzEncoder::new(Vec::new(), Compression::default()))
            },
            CompressionFormat::Brotli => {
                Compressor::Brotli(Box::new(BrotliEncoder::new(Vec::new(), 4096, 5, 22)))
            },
        }
    }

    fn get_ref(&self) -> &Vec<u8> {
        match self {
            Compressor::Deflate(zlib_encoder) => zlib_encoder.get_ref(),
            Compressor::DeflateRaw(deflate_encoder) => deflate_encoder.get_ref(),
            Compressor::Gzip(gz_encoder) => gz_encoder.get_ref(),
            Compressor::Brotli(brotli_encoder) => brotli_encoder.get_ref(),
        }
    }

    fn get_mut(&mut self) -> &mut Vec<u8> {
        match self {
            Compressor::Deflate(zlib_encoder) => zlib_encoder.get_mut(),
            Compressor::DeflateRaw(deflate_encoder) => deflate_encoder.get_mut(),
            Compressor::Gzip(gz_encoder) => gz_encoder.get_mut(),
            Compressor::Brotli(brotli_encoder) => brotli_encoder.get_mut(),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), io::Error> {
        match self {
            Compressor::Deflate(zlib_encoder) => zlib_encoder.write_all(buf),
            Compressor::DeflateRaw(deflate_encoder) => deflate_encoder.write_all(buf),
            Compressor::Gzip(gz_encoder) => gz_encoder.write_all(buf),
            Compressor::Brotli(brotli_encoder) => brotli_encoder.write_all(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Compressor::Deflate(zlib_encoder) => zlib_encoder.flush(),
            Compressor::DeflateRaw(deflate_encoder) => deflate_encoder.flush(),
            Compressor::Gzip(gz_encoder) => gz_encoder.flush(),
            Compressor::Brotli(brotli_encoder) => brotli_encoder.flush(),
        }
    }

    fn try_finish(&mut self) -> io::Result<()> {
        match self {
            Compressor::Deflate(zlib_encoder) => zlib_encoder.try_finish(),
            Compressor::DeflateRaw(deflate_encoder) => deflate_encoder.try_finish(),
            Compressor::Gzip(gz_encoder) => gz_encoder.try_finish(),
            Compressor::Brotli(brotli_encoder) => brotli_encoder.flush(),
        }
    }
}

impl MallocSizeOf for Compressor {
    #[expect(unsafe_code)]
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match self {
            Compressor::Deflate(zlib_encoder) => zlib_encoder.size_of(ops),
            Compressor::DeflateRaw(deflate_encoder) => deflate_encoder.size_of(ops),
            Compressor::Gzip(gz_encoder) => gz_encoder.size_of(ops),
            Compressor::Brotli(brotli_dencoder) => unsafe {
                ops.malloc_size_of(&**brotli_dencoder)
            },
        }
    }
}

/// <https://compression.spec.whatwg.org/#compressionstream>
#[dom_struct]
pub(crate) struct CompressionStream {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#generictransformstream>
    transform: Dom<TransformStream>,

    /// <https://compression.spec.whatwg.org/#compressionstream-format>
    format: CompressionFormat,

    // <https://compression.spec.whatwg.org/#compressionstream-context>
    #[no_trace]
    context: RefCell<Compressor>,
}

impl CompressionStream {
    fn new_inherited(transform: &TransformStream, format: CompressionFormat) -> CompressionStream {
        CompressionStream {
            reflector_: Reflector::new(),
            transform: Dom::from_ref(transform),
            format,
            context: RefCell::new(Compressor::new(format)),
        }
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        transform: &TransformStream,
        format: CompressionFormat,
        can_gc: CanGc,
    ) -> DomRoot<CompressionStream> {
        reflect_dom_object_with_proto(
            Box::new(CompressionStream::new_inherited(transform, format)),
            global,
            proto,
            can_gc,
        )
    }
}

impl CompressionStreamMethods<crate::DomTypeHolder> for CompressionStream {
    /// <https://compression.spec.whatwg.org/#dom-compressionstream-compressionstream>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        format: CompressionFormat,
    ) -> Fallible<DomRoot<CompressionStream>> {
        // Step 1. If format is unsupported in CompressionStream, then throw a TypeError.
        // NOTE: All of "deflate", "deflate-raw", "gzip" and "br" are supported.

        // Step 2. Set this’s format to format.
        // Step 5. Set this’s transform to a new TransformStream.
        let transform = TransformStream::new_with_proto(global, None, can_gc);
        let compression_stream =
            CompressionStream::new_with_proto(global, proto, &transform, format, can_gc);

        // Step 3. Let transformAlgorithm be an algorithm which takes a chunk argument and runs the
        // compress and enqueue a chunk algorithm with this and chunk.
        // Step 4. Let flushAlgorithm be an algorithm which takes no argument and runs the compress
        // flush and enqueue algorithm with this.
        let transformer_type = TransformerType::Compressor(compression_stream.clone());

        // Step 6. Set up this’s transform with transformAlgorithm set to transformAlgorithm and
        // flushAlgorithm set to flushAlgorithm.
        let cx = GlobalScope::get_cx();
        transform.set_up(cx, global, transformer_type, can_gc)?;

        Ok(compression_stream)
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

/// <https://compression.spec.whatwg.org/#compress-and-enqueue-a-chunk>
pub(crate) fn compress_and_enqueue_a_chunk(
    cx: SafeJSContext,
    global: &GlobalScope,
    cs: &CompressionStream,
    chunk: SafeHandleValue,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc,
) -> Fallible<()> {
    // Step 1. If chunk is not a BufferSource type, then throw a TypeError.
    let chunk = convert_chunk_to_vec(cx, chunk, can_gc)?;

    // Step 2. Let buffer be the result of compressing chunk with cs’s format and context.
    // NOTE: In our implementation, the enum type of context already indicates the format.
    let mut compressor = cs.context.borrow_mut();
    let offset = compressor.get_ref().len();
    compressor
        .write_all(&chunk)
        .map_err(|_| Error::Type("CompressionStream: write_all() failed".to_string()))?;
    compressor
        .flush()
        .map_err(|_| Error::Type("CompressionStream: flush() failed".to_string()))?;
    let buffer = &compressor.get_ref()[offset..];

    // Step 3. If buffer is empty, return.
    if buffer.is_empty() {
        return Ok(());
    }

    // Step 4. Let arrays be the result of splitting buffer into one or more non-empty pieces and
    // converting them into Uint8Arrays.
    // Step 5. For each Uint8Array array of arrays, enqueue array in cs’s transform.
    // NOTE: We process the result in a single Uint8Array.
    rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
    let buffer_source =
        create_buffer_source::<Uint8>(cx, buffer, js_object.handle_mut(), can_gc)
            .map_err(|_| Error::Type("Cannot convert byte sequence to Uint8Array".to_owned()))?;
    rooted!(in(*cx) let mut rval = UndefinedValue());
    buffer_source.safe_to_jsval(cx, rval.handle_mut(), can_gc);
    controller.enqueue(cx, global, rval.handle(), can_gc)?;

    // NOTE: We don't need to keep result that has been copied to Uint8Array. Clear the inner
    // buffer of compressor to save memory.
    compressor.get_mut().clear();

    Ok(())
}

/// <https://compression.spec.whatwg.org/#compress-flush-and-enqueue>
pub(crate) fn compress_flush_and_enqueue(
    cx: SafeJSContext,
    global: &GlobalScope,
    cs: &CompressionStream,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc,
) -> Fallible<()> {
    // Step 1. Let buffer be the result of compressing an empty input with cs’s format and context,
    // with the finish flag.
    // NOTE: In our implementation, the enum type of context already indicates the format.
    let mut compressor = cs.context.borrow_mut();
    let offset = compressor.get_ref().len();
    compressor
        .try_finish()
        .map_err(|_| Error::Type("CompressionStream: try_finish() failed".to_string()))?;
    let buffer = &compressor.get_ref()[offset..];

    // Step 2. If buffer is empty, return.
    if buffer.is_empty() {
        return Ok(());
    }

    // Step 3. Let arrays be the result of splitting buffer into one or more non-empty pieces and
    // converting them into Uint8Arrays.
    // Step 4. For each Uint8Array array of arrays, enqueue array in cs’s transform.
    // NOTE: We process the result in a single Uint8Array.
    rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
    let buffer_source =
        create_buffer_source::<Uint8>(cx, buffer, js_object.handle_mut(), can_gc)
            .map_err(|_| Error::Type("Cannot convert byte sequence to Uint8Array".to_owned()))?;
    rooted!(in(*cx) let mut rval = UndefinedValue());
    buffer_source.safe_to_jsval(cx, rval.handle_mut(), can_gc);
    controller.enqueue(cx, global, rval.handle(), can_gc)?;

    // NOTE: We don't need to keep result that has been copied to Uint8Array. Clear the inner
    // buffer of compressor to save memory.
    compressor.get_mut().clear();

    Ok(())
}

pub(crate) fn convert_chunk_to_vec(
    cx: SafeJSContext,
    chunk: SafeHandleValue,
    can_gc: CanGc,
) -> Result<Vec<u8>, Error> {
    let conversion_result = ArrayBufferViewOrArrayBuffer::safe_from_jsval(cx, chunk, (), can_gc)
        .map_err(|_| {
            Error::Type("Unable to convert chunk into ArrayBuffer or ArrayBufferView".to_string())
        })?;
    let buffer_source = conversion_result.get_success_value().ok_or_else(|| {
        Error::Type("Unable to convert chunk into ArrayBuffer or ArrayBufferView".to_string())
    })?;
    match buffer_source {
        ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => Ok(view.to_vec()),
        ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => Ok(buffer.to_vec()),
    }
}
