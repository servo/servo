/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use dom_struct::dom_struct;
use js::gc::CustomAutoRooterGuard;
use js::jsapi::JSObject;
use js::rust::HandleObject;
use js::typedarray;
use js::typedarray::HeapUint8Array;
use script_bindings::trace::RootedTraceableBox;

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::TextEncoderBinding::{
    TextEncoderEncodeIntoResult, TextEncoderMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct TextEncoder {
    reflector_: Reflector,
}

impl TextEncoder {
    fn new_inherited() -> TextEncoder {
        TextEncoder {
            reflector_: Reflector::new(),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<TextEncoder> {
        reflect_dom_object_with_proto(
            Box::new(TextEncoder::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }
}

impl TextEncoderMethods<crate::DomTypeHolder> for TextEncoder {
    /// <https://encoding.spec.whatwg.org/#dom-textencoder>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<TextEncoder>> {
        Ok(TextEncoder::new(global, proto, can_gc))
    }

    /// <https://encoding.spec.whatwg.org/#dom-textencoder-encoding>
    fn Encoding(&self) -> DOMString {
        DOMString::from("utf-8")
    }

    /// <https://encoding.spec.whatwg.org/#dom-textencoder-encode>
    fn Encode(
        &self,
        cx: JSContext,
        input: USVString,
        can_gc: CanGc,
    ) -> RootedTraceableBox<HeapUint8Array> {
        let encoded = input.0.as_bytes();

        rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
        create_buffer_source(cx, encoded, js_object.handle_mut(), can_gc)
            .expect("Converting input to uint8 array should never fail")
    }

    /// <https://encoding.spec.whatwg.org/#dom-textencoder-encodeinto>
    #[expect(unsafe_code)]
    fn EncodeInto(
        &self,
        source: USVString,
        mut destination: CustomAutoRooterGuard<typedarray::Uint8Array>,
    ) -> TextEncoderEncodeIntoResult {
        let available = destination.len();

        // Bail out if the destination has no space available.
        if available == 0 {
            return TextEncoderEncodeIntoResult {
                read: Some(0),
                written: Some(0),
            };
        }

        let mut read = 0;
        let mut written = 0;

        let dest = unsafe { destination.as_mut_slice() };

        // Step 3, 4, 5, 6
        // Turn the source into a queue of scalar values.
        // Iterate over the source values.
        for result in source.0.chars() {
            let utf8_len = result.len_utf8();
            if available - written >= utf8_len {
                // Step 6.4.1
                // If destination’s byte length − written is greater than or equal to the number of bytes in result
                read += if result > '\u{FFFF}' { 2 } else { 1 };

                // Write the bytes in result into destination, with startingOffset set to written.
                let target = &mut dest[written..written + utf8_len];
                result.encode_utf8(target);

                // Increment written by the number of bytes in result.
                written += utf8_len;
            } else {
                // Step 6.4.2
                // Bail out when destination buffer is full.
                break;
            }
        }

        TextEncoderEncodeIntoResult {
            read: Some(read),
            written: Some(written as _),
        }
    }
}
