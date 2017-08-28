/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CryptoBinding;
use dom::bindings::codegen::Bindings::CryptoBinding::CryptoMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use js::jsapi;
use servo_rand::{ServoRng, Rng};

unsafe_no_jsmanaged_fields!(ServoRng);

// https://developer.mozilla.org/en-US/docs/Web/API/Crypto
#[dom_struct]
pub struct Crypto {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Defined in rand"]
    rng: DOMRefCell<ServoRng>,
}

impl Crypto {
    fn new_inherited() -> Crypto {
        Crypto {
            reflector_: Reflector::new(),
            rng: DOMRefCell::new(ServoRng::new()),
        }
    }

    pub fn new(global: &GlobalScope) -> Root<Crypto> {
        reflect_dom_object(box Crypto::new_inherited(), global, CryptoBinding::Wrap)
    }
}

impl CryptoMethods for Crypto {
    #[allow(unsafe_code)]
    // https://dvcs.w3.org/hg/webcrypto-api/raw-file/tip/spec/Overview.html#Crypto-method-getRandomValues
    unsafe fn GetRandomValues(&self,
                       _cx: *mut jsapi::JSContext,
                       input: *mut jsapi::JSObject)
                       -> Fallible<NonZero<*mut jsapi::JSObject>> {
        assert!(!input.is_null());
        typedarray!(in(_cx) let mut array_buffer_view: ArrayBufferView = input);
        let (array_type, mut data) = match array_buffer_view.as_mut() {
            Ok(x) => (x.get_array_type(), x.as_mut_slice()),
            Err(_) => {
                return Err(Error::Type("Argument to Crypto.getRandomValues is not an ArrayBufferView"
                                       .to_owned()));
            }
        };

        if !is_integer_buffer(array_type) {
            return Err(Error::TypeMismatch);
        }

        if data.len() > 65536 {
            return Err(Error::QuotaExceeded);
        }

        self.rng.borrow_mut().fill_bytes(&mut data);

        Ok(NonZero::new_unchecked(input))
    }
}

fn is_integer_buffer(array_type: jsapi::js::Scalar::Type) -> bool {
    match array_type {
        jsapi::js::Scalar::Type::Uint8 |
        jsapi::js::Scalar::Type::Uint8Clamped |
        jsapi::js::Scalar::Type::Int8 |
        jsapi::js::Scalar::Type::Uint16 |
        jsapi::js::Scalar::Type::Int16 |
        jsapi::js::Scalar::Type::Uint32 |
        jsapi::js::Scalar::Type::Int32 => true,
        _ => false,
    }
}
