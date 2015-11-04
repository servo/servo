/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CryptoBinding;
use dom::bindings::codegen::Bindings::CryptoBinding::CryptoMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use js::jsapi::{JSContext, JSObject};
use js::jsapi::{JS_GetArrayBufferViewType, JS_GetObjectAsArrayBufferView, Type};
use rand::{OsRng, Rng};
use std::{ptr, slice};

no_jsmanaged_fields!(OsRng);

// https://developer.mozilla.org/en-US/docs/Web/API/Crypto
#[dom_struct]
pub struct Crypto {
    reflector_: Reflector,
    rng: DOMRefCell<OsRng>,
}

impl Crypto {
    fn new_inherited() -> Crypto {
        Crypto {
            reflector_: Reflector::new(),
            rng: DOMRefCell::new(OsRng::new().unwrap()),
        }
    }

    pub fn new(global: GlobalRef) -> Root<Crypto> {
        reflect_dom_object(box Crypto::new_inherited(), global, CryptoBinding::Wrap)
    }
}

impl CryptoMethods for Crypto {
    #[allow(unsafe_code)]
    // https://dvcs.w3.org/hg/webcrypto-api/raw-file/tip/spec/Overview.html#Crypto-method-getRandomValues
    fn GetRandomValues(&self, _cx: *mut JSContext, input: *mut JSObject)
                       -> Fallible<*mut JSObject> {
        let mut length = 0;
        let mut data = ptr::null_mut();
        if unsafe { JS_GetObjectAsArrayBufferView(input, &mut length, &mut data).is_null() } {
            return Err(Error::Type("Argument to Crypto.getRandomValues is not an ArrayBufferView".to_owned()));
        }
        if !is_integer_buffer(input) {
            return Err(Error::TypeMismatch);
        }
        if length > 65536 {
            return Err(Error::QuotaExceeded);
        }

        let mut buffer = unsafe {
            slice::from_raw_parts_mut(data, length as usize)
        };

        self.rng.borrow_mut().fill_bytes(&mut buffer);

        Ok(input)
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
        _ => false
    }
}
