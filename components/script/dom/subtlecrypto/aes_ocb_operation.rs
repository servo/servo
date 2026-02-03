/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aes::cipher::crypto_common::Key;
use aes::{Aes128, Aes192, Aes256};
use cipher::generic_array::typenum::{GrEq, IsGreaterOrEqual, IsLessOrEqual, LeEq, NonZero};
use cipher::{ArrayLength, BlockDecrypt, BlockEncrypt, BlockSizeUser};
use ocb3::aead::AeadMutInPlace;
use ocb3::aead::consts::{U6, U7, U8, U9, U10, U11, U12, U13, U14, U15, U16};
use ocb3::{KeyInit, Nonce, Ocb3};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::KeyUsage;
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyFormat;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::aes_common::AesAlgorithm;
use crate::dom::subtlecrypto::{
    ExportedKey, SubtleAeadParams, SubtleAesDerivedKeyParams, SubtleAesKeyGenParams, aes_common,
};
use crate::script_runtime::CanGc;

/// <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-encrypt>
pub(crate) fn encrypt(
    normalized_algorithm: &SubtleAeadParams,
    key: &CryptoKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the iv member of normalizedAlgorithm has a length greater than 15 bytes, then
    // throw an OperationError.
    if normalized_algorithm.iv.len() > 15 {
        return Err(Error::Operation(Some(
            "The iv member of normalizedAlgorithm has a length greater than 15 bytes".to_string(),
        )));
    }

    // Step 2.
    // If the tagLength member of normalizedAlgorithm is not present:
    //     Let tagLength be 128.
    // If the tagLength member of normalizedAlgorithm is one of 64, 96 or 128:
    //     Let tagLength be equal to the tagLength member of normalizedAlgorithm
    // Otherwise:
    //     throw an OperationError.
    let tag_length = match normalized_algorithm.tag_length {
        None => 128,
        Some(tag_length) if matches!(tag_length, 64 | 96 | 128) => tag_length,
        _ => {
            return Err(Error::Operation(Some(
                "The tagLength member of normalizedAlgorithm is present, \
                and not one of 64, 96, or 128"
                    .to_string(),
            )));
        },
    };

    // Step 3. Let additionalData be the additionalData member of normalizedAlgorithm if present or
    // the empty octet string otherwise.
    let additional_data = normalized_algorithm
        .additional_data
        .as_deref()
        .unwrap_or_default();

    // Step 4. Let C be the output that results from performing the OCB-ENCRYPT function described
    // in Section 4.2 of [RFC7253] using AES as the block cipher, using the key represented by
    // [[handle]] internal slot of key as the K input parameter, the iv member of
    // normalizedAlgorithm as the N input parameter, additionalData as the A input parameter,
    // plaintext as the P input parameter, and tagLength as the TAGLEN global parameter.
    //
    // NOTE: We only support IV(nonce) size from 6 bytes to 15 bytes because of the restriction
    // from the `ocb3` crate <https://docs.rs/ocb3/latest/ocb3/struct.Ocb3.html>. This range is
    // suggested in the paper <https://eprint.iacr.org/2023/326.pdf> to prevent an attack.
    let iv = &normalized_algorithm.iv;
    let c = match key.handle() {
        Handle::Aes128Key(key) => match (iv.len(), tag_length) {
            (6, 64) => ocb_encrypt::<Aes128, U6, U8>(key, plaintext, iv, additional_data)?,
            (7, 64) => ocb_encrypt::<Aes128, U7, U8>(key, plaintext, iv, additional_data)?,
            (8, 64) => ocb_encrypt::<Aes128, U8, U8>(key, plaintext, iv, additional_data)?,
            (9, 64) => ocb_encrypt::<Aes128, U9, U8>(key, plaintext, iv, additional_data)?,
            (10, 64) => ocb_encrypt::<Aes128, U10, U8>(key, plaintext, iv, additional_data)?,
            (11, 64) => ocb_encrypt::<Aes128, U11, U8>(key, plaintext, iv, additional_data)?,
            (12, 64) => ocb_encrypt::<Aes128, U12, U8>(key, plaintext, iv, additional_data)?,
            (13, 64) => ocb_encrypt::<Aes128, U13, U8>(key, plaintext, iv, additional_data)?,
            (14, 64) => ocb_encrypt::<Aes128, U14, U8>(key, plaintext, iv, additional_data)?,
            (15, 64) => ocb_encrypt::<Aes128, U15, U8>(key, plaintext, iv, additional_data)?,

            (6, 96) => ocb_encrypt::<Aes128, U6, U12>(key, plaintext, iv, additional_data)?,
            (7, 96) => ocb_encrypt::<Aes128, U7, U12>(key, plaintext, iv, additional_data)?,
            (8, 96) => ocb_encrypt::<Aes128, U8, U12>(key, plaintext, iv, additional_data)?,
            (9, 96) => ocb_encrypt::<Aes128, U9, U12>(key, plaintext, iv, additional_data)?,
            (10, 96) => ocb_encrypt::<Aes128, U10, U12>(key, plaintext, iv, additional_data)?,
            (11, 96) => ocb_encrypt::<Aes128, U11, U12>(key, plaintext, iv, additional_data)?,
            (12, 96) => ocb_encrypt::<Aes128, U12, U12>(key, plaintext, iv, additional_data)?,
            (13, 96) => ocb_encrypt::<Aes128, U13, U12>(key, plaintext, iv, additional_data)?,
            (14, 96) => ocb_encrypt::<Aes128, U14, U12>(key, plaintext, iv, additional_data)?,
            (15, 96) => ocb_encrypt::<Aes128, U15, U12>(key, plaintext, iv, additional_data)?,

            (6, 128) => ocb_encrypt::<Aes128, U6, U16>(key, plaintext, iv, additional_data)?,
            (7, 128) => ocb_encrypt::<Aes128, U7, U16>(key, plaintext, iv, additional_data)?,
            (8, 128) => ocb_encrypt::<Aes128, U8, U16>(key, plaintext, iv, additional_data)?,
            (9, 128) => ocb_encrypt::<Aes128, U9, U16>(key, plaintext, iv, additional_data)?,
            (10, 128) => ocb_encrypt::<Aes128, U10, U16>(key, plaintext, iv, additional_data)?,
            (11, 128) => ocb_encrypt::<Aes128, U11, U16>(key, plaintext, iv, additional_data)?,
            (12, 128) => ocb_encrypt::<Aes128, U12, U16>(key, plaintext, iv, additional_data)?,
            (13, 128) => ocb_encrypt::<Aes128, U13, U16>(key, plaintext, iv, additional_data)?,
            (14, 128) => ocb_encrypt::<Aes128, U14, U16>(key, plaintext, iv, additional_data)?,
            (15, 128) => ocb_encrypt::<Aes128, U15, U16>(key, plaintext, iv, additional_data)?,

            _ => {
                return Err(Error::Operation(Some(format!(
                    "Unsupported IV size ({}-bytes) and/or tag length ({}-bit)",
                    iv.len(),
                    tag_length
                ))));
            },
        },
        Handle::Aes192Key(key) => match (iv.len(), tag_length) {
            (6, 64) => ocb_encrypt::<Aes192, U6, U8>(key, plaintext, iv, additional_data)?,
            (7, 64) => ocb_encrypt::<Aes192, U7, U8>(key, plaintext, iv, additional_data)?,
            (8, 64) => ocb_encrypt::<Aes192, U8, U8>(key, plaintext, iv, additional_data)?,
            (9, 64) => ocb_encrypt::<Aes192, U9, U8>(key, plaintext, iv, additional_data)?,
            (10, 64) => ocb_encrypt::<Aes192, U10, U8>(key, plaintext, iv, additional_data)?,
            (11, 64) => ocb_encrypt::<Aes192, U11, U8>(key, plaintext, iv, additional_data)?,
            (12, 64) => ocb_encrypt::<Aes192, U12, U8>(key, plaintext, iv, additional_data)?,
            (13, 64) => ocb_encrypt::<Aes192, U13, U8>(key, plaintext, iv, additional_data)?,
            (14, 64) => ocb_encrypt::<Aes192, U14, U8>(key, plaintext, iv, additional_data)?,
            (15, 64) => ocb_encrypt::<Aes192, U15, U8>(key, plaintext, iv, additional_data)?,

            (6, 96) => ocb_encrypt::<Aes192, U6, U12>(key, plaintext, iv, additional_data)?,
            (7, 96) => ocb_encrypt::<Aes192, U7, U12>(key, plaintext, iv, additional_data)?,
            (8, 96) => ocb_encrypt::<Aes192, U8, U12>(key, plaintext, iv, additional_data)?,
            (9, 96) => ocb_encrypt::<Aes192, U9, U12>(key, plaintext, iv, additional_data)?,
            (10, 96) => ocb_encrypt::<Aes192, U10, U12>(key, plaintext, iv, additional_data)?,
            (11, 96) => ocb_encrypt::<Aes192, U11, U12>(key, plaintext, iv, additional_data)?,
            (12, 96) => ocb_encrypt::<Aes192, U12, U12>(key, plaintext, iv, additional_data)?,
            (13, 96) => ocb_encrypt::<Aes192, U13, U12>(key, plaintext, iv, additional_data)?,
            (14, 96) => ocb_encrypt::<Aes192, U14, U12>(key, plaintext, iv, additional_data)?,
            (15, 96) => ocb_encrypt::<Aes192, U15, U12>(key, plaintext, iv, additional_data)?,

            (6, 128) => ocb_encrypt::<Aes192, U6, U16>(key, plaintext, iv, additional_data)?,
            (7, 128) => ocb_encrypt::<Aes192, U7, U16>(key, plaintext, iv, additional_data)?,
            (8, 128) => ocb_encrypt::<Aes192, U8, U16>(key, plaintext, iv, additional_data)?,
            (9, 128) => ocb_encrypt::<Aes192, U9, U16>(key, plaintext, iv, additional_data)?,
            (10, 128) => ocb_encrypt::<Aes192, U10, U16>(key, plaintext, iv, additional_data)?,
            (11, 128) => ocb_encrypt::<Aes192, U11, U16>(key, plaintext, iv, additional_data)?,
            (12, 128) => ocb_encrypt::<Aes192, U12, U16>(key, plaintext, iv, additional_data)?,
            (13, 128) => ocb_encrypt::<Aes192, U13, U16>(key, plaintext, iv, additional_data)?,
            (14, 128) => ocb_encrypt::<Aes192, U14, U16>(key, plaintext, iv, additional_data)?,
            (15, 128) => ocb_encrypt::<Aes192, U15, U16>(key, plaintext, iv, additional_data)?,

            _ => {
                return Err(Error::Operation(Some(format!(
                    "Unsupported IV size ({}-bytes) and/or tag length ({}-bit)",
                    iv.len(),
                    tag_length
                ))));
            },
        },
        Handle::Aes256Key(key) => match (iv.len(), tag_length) {
            (6, 64) => ocb_encrypt::<Aes256, U6, U8>(key, plaintext, iv, additional_data)?,
            (7, 64) => ocb_encrypt::<Aes256, U7, U8>(key, plaintext, iv, additional_data)?,
            (8, 64) => ocb_encrypt::<Aes256, U8, U8>(key, plaintext, iv, additional_data)?,
            (9, 64) => ocb_encrypt::<Aes256, U9, U8>(key, plaintext, iv, additional_data)?,
            (10, 64) => ocb_encrypt::<Aes256, U10, U8>(key, plaintext, iv, additional_data)?,
            (11, 64) => ocb_encrypt::<Aes256, U11, U8>(key, plaintext, iv, additional_data)?,
            (12, 64) => ocb_encrypt::<Aes256, U12, U8>(key, plaintext, iv, additional_data)?,
            (13, 64) => ocb_encrypt::<Aes256, U13, U8>(key, plaintext, iv, additional_data)?,
            (14, 64) => ocb_encrypt::<Aes256, U14, U8>(key, plaintext, iv, additional_data)?,
            (15, 64) => ocb_encrypt::<Aes256, U15, U8>(key, plaintext, iv, additional_data)?,

            (6, 96) => ocb_encrypt::<Aes256, U6, U12>(key, plaintext, iv, additional_data)?,
            (7, 96) => ocb_encrypt::<Aes256, U7, U12>(key, plaintext, iv, additional_data)?,
            (8, 96) => ocb_encrypt::<Aes256, U8, U12>(key, plaintext, iv, additional_data)?,
            (9, 96) => ocb_encrypt::<Aes256, U9, U12>(key, plaintext, iv, additional_data)?,
            (10, 96) => ocb_encrypt::<Aes256, U10, U12>(key, plaintext, iv, additional_data)?,
            (11, 96) => ocb_encrypt::<Aes256, U11, U12>(key, plaintext, iv, additional_data)?,
            (12, 96) => ocb_encrypt::<Aes256, U12, U12>(key, plaintext, iv, additional_data)?,
            (13, 96) => ocb_encrypt::<Aes256, U13, U12>(key, plaintext, iv, additional_data)?,
            (14, 96) => ocb_encrypt::<Aes256, U14, U12>(key, plaintext, iv, additional_data)?,
            (15, 96) => ocb_encrypt::<Aes256, U15, U12>(key, plaintext, iv, additional_data)?,

            (6, 128) => ocb_encrypt::<Aes256, U6, U16>(key, plaintext, iv, additional_data)?,
            (7, 128) => ocb_encrypt::<Aes256, U7, U16>(key, plaintext, iv, additional_data)?,
            (8, 128) => ocb_encrypt::<Aes256, U8, U16>(key, plaintext, iv, additional_data)?,
            (9, 128) => ocb_encrypt::<Aes256, U9, U16>(key, plaintext, iv, additional_data)?,
            (10, 128) => ocb_encrypt::<Aes256, U10, U16>(key, plaintext, iv, additional_data)?,
            (11, 128) => ocb_encrypt::<Aes256, U11, U16>(key, plaintext, iv, additional_data)?,
            (12, 128) => ocb_encrypt::<Aes256, U12, U16>(key, plaintext, iv, additional_data)?,
            (13, 128) => ocb_encrypt::<Aes256, U13, U16>(key, plaintext, iv, additional_data)?,
            (14, 128) => ocb_encrypt::<Aes256, U14, U16>(key, plaintext, iv, additional_data)?,
            (15, 128) => ocb_encrypt::<Aes256, U15, U16>(key, plaintext, iv, additional_data)?,

            _ => {
                return Err(Error::Operation(Some(format!(
                    "Unsupported IV size ({}-bytes) and/or tag length ({}-bit)",
                    iv.len(),
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

    // Step 5. Return C.
    Ok(c)
}

/// Helper for Step 5 of <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-encrypt>
fn ocb_encrypt<Cipher, NonceSize, TagSize>(
    key: &Key<Cipher>,
    plaintext: &[u8],
    iv: &[u8],
    additional_data: &[u8],
) -> Result<Vec<u8>, Error>
where
    Cipher: BlockSizeUser<BlockSize = U16> + BlockEncrypt + KeyInit + BlockDecrypt,
    NonceSize: ArrayLength<u8> + IsGreaterOrEqual<U6> + IsLessOrEqual<U15>,
    GrEq<NonceSize, U6>: NonZero,
    LeEq<NonceSize, U15>: NonZero,
    TagSize: ArrayLength<u8> + NonZero + IsLessOrEqual<U16>,
    LeEq<TagSize, U16>: NonZero,
{
    let mut c = plaintext.to_vec();
    let mut cipher = Ocb3::<Cipher, NonceSize, TagSize>::new(key);
    cipher
        .encrypt_in_place(Nonce::from_slice(iv), additional_data, &mut c)
        .map_err(|_| {
            Error::Operation(Some("AES-OCB authenticated encryption failed".to_string()))
        })?;
    Ok(c)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-decrypt>
pub(crate) fn decrypt(
    normalized_algorithm: &SubtleAeadParams,
    key: &CryptoKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the iv member of normalizedAlgorithm has a length greater than 15 bytes, then
    // throw an OperationError.
    if normalized_algorithm.iv.len() > 15 {
        return Err(Error::Operation(Some(
            "The iv member of normalizedAlgorithm has a length greater than 15 bytes".to_string(),
        )));
    }

    // Step 2.
    // If the tagLength member of normalizedAlgorithm is not present:
    //     Let tagLength be 128.
    // If the tagLength member of normalizedAlgorithm is one of 64, 96 or 128:
    //     Let tagLength be equal to the tagLength member of normalizedAlgorithm
    // Otherwise:
    //     throw an OperationError.
    let tag_length = match normalized_algorithm.tag_length {
        None => 128,
        Some(tag_length) if matches!(tag_length, 64 | 96 | 128) => tag_length,
        _ => {
            return Err(Error::Operation(Some(
                "The tagLength member of normalizedAlgorithm is present, \
                and not one of 64, 96, or 128"
                    .to_string(),
            )));
        },
    };

    // Step 3. If ciphertext has a length less than tagLength bits, then throw an OperationError.
    if ciphertext.len() * 8 < tag_length as usize {
        return Err(Error::Operation(Some(
            "Ciphertext has a length less than tagLength bits".to_string(),
        )));
    }

    // Step 4. Let additionalData be the additionalData member of normalizedAlgorithm if present or
    // the empty octet string otherwise.
    let additional_data = normalized_algorithm
        .additional_data
        .as_deref()
        .unwrap_or_default();

    // Step 5. Perform the OCB-DECRYPT function described in Section 4.3 of [RFC7253] using AES as
    // the block cipher, using the key represented by [[handle]] internal slot of key as the K
    // input parameter, the iv member of normalizedAlgorithm as the N input parameter,
    // additionalData as the A input parameter, ciphertext as the C input parameter, and tagLength
    // as the TAGLEN global parameter.
    //
    // If the result of the algorithm is the indication of authentication failure, "INVALID":
    //     throw an OperationError
    // Otherwise:
    //     Let plaintext be the output P of OCB-DECRYPT.
    //
    // NOTE: We only support IV(nonce) size from 6 bytes to 15 bytes because of the restriction
    // from the `ocb3` crate <https://docs.rs/ocb3/latest/ocb3/struct.Ocb3.html>. This range is
    // suggested in the paper <https://eprint.iacr.org/2023/326.pdf> to prevent an attack.
    let iv = &normalized_algorithm.iv;
    let plaintext = match key.handle() {
        Handle::Aes128Key(key) => match (iv.len(), tag_length) {
            (6, 64) => ocb_decrypt::<Aes128, U6, U8>(key, ciphertext, iv, additional_data)?,
            (7, 64) => ocb_decrypt::<Aes128, U7, U8>(key, ciphertext, iv, additional_data)?,
            (8, 64) => ocb_decrypt::<Aes128, U8, U8>(key, ciphertext, iv, additional_data)?,
            (9, 64) => ocb_decrypt::<Aes128, U9, U8>(key, ciphertext, iv, additional_data)?,
            (10, 64) => ocb_decrypt::<Aes128, U10, U8>(key, ciphertext, iv, additional_data)?,
            (11, 64) => ocb_decrypt::<Aes128, U11, U8>(key, ciphertext, iv, additional_data)?,
            (12, 64) => ocb_decrypt::<Aes128, U12, U8>(key, ciphertext, iv, additional_data)?,
            (13, 64) => ocb_decrypt::<Aes128, U13, U8>(key, ciphertext, iv, additional_data)?,
            (14, 64) => ocb_decrypt::<Aes128, U14, U8>(key, ciphertext, iv, additional_data)?,
            (15, 64) => ocb_decrypt::<Aes128, U15, U8>(key, ciphertext, iv, additional_data)?,

            (6, 96) => ocb_decrypt::<Aes128, U6, U12>(key, ciphertext, iv, additional_data)?,
            (7, 96) => ocb_decrypt::<Aes128, U7, U12>(key, ciphertext, iv, additional_data)?,
            (8, 96) => ocb_decrypt::<Aes128, U8, U12>(key, ciphertext, iv, additional_data)?,
            (9, 96) => ocb_decrypt::<Aes128, U9, U12>(key, ciphertext, iv, additional_data)?,
            (10, 96) => ocb_decrypt::<Aes128, U10, U12>(key, ciphertext, iv, additional_data)?,
            (11, 96) => ocb_decrypt::<Aes128, U11, U12>(key, ciphertext, iv, additional_data)?,
            (12, 96) => ocb_decrypt::<Aes128, U12, U12>(key, ciphertext, iv, additional_data)?,
            (13, 96) => ocb_decrypt::<Aes128, U13, U12>(key, ciphertext, iv, additional_data)?,
            (14, 96) => ocb_decrypt::<Aes128, U14, U12>(key, ciphertext, iv, additional_data)?,
            (15, 96) => ocb_decrypt::<Aes128, U15, U12>(key, ciphertext, iv, additional_data)?,

            (6, 128) => ocb_decrypt::<Aes128, U6, U16>(key, ciphertext, iv, additional_data)?,
            (7, 128) => ocb_decrypt::<Aes128, U7, U16>(key, ciphertext, iv, additional_data)?,
            (8, 128) => ocb_decrypt::<Aes128, U8, U16>(key, ciphertext, iv, additional_data)?,
            (9, 128) => ocb_decrypt::<Aes128, U9, U16>(key, ciphertext, iv, additional_data)?,
            (10, 128) => ocb_decrypt::<Aes128, U10, U16>(key, ciphertext, iv, additional_data)?,
            (11, 128) => ocb_decrypt::<Aes128, U11, U16>(key, ciphertext, iv, additional_data)?,
            (12, 128) => ocb_decrypt::<Aes128, U12, U16>(key, ciphertext, iv, additional_data)?,
            (13, 128) => ocb_decrypt::<Aes128, U13, U16>(key, ciphertext, iv, additional_data)?,
            (14, 128) => ocb_decrypt::<Aes128, U14, U16>(key, ciphertext, iv, additional_data)?,
            (15, 128) => ocb_decrypt::<Aes128, U15, U16>(key, ciphertext, iv, additional_data)?,

            _ => {
                return Err(Error::Operation(Some(format!(
                    "Unsupported IV size ({}-bytes) and/or tag length ({}-bit)",
                    iv.len(),
                    tag_length
                ))));
            },
        },
        Handle::Aes192Key(key) => match (iv.len(), tag_length) {
            (6, 64) => ocb_decrypt::<Aes192, U6, U8>(key, ciphertext, iv, additional_data)?,
            (7, 64) => ocb_decrypt::<Aes192, U7, U8>(key, ciphertext, iv, additional_data)?,
            (8, 64) => ocb_decrypt::<Aes192, U8, U8>(key, ciphertext, iv, additional_data)?,
            (9, 64) => ocb_decrypt::<Aes192, U9, U8>(key, ciphertext, iv, additional_data)?,
            (10, 64) => ocb_decrypt::<Aes192, U10, U8>(key, ciphertext, iv, additional_data)?,
            (11, 64) => ocb_decrypt::<Aes192, U11, U8>(key, ciphertext, iv, additional_data)?,
            (12, 64) => ocb_decrypt::<Aes192, U12, U8>(key, ciphertext, iv, additional_data)?,
            (13, 64) => ocb_decrypt::<Aes192, U13, U8>(key, ciphertext, iv, additional_data)?,
            (14, 64) => ocb_decrypt::<Aes192, U14, U8>(key, ciphertext, iv, additional_data)?,
            (15, 64) => ocb_decrypt::<Aes192, U15, U8>(key, ciphertext, iv, additional_data)?,

            (6, 96) => ocb_decrypt::<Aes192, U6, U12>(key, ciphertext, iv, additional_data)?,
            (7, 96) => ocb_decrypt::<Aes192, U7, U12>(key, ciphertext, iv, additional_data)?,
            (8, 96) => ocb_decrypt::<Aes192, U8, U12>(key, ciphertext, iv, additional_data)?,
            (9, 96) => ocb_decrypt::<Aes192, U9, U12>(key, ciphertext, iv, additional_data)?,
            (10, 96) => ocb_decrypt::<Aes192, U10, U12>(key, ciphertext, iv, additional_data)?,
            (11, 96) => ocb_decrypt::<Aes192, U11, U12>(key, ciphertext, iv, additional_data)?,
            (12, 96) => ocb_decrypt::<Aes192, U12, U12>(key, ciphertext, iv, additional_data)?,
            (13, 96) => ocb_decrypt::<Aes192, U13, U12>(key, ciphertext, iv, additional_data)?,
            (14, 96) => ocb_decrypt::<Aes192, U14, U12>(key, ciphertext, iv, additional_data)?,
            (15, 96) => ocb_decrypt::<Aes192, U15, U12>(key, ciphertext, iv, additional_data)?,

            (6, 128) => ocb_decrypt::<Aes192, U6, U16>(key, ciphertext, iv, additional_data)?,
            (7, 128) => ocb_decrypt::<Aes192, U7, U16>(key, ciphertext, iv, additional_data)?,
            (8, 128) => ocb_decrypt::<Aes192, U8, U16>(key, ciphertext, iv, additional_data)?,
            (9, 128) => ocb_decrypt::<Aes192, U9, U16>(key, ciphertext, iv, additional_data)?,
            (10, 128) => ocb_decrypt::<Aes192, U10, U16>(key, ciphertext, iv, additional_data)?,
            (11, 128) => ocb_decrypt::<Aes192, U11, U16>(key, ciphertext, iv, additional_data)?,
            (12, 128) => ocb_decrypt::<Aes192, U12, U16>(key, ciphertext, iv, additional_data)?,
            (13, 128) => ocb_decrypt::<Aes192, U13, U16>(key, ciphertext, iv, additional_data)?,
            (14, 128) => ocb_decrypt::<Aes192, U14, U16>(key, ciphertext, iv, additional_data)?,
            (15, 128) => ocb_decrypt::<Aes192, U15, U16>(key, ciphertext, iv, additional_data)?,

            _ => {
                return Err(Error::Operation(Some(format!(
                    "Unsupported IV size ({}-bytes) and/or tag length ({}-bit)",
                    iv.len(),
                    tag_length
                ))));
            },
        },
        Handle::Aes256Key(key) => match (iv.len(), tag_length) {
            (6, 64) => ocb_decrypt::<Aes256, U6, U8>(key, ciphertext, iv, additional_data)?,
            (7, 64) => ocb_decrypt::<Aes256, U7, U8>(key, ciphertext, iv, additional_data)?,
            (8, 64) => ocb_decrypt::<Aes256, U8, U8>(key, ciphertext, iv, additional_data)?,
            (9, 64) => ocb_decrypt::<Aes256, U9, U8>(key, ciphertext, iv, additional_data)?,
            (10, 64) => ocb_decrypt::<Aes256, U10, U8>(key, ciphertext, iv, additional_data)?,
            (11, 64) => ocb_decrypt::<Aes256, U11, U8>(key, ciphertext, iv, additional_data)?,
            (12, 64) => ocb_decrypt::<Aes256, U12, U8>(key, ciphertext, iv, additional_data)?,
            (13, 64) => ocb_decrypt::<Aes256, U13, U8>(key, ciphertext, iv, additional_data)?,
            (14, 64) => ocb_decrypt::<Aes256, U14, U8>(key, ciphertext, iv, additional_data)?,
            (15, 64) => ocb_decrypt::<Aes256, U15, U8>(key, ciphertext, iv, additional_data)?,

            (6, 96) => ocb_decrypt::<Aes256, U6, U12>(key, ciphertext, iv, additional_data)?,
            (7, 96) => ocb_decrypt::<Aes256, U7, U12>(key, ciphertext, iv, additional_data)?,
            (8, 96) => ocb_decrypt::<Aes256, U8, U12>(key, ciphertext, iv, additional_data)?,
            (9, 96) => ocb_decrypt::<Aes256, U9, U12>(key, ciphertext, iv, additional_data)?,
            (10, 96) => ocb_decrypt::<Aes256, U10, U12>(key, ciphertext, iv, additional_data)?,
            (11, 96) => ocb_decrypt::<Aes256, U11, U12>(key, ciphertext, iv, additional_data)?,
            (12, 96) => ocb_decrypt::<Aes256, U12, U12>(key, ciphertext, iv, additional_data)?,
            (13, 96) => ocb_decrypt::<Aes256, U13, U12>(key, ciphertext, iv, additional_data)?,
            (14, 96) => ocb_decrypt::<Aes256, U14, U12>(key, ciphertext, iv, additional_data)?,
            (15, 96) => ocb_decrypt::<Aes256, U15, U12>(key, ciphertext, iv, additional_data)?,

            (6, 128) => ocb_decrypt::<Aes256, U6, U16>(key, ciphertext, iv, additional_data)?,
            (7, 128) => ocb_decrypt::<Aes256, U7, U16>(key, ciphertext, iv, additional_data)?,
            (8, 128) => ocb_decrypt::<Aes256, U8, U16>(key, ciphertext, iv, additional_data)?,
            (9, 128) => ocb_decrypt::<Aes256, U9, U16>(key, ciphertext, iv, additional_data)?,
            (10, 128) => ocb_decrypt::<Aes256, U10, U16>(key, ciphertext, iv, additional_data)?,
            (11, 128) => ocb_decrypt::<Aes256, U11, U16>(key, ciphertext, iv, additional_data)?,
            (12, 128) => ocb_decrypt::<Aes256, U12, U16>(key, ciphertext, iv, additional_data)?,
            (13, 128) => ocb_decrypt::<Aes256, U13, U16>(key, ciphertext, iv, additional_data)?,
            (14, 128) => ocb_decrypt::<Aes256, U14, U16>(key, ciphertext, iv, additional_data)?,
            (15, 128) => ocb_decrypt::<Aes256, U15, U16>(key, ciphertext, iv, additional_data)?,

            _ => {
                return Err(Error::Operation(Some(format!(
                    "Unsupported IV size ({}-bytes) and/or tag length ({}-bit)",
                    iv.len(),
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

    // Step 6. Return plaintext.
    Ok(plaintext)
}

/// Helper for Step 5 of <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-decrypt>
fn ocb_decrypt<Cipher, NonceSize, TagSize>(
    key: &Key<Cipher>,
    ciphertext: &[u8],
    iv: &[u8],
    additional_data: &[u8],
) -> Result<Vec<u8>, Error>
where
    Cipher: BlockSizeUser<BlockSize = U16> + BlockEncrypt + KeyInit + BlockDecrypt,
    NonceSize: ArrayLength<u8> + IsGreaterOrEqual<U6> + IsLessOrEqual<U15>,
    GrEq<NonceSize, U6>: NonZero,
    LeEq<NonceSize, U15>: NonZero,
    TagSize: ArrayLength<u8> + NonZero + IsLessOrEqual<U16>,
    LeEq<TagSize, U16>: NonZero,
{
    let mut plaintext = ciphertext.to_vec();
    let mut cipher = Ocb3::<Cipher, NonceSize, TagSize>::new(key);
    cipher
        .decrypt_in_place(Nonce::from_slice(iv), additional_data, &mut plaintext)
        .map_err(|_| {
            Error::Operation(Some("AES-OCB authenticated decryption failed".to_string()))
        })?;
    Ok(plaintext)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-generate-key>
pub(crate) fn generate_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    aes_common::generate_key(
        AesAlgorithm::AesOcb,
        global,
        normalized_algorithm,
        extractable,
        usages,
        can_gc,
    )
}

/// <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    aes_common::import_key(
        AesAlgorithm::AesOcb,
        global,
        format,
        key_data,
        extractable,
        usages,
        can_gc,
    )
}

/// <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    aes_common::export_key(AesAlgorithm::AesOcb, format, key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-get-key-length>
pub(crate) fn get_key_length(
    normalized_derived_key_algorithm: &SubtleAesDerivedKeyParams,
) -> Result<Option<u32>, Error> {
    aes_common::get_key_length(normalized_derived_key_algorithm)
}
