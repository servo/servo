/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use encoding_rs::Encoding;
use js::conversions::{FromJSValConvertible, ToJSValConvertible};
use js::jsval::UndefinedValue;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};

use crate::DomTypes;
use crate::dom::bindings::codegen::Bindings::TextDecoderBinding;
use crate::dom::bindings::codegen::Bindings::TextDecoderStreamBinding::TextDecoderStreamMethods;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::textdecodercommon::TextDecoderCommon;
use crate::dom::transformstreamdefaultcontroller::{
    TransformerFlushAlgorithm, TransformerFlushAlgorithmType, TransformerTransformAlgorithm,
    TransformerTransformAlgorithmType,
};
use crate::dom::types::{Promise, TransformStream, TransformStreamDefaultController};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

// https://encoding.spec.whatwg.org/#decode-and-enqueue-a-chunk
#[allow(unsafe_code)]
fn decode_and_enqueue_a_chunk(
    cx: SafeJSContext,
    global: &GlobalScope,
    chunk: SafeHandleValue,
    decoder: &TextDecoderCommon,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc,
) -> Fallible<()> {
    // Step 1. Let bufferSource be the result of converting chunk to an AllowSharedBufferSource.
    let conversion_result = unsafe {
        ArrayBufferViewOrArrayBuffer::from_jsval(*cx, chunk, ()).map_err(|_| {
            Error::Type("Unable to convert chunk into ArrayBuffer or ArrayBufferView".to_string())
        })?
    };
    let buffer_source = conversion_result.get_success_value().ok_or(Error::Type(
        "Unable to convert chunk into ArrayBuffer or ArrayBufferView".to_string(),
    ))?;

    let output_chunk = decoder.decode(Some(buffer_source), true)?;
    if output_chunk.is_empty() {
        return Ok(())
    }

    rooted!(in(*cx) let mut rval = UndefinedValue());
    unsafe { output_chunk.to_jsval(*cx, rval.handle_mut()) };

    controller.enqueue(cx, global, rval.handle(), can_gc)
}

#[derive(JSTraceable, MallocSizeOf)]
struct TextDecoderStreamTransformAlgorithm {
    #[ignore_malloc_size_of = "Rc is hard"]
    decoder: Rc<TextDecoderCommon>,
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
        decode_and_enqueue_a_chunk(cx, global, chunk, &self.decoder, controller, can_gc)
            .map(|_| Promise::new_resolved(global, cx, (), can_gc))
    }
}

// https://encoding.spec.whatwg.org/#flush-and-enqueue
#[allow(unsafe_code)]
fn flush_and_enqueue(
    cx: SafeJSContext,
    global: &GlobalScope,
    decoder: &TextDecoderCommon,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc,
) -> Fallible<()> {
    let output_chunk = decoder.decode(None, false)?;
    if output_chunk.is_empty() {
        return Ok(())
    }

    rooted!(in(*cx) let mut rval = UndefinedValue());
    unsafe { output_chunk.to_jsval(*cx, rval.handle_mut()) };

    controller.enqueue(cx, global, rval.handle(), can_gc)
}

#[derive(JSTraceable, MallocSizeOf)]
struct TextDecoderStreamFlushAlgorithm {
    #[ignore_malloc_size_of = "Rc is hard"]
    decoder: Rc<TextDecoderCommon>,
}

impl TransformerFlushAlgorithm for TextDecoderStreamFlushAlgorithm {
    fn run(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        controller: &TransformStreamDefaultController,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        flush_and_enqueue(cx, global, &self.decoder, controller, can_gc)
            .map(|_| Promise::new_resolved(global, cx, (), can_gc))
    }
}

#[dom_struct]
#[allow(non_snake_case)]
pub(crate) struct TextDecoderStream {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "Rc is hard"]
    decoder: Rc<TextDecoderCommon>,
    transform: Dom<TransformStream>,
}

#[allow(non_snake_case)]
impl TextDecoderStream {
    fn new_inherited(
        decoder: Rc<TextDecoderCommon>,
        transform: &TransformStream,
    ) -> TextDecoderStream {
        TextDecoderStream {
            reflector_: Reflector::new(),
            decoder,
            transform: Dom::from_ref(transform),
        }
    }

    fn new_with_proto(
        cx: SafeJSContext,
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        encoding: &'static Encoding,
        fatal: bool,
        ignoreBOM: bool,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Self>> {
        let decoder = Rc::new(TextDecoderCommon::new_inherited(encoding, fatal, ignoreBOM));
        let transform = Rc::new(TextDecoderStreamTransformAlgorithm {
            decoder: decoder.clone(),
        });
        let flush = Rc::new(TextDecoderStreamFlushAlgorithm {
            decoder: decoder.clone(),
        });
        let cancel = None;

        let transform_wrapper = TransformerTransformAlgorithmType::Native(transform.clone());
        let flush_wrapper = TransformerFlushAlgorithmType::Native(flush.clone());

        let transform_stream = TransformStream::new_with_proto(global, None, can_gc);
        transform_stream.set_up(
            cx,
            global,
            cancel,
            Some(flush_wrapper),
            transform_wrapper,
            can_gc,
        )?;

        let stream = TextDecoderStream::new_inherited(decoder, &transform_stream);

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
        self.decoder.encoding()
    }

    // https://encoding.spec.whatwg.org/#dom-textdecoder-fatal
    fn Fatal(&self) -> bool {
        self.decoder.fatal()
    }

    // https://encoding.spec.whatwg.org/#dom-textdecoder-ignorebom
    fn IgnoreBOM(&self) -> bool {
        self.decoder.ignore_bom()
    }

    // https://streams.spec.whatwg.org/#dom-generictransformstream-readable
    fn Readable(&self) -> DomRoot<<crate::DomTypeHolder as DomTypes>::ReadableStream> {
        self.transform.get_readable()
    }

    // https://streams.spec.whatwg.org/#dom-generictransformstream-writable
    fn Writable(&self) -> DomRoot<<crate::DomTypeHolder as DomTypes>::WritableStream> {
        self.transform.get_writable()
    }
}
