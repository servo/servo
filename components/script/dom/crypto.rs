/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CryptoBinding;
use dom::bindings::codegen::Bindings::CryptoBinding::CryptoMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::typedarray::{ArrayBufferView, TypedArrayRooter};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::bindings::cell::DOMRefCell;

use js::jsapi::{JSContext, JSObject, Type};

use rand::{Rng, OsRng};

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

impl<'a> CryptoMethods for &'a Crypto {
    #[allow(unsafe_code)]
    // https://dvcs.w3.org/hg/webcrypto-api/raw-file/tip/spec/Overview.html#Crypto-method-getRandomValues
    fn GetRandomValues(self, _cx: *mut JSContext, input: *mut JSObject)
                       -> Fallible<*mut JSObject> {
        let mut rooter = TypedArrayRooter::new();
        let mut array_buffer_view = match ArrayBufferView::from(input, &mut rooter) {
            Ok(view) => view,
            Err(_) =>
                return Err(Error::Type("Argument to Crypto.getRandomValues is not an \
                                        ArrayBufferView".to_owned())),
        };
        array_buffer_view.init();
        if !is_integer_buffer(&array_buffer_view) {
            return Err(Error::TypeMismatch);
        }

        let mut data = array_buffer_view.extract();
        let mut buffer = unsafe {
            data.as_mut_untyped_slice()
        };

        if buffer.len() > 65536 {
            return Err(Error::QuotaExceeded);
        }

        self.rng.borrow_mut().fill_bytes(buffer);

        Ok(input)
    }
}

fn is_integer_buffer(input: &ArrayBufferView) -> bool {
    match input.element_type() {
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
