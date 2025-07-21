/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;

use dom_struct::dom_struct;
use encoding_rs::Encoding;
use js::rust::HandleObject;

use crate::dom::bindings::buffer_source::HeapBufferSource;
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
}

#[allow(non_snake_case)]
impl TextDecoder {
    fn new_inherited(encoding: &'static Encoding, fatal: bool, ignoreBOM: bool) -> TextDecoder {
        let decoder = TextDecoderCommon::new_inherited(encoding, fatal, ignoreBOM);
        TextDecoder {
            reflector_: Reflector::new(),
            decoder,
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
        self.decoder.encoding()
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
        let cx = GlobalScope::get_cx();
        let input =
            input.map(|value| HeapBufferSource::from_array_buffer_view_or_array_buffer(cx, &value));
        self.decoder
            .decode(cx, input.as_ref(), options.stream)
            .map(USVString)
    }
}
