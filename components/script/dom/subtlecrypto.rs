/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::conversions::ConversionResult;
use js::jsapi::JSObject;
use js::jsval::ObjectValue;
use js::typedarray::ArrayBufferU8;
use servo_rand::{RngCore, ServoRng};

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{
    AesKeyGenParams, Algorithm, AlgorithmIdentifier, KeyAlgorithm, KeyFormat, SubtleCryptoMethods,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::realms::InRealm;
use crate::script_runtime::JSContext;
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
    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-generateKey>
    fn GenerateKey(
        &self,
        cx: JSContext,
        algorithm: AlgorithmIdentifier,
        extractable: bool,
        key_usages: Vec<KeyUsage>,
        comp: InRealm,
    ) -> Rc<Promise> {
        let normalized_algorithm = normalize_algorithm(cx, algorithm, "generateKey");
        let promise = Promise::new_in_current_realm(comp);
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
                        subtle.generate_key_aes_cbc(key_usages, key_gen_params, extractable)
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

    /// <https://w3c.github.io/webcrypto/#SubtleCrypto-method-exportKey>
    #[allow(unsafe_code)]
    fn ExportKey(&self, format: KeyFormat, key: &CryptoKey, comp: InRealm) -> Rc<Promise> {
        let promise = Promise::new_in_current_realm(comp);

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
                    ALG_AES_CBC => subtle.export_key_aes_cbc(format, &key),
                    _ => Err(Error::NotSupported),
                };
                match exported_key {
                    Ok(k) => {
                        let cx = GlobalScope::get_cx();
                        rooted!(in(*cx) let mut array_buffer_ptr = ptr::null_mut::<JSObject>());
                        create_buffer_source::<ArrayBufferU8>(cx, &k, array_buffer_ptr.handle_mut())
                            .expect("failed to create buffer source for exported key.");
                        promise.resolve_native(&array_buffer_ptr.get())
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
pub struct SubtleAesKeyGenParams {
    #[allow(dead_code)]
    pub name: String,
    pub length: u16,
}

impl From<AesKeyGenParams> for SubtleAesKeyGenParams {
    fn from(params: AesKeyGenParams) -> Self {
        SubtleAesKeyGenParams {
            name: params.parent.name.to_string(),
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
                (ALG_AES_CBC, "generateKey") => {
                    let params_result =
                        AesKeyGenParams::new(cx, value.handle()).map_err(|_| Error::Operation)?;
                    let ConversionResult::Success(params) = params_result else {
                        return Err(Error::Syntax);
                    };
                    Ok(NormalizedAlgorithm::AesKeyGenParams(params.into()))
                },
                _ => Err(Error::NotSupported),
            }
        },
    }
}

impl SubtleCrypto {
    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    fn generate_key_aes_cbc(
        &self,
        usages: Vec<KeyUsage>,
        key_gen_params: SubtleAesKeyGenParams,
        extractable: bool,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        if !matches!(key_gen_params.length, 128 | 192 | 256) {
            return Err(Error::Operation);
        }

        if usages.iter().any(|usage| {
            !matches!(
                usage,
                KeyUsage::Encrypt | KeyUsage::Decrypt | KeyUsage::WrapKey | KeyUsage::UnwrapKey
            )
        }) || usages.is_empty()
        {
            return Err(Error::Syntax);
        }

        let mut rand = vec![0; key_gen_params.length as usize];
        self.rng.borrow_mut().fill_bytes(&mut rand);
        let handle = match key_gen_params.length {
            128 => Handle::Aes128(rand),
            192 => Handle::Aes192(rand),
            256 => Handle::Aes256(rand),
            _ => return Err(Error::Operation),
        };

        Ok(CryptoKey::new(
            &self.global(),
            KeyType::Secret,
            extractable,
            KeyAlgorithm {
                name: DOMString::from(ALG_AES_CBC),
            },
            usages,
            handle,
        ))
    }

    /// <https://w3c.github.io/webcrypto/#aes-cbc-operations>
    fn export_key_aes_cbc(&self, format: KeyFormat, key: &CryptoKey) -> Result<Vec<u8>, Error> {
        match format {
            KeyFormat::Raw => match key.handle() {
                Handle::Aes128(key) => Ok(key.as_slice().to_vec()),
                Handle::Aes192(key) => Ok(key.as_slice().to_vec()),
                Handle::Aes256(key) => Ok(key.as_slice().to_vec()),
            },
            KeyFormat::Jwk => {
                // TODO: Support jwk
                Err(Error::NotSupported)
            },
            _ => Err(Error::NotSupported),
        }
    }
}
