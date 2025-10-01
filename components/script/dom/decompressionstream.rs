/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::io::{self, Write};
use std::ptr;

use dom_struct::dom_struct;
use flate2::write::{DeflateDecoder, GzDecoder, ZlibDecoder};
use js::jsapi::JSObject;
use js::jsval::UndefinedValue;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};
use js::typedarray::Uint8Array;
use script_bindings::conversions::{SafeFromJSValConvertible, SafeToJSValConvertible};

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::CompressionStreamBinding::CompressionFormat;
use crate::dom::bindings::codegen::Bindings::DecompressionStreamBinding::DecompressionStreamMethods;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
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
        }
    }

    fn get_ref(&self) -> &Vec<u8> {
        match self {
            Decompressor::Deflate(zlib_decoder) => zlib_decoder.get_ref(),
            Decompressor::DeflateRaw(deflate_decoder) => deflate_decoder.get_ref(),
            Decompressor::Gzip(gz_decoder) => gz_decoder.get_ref(),
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        match self {
            Decompressor::Deflate(zlib_decoder) => zlib_decoder.write(buf),
            Decompressor::DeflateRaw(deflate_decoder) => deflate_decoder.write(buf),
            Decompressor::Gzip(gz_decoder) => gz_decoder.write(buf),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), io::Error> {
        match self {
            Decompressor::Deflate(zlib_decoder) => zlib_decoder.write_all(buf),
            Decompressor::DeflateRaw(deflate_decoder) => deflate_decoder.write_all(buf),
            Decompressor::Gzip(gz_decoder) => gz_decoder.write_all(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Decompressor::Deflate(zlib_decoder) => zlib_decoder.flush(),
            Decompressor::DeflateRaw(deflate_decoder) => deflate_decoder.flush(),
            Decompressor::Gzip(gz_decoder) => gz_decoder.flush(),
        }
    }

    fn try_finish(&mut self) -> io::Result<()> {
        match self {
            Decompressor::Deflate(zlib_decoder) => zlib_decoder.try_finish(),
            Decompressor::DeflateRaw(deflate_decoder) => deflate_decoder.try_finish(),
            Decompressor::Gzip(gz_decoder) => gz_decoder.try_finish(),
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
    #[ignore_malloc_size_of = "defined in flate2"]
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

    /// <https://compression.spec.whatwg.org/#dom-decompressionstream-decompressionstream>
    fn new_with_proto(
        cx: SafeJSContext,
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        format: CompressionFormat,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<DecompressionStream>> {
        // Step 1. If format is unsupported in DecompressionStream, then throw a TypeError.
        // NOTE: All of "deflate", "deflate_raw" and "gzip" are supported.

        // Step 2. Set this’s format to format.
        // Step 5. Set this’s transform to a new TransformStream.
        let transform_stream = TransformStream::new_with_proto(global, None, can_gc);
        let this = reflect_dom_object_with_proto(
            Box::new(DecompressionStream::new_inherited(
                &transform_stream,
                format,
            )),
            global,
            proto,
            can_gc,
        );

        // Step 3. Let transformAlgorithm be an algorithm which takes a chunk argument and runs the
        // decompress and enqueue a chunk algorithm with this and chunk.
        // Step 4. Let flushAlgorithm be an algorithm which takes no argument and runs the
        // decompress flush and enqueue algorithm with this.
        let transformer_type = TransformerType::Decompressor(this.clone());

        // Step 6. Set up this’s transform with transformAlgorithm set to transformAlgorithm and
        // flushAlgorithm set to flushAlgorithm.
        transform_stream.set_up(cx, global, transformer_type, can_gc)?;

        Ok(this)
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
        let cx = GlobalScope::get_cx();
        DecompressionStream::new_with_proto(cx, global, proto, format, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#dom-generictransformstream-readable>
    fn Readable(&self) -> DomRoot<ReadableStream> {
        self.transform.get_readable()
    }

    /// <https://streams.spec.whatwg.org/#dom-generictransformstream-writable>
    fn Writable(&self) -> DomRoot<WritableStream> {
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
    let conversion_result =
        ArrayBufferViewOrArrayBuffer::safe_from_jsval(cx, chunk, ()).map_err(|_| {
            Error::Type("Unable to convert chunk into ArrayBuffer or ArrayBufferView".to_string())
        })?;
    let buffer_source = conversion_result.get_success_value().ok_or_else(|| {
        Error::Type("Unable to convert chunk into ArrayBuffer or ArrayBufferView".to_string())
    })?;
    let chunk = match buffer_source {
        ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
        ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
    };

    // Step 2. Let buffer be the result of decompressing chunk with ds’s format and context. If
    // this results in an error, then throw a TypeError.
    // NOTE: In our implementation, the enum type of context already indicates the format.
    let mut decompressor = ds.context.borrow_mut();
    let offset = decompressor.get_ref().len();
    decompressor
        .write_all(&chunk)
        .map_err(|_| Error::Type("Fail to write all to decompressor".to_string()))?;
    decompressor
        .flush()
        .map_err(|_| Error::Type("Fail to flush decompressor".to_string()))?;
    let buffer = &decompressor.get_ref()[offset..];

    // Step 3. If buffer is empty, return.
    if buffer.is_empty() {
        return Ok(());
    }

    // Step 4. Let arrays be the result of splitting buffer into one or more non-empty pieces and
    // converting them into Uint8Arrays.
    // Step 5. For each Uint8Array array of arrays, enqueue array in ds’s transform.
    // NOTE: We process the result in a single Uint8Array.
    rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
    let array: Uint8Array = create_buffer_source(cx, buffer, js_object.handle_mut(), can_gc)
        .map_err(|_| Error::Type("Cannot convert byte sequence to Uint8Array".to_owned()))?;
    rooted!(in(*cx) let mut rval = UndefinedValue());
    array.safe_to_jsval(cx, rval.handle_mut());
    controller.enqueue(cx, global, rval.handle(), can_gc)?;

    // Step 6. If the end of the compressed input has been reached, and ds’s context has not fully
    // consumed chunk, then throw a TypeError.
    // NOTE: Done by `write_all` in Step 2.

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
        .map_err(|_| Error::Type("Fail to finish decompressor".to_string()))? ==
        0;
    decompressor
        .try_finish()
        .map_err(|_| Error::Type("Fail to finish decompressor".to_string()))?;
    let buffer = &decompressor.get_ref()[offset..];

    // Step 2. If buffer is empty, return.
    if !buffer.is_empty() {
        // Step 2.1. Let arrays be the result of splitting buffer into one or more non-empty pieces
        // and converting them into Uint8Arrays.
        // Step 2.2. For each Uint8Array array of arrays, enqueue array in ds’s transform.
        // NOTE: We process the result in a single Uint8Array.
        rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
        let array: Uint8Array = create_buffer_source(cx, buffer, js_object.handle_mut(), can_gc)
            .map_err(|_| Error::Type("Cannot convert byte sequence to Uint8Array".to_owned()))?;
        rooted!(in(*cx) let mut rval = UndefinedValue());
        array.safe_to_jsval(cx, rval.handle_mut());
        controller.enqueue(cx, global, rval.handle(), can_gc)?;
    }

    // Step 3. If the end of the compressed input has not been reached, then throw a TypeError.
    // NOTE: To test whether the compressed input has reached the end, we try to write one more
    // byte to the inner decoder of decompressor. If the decoder accepts the extra byte, it
    // indicates that the end has not been reached. Otherwise, the end has been reached. This test
    // has to been done before calling `try_finish`, so we execute it in Step 1, and store the
    // result in `is_ended`.
    if !is_ended {
        return Err(Error::Type(
            "The end of the compressed input has not been reached".to_string(),
        ));
    }

    Ok(())
}
