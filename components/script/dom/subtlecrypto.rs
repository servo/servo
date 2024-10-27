/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;
use std::rc::Rc;

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, StreamCipher};
use aes::{Aes128, Aes192, Aes256};
use base64::prelude::*;
use dom_struct::dom_struct;
use js::conversions::ConversionResult;
use js::jsapi::JSObject;
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
    AesCbcParams, AesCtrParams, AesKeyGenParams, Algorithm, AlgorithmIdentifier, JsonWebKey,
    KeyAlgorithm, KeyFormat, SubtleCryptoMethods,
};
use crate::dom::bindings::codegen::UnionTypes::{
    ArrayBufferViewOrArrayBuffer, ArrayBufferViewOrArrayBufferOrJsonWebKey,
};
use crate::dom::bindings::error::Error;
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
const ALG_SHA1: &str = "SHA1";
const ALG_SHA256: &str = "SHA256";
const ALG_SHA384: &str = "SHA384";
const ALG_SHA512: &str = "SHA512";
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
        let normalized_algorithm = normalize_algorithm(cx, algorithm, "encrypt");
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let data = match data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        let (task_source, canceller) = self.task_source_with_canceller();
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);
        let alg = normalized_algorithm.clone();
        let key_alg = key.algorithm();
        let valid_usage = key.usages().contains(&KeyUsage::Encrypt);
        let _ = task_source.queue_with_canceller(
            task!(encrypt: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());
                let text = match alg {
                    Ok(NormalizedAlgorithm::AesCbcParams(key_gen_params)) => {
                        if !valid_usage || key_gen_params.name != key_alg {
                            Err(Error::InvalidAccess)
                        } else {
                            match subtle.encrypt_aes_cbc(
                                key_gen_params, &key, &data, cx, array_buffer_ptr.handle_mut()
                            ) {
                                Ok(_) => Ok(array_buffer_ptr.handle()),
                                Err(e) => Err(e),
                            }
                        }
                    },
                    Ok(NormalizedAlgorithm::AesCtrParams(key_gen_params)) => {
                        if !valid_usage || key_gen_params.name != key_alg {
                            Err(Error::InvalidAccess)
                        } else {
                            match subtle.encrypt_decrypt_aes_ctr(
                                key_gen_params, &key, &data, cx, array_buffer_ptr.handle_mut()
                            ) {
                                Ok(_) => Ok(array_buffer_ptr.handle()),
                                Err(e) => Err(e),
                            }
                        }
                    },
                    _ => Err(Error::NotSupported),
                };
                match text {
                    Ok(text) => promise.resolve_native(&*text),
                    Err(e) => promise.reject_error(e),
                }
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
        let normalized_algorithm = normalize_algorithm(cx, algorithm, "decrypt");
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let data = match data {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        let (task_source, canceller) = self.task_source_with_canceller();
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let trusted_key = Trusted::new(key);
        let alg = normalized_algorithm.clone();
        let key_alg = key.algorithm();
        let valid_usage = key.usages().contains(&KeyUsage::Decrypt);
        let _ = task_source.queue_with_canceller(
            task!(decrypt: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = trusted_key.root();
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());
                let text = match alg {
                    Ok(NormalizedAlgorithm::AesCbcParams(key_gen_params)) => {
                        if !valid_usage || key_gen_params.name != key_alg {
                            Err(Error::InvalidAccess)
                        } else {
                            match subtle.decrypt_aes_cbc(
                                key_gen_params, &key, &data, cx, array_buffer_ptr.handle_mut()
                            ) {
                                Ok(_) => Ok(array_buffer_ptr.handle()),
                                Err(e) => Err(e),
                            }
                        }
                    },
                    Ok(NormalizedAlgorithm::AesCtrParams(key_gen_params)) => {
                        if !valid_usage || key_gen_params.name != key_alg {
                            Err(Error::InvalidAccess)
                        } else {
                            match subtle.encrypt_decrypt_aes_ctr(
                                key_gen_params, &key, &data, cx, array_buffer_ptr.handle_mut()
                            ) {
                                Ok(_) => Ok(array_buffer_ptr.handle()),
                                Err(e) => Err(e),
                            }
                        }
                    },
                    _ => Err(Error::NotSupported),
                };
                match text {
                    Ok(text) => promise.resolve_native(&*text),
                    Err(e) => promise.reject_error(e),
                }
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
        let normalized_algorithm = normalize_algorithm(cx, algorithm, "generateKey");
        let promise = Promise::new_in_current_realm(comp, can_gc);
        if let Err(e) = normalized_algorithm {
            promise.reject_error(e);
            return promise;
        }

        let (task_source, canceller) = self.task_source_with_canceller();
        let this = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let alg = normalized_algorithm.clone();
        let _ = task_source.queue_with_canceller(
            task!(generate_key: move || {
                let subtle = this.root();
                let promise = trusted_promise.root();
                let key = match alg {
                    Ok(NormalizedAlgorithm::AesKeyGenParams(key_gen_params)) => {
                        subtle.generate_key_aes(key_usages, key_gen_params, extractable)
                    },
                    _ => Err(Error::NotSupported),
                };
                match key {
                    Ok(key) => promise.resolve_native(&key),
                    Err(e) => promise.reject_error(e),
                }
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
        let normalized_algorithm = normalize_algorithm(cx, algorithm, "importKey");
        let promise = Promise::new_in_current_realm(comp, can_gc);
        if let Err(e) = normalized_algorithm {
            promise.reject_error(e);
            return promise;
        }

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
                let alg = match normalized_algorithm {
                    Ok(NormalizedAlgorithm::Algorithm(name)) => name,
                    _ => {
                        promise.reject_error(Error::NotSupported);
                        return;
                    },
                };

                let imported_key = match alg.name.as_str() {
                    ALG_AES_CBC => subtle.import_key_aes(format, &data, extractable, key_usages, ALG_AES_CBC),
                    ALG_AES_CTR => subtle.import_key_aes(format, &data, extractable, key_usages, ALG_AES_CTR),
                    _ => Err(Error::NotSupported),
                };
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

#[derive(Clone)]
pub enum NormalizedAlgorithm {
    #[allow(dead_code)]
    Algorithm(SubtleAlgorithm),
    AesCbcParams(SubtleAesCbcParams),
    AesCtrParams(SubtleAesCtrParams),
    AesKeyGenParams(SubtleAesKeyGenParams),
}

// These "subtle" structs are proxies for the codegen'd dicts which don't hold a DOMString
// so they can be sent safely when running steps in parallel.

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
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

/// <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm>
#[allow(unsafe_code)]
fn normalize_algorithm(
    cx: JSContext,
    algorithm: AlgorithmIdentifier,
    operation: &str,
) -> Result<NormalizedAlgorithm, Error> {
    match algorithm {
        AlgorithmIdentifier::String(name) => Ok(NormalizedAlgorithm::Algorithm(name.into())),
        AlgorithmIdentifier::Object(obj) => {
            rooted!(in(*cx) let value = ObjectValue(unsafe { *obj.get_unsafe() }));
            let Ok(ConversionResult::Success(algorithm)) = Algorithm::new(cx, value.handle())
            else {
                return Err(Error::Syntax);
            };
            match (algorithm.name.str().to_uppercase().as_str(), operation) {
                (ALG_AES_CBC, "encrypt") | (ALG_AES_CBC, "decrypt") => {
                    let params_result =
                        AesCbcParams::new(cx, value.handle()).map_err(|_| Error::Operation)?;
                    let ConversionResult::Success(params) = params_result else {
                        return Err(Error::Syntax);
                    };
                    Ok(NormalizedAlgorithm::AesCbcParams(params.into()))
                },
                (ALG_AES_CTR, "encrypt") | (ALG_AES_CTR, "decrypt") => {
                    let params_result =
                        AesCtrParams::new(cx, value.handle()).map_err(|_| Error::Operation)?;
                    let ConversionResult::Success(params) = params_result else {
                        return Err(Error::Syntax);
                    };
                    Ok(NormalizedAlgorithm::AesCtrParams(params.into()))
                },
                (ALG_AES_CBC, "generateKey") | (ALG_AES_CTR, "generateKey") => {
                    let params_result =
                        AesKeyGenParams::new(cx, value.handle()).map_err(|_| Error::Operation)?;
                    let ConversionResult::Success(params) = params_result else {
                        return Err(Error::Syntax);
                    };
                    Ok(NormalizedAlgorithm::AesKeyGenParams(params.into()))
                },
                (ALG_AES_CBC, "importKey") => Ok(NormalizedAlgorithm::Algorithm(SubtleAlgorithm {
                    name: ALG_AES_CBC.to_string(),
                })),
                (ALG_AES_CTR, "importKey") => Ok(NormalizedAlgorithm::Algorithm(SubtleAlgorithm {
                    name: ALG_AES_CTR.to_string(),
                })),
                _ => Err(Error::NotSupported),
            }
        },
    }
}

impl SubtleCrypto {
    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    fn encrypt_aes_cbc(
        &self,
        params: SubtleAesCbcParams,
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
        };

        create_buffer_source::<ArrayBufferU8>(cx, &ct, handle)
            .expect("failed to create buffer source for exported key.");

        Ok(())
    }

    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    fn decrypt_aes_cbc(
        &self,
        params: SubtleAesCbcParams,
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
        };

        create_buffer_source::<ArrayBufferU8>(cx, plaintext, handle)
            .expect("failed to create buffer source for exported key.");

        Ok(())
    }

    /// <https://w3c.github.io/webcrypto/#aes-ctr-operations>
    fn encrypt_decrypt_aes_ctr(
        &self,
        params: SubtleAesCtrParams,
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
        };

        create_buffer_source::<ArrayBufferU8>(cx, &ciphertext, handle)
            .expect("failed to create buffer source for exported key.");

        Ok(())
    }

    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    /// <https://w3c.github.io/webcrypto/#aes-ctr-operations>
    fn generate_key_aes(
        &self,
        usages: Vec<KeyUsage>,
        key_gen_params: SubtleAesKeyGenParams,
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

        Ok(CryptoKey::new(
            &self.global(),
            KeyType::Secret,
            extractable,
            KeyAlgorithm { name },
            usages,
            handle,
        ))
    }

    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    /// <https://w3c.github.io/webcrypto/#aes-ctr-operations>
    fn import_key_aes(
        &self,
        format: KeyFormat,
        data: &[u8],
        extractable: bool,
        usages: Vec<KeyUsage>,
        alg: &str,
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
        let name = DOMString::from(alg);
        Ok(CryptoKey::new(
            &self.global(),
            KeyType::Secret,
            extractable,
            KeyAlgorithm { name },
            usages,
            handle,
        ))
    }

    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    /// <https://w3c.github.io/webcrypto/#aes-ctr-operations>
    fn export_key_aes(&self, format: KeyFormat, key: &CryptoKey) -> Result<AesExportedKey, Error> {
        match format {
            KeyFormat::Raw => match key.handle() {
                Handle::Aes128(key_data) => Ok(AesExportedKey::Raw(key_data.as_slice().to_vec())),
                Handle::Aes192(key_data) => Ok(AesExportedKey::Raw(key_data.as_slice().to_vec())),
                Handle::Aes256(key_data) => Ok(AesExportedKey::Raw(key_data.as_slice().to_vec())),
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
