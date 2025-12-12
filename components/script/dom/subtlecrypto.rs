/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod aes_operation;
mod argon2_operation;
mod chacha20_poly1305_operation;
mod cshake_operation;
mod ecdh_operation;
mod ecdsa_operation;
mod ed25519_operation;
mod hkdf_operation;
mod hmac_operation;
mod pbkdf2_operation;
mod rsa_oaep_operation;
mod rsa_pss_operation;
mod rsassa_pkcs1_v1_5_operation;
mod sha3_operation;
mod sha_operation;
mod x25519_operation;

use std::ptr;
use std::rc::Rc;
use std::str::FromStr;

use dom_struct::dom_struct;
use js::conversions::ConversionResult;
use js::jsapi::{Heap, JSObject};
use js::jsval::{ObjectValue, UndefinedValue};
use js::rust::wrappers::JS_ParseJSON;
use js::rust::{HandleValue, MutableHandleValue};
use js::typedarray::ArrayBufferU8;

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, CryptoKeyPair, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{
    AeadParams, AesCbcParams, AesCtrParams, AesDerivedKeyParams, AesGcmParams, AesKeyAlgorithm,
    AesKeyGenParams, Algorithm, AlgorithmIdentifier, Argon2Params, CShakeParams, EcKeyAlgorithm,
    EcKeyGenParams, EcKeyImportParams, EcdhKeyDeriveParams, EcdsaParams, HkdfParams,
    HmacImportParams, HmacKeyAlgorithm, HmacKeyGenParams, JsonWebKey, KeyAlgorithm, KeyFormat,
    Pbkdf2Params, RsaHashedImportParams, RsaHashedKeyAlgorithm, RsaKeyAlgorithm,
    RsaOtherPrimesInfo, SubtleCryptoMethods,
};
use crate::dom::bindings::codegen::UnionTypes::{
    ArrayBufferViewOrArrayBuffer, ArrayBufferViewOrArrayBufferOrJsonWebKey, ObjectOrString,
};
use crate::dom::bindings::conversions::{SafeFromJSValConvertible, SafeToJSValConvertible};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, serialize_jsval_to_json_utf8};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::cryptokey::{CryptoKey, CryptoKeyOrCryptoKeyPair};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::realms::InRealm;
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
const ALG_CHACHA20_POLY1305: &str = "ChaCha20-Poly1305";
const ALG_SHA3_256: &str = "SHA3-256";
const ALG_SHA3_384: &str = "SHA3-384";
const ALG_SHA3_512: &str = "SHA3-512";
const ALG_CSHAKE_128: &str = "cSHAKE128";
const ALG_CSHAKE_256: &str = "cSHAKE256";
const ALG_ARGON2D: &str = "Argon2d";
const ALG_ARGON2I: &str = "Argon2i";
const ALG_ARGON2ID: &str = "Argon2id";

static SUPPORTED_ALGORITHMS: &[&str] = &[
    ALG_RSASSA_PKCS1_V1_5,
    ALG_RSA_PSS,
    ALG_RSA_OAEP,
    ALG_ECDSA,
    ALG_ECDH,
    ALG_ED25519,
    ALG_X25519,
    ALG_AES_CTR,
    ALG_AES_CBC,
    ALG_AES_GCM,
    ALG_AES_KW,
    ALG_HMAC,
    ALG_SHA1,
    ALG_SHA256,
    ALG_SHA384,
    ALG_SHA512,
    ALG_HKDF,
    ALG_PBKDF2,
    ALG_CHACHA20_POLY1305,
    ALG_SHA3_256,
    ALG_SHA3_384,
    ALG_SHA3_512,
    ALG_CSHAKE_128,
    ALG_CSHAKE_256,
    ALG_ARGON2D,
    ALG_ARGON2I,
    ALG_ARGON2ID,
];

// Named elliptic curves
const NAMED_CURVE_P256: &str = "P-256";
const NAMED_CURVE_P384: &str = "P-384";
const NAMED_CURVE_P521: &str = "P-521";

static SUPPORTED_CURVES: &[&str] = &[NAMED_CURVE_P256, NAMED_CURVE_P384, NAMED_CURVE_P521];

/// <https://w3c.github.io/webcrypto/#supported-operation>
#[expect(dead_code)]
enum Operation {
    Encrypt,
    Decrypt,
    Sign,
    Verify,
    Digest,
    GenerateKey,
    DeriveKey,
    DeriveBits,
    ImportKey,
    ExportKey,
    WrapKey,
    UnwrapKey,
    GetKeyLength,
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

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<SubtleCrypto> {
        reflect_dom_object(Box::new(SubtleCrypto::new_inherited()), global, can_gc)
    }

    /// Queue a global task on the crypto task source, given realm's global object, to resolve
    /// promise with the result of creating an ArrayBuffer in realm, containing data. If it fails
    /// to create buffer source, reject promise with a JSFailedError.
    fn resolve_promise_with_data(&self, promise: Rc<Promise>, data: Vec<u8>) {
        let trusted_promise = TrustedPromise::new(promise);
        self.global().task_manager().crypto_task_source().queue(
            task!(resolve_data: move || {
                let promise = trusted_promise.root();

                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());
                match create_buffer_source::<ArrayBufferU8>(cx, &data, array_buffer_ptr.handle_mut(), CanGc::note()) {
                    Ok(_) => promise.resolve_native(&*array_buffer_ptr, CanGc::note()),
                    Err(_) => promise.reject_error(Error::JSFailed, CanGc::note()),
                }
            }),
        );
    }

    /// Queue a global task on the crypto task source, given realm's global object, to resolve
    /// promise with the result of converting a JsonWebKey dictionary to an ECMAScript Object in
    /// realm, as defined by [WebIDL].
    fn resolve_promise_with_jwk(&self, promise: Rc<Promise>, jwk: Box<JsonWebKey>) {
        // NOTE: Serialize the JsonWebKey dictionary by stringifying it, in order to pass it to
        // other threads.
        let cx = GlobalScope::get_cx();
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
            .queue(task!(resolve_jwk: move || {
                let subtle = trusted_subtle.root();
                let promise = trusted_promise.root();

                let cx = GlobalScope::get_cx();
                match JsonWebKey::parse(cx, stringified_jwk.as_bytes()) {
                    Ok(jwk) => {
                        rooted!(in(*cx) let mut rval = UndefinedValue());
                        jwk.safe_to_jsval(cx, rval.handle_mut(), CanGc::note());
                        rooted!(in(*cx) let mut object = rval.to_object());
                        promise.resolve_native(&*object, CanGc::note());
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
            .queue(task!(resolve_key: move || {
                let key = trusted_key.root();
                let promise = trusted_promise.root();
                promise.resolve_native(&key, CanGc::note());
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
            .queue(task!(resolve_key: move || {
                let key_pair = CryptoKeyPair {
                    privateKey: trusted_private_key.map(|trusted_key| trusted_key.root()),
                    publicKey: trusted_public_key.map(|trusted_key| trusted_key.root()),
                };
                let promise = trusted_promise.root();
                promise.resolve_native(&key_pair, CanGc::note());
            }));
    }

    /// Queue a global task on the crypto task source, given realm's global object, to resolve
    /// promise with a bool value.
    fn resolve_promise_with_bool(&self, promise: Rc<Promise>, result: bool) {
        let trusted_promise = TrustedPromise::new(promise);
        self.global().task_manager().crypto_task_source().queue(
            task!(generate_key_result: move || {
                let promise = trusted_promise.root();
                promise.resolve_native(&result, CanGc::note());
            }),
        );
    }

    /// Queue a global task on the crypto task source, given realm's global object, to reject
    /// promise with an error.
    fn reject_promise_with_error(&self, promise: Rc<Promise>, error: Error) {
        let trusted_promise = TrustedPromise::new(promise);
        self.global()
            .task_manager()
            .crypto_task_source()
            .queue(task!(reject_error: move || {
                let promise = trusted_promise.root();
                promise.reject_error(error, CanGc::note());
            }));
    }
}

impl SubtleCryptoMethods<crate::DomTypeHolder> for SubtleCrypto {
    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-encrypt>
    fn Encrypt(
        &self,
        cx: JSContext,
        algorithm: AlgorithmIdentifier,
        key: &CryptoKey,
        data: ArrayBufferViewOrArrayBuffer,
        comp: InRealm,
        can_gc: CanGc,
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
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm =
            match normalize_algorithm(cx, &Operation::Encrypt, &algorithm, can_gc) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, can_gc);
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
        cx: JSContext,
        algorithm: AlgorithmIdentifier,
        key: &CryptoKey,
        data: ArrayBufferViewOrArrayBuffer,
        comp: InRealm,
        can_gc: CanGc,
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
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm =
            match normalize_algorithm(cx, &Operation::Decrypt, &algorithm, can_gc) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, can_gc);
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
        cx: JSContext,
        algorithm: AlgorithmIdentifier,
        key: &CryptoKey,
        data: ArrayBufferViewOrArrayBuffer,
        comp: InRealm,
        can_gc: CanGc,
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
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm =
            match normalize_algorithm(cx, &Operation::Sign, &algorithm, can_gc) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, can_gc);
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
        cx: JSContext,
        algorithm: AlgorithmIdentifier,
        key: &CryptoKey,
        signature: ArrayBufferViewOrArrayBuffer,
        data: ArrayBufferViewOrArrayBuffer,
        comp: InRealm,
        can_gc: CanGc,
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
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm =
            match normalize_algorithm(cx, &Operation::Verify, &algorithm, can_gc) {
                Ok(algorithm) => algorithm,
                Err(error) => {
                    promise.reject_error(error, can_gc);
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
        cx: JSContext,
        algorithm: AlgorithmIdentifier,
        data: ArrayBufferViewOrArrayBuffer,
        comp: InRealm,
        can_gc: CanGc,
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
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm =
            match normalize_algorithm(cx, &Operation::Digest, &algorithm, can_gc) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, can_gc);
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
            .queue(task!(generate_key: move || {
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
        cx: JSContext,
        algorithm: AlgorithmIdentifier,
        extractable: bool,
        key_usages: Vec<KeyUsage>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm, extractable and usages be the algorithm, extractable and
        // keyUsages parameters passed to the generateKey() method, respectively.

        // Step 2. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set
        // to algorithm and op set to "generateKey".
        // Step 3. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm =
            match normalize_algorithm(cx, &Operation::GenerateKey, &algorithm, can_gc) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, can_gc);
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
            .queue(task!(generate_key: move || {
                let subtle = trusted_subtle.root();
                let promise = trusted_promise.root();

                // Step 7. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 8. Let result be the result of performing the generate key operation
                // specified by normalizedAlgorithm using algorithm, extractable and usages.
                let result = match normalized_algorithm.generate_key(
                    &subtle.global(),
                    extractable,
                    key_usages,
                    CanGc::note(),
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
                // TODO: Implement CryptoKeyPair case
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
                        if crypto_key_pair.privateKey.as_ref().is_none_or(|private_key| private_key.usages().is_empty()) {
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
        cx: JSContext,
        algorithm: AlgorithmIdentifier,
        base_key: &CryptoKey,
        derived_key_type: AlgorithmIdentifier,
        extractable: bool,
        usages: Vec<KeyUsage>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm, baseKey, derivedKeyType, extractable and usages be the algorithm,
        // baseKey, derivedKeyType, extractable and keyUsages parameters passed to the deriveKey()
        // method, respectively.
        // NOTE: We did that in method parameter.

        // Step 2. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set
        // to algorithm and op set to "deriveBits".
        // Step 3. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm =
            match normalize_algorithm(cx, &Operation::DeriveBits, &algorithm, can_gc) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, can_gc);
                    return promise;
                },
            };

        // Step 4. Let normalizedDerivedKeyAlgorithmImport be the result of normalizing an
        // algorithm, with alg set to derivedKeyType and op set to "importKey".
        // Step 5. If an error occurred, return a Promise rejected with
        // normalizedDerivedKeyAlgorithmImport.
        let normalized_derived_key_algorithm_import =
            match normalize_algorithm(cx, &Operation::ImportKey, &derived_key_type, can_gc) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, can_gc);
                    return promise;
                },
            };

        // Step 6. Let normalizedDerivedKeyAlgorithmLength be the result of normalizing an
        // algorithm, with alg set to derivedKeyType and op set to "get key length".
        // Step 7. If an error occurred, return a Promise rejected with
        // normalizedDerivedKeyAlgorithmLength.
        let normalized_derived_key_algorithm_length =
            match normalize_algorithm(cx, &Operation::GetKeyLength, &derived_key_type, can_gc) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, can_gc);
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
            task!(derive_key: move || {
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
                    &subtle.global(),
                    KeyFormat::Raw_secret,
                    &secret,
                    extractable,
                    usages.clone(),
                    CanGc::note(),
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
        cx: JSContext,
        algorithm: AlgorithmIdentifier,
        base_key: &CryptoKey,
        length: Option<u32>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm, baseKey and length, be the algorithm, baseKey and length
        // parameters passed to the deriveBits() method, respectively.
        // NOTE: We did that in method parameter.

        // Step 2. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set
        // to algorithm and op set to "deriveBits".
        // Step 3. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm =
            match normalize_algorithm(cx, &Operation::DeriveBits, &algorithm, can_gc) {
                Ok(normalized_algorithm) => normalized_algorithm,
                Err(error) => {
                    promise.reject_error(error, can_gc);
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
        cx: JSContext,
        format: KeyFormat,
        key_data: ArrayBufferViewOrArrayBufferOrJsonWebKey,
        algorithm: AlgorithmIdentifier,
        extractable: bool,
        key_usages: Vec<KeyUsage>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 1. Let format, algorithm, extractable and usages, be the format, algorithm,
        // extractable and keyUsages parameters passed to the importKey() method, respectively.

        // Step 5. Let realm be the relevant realm of this.
        // Step 6. Let promise be a new Promise.
        let promise = Promise::new_in_current_realm(comp, can_gc);

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
                            Error::Type("The keyData type does not match the format".to_string()),
                            can_gc,
                        );
                        return promise;
                    },

                    ArrayBufferViewOrArrayBufferOrJsonWebKey::JsonWebKey(jwk) => {
                        // Step 2.2. Let keyData be the keyData parameter passed to the importKey() method.
                        //
                        // NOTE: Serialize JsonWebKey throught stringifying it.
                        // JsonWebKey::stringify internally relies on ToJSON, so it will raise an
                        // exception when a JS error is thrown. When this happens, we report the
                        // error.
                        match jwk.stringify(cx) {
                            Ok(stringified) => stringified.as_bytes().to_vec(),
                            Err(error) => {
                                promise.reject_error(error, can_gc);
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
                            Error::Type("The keyData type does not match the format".to_string()),
                            can_gc,
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
        let normalized_algorithm =
            match normalize_algorithm(cx, &Operation::ImportKey, &algorithm, can_gc) {
                Ok(algorithm) => algorithm,
                Err(error) => {
                    promise.reject_error(error, can_gc);
                    return promise;
                },
            };

        // Step 7. Return promise and perform the remaining steps in parallel.
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(import_key: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();

                // Step 8. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 9. Let result be the CryptoKey object that results from performing the
                // import key operation specified by normalizedAlgorithm using keyData, algorithm,
                // format, extractable and usages.
                let result = match normalized_algorithm.import_key(
                    &subtle.global(),
                    format,
                    &key_data,
                    extractable,
                    key_usages.clone(),
                    CanGc::note()
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
                result.set_usages(&key_usages);

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
    fn ExportKey(
        &self,
        format: KeyFormat,
        key: &CryptoKey,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 1. Let format and key be the format and key parameters passed to the exportKey()
        // method, respectively.
        // NOTE: We did that in method parameter.

        // Step 2. Let realm be the relevant realm of this.
        // Step 3. Let promise be a new Promise.
        let promise = Promise::new_in_current_realm(comp, can_gc);

        // Step 4. Return promise and perform the remaining steps in parallel.
        let trusted_subtle = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(export_key: move || {
                let subtle = trusted_subtle.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();

                // Step 5. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 6. If the name member of the [[algorithm]] internal slot of key does not
                // identify a registered algorithm that supports the export key operation, then
                // throw a NotSupportedError.
                if matches!(
                    key.algorithm().name(),
                    ALG_SHA1 | ALG_SHA256 | ALG_SHA384 | ALG_SHA512 | ALG_HKDF | ALG_PBKDF2
                ) {
                    subtle.reject_promise_with_error(promise, Error::NotSupported(None));
                    return;
                }

                // Step 7. If the [[extractable]] internal slot of key is false, then throw an
                // InvalidAccessError.
                if !key.Extractable() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 8. Let result be the result of performing the export key operation
                // specified by the [[algorithm]] internal slot of key using key and format.
                let result = match perform_export_key_operation(format, &key) {
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
                        subtle.resolve_promise_with_jwk(promise, jwk);
                    },
                }
            }));
        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-wrapKey>
    fn WrapKey(
        &self,
        cx: JSContext,
        format: KeyFormat,
        key: &CryptoKey,
        wrapping_key: &CryptoKey,
        algorithm: AlgorithmIdentifier,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 1. Let format, key, wrappingKey and algorithm be the format, key, wrappingKey and
        // wrapAlgorithm parameters passed to the wrapKey() method, respectively.
        // NOTE: We did that in method parameter.

        // Step 2. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set
        // to algorithm and op set to "wrapKey".
        let mut normalized_algorithm_result =
            normalize_algorithm(cx, &Operation::WrapKey, &algorithm, can_gc);

        // Step 3. If an error occurred, let normalizedAlgorithm be the result of normalizing an
        // algorithm, with alg set to algorithm and op set to "encrypt".
        if normalized_algorithm_result.is_err() {
            normalized_algorithm_result =
                normalize_algorithm(cx, &Operation::Encrypt, &algorithm, can_gc);
        }

        // Step 4. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm = match normalized_algorithm_result {
            Ok(normalized_algorithm) => normalized_algorithm,
            Err(error) => {
                promise.reject_error(error, can_gc);
                return promise;
            },
        };

        // Step 5. Let realm be the relevant realm of this.
        // Step 6. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 4.

        // Step 7. Return promise and perform the remaining steps in parallel.
        let trusted_subtle = Trusted::new(self);
        let trusted_key = Trusted::new(key);
        let trusted_wrapping_key = Trusted::new(wrapping_key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(wrap_key: move || {
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
                if normalized_algorithm.name() != wrapping_key.algorithm().name() {
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

                if matches!(
                    key.algorithm().name(),
                    ALG_SHA1 | ALG_SHA256 | ALG_SHA384 | ALG_SHA512 | ALG_HKDF | ALG_PBKDF2
                ) {
                    subtle.reject_promise_with_error(promise, Error::NotSupported(None));
                    return;
                }


                // Step 12. If the [[extractable]] internal slot of key is false, then throw an
                // InvalidAccessError.
                if !key.Extractable() {
                    subtle.reject_promise_with_error(promise, Error::InvalidAccess(None));
                    return;
                }

                // Step 13. Let exportedKey be the result of performing the export key operation
                // specified by the [[algorithm]] internal slot of key using key and format.
                let exported_key = match perform_export_key_operation(format, &key) {
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
                let cx = GlobalScope::get_cx();
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
                let mut result = normalized_algorithm.wrap_key(&wrapping_key, &bytes);
                if result.is_err() {
                    result = normalized_algorithm.encrypt(&wrapping_key, &bytes);
                }
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
        cx: JSContext,
        format: KeyFormat,
        wrapped_key: ArrayBufferViewOrArrayBuffer,
        unwrapping_key: &CryptoKey,
        algorithm: AlgorithmIdentifier,
        unwrapped_key_algorithm: AlgorithmIdentifier,
        extractable: bool,
        usages: Vec<KeyUsage>,
        comp: InRealm,
        can_gc: CanGc,
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
        let mut normalized_algorithm =
            normalize_algorithm(cx, &Operation::UnwrapKey, &algorithm, can_gc);

        // Step 4. If an error occurred, let normalizedAlgorithm be the result of normalizing an
        // algorithm, with alg set to algorithm and op set to "decrypt".
        if normalized_algorithm.is_err() {
            normalized_algorithm = normalize_algorithm(cx, &Operation::Decrypt, &algorithm, can_gc);
        }

        // Step 5. If an error occurred, return a Promise rejected with normalizedAlgorithm.
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm = match normalized_algorithm {
            Ok(algorithm) => algorithm,
            Err(error) => {
                promise.reject_error(error, can_gc);
                return promise;
            },
        };

        // Step 6. Let normalizedKeyAlgorithm be the result of normalizing an algorithm, with alg
        // set to unwrappedKeyAlgorithm and op set to "importKey".
        // Step 7. If an error occurred, return a Promise rejected with normalizedKeyAlgorithm.
        let normalized_key_algorithm = match normalize_algorithm(
            cx,
            &Operation::ImportKey,
            &unwrapped_key_algorithm,
            can_gc,
        ) {
            Ok(algorithm) => algorithm,
            Err(error) => {
                promise.reject_error(error, can_gc);
                return promise;
            },
        };

        // Step 8. Let realm be the relevant realm of this.
        // Step 9. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 5.

        // Step 10. Return promise and perform the remaining steps in parallel.
        let trusted_subtle = Trusted::new(self);
        let trusted_unwrapping_key = Trusted::new(unwrapping_key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global().task_manager().dom_manipulation_task_source().queue(
            task!(unwrap_key: move || {
                let subtle = trusted_subtle.root();
                let unwrapping_key = trusted_unwrapping_key.root();
                let promise = trusted_promise.root();

                // Step 11. If the following steps or referenced procedures say to throw an error,
                // queue a global task on the crypto task source, given realm's global object, to
                // reject promise with the returned error; and then terminate the algorithm.

                // Step 12. If the name member of normalizedAlgorithm is not equal to the name
                // attribute of the [[algorithm]] internal slot of unwrappingKey then throw an
                // InvalidAccessError.
                if normalized_algorithm.name() != unwrapping_key.algorithm().name() {
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
                let mut bytes = normalized_algorithm.unwrap_key(&unwrapping_key, &wrapped_key);
                if bytes.is_err() {
                    bytes = normalized_algorithm.decrypt(&unwrapping_key, &wrapped_key);
                }
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
                let cx = GlobalScope::get_cx();
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
                    &subtle.global(),
                    format,
                    &key,
                    extractable,
                    usages.clone(),
                    CanGc::note(),
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
}

// These "subtle" structs are proxies for the codegen'd dicts which don't hold a DOMString
// so they can be sent safely when running steps in parallel.

/// <https://w3c.github.io/webcrypto/#dfn-Algorithm>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,
}

impl From<Algorithm> for SubtleAlgorithm {
    fn from(params: Algorithm) -> Self {
        SubtleAlgorithm {
            name: params.name.to_string(),
        }
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-KeyAlgorithm>
#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct SubtleKeyAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-keyalgorithm-name>
    name: String,
}

impl SubtleKeyAlgorithm {
    fn block_size_in_bits(&self) -> Result<u32, Error> {
        let size = match self.name.as_str() {
            ALG_SHA1 => 512,
            ALG_SHA256 => 512,
            ALG_SHA384 => 1024,
            ALG_SHA512 => 1024,
            _ => {
                return Err(Error::NotSupported(None));
            },
        };

        Ok(size)
    }
}

impl From<NormalizedAlgorithm> for SubtleKeyAlgorithm {
    fn from(value: NormalizedAlgorithm) -> Self {
        SubtleKeyAlgorithm {
            name: value.name().to_string(),
        }
    }
}

impl SafeToJSValConvertible for SubtleKeyAlgorithm {
    fn safe_to_jsval(&self, cx: JSContext, rval: MutableHandleValue, can_gc: CanGc) {
        let dictionary = KeyAlgorithm {
            name: self.name.clone().into(),
        };
        dictionary.safe_to_jsval(cx, rval, can_gc);
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
    hash: Box<NormalizedAlgorithm>,
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
    hash: Box<NormalizedAlgorithm>,
}

impl TryFrom<RootedTraceableBox<RsaHashedImportParams>> for SubtleRsaHashedImportParams {
    type Error = Error;

    fn try_from(value: RootedTraceableBox<RsaHashedImportParams>) -> Result<Self, Self::Error> {
        let cx = GlobalScope::get_cx();
        Ok(SubtleRsaHashedImportParams {
            name: value.parent.name.to_string(),
            hash: Box::new(normalize_algorithm(
                cx,
                &Operation::Digest,
                &value.hash,
                CanGc::note(),
            )?),
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-EcdsaParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleEcdsaParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-EcdsaParams-hash>
    hash: SubtleKeyAlgorithm,
}

impl TryFrom<RootedTraceableBox<EcdsaParams>> for SubtleEcdsaParams {
    type Error = Error;

    fn try_from(value: RootedTraceableBox<EcdsaParams>) -> Result<Self, Error> {
        let cx = GlobalScope::get_cx();
        let hash = normalize_algorithm(cx, &Operation::Digest, &value.hash, CanGc::note())?;
        Ok(SubtleEcdsaParams {
            name: value.parent.name.to_string(),
            hash: hash.into(),
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

impl From<EcKeyGenParams> for SubtleEcKeyGenParams {
    fn from(value: EcKeyGenParams) -> Self {
        SubtleEcKeyGenParams {
            name: value.parent.name.to_string(),
            named_curve: value.namedCurve.to_string(),
        }
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

impl From<EcKeyImportParams> for SubtleEcKeyImportParams {
    fn from(value: EcKeyImportParams) -> Self {
        SubtleEcKeyImportParams {
            name: value.parent.name.to_string(),
            named_curve: value.namedCurve.to_string(),
        }
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

impl From<EcdhKeyDeriveParams> for SubtleEcdhKeyDeriveParams {
    fn from(value: EcdhKeyDeriveParams) -> Self {
        SubtleEcdhKeyDeriveParams {
            name: value.parent.name.to_string(),
            public: Trusted::new(&value.public),
        }
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

impl From<RootedTraceableBox<AesCtrParams>> for SubtleAesCtrParams {
    fn from(params: RootedTraceableBox<AesCtrParams>) -> Self {
        let counter = match &params.counter {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        SubtleAesCtrParams {
            name: params.parent.name.to_string(),
            counter,
            length: params.length,
        }
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

impl From<AesKeyGenParams> for SubtleAesKeyGenParams {
    fn from(params: AesKeyGenParams) -> Self {
        SubtleAesKeyGenParams {
            name: params.parent.name.to_string(),
            length: params.length,
        }
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

impl From<AesDerivedKeyParams> for SubtleAesDerivedKeyParams {
    fn from(params: AesDerivedKeyParams) -> Self {
        SubtleAesDerivedKeyParams {
            name: params.parent.name.to_string(),
            length: params.length,
        }
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

impl From<RootedTraceableBox<AesCbcParams>> for SubtleAesCbcParams {
    fn from(params: RootedTraceableBox<AesCbcParams>) -> Self {
        let iv = match &params.iv {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        SubtleAesCbcParams {
            name: params.parent.name.to_string(),
            iv,
        }
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

impl From<RootedTraceableBox<AesGcmParams>> for SubtleAesGcmParams {
    fn from(params: RootedTraceableBox<AesGcmParams>) -> Self {
        let iv = match &params.iv {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        let additional_data = params.additionalData.as_ref().map(|data| match data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        });

        SubtleAesGcmParams {
            name: params.parent.name.to_string(),
            iv,
            additional_data,
            tag_length: params.tagLength,
        }
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-HmacImportParams>
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleHmacImportParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacImportParams-hash>
    hash: SubtleKeyAlgorithm,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacImportParams-length>
    length: Option<u32>,
}

impl TryFrom<RootedTraceableBox<HmacImportParams>> for SubtleHmacImportParams {
    type Error = Error;

    fn try_from(params: RootedTraceableBox<HmacImportParams>) -> Result<Self, Error> {
        let cx = GlobalScope::get_cx();
        let hash = normalize_algorithm(cx, &Operation::Digest, &params.hash, CanGc::note())?;
        Ok(SubtleHmacImportParams {
            name: params.parent.name.to_string(),
            hash: hash.into(),
            length: params.length,
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
#[derive(Clone, Debug, MallocSizeOf)]
struct SubtleHmacKeyGenParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyGenParams-hash>
    hash: SubtleKeyAlgorithm,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyGenParams-length>
    length: Option<u32>,
}

impl TryFrom<RootedTraceableBox<HmacKeyGenParams>> for SubtleHmacKeyGenParams {
    type Error = Error;

    fn try_from(params: RootedTraceableBox<HmacKeyGenParams>) -> Result<Self, Error> {
        let cx = GlobalScope::get_cx();
        let hash = normalize_algorithm(cx, &Operation::Digest, &params.hash, CanGc::note())?;
        Ok(SubtleHmacKeyGenParams {
            name: params.parent.name.to_string(),
            hash: hash.into(),
            length: params.length,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-HkdfParams>
#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct SubtleHkdfParams {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-HkdfParams-hash>
    hash: SubtleKeyAlgorithm,

    /// <https://w3c.github.io/webcrypto/#dfn-HkdfParams-salt>
    salt: Vec<u8>,

    /// <https://w3c.github.io/webcrypto/#dfn-HkdfParams-info>
    info: Vec<u8>,
}

impl TryFrom<RootedTraceableBox<HkdfParams>> for SubtleHkdfParams {
    type Error = Error;

    fn try_from(params: RootedTraceableBox<HkdfParams>) -> Result<Self, Error> {
        let cx = GlobalScope::get_cx();
        let hash = normalize_algorithm(cx, &Operation::Digest, &params.hash, CanGc::note())?;
        let salt = match &params.salt {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        let info = match &params.info {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        Ok(SubtleHkdfParams {
            name: params.parent.name.to_string(),
            hash: hash.into(),
            salt,
            info,
        })
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params>
#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct SubtlePbkdf2Params {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    name: String,

    /// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params-salt>
    salt: Vec<u8>,

    /// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params-iterations>
    iterations: u32,

    /// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params-hash>
    hash: SubtleKeyAlgorithm,
}

impl TryFrom<RootedTraceableBox<Pbkdf2Params>> for SubtlePbkdf2Params {
    type Error = Error;

    fn try_from(params: RootedTraceableBox<Pbkdf2Params>) -> Result<Self, Error> {
        let cx = GlobalScope::get_cx();
        let salt = match &params.salt {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        let hash = normalize_algorithm(cx, &Operation::Digest, &params.hash, CanGc::note())?;
        Ok(SubtlePbkdf2Params {
            name: params.parent.name.to_string(),
            salt,
            iterations: params.iterations,
            hash: hash.into(),
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

impl From<RootedTraceableBox<AeadParams>> for SubtleAeadParams {
    fn from(value: RootedTraceableBox<AeadParams>) -> Self {
        let iv = match &value.iv {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        let additional_data = value.additionalData.as_ref().map(|data| match data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        });

        SubtleAeadParams {
            name: value.parent.name.to_string(),
            iv,
            additional_data,
            tag_length: value.tagLength,
        }
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

impl From<RootedTraceableBox<CShakeParams>> for SubtleCShakeParams {
    fn from(value: RootedTraceableBox<CShakeParams>) -> Self {
        let function_name = value
            .functionName
            .as_ref()
            .map(|function_name| match function_name {
                ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
                ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
            });
        let customization = value
            .customization
            .as_ref()
            .map(|customization| match customization {
                ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
                ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
            });
        SubtleCShakeParams {
            name: value.parent.name.to_string(),
            length: value.length,
            function_name,
            customization,
        }
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

impl From<RootedTraceableBox<Argon2Params>> for SubtleArgon2Params {
    fn from(value: RootedTraceableBox<Argon2Params>) -> Self {
        let nonce = match &value.nonce {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        let secret_value = value
            .secretValue
            .as_ref()
            .map(|secret_value| match secret_value {
                ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
                ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
            });
        let associated_data =
            value
                .associatedData
                .as_ref()
                .map(|associated_data| match associated_data {
                    ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
                    ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
                });
        SubtleArgon2Params {
            name: value.parent.name.to_string(),
            nonce,
            parallelism: value.parallelism,
            memory: value.memory,
            passes: value.passes,
            version: value.version,
            secret_value,
            associated_data,
        }
    }
}

/// Helper to abstract the conversion process of a JS value into many different WebIDL dictionaries.
fn dictionary_from_jsval<T>(cx: JSContext, value: HandleValue, can_gc: CanGc) -> Fallible<T>
where
    T: SafeFromJSValConvertible<Config = ()>,
{
    let conversion = T::safe_from_jsval(cx, value, (), can_gc).map_err(|_| Error::JSFailed)?;
    match conversion {
        ConversionResult::Success(dictionary) => Ok(dictionary),
        ConversionResult::Failure(error) => Err(Error::Type(error.into())),
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

impl From<NormalizedAlgorithm> for KeyAlgorithmAndDerivatives {
    fn from(value: NormalizedAlgorithm) -> Self {
        KeyAlgorithmAndDerivatives::KeyAlgorithm(SubtleKeyAlgorithm {
            name: value.name().to_string(),
        })
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

#[expect(unused)]
trait RsaOtherPrimesInfoExt {
    fn from_value(value: &serde_json::Value) -> Result<RsaOtherPrimesInfo, Error>;
}

impl RsaOtherPrimesInfoExt for RsaOtherPrimesInfo {
    fn from_value(value: &serde_json::Value) -> Result<RsaOtherPrimesInfo, Error> {
        let serde_json::Value::Object(object) = value else {
            return Err(Error::Data(None));
        };

        let mut rsa_other_primes_info: RsaOtherPrimesInfo = Default::default();
        for (key, value) in object {
            match key.as_str() {
                "r" => {
                    rsa_other_primes_info.r =
                        Some(DOMString::from(value.as_str().ok_or(Error::Data(None))?))
                },
                "d" => {
                    rsa_other_primes_info.d =
                        Some(DOMString::from(value.as_str().ok_or(Error::Data(None))?))
                },
                "t" => {
                    rsa_other_primes_info.t =
                        Some(DOMString::from(value.as_str().ok_or(Error::Data(None))?))
                },
                _ => {
                    // Additional members can be present in the JWK; if not understood by
                    // implementations encountering them, they MUST be ignored.
                },
            }
        }

        Ok(rsa_other_primes_info)
    }
}

trait JsonWebKeyExt {
    fn parse(cx: JSContext, data: &[u8]) -> Result<JsonWebKey, Error>;
    fn stringify(&self, cx: JSContext) -> Result<DOMString, Error>;
    fn get_usages_from_key_ops(&self) -> Result<Vec<KeyUsage>, Error>;
    #[expect(unused)]
    fn get_rsa_other_primes_info_from_oth(&self) -> Result<&[RsaOtherPrimesInfo], Error>;
    fn check_key_ops(&self, specified_usages: &[KeyUsage]) -> Result<(), Error>;
}

impl JsonWebKeyExt for JsonWebKey {
    /// <https://w3c.github.io/webcrypto/#concept-parse-a-jwk>
    #[expect(unsafe_code)]
    fn parse(cx: JSContext, data: &[u8]) -> Result<JsonWebKey, Error> {
        // Step 1. Let data be the sequence of bytes to be parsed.
        // (It is given as a method paramter.)

        // Step 2. Let json be the Unicode string that results from interpreting data according to UTF-8.
        let json = String::from_utf8_lossy(data);

        // Step 3. Convert json to UTF-16.
        let json: Vec<_> = json.encode_utf16().collect();

        // Step 4. Let result be the object literal that results from executing the JSON.parse
        // internal function in the context of a new global object, with text argument set to a
        // JavaScript String containing json.
        rooted!(in(*cx) let mut result = UndefinedValue());
        unsafe {
            if !JS_ParseJSON(*cx, json.as_ptr(), json.len() as u32, result.handle_mut()) {
                return Err(Error::JSFailed);
            }
        }

        // Step 5. Let key be the result of converting result to the IDL dictionary type of JsonWebKey.
        let key = match JsonWebKey::new(cx, result.handle(), CanGc::note()) {
            Ok(ConversionResult::Success(key)) => key,
            Ok(ConversionResult::Failure(error)) => {
                return Err(Error::Type(error.to_string()));
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
    fn stringify(&self, cx: JSContext) -> Result<DOMString, Error> {
        rooted!(in(*cx) let mut data = UndefinedValue());
        self.safe_to_jsval(cx, data.handle_mut(), CanGc::note());
        serialize_jsval_to_json_utf8(cx, data.handle())
    }

    fn get_usages_from_key_ops(&self) -> Result<Vec<KeyUsage>, Error> {
        let mut usages = vec![];
        for op in self.key_ops.as_ref().ok_or(Error::Data(None))? {
            usages.push(KeyUsage::from_str(&op.str()).map_err(|_| Error::Data(None))?);
        }
        Ok(usages)
    }

    fn get_rsa_other_primes_info_from_oth(&self) -> Result<&[RsaOtherPrimesInfo], Error> {
        self.oth.as_deref().ok_or(Error::Data(None))
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
}

/// The successful output of [`normalize_algorithm`], in form of an union type of (our "subtle"
/// binding of) IDL dictionary types.
///
/// <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm>
#[derive(Clone, MallocSizeOf)]
enum NormalizedAlgorithm {
    Algorithm(SubtleAlgorithm),
    RsaHashedImportParams(SubtleRsaHashedImportParams),
    EcdsaParams(SubtleEcdsaParams),
    EcKeyGenParams(SubtleEcKeyGenParams),
    EcKeyImportParams(SubtleEcKeyImportParams),
    EcdhKeyDeriveParams(SubtleEcdhKeyDeriveParams),
    AesCtrParams(SubtleAesCtrParams),
    AesKeyGenParams(SubtleAesKeyGenParams),
    AesDerivedKeyParams(SubtleAesDerivedKeyParams),
    AesCbcParams(SubtleAesCbcParams),
    AesGcmParams(SubtleAesGcmParams),
    HmacImportParams(SubtleHmacImportParams),
    HmacKeyGenParams(SubtleHmacKeyGenParams),
    HkdfParams(SubtleHkdfParams),
    Pbkdf2Params(SubtlePbkdf2Params),
    AeadParams(SubtleAeadParams),
    CShakeParams(SubtleCShakeParams),
    Argon2Params(SubtleArgon2Params),
}

/// <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm>
fn normalize_algorithm(
    cx: JSContext,
    op: &Operation,
    alg: &AlgorithmIdentifier,
    can_gc: CanGc,
) -> Result<NormalizedAlgorithm, Error> {
    match alg {
        // If alg is an instance of a DOMString:
        ObjectOrString::String(name) => {
            // Return the result of running the normalize an algorithm algorithm, with the alg set
            // to a new Algorithm dictionary whose name attribute is alg, and with the op set to
            // op.
            let alg = Algorithm {
                name: name.to_owned(),
            };
            rooted!(in(*cx) let mut alg_value = UndefinedValue());
            alg.safe_to_jsval(cx, alg_value.handle_mut(), CanGc::note());
            let alg_obj = RootedTraceableBox::new(Heap::default());
            alg_obj.set(alg_value.to_object());
            normalize_algorithm(cx, op, &ObjectOrString::Object(alg_obj), can_gc)
        },
        // If alg is an object:
        ObjectOrString::Object(obj) => {
            // Step 1. Let registeredAlgorithms be the associative container stored at the op key
            // of supportedAlgorithms.
            // NOTE: The supportedAlgorithms and registeredAlgorithms are expressed as match arms
            // in Step 5.2 - Step 10.

            // Stpe 2. Let initialAlg be the result of converting the ECMAScript object represented
            // by alg to the IDL dictionary type Algorithm, as defined by [WebIDL].
            // Step 3. If an error occurred, return the error and terminate this algorithm.
            rooted!(in(*cx) let value = ObjectValue(obj.get()));
            let initial_alg = dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;

            // Step 4. Let algName be the value of the name attribute of initialAlg.
            // Step 5.
            //     If registeredAlgorithms contains a key that is a case-insensitive string match
            //     for algName:
            //         Step 5.1. Set algName to the value of the matching key.
            //     Otherwise:
            //         Return a new NotSupportedError and terminate this algorithm.
            let Some(&alg_name) = SUPPORTED_ALGORITHMS.iter().find(|supported_algorithm| {
                supported_algorithm.eq_ignore_ascii_case(&initial_alg.name.str())
            }) else {
                return Err(Error::NotSupported(None));
            };

            // Step 5.2. Let desiredType be the IDL dictionary type stored at algName in
            // registeredAlgorithms.
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
            // NOTE: Instead of calculating the desiredType in Step 5.2 and filling in the IDL
            // dictionary in Step 7-10, we directly convert the JS object to our "subtle" binding
            // structs to complete Step 6, and put it in the NormalizedAlgorithm enum.
            //
            // NOTE: Step 10.1.3 is done by the `From` and `TryFrom` trait implementation of
            // "subtle" binding structs.
            let normalized_algorithm = match (alg_name, op) {
                // <>https://w3c.github.io/webcrypto/#rsassa-pkcs1-registration>
                (ALG_RSASSA_PKCS1_V1_5, Operation::ImportKey) => {
                    let mut params = dictionary_from_jsval::<
                        RootedTraceableBox<RsaHashedImportParams>,
                    >(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::RsaHashedImportParams(params.try_into()?)
                },

                // <https://w3c.github.io/webcrypto/#rsa-pss-registration>
                (ALG_RSA_PSS, Operation::ImportKey) => {
                    let mut params = dictionary_from_jsval::<
                        RootedTraceableBox<RsaHashedImportParams>,
                    >(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::RsaHashedImportParams(params.try_into()?)
                },

                // <https://w3c.github.io/webcrypto/#rsa-oaep-registration>
                (ALG_RSA_OAEP, Operation::ImportKey) => {
                    let mut params = dictionary_from_jsval::<
                        RootedTraceableBox<RsaHashedImportParams>,
                    >(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::RsaHashedImportParams(params.try_into()?)
                },

                // <https://w3c.github.io/webcrypto/#ecdsa-registration>
                (ALG_ECDSA, Operation::Sign) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<EcdsaParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::EcdsaParams(params.try_into()?)
                },
                (ALG_ECDSA, Operation::Verify) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<EcdsaParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::EcdsaParams(params.try_into()?)
                },
                (ALG_ECDSA, Operation::GenerateKey) => {
                    let mut params =
                        dictionary_from_jsval::<EcKeyGenParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::EcKeyGenParams(params.into())
                },
                (ALG_ECDSA, Operation::ImportKey) => {
                    let mut params =
                        dictionary_from_jsval::<EcKeyImportParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::EcKeyImportParams(params.into())
                },
                (ALG_ECDSA, Operation::ExportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },

                // <https://w3c.github.io/webcrypto/#ecdh-registration>
                (ALG_ECDH, Operation::GenerateKey) => {
                    let mut params =
                        dictionary_from_jsval::<EcKeyGenParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::EcKeyGenParams(params.into())
                },
                (ALG_ECDH, Operation::DeriveBits) => {
                    let mut params =
                        dictionary_from_jsval::<EcdhKeyDeriveParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::EcdhKeyDeriveParams(params.into())
                },
                (ALG_ECDH, Operation::ImportKey) => {
                    let mut params =
                        dictionary_from_jsval::<EcKeyImportParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::EcKeyImportParams(params.into())
                },
                (ALG_ECDH, Operation::ExportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },

                // <https://w3c.github.io/webcrypto/#ed25519-registration>
                (ALG_ED25519, Operation::Sign) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_ED25519, Operation::Verify) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_ED25519, Operation::GenerateKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_ED25519, Operation::ImportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_ED25519, Operation::ExportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },

                // <https://w3c.github.io/webcrypto/#x25519-registration>
                (ALG_X25519, Operation::DeriveBits) => {
                    let mut params =
                        dictionary_from_jsval::<EcdhKeyDeriveParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::EcdhKeyDeriveParams(params.into())
                },
                (ALG_X25519, Operation::GenerateKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_X25519, Operation::ImportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_X25519, Operation::ExportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },

                // <https://w3c.github.io/webcrypto/#aes-ctr-registration>
                (ALG_AES_CTR, Operation::Encrypt) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<AesCtrParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesCtrParams(params.into())
                },
                (ALG_AES_CTR, Operation::Decrypt) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<AesCtrParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesCtrParams(params.into())
                },
                (ALG_AES_CTR, Operation::GenerateKey) => {
                    let mut params =
                        dictionary_from_jsval::<AesKeyGenParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesKeyGenParams(params.into())
                },
                (ALG_AES_CTR, Operation::ImportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_AES_CTR, Operation::ExportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_AES_CTR, Operation::GetKeyLength) => {
                    let mut params =
                        dictionary_from_jsval::<AesDerivedKeyParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesDerivedKeyParams(params.into())
                },

                // <https://w3c.github.io/webcrypto/#aes-cbc-registration>
                (ALG_AES_CBC, Operation::Encrypt) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<AesCbcParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesCbcParams(params.into())
                },
                (ALG_AES_CBC, Operation::Decrypt) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<AesCbcParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesCbcParams(params.into())
                },
                (ALG_AES_CBC, Operation::GenerateKey) => {
                    let mut params =
                        dictionary_from_jsval::<AesKeyGenParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesKeyGenParams(params.into())
                },
                (ALG_AES_CBC, Operation::ImportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_AES_CBC, Operation::ExportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_AES_CBC, Operation::GetKeyLength) => {
                    let mut params =
                        dictionary_from_jsval::<AesDerivedKeyParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesDerivedKeyParams(params.into())
                },

                // <https://w3c.github.io/webcrypto/#aes-gcm-registration>
                (ALG_AES_GCM, Operation::Encrypt) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<AesGcmParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesGcmParams(params.into())
                },
                (ALG_AES_GCM, Operation::Decrypt) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<AesGcmParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesGcmParams(params.into())
                },
                (ALG_AES_GCM, Operation::GenerateKey) => {
                    let mut params =
                        dictionary_from_jsval::<AesKeyGenParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesKeyGenParams(params.into())
                },
                (ALG_AES_GCM, Operation::ImportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_AES_GCM, Operation::ExportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_AES_GCM, Operation::GetKeyLength) => {
                    let mut params =
                        dictionary_from_jsval::<AesDerivedKeyParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesDerivedKeyParams(params.into())
                },

                // <https://w3c.github.io/webcrypto/#aes-kw-registration>
                (ALG_AES_KW, Operation::WrapKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_AES_KW, Operation::UnwrapKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_AES_KW, Operation::GenerateKey) => {
                    let mut params =
                        dictionary_from_jsval::<AesKeyGenParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesKeyGenParams(params.into())
                },
                (ALG_AES_KW, Operation::ImportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_AES_KW, Operation::ExportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_AES_KW, Operation::GetKeyLength) => {
                    let mut params =
                        dictionary_from_jsval::<AesDerivedKeyParams>(cx, value.handle(), can_gc)?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AesDerivedKeyParams(params.into())
                },

                // <https://w3c.github.io/webcrypto/#hmac-registration>
                (ALG_HMAC, Operation::Sign) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_HMAC, Operation::Verify) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_HMAC, Operation::GenerateKey) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<HmacKeyGenParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::HmacKeyGenParams(params.try_into()?)
                },
                (ALG_HMAC, Operation::ImportKey) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<HmacImportParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::HmacImportParams(params.try_into()?)
                },
                (ALG_HMAC, Operation::ExportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_HMAC, Operation::GetKeyLength) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<HmacImportParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::HmacImportParams(params.try_into()?)
                },

                // <https://w3c.github.io/webcrypto/#sha-registration>
                (ALG_SHA1, Operation::Digest) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_SHA256, Operation::Digest) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_SHA384, Operation::Digest) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_SHA512, Operation::Digest) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },

                // <https://w3c.github.io/webcrypto/#hkdf-registration>
                (ALG_HKDF, Operation::DeriveBits) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<HkdfParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::HkdfParams(params.try_into()?)
                },
                (ALG_HKDF, Operation::ImportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_HKDF, Operation::GetKeyLength) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },

                // <https://w3c.github.io/webcrypto/#pbkdf2-registration>
                (ALG_PBKDF2, Operation::DeriveBits) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<Pbkdf2Params>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Pbkdf2Params(params.try_into()?)
                },
                (ALG_PBKDF2, Operation::ImportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_PBKDF2, Operation::GetKeyLength) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },

                // <https://wicg.github.io/webcrypto-modern-algos/#chacha20-poly1305-registration>
                (ALG_CHACHA20_POLY1305, Operation::Encrypt) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<AeadParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AeadParams(params.into())
                },
                (ALG_CHACHA20_POLY1305, Operation::Decrypt) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<AeadParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::AeadParams(params.into())
                },
                (ALG_CHACHA20_POLY1305, Operation::GenerateKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_CHACHA20_POLY1305, Operation::ImportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_CHACHA20_POLY1305, Operation::ExportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_CHACHA20_POLY1305, Operation::GetKeyLength) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },

                // <https://wicg.github.io/webcrypto-modern-algos/#sha3-registration>
                (ALG_SHA3_256, Operation::Digest) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_SHA3_384, Operation::Digest) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_SHA3_512, Operation::Digest) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },

                // <https://wicg.github.io/webcrypto-modern-algos/#cshake-registration>
                (ALG_CSHAKE_128, Operation::Digest) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<CShakeParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::CShakeParams(params.into())
                },
                (ALG_CSHAKE_256, Operation::Digest) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<CShakeParams>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::CShakeParams(params.into())
                },

                // <https://wicg.github.io/webcrypto-modern-algos/#argon2-registration>
                (ALG_ARGON2D | ALG_ARGON2I | ALG_ARGON2ID, Operation::DeriveBits) => {
                    let mut params = dictionary_from_jsval::<RootedTraceableBox<Argon2Params>>(
                        cx,
                        value.handle(),
                        can_gc,
                    )?;
                    params.parent.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Argon2Params(params.into())
                },
                (ALG_ARGON2D | ALG_ARGON2I | ALG_ARGON2ID, Operation::ImportKey) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },
                (ALG_ARGON2D | ALG_ARGON2I | ALG_ARGON2ID, Operation::GetKeyLength) => {
                    let mut params =
                        dictionary_from_jsval::<Algorithm>(cx, value.handle(), can_gc)?;
                    params.name = DOMString::from(alg_name);
                    NormalizedAlgorithm::Algorithm(params.into())
                },

                _ => return Err(Error::NotSupported(None)),
            };

            // Step 11. Return normalizedAlgorithm.
            Ok(normalized_algorithm)
        },
    }
}

impl NormalizedAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    fn name(&self) -> &str {
        match self {
            NormalizedAlgorithm::Algorithm(algo) => &algo.name,
            NormalizedAlgorithm::RsaHashedImportParams(algo) => &algo.name,
            NormalizedAlgorithm::EcdsaParams(algo) => &algo.name,
            NormalizedAlgorithm::EcKeyGenParams(algo) => &algo.name,
            NormalizedAlgorithm::EcKeyImportParams(algo) => &algo.name,
            NormalizedAlgorithm::EcdhKeyDeriveParams(algo) => &algo.name,
            NormalizedAlgorithm::AesCtrParams(algo) => &algo.name,
            NormalizedAlgorithm::AesKeyGenParams(algo) => &algo.name,
            NormalizedAlgorithm::AesDerivedKeyParams(algo) => &algo.name,
            NormalizedAlgorithm::AesCbcParams(algo) => &algo.name,
            NormalizedAlgorithm::AesGcmParams(algo) => &algo.name,
            NormalizedAlgorithm::HmacImportParams(algo) => &algo.name,
            NormalizedAlgorithm::HmacKeyGenParams(algo) => &algo.name,
            NormalizedAlgorithm::HkdfParams(algo) => &algo.name,
            NormalizedAlgorithm::Pbkdf2Params(algo) => &algo.name,
            NormalizedAlgorithm::AeadParams(algo) => &algo.name,
            NormalizedAlgorithm::CShakeParams(algo) => &algo.name,
            NormalizedAlgorithm::Argon2Params(algo) => &algo.name,
        }
    }

    fn encrypt(&self, key: &CryptoKey, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
        match (self.name(), self) {
            (ALG_AES_CTR, NormalizedAlgorithm::AesCtrParams(algo)) => {
                aes_operation::encrypt_aes_ctr(algo, key, plaintext)
            },
            (ALG_AES_CBC, NormalizedAlgorithm::AesCbcParams(algo)) => {
                aes_operation::encrypt_aes_cbc(algo, key, plaintext)
            },
            (ALG_AES_GCM, NormalizedAlgorithm::AesGcmParams(algo)) => {
                aes_operation::encrypt_aes_gcm(algo, key, plaintext)
            },
            (ALG_CHACHA20_POLY1305, NormalizedAlgorithm::AeadParams(algo)) => {
                chacha20_poly1305_operation::encrypt(algo, key, plaintext)
            },
            _ => Err(Error::NotSupported(None)),
        }
    }

    fn decrypt(&self, key: &CryptoKey, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        match (self.name(), self) {
            (ALG_AES_CTR, NormalizedAlgorithm::AesCtrParams(algo)) => {
                aes_operation::decrypt_aes_ctr(algo, key, ciphertext)
            },
            (ALG_AES_CBC, NormalizedAlgorithm::AesCbcParams(algo)) => {
                aes_operation::decrypt_aes_cbc(algo, key, ciphertext)
            },
            (ALG_AES_GCM, NormalizedAlgorithm::AesGcmParams(algo)) => {
                aes_operation::decrypt_aes_gcm(algo, key, ciphertext)
            },
            (ALG_CHACHA20_POLY1305, NormalizedAlgorithm::AeadParams(algo)) => {
                chacha20_poly1305_operation::decrypt(algo, key, ciphertext)
            },
            _ => Err(Error::NotSupported(None)),
        }
    }

    fn sign(&self, key: &CryptoKey, message: &[u8]) -> Result<Vec<u8>, Error> {
        match (self.name(), self) {
            (ALG_ECDSA, NormalizedAlgorithm::EcdsaParams(algo)) => {
                ecdsa_operation::sign(algo, key, message)
            },
            (ALG_ED25519, NormalizedAlgorithm::Algorithm(_algo)) => {
                ed25519_operation::sign(key, message)
            },
            (ALG_HMAC, NormalizedAlgorithm::Algorithm(_algo)) => hmac_operation::sign(key, message),
            _ => Err(Error::NotSupported(None)),
        }
    }

    fn verify(&self, key: &CryptoKey, message: &[u8], signature: &[u8]) -> Result<bool, Error> {
        match (self.name(), self) {
            (ALG_ECDSA, NormalizedAlgorithm::EcdsaParams(algo)) => {
                ecdsa_operation::verify(algo, key, message, signature)
            },
            (ALG_ED25519, NormalizedAlgorithm::Algorithm(_algo)) => {
                ed25519_operation::verify(key, message, signature)
            },
            (ALG_HMAC, NormalizedAlgorithm::Algorithm(_algo)) => {
                hmac_operation::verify(key, message, signature)
            },
            _ => Err(Error::NotSupported(None)),
        }
    }

    fn digest(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
        match (self.name(), self) {
            (ALG_SHA1, NormalizedAlgorithm::Algorithm(algo)) => {
                sha_operation::digest(algo, message)
            },
            (ALG_SHA256, NormalizedAlgorithm::Algorithm(algo)) => {
                sha_operation::digest(algo, message)
            },
            (ALG_SHA384, NormalizedAlgorithm::Algorithm(algo)) => {
                sha_operation::digest(algo, message)
            },
            (ALG_SHA512, NormalizedAlgorithm::Algorithm(algo)) => {
                sha_operation::digest(algo, message)
            },
            (ALG_SHA3_256, NormalizedAlgorithm::Algorithm(algo)) => {
                sha3_operation::digest(algo, message)
            },
            (ALG_SHA3_384, NormalizedAlgorithm::Algorithm(algo)) => {
                sha3_operation::digest(algo, message)
            },
            (ALG_SHA3_512, NormalizedAlgorithm::Algorithm(algo)) => {
                sha3_operation::digest(algo, message)
            },
            (ALG_CSHAKE_128, NormalizedAlgorithm::CShakeParams(algo)) => {
                cshake_operation::digest(algo, message)
            },
            (ALG_CSHAKE_256, NormalizedAlgorithm::CShakeParams(algo)) => {
                cshake_operation::digest(algo, message)
            },
            _ => Err(Error::NotSupported(None)),
        }
    }

    fn generate_key(
        &self,
        global: &GlobalScope,
        extractable: bool,
        usages: Vec<KeyUsage>,
        can_gc: CanGc,
    ) -> Result<CryptoKeyOrCryptoKeyPair, Error> {
        match (self.name(), self) {
            (ALG_ECDSA, NormalizedAlgorithm::EcKeyGenParams(algo)) => {
                ecdsa_operation::generate_key(global, algo, extractable, usages, can_gc)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            (ALG_ECDH, NormalizedAlgorithm::EcKeyGenParams(algo)) => {
                ecdh_operation::generate_key(global, algo, extractable, usages, can_gc)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            (ALG_ED25519, NormalizedAlgorithm::Algorithm(_algo)) => {
                ed25519_operation::generate_key(global, extractable, usages, can_gc)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            (ALG_X25519, NormalizedAlgorithm::Algorithm(_algo)) => {
                x25519_operation::generate_key(global, extractable, usages, can_gc)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            (ALG_AES_CTR, NormalizedAlgorithm::AesKeyGenParams(algo)) => {
                aes_operation::generate_key_aes_ctr(global, algo, extractable, usages, can_gc)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            (ALG_AES_CBC, NormalizedAlgorithm::AesKeyGenParams(algo)) => {
                aes_operation::generate_key_aes_cbc(global, algo, extractable, usages, can_gc)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            (ALG_AES_GCM, NormalizedAlgorithm::AesKeyGenParams(algo)) => {
                aes_operation::generate_key_aes_gcm(global, algo, extractable, usages, can_gc)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            (ALG_AES_KW, NormalizedAlgorithm::AesKeyGenParams(algo)) => {
                aes_operation::generate_key_aes_kw(global, algo, extractable, usages, can_gc)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            (ALG_HMAC, NormalizedAlgorithm::HmacKeyGenParams(algo)) => {
                hmac_operation::generate_key(global, algo, extractable, usages, can_gc)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            (ALG_CHACHA20_POLY1305, NormalizedAlgorithm::Algorithm(_algo)) => {
                chacha20_poly1305_operation::generate_key(global, extractable, usages, can_gc)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            _ => Err(Error::NotSupported(None)),
        }
    }

    fn derive_bits(&self, key: &CryptoKey, length: Option<u32>) -> Result<Vec<u8>, Error> {
        match (self.name(), self) {
            (ALG_ECDH, NormalizedAlgorithm::EcdhKeyDeriveParams(algo)) => {
                ecdh_operation::derive_bits(algo, key, length)
            },
            (ALG_X25519, NormalizedAlgorithm::EcdhKeyDeriveParams(algo)) => {
                x25519_operation::derive_bits(algo, key, length)
            },
            (ALG_HKDF, NormalizedAlgorithm::HkdfParams(algo)) => {
                hkdf_operation::derive_bits(algo, key, length)
            },
            (ALG_PBKDF2, NormalizedAlgorithm::Pbkdf2Params(algo)) => {
                pbkdf2_operation::derive_bits(algo, key, length)
            },
            (ALG_ARGON2D, NormalizedAlgorithm::Argon2Params(algo)) => {
                argon2_operation::derive_bits(algo, key, length)
            },
            (ALG_ARGON2I, NormalizedAlgorithm::Argon2Params(algo)) => {
                argon2_operation::derive_bits(algo, key, length)
            },
            (ALG_ARGON2ID, NormalizedAlgorithm::Argon2Params(algo)) => {
                argon2_operation::derive_bits(algo, key, length)
            },
            _ => Err(Error::NotSupported(None)),
        }
    }

    fn import_key(
        &self,
        global: &GlobalScope,
        format: KeyFormat,
        key_data: &[u8],
        extractable: bool,
        usages: Vec<KeyUsage>,
        can_gc: CanGc,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        match (self.name(), self) {
            (ALG_RSASSA_PKCS1_V1_5, NormalizedAlgorithm::RsaHashedImportParams(algo)) => {
                rsassa_pkcs1_v1_5_operation::import_key(
                    global,
                    algo,
                    format,
                    key_data,
                    extractable,
                    usages,
                    can_gc,
                )
            },
            (ALG_RSA_PSS, NormalizedAlgorithm::RsaHashedImportParams(algo)) => {
                rsa_pss_operation::import_key(
                    global,
                    algo,
                    format,
                    key_data,
                    extractable,
                    usages,
                    can_gc,
                )
            },
            (ALG_RSA_OAEP, NormalizedAlgorithm::RsaHashedImportParams(algo)) => {
                rsa_oaep_operation::import_key(
                    global,
                    algo,
                    format,
                    key_data,
                    extractable,
                    usages,
                    can_gc,
                )
            },
            (ALG_ECDSA, NormalizedAlgorithm::EcKeyImportParams(algo)) => {
                ecdsa_operation::import_key(
                    global,
                    algo,
                    format,
                    key_data,
                    extractable,
                    usages,
                    can_gc,
                )
            },
            (ALG_ECDH, NormalizedAlgorithm::EcKeyImportParams(algo)) => ecdh_operation::import_key(
                global,
                algo,
                format,
                key_data,
                extractable,
                usages,
                can_gc,
            ),
            (ALG_ED25519, NormalizedAlgorithm::Algorithm(_algo)) => {
                ed25519_operation::import_key(global, format, key_data, extractable, usages, can_gc)
            },
            (ALG_X25519, NormalizedAlgorithm::Algorithm(_algo)) => {
                x25519_operation::import_key(global, format, key_data, extractable, usages, can_gc)
            },
            (ALG_AES_CTR, NormalizedAlgorithm::Algorithm(_algo)) => {
                aes_operation::import_key_aes_ctr(
                    global,
                    format,
                    key_data,
                    extractable,
                    usages,
                    can_gc,
                )
            },
            (ALG_AES_CBC, NormalizedAlgorithm::Algorithm(_algo)) => {
                aes_operation::import_key_aes_cbc(
                    global,
                    format,
                    key_data,
                    extractable,
                    usages,
                    can_gc,
                )
            },
            (ALG_AES_GCM, NormalizedAlgorithm::Algorithm(_algo)) => {
                aes_operation::import_key_aes_gcm(
                    global,
                    format,
                    key_data,
                    extractable,
                    usages,
                    can_gc,
                )
            },
            (ALG_AES_KW, NormalizedAlgorithm::Algorithm(_algo)) => {
                aes_operation::import_key_aes_kw(
                    global,
                    format,
                    key_data,
                    extractable,
                    usages,
                    can_gc,
                )
            },
            (ALG_HMAC, NormalizedAlgorithm::HmacImportParams(algo)) => hmac_operation::import_key(
                global,
                algo,
                format,
                key_data,
                extractable,
                usages,
                can_gc,
            ),
            (ALG_HKDF, NormalizedAlgorithm::Algorithm(_algo)) => {
                hkdf_operation::import_key(global, format, key_data, extractable, usages, can_gc)
            },
            (ALG_PBKDF2, NormalizedAlgorithm::Algorithm(_algo)) => {
                pbkdf2_operation::import_key(global, format, key_data, extractable, usages, can_gc)
            },
            (ALG_CHACHA20_POLY1305, NormalizedAlgorithm::Algorithm(_algo)) => {
                chacha20_poly1305_operation::import_key(
                    global,
                    format,
                    key_data,
                    extractable,
                    usages,
                    can_gc,
                )
            },
            (ALG_ARGON2D, NormalizedAlgorithm::Algorithm(algo)) => argon2_operation::import_key(
                global,
                algo,
                format,
                key_data,
                extractable,
                usages,
                can_gc,
            ),
            (ALG_ARGON2I, NormalizedAlgorithm::Algorithm(algo)) => argon2_operation::import_key(
                global,
                algo,
                format,
                key_data,
                extractable,
                usages,
                can_gc,
            ),
            (ALG_ARGON2ID, NormalizedAlgorithm::Algorithm(algo)) => argon2_operation::import_key(
                global,
                algo,
                format,
                key_data,
                extractable,
                usages,
                can_gc,
            ),
            _ => Err(Error::NotSupported(None)),
        }
    }

    fn wrap_key(&self, key: &CryptoKey, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
        match (self.name(), self) {
            (ALG_AES_KW, NormalizedAlgorithm::Algorithm(_algo)) => {
                aes_operation::wrap_key_aes_kw(key, plaintext)
            },
            _ => Err(Error::NotSupported(None)),
        }
    }

    fn unwrap_key(&self, key: &CryptoKey, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        match (self.name(), self) {
            (ALG_AES_KW, NormalizedAlgorithm::Algorithm(_algo)) => {
                aes_operation::unwrap_key_aes_kw(key, ciphertext)
            },
            _ => Err(Error::NotSupported(None)),
        }
    }

    fn get_key_length(&self) -> Result<Option<u32>, Error> {
        match (self.name(), self) {
            (ALG_AES_CTR, NormalizedAlgorithm::AesDerivedKeyParams(algo)) => {
                aes_operation::get_key_length_aes_ctr(algo)
            },
            (ALG_AES_CBC, NormalizedAlgorithm::AesDerivedKeyParams(algo)) => {
                aes_operation::get_key_length_aes_cbc(algo)
            },
            (ALG_AES_GCM, NormalizedAlgorithm::AesDerivedKeyParams(algo)) => {
                aes_operation::get_key_length_aes_gcm(algo)
            },
            (ALG_AES_KW, NormalizedAlgorithm::AesDerivedKeyParams(algo)) => {
                aes_operation::get_key_length_aes_kw(algo)
            },
            (ALG_HMAC, NormalizedAlgorithm::HmacImportParams(algo)) => {
                hmac_operation::get_key_length(algo)
            },
            (ALG_HKDF, NormalizedAlgorithm::Algorithm(_algo)) => hkdf_operation::get_key_length(),
            (ALG_PBKDF2, NormalizedAlgorithm::Algorithm(_algo)) => {
                pbkdf2_operation::get_key_length()
            },
            (ALG_CHACHA20_POLY1305, NormalizedAlgorithm::Algorithm(_algo)) => {
                chacha20_poly1305_operation::get_key_length()
            },
            (ALG_ARGON2D, NormalizedAlgorithm::Algorithm(_algo)) => {
                argon2_operation::get_key_length()
            },
            (ALG_ARGON2I, NormalizedAlgorithm::Algorithm(_algo)) => {
                argon2_operation::get_key_length()
            },
            (ALG_ARGON2ID, NormalizedAlgorithm::Algorithm(_algo)) => {
                argon2_operation::get_key_length()
            },
            _ => Err(Error::NotSupported(None)),
        }
    }
}

/// Return the result of performing the export key operation specified by the [[algorithm]]
/// internal slot of key using key and format.
///
/// According to the WebCrypto API spec, the export key operation does not rely on the algorithm
/// normalization, We create this helper function to minic the functions of NormalizedAlgorithm
/// for export key operation.
fn perform_export_key_operation(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    match key.algorithm().name() {
        ALG_ECDSA => ecdsa_operation::export_key(format, key),
        ALG_ECDH => ecdh_operation::export_key(format, key),
        ALG_ED25519 => ed25519_operation::export_key(format, key),
        ALG_X25519 => x25519_operation::export_key(format, key),
        ALG_AES_CTR => aes_operation::export_key_aes_ctr(format, key),
        ALG_AES_CBC => aes_operation::export_key_aes_cbc(format, key),
        ALG_AES_GCM => aes_operation::export_key_aes_gcm(format, key),
        ALG_AES_KW => aes_operation::export_key_aes_kw(format, key),
        ALG_HMAC => hmac_operation::export_key(format, key),
        ALG_CHACHA20_POLY1305 => chacha20_poly1305_operation::export_key(format, key),
        _ => Err(Error::NotSupported(None)),
    }
}
