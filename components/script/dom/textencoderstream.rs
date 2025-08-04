/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::Cell;
use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::conversions::{ConversionResult, FromJSValConvertible, ToJSValConvertible};
use js::jsapi::{
    IsArrayObject, JS_DeprecatedStringHasLatin1Chars, JS_GetLatin1StringCharsAndLength,
    JS_GetTwoByteStringCharsAndLength, JSObject, JSType, ToPrimitive,
};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue, IntoHandle};
use js::typedarray::Uint8Array;

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::TextEncoderStreamBinding::TextEncoderStreamMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::transformstreamdefaultcontroller::TransformerType;
use crate::dom::types::{GlobalScope, TransformStream, TransformStreamDefaultController};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::{DomTypeHolder, DomTypes};

enum MaybeIllFormed<'a> {
    Str(&'static str),
    Number(f64),
    Latin1(&'a [u8]),
    CodeUnits(&'a [u16]),
}

/// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tostring>
#[allow(unsafe_code)]
fn jsval_to_string<'a>(
    cx: SafeJSContext,
    value: &JSVal, // TODO: how to argure the lifetime here
) -> Fallible<MaybeIllFormed<'a>> {
    // Step 1. If argument is a String, return argument.
    if value.is_string() {
        let jsstr = std::ptr::NonNull::new(value.to_string()).ok_or_else(|| {
            log::error!("ToString failed");
            Error::JSFailed // ToString may set the exception
        })?;
        unsafe {
            if JS_DeprecatedStringHasLatin1Chars(jsstr.as_ptr()) {
                // return Ok(MaybeIllFormed::String(latin1_to_string(*cx, jsstr.as_ptr())))
                let mut len = 0;
                let chars =
                    JS_GetLatin1StringCharsAndLength(*cx, ptr::null(), jsstr.as_ptr(), &mut len);
                let chars = std::slice::from_raw_parts(chars, len);
                return Ok(MaybeIllFormed::Latin1(chars));
            }
        }
        // Step 2. Convert input to an I/O queue of code units.
        let mut len = 0;
        let data = unsafe {
            JS_GetTwoByteStringCharsAndLength(*cx, std::ptr::null(), jsstr.as_ptr(), &mut len)
        };
        let maybe_ill_formed_code_units = unsafe { std::slice::from_raw_parts(data, len) };
        log::debug!("maybe ill formed: {:?}", maybe_ill_formed_code_units);
        return Ok(MaybeIllFormed::CodeUnits(maybe_ill_formed_code_units));
    }

    // Step 2. If argument is a Symbol, throw a TypeError exception.
    if value.is_symbol() {
        return Err(Error::Type("Cannot convert symbol to string".to_owned()));
    }

    // Step 3. If argument is undefined, return "undefined".
    if value.is_undefined() {
        return Ok(MaybeIllFormed::Str("undefined"));
    }

    // Step 4. If argument is null, return "null".
    if value.is_null() {
        return Ok(MaybeIllFormed::Str("null"));
    }

    // Step 5. If argument is true, return "true".
    // Step 6. If argument is false, return "false".
    if value.is_boolean() {
        let s = match value.to_boolean() {
            true => "true",
            false => "false",
        };
        return Ok(MaybeIllFormed::Str(s));
    }

    // Step 7. If argument is a Number, return Number::toString(argument, 10).
    if value.is_number() {
        return Ok(MaybeIllFormed::Number(value.to_number()));
    }

    // Step 8. If argument is a BigInt, return BigInt::toString(argument, 10).
    // TODO

    // Step 9. Assert: argument is an Object.
    assert!(value.is_object());

    // Step 10. Let primValue be ? ToPrimitive(argument, string).
    rooted!(in(*cx) let mut prim_value = UndefinedValue());
    rooted!(in(*cx) let obj = value.to_object());
    let is_success = unsafe {
        ToPrimitive(
            *cx,
            obj.handle().into_handle(),
            JSType::JSTYPE_STRING,
            prim_value.handle_mut().into(),
        )
    };
    if !is_success {
        return Err(Error::JSFailed); // TODO: double check if an error is thrown
    }

    // Step 11. Assert: primValue is not an Object.
    assert!(!prim_value.is_object());

    // Step 12. Return ? ToString(primValue).
    jsval_to_string(cx, &prim_value.handle())
}

/// <https://encoding.spec.whatwg.org/#textencoderstream-encoder>
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct Encoder {
    /// <https://encoding.spec.whatwg.org/#textencoderstream-pending-high-surrogate>
    leading_surrogate: Cell<Option<u16>>,
}

impl Encoder {
    fn new() -> Self {
        Self {
            leading_surrogate: Cell::new(None),
        }
    }

    fn encode<'a>(&self, maybe_ill_formed: MaybeIllFormed<'a>) -> Cow<'a, str> {
        match maybe_ill_formed {
            MaybeIllFormed::Str(s) => {
                if let Some(_leading_surrogate) = self.leading_surrogate.take() {
                    let mut output = String::with_capacity(1 + s.len());
                    output.push('\u{FFFD}');
                    output.push_str(s);
                    Cow::Owned(output)
                } else {
                    Cow::Borrowed(s)
                }
            },
            MaybeIllFormed::Latin1(bytes) => Cow::Owned(self.encode_latin1(bytes)),
            MaybeIllFormed::Number(num) => {
                // We have a number so the input must not be empty
                let s = num.to_string();
                if let Some(_leading_surrogate) = self.leading_surrogate.take() {
                    let mut output = String::with_capacity(1 + s.len());
                    output.push('\u{FFFD}');
                    output.push_str(&s);
                    Cow::Owned(output)
                } else {
                    Cow::Owned(s)
                }
            },
            MaybeIllFormed::CodeUnits(code_units) => Cow::Owned(self.encode_code_units(code_units)),
        }
    }

    fn encode_latin1(&self, bytes: &[u8]) -> String {
        let mut output = String::with_capacity(bytes.len());
        for &b in bytes {
            if let Some(_leading_surrogate) = self.leading_surrogate.take() {
                output.push('\u{FFFD}');
            }

            output.push(b as char);
        }
        output
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
                                let c = char::decode_utf16([leading_surrogate, unpaired_surrogate])
                                    .next()
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
    can_gc: CanGc,
) -> Fallible<()> {
    let mut is_array = false;
    unsafe { IsArrayObject(*cx, chunk.into_handle(), &mut is_array) };
    let output = if is_array {
        match unsafe {
            Vec::<JSVal>::from_jsval(*cx, chunk, ())
                .map_err(|_| Error::Type("Failed to convert array".to_owned()))?
        } {
            ConversionResult::Success(arr) => {
                let s = arr
                    .into_iter()
                    .map(|val| {
                        let maybe_ill_formed = jsval_to_string(cx, &val)?;
                        Ok(encoder.encode(maybe_ill_formed))
                    })
                    .collect::<Fallible<String>>()?;
                Cow::Owned(s)
            },
            ConversionResult::Failure(failure) => return Err(Error::Type(failure.to_string())),
        }
    } else {
        let maybe_ill_formed = jsval_to_string(cx, &chunk)?;
        encoder.encode(maybe_ill_formed)
    };

    if output.is_empty() {
        return Ok(());
    }

    log::debug!("output: {:?}", output);

    let output = output.as_bytes();
    rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
    let chunk: Uint8Array = create_buffer_source(cx, output, js_object.handle_mut(), can_gc)
        .map_err(|_| Error::Type("Cannot convert byte sequence to Uint8Array".to_owned()))?;
    rooted!(in(*cx) let mut chunk_val = UndefinedValue());
    unsafe {
        chunk.to_jsval(*cx, chunk_val.handle_mut());
    }
    controller.enqueue(cx, global, chunk_val.handle(), can_gc)
}

#[allow(unsafe_code)]
pub(crate) fn encode_and_flush(
    cx: SafeJSContext,
    global: &GlobalScope,
    encoder: &Encoder,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc,
) -> Fallible<()> {
    if encoder.leading_surrogate.get().is_some() {
        let output = [0xEF_u8, 0xBF, 0xBD];
        rooted!(in(*cx) let mut chunk = UndefinedValue());
        unsafe {
            output.to_jsval(*cx, chunk.handle_mut());
        }
        return controller.enqueue(cx, global, chunk.handle(), can_gc);
    }

    Ok(())
}

/// <https://encoding.spec.whatwg.org/#textencoderstream>
#[dom_struct]
pub(crate) struct TextEncoderStream {
    reflector_: Reflector,

    /// <https://encoding.spec.whatwg.org/#textencoderstream-encoder>
    #[ignore_malloc_size_of = "Rc is hard"]
    encoder: Rc<Encoder>,

    transform: Dom<TransformStream>,
}

impl TextEncoderStream {
    fn new_inherited(encoder: Rc<Encoder>, transform: &TransformStream) -> TextEncoderStream {
        Self {
            reflector_: Reflector::new(),
            encoder,
            transform: Dom::from_ref(transform),
        }
    }

    fn new_with_proto(
        cx: SafeJSContext,
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<TextEncoderStream>> {
        let encoder = Rc::new(Encoder::new());
        let transformer_type = TransformerType::Encoder(encoder.clone());

        let transform = TransformStream::new_with_proto(global, None, can_gc);
        transform.set_up(cx, global, transformer_type, can_gc)?;

        Ok(reflect_dom_object_with_proto(
            Box::new(TextEncoderStream::new_inherited(encoder, &transform)),
            global,
            proto,
            can_gc,
        ))
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
