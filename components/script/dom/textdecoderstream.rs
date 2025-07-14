/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use encoding_rs::{Decoder, Encoding};
use js::conversions::{FromJSValConvertible, ToJSValConvertible};
use js::jsapi::JSContext;
use js::jsval::UndefinedValue;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue};
use script_bindings::codegen::GenericBindings::TransformStreamDefaultControllerBinding::TransformStreamDefaultControllerMethods;

use crate::dom::bindings::codegen::UnionTypes::ArrayBufferOrArrayBufferView;
use crate::dom::types::{TransformStream, TransformStreamDefaultController};
use crate::DomTypes;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::codegen::Bindings::TextDecoderBinding;
use crate::dom::bindings::codegen::Bindings::TextDecoderStreamBinding::TextDecoderStreamMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

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
    transform_stream: Dom<TransformStream>,
}

#[allow(non_snake_case)]
impl TextDecoderStream {
    fn new_inherited(encoding: &'static Encoding, fatal: bool, ignoreBOM: bool, transform_stream: &TransformStream) -> Self {
        let decoder = if ignoreBOM {
            encoding.new_decoder()
        } else {
            encoding.new_decoder_without_bom_handling()
        };

        Self {
            reflector_: Reflector::new(),
            encoding,
            fatal,
            ignoreBOM,
            decoder: RefCell::new(decoder),
            in_stream: RefCell::new(Vec::new()),
            transform_stream: Dom::from_ref(transform_stream)
        }
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        encoding: &'static Encoding,
        fatal: bool,
        ignoreBOM: bool,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        // The new TextDecoderStream(label, options) constructor steps are: 
        // Step 9. Let transformStream be a new TransformStream. 
        let transform_stream = TransformStream::new_with_proto(global, None, can_gc);
        reflect_dom_object_with_proto(
            Box::new(Self::new_inherited(encoding, fatal, ignoreBOM, &transform_stream)),
            global,
            proto,
            can_gc,
        )
    }

    // https://encoding.spec.whatwg.org/#decode-and-enqueue-a-chunk
    #[allow(unsafe_code)]
    fn decode_and_enqueue_a_chunk(
        &self, 
        cx: SafeJSContext,
        chunk: SafeHandleValue, 
        controller: &TransformStreamDefaultController,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Step 1. Let bufferSource be the result of converting chunk to an AllowSharedBufferSource. 
        let conversion_result = unsafe {
            ArrayBufferOrArrayBufferView::from_jsval(*cx, chunk, ())
                .map_err(|_| Error::Type("Unable to convert chunk into ArrayBuffer or ArrayBufferView".to_string()))?
        };
        let buffer_source = conversion_result.get_success_value()
            .ok_or(Error::Type("Unable to convert chunk into ArrayBuffer or ArrayBufferView".to_string()))?;

        // Step 2. Push a copy of bufferSource to decoder’s I/O queue. 
        unsafe {
            match buffer_source {
                ArrayBufferOrArrayBufferView::ArrayBuffer(a) => self.in_stream.borrow_mut().extend_from_slice(a.as_slice()),
                ArrayBufferOrArrayBufferView::ArrayBufferView(a) => self.in_stream.borrow_mut().extend_from_slice(a.as_slice()),
            }
        }

        // Step 3. Let output be the I/O queue of scalar values « end-of-queue ». 
        let mut decoder = self.decoder.borrow_mut();
        let mut in_stream = self.in_stream.borrow_mut();
        let mut output_chunk = String::with_capacity(
            decoder.max_utf8_buffer_length(in_stream.len())
                .expect("failed to calculate MAX UTF8 buffer length")
        );

        // Step 4.
        let (_result, read, _replaced) = decoder.decode_to_string(&in_stream, &mut output_chunk, false);
        let mut remaining = in_stream.split_off(read);
        std::mem::swap(&mut* in_stream, &mut remaining);

        rooted!(in(*cx) let mut rval = UndefinedValue());
        unsafe { output_chunk.to_jsval(*cx, rval.handle_mut()) };

        controller.Enqueue(cx, rval.handle(), can_gc)?;

        Ok(())
    }

    // https://encoding.spec.whatwg.org/#flush-and-enqueue
    fn flush_and_enqueue() {
        todo!()
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

        Ok(Self::new_with_proto(
            global,
            proto,
            encoding,
            options.fatal,
            options.ignoreBOM,
            can_gc,
        ))
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
        todo!()
    }

    fn Writable(&self) -> DomRoot<<crate::DomTypeHolder as DomTypes>::WritableStream> {
        todo!()
    }
}
