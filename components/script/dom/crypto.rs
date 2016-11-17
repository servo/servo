/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CryptoBinding;
use dom::bindings::codegen::Bindings::CryptoBinding::CryptoMethods;
use dom::bindings::conversions::array_buffer_view_data;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;
use js::jsapi::{JSContext, JSObject};
use js::jsapi::{JS_GetArrayBufferViewType, Type};
use rand::{OsRng, Rng};

no_jsmanaged_fields!(OsRng);

// https://developer.mozilla.org/en-US/docs/Web/API/Crypto
#[dom_struct]
pub struct Crypto {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Defined in rand"]
    rng: DOMRefCell<OsRng>,
}

impl Crypto {
    fn new_inherited() -> Crypto {
        Crypto {
            reflector_: Reflector::new(),
            rng: DOMRefCell::new(OsRng::new().unwrap()),
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
                       _cx: *mut JSContext,
                       input: *mut JSObject)
                       -> Fallible<NonZero<*mut JSObject>> {
        assert!(!input.is_null());
        let mut data = match array_buffer_view_data::<u8>(input) {
            Some(data) => data,
            None => {
                return Err(Error::Type("Argument to Crypto.getRandomValues is not an ArrayBufferView"
                                       .to_owned()));
            }
        };

        if !is_integer_buffer(input) {
            return Err(Error::TypeMismatch);
        }

        if data.len() > 65536 {
            return Err(Error::QuotaExceeded);
        }

        self.rng.borrow_mut().fill_bytes(&mut data);

        Ok(NonZero::new(input))
    }
}

#[allow(unsafe_code)]
fn is_integer_buffer(input: *mut JSObject) -> bool {
    match unsafe { JS_GetArrayBufferViewType(input) } {
        Type::Uint8 |
        Type::Uint8Clamped |
        Type::Int8 |
        Type::Uint16 |
        Type::Int16 |
        Type::Uint32 |
        Type::Int32 => true,
        _ => false,
    }
}
