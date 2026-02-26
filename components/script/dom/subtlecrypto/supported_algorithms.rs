/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! <https://w3c.github.io/webcrypto/#dfn-supportedAlgorithms>
//!
//! This module implements the internal object
//! [supportedAlgorithms](https://w3c.github.io/webcrypto/#dfn-supportedAlgorithms) for algorithm
//! registration.
//!
//! For each operation v in the list of [supported
//! operations](https://w3c.github.io/webcrypto/#supported-operation), we define a struct to
//! represent it, which acts a key of the internal object supportedAlgorithms.
//!
//! We then implement the [`Operation`] trait for these structs. When implementing the trait for
//! each of these strcuts, we set the associated type [`RegisteredAlgorithm`] of [`Operation`] to
//! an enum as the value of the operation v in supportedAlgorithms. The enum lists all algorithhms
//! supporting the operation v as its variants.
//!
//! To [define an algorithm](https://w3c.github.io/webcrypto/#concept-define-an-algorithm), each
//! variant in the enum has an inner type corresponding to the desired input IDL dictionary type
//! for the supported algorithm represented by the variant. Moreover, the enum also need to
//! implement the [`NormalizedAlgorithm`] trait since it is used as the output of
//! [`super::normalize_algorithm`].
//!
//! For example, we define the [`EncryptOperation`] struct to represent the "encrypt" operation,
//! and implement the [`Operation`] trait for it. The associated type [`RegisteredAlgorithm`] of
//! [`Operation`]  is set to the [`EncryptAlgorithm`] enum, whose variants are cryptographic
//! algorithms that support the "encrypt" operation. The variant [`EncryptAlgorithm::AesCtr`] has
//! an inner type [`SubtleAesCtrParams`] since the desired input IDL dictionary type for "encrypt"
//! operation of AES-CTR algorithm is the `AesCtrParams` dictionary. The [`EncryptAlgorithm`] enum
//! also implements the [`NormalizedAlgorithm`] trait accordingly.

use js::context::JSContext;
use js::rust::HandleValue;
use script_bindings::codegen::GenericBindings::CryptoKeyBinding::KeyUsage;
use script_bindings::codegen::GenericBindings::SubtleCryptoBinding::KeyFormat;
use script_bindings::root::DomRoot;

use crate::dom::cryptokey::CryptoKeyOrCryptoKeyPair;
use crate::dom::subtlecrypto::{CryptoAlgorithm, ExportedKey, SubtleAeadParams, SubtleAesCbcParams, SubtleAesCtrParams, SubtleAesDerivedKeyParams, SubtleAesGcmParams, SubtleAesKeyGenParams, SubtleAlgorithm, SubtleArgon2Params, SubtleCShakeParams, SubtleContextParams, SubtleEcKeyGenParams, SubtleEcKeyImportParams, SubtleEcdhKeyDeriveParams, SubtleEcdsaParams, SubtleEncapsulatedBits, SubtleHkdfParams, SubtleHmacImportParams, SubtleHmacKeyGenParams, SubtlePbkdf2Params, SubtleRsaHashedImportParams, SubtleRsaHashedKeyGenParams, SubtleRsaOaepParams, SubtleRsaPssParams, TryIntoWithCx};
use crate::dom::bindings::error::{Fallible, Error};
use crate::dom::types::{CryptoKey, GlobalScope};

use crate::dom::subtlecrypto::aes_cbc_operation;
use crate::dom::subtlecrypto::aes_ctr_operation;
use crate::dom::subtlecrypto::aes_gcm_operation;
use crate::dom::subtlecrypto::aes_kw_operation;
use crate::dom::subtlecrypto::aes_ocb_operation;
use crate::dom::subtlecrypto::argon2_operation;
use crate::dom::subtlecrypto::chacha20_poly1305_operation;
use crate::dom::subtlecrypto::cshake_operation;
use crate::dom::subtlecrypto::ecdh_operation;
use crate::dom::subtlecrypto::ecdsa_operation;
use crate::dom::subtlecrypto::ed25519_operation;
use crate::dom::subtlecrypto::hkdf_operation;
use crate::dom::subtlecrypto::hmac_operation;
use crate::dom::subtlecrypto::ml_dsa_operation;
use crate::dom::subtlecrypto::ml_kem_operation;
use crate::dom::subtlecrypto::pbkdf2_operation;
use crate::dom::subtlecrypto::rsa_oaep_operation;
use crate::dom::subtlecrypto::rsa_pss_operation;
use crate::dom::subtlecrypto::rsassa_pkcs1_v1_5_operation;
use crate::dom::subtlecrypto::sha3_operation;
use crate::dom::subtlecrypto::sha_operation;
use crate::dom::subtlecrypto::x25519_operation;

pub(crate) trait Operation {
    type RegisteredAlgorithm: NormalizedAlgorithm;
}

pub(crate) trait NormalizedAlgorithm: Sized {
    /// Step 4 - 10 of <https://w3c.github.io/webcrypto/#algorithm-normalization-normalize-an-algorithm>
    fn from_object_value(
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self>;
    fn name(&self) -> &str;
}

// The value of the key "encrypt" in the internal object supportedAlgorithms
pub(crate) struct EncryptOperation {}

impl Operation for EncryptOperation {
    type RegisteredAlgorithm = EncryptAlgorithm;
}

/// Normalized algorithm for the "encrypt" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
pub(crate) enum EncryptAlgorithm {
    RsaOaep(SubtleRsaOaepParams),
    AesCtr(SubtleAesCtrParams),
    AesCbc(SubtleAesCbcParams),
    AesGcm(SubtleAesGcmParams),
    AesOcb(SubtleAeadParams),
    ChaCha20Poly1305(SubtleAeadParams),
}

impl NormalizedAlgorithm for EncryptAlgorithm {
    fn from_object_value(
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::RsaOaep => Ok(EncryptAlgorithm::RsaOaep(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCtr => Ok(EncryptAlgorithm::AesCtr(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCbc => Ok(EncryptAlgorithm::AesCbc(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesGcm => Ok(EncryptAlgorithm::AesGcm(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesOcb => Ok(EncryptAlgorithm::AesOcb(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::ChaCha20Poly1305 => Ok(EncryptAlgorithm::ChaCha20Poly1305(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"encrypt\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            EncryptAlgorithm::RsaOaep(algo) => &algo.name,
            EncryptAlgorithm::AesCtr(algo) => &algo.name,
            EncryptAlgorithm::AesCbc(algo) => &algo.name,
            EncryptAlgorithm::AesGcm(algo) => &algo.name,
            EncryptAlgorithm::AesOcb(algo) => &algo.name,
            EncryptAlgorithm::ChaCha20Poly1305(algo) => &algo.name,
        }
    }
}

impl EncryptAlgorithm {
    pub(crate) fn encrypt(&self, key: &CryptoKey, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            EncryptAlgorithm::RsaOaep(algo) => {
                rsa_oaep_operation::encrypt(algo, key, plaintext)
            },
            EncryptAlgorithm::AesCtr(algo) => {
                aes_ctr_operation::encrypt(algo, key, plaintext)
            },
            EncryptAlgorithm::AesCbc(algo) => {
                aes_cbc_operation::encrypt(algo, key, plaintext)
            },
            EncryptAlgorithm::AesGcm(algo) => {
                aes_gcm_operation::encrypt(algo, key, plaintext)
            },
            EncryptAlgorithm::AesOcb(algo) => {
                aes_ocb_operation::encrypt(algo, key, plaintext)
            },
            EncryptAlgorithm::ChaCha20Poly1305(algo) => {
                chacha20_poly1305_operation::encrypt(algo, key, plaintext)
            },
        }
    }
}

// The value of the key "decrypt" in the internal object supportedAlgorithms
pub(crate) struct DecryptOperation {}

impl Operation for DecryptOperation {
    type RegisteredAlgorithm = DecryptAlgorithm;
}

/// Normalized algorithm for the "decrypt" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
pub(crate) enum DecryptAlgorithm {
    RsaOaep(SubtleRsaOaepParams),
    AesCtr(SubtleAesCtrParams),
    AesCbc(SubtleAesCbcParams),
    AesGcm(SubtleAesGcmParams),
    AesOcb(SubtleAeadParams),
    ChaCha20Poly1305(SubtleAeadParams),
}

impl NormalizedAlgorithm for DecryptAlgorithm {
    fn from_object_value(
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::RsaOaep => Ok(DecryptAlgorithm::RsaOaep(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCtr => Ok(DecryptAlgorithm::AesCtr(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCbc => Ok(DecryptAlgorithm::AesCbc(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesGcm => Ok(DecryptAlgorithm::AesGcm(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesOcb => Ok(DecryptAlgorithm::AesOcb(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::ChaCha20Poly1305 => Ok(DecryptAlgorithm::ChaCha20Poly1305(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"decrypt\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            DecryptAlgorithm::RsaOaep(algo) => &algo.name,
            DecryptAlgorithm::AesCtr(algo) => &algo.name,
            DecryptAlgorithm::AesCbc(algo) => &algo.name,
            DecryptAlgorithm::AesGcm(algo) => &algo.name,
            DecryptAlgorithm::AesOcb(algo) => &algo.name,
            DecryptAlgorithm::ChaCha20Poly1305(algo) => &algo.name,
        }
    }
}

impl DecryptAlgorithm {
    pub(crate) fn decrypt(&self, key: &CryptoKey, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            DecryptAlgorithm::RsaOaep(algo) => {
                rsa_oaep_operation::decrypt(algo, key, ciphertext)
            },
            DecryptAlgorithm::AesCtr(algo) => {
                aes_ctr_operation::decrypt(algo, key, ciphertext)
            },
            DecryptAlgorithm::AesCbc(algo) => {
                aes_cbc_operation::decrypt(algo, key, ciphertext)
            },
            DecryptAlgorithm::AesGcm(algo) => {
                aes_gcm_operation::decrypt(algo, key, ciphertext)
            },
            DecryptAlgorithm::AesOcb(algo) => {
                aes_ocb_operation::decrypt(algo, key, ciphertext)
            },
            DecryptAlgorithm::ChaCha20Poly1305(algo) => {
                chacha20_poly1305_operation::decrypt(algo, key, ciphertext)
            },
        }
    }
}

// The value of the key "sign" in the internal object supportedAlgorithms
pub(crate) struct SignOperation {}

impl Operation for SignOperation {
    type RegisteredAlgorithm = SignAlgorithm;
}

/// Normalized algorithm for the "sign" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
pub(crate) enum SignAlgorithm {
    RsassaPkcs1V1_5(SubtleAlgorithm),
    RsaPss(SubtleRsaPssParams),
    Ecdsa(SubtleEcdsaParams),
    Ed25519(SubtleAlgorithm),
    Hmac(SubtleAlgorithm),
    MlDsa(SubtleContextParams),
}

impl NormalizedAlgorithm for SignAlgorithm {
    fn from_object_value(
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::RsassaPkcs1V1_5 => Ok(SignAlgorithm::RsassaPkcs1V1_5(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::RsaPss => Ok(SignAlgorithm::RsaPss(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ecdsa => Ok(SignAlgorithm::Ecdsa(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ed25519 => Ok(SignAlgorithm::Ed25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hmac => Ok(SignAlgorithm::Hmac(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::MlDsa44 | CryptoAlgorithm::MlDsa65 | CryptoAlgorithm::MlDsa87 => Ok(SignAlgorithm::MlDsa(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"sign\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            SignAlgorithm::RsassaPkcs1V1_5(algo) => &algo.name,
            SignAlgorithm::RsaPss(algo) => &algo.name,
            SignAlgorithm::Ecdsa(algo) => &algo.name,
            SignAlgorithm::Ed25519(algo) => &algo.name,
            SignAlgorithm::Hmac(algo) => &algo.name,
            SignAlgorithm::MlDsa(algo) => &algo.name,
        }
    }
}

impl SignAlgorithm {
    pub(crate) fn sign(&self, key: &CryptoKey, message: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            SignAlgorithm::RsassaPkcs1V1_5(_algo) => {
                rsassa_pkcs1_v1_5_operation::sign(key, message)
            },
            SignAlgorithm::RsaPss(algo) => {
                rsa_pss_operation::sign(algo, key, message)
            },
            SignAlgorithm::Ecdsa(algo) => {
                ecdsa_operation::sign(algo, key, message)
            },
            SignAlgorithm::Ed25519(_algo) => {
                ed25519_operation::sign(key, message)
            },
            SignAlgorithm::Hmac(_algo) => {
                hmac_operation::sign(key, message)
            },
            SignAlgorithm::MlDsa(algo) => {
                ml_dsa_operation::sign(algo, key, message)
            },
        }
    }
}

// The value of the key "verify" in the internal object supportedAlgorithms
pub(crate) struct VerifyOperation {}

impl Operation for VerifyOperation {
    type RegisteredAlgorithm = VerifyAlgorithm;
}

/// Normalized algorithm for the "verify" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
pub(crate) enum VerifyAlgorithm {
    RsassaPkcs1V1_5(SubtleAlgorithm),
    RsaPss(SubtleRsaPssParams),
    Ecdsa(SubtleEcdsaParams),
    Ed25519(SubtleAlgorithm),
    Hmac(SubtleAlgorithm),
    MlDsa(SubtleContextParams),
}

impl NormalizedAlgorithm for VerifyAlgorithm {
    fn from_object_value(
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::RsassaPkcs1V1_5 => Ok(VerifyAlgorithm::RsassaPkcs1V1_5(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::RsaPss => Ok(VerifyAlgorithm::RsaPss(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ecdsa => Ok(VerifyAlgorithm::Ecdsa(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ed25519 => Ok(VerifyAlgorithm::Ed25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hmac => Ok(VerifyAlgorithm::Hmac(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::MlDsa44 | CryptoAlgorithm::MlDsa65 | CryptoAlgorithm::MlDsa87 => Ok(VerifyAlgorithm::MlDsa(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"verify\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            VerifyAlgorithm::RsassaPkcs1V1_5(algo) => &algo.name,
            VerifyAlgorithm::RsaPss(algo) => &algo.name,
            VerifyAlgorithm::Ecdsa(algo) => &algo.name,
            VerifyAlgorithm::Ed25519(algo) => &algo.name,
            VerifyAlgorithm::Hmac(algo) => &algo.name,
            VerifyAlgorithm::MlDsa(algo) => &algo.name,
        }
    }
}

impl VerifyAlgorithm {
    pub(crate) fn verify(&self, key: &CryptoKey, message: &[u8], signature: &[u8]) -> Result<bool, Error> {
        match self {
            VerifyAlgorithm::RsassaPkcs1V1_5(_algo) => {
                rsassa_pkcs1_v1_5_operation::verify(key, message, signature)
            },
            VerifyAlgorithm::RsaPss(algo) => {
                rsa_pss_operation::verify(algo, key, message, signature)
            },
            VerifyAlgorithm::Ecdsa(algo) => {
                ecdsa_operation::verify(algo, key, message, signature)
            },
            VerifyAlgorithm::Ed25519(_algo) => {
                ed25519_operation::verify(key, message, signature)
            },
            VerifyAlgorithm::Hmac(_algo) => {
                hmac_operation::verify(key, message, signature)
            },
            VerifyAlgorithm::MlDsa(algo) => {
                ml_dsa_operation::verify(algo, key, message, signature)
            },
        }
    }
}

// The value of the key "digest" in the internal object supportedAlgorithms
pub(crate) struct DigestOperation {}

impl Operation for DigestOperation {
    type RegisteredAlgorithm = DigestAlgorithm;
}

/// Normalized algorithm for the "digest" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
#[derive(Clone, MallocSizeOf)]
pub(crate) enum DigestAlgorithm {
    Sha(SubtleAlgorithm),
    Sha3(SubtleAlgorithm),
    CShake(SubtleCShakeParams),
}

impl NormalizedAlgorithm for DigestAlgorithm {
    fn from_object_value(
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::Sha1 | CryptoAlgorithm::Sha256 | CryptoAlgorithm::Sha384 | CryptoAlgorithm::Sha512 => Ok(DigestAlgorithm::Sha(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Sha3_256 | CryptoAlgorithm::Sha3_384 | CryptoAlgorithm::Sha3_512 => Ok(DigestAlgorithm::Sha3(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::CShake128 | CryptoAlgorithm::CShake256 => Ok(DigestAlgorithm::CShake(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"digest\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            DigestAlgorithm::Sha(algo) => &algo.name,
            DigestAlgorithm::Sha3(algo) => &algo.name,
            DigestAlgorithm::CShake(algo) => &algo.name,
        }
    }
}

impl DigestAlgorithm {
    pub(crate) fn digest(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            DigestAlgorithm::Sha(algo) => {
                sha_operation::digest(algo, message)
            },
            DigestAlgorithm::Sha3(algo) => {
                sha3_operation::digest(algo, message)
            },
            DigestAlgorithm::CShake(algo) => {
                cshake_operation::digest(algo, message)
            },
        }
    }
}

// The value of the key "deriveBits" in the internal object supportedAlgorithms
pub(crate) struct DeriveBitsOperation {}

impl Operation for DeriveBitsOperation {
    type RegisteredAlgorithm = DeriveBitsAlgorithm;
}

/// Normalized algorithm for the "deriveBits" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
pub(crate) enum DeriveBitsAlgorithm {
    Ecdh(SubtleEcdhKeyDeriveParams),
    X25519(SubtleEcdhKeyDeriveParams),
    Hkdf(SubtleHkdfParams),
    Pbkdf2(SubtlePbkdf2Params),
    Argon2(SubtleArgon2Params),
}

impl NormalizedAlgorithm for DeriveBitsAlgorithm {
    fn from_object_value(
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::Ecdh => Ok(DeriveBitsAlgorithm::Ecdh(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::X25519 => Ok(DeriveBitsAlgorithm::X25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hkdf => Ok(DeriveBitsAlgorithm::Hkdf(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Pbkdf2 => Ok(DeriveBitsAlgorithm::Pbkdf2(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Argon2D |
            CryptoAlgorithm::Argon2I |
            CryptoAlgorithm::Argon2ID => Ok(DeriveBitsAlgorithm::Argon2(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"deriveBits\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            DeriveBitsAlgorithm::Ecdh(algo) => &algo.name,
            DeriveBitsAlgorithm::X25519(algo) => &algo.name,
            DeriveBitsAlgorithm::Hkdf(algo) => &algo.name,
            DeriveBitsAlgorithm::Pbkdf2(algo) => &algo.name,
            DeriveBitsAlgorithm::Argon2(algo) => &algo.name,
        }
    }
}

impl DeriveBitsAlgorithm {
    pub(crate) fn derive_bits(&self, key: &CryptoKey, length: Option<u32>) -> Result<Vec<u8>, Error> {
        match self {
            DeriveBitsAlgorithm::Ecdh(algo) => {
                ecdh_operation::derive_bits(algo, key, length)
            },
            DeriveBitsAlgorithm::X25519(algo) => {
                x25519_operation::derive_bits(algo, key, length)
            },
            DeriveBitsAlgorithm::Hkdf(algo) => {
                hkdf_operation::derive_bits(algo, key, length)
            },
            DeriveBitsAlgorithm::Pbkdf2(algo) => {
                pbkdf2_operation::derive_bits(algo, key, length)
            },
            DeriveBitsAlgorithm::Argon2(algo) => {
                argon2_operation::derive_bits(algo, key, length)
            },
        }
    }
}

// The value of the key "wrapKey" in the internal object supportedAlgorithms
pub(crate) struct WrapKeyOperation {}

impl Operation for WrapKeyOperation {
    type RegisteredAlgorithm = WrapKeyAlgorithm;
}

/// Normalized algorithm for the "wrapKey" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
pub(crate) enum WrapKeyAlgorithm {
    AesKw(SubtleAlgorithm),
}

impl NormalizedAlgorithm for WrapKeyAlgorithm {
    fn from_object_value(
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::AesKw => Ok(WrapKeyAlgorithm::AesKw(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"wrapKey\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            WrapKeyAlgorithm::AesKw(algo) => &algo.name,
        }
    }
}

impl WrapKeyAlgorithm {
    pub(crate) fn wrap_key(&self, key: &CryptoKey, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            WrapKeyAlgorithm::AesKw(_algo) => {
                aes_kw_operation::wrap_key(key, plaintext)
            },
        }
    }
}

// The value of the key "unwrapKey" in the internal object supportedAlgorithms
pub(crate) struct UnwrapKeyOperation {}

impl Operation for UnwrapKeyOperation {
    type RegisteredAlgorithm = UnwrapKeyAlgorithm;
}

/// Normalized algorithm for the "unwrapKey" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
pub(crate) enum UnwrapKeyAlgorithm {
    AesKw(SubtleAlgorithm),
}

impl NormalizedAlgorithm for UnwrapKeyAlgorithm {
    fn from_object_value(
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::AesKw => Ok(UnwrapKeyAlgorithm::AesKw(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"unwrapKey\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            UnwrapKeyAlgorithm::AesKw(algo) => &algo.name,
        }
    }
}

impl UnwrapKeyAlgorithm {
    pub(crate) fn unwrap_key(&self, key: &CryptoKey, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            UnwrapKeyAlgorithm::AesKw(_algo) => {
                aes_kw_operation::unwrap_key(key, ciphertext)
            },
        }
    }
}

// The value of the key "unwrapKey" in the internal object supportedAlgorithms
pub(crate) struct GenerateKeyOperation {}

impl Operation for GenerateKeyOperation {
    type RegisteredAlgorithm = GenerateKeyAlgorithm;
}

/// Normalized algorithm for the "generateKey" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
pub(crate) enum GenerateKeyAlgorithm {
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
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::RsassaPkcs1V1_5 => Ok(GenerateKeyAlgorithm::RsassaPkcs1V1_5(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::RsaPss => Ok(GenerateKeyAlgorithm::RsaPss(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::RsaOaep => Ok(GenerateKeyAlgorithm::RsaOaep(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ecdsa => Ok(GenerateKeyAlgorithm::Ecdsa(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ecdh => Ok(GenerateKeyAlgorithm::Ecdh(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ed25519 => Ok(GenerateKeyAlgorithm::Ed25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::X25519 => Ok(GenerateKeyAlgorithm::X25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCtr => Ok(GenerateKeyAlgorithm::AesCtr(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCbc => Ok(GenerateKeyAlgorithm::AesCbc(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesGcm => Ok(GenerateKeyAlgorithm::AesGcm(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesKw => Ok(GenerateKeyAlgorithm::AesKw(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hmac => Ok(GenerateKeyAlgorithm::Hmac(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::MlKem512 | CryptoAlgorithm::MlKem768 | CryptoAlgorithm::MlKem1024 => Ok(GenerateKeyAlgorithm::MlKem(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::MlDsa44 | CryptoAlgorithm::MlDsa65 | CryptoAlgorithm::MlDsa87 => Ok(GenerateKeyAlgorithm::MlDsa(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesOcb => Ok(GenerateKeyAlgorithm::AesOcb(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::ChaCha20Poly1305 => Ok(GenerateKeyAlgorithm::ChaCha20Poly1305(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"generateKey\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            GenerateKeyAlgorithm::RsassaPkcs1V1_5(algo) => &algo.name,
            GenerateKeyAlgorithm::RsaPss(algo) => &algo.name,
            GenerateKeyAlgorithm::RsaOaep(algo) => &algo.name,
            GenerateKeyAlgorithm::Ecdsa(algo) => &algo.name,
            GenerateKeyAlgorithm::Ecdh(algo) => &algo.name,
            GenerateKeyAlgorithm::Ed25519(algo) => &algo.name,
            GenerateKeyAlgorithm::X25519(algo) => &algo.name,
            GenerateKeyAlgorithm::AesCtr(algo) => &algo.name,
            GenerateKeyAlgorithm::AesCbc(algo) => &algo.name,
            GenerateKeyAlgorithm::AesGcm(algo) => &algo.name,
            GenerateKeyAlgorithm::AesKw(algo) => &algo.name,
            GenerateKeyAlgorithm::Hmac(algo) => &algo.name,
            GenerateKeyAlgorithm::MlKem(algo) => &algo.name,
            GenerateKeyAlgorithm::MlDsa(algo) => &algo.name,
            GenerateKeyAlgorithm::AesOcb(algo) => &algo.name,
            GenerateKeyAlgorithm::ChaCha20Poly1305(algo) => &algo.name,
        }
    }
}

impl GenerateKeyAlgorithm {
    pub(crate) fn generate_key(
        &self,
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        extractable: bool,
        usages: Vec<KeyUsage>,
    ) -> Result<CryptoKeyOrCryptoKeyPair, Error> {
        match self {
            GenerateKeyAlgorithm::RsassaPkcs1V1_5(algo) => {
                rsassa_pkcs1_v1_5_operation::generate_key(cx, global, algo, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::RsaPss(algo) => {
                rsa_pss_operation::generate_key(cx, global, algo, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::RsaOaep(algo) => {
                rsa_oaep_operation::generate_key(cx, global, algo, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::Ecdsa(algo) => {
                ecdsa_operation::generate_key(cx, global, algo, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::Ecdh(algo) => {
                ecdh_operation::generate_key(cx, global, algo, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::Ed25519(_algo) => {
                ed25519_operation::generate_key(cx, global, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::X25519(_algo) => {
                x25519_operation::generate_key(cx, global, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::AesCtr(algo) => {
                aes_ctr_operation::generate_key(cx, global, algo, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            GenerateKeyAlgorithm::AesCbc(algo) => {
                aes_cbc_operation::generate_key(cx, global, algo, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            GenerateKeyAlgorithm::AesGcm(algo) => {
                aes_gcm_operation::generate_key(cx, global, algo, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            GenerateKeyAlgorithm::AesKw(algo) => {
                aes_kw_operation::generate_key(cx, global, algo, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            GenerateKeyAlgorithm::Hmac(algo) => {
                hmac_operation::generate_key(cx, global, algo, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            GenerateKeyAlgorithm::MlKem(algo) => {
                ml_kem_operation::generate_key(cx, global, algo, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::MlDsa(algo) => {
                ml_dsa_operation::generate_key(cx, global, algo, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKeyPair)
            },
            GenerateKeyAlgorithm::AesOcb(algo) => {
                aes_ocb_operation::generate_key(cx, global, algo, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
            GenerateKeyAlgorithm::ChaCha20Poly1305(_algo) => {
                chacha20_poly1305_operation::generate_key(cx, global, extractable, usages)
                    .map(CryptoKeyOrCryptoKeyPair::CryptoKey)
            },
        }
    }
}

// The value of the key "importKey" in the internal object supportedAlgorithms
pub(crate) struct ImportKeyOperation {}

impl Operation for ImportKeyOperation {
    type RegisteredAlgorithm = ImportKeyAlgorithm;
}

/// Normalized algorithm for the "importKey" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
pub(crate) enum ImportKeyAlgorithm {
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
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::RsassaPkcs1V1_5 => Ok(ImportKeyAlgorithm::RsassaPkcs1V1_5(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::RsaPss => Ok(ImportKeyAlgorithm::RsaPss(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::RsaOaep => Ok(ImportKeyAlgorithm::RsaOaep(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ecdsa => Ok(ImportKeyAlgorithm::Ecdsa(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ecdh => Ok(ImportKeyAlgorithm::Ecdh(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ed25519 => Ok(ImportKeyAlgorithm::Ed25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::X25519 => Ok(ImportKeyAlgorithm::X25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCtr => Ok(ImportKeyAlgorithm::AesCtr(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCbc => Ok(ImportKeyAlgorithm::AesCbc(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesGcm => Ok(ImportKeyAlgorithm::AesGcm(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesKw => Ok(ImportKeyAlgorithm::AesKw(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hmac => Ok(ImportKeyAlgorithm::Hmac(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hkdf => Ok(ImportKeyAlgorithm::Hkdf(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Pbkdf2 => Ok(ImportKeyAlgorithm::Pbkdf2(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::MlKem512 | CryptoAlgorithm::MlKem768 | CryptoAlgorithm::MlKem1024 => Ok(ImportKeyAlgorithm::MlKem(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::MlDsa44 | CryptoAlgorithm::MlDsa65 | CryptoAlgorithm::MlDsa87 => Ok(ImportKeyAlgorithm::MlDsa(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesOcb => Ok(ImportKeyAlgorithm::AesOcb(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::ChaCha20Poly1305 => Ok(ImportKeyAlgorithm::ChaCha20Poly1305(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Argon2D | CryptoAlgorithm::Argon2I | CryptoAlgorithm::Argon2ID => Ok(ImportKeyAlgorithm::Argon2(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"importKey\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            ImportKeyAlgorithm::RsassaPkcs1V1_5(algo) => &algo.name,
            ImportKeyAlgorithm::RsaPss(algo) => &algo.name,
            ImportKeyAlgorithm::RsaOaep(algo) => &algo.name,
            ImportKeyAlgorithm::Ecdsa(algo) => &algo.name,
            ImportKeyAlgorithm::Ecdh(algo) => &algo.name,
            ImportKeyAlgorithm::Ed25519(algo) => &algo.name,
            ImportKeyAlgorithm::X25519(algo) => &algo.name,
            ImportKeyAlgorithm::AesCtr(algo) => &algo.name,
            ImportKeyAlgorithm::AesCbc(algo) => &algo.name,
            ImportKeyAlgorithm::AesGcm(algo) => &algo.name,
            ImportKeyAlgorithm::AesKw(algo) => &algo.name,
            ImportKeyAlgorithm::Hmac(algo) => &algo.name,
            ImportKeyAlgorithm::Hkdf(algo) => &algo.name,
            ImportKeyAlgorithm::Pbkdf2(algo) => &algo.name,
            ImportKeyAlgorithm::MlKem(algo) => &algo.name,
            ImportKeyAlgorithm::MlDsa(algo) => &algo.name,
            ImportKeyAlgorithm::AesOcb(algo) => &algo.name,
            ImportKeyAlgorithm::ChaCha20Poly1305(algo) => &algo.name,
            ImportKeyAlgorithm::Argon2(algo) => &algo.name,
        }
    }
}

impl ImportKeyAlgorithm {
   pub(crate)  fn import_key(
        &self,
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        format: KeyFormat,
        key_data: &[u8],
        extractable: bool,
        usages: Vec<KeyUsage>,
    ) -> Result<DomRoot<CryptoKey>, Error> {
        match self {
            ImportKeyAlgorithm::RsassaPkcs1V1_5(algo) => {
                rsassa_pkcs1_v1_5_operation::import_key(
                    cx,
                    global,
                    algo,
                    format,
                    key_data,
                    extractable,
                    usages,
                )
            },
            ImportKeyAlgorithm::RsaPss(algo) => {
                rsa_pss_operation::import_key(
                    cx,
                    global,
                    algo,
                    format,
                    key_data,
                    extractable,
                    usages,
                )
            },
            ImportKeyAlgorithm::RsaOaep(algo) => {
                rsa_oaep_operation::import_key(
                    cx,
                    global,
                    algo,
                    format,
                    key_data,
                    extractable,
                    usages,
                )
            },
            ImportKeyAlgorithm::Ecdsa(algo) => {
                ecdsa_operation::import_key(cx, global, algo, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::Ecdh(algo) => {
                ecdh_operation::import_key(cx, global, algo, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::Ed25519(_algo) => {
                ed25519_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::X25519(_algo) => {
                x25519_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::AesCtr(_algo) => {
                aes_ctr_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::AesCbc(_algo) => {
                aes_cbc_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::AesGcm(_algo) => {
                aes_gcm_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::AesKw(_algo) => {
                aes_kw_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::Hmac(algo) => {
                hmac_operation::import_key(cx, global, algo, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::Hkdf(_algo) => {
                hkdf_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::Pbkdf2(_algo) => {
                pbkdf2_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::MlKem(algo) => {
                ml_kem_operation::import_key(
                    cx,
                    global,
                    algo,
                    format,
                    key_data,
                    extractable,
                    usages,
                )
            },
            ImportKeyAlgorithm::MlDsa(algo) => {
                ml_dsa_operation::import_key(
                    cx,
                    global,
                    algo,
                    format,
                    key_data,
                    extractable,
                    usages,
                )
            },
            ImportKeyAlgorithm::AesOcb(_algo) => {
                aes_ocb_operation::import_key(cx, global, format, key_data, extractable, usages)
            },
            ImportKeyAlgorithm::ChaCha20Poly1305(_algo) => {
                chacha20_poly1305_operation::import_key(
                    cx,
                    global,
                    format,
                    key_data,
                    extractable,
                    usages,
                )
            },
            ImportKeyAlgorithm::Argon2(algo) => {
                argon2_operation::import_key(
                    cx,
                    global,
                    algo,
                    format,
                    key_data,
                    extractable,
                    usages,
                )
            },
        }
    }
}

// The value of the key "exportKey" in the internal object supportedAlgorithms
pub(crate) struct ExportKeyOperation {}

impl Operation for ExportKeyOperation {
    type RegisteredAlgorithm = ExportKeyAlgorithm;
}

/// Normalized algorithm for the "exportKey" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
pub(crate) enum ExportKeyAlgorithm {
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
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::RsassaPkcs1V1_5 => Ok(ExportKeyAlgorithm::RsassaPkcs1V1_5(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::RsaPss => Ok(ExportKeyAlgorithm::RsaPss(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::RsaOaep => Ok(ExportKeyAlgorithm::RsaOaep(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ecdsa => Ok(ExportKeyAlgorithm::Ecdsa(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ecdh => Ok(ExportKeyAlgorithm::Ecdh(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Ed25519 => Ok(ExportKeyAlgorithm::Ed25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::X25519 => Ok(ExportKeyAlgorithm::X25519(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCtr => Ok(ExportKeyAlgorithm::AesCtr(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCbc => Ok(ExportKeyAlgorithm::AesCbc(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesGcm => Ok(ExportKeyAlgorithm::AesGcm(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesKw => Ok(ExportKeyAlgorithm::AesKw(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hmac => Ok(ExportKeyAlgorithm::Hmac(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::MlKem512 | CryptoAlgorithm::MlKem768 | CryptoAlgorithm::MlKem1024 => Ok(ExportKeyAlgorithm::MlKem(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::MlDsa44 | CryptoAlgorithm::MlDsa65 | CryptoAlgorithm::MlDsa87 => Ok(ExportKeyAlgorithm::MlDsa(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesOcb => Ok(ExportKeyAlgorithm::AesOcb(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::ChaCha20Poly1305 => Ok(ExportKeyAlgorithm::ChaCha20Poly1305(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"exportKey\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            ExportKeyAlgorithm::RsassaPkcs1V1_5(algo) => &algo.name,
            ExportKeyAlgorithm::RsaPss(algo) => &algo.name,
            ExportKeyAlgorithm::RsaOaep(algo) => &algo.name,
            ExportKeyAlgorithm::Ecdsa(algo) => &algo.name,
            ExportKeyAlgorithm::Ecdh(algo) => &algo.name,
            ExportKeyAlgorithm::Ed25519(algo) => &algo.name,
            ExportKeyAlgorithm::X25519(algo) => &algo.name,
            ExportKeyAlgorithm::AesCtr(algo) => &algo.name,
            ExportKeyAlgorithm::AesCbc(algo) => &algo.name,
            ExportKeyAlgorithm::AesGcm(algo) => &algo.name,
            ExportKeyAlgorithm::AesKw(algo) => &algo.name,
            ExportKeyAlgorithm::Hmac(algo) => &algo.name,
            ExportKeyAlgorithm::MlKem(algo) => &algo.name,
            ExportKeyAlgorithm::MlDsa(algo) => &algo.name,
            ExportKeyAlgorithm::AesOcb(algo) => &algo.name,
            ExportKeyAlgorithm::ChaCha20Poly1305(algo) => &algo.name,
        }
    }
}

impl ExportKeyAlgorithm {
    pub(crate) fn export_key(&self, format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
        match self {
            ExportKeyAlgorithm::RsassaPkcs1V1_5(_algo) => {
                rsassa_pkcs1_v1_5_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::RsaPss(_algo) => {
                rsa_pss_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::RsaOaep(_algo) => {
                rsa_oaep_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::Ecdsa(_algo) => {
                ecdsa_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::Ecdh(_algo) => {
                ecdh_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::Ed25519(_algo) => {
                ed25519_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::X25519(_algo) => {
                x25519_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::AesCtr(_algo) => {
                aes_ctr_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::AesCbc(_algo) => {
                aes_cbc_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::AesGcm(_algo) => {
                aes_gcm_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::AesKw(_algo) => {
                aes_kw_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::Hmac(_algo) => {
                hmac_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::MlKem(_algo) => {
                ml_kem_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::MlDsa(_algo) => {
                ml_dsa_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::AesOcb(_algo) => {
                aes_ocb_operation::export_key(format, key)
            },
            ExportKeyAlgorithm::ChaCha20Poly1305(_algo) => {
                chacha20_poly1305_operation::export_key(format, key)
            },
        }
    }
}

// The value of the key "get key length" in the internal object supportedAlgorithms
pub(crate) struct GetKeyLengthOperation {}

impl Operation for GetKeyLengthOperation {
    type RegisteredAlgorithm = GetKeyLengthAlgorithm;
}

/// Normalized algorithm for the "get key length" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
pub(crate) enum GetKeyLengthAlgorithm {
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
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::AesCtr => Ok(GetKeyLengthAlgorithm::AesCtr(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesCbc => Ok(GetKeyLengthAlgorithm::AesCbc(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesGcm => Ok(GetKeyLengthAlgorithm::AesGcm(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesKw => Ok(GetKeyLengthAlgorithm::AesKw(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hmac => Ok(GetKeyLengthAlgorithm::Hmac(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Hkdf => Ok(GetKeyLengthAlgorithm::Hkdf(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Pbkdf2 => Ok(GetKeyLengthAlgorithm::Pbkdf2(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::AesOcb => Ok(GetKeyLengthAlgorithm::AesOcb(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::ChaCha20Poly1305 => Ok(GetKeyLengthAlgorithm::ChaCha20Poly1305(value.try_into_with_cx(cx)?)),
            CryptoAlgorithm::Argon2D | CryptoAlgorithm::Argon2I | CryptoAlgorithm::Argon2ID => Ok(GetKeyLengthAlgorithm::Argon2(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"get key length\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            GetKeyLengthAlgorithm::AesCtr(algo) => &algo.name,
            GetKeyLengthAlgorithm::AesCbc(algo) => &algo.name,
            GetKeyLengthAlgorithm::AesGcm(algo) => &algo.name,
            GetKeyLengthAlgorithm::AesKw(algo) => &algo.name,
            GetKeyLengthAlgorithm::Hmac(algo) => &algo.name,
            GetKeyLengthAlgorithm::Hkdf(algo) => &algo.name,
            GetKeyLengthAlgorithm::Pbkdf2(algo) => &algo.name,
            GetKeyLengthAlgorithm::AesOcb(algo) => &algo.name,
            GetKeyLengthAlgorithm::ChaCha20Poly1305(algo) => &algo.name,
            GetKeyLengthAlgorithm::Argon2(algo) => &algo.name,
        }
    }
}

impl GetKeyLengthAlgorithm {
    pub(crate) fn get_key_length(&self) -> Result<Option<u32>, Error> {
        match self {
            GetKeyLengthAlgorithm::AesCtr(algo) => {
                aes_ctr_operation::get_key_length(algo)
            },
            GetKeyLengthAlgorithm::AesCbc(algo) => {
                aes_cbc_operation::get_key_length(algo)
            },
            GetKeyLengthAlgorithm::AesGcm(algo) => {
                aes_gcm_operation::get_key_length(algo)
            },
            GetKeyLengthAlgorithm::AesKw(algo) => {
                aes_kw_operation::get_key_length(algo)
            },
            GetKeyLengthAlgorithm::Hmac(algo) => {
                hmac_operation::get_key_length(algo)
            },
            GetKeyLengthAlgorithm::Hkdf(_algo) => {
                hkdf_operation::get_key_length()
            },
            GetKeyLengthAlgorithm::Pbkdf2(_algo) => {
                pbkdf2_operation::get_key_length()
            },
            GetKeyLengthAlgorithm::AesOcb(algo) => {
                aes_ocb_operation::get_key_length(algo)
            },
            GetKeyLengthAlgorithm::ChaCha20Poly1305(_algo) => {
                chacha20_poly1305_operation::get_key_length()
            },
            GetKeyLengthAlgorithm::Argon2(_algo) => {
                argon2_operation::get_key_length()
            },
        }
    }
}

// The value of the key "encapsulate" in the internal object supportedAlgorithms
pub(crate) struct EncapsulateOperation {}

impl Operation for EncapsulateOperation {
    type RegisteredAlgorithm = EncapsulateAlgorithm;
}

/// Normalized algorithm for the "encapsulate" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
pub(crate) enum EncapsulateAlgorithm {
    MlKem(SubtleAlgorithm),
}

impl NormalizedAlgorithm for EncapsulateAlgorithm {
    fn from_object_value(
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::MlKem512 | CryptoAlgorithm::MlKem768 | CryptoAlgorithm::MlKem1024 => Ok(EncapsulateAlgorithm::MlKem(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"encapsulate\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            EncapsulateAlgorithm::MlKem(algo) => &algo.name,
        }
    }
}

impl EncapsulateAlgorithm {
    pub(crate) fn encapsulate(&self, key: &CryptoKey) -> Result<SubtleEncapsulatedBits, Error> {
        match self {
            EncapsulateAlgorithm::MlKem(algo) => {
                ml_kem_operation::encapsulate(algo, key)
            },
        }
    }
}

// The value of the key "decapsulate" in the internal object supportedAlgorithms
pub(crate) struct DecapsulateOperation {}

impl Operation for DecapsulateOperation {
    type RegisteredAlgorithm = DecapsulateAlgorithm;
}

/// Normalized algorithm for the "decapsulate" operation, used as output of
/// <https://w3c.github.io/webcrypto/#dfn-normalize-an-algorithm>
pub(crate) enum DecapsulateAlgorithm {
    MlKem(SubtleAlgorithm),
}

impl NormalizedAlgorithm for DecapsulateAlgorithm {
    fn from_object_value(
        cx: &mut JSContext,
        alg_name: CryptoAlgorithm,
        value: HandleValue,
    ) -> Fallible<Self> {
        match alg_name {
            CryptoAlgorithm::MlKem512 | CryptoAlgorithm::MlKem768 | CryptoAlgorithm::MlKem1024 => Ok(DecapsulateAlgorithm::MlKem(value.try_into_with_cx(cx)?)),
            _ => Err(Error::NotSupported(Some(format!("{} does not support \"decapsulate\" operation", alg_name.as_str())))),
        }
    }

    fn name(&self) -> &str {
        match self {
            DecapsulateAlgorithm::MlKem(algo) => &algo.name,
        }
    }
}

impl DecapsulateAlgorithm {
    pub(crate) fn decapsulate(&self, key: &CryptoKey, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            DecapsulateAlgorithm::MlKem(algo) => {
                ml_kem_operation::decapsulate(algo, key, ciphertext)
            },
        }
    }
}
