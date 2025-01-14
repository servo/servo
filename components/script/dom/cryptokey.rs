/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr::NonNull;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject, Value};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, KeyType, KeyUsage,
};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::js::conversions::ToJSValConvertible;
use crate::script_runtime::{CanGc, JSContext};

/// The underlying cryptographic data this key represents
#[allow(dead_code)]
#[derive(MallocSizeOf)]
pub(crate) enum Handle {
    Aes128(Vec<u8>),
    Aes192(Vec<u8>),
    Aes256(Vec<u8>),
    Pbkdf2(Vec<u8>),
    Hkdf(Vec<u8>),
    Hmac(Vec<u8>),
}

/// <https://w3c.github.io/webcrypto/#cryptokey-interface>
#[dom_struct]
pub(crate) struct CryptoKey {
    reflector_: Reflector,

    /// <https://w3c.github.io/webcrypto/#dom-cryptokey-type>
    key_type: KeyType,

    /// <https://w3c.github.io/webcrypto/#dom-cryptokey-extractable>
    extractable: Cell<bool>,

    /// The name of the algorithm used
    ///
    /// This is always the same as the `name` of the
    /// [`[[algorithm]]`](https://w3c.github.io/webcrypto/#dom-cryptokey-algorithm)
    /// internal slot, but we store it here again for convenience
    algorithm: DOMString,

    /// <https://w3c.github.io/webcrypto/#dom-cryptokey-algorithm>
    #[ignore_malloc_size_of = "Defined in mozjs"]
    algorithm_object: Heap<*mut JSObject>,

    /// <https://w3c.github.io/webcrypto/#dom-cryptokey-usages>
    usages: Vec<KeyUsage>,
    #[no_trace]
    handle: Handle,
}

impl CryptoKey {
    fn new_inherited(
        key_type: KeyType,
        extractable: bool,
        usages: Vec<KeyUsage>,
        algorithm: DOMString,
        handle: Handle,
    ) -> CryptoKey {
        CryptoKey {
            reflector_: Reflector::new(),
            key_type,
            extractable: Cell::new(extractable),
            algorithm,
            algorithm_object: Heap::default(),
            usages,
            handle,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        key_type: KeyType,
        extractable: bool,
        algorithm: DOMString,
        algorithm_object: HandleObject,
        usages: Vec<KeyUsage>,
        handle: Handle,
    ) -> DomRoot<CryptoKey> {
        let object = reflect_dom_object(
            Box::new(CryptoKey::new_inherited(
                key_type,
                extractable,
                usages,
                algorithm,
                handle,
            )),
            global,
            CanGc::note(),
        );

        object.algorithm_object.set(algorithm_object.get());

        object
    }

    pub(crate) fn algorithm(&self) -> String {
        self.algorithm.to_string()
    }

    pub(crate) fn usages(&self) -> &[KeyUsage] {
        &self.usages
    }

    pub(crate) fn handle(&self) -> &Handle {
        &self.handle
    }
}

impl CryptoKeyMethods<crate::DomTypeHolder> for CryptoKey {
    /// <https://w3c.github.io/webcrypto/#dom-cryptokey-type>
    fn Type(&self) -> KeyType {
        self.key_type
    }

    /// <https://w3c.github.io/webcrypto/#dom-cryptokey-extractable>
    fn Extractable(&self) -> bool {
        self.extractable.get()
    }

    /// <https://w3c.github.io/webcrypto/#dom-cryptokey-algorithm>
    fn Algorithm(&self, _cx: JSContext) -> NonNull<JSObject> {
        NonNull::new(self.algorithm_object.get()).unwrap()
    }

    #[allow(unsafe_code)]
    /// <https://w3c.github.io/webcrypto/#dom-cryptokey-usages>
    fn Usages(&self, cx: JSContext) -> NonNull<JSObject> {
        unsafe {
            rooted!(in(*cx) let mut usages: Value);
            self.usages.to_jsval(*cx, usages.handle_mut());
            NonNull::new(usages.to_object()).unwrap()
        }
    }
}

impl Handle {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Aes128(bytes) => bytes,
            Self::Aes192(bytes) => bytes,
            Self::Aes256(bytes) => bytes,
            Self::Pbkdf2(bytes) => bytes,
            Self::Hkdf(bytes) => bytes,
            Self::Hmac(bytes) => bytes,
        }
    }
}
