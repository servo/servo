/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::num::NonZero;
use std::ptr;
use std::rc::Rc;

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, StreamCipher};
use aes::{Aes128, Aes192, Aes256};
use aes_gcm::{AeadInPlace, AesGcm, KeyInit};
use aes_kw::{KekAes128, KekAes192, KekAes256};
use aws_lc_rs::{digest, hkdf, hmac, pbkdf2};
use base64::prelude::*;
use cipher::consts::{U12, U16, U32};
use dom_struct::dom_struct;
use js::conversions::ConversionResult;
use js::jsapi::{JSObject, JS_NewObject};
use js::jsval::ObjectValue;
use js::rust::MutableHandleObject;
use js::typedarray::ArrayBufferU8;
use servo_rand::{RngCore, ServoRng};

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{
    AesCbcParams, AesCtrParams, AesDerivedKeyParams, AesGcmParams, AesKeyAlgorithm,
    AesKeyGenParams, Algorithm, AlgorithmIdentifier, HkdfParams, HmacImportParams,
    HmacKeyAlgorithm, HmacKeyGenParams, JsonWebKey, KeyAlgorithm, KeyFormat, Pbkdf2Params,
    SubtleCryptoMethods,
};
use crate::dom::bindings::codegen::UnionTypes::{
    ArrayBufferViewOrArrayBuffer, ArrayBufferViewOrArrayBufferOrJsonWebKey,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::import::module::SafeJSContext;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::realms::InRealm;
use crate::script_runtime::{CanGc, JSContext};

// String constants for algorithms/curves
const ALG_AES_CBC: &str = "AES-CBC";
const ALG_AES_CTR: &str = "AES-CTR";
const ALG_AES_GCM: &str = "AES-GCM";
const ALG_AES_KW: &str = "AES-KW";
const ALG_SHA1: &str = "SHA-1";
const ALG_SHA256: &str = "SHA-256";
const ALG_SHA384: &str = "SHA-384";
const ALG_SHA512: &str = "SHA-512";
const ALG_HMAC: &str = "HMAC";
const ALG_HKDF: &str = "HKDF";
const ALG_PBKDF2: &str = "PBKDF2";
const ALG_RSASSA_PKCS1: &str = "RSASSA-PKCS1-v1_5";
const ALG_RSA_OAEP: &str = "RSA-OAEP";
const ALG_RSA_PSS: &str = "RSA-PSS";
const ALG_ECDH: &str = "ECDH";
const ALG_ECDSA: &str = "ECDSA";

#[allow(dead_code)]
static SUPPORTED_ALGORITHMS: &[&str] = &[
    ALG_AES_CBC,
    ALG_AES_CTR,
    ALG_AES_GCM,
    ALG_AES_KW,
    ALG_SHA1,
    ALG_SHA256,
    ALG_SHA384,
    ALG_SHA512,
    ALG_HMAC,
    ALG_HKDF,
    ALG_PBKDF2,
    ALG_RSASSA_PKCS1,
    ALG_RSA_OAEP,
    ALG_RSA_PSS,
    ALG_ECDH,
    ALG_ECDSA,
];

const NAMED_CURVE_P256: &str = "P-256";
const NAMED_CURVE_P384: &str = "P-384";
const NAMED_CURVE_P521: &str = "P-521";
#[allow(dead_code)]
static SUPPORTED_CURVES: &[&str] = &[NAMED_CURVE_P256, NAMED_CURVE_P384, NAMED_CURVE_P521];

type Aes128CbcEnc = cbc::Encryptor<Aes128>;
type Aes128CbcDec = cbc::Decryptor<Aes128>;
type Aes192CbcEnc = cbc::Encryptor<Aes192>;
type Aes192CbcDec = cbc::Decryptor<Aes192>;
type Aes256CbcEnc = cbc::Encryptor<Aes256>;
type Aes256CbcDec = cbc::Decryptor<Aes256>;
type Aes128Ctr = ctr::Ctr64BE<Aes128>;
type Aes192Ctr = ctr::Ctr64BE<Aes192>;
type Aes256Ctr = ctr::Ctr64BE<Aes256>;

type Aes128Gcm96Iv = AesGcm<Aes128, U12>;
type Aes128Gcm128Iv = AesGcm<Aes128, U16>;
type Aes192Gcm96Iv = AesGcm<Aes192, U12>;
type Aes256Gcm96Iv = AesGcm<Aes256, U12>;
type Aes128Gcm256Iv = AesGcm<Aes128, U32>;
type Aes192Gcm256Iv = AesGcm<Aes192, U32>;
type Aes256Gcm256Iv = AesGcm<Aes256, U32>;

#[dom_struct]
pub(crate) struct SubtleCrypto {
    reflector_: Reflector,
    #[no_trace]
    rng: DomRefCell<ServoRng>,
}

impl SubtleCrypto {
    fn new_inherited() -> SubtleCrypto {
        SubtleCrypto {
            reflector_: Reflector::new(),
            rng: DomRefCell::new(ServoRng::default()),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<SubtleCrypto> {
        reflect_dom_object(Box::new(SubtleCrypto::new_inherited()), global, can_gc)
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
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm = match normalize_algorithm_for_encrypt_or_decrypt(cx, &algorithm)
        {
            Ok(algorithm) => algorithm,
            Err(e) => {
                promise.reject_error(e);
                return promise;
            },
        };
        let data = match data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);
        let key_alg = key.algorithm();
        let valid_usage = key.usages().contains(&KeyUsage::Encrypt);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(encrypt: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();

                if !valid_usage || normalized_algorithm.name() != key_alg {
                    promise.reject_error(Error::InvalidAccess);
                    return;
                }

                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());

                if let Err(e) = normalized_algorithm.encrypt(
                    &subtle,
                    &key,
                    &data,
                    cx,
                    array_buffer_ptr.handle_mut(),
                    CanGc::note(),
                ) {
                    promise.reject_error(e);
                    return;
                }
                promise.resolve_native(&*array_buffer_ptr.handle(), CanGc::note());
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
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm = match normalize_algorithm_for_encrypt_or_decrypt(cx, &algorithm)
        {
            Ok(algorithm) => algorithm,
            Err(e) => {
                promise.reject_error(e);
                return promise;
            },
        };
        let data = match data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);
        let key_alg = key.algorithm();
        let valid_usage = key.usages().contains(&KeyUsage::Decrypt);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(decrypt: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());

                if !valid_usage || normalized_algorithm.name() != key_alg {
                    promise.reject_error(Error::InvalidAccess);
                    return;
                }

                if let Err(e) = normalized_algorithm.decrypt(
                    &subtle,
                    &key,
                    &data,
                    cx,
                    array_buffer_ptr.handle_mut(),
                    CanGc::note(),
                ) {
                    promise.reject_error(e);
                    return;
                }

                promise.resolve_native(&*array_buffer_ptr.handle(), CanGc::note());
            }));
        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-sign>
    fn Sign(
        &self,
        cx: SafeJSContext,
        algorithm: AlgorithmIdentifier,
        key: &CryptoKey,
        data: ArrayBufferViewOrArrayBuffer,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm and key be the algorithm and key parameters passed to the sign() method, respectively.

        // Step 2. Let data be the result of getting a copy of the bytes held by the data parameter passed to
        // the sign() method.
        let data = match &data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        // Step 3. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set to algorithm and
        // op set to "sign".
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm = match normalize_algorithm_for_sign_or_verify(cx, &algorithm) {
            Ok(algorithm) => algorithm,
            Err(e) => {
                // Step 4. If an error occurred, return a Promise rejected with normalizedAlgorithm.
                promise.reject_error(e);
                return promise;
            },
        };

        // Step 5. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 4.

        // Step 6. Return promise and perform the remaining steps in parallel.
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);

        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(sign: move || {
                // Step 7. If the following steps or referenced procedures say to throw an error, reject promise
                // with the returned error and then terminate the algorithm.
                let promise = trusted_promise.root();
                let key = trusted_key.root();

                // Step 8. If the name member of normalizedAlgorithm is not equal to the name attribute of the
                // [[algorithm]] internal slot of key then throw an InvalidAccessError.
                if normalized_algorithm.name() != key.algorithm() {
                    promise.reject_error(Error::InvalidAccess);
                    return;
                }

                // Step 9. If the [[usages]] internal slot of key does not contain an entry that is "sign",
                // then throw an InvalidAccessError.
                if !key.usages().contains(&KeyUsage::Sign) {
                    promise.reject_error(Error::InvalidAccess);
                    return;
                }

                // Step 10.  Let result be the result of performing the sign operation specified by normalizedAlgorithm
                // using key and algorithm and with data as message.
                let cx = GlobalScope::get_cx();
                let result = match normalized_algorithm.sign(cx, &key, &data) {
                    Ok(signature) => signature,
                    Err(e) => {
                        promise.reject_error(e);
                        return;
                    }
                };

                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());
                create_buffer_source::<ArrayBufferU8>(cx, &result, array_buffer_ptr.handle_mut(), CanGc::note())
                    .expect("failed to create buffer source for exported key.");

                // Step 9. Resolve promise with result.
                promise.resolve_native(&*array_buffer_ptr, CanGc::note());
            }));

        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-verify>
    fn Verify(
        &self,
        cx: SafeJSContext,
        algorithm: AlgorithmIdentifier,
        key: &CryptoKey,
        signature: ArrayBufferViewOrArrayBuffer,
        data: ArrayBufferViewOrArrayBuffer,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm and key be the algorithm and key parameters passed to the verify() method,
        // respectively.

        // Step 2. Let signature be the result of getting a copy of the bytes held by the signature parameter passed
        // to the verify() method.
        let signature = match &signature {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        // Step 3. Let data be the result of getting a copy of the bytes held by the data parameter passed to the
        // verify() method.
        let data = match &data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        // Step 4. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set to
        // algorithm and op set to "verify".
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm = match normalize_algorithm_for_sign_or_verify(cx, &algorithm) {
            Ok(algorithm) => algorithm,
            Err(e) => {
                // Step 5. If an error occurred, return a Promise rejected with normalizedAlgorithm.
                promise.reject_error(e);
                return promise;
            },
        };

        // Step 6. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 6.

        // Step 7. Return promise and perform the remaining steps in parallel.
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);

        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(sign: move || {
                // Step 8. If the following steps or referenced procedures say to throw an error, reject promise
                // with the returned error and then terminate the algorithm.
                let promise = trusted_promise.root();
                let key = trusted_key.root();

                // Step 9. If the name member of normalizedAlgorithm is not equal to the name attribute of the
                // [[algorithm]] internal slot of key then throw an InvalidAccessError.
                if normalized_algorithm.name() != key.algorithm() {
                    promise.reject_error(Error::InvalidAccess);
                    return;
                }

                // Step 10. If the [[usages]] internal slot of key does not contain an entry that is "verify",
                // then throw an InvalidAccessError.
                if !key.usages().contains(&KeyUsage::Verify) {
                    promise.reject_error(Error::InvalidAccess);
                    return;
                }

                // Step 1. Let result be the result of performing the verify operation specified by normalizedAlgorithm
                // using key, algorithm and signature and with data as message.
                let cx = GlobalScope::get_cx();
                let result = match normalized_algorithm.verify(cx, &key, &data, &signature) {
                    Ok(result) => result,
                    Err(e) => {
                        promise.reject_error(e);
                        return;
                    }
                };

                // Step 9. Resolve promise with result.
                promise.resolve_native(&result, CanGc::note());
            }));

        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-digest>
    fn Digest(
        &self,
        cx: SafeJSContext,
        algorithm: AlgorithmIdentifier,
        data: ArrayBufferViewOrArrayBuffer,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm be the algorithm parameter passed to the digest() method.

        // Step 2. Let data be the result of getting a copy of the bytes held by the
        // data parameter passed to the digest() method.
        let data = match data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        // Step 3. Let normalizedAlgorithm be the result of normalizing an algorithm,
        // with alg set to algorithm and op set to "digest".
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm = match normalize_algorithm_for_digest(cx, &algorithm) {
            Ok(normalized_algorithm) => normalized_algorithm,
            Err(e) => {
                // Step 4. If an error occurred, return a Promise rejected with normalizedAlgorithm.
                promise.reject_error(e);
                return promise;
            },
        };

        // Step 5. Let promise be a new Promise.
        // NOTE: We did that in preparation of Step 4.

        // Step 6. Return promise and perform the remaining steps in parallel.
        let trusted_promise = TrustedPromise::new(promise.clone());

        self.global().task_manager().dom_manipulation_task_source().queue(
            task!(generate_key: move || {
                // Step 7. If the following steps or referenced procedures say to throw an error, reject promise
                // with the returned error and then terminate the algorithm.
                let promise = trusted_promise.root();

                // Step 8. Let result be the result of performing the digest operation specified by
                // normalizedAlgorithm using algorithm, with data as message.
                let digest = match normalized_algorithm.digest(&data) {
                    Ok(digest) => digest,
                    Err(e) => {
                        promise.reject_error(e);
                        return;
                    }
                };

                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());
                create_buffer_source::<ArrayBufferU8>(cx, digest.as_ref(), array_buffer_ptr.handle_mut(), CanGc::note())
                    .expect("failed to create buffer source for exported key.");


                // Step 9. Resolve promise with result.
                promise.resolve_native(&*array_buffer_ptr, CanGc::note());
            })
        );

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
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm = match normalize_algorithm_for_generate_key(cx, &algorithm) {
            Ok(algorithm) => algorithm,
            Err(e) => {
                promise.reject_error(e);
                return promise;
            },
        };

        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(generate_key: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = normalized_algorithm.generate_key(&subtle, key_usages, extractable, CanGc::note());

                match key {
                    Ok(key) => promise.resolve_native(&key, CanGc::note()),
                    Err(e) => promise.reject_error(e),
                }
            }));

        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-deriveKey>
    fn DeriveKey(
        &self,
        cx: SafeJSContext,
        algorithm: AlgorithmIdentifier,
        base_key: &CryptoKey,
        derived_key_type: AlgorithmIdentifier,
        extractable: bool,
        key_usages: Vec<KeyUsage>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 1. Let algorithm, baseKey, derivedKeyType, extractable and usages be the algorithm, baseKey,
        // derivedKeyType, extractable and keyUsages parameters passed to the deriveKey() method, respectively.

        // Step 2. Let normalizedAlgorithm be the result of normalizing an algorithm, with alg set to algorithm
        // and op set to "deriveBits".
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm = match normalize_algorithm_for_derive_bits(cx, &algorithm) {
            Ok(algorithm) => algorithm,
            Err(e) => {
                // Step 3. If an error occurred, return a Promise rejected with normalizedAlgorithm.
                promise.reject_error(e);
                return promise;
            },
        };

        // Step 4. Let normalizedDerivedKeyAlgorithmImport be the result of normalizing an algorithm,
        // with alg set to derivedKeyType and op set to "importKey".
        let normalized_derived_key_algorithm_import =
            match normalize_algorithm_for_import_key(cx, &derived_key_type) {
                Ok(algorithm) => algorithm,
                Err(e) => {
                    // Step 5. If an error occurred, return a Promise rejected with normalizedDerivedKeyAlgorithmImport.
                    promise.reject_error(e);
                    return promise;
                },
            };

        // Step 6. Let normalizedDerivedKeyAlgorithmLength be the result of normalizing an algorithm, with alg set
        // to derivedKeyType and op set to "get key length".
        let normalized_derived_key_algorithm_length =
            match normalize_algorithm_for_get_key_length(cx, &derived_key_type) {
                Ok(algorithm) => algorithm,
                Err(e) => {
                    // Step 7. If an error occurred, return a Promise rejected with normalizedDerivedKeyAlgorithmLength.
                    promise.reject_error(e);
                    return promise;
                },
            };

        // Step 8. Let promise be a new Promise.
        // NOTE: We created the promise earlier, after Step 1.

        // Step 9. Return promise and perform the remaining steps in parallel.
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_base_key = Trusted::new(base_key);
        let this = Trusted::new(self);
        self.global().task_manager().dom_manipulation_task_source().queue(
            task!(derive_key: move || {
                // Step 10. If the following steps or referenced procedures say to throw an error, reject promise
                // with the returned error and then terminate the algorithm.

                // TODO Step 11. If the name member of normalizedAlgorithm is not equal to the name attribute of the #
                // [[algorithm]] internal slot of baseKey then throw an InvalidAccessError.
                let promise = trusted_promise.root();
                let base_key = trusted_base_key.root();
                let subtle = this.root();

                // Step 12. If the [[usages]] internal slot of baseKey does not contain an entry that is
                // "deriveKey", then throw an InvalidAccessError.
                if !base_key.usages().contains(&KeyUsage::DeriveKey) {
                    promise.reject_error(Error::InvalidAccess);
                    return;
                }

                // Step 13. Let length be the result of performing the get key length algorithm specified by
                // normalizedDerivedKeyAlgorithmLength using derivedKeyType.
                let length = match normalized_derived_key_algorithm_length.get_key_length() {
                    Ok(length) => length,
                    Err(e) => {
                        promise.reject_error(e);
                        return;
                    }
                };

                // Step 14. Let secret be the result of performing the derive bits operation specified by
                // normalizedAlgorithm using key, algorithm and length.
                let secret = match normalized_algorithm.derive_bits(&base_key, Some(length)){
                    Ok(secret) => secret,
                    Err(e) => {
                        promise.reject_error(e);
                        return;
                    }
                };

                // Step 15.  Let result be the result of performing the import key operation specified by
                // normalizedDerivedKeyAlgorithmImport using "raw" as format, secret as keyData, derivedKeyType as
                // algorithm and using extractable and usages.
                let result = normalized_derived_key_algorithm_import.import_key(
                    &subtle,
                    KeyFormat::Raw,
                    &secret,
                    extractable,
                    key_usages,
                    CanGc::note()
                );
                let result = match result  {
                    Ok(key) => key,
                    Err(e) => {
                        promise.reject_error(e);
                        return;
                    }
                };

                // Step 17. If the [[type]] internal slot of result is "secret" or "private" and usages
                // is empty, then throw a SyntaxError.
                if matches!(result.Type(), KeyType::Secret | KeyType::Private) && result.usages().is_empty() {
                    promise.reject_error(Error::Syntax);
                    return;
                }

                // Step 17. Resolve promise with result.
                promise.resolve_native(&*result, CanGc::note());
            }),
        );

        promise
    }

    /// <https://w3c.github.io/webcrypto/#dfn-SubtleCrypto-method-deriveBits>
    fn DeriveBits(
        &self,
        cx: SafeJSContext,
        algorithm: AlgorithmIdentifier,
        base_key: &CryptoKey,
        length: Option<u32>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 1.  Let algorithm, baseKey and length, be the algorithm, baseKey and
        // length parameters passed to the deriveBits() method, respectively.

        // Step 2. Let normalizedAlgorithm be the result of normalizing an algorithm,
        // with alg set to algorithm and op set to "deriveBits".
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm = match normalize_algorithm_for_derive_bits(cx, &algorithm) {
            Ok(algorithm) => algorithm,
            Err(e) => {
                // Step 3. If an error occurred, return a Promise rejected with normalizedAlgorithm.
                promise.reject_error(e);
                return promise;
            },
        };

        // Step 4. Let promise be a new Promise object.
        // NOTE: We did that in preparation of Step 3.

        // Step 5. Return promise and perform the remaining steps in parallel.
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_base_key = Trusted::new(base_key);

        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(import_key: move || {
                // Step 6. If the following steps or referenced procedures say to throw an error,
                // reject promise with the returned error and then terminate the algorithm.

                // TODO Step 7. If the name member of normalizedAlgorithm is not equal to the name attribute
                // of the [[algorithm]] internal slot of baseKey then throw an InvalidAccessError.
                let promise = trusted_promise.root();
                let base_key = trusted_base_key.root();

                // Step 8. If the [[usages]] internal slot of baseKey does not contain an entry that
                // is "deriveBits", then throw an InvalidAccessError.
                if !base_key.usages().contains(&KeyUsage::DeriveBits) {
                    promise.reject_error(Error::InvalidAccess);
                    return;
                }

                // Step 9. Let result be the result of creating an ArrayBuffer containing the result of performing the
                // derive bits operation specified by normalizedAlgorithm using baseKey, algorithm and length.
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());
                let result = match normalized_algorithm.derive_bits(&base_key, length) {
                    Ok(derived_bits) => derived_bits,
                    Err(e) => {
                        promise.reject_error(e);
                        return;
                    }
                };

                create_buffer_source::<ArrayBufferU8>(cx, &result, array_buffer_ptr.handle_mut(), CanGc::note())
                    .expect("failed to create buffer source for derived bits.");

                // Step 10. Resolve promise with result.
                promise.resolve_native(&*array_buffer_ptr, CanGc::note());
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
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm = match normalize_algorithm_for_import_key(cx, &algorithm) {
            Ok(algorithm) => algorithm,
            Err(e) => {
                promise.reject_error(e);
                return promise;
            },
        };

        let data = match key_data {
            ArrayBufferViewOrArrayBufferOrJsonWebKey::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBufferOrJsonWebKey::JsonWebKey(json_web_key) => {
                let data_string = match json_web_key.k {
                    Some(s) => s.to_string(),
                    None => {
                        promise.reject_error(Error::Syntax);
                        return promise;
                    },
                };

                match base64::engine::general_purpose::STANDARD_NO_PAD
                    .decode(data_string.as_bytes())
                {
                    Ok(data) => data,
                    Err(_) => {
                        promise.reject_error(Error::Syntax);
                        return promise;
                    },
                }
            },
            ArrayBufferViewOrArrayBufferOrJsonWebKey::ArrayBuffer(array_buffer) => {
                array_buffer.to_vec()
            },
        };

        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(import_key: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let imported_key = normalized_algorithm.import_key(&subtle,
                    format, &data, extractable, key_usages, CanGc::note());
                match imported_key {
                    Ok(k) => promise.resolve_native(&k, CanGc::note()),
                    Err(e) => promise.reject_error(e),
                };
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
        let promise = Promise::new_in_current_realm(comp, can_gc);

        let this = Trusted::new(self);
        let trusted_key = Trusted::new(key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global().task_manager().dom_manipulation_task_source().queue(
            task!(export_key: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();
                let alg_name = key.algorithm();
                if matches!(
                    alg_name.as_str(), ALG_SHA1 | ALG_SHA256 | ALG_SHA384 | ALG_SHA512 | ALG_HKDF | ALG_PBKDF2
                ) {
                    promise.reject_error(Error::NotSupported);
                    return;
                }
                if !key.Extractable() {
                    promise.reject_error(Error::InvalidAccess);
                    return;
                }
                let exported_key = match alg_name.as_str() {
                    ALG_AES_CBC | ALG_AES_CTR | ALG_AES_KW | ALG_AES_GCM => subtle.export_key_aes(format, &key),
                    _ => Err(Error::NotSupported),
                };
                match exported_key {
                    Ok(k) => {
                        match k {
                            AesExportedKey::Raw(k) => {
                                let cx = GlobalScope::get_cx();
                                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());
                                create_buffer_source::<ArrayBufferU8>(cx, &k, array_buffer_ptr.handle_mut(),
                                    CanGc::note())
                                    .expect("failed to create buffer source for exported key.");
                                promise.resolve_native(&array_buffer_ptr.get(), CanGc::note())
                            },
                            AesExportedKey::Jwk(k) => {
                                promise.resolve_native(&k, CanGc::note())
                            },
                        }
                    },
                    Err(e) => promise.reject_error(e),
                }
            }),
        );

        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-wrapKey>
    fn WrapKey(
        &self,
        cx: JSContext,
        format: KeyFormat,
        key: &CryptoKey,
        wrapping_key: &CryptoKey,
        wrap_algorithm: AlgorithmIdentifier,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let normalized_algorithm = match normalize_algorithm_for_key_wrap(cx, &wrap_algorithm) {
            Ok(algorithm) => algorithm,
            Err(e) => {
                promise.reject_error(e);
                return promise;
            },
        };

        let this = Trusted::new(self);
        let trusted_key = Trusted::new(key);
        let trusted_wrapping_key = Trusted::new(wrapping_key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global().task_manager().dom_manipulation_task_source().queue(
            task!(wrap_key: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();
                let wrapping_key = trusted_wrapping_key.root();
                let alg_name = key.algorithm();
                let wrapping_alg_name = wrapping_key.algorithm();
                let valid_wrap_usage = wrapping_key.usages().contains(&KeyUsage::WrapKey);
                let names_match = normalized_algorithm.name() == wrapping_alg_name.as_str();

                if !valid_wrap_usage || !names_match || !key.Extractable() {
                    promise.reject_error(Error::InvalidAccess);
                    return;
                }

                if matches!(
                    alg_name.as_str(), ALG_SHA1 | ALG_SHA256 | ALG_SHA384 | ALG_SHA512 | ALG_HKDF | ALG_PBKDF2
                ) {
                    promise.reject_error(Error::NotSupported);
                    return;
                }

                let exported_key = match subtle.export_key_aes(format, &key) {
                    Ok(k) => k,
                    Err(e) => {
                        promise.reject_error(e);
                        return;
                    },
                };

                let bytes = match exported_key {
                    AesExportedKey::Raw(k) => k,
                    AesExportedKey::Jwk(key) => {
                        // The spec states to convert this to an ECMAscript object and stringify it, but since we know
                        // that the output will be a string of JSON we can just construct it manually
                        // TODO: Support more than just a subset of the JWK dict, or find a way to
                        // stringify via SM internals
                        let Some(k) = key.k else {
                            promise.reject_error(Error::Syntax);
                            return;
                        };
                        let Some(alg) = key.alg else {
                            promise.reject_error(Error::Syntax);
                            return;
                        };
                        let Some(ext) = key.ext else {
                            promise.reject_error(Error::Syntax);
                            return;
                        };
                        let Some(key_ops) = key.key_ops else {
                            promise.reject_error(Error::Syntax);
                            return;
                        };
                        let key_ops_str = key_ops.iter().map(|op| op.to_string()).collect::<Vec<String>>();
                        format!("{{
                            \"kty\": \"oct\",
                            \"k\": \"{}\",
                            \"alg\": \"{}\",
                            \"ext\": {},
                            \"key_ops\": {:?}
                        }}", k, alg, ext, key_ops_str)
                        .into_bytes()
                    },
                };

                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());

                let result = match normalized_algorithm {
                    KeyWrapAlgorithm::AesKw => {
                        subtle.wrap_key_aes_kw(&wrapping_key, &bytes, cx, array_buffer_ptr.handle_mut(), CanGc::note())
                    },
                    KeyWrapAlgorithm::AesCbc(params) => {
                        subtle.encrypt_aes_cbc(&params, &wrapping_key, &bytes, cx, array_buffer_ptr.handle_mut(),
                            CanGc::note())
                    },
                    KeyWrapAlgorithm::AesCtr(params) => {
                        subtle.encrypt_decrypt_aes_ctr(
                            &params, &wrapping_key, &bytes, cx, array_buffer_ptr.handle_mut(), CanGc::note()
                        )
                    },
                    KeyWrapAlgorithm::AesGcm(params) => {
                        subtle.encrypt_aes_gcm(
                            &params, &wrapping_key, &bytes, cx, array_buffer_ptr.handle_mut(), CanGc::note()
                        )
                    },
                };

                match result {
                    Ok(_) => promise.resolve_native(&*array_buffer_ptr, CanGc::note()),
                    Err(e) => promise.reject_error(e),
                }
            }),
        );

        promise
    }

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-unwrapKey>
    fn UnwrapKey(
        &self,
        cx: JSContext,
        format: KeyFormat,
        wrapped_key: ArrayBufferViewOrArrayBuffer,
        unwrapping_key: &CryptoKey,
        unwrap_algorithm: AlgorithmIdentifier,
        unwrapped_key_algorithm: AlgorithmIdentifier,
        extractable: bool,
        key_usages: Vec<KeyUsage>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let wrapped_key_bytes = match wrapped_key {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        let normalized_algorithm = match normalize_algorithm_for_key_wrap(cx, &unwrap_algorithm) {
            Ok(algorithm) => algorithm,
            Err(e) => {
                promise.reject_error(e);
                return promise;
            },
        };
        let normalized_key_algorithm =
            match normalize_algorithm_for_import_key(cx, &unwrapped_key_algorithm) {
                Ok(algorithm) => algorithm,
                Err(e) => {
                    promise.reject_error(e);
                    return promise;
                },
            };

        let this = Trusted::new(self);
        let trusted_key = Trusted::new(unwrapping_key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        self.global().task_manager().dom_manipulation_task_source().queue(
            task!(unwrap_key: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let unwrapping_key = trusted_key.root();
                let alg_name = unwrapping_key.algorithm();
                let valid_usage = unwrapping_key.usages().contains(&KeyUsage::UnwrapKey);

                if !valid_usage || normalized_algorithm.name() != alg_name.as_str() {
                    promise.reject_error(Error::InvalidAccess);
                    return;
                }

                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());

                let result = match normalized_algorithm {
                    KeyWrapAlgorithm::AesKw => {
                        subtle.unwrap_key_aes_kw(&unwrapping_key, &wrapped_key_bytes, cx, array_buffer_ptr.handle_mut(),
                            CanGc::note())
                    },
                    KeyWrapAlgorithm::AesCbc(params) => {
                        subtle.decrypt_aes_cbc(
                            &params, &unwrapping_key, &wrapped_key_bytes, cx, array_buffer_ptr.handle_mut(),
                            CanGc::note()
                        )
                    },
                    KeyWrapAlgorithm::AesCtr(params) => {
                        subtle.encrypt_decrypt_aes_ctr(
                            &params, &unwrapping_key, &wrapped_key_bytes, cx, array_buffer_ptr.handle_mut(),
                            CanGc::note()
                        )
                    },
                    KeyWrapAlgorithm::AesGcm(params) => {
                        subtle.decrypt_aes_gcm(
                            &params, &unwrapping_key, &wrapped_key_bytes, cx, array_buffer_ptr.handle_mut(),
                            CanGc::note()
                        )
                    },
                };

                let bytes = match result {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        promise.reject_error(e);
                        return;
                    },
                };

                let import_key_bytes = match format {
                    KeyFormat::Raw | KeyFormat::Spki | KeyFormat::Pkcs8 => bytes,
                    KeyFormat::Jwk => {
                        match parse_jwk(&bytes, normalized_key_algorithm.clone(), extractable, &key_usages) {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                promise.reject_error(e);
                                return;
                            }
                        }
                    },
                };
                match normalized_key_algorithm.import_key(&subtle, format, &import_key_bytes,
                    extractable, key_usages, CanGc::note()) {
                    Ok(imported_key) => promise.resolve_native(&imported_key, CanGc::note()),
                    Err(e) => promise.reject_error(e),
                }
            }),
        );

        promise
    }
}

// These "subtle" structs are proxies for the codegen'd dicts which don't hold a DOMString
// so they can be sent safely when running steps in parallel.

#[derive(Clone, Debug)]
pub(crate) struct SubtleAlgorithm {
    #[allow(dead_code)]
    pub(crate) name: String,
}

impl From<DOMString> for SubtleAlgorithm {
    fn from(name: DOMString) -> Self {
        SubtleAlgorithm {
            name: name.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SubtleAesCbcParams {
    #[allow(dead_code)]
    pub(crate) name: String,
    pub(crate) iv: Vec<u8>,
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

#[derive(Clone, Debug)]
pub(crate) struct SubtleAesCtrParams {
    pub(crate) name: String,
    pub(crate) counter: Vec<u8>,
    pub(crate) length: u8,
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

#[derive(Clone, Debug)]
pub(crate) struct SubtleAesGcmParams {
    pub(crate) name: String,
    pub(crate) iv: Vec<u8>,
    pub(crate) additional_data: Option<Vec<u8>>,
    pub(crate) tag_length: Option<u8>,
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

#[derive(Clone, Debug)]
pub(crate) struct SubtleAesKeyGenParams {
    pub(crate) name: String,
    pub(crate) length: u16,
}

impl From<AesKeyGenParams> for SubtleAesKeyGenParams {
    fn from(params: AesKeyGenParams) -> Self {
        SubtleAesKeyGenParams {
            name: params.parent.name.to_string().to_uppercase(),
            length: params.length,
        }
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-HmacImportParams>
#[derive(Clone)]
struct SubtleHmacImportParams {
    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyAlgorithm-hash>
    hash: DigestAlgorithm,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyGenParams-length>
    length: Option<u32>,
}

impl SubtleHmacImportParams {
    fn new(cx: JSContext, params: RootedTraceableBox<HmacImportParams>) -> Fallible<Self> {
        let hash = normalize_algorithm_for_digest(cx, &params.hash)?;
        let params = Self {
            hash,
            length: params.length,
        };
        Ok(params)
    }

    /// <https://w3c.github.io/webcrypto/#hmac-operations>
    fn get_key_length(&self) -> Result<u32, Error> {
        // Step 1.
        let length = match self.length {
            // If the length member of normalizedDerivedKeyAlgorithm is not present:
            None => {
                // Let length be the block size in bits of the hash function identified by the hash member of
                // normalizedDerivedKeyAlgorithm.
                match self.hash {
                    DigestAlgorithm::Sha1 => 160,
                    DigestAlgorithm::Sha256 => 256,
                    DigestAlgorithm::Sha384 => 384,
                    DigestAlgorithm::Sha512 => 512,
                }
            },
            // Otherwise, if the length member of normalizedDerivedKeyAlgorithm is non-zero:
            Some(length) if length != 0 => {
                // Let length be equal to the length member of normalizedDerivedKeyAlgorithm.
                length
            },
            // Otherwise:
            _ => {
                // throw a TypeError.
                return Err(Error::Type("[[length]] must not be zero".to_string()));
            },
        };

        // Step 2. Return length.
        Ok(length)
    }
}

struct SubtleHmacKeyGenParams {
    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyGenParams-hash>
    hash: DigestAlgorithm,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyGenParams-length>
    length: Option<u32>,
}

impl SubtleHmacKeyGenParams {
    fn new(cx: JSContext, params: RootedTraceableBox<HmacKeyGenParams>) -> Fallible<Self> {
        let hash = normalize_algorithm_for_digest(cx, &params.hash)?;
        let params = Self {
            hash,
            length: params.length,
        };
        Ok(params)
    }
}
/// <https://w3c.github.io/webcrypto/#hkdf-params>
#[derive(Clone, Debug)]
pub(crate) struct SubtleHkdfParams {
    /// <https://w3c.github.io/webcrypto/#dfn-HkdfParams-hash>
    hash: DigestAlgorithm,

    /// <https://w3c.github.io/webcrypto/#dfn-HkdfParams-salt>
    salt: Vec<u8>,

    /// <https://w3c.github.io/webcrypto/#dfn-HkdfParams-info>
    info: Vec<u8>,
}

impl SubtleHkdfParams {
    fn new(cx: JSContext, params: RootedTraceableBox<HkdfParams>) -> Fallible<Self> {
        let hash = normalize_algorithm_for_digest(cx, &params.hash)?;
        let salt = match &params.salt {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };
        let info = match &params.info {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        let params = Self { hash, salt, info };

        Ok(params)
    }
}

/// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params>
#[derive(Clone, Debug)]
pub(crate) struct SubtlePbkdf2Params {
    /// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params-salt>
    salt: Vec<u8>,

    /// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params-iterations>
    iterations: u32,

    /// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params-hash>
    hash: DigestAlgorithm,
}

impl SubtlePbkdf2Params {
    fn new(cx: JSContext, params: RootedTraceableBox<Pbkdf2Params>) -> Fallible<Self> {
        let salt = match &params.salt {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        let params = Self {
            salt,
            iterations: params.iterations,
            hash: normalize_algorithm_for_digest(cx, &params.hash)?,
        };

        Ok(params)
    }
}

enum GetKeyLengthAlgorithm {
    Aes(u16),
    Hmac(SubtleHmacImportParams),
}

#[derive(Clone, Copy, Debug)]
enum DigestAlgorithm {
    /// <https://w3c.github.io/webcrypto/#sha>
    Sha1,

    /// <https://w3c.github.io/webcrypto/#sha>
    Sha256,

    /// <https://w3c.github.io/webcrypto/#sha>
    Sha384,

    /// <https://w3c.github.io/webcrypto/#sha>
    Sha512,
}

/// A normalized algorithm returned by [`normalize_algorithm`] with operation `"importKey"`
///
/// [`normalize_algorithm`]: https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm
#[derive(Clone)]
enum ImportKeyAlgorithm {
    AesCbc,
    AesCtr,
    AesKw,
    AesGcm,
    Hmac(SubtleHmacImportParams),
    Pbkdf2,
    Hkdf,
}

/// A normalized algorithm returned by [`normalize_algorithm`] with operation `"deriveBits"`
///
/// [`normalize_algorithm`]: https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm
enum DeriveBitsAlgorithm {
    Pbkdf2(SubtlePbkdf2Params),
    Hkdf(SubtleHkdfParams),
}

/// A normalized algorithm returned by [`normalize_algorithm`] with operation `"encrypt"` or `"decrypt"`
///
/// [`normalize_algorithm`]: https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm
#[allow(clippy::enum_variant_names)]
enum EncryptionAlgorithm {
    AesCbc(SubtleAesCbcParams),
    AesCtr(SubtleAesCtrParams),
    AesGcm(SubtleAesGcmParams),
}

/// A normalized algorithm returned by [`normalize_algorithm`] with operation `"sign"` or `"verify"`
///
/// [`normalize_algorithm`]: https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm
enum SignatureAlgorithm {
    Hmac,
}

/// A normalized algorithm returned by [`normalize_algorithm`] with operation `"generateKey"`
///
/// [`normalize_algorithm`]: https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm
enum KeyGenerationAlgorithm {
    Aes(SubtleAesKeyGenParams),
    Hmac(SubtleHmacKeyGenParams),
}

/// A normalized algorithm returned by [`normalize_algorithm`] with operation `"wrapKey"` or `"unwrapKey"`
///
/// [`normalize_algorithm`]: https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm
#[allow(clippy::enum_variant_names)]
enum KeyWrapAlgorithm {
    AesKw,
    AesCbc(SubtleAesCbcParams),
    AesCtr(SubtleAesCtrParams),
    AesGcm(SubtleAesGcmParams),
}

macro_rules! value_from_js_object {
    ($t: ty, $cx: ident, $value: ident) => {{
        let params_result = <$t>::new($cx, $value.handle()).map_err(|_| Error::JSFailed)?;
        let ConversionResult::Success(params) = params_result else {
            return Err(Error::Syntax);
        };
        params
    }};
}

/// <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm> with operation `"get key length"`
fn normalize_algorithm_for_get_key_length(
    cx: JSContext,
    algorithm: &AlgorithmIdentifier,
) -> Result<GetKeyLengthAlgorithm, Error> {
    match algorithm {
        AlgorithmIdentifier::Object(obj) => {
            rooted!(in(*cx) let value = ObjectValue(obj.get()));
            let algorithm = value_from_js_object!(Algorithm, cx, value);

            let name = algorithm.name.str();
            let normalized_algorithm = if name.eq_ignore_ascii_case(ALG_AES_CBC) ||
                name.eq_ignore_ascii_case(ALG_AES_CTR) ||
                name.eq_ignore_ascii_case(ALG_AES_GCM)
            {
                let params = value_from_js_object!(AesDerivedKeyParams, cx, value);
                GetKeyLengthAlgorithm::Aes(params.length)
            } else if name.eq_ignore_ascii_case(ALG_HMAC) {
                let params = value_from_js_object!(HmacImportParams, cx, value);
                let subtle_params = SubtleHmacImportParams::new(cx, params)?;
                return Ok(GetKeyLengthAlgorithm::Hmac(subtle_params));
            } else {
                return Err(Error::NotSupported);
            };

            Ok(normalized_algorithm)
        },
        AlgorithmIdentifier::String(_) => {
            // All algorithms that support "get key length" require additional parameters
            Err(Error::NotSupported)
        },
    }
}

/// <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm> with operation `"digest"`
fn normalize_algorithm_for_digest(
    cx: JSContext,
    algorithm: &AlgorithmIdentifier,
) -> Result<DigestAlgorithm, Error> {
    let name = match algorithm {
        AlgorithmIdentifier::Object(obj) => {
            rooted!(in(*cx) let value = ObjectValue(obj.get()));
            let algorithm = value_from_js_object!(Algorithm, cx, value);

            algorithm.name.str().to_uppercase()
        },
        AlgorithmIdentifier::String(name) => name.str().to_uppercase(),
    };

    let normalized_algorithm = match name.as_str() {
        ALG_SHA1 => DigestAlgorithm::Sha1,
        ALG_SHA256 => DigestAlgorithm::Sha256,
        ALG_SHA384 => DigestAlgorithm::Sha384,
        ALG_SHA512 => DigestAlgorithm::Sha512,
        _ => return Err(Error::NotSupported),
    };

    Ok(normalized_algorithm)
}

/// <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm> with operation `"importKey"`
fn normalize_algorithm_for_import_key(
    cx: JSContext,
    algorithm: &AlgorithmIdentifier,
) -> Result<ImportKeyAlgorithm, Error> {
    let name = match algorithm {
        AlgorithmIdentifier::Object(obj) => {
            rooted!(in(*cx) let value = ObjectValue(obj.get()));
            let algorithm = value_from_js_object!(Algorithm, cx, value);

            let name = algorithm.name.str().to_uppercase();
            if name == ALG_HMAC {
                let params = value_from_js_object!(HmacImportParams, cx, value);
                let subtle_params = SubtleHmacImportParams::new(cx, params)?;
                return Ok(ImportKeyAlgorithm::Hmac(subtle_params));
            }

            name
        },
        AlgorithmIdentifier::String(name) => name.str().to_uppercase(),
    };

    let normalized_algorithm = match name.as_str() {
        ALG_AES_CBC => ImportKeyAlgorithm::AesCbc,
        ALG_AES_CTR => ImportKeyAlgorithm::AesCtr,
        ALG_AES_KW => ImportKeyAlgorithm::AesKw,
        ALG_AES_GCM => ImportKeyAlgorithm::AesGcm,
        ALG_PBKDF2 => ImportKeyAlgorithm::Pbkdf2,
        ALG_HKDF => ImportKeyAlgorithm::Hkdf,
        _ => return Err(Error::NotSupported),
    };

    Ok(normalized_algorithm)
}

/// <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm> with operation `"deriveBits"`
fn normalize_algorithm_for_derive_bits(
    cx: JSContext,
    algorithm: &AlgorithmIdentifier,
) -> Result<DeriveBitsAlgorithm, Error> {
    let AlgorithmIdentifier::Object(obj) = algorithm else {
        // All algorithms that support "deriveBits" require additional parameters
        return Err(Error::NotSupported);
    };

    rooted!(in(*cx) let value = ObjectValue(obj.get()));
    let algorithm = value_from_js_object!(Algorithm, cx, value);

    let normalized_algorithm = if algorithm.name.str().eq_ignore_ascii_case(ALG_PBKDF2) {
        let params = value_from_js_object!(Pbkdf2Params, cx, value);
        let subtle_params = SubtlePbkdf2Params::new(cx, params)?;
        DeriveBitsAlgorithm::Pbkdf2(subtle_params)
    } else if algorithm.name.str().eq_ignore_ascii_case(ALG_HKDF) {
        let params = value_from_js_object!(HkdfParams, cx, value);
        let subtle_params = SubtleHkdfParams::new(cx, params)?;
        DeriveBitsAlgorithm::Hkdf(subtle_params)
    } else {
        return Err(Error::NotSupported);
    };

    Ok(normalized_algorithm)
}

/// <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm> with operation `"deriveBits"`
fn normalize_algorithm_for_encrypt_or_decrypt(
    cx: JSContext,
    algorithm: &AlgorithmIdentifier,
) -> Result<EncryptionAlgorithm, Error> {
    let AlgorithmIdentifier::Object(obj) = algorithm else {
        // All algorithms that support "encrypt" or "decrypt" require additional parameters
        return Err(Error::NotSupported);
    };

    rooted!(in(*cx) let value = ObjectValue(obj.get()));
    let algorithm = value_from_js_object!(Algorithm, cx, value);

    let name = algorithm.name.str();
    let normalized_algorithm = if name.eq_ignore_ascii_case(ALG_AES_CBC) {
        let params = value_from_js_object!(AesCbcParams, cx, value);
        EncryptionAlgorithm::AesCbc(params.into())
    } else if name.eq_ignore_ascii_case(ALG_AES_CTR) {
        let params = value_from_js_object!(AesCtrParams, cx, value);
        EncryptionAlgorithm::AesCtr(params.into())
    } else if name.eq_ignore_ascii_case(ALG_AES_GCM) {
        let params = value_from_js_object!(AesGcmParams, cx, value);
        EncryptionAlgorithm::AesGcm(params.into())
    } else {
        return Err(Error::NotSupported);
    };

    Ok(normalized_algorithm)
}

/// <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm> with operation `"sign"`
/// or `"verify"`
fn normalize_algorithm_for_sign_or_verify(
    cx: JSContext,
    algorithm: &AlgorithmIdentifier,
) -> Result<SignatureAlgorithm, Error> {
    let name = match algorithm {
        AlgorithmIdentifier::Object(obj) => {
            rooted!(in(*cx) let value = ObjectValue(obj.get()));
            let algorithm = value_from_js_object!(Algorithm, cx, value);

            algorithm.name.str().to_uppercase()
        },
        AlgorithmIdentifier::String(name) => name.str().to_uppercase(),
    };

    let normalized_algorithm = match name.as_str() {
        ALG_HMAC => SignatureAlgorithm::Hmac,
        _ => return Err(Error::NotSupported),
    };

    Ok(normalized_algorithm)
}

/// <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm> with operation `"generateKey"`
fn normalize_algorithm_for_generate_key(
    cx: JSContext,
    algorithm: &AlgorithmIdentifier,
) -> Result<KeyGenerationAlgorithm, Error> {
    let AlgorithmIdentifier::Object(obj) = algorithm else {
        // All algorithms that support "generateKey" require additional parameters
        return Err(Error::NotSupported);
    };

    rooted!(in(*cx) let value = ObjectValue(obj.get()));
    let algorithm = value_from_js_object!(Algorithm, cx, value);

    let name = algorithm.name.str();
    let normalized_algorithm = if name.eq_ignore_ascii_case(ALG_AES_CBC) ||
        name.eq_ignore_ascii_case(ALG_AES_CTR) ||
        name.eq_ignore_ascii_case(ALG_AES_KW) ||
        name.eq_ignore_ascii_case(ALG_AES_GCM)
    {
        let params = value_from_js_object!(AesKeyGenParams, cx, value);
        KeyGenerationAlgorithm::Aes(params.into())
    } else if name.eq_ignore_ascii_case(ALG_HMAC) {
        let params = value_from_js_object!(HmacKeyGenParams, cx, value);
        let subtle_params = SubtleHmacKeyGenParams::new(cx, params)?;
        KeyGenerationAlgorithm::Hmac(subtle_params)
    } else {
        return Err(Error::NotSupported);
    };

    Ok(normalized_algorithm)
}

/// <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm> with operation `"wrapKey"` or `"unwrapKey"`
fn normalize_algorithm_for_key_wrap(
    cx: JSContext,
    algorithm: &AlgorithmIdentifier,
) -> Result<KeyWrapAlgorithm, Error> {
    let name = match algorithm {
        AlgorithmIdentifier::Object(obj) => {
            rooted!(in(*cx) let value = ObjectValue(obj.get()));
            let algorithm = value_from_js_object!(Algorithm, cx, value);

            algorithm.name.str().to_uppercase()
        },
        AlgorithmIdentifier::String(name) => name.str().to_uppercase(),
    };

    let normalized_algorithm = match name.as_str() {
        ALG_AES_KW => KeyWrapAlgorithm::AesKw,
        ALG_AES_CBC => {
            let AlgorithmIdentifier::Object(obj) = algorithm else {
                return Err(Error::Syntax);
            };
            rooted!(in(*cx) let value = ObjectValue(obj.get()));
            KeyWrapAlgorithm::AesCbc(value_from_js_object!(AesCbcParams, cx, value).into())
        },
        ALG_AES_CTR => {
            let AlgorithmIdentifier::Object(obj) = algorithm else {
                return Err(Error::Syntax);
            };
            rooted!(in(*cx) let value = ObjectValue(obj.get()));
            KeyWrapAlgorithm::AesCtr(value_from_js_object!(AesCtrParams, cx, value).into())
        },
        ALG_AES_GCM => {
            let AlgorithmIdentifier::Object(obj) = algorithm else {
                return Err(Error::Syntax);
            };
            rooted!(in(*cx) let value = ObjectValue(obj.get()));
            KeyWrapAlgorithm::AesGcm(value_from_js_object!(AesGcmParams, cx, value).into())
        },
        _ => return Err(Error::NotSupported),
    };

    Ok(normalized_algorithm)
}

impl SubtleCrypto {
    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    fn encrypt_aes_cbc(
        &self,
        params: &SubtleAesCbcParams,
        key: &CryptoKey,
        data: &[u8],
        cx: JSContext,
        handle: MutableHandleObject,
        can_gc: CanGc,
    ) -> Result<Vec<u8>, Error> {
        if params.iv.len() != 16 {
            return Err(Error::Operation);
        }

        let plaintext = Vec::from(data);
        let iv = GenericArray::from_slice(&params.iv);

        let ct = match key.handle() {
            Handle::Aes128(data) => {
                let key_data = GenericArray::from_slice(data);
                Aes128CbcEnc::new(key_data, iv).encrypt_padded_vec_mut::<Pkcs7>(&plaintext)
            },
            Handle::Aes192(data) => {
                let key_data = GenericArray::from_slice(data);
                Aes192CbcEnc::new(key_data, iv).encrypt_padded_vec_mut::<Pkcs7>(&plaintext)
            },
            Handle::Aes256(data) => {
                let key_data = GenericArray::from_slice(data);
                Aes256CbcEnc::new(key_data, iv).encrypt_padded_vec_mut::<Pkcs7>(&plaintext)
            },
            _ => return Err(Error::Data),
        };

        create_buffer_source::<ArrayBufferU8>(cx, &ct, handle, can_gc)
            .expect("failed to create buffer source for exported key.");

        Ok(ct)
    }

    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    fn decrypt_aes_cbc(
        &self,
        params: &SubtleAesCbcParams,
        key: &CryptoKey,
        data: &[u8],
        cx: JSContext,
        handle: MutableHandleObject,
        can_gc: CanGc,
    ) -> Result<Vec<u8>, Error> {
        if params.iv.len() != 16 {
            return Err(Error::Operation);
        }

        let mut ciphertext = Vec::from(data);
        let iv = GenericArray::from_slice(&params.iv);

        let plaintext = match key.handle() {
            Handle::Aes128(data) => {
                let key_data = GenericArray::from_slice(data);
                Aes128CbcDec::new(key_data, iv)
                    .decrypt_padded_mut::<Pkcs7>(ciphertext.as_mut_slice())
                    .map_err(|_| Error::Operation)?
            },
            Handle::Aes192(data) => {
                let key_data = GenericArray::from_slice(data);
                Aes192CbcDec::new(key_data, iv)
                    .decrypt_padded_mut::<Pkcs7>(ciphertext.as_mut_slice())
                    .map_err(|_| Error::Operation)?
            },
            Handle::Aes256(data) => {
                let key_data = GenericArray::from_slice(data);
                Aes256CbcDec::new(key_data, iv)
                    .decrypt_padded_mut::<Pkcs7>(ciphertext.as_mut_slice())
                    .map_err(|_| Error::Operation)?
            },
            _ => return Err(Error::Data),
        };

        create_buffer_source::<ArrayBufferU8>(cx, plaintext, handle, can_gc)
            .expect("failed to create buffer source for exported key.");

        Ok(plaintext.to_vec())
    }

    /// <https://w3c.github.io/webcrypto/#aes-ctr-operations>
    fn encrypt_decrypt_aes_ctr(
        &self,
        params: &SubtleAesCtrParams,
        key: &CryptoKey,
        data: &[u8],
        cx: JSContext,
        handle: MutableHandleObject,
        can_gc: CanGc,
    ) -> Result<Vec<u8>, Error> {
        if params.counter.len() != 16 || params.length == 0 || params.length > 128 {
            return Err(Error::Operation);
        }

        let mut ciphertext = Vec::from(data);
        let counter = GenericArray::from_slice(&params.counter);

        match key.handle() {
            Handle::Aes128(data) => {
                let key_data = GenericArray::from_slice(data);
                Aes128Ctr::new(key_data, counter).apply_keystream(&mut ciphertext)
            },
            Handle::Aes192(data) => {
                let key_data = GenericArray::from_slice(data);
                Aes192Ctr::new(key_data, counter).apply_keystream(&mut ciphertext)
            },
            Handle::Aes256(data) => {
                let key_data = GenericArray::from_slice(data);
                Aes256Ctr::new(key_data, counter).apply_keystream(&mut ciphertext)
            },
            _ => return Err(Error::Data),
        };

        create_buffer_source::<ArrayBufferU8>(cx, &ciphertext, handle, can_gc)
            .expect("failed to create buffer source for exported key.");

        Ok(ciphertext)
    }

    /// <https://w3c.github.io/webcrypto/#aes-gcm-operations>
    fn encrypt_aes_gcm(
        &self,
        params: &SubtleAesGcmParams,
        key: &CryptoKey,
        plaintext: &[u8],
        cx: JSContext,
        handle: MutableHandleObject,
        can_gc: CanGc,
    ) -> Result<Vec<u8>, Error> {
        // Step 1. If plaintext has a length greater than 2^39 - 256 bytes, then throw an OperationError.
        if plaintext.len() as u64 > (2 << 39) - 256 {
            return Err(Error::Operation);
        }

        // Step 2. If the iv member of normalizedAlgorithm has a length greater than 2^64 - 1 bytes,
        // then throw an OperationError.
        // NOTE: servo does not currently support 128-bit platforms, so this can never happen

        // Step 3. If the additionalData member of normalizedAlgorithm is present and has a length greater than 2^64 - 1
        // bytes, then throw an OperationError.
        if params
            .additional_data
            .as_ref()
            .is_some_and(|data| data.len() > u64::MAX as usize)
        {
            return Err(Error::Operation);
        }

        // Step 4.
        let tag_length = match params.tag_length {
            // If the tagLength member of normalizedAlgorithm is not present:
            None => {
                // Let tagLength be 128.
                128
            },
            // If the tagLength member of normalizedAlgorithm is one of 32, 64, 96, 104, 112, 120 or 128:
            Some(length) if matches!(length, 32 | 64 | 96 | 104 | 112 | 120 | 128) => {
                // Let tagLength be equal to the tagLength member of normalizedAlgorithm
                length
            },
            // Otherwise:
            _ => {
                // throw an OperationError.
                return Err(Error::Operation);
            },
        };

        // Step 5. Let additionalData be the contents of the additionalData member of normalizedAlgorithm if present
        // or the empty octet string otherwise.
        let additional_data = params.additional_data.as_deref().unwrap_or_default();

        // Step 6. Let C and T be the outputs that result from performing the Authenticated Encryption Function
        // described in Section 7.1 of [NIST-SP800-38D] using AES as the block cipher, the contents of the iv member
        // of normalizedAlgorithm as the IV input parameter, the contents of additionalData as the A input parameter,
        // tagLength as the t pre-requisite and the contents of plaintext as the input plaintext.
        let key_length = key.handle().as_bytes().len();
        let iv_length = params.iv.len();
        let mut ciphertext = plaintext.to_vec();
        let key_bytes = key.handle().as_bytes();
        let tag = match (key_length, iv_length) {
            (16, 12) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes128Gcm96Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
            },
            (16, 16) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes128Gcm128Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
            },
            (24, 12) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes192Gcm96Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
            },
            (32, 12) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes256Gcm96Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
            },
            (16, 32) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes128Gcm256Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
            },
            (24, 32) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes192Gcm256Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
            },
            (32, 32) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes256Gcm256Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
            },
            _ => {
                log::warn!(
                    "Missing AES-GCM encryption implementation with {key_length}-byte key and {iv_length}-byte IV"
                );
                return Err(Error::NotSupported);
            },
        };

        // Step 7. Let ciphertext be equal to C | T, where '|' denotes concatenation.
        ciphertext.extend_from_slice(&tag.unwrap()[..tag_length as usize / 8]);

        // Step 8. Return the result of creating an ArrayBuffer containing ciphertext.
        create_buffer_source::<ArrayBufferU8>(cx, &ciphertext, handle, can_gc)
            .expect("failed to create buffer source for encrypted ciphertext");

        Ok(ciphertext)
    }

    /// <https://w3c.github.io/webcrypto/#aes-gcm-operations>
    fn decrypt_aes_gcm(
        &self,
        params: &SubtleAesGcmParams,
        key: &CryptoKey,
        ciphertext: &[u8],
        cx: JSContext,
        handle: MutableHandleObject,
        can_gc: CanGc,
    ) -> Result<Vec<u8>, Error> {
        // Step 1.
        // FIXME: aes_gcm uses a fixed tag length
        let tag_length = match params.tag_length {
            // If the tagLength member of normalizedAlgorithm is not present:
            None => {
                // Let tagLength be 128.
                128
            },
            // If the tagLength member of normalizedAlgorithm is one of 32, 64, 96, 104, 112, 120 or 128:
            Some(length) if matches!(length, 32 | 64 | 96 | 104 | 112 | 120 | 128) => {
                // Let tagLength be equal to the tagLength member of normalizedAlgorithm
                length as usize
            },
            // Otherwise:
            _ => {
                // throw an OperationError.
                return Err(Error::Operation);
            },
        };

        // Step 2. If ciphertext has a length less than tagLength bits, then throw an OperationError.
        if ciphertext.len() < tag_length / 8 {
            return Err(Error::Operation);
        }

        // Step 3. If the iv member of normalizedAlgorithm has a length greater than 2^64 - 1 bytes,
        // then throw an OperationError.
        // NOTE: servo does not currently support 128-bit platforms, so this can never happen

        // Step 4. If the additionalData member of normalizedAlgorithm is present and has a length greater than 2^64 - 1
        // bytes, then throw an OperationError.
        // NOTE: servo does not currently support 128-bit platforms, so this can never happen

        // Step 5. Let tag be the last tagLength bits of ciphertext.
        // Step 6. Let actualCiphertext be the result of removing the last tagLength bits from ciphertext.
        // NOTE: aes_gcm splits the ciphertext for us

        // Step 7. Let additionalData be the contents of the additionalData member of normalizedAlgorithm if present or
        // the empty octet string otherwise.
        let additional_data = params.additional_data.as_deref().unwrap_or_default();

        // Step 8.  Perform the Authenticated Decryption Function described in Section 7.2 of [NIST-SP800-38D] using AES
        // as the block cipher, the contents of the iv member of normalizedAlgorithm as the IV input parameter, the
        // contents of additionalData as the A input parameter, tagLength as the t pre-requisite, the contents of
        // actualCiphertext as the input ciphertext, C and the contents of tag as the authentication tag, T.
        let mut plaintext = ciphertext.to_vec();
        let key_length = key.handle().as_bytes().len();
        let iv_length = params.iv.len();
        let key_bytes = key.handle().as_bytes();
        let result = match (key_length, iv_length) {
            (16, 12) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes128Gcm96Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .decrypt_in_place(nonce, additional_data, &mut plaintext)
            },
            (16, 16) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes128Gcm128Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .decrypt_in_place(nonce, additional_data, &mut plaintext)
            },
            (24, 12) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes192Gcm96Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .decrypt_in_place(nonce, additional_data, &mut plaintext)
            },
            (32, 12) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes256Gcm96Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .decrypt_in_place(nonce, additional_data, &mut plaintext)
            },
            (16, 32) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes128Gcm256Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .decrypt_in_place(nonce, additional_data, &mut plaintext)
            },
            (24, 32) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes192Gcm256Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .decrypt_in_place(nonce, additional_data, &mut plaintext)
            },
            (32, 32) => {
                let nonce = GenericArray::from_slice(&params.iv);
                <Aes256Gcm256Iv>::new_from_slice(key_bytes)
                    .expect("key length did not match")
                    .decrypt_in_place(nonce, additional_data, &mut plaintext)
            },
            _ => {
                log::warn!(
                    "Missing AES-GCM decryption implementation with {key_length}-byte key and {iv_length}-byte IV"
                );
                return Err(Error::NotSupported);
            },
        };

        // If the result of the algorithm is the indication of inauthenticity, "FAIL":
        if result.is_err() {
            // throw an OperationError
            return Err(Error::Operation);
        }
        // Otherwise:
        // Let plaintext be the output P of the Authenticated Decryption Function.

        // Step 9. Return the result of creating an ArrayBuffer containing plaintext.
        create_buffer_source::<ArrayBufferU8>(cx, &plaintext, handle, can_gc)
            .expect("failed to create buffer source for decrypted plaintext");

        Ok(plaintext)
    }

    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    /// <https://w3c.github.io/webcrypto/#aes-ctr-operations>
    /// <https://w3c.github.io/webcrypto/#aes-kw-operations>
    #[allow(unsafe_code)]
    fn generate_key_aes(
        &self,
        usages: Vec<KeyUsage>,
        key_gen_params: &SubtleAesKeyGenParams,
        extractable: bool,
        can_gc: CanGc,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        let mut rand = vec![0; key_gen_params.length as usize / 8];
        self.rng.borrow_mut().fill_bytes(&mut rand);
        let handle = match key_gen_params.length {
            128 => Handle::Aes128(rand),
            192 => Handle::Aes192(rand),
            256 => Handle::Aes256(rand),
            _ => return Err(Error::Operation),
        };

        match key_gen_params.name.as_str() {
            ALG_AES_CBC | ALG_AES_CTR | ALG_AES_GCM => {
                if usages.iter().any(|usage| {
                    !matches!(
                        usage,
                        KeyUsage::Encrypt |
                            KeyUsage::Decrypt |
                            KeyUsage::WrapKey |
                            KeyUsage::UnwrapKey
                    )
                }) || usages.is_empty()
                {
                    return Err(Error::Syntax);
                }
            },
            ALG_AES_KW => {
                if usages
                    .iter()
                    .any(|usage| !matches!(usage, KeyUsage::WrapKey | KeyUsage::UnwrapKey)) ||
                    usages.is_empty()
                {
                    return Err(Error::Syntax);
                }
            },
            _ => return Err(Error::NotSupported),
        }

        let name = match key_gen_params.name.as_str() {
            ALG_AES_CBC => DOMString::from(ALG_AES_CBC),
            ALG_AES_CTR => DOMString::from(ALG_AES_CTR),
            ALG_AES_KW => DOMString::from(ALG_AES_KW),
            ALG_AES_GCM => DOMString::from(ALG_AES_GCM),
            _ => return Err(Error::NotSupported),
        };

        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut algorithm_object = unsafe {JS_NewObject(*cx, ptr::null()) });
        assert!(!algorithm_object.is_null());

        AesKeyAlgorithm::from_name_and_size(
            name.clone(),
            key_gen_params.length,
            algorithm_object.handle_mut(),
            cx,
        );

        let crypto_key = CryptoKey::new(
            &self.global(),
            KeyType::Secret,
            extractable,
            name,
            algorithm_object.handle(),
            usages,
            handle,
            can_gc,
        );

        Ok(crypto_key)
    }

    /// <https://w3c.github.io/webcrypto/#hmac-operations>
    #[allow(unsafe_code)]
    fn generate_key_hmac(
        &self,
        usages: Vec<KeyUsage>,
        params: &SubtleHmacKeyGenParams,
        extractable: bool,
        can_gc: CanGc,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        // Step 1. If usages contains any entry which is not "sign" or "verify", then throw a SyntaxError.
        if usages
            .iter()
            .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify))
        {
            return Err(Error::Syntax);
        }

        // Step 2.
        let length = match params.length {
            // If the length member of normalizedAlgorithm is not present:
            None => {
                // Let length be the block size in bits of the hash function identified by the
                // hash member of normalizedAlgorithm.
                params.hash.block_size_in_bits() as u32
            },
            // Otherwise, if the length member of normalizedAlgorithm is non-zero:
            Some(length) if length != 0 => {
                // Let length be equal to the length member of normalizedAlgorithm.
                length
            },
            // Otherwise:
            _ => {
                // throw an OperationError.
                return Err(Error::Operation);
            },
        };

        // Step 3. Generate a key of length length bits.
        let mut key_data = vec![0; length as usize];
        self.rng.borrow_mut().fill_bytes(&mut key_data);

        // Step 4. If the key generation step fails, then throw an OperationError.
        // NOTE: Our key generation is infallible.

        // Step 5. Let key be a new CryptoKey object representing the generated key.
        // Step 6. Let algorithm be a new HmacKeyAlgorithm.
        // Step 7. Set the name attribute of algorithm to "HMAC".
        // Step 8. Let hash be a new KeyAlgorithm.
        // Step 9. Set the name attribute of hash to equal the name member of the hash member of normalizedAlgorithm.
        // Step 10. Set the hash attribute of algorithm to hash.
        // Step 11. Set the [[type]] internal slot of key to "secret".
        // Step 12. Set the [[algorithm]] internal slot of key to algorithm.
        // Step 13. Set the [[extractable]] internal slot of key to be extractable.
        // Step 14. Set the [[usages]] internal slot of key to be usages.
        let name = DOMString::from(ALG_HMAC);
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut algorithm_object = unsafe {JS_NewObject(*cx, ptr::null()) });
        assert!(!algorithm_object.is_null());
        HmacKeyAlgorithm::from_length_and_hash(
            length,
            params.hash,
            algorithm_object.handle_mut(),
            cx,
        );

        let key = CryptoKey::new(
            &self.global(),
            KeyType::Secret,
            extractable,
            name,
            algorithm_object.handle(),
            usages,
            Handle::Hmac(key_data),
            can_gc,
        );

        // Step 15. Return key.
        Ok(key)
    }

    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    /// <https://w3c.github.io/webcrypto/#aes-ctr-operations>
    #[allow(unsafe_code)]
    fn import_key_aes(
        &self,
        format: KeyFormat,
        data: &[u8],
        extractable: bool,
        usages: Vec<KeyUsage>,
        alg_name: &str,
        can_gc: CanGc,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        if usages.iter().any(|usage| {
            !matches!(
                usage,
                KeyUsage::Encrypt | KeyUsage::Decrypt | KeyUsage::WrapKey | KeyUsage::UnwrapKey
            )
        }) || usages.is_empty()
        {
            return Err(Error::Syntax);
        }
        if !matches!(format, KeyFormat::Raw | KeyFormat::Jwk) {
            return Err(Error::NotSupported);
        }
        let handle = match data.len() * 8 {
            128 => Handle::Aes128(data.to_vec()),
            192 => Handle::Aes192(data.to_vec()),
            256 => Handle::Aes256(data.to_vec()),
            _ => {
                return Err(Error::Data);
            },
        };

        let name = DOMString::from(alg_name.to_string());

        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut algorithm_object = unsafe { JS_NewObject(*cx, ptr::null()) });
        assert!(!algorithm_object.is_null());

        AesKeyAlgorithm::from_name_and_size(
            name.clone(),
            (data.len() * 8) as u16,
            algorithm_object.handle_mut(),
            cx,
        );
        let crypto_key = CryptoKey::new(
            &self.global(),
            KeyType::Secret,
            extractable,
            name,
            algorithm_object.handle(),
            usages,
            handle,
            can_gc,
        );

        Ok(crypto_key)
    }

    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    /// <https://w3c.github.io/webcrypto/#aes-ctr-operations>
    fn export_key_aes(&self, format: KeyFormat, key: &CryptoKey) -> Result<AesExportedKey, Error> {
        match format {
            KeyFormat::Raw => match key.handle() {
                Handle::Aes128(key_data) => Ok(AesExportedKey::Raw(key_data.as_slice().to_vec())),
                Handle::Aes192(key_data) => Ok(AesExportedKey::Raw(key_data.as_slice().to_vec())),
                Handle::Aes256(key_data) => Ok(AesExportedKey::Raw(key_data.as_slice().to_vec())),
                _ => Err(Error::Data),
            },
            KeyFormat::Jwk => {
                let (alg, k) = match key.handle() {
                    Handle::Aes128(key_data) => {
                        data_to_jwk_params(key.algorithm().as_str(), "128", key_data.as_slice())
                    },
                    Handle::Aes192(key_data) => {
                        data_to_jwk_params(key.algorithm().as_str(), "192", key_data.as_slice())
                    },
                    Handle::Aes256(key_data) => {
                        data_to_jwk_params(key.algorithm().as_str(), "256", key_data.as_slice())
                    },
                    _ => return Err(Error::Data),
                };
                let key_ops = key
                    .usages()
                    .iter()
                    .map(|usage| DOMString::from(usage.as_str()))
                    .collect::<Vec<DOMString>>();
                let jwk = JsonWebKey {
                    alg: Some(alg),
                    crv: None,
                    d: None,
                    dp: None,
                    dq: None,
                    e: None,
                    ext: Some(key.Extractable()),
                    k: Some(k),
                    key_ops: Some(key_ops),
                    kty: Some(DOMString::from("oct")),
                    n: None,
                    oth: None,
                    p: None,
                    q: None,
                    qi: None,
                    use_: None,
                    x: None,
                    y: None,
                };
                Ok(AesExportedKey::Jwk(Box::new(jwk)))
            },
            _ => Err(Error::NotSupported),
        }
    }

    /// <https://w3c.github.io/webcrypto/#hkdf-operations>
    #[allow(unsafe_code)]
    fn import_key_hkdf(
        &self,
        format: KeyFormat,
        data: &[u8],
        extractable: bool,
        usages: Vec<KeyUsage>,
        can_gc: CanGc,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        // Step 1. Let keyData be the key data to be imported.
        // Step 2.  If format is "raw":
        if format == KeyFormat::Raw {
            // Step 1. If usages contains a value that is not "deriveKey" or "deriveBits", then throw a SyntaxError.
            if usages
                .iter()
                .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits)) ||
                usages.is_empty()
            {
                return Err(Error::Syntax);
            }

            // Step 2. If extractable is not false, then throw a SyntaxError.
            if extractable {
                return Err(Error::Syntax);
            }

            // Step 3. Let key be a new CryptoKey representing the key data provided in keyData.
            // Step 4. Set the [[type]] internal slot of key to "secret".
            // Step 5.  Let algorithm be a new KeyAlgorithm object.
            // Step 6. Set the name attribute of algorithm to "HKDF".
            // Step 7. Set the [[algorithm]] internal slot of key to algorithm.
            let name = DOMString::from(ALG_HKDF);
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut algorithm_object = unsafe {JS_NewObject(*cx, ptr::null()) });
            assert!(!algorithm_object.is_null());
            KeyAlgorithm::from_name(name.clone(), algorithm_object.handle_mut(), cx);

            let key = CryptoKey::new(
                &self.global(),
                KeyType::Secret,
                extractable,
                name,
                algorithm_object.handle(),
                usages,
                Handle::Hkdf(data.to_vec()),
                can_gc,
            );

            // Step 8. Return key.
            Ok(key)
        } else {
            // throw a NotSupportedError.
            Err(Error::NotSupported)
        }
    }

    /// <https://w3c.github.io/webcrypto/#hmac-operations>
    #[allow(unsafe_code)]
    fn import_key_hmac(
        &self,
        normalized_algorithm: &SubtleHmacImportParams,
        format: KeyFormat,
        key_data: &[u8],
        extractable: bool,
        usages: Vec<KeyUsage>,
        can_gc: CanGc,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        // Step 1. Let keyData be the key data to be imported.
        // Step 2. If usages contains an entry which is not "sign" or "verify", then throw a SyntaxError.
        // Note: This is not explicitly spec'ed, but also throw a SyntaxError if usages is empty
        if usages
            .iter()
            .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify)) ||
            usages.is_empty()
        {
            return Err(Error::Syntax);
        }

        // Step 3. Let hash be a new KeyAlgorithm.
        let hash;

        // Step 4.
        let data;
        match format {
            // Key data has already been extracted in the case of JWK,
            // so both raw and jwk can be treated the same here.
            KeyFormat::Raw | KeyFormat::Jwk => {
                // Step 4.1 Let data be the octet string contained in keyData.
                data = key_data.to_vec();

                // Step 4.2 Set hash to equal the hash member of normalizedAlgorithm.
                hash = normalized_algorithm.hash;
            },
            // Otherwise:
            _ => {
                // throw a NotSupportedError.
                return Err(Error::NotSupported);
            },
        }

        // Step 5. Let length be equivalent to the length, in octets, of data, multiplied by 8.
        let mut length = data.len() as u32 * 8;

        // Step 6. If length is zero then throw a DataError.
        if length == 0 {
            return Err(Error::Data);
        }

        // Step 7. If the length member of normalizedAlgorithm is present:
        if let Some(given_length) = normalized_algorithm.length {
            //  If the length member of normalizedAlgorithm is greater than length:
            if given_length > length {
                // throw a DataError.
                return Err(Error::Data);
            }
            // Otherwise:
            else {
                // Set length equal to the length member of normalizedAlgorithm.
                length = given_length;
            }
        }

        // Step 8. Let key be a new CryptoKey object representing an HMAC key with the first length bits of data.
        // Step 9. Set the [[type]] internal slot of key to "secret".
        // Step 10. Let algorithm be a new HmacKeyAlgorithm.
        // Step 11. Set the name attribute of algorithm to "HMAC".
        // Step 12. Set the length attribute of algorithm to length.
        // Step 13. Set the hash attribute of algorithm to hash.
        // Step 14. Set the [[algorithm]] internal slot of key to algorithm.
        let truncated_data = data[..length as usize / 8].to_vec();
        let name = DOMString::from(ALG_HMAC);
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut algorithm_object = unsafe { JS_NewObject(*cx, ptr::null()) });
        assert!(!algorithm_object.is_null());
        HmacKeyAlgorithm::from_length_and_hash(length, hash, algorithm_object.handle_mut(), cx);

        let key = CryptoKey::new(
            &self.global(),
            KeyType::Secret,
            extractable,
            name,
            algorithm_object.handle(),
            usages,
            Handle::Hmac(truncated_data),
            can_gc,
        );

        // Step 15. Return key.
        Ok(key)
    }

    /// <https://w3c.github.io/webcrypto/#aes-kw-operations>
    fn wrap_key_aes_kw(
        &self,
        wrapping_key: &CryptoKey,
        bytes: &[u8],
        cx: JSContext,
        handle: MutableHandleObject,
        can_gc: CanGc,
    ) -> Result<Vec<u8>, Error> {
        // Step 1. If plaintext is not a multiple of 64 bits in length, then throw an OperationError.
        if bytes.len() % 8 != 0 {
            return Err(Error::Operation);
        }

        // Step 2. Let ciphertext be the result of performing the Key Wrap operation described in Section 2.2.1
        //         of [RFC3394] with plaintext as the plaintext to be wrapped and using the default Initial Value
        //         defined in Section 2.2.3.1 of the same document.
        let wrapped_key = match wrapping_key.handle() {
            Handle::Aes128(key_data) => {
                let key_array = GenericArray::from_slice(key_data.as_slice());
                let kek = KekAes128::new(key_array);
                match kek.wrap_vec(bytes) {
                    Ok(key) => key,
                    Err(_) => return Err(Error::Operation),
                }
            },
            Handle::Aes192(key_data) => {
                let key_array = GenericArray::from_slice(key_data.as_slice());
                let kek = KekAes192::new(key_array);
                match kek.wrap_vec(bytes) {
                    Ok(key) => key,
                    Err(_) => return Err(Error::Operation),
                }
            },
            Handle::Aes256(key_data) => {
                let key_array = GenericArray::from_slice(key_data.as_slice());
                let kek = KekAes256::new(key_array);
                match kek.wrap_vec(bytes) {
                    Ok(key) => key,
                    Err(_) => return Err(Error::Operation),
                }
            },
            _ => return Err(Error::Operation),
        };

        create_buffer_source::<ArrayBufferU8>(cx, &wrapped_key, handle, can_gc)
            .expect("failed to create buffer source for wrapped key.");

        // 3. Return ciphertext.
        Ok(wrapped_key)
    }

    /// <https://w3c.github.io/webcrypto/#aes-kw-operations>
    fn unwrap_key_aes_kw(
        &self,
        wrapping_key: &CryptoKey,
        bytes: &[u8],
        cx: JSContext,
        handle: MutableHandleObject,
        can_gc: CanGc,
    ) -> Result<Vec<u8>, Error> {
        // Step 1. Let plaintext be the result of performing the Key Unwrap operation described in Section 2.2.2
        //         of [RFC3394] with ciphertext as the input ciphertext and using the default Initial Value defined
        //         in Section 2.2.3.1 of the same document.
        // Step 2. If the Key Unwrap operation returns an error, then throw an OperationError.
        let unwrapped_key = match wrapping_key.handle() {
            Handle::Aes128(key_data) => {
                let key_array = GenericArray::from_slice(key_data.as_slice());
                let kek = KekAes128::new(key_array);
                match kek.unwrap_vec(bytes) {
                    Ok(key) => key,
                    Err(_) => return Err(Error::Operation),
                }
            },
            Handle::Aes192(key_data) => {
                let key_array = GenericArray::from_slice(key_data.as_slice());
                let kek = KekAes192::new(key_array);
                match kek.unwrap_vec(bytes) {
                    Ok(key) => key,
                    Err(_) => return Err(Error::Operation),
                }
            },
            Handle::Aes256(key_data) => {
                let key_array = GenericArray::from_slice(key_data.as_slice());
                let kek = KekAes256::new(key_array);
                match kek.unwrap_vec(bytes) {
                    Ok(key) => key,
                    Err(_) => return Err(Error::Operation),
                }
            },
            _ => return Err(Error::Operation),
        };

        create_buffer_source::<ArrayBufferU8>(cx, &unwrapped_key, handle, can_gc)
            .expect("failed to create buffer source for unwrapped key.");

        // 3. Return plaintext.
        Ok(unwrapped_key)
    }

    /// <https://w3c.github.io/webcrypto/#pbkdf2-operations>
    #[allow(unsafe_code)]
    fn import_key_pbkdf2(
        &self,
        format: KeyFormat,
        data: &[u8],
        extractable: bool,
        usages: Vec<KeyUsage>,
        can_gc: CanGc,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        // Step 1. If format is not "raw", throw a NotSupportedError
        if format != KeyFormat::Raw {
            return Err(Error::NotSupported);
        }

        // Step 2. If usages contains a value that is not "deriveKey" or "deriveBits", then throw a SyntaxError.
        if usages
            .iter()
            .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits)) ||
            usages.is_empty()
        {
            return Err(Error::Syntax);
        }

        // Step 3. If extractable is not false, then throw a SyntaxError.
        if extractable {
            return Err(Error::Syntax);
        }

        // Step 4. Let key be a new CryptoKey representing keyData.
        // Step 5. Set the [[type]] internal slot of key to "secret".
        // Step 6. Let algorithm be a new KeyAlgorithm object.
        // Step 7. Set the name attribute of algorithm to "PBKDF2".
        // Step 8. Set the [[algorithm]] internal slot of key to algorithm.
        let name = DOMString::from(ALG_PBKDF2);
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut algorithm_object = unsafe {JS_NewObject(*cx, ptr::null()) });
        assert!(!algorithm_object.is_null());
        KeyAlgorithm::from_name(name.clone(), algorithm_object.handle_mut(), cx);

        let key = CryptoKey::new(
            &self.global(),
            KeyType::Secret,
            extractable,
            name,
            algorithm_object.handle(),
            usages,
            Handle::Pbkdf2(data.to_vec()),
            can_gc,
        );

        // Step 9. Return key.
        Ok(key)
    }
}

pub(crate) enum AesExportedKey {
    Raw(Vec<u8>),
    Jwk(Box<JsonWebKey>),
}

fn data_to_jwk_params(alg: &str, size: &str, key: &[u8]) -> (DOMString, DOMString) {
    let jwk_alg = match alg {
        ALG_AES_CBC => DOMString::from(format!("A{}CBC", size)),
        ALG_AES_CTR => DOMString::from(format!("A{}CTR", size)),
        ALG_AES_KW => DOMString::from(format!("A{}KW", size)),
        ALG_AES_GCM => DOMString::from(format!("A{}GCM", size)),
        _ => unreachable!(),
    };
    let data = base64::engine::general_purpose::STANDARD_NO_PAD.encode(key);
    (jwk_alg, DOMString::from(data))
}

impl KeyAlgorithm {
    /// Fill the object referenced by `out` with an [KeyAlgorithm]
    /// of the specified name and size.
    #[allow(unsafe_code)]
    fn from_name(name: DOMString, out: MutableHandleObject, cx: JSContext) {
        let key_algorithm = Self { name };

        unsafe {
            key_algorithm.to_jsobject(*cx, out);
        }
    }
}

impl HmacKeyAlgorithm {
    #[allow(unsafe_code)]
    fn from_length_and_hash(
        length: u32,
        hash: DigestAlgorithm,
        out: MutableHandleObject,
        cx: JSContext,
    ) {
        let hmac_key_algorithm = Self {
            parent: KeyAlgorithm {
                name: ALG_HMAC.into(),
            },
            length,
            hash: KeyAlgorithm { name: hash.name() },
        };

        unsafe {
            hmac_key_algorithm.to_jsobject(*cx, out);
        }
    }
}

impl AesKeyAlgorithm {
    /// Fill the object referenced by `out` with an [AesKeyAlgorithm]
    /// of the specified name and size.
    #[allow(unsafe_code)]
    fn from_name_and_size(name: DOMString, size: u16, out: MutableHandleObject, cx: JSContext) {
        let key_algorithm = Self {
            parent: KeyAlgorithm { name },
            length: size,
        };

        unsafe {
            key_algorithm.to_jsobject(*cx, out);
        }
    }
}

impl SubtleHkdfParams {
    /// <https://w3c.github.io/webcrypto/#hkdf-operations>
    fn derive_bits(&self, key: &CryptoKey, length: Option<u32>) -> Result<Vec<u8>, Error> {
        // Step 1. If length is null or zero, or is not a multiple of 8, then throw an OperationError.
        let Some(length) = length else {
            return Err(Error::Operation);
        };
        if length == 0 || length % 8 != 0 {
            return Err(Error::Operation);
        };

        // Step 3. Let keyDerivationKey be the secret represented by [[handle]] internal slot of key.
        let key_derivation_key = key.handle().as_bytes();

        // Step 4. Let result be the result of performing the HKDF extract and then the HKDF expand step described
        // in Section 2 of [RFC5869] using:
        // * the hash member of normalizedAlgorithm as Hash,
        // * keyDerivationKey as the input keying material, IKM,
        // * the contents of the salt member of normalizedAlgorithm as salt,
        // * the contents of the info member of normalizedAlgorithm as info,
        // * length divided by 8 as the value of L,
        let mut result = vec![0; length as usize / 8];
        let algorithm = match self.hash {
            DigestAlgorithm::Sha1 => hkdf::HKDF_SHA1_FOR_LEGACY_USE_ONLY,
            DigestAlgorithm::Sha256 => hkdf::HKDF_SHA256,
            DigestAlgorithm::Sha384 => hkdf::HKDF_SHA384,
            DigestAlgorithm::Sha512 => hkdf::HKDF_SHA512,
        };
        let salt = hkdf::Salt::new(algorithm, &self.salt);
        let info = self.info.as_slice();
        let pseudo_random_key = salt.extract(key_derivation_key);

        let Ok(output_key_material) =
            pseudo_random_key.expand(std::slice::from_ref(&info), algorithm)
        else {
            // Step 5. If the key derivation operation fails, then throw an OperationError.
            return Err(Error::Operation);
        };

        if output_key_material.fill(&mut result).is_err() {
            return Err(Error::Operation);
        };

        // Step 6. Return the result of creating an ArrayBuffer containing result.
        // NOTE: The ArrayBuffer is created by the caller
        Ok(result)
    }
}

impl SubtlePbkdf2Params {
    /// <https://w3c.github.io/webcrypto/#pbkdf2-operations>
    fn derive_bits(&self, key: &CryptoKey, length: Option<u32>) -> Result<Vec<u8>, Error> {
        // Step 1. If length is null or zero, or is not a multiple of 8, then throw an OperationError.
        let Some(length) = length else {
            return Err(Error::Operation);
        };
        if length == 0 || length % 8 != 0 {
            return Err(Error::Operation);
        };

        // Step 2. If the iterations member of normalizedAlgorithm is zero, then throw an OperationError.
        let Ok(iterations) = NonZero::<u32>::try_from(self.iterations) else {
            return Err(Error::Operation);
        };

        // Step 3. Let prf be the MAC Generation function described in Section 4 of [FIPS-198-1]
        // using the hash function described by the hash member of normalizedAlgorithm.
        let prf = match self.hash {
            DigestAlgorithm::Sha1 => pbkdf2::PBKDF2_HMAC_SHA1,
            DigestAlgorithm::Sha256 => pbkdf2::PBKDF2_HMAC_SHA256,
            DigestAlgorithm::Sha384 => pbkdf2::PBKDF2_HMAC_SHA384,
            DigestAlgorithm::Sha512 => pbkdf2::PBKDF2_HMAC_SHA512,
        };

        // Step 4. Let result be the result of performing the PBKDF2 operation defined in Section 5.2 of [RFC8018] using
        // prf as the pseudo-random function, PRF, the password represented by [[handle]] internal slot of key as
        // the password, P, the contents of the salt attribute of normalizedAlgorithm as the salt, S, the value of
        // the iterations attribute of normalizedAlgorithm as the iteration count, c, and length divided by 8 as the
        // intended key length, dkLen.
        let mut result = vec![0; length as usize / 8];
        pbkdf2::derive(
            prf,
            iterations,
            &self.salt,
            key.handle().as_bytes(),
            &mut result,
        );

        // Step 5. If the key derivation operation fails, then throw an OperationError.
        // TODO: Investigate when key derivation can fail and how ring handles that case
        // (pbkdf2::derive does not return a Result type)

        // Step 6. Return result
        Ok(result)
    }
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations>
fn get_key_length_for_aes(length: u16) -> Result<u32, Error> {
    // Step 1. If the length member of normalizedDerivedKeyAlgorithm is not 128, 192 or 256,
    // then throw an OperationError.
    if !matches!(length, 128 | 192 | 256) {
        return Err(Error::Operation);
    }

    // Step 2. Return the length member of normalizedDerivedKeyAlgorithm.
    Ok(length as u32)
}

impl GetKeyLengthAlgorithm {
    fn get_key_length(&self) -> Result<u32, Error> {
        match self {
            Self::Aes(length) => get_key_length_for_aes(*length),
            Self::Hmac(params) => params.get_key_length(),
        }
    }
}

impl DigestAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    fn name(&self) -> DOMString {
        match self {
            Self::Sha1 => ALG_SHA1,
            Self::Sha256 => ALG_SHA256,
            Self::Sha384 => ALG_SHA384,
            Self::Sha512 => ALG_SHA512,
        }
        .into()
    }

    fn digest(&self, data: &[u8]) -> Result<impl AsRef<[u8]>, Error> {
        let algorithm = match self {
            Self::Sha1 => &digest::SHA1_FOR_LEGACY_USE_ONLY,
            Self::Sha256 => &digest::SHA256,
            Self::Sha384 => &digest::SHA384,
            Self::Sha512 => &digest::SHA512,
        };
        Ok(digest::digest(algorithm, data))
    }

    fn block_size_in_bits(&self) -> usize {
        match self {
            Self::Sha1 => 160,
            Self::Sha256 => 256,
            Self::Sha384 => 384,
            Self::Sha512 => 512,
        }
    }
}

impl ImportKeyAlgorithm {
    fn import_key(
        &self,
        subtle: &SubtleCrypto,
        format: KeyFormat,
        secret: &[u8],
        extractable: bool,
        key_usages: Vec<KeyUsage>,
        can_gc: CanGc,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        match self {
            Self::AesCbc => {
                subtle.import_key_aes(format, secret, extractable, key_usages, ALG_AES_CBC, can_gc)
            },
            Self::AesCtr => {
                subtle.import_key_aes(format, secret, extractable, key_usages, ALG_AES_CTR, can_gc)
            },
            Self::AesKw => {
                subtle.import_key_aes(format, secret, extractable, key_usages, ALG_AES_KW, can_gc)
            },
            Self::AesGcm => {
                subtle.import_key_aes(format, secret, extractable, key_usages, ALG_AES_GCM, can_gc)
            },
            Self::Hmac(params) => {
                subtle.import_key_hmac(params, format, secret, extractable, key_usages, can_gc)
            },
            Self::Pbkdf2 => {
                subtle.import_key_pbkdf2(format, secret, extractable, key_usages, can_gc)
            },
            Self::Hkdf => subtle.import_key_hkdf(format, secret, extractable, key_usages, can_gc),
        }
    }
}

impl DeriveBitsAlgorithm {
    fn derive_bits(&self, key: &CryptoKey, length: Option<u32>) -> Result<Vec<u8>, Error> {
        match self {
            Self::Pbkdf2(pbkdf2_params) => pbkdf2_params.derive_bits(key, length),
            Self::Hkdf(hkdf_params) => hkdf_params.derive_bits(key, length),
        }
    }
}

impl EncryptionAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    fn name(&self) -> &str {
        match self {
            Self::AesCbc(params) => &params.name,
            Self::AesCtr(params) => &params.name,
            Self::AesGcm(params) => &params.name,
        }
    }

    // FIXME: This doesn't really need the "SubtleCrypto" argument
    fn encrypt(
        &self,
        subtle: &SubtleCrypto,
        key: &CryptoKey,
        data: &[u8],
        cx: JSContext,
        result: MutableHandleObject,
        can_gc: CanGc,
    ) -> Result<Vec<u8>, Error> {
        match self {
            Self::AesCbc(params) => subtle.encrypt_aes_cbc(params, key, data, cx, result, can_gc),
            Self::AesCtr(params) => {
                subtle.encrypt_decrypt_aes_ctr(params, key, data, cx, result, can_gc)
            },
            Self::AesGcm(params) => subtle.encrypt_aes_gcm(params, key, data, cx, result, can_gc),
        }
    }

    // FIXME: This doesn't really need the "SubtleCrypto" argument
    fn decrypt(
        &self,
        subtle: &SubtleCrypto,
        key: &CryptoKey,
        data: &[u8],
        cx: JSContext,
        result: MutableHandleObject,
        can_gc: CanGc,
    ) -> Result<Vec<u8>, Error> {
        match self {
            Self::AesCbc(params) => subtle.decrypt_aes_cbc(params, key, data, cx, result, can_gc),
            Self::AesCtr(params) => {
                subtle.encrypt_decrypt_aes_ctr(params, key, data, cx, result, can_gc)
            },
            Self::AesGcm(params) => subtle.decrypt_aes_gcm(params, key, data, cx, result, can_gc),
        }
    }
}

impl SignatureAlgorithm {
    fn name(&self) -> &str {
        match self {
            Self::Hmac => ALG_HMAC,
        }
    }

    fn sign(&self, cx: JSContext, key: &CryptoKey, data: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            Self::Hmac => sign_hmac(cx, key, data).map(|s| s.as_ref().to_vec()),
        }
    }

    fn verify(
        &self,
        cx: JSContext,
        key: &CryptoKey,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, Error> {
        match self {
            Self::Hmac => verify_hmac(cx, key, data, signature),
        }
    }
}

impl KeyGenerationAlgorithm {
    // FIXME: This doesn't really need the "SubtleCrypto" argument
    fn generate_key(
        &self,
        subtle: &SubtleCrypto,
        usages: Vec<KeyUsage>,
        extractable: bool,
        can_gc: CanGc,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        match self {
            Self::Aes(params) => subtle.generate_key_aes(usages, params, extractable, can_gc),
            Self::Hmac(params) => subtle.generate_key_hmac(usages, params, extractable, can_gc),
        }
    }
}

/// <https://w3c.github.io/webcrypto/#hmac-operations>
fn sign_hmac(cx: JSContext, key: &CryptoKey, data: &[u8]) -> Result<impl AsRef<[u8]>, Error> {
    // Step 1. Let mac be the result of performing the MAC Generation operation described in Section 4 of [FIPS-198-1]
    // using the key represented by [[handle]] internal slot of key, the hash function identified by the hash attribute
    // of the [[algorithm]] internal slot of key and message as the input data text.
    rooted!(in(*cx) let mut algorithm_slot = ObjectValue(key.Algorithm(cx).as_ptr()));
    let params = value_from_js_object!(HmacKeyAlgorithm, cx, algorithm_slot);

    let hash_algorithm = match params.hash.name.str() {
        ALG_SHA1 => hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
        ALG_SHA256 => hmac::HMAC_SHA256,
        ALG_SHA384 => hmac::HMAC_SHA384,
        ALG_SHA512 => hmac::HMAC_SHA512,
        _ => return Err(Error::NotSupported),
    };

    let sign_key = hmac::Key::new(hash_algorithm, key.handle().as_bytes());
    let mac = hmac::sign(&sign_key, data);

    // Step 2. Return the result of creating an ArrayBuffer containing mac.
    // NOTE: This is done by the caller
    Ok(mac)
}

/// <https://w3c.github.io/webcrypto/#hmac-operations>
fn verify_hmac(
    cx: JSContext,
    key: &CryptoKey,
    data: &[u8],
    signature: &[u8],
) -> Result<bool, Error> {
    // Step 1. Let mac be the result of performing the MAC Generation operation described in Section 4 of [FIPS-198-1]
    // using the key represented by [[handle]] internal slot of key, the hash function identified by the hash attribute
    // of the [[algorithm]] internal slot of key and message as the input data text.
    let mac = sign_hmac(cx, key, data)?;

    // Step 2. Return true if mac is equal to signature and false otherwise.
    let is_valid = mac.as_ref() == signature;
    Ok(is_valid)
}

impl KeyWrapAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    fn name(&self) -> &str {
        match self {
            Self::AesKw => ALG_AES_KW,
            Self::AesCbc(key_gen_params) => &key_gen_params.name,
            Self::AesCtr(key_gen_params) => &key_gen_params.name,
            Self::AesGcm(_) => ALG_AES_GCM,
        }
    }
}

/// <https://w3c.github.io/webcrypto/#concept-parse-a-jwk>
fn parse_jwk(
    bytes: &[u8],
    import_alg: ImportKeyAlgorithm,
    extractable: bool,
    key_usages: &[KeyUsage],
) -> Result<Vec<u8>, Error> {
    let value = serde_json::from_slice(bytes)
        .map_err(|_| Error::Type("Failed to parse JWK string".into()))?;
    let serde_json::Value::Object(obj) = value else {
        return Err(Error::Data);
    };

    let kty = get_jwk_string(&obj, "kty")?;
    let ext = get_jwk_bool(&obj, "ext")?;
    if !ext && extractable {
        return Err(Error::Data);
    }

    // If the key_ops field of jwk is present, and is invalid according to the requirements of JSON Web Key [JWK]
    // or does not contain all of the specified usages values, then throw a DataError.
    if let Some(serde_json::Value::Array(key_ops)) = obj.get("key_ops") {
        if key_ops.iter().any(|op| {
            let op_string = match op {
                serde_json::Value::String(op_string) => op_string,
                _ => return true,
            };
            let usage = match usage_from_str(op_string) {
                Ok(usage) => usage,
                Err(_) => {
                    return true;
                },
            };
            !key_usages.contains(&usage)
        }) {
            return Err(Error::Data);
        }
    }

    match import_alg {
        ImportKeyAlgorithm::AesCbc |
        ImportKeyAlgorithm::AesCtr |
        ImportKeyAlgorithm::AesKw |
        ImportKeyAlgorithm::AesGcm => {
            if kty != "oct" {
                return Err(Error::Data);
            }
            let k = get_jwk_string(&obj, "k")?;
            let alg = get_jwk_string(&obj, "alg")?;

            let data = base64::engine::general_purpose::STANDARD_NO_PAD
                .decode(k.as_bytes())
                .map_err(|_| Error::Data)?;

            let expected_alg = match (data.len() * 8, &import_alg) {
                (128, ImportKeyAlgorithm::AesCbc) => "A128CBC",
                (128, ImportKeyAlgorithm::AesCtr) => "A128CTR",
                (128, ImportKeyAlgorithm::AesKw) => "A128KW",
                (128, ImportKeyAlgorithm::AesGcm) => "A128GCM",
                (192, ImportKeyAlgorithm::AesCbc) => "A192CBC",
                (192, ImportKeyAlgorithm::AesCtr) => "A192CTR",
                (192, ImportKeyAlgorithm::AesKw) => "A192KW",
                (192, ImportKeyAlgorithm::AesGcm) => "A192GCM",
                (256, ImportKeyAlgorithm::AesCbc) => "A256CBC",
                (256, ImportKeyAlgorithm::AesCtr) => "A256CTR",
                (256, ImportKeyAlgorithm::AesKw) => "A256KW",
                (256, ImportKeyAlgorithm::AesGcm) => "A256GCM",
                _ => return Err(Error::Data),
            };

            if alg != expected_alg {
                return Err(Error::Data);
            }

            if let Some(serde_json::Value::String(use_)) = obj.get("use") {
                if use_ != "enc" {
                    return Err(Error::Data);
                }
            }

            Ok(data)
        },
        ImportKeyAlgorithm::Hmac(params) => {
            if kty != "oct" {
                return Err(Error::Data);
            }
            let k = get_jwk_string(&obj, "k")?;
            let alg = get_jwk_string(&obj, "alg")?;

            let expected_alg = match params.hash {
                DigestAlgorithm::Sha1 => "HS1",
                DigestAlgorithm::Sha256 => "HS256",
                DigestAlgorithm::Sha384 => "HS384",
                DigestAlgorithm::Sha512 => "HS512",
            };

            if alg != expected_alg {
                return Err(Error::Data);
            }

            if let Some(serde_json::Value::String(use_)) = obj.get("use") {
                if use_ != "sign" {
                    return Err(Error::Data);
                }
            }

            base64::engine::general_purpose::STANDARD_NO_PAD
                .decode(k.as_bytes())
                .map_err(|_| Error::Data)
        },
        _ => Err(Error::NotSupported),
    }
}

fn get_jwk_string(
    value: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<String, Error> {
    let s = value
        .get(key)
        .ok_or(Error::Data)?
        .as_str()
        .ok_or(Error::Data)?;
    Ok(s.to_string())
}

fn get_jwk_bool(
    value: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<bool, Error> {
    let b = value
        .get(key)
        .ok_or(Error::Data)?
        .as_bool()
        .ok_or(Error::Data)?;
    Ok(b)
}

fn usage_from_str(op: &str) -> Result<KeyUsage, Error> {
    let usage = match op {
        "encrypt" => KeyUsage::Encrypt,
        "decrypt" => KeyUsage::Decrypt,
        "sign" => KeyUsage::Sign,
        "verify" => KeyUsage::Verify,
        "deriveKey" => KeyUsage::DeriveKey,
        "deriveBits" => KeyUsage::DeriveBits,
        "wrapKey" => KeyUsage::WrapKey,
        "unwrapKey" => KeyUsage::UnwrapKey,
        _ => {
            return Err(Error::Data);
        },
    };
    Ok(usage)
}
