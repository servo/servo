/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::JS_GetTwoByteStringCharsAndLength;
use js::jsval::UndefinedValue;
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue, ToString};

use crate::dom::bindings::codegen::Bindings::TextEncoderStreamBinding::TextEncoderStreamMethods;
use crate::dom::bindings::error::{Fallible, Error};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::{DomRoot, Dom};
use crate::dom::bindings::str::DOMString;
use crate::dom::transformstreamdefaultcontroller::TransformerType;
use crate::dom::types::{GlobalScope, TransformStream, TransformStreamDefaultController};
use crate::script_runtime::CanGc;
use crate::{DomTypeHolder, DomTypes};
use crate::script_runtime::{JSContext as SafeJSContext};

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct Encoder {
    leading_surrogate: Cell<Option<u16>>,
}

impl Encoder {
    fn new() -> Self {
        Self {
            leading_surrogate: Cell::new(None)
        }
    }
}

enum CodePointType {
    ScalarValue,
    LeadingSurrogate,
    TrailingSurrogate,
}

fn code_point_type(value: u16) -> CodePointType {
    match value {
        0xD800..=0xDBFF => CodePointType::LeadingSurrogate,
        0xDC00..=0xDFFF => CodePointType::TrailingSurrogate,
        _ => CodePointType::ScalarValue,
    }
}

#[allow(unsafe_code)]
pub(crate) fn encode_and_enqueue_a_chunk(
    cx: SafeJSContext,
    global: &GlobalScope,
    chunk: SafeHandleValue,
    encoder: &Encoder,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc
) -> Fallible<()> {
    // Step 1. Let input be the result of converting chunk to a DOMString. 
    // NOTE: DOMString is a wrapper over rust's String, which must be valid utf-8
    //      so we will need to inspect each u16
    let js_str = unsafe {std::ptr::NonNull::new(ToString(*cx, chunk))
        .ok_or_else(|| {
            log::error!("ToString failed");
            Error::JSFailed // ToString may set the exception
        })?};
    let mut len = 0;
    let data = unsafe {
        JS_GetTwoByteStringCharsAndLength(*cx, std::ptr::null(), js_str.as_ptr(), &mut len)
    };
    // Step 2. Convert input to an I/O queue of code units. 
    let maybe_ill_formed_code_units = unsafe { std::slice::from_raw_parts(data, len) };

    let mut output = String::with_capacity(len);
    for result in char::decode_utf16(maybe_ill_formed_code_units.iter().cloned()) {
        match result {
            Ok(c) => {
                if let Some(_leading_surrogate) = encoder.leading_surrogate.take() {
                    output.push('\u{FFFD}');
                }

                output.push(c);
            },
            Err(error) => {
                // output.push('\u{FFFD}');
                // encoder.unpaired_surrogate.replace(Some(error.unpaired_surrogate()));
                let unpaired_surrogate = error.unpaired_surrogate();
                match code_point_type(unpaired_surrogate) {
                    CodePointType::ScalarValue => unreachable!(),
                    CodePointType::LeadingSurrogate => {
                        if let Some(_leading_surrogate) = encoder.leading_surrogate.take() {
                            output.push('\u{FFFD}');
                        }

                        encoder.leading_surrogate.replace(Some(unpaired_surrogate));
                    },
                    CodePointType::TrailingSurrogate => match encoder.leading_surrogate.take() {
                        Some(leading_surrogate) => {
                            let c = char::decode_utf16([leading_surrogate, unpaired_surrogate]).next()
                                .expect("A pair of surrogate is supplied")
                                .expect("Decoding a pair of surrogate cannot fail");
                            output.push(c);
                        },
                        None => output.push('\u{FFFD}'),
                    },
                }
            },
        }
    }

    if output.is_empty() {
        return Ok(())
    }

    log::debug!("output: {:?}", output);

    let output = output.as_bytes();
    rooted!(in(*cx) let mut chunk = UndefinedValue());
    unsafe { output.to_jsval(*cx, chunk.handle_mut());}
    controller.enqueue(cx, global, chunk.handle(), can_gc)
}

#[allow(unsafe_code)]
pub(crate) fn encode_and_flush(
    cx: SafeJSContext,
    global: &GlobalScope,
    encoder: &Encoder,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc
) -> Fallible<()> {
    if encoder.leading_surrogate.get().is_some() {
        let output = [0xEF_u8, 0xBF, 0xBD];
        rooted!(in(*cx) let mut chunk = UndefinedValue());
        unsafe { output.to_jsval(*cx, chunk.handle_mut()); }
        return controller.enqueue(cx, global, chunk.handle(), can_gc)
    }

    Ok(())
}

/// <https://encoding.spec.whatwg.org/#textencoderstream>
#[dom_struct]
pub(crate) struct TextEncoderStream {
    reflector_: Reflector,

    #[ignore_malloc_size_of = "Rc is hard"]
    encoder: Rc<Encoder>,

    transform: Dom<TransformStream>,
}

impl TextEncoderStream {
    fn new_inherited(encoder: Rc<Encoder>, transform: &TransformStream) -> TextEncoderStream {
        Self {
            reflector_: Reflector::new(),
            encoder,
            transform: Dom::from_ref(transform)
        }
    }

    fn new_with_proto(
        cx: SafeJSContext,
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc
    ) -> Fallible<DomRoot<TextEncoderStream>> {
        let encoder = Rc::new(Encoder::new());
        let transformer_type = TransformerType::Encoder(encoder.clone());

        let transform = TransformStream::new_with_proto(global, None, can_gc);
        transform.set_up(cx, global, transformer_type, can_gc)?;

        Ok(reflect_dom_object_with_proto(Box::new(TextEncoderStream::new_inherited(encoder, &transform)), global, proto, can_gc))
    }
}

#[allow(non_snake_case)]
impl TextEncoderStreamMethods<DomTypeHolder> for TextEncoderStream {
    /// <https://encoding.spec.whatwg.org/#dom-textencoderstream>
    fn Constructor(
        global: &<DomTypeHolder as DomTypes>::GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<<DomTypeHolder as DomTypes>::TextEncoderStream>> {
        TextEncoderStream::new_with_proto(GlobalScope::get_cx(), global, proto, can_gc)
    }

    /// <https://encoding.spec.whatwg.org/#dom-textencoder-encoding>
    fn Encoding(&self) -> DOMString {
        DOMString::from("utf-8")
    }

    /// <https://streams.spec.whatwg.org/#dom-generictransformstream-readable>
    fn Readable(&self) -> DomRoot<<DomTypeHolder as script_bindings::DomTypes>::ReadableStream> {
        self.transform.get_readable()
    }

    /// <https://streams.spec.whatwg.org/#dom-generictransformstream-writable>
    fn Writable(&self) -> DomRoot<<DomTypeHolder as DomTypes>::WritableStream> {
        self.transform.get_writable()
    }
}
