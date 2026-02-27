/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod aes_cbc_operation;
mod aes_common;
mod aes_ctr_operation;
mod aes_gcm_operation;
mod aes_kw_operation;
mod aes_ocb_operation;
mod argon2_operation;
mod chacha20_poly1305_operation;
mod cshake_operation;
mod ecdh_operation;
mod ecdsa_operation;
mod ed25519_operation;
mod hkdf_operation;
mod hmac_operation;
mod ml_dsa_operation;
mod ml_kem_operation;
mod pbkdf2_operation;
mod rsa_common;
mod rsa_oaep_operation;
mod rsa_pss_operation;
mod rsassa_pkcs1_v1_5_operation;
mod sha3_operation;
mod sha_operation;
mod x25519_operation;

use std::fmt::Display;
use std::ptr;
use std::rc::Rc;
use std::str::FromStr;

use base64ct::{Base64UrlUnpadded, Encoding};
use dom_struct::dom_struct;
use js::conversions::ConversionResult;
use js::jsapi::{Heap, JSObject};
use js::jsval::{ObjectValue, UndefinedValue};
use js::realm::CurrentRealm;
use js::rust::wrappers2::JS_ParseJSON;
use js::rust::{HandleValue, MutableHandleValue};
use js::typedarray::ArrayBufferU8;
use strum::{EnumString, IntoStaticStr, VariantArray};

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, CryptoKeyPair, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{
    AeadParams, AesCbcParams, AesCtrParams, AesDerivedKeyParams, AesGcmParams, AesKeyAlgorithm,
    AesKeyGenParams, Algorithm, AlgorithmIdentifier, Argon2Params, CShakeParams, ContextParams,
    EcKeyAlgorithm, EcKeyGenParams, EcKeyImportParams, EcdhKeyDeriveParams, EcdsaParams,
    EncapsulatedBits, EncapsulatedKey, HkdfParams, HmacImportParams, HmacKeyAlgorithm,
    HmacKeyGenParams, JsonWebKey, KeyAlgorithm, KeyFormat, Pbkdf2Params, RsaHashedImportParams,
    RsaHashedKeyAlgorithm, RsaHashedKeyGenParams, RsaKeyAlgorithm, RsaOaepParams, RsaPssParams,
    SubtleCryptoMethods,
};
use crate::dom::bindings::codegen::UnionTypes::{
    ArrayBufferViewOrArrayBuffer, ArrayBufferViewOrArrayBufferOrJsonWebKey, ObjectOrString,
};
use crate::dom::bindings::conversions::{SafeFromJSValConvertible, SafeToJSValConvertible};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_cx};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, serialize_jsval_to_json_utf8};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::utils::set_dictionary_property;
use crate::dom::cryptokey::{CryptoKey, CryptoKeyOrCryptoKeyPair};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::{CanGc, JSContext};

// Regconized algorithm name from <https://w3c.github.io/webcrypto/>
const ALG_RSASSA_PKCS1_V1_5: &str = "RSASSA-PKCS1-v1_5";
const ALG_RSA_PSS: &str = "RSA-PSS";
const ALG_RSA_OAEP: &str = "RSA-OAEP";
const ALG_ECDSA: &str = "ECDSA";
const ALG_ECDH: &str = "ECDH";
const ALG_ED25519: &str = "Ed25519";
const ALG_X25519: &str = "X25519";
const ALG_AES_CTR: &str = "AES-CTR";
const ALG_AES_CBC: &str = "AES-CBC";
const ALG_AES_GCM: &str = "AES-GCM";
const ALG_AES_KW: &str = "AES-KW";
const ALG_HMAC: &str = "HMAC";
const ALG_SHA1: &str = "SHA-1";
const ALG_SHA256: &str = "SHA-256";
const ALG_SHA384: &str = "SHA-384";
const ALG_SHA512: &str = "SHA-512";
const ALG_HKDF: &str = "HKDF";
const ALG_PBKDF2: &str = "PBKDF2";

// Regconized algorithm name from <https://wicg.github.io/webcrypto-modern-algos/>
const ALG_ML_KEM_512: &str = "ML-KEM-512";
const ALG_ML_KEM_768: &str = "ML-KEM-768";
const ALG_ML_KEM_1024: &str = "ML-KEM-1024";
const ALG_ML_DSA_44: &str = "ML-DSA-44";
const ALG_ML_DSA_65: &str = "ML-DSA-65";
const ALG_ML_DSA_87: &str = "ML-DSA-87";
const ALG_AES_OCB: &str = "AES-OCB";
const ALG_CHACHA20_POLY1305: &str = "ChaCha20-Poly1305";
const ALG_SHA3_256: &str = "SHA3-256";
const ALG_SHA3_384: &str = "SHA3-384";
const ALG_SHA3_512: &str = "SHA3-512";
const ALG_CSHAKE_128: &str = "cSHAKE128";
const ALG_CSHAKE_256: &str = "cSHAKE256";
const ALG_ARGON2D: &str = "Argon2d";
const ALG_ARGON2I: &str = "Argon2i";
const ALG_ARGON2ID: &str = "Argon2id";

// Named elliptic curves
const NAMED_CURVE_P256: &str = "P-256";
const NAMED_CURVE_P384: &str = "P-384";
const NAMED_CURVE_P521: &str = "P-521";

static SUPPORTED_CURVES: &[&str] = &[NAMED_CURVE_P256, NAMED_CURVE_P384, NAMED_CURVE_P521];

#[derive(EnumString, VariantArray, IntoStaticStr, Clone, Copy)]
enum CryptoAlgorithm {
    #[strum(serialize = "RSASSA-PKCS1-v1_5")]
    RsassaPkcs1V1_5,
    #[strum(serialize = "RSA-PSS")]
    RsaPss,
    #[strum(serialize = "RSA-OAEP")]
    RsaOaep,
    #[strum(serialize = "ECDSA")]
    Ecdsa,
    #[strum(serialize = "ECDH")]
    Ecdh,
    #[strum(serialize = "Ed25519")]
    Ed25519,
    #[strum(serialize = "X25519")]
    X25519,
    #[strum(serialize = "AES-CTR")]
    AesCtr,
    #[strum(serialize = "AES-CBC")]
    AesCbc,
    #[strum(serialize = "AES-GCM")]
    AesGcm,
    #[strum(serialize = "AES-KW")]
    AesKw,
    #[strum(serialize = "HMAC")]
    Hmac,
    #[strum(serialize = "SHA-1")]
    Sha1,
    #[strum(serialize = "SHA-256")]
    Sha256,
    #[strum(serialize = "SHA-384")]
    Sha384,
    #[strum(serialize = "SHA-512")]
    Sha512,
    #[strum(serialize = "HKDF")]
    Hkdf,
    #[strum(serialize = "PBKDF2")]
    Pbkdf2,
    #[strum(serialize = "ML-KEM-512")]
    MlKem512,
    #[strum(serialize = "ML-KEM-768")]
    MlKem768,
    #[strum(serialize = "ML-KEM-1024")]
    MlKem1024,
    #[strum(serialize = "ML-DSA-44")]
    MlDsa44,
    #[strum(serialize = "ML-DSA-65")]
    MlDsa65,
    #[strum(serialize = "ML-DSA-87")]
    MlDsa87,
    #[strum(serialize = "AES-OCB")]
    AesOcb,
    #[strum(serialize = "ChaCha20-Poly1305")]
    ChaCha20Poly1305,
    #[strum(serialize = "SHA3-256")]
    Sha3_256,
    #[strum(serialize = "SHA3-384")]
    Sha3_384,
    #[strum(serialize = "SHA3-512")]
    Sha3_512,
    #[strum(serialize = "cSHAKE128")]
    CShake128,
    #[strum(serialize = "cSHAKE256")]
    CShake256,
    #[strum(serialize = "Argon2d")]
    Argon2D,
    #[strum(serialize = "Argon2i")]
    Argon2I,
    #[strum(serialize = "Argon2id")]
    Argon2ID,
}

impl CryptoAlgorithm {
    /// <https://w3c.github.io/webcrypto/#recognized-algorithm-name>
    fn as_str(&self) -> &'static str {
        (*self).into()
    }

    fn from_str_ignore_case(algorithm_name: &str) -> Fallible<CryptoAlgorithm> {
        Self::VARIANTS
            .iter()
            .find(|algorithm| algorithm.as_str().eq_ignore_ascii_case(algorithm_name))
            .cloned()
            .ok_or(Error::NotSupported(Some(format!(
                "Unsupported algorithm: {algorithm_name}"
            ))))
    }
}

#[dom_struct]
pub(crate) struct SubtleCrypto {
    reflector_: Reflector,
}

impl SubtleCrypto {
    fn new_inherited() -> SubtleCrypto {
        SubtleCrypto {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
    ) -> DomRoot<SubtleCrypto> {
        reflect_dom_object_with_cx(Box::new(SubtleCrypto::new_inherited()), global, cx)
    }

    /// Queue a global task on the crypto task source, given realm's global object, to resolve
    /// promise with the result of creating an ArrayBuffer in realm, containing data. If it fails
    /// to create buffer source, reject promise with a JSFailedError.
    fn resolve_promise_with_data(&self, promise: Rc<Promise>, data: Vec<u8>) {
        let trusted_promise = TrustedPromise::new(promise);
        self.global()
            .task_manager()
            .crypto_task_source()
            .queue(task!(resolve_data: move |cx| {
                let promise = trusted_promise.root();

                rooted!(&in(cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());
                match create_buffer_source::<ArrayBufferU8>(
                    cx.into(),
                    &data,
                    array_buffer_ptr.handle_mut(),
                    CanGc::from_cx(cx),
                ) {
                    Ok(_) => promise.resolve_native(&*array_buffer_ptr, CanGc::from_cx(cx)),
                    Err(_) => promise.reject_error(Error::JSFailed, CanGc::from_cx(cx)),
                }
            }));
    }

    /// Queue a global task on the crypto task source, given realm's global object, to resolve
    /// promise with the result of converting a JsonWebKey dictionary to an ECMAScript Object in
    /// realm, as defined by [WebIDL].
    fn resolve_promise_with_jwk(
        &self,
        cx: &mut js::context::JSContext,
        promise: Rc<Promise>,
        jwk: Box<JsonWebKey>,
    ) {
        // NOTE: Serialize the JsonWebKey dictionary by stringifying it, in order to pass it to
        // other threads.
        let stringified_jwk = match jwk.stringify(cx) {
            Ok(stringified_jwk) => stringified_jwk.to_string(),
            Err(error) => {
                self.reject_promise_with_error(promise, error);
                return;
            },
        };

        let trusted_subtle = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise);
        self.global()
            .task_manager()
            .crypto_task_source()
            .queue(task!(resolve_jwk: move |cx| {
                let subtle = trusted_subtle.root();
                let promise = trusted_promise.root();

                match JsonWebKey::parse(cx, stringified_jwk.as_bytes()) {
                    Ok(jwk) => {
                        rooted!(&in(cx) let mut rval = UndefinedValue());
                        jwk.safe_to_jsval(cx.into(), rval.handle_mut(), CanGc::from_cx(cx));
                        rooted!(&in(cx) let mut object = rval.to_object());
                        promise.resolve_native(&*object, CanGc::from_cx(cx));
                    },
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                }
            }));
    }

    /// Queue a global task on the crypto task source, given realm's global object, to resolve
    /// promise with a CryptoKey.
    fn resolve_promise_with_key(&self, promise: Rc<Promise>, key: DomRoot<CryptoKey>) {
        let trusted_key = Trusted::new(&*key);
        let trusted_promise = TrustedPromise::new(promise);
        self.global()
            .task_manager()
            .crypto_task_source()
            .queue(task!(resolve_key: move |cx| {
                let key = trusted_key.root();
                let promise = trusted_promise.root();
                promise.resolve_native(&key, CanGc::from_cx(cx));
            }));
    }

    /// Queue a global task on the crypto task source, given realm's global object, to resolve
    /// promise with a CryptoKeyPair.
    fn resolve_promise_with_key_pair(&self, promise: Rc<Promise>, key_pair: CryptoKeyPair) {
        let trusted_private_key = key_pair.privateKey.map(|key| Trusted::new(&*key));
        let trusted_public_key = key_pair.publicKey.map(|key| Trusted::new(&*key));
        let trusted_promise = TrustedPromise::new(promise);
        self.global()
            .task_manager()
            .crypto_task_source()
            .queue(task!(resolve_key: move |cx| {
                let key_pair = CryptoKeyPair {
                    privateKey: trusted_private_key.map(|trusted_key| trusted_key.root()),
                    publicKey: trusted_public_key.map(|trusted_key| trusted_key.root()),
                };
                let promise = trusted_promise.root();
                promise.resolve_native(&key_pair, CanGc::from_cx(cx));
            }));
    }

    /// Queue a global task on the crypto task source, given realm's global object, to resolve
    /// promise with a bool value.
    fn resolve_promise_with_bool(&self, promise: Rc<Promise>, result: bool) {
        let trusted_promise = TrustedPromise::new(promise);
        self.global()
            .task_manager()
            .crypto_task_source()
            .queue(task!(resolve_bool: move |cx| {
                let promise = trusted_promise.root();
                promise.resolve_native(&result, CanGc::from_cx(cx));
            }));
    }

    /// Queue a global task on the crypto task source, given realm's global object, to reject
    /// promise with an error.
    fn reject_promise_with_error(&self, promise: Rc<Promise>, error: Error) {
        let trusted_promise = TrustedPromise::new(promise);
        self.global()
            .task_manager()
            .crypto_task_source()
            .queue(task!(reject_error: move |cx| {
                let promise = trusted_promise.root();
                promise.reject_error(error, CanGc::from_cx(cx));
            }));
    }

    /// Queue a global task on the crypto task source, given realm's global object, to resolve
    /// promise with the result of converting EncapsulatedKey to an ECMAScript Object in realm, as
    /// defined by [WebIDL].
    fn resolve_promise_with_encapsulated_key(
        &self,
        promise: Rc<Promise>,
        encapsulated_key: SubtleEncapsulatedKey,
    ) {
        let trusted_promise = TrustedPromise::new(promise);
        self.global().task_manager().crypto_task_source().queue(
            task!(resolve_encapsulated_key: move |cx| {
                let promise = trusted_promise.root();
                promise.resolve_native(&encapsulated_key, CanGc::from_cx(cx));
            }),
        );
    }

    /// Queue a global task on the crypto task source, given realm's global object, to resolve
    /// promise with the result of converting EncapsulateBits to an ECMAScript Object in realm, as
    /// defined by [WebIDL].
    fn resolve_promise_with_encapsulated_bits(
        &self,
        promise: Rc<Promise>,
        encapsulated_bits: SubtleEncapsulatedBits,
    ) {
        let trusted_promise = TrustedPromise::new(promise);
        self.global().task_manager().crypto_task_source().queue(
            task!(resolve_encapsulated_bits: move |cx| {
                let promise = trusted_promise.root();
                promise.resolve_native(&encapsulated_bits, CanGc::from_cx(cx));
            }),
        );
    }
}

impl SubtleCryptoMethods<crate::DomTypeHolder> for SubtleCrypto {
    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-encrypt>
    fn Encrypt(
        &self,
        cx: &mut CurrentRealm,
        algorithm: AlgorithmIdentifier,
        key: &CryptoKey,
        data: ArrayBufferViewOrArrayBuffer,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm and key be the algorithm and key parameters passed to the
        // encrypt() method, respectively.
        // NOTE: We did that in method parameter.

        // Step 2. Let data be the result of getting a copy of the bytes held by the data parameter
        // passed to the encrypt() method.
        let data = match data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        // Step 3. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set
        // to algorithm and op set to "encrypt".
        // Step 4. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let promise = Promise::new_in_realm(cx);
        let normalized_algorithm = match normalize_algorithm::<EncryptOperation>(cx, &algorithm) {
            Ok(normalized_algorithm) => normalized_algorithm,
            Err(error) => {
                promise.reject_error(error, CanGc::from_cx(cx));
                return promise;
            },
        };

        // Step 5. Let realm be the relevant realm of this.
        // Step 6. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 4.

        // Step 7. Return promise and perform the remaining steps in parallel.
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(encrypt: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();

                // Step 8. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 9. If the name member of normalizedAlgorithm is not equal to the name
                // attribute of the [[algorithm]] internal slot of key then throw an
                // InvalidAccessError.
                if normalized_algorithm.name() != key.algorithm().name() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 10. If the [[usages]] internal slot of key does not contain an entry that
                // is "encrypt", then throw an InvalidAccessError.
                if !key.usages().contains(&KeyUsage::Encrypt) {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 11. Let ciphertext be the result of performing the encrypt operation
                // specified by normalizedAlgorithm using algorithm and key and with data as
                // plaintext.
                let ciphertext = match normalized_algorithm.encrypt(&key, &data) {
                    Ok(ciphertext) => ciphertext,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 12. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 13. Let result be the result of creating an ArrayBuffer in realm,
                // containing ciphertext.
                // Step 14. Resolve promise with result.
                subtle.resolve_promise_with_data(promise, ciphertext);
            }));
        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-decrypt>
    fn Decrypt(
        &self,
        cx: &mut CurrentRealm,
        algorithm: AlgorithmIdentifier,
        key: &CryptoKey,
        data: ArrayBufferViewOrArrayBuffer,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm and key be the algorithm and key parameters passed to the
        // decrypt() method, respectively.
        // NOTE: We did that in method parameter.

        // Step 2. Let data be the result of getting a copy of the bytes held by the data parameter
        // passed to the decrypt() method.
        let data = match data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        // Step 3. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set
        // to algorithm and op set to "decrypt".
        // Step 4. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let promise = Promise::new_in_realm(cx);
        let normalized_algorithm = match normalize_algorithm::<DecryptOperation>(cx, &algorithm) {
            Ok(normalized_algorithm) => normalized_algorithm,
            Err(error) => {
                promise.reject_error(error, CanGc::from_cx(cx));
                return promise;
            },
        };

        // Step 5. Let realm be the relevant realm of this.
        // Step 6. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 4.

        // Step 7. Return promise and perform the remaining steps in parallel.
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(decrypt: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();

                // Step 8. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 9. If the name member of normalizedAlgorithm is not equal to the name
                // attribute of the [[algorithm]] internal slot of key then throw an
                // InvalidAccessError.
                if normalized_algorithm.name() != key.algorithm().name() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 10. If the [[usages]] internal slot of key does not contain an entry that
                // is "decrypt", then throw an InvalidAccessError.
                if !key.usages().contains(&KeyUsage::Decrypt) {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 11. Let plaintext be the result of performing the decrypt operation
                // specified by normalizedAlgorithm using key and algorithm and with data as
                // ciphertext.
                let plaintext = match normalized_algorithm.decrypt(&key, &data) {
                    Ok(plaintext) => plaintext,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 12. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 13. Let result be the result of creating an ArrayBuffer in realm,
                // containing plaintext.
                // Step 14. Resolve promise with result.
                subtle.resolve_promise_with_data(promise, plaintext);
            }));
        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-sign>
    fn Sign(
        &self,
        cx: &mut CurrentRealm,
        algorithm: AlgorithmIdentifier,
        key: &CryptoKey,
        data: ArrayBufferViewOrArrayBuffer,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm and key be the algorithm and key parameters passed to the sign()
        // method, respectively.
        // NOTE: We did that in method parameter.

        // Step 2. Let data be the result of getting a copy of the bytes held by the data parameter
        // passed to the sign() method.
        let data = match &data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        // Step 3. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set
        // to algorithm and op set to "sign".
        // Step 4. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let promise = Promise::new_in_realm(cx);
        let normalized_algorithm = match normalize_algorithm::<SignOperation>(cx, &algorithm) {
            Ok(normalized_algorithm) => normalized_algorithm,
            Err(error) => {
                promise.reject_error(error, CanGc::from_cx(cx));
                return promise;
            },
        };

        // Step 5. Let realm be the relevant realm of this.
        // Step 6. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 4.

        // Step 7. Return promise and perform the remaining steps in parallel.
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(sign: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();

                // Step 8. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 9. If the name member of normalizedAlgorithm is not equal to the name
                // attribute of the [[algorithm]] internal slot of key then throw an
                // InvalidAccessError.
                if normalized_algorithm.name() != key.algorithm().name() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 10. If the [[usages]] internal slot of key does not contain an entry that
                // is "sign", then throw an InvalidAccessError.
                if !key.usages().contains(&KeyUsage::Sign) {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 11. Let signature be the result of performing the sign operation specified
                // by normalizedAlgorithm using key and algorithm and with data as message.
                let signature = match normalized_algorithm.sign(&key, &data) {
                    Ok(signature) => signature,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 12. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 13. Let result be the result of creating an ArrayBuffer in realm,
                // containing signature.
                // Step 14. Resolve promise with result.
                subtle.resolve_promise_with_data(promise, signature);
            }));
        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-verify>
    fn Verify(
        &self,
        cx: &mut CurrentRealm,
        algorithm: AlgorithmIdentifier,
        key: &CryptoKey,
        signature: ArrayBufferViewOrArrayBuffer,
        data: ArrayBufferViewOrArrayBuffer,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm and key be the algorithm and key parameters passed to the verify()
        // method, respectively.
        // NOTE: We did that in method parameter.

        // Step 2. Let signature be the result of getting a copy of the bytes held by the signature
        // parameter passed to the verify() method.
        let signature = match &signature {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        // Step 3. Let data be the result of getting a copy of the bytes held by the data parameter
        // passed to the verify() method.
        let data = match &data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        // Step 4. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set to
        // algorithm and op set to "verify".
        // Step 5. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let promise = Promise::new_in_realm(cx);
        let normalized_algorithm = match normalize_algorithm::<VerifyOperation>(cx, &algorithm) {
            Ok(algorithm) => algorithm,
            Err(error) => {
                promise.reject_error(error, CanGc::from_cx(cx));
                return promise;
            },
        };

        // Step 6. Let realm be the relevant realm of this.
        // Step 7. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 5.

        // Step 8. Return promise and perform the remaining steps in parallel.
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(sign: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();

                // Step 9. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 10. If the name member of normalizedAlgorithm is not equal to the name
                // attribute of the [[algorithm]] internal slot of key then throw an
                // InvalidAccessError.
                if normalized_algorithm.name() != key.algorithm().name() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 11. If the [[usages]] internal slot of key does not contain an entry that
                // is "verify", then throw an InvalidAccessError.
                if !key.usages().contains(&KeyUsage::Verify) {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 12. Let result be the result of performing the verify operation specified
                // by normalizedAlgorithm using key, algorithm and signature and with data as
                // message.
                let result = match normalized_algorithm.verify(&key, &data, &signature) {
                    Ok(result) => result,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 13. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 14. Resolve promise with result.
                subtle.resolve_promise_with_bool(promise, result);
            }));
        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-digest>
    fn Digest(
        &self,
        cx: &mut CurrentRealm,
        algorithm: AlgorithmIdentifier,
        data: ArrayBufferViewOrArrayBuffer,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm be the algorithm parameter passed to the digest() method.
        // NOTE: We did that in method parameter.

        // Step 2. Let data be the result of getting a copy of the bytes held by the
        // data parameter passed to the digest() method.
        let data = match data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        // Step 3. Let normalizedAlgorithm be the result of normalizing an algorithm,
        // with alg set to algorithm and op set to "digest".
        // Step 4. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let promise = Promise::new_in_realm(cx);
        let normalized_algorithm = match normalize_algorithm::<DigestOperation>(cx, &algorithm) {
            Ok(normalized_algorithm) => normalized_algorithm,
            Err(error) => {
                promise.reject_error(error, CanGc::from_cx(cx));
                return promise;
            },
        };

        // Step 5. Let realm be the relevant realm of this.
        // Step 6. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 3.

        // Step 7. Return promise and perform the remaining steps in parallel.
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(digest_: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();

                // Step 8. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 9. Let digest be the result of performing the digest operation specified by
                // normalizedAlgorithm using algorithm, with data as message.
                let digest = match normalized_algorithm.digest(&data) {
                    Ok(digest) => digest,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    }
                };

                // Step 10. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 11. Let result be the result of creating an ArrayBuffer in realm,
                // containing digest.
                // Step 12. Resolve promise with result.
                subtle.resolve_promise_with_data(promise, digest);
            }));
        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-generateKey>
    fn GenerateKey(
        &self,
        cx: &mut CurrentRealm,
        algorithm: AlgorithmIdentifier,
        extractable: bool,
        key_usages: Vec<KeyUsage>,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm, extractable and usages be the algorithm, extractable and
        // keyUsages parameters passed to the generateKey() method, respectively.

        // Step 2. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set
        // to algorithm and op set to "generateKey".
        // Step 3. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let promise = Promise::new_in_realm(cx);
        let normalized_algorithm = match normalize_algorithm::<GenerateKeyOperation>(cx, &algorithm)
        {
            Ok(normalized_algorithm) => normalized_algorithm,
            Err(error) => {
                promise.reject_error(error, CanGc::from_cx(cx));
                return promise;
            },
        };

        // Step 4. Let realm be the relevant realm of this.
        // Step 5. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 3.

        // Step 6. Return promise and perform the remaining steps in parallel.
        let trusted_subtle = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(generate_key: move |cx| {
                let subtle = trusted_subtle.root();
                let promise = trusted_promise.root();

                // Step 7. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 8. Let result be the result of performing the generate key operation
                // specified by normalizedAlgorithm using algorithm, extractable and usages.
                let result = match normalized_algorithm.generate_key(
                    cx,
                    &subtle.global(),
                    extractable,
                    key_usages,
                ) {
                    Ok(result) => result,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    }
                };

                // Step 9.
                // If result is a CryptoKey object:
                //     If the [[type]] internal slot of result is "secret" or "private" and usages
                //     is empty, then throw a SyntaxError.
                // If result is a CryptoKeyPair object:
                //     If the [[usages]] internal slot of the privateKey attribute of result is the
                //     empty sequence, then throw a SyntaxError.
                match &result {
                    CryptoKeyOrCryptoKeyPair::CryptoKey(crpyto_key) => {
                        if matches!(crpyto_key.Type(), KeyType::Secret | KeyType::Private)
                            && crpyto_key.usages().is_empty()
                        {
                            subtle.reject_promise_with_error(promise, Error::Syntax(None));
                            return;
                        }
                    },
                    CryptoKeyOrCryptoKeyPair::CryptoKeyPair(crypto_key_pair) => {
                        if crypto_key_pair
                            .privateKey
                            .as_ref()
                            .is_none_or(|private_key| private_key.usages().is_empty())
                        {
                            subtle.reject_promise_with_error(promise, Error::Syntax(None));
                            return;
                        }
                    }
                };

                // Step 10. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 11. Let result be the result of converting result to an ECMAScript Object
                // in realm, as defined by [WebIDL].
                // Step 12. Resolve promise with result.
                match result {
                    CryptoKeyOrCryptoKeyPair::CryptoKey(key) => {
                        subtle.resolve_promise_with_key(promise, key);
                    },
                    CryptoKeyOrCryptoKeyPair::CryptoKeyPair(key_pair) => {
                        subtle.resolve_promise_with_key_pair(promise, key_pair);
                    },
                }
            }));

        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-deriveKey>
    fn DeriveKey(
        &self,
        cx: &mut CurrentRealm,
        algorithm: AlgorithmIdentifier,
        base_key: &CryptoKey,
        derived_key_type: AlgorithmIdentifier,
        extractable: bool,
        usages: Vec<KeyUsage>,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm, baseKey, derivedKeyType, extractable and usages be the algorithm,
        // baseKey, derivedKeyType, extractable and keyUsages parameters passed to the deriveKey()
        // method, respectively.
        // NOTE: We did that in method parameter.

        // Step 2. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set
        // to algorithm and op set to "deriveBits".
        // Step 3. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let promise = Promise::new_in_realm(cx);
        let normalized_algorithm = match normalize_algorithm::<DeriveBitsOperation>(cx, &algorithm)
        {
            Ok(normalized_algorithm) => normalized_algorithm,
            Err(error) => {
                promise.reject_error(error, CanGc::from_cx(cx));
                return promise;
            },
        };

        // Step 4. Let normalizedDerivedKeyAlgorithmImport be the result of normalizing an
        // algorithm, with alg set to derivedKeyType and op set to "importKey".
        // Step 5. If an error occurred, return a Promise rejected with
        // normalizedDerivedKeyAlgorithmImport.
        let normalized_derived_key_algorithm_import =
            match normalize_algorithm::<ImportKeyOperation>(cx, &derived_key_type) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, CanGc::from_cx(cx));
                    return promise;
                },
            };

        // Step 6. Let normalizedDerivedKeyAlgorithmLength be the result of normalizing an
        // algorithm, with alg set to derivedKeyType and op set to "get key length".
        // Step 7. If an error occurred, return a Promise rejected with
        // normalizedDerivedKeyAlgorithmLength.
        let normalized_derived_key_algorithm_length =
            match normalize_algorithm::<GetKeyLengthOperation>(cx, &derived_key_type) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, CanGc::from_cx(cx));
                    return promise;
                },
            };

        // Step 8. Let realm be the relevant realm of this.
        // Step 9. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 3.

        // Step 10. Return promise and perform the remaining steps in parallel.
        let trusted_subtle = Trusted::new(self);
        let trusted_base_key = Trusted::new(base_key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global().task_manager().dom_manipulation_task_source().queue(
            task!(derive_key: move |cx| {
                let subtle = trusted_subtle.root();
                let base_key = trusted_base_key.root();
                let promise = trusted_promise.root();

                // Step 11. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 12. If the name member of normalizedAlgorithm is not equal to the name
                // attribute of the [[algorithm]] internal slot of baseKey then throw an
                // InvalidAccessError.
                if normalized_algorithm.name() != base_key.algorithm().name() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 13. If the [[usages]] internal slot of baseKey does not contain an entry
                // that is "deriveKey", then throw an InvalidAccessError.
                if !base_key.usages().contains(&KeyUsage::DeriveKey) {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 14. Let length be the result of performing the get key length algorithm
                // specified by normalizedDerivedKeyAlgorithmLength using derivedKeyType.
                let length = match normalized_derived_key_algorithm_length.get_key_length() {
                    Ok(length) => length,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    }
                };

                // Step 15. Let secret be the result of performing the derive bits operation
                // specified by normalizedAlgorithm using key, algorithm and length.
                let secret = match normalized_algorithm.derive_bits(&base_key, length) {
                    Ok(secret) => secret,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    }
                };

                // Step 16. Let result be the result of performing the import key operation
                // specified by normalizedDerivedKeyAlgorithmImport using "raw" as format, secret
                // as keyData, derivedKeyType as algorithm and using extractable and usages.
                // NOTE: Use "raw-secret" instead, according to
                // <https://wicg.github.io/webcrypto-modern-algos/#subtlecrypto-interface-keyformat>.
                let result = match normalized_derived_key_algorithm_import.import_key(
                    cx,
                    &subtle.global(),
                    KeyFormat::Raw_secret,
                    &secret,
                    extractable,
                    usages.clone(),
                ) {
                    Ok(algorithm) => algorithm,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 17. If the [[type]] internal slot of result is "secret" or "private" and
                // usages is empty, then throw a SyntaxError.
                if matches!(result.Type(), KeyType::Secret | KeyType::Private) && usages.is_empty() {
                    subtle.reject_promise_with_error(promise, Error::Syntax(None));
                    return;
                }

                // Step 18. Set the [[extractable]] internal slot of result to extractable.
                // Step 19. Set the [[usages]] internal slot of result to the normalized value of
                // usages.
                // NOTE: Done by normalized_derived_key_algorithm_import.import_key in Step 16.

                // Step 20. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 20. Let result be the result of converting result to an ECMAScript Object
                // in realm, as defined by [WebIDL].
                // Step 20. Resolve promise with result.
                subtle.resolve_promise_with_key(promise, result);
            }),
        );
        promise
    }

    /// <https://w3c.github.io/webcrypto/#dfn-SubtleCrypto-method-deriveBits>
    fn DeriveBits(
        &self,
        cx: &mut CurrentRealm,
        algorithm: AlgorithmIdentifier,
        base_key: &CryptoKey,
        length: Option<u32>,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm, baseKey and length, be the algorithm, baseKey and length
        // parameters passed to the deriveBits() method, respectively.
        // NOTE: We did that in method parameter.

        // Step 2. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set
        // to algorithm and op set to "deriveBits".
        // Step 3. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let promise = Promise::new_in_realm(cx);
        let normalized_algorithm = match normalize_algorithm::<DeriveBitsOperation>(cx, &algorithm)
        {
            Ok(normalized_algorithm) => normalized_algorithm,
            Err(error) => {
                promise.reject_error(error, CanGc::from_cx(cx));
                return promise;
            },
        };

        // Step 4. Let realm be the relevant realm of this.
        // Step 5. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 3.

        // Step 5. Return promise and perform the remaining steps in parallel.
        let trsuted_subtle = Trusted::new(self);
        let trusted_base_key = Trusted::new(base_key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(import_key: move || {
                let subtle = trsuted_subtle.root();
                let base_key = trusted_base_key.root();
                let promise = trusted_promise.root();

                // Step 7. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 8. If the name member of normalizedAlgorithm is not equal to the name
                // attribute of the [[algorithm]] internal slot of baseKey then throw an
                // InvalidAccessError.
                if normalized_algorithm.name() != base_key.algorithm().name() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 9. If the [[usages]] internal slot of baseKey does not contain an entry
                // that is "deriveBits", then throw an InvalidAccessError.
                if !base_key.usages().contains(&KeyUsage::DeriveBits) {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 10. Let bits be the result of performing the derive bits operation
                // specified by normalizedAlgorithm using baseKey, algorithm and length.
                let bits = match normalized_algorithm.derive_bits(&base_key, length) {
                    Ok(bits) => bits,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    }
                };

                // Step 11. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 12. Let result be the result of creating an ArrayBuffer in realm,
                // containing bits.
                // Step 13. Resolve promise with result.
                subtle.resolve_promise_with_data(promise, bits);
            }));
        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-importKey>
    fn ImportKey(
        &self,
        cx: &mut CurrentRealm,
        format: KeyFormat,
        key_data: ArrayBufferViewOrArrayBufferOrJsonWebKey,
        algorithm: AlgorithmIdentifier,
        extractable: bool,
        key_usages: Vec<KeyUsage>,
    ) -> Rc<Promise> {
        // Step 1. Let format, algorithm, extractable and usages, be the format, algorithm,
        // extractable and keyUsages parameters passed to the importKey() method, respectively.

        // Step 5. Let realm be the relevant realm of this.
        // Step 6. Let promise be a new Promise.
        let promise = Promise::new_in_realm(cx);

        // Step 2.
        let key_data = match format {
            // If format is equal to the string "jwk":
            KeyFormat::Jwk => {
                match key_data {
                    ArrayBufferViewOrArrayBufferOrJsonWebKey::ArrayBufferView(_) |
                    ArrayBufferViewOrArrayBufferOrJsonWebKey::ArrayBuffer(_) => {
                        // Step 2.1. If the keyData parameter passed to the importKey() method is
                        // not a JsonWebKey dictionary, throw a TypeError.
                        promise.reject_error(
                            Error::Type(c"The keyData type does not match the format".to_owned()),
                            CanGc::from_cx(cx),
                        );
                        return promise;
                    },

                    ArrayBufferViewOrArrayBufferOrJsonWebKey::JsonWebKey(jwk) => {
                        // Step 2.2. Let keyData be the keyData parameter passed to the importKey()
                        // method.
                        //
                        // NOTE: Serialize JsonWebKey throught stringifying it.
                        // JsonWebKey::stringify internally relies on ToJSON, so it will raise an
                        // exception when a JS error is thrown. When this happens, we report the
                        // error.
                        match jwk.stringify(cx) {
                            Ok(stringified) => stringified.as_bytes().to_vec(),
                            Err(error) => {
                                promise.reject_error(error, CanGc::from_cx(cx));
                                return promise;
                            },
                        }
                    },
                }
            },
            // Otherwise:
            _ => {
                match key_data {
                    // Step 2.1. If the keyData parameter passed to the importKey() method is a
                    // JsonWebKey dictionary, throw a TypeError.
                    ArrayBufferViewOrArrayBufferOrJsonWebKey::JsonWebKey(_) => {
                        promise.reject_error(
                            Error::Type(c"The keyData type does not match the format".to_owned()),
                            CanGc::from_cx(cx),
                        );
                        return promise;
                    },

                    // Step 2.2. Let keyData be the result of getting a copy of the bytes held by
                    // the keyData parameter passed to the importKey() method.
                    ArrayBufferViewOrArrayBufferOrJsonWebKey::ArrayBufferView(view) => {
                        view.to_vec()
                    },
                    ArrayBufferViewOrArrayBufferOrJsonWebKey::ArrayBuffer(buffer) => {
                        buffer.to_vec()
                    },
                }
            },
        };

        // Step 3. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set
        // to algorithm and op set to "importKey".
        // Step 4. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let normalized_algorithm = match normalize_algorithm::<ImportKeyOperation>(cx, &algorithm) {
            Ok(algorithm) => algorithm,
            Err(error) => {
                promise.reject_error(error, CanGc::from_cx(cx));
                return promise;
            },
        };

        // Step 7. Return promise and perform the remaining steps in parallel.
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(import_key: move |cx| {
                let subtle = this.root();
                let promise = trusted_promise.root();

                // Step 8. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 9. Let result be the CryptoKey object that results from performing the
                // import key operation specified by normalizedAlgorithm using keyData, algorithm,
                // format, extractable and usages.
                let result = match normalized_algorithm.import_key(
                    cx,
                    &subtle.global(),
                    format,
                    &key_data,
                    extractable,
                    key_usages.clone(),
                ) {
                    Ok(key) => key,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 10. If the [[type]] internal slot of result is "secret" or "private" and
                // usages is empty, then throw a SyntaxError.
                if matches!(result.Type(), KeyType::Secret | KeyType::Private) && key_usages.is_empty() {
                    subtle.reject_promise_with_error(promise, Error::Syntax(None));
                    return;
                }

                // Step 11. Set the [[extractable]] internal slot of result to extractable.
                result.set_extractable(extractable);

                // Step 12. Set the [[usages]] internal slot of result to the normalized value of usages.
                result.set_usages(cx, &key_usages);

                // Step 13. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 14. Let result be the result of converting result to an ECMAScript Object
                // in realm, as defined by [WebIDL].
                // Step 15. Resolve promise with result.
                subtle.resolve_promise_with_key(promise, result);
            }));

        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-exportKey>
    fn ExportKey(&self, cx: &mut CurrentRealm, format: KeyFormat, key: &CryptoKey) -> Rc<Promise> {
        // Step 1. Let format and key be the format and key parameters passed to the exportKey()
        // method, respectively.
        // NOTE: We did that in method parameter.

        // Step 2. Let realm be the relevant realm of this.
        // Step 3. Let promise be a new Promise.
        let promise = Promise::new_in_realm(cx);

        // Step 4. Return promise and perform the remaining steps in parallel.
        let trusted_subtle = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(export_key: move |cx| {
                let subtle = trusted_subtle.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();

                // Step 5. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 6. If the name member of the [[algorithm]] internal slot of key does not
                // identify a registered algorithm that supports the export key operation, then
                // throw a NotSupportedError.
                //
                // NOTE: We rely on [`normalize_algorithm`] to check whether the algorithm supports
                // the export key operation.
                let export_key_algorithm = match normalize_algorithm::<ExportKeyOperation>(
                    cx,
                    &AlgorithmIdentifier::String(DOMString::from(key.algorithm().name())),
                ) {
                    Ok(normalized_algorithm) => normalized_algorithm,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 7. If the [[extractable]] internal slot of key is false, then throw an
                // InvalidAccessError.
                if !key.Extractable() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 8. Let result be the result of performing the export key operation
                // specified by the [[algorithm]] internal slot of key using key and format.
                let result = match export_key_algorithm.export_key(format, &key) {
                    Ok(exported_key) => exported_key,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 9. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 10.
                // If format is equal to the string "jwk":
                //     Let result be the result of converting result to an ECMAScript Object in
                //     realm, as defined by [WebIDL].
                // Otherwise:
                //     Let result be the result of creating an ArrayBuffer in realm, containing
                //     result.
                // Step 11. Resolve promise with result.
                // NOTE: We determine the format by pattern matching on result, which is an
                // ExportedKey enum.
                match result {
                    ExportedKey::Bytes(bytes) => {
                        subtle.resolve_promise_with_data(promise, bytes);
                    },
                    ExportedKey::Jwk(jwk) => {
                        subtle.resolve_promise_with_jwk(cx, promise, jwk);
                    },
                }
            }));
        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-wrapKey>
    fn WrapKey(
        &self,
        cx: &mut CurrentRealm,
        format: KeyFormat,
        key: &CryptoKey,
        wrapping_key: &CryptoKey,
        algorithm: AlgorithmIdentifier,
    ) -> Rc<Promise> {
        // Step 1. Let format, key, wrappingKey and algorithm be the format, key, wrappingKey and
        // wrapAlgorithm parameters passed to the wrapKey() method, respectively.
        // NOTE: We did that in method parameter.

        // Step 2. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set
        // to algorithm and op set to "wrapKey".
        // Step 3. If an error occurred, let normalizedAlgorithm be the result of normalizing an
        // algorithm, with alg set to algorithm and op set to "encrypt".
        // Step 4. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        enum WrapKeyAlgorithmOrEncryptAlgorithm {
            WrapKeyAlgorithm(WrapKeyAlgorithm),
            EncryptAlgorithm(EncryptAlgorithm),
        }
        let normalized_algorithm = if let Ok(algorithm) =
            normalize_algorithm::<WrapKeyOperation>(cx, &algorithm)
        {
            WrapKeyAlgorithmOrEncryptAlgorithm::WrapKeyAlgorithm(algorithm)
        } else {
            match normalize_algorithm::<EncryptOperation>(cx, &algorithm) {
                Ok(algorithm) => WrapKeyAlgorithmOrEncryptAlgorithm::EncryptAlgorithm(algorithm),
                Err(error) => {
                    let promise = Promise::new_in_realm(cx);
                    promise.reject_error(error, CanGc::from_cx(cx));
                    return promise;
                },
            }
        };

        // Step 5. Let realm be the relevant realm of this.
        // Step 6. Let promise be a new Promise.
        let promise = Promise::new_in_realm(cx);

        // Step 7. Return promise and perform the remaining steps in parallel.
        let trusted_subtle = Trusted::new(self);
        let trusted_key = Trusted::new(key);
        let trusted_wrapping_key = Trusted::new(wrapping_key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(wrap_key: move |cx| {
                let subtle = trusted_subtle.root();
                let key = trusted_key.root();
                let wrapping_key = trusted_wrapping_key.root();
                let promise = trusted_promise.root();

                // Step 8. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 9. If the name member of normalizedAlgorithm is not equal to the name
                // attribute of the [[algorithm]] internal slot of wrappingKey then throw an
                // InvalidAccessError.
                let normalized_algorithm_name = match &normalized_algorithm {
                    WrapKeyAlgorithmOrEncryptAlgorithm::WrapKeyAlgorithm(algorithm) => {
                        algorithm.name()
                    },
                    WrapKeyAlgorithmOrEncryptAlgorithm::EncryptAlgorithm(algorithm) => {
                        algorithm.name()
                    },
                };
                if normalized_algorithm_name != wrapping_key.algorithm().name() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 10. If the [[usages]] internal slot of wrappingKey does not contain an
                // entry that is "wrapKey", then throw an InvalidAccessError.
                if !wrapping_key.usages().contains(&KeyUsage::WrapKey) {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 11. If the algorithm identified by the [[algorithm]] internal slot of key
                // does not support the export key operation, then throw a NotSupportedError.
                //
                // NOTE: We rely on [`normalize_algorithm`] to check whether the algorithm supports
                // the export key operation.
                let export_key_algorithm = match normalize_algorithm::<ExportKeyOperation>(
                    cx,
                    &AlgorithmIdentifier::String(DOMString::from(key.algorithm().name())),
                ) {
                    Ok(normalized_algorithm) => normalized_algorithm,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 12. If the [[extractable]] internal slot of key is false, then throw an
                // InvalidAccessError.
                if !key.Extractable() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 13. Let exportedKey be the result of performing the export key operation
                // specified by the [[algorithm]] internal slot of key using key and format.
                let exported_key = match export_key_algorithm.export_key(format, &key) {
                    Ok(exported_key) => exported_key,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 14.
                // If format is equal to the string "jwk":
                //     Step 14.1. Let json be the result of representing exportedKey as a UTF-16
                //     string conforming to the JSON grammar; for example, by executing the
                //     JSON.stringify algorithm specified in [ECMA-262] in the context of a new
                //     global object.
                //     Step 14.2. Let bytes be the result of UTF-8 encoding json.
                // Otherwise:
                //     Let bytes be exportedKey.
                // NOTE: We determine the format by pattern matching on result, which is an
                // ExportedKey enum.
                let bytes = match exported_key {
                    ExportedKey::Bytes(bytes) => bytes,
                    ExportedKey::Jwk(jwk) => match jwk.stringify(cx) {
                        Ok(stringified_jwk) => stringified_jwk.as_bytes().to_vec(),
                        Err(error) => {
                            subtle.reject_promise_with_error(promise, error);
                            return;
                        },
                    },
                };

                // Step 15.
                // If normalizedAlgorithm supports the wrap key operation:
                //     Let result be the result of performing the wrap key operation specified by
                //     normalizedAlgorithm using algorithm, wrappingKey as key and bytes as
                //     plaintext.
                // Otherwise, if normalizedAlgorithm supports the encrypt operation:
                //     Let result be the result of performing the encrypt operation specified by
                //     normalizedAlgorithm using algorithm, wrappingKey as key and bytes as
                //     plaintext.
                // Otherwise:
                //     throw a NotSupportedError.
                let result = match normalized_algorithm {
                    WrapKeyAlgorithmOrEncryptAlgorithm::WrapKeyAlgorithm(algorithm) => {
                        algorithm.wrap_key(&wrapping_key, &bytes)
                    },
                    WrapKeyAlgorithmOrEncryptAlgorithm::EncryptAlgorithm(algorithm) => {
                        algorithm.encrypt(&wrapping_key, &bytes)
                    },
                };
                let result = match result {
                    Ok(result) => result,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 16. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 17. Let result be the result of creating an ArrayBuffer in realm,
                // containing result.
                // Step 18. Resolve promise with result.
                subtle.resolve_promise_with_data(promise, result);
            }));
        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-unwrapKey>
    fn UnwrapKey(
        &self,
        cx: &mut CurrentRealm,
        format: KeyFormat,
        wrapped_key: ArrayBufferViewOrArrayBuffer,
        unwrapping_key: &CryptoKey,
        algorithm: AlgorithmIdentifier,
        unwrapped_key_algorithm: AlgorithmIdentifier,
        extractable: bool,
        usages: Vec<KeyUsage>,
    ) -> Rc<Promise> {
        // Step 1. Let format, unwrappingKey, algorithm, unwrappedKeyAlgorithm, extractable and
        // usages, be the format, unwrappingKey, unwrapAlgorithm, unwrappedKeyAlgorithm,
        // extractable and keyUsages parameters passed to the unwrapKey() method, respectively.
        // NOTE: We did that in method parameter.

        // Step 2. Let wrappedKey be the result of getting a copy of the bytes held by the
        // wrappedKey parameter passed to the unwrapKey() method.
        let wrapped_key = match wrapped_key {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        // Step 3. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set
        // to algorithm and op set to "unwrapKey".
        // Step 4. If an error occurred, let normalizedAlgorithm be the result of normalizing an
        // algorithm, with alg set to algorithm and op set to "decrypt".
        // Step 5. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        enum UnwrapKeyAlgorithmOrDecryptAlgorithm {
            UnwrapKeyAlgorithm(UnwrapKeyAlgorithm),
            DecryptAlgorithm(DecryptAlgorithm),
        }
        let normalized_algorithm = if let Ok(algorithm) =
            normalize_algorithm::<UnwrapKeyOperation>(cx, &algorithm)
        {
            UnwrapKeyAlgorithmOrDecryptAlgorithm::UnwrapKeyAlgorithm(algorithm)
        } else {
            match normalize_algorithm::<DecryptOperation>(cx, &algorithm) {
                Ok(algorithm) => UnwrapKeyAlgorithmOrDecryptAlgorithm::DecryptAlgorithm(algorithm),
                Err(error) => {
                    let promise = Promise::new_in_realm(cx);
                    promise.reject_error(error, CanGc::from_cx(cx));
                    return promise;
                },
            }
        };

        // Step 6. Let normalizedKeyAlgorithm be the result of normalizing an algorithm, with alg
        // set to unwrappedKeyAlgorithm and op set to "importKey".
        // Step 7. If an error occurred, return a Promise rejected with normalizedKeyAlgorithm.
        let normalized_key_algorithm =
            match normalize_algorithm::<ImportKeyOperation>(cx, &unwrapped_key_algorithm) {
                Ok(algorithm) => algorithm,
                Err(error) => {
                    let promise = Promise::new_in_realm(cx);
                    promise.reject_error(error, CanGc::from_cx(cx));
                    return promise;
                },
            };

        // Step 8. Let realm be the relevant realm of this.
        // Step 9. Let promise be a new Promise.
        let promise = Promise::new_in_realm(cx);

        // Step 10. Return promise and perform the remaining steps in parallel.
        let trusted_subtle = Trusted::new(self);
        let trusted_unwrapping_key = Trusted::new(unwrapping_key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global().task_manager().dom_manipulation_task_source().queue(
            task!(unwrap_key: move |cx| {
                let subtle = trusted_subtle.root();
                let unwrapping_key = trusted_unwrapping_key.root();
                let promise = trusted_promise.root();

                // Step 11. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 12. If the name member of normalizedAlgorithm is not equal to the name
                // attribute of the [[algorithm]] internal slot of unwrappingKey then throw an
                // InvalidAccessError.
                let normalized_algorithm_name = match &normalized_algorithm {
                    UnwrapKeyAlgorithmOrDecryptAlgorithm::UnwrapKeyAlgorithm(algorithm) => {
                        algorithm.name()
                    },
                    UnwrapKeyAlgorithmOrDecryptAlgorithm::DecryptAlgorithm(algorithm) => {
                        algorithm.name()
                    },
                };
                if normalized_algorithm_name != unwrapping_key.algorithm().name() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 13. If the [[usages]] internal slot of unwrappingKey does not contain an
                // entry that is "unwrapKey", then throw an InvalidAccessError.
                if !unwrapping_key.usages().contains(&KeyUsage::UnwrapKey) {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 14.
                // If normalizedAlgorithm supports an unwrap key operation:
                //     Let bytes be the result of performing the unwrap key operation specified by
                //     normalizedAlgorithm using algorithm, unwrappingKey as key and wrappedKey as
                //     ciphertext.
                // Otherwise, if normalizedAlgorithm supports a decrypt operation:
                //     Let bytes be the result of performing the decrypt operation specified by
                //     normalizedAlgorithm using algorithm, unwrappingKey as key and wrappedKey as
                //     ciphertext.
                // Otherwise:
                //     throw a NotSupportedError.
                let bytes = match normalized_algorithm {
                    UnwrapKeyAlgorithmOrDecryptAlgorithm::UnwrapKeyAlgorithm(algorithm) => {
                        algorithm.unwrap_key(&unwrapping_key, &wrapped_key)
                    },
                    UnwrapKeyAlgorithmOrDecryptAlgorithm::DecryptAlgorithm(algorithm) => {
                        algorithm.decrypt(&unwrapping_key, &wrapped_key)
                    },
                };
                let bytes = match bytes {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 15.
                // If format is equal to the string "jwk":
                //     Let key be the result of executing the parse a JWK algorithm, with bytes as
                //     the data to be parsed.
                //     NOTE: We only parse bytes by executing the parse a JWK algorithm, but keep
                //     it as raw bytes for later steps, instead of converting it to a JsonWebKey
                //     dictionary.
                //
                // Otherwise:
                //     Let key be bytes.
                if format == KeyFormat::Jwk {
                    if let Err(error) = JsonWebKey::parse(cx, &bytes) {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    }
                }
                let key = bytes;

                // Step 16. Let result be the result of performing the import key operation
                // specified by normalizedKeyAlgorithm using unwrappedKeyAlgorithm as algorithm,
                // format, usages and extractable and with key as keyData.
                let result = match normalized_key_algorithm.import_key(
                    cx,
                    &subtle.global(),
                    format,
                    &key,
                    extractable,
                    usages.clone(),
                ) {
                    Ok(result) => result,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 17. If the [[type]] internal slot of result is "secret" or "private" and
                // usages is empty, then throw a SyntaxError.
                if matches!(result.Type(), KeyType::Secret | KeyType::Private) && usages.is_empty() {
                    subtle.reject_promise_with_error(promise, Error::Syntax(None));
                    return;
                }

                // Step 18. Set the [[extractable]] internal slot of result to extractable.
                // Step 19. Set the [[usages]] internal slot of result to the normalized value of
                // usages.
                // NOTE: Done by normalized_algorithm.import_key in Step 16.

                // Step 20. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 21. Let result be the result of converting result to an ECMAScript Object
                // in realm, as defined by [WebIDL].
                // Step 22. Resolve promise with result.
                subtle.resolve_promise_with_key(promise, result);
            }),
        );
        promise
    }

    /// <https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-encapsulateKey>
    fn EncapsulateKey(
        &self,
        cx: &mut CurrentRealm,
        encapsulation_algorithm: AlgorithmIdentifier,
        encapsulation_key: &CryptoKey,
        shared_key_algorithm: AlgorithmIdentifier,
        extractable: bool,
        usages: Vec<KeyUsage>,
    ) -> Rc<Promise> {
        // Step 1. Let encapsulationAlgorithm, encapsulationKey, sharedKeyAlgorithm, extractable
        // and usages be the encapsulationAlgorithm, encapsulationKey, sharedKeyAlgorithm,
        // extractable and keyUsages parameters passed to the encapsulateKey() method,
        // respectively.

        // Step 2. Let normalizedEncapsulationAlgorithm be the result of normalizing an algorithm,
        // with alg set to encapsulationAlgorithm and op set to "encapsulate".
        // Step 3. If an error occurred, return a Promise rejected with
        // normalizedEncapsulationAlgorithm.
        let promise = Promise::new_in_realm(cx);
        let normalized_encapsulation_algorithm =
            match normalize_algorithm::<EncapsulateOperation>(cx, &encapsulation_algorithm) {
                Ok(algorithm) => algorithm,
                Err(error) => {
                    promise.reject_error(error, CanGc::from_cx(cx));
                    return promise;
                },
            };

        // Step 4. Let normalizedSharedKeyAlgorithm be the result of normalizing an algorithm, with
        // alg set to sharedKeyAlgorithm and op set to "importKey".
        // Step 5. If an error occurred, return a Promise rejected with
        // normalizedSharedKeyAlgorithm.
        let normalized_shared_key_algorithm =
            match normalize_algorithm::<ImportKeyOperation>(cx, &shared_key_algorithm) {
                Ok(algorithm) => algorithm,
                Err(error) => {
                    promise.reject_error(error, CanGc::from_cx(cx));
                    return promise;
                },
            };

        // Step 6. Let realm be the relevant realm of this.
        // Step 7. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 3.

        // Step 8. Return promise and perform the remaining steps in parallel.
        let trusted_subtle = Trusted::new(self);
        let trusted_encapsulated_key = Trusted::new(encapsulation_key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global().task_manager().dom_manipulation_task_source().queue(
            task!(encapsulate_keys: move |cx| {
                let subtle = trusted_subtle.root();
                let encapsulation_key = trusted_encapsulated_key.root();
                let promise = trusted_promise.root();

                // Step 9. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 10. If the name member of normalizedEncapsulationAlgorithm is not equal to
                // the name attribute of the [[algorithm]] internal slot of encapsulationKey then
                // throw an InvalidAccessError.
                if normalized_encapsulation_algorithm.name() != encapsulation_key.algorithm().name() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(Some(
                        "[[algorithm]] internal slot of encapsulationKey is not equal to \
                        normalizedEncapsulationAlgorithm".to_string(),
                    )));
                    return;
                }

                // Step 11. If the [[usages]] internal slot of encapsulationKey does not contain an
                // entry that is "encapsulateKey", then throw an InvalidAccessError.
                if !encapsulation_key.usages().contains(&KeyUsage::EncapsulateKey) {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(Some(
                        "[[usages]] internal slot of encapsulationKey does not contain an \
                        entry that is \"encapsulateBits\"".to_string(),
                    )));
                    return;
                }

                // Step 12. Let encapsulatedBits be the result of performing the encapsulate
                // operation specified by the [[algorithm]] internal slot of encapsulationKey using
                // encapsulationKey.
                // NOTE: Step 10 guarantees normalizedEncapsulationAlgorithm specifies the same
                // algorithm as the [[algorithm]] internal slot of encapsulationKey.
                let encapsulated_bits_result =
                    normalized_encapsulation_algorithm.encapsulate(&encapsulation_key);
                let encapsulated_bits = match encapsulated_bits_result {
                    Ok(encapsulated_bits) => encapsulated_bits,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 13. Let sharedKey be the result of performing the import key operation
                // specified by normalizedSharedKeyAlgorithm using "raw-secret" as format, the
                // sharedKey field of encapsulatedBits as keyData, sharedKeyAlgorithm as algorithm
                // and using extractable and usages.
                // Step 14. Set the [[extractable]] internal slot of sharedKey to extractable.
                // Step 15. Set the [[usages]] internal slot of sharedKey to the normalized value
                // of usages.
                let encapsulated_shared_key = match &encapsulated_bits.shared_key {
                    Some(shared_key) => shared_key,
                    None => {
                        subtle.reject_promise_with_error(promise, Error::Operation(Some(
                            "Shared key is missing in the result of the encapsulate operation"
                                .to_string())));
                        return;
                    },
                };
                let shared_key_result = normalized_shared_key_algorithm.import_key(
                    cx,
                    &subtle.global(),
                    KeyFormat::Raw_secret,
                    encapsulated_shared_key,
                    extractable,
                    usages.clone(),
                );
                let shared_key = match shared_key_result {
                    Ok(shared_key) => shared_key,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 16. Let encapsulatedKey be a new EncapsulatedKey dictionary with sharedKey
                // set to sharedKey and ciphertext set to the ciphertext field of encapsulatedBits.
                let encapsulated_key = SubtleEncapsulatedKey {
                    shared_key: Some(Trusted::new(&shared_key)),
                    ciphertext:encapsulated_bits.ciphertext,
                };

                // Step 17. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 18. Let result be the result of converting encapsulatedKey to an ECMAScript
                // Object in realm, as defined by [WebIDL].
                // Step 19. Resolve promise with result.
                subtle.resolve_promise_with_encapsulated_key(promise, encapsulated_key);
            })
        );
        promise
    }

    /// <https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-encapsulateBits>
    fn EncapsulateBits(
        &self,
        cx: &mut CurrentRealm,
        encapsulation_algorithm: AlgorithmIdentifier,
        encapsulation_key: &CryptoKey,
    ) -> Rc<Promise> {
        // Step 1. Let encapsulationAlgorithm and encapsulationKey be the encapsulationAlgorithm
        // and encapsulationKey parameters passed to the encapsulateBits() method, respectively.

        // Step 2. Let normalizedEncapsulationAlgorithm be the result of normalizing an algorithm,
        // with alg set to encapsulationAlgorithm and op set to "encapsulate".
        // Step 3. If an error occurred, return a Promise rejected with
        // normalizedEncapsulationAlgorithm.
        let promise = Promise::new_in_realm(cx);
        let normalized_encapsulation_algorithm =
            match normalize_algorithm::<EncapsulateOperation>(cx, &encapsulation_algorithm) {
                Ok(algorithm) => algorithm,
                Err(error) => {
                    promise.reject_error(error, CanGc::from_cx(cx));
                    return promise;
                },
            };

        // Step 4. Let realm be the relevant realm of this.
        // Step 5. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 3.

        // Step 6. Return promise and perform the remaining steps in parallel.
        let trusted_subtle = Trusted::new(self);
        let trusted_encapsulation_key = Trusted::new(encapsulation_key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global().task_manager().dom_manipulation_task_source().queue(
            task!(derive_key: move || {
                let subtle = trusted_subtle.root();
                let encapsulation_key = trusted_encapsulation_key.root();
                let promise = trusted_promise.root();

                // Step 7. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 8. If the name member of normalizedEncapsulationAlgorithm is not equal to
                // the name attribute of the [[algorithm]] internal slot of encapsulationKey then
                // throw an InvalidAccessError.
                if normalized_encapsulation_algorithm.name() != encapsulation_key.algorithm().name() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(Some(
                        "[[algorithm]] internal slot of encapsulationKey is not equal to \
                        normalizedEncapsulationAlgorithm".to_string(),
                    )));
                    return;
                }

                // Step 9. If the [[usages]] internal slot of encapsulationKey does not contain an
                // entry that is "encapsulateBits", then throw an InvalidAccessError.
                if !encapsulation_key.usages().contains(&KeyUsage::EncapsulateBits) {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(Some(
                        "[[usages]] internal slot of encapsulationKey does not contain an \
                        entry that is \"encapsulateBits\"".to_string(),
                    )));
                    return;
                }

                // Step 10. Let encapsulatedBits be the result of performing the encapsulate
                // operation specified by the [[algorithm]] internal slot of encapsulationKey using
                // encapsulationKey.
                // NOTE: Step 8 guarantees normalizedEncapsulationAlgorithm specifies the same
                // algorithm as the [[algorithm]] internal slot of encapsulationKey.
                let encapsulated_bits =
                    match normalized_encapsulation_algorithm.encapsulate(&encapsulation_key) {
                        Ok(encapsulated_bits) => encapsulated_bits,
                        Err(error) => {
                            subtle.reject_promise_with_error(promise, error);
                            return;
                        },
                    };

                // Step 11. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 12. Let result be the result of converting encapsulatedBits to an
                // ECMAScript Object in realm, as defined by [WebIDL].
                // Step 13. Resolve promise with result.
                subtle.resolve_promise_with_encapsulated_bits(promise, encapsulated_bits);
            }),
        );
        promise
    }

    /// <https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-decapsulateKey>
    fn DecapsulateKey(
        &self,
        cx: &mut CurrentRealm,
        decapsulation_algorithm: AlgorithmIdentifier,
        decapsulation_key: &CryptoKey,
        ciphertext: ArrayBufferViewOrArrayBuffer,
        shared_key_algorithm: AlgorithmIdentifier,
        extractable: bool,
        usages: Vec<KeyUsage>,
    ) -> Rc<Promise> {
        // Step 1. Let decapsulationAlgorithm, decapsulationKey, sharedKeyAlgorithm, extractable
        // and usages be the decapsulationAlgorithm, decapsulationKey, sharedKeyAlgorithm,
        // extractable and keyUsages parameters passed to the decapsulateKey() method,
        // respectively.

        // Step 2. Let ciphertext be the result of getting a copy of the bytes held by the
        // ciphertext parameter passed to the decapsulateKey() method.
        let ciphertext = match ciphertext {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        // Step 3. Let normalizedDecapsulationAlgorithm be the result of normalizing an algorithm,
        // with alg set to decapsulationAlgorithm and op set to "decapsulate".
        // Step 4. If an error occurred, return a Promise rejected with
        // normalizedDecapsulationAlgorithm.
        let promise = Promise::new_in_realm(cx);
        let normalized_decapsulation_algorithm =
            match normalize_algorithm::<DecapsulateOperation>(cx, &decapsulation_algorithm) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, CanGc::from_cx(cx));
                    return promise;
                },
            };

        // Step 5. Let normalizedSharedKeyAlgorithm be the result of normalizing an algorithm, with
        // alg set to sharedKeyAlgorithm and op set to "importKey".
        // Step 6. If an error occurred, return a Promise rejected with
        // normalizedSharedKeyAlgorithm.
        let normalized_shared_key_algorithm =
            match normalize_algorithm::<ImportKeyOperation>(cx, &shared_key_algorithm) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, CanGc::from_cx(cx));
                    return promise;
                },
            };

        // Step 7. Let realm be the relevant realm of this.
        // Step 8. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 4.

        // Step 9. Return promise and perform the remaining steps in parallel.
        let trusted_subtle = Trusted::new(self);
        let trusted_decapsulation_key = Trusted::new(decapsulation_key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(decapsulate_key: move |cx| {
                let subtle = trusted_subtle.root();
                let promise = trusted_promise.root();
                let decapsulation_key = trusted_decapsulation_key.root();

                // Step 10. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 11. If the name member of normalizedDecapsulationAlgorithm is not equal to
                // the name attribute of the [[algorithm]] internal slot of decapsulationKey then
                // throw an InvalidAccessError.
                if normalized_decapsulation_algorithm.name() != decapsulation_key.algorithm().name() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(Some(
                        "[[algorithm]] internal slot of decapsulationKey is not equal to \
                        normalizedDecapsulationAlgorithm".to_string()
                    )));
                    return;
                }

                // Step 12. If the [[usages]] internal slot of decapsulationKey does not contain an
                // entry that is "decapsulateKey", then throw an InvalidAccessError.
                if !decapsulation_key.usages().contains(&KeyUsage::DecapsulateKey) {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(Some(
                        "[[usages]] internal slot of decapsulationKey does not contain an \
                        entry that is \"decapsulateBits\"".to_string(),
                    )));
                    return;
                }

                // Step 13. Let decapsulatedBits be the result of performing the decapsulate
                // operation specified by the [[algorithm]] internal slot of decapsulationKey using
                // decapsulationKey and ciphertext.
                // NOTE: Step 11 guarantees normalizedDecapsulationAlgorithm specifies the same
                // algorithm as the [[algorithm]] internal slot of decapsulationKey.
                let decapsulated_bits_result =
                    normalized_decapsulation_algorithm.decapsulate(&decapsulation_key, &ciphertext);
                let decapsulated_bits = match decapsulated_bits_result {
                    Ok(decapsulated_bits) => decapsulated_bits,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };


                // Step 14. Let sharedKey be the result of performing the import key operation
                // specified by normalizedSharedKeyAlgorithm using "raw-secret" as format, the
                // decapsulatedBits as keyData, sharedKeyAlgorithm as algorithm and using
                // extractable and usages.
                // Step 15. Set the [[extractable]] internal slot of sharedKey to extractable.
                // Step 16. Set the [[usages]] internal slot of sharedKey to the normalized value
                // of usages.
                let shared_key_result = normalized_shared_key_algorithm.import_key(
                    cx,
                    &subtle.global(),
                    KeyFormat::Raw_secret,
                    &decapsulated_bits,
                    extractable,
                    usages.clone(),
                );
                let shared_key = match shared_key_result {
                    Ok(shared_key) => shared_key,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };


                // Step 17. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 18. Let result be the result of converting sharedKey to an ECMAScript
                // Object in realm, as defined by [WebIDL].
                // Step 19. Resolve promise with result.
                subtle.resolve_promise_with_key(promise, shared_key);
            }));
        promise
    }

    /// <https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-decapsulateBits>
    fn DecapsulateBits(
        &self,
        cx: &mut CurrentRealm,
        decapsulation_algorithm: AlgorithmIdentifier,
        decapsulation_key: &CryptoKey,
        ciphertext: ArrayBufferViewOrArrayBuffer,
    ) -> Rc<Promise> {
        // Step 1. Let decapsulationAlgorithm and decapsulationKey be the decapsulationAlgorithm
        // and decapsulationKey parameters passed to the decapsulateBits() method, respectively.

        // Step 2. Let ciphertext be the result of getting a copy of the bytes held by the
        // ciphertext parameter passed to the decapsulateBits() method.
        let ciphertext = match ciphertext {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        // Step 3. Let normalizedDecapsulationAlgorithm be the result of normalizing an algorithm,
        // with alg set to decapsulationAlgorithm and op set to "decapsulate".
        // Step 4. If an error occurred, return a Promise rejected with
        // normalizedDecapsulationAlgorithm.
        let promise = Promise::new_in_realm(cx);
        let normalized_decapsulation_algorithm =
            match normalize_algorithm::<DecapsulateOperation>(cx, &decapsulation_algorithm) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, CanGc::from_cx(cx));
                    return promise;
                },
            };

        // Step 5. Let realm be the relevant realm of this.
        // Step 6. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 4.

        // Step 7. Return promise and perform the remaining steps in parallel.
        let trusted_subtle = Trusted::new(self);
        let trusted_decapsulation_key = Trusted::new(decapsulation_key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(decapsulate_bits: move || {
                let subtle = trusted_subtle.root();
                let promise = trusted_promise.root();
                let decapsulation_key = trusted_decapsulation_key.root();

                // Step 8. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 9. If the name member of normalizedDecapsulationAlgorithm is not equal to
                // the name attribute of the [[algorithm]] internal slot of decapsulationKey then
                // throw an InvalidAccessError.
                if normalized_decapsulation_algorithm.name() != decapsulation_key.algorithm().name() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(Some(
                        "[[algorithm]] internal slot of decapsulationKey is not equal to \
                        normalizedDecapsulationAlgorithm".to_string()
                    )));
                    return;
                }

                // Step 10. If the [[usages]] internal slot of decapsulationKey does not contain an
                // entry that is "decapsulateBits", then throw an InvalidAccessError.
                if !decapsulation_key.usages().contains(&KeyUsage::DecapsulateBits) {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(Some(
                        "[[usages]] internal slot of decapsulationKey does not contain an \
                        entry that is \"decapsulateBits\"".to_string(),
                    )));
                    return;
                }

                // Step 11. Let decapsulatedBits be the result of performing the decapsulate
                // operation specified by the [[algorithm]] internal slot of decapsulationKey using
                // decapsulationKey and ciphertext.
                // NOTE: Step 9 guarantees normalizedDecapsulationAlgorithm specifies the same
                // algorithm as the [[algorithm]] internal slot of decapsulationKey.
                let decapsulated_bits_result =
                    normalized_decapsulation_algorithm.decapsulate(&decapsulation_key, &ciphertext);
                let decapsulated_bits = match decapsulated_bits_result {
                    Ok(decapsulated_bits) => decapsulated_bits,
                    Err(error) => {
                        subtle.reject_promise_with_error(promise, error);
                        return;
                    },
                };

                // Step 12. Queue a global task on the crypto task source, given realm's global
                // object, to perform the remaining steps.
                // Step 13. Let result be the result of creating an ArrayBuffer in realm,
                // containing decapsulatedBits.
                // Step 14. Resolve promise with result.
                subtle.resolve_promise_with_data(promise, decapsulated_bits);
            }));
        promise
    }
}

/// Alternative to std::convert::TryFrom, with `&mut js::context::JSContext`
trait TryFromWithCx<T>: Sized {
    type Error;

    fn try_from_with_cx(value: T, cx: &mut js::context::JSContext) -> Result<Self, Self::Error>;
}

/// Alternative to std::convert::TryInto, with `&mut js::context::JSContext`
trait TryIntoWithCx<T>: Sized {
    type Error;

    fn try_into_with_cx(self, cx: &mut js::context::JSContext) -> Result<T, Self::Error>;
}

impl<T, U> TryIntoWithCx<U> for T
where
    U: TryFromWithCx<T>,
{
    type Error = U::Error;

    fn try_into_with_cx(self, cx: &mut js::context::JSContext) -> Result<U, Self::Error> {
        U::try_from_with_cx(self, cx)
    }
}

// These "subtle" structs are proxies for the codegen'd dicts which don't hold a DOMString
// so they can be sent safely when running steps in parallel.

/// <https://w3c.github.io/webcrypto/#dfn-Algorithm>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleAlgorithm {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<Algorithm>(cx, value)?;

        Ok(SubtleAlgorithm {
            name: dictionary.name.to_string(),
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-KeyAlgorithm>
#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct SubtleKeyAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-keyalgorithm-name>
    name: String,
}

impl SafeToJSValConvertible for SubtleKeyAlgorithm {
    fn safe_to_jsval(&self, cx: JSContext, rval: MutableHandleValue, can_gc: CanGc) {
        let dictionary = KeyAlgorithm {
            name: self.name.clone().into(),
        };
        dictionary.safe_to_jsval(cx, rval, can_gc);
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-RsaHashedKeyGenParams>
#[derive(Clone, MallocSizeOf)]
pub(crate) struct SubtleRsaHashedKeyGenParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-RsaKeyGenParams-modulusLength>
    modulus_length: u32,

    /// <https://w3c.github.io/webcrypto/#dfn-RsaKeyGenParams-publicExponent>
    public_exponent: Vec<u8>,

    /// <https://w3c.github.io/webcrypto/#dfn-RsaHashedKeyGenParams-hash>
    hash: DigestAlgorithm,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleRsaHashedKeyGenParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary =
            dictionary_from_jsval::<RootedTraceableBox<RsaHashedKeyGenParams>>(cx, value)?;

        Ok(SubtleRsaHashedKeyGenParams {
            name: dictionary.parent.parent.name.to_string(),
            modulus_length: dictionary.parent.modulusLength,
            public_exponent: dictionary.parent.publicExponent.to_vec(),
            hash: normalize_algorithm::<DigestOperation>(cx, &dictionary.hash)?,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-RsaHashedKeyAlgorithm>
#[derive(Clone, MallocSizeOf)]
pub(crate) struct SubtleRsaHashedKeyAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-keyalgorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-RsaKeyAlgorithm-modulusLength>
    modulus_length: u32,

    /// <https://w3c.github.io/webcrypto/#dfn-RsaKeyAlgorithm-publicExponent>
    public_exponent: Vec<u8>,

    /// <https://w3c.github.io/webcrypto/#dfn-RsaHashedKeyAlgorithm-hash>
    hash: DigestAlgorithm,
}

impl SafeToJSValConvertible for SubtleRsaHashedKeyAlgorithm {
    fn safe_to_jsval(&self, cx: JSContext, rval: MutableHandleValue, can_gc: CanGc) {
        rooted!(in(*cx) let mut js_object = ptr::null_mut::<JSObject>());
        let public_exponent =
            create_buffer_source(cx, &self.public_exponent, js_object.handle_mut(), can_gc)
                .expect("Fail to convert publicExponent to Uint8Array");
        let key_algorithm = KeyAlgorithm {
            name: self.name.clone().into(),
        };
        let rsa_key_algorithm = RootedTraceableBox::new(RsaKeyAlgorithm {
            parent: key_algorithm,
            modulusLength: self.modulus_length,
            publicExponent: public_exponent,
        });
        let rsa_hashed_key_algorithm = RootedTraceableBox::new(RsaHashedKeyAlgorithm {
            parent: rsa_key_algorithm,
            hash: KeyAlgorithm {
                name: self.hash.name().into(),
            },
        });
        rsa_hashed_key_algorithm.safe_to_jsval(cx, rval, can_gc);
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-RsaHashedImportParams>
#[derive(Clone, MallocSizeOf)]
struct SubtleRsaHashedImportParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-RsaHashedImportParams-hash>
    hash: DigestAlgorithm,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleRsaHashedImportParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary =
            dictionary_from_jsval::<RootedTraceableBox<RsaHashedImportParams>>(cx, value)?;

        Ok(SubtleRsaHashedImportParams {
            name: dictionary.parent.name.to_string(),
            hash: normalize_algorithm::<DigestOperation>(cx, &dictionary.hash)?,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-RsaPssParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleRsaPssParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-RsaPssParams-saltLength>
    salt_length: u32,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleRsaPssParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RsaPssParams>(cx, value)?;

        Ok(SubtleRsaPssParams {
            name: dictionary.parent.name.to_string(),
            salt_length: dictionary.saltLength,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-RsaOaepParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleRsaOaepParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,
    /// <https://w3c.github.io/webcrypto/#dfn-RsaOaepParams-label>
    label: Option<Vec<u8>>,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleRsaOaepParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RootedTraceableBox<RsaOaepParams>>(cx, value)?;

        let label = dictionary.label.as_ref().map(|label| match label {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        });

        Ok(SubtleRsaOaepParams {
            name: dictionary.parent.name.to_string(),
            label,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-EcdsaParams>
#[derive(Clone, MallocSizeOf)]
struct SubtleEcdsaParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-EcdsaParams-hash>
    hash: DigestAlgorithm,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleEcdsaParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RootedTraceableBox<EcdsaParams>>(cx, value)?;

        Ok(SubtleEcdsaParams {
            name: dictionary.parent.name.to_string(),
            hash: normalize_algorithm::<DigestOperation>(cx, &dictionary.hash)?,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-EcKeyGenParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleEcKeyGenParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-EcKeyGenParams-namedCurve>
    named_curve: String,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleEcKeyGenParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<EcKeyGenParams>(cx, value)?;

        Ok(SubtleEcKeyGenParams {
            name: dictionary.parent.name.to_string(),
            named_curve: dictionary.namedCurve.to_string(),
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-EcKeyAlgorithm>
#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct SubtleEcKeyAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-keyalgorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-EcKeyAlgorithm-namedCurve>
    named_curve: String,
}

impl SafeToJSValConvertible for SubtleEcKeyAlgorithm {
    fn safe_to_jsval(&self, cx: JSContext, rval: MutableHandleValue, can_gc: CanGc) {
        let parent = KeyAlgorithm {
            name: self.name.clone().into(),
        };
        let dictionary = EcKeyAlgorithm {
            parent,
            namedCurve: self.named_curve.clone().into(),
        };
        dictionary.safe_to_jsval(cx, rval, can_gc);
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-EcKeyImportParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleEcKeyImportParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-EcKeyImportParams-namedCurve>
    named_curve: String,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleEcKeyImportParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<EcKeyImportParams>(cx, value)?;

        Ok(SubtleEcKeyImportParams {
            name: dictionary.parent.name.to_string(),
            named_curve: dictionary.namedCurve.to_string(),
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-EcdhKeyDeriveParams>
#[derive(Clone, MallocSizeOf)]
struct SubtleEcdhKeyDeriveParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-EcdhKeyDeriveParams-public>
    public: Trusted<CryptoKey>,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleEcdhKeyDeriveParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<EcdhKeyDeriveParams>(cx, value)?;

        Ok(SubtleEcdhKeyDeriveParams {
            name: dictionary.parent.name.to_string(),
            public: Trusted::new(&dictionary.public),
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-AesCtrParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleAesCtrParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-AesCtrParams-counter>
    counter: Vec<u8>,

    /// <https://w3c.github.io/webcrypto/#dfn-AesCtrParams-length>
    length: u8,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleAesCtrParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RootedTraceableBox<AesCtrParams>>(cx, value)?;

        let counter = match &dictionary.counter {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        Ok(SubtleAesCtrParams {
            name: dictionary.parent.name.to_string(),
            counter,
            length: dictionary.length,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-AesKeyAlgorithm>
#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct SubtleAesKeyAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-keyalgorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-AesKeyAlgorithm-length>
    length: u16,
}

impl SafeToJSValConvertible for SubtleAesKeyAlgorithm {
    fn safe_to_jsval(&self, cx: JSContext, rval: MutableHandleValue, can_gc: CanGc) {
        let parent = KeyAlgorithm {
            name: self.name.clone().into(),
        };
        let dictionary = AesKeyAlgorithm {
            parent,
            length: self.length,
        };
        dictionary.safe_to_jsval(cx, rval, can_gc);
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-AesKeyGenParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleAesKeyGenParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-AesKeyGenParams-length>
    length: u16,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleAesKeyGenParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<AesKeyGenParams>(cx, value)?;

        Ok(SubtleAesKeyGenParams {
            name: dictionary.parent.name.to_string(),
            length: dictionary.length,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-AesDerivedKeyParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleAesDerivedKeyParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-AesDerivedKeyParams-length>
    length: u16,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleAesDerivedKeyParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<AesDerivedKeyParams>(cx, value)?;

        Ok(SubtleAesDerivedKeyParams {
            name: dictionary.parent.name.to_string(),
            length: dictionary.length,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-AesCbcParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleAesCbcParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-AesCbcParams-iv>
    iv: Vec<u8>,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleAesCbcParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RootedTraceableBox<AesCbcParams>>(cx, value)?;

        let iv = match &dictionary.iv {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        Ok(SubtleAesCbcParams {
            name: dictionary.parent.name.to_string(),
            iv,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-AesGcmParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleAesGcmParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-AesGcmParams-iv>
    iv: Vec<u8>,

    /// <https://w3c.github.io/webcrypto/#dfn-AesGcmParams-additionalData>
    additional_data: Option<Vec<u8>>,

    /// <https://w3c.github.io/webcrypto/#dfn-AesGcmParams-tagLength>
    tag_length: Option<u8>,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleAesGcmParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RootedTraceableBox<AesGcmParams>>(cx, value)?;

        let iv = match &dictionary.iv {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        let additional_data = dictionary.additionalData.as_ref().map(|data| match data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        });

        Ok(SubtleAesGcmParams {
            name: dictionary.parent.name.to_string(),
            iv,
            additional_data,
            tag_length: dictionary.tagLength,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-HmacImportParams>
#[derive(Clone, MallocSizeOf)]
struct SubtleHmacImportParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacImportParams-hash>
    hash: DigestAlgorithm,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacImportParams-length>
    length: Option<u32>,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleHmacImportParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RootedTraceableBox<HmacImportParams>>(cx, value)?;

        Ok(SubtleHmacImportParams {
            name: dictionary.parent.name.to_string(),
            hash: normalize_algorithm::<DigestOperation>(cx, &dictionary.hash)?,
            length: dictionary.length,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-HmacKeyAlgorithm>
#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct SubtleHmacKeyAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-keyalgorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyAlgorithm-hash>
    hash: SubtleKeyAlgorithm,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyGenParams-length>
    length: u32,
}

impl SafeToJSValConvertible for SubtleHmacKeyAlgorithm {
    fn safe_to_jsval(&self, cx: JSContext, rval: MutableHandleValue, can_gc: CanGc) {
        let parent = KeyAlgorithm {
            name: self.name.clone().into(),
        };
        let hash = KeyAlgorithm {
            name: self.hash.name.clone().into(),
        };
        let dictionary = HmacKeyAlgorithm {
            parent,
            hash,
            length: self.length,
        };
        dictionary.safe_to_jsval(cx, rval, can_gc);
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-HmacKeyGenParams>
#[derive(Clone, MallocSizeOf)]
struct SubtleHmacKeyGenParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyGenParams-hash>
    hash: DigestAlgorithm,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyGenParams-length>
    length: Option<u32>,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleHmacKeyGenParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RootedTraceableBox<HmacKeyGenParams>>(cx, value)?;

        Ok(SubtleHmacKeyGenParams {
            name: dictionary.parent.name.to_string(),
            hash: normalize_algorithm::<DigestOperation>(cx, &dictionary.hash)?,
            length: dictionary.length,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-HkdfParams>
#[derive(Clone, MallocSizeOf)]
pub(crate) struct SubtleHkdfParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-HkdfParams-hash>
    hash: DigestAlgorithm,

    /// <https://w3c.github.io/webcrypto/#dfn-HkdfParams-salt>
    salt: Vec<u8>,

    /// <https://w3c.github.io/webcrypto/#dfn-HkdfParams-info>
    info: Vec<u8>,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleHkdfParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RootedTraceableBox<HkdfParams>>(cx, value)?;

        let salt = match &dictionary.salt {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        let info = match &dictionary.info {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        Ok(SubtleHkdfParams {
            name: dictionary.parent.name.to_string(),
            hash: normalize_algorithm::<DigestOperation>(cx, &dictionary.hash)?,
            salt,
            info,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params>
#[derive(Clone, MallocSizeOf)]
pub(crate) struct SubtlePbkdf2Params {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params-salt>
    salt: Vec<u8>,

    /// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params-iterations>
    iterations: u32,

    /// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params-hash>
    hash: DigestAlgorithm,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtlePbkdf2Params {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RootedTraceableBox<Pbkdf2Params>>(cx, value)?;

        let salt = match &dictionary.salt {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        Ok(SubtlePbkdf2Params {
            name: dictionary.parent.name.to_string(),
            salt,
            iterations: dictionary.iterations,
            hash: normalize_algorithm::<DigestOperation>(cx, &dictionary.hash)?,
        })
    }
}

/// <https://wicg.github.io/webcrypto-modern-algos/#dfn-ContextParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleContextParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-ContextParams-context>
    context: Option<Vec<u8>>,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleContextParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RootedTraceableBox<ContextParams>>(cx, value)?;

        let context = dictionary.context.as_ref().map(|context| match context {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        });

        Ok(SubtleContextParams {
            name: dictionary.parent.name.to_string(),
            context,
        })
    }
}

/// <https://wicg.github.io/webcrypto-modern-algos/#dfn-AeadParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleAeadParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-AeadParams-iv>
    iv: Vec<u8>,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-AeadParams-additionalData>
    additional_data: Option<Vec<u8>>,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-AeadParams-tagLength>
    tag_length: Option<u8>,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleAeadParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RootedTraceableBox<AeadParams>>(cx, value)?;

        let iv = match &dictionary.iv {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        let additional_data = dictionary.additionalData.as_ref().map(|data| match data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        });

        Ok(SubtleAeadParams {
            name: dictionary.parent.name.to_string(),
            iv,
            additional_data,
            tag_length: dictionary.tagLength,
        })
    }
}

/// <https://wicg.github.io/webcrypto-modern-algos/#dfn-CShakeParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleCShakeParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-CShakeParams-length>
    length: u32,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-CShakeParams-functionName>
    function_name: Option<Vec<u8>>,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-CShakeParams-customization>
    customization: Option<Vec<u8>>,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleCShakeParams {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RootedTraceableBox<CShakeParams>>(cx, value)?;

        let function_name =
            dictionary
                .functionName
                .as_ref()
                .map(|function_name| match function_name {
                    ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
                    ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
                });
        let customization =
            dictionary
                .customization
                .as_ref()
                .map(|customization| match customization {
                    ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
                    ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
                });

        Ok(SubtleCShakeParams {
            name: dictionary.parent.name.to_string(),
            length: dictionary.length,
            function_name,
            customization,
        })
    }
}

/// <https://wicg.github.io/webcrypto-modern-algos/#dfn-Argon2Params>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleArgon2Params {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-Argon2Params-nonce>
    nonce: Vec<u8>,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-Argon2Params-parallelism>
    parallelism: u32,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-Argon2Params-memory>
    memory: u32,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-Argon2Params-passes>
    passes: u32,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-Argon2Params-version>
    version: Option<u8>,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-Argon2Params-secretValue>
    secret_value: Option<Vec<u8>>,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-Argon2Params-associatedData>
    associated_data: Option<Vec<u8>>,
}

impl<'a> TryFromWithCx<HandleValue<'a>> for SubtleArgon2Params {
    type Error = Error;

    fn try_from_with_cx(
        value: HandleValue<'a>,
        cx: &mut js::context::JSContext,
    ) -> Result<Self, Self::Error> {
        let dictionary = dictionary_from_jsval::<RootedTraceableBox<Argon2Params>>(cx, value)?;

        let nonce = match &dictionary.nonce {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        let secret_value = dictionary
            .secretValue
            .as_ref()
            .map(|secret_value| match secret_value {
                ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
                ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
            });
        let associated_data =
            dictionary
                .associatedData
                .as_ref()
                .map(|associated_data| match associated_data {
                    ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
                    ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
                });

        Ok(SubtleArgon2Params {
            name: dictionary.parent.name.to_string(),
            nonce,
            parallelism: dictionary.parallelism,
            memory: dictionary.memory,
            passes: dictionary.passes,
            version: dictionary.version,
            secret_value,
            associated_data,
        })
    }
}

/// <https://wicg.github.io/webcrypto-modern-algos/#dfn-EncapsulatedKey>
struct SubtleEncapsulatedKey {
    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-EncapsulatedKey-sharedKey>
    shared_key: Option<Trusted<CryptoKey>>,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-EncapsulatedKey-ciphertext>
    ciphertext: Option<Vec<u8>>,
}

impl SafeToJSValConvertible for SubtleEncapsulatedKey {
    fn safe_to_jsval(&self, cx: JSContext, rval: MutableHandleValue, can_gc: CanGc) {
        let shared_key = self.shared_key.as_ref().map(|shared_key| shared_key.root());
        let ciphertext = self.ciphertext.as_ref().map(|data| {
            rooted!(in(*cx) let mut ciphertext_ptr = ptr::null_mut::<JSObject>());
            create_buffer_source::<ArrayBufferU8>(cx, data, ciphertext_ptr.handle_mut(), can_gc)
                .expect("Failed to convert ciphertext to ArrayBufferU8")
        });
        let encapsulated_key = RootedTraceableBox::new(EncapsulatedKey {
            sharedKey: shared_key,
            ciphertext,
        });
        encapsulated_key.safe_to_jsval(cx, rval, can_gc);
    }
}

/// <https://wicg.github.io/webcrypto-modern-algos/#dfn-EncapsulatedBits>
struct SubtleEncapsulatedBits {
    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-EncapsulatedBits-sharedKey>
    shared_key: Option<Vec<u8>>,

    /// <https://wicg.github.io/webcrypto-modern-algos/#dfn-EncapsulatedBits-ciphertext>
    ciphertext: Option<Vec<u8>>,
}

impl SafeToJSValConvertible for SubtleEncapsulatedBits {
    fn safe_to_jsval(&self, cx: JSContext, rval: MutableHandleValue, can_gc: CanGc) {
        let shared_key = self.shared_key.as_ref().map(|data| {
            rooted!(in(*cx) let mut shared_key_ptr = ptr::null_mut::<JSObject>());
            create_buffer_source::<ArrayBufferU8>(cx, data, shared_key_ptr.handle_mut(), can_gc)
                .expect("Failed to convert shared key to ArrayBufferU8")
        });
        let ciphertext = self.ciphertext.as_ref().map(|data| {
            rooted!(in(*cx) let mut ciphertext_ptr = ptr::null_mut::<JSObject>());
            create_buffer_source::<ArrayBufferU8>(cx, data, ciphertext_ptr.handle_mut(), can_gc)
                .expect("Failed to convert ciphertext to ArrayBufferU8")
        });
        let encapsulated_bits = RootedTraceableBox::new(EncapsulatedBits {
            sharedKey: shared_key,
            ciphertext,
        });
        encapsulated_bits.safe_to_jsval(cx, rval, can_gc);
    }
}

/// Helper to abstract the conversion process of a JS value into many different WebIDL dictionaries.
fn dictionary_from_jsval<T>(cx: &mut js::context::JSContext, value: HandleValue) -> Fallible<T>
where
    T: SafeFromJSValConvertible<Config = ()>,
{
    let conversion = T::safe_from_jsval(cx.into(), value, (), CanGc::from_cx(cx))
        .map_err(|_| Error::JSFailed)?;
    match conversion {
        ConversionResult::Success(dictionary) => Ok(dictionary),
        ConversionResult::Failure(error) => Err(Error::Type(error.into_owned())),
    }
}

/// The returned type of the successful export key operation. `Bytes` should be used when the key
/// is exported in "raw", "spki" or "pkcs8" format. `Jwk` should be used when the key is exported
/// in "jwk" format.
enum ExportedKey {
    Bytes(Vec<u8>),
    Jwk(Box<JsonWebKey>),
}

/// Union type of KeyAlgorithm and IDL dictionary types derived from it. Note that we actually use
/// our "subtle" structs of the corresponding IDL dictionary types so that they can be easily
/// passed to another threads.
#[derive(Clone, MallocSizeOf)]
#[expect(clippy::enum_variant_names)]
pub(crate) enum KeyAlgorithmAndDerivatives {
    KeyAlgorithm(SubtleKeyAlgorithm),
    RsaHashedKeyAlgorithm(SubtleRsaHashedKeyAlgorithm),
    EcKeyAlgorithm(SubtleEcKeyAlgorithm),
    AesKeyAlgorithm(SubtleAesKeyAlgorithm),
    HmacKeyAlgorithm(SubtleHmacKeyAlgorithm),
}

impl KeyAlgorithmAndDerivatives {
    fn name(&self) -> &str {
        match self {
            KeyAlgorithmAndDerivatives::KeyAlgorithm(algo) => &algo.name,
            KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algo) => &algo.name,
            KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algo) => &algo.name,
            KeyAlgorithmAndDerivatives::AesKeyAlgorithm(algo) => &algo.name,
            KeyAlgorithmAndDerivatives::HmacKeyAlgorithm(algo) => &algo.name,
        }
    }
}

impl SafeToJSValConvertible for KeyAlgorithmAndDerivatives {
    fn safe_to_jsval(&self, cx: JSContext, rval: MutableHandleValue, can_gc: CanGc) {
        match self {
            KeyAlgorithmAndDerivatives::KeyAlgorithm(algo) => algo.safe_to_jsval(cx, rval, can_gc),
            KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algo) => {
                algo.safe_to_jsval(cx, rval, can_gc)
            },
            KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algo) => {
                algo.safe_to_jsval(cx, rval, can_gc)
            },
            KeyAlgorithmAndDerivatives::AesKeyAlgorithm(algo) => {
                algo.safe_to_jsval(cx, rval, can_gc)
            },
            KeyAlgorithmAndDerivatives::HmacKeyAlgorithm(algo) => {
                algo.safe_to_jsval(cx, rval, can_gc)
            },
        }
    }
}

#[derive(Clone, Copy)]
enum JwkStringField {
    X,
    Y,
    D,
    N,
    E,
    P,
    Q,
    DP,
    DQ,
    QI,
    K,
    Priv,
    Pub,
}

impl Display for JwkStringField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let field_name = match self {
            JwkStringField::X => "x",
            JwkStringField::Y => "y",
            JwkStringField::D => "d",
            JwkStringField::N => "n",
            JwkStringField::E => "e",
            JwkStringField::P => "q",
            JwkStringField::Q => "q",
            JwkStringField::DP => "dp",
            JwkStringField::DQ => "dq",
            JwkStringField::QI => "qi",
            JwkStringField::K => "k",
            JwkStringField::Priv => "priv",
            JwkStringField::Pub => "pub",
        };
        write!(f, "{}", field_name)
    }
}

trait JsonWebKeyExt {
    fn parse(cx: &mut js::context::JSContext, data: &[u8]) -> Result<JsonWebKey, Error>;
    fn stringify(&self, cx: &mut js::context::JSContext) -> Result<DOMString, Error>;
    fn get_usages_from_key_ops(&self) -> Result<Vec<KeyUsage>, Error>;
    fn check_key_ops(&self, specified_usages: &[KeyUsage]) -> Result<(), Error>;
    fn set_key_ops(&mut self, usages: Vec<KeyUsage>);
    fn encode_string_field(&mut self, field: JwkStringField, data: &[u8]);
    fn decode_optional_string_field(&self, field: JwkStringField)
    -> Result<Option<Vec<u8>>, Error>;
    fn decode_required_string_field(&self, field: JwkStringField) -> Result<Vec<u8>, Error>;
    fn decode_primes_from_oth_field(&self, primes: &mut Vec<Vec<u8>>) -> Result<(), Error>;
}

impl JsonWebKeyExt for JsonWebKey {
    /// <https://w3c.github.io/webcrypto/#concept-parse-a-jwk>
    #[expect(unsafe_code)]
    fn parse(cx: &mut js::context::JSContext, data: &[u8]) -> Result<JsonWebKey, Error> {
        // Step 1. Let data be the sequence of bytes to be parsed.
        // (It is given as a method paramter.)

        // Step 2. Let json be the Unicode string that results from interpreting data according to UTF-8.
        let json = String::from_utf8_lossy(data);

        // Step 3. Convert json to UTF-16.
        let json: Vec<_> = json.encode_utf16().collect();

        // Step 4. Let result be the object literal that results from executing the JSON.parse
        // internal function in the context of a new global object, with text argument set to a
        // JavaScript String containing json.
        rooted!(&in(cx) let mut result = UndefinedValue());
        unsafe {
            if !JS_ParseJSON(cx, json.as_ptr(), json.len() as u32, result.handle_mut()) {
                return Err(Error::JSFailed);
            }
        }

        // Step 5. Let key be the result of converting result to the IDL dictionary type of JsonWebKey.
        let key = match JsonWebKey::new(cx.into(), result.handle(), CanGc::from_cx(cx)) {
            Ok(ConversionResult::Success(key)) => key,
            Ok(ConversionResult::Failure(error)) => {
                return Err(Error::Type(error.into_owned()));
            },
            Err(()) => {
                return Err(Error::JSFailed);
            },
        };

        // Step 6. If the kty field of key is not defined, then throw a DataError.
        if key.kty.is_none() {
            return Err(Error::Data(None));
        }

        // Step 7. Result key.
        Ok(key)
    }

    /// Convert a JsonWebKey value to DOMString. We first convert the JsonWebKey value to
    /// JavaScript value, and then serialize it by performing steps in
    /// <https://infra.spec.whatwg.org/#serialize-a-javascript-value-to-a-json-string>. This acts
    /// like the opposite of JsonWebKey::parse if you further convert the stringified result to
    /// bytes.
    fn stringify(&self, cx: &mut js::context::JSContext) -> Result<DOMString, Error> {
        rooted!(&in(cx) let mut data = UndefinedValue());
        self.safe_to_jsval(cx.into(), data.handle_mut(), CanGc::from_cx(cx));
        serialize_jsval_to_json_utf8(cx.into(), data.handle())
    }

    fn get_usages_from_key_ops(&self) -> Result<Vec<KeyUsage>, Error> {
        let mut usages = vec![];
        for op in self.key_ops.as_ref().ok_or(Error::Data(None))? {
            usages.push(KeyUsage::from_str(&op.str()).map_err(|_| Error::Data(None))?);
        }
        Ok(usages)
    }

    /// If the key_ops field of jwk is present, and is invalid according to the requirements of
    /// JSON Web Key [JWK] or does not contain all of the specified usages values, then throw a
    /// DataError.
    fn check_key_ops(&self, specified_usages: &[KeyUsage]) -> Result<(), Error> {
        // If the key_ops field of jwk is present,
        if let Some(ref key_ops) = self.key_ops {
            // and is invalid according to the requirements of JSON Web Key [JWK]:
            // 1. Duplicate key operation values MUST NOT be present in the array.
            if key_ops
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len() <
                key_ops.len()
            {
                return Err(Error::Data(None));
            }
            // 2. The "use" and "key_ops" JWK members SHOULD NOT be used together; however, if both
            //    are used, the information they convey MUST be consistent.
            if let Some(ref use_) = self.use_ {
                if key_ops.iter().any(|op| op != use_) {
                    return Err(Error::Data(None));
                }
            }

            // or does not contain all of the specified usages values
            let key_ops_as_usages = self.get_usages_from_key_ops()?;
            if !specified_usages
                .iter()
                .all(|specified_usage| key_ops_as_usages.contains(specified_usage))
            {
                return Err(Error::Data(None));
            }
        }

        Ok(())
    }

    // Set the key_ops attribute of jwk to equal the given usages.
    fn set_key_ops(&mut self, usages: Vec<KeyUsage>) {
        self.key_ops = Some(
            usages
                .into_iter()
                .map(|usage| DOMString::from(usage.as_str()))
                .collect(),
        );
    }

    // Encode a byte sequence to a base64url-encoded string, and set the field to the encoded
    // string.
    fn encode_string_field(&mut self, field: JwkStringField, data: &[u8]) {
        let encoded_data = DOMString::from(Base64UrlUnpadded::encode_string(data));
        match field {
            JwkStringField::X => self.x = Some(encoded_data),
            JwkStringField::Y => self.y = Some(encoded_data),
            JwkStringField::D => self.d = Some(encoded_data),
            JwkStringField::N => self.n = Some(encoded_data),
            JwkStringField::E => self.e = Some(encoded_data),
            JwkStringField::P => self.p = Some(encoded_data),
            JwkStringField::Q => self.q = Some(encoded_data),
            JwkStringField::DP => self.dp = Some(encoded_data),
            JwkStringField::DQ => self.dq = Some(encoded_data),
            JwkStringField::QI => self.qi = Some(encoded_data),
            JwkStringField::K => self.k = Some(encoded_data),
            JwkStringField::Priv => self.priv_ = Some(encoded_data),
            JwkStringField::Pub => self.pub_ = Some(encoded_data),
        }
    }

    // Decode a field from a base64url-encoded string to a byte sequence. If the field is not a
    // valid base64url-encoded string, then throw a DataError.
    fn decode_optional_string_field(
        &self,
        field: JwkStringField,
    ) -> Result<Option<Vec<u8>>, Error> {
        let field_string = match field {
            JwkStringField::X => &self.x,
            JwkStringField::Y => &self.y,
            JwkStringField::D => &self.d,
            JwkStringField::N => &self.n,
            JwkStringField::E => &self.e,
            JwkStringField::P => &self.p,
            JwkStringField::Q => &self.q,
            JwkStringField::DP => &self.dp,
            JwkStringField::DQ => &self.dq,
            JwkStringField::QI => &self.qi,
            JwkStringField::K => &self.k,
            JwkStringField::Priv => &self.priv_,
            JwkStringField::Pub => &self.pub_,
        };

        field_string
            .as_ref()
            .map(|field_string| Base64UrlUnpadded::decode_vec(&field_string.str()))
            .transpose()
            .map_err(|_| Error::Data(Some(format!("Failed to decode {} field in jwk", field))))
    }

    // Decode a field from a base64url-encoded string to a byte sequence. If the field is not
    // present or it is not a valid base64url-encoded string, then throw a DataError.
    fn decode_required_string_field(&self, field: JwkStringField) -> Result<Vec<u8>, Error> {
        self.decode_optional_string_field(field)?
            .ok_or(Error::Data(Some(format!(
                "The {} field is not present in jwk",
                field
            ))))
    }

    // Decode the "r", "d" and "t" field of each entry in the "oth" array, from a base64url-encoded
    // string to a byte sequence, and append the decoded "r" field to the `primes` list, in the
    // order of presence in the "oth" array.
    //
    // If the "oth" field is present and any of the "p", "q", "dp", "dq" or "qi" field is not
    // present, then throw a DataError. For each entry in the "oth" array, if any of the "r", "d"
    // and "t" field is not present or it is not a valid base64url-encoded string, then throw a
    // DataError.
    fn decode_primes_from_oth_field(&self, primes: &mut Vec<Vec<u8>>) -> Result<(), Error> {
        if self.oth.is_some() &&
            (self.p.is_none() ||
                self.q.is_none() ||
                self.dp.is_none() ||
                self.dq.is_none() ||
                self.qi.is_none())
        {
            return Err(Error::Data(Some(
                "The oth field is present while at least one of p, q, dp, dq, qi is missing, in jwk".to_string()
            )));
        }

        for rsa_other_prime_info in self.oth.as_ref().unwrap_or(&Vec::new()) {
            let r = Base64UrlUnpadded::decode_vec(
                &rsa_other_prime_info
                    .r
                    .as_ref()
                    .ok_or(Error::Data(Some(
                        "The r field is not present in one of the entry of oth field in jwk"
                            .to_string(),
                    )))?
                    .str(),
            )
            .map_err(|_| {
                Error::Data(Some(
                    "Fail to decode r field in one of the entry of oth field in jwk".to_string(),
                ))
            })?;
            primes.push(r);

            let _d = Base64UrlUnpadded::decode_vec(
                &rsa_other_prime_info
                    .d
                    .as_ref()
                    .ok_or(Error::Data(Some(
                        "The d field is not present in one of the entry of oth field in jwk"
                            .to_string(),
                    )))?
                    .str(),
            )
            .map_err(|_| {
                Error::Data(Some(
                    "Fail to decode d field in one of the entry of oth field in jwk".to_string(),
                ))
            })?;

            let _t = Base64UrlUnpadded::decode_vec(
                &rsa_other_prime_info
                    .t
                    .as_ref()
                    .ok_or(Error::Data(Some(
                        "The t field is not present in one of the entry of oth field in jwk"
                            .to_string(),
                    )))?
                    .str(),
            )
            .map_err(|_| {
                Error::Data(Some(
                    "Fail to decode t field in one of the entry of oth field in jwk".to_string(),
                ))
            })?;
        }

        Ok(())
    }
}

/// <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm>
fn normalize_algorithm<Op: Operation>(
    cx: &mut js::context::JSContext,
    algorithm: &AlgorithmIdentifier,
) -> Result<Op::RegisteredAlgorithm, Error> {
    match algorithm {
        // If alg is an instance of a DOMString:
        ObjectOrString::String(name) => {
            // Return the result of running the normalize an algorithm algorithm, with the alg set
            // to a new Algorithm dictionary whose name attribute is alg, and with the op set to
            // op.
            let algorithm = Algorithm {
                name: name.to_owned(),
            };
            rooted!(&in(cx) let mut algorithm_value = UndefinedValue());
            algorithm.safe_to_jsval(cx.into(), algorithm_value.handle_mut(), CanGc::from_cx(cx));
            let algorithm_object = RootedTraceableBox::new(Heap::default());
            algorithm_object.set(algorithm_value.to_object());
            normalize_algorithm::<Op>(cx, &ObjectOrString::Object(algorithm_object))
        },
        // If alg is an object:
        ObjectOrString::Object(object) => {
            // Step 1. Let registeredAlgorithms be the associative container stored at the op key
            // of supportedAlgorithms.

            // Stpe 2. Let initialAlg be the result of converting the ECMAScript object represented
            // by alg to the IDL dictionary type Algorithm, as defined by [WebIDL].
            // Step 3. If an error occurred, return the error and terminate this algorithm.
            rooted!(&in(cx) let value = ObjectValue(object.get()));
            let initial_algorithm = dictionary_from_jsval::<Algorithm>(cx, value.handle())?;

            // Step 4. Let algName be the value of the name attribute of initialAlg.
            let algorithm_name =
                CryptoAlgorithm::from_str_ignore_case(&initial_algorithm.name.str())?;

            // Step 5.
            //     If registeredAlgorithms contains a key that is a case-insensitive string match
            //     for algName:
            //         Step 5.1. Set algName to the value of the matching key.
            //         Step 5.2. Let desiredType be the IDL dictionary type stored at algName in
            //         registeredAlgorithms.
            //     Otherwise:
            //         Return a new NotSupportedError and terminate this algorithm.
            // Step 6. Let normalizedAlgorithm be the result of converting the ECMAScript object
            // represented by alg to the IDL dictionary type desiredType, as defined by [WebIDL].
            // Step 7. Set the name attribute of normalizedAlgorithm to algName.
            // Step 8. If an error occurred, return the error and terminate this algorithm.
            // Step 9. Let dictionaries be a list consisting of the IDL dictionary type desiredType
            // and all of desiredType's inherited dictionaries, in order from least to most
            // derived.
            // Step 10. For each dictionary dictionary in dictionaries:
            //     Step 10.1. For each dictionary member member declared on dictionary, in order:
            //         Step 10.1.1. Let key be the identifier of member.
            //         Step 10.1.2. Let idlValue be the value of the dictionary member with key
            //         name of key on normalizedAlgorithm.
            //         Step 10.1.3.
            //             If member is of the type BufferSource and is present:
            //                 Set the dictionary member on normalizedAlgorithm with key name key
            //                 to the result of getting a copy of the bytes held by idlValue,
            //                 replacing the current value.
            //             If member is of the type HashAlgorithmIdentifier:
            //                 Set the dictionary member on normalizedAlgorithm with key name key
            //                 to the result of normalizing an algorithm, with the alg set to
            //                 idlValue and the op set to "digest".
            //             If member is of the type AlgorithmIdentifier:
            //                 Set the dictionary member on normalizedAlgorithm with key name key
            //                 to the result of normalizing an algorithm, with the alg set to
            //                 idlValue and the op set to the operation defined by the
            //                 specification that defines the algorithm identified by algName.
            //
            // NOTE: Step 7 is done by writing algName back to the name attribute of the JS object
            // before dictionary conversion in Step 6, in order to streamline the conversion. Step
            // 9 and 10 are done by the calling `TryIntoWithCx::try_into_with_cx` within the trait
            // implementation of `Op::RegisteredAlgorithm::from_object_value`.
            rooted!(&in(cx) let mut algorithm_name_value = UndefinedValue());
            algorithm_name.as_str().safe_to_jsval(
                cx.into(),
                algorithm_name_value.handle_mut(),
                CanGc::from_cx(cx),
            );
            set_dictionary_property(
                cx.into(),
                object.handle(),
                c"name",
                algorithm_name_value.handle(),
            )
            .map_err(|_| Error::JSFailed)?;
            let normalized_algorithm =
                Op::RegisteredAlgorithm::from_object_value(cx, algorithm_name, value.handle())?;

            // Step 11. Return normalizedAlgorithm.
            Ok(normalized_algorithm)
        },
    }
}

// <https://w3c.github.io/webcrypto/#dfn-supportedAlgorithms>
//
// We implement the internal object
// [supportedAlgorithms](https://w3c.github.io/webcrypto/#dfn-supportedAlgorithms) for algorithm
// registration, in the following way.
//
// For each operation v in the list of [supported
// operations](https://w3c.github.io/webcrypto/#supported-operation), we define a struct to
// represent it, which acts a key of the internal object supportedAlgorithms.
//
// We then implement the [`Operation`] trait for these structs. When implementing the trait for
// each of these structs, we set the associated type [`RegisteredAlgorithm`] of [`Operation`] to an
// enum as the value of the operation v in supportedAlgorithms. The enum lists all algorithhms
// supporting the operation v as its variants.
//
// To [define an algorithm](https://w3c.github.io/webcrypto/#concept-define-an-algorithm), each
// variant in the enum has an inner type corresponding to the desired input IDL dictionary type for
// the supported algorithm represented by the variant. Moreover, the enum also need to implement
// the [`NormalizedAlgorithm`] trait since it is used as the output of
// [`normalize_algorithm`].
//
// For example, we define the [`EncryptOperation`] struct to represent the "encrypt" operation, and
// implement the [`Operation`] trait for it. The associated type [`RegisteredAlgorithm`] of
// [`Operation`]  is set to the [`EncryptAlgorithm`] enum, whose variants are cryptographic
// algorithms that support the "encrypt" operation. The variant [`EncryptAlgorithm::AesCtr`] has an
// inner type [`SubtleAesCtrParams`] since the desired input IDL dictionary type for "encrypt"
// operation of AES-CTR algorithm is the `AesCtrParams` dictionary. The [`EncryptAlgorithm`] enum
// also implements the [`NormalizedAlgorithm`] trait accordingly.
//
// The algorithm registrations are specified in:
// RSASSA-PKCS1-v1_5: <https://w3c.github.io/webcrypto/#rsassa-pkcs1-registration>
// RSA-PSS:           <https://w3c.github.io/webcrypto/#rsa-pss-registration>
// RSA-OAEP:          <https://w3c.github.io/webcrypto/#rsa-oaep-registration>
// ECDSA:             <https://w3c.github.io/webcrypto/#ecdsa-registration>
// ECDH:              <https://w3c.github.io/webcrypto/#ecdh-registration>
// Ed25519:           <https://w3c.github.io/webcrypto/#ed25519-registration>
// X25519:            <https://w3c.github.io/webcrypto/#x25519-registration>
// AES-CTR:           <https://w3c.github.io/webcrypto/#aes-ctr-registration>
// AES-CBC:           <https://w3c.github.io/webcrypto/#aes-cbc-registration>
// AES-GCM:           <https://w3c.github.io/webcrypto/#aes-gcm-registration>
// AES-KW:            <https://w3c.github.io/webcrypto/#aes-kw-registration>
// HMAC:              <https://w3c.github.io/webcrypto/#hmac-registration>
// SHA:               <https://w3c.github.io/webcrypto/#sha-registration>
// HKDF:              <https://w3c.github.io/webcrypto/#hkdf-registration>
// PBKDF2:            <https://w3c.github.io/webcrypto/#pbkdf2-registration>
// ML-KEM:            <https://wicg.github.io/webcrypto-modern-algos/#ml-kem-registration>
// ML-DSA:            <https://wicg.github.io/webcrypto-modern-algos/#ml-dsa-registration>
// AES-OCB:           <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-registration>
// ChaCha20-Poly1305: <https://wicg.github.io/webcrypto-modern-algos/#chacha20-poly1305-registration>
// SHA-3:             <https://wicg.github.io/webcrypto-modern-algos/#sha3-registration>
// cSHAKE:            <https://wicg.github.io/webcrypto-modern-algos/#cshake-registration>
// Argon2:            <https://wicg.github.io/webcrypto-modern-algos/#argon2-registration>

trait Operation {
    type RegisteredAlgorithm: NormalizedAlgorithm;
}

trait NormalizedAlgorithm: Sized {
    /// Step 4 - 10 of <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm>
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self>;
    fn name(&self) -> &str;
}

/// The value of the key "encrypt" in the internal object supportedAlgorithms
struct EncryptOperation {}

impl Operation for EncryptOperation {
    type RegisteredAlgorithm = EncryptAlgorithm;
}

/// Normalized algorithm for the "encrypt" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
enum EncryptAlgorithm {
    RsaOaep(SubtleRsaOaepParams),
    AesCtr(SubtleAesCtrParams),
    AesCbc(SubtleAesCbcParams),
    AesGcm(SubtleAesGcmParams),
    AesOcb(SubtleAeadParams),
    ChaCha20Poly1305(SubtleAeadParams),
}

impl NormalizedAlgorithm for EncryptAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::RsaOaep => Ok(EncryptAlgorithm::RsaOaep(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCtr => Ok(EncryptAlgorithm::AesCtr(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCbc => Ok(EncryptAlgorithm::AesCbc(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesGcm => Ok(EncryptAlgorithm::AesGcm(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesOcb => Ok(EncryptAlgorithm::AesOcb(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::ChaCha20Poly1305 => Ok(EncryptAlgorithm::ChaCha20Poly1305(
                value.try_into_with_cx(cx)?,
            )),
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"encrypt\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            EncryptAlgorithm::RsaOaep(algorithm) => &algorithm.name,
            EncryptAlgorithm::AesCtr(algorithm) => &algorithm.name,
            EncryptAlgorithm::AesCbc(algorithm) => &algorithm.name,
            EncryptAlgorithm::AesGcm(algorithm) => &algorithm.name,
            EncryptAlgorithm::AesOcb(algorithm) => &algorithm.name,
            EncryptAlgorithm::ChaCha20Poly1305(algorithm) => &algorithm.name,
        }
    }
}

impl EncryptAlgorithm {
    fn encrypt(&self, key: &CryptoKey, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            EncryptAlgorithm::RsaOaep(algorithm) => {
                rsa_oaep_operation::encrypt(algorithm, key, plaintext)
            },
            EncryptAlgorithm::AesCtr(algorithm) => {
                aes_ctr_operation::encrypt(algorithm, key, plaintext)
            },
            EncryptAlgorithm::AesCbc(algorithm) => {
                aes_cbc_operation::encrypt(algorithm, key, plaintext)
            },
            EncryptAlgorithm::AesGcm(algorithm) => {
                aes_gcm_operation::encrypt(algorithm, key, plaintext)
            },
            EncryptAlgorithm::AesOcb(algorithm) => {
                aes_ocb_operation::encrypt(algorithm, key, plaintext)
            },
            EncryptAlgorithm::ChaCha20Poly1305(algorithm) => {
                chacha20_poly1305_operation::encrypt(algorithm, key, plaintext)
            },
        }
    }
}

/// The value of the key "decrypt" in the internal object supportedAlgorithms
struct DecryptOperation {}

impl Operation for DecryptOperation {
    type RegisteredAlgorithm = DecryptAlgorithm;
}

/// Normalized algorithm for the "decrypt" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
enum DecryptAlgorithm {
    RsaOaep(SubtleRsaOaepParams),
    AesCtr(SubtleAesCtrParams),
    AesCbc(SubtleAesCbcParams),
    AesGcm(SubtleAesGcmParams),
    AesOcb(SubtleAeadParams),
    ChaCha20Poly1305(SubtleAeadParams),
}

impl NormalizedAlgorithm for DecryptAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::RsaOaep => Ok(DecryptAlgorithm::RsaOaep(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCtr => Ok(DecryptAlgorithm::AesCtr(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCbc => Ok(DecryptAlgorithm::AesCbc(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesGcm => Ok(DecryptAlgorithm::AesGcm(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesOcb => Ok(DecryptAlgorithm::AesOcb(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::ChaCha20Poly1305 => Ok(DecryptAlgorithm::ChaCha20Poly1305(
                value.try_into_with_cx(cx)?,
            )),
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"decrypt\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            DecryptAlgorithm::RsaOaep(algorithm) => &algorithm.name,
            DecryptAlgorithm::AesCtr(algorithm) => &algorithm.name,
            DecryptAlgorithm::AesCbc(algorithm) => &algorithm.name,
            DecryptAlgorithm::AesGcm(algorithm) => &algorithm.name,
            DecryptAlgorithm::AesOcb(algorithm) => &algorithm.name,
            DecryptAlgorithm::ChaCha20Poly1305(algorithm) => &algorithm.name,
        }
    }
}

impl DecryptAlgorithm {
    fn decrypt(&self, key: &CryptoKey, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            DecryptAlgorithm::RsaOaep(algorithm) => {
                rsa_oaep_operation::decrypt(algorithm, key, ciphertext)
            },
            DecryptAlgorithm::AesCtr(algorithm) => {
                aes_ctr_operation::decrypt(algorithm, key, ciphertext)
            },
            DecryptAlgorithm::AesCbc(algorithm) => {
                aes_cbc_operation::decrypt(algorithm, key, ciphertext)
            },
            DecryptAlgorithm::AesGcm(algorithm) => {
                aes_gcm_operation::decrypt(algorithm, key, ciphertext)
            },
            DecryptAlgorithm::AesOcb(algorithm) => {
                aes_ocb_operation::decrypt(algorithm, key, ciphertext)
            },
            DecryptAlgorithm::ChaCha20Poly1305(algorithm) => {
                chacha20_poly1305_operation::decrypt(algorithm, key, ciphertext)
            },
        }
    }
}

/// The value of the key "sign" in the internal object supportedAlgorithms
struct SignOperation {}

impl Operation for SignOperation {
    type RegisteredAlgorithm = SignAlgorithm;
}

/// Normalized algorithm for the "sign" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
enum SignAlgorithm {
    RsassaPkcs1V1_5(SubtleAlgorithm),
    RsaPss(SubtleRsaPssParams),
    Ecdsa(SubtleEcdsaParams),
    Ed25519(SubtleAlgorithm),
    Hmac(SubtleAlgorithm),
    MlDsa(SubtleContextParams),
}

impl NormalizedAlgorithm for SignAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::RsassaPkcs1V1_5 => {
                Ok(SignAlgorithm::RsassaPkcs1V1_5(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::RsaPss => Ok(SignAlgorithm::RsaPss(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ecdsa => Ok(SignAlgorithm::Ecdsa(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ed25519 => Ok(SignAlgorithm::Ed25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hmac => Ok(SignAlgorithm::Hmac(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::MlDsa44 | CryptoAlgorithm::MlDsa65 | CryptoAlgorithm::MlDsa87 => {
                Ok(SignAlgorithm::MlDsa(value.try_into_with_cx(cx)?))
            },
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"sign\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            SignAlgorithm::RsassaPkcs1V1_5(algorithm) => &algorithm.name,
            SignAlgorithm::RsaPss(algorithm) => &algorithm.name,
            SignAlgorithm::Ecdsa(algorithm) => &algorithm.name,
            SignAlgorithm::Ed25519(algorithm) => &algorithm.name,
            SignAlgorithm::Hmac(algorithm) => &algorithm.name,
            SignAlgorithm::MlDsa(algorithm) => &algorithm.name,
        }
    }
}

impl SignAlgorithm {
    fn sign(&self, key: &CryptoKey, message: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            SignAlgorithm::RsassaPkcs1V1_5(_algorithm) => {
                rsassa_pkcs1_v1_5_operation::sign(key, message)
            },
            SignAlgorithm::RsaPss(algorithm) => rsa_pss_operation::sign(algorithm, key, message),
            SignAlgorithm::Ecdsa(algorithm) => ecdsa_operation::sign(algorithm, key, message),
            SignAlgorithm::Ed25519(_algorithm) => ed25519_operation::sign(key, message),
            SignAlgorithm::Hmac(_algorithm) => hmac_operation::sign(key, message),
            SignAlgorithm::MlDsa(algorithm) => ml_dsa_operation::sign(algorithm, key, message),
        }
    }
}

/// The value of the key "verify" in the internal object supportedAlgorithms
struct VerifyOperation {}

impl Operation for VerifyOperation {
    type RegisteredAlgorithm = VerifyAlgorithm;
}

/// Normalized algorithm for the "verify" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
enum VerifyAlgorithm {
    RsassaPkcs1V1_5(SubtleAlgorithm),
    RsaPss(SubtleRsaPssParams),
    Ecdsa(SubtleEcdsaParams),
    Ed25519(SubtleAlgorithm),
    Hmac(SubtleAlgorithm),
    MlDsa(SubtleContextParams),
}

impl NormalizedAlgorithm for VerifyAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::RsassaPkcs1V1_5 => Ok(VerifyAlgorithm::RsassaPkcs1V1_5(
                value.try_into_with_cx(cx)?,
            )),
            CryptoAlgorithm::RsaPss => Ok(VerifyAlgorithm::RsaPss(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ecdsa => Ok(VerifyAlgorithm::Ecdsa(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ed25519 => Ok(VerifyAlgorithm::Ed25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hmac => Ok(VerifyAlgorithm::Hmac(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::MlDsa44 | CryptoAlgorithm::MlDsa65 | CryptoAlgorithm::MlDsa87 => {
                Ok(VerifyAlgorithm::MlDsa(value.try_into_with_cx(cx)?))
            },
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"verify\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            VerifyAlgorithm::RsassaPkcs1V1_5(algorithm) => &algorithm.name,
            VerifyAlgorithm::RsaPss(algorithm) => &algorithm.name,
            VerifyAlgorithm::Ecdsa(algorithm) => &algorithm.name,
            VerifyAlgorithm::Ed25519(algorithm) => &algorithm.name,
            VerifyAlgorithm::Hmac(algorithm) => &algorithm.name,
            VerifyAlgorithm::MlDsa(algorithm) => &algorithm.name,
        }
    }
}

impl VerifyAlgorithm {
    fn verify(&self, key: &CryptoKey, message: &[u8], signature: &[u8]) -> Result<bool, Error> {
        match self {
            VerifyAlgorithm::RsassaPkcs1V1_5(_algorithm) => {
                rsassa_pkcs1_v1_5_operation::verify(key, message, signature)
            },
            VerifyAlgorithm::RsaPss(algorithm) => {
                rsa_pss_operation::verify(algorithm, key, message, signature)
            },
            VerifyAlgorithm::Ecdsa(algorithm) => {
                ecdsa_operation::verify(algorithm, key, message, signature)
            },
            VerifyAlgorithm::Ed25519(_algorithm) => {
                ed25519_operation::verify(key, message, signature)
            },
            VerifyAlgorithm::Hmac(_algorithm) => hmac_operation::verify(key, message, signature),
            VerifyAlgorithm::MlDsa(algorithm) => {
                ml_dsa_operation::verify(algorithm, key, message, signature)
            },
        }
    }
}

/// The value of the key "digest" in the internal object supportedAlgorithms
struct DigestOperation {}

impl Operation for DigestOperation {
    type RegisteredAlgorithm = DigestAlgorithm;
}

/// Normalized algorithm for the "digest" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
#[derive(Clone, MallocSizeOf)]
enum DigestAlgorithm {
    Sha(SubtleAlgorithm),
    Sha3(SubtleAlgorithm),
    CShake(SubtleCShakeParams),
}

impl NormalizedAlgorithm for DigestAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::Sha1 |
            CryptoAlgorithm::Sha256 |
            CryptoAlgorithm::Sha384 |
            CryptoAlgorithm::Sha512 => Ok(DigestAlgorithm::Sha(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Sha3_256 | CryptoAlgorithm::Sha3_384 | CryptoAlgorithm::Sha3_512 => {
                Ok(DigestAlgorithm::Sha3(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::CShake128 | CryptoAlgorithm::CShake256 => {
                Ok(DigestAlgorithm::CShake(value.try_into_with_cx(cx)?))
            },
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"digest\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            DigestAlgorithm::Sha(algorithm) => &algorithm.name,
            DigestAlgorithm::Sha3(algorithm) => &algorithm.name,
            DigestAlgorithm::CShake(algorithm) => &algorithm.name,
        }
    }
}

impl DigestAlgorithm {
    fn digest(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            DigestAlgorithm::Sha(algorithm) => sha_operation::digest(algorithm, message),
            DigestAlgorithm::Sha3(algorithm) => sha3_operation::digest(algorithm, message),
            DigestAlgorithm::CShake(algorithm) => cshake_operation::digest(algorithm, message),
        }
    }
}

/// The value of the key "deriveBits" in the internal object supportedAlgorithms
struct DeriveBitsOperation {}

impl Operation for DeriveBitsOperation {
    type RegisteredAlgorithm = DeriveBitsAlgorithm;
}

/// Normalized algorithm for the "deriveBits" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
enum DeriveBitsAlgorithm {
    Ecdh(SubtleEcdhKeyDeriveParams),
    X25519(SubtleEcdhKeyDeriveParams),
    Hkdf(SubtleHkdfParams),
    Pbkdf2(SubtlePbkdf2Params),
    Argon2(SubtleArgon2Params),
}

impl NormalizedAlgorithm for DeriveBitsAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::Ecdh => Ok(DeriveBitsAlgorithm::Ecdh(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::X25519 => Ok(DeriveBitsAlgorithm::X25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hkdf => Ok(DeriveBitsAlgorithm::Hkdf(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Pbkdf2 => Ok(DeriveBitsAlgorithm::Pbkdf2(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Argon2D | CryptoAlgorithm::Argon2I | CryptoAlgorithm::Argon2ID => {
                Ok(DeriveBitsAlgorithm::Argon2(value.try_into_with_cx(cx)?))
            },
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"deriveBits\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            DeriveBitsAlgorithm::Ecdh(algorithm) => &algorithm.name,
            DeriveBitsAlgorithm::X25519(algorithm) => &algorithm.name,
            DeriveBitsAlgorithm::Hkdf(algorithm) => &algorithm.name,
            DeriveBitsAlgorithm::Pbkdf2(algorithm) => &algorithm.name,
            DeriveBitsAlgorithm::Argon2(algorithm) => &algorithm.name,
        }
    }
}

impl DeriveBitsAlgorithm {
    fn derive_bits(&self, key: &CryptoKey, length: Option<u32>) -> Result<Vec<u8>, Error> {
        match self {
            DeriveBitsAlgorithm::Ecdh(algorithm) => {
                ecdh_operation::derive_bits(algorithm, key, length)
            },
            DeriveBitsAlgorithm::X25519(algorithm) => {
                x25519_operation::derive_bits(algorithm, key, length)
            },
            DeriveBitsAlgorithm::Hkdf(algorithm) => {
                hkdf_operation::derive_bits(algorithm, key, length)
            },
            DeriveBitsAlgorithm::Pbkdf2(algorithm) => {
                pbkdf2_operation::derive_bits(algorithm, key, length)
            },
            DeriveBitsAlgorithm::Argon2(algorithm) => {
                argon2_operation::derive_bits(algorithm, key, length)
            },
        }
    }
}

/// The value of the key "wrapKey" in the internal object supportedAlgorithms
struct WrapKeyOperation {}

impl Operation for WrapKeyOperation {
    type RegisteredAlgorithm = WrapKeyAlgorithm;
}

/// Normalized algorithm for the "wrapKey" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
enum WrapKeyAlgorithm {
    AesKw(SubtleAlgorithm),
}

impl NormalizedAlgorithm for WrapKeyAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::AesKw => Ok(WrapKeyAlgorithm::AesKw(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"wrapKey\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            WrapKeyAlgorithm::AesKw(algorithm) => &algorithm.name,
        }
    }
}

impl WrapKeyAlgorithm {
    fn wrap_key(&self, key: &CryptoKey, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            WrapKeyAlgorithm::AesKw(_algorithm) => aes_kw_operation::wrap_key(key, plaintext),
        }
    }
}

/// The value of the key "unwrapKey" in the internal object supportedAlgorithms
struct UnwrapKeyOperation {}

impl Operation for UnwrapKeyOperation {
    type RegisteredAlgorithm = UnwrapKeyAlgorithm;
}

/// Normalized algorithm for the "unwrapKey" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
enum UnwrapKeyAlgorithm {
    AesKw(SubtleAlgorithm),
}

impl NormalizedAlgorithm for UnwrapKeyAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::AesKw => Ok(UnwrapKeyAlgorithm::AesKw(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"unwrapKey\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            UnwrapKeyAlgorithm::AesKw(algorithm) => &algorithm.name,
        }
    }
}

impl UnwrapKeyAlgorithm {
    fn unwrap_key(&self, key: &CryptoKey, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            UnwrapKeyAlgorithm::AesKw(_algorithm) => aes_kw_operation::unwrap_key(key, ciphertext),
        }
    }
}

/// The value of the key "unwrapKey" in the internal object supportedAlgorithms
struct GenerateKeyOperation {}

impl Operation for GenerateKeyOperation {
    type RegisteredAlgorithm = GenerateKeyAlgorithm;
}

/// Normalized algorithm for the "generateKey" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
enum GenerateKeyAlgorithm {
    RsassaPkcs1V1_5(SubtleRsaHashedKeyGenParams),
    RsaPss(SubtleRsaHashedKeyGenParams),
    RsaOaep(SubtleRsaHashedKeyGenParams),
    Ecdsa(SubtleEcKeyGenParams),
    Ecdh(SubtleEcKeyGenParams),
    Ed25519(SubtleAlgorithm),
    X25519(SubtleAlgorithm),
    AesCtr(SubtleAesKeyGenParams),
    AesCbc(SubtleAesKeyGenParams),
    AesGcm(SubtleAesKeyGenParams),
    AesKw(SubtleAesKeyGenParams),
    Hmac(SubtleHmacKeyGenParams),
    MlKem(SubtleAlgorithm),
    MlDsa(SubtleAlgorithm),
    AesOcb(SubtleAesKeyGenParams),
    ChaCha20Poly1305(SubtleAlgorithm),
}

impl NormalizedAlgorithm for GenerateKeyAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::RsassaPkcs1V1_5 => Ok(GenerateKeyAlgorithm::RsassaPkcs1V1_5(
                value.try_into_with_cx(cx)?,
            )),
            CryptoAlgorithm::RsaPss => {
                Ok(GenerateKeyAlgorithm::RsaPss(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::RsaOaep => {
                Ok(GenerateKeyAlgorithm::RsaOaep(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::Ecdsa => Ok(GenerateKeyAlgorithm::Ecdsa(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ecdh => Ok(GenerateKeyAlgorithm::Ecdh(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ed25519 => {
                Ok(GenerateKeyAlgorithm::Ed25519(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::X25519 => {
                Ok(GenerateKeyAlgorithm::X25519(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::AesCtr => {
                Ok(GenerateKeyAlgorithm::AesCtr(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::AesCbc => {
                Ok(GenerateKeyAlgorithm::AesCbc(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::AesGcm => {
                Ok(GenerateKeyAlgorithm::AesGcm(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::AesKw => Ok(GenerateKeyAlgorithm::AesKw(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hmac => Ok(GenerateKeyAlgorithm::Hmac(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::MlKem512 | CryptoAlgorithm::MlKem768 | CryptoAlgorithm::MlKem1024 => {
                Ok(GenerateKeyAlgorithm::MlKem(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::MlDsa44 | CryptoAlgorithm::MlDsa65 | CryptoAlgorithm::MlDsa87 => {
                Ok(GenerateKeyAlgorithm::MlDsa(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::AesOcb => {
                Ok(GenerateKeyAlgorithm::AesOcb(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::ChaCha20Poly1305 => Ok(GenerateKeyAlgorithm::ChaCha20Poly1305(
                value.try_into_with_cx(cx)?,
            )),
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"generateKey\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            GenerateKeyAlgorithm::RsassaPkcs1V1_5(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::RsaPss(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::RsaOaep(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::Ecdsa(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::Ecdh(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::Ed25519(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::X25519(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::AesCtr(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::AesCbc(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::AesGcm(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::AesKw(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::Hmac(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::MlKem(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::MlDsa(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::AesOcb(algorithm) => &algorithm.name,
            GenerateKeyAlgorithm::ChaCha20Poly1305(algorithm) => &algorithm.name,
        }
    }
}

impl GenerateKeyAlgorithm {
    fn generate_key(
        &self,
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        extractable: bool,
        usages: Vec<KeyUsage>,
    ) -> Result<CryptoKeyOrCryptoKeyPair, Error> {
        match self {
            GenerateKeyAlgorithm::RsassaPkcs1V1_5(algorithm) => {
                rsassa_pkcs1_v1_5_operation::generate_key(
                    cx,
                    global,
                    algorithm,
                    extractable,
                    usages,
                )
                .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::RsaPss(algorithm) => {
                rsa_pss_operation::generate_key(cx, global, algorithm, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::RsaOaep(algorithm) => {
                rsa_oaep_operation::generate_key(cx, global, algorithm, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::Ecdsa(algorithm) => {
                ecdsa_operation::generate_key(cx, global, algorithm, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::Ecdh(algorithm) => {
                ecdh_operation::generate_key(cx, global, algorithm, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::Ed25519(_algorithm) => {
                ed25519_operation::generate_key(cx, global, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::X25519(_algorithm) => {
                x25519_operation::generate_key(cx, global, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::AesCtr(algorithm) => {
                aes_ctr_operation::generate_key(cx, global, algorithm, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            GenerateKeyAlgorithm::AesCbc(algorithm) => {
                aes_cbc_operation::generate_key(cx, global, algorithm, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            GenerateKeyAlgorithm::AesGcm(algorithm) => {
                aes_gcm_operation::generate_key(cx, global, algorithm, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            GenerateKeyAlgorithm::AesKw(algorithm) => {
                aes_kw_operation::generate_key(cx, global, algorithm, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            GenerateKeyAlgorithm::Hmac(algorithm) => {
                hmac_operation::generate_key(cx, global, algorithm, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            GenerateKeyAlgorithm::MlKem(algorithm) => {
                ml_kem_operation::generate_key(cx, global, algorithm, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::MlDsa(algorithm) => {
                ml_dsa_operation::generate_key(cx, global, algorithm, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::AesOcb(algorithm) => {
                aes_ocb_operation::generate_key(cx, global, algorithm, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            GenerateKeyAlgorithm::ChaCha20Poly1305(_algorithm) => {
                chacha20_poly1305_operation::generate_key(cx, global, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
        }
    }
}

/// The value of the key "importKey" in the internal object supportedAlgorithms
struct ImportKeyOperation {}

impl Operation for ImportKeyOperation {
    type RegisteredAlgorithm = ImportKeyAlgorithm;
}

/// Normalized algorithm for the "importKey" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
enum ImportKeyAlgorithm {
    RsassaPkcs1V1_5(SubtleRsaHashedImportParams),
    RsaPss(SubtleRsaHashedImportParams),
    RsaOaep(SubtleRsaHashedImportParams),
    Ecdsa(SubtleEcKeyImportParams),
    Ecdh(SubtleEcKeyImportParams),
    Ed25519(SubtleAlgorithm),
    X25519(SubtleAlgorithm),
    AesCtr(SubtleAlgorithm),
    AesCbc(SubtleAlgorithm),
    AesGcm(SubtleAlgorithm),
    AesKw(SubtleAlgorithm),
    Hmac(SubtleHmacImportParams),
    Hkdf(SubtleAlgorithm),
    Pbkdf2(SubtleAlgorithm),
    MlKem(SubtleAlgorithm),
    MlDsa(SubtleAlgorithm),
    AesOcb(SubtleAlgorithm),
    ChaCha20Poly1305(SubtleAlgorithm),
    Argon2(SubtleAlgorithm),
}

impl NormalizedAlgorithm for ImportKeyAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::RsassaPkcs1V1_5 => Ok(ImportKeyAlgorithm::RsassaPkcs1V1_5(
                value.try_into_with_cx(cx)?,
            )),
            CryptoAlgorithm::RsaPss => Ok(ImportKeyAlgorithm::RsaPss(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::RsaOaep => {
                Ok(ImportKeyAlgorithm::RsaOaep(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::Ecdsa => Ok(ImportKeyAlgorithm::Ecdsa(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ecdh => Ok(ImportKeyAlgorithm::Ecdh(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ed25519 => {
                Ok(ImportKeyAlgorithm::Ed25519(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::X25519 => Ok(ImportKeyAlgorithm::X25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCtr => Ok(ImportKeyAlgorithm::AesCtr(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCbc => Ok(ImportKeyAlgorithm::AesCbc(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesGcm => Ok(ImportKeyAlgorithm::AesGcm(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesKw => Ok(ImportKeyAlgorithm::AesKw(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hmac => Ok(ImportKeyAlgorithm::Hmac(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hkdf => Ok(ImportKeyAlgorithm::Hkdf(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Pbkdf2 => Ok(ImportKeyAlgorithm::Pbkdf2(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::MlKem512 | CryptoAlgorithm::MlKem768 | CryptoAlgorithm::MlKem1024 => {
                Ok(ImportKeyAlgorithm::MlKem(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::MlDsa44 | CryptoAlgorithm::MlDsa65 | CryptoAlgorithm::MlDsa87 => {
                Ok(ImportKeyAlgorithm::MlDsa(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::AesOcb => Ok(ImportKeyAlgorithm::AesOcb(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::ChaCha20Poly1305 => Ok(ImportKeyAlgorithm::ChaCha20Poly1305(
                value.try_into_with_cx(cx)?,
            )),
            CryptoAlgorithm::Argon2D | CryptoAlgorithm::Argon2I | CryptoAlgorithm::Argon2ID => {
                Ok(ImportKeyAlgorithm::Argon2(value.try_into_with_cx(cx)?))
            },
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"importKey\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            ImportKeyAlgorithm::RsassaPkcs1V1_5(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::RsaPss(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::RsaOaep(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::Ecdsa(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::Ecdh(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::Ed25519(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::X25519(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::AesCtr(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::AesCbc(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::AesGcm(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::AesKw(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::Hmac(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::Hkdf(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::Pbkdf2(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::MlKem(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::MlDsa(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::AesOcb(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::ChaCha20Poly1305(algorithm) => &algorithm.name,
            ImportKeyAlgorithm::Argon2(algorithm) => &algorithm.name,
        }
    }
}

impl ImportKeyAlgorithm {
    fn import_key(
        &self,
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        format: KeyFormat,
        key_data: &[u8],
        extractable: bool,
        usages: Vec<KeyUsage>,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        match self {
            ImportKeyAlgorithm::RsassaPkcs1V1_5(algorithm) => {
                rsassa_pkcs1_v1_5_operation::import_key(
                    cx,
                    global,
                    algorithm,
                    format,
                    key_data,
                    extractable,
                    usages,
                )
            },
            ImportKeyAlgorithm::RsaPss(algorithm) => rsa_pss_operation::import_key(
                cx,
                global,
                algorithm,
                format,
                key_data,
                extractable,
                usages,
            ),
            ImportKeyAlgorithm::RsaOaep(algorithm) => rsa_oaep_operation::import_key(
                cx,
                global,
                algorithm,
                format,
                key_data,
                extractable,
                usages,
            ),
            ImportKeyAlgorithm::Ecdsa(algorithm) => ecdsa_operation::import_key(
                cx,
                global,
                algorithm,
                format,
                key_data,
                extractable,
                usages,
            ),
            ImportKeyAlgorithm::Ecdh(algorithm) => ecdh_operation::import_key(
                cx,
                global,
                algorithm,
                format,
                key_data,
                extractable,
                usages,
            ),
            ImportKeyAlgorithm::Ed25519(_algorithm) => {
                ed25519_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::X25519(_algorithm) => {
                x25519_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::AesCtr(_algorithm) => {
                aes_ctr_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::AesCbc(_algorithm) => {
                aes_cbc_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::AesGcm(_algorithm) => {
                aes_gcm_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::AesKw(_algorithm) => {
                aes_kw_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::Hmac(algorithm) => hmac_operation::import_key(
                cx,
                global,
                algorithm,
                format,
                key_data,
                extractable,
                usages,
            ),
            ImportKeyAlgorithm::Hkdf(_algorithm) => {
                hkdf_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::Pbkdf2(_algorithm) => {
                pbkdf2_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::MlKem(algorithm) => ml_kem_operation::import_key(
                cx,
                global,
                algorithm,
                format,
                key_data,
                extractable,
                usages,
            ),
            ImportKeyAlgorithm::MlDsa(algorithm) => ml_dsa_operation::import_key(
                cx,
                global,
                algorithm,
                format,
                key_data,
                extractable,
                usages,
            ),
            ImportKeyAlgorithm::AesOcb(_algorithm) => {
                aes_ocb_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::ChaCha20Poly1305(_algorithm) => {
                chacha20_poly1305_operation::import_key(
                    cx,
                    global,
                    format,
                    key_data,
                    extractable,
                    usages,
                )
            },
            ImportKeyAlgorithm::Argon2(algorithm) => argon2_operation::import_key(
                cx,
                global,
                algorithm,
                format,
                key_data,
                extractable,
                usages,
            ),
        }
    }
}

/// The value of the key "exportKey" in the internal object supportedAlgorithms
struct ExportKeyOperation {}

impl Operation for ExportKeyOperation {
    type RegisteredAlgorithm = ExportKeyAlgorithm;
}

/// Normalized algorithm for the "exportKey" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
enum ExportKeyAlgorithm {
    RsassaPkcs1V1_5(SubtleAlgorithm),
    RsaPss(SubtleAlgorithm),
    RsaOaep(SubtleAlgorithm),
    Ecdsa(SubtleAlgorithm),
    Ecdh(SubtleAlgorithm),
    Ed25519(SubtleAlgorithm),
    X25519(SubtleAlgorithm),
    AesCtr(SubtleAlgorithm),
    AesCbc(SubtleAlgorithm),
    AesGcm(SubtleAlgorithm),
    AesKw(SubtleAlgorithm),
    Hmac(SubtleAlgorithm),
    MlKem(SubtleAlgorithm),
    MlDsa(SubtleAlgorithm),
    AesOcb(SubtleAlgorithm),
    ChaCha20Poly1305(SubtleAlgorithm),
}

impl NormalizedAlgorithm for ExportKeyAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::RsassaPkcs1V1_5 => Ok(ExportKeyAlgorithm::RsassaPkcs1V1_5(
                value.try_into_with_cx(cx)?,
            )),
            CryptoAlgorithm::RsaPss => Ok(ExportKeyAlgorithm::RsaPss(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::RsaOaep => {
                Ok(ExportKeyAlgorithm::RsaOaep(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::Ecdsa => Ok(ExportKeyAlgorithm::Ecdsa(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ecdh => Ok(ExportKeyAlgorithm::Ecdh(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ed25519 => {
                Ok(ExportKeyAlgorithm::Ed25519(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::X25519 => Ok(ExportKeyAlgorithm::X25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCtr => Ok(ExportKeyAlgorithm::AesCtr(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCbc => Ok(ExportKeyAlgorithm::AesCbc(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesGcm => Ok(ExportKeyAlgorithm::AesGcm(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesKw => Ok(ExportKeyAlgorithm::AesKw(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hmac => Ok(ExportKeyAlgorithm::Hmac(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::MlKem512 | CryptoAlgorithm::MlKem768 | CryptoAlgorithm::MlKem1024 => {
                Ok(ExportKeyAlgorithm::MlKem(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::MlDsa44 | CryptoAlgorithm::MlDsa65 | CryptoAlgorithm::MlDsa87 => {
                Ok(ExportKeyAlgorithm::MlDsa(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::AesOcb => Ok(ExportKeyAlgorithm::AesOcb(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::ChaCha20Poly1305 => Ok(ExportKeyAlgorithm::ChaCha20Poly1305(
                value.try_into_with_cx(cx)?,
            )),
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"exportKey\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            ExportKeyAlgorithm::RsassaPkcs1V1_5(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::RsaPss(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::RsaOaep(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::Ecdsa(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::Ecdh(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::Ed25519(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::X25519(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::AesCtr(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::AesCbc(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::AesGcm(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::AesKw(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::Hmac(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::MlKem(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::MlDsa(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::AesOcb(algorithm) => &algorithm.name,
            ExportKeyAlgorithm::ChaCha20Poly1305(algorithm) => &algorithm.name,
        }
    }
}

impl ExportKeyAlgorithm {
    fn export_key(&self, format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
        match self {
            ExportKeyAlgorithm::RsassaPkcs1V1_5(_algorithm) => {
                rsassa_pkcs1_v1_5_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::RsaPss(_algorithm) => rsa_pss_operation::export_key(format, key),
            ExportKeyAlgorithm::RsaOaep(_algorithm) => rsa_oaep_operation::export_key(format, key),
            ExportKeyAlgorithm::Ecdsa(_algorithm) => ecdsa_operation::export_key(format, key),
            ExportKeyAlgorithm::Ecdh(_algorithm) => ecdh_operation::export_key(format, key),
            ExportKeyAlgorithm::Ed25519(_algorithm) => ed25519_operation::export_key(format, key),
            ExportKeyAlgorithm::X25519(_algorithm) => x25519_operation::export_key(format, key),
            ExportKeyAlgorithm::AesCtr(_algorithm) => aes_ctr_operation::export_key(format, key),
            ExportKeyAlgorithm::AesCbc(_algorithm) => aes_cbc_operation::export_key(format, key),
            ExportKeyAlgorithm::AesGcm(_algorithm) => aes_gcm_operation::export_key(format, key),
            ExportKeyAlgorithm::AesKw(_algorithm) => aes_kw_operation::export_key(format, key),
            ExportKeyAlgorithm::Hmac(_algorithm) => hmac_operation::export_key(format, key),
            ExportKeyAlgorithm::MlKem(_algorithm) => ml_kem_operation::export_key(format, key),
            ExportKeyAlgorithm::MlDsa(_algorithm) => ml_dsa_operation::export_key(format, key),
            ExportKeyAlgorithm::AesOcb(_algorithm) => aes_ocb_operation::export_key(format, key),
            ExportKeyAlgorithm::ChaCha20Poly1305(_algorithm) => {
                chacha20_poly1305_operation::export_key(format, key)
            },
        }
    }
}

/// The value of the key "get key length" in the internal object supportedAlgorithms
struct GetKeyLengthOperation {}

impl Operation for GetKeyLengthOperation {
    type RegisteredAlgorithm = GetKeyLengthAlgorithm;
}

/// Normalized algorithm for the "get key length" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
enum GetKeyLengthAlgorithm {
    AesCtr(SubtleAesDerivedKeyParams),
    AesCbc(SubtleAesDerivedKeyParams),
    AesGcm(SubtleAesDerivedKeyParams),
    AesKw(SubtleAesDerivedKeyParams),
    Hmac(SubtleHmacImportParams),
    Hkdf(SubtleAlgorithm),
    Pbkdf2(SubtleAlgorithm),
    AesOcb(SubtleAesDerivedKeyParams),
    ChaCha20Poly1305(SubtleAlgorithm),
    Argon2(SubtleAlgorithm),
}

impl NormalizedAlgorithm for GetKeyLengthAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::AesCtr => {
                Ok(GetKeyLengthAlgorithm::AesCtr(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::AesCbc => {
                Ok(GetKeyLengthAlgorithm::AesCbc(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::AesGcm => {
                Ok(GetKeyLengthAlgorithm::AesGcm(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::AesKw => Ok(GetKeyLengthAlgorithm::AesKw(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hmac => Ok(GetKeyLengthAlgorithm::Hmac(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hkdf => Ok(GetKeyLengthAlgorithm::Hkdf(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Pbkdf2 => {
                Ok(GetKeyLengthAlgorithm::Pbkdf2(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::AesOcb => {
                Ok(GetKeyLengthAlgorithm::AesOcb(value.try_into_with_cx(cx)?))
            },
            CryptoAlgorithm::ChaCha20Poly1305 => Ok(GetKeyLengthAlgorithm::ChaCha20Poly1305(
                value.try_into_with_cx(cx)?,
            )),
            CryptoAlgorithm::Argon2D | CryptoAlgorithm::Argon2I | CryptoAlgorithm::Argon2ID => {
                Ok(GetKeyLengthAlgorithm::Argon2(value.try_into_with_cx(cx)?))
            },
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"get key length\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            GetKeyLengthAlgorithm::AesCtr(algorithm) => &algorithm.name,
            GetKeyLengthAlgorithm::AesCbc(algorithm) => &algorithm.name,
            GetKeyLengthAlgorithm::AesGcm(algorithm) => &algorithm.name,
            GetKeyLengthAlgorithm::AesKw(algorithm) => &algorithm.name,
            GetKeyLengthAlgorithm::Hmac(algorithm) => &algorithm.name,
            GetKeyLengthAlgorithm::Hkdf(algorithm) => &algorithm.name,
            GetKeyLengthAlgorithm::Pbkdf2(algorithm) => &algorithm.name,
            GetKeyLengthAlgorithm::AesOcb(algorithm) => &algorithm.name,
            GetKeyLengthAlgorithm::ChaCha20Poly1305(algorithm) => &algorithm.name,
            GetKeyLengthAlgorithm::Argon2(algorithm) => &algorithm.name,
        }
    }
}

impl GetKeyLengthAlgorithm {
    fn get_key_length(&self) -> Result<Option<u32>, Error> {
        match self {
            GetKeyLengthAlgorithm::AesCtr(algorithm) => {
                aes_ctr_operation::get_key_length(algorithm)
            },
            GetKeyLengthAlgorithm::AesCbc(algorithm) => {
                aes_cbc_operation::get_key_length(algorithm)
            },
            GetKeyLengthAlgorithm::AesGcm(algorithm) => {
                aes_gcm_operation::get_key_length(algorithm)
            },
            GetKeyLengthAlgorithm::AesKw(algorithm) => aes_kw_operation::get_key_length(algorithm),
            GetKeyLengthAlgorithm::Hmac(algorithm) => hmac_operation::get_key_length(algorithm),
            GetKeyLengthAlgorithm::Hkdf(_algorithm) => hkdf_operation::get_key_length(),
            GetKeyLengthAlgorithm::Pbkdf2(_algorithm) => pbkdf2_operation::get_key_length(),
            GetKeyLengthAlgorithm::AesOcb(algorithm) => {
                aes_ocb_operation::get_key_length(algorithm)
            },
            GetKeyLengthAlgorithm::ChaCha20Poly1305(_algorithm) => {
                chacha20_poly1305_operation::get_key_length()
            },
            GetKeyLengthAlgorithm::Argon2(_algorithm) => argon2_operation::get_key_length(),
        }
    }
}

/// The value of the key "encapsulate" in the internal object supportedAlgorithms
struct EncapsulateOperation {}

impl Operation for EncapsulateOperation {
    type RegisteredAlgorithm = EncapsulateAlgorithm;
}

/// Normalized algorithm for the "encapsulate" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
enum EncapsulateAlgorithm {
    MlKem(SubtleAlgorithm),
}

impl NormalizedAlgorithm for EncapsulateAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::MlKem512 | CryptoAlgorithm::MlKem768 | CryptoAlgorithm::MlKem1024 => {
                Ok(EncapsulateAlgorithm::MlKem(value.try_into_with_cx(cx)?))
            },
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"encapsulate\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            EncapsulateAlgorithm::MlKem(algorithm) => &algorithm.name,
        }
    }
}

impl EncapsulateAlgorithm {
    fn encapsulate(&self, key: &CryptoKey) -> Result<SubtleEncapsulatedBits, Error> {
        match self {
            EncapsulateAlgorithm::MlKem(algorithm) => ml_kem_operation::encapsulate(algorithm, key),
        }
    }
}

// The value of the key "decapsulate" in the internal object supportedAlgorithms
struct DecapsulateOperation {}

impl Operation for DecapsulateOperation {
    type RegisteredAlgorithm = DecapsulateAlgorithm;
}

/// Normalized algorithm for the "decapsulate" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
enum DecapsulateAlgorithm {
    MlKem(SubtleAlgorithm),
}

impl NormalizedAlgorithm for DecapsulateAlgorithm {
    fn from_object_value(
        cx: &mut js::context::JSContext,
        algorithm_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match algorithm_name {
            CryptoAlgorithm::MlKem512 | CryptoAlgorithm::MlKem768 | CryptoAlgorithm::MlKem1024 => {
                Ok(DecapsulateAlgorithm::MlKem(value.try_into_with_cx(cx)?))
            },
            _ => Err(Error::NotSupported(Some(format!(
                "{} does not support \"decapsulate\" operation",
                algorithm_name.as_str()
            )))),
        }
    }

    fn name(&self) -> &str {
        match self {
            DecapsulateAlgorithm::MlKem(algorithm) => &algorithm.name,
        }
    }
}

impl DecapsulateAlgorithm {
    fn decapsulate(&self, key: &CryptoKey, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            DecapsulateAlgorithm::MlKem(algorithm) => {
                ml_kem_operation::decapsulate(algorithm, key, ciphertext)
            },
        }
    }
}
