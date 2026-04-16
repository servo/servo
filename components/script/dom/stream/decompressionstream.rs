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
use crate::dom::stream::compressionstream::{BROTLI_BUFFER_SIZE, convert_chunk_to_vec};
use crate::dom::stream::transformstreamdefaultcontroller::TransformerType;
use crate::dom::types::{
    GlobalScope, ReadableStream, TransformStream, TransformStreamDefaultController, WritableStream,
};
use crate::script_runtime::CanGc;

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
    context: RefCell<DecompressionContext>,
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
            context: RefCell::new(DecompressionContext::new(format)),
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
        // NOTE: All of "brotli", "deflate", "deflate-raw" and "gzip" are supported.

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
    cx: &mut js::context::JSContext,
    global: &GlobalScope,
    ds: &DecompressionStream,
    chunk: SafeHandleValue,
    controller: &TransformStreamDefaultController,
) -> Fallible<()> {
    // Step 1. If chunk is not a BufferSource type, then throw a TypeError.
    let chunk = convert_chunk_to_vec(cx.into(), chunk, CanGc::from_cx(cx))?;

    // Step 2. Let buffer be the result of decompressing chunk with ds’s format and context. If
    // this results in an error, then throw a TypeError.
    // NOTE: In our implementation, the enum type of context already indicates the format.
    let mut decompression_context = ds.context.borrow_mut();
    let buffer = decompression_context
        .decompress(&chunk)
        .map_err(|_| Error::Type(c"Failed to decompress a chunk of compressed input".into()))?;

    // Step 3. If buffer is empty, return.
    if buffer.is_empty() {
        return Ok(());
    }

    // Step 4. Let arrays be the result of splitting buffer into one or more non-empty pieces and
    // converting them into Uint8Arrays.
    // Step 5. For each Uint8Array array of arrays, enqueue array in ds’s transform.
    // NOTE: We process the result in a single Uint8Array.
    rooted!(&in(cx) let mut js_object = ptr::null_mut::<JSObject>());
    let array = create_buffer_source::<Uint8>(
        cx.into(),
        &buffer,
        js_object.handle_mut(),
        CanGc::from_cx(cx),
    )
    .map_err(|_| Error::Type(c"Cannot convert byte sequence to Uint8Array".to_owned()))?;
    rooted!(&in(cx) let mut rval = UndefinedValue());
    array.safe_to_jsval(cx.into(), rval.handle_mut(), CanGc::from_cx(cx));
    controller.enqueue(cx, global, rval.handle())?;

    // Step 6. If the end of the compressed input has been reached, and ds’s context has not fully
    // consumed chunk, then throw a TypeError.
    if decompression_context.is_ended {
        return Err(Error::Type(
            c"The end of the compressed input has been reached".to_owned(),
        ));
    }

    Ok(())
}

/// <https://compression.spec.whatwg.org/#decompress-flush-and-enqueue>
pub(crate) fn decompress_flush_and_enqueue(
    cx: &mut js::context::JSContext,
    global: &GlobalScope,
    ds: &DecompressionStream,
    controller: &TransformStreamDefaultController,
) -> Fallible<()> {
    // Step 1. Let buffer be the result of decompressing an empty input with ds’s format and
    // context, with the finish flag.
    // NOTE: In our implementation, the enum type of context already indicates the format.
    let mut decompression_context = ds.context.borrow_mut();
    let buffer = decompression_context
        .finalize()
        .map_err(|_| Error::Type(c"Failed to finalize the decompression stream".into()))?;

    // Step 2. If buffer is empty, return.
    if !buffer.is_empty() {
        // Step 2.1. Let arrays be the result of splitting buffer into one or more non-empty pieces
        // and converting them into Uint8Arrays.
        // Step 2.2. For each Uint8Array array of arrays, enqueue array in ds’s transform.
        // NOTE: We process the result in a single Uint8Array.
        rooted!(&in(cx) let mut js_object = ptr::null_mut::<JSObject>());
        let array = create_buffer_source::<Uint8>(
            cx.into(),
            &buffer,
            js_object.handle_mut(),
            CanGc::from_cx(cx),
        )
        .map_err(|_| Error::Type(c"Cannot convert byte sequence to Uint8Array".to_owned()))?;
        rooted!(&in(cx) let mut rval = UndefinedValue());
        array.safe_to_jsval(cx.into(), rval.handle_mut(), CanGc::from_cx(cx));
        controller.enqueue(cx, global, rval.handle())?;
    }

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
    if !decompression_context.is_ended {
        return Err(Error::Type(
            c"The end of the compressed input has not been reached".to_owned(),
        ));
    }

    Ok(())
}

/// An enum grouping decoders of differenct compression algorithms.
enum Decoder {
    Brotli(Box<BrotliDecoder<Vec<u8>>>),
    Deflate(ZlibDecoder<Vec<u8>>),
    DeflateRaw(DeflateDecoder<Vec<u8>>),
    Gzip(GzDecoder<Vec<u8>>),
}

impl MallocSizeOf for Decoder {
    #[expect(unsafe_code)]
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match self {
            Decoder::Brotli(decoder) => unsafe { ops.malloc_size_of(&**decoder) },
            Decoder::Deflate(decoder) => decoder.size_of(ops),
            Decoder::DeflateRaw(decoder) => decoder.size_of(ops),
            Decoder::Gzip(decoder) => decoder.size_of(ops),
        }
    }
}

/// <https://compression.spec.whatwg.org/#decompressionstream-context>
/// Used to encapsulate the logic of decoder.
#[derive(MallocSizeOf)]
struct DecompressionContext {
    decoder: Decoder,
    is_ended: bool,
}

impl DecompressionContext {
    fn new(format: CompressionFormat) -> DecompressionContext {
        let decoder = match format {
            CompressionFormat::Brotli => {
                Decoder::Brotli(Box::new(BrotliDecoder::new(Vec::new(), BROTLI_BUFFER_SIZE)))
            },
            CompressionFormat::Deflate => Decoder::Deflate(ZlibDecoder::new(Vec::new())),
            CompressionFormat::Deflate_raw => Decoder::DeflateRaw(DeflateDecoder::new(Vec::new())),
            CompressionFormat::Gzip => Decoder::Gzip(GzDecoder::new(Vec::new())),
        };
        DecompressionContext {
            decoder,
            is_ended: false,
        }
    }

    fn decompress(&mut self, mut chunk: &[u8]) -> Result<Vec<u8>, io::Error> {
        let mut result = Vec::new();

        match &mut self.decoder {
            Decoder::Brotli(decoder) => {
                while !chunk.is_empty() {
                    let written = decoder.write(chunk)?;
                    if written == 0 {
                        self.is_ended = true;
                        break;
                    }
                    chunk = &chunk[written..];
                }
                decoder.flush()?;
                result.append(decoder.get_mut());
            },
            Decoder::Deflate(decoder) => {
                while !chunk.is_empty() {
                    let written = decoder.write(chunk)?;
                    if written == 0 {
                        self.is_ended = true;
                        break;
                    }
                    chunk = &chunk[written..];
                }
                decoder.flush()?;
                result.append(decoder.get_mut());
            },
            Decoder::DeflateRaw(decoder) => {
                while !chunk.is_empty() {
                    let written = decoder.write(chunk)?;
                    if written == 0 {
                        self.is_ended = true;
                        break;
                    }
                    chunk = &chunk[written..];
                }
                decoder.flush()?;
                result.append(decoder.get_mut());
            },
            Decoder::Gzip(decoder) => {
                while !chunk.is_empty() {
                    let written = decoder.write(chunk)?;
                    if written == 0 {
                        self.is_ended = true;
                        break;
                    }
                    chunk = &chunk[written..];
                }
                decoder.flush()?;
                result.append(decoder.get_mut());
            },
        }

        Ok(result)
    }

    fn finalize(&mut self) -> Result<Vec<u8>, io::Error> {
        let mut result = Vec::new();

        match &mut self.decoder {
            Decoder::Brotli(decoder) => {
                if decoder.close().is_ok() {
                    self.is_ended = true;
                };
                result.append(decoder.get_mut());
            },
            Decoder::Deflate(decoder) => {
                // Compressed data in "Deflate" format does not have trailing bytes. Therefore,
                // `ZlibEncoder::try_finish` is designed not to throw an error when the end of
                // compressed input has not been reached, in order to decompress as much of the
                // input as possible.
                //
                // To detect whether the end is reached, the workaround is to write one more byte to
                // the encoder. Refusing to take the extra byte indicates the end has been reached.
                //
                // Note that we need to pull out the data in buffer first to avoid the extra byte
                // contaminate the output.
                decoder.flush()?;
                result.append(decoder.get_mut());
                if decoder.write(&[0])? == 0 {
                    self.is_ended = true;
                }
                decoder.try_finish()?;
            },
            Decoder::DeflateRaw(decoder) => {
                if decoder.try_finish().is_ok() {
                    self.is_ended = true;
                };
                result.append(decoder.get_mut());
            },
            Decoder::Gzip(decoder) => {
                if decoder.try_finish().is_ok() {
                    self.is_ended = true;
                };
                result.append(decoder.get_mut());
            },
        }

        Ok(result)
    }
}
