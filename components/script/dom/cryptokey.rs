/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr::NonNull;

use aes::{Aes128, Aes192, Aes256};
use dom_struct::dom_struct;
use js::jsapi::{JSObject, Value};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyAlgorithm;
use crate::dom::bindings::reflector::Reflector;
use crate::js::conversions::ToJSValConvertible;
use crate::script_runtime::JSContext;

// The underlying cryptographic data this key represents
#[allow(dead_code)]
pub enum Handle {
    Aes128(Aes128),
    Aes192(Aes192),
    Aes256(Aes256),
}

#[dom_struct]
pub struct CryptoKey {
    reflector_: Reflector,
    key_type: KeyType,
    extractable: bool,
    #[ignore_malloc_size_of = ""]
    algorithm: KeyAlgorithm,
    usages: Vec<KeyUsage>,
    #[ignore_malloc_size_of = "Defined in external cryptography crates"]
    #[no_trace]
    handle: Handle,
}

impl CryptoKey {
    pub fn new_inherited(
        key_type: KeyType,
        extractable: bool,
        algorithm: KeyAlgorithm,
        usages: Vec<KeyUsage>,
        handle: Handle,
    ) -> CryptoKey {
        CryptoKey {
            reflector_: Reflector::new(),
            key_type,
            extractable,
            algorithm,
            usages,
            handle,
        }
    }
}

impl CryptoKeyMethods for CryptoKey {
    /// <https://w3c.github.io/webcrypto/#cryptokey-interface-members>
    fn Type(&self) -> KeyType {
        self.key_type.clone()
    }

    /// <https://w3c.github.io/webcrypto/#cryptokey-interface-members>
    fn Extractable(&self) -> bool {
        self.extractable
    }

    #[allow(unsafe_code)]
    /// <https://w3c.github.io/webcrypto/#cryptokey-interface-members>
    fn Algorithm(&self, cx: JSContext) -> NonNull<JSObject> {
        unsafe {
            rooted!(in(*cx) let mut algorithm: Value);
            self.algorithm.to_jsval(*cx, algorithm.handle_mut());
            NonNull::new(algorithm.to_object()).unwrap()
        }
    }

    #[allow(unsafe_code)]
    /// <https://w3c.github.io/webcrypto/#cryptokey-interface-members>
    fn Usages(&self, cx: JSContext) -> NonNull<JSObject> {
        unsafe {
            rooted!(in(*cx) let mut usages: Value);
            self.usages.to_jsval(*cx, usages.handle_mut());
            NonNull::new(usages.to_object()).unwrap()
        }
    }
}
