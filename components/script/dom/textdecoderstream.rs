/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use dom_struct::dom_struct;
use encoding_rs::{Decoder, DecoderResult, Encoding};
use js::conversions::{FromJSValConvertible, ToJSValConvertible};
use js::jsval::UndefinedValue;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};
use script_bindings::codegen::GenericBindings::TransformStreamDefaultControllerBinding::TransformStreamDefaultControllerMethods;

use crate::DomTypes;
use crate::dom::bindings::codegen::Bindings::TextDecoderBinding;
use crate::dom::bindings::codegen::Bindings::TextDecoderStreamBinding::TextDecoderStreamMethods;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferOrArrayBufferView;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::transformstreamdefaultcontroller::{
    TransformerFlushAlgorithm, TransformerFlushAlgorithmType, TransformerTransformAlgorithm, TransformerTransformAlgorithmType
};
use crate::dom::types::{Promise, TransformStream, TransformStreamDefaultController};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

// The common part of [`decode_and_enqueue_a_chunk`] and [`flush_and_enqueue`]
#[allow(unsafe_code)]
fn decode_and_enqueue(
    cx: SafeJSContext,
    fatal: bool,
    decoder: &RefCell<Decoder>,
    in_stream: &RefCell<Vec<u8>>,
    controller: &TransformStreamDefaultController,
    last: bool,
    can_gc: CanGc,
) -> Fallible<()> {
    // Step 3. Let output be the I/O queue of scalar values « end-of-queue ».
    let mut decoder = decoder.borrow_mut();
    let mut in_stream = in_stream.borrow_mut();
    
    // Step 4.
    let (mut remaining, output_chunk) = if fatal {
        let mut output_chunk = String::with_capacity(
            decoder
                .max_utf8_buffer_length_without_replacement(in_stream.len())
                .expect("failed to calculate MX UTF8 buffer length")
        );
        match decoder.decode_to_string_without_replacement(
            &in_stream,
            &mut output_chunk,
            last,
        ) {
            (DecoderResult::InputEmpty, read) => (in_stream.split_off(read), output_chunk),
            _ => return Err(Error::Type("Decoding failed".to_owned())),
        }
    } else {
        let mut output_chunk = String::with_capacity(
            decoder
                .max_utf8_buffer_length(in_stream.len())
                .expect("failed to calculate MAX UTF8 buffer length"),
        );
        let (_result, read, _replaced) = decoder.decode_to_string(&in_stream, &mut output_chunk, last);
        let remaining = in_stream.split_off(read);
        (remaining, output_chunk)
    };
    std::mem::swap(&mut *in_stream, &mut remaining);

    rooted!(in(*cx) let mut rval = UndefinedValue());
    unsafe { output_chunk.to_jsval(*cx, rval.handle_mut()) };

    controller.Enqueue(cx, rval.handle(), can_gc)?;

    Ok(())
}

// https://encoding.spec.whatwg.org/#decode-and-enqueue-a-chunk
#[allow(unsafe_code)]
fn decode_and_enqueue_a_chunk(
    cx: SafeJSContext,
    fatal: bool,
    chunk: SafeHandleValue,
    decoder: &RefCell<Decoder>,
    in_stream: &RefCell<Vec<u8>>,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc,
) -> Fallible<()> {
    // Step 1. Let bufferSource be the result of converting chunk to an AllowSharedBufferSource.
    let conversion_result = unsafe {
        ArrayBufferOrArrayBufferView::from_jsval(*cx, chunk, ()).map_err(|_| {
            Error::Type("Unable to convert chunk into ArrayBuffer or ArrayBufferView".to_string())
        })?
    };
    let buffer_source = conversion_result.get_success_value().ok_or(Error::Type(
        "Unable to convert chunk into ArrayBuffer or ArrayBufferView".to_string(),
    ))?;

    // Step 2. Push a copy of bufferSource to decoder’s I/O queue.
    unsafe {
        match buffer_source {
            ArrayBufferOrArrayBufferView::ArrayBuffer(a) => {
                in_stream.borrow_mut().extend_from_slice(a.as_slice())
            },
            ArrayBufferOrArrayBufferView::ArrayBufferView(a) => {
                in_stream.borrow_mut().extend_from_slice(a.as_slice())
            },
        }
    }

    decode_and_enqueue(cx, fatal, decoder, in_stream, controller, false, can_gc)
}

#[derive(JSTraceable, MallocSizeOf)]
struct TextDecoderStreamTransformAlgorithm {
    stream: MutNullableDom<TextDecoderStream>,
}

impl TextDecoderStreamTransformAlgorithm {
    fn set_stream(&self, stream: &TextDecoderStream) {
        self.stream.set(Some(stream));
    }
}

impl TransformerTransformAlgorithm for TextDecoderStreamTransformAlgorithm {
    fn run(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: SafeHandleValue,
        controller: &TransformStreamDefaultController,
        can_gc: CanGc,
    ) -> Fallible<std::rc::Rc<super::types::Promise>> {
        let stream = self.stream.get()
            .expect("stream is not set");

        decode_and_enqueue_a_chunk(
            cx,
            stream.fatal,
            chunk,
            &stream.decoder,
            &stream.in_stream,
            controller,
            can_gc,
        )
        .map(|_| Promise::new_resolved(global, cx, (), can_gc))
    }
}

// https://encoding.spec.whatwg.org/#flush-and-enqueue
fn flush_and_enqueue(
    cx: SafeJSContext,
    fatal: bool,
    decoder: &RefCell<Decoder>,
    in_stream: &RefCell<Vec<u8>>,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc,
) -> Fallible<()> {
    decode_and_enqueue(cx, fatal, decoder, in_stream, controller, true, can_gc)
}

#[derive(JSTraceable, MallocSizeOf)]
struct TextDecoderStreamFlushAlgorithm {
    stream: MutNullableDom<TextDecoderStream>,
}

impl TextDecoderStreamFlushAlgorithm {
    fn set_stream(&self, stream: &TextDecoderStream) {
        self.stream.set(Some(stream));
    }
}

impl TransformerFlushAlgorithm for TextDecoderStreamFlushAlgorithm {
    fn run(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        controller: &TransformStreamDefaultController,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        let stream = self.stream.get()
            .expect("stream is not set");

        flush_and_enqueue(cx, stream.fatal, &stream.decoder, &stream.in_stream, controller, can_gc)
            .map(|_| Promise::new_resolved(global, cx, (), can_gc))
    }
}

#[dom_struct]
#[allow(non_snake_case)]
pub(crate) struct TextDecoderStream {
    reflector_: Reflector,
    #[no_trace]
    encoding: &'static Encoding,
    fatal: bool,
    ignoreBOM: bool,
    #[ignore_malloc_size_of = "defined in encoding_rs"]
    #[no_trace]
    decoder: RefCell<Decoder>,
    in_stream: RefCell<Vec<u8>>,
    transform: Dom<TransformStream>,
}

#[allow(non_snake_case)]
impl TextDecoderStream {
    fn new_inherited(
        encoding: &'static Encoding,
        fatal: bool,
        ignoreBOM: bool,
        transform_stream: &TransformStream,
    ) -> Self {
        let decoder = if ignoreBOM {
            encoding.new_decoder()
        } else {
            encoding.new_decoder_without_bom_handling()
        };
        let decoder = RefCell::new(decoder);
        let in_stream = RefCell::new(Vec::new());

        Self {
            reflector_: Reflector::new(),
            encoding,
            fatal,
            ignoreBOM,
            decoder,
            in_stream,
            transform: Dom::from_ref(transform_stream),
        }
    }

    pub(crate) fn new_with_proto(
        cx: SafeJSContext,
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        encoding: &'static Encoding,
        fatal: bool,
        ignoreBOM: bool,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Self>> {
        let transform = Rc::new(TextDecoderStreamTransformAlgorithm {
            stream: MutNullableDom::new(None),
        });
        let flush = Rc::new(TextDecoderStreamFlushAlgorithm {
            stream: MutNullableDom::new(None),
        });
        let cancel = None;

        let transform_wrapper = TransformerTransformAlgorithmType::Native(transform.clone());
        let flush_wrapper = TransformerFlushAlgorithmType::Native(flush.clone());

        let transform_stream = TransformStream::new_with_proto(global, None, can_gc);
        transform_stream.set_up(cx, global, cancel, Some(flush_wrapper), transform_wrapper, can_gc)?;

        let stream = TextDecoderStream::new_inherited(encoding, fatal, ignoreBOM, &transform_stream);
        transform.set_stream(&stream);
        flush.set_stream(&stream);

        Ok(reflect_dom_object_with_proto(
            Box::new(stream),
            global,
            proto,
            can_gc,
        ))
    }
}

#[allow(non_snake_case)]
impl TextDecoderStreamMethods<crate::DomTypeHolder> for TextDecoderStream {
    // https://encoding.spec.whatwg.org/#dom-textdecoderstream
    fn Constructor(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        label: DOMString,
        options: &TextDecoderBinding::TextDecoderOptions,
    ) -> Fallible<DomRoot<TextDecoderStream>> {
        let encoding = match Encoding::for_label_no_replacement(label.as_bytes()) {
            Some(enc) => enc,
            None => {
                return Err(Error::Range(
                    "The given encoding is not supported".to_owned(),
                ));
            },
        };

        Self::new_with_proto(
            GlobalScope::get_cx(),
            global,
            proto,
            encoding,
            options.fatal,
            options.ignoreBOM,
            can_gc,
        )
    }

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

    fn Readable(&self) -> DomRoot<<crate::DomTypeHolder as DomTypes>::ReadableStream> {
        self.transform.get_readable()
    }

    fn Writable(&self) -> DomRoot<<crate::DomTypeHolder as DomTypes>::WritableStream> {
        self.transform.get_writable()
    }
}
