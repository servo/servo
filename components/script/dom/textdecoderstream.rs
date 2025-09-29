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
use crate::dom::transformstreamdefaultcontroller::TransformerType;
use crate::dom::types::{TransformStream, TransformStreamDefaultController};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://encoding.spec.whatwg.org/#decode-and-enqueue-a-chunk>
#[allow(unsafe_code)]
pub(crate) fn decode_and_enqueue_a_chunk(
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
    let buffer_source = conversion_result.get_success_value().ok_or_else(|| {
        Error::Type("Unable to convert chunk into ArrayBuffer or ArrayBufferView".to_string())
    })?;

    // Step 2. Push a copy of bufferSource to decoder’s I/O queue.
    // Step 3. Let output be the I/O queue of scalar values « end-of-queue ».
    // Step 4. While true:
    // Step 4.1 Let item be the result of reading from decoder’s I/O queue.
    // Step 4.2 If item is end-of-queue:
    // Step 4.2.1 Let outputChunk be the result of running serialize I/O queue with decoder and output.
    // Step 4.2.3 Return.
    // Step 4.3 Let result be the result of processing an item with item, decoder’s decoder,
    //      decoder’s I/O queue, output, and decoder’s error mode.
    // Step 4.4 If result is error, then throw a TypeError.
    let output_chunk = decoder.decode(Some(buffer_source), false)?;

    // Step 4.2.2 If outputChunk is not the empty string, then enqueue
    //      outputChunk in decoder’s transform.
    if output_chunk.is_empty() {
        return Ok(());
    }
    rooted!(in(*cx) let mut rval = UndefinedValue());
    unsafe { output_chunk.to_jsval(*cx, rval.handle_mut()) };
    controller.enqueue(cx, global, rval.handle(), can_gc)
}

/// <https://encoding.spec.whatwg.org/#flush-and-enqueue>
#[allow(unsafe_code)]
pub(crate) fn flush_and_enqueue(
    cx: SafeJSContext,
    global: &GlobalScope,
    decoder: &TextDecoderCommon,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc,
) -> Fallible<()> {
    // Step 1. Let output be the I/O queue of scalar values « end-of-queue ».
    // Step 2. While true:
    // Step 2.1 Let item be the result of reading from decoder’s I/O queue.
    // Step 2.2 Let result be the result of processing an item with item,
    //      decoder’s decoder, decoder’s I/O queue, output, and decoder’s error mode.
    // Step 2.3 If result is finished:
    // Step 2.3.1 Let outputChunk be the result of running serialize I/O queue
    //      with decoder and output.
    // Step 2.3.3 Return.
    // Step 2.3.4 Otherwise, if result is error, throw a TypeError.
    let output_chunk = decoder.decode(None, true)?;

    // Step 2.3.2 If outputChunk is not the empty string, then enqueue
    //      outputChunk in decoder’s transform.
    if output_chunk.is_empty() {
        return Ok(());
    }
    rooted!(in(*cx) let mut rval = UndefinedValue());
    unsafe { output_chunk.to_jsval(*cx, rval.handle_mut()) };
    controller.enqueue(cx, global, rval.handle(), can_gc)
}

/// <https://encoding.spec.whatwg.org/#textdecoderstream>
#[dom_struct]
pub(crate) struct TextDecoderStream {
    reflector_: Reflector,

    /// <https://encoding.spec.whatwg.org/#textdecodercommon>
    #[conditional_malloc_size_of]
    decoder: Rc<TextDecoderCommon>,

    /// <https://streams.spec.whatwg.org/#generictransformstream>
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
        let transformer_type = TransformerType::Decoder(decoder.clone());

        let transform_stream = TransformStream::new_with_proto(global, None, can_gc);
        transform_stream.set_up(cx, global, transformer_type, can_gc)?;

        Ok(reflect_dom_object_with_proto(
            Box::new(TextDecoderStream::new_inherited(decoder, &transform_stream)),
            global,
            proto,
            can_gc,
        ))
    }
}

#[allow(non_snake_case)]
impl TextDecoderStreamMethods<crate::DomTypeHolder> for TextDecoderStream {
    /// <https://encoding.spec.whatwg.org/#dom-textdecoderstream>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        label: DOMString,
        options: &TextDecoderBinding::TextDecoderOptions,
    ) -> Fallible<DomRoot<TextDecoderStream>> {
        let encoding = match Encoding::for_label_no_replacement(&label.as_bytes()) {
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

    /// <https://encoding.spec.whatwg.org/#dom-textdecoder-encoding>
    fn Encoding(&self) -> DOMString {
        DOMString::from(self.decoder.encoding().name().to_ascii_lowercase())
    }

    /// <https://encoding.spec.whatwg.org/#dom-textdecoder-fatal>
    fn Fatal(&self) -> bool {
        self.decoder.fatal()
    }

    /// <https://encoding.spec.whatwg.org/#dom-textdecoder-ignorebom>
    fn IgnoreBOM(&self) -> bool {
        self.decoder.ignore_bom()
    }

    /// <https://streams.spec.whatwg.org/#dom-generictransformstream-readable>
    fn Readable(&self) -> DomRoot<<crate::DomTypeHolder as DomTypes>::ReadableStream> {
        self.transform.get_readable()
    }

    /// <https://streams.spec.whatwg.org/#dom-generictransformstream-writable>
    fn Writable(&self) -> DomRoot<<crate::DomTypeHolder as DomTypes>::WritableStream> {
        self.transform.get_writable()
    }
}
