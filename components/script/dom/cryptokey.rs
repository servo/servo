/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr::NonNull;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject, Value};
use rsa::{RsaPrivateKey, RsaPublicKey};
use script_bindings::conversions::SafeToJSValConvertible;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, KeyType, KeyUsage,
};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::KeyAlgorithmAndDerivatives;
use crate::script_runtime::{CanGc, JSContext};

pub(crate) enum CryptoKeyOrCryptoKeyPair {
    CryptoKey(DomRoot<CryptoKey>),
    // TODO: CryptoKeyPair(CryptoKeyPair),
}

/// The underlying cryptographic data this key represents
#[allow(clippy::large_enum_variant)]
pub(crate) enum Handle {
    RsaPrivateKey(RsaPrivateKey),
    RsaPublicKey(RsaPublicKey),
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

    /// <https://w3c.github.io/webcrypto/#dfn-CryptoKey-slot-type>
    key_type: KeyType,

    /// <https://w3c.github.io/webcrypto/#dfn-CryptoKey-slot-extractable>
    extractable: Cell<bool>,

    /// <https://w3c.github.io/webcrypto/#dfn-CryptoKey-slot-algorithm>
    ///
    /// The contents of the [[algorithm]] internal slot shall be, or be derived from, a
    /// KeyAlgorithm.
    #[no_trace]
    algorithm: KeyAlgorithmAndDerivatives,

    /// <https://w3c.github.io/webcrypto/#dfn-CryptoKey-slot-algorithm_cached>
    #[ignore_malloc_size_of = "Defined in mozjs"]
    algorithm_cached: Heap<*mut JSObject>,

    /// <https://w3c.github.io/webcrypto/#dfn-CryptoKey-slot-usages>
    ///
    /// The contents of the [[usages]] internal slot shall be of type Sequence<KeyUsage>.
    usages: DomRefCell<Vec<KeyUsage>>,

    /// <https://w3c.github.io/webcrypto/#dfn-CryptoKey-slot-usages_cached>
    #[ignore_malloc_size_of = "Defined in mozjs"]
    usages_cached: Heap<*mut JSObject>,

    /// <https://w3c.github.io/webcrypto/#dfn-CryptoKey-slot-handle>
    #[ignore_malloc_size_of = "Defined in RustCrypto"]
    #[no_trace]
    handle: Handle,
}

impl CryptoKey {
    fn new_inherited(
        key_type: KeyType,
        extractable: bool,
        algorithm: KeyAlgorithmAndDerivatives,
        usages: Vec<KeyUsage>,
        handle: Handle,
    ) -> CryptoKey {
        CryptoKey {
            reflector_: Reflector::new(),
            key_type,
            extractable: Cell::new(extractable),
            algorithm,
            algorithm_cached: Heap::default(),
            usages: DomRefCell::new(usages),
            usages_cached: Heap::default(),
            handle,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        global: &GlobalScope,
        key_type: KeyType,
        extractable: bool,
        algorithm: KeyAlgorithmAndDerivatives,
        usages: Vec<KeyUsage>,
        handle: Handle,
        can_gc: CanGc,
    ) -> DomRoot<CryptoKey> {
        let crypto_key = reflect_dom_object(
            Box::new(CryptoKey::new_inherited(
                key_type,
                extractable,
                algorithm.clone(),
                usages.clone(),
                handle,
            )),
            global,
            can_gc,
        );

        let cx = GlobalScope::get_cx();

        // Create and store a cached object of algorithm
        rooted!(in(*cx) let mut algorithm_object_value: Value);
        algorithm.safe_to_jsval(cx, algorithm_object_value.handle_mut());
        crypto_key
            .algorithm_cached
            .set(algorithm_object_value.to_object());

        // Create and store a cached object of usages
        rooted!(in(*cx) let mut usages_object_value: Value);
        usages.safe_to_jsval(cx, usages_object_value.handle_mut());
        crypto_key
            .usages_cached
            .set(usages_object_value.to_object());

        crypto_key
    }

    pub(crate) fn algorithm(&self) -> &KeyAlgorithmAndDerivatives {
        &self.algorithm
    }

    pub(crate) fn usages(&self) -> Vec<KeyUsage> {
        self.usages.borrow().clone()
    }

    pub(crate) fn handle(&self) -> &Handle {
        &self.handle
    }

    pub(crate) fn set_extractable(&self, extractable: bool) {
        self.extractable.set(extractable);
    }

    pub(crate) fn set_usages(&self, usages: &[KeyUsage]) {
        *self.usages.borrow_mut() = usages.to_owned();

        // Create and store a cached object of usages
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut usages_object_value: Value);
        usages.safe_to_jsval(cx, usages_object_value.handle_mut());
        self.usages_cached.set(usages_object_value.to_object());
    }
}

impl CryptoKeyMethods<crate::DomTypeHolder> for CryptoKey {
    /// <https://w3c.github.io/webcrypto/#dom-cryptokey-type>
    fn Type(&self) -> KeyType {
        // Reflects the [[type]] internal slot, which contains the type of the underlying key.
        self.key_type
    }

    /// <https://w3c.github.io/webcrypto/#dom-cryptokey-extractable>
    fn Extractable(&self) -> bool {
        // Reflects the [[extractable]] internal slot, which indicates whether or not the raw
        // keying material may be exported by the application.
        self.extractable.get()
    }

    /// <https://w3c.github.io/webcrypto/#dom-cryptokey-algorithm>
    fn Algorithm(&self, _cx: JSContext) -> NonNull<JSObject> {
        // Returns the cached ECMAScript object associated with the [[algorithm]] internal slot.
        NonNull::new(self.algorithm_cached.get()).unwrap()
    }

    /// <https://w3c.github.io/webcrypto/#dom-cryptokey-usages>
    fn Usages(&self, _cx: JSContext) -> NonNull<JSObject> {
        // Returns the cached ECMAScript object associated with the [[usages]] internal slot, which
        // indicates which cryptographic operations are permissible to be used with this key.
        NonNull::new(self.usages_cached.get()).unwrap()
    }
}

impl Handle {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        match self {
            Self::RsaPrivateKey(_) => unreachable!(),
            Self::RsaPublicKey(_) => unreachable!(),
            Self::Aes128(bytes) => bytes,
            Self::Aes192(bytes) => bytes,
            Self::Aes256(bytes) => bytes,
            Self::Pbkdf2(bytes) => bytes,
            Self::Hkdf(bytes) => bytes,
            Self::Hmac(bytes) => bytes,
        }
    }
}
