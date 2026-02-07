/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aes::cipher::crypto_common::Key;
use aes::cipher::typenum::{U12, U13, U14, U15, U16, U32};
use aes::{Aes128, Aes192, Aes256};
use aes_gcm::aead::AeadMutInPlace;
use aes_gcm::{AesGcm, KeyInit};
use cipher::{ArrayLength, BlockCipher, BlockEncrypt, BlockSizeUser};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::KeyUsage;
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyFormat;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::aes_common::AesAlgorithm;
use crate::dom::subtlecrypto::{
    ExportedKey, SubtleAesDerivedKeyParams, SubtleAesGcmParams, SubtleAesKeyGenParams, aes_common,
};
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-encrypt>
pub(crate) fn encrypt(
    normalized_algorithm: &SubtleAesGcmParams,
    key: &CryptoKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If plaintext has a length greater than 2^39 - 256 bytes, then throw an
    // OperationError.
    if plaintext.len() as u64 > (1 << 39) - 256 {
        return Err(Error::Operation(Some("The plaintext is too long".into())));
    }

    // Step 2. If the iv member of normalizedAlgorithm has a length greater than 2^64 - 1 bytes,
    // then throw an OperationError.
    if normalized_algorithm.iv.len() > u64::MAX as usize {
        return Err(Error::Operation(Some(
            "The iv member of normalizedAlgorithm is too long".into(),
        )));
    }

    // Step 3. If the additionalData member of normalizedAlgorithm is present and has a length
    // greater than 2^64 - 1 bytes, then throw an OperationError.
    if normalized_algorithm
        .additional_data
        .as_ref()
        .is_some_and(|data| data.len() > u64::MAX as usize)
    {
        return Err(Error::Operation(Some(
            "The additional authentication data is too long".into(),
        )));
    }

    // Step 4.
    // If the tagLength member of normalizedAlgorithm is not present:
    //     Let tagLength be 128.
    // If the tagLength member of normalizedAlgorithm is one of 32, 64, 96, 104, 112, 120 or 128:
    //     Let tagLength be equal to the tagLength member of normalizedAlgorithm
    // Otherwise:
    //     throw an OperationError.
    let tag_length = match normalized_algorithm.tag_length {
        None => 128,
        Some(tag_length) if matches!(tag_length, 32 | 64 | 96 | 104 | 112 | 120 | 128) => {
            tag_length
        },
        _ => {
            return Err(Error::Operation(Some(
                "The tagLength member of normalizedAlgorithm is present, \
                and not one of 32, 64, 96, 104, 112, 120 or 128"
                    .to_string(),
            )));
        },
    };

    // Step 5. Let additionalData be the additionalData member of normalizedAlgorithm if present or
    // an empty byte sequence otherwise.
    let additional_data = normalized_algorithm
        .additional_data
        .as_deref()
        .unwrap_or_default();

    // Step 6. Let C and T be the outputs that result from performing the Authenticated Encryption
    // Function described in Section 7.1 of [NIST-SP800-38D] using AES as the block cipher, the iv
    // member of normalizedAlgorithm as the IV input parameter, additionalData as the A input
    // parameter, tagLength as the t pre-requisite and plaintext as the input plaintext.
    // Step 7. Let ciphertext be equal to C | T, where '|' denotes concatenation.
    //
    // NOTE: We currently support:
    // - IV: 96-bit, 128-bit, 256-bit
    // - Tag length: 32-bit, 64-bit, 96-bit, 104-bit, 112-bit, 120-bit, 128-bit
    //
    // NOTE: The crate `aes-gcm` does not directly support 32-bit and 64-bit tag length. Our
    // workaround is to perform the Authenticated Encryption Function using 96-bit tag length, and
    // then remove the last 64 bits for 32-bit tag length, and remove last 32 bits for 64-bit tag
    // length, from the ciphertext.
    let iv = normalized_algorithm.iv.as_slice();
    let ciphertext = match key.handle() {
        Handle::Aes128Key(key) => match (iv.len(), tag_length) {
            // 96-bit nonce
            (12, 32) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes128, U12, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 8);
                ciphertext
            },
            (12, 64) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes128, U12, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 4);
                ciphertext
            },
            (12, 96) => gcm_encrypt::<Aes128, U12, U12>(key, plaintext, iv, additional_data)?,
            (12, 104) => gcm_encrypt::<Aes128, U12, U13>(key, plaintext, iv, additional_data)?,
            (12, 112) => gcm_encrypt::<Aes128, U12, U14>(key, plaintext, iv, additional_data)?,
            (12, 120) => gcm_encrypt::<Aes128, U12, U15>(key, plaintext, iv, additional_data)?,
            (12, 128) => gcm_encrypt::<Aes128, U12, U16>(key, plaintext, iv, additional_data)?,

            // 128-bit nonce
            (16, 32) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes128, U16, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 8);
                ciphertext
            },
            (16, 64) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes128, U16, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 4);
                ciphertext
            },
            (16, 96) => gcm_encrypt::<Aes128, U16, U12>(key, plaintext, iv, additional_data)?,
            (16, 104) => gcm_encrypt::<Aes128, U16, U13>(key, plaintext, iv, additional_data)?,
            (16, 112) => gcm_encrypt::<Aes128, U16, U14>(key, plaintext, iv, additional_data)?,
            (16, 120) => gcm_encrypt::<Aes128, U16, U15>(key, plaintext, iv, additional_data)?,
            (16, 128) => gcm_encrypt::<Aes128, U16, U16>(key, plaintext, iv, additional_data)?,

            // 256-bit nonce
            (32, 32) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes128, U32, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 8);
                ciphertext
            },
            (32, 64) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes128, U32, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 4);
                ciphertext
            },
            (32, 96) => gcm_encrypt::<Aes128, U32, U12>(key, plaintext, iv, additional_data)?,
            (32, 104) => gcm_encrypt::<Aes128, U32, U13>(key, plaintext, iv, additional_data)?,
            (32, 112) => gcm_encrypt::<Aes128, U32, U14>(key, plaintext, iv, additional_data)?,
            (32, 120) => gcm_encrypt::<Aes128, U32, U15>(key, plaintext, iv, additional_data)?,
            (32, 128) => gcm_encrypt::<Aes128, U32, U16>(key, plaintext, iv, additional_data)?,
            _ => {
                return Err(Error::Operation(Some(format!(
                    "Unsupported iv size ({}-bit) and/or tag length ({}-bit)",
                    iv.len() * 8,
                    tag_length
                ))));
            },
        },
        Handle::Aes192Key(key) => match (iv.len(), tag_length) {
            // 96-bit nonce
            (12, 32) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes192, U12, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 8);
                ciphertext
            },
            (12, 64) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes192, U12, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 4);
                ciphertext
            },
            (12, 96) => gcm_encrypt::<Aes192, U12, U12>(key, plaintext, iv, additional_data)?,
            (12, 104) => gcm_encrypt::<Aes192, U12, U13>(key, plaintext, iv, additional_data)?,
            (12, 112) => gcm_encrypt::<Aes192, U12, U14>(key, plaintext, iv, additional_data)?,
            (12, 120) => gcm_encrypt::<Aes192, U12, U15>(key, plaintext, iv, additional_data)?,
            (12, 128) => gcm_encrypt::<Aes192, U12, U16>(key, plaintext, iv, additional_data)?,

            // 128-bit nonce
            (16, 32) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes192, U16, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 8);
                ciphertext
            },
            (16, 64) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes192, U16, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 4);
                ciphertext
            },
            (16, 96) => gcm_encrypt::<Aes192, U16, U12>(key, plaintext, iv, additional_data)?,
            (16, 104) => gcm_encrypt::<Aes192, U16, U13>(key, plaintext, iv, additional_data)?,
            (16, 112) => gcm_encrypt::<Aes192, U16, U14>(key, plaintext, iv, additional_data)?,
            (16, 120) => gcm_encrypt::<Aes192, U16, U15>(key, plaintext, iv, additional_data)?,
            (16, 128) => gcm_encrypt::<Aes192, U16, U16>(key, plaintext, iv, additional_data)?,

            // 256-bit nonce
            (32, 32) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes192, U32, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 8);
                ciphertext
            },
            (32, 64) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes192, U32, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 4);
                ciphertext
            },
            (32, 96) => gcm_encrypt::<Aes192, U32, U12>(key, plaintext, iv, additional_data)?,
            (32, 104) => gcm_encrypt::<Aes192, U32, U13>(key, plaintext, iv, additional_data)?,
            (32, 112) => gcm_encrypt::<Aes192, U32, U14>(key, plaintext, iv, additional_data)?,
            (32, 120) => gcm_encrypt::<Aes192, U32, U15>(key, plaintext, iv, additional_data)?,
            (32, 128) => gcm_encrypt::<Aes192, U32, U16>(key, plaintext, iv, additional_data)?,
            _ => {
                return Err(Error::Operation(Some(format!(
                    "Unsupported iv size ({}-bit) and/or tag length ({}-bit)",
                    iv.len() * 8,
                    tag_length
                ))));
            },
        },
        Handle::Aes256Key(key) => match (iv.len(), tag_length) {
            // 96-bit nonce
            (12, 32) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes256, U12, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 8);
                ciphertext
            },
            (12, 64) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes256, U12, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 4);
                ciphertext
            },
            (12, 96) => gcm_encrypt::<Aes256, U12, U12>(key, plaintext, iv, additional_data)?,
            (12, 104) => gcm_encrypt::<Aes256, U12, U13>(key, plaintext, iv, additional_data)?,
            (12, 112) => gcm_encrypt::<Aes256, U12, U14>(key, plaintext, iv, additional_data)?,
            (12, 120) => gcm_encrypt::<Aes256, U12, U15>(key, plaintext, iv, additional_data)?,
            (12, 128) => gcm_encrypt::<Aes256, U12, U16>(key, plaintext, iv, additional_data)?,

            // 128-bit nonce
            (16, 32) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes256, U16, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 8);
                ciphertext
            },
            (16, 64) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes256, U16, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 4);
                ciphertext
            },
            (16, 96) => gcm_encrypt::<Aes256, U16, U12>(key, plaintext, iv, additional_data)?,
            (16, 104) => gcm_encrypt::<Aes256, U16, U13>(key, plaintext, iv, additional_data)?,
            (16, 112) => gcm_encrypt::<Aes256, U16, U14>(key, plaintext, iv, additional_data)?,
            (16, 120) => gcm_encrypt::<Aes256, U16, U15>(key, plaintext, iv, additional_data)?,
            (16, 128) => gcm_encrypt::<Aes256, U16, U16>(key, plaintext, iv, additional_data)?,

            // 256-bit nonce
            (32, 32) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes256, U32, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 8);
                ciphertext
            },
            (32, 64) => {
                let mut ciphertext =
                    gcm_encrypt::<Aes256, U32, U12>(key, plaintext, iv, additional_data)?;
                ciphertext.truncate(ciphertext.len() - 4);
                ciphertext
            },
            (32, 96) => gcm_encrypt::<Aes256, U32, U12>(key, plaintext, iv, additional_data)?,
            (32, 104) => gcm_encrypt::<Aes256, U32, U13>(key, plaintext, iv, additional_data)?,
            (32, 112) => gcm_encrypt::<Aes256, U32, U14>(key, plaintext, iv, additional_data)?,
            (32, 120) => gcm_encrypt::<Aes256, U32, U15>(key, plaintext, iv, additional_data)?,
            (32, 128) => gcm_encrypt::<Aes256, U32, U16>(key, plaintext, iv, additional_data)?,
            _ => {
                return Err(Error::Operation(Some(format!(
                    "Unsupported iv size ({}-bit) and/or tag length ({}-bit)",
                    iv.len() * 8,
                    tag_length
                ))));
            },
        },
        _ => {
            return Err(Error::Operation(Some(
                "The key handle is not representing an AES key".to_string(),
            )));
        },
    };

    // Step 8. Return ciphertext.
    Ok(ciphertext)
}

/// Helper for Step 6 and 7 of <https://w3c.github.io/webcrypto/#aes-gcm-operations-encrypt>
fn gcm_encrypt<Aes, NonceSize, TagSize>(
    key: &Key<Aes>,
    plaintext: &[u8],
    iv: &[u8],
    additional_data: &[u8],
) -> Result<Vec<u8>, Error>
where
    Aes: BlockCipher + BlockSizeUser<BlockSize = U16> + BlockEncrypt + KeyInit,
    NonceSize: ArrayLength<u8>,
    TagSize: aes_gcm::TagSize,
{
    let mut ciphertext = plaintext.to_vec();

    let mut cipher = AesGcm::<Aes, NonceSize, TagSize>::new(key);
    cipher
        .encrypt_in_place(iv.into(), additional_data, &mut ciphertext)
        .map_err(|_| {
            Error::Operation(Some(
                "AES-GCM failed to perform the Authenticated Encryption Function".to_string(),
            ))
        })?;

    Ok(ciphertext)
}

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-decrypt>
pub(crate) fn decrypt(
    normalized_algorithm: &SubtleAesGcmParams,
    key: &CryptoKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1.
    // If the tagLength member of normalizedAlgorithm is not present:
    //     Let tagLength be 128.
    // If the tagLength member of normalizedAlgorithm is one of 32, 64, 96, 104, 112, 120 or 128:
    //     Let tagLength be equal to the tagLength member of normalizedAlgorithm
    // Otherwise:
    //     throw an OperationError.
    let tag_length = match normalized_algorithm.tag_length {
        None => 128,
        Some(tag_length) if matches!(tag_length, 32 | 64 | 96 | 104 | 112 | 120 | 128) => {
            tag_length
        },
        _ => {
            return Err(Error::Operation(Some(
                "The tagLength member of normalizedAlgorithm is present, \
                and not one of 32, 64, 96, 104, 112, 120 or 128"
                    .to_string(),
            )));
        },
    };

    // Step 2. If ciphertext has a length in bits less than tagLength, then throw an
    // OperationError.
    if ciphertext.len() * 8 < tag_length as usize {
        return Err(Error::Operation(Some(
            "The ciphertext is shorter than the tag".into(),
        )));
    }

    // Step 2. If the iv member of normalizedAlgorithm has a length greater than 2^64 - 1 bytes,
    // then throw an OperationError.
    if normalized_algorithm.iv.len() > u64::MAX as usize {
        return Err(Error::Operation(Some(
            "The iv member of normalizedAlgorithm is too long".into(),
        )));
    }

    // Step 3. If the additionalData member of normalizedAlgorithm is present and has a length
    // greater than 2^64 - 1 bytes, then throw an OperationError.
    if normalized_algorithm
        .additional_data
        .as_ref()
        .is_some_and(|data| data.len() > u64::MAX as usize)
    {
        return Err(Error::Operation(Some(
            "The additional authentication data is too long".into(),
        )));
    }

    // Step 5. Let tag be the last tagLength bits of ciphertext.
    // Step 6. Let actualCiphertext be the result of removing the last tagLength bits from
    // ciphertext.
    // NOTE: aes_gcm splits the ciphertext for us

    // Step 7. Let additionalData be the additionalData member of normalizedAlgorithm if present or
    // an empty byte sequence otherwise.
    let additional_data = normalized_algorithm
        .additional_data
        .as_deref()
        .unwrap_or_default();

    // Step 8. Perform the Authenticated Decryption Function described in Section 7.2 of
    // [NIST-SP800-38D] using AES as the block cipher, the iv member of normalizedAlgorithm as the
    // IV input parameter, additionalData as the A input parameter, tagLength as the t
    // pre-requisite, actualCiphertext as the input ciphertext, C and tag as the authentication
    // tag, T.
    //
    // If the result of the algorithm is the indication of inauthenticity, "FAIL":
    //     throw an OperationError
    // Otherwise:
    //     Let plaintext be the output P of the Authenticated Decryption Function.
    //
    // NOTE: We currently support:
    // - IV: 96-bit, 128-bit, 256-bit
    // - Tag length: 96-bit, 104-bit, 112-bit, 120-bit, 128-bit
    //
    // NOTE: We currently do not support 32-bit and 64-bit tag length in decryption since the crate
    // `aes-gcm` does not support 32-bit and 64-bit tag length.
    let iv = normalized_algorithm.iv.as_slice();
    let plaintext = match key.handle() {
        Handle::Aes128Key(key) => match (iv.len(), tag_length) {
            // 96-bit nonce
            (12, 96) => gcm_decrypt::<Aes128, U12, U12>(key, ciphertext, iv, additional_data)?,
            (12, 104) => gcm_decrypt::<Aes128, U12, U13>(key, ciphertext, iv, additional_data)?,
            (12, 112) => gcm_decrypt::<Aes128, U12, U14>(key, ciphertext, iv, additional_data)?,
            (12, 120) => gcm_decrypt::<Aes128, U12, U15>(key, ciphertext, iv, additional_data)?,
            (12, 128) => gcm_decrypt::<Aes128, U12, U16>(key, ciphertext, iv, additional_data)?,

            // 128-bit nonce
            (16, 96) => gcm_decrypt::<Aes128, U16, U12>(key, ciphertext, iv, additional_data)?,
            (16, 104) => gcm_decrypt::<Aes128, U16, U13>(key, ciphertext, iv, additional_data)?,
            (16, 112) => gcm_decrypt::<Aes128, U16, U14>(key, ciphertext, iv, additional_data)?,
            (16, 120) => gcm_decrypt::<Aes128, U16, U15>(key, ciphertext, iv, additional_data)?,
            (16, 128) => gcm_decrypt::<Aes128, U16, U16>(key, ciphertext, iv, additional_data)?,

            // 256-bit nonce
            (32, 96) => gcm_decrypt::<Aes128, U32, U12>(key, ciphertext, iv, additional_data)?,
            (32, 104) => gcm_decrypt::<Aes128, U32, U13>(key, ciphertext, iv, additional_data)?,
            (32, 112) => gcm_decrypt::<Aes128, U32, U14>(key, ciphertext, iv, additional_data)?,
            (32, 120) => gcm_decrypt::<Aes128, U32, U15>(key, ciphertext, iv, additional_data)?,
            (32, 128) => gcm_decrypt::<Aes128, U32, U16>(key, ciphertext, iv, additional_data)?,

            _ => {
                return Err(Error::Operation(Some(format!(
                    "Unsupported iv size ({}-bit) and/or tag length ({}-bit)",
                    iv.len() * 8,
                    tag_length
                ))));
            },
        },
        Handle::Aes192Key(key) => match (iv.len(), tag_length) {
            // 96-bit nonce
            (12, 96) => gcm_decrypt::<Aes192, U12, U12>(key, ciphertext, iv, additional_data)?,
            (12, 104) => gcm_decrypt::<Aes192, U12, U13>(key, ciphertext, iv, additional_data)?,
            (12, 112) => gcm_decrypt::<Aes192, U12, U14>(key, ciphertext, iv, additional_data)?,
            (12, 120) => gcm_decrypt::<Aes192, U12, U15>(key, ciphertext, iv, additional_data)?,
            (12, 128) => gcm_decrypt::<Aes192, U12, U16>(key, ciphertext, iv, additional_data)?,

            // 128-bit nonce
            (16, 96) => gcm_decrypt::<Aes192, U16, U12>(key, ciphertext, iv, additional_data)?,
            (16, 104) => gcm_decrypt::<Aes192, U16, U13>(key, ciphertext, iv, additional_data)?,
            (16, 112) => gcm_decrypt::<Aes192, U16, U14>(key, ciphertext, iv, additional_data)?,
            (16, 120) => gcm_decrypt::<Aes192, U16, U15>(key, ciphertext, iv, additional_data)?,
            (16, 128) => gcm_decrypt::<Aes192, U16, U16>(key, ciphertext, iv, additional_data)?,

            // 256-bit nonce
            (32, 96) => gcm_decrypt::<Aes192, U32, U12>(key, ciphertext, iv, additional_data)?,
            (32, 104) => gcm_decrypt::<Aes192, U32, U13>(key, ciphertext, iv, additional_data)?,
            (32, 112) => gcm_decrypt::<Aes192, U32, U14>(key, ciphertext, iv, additional_data)?,
            (32, 120) => gcm_decrypt::<Aes192, U32, U15>(key, ciphertext, iv, additional_data)?,
            (32, 128) => gcm_decrypt::<Aes192, U32, U16>(key, ciphertext, iv, additional_data)?,

            _ => {
                return Err(Error::Operation(Some(format!(
                    "Unsupported iv size ({}-bit) and/or tag length ({}-bit)",
                    iv.len() * 8,
                    tag_length
                ))));
            },
        },
        Handle::Aes256Key(key) => match (iv.len(), tag_length) {
            // 96-bit nonce
            (12, 96) => gcm_decrypt::<Aes256, U12, U12>(key, ciphertext, iv, additional_data)?,
            (12, 104) => gcm_decrypt::<Aes256, U12, U13>(key, ciphertext, iv, additional_data)?,
            (12, 112) => gcm_decrypt::<Aes256, U12, U14>(key, ciphertext, iv, additional_data)?,
            (12, 120) => gcm_decrypt::<Aes256, U12, U15>(key, ciphertext, iv, additional_data)?,
            (12, 128) => gcm_decrypt::<Aes256, U12, U16>(key, ciphertext, iv, additional_data)?,

            // 128-bit nonce
            (16, 96) => gcm_decrypt::<Aes256, U16, U12>(key, ciphertext, iv, additional_data)?,
            (16, 104) => gcm_decrypt::<Aes256, U16, U13>(key, ciphertext, iv, additional_data)?,
            (16, 112) => gcm_decrypt::<Aes256, U16, U14>(key, ciphertext, iv, additional_data)?,
            (16, 120) => gcm_decrypt::<Aes256, U16, U15>(key, ciphertext, iv, additional_data)?,
            (16, 128) => gcm_decrypt::<Aes256, U16, U16>(key, ciphertext, iv, additional_data)?,

            // 256-bit nonce
            (32, 96) => gcm_decrypt::<Aes256, U32, U12>(key, ciphertext, iv, additional_data)?,
            (32, 104) => gcm_decrypt::<Aes256, U32, U13>(key, ciphertext, iv, additional_data)?,
            (32, 112) => gcm_decrypt::<Aes256, U32, U14>(key, ciphertext, iv, additional_data)?,
            (32, 120) => gcm_decrypt::<Aes256, U32, U15>(key, ciphertext, iv, additional_data)?,
            (32, 128) => gcm_decrypt::<Aes256, U32, U16>(key, ciphertext, iv, additional_data)?,

            _ => {
                return Err(Error::Operation(Some(format!(
                    "Unsupported iv size ({}-bit) and/or tag length ({}-bit)",
                    iv.len() * 8,
                    tag_length
                ))));
            },
        },
        _ => {
            return Err(Error::Operation(Some(
                "The key handle is not representing an AES key".to_string(),
            )));
        },
    };

    // Step 9. Return plaintext.
    Ok(plaintext)
}

/// Helper for Step 8 of <https://w3c.github.io/webcrypto/#aes-gcm-operations-decrypt>
fn gcm_decrypt<Aes, NonceSize, TagSize>(
    key: &Key<Aes>,
    ciphertext: &[u8],
    iv: &[u8],
    additional_data: &[u8],
) -> Result<Vec<u8>, Error>
where
    Aes: BlockCipher + BlockSizeUser<BlockSize = U16> + BlockEncrypt + KeyInit,
    NonceSize: ArrayLength<u8>,
    TagSize: aes_gcm::TagSize,
{
    let mut plaintext = ciphertext.to_vec();

    let mut cipher = AesGcm::<Aes, NonceSize, TagSize>::new(key);
    cipher
        .decrypt_in_place(iv.into(), additional_data, &mut plaintext)
        .map_err(|_| {
            Error::Operation(Some(
                "AES-GCM failed to perform the Authenticated Decryption Function".to_string(),
            ))
        })?;

    Ok(plaintext)
}

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-generate-key>
pub(crate) fn generate_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    aes_common::generate_key(
        AesAlgorithm::AesGcm,
        global,
        normalized_algorithm,
        extractable,
        usages,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    aes_common::import_key(
        AesAlgorithm::AesGcm,
        global,
        format,
        key_data,
        extractable,
        usages,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    aes_common::export_key(AesAlgorithm::AesGcm, format, key)
}

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-get-key-length>
pub(crate) fn get_key_length(
    normalized_derived_key_algorithm: &SubtleAesDerivedKeyParams,
) -> Result<Option<u32>, Error> {
    aes_common::get_key_length(normalized_derived_key_algorithm)
}
