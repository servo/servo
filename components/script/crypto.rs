/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aes::{Aes128, Aes192, Aes256};
use aes_gcm::AesGcm;
use aes_gcm::aead::consts::{U12, U16, U32};
use aws_lc_rs::{digest, hmac};
use base64::Engine;
use js::conversions::ConversionResult;
use js::gc::MutableHandleObject;
use js::jsval::ObjectValue;
use params::{
    SubtleAesCbcParams, SubtleAesCtrParams, SubtleAesGcmParams, SubtleAesKeyGenParams,
    SubtleHkdfParams, SubtleHmacImportParams, SubtleHmacKeyGenParams, SubtlePbkdf2Params,
};
use script_bindings::codegen::GenericBindings::CryptoKeyBinding::{CryptoKeyMethods, KeyUsage};
use script_bindings::codegen::GenericBindings::SubtleCryptoBinding::KeyFormat;
use script_bindings::error::Error;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::{CanGc, JSContext};
use script_bindings::str::DOMString;

use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{
    AesCbcParams, AesCtrParams, AesDerivedKeyParams, AesGcmParams, AesKeyAlgorithm,
    AesKeyGenParams, Algorithm, AlgorithmIdentifier, HkdfParams, HmacImportParams,
    HmacKeyAlgorithm, HmacKeyGenParams, KeyAlgorithm, Pbkdf2Params,
};
use crate::dom::cryptokey::CryptoKey;
use crate::dom::subtlecrypto::SubtleCrypto;

pub(crate) mod params;

// String constants for algorithms/curves
pub(crate) const ALG_AES_CBC: &str = "AES-CBC";
pub(crate) const ALG_AES_CTR: &str = "AES-CTR";
pub(crate) const ALG_AES_GCM: &str = "AES-GCM";
pub(crate) const ALG_AES_KW: &str = "AES-KW";
pub(crate) const ALG_SHA1: &str = "SHA-1";
pub(crate) const ALG_SHA256: &str = "SHA-256";
pub(crate) const ALG_SHA384: &str = "SHA-384";
pub(crate) const ALG_SHA512: &str = "SHA-512";
pub(crate) const ALG_HMAC: &str = "HMAC";
pub(crate) const ALG_HKDF: &str = "HKDF";
pub(crate) const ALG_PBKDF2: &str = "PBKDF2";
pub(crate) const ALG_RSASSA_PKCS1: &str = "RSASSA-PKCS1-v1_5";
pub(crate) const ALG_RSA_OAEP: &str = "RSA-OAEP";
pub(crate) const ALG_RSA_PSS: &str = "RSA-PSS";
pub(crate) const ALG_ECDH: &str = "ECDH";
pub(crate) const ALG_ECDSA: &str = "ECDSA";

#[allow(dead_code)]
pub(crate) static SUPPORTED_ALGORITHMS: &[&str] = &[
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

pub(crate) const NAMED_CURVE_P256: &str = "P-256";
pub(crate) const NAMED_CURVE_P384: &str = "P-384";
pub(crate) const NAMED_CURVE_P521: &str = "P-521";
#[allow(dead_code)]
pub(crate) static SUPPORTED_CURVES: &[&str] =
    &[NAMED_CURVE_P256, NAMED_CURVE_P384, NAMED_CURVE_P521];

pub(crate) type Aes128CbcEnc = cbc::Encryptor<Aes128>;
pub(crate) type Aes128CbcDec = cbc::Decryptor<Aes128>;
pub(crate) type Aes192CbcEnc = cbc::Encryptor<Aes192>;
pub(crate) type Aes192CbcDec = cbc::Decryptor<Aes192>;
pub(crate) type Aes256CbcEnc = cbc::Encryptor<Aes256>;
pub(crate) type Aes256CbcDec = cbc::Decryptor<Aes256>;
pub(crate) type Aes128Ctr = ctr::Ctr64BE<Aes128>;
pub(crate) type Aes192Ctr = ctr::Ctr64BE<Aes192>;
pub(crate) type Aes256Ctr = ctr::Ctr64BE<Aes256>;

pub(crate) type Aes128Gcm96Iv = AesGcm<Aes128, U12>;
pub(crate) type Aes128Gcm128Iv = AesGcm<Aes128, U16>;
pub(crate) type Aes192Gcm96Iv = AesGcm<Aes192, U12>;
pub(crate) type Aes256Gcm96Iv = AesGcm<Aes256, U12>;
pub(crate) type Aes128Gcm256Iv = AesGcm<Aes128, U32>;
pub(crate) type Aes192Gcm256Iv = AesGcm<Aes192, U32>;
pub(crate) type Aes256Gcm256Iv = AesGcm<Aes256, U32>;

// These "subtle" structs are proxies for the codegen'd dicts which don't hold a DOMString
// so they can be sent safely when running steps in parallel.

#[allow(dead_code)]
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

pub(crate) enum GetKeyLengthAlgorithm {
    Aes(u16),
    Hmac(SubtleHmacImportParams),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum DigestAlgorithm {
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
pub(crate) enum ImportKeyAlgorithm {
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
pub(crate) enum DeriveBitsAlgorithm {
    Pbkdf2(SubtlePbkdf2Params),
    Hkdf(SubtleHkdfParams),
}

/// A normalized algorithm returned by [`normalize_algorithm`] with operation `"encrypt"` or `"decrypt"`
///
/// [`normalize_algorithm`]: https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm
#[allow(clippy::enum_variant_names)]
pub(crate) enum EncryptionAlgorithm {
    AesCbc(SubtleAesCbcParams),
    AesCtr(SubtleAesCtrParams),
    AesGcm(SubtleAesGcmParams),
}

/// A normalized algorithm returned by [`normalize_algorithm`] with operation `"sign"` or `"verify"`
///
/// [`normalize_algorithm`]: https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm
pub(crate) enum SignatureAlgorithm {
    Hmac,
}

/// A normalized algorithm returned by [`normalize_algorithm`] with operation `"generateKey"`
///
/// [`normalize_algorithm`]: https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm
pub(crate) enum KeyGenerationAlgorithm {
    Aes(SubtleAesKeyGenParams),
    Hmac(SubtleHmacKeyGenParams),
}

/// A normalized algorithm returned by [`normalize_algorithm`] with operation `"wrapKey"` or `"unwrapKey"`
///
/// [`normalize_algorithm`]: https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm
#[allow(clippy::enum_variant_names)]
pub(crate) enum KeyWrapAlgorithm {
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

pub(crate) trait AlgorithmFromName {
    fn from_name(name: DOMString, out: MutableHandleObject, cx: JSContext);
}

impl AlgorithmFromName for KeyAlgorithm {
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

pub(crate) trait AlgorithmFromLengthAndHash {
    fn from_length_and_hash(
        length: u32,
        hash: DigestAlgorithm,
        out: MutableHandleObject,
        cx: JSContext,
    );
}

impl AlgorithmFromLengthAndHash for HmacKeyAlgorithm {
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

pub(crate) trait AlgorithmFromNameAndSize {
    fn from_name_and_size(name: DOMString, size: u16, out: MutableHandleObject, cx: JSContext);
}

impl AlgorithmFromNameAndSize for AesKeyAlgorithm {
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

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations>
pub(crate) fn get_key_length_for_aes(length: u16) -> Result<u32, Error> {
    // Step 1. If the length member of normalizedDerivedKeyAlgorithm is not 128, 192 or 256,
    // then throw an OperationError.
    if !matches!(length, 128 | 192 | 256) {
        return Err(Error::Operation);
    }

    // Step 2. Return the length member of normalizedDerivedKeyAlgorithm.
    Ok(length as u32)
}

impl GetKeyLengthAlgorithm {
    pub(crate) fn get_key_length(&self) -> Result<u32, Error> {
        match self {
            Self::Aes(length) => get_key_length_for_aes(*length),
            Self::Hmac(params) => params.get_key_length(),
        }
    }
}

impl DigestAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    pub(crate) fn name(&self) -> DOMString {
        match self {
            Self::Sha1 => ALG_SHA1,
            Self::Sha256 => ALG_SHA256,
            Self::Sha384 => ALG_SHA384,
            Self::Sha512 => ALG_SHA512,
        }
        .into()
    }

    pub(crate) fn digest(&self, data: &[u8]) -> Result<impl AsRef<[u8]>, Error> {
        let algorithm = match self {
            Self::Sha1 => &digest::SHA1_FOR_LEGACY_USE_ONLY,
            Self::Sha256 => &digest::SHA256,
            Self::Sha384 => &digest::SHA384,
            Self::Sha512 => &digest::SHA512,
        };
        Ok(digest::digest(algorithm, data))
    }

    pub(crate) fn block_size_in_bits(&self) -> usize {
        match self {
            Self::Sha1 => 160,
            Self::Sha256 => 256,
            Self::Sha384 => 384,
            Self::Sha512 => 512,
        }
    }
}

impl ImportKeyAlgorithm {
    pub(crate) fn import_key(
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
    pub(crate) fn derive_bits(
        &self,
        key: &CryptoKey,
        length: Option<u32>,
    ) -> Result<Vec<u8>, Error> {
        match self {
            Self::Pbkdf2(pbkdf2_params) => pbkdf2_params.derive_bits(key, length),
            Self::Hkdf(hkdf_params) => hkdf_params.derive_bits(key, length),
        }
    }
}

impl EncryptionAlgorithm {
    /// <https://w3c.github.io/webcrypto/#dom-algorithm-name>
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::AesCbc(params) => &params.name,
            Self::AesCtr(params) => &params.name,
            Self::AesGcm(params) => &params.name,
        }
    }

    // FIXME: This doesn't really need the "SubtleCrypto" argument
    pub(crate) fn encrypt(
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
    pub(crate) fn decrypt(
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
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Hmac => ALG_HMAC,
        }
    }

    pub(crate) fn sign(
        &self,
        cx: JSContext,
        key: &CryptoKey,
        data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        match self {
            Self::Hmac => sign_hmac(cx, key, data).map(|s| s.as_ref().to_vec()),
        }
    }

    pub(crate) fn verify(
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
    pub(crate) fn generate_key(
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
pub(crate) fn sign_hmac(
    cx: JSContext,
    key: &CryptoKey,
    data: &[u8],
) -> Result<impl AsRef<[u8]>, Error> {
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
pub(crate) fn verify_hmac(
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
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::AesKw => ALG_AES_KW,
            Self::AesCbc(key_gen_params) => &key_gen_params.name,
            Self::AesCtr(key_gen_params) => &key_gen_params.name,
            Self::AesGcm(_) => ALG_AES_GCM,
        }
    }
}

/// <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm> with operation `"get key length"`
pub(crate) fn normalize_algorithm_for_get_key_length(
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
pub(crate) fn normalize_algorithm_for_digest(
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
pub(crate) fn normalize_algorithm_for_import_key(
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
pub(crate) fn normalize_algorithm_for_derive_bits(
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
pub(crate) fn normalize_algorithm_for_encrypt_or_decrypt(
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
pub(crate) fn normalize_algorithm_for_sign_or_verify(
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
pub(crate) fn normalize_algorithm_for_generate_key(
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
pub(crate) fn normalize_algorithm_for_key_wrap(
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

/// <https://w3c.github.io/webcrypto/#concept-parse-a-jwk>
pub(crate) fn parse_jwk(
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
