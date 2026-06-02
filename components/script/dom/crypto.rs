/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject, Type};
use js::rust::CustomAutoRooterGuard;
use js::typedarray::{ArrayBufferView, ArrayBufferViewU8, HeapArrayBufferView, TypedArray};
use rand::TryRngCore;
use rand::rngs::OsRng;
use script_bindings::trace::RootedTraceableBox;
use uuid::Uuid;

use crate::dom::bindings::codegen::Bindings::CryptoBinding::CryptoMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::SubtleCrypto;
use crate::script_runtime::{CanGc, JSContext};

// https://developer.mozilla.org/en-US/docs/Web/API/Crypto
#[dom_struct]
pub(crate) struct Crypto {
    reflector_: Reflector,
    subtle: MutNullableDom<SubtleCrypto>,
}

impl Crypto {
    fn new_inherited() -> Crypto {
        Crypto {
            reflector_: Reflector::new(),
            subtle: MutNullableDom::default(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<Crypto> {
        reflect_dom_object(Box::new(Crypto::new_inherited()), global, can_gc)
    }
}

impl CryptoMethods<crate::DomTypeHolder> for Crypto {
    /// <https://w3c.github.io/webcrypto/#dfn-Crypto-attribute-subtle>
    fn Subtle(&self, can_gc: CanGc) -> DomRoot<SubtleCrypto> {
        self.subtle
            .or_init(|| SubtleCrypto::new(&self.global(), can_gc))
    }

    #[expect(unsafe_code)]
    /// <https://w3c.github.io/webcrypto/#Crypto-method-getRandomValues>
    fn GetRandomValues(
        &self,
        _cx: JSContext,
        mut input: CustomAutoRooterGuard<ArrayBufferView>,
    ) -> Fallible<RootedTraceableBox<HeapArrayBufferView>> {
        let array_type = input.get_array_type();

        if !is_integer_buffer(array_type) {
            Err(Error::TypeMismatch(None))
        } else {
            let data = unsafe { input.as_mut_slice() };
            if data.len() > 65536 {
                return Err(Error::QuotaExceeded {
                    quota: None,
                    requested: None,
                });
            }

            if OsRng.try_fill_bytes(data).is_err() {
                return Err(Error::JSFailed);
            }

            let underlying_object = unsafe { input.underlying_object() };
            TypedArray::<ArrayBufferViewU8, Box<Heap<*mut JSObject>>>::from(*underlying_object)
                .map(RootedTraceableBox::new)
                .map_err(|_| Error::JSFailed)
        }
    }

    /// <https://w3c.github.io/webcrypto/#Crypto-method-randomUUID>
    fn RandomUUID(&self) -> DOMString {
        let uuid = Uuid::new_v4();
        uuid.hyphenated()
            .encode_lower(&mut Uuid::encode_buffer())
            .to_owned()
            .into()
    }
}

fn is_integer_buffer(array_type: Type) -> bool {
    matches!(
        array_type,
        Type::Uint8 |
            Type::Uint8Clamped |
            Type::Int8 |
            Type::Uint16 |
            Type::Int16 |
            Type::Uint32 |
            Type::Int32 |
            Type::BigInt64 |
            Type::BigUint64
    )
}
