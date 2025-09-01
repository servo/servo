/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::num::NonZero;

use aws_lc_rs::{hkdf, pbkdf2};
use script_bindings::error::{Error, Fallible};
use script_bindings::script_runtime::JSContext;
use script_bindings::trace::RootedTraceableBox;

use crate::crypto;
use crate::crypto::DigestAlgorithm;
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{
    AesCbcParams, AesCtrParams, AesGcmParams, AesKeyGenParams, HkdfParams, HmacImportParams,
    HmacKeyGenParams, Pbkdf2Params,
};
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use crate::dom::cryptokey::CryptoKey;

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
pub(crate) struct SubtleHmacImportParams {
    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyAlgorithm-hash>
    pub(crate) hash: DigestAlgorithm,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyGenParams-length>
    pub(crate) length: Option<u32>,
}

impl SubtleHmacImportParams {
    pub(crate) fn new(
        cx: JSContext,
        params: RootedTraceableBox<HmacImportParams>,
    ) -> Fallible<Self> {
        let hash = crypto::normalize_algorithm_for_digest(cx, &params.hash)?;
        let params = Self {
            hash,
            length: params.length,
        };
        Ok(params)
    }

    /// <https://w3c.github.io/webcrypto/#hmac-operations>
    pub(crate) fn get_key_length(&self) -> Result<u32, Error> {
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

pub(crate) struct SubtleHmacKeyGenParams {
    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyGenParams-hash>
    pub(crate) hash: DigestAlgorithm,

    /// <https://w3c.github.io/webcrypto/#dfn-HmacKeyGenParams-length>
    pub(crate) length: Option<u32>,
}

impl SubtleHmacKeyGenParams {
    pub(crate) fn new(
        cx: JSContext,
        params: RootedTraceableBox<HmacKeyGenParams>,
    ) -> Fallible<Self> {
        let hash = crypto::normalize_algorithm_for_digest(cx, &params.hash)?;
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
    pub(crate) hash: DigestAlgorithm,

    /// <https://w3c.github.io/webcrypto/#dfn-HkdfParams-salt>
    pub(crate) salt: Vec<u8>,

    /// <https://w3c.github.io/webcrypto/#dfn-HkdfParams-info>
    pub(crate) info: Vec<u8>,
}

impl SubtleHkdfParams {
    pub(crate) fn new(cx: JSContext, params: RootedTraceableBox<HkdfParams>) -> Fallible<Self> {
        let hash = crypto::normalize_algorithm_for_digest(cx, &params.hash)?;
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
    pub(crate) salt: Vec<u8>,

    /// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params-iterations>
    pub(crate) iterations: u32,

    /// <https://w3c.github.io/webcrypto/#dfn-Pbkdf2Params-hash>
    pub(crate) hash: DigestAlgorithm,
}

impl SubtlePbkdf2Params {
    pub(crate) fn new(cx: JSContext, params: RootedTraceableBox<Pbkdf2Params>) -> Fallible<Self> {
        let salt = match &params.salt {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
        };

        let params = Self {
            salt,
            iterations: params.iterations,
            hash: crypto::normalize_algorithm_for_digest(cx, &params.hash)?,
        };

        Ok(params)
    }
}

impl SubtleHkdfParams {
    /// <https://w3c.github.io/webcrypto/#hkdf-operations>
    pub(crate) fn derive_bits(
        &self,
        key: &CryptoKey,
        length: Option<u32>,
    ) -> Result<Vec<u8>, Error> {
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
    pub(crate) fn derive_bits(
        &self,
        key: &CryptoKey,
        length: Option<u32>,
    ) -> Result<Vec<u8>, Error> {
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
