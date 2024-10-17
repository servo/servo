/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr::NonNull;

use dom_struct::dom_struct;
use js::jsapi::{JSObject, Value};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{AesKeyAlgorithm, KeyAlgorithm};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::js::conversions::ToJSValConvertible;
use crate::script_runtime::JSContext;

/// The underlying cryptographic data this key represents
#[allow(dead_code)]
pub enum Handle {
    Aes128(Vec<u8>),
    Aes192(Vec<u8>),
    Aes256(Vec<u8>),
}

/// <https://w3c.github.io/webcrypto/#cryptokey-interface>
#[dom_struct]
pub struct CryptoKey {
    reflector_: Reflector,
    key_type: KeyType,
    extractable: Cell<bool>,
    // This would normally be KeyAlgorithm but we cannot Send DOMString, which
    // is a member of Algorithm
    algorithm: DomRefCell<String>,
    usages: Vec<KeyUsage>,
    #[ignore_malloc_size_of = "Defined in external cryptography crates"]
    #[no_trace]
    handle: Handle,
}

impl CryptoKey {
    fn new_inherited(
        key_type: KeyType,
        extractable: bool,
        algorithm: KeyAlgorithm,
        usages: Vec<KeyUsage>,
        handle: Handle,
    ) -> CryptoKey {
        CryptoKey {
            reflector_: Reflector::new(),
            key_type,
            extractable: Cell::new(extractable),
            algorithm: DomRefCell::new(algorithm.name.to_string()),
            usages,
            handle,
        }
    }

    pub fn new(
        global: &GlobalScope,
        key_type: KeyType,
        extractable: bool,
        algorithm: KeyAlgorithm,
        usages: Vec<KeyUsage>,
        handle: Handle,
    ) -> DomRoot<CryptoKey> {
        reflect_dom_object(
            Box::new(CryptoKey::new_inherited(
                key_type,
                extractable,
                algorithm,
                usages,
                handle,
            )),
            global,
        )
    }

    pub fn algorithm(&self) -> String {
        self.algorithm.borrow().to_string()
    }

    pub fn usages(&self) -> &[KeyUsage] {
        &self.usages
    }

    pub fn handle(&self) -> &Handle {
        &self.handle
    }
}

impl CryptoKeyMethods for CryptoKey {
    /// <https://w3c.github.io/webcrypto/#cryptokey-interface-members>
    fn Type(&self) -> KeyType {
        self.key_type
    }

    /// <https://w3c.github.io/webcrypto/#cryptokey-interface-members>
    fn Extractable(&self) -> bool {
        self.extractable.get()
    }

    #[allow(unsafe_code)]
    /// <https://w3c.github.io/webcrypto/#cryptokey-interface-members>
    fn Algorithm(&self, cx: JSContext) -> NonNull<JSObject> {
        let parent = KeyAlgorithm {
            name: DOMString::from_string(self.algorithm()),
        };
        let algorithm = match self.handle() {
            Handle::Aes128(_) => AesKeyAlgorithm {
                parent,
                length: 128,
            },
            Handle::Aes192(_) => AesKeyAlgorithm {
                parent,
                length: 192,
            },
            Handle::Aes256(_) => AesKeyAlgorithm {
                parent,
                length: 256,
            },
        };
        unsafe {
            rooted!(in(*cx) let mut alg: Value);
            algorithm.to_jsval(*cx, alg.handle_mut());
            NonNull::new(alg.to_object()).unwrap()
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
