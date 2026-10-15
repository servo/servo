/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsapi::Type;
use js::rust::CustomAutoRooterGuard;
use js::typedarray::{ArrayBufferView, HeapArrayBufferView};
use jstraceable_derive::JSTraceable;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::buffer_source::HeapBufferSource;
use script_bindings::codegen::GenericBindings::CryptoBinding::{CryptoMethods, Wrap as CryptoWrap};
use script_bindings::dom::MutNullableDom;
use script_bindings::error::{Error, Fallible};
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use script_bindings::root::DomRoot;
use script_bindings::str::DOMString;
use script_bindings::trace::RootedTraceableBox;
use uuid::Uuid;

use crate::subtlecrypto::SubtleCrypto;
use crate::traits::Equivalence;

/// <https://w3c.github.io/webcrypto/#crypto-interface>
#[dom_struct]
pub struct Crypto<D: DomTypes> {
    reflector_: Reflector,
    subtle: MutNullableDom<SubtleCrypto<D>>,
}

impl<D> Crypto<D>
where
    D: Equivalence,
{
    fn new_inherited() -> Crypto<D> {
        Crypto {
            reflector_: Reflector::new(),
            subtle: MutNullableDom::default(),
        }
    }

    pub fn new(cx: &mut JSContext, global: &D::GlobalScope) -> DomRoot<Crypto<D>> {
        reflect_dom_object_with_wrap::<D, _, _>(
            cx,
            Box::new(Crypto::new_inherited()),
            global,
            CryptoWrap::<D>,
        )
    }
}

impl<D> CryptoMethods<D> for Crypto<D>
where
    D: Equivalence,
    Self: DomGlobalGeneric<D>,
{
    /// <https://w3c.github.io/webcrypto/#dfn-Crypto-attribute-subtle>
    fn Subtle(&self, cx: &mut JSContext) -> DomRoot<SubtleCrypto<D>> {
        self.subtle
            .or_init(|| SubtleCrypto::new(cx, &*self.global_from_reflector()))
    }

    /// <https://w3c.github.io/webcrypto/#Crypto-method-getRandomValues>
    fn GetRandomValues(
        &self,
        cx: &mut JSContext,
        mut array: CustomAutoRooterGuard<ArrayBufferView>,
    ) -> Fallible<RootedTraceableBox<HeapArrayBufferView>> {
        // Step 1. If array is not an Int8Array, Uint8Array, Uint8ClampedArray, Int16Array,
        // Uint16Array, Int32Array, Uint32Array, BigInt64Array, or BigUint64Array, then throw a
        // TypeMismatchError and terminate the algorithm.
        if !matches!(
            array.get_array_type(),
            Type::Int8 |
                Type::Uint8 |
                Type::Uint8Clamped |
                Type::Int16 |
                Type::Uint16 |
                Type::Int32 |
                Type::Uint32 |
                Type::BigInt64 |
                Type::BigUint64
        ) {
            return Err(Error::TypeMismatch(Some(
                "The array must be an integer-based TypedArray".into(),
            )));
        }

        // Step 2. Let byteLength be the byte length of array.
        // Step 3. If byteLength is greater than 65536, throw a QuotaExceededError and terminate the
        // algorithm.
        // Step 4. Let bytes be a byte sequence of length byteLength.
        let bytes = array.as_mut_slice_safe(cx.no_gc()).unwrap_or_default();
        let byte_length = bytes.len();
        if byte_length > 65536 {
            return Err(Error::QuotaExceeded {
                quota: None,
                requested: None,
            });
        }

        // Step 5. Fill bytes with cryptographically secure random bytes.
        // Step 6. Write bytes into array.
        rand::fill(bytes);

        // Step 7. Return array.
        let array = HeapBufferSource::from_view(cx, array);
        array.get_typed_array().map_err(|_| Error::JSFailed)
    }

    /// <https://w3c.github.io/webcrypto/#Crypto-method-randomUUID>
    fn RandomUUID(&self) -> DOMString {
        // Step 1. Let bytes be a byte sequence of length 16.
        // Step 2. Fill bytes with cryptographically secure random bytes.
        // Step 3. Set the 4 most significant bits of bytes[6], which represent the UUID version, to 0100.
        // Step 4. Set the 2 most significant bits of bytes[8], which represent the UUID variant, to 10.
        // Step 5.
        // Return the string concatenation of «
        //     hexadecimal representation of bytes[0], hexadecimal representation of bytes[1],
        //     hexadecimal representation of bytes[2], hexadecimal representation of bytes[3],
        //     "-",
        //     hexadecimal representation of bytes[4], hexadecimal representation of bytes[5],
        //     "-",
        //     hexadecimal representation of bytes[6], hexadecimal representation of bytes[7],
        //     "-",
        //     hexadecimal representation of bytes[8], hexadecimal representation of bytes[9],
        //     "-",
        //     hexadecimal representation of bytes[10], hexadecimal representation of bytes[11],
        //     hexadecimal representation of bytes[12], hexadecimal representation of bytes[13],
        //     hexadecimal representation of bytes[14], hexadecimal representation of bytes[15]
        // ».
        let uuid = Uuid::new_v4();
        uuid.hyphenated()
            .encode_lower(&mut Uuid::encode_buffer())
            .to_owned()
            .into()
    }
}
