/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr::NonNull;
use std::str::FromStr;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject, Value};
use malloc_size_of::MallocSizeOf;
use rustc_hash::FxHashMap;
use script_bindings::cell::DomRefCell;
use script_bindings::conversions::SafeToJSValConvertible;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use servo_base::id::{CryptoKeyId, CryptoKeyIndex};
use servo_constellation_traits::{SerializableCryptoKey, SerializableCryptoKeyHandle};
use zeroize::Zeroizing;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, CryptoKeyPair, KeyType, KeyUsage,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::Serializable;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::KeyAlgorithmAndDerivatives;
use crate::script_runtime::{CanGc, JSContext};

pub(crate) enum CryptoKeyOrCryptoKeyPair {
    CryptoKey(DomRoot<CryptoKey>),
    CryptoKeyPair(CryptoKeyPair),
}

/// The underlying cryptographic data this key represents.
///
/// Please make sure the inner types for secret variants implement the `zeroize::ZeroizeOnDrop`
/// trait, which signifies that the type will call `Zeroize::zeroize` on `Drop` to securely erase
/// the secret from memory.
pub(crate) enum Handle {
    RsaPrivateKey(rsa::RsaPrivateKey),
    RsaPublicKey(rsa::RsaPublicKey),
    P256PrivateKey(p256::SecretKey),
    P384PrivateKey(p384::SecretKey),
    P521PrivateKey(p521::SecretKey),
    P256PublicKey(p256::PublicKey),
    P384PublicKey(p384::PublicKey),
    P521PublicKey(p521::PublicKey),
    Ed25519PrivateKey(Zeroizing<Vec<u8>>),
    Ed25519PublicKey(Vec<u8>),
    X25519PrivateKey(x25519_dalek::StaticSecret),
    X25519PublicKey(x25519_dalek::PublicKey),
    Aes128Key(aes::cipher::crypto_common::Key<aes::Aes128>),
    Aes192Key(aes::cipher::crypto_common::Key<aes::Aes192>),
    Aes256Key(aes::cipher::crypto_common::Key<aes::Aes256>),
    HkdfSecret(Zeroizing<Vec<u8>>),
    Pbkdf2(Zeroizing<Vec<u8>>),
    Hmac(Zeroizing<Vec<u8>>),
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
    Argon2Password(Zeroizing<Vec<u8>>),
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
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        key_type: KeyType,
        extractable: bool,
        algorithm: KeyAlgorithmAndDerivatives,
        usages: Vec<KeyUsage>,
        handle: Handle,
    ) -> DomRoot<CryptoKey> {
        let crypto_key = reflect_dom_object_with_cx(
            Box::new(CryptoKey::new_inherited(
                key_type,
                extractable,
                algorithm.clone(),
                usages.clone(),
                handle,
            )),
            global,
            cx,
        );

        // Create and store a cached object of algorithm
        rooted!(&in(cx) let mut algorithm_object_value: Value);
        algorithm.safe_to_jsval(
            cx.into(),
            algorithm_object_value.handle_mut(),
            CanGc::from_cx(cx),
        );
        crypto_key
            .algorithm_cached
            .set(algorithm_object_value.to_object());

        // Create and store a cached object of usages
        rooted!(&in(cx) let mut usages_object_value: Value);
        usages.safe_to_jsval(
            cx.into(),
            usages_object_value.handle_mut(),
            CanGc::from_cx(cx),
        );
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

    pub(crate) fn set_usages(&self, cx: &mut js::context::JSContext, usages: &[KeyUsage]) {
        *self.usages.borrow_mut() = usages.to_owned();

        // Create and store a cached object of usages
        rooted!(&in(cx) let mut usages_object_value: Value);
        usages.safe_to_jsval(
            cx.into(),
            usages_object_value.handle_mut(),
            CanGc::from_cx(cx),
        );
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

impl Serializable for CryptoKey {
    type Index = CryptoKeyIndex;
    type Data = SerializableCryptoKey;

    /// <https://w3c.github.io/webcrypto/#cryptokey-interface-serializable>
    fn serialize(&self) -> Result<(CryptoKeyId, Self::Data), ()> {
        // Step 1. Set serialized.[[Type]] to the [[type]] internal slot of value.
        // Step 2. Set serialized.[[Extractable]] to the [[extractable]] internal slot of value.
        // Step 3. Set serialized.[[Algorithm]] to the sub-serialization of the [[algorithm]]
        // internal slot of value.
        // Step 4. Set serialized.[[Usages]] to the sub-serialization of the [[usages]] internal
        // slot of value.
        // Step 5. Set serialized.[[Handle]] to the [[handle]] internal slot of value.
        let serialized = SerializableCryptoKey {
            key_type: self.key_type.as_str().into(),
            extractable: self.extractable.get(),
            algorithm: (&self.algorithm).into(),
            usages: self
                .usages
                .borrow()
                .iter()
                .map(|usage| usage.as_str().into())
                .collect(),
            handle: (&self.handle).try_into()?,
        };
        Ok((CryptoKeyId::new(), serialized))
    }

    /// <https://w3c.github.io/webcrypto/#cryptokey-interface-serializable>
    fn deserialize(
        cx: &mut js::context::JSContext,
        owner: &GlobalScope,
        serialized: Self::Data,
    ) -> Result<DomRoot<Self>, ()> {
        // Step 1. Initialize the [[type]] internal slot of value to serialized.[[Type]].
        // Step 2. Initialize the [[extractable]] internal slot of value to
        // serialized.[[Extractable]].
        // Step 3. Initialize the [[algorithm]] internal slot of value to the sub-deserialization of
        // serialized.[[Algorithm]].
        // Step 4. Initialize the [[usages]] internal slot of value to the sub-deserialization of
        // serialized.[[Usages]].
        // Step 5. Initialize the [[handle]] internal slot of value to serialized.[[Handle]].
        Ok(CryptoKey::new(
            cx,
            owner,
            KeyType::from_str(&serialized.key_type)?,
            serialized.extractable,
            serialized.algorithm.try_into()?,
            serialized
                .usages
                .iter()
                .map(|usage| KeyUsage::from_str(usage))
                .collect::<Result<Vec<_>, _>>()?,
            serialized.handle.try_into()?,
        ))
    }

    fn serialized_storage<'a>(
        reader: StructuredData<'a, '_>,
    ) -> &'a mut Option<FxHashMap<CryptoKeyId, Self::Data>> {
        match reader {
            StructuredData::Reader(reader) => &mut reader.crypto_keys,
            StructuredData::Writer(writer) => &mut writer.crypto_keys,
        }
    }
}

impl Handle {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Pbkdf2(bytes) => bytes,
            Self::Hmac(bytes) => bytes,
            Self::Ed25519PrivateKey(bytes) => bytes,
            Self::Ed25519PublicKey(bytes) => bytes,
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
            Handle::Ed25519PrivateKey(bytes) => bytes.size_of(ops),
            Handle::Ed25519PublicKey(bytes) => bytes.size_of(ops),
            Handle::X25519PrivateKey(private_key) => private_key.size_of(ops),
            Handle::X25519PublicKey(public_key) => public_key.size_of(ops),
            Handle::Aes128Key(key) => key.size_of(ops),
            Handle::Aes192Key(key) => key.size_of(ops),
            Handle::Aes256Key(key) => key.size_of(ops),
            Handle::HkdfSecret(secret) => secret.size_of(ops),
            Handle::Pbkdf2(bytes) => bytes.size_of(ops),
            Handle::Hmac(bytes) => bytes.size_of(ops),
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

impl TryFrom<SerializableCryptoKeyHandle> for Handle {
    type Error = ();

    fn try_from(value: SerializableCryptoKeyHandle) -> Result<Self, Self::Error> {
        match &value {
            SerializableCryptoKeyHandle::RsaPrivateKey(private_key) => Ok(Handle::RsaPrivateKey(
                pkcs8::DecodePrivateKey::from_pkcs8_der(private_key).map_err(|_| ())?,
            )),
            SerializableCryptoKeyHandle::RsaPublicKey(public_key) => Ok(Handle::RsaPublicKey(
                pkcs8::spki::DecodePublicKey::from_public_key_der(public_key).map_err(|_| ())?,
            )),
            SerializableCryptoKeyHandle::P256PrivateKey(private_key) => Ok(Handle::P256PrivateKey(
                p256::SecretKey::from_sec1_der(private_key).map_err(|_| ())?,
            )),
            SerializableCryptoKeyHandle::P384PrivateKey(private_key) => Ok(Handle::P384PrivateKey(
                p384::SecretKey::from_sec1_der(private_key).map_err(|_| ())?,
            )),
            SerializableCryptoKeyHandle::P521PrivateKey(private_key) => Ok(Handle::P521PrivateKey(
                p521::SecretKey::from_sec1_der(private_key).map_err(|_| ())?,
            )),
            SerializableCryptoKeyHandle::P256PublicKey(public_key) => Ok(Handle::P256PublicKey(
                p256::PublicKey::from_sec1_bytes(public_key).map_err(|_| ())?,
            )),
            SerializableCryptoKeyHandle::P384PublicKey(public_key) => Ok(Handle::P384PublicKey(
                p384::PublicKey::from_sec1_bytes(public_key).map_err(|_| ())?,
            )),
            SerializableCryptoKeyHandle::P521PublicKey(public_key) => Ok(Handle::P521PublicKey(
                p521::PublicKey::from_sec1_bytes(public_key).map_err(|_| ())?,
            )),
            SerializableCryptoKeyHandle::Ed25519PrivateKey(bytes) => {
                Ok(Handle::Ed25519PrivateKey(bytes.clone().into()))
            },
            SerializableCryptoKeyHandle::Ed25519PublicKey(bytes) => {
                Ok(Handle::Ed25519PublicKey(bytes.clone()))
            },
            SerializableCryptoKeyHandle::X25519PrivateKey(private_key) => {
                Ok(Handle::X25519PrivateKey((*private_key).into()))
            },
            SerializableCryptoKeyHandle::X25519PublicKey(public_key) => {
                Ok(Handle::X25519PublicKey((*public_key).into()))
            },
            SerializableCryptoKeyHandle::Aes128Key(key) => {
                Ok(Handle::Aes128Key(aes::cipher::crypto_common::Key::<
                    aes::Aes128,
                >::clone_from_slice(key)))
            },
            SerializableCryptoKeyHandle::Aes192Key(key) => {
                Ok(Handle::Aes192Key(aes::cipher::crypto_common::Key::<
                    aes::Aes192,
                >::clone_from_slice(key)))
            },
            SerializableCryptoKeyHandle::Aes256Key(key) => {
                Ok(Handle::Aes256Key(aes::cipher::crypto_common::Key::<
                    aes::Aes256,
                >::clone_from_slice(key)))
            },
            SerializableCryptoKeyHandle::Hmac(bytes) => Ok(Handle::Hmac(bytes.clone().into())),
            SerializableCryptoKeyHandle::HkdfSecret(bytes) => {
                Ok(Handle::HkdfSecret(bytes.clone().into()))
            },
            SerializableCryptoKeyHandle::Pbkdf2(bytes) => Ok(Handle::Pbkdf2(bytes.clone().into())),
            SerializableCryptoKeyHandle::MlKem512PrivateKey(seed) => {
                Ok(Handle::MlKem512PrivateKey((
                    seed.0.as_slice().try_into().map_err(|_| ())?,
                    seed.1.as_slice().try_into().map_err(|_| ())?,
                )))
            },
            SerializableCryptoKeyHandle::MlKem768PrivateKey(seed) => {
                Ok(Handle::MlKem768PrivateKey((
                    seed.0.as_slice().try_into().map_err(|_| ())?,
                    seed.1.as_slice().try_into().map_err(|_| ())?,
                )))
            },
            SerializableCryptoKeyHandle::MlKem1024PrivateKey(seed) => {
                Ok(Handle::MlKem1024PrivateKey((
                    seed.0.as_slice().try_into().map_err(|_| ())?,
                    seed.1.as_slice().try_into().map_err(|_| ())?,
                )))
            },
            SerializableCryptoKeyHandle::MlKem512PublicKey(public_key) => {
                Ok(Handle::MlKem512PublicKey(Box::new(
                    public_key.as_slice().try_into().map_err(|_| ())?,
                )))
            },
            SerializableCryptoKeyHandle::MlKem768PublicKey(public_key) => {
                Ok(Handle::MlKem768PublicKey(Box::new(
                    public_key.as_slice().try_into().map_err(|_| ())?,
                )))
            },
            SerializableCryptoKeyHandle::MlKem1024PublicKey(public_key) => {
                Ok(Handle::MlKem1024PublicKey(Box::new(
                    public_key.as_slice().try_into().map_err(|_| ())?,
                )))
            },
            SerializableCryptoKeyHandle::MlDsa44PrivateKey(seed) => Ok(Handle::MlDsa44PrivateKey(
                seed.as_slice().try_into().map_err(|_| ())?,
            )),
            SerializableCryptoKeyHandle::MlDsa65PrivateKey(seed) => Ok(Handle::MlDsa65PrivateKey(
                seed.as_slice().try_into().map_err(|_| ())?,
            )),
            SerializableCryptoKeyHandle::MlDsa87PrivateKey(seed) => Ok(Handle::MlDsa87PrivateKey(
                seed.as_slice().try_into().map_err(|_| ())?,
            )),
            SerializableCryptoKeyHandle::MlDsa44PublicKey(public_key) => {
                Ok(Handle::MlDsa44PublicKey(Box::new(
                    public_key.as_slice().try_into().map_err(|_| ())?,
                )))
            },
            SerializableCryptoKeyHandle::MlDsa65PublicKey(public_key) => {
                Ok(Handle::MlDsa65PublicKey(Box::new(
                    public_key.as_slice().try_into().map_err(|_| ())?,
                )))
            },
            SerializableCryptoKeyHandle::MlDsa87PublicKey(public_key) => {
                Ok(Handle::MlDsa87PublicKey(Box::new(
                    public_key.as_slice().try_into().map_err(|_| ())?,
                )))
            },
            SerializableCryptoKeyHandle::ChaCha20Poly1305Key(key) => Ok(
                Handle::ChaCha20Poly1305Key(chacha20poly1305::Key::clone_from_slice(key)),
            ),
            SerializableCryptoKeyHandle::Argon2Password(password) => {
                Ok(Handle::Argon2Password(password.clone().into()))
            },
        }
    }
}

/// To serialize the key in the `Handle`, we convert the key into byte sequences. For most
/// cryptographic algorithms, this conversion is straightforward since the key can natually be
/// expressed as a byte sequence. However, some cryptographic algorithms require preprocessing
/// before their key can be represented in byte sequences. For example, an RSA private key needs to
/// be first converted into DER-encoded PKCS#8 format before it can be expressed as a byte sequence.
impl TryFrom<&Handle> for SerializableCryptoKeyHandle {
    type Error = ();

    fn try_from(value: &Handle) -> Result<Self, Self::Error> {
        match value {
            Handle::RsaPrivateKey(private_key) => Ok(SerializableCryptoKeyHandle::RsaPrivateKey(
                pkcs8::EncodePrivateKey::to_pkcs8_der(private_key)
                    .map_err(|_| ())?
                    .to_bytes()
                    .to_vec(),
            )),
            Handle::RsaPublicKey(public_key) => Ok(SerializableCryptoKeyHandle::RsaPublicKey(
                pkcs8::spki::EncodePublicKey::to_public_key_der(public_key)
                    .map_err(|_| ())?
                    .into_vec()
                    .to_vec(),
            )),
            Handle::P256PrivateKey(private_key) => Ok(SerializableCryptoKeyHandle::P256PrivateKey(
                private_key.to_sec1_der().map_err(|_| ())?.to_vec(),
            )),
            Handle::P384PrivateKey(private_key) => Ok(SerializableCryptoKeyHandle::P384PrivateKey(
                private_key.to_sec1_der().map_err(|_| ())?.to_vec(),
            )),
            Handle::P521PrivateKey(private_key) => Ok(SerializableCryptoKeyHandle::P521PrivateKey(
                private_key.to_sec1_der().map_err(|_| ())?.to_vec(),
            )),
            Handle::P256PublicKey(public_key) => Ok(SerializableCryptoKeyHandle::P256PublicKey(
                public_key.to_sec1_bytes().to_vec(),
            )),
            Handle::P384PublicKey(public_key) => Ok(SerializableCryptoKeyHandle::P384PublicKey(
                public_key.to_sec1_bytes().to_vec(),
            )),
            Handle::P521PublicKey(public_key) => Ok(SerializableCryptoKeyHandle::P521PublicKey(
                public_key.to_sec1_bytes().to_vec(),
            )),
            Handle::Ed25519PrivateKey(bytes) => Ok(SerializableCryptoKeyHandle::Ed25519PrivateKey(
                bytes.to_vec(),
            )),
            Handle::Ed25519PublicKey(bytes) => Ok(SerializableCryptoKeyHandle::Ed25519PublicKey(
                bytes.to_vec(),
            )),
            Handle::X25519PrivateKey(private_key) => Ok(
                SerializableCryptoKeyHandle::X25519PrivateKey(private_key.to_bytes()),
            ),
            Handle::X25519PublicKey(public_key) => Ok(
                SerializableCryptoKeyHandle::X25519PublicKey(public_key.to_bytes()),
            ),
            Handle::Aes128Key(key) => Ok(SerializableCryptoKeyHandle::Aes128Key(
                key.as_slice().into(),
            )),
            Handle::Aes192Key(key) => Ok(SerializableCryptoKeyHandle::Aes192Key(
                key.as_slice().into(),
            )),
            Handle::Aes256Key(key) => Ok(SerializableCryptoKeyHandle::Aes256Key(
                key.as_slice().into(),
            )),
            Handle::Hmac(bytes) => Ok(SerializableCryptoKeyHandle::Hmac(bytes.to_vec())),
            Handle::HkdfSecret(bytes) => {
                Ok(SerializableCryptoKeyHandle::HkdfSecret(bytes.to_vec()))
            },
            Handle::Pbkdf2(bytes) => Ok(SerializableCryptoKeyHandle::Pbkdf2(bytes.to_vec())),
            Handle::MlKem512PrivateKey(seed) => {
                Ok(SerializableCryptoKeyHandle::MlKem512PrivateKey((
                    seed.0.as_slice().to_vec(),
                    seed.1.as_slice().to_vec(),
                )))
            },
            Handle::MlKem768PrivateKey(seed) => {
                Ok(SerializableCryptoKeyHandle::MlKem768PrivateKey((
                    seed.0.as_slice().to_vec(),
                    seed.1.as_slice().to_vec(),
                )))
            },
            Handle::MlKem1024PrivateKey(seed) => {
                Ok(SerializableCryptoKeyHandle::MlKem1024PrivateKey((
                    seed.0.as_slice().to_vec(),
                    seed.1.as_slice().to_vec(),
                )))
            },
            Handle::MlKem512PublicKey(public_key) => Ok(
                SerializableCryptoKeyHandle::MlKem512PublicKey(public_key.as_slice().to_vec()),
            ),
            Handle::MlKem768PublicKey(public_key) => Ok(
                SerializableCryptoKeyHandle::MlKem768PublicKey(public_key.as_slice().to_vec()),
            ),
            Handle::MlKem1024PublicKey(public_key) => Ok(
                SerializableCryptoKeyHandle::MlKem1024PublicKey(public_key.as_slice().to_vec()),
            ),
            Handle::MlDsa44PrivateKey(seed) => Ok(SerializableCryptoKeyHandle::MlDsa44PrivateKey(
                seed.as_slice().to_vec(),
            )),
            Handle::MlDsa65PrivateKey(seed) => Ok(SerializableCryptoKeyHandle::MlDsa65PrivateKey(
                seed.as_slice().to_vec(),
            )),
            Handle::MlDsa87PrivateKey(seed) => Ok(SerializableCryptoKeyHandle::MlDsa87PrivateKey(
                seed.as_slice().to_vec(),
            )),
            Handle::MlDsa44PublicKey(public_key) => Ok(
                SerializableCryptoKeyHandle::MlDsa44PublicKey(public_key.as_slice().to_vec()),
            ),
            Handle::MlDsa65PublicKey(public_key) => Ok(
                SerializableCryptoKeyHandle::MlDsa65PublicKey(public_key.as_slice().to_vec()),
            ),
            Handle::MlDsa87PublicKey(public_key) => Ok(
                SerializableCryptoKeyHandle::MlDsa87PublicKey(public_key.as_slice().to_vec()),
            ),
            Handle::ChaCha20Poly1305Key(key) => Ok(
                SerializableCryptoKeyHandle::ChaCha20Poly1305Key(key.as_slice().to_vec()),
            ),
            Handle::Argon2Password(password) => Ok(SerializableCryptoKeyHandle::Argon2Password(
                password.to_vec(),
            )),
        }
    }
}
