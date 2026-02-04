/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aes::{Aes128, Aes192, Aes256};
use cbc::{Decryptor, Encryptor};
use cipher::block_padding::Pkcs7;
use cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::KeyUsage;
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyFormat;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::aes_common::AesAlgorithm;
use crate::dom::subtlecrypto::{
    ExportedKey, SubtleAesCbcParams, SubtleAesDerivedKeyParams, SubtleAesKeyGenParams, aes_common,
};
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-encrypt>
pub(crate) fn encrypt(
    normalized_algorithm: &SubtleAesCbcParams,
    key: &CryptoKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the iv member of normalizedAlgorithm does not have a length of 16 bytes, then
    // throw an OperationError.
    if normalized_algorithm.iv.len() != 16 {
        return Err(Error::Operation(Some(
            "The initialization vector length is not 16 bytes".into(),
        )));
    }

    // Step 2. Let paddedPlaintext be the result of adding padding octets to plaintext according to
    // the procedure defined in Section 10.3 of [RFC2315], step 2, with a value of k of 16.
    // Step 3. Let ciphertext be the result of performing the CBC Encryption operation described in
    // Section 6.2 of [NIST-SP800-38A] using AES as the block cipher, the iv member of
    // normalizedAlgorithm as the IV input parameter and paddedPlaintext as the input plaintext.
    let iv = normalized_algorithm.iv.as_slice();
    let ciphertext = match key.handle() {
        Handle::Aes128Key(key) => {
            let encryptor = Encryptor::<Aes128>::new(key, iv.into());
            encryptor.encrypt_padded_vec_mut::<Pkcs7>(plaintext)
        },
        Handle::Aes192Key(key) => {
            let encryptor = Encryptor::<Aes192>::new(key, iv.into());
            encryptor.encrypt_padded_vec_mut::<Pkcs7>(plaintext)
        },
        Handle::Aes256Key(key) => {
            let encryptor = Encryptor::<Aes256>::new(key, iv.into());
            encryptor.encrypt_padded_vec_mut::<Pkcs7>(plaintext)
        },
        _ => {
            return Err(Error::Operation(Some(
                "The key handle is not representing an AES key".to_string(),
            )));
        },
    };

    // Step 4. Return ciphertext.
    Ok(ciphertext)
}

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-decrypt>
pub(crate) fn decrypt(
    normalized_algorithm: &SubtleAesCbcParams,
    key: &CryptoKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the iv member of normalizedAlgorithm does not have a length of 16 bytes, then
    // throw an OperationError.
    if normalized_algorithm.iv.len() != 16 {
        return Err(Error::Operation(Some(
            "The initialization vector length is not 16 bytes".into(),
        )));
    }

    // Step 2. If the length of ciphertext is zero or is not a multiple of 16 bytes, then throw an
    // OperationError.
    if ciphertext.is_empty() {
        return Err(Error::Operation(Some("The ciphertext is empty".into())));
    }
    if ciphertext.len() % 16 != 0 {
        return Err(Error::Operation(Some(
            "The ciphertext length is not a multiple of 16 bytes".into(),
        )));
    }

    // Step 3. Let paddedPlaintext be the result of performing the CBC Decryption operation
    // described in Section 6.2 of [NIST-SP800-38A] using AES as the block cipher, the iv member of
    // normalizedAlgorithm as the IV input parameter and ciphertext as the input ciphertext.
    // Step 4. Let p be the value of the last octet of paddedPlaintext.
    // Step 5. If p is zero or greater than 16, or if any of the last p octets of paddedPlaintext
    // have a value which is not p, then throw an OperationError.
    // Step 6. Let plaintext be the result of removing p octets from the end of paddedPlaintext.
    let iv = normalized_algorithm.iv.as_slice();
    let plaintext = match key.handle() {
        Handle::Aes128Key(key) => {
            let decryptor = Decryptor::<Aes128>::new(key, iv.into());
            decryptor
                .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
                .map_err(|_| {
                    Error::Operation(Some("Failed to perform AES-CBC decryption".to_string()))
                })?
        },
        Handle::Aes192Key(key) => {
            let decryptor = Decryptor::<Aes192>::new(key, iv.into());
            decryptor
                .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
                .map_err(|_| {
                    Error::Operation(Some("Failed to perform AES-CBC decryption".to_string()))
                })?
        },
        Handle::Aes256Key(key) => {
            let decryptor = Decryptor::<Aes256>::new(key, iv.into());
            decryptor
                .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
                .map_err(|_| {
                    Error::Operation(Some("Failed to perform AES-CBC decryption".to_string()))
                })?
        },
        _ => {
            return Err(Error::Operation(Some(
                "The key handle is not representing an AES key".to_string(),
            )));
        },
    };
    // Step 7. Return plaintext.
    Ok(plaintext)
}

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-generate-key>
pub(crate) fn generate_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    aes_common::generate_key(
        AesAlgorithm::AesCbc,
        global,
        normalized_algorithm,
        extractable,
        usages,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    aes_common::import_key(
        AesAlgorithm::AesCbc,
        global,
        format,
        key_data,
        extractable,
        usages,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    aes_common::export_key(AesAlgorithm::AesCbc, format, key)
}

/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-get-key-length>
pub(crate) fn get_key_length(
    normalized_derived_key_algorithm: &SubtleAesDerivedKeyParams,
) -> Result<Option<u32>, Error> {
    aes_common::get_key_length(normalized_derived_key_algorithm)
}
