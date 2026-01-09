/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr::NonNull;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject, Value};
use malloc_size_of::MallocSizeOf;
use script_bindings::conversions::SafeToJSValConvertible;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, CryptoKeyPair, KeyType, KeyUsage,
};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::KeyAlgorithmAndDerivatives;
use crate::script_runtime::{CanGc, JSContext};

pub(crate) enum CryptoKeyOrCryptoKeyPair {
    CryptoKey(DomRoot<CryptoKey>),
    CryptoKeyPair(CryptoKeyPair),
}

/// The underlying cryptographic data this key represents
pub(crate) enum Handle {
    RsaPrivateKey(rsa::RsaPrivateKey),
    RsaPublicKey(rsa::RsaPublicKey),
    P256PrivateKey(p256::SecretKey),
    P384PrivateKey(p384::SecretKey),
    P521PrivateKey(p521::SecretKey),
    P256PublicKey(p256::PublicKey),
    P384PublicKey(p384::PublicKey),
    P521PublicKey(p521::PublicKey),
    X25519PrivateKey(x25519_dalek::StaticSecret),
    X25519PublicKey(x25519_dalek::PublicKey),
    Aes128(Vec<u8>),
    Aes192(Vec<u8>),
    Aes256(Vec<u8>),
    Aes128Key(aes::cipher::crypto_common::Key<aes::Aes128>),
    Aes192Key(aes::cipher::crypto_common::Key<aes::Aes192>),
    Aes256Key(aes::cipher::crypto_common::Key<aes::Aes256>),
    HkdfSecret(Vec<u8>),
    Pbkdf2(Vec<u8>),
    Hmac(Vec<u8>),
    Ed25519(Vec<u8>),
    MlKem512PrivateKey((ml_kem::B32, ml_kem::B32)),
    MlKem768PrivateKey((ml_kem::B32, ml_kem::B32)),
    MlKem1024PrivateKey((ml_kem::B32, ml_kem::B32)),
    MlKem512PublicKey(Box<ml_kem::Encoded<ml_kem::kem::EncapsulationKey<ml_kem::MlKem512Params>>>),
    MlKem768PublicKey(Box<ml_kem::Encoded<ml_kem::kem::EncapsulationKey<ml_kem::MlKem768Params>>>),
    MlKem1024PublicKey(
        Box<ml_kem::Encoded<ml_kem::kem::EncapsulationKey<ml_kem::MlKem1024Params>>>,
    ),
    MlDsa44PrivateKey(ml_dsa::B32),
    MlDsa65PrivateKey(ml_dsa::B32),
    MlDsa87PrivateKey(ml_dsa::B32),
    MlDsa44PublicKey(Box<ml_dsa::EncodedVerifyingKey<ml_dsa::MlDsa44>>),
    MlDsa65PublicKey(Box<ml_dsa::EncodedVerifyingKey<ml_dsa::MlDsa65>>),
    MlDsa87PublicKey(Box<ml_dsa::EncodedVerifyingKey<ml_dsa::MlDsa87>>),
    ChaCha20Poly1305Key(chacha20poly1305::Key),
    Argon2Password(Vec<u8>),
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
        algorithm.safe_to_jsval(cx, algorithm_object_value.handle_mut(), can_gc);
        crypto_key
            .algorithm_cached
            .set(algorithm_object_value.to_object());

        // Create and store a cached object of usages
        rooted!(in(*cx) let mut usages_object_value: Value);
        usages.safe_to_jsval(cx, usages_object_value.handle_mut(), can_gc);
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
        usages.safe_to_jsval(cx, usages_object_value.handle_mut(), CanGc::note());
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
            Self::Aes128(bytes) => bytes,
            Self::Aes192(bytes) => bytes,
            Self::Aes256(bytes) => bytes,
            Self::Pbkdf2(bytes) => bytes,
            Self::Hmac(bytes) => bytes,
            Self::Ed25519(bytes) => bytes,
            _ => unreachable!(),
        }
    }
}

impl MallocSizeOf for Handle {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        match self {
            Handle::RsaPrivateKey(private_key) => private_key.size_of(ops),
            Handle::RsaPublicKey(public_key) => public_key.size_of(ops),
            Handle::P256PrivateKey(private_key) => private_key.size_of(ops),
            Handle::P384PrivateKey(private_key) => private_key.size_of(ops),
            Handle::P521PrivateKey(private_key) => private_key.size_of(ops),
            Handle::P256PublicKey(public_key) => public_key.size_of(ops),
            Handle::P384PublicKey(public_key) => public_key.size_of(ops),
            Handle::P521PublicKey(public_key) => public_key.size_of(ops),
            Handle::X25519PrivateKey(private_key) => private_key.size_of(ops),
            Handle::X25519PublicKey(public_key) => public_key.size_of(ops),
            Handle::Aes128(bytes) => bytes.size_of(ops),
            Handle::Aes192(bytes) => bytes.size_of(ops),
            Handle::Aes256(bytes) => bytes.size_of(ops),
            Handle::Aes128Key(key) => key.size_of(ops),
            Handle::Aes192Key(key) => key.size_of(ops),
            Handle::Aes256Key(key) => key.size_of(ops),
            Handle::HkdfSecret(secret) => secret.size_of(ops),
            Handle::Pbkdf2(bytes) => bytes.size_of(ops),
            Handle::Hmac(bytes) => bytes.size_of(ops),
            Handle::Ed25519(bytes) => bytes.size_of(ops),
            Handle::MlKem512PrivateKey(seed) => seed.0.size_of(ops) + seed.1.size_of(ops),
            Handle::MlKem768PrivateKey(seed) => seed.0.size_of(ops) + seed.1.size_of(ops),
            Handle::MlKem1024PrivateKey(seed) => seed.0.size_of(ops) + seed.1.size_of(ops),
            Handle::MlKem512PublicKey(public_key) => public_key.size_of(ops),
            Handle::MlKem768PublicKey(public_key) => public_key.size_of(ops),
            Handle::MlKem1024PublicKey(public_key) => public_key.size_of(ops),
            Handle::MlDsa44PrivateKey(seed) => seed.size_of(ops),
            Handle::MlDsa65PrivateKey(seed) => seed.size_of(ops),
            Handle::MlDsa87PrivateKey(seed) => seed.size_of(ops),
            Handle::MlDsa44PublicKey(public_key) => public_key.size_of(ops),
            Handle::MlDsa65PublicKey(public_key) => public_key.size_of(ops),
            Handle::MlDsa87PublicKey(public_key) => public_key.size_of(ops),
            Handle::ChaCha20Poly1305Key(key) => key.size_of(ops),
            Handle::Argon2Password(password) => password.size_of(ops),
        }
    }
}
