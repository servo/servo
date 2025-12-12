/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::num::{NonZero, NonZeroU16};
use std::ptr::{self, NonNull};

use dom_struct::dom_struct;
use js::conversions::latin1_to_string;
use js::jsapi::{
    JS_DeprecatedStringHasLatin1Chars, JS_GetTwoByteStringCharsAndLength, JS_IsExceptionPending,
    JSObject, JSType, ToPrimitive,
};
use js::jsval::UndefinedValue;
use js::rust::{
    HandleObject as SafeHandleObject, HandleValue as SafeHandleValue,
    MutableHandleValue as SafeMutableHandleValue, ToString,
};
use js::typedarray::Uint8;
use script_bindings::conversions::SafeToJSValConvertible;

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::TextEncoderStreamBinding::TextEncoderStreamMethods;
use crate::dom::bindings::error::{Error, Fallible, throw_dom_exception};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::transformstreamdefaultcontroller::TransformerType;
use crate::dom::types::{GlobalScope, TransformStream, TransformStreamDefaultController};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::{DomTypeHolder, DomTypes};

/// String converted from an input JS Value
enum ConvertedInput<'a> {
    String(String),
    CodeUnits(&'a [u16]),
}

/// Converts a JS value to primitive type so that it can be used with
/// `ToString`.
///
/// Set `rval` to `chunk` if `chunk` is a primitive JS value. Otherwise, convert
/// `chunk` into a primitive JS value and then set `rval` to the converted
/// primitive. This follows the `ToString` procedure with the exception that it
/// does not convert the value to string.
///
/// See below for the `ToString` procedure in spec:
/// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tostring>
#[expect(unsafe_code)]
fn jsval_to_primitive(
    cx: SafeJSContext,
    global: &GlobalScope,
    chunk: SafeHandleValue,
    mut rval: SafeMutableHandleValue,
    can_gc: CanGc,
) -> Fallible<()> {
    // Step 1. If argument is a String, return argument.
    // Step 2. If argument is a Symbol, throw a TypeError exception.
    // Step 3. If argument is undefined, return "undefined".
    // Step 4. If argument is null, return "null".
    // Step 5. If argument is true, return "true".
    // Step 6. If argument is false, return "false".
    // Step 7. If argument is a Number, return Number::toString(argument, 10).
    // Step 8. If argument is a BigInt, return BigInt::toString(argument, 10).
    if chunk.is_primitive() {
        rval.set(chunk.get());

        return Ok(());
    }

    // Step 9. Assert: argument is an Object.
    assert!(chunk.is_object());

    // Step 10. Let primValue be ? ToPrimitive(argument, string).
    rooted!(in(*cx) let obj = chunk.to_object());
    let is_success =
        unsafe { ToPrimitive(*cx, obj.handle().into(), JSType::JSTYPE_STRING, rval.into()) };
    log::debug!("ToPrimitive is_success={:?}", is_success);
    if !is_success {
        unsafe {
            if !JS_IsExceptionPending(*cx) {
                throw_dom_exception(
                    cx,
                    global,
                    Error::Type("Cannot convert JSObject to primitive".to_owned()),
                    can_gc,
                );
            }
        }
        return Err(Error::JSFailed);
    }

    Ok(())
}

/// <https://encoding.spec.whatwg.org/#textencoderstream-encoder>
#[derive(Default, JSTraceable, MallocSizeOf)]
pub(crate) struct Encoder {
    /// <https://encoding.spec.whatwg.org/#textencoderstream-pending-high-surrogate>
    leading_surrogate: Cell<Option<NonZeroU16>>,
}

impl Encoder {
    fn encode(&self, maybe_ill_formed: ConvertedInput<'_>) -> String {
        match maybe_ill_formed {
            ConvertedInput::String(s) => {
                // Rust String is already UTF-8 encoded and cannot contain
                // surrogate
                if !s.is_empty() && self.leading_surrogate.take().is_some() {
                    let mut output = String::with_capacity(1 + s.len());
                    output.push('\u{FFFD}');
                    output.push_str(&s);
                    return output;
                }

                s
            },
            ConvertedInput::CodeUnits(code_units) => self.encode_from_code_units(code_units),
        }
    }

    /// Encode an input slice of code unit into unicode scalar values
    fn encode_from_code_units(&self, input: &[u16]) -> String {
        // <https://encoding.spec.whatwg.org/#encode-and-enqueue-a-chunk>
        //
        // Step 3. Let output be the I/O queue of bytes « end-of-queue ».
        let mut output = String::with_capacity(input.len());
        // Step 4. While true:
        // Step 4.1 Let item be the result of reading from input.
        for result in char::decode_utf16(input.iter().cloned()) {
            // Step 4.3 Let result be the result of executing the convert code unit
            //      to scalar value algorithm with encoder, item and input.

            // <https://encoding.spec.whatwg.org/#convert-code-unit-to-scalar-value>
            match result {
                Ok(c) => {
                    // Step 1. If encoder’s leading surrogate is non-null:
                    // Step 1.1 Let leadingSurrogate be encoder’s leading surrogate.
                    // Step 1.2 Set encoder’s leading surrogate to null.
                    if self.leading_surrogate.take().is_some() {
                        // Step 1.5 Return U+FFFD (�).
                        output.push('\u{FFFD}');
                    }

                    // Step 1.4 Restore item to input.
                    // Note: pushing item to output is equivalent to restoring item to input
                    //      and rerun the convert-code-unit-to-scalar-value algo
                    output.push(c);
                },
                Err(error) => {
                    let unpaired_surrogate = error.unpaired_surrogate();
                    match code_point_type(unpaired_surrogate) {
                        CodePointType::LeadingSurrogate => {
                            // Step 1.1 If encoder’s leading surrogate is non-null:
                            // Step 1.2 Set encoder’s leading surrogate to null.
                            if self.leading_surrogate.take().is_some() {
                                output.push('\u{FFFD}');
                            }

                            // Step 1.4 Restore item to input.
                            // Note: Replacing encoder's leading_surrogate is equivalent
                            //      to restore item back to input and rerun the convert-
                            //      code-unit-to-scalar-value algo.
                            // Step 2. If item is a leading surrogate, then set encoder’s
                            //      leading surrogate to item and return continue.
                            self.leading_surrogate
                                .replace(NonZero::new(unpaired_surrogate));
                        },
                        CodePointType::TrailingSurrogate => match self.leading_surrogate.take() {
                            // Step 1.1 If encoder’s leading surrogate is non-null:
                            // Step 1.2 Set encoder’s leading surrogate to null.
                            Some(leading_surrogate) => {
                                // Step 1.3 If item is a trailing surrogate, then return a scalar
                                //      value from surrogates given leadingSurrogate and item.
                                let c = char::decode_utf16([
                                    leading_surrogate.get(),
                                    unpaired_surrogate,
                                ])
                                .next()
                                .expect("A pair of surrogate is supplied")
                                .expect("Decoding a pair of surrogate cannot fail");
                                output.push(c);
                            },
                            // Step 3. If item is a trailing surrogate, then return U+FFFD (�).
                            None => output.push('\u{FFFD}'),
                        },
                        CodePointType::ScalarValue => unreachable!("Scalar Value won't fail"),
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
#[expect(unsafe_code)]
pub(crate) fn encode_and_enqueue_a_chunk(
    cx: SafeJSContext,
    global: &GlobalScope,
    chunk: SafeHandleValue,
    encoder: &Encoder,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc,
) -> Fallible<()> {
    // Step 1. Let input be the result of converting chunk to a DOMString.
    // Step 2. Convert input to an I/O queue of code units.
    rooted!(in(*cx) let mut rval = UndefinedValue());
    jsval_to_primitive(cx, global, chunk, rval.handle_mut(), can_gc)?;

    assert!(!rval.is_object());
    rooted!(in(*cx) let jsstr = unsafe { ToString(*cx, rval.handle()) });
    if jsstr.is_null() {
        unsafe {
            if !JS_IsExceptionPending(*cx) {
                throw_dom_exception(
                    cx,
                    global,
                    Error::Type("Cannot convert JS primitive to string".to_owned()),
                    can_gc,
                );
            }
        }

        return Err(Error::JSFailed);
    }

    let input = unsafe {
        if JS_DeprecatedStringHasLatin1Chars(*jsstr) {
            let s = NonNull::new(*jsstr).expect("jsstr cannot be null");
            ConvertedInput::String(latin1_to_string(*cx, s))
        } else {
            let mut len = 0;
            let data = JS_GetTwoByteStringCharsAndLength(*cx, std::ptr::null(), *jsstr, &mut len);
            let maybe_ill_formed_code_units = std::slice::from_raw_parts(data, len);
            ConvertedInput::CodeUnits(maybe_ill_formed_code_units)
        }
    };

    // Step 3. Let output be the I/O queue of bytes « end-of-queue ».
    // Step 4. While true:
    // Step 4.1 Let item be the result of reading from input.
    // Step 4.3 Let result be the result of executing the convert code unit
    //      to scalar value algorithm with encoder, item and input.
    // Step 4.4 If result is not continue, then process an item with result,
    //      encoder’s encoder, input, output, and "fatal".
    let output = encoder.encode(input);

    // Step 4.2 If item is end-of-queue:
    // Step 4.2.1 Convert output into a byte sequence.
    let output = output.as_bytes();
    // Step 4.2.2 If output is not empty:
    if output.is_empty() {
        // Step 4.2.3
        return Ok(());
    }

    // Step 4.2.2.1 Let chunk be the result of creating a Uint8Array object
    //      given output and encoder’s relevant realm.
    rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
    let chunk = create_buffer_source::<Uint8>(cx, output, js_object.handle_mut(), can_gc)
        .map_err(|_| Error::Type("Cannot convert byte sequence to Uint8Array".to_owned()))?;
    rooted!(in(*cx) let mut rval = UndefinedValue());
    chunk.safe_to_jsval(cx, rval.handle_mut(), can_gc);
    // Step 4.2.2.2 Enqueue chunk into encoder’s transform.
    controller.enqueue(cx, global, rval.handle(), can_gc)?;
    Ok(())
}

/// <https://encoding.spec.whatwg.org/#encode-and-flush>
pub(crate) fn encode_and_flush(
    cx: SafeJSContext,
    global: &GlobalScope,
    encoder: &Encoder,
    controller: &TransformStreamDefaultController,
    can_gc: CanGc,
) -> Fallible<()> {
    // Step 1. If encoder’s leading surrogate is non-null:
    if encoder.leading_surrogate.get().is_some() {
        // Step 1.1 Let chunk be the result of creating a Uint8Array object
        //      given « 0xEF, 0xBF, 0xBD » and encoder’s relevant realm.
        rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
        let chunk = create_buffer_source::<Uint8>(
            cx,
            &[0xEF_u8, 0xBF, 0xBD],
            js_object.handle_mut(),
            can_gc,
        )
        .map_err(|_| Error::Type("Cannot convert byte sequence to Uint8Array".to_owned()))?;
        rooted!(in(*cx) let mut rval = UndefinedValue());
        chunk.safe_to_jsval(cx, rval.handle_mut(), can_gc);
        // Step 1.2 Enqueue chunk into encoder’s transform.
        return controller.enqueue(cx, global, rval.handle(), can_gc);
    }

    Ok(())
}

/// <https://encoding.spec.whatwg.org/#textencoderstream>
#[dom_struct]
pub(crate) struct TextEncoderStream {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#generictransformstream>
    transform: Dom<TransformStream>,
}

impl TextEncoderStream {
    fn new_inherited(transform: &TransformStream) -> TextEncoderStream {
        Self {
            reflector_: Reflector::new(),
            transform: Dom::from_ref(transform),
        }
    }

    /// <https://encoding.spec.whatwg.org/#dom-textencoderstream>
    fn new_with_proto(
        cx: SafeJSContext,
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<TextEncoderStream>> {
        // Step 1. Set this’s encoder to an instance of the UTF-8 encoder.
        let encoder = Encoder::default();

        // Step 2. Let transformAlgorithm be an algorithm which takes a chunk argument
        //      and runs the encode and enqueue a chunk algorithm with this and chunk.
        // Step 3. Let flushAlgorithm be an algorithm which runs the encode and flush
        //      algorithm with this.
        let transformer_type = TransformerType::Encoder(encoder);

        // Step 4. Let transformStream be a new TransformStream.
        let transform = TransformStream::new_with_proto(global, None, can_gc);
        // Step 5. Set up transformStream with transformAlgorithm set to transformAlgorithm
        //      and flushAlgorithm set to flushAlgorithm.
        transform.set_up(cx, global, transformer_type, can_gc)?;

        // Step 6. Set this’s transform to transformStream.
        Ok(reflect_dom_object_with_proto(
            Box::new(TextEncoderStream::new_inherited(&transform)),
            global,
            proto,
            can_gc,
        ))
    }
}

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
        // Returns "utf-8".
        DOMString::from("utf-8")
    }

    /// <https://streams.spec.whatwg.org/#dom-generictransformstream-readable>
    fn Readable(&self) -> DomRoot<<DomTypeHolder as DomTypes>::ReadableStream> {
        self.transform.get_readable()
    }

    /// <https://streams.spec.whatwg.org/#dom-generictransformstream-writable>
    fn Writable(&self) -> DomRoot<<DomTypeHolder as DomTypes>::WritableStream> {
        self.transform.get_writable()
    }
}
