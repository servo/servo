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
use base64::prelude::*;
use dom_struct::dom_struct;
use js::conversions::ConversionResult;
use js::jsapi::{JSObject, JS_NewObject};
use js::jsval::ObjectValue;
use js::rust::MutableHandleObject;
use js::typedarray::ArrayBufferU8;
use ring::{digest, hkdf, hmac, pbkdf2};
use servo_rand::{RngCore, ServoRng};

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{
    AesCbcParams, AesCtrParams, AesDerivedKeyParams, AesKeyAlgorithm, AesKeyGenParams, Algorithm,
    AlgorithmIdentifier, HkdfParams, HmacImportParams, HmacKeyAlgorithm, JsonWebKey, KeyAlgorithm,
    KeyFormat, Pbkdf2Params, SubtleCryptoMethods,
};
use crate::dom::bindings::codegen::UnionTypes::{
    ArrayBufferViewOrArrayBuffer, ArrayBufferViewOrArrayBufferOrJsonWebKey,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::import::module::SafeJSContext;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::realms::InRealm;
use crate::script_runtime::{CanGc, JSContext};
use crate::task::TaskCanceller;
use crate::task_source::dom_manipulation::DOMManipulationTaskSource;
use crate::task_source::TaskSource;

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

#[dom_struct]
pub struct SubtleCrypto {
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

    pub(crate) fn new(global: &GlobalScope) -> DomRoot<SubtleCrypto> {
        reflect_dom_object(Box::new(SubtleCrypto::new_inherited()), global)
    }

    fn task_source_with_canceller(&self) -> (DOMManipulationTaskSource, TaskCanceller) {
        if let Some(window) = self.global().downcast::<Window>() {
            window
                .task_manager()
                .dom_manipulation_task_source_with_canceller()
        } else if let Some(worker_global) = self.global().downcast::<WorkerGlobalScope>() {
            let task_source = worker_global.dom_manipulation_task_source();
            let canceller = worker_global.task_canceller();
            (task_source, canceller)
        } else {
            unreachable!("Couldn't downcast to Window or WorkerGlobalScope!");
        }
    }
}

impl SubtleCryptoMethods for SubtleCrypto {
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

        let (task_source, canceller) = self.task_source_with_canceller();
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);
        let key_alg = key.algorithm();
        let valid_usage = key.usages().contains(&KeyUsage::Encrypt);
        let _ = task_source.queue_with_canceller(
            task!(encrypt: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();

                if !valid_usage || normalized_algorithm.name() != key_alg {
                    promise.reject_error(Error::InvalidAccess);
                    return;
                }

                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());

                if let Err(e) = normalized_algorithm.encrypt(&subtle, &key, &data, cx, array_buffer_ptr.handle_mut()) {
                    promise.reject_error(e);
                    return;
                }
                promise.resolve_native(&*array_buffer_ptr.handle());
            }),
            &canceller,
        );

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

        let (task_source, canceller) = self.task_source_with_canceller();
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);
        let key_alg = key.algorithm();
        let valid_usage = key.usages().contains(&KeyUsage::Decrypt);
        let _ = task_source.queue_with_canceller(
            task!(decrypt: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());

                if !valid_usage || normalized_algorithm.name() != key_alg {
                    promise.reject_error(Error::InvalidAccess);
                    return;
                }

                if let Err(e) = normalized_algorithm.decrypt(&subtle, &key, &data, cx, array_buffer_ptr.handle_mut()) {
                    promise.reject_error(e);
                    return;
                }

                promise.resolve_native(&*array_buffer_ptr.handle());
            }),
            &canceller,
        );

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
        let (task_source, canceller) = self.task_source_with_canceller();
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);

        let _ = task_source.queue_with_canceller(
            task!(sign: move || {
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
                create_buffer_source::<ArrayBufferU8>(cx, &result, array_buffer_ptr.handle_mut())
                    .expect("failed to create buffer source for exported key.");

                // Step 9. Resolve promise with result.
                promise.resolve_native(&*array_buffer_ptr);
            }),
            &canceller,
        );

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
        let (task_source, canceller) = self.task_source_with_canceller();
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);

        let _ = task_source.queue_with_canceller(
            task!(sign: move || {
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
                promise.resolve_native(&result);
            }),
            &canceller,
        );

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
        let (task_source, canceller) = self.task_source_with_canceller();
        let trusted_promise = TrustedPromise::new(promise.clone());

        let _ = task_source.queue_with_canceller(
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
                create_buffer_source::<ArrayBufferU8>(cx, digest.as_ref(), array_buffer_ptr.handle_mut())
                    .expect("failed to create buffer source for exported key.");


                // Step 9. Resolve promise with result.
                promise.resolve_native(&*array_buffer_ptr);
            }),
            &canceller,
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

        let (task_source, canceller) = self.task_source_with_canceller();
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let _ = task_source.queue_with_canceller(
            task!(generate_key: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = normalized_algorithm.generate_key(&subtle, key_usages, extractable);

                match key {
                    Ok(key) => promise.resolve_native(&key),
                    Err(e) => promise.reject_error(e),
                }
            }),
            &canceller,
        );

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
        let (task_source, canceller) = self.task_source_with_canceller();
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_base_key = Trusted::new(base_key);
        let this = Trusted::new(self);
        let _ = task_source.queue_with_canceller(
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
                    key_usages
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
                promise.resolve_native(&*result);
            }),
            &canceller,
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
        let (task_source, canceller) = self.task_source_with_canceller();
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_base_key = Trusted::new(base_key);

        let _ = task_source.queue_with_canceller(
            task!(import_key: move || {
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

                create_buffer_source::<ArrayBufferU8>(cx, &result, array_buffer_ptr.handle_mut())
                    .expect("failed to create buffer source for derived bits.");

                // Step 10. Resolve promise with result.
                promise.resolve_native(&*array_buffer_ptr);
            }),
            &canceller,
        );

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

        // TODO: Figure out a way to Send this data so per-algorithm JWK checks can happen
        let data = match key_data {
            ArrayBufferViewOrArrayBufferOrJsonWebKey::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBufferOrJsonWebKey::JsonWebKey(json_web_key) => {
                if let Some(mut data_string) = json_web_key.k {
                    while data_string.len() % 4 != 0 {
                        data_string.push_str("=");
                    }
                    match BASE64_STANDARD.decode(data_string.to_string()) {
                        Ok(data) => data,
                        Err(_) => {
                            promise.reject_error(Error::Syntax);
                            return promise;
                        },
                    }
                } else {
                    promise.reject_error(Error::Syntax);
                    return promise;
                }
            },
            ArrayBufferViewOrArrayBufferOrJsonWebKey::ArrayBuffer(array_buffer) => {
                array_buffer.to_vec()
            },
        };

        let (task_source, canceller) = self.task_source_with_canceller();
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let _ = task_source.queue_with_canceller(
            task!(import_key: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let imported_key = normalized_algorithm.import_key(&subtle, format, &data, extractable, key_usages);
                match imported_key {
                    Ok(k) => promise.resolve_native(&k),
                    Err(e) => promise.reject_error(e),
                };
            }),
            &canceller,
        );

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

        let (task_source, canceller) = self.task_source_with_canceller();
        let this = Trusted::new(self);
        let trusted_key = Trusted::new(key);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let _ = task_source.queue_with_canceller(
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
                    ALG_AES_CBC | ALG_AES_CTR => subtle.export_key_aes(format, &key),
                    _ => Err(Error::NotSupported),
                };
                match exported_key {
                    Ok(k) => {
                        match k {
                            AesExportedKey::Raw(k) => {
                                let cx = GlobalScope::get_cx();
                                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());
                                create_buffer_source::<ArrayBufferU8>(cx, &k, array_buffer_ptr.handle_mut())
                                    .expect("failed to create buffer source for exported key.");
                                promise.resolve_native(&array_buffer_ptr.get())
                            },
                            AesExportedKey::Jwk(k) => {
                                promise.resolve_native(&k)
                            },
                        }
                    },
                    Err(e) => promise.reject_error(e),
                }
            }),
            &canceller,
        );

        promise
    }
}

// These "subtle" structs are proxies for the codegen'd dicts which don't hold a DOMString
// so they can be sent safely when running steps in parallel.

#[derive(Clone, Debug)]
pub struct SubtleAlgorithm {
    #[allow(dead_code)]
    pub name: String,
}

impl From<DOMString> for SubtleAlgorithm {
    fn from(name: DOMString) -> Self {
        SubtleAlgorithm {
            name: name.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubtleAesCbcParams {
    #[allow(dead_code)]
    pub name: String,
    pub iv: Vec<u8>,
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
pub struct SubtleAesCtrParams {
    pub name: String,
    pub counter: Vec<u8>,
    pub length: u8,
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
pub struct SubtleAesKeyGenParams {
    pub name: String,
    pub length: u16,
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

/// <https://w3c.github.io/webcrypto/#hkdf-params>
#[derive(Clone, Debug)]
pub struct SubtleHkdfParams {
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
pub struct SubtlePbkdf2Params {
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
enum ImportKeyAlgorithm {
    AesCbc,
    AesCtr,
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
enum EncryptionAlgorithm {
    AesCbc(SubtleAesCbcParams),
    AesCtr(SubtleAesCtrParams),
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
                name.eq_ignore_ascii_case(ALG_AES_CTR)
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
    let normalized_algorithm =
        if name.eq_ignore_ascii_case(ALG_AES_CBC) || name.eq_ignore_ascii_case(ALG_AES_CTR) {
            let params = value_from_js_object!(AesKeyGenParams, cx, value);
            KeyGenerationAlgorithm::Aes(params.into())
        } else {
            return Err(Error::NotSupported);
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
    ) -> Result<(), Error> {
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

        create_buffer_source::<ArrayBufferU8>(cx, &ct, handle)
            .expect("failed to create buffer source for exported key.");

        Ok(())
    }

    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    fn decrypt_aes_cbc(
        &self,
        params: &SubtleAesCbcParams,
        key: &CryptoKey,
        data: &[u8],
        cx: JSContext,
        handle: MutableHandleObject,
    ) -> Result<(), Error> {
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

        create_buffer_source::<ArrayBufferU8>(cx, plaintext, handle)
            .expect("failed to create buffer source for exported key.");

        Ok(())
    }

    /// <https://w3c.github.io/webcrypto/#aes-ctr-operations>
    fn encrypt_decrypt_aes_ctr(
        &self,
        params: &SubtleAesCtrParams,
        key: &CryptoKey,
        data: &[u8],
        cx: JSContext,
        handle: MutableHandleObject,
    ) -> Result<(), Error> {
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

        create_buffer_source::<ArrayBufferU8>(cx, &ciphertext, handle)
            .expect("failed to create buffer source for exported key.");

        Ok(())
    }

    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    /// <https://w3c.github.io/webcrypto/#aes-ctr-operations>
    #[allow(unsafe_code)]
    fn generate_key_aes(
        &self,
        usages: Vec<KeyUsage>,
        key_gen_params: &SubtleAesKeyGenParams,
        extractable: bool,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        let mut rand = vec![0; key_gen_params.length as usize];
        self.rng.borrow_mut().fill_bytes(&mut rand);
        let handle = match key_gen_params.length {
            128 => Handle::Aes128(rand),
            192 => Handle::Aes192(rand),
            256 => Handle::Aes256(rand),
            _ => return Err(Error::Operation),
        };

        if usages.iter().any(|usage| {
            !matches!(
                usage,
                KeyUsage::Encrypt | KeyUsage::Decrypt | KeyUsage::WrapKey | KeyUsage::UnwrapKey
            )
        }) || usages.is_empty()
        {
            return Err(Error::Syntax);
        }

        let name = match key_gen_params.name.as_str() {
            ALG_AES_CBC => DOMString::from(ALG_AES_CBC),
            ALG_AES_CTR => DOMString::from(ALG_AES_CTR),
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
        );

        Ok(crypto_key)
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
            _ => return Err(Error::Data),
        };

        let name = DOMString::from(alg_name.to_string());

        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut algorithm_object = unsafe {JS_NewObject(*cx, ptr::null()) });
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
                let jwk = JsonWebKey {
                    alg: Some(alg),
                    crv: None,
                    d: None,
                    dp: None,
                    dq: None,
                    e: None,
                    ext: Some(key.Extractable()),
                    k: Some(k),
                    key_ops: None,
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
    ) -> Result<DomRoot<CryptoKey>, Error> {
        // Step 1. Let keyData be the key data to be imported.
        // Step 2.  If format is "raw":
        if format == KeyFormat::Raw {
            // Step 1. If usages contains a value that is not "deriveKey" or "deriveBits", then throw a SyntaxError.
            if usages
                .iter()
                .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
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
    ) -> Result<DomRoot<CryptoKey>, Error> {
        // Step 1. Let keyData be the key data to be imported.
        // Step 2. If usages contains an entry which is not "sign" or "verify", then throw a SyntaxError.
        if usages
            .iter()
            .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify))
        {
            return Err(Error::Syntax);
        }

        // Step 3. Let hash be a new KeyAlgorithm.
        let hash;

        // Step 4.
        let data;
        match format {
            // If format is "raw":
            KeyFormat::Raw => {
                // Step 4.1 Let data be the octet string contained in keyData.
                data = key_data;

                // Step 4.2 Set hash to equal the hash member of normalizedAlgorithm.
                hash = normalized_algorithm.hash;
            },
            // If format is "jwk":
            KeyFormat::Jwk => {
                // TODO: This seems to require having key_data be more than just &[u8]
                return Err(Error::NotSupported);
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
        rooted!(in(*cx) let mut algorithm_object = unsafe {JS_NewObject(*cx, ptr::null()) });
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
        );

        // Step 15. Return key.
        Ok(key)
    }

    /// <https://w3c.github.io/webcrypto/#pbkdf2-operations>
    #[allow(unsafe_code)]
    fn import_key_pbkdf2(
        &self,
        format: KeyFormat,
        data: &[u8],
        extractable: bool,
        usages: Vec<KeyUsage>,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        // Step 1. If format is not "raw", throw a NotSupportedError
        if format != KeyFormat::Raw {
            return Err(Error::NotSupported);
        }

        // Step 2. If usages contains a value that is not "deriveKey" or "deriveBits", then throw a SyntaxError.
        if usages
            .iter()
            .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
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
        );

        // Step 9. Return key.
        Ok(key)
    }
}

pub enum AesExportedKey {
    Raw(Vec<u8>),
    Jwk(Box<JsonWebKey>),
}

fn data_to_jwk_params(alg: &str, size: &str, key: &[u8]) -> (DOMString, DOMString) {
    let jwk_alg = match alg {
        ALG_AES_CBC => DOMString::from(format!("A{}CBC", size)),
        ALG_AES_CTR => DOMString::from(format!("A{}CTR", size)),
        _ => unreachable!(),
    };
    let mut data = BASE64_STANDARD.encode(key);
    data.retain(|c| c != '=');
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
}

impl ImportKeyAlgorithm {
    fn import_key(
        &self,
        subtle: &SubtleCrypto,
        format: KeyFormat,
        secret: &[u8],
        extractable: bool,
        key_usages: Vec<KeyUsage>,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        match self {
            Self::AesCbc => {
                subtle.import_key_aes(format, secret, extractable, key_usages, ALG_AES_CBC)
            },
            Self::AesCtr => {
                subtle.import_key_aes(format, secret, extractable, key_usages, ALG_AES_CTR)
            },
            Self::Hmac(params) => {
                subtle.import_key_hmac(params, format, secret, extractable, key_usages)
            },
            Self::Pbkdf2 => subtle.import_key_pbkdf2(format, secret, extractable, key_usages),
            Self::Hkdf => subtle.import_key_hkdf(format, secret, extractable, key_usages),
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
            Self::AesCbc(key_gen_params) => &key_gen_params.name,
            Self::AesCtr(key_gen_params) => &key_gen_params.name,
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
    ) -> Result<(), Error> {
        match self {
            Self::AesCbc(key_gen_params) => {
                subtle.encrypt_aes_cbc(key_gen_params, key, data, cx, result)
            },
            Self::AesCtr(key_gen_params) => {
                subtle.encrypt_decrypt_aes_ctr(key_gen_params, key, data, cx, result)
            },
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
    ) -> Result<(), Error> {
        match self {
            Self::AesCbc(key_gen_params) => {
                subtle.decrypt_aes_cbc(key_gen_params, key, data, cx, result)
            },
            Self::AesCtr(key_gen_params) => {
                subtle.encrypt_decrypt_aes_ctr(key_gen_params, key, data, cx, result)
            },
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
    ) -> Result<DomRoot<CryptoKey>, Error> {
        match self {
            Self::Aes(params) => subtle.generate_key_aes(usages, params, extractable),
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
