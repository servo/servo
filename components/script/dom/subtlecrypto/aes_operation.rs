/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, StreamCipher};
use aes::{Aes128, Aes192, Aes256};
use aes_gcm::{AeadInPlace, AesGcm, KeyInit};
use cipher::consts::{U12, U16, U32};

use crate::dom::bindings::error::Error;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::subtlecrypto::{SubtleAesCbcParams, SubtleAesCtrParams, SubtleAesGcmParams};

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

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-encrypt>
pub(crate) fn encrypt_aes_ctr(
    normalized_algorithm: &SubtleAesCtrParams,
    key: &CryptoKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the counter member of normalizedAlgorithm does not have a length of 16 bytes,
    // then throw an OperationError.
    // Step 2. If the length member of normalizedAlgorithm is zero or is greater than 128, then
    // throw an OperationError.
    if normalized_algorithm.counter.len() != 16 ||
        normalized_algorithm.length == 0 ||
        normalized_algorithm.length > 128
    {
        return Err(Error::Operation);
    }

    // Step 3. Let ciphertext be the result of performing the CTR Encryption operation described in
    // Section 6.5 of [NIST-SP800-38A] using AES as the block cipher, the counter member of
    // normalizedAlgorithm as the initial value of the counter block, the length member of
    // normalizedAlgorithm as the input parameter m to the standard counter block incrementing
    // function defined in Appendix B.1 of [NIST-SP800-38A] and plaintext as the input plaintext.
    let mut ciphertext = Vec::from(plaintext);
    let counter = GenericArray::from_slice(&normalized_algorithm.counter);

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

    // Step 3. Return ciphertext.
    Ok(ciphertext)
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-encrypt>
pub(crate) fn decrypt_aes_ctr(
    normalized_algorithm: &SubtleAesCtrParams,
    key: &CryptoKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    // NOTE: Share implementation with `encrypt_aes_ctr`
    encrypt_aes_ctr(normalized_algorithm, key, ciphertext)
}

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-encrypt>
pub(crate) fn encrypt_aes_cbc(
    normalized_algorithm: &SubtleAesCbcParams,
    key: &CryptoKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the iv member of normalizedAlgorithm does not have a length of 16 bytes, then
    // throw an OperationError.
    if normalized_algorithm.iv.len() != 16 {
        return Err(Error::Operation);
    }

    // Step 2. Let paddedPlaintext be the result of adding padding octets to plaintext according to
    // the procedure defined in Section 10.3 of [RFC2315], step 2, with a value of k of 16.
    // Step 3. Let ciphertext be the result of performing the CBC Encryption operation described in
    // Section 6.2 of [NIST-SP800-38A] using AES as the block cipher, the iv member of
    // normalizedAlgorithm as the IV input parameter and paddedPlaintext as the input plaintext.
    let plaintext = Vec::from(plaintext);
    let iv = GenericArray::from_slice(&normalized_algorithm.iv);
    let ciphertext = match key.handle() {
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

    // Step 4. Return ciphertext.
    Ok(ciphertext)
}

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-decrypt>
pub(crate) fn decrypt_aes_cbc(
    normalized_algorithm: &SubtleAesCbcParams,
    key: &CryptoKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the iv member of normalizedAlgorithm does not have a length of 16 bytes, then
    // throw an OperationError.
    if normalized_algorithm.iv.len() != 16 {
        return Err(Error::Operation);
    }

    // Step 2. If the length of ciphertext is zero or is not a multiple of 16 bytes, then throw an
    // OperationError.
    if ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
        return Err(Error::Operation);
    }

    // Step 3. Let paddedPlaintext be the result of performing the CBC Decryption operation
    // described in Section 6.2 of [NIST-SP800-38A] using AES as the block cipher, the iv member of
    // normalizedAlgorithm as the IV input parameter and ciphertext as the input ciphertext.
    // Step 4. Let p be the value of the last octet of paddedPlaintext.
    // Step 5. If p is zero or greater than 16, or if any of the last p octets of paddedPlaintext
    // have a value which is not p, then throw an OperationError.
    // Step 6. Let plaintext be the result of removing p octets from the end of paddedPlaintext.
    let mut ciphertext = Vec::from(ciphertext);
    let iv = GenericArray::from_slice(&normalized_algorithm.iv);
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

    // Step 7. Return plaintext.
    Ok(plaintext.to_vec())
}

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-encrypt>
pub(crate) fn encrypt_aes_gcm(
    normalized_algorithm: &SubtleAesGcmParams,
    key: &CryptoKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If plaintext has a length greater than 2^39 - 256 bytes, then throw an OperationError.
    if plaintext.len() as u64 > (2 << 39) - 256 {
        return Err(Error::Operation);
    }

    // Step 2. If the iv member of normalizedAlgorithm has a length greater than 2^64 - 1 bytes,
    // then throw an OperationError.
    // NOTE: servo does not currently support 128-bit platforms, so this can never happen

    // Step 3. If the additionalData member of normalizedAlgorithm is present and has a length
    // greater than 2^64 - 1 bytes, then throw an OperationError.
    if normalized_algorithm
        .additional_data
        .as_ref()
        .is_some_and(|data| data.len() > u64::MAX as usize)
    {
        return Err(Error::Operation);
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
        Some(length) if matches!(length, 32 | 64 | 96 | 104 | 112 | 120 | 128) => length,
        _ => {
            return Err(Error::Operation);
        },
    };

    // Step 5. Let additionalData be the additionalData member of normalizedAlgorithm if present or
    // an empty byte sequence otherwise.
    let additional_data = normalized_algorithm
        .additional_data
        .as_deref()
        .unwrap_or_default();

    // Step 6. Let C and T be the outputs that result from performing the Authenticated Encryption
    // Function described in Section 7.1 of [NIST-SP800-38D] using AES as the block cipher, the
    // contents of the iv member of normalizedAlgorithm as the IV input parameter, the contents of
    // additionalData as the A input parameter, tagLength as the t pre-requisite and the contents
    // of plaintext as the input plaintext.
    let key_length = key.handle().as_bytes().len();
    let iv_length = normalized_algorithm.iv.len();
    let mut ciphertext = plaintext.to_vec();
    let key_bytes = key.handle().as_bytes();
    let tag = match (key_length, iv_length) {
        (16, 12) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes128Gcm96Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
        },
        (16, 16) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes128Gcm128Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
        },
        (24, 12) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes192Gcm96Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
        },
        (32, 12) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes256Gcm96Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
        },
        (16, 32) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes128Gcm256Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
        },
        (24, 32) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes192Gcm256Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .encrypt_in_place_detached(nonce, additional_data, &mut ciphertext)
        },
        (32, 32) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
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

    // Step 8. Return ciphertext.
    Ok(ciphertext)
}

/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-decrypt>
pub(crate) fn decrypt_aes_gcm(
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
        Some(length) if matches!(length, 32 | 64 | 96 | 104 | 112 | 120 | 128) => length as usize,
        _ => {
            return Err(Error::Operation);
        },
    };

    // Step 2. If ciphertext has a length in bits less than tagLength, then throw an
    // OperationError.
    if ciphertext.len() * 8 < tag_length {
        return Err(Error::Operation);
    }

    // Step 3. If the iv member of normalizedAlgorithm has a length greater than 2^64 - 1 bytes,
    // then throw an OperationError.
    // NOTE: servo does not currently support 128-bit platforms, so this can never happen

    // Step 4. If the additionalData member of normalizedAlgorithm is present and has a length
    // greater than 2^64 - 1 bytes, then throw an OperationError.
    // NOTE: servo does not currently support 128-bit platforms, so this can never happen

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
    // If the result of the algorithm is the indication of inauthenticity, "FAIL":
    //     throw an OperationError
    // Otherwise:
    //     Let plaintext be the output P of the Authenticated Decryption Function.
    let mut plaintext = ciphertext.to_vec();
    let key_length = key.handle().as_bytes().len();
    let iv_length = normalized_algorithm.iv.len();
    let key_bytes = key.handle().as_bytes();
    let result = match (key_length, iv_length) {
        (16, 12) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes128Gcm96Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .decrypt_in_place(nonce, additional_data, &mut plaintext)
        },
        (16, 16) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes128Gcm128Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .decrypt_in_place(nonce, additional_data, &mut plaintext)
        },
        (24, 12) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes192Gcm96Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .decrypt_in_place(nonce, additional_data, &mut plaintext)
        },
        (32, 12) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes256Gcm96Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .decrypt_in_place(nonce, additional_data, &mut plaintext)
        },
        (16, 32) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes128Gcm256Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .decrypt_in_place(nonce, additional_data, &mut plaintext)
        },
        (24, 32) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
            <Aes192Gcm256Iv>::new_from_slice(key_bytes)
                .expect("key length did not match")
                .decrypt_in_place(nonce, additional_data, &mut plaintext)
        },
        (32, 32) => {
            let nonce = GenericArray::from_slice(&normalized_algorithm.iv);
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
    if result.is_err() {
        return Err(Error::Operation);
    }

    Ok(plaintext)
}
