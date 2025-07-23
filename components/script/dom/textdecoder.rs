/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::cell::Cell;

use dom_struct::dom_struct;
use encoding_rs::Encoding;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::TextDecoderBinding;
use crate::dom::bindings::codegen::Bindings::TextDecoderBinding::{
    TextDecodeOptions, TextDecoderMethods,
};
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::globalscope::GlobalScope;
use crate::dom::textdecodercommon::TextDecoderCommon;
use crate::script_runtime::CanGc;

/// <https://encoding.spec.whatwg.org/#textdecoder>
#[dom_struct]
#[allow(non_snake_case)]
pub(crate) struct TextDecoder {
    reflector_: Reflector,

    /// <https://encoding.spec.whatwg.org/#textdecodercommon>
    decoder: TextDecoderCommon,

    /// <https://encoding.spec.whatwg.org/#textdecoder-do-not-flush-flag>
    do_not_flush: Cell<bool>,
}

#[allow(non_snake_case)]
impl TextDecoder {
    fn new_inherited(encoding: &'static Encoding, fatal: bool, ignoreBOM: bool) -> TextDecoder {
        let decoder = TextDecoderCommon::new_inherited(encoding, fatal, ignoreBOM);
        TextDecoder {
            reflector_: Reflector::new(),
            decoder,
            do_not_flush: Cell::new(false),
        }
    }

    fn make_range_error() -> Fallible<DomRoot<TextDecoder>> {
        Err(Error::Range(
            "The given encoding is not supported.".to_owned(),
        ))
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        encoding: &'static Encoding,
        fatal: bool,
        ignoreBOM: bool,
        can_gc: CanGc,
    ) -> DomRoot<TextDecoder> {
        reflect_dom_object_with_proto(
            Box::new(TextDecoder::new_inherited(encoding, fatal, ignoreBOM)),
            global,
            proto,
            can_gc,
        )
    }
}

#[allow(non_snake_case)]
impl TextDecoderMethods<crate::DomTypeHolder> for TextDecoder {
    /// <https://encoding.spec.whatwg.org/#dom-textdecoder>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        label: DOMString,
        options: &TextDecoderBinding::TextDecoderOptions,
    ) -> Fallible<DomRoot<TextDecoder>> {
        let encoding = match Encoding::for_label_no_replacement(label.as_bytes()) {
            None => return TextDecoder::make_range_error(),
            Some(enc) => enc,
        };
        Ok(TextDecoder::new(
            global,
            proto,
            encoding,
            options.fatal,
            options.ignoreBOM,
            can_gc,
        ))
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

    /// <https://encoding.spec.whatwg.org/#dom-textdecoder-decode>
    fn Decode(
        &self,
        input: Option<ArrayBufferViewOrArrayBuffer>,
        options: &TextDecodeOptions,
    ) -> Fallible<USVString> {
        // Step 1. If this’s do not flush is false, then set this’s decoder to a new
        // instance of this’s encoding’s decoder, this’s I/O queue to the I/O queue
        // of bytes « end-of-queue », and this’s BOM seen to false.
        if !self.do_not_flush.get() {
            if self.decoder.ignore_bom() {
                self.decoder
                    .decoder()
                    .replace(self.decoder.encoding().new_decoder_without_bom_handling());
            } else {
                self.decoder
                    .decoder()
                    .replace(self.decoder.encoding().new_decoder());
            }
            self.decoder.io_queue().replace(Vec::new());
        }

        // Step 2. Set this’s do not flush to options["stream"].
        self.do_not_flush.set(options.stream);

        // Step 3. If input is given, then push a copy of input to this’s I/O queue.
        // Step 4. Let output be the I/O queue of scalar values « end-of-queue ».
        // Step 5. While true:
        // Step 5.1 Let item be the result of reading from this’s I/O queue.
        // Step 5.2 If item is end-of-queue and this’s do not flush is true,
        //      then return the result of running serialize I/O queue with this and output.
        // Step 5.3 Otherwise:
        // Step 5.3.1 Let result be the result of processing an item with item, this’s decoder,
        //      this’s I/O queue, output, and this’s error mode.
        // Step 5.3.2 If result is finished, then return the result of running serialize I/O
        //      queue with this and output.
        self.decoder
            .decode(input.as_ref(), !options.stream)
            .map(USVString)
    }
}
