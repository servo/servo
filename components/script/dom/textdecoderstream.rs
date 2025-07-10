/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use encoding_rs::{Decoder, Encoding};
use js::rust::HandleObject;

use crate::dom::types::TransformStream;
use crate::DomTypes;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::codegen::Bindings::TextDecoderBinding;
use crate::dom::bindings::codegen::Bindings::TextDecoderStreamBinding::TextDecoderStreamMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

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
            transform_stream: Dom::from_ref(transform_stream)
        }
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
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
    fn decode_and_enqueue_a_chunk(&self, chunk: &[u8]) {
        todo!()
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
        proto: Option<HandleObject>,
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
