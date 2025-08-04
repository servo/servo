/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::Cell;
use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::conversions::{jsstr_to_string, ConversionResult, FromJSValConvertible, ToJSValConvertible};
use js::jsapi::{HandleValueArray, JSObject, JS_GetTwoByteStringCharsAndLength, JS_IsTypedArrayObject, JS_ValueIsUndefined};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::wrappers::ToPrimitive;
use js::typedarray::{TypedArray, Uint8Array};
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue, ToBoolean, ToInt32, ToNumber, ToString, IntoHandle};

use crate::dom::bindings::buffer_source::create_buffer_source;
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

enum StrOrCodeUnits<'a> {
    Str(&'a str),
    String(String),
    Array(Vec<String>),
    CodeUnits(&'a [u16]),
}

#[allow(unsafe_code)]
fn convert_chunk_to_dom_string<'chunk>(
    cx: SafeJSContext,
    chunk: &'chunk SafeHandleValue,
) -> Fallible<StrOrCodeUnits<'chunk>> {
    log::debug!("chunk.bits: {:?}", chunk.asBits_);

    if chunk.is_undefined() {
        log::debug!("chunk is undefined");
        return Ok(StrOrCodeUnits::Str("undefined"))
    } 
    
    if chunk.is_null() {
        log::debug!("chunk is null");
        return Ok(StrOrCodeUnits::Str("null"))
    } 
    
    if chunk.is_boolean() {
        log::debug!("chunk is boolean");
        return Ok(StrOrCodeUnits::String(chunk.to_boolean().to_string()))
    } 
    
    if chunk.is_int32() {
        log::debug!("chunk is i32");
        return Ok(StrOrCodeUnits::String(chunk.to_int32().to_string()))
    } 
    
    if chunk.is_double() {
        log::debug!("chunk is number");
        return Ok(StrOrCodeUnits::String(chunk.to_double().to_string()))
    }

    if let Ok(result) = unsafe { Vec::<JSVal>::from_jsval(*cx, *chunk, ()) } {
        log::debug!("try converting to vec");
        log::debug!("conversion_result: {:?}", result);

        if let ConversionResult::Success(vals) = result {
            let arr = vals.into_iter().map(|val| {
                let jsstr = val.to_string();
                unsafe { jsstr_to_string(*cx, jsstr) }
            }).collect();
            return Ok(StrOrCodeUnits::Array(arr))
        }
    }

    if chunk.is_object() {
        log::debug!("chunk is object");

        let obj = chunk.to_object();
        dbg!(&obj);

        // // return Ok(StrOrCodeUnits::Str("[object Object]"))
        // let obj = chunk.to_object();
        // let handle = unsafe { SafeHandleObject::from_marked_location(&obj as *const *mut _) };

        // rooted!(in(*cx) let mut value = UndefinedValue());
        // let _result = unsafe { ToPrimitive(*cx, handle, js::jsapi::JSType::JSTYPE_STRING, value.handle_mut()) };
        // log::debug!("ToPrimitive result: {:?}", _result);

        // let jsstr = value.to_string();
        // let s = unsafe { jsstr_to_string(*cx, jsstr) };
        // return Ok(StrOrCodeUnits::String(s))
    }
    
    // Step 1. Let input be the result of converting chunk to a DOMString. 
    // NOTE: servo's DOMString type is a wrapper over rust's String, which must be 
    //      valid utf-8 so we will need to inspect each u16
    let js_str = unsafe {std::ptr::NonNull::new(ToString(*cx, *chunk))
        .ok_or_else(|| {
            log::error!("ToString failed");
            Error::JSFailed // ToString may set the exception
        })?};
    // Step 2. Convert input to an I/O queue of code units. 
    let mut len = 0;
    let data = unsafe {
        JS_GetTwoByteStringCharsAndLength(*cx, std::ptr::null(), js_str.as_ptr(), &mut len)
    };
    let maybe_ill_formed_code_units = unsafe { std::slice::from_raw_parts(data, len) };

    log::debug!("code_units: {:?}", maybe_ill_formed_code_units);

    Ok(StrOrCodeUnits::CodeUnits(maybe_ill_formed_code_units))
}

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

    #[allow(unsafe_code)]
    fn encode_code_units(&self, code_units: &[u16]) -> String {
        let mut output = String::with_capacity(code_units.len());
        for result in char::decode_utf16(code_units.iter().cloned()) {
            match result {
                Ok(c) => {
                    if let Some(_leading_surrogate) = self.leading_surrogate.take() {
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
                            if let Some(_leading_surrogate) = self.leading_surrogate.take() {
                                output.push('\u{FFFD}');
                            }

                            self.leading_surrogate.replace(Some(unpaired_surrogate));
                        },
                        CodePointType::TrailingSurrogate => match self.leading_surrogate.take() {
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

        output
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

/// <https://encoding.spec.whatwg.org/#encode-and-enqueue-a-chunk>
#[allow(unsafe_code)]
pub(crate) fn encode_and_enqueue_a_chunk(
    cx: SafeJSContext,
    global: &GlobalScope,
    chunk: SafeHandleValue,
    encoder: &Encoder,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc
) -> Fallible<()> {
    let input = convert_chunk_to_dom_string(cx, &chunk)?;

    let output = match input {
        StrOrCodeUnits::Str(s) => Cow::Borrowed(s),
        StrOrCodeUnits::String(s) => Cow::Owned(s),
        StrOrCodeUnits::Array(items) => {
            let s: String = items.into_iter().collect();
            Cow::Owned(s)
        },
        StrOrCodeUnits::CodeUnits(code_units) => {
                        Cow::Owned(encoder.encode_code_units(code_units))
            },
    };

    if output.is_empty() {
        return Ok(())
    }

    log::debug!("output: {:?}", output);

    let output = output.as_bytes();
    rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
    let chunk: Uint8Array = create_buffer_source(cx, output, js_object.handle_mut(), can_gc)
        .map_err(|_| Error::Type("Cannot convert byte sequence to Uint8Array".to_owned()))?;
    rooted!(in(*cx) let mut chunk_val = UndefinedValue());
    unsafe { chunk.to_jsval(*cx, chunk_val.handle_mut());}
    controller.enqueue(cx, global, chunk_val.handle(), can_gc)
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
