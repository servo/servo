/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aes::{Aes128, Aes192, Aes256};
use cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr128BE;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::KeyUsage;
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyFormat;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::aes_common::AesAlgorithm;
use crate::dom::subtlecrypto::{
    ExportedKey, SubtleAesCtrParams, SubtleAesDerivedKeyParams, SubtleAesKeyGenParams, aes_common,
};
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-encrypt>
pub(crate) fn encrypt(
    normalized_algorithm: &SubtleAesCtrParams,
    key: &CryptoKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the counter member of normalizedAlgorithm does not have a length of 16 bytes,
    // then throw an OperationError.
    if normalized_algorithm.counter.len() != 16 {
        return Err(Error::Operation(Some(
            "The initial counter block length is not 16 bytes".into(),
        )));
    }

    // Step 2. If the length member of normalizedAlgorithm is zero or is greater than 128, then
    // throw an OperationError.
    if normalized_algorithm.length == 0 {
        return Err(Error::Operation(Some("The counter length is zero".into())));
    }
    if normalized_algorithm.length > 128 {
        return Err(Error::Operation(Some(
            "The counter length is greater than 128".into(),
        )));
    }

    // Step 3. Let ciphertext be the result of performing the CTR Encryption operation described in
    // Section 6.5 of [NIST-SP800-38A] using AES as the block cipher, the counter member of
    // normalizedAlgorithm as the initial value of the counter block, the length member of
    // normalizedAlgorithm as the input parameter m to the standard counter block incrementing
    // function defined in Appendix B.1 of [NIST-SP800-38A] and plaintext as the input plaintext.
    let iv = normalized_algorithm.counter.as_slice();
    let mut ciphertext = plaintext.to_vec();
    match key.handle() {
        Handle::Aes128Key(key) => {
            let mut cipher = Ctr128BE::<Aes128>::new(key, iv.into());
            cipher.apply_keystream(&mut ciphertext);
        },
        Handle::Aes192Key(key) => {
            let mut cipher = Ctr128BE::<Aes192>::new(key, iv.into());
            cipher.apply_keystream(&mut ciphertext);
        },
        Handle::Aes256Key(key) => {
            let mut cipher = Ctr128BE::<Aes256>::new(key, iv.into());
            cipher.apply_keystream(&mut ciphertext);
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

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-decrypt>
pub(crate) fn decrypt(
    normalized_algorithm: &SubtleAesCtrParams,
    key: &CryptoKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the counter member of normalizedAlgorithm does not have a length of 16 bytes,
    // then throw an OperationError.
    if normalized_algorithm.counter.len() != 16 {
        return Err(Error::Operation(Some(
            "The initial counter block length is not 16 bytes".into(),
        )));
    }

    // Step 2. If the length member of normalizedAlgorithm is zero or is greater than 128, then
    // throw an OperationError.
    if normalized_algorithm.length == 0 {
        return Err(Error::Operation(Some("The counter length is zero".into())));
    }
    if normalized_algorithm.length > 128 {
        return Err(Error::Operation(Some(
            "The counter length is greater than 128".into(),
        )));
    }

    // Step 3. Let plaintext be the result of performing the CTR Decryption operation described in
    // Section 6.5 of [NIST-SP800-38A] using AES as the block cipher, the counter member of
    // normalizedAlgorithm as the initial value of the counter block, the length member of
    // normalizedAlgorithm as the input parameter m to the standard counter block incrementing
    // function defined in Appendix B.1 of [NIST-SP800-38A] and ciphertext as the input ciphertext.
    let iv = normalized_algorithm.counter.as_slice();
    let mut plaintext = ciphertext.to_vec();
    match key.handle() {
        Handle::Aes128Key(key) => {
            let mut cipher = Ctr128BE::<Aes128>::new(key, iv.into());
            cipher.apply_keystream(&mut plaintext);
        },
        Handle::Aes192Key(key) => {
            let mut cipher = Ctr128BE::<Aes192>::new(key, iv.into());
            cipher.apply_keystream(&mut plaintext);
        },
        Handle::Aes256Key(key) => {
            let mut cipher = Ctr128BE::<Aes256>::new(key, iv.into());
            cipher.apply_keystream(&mut plaintext);
        },
        _ => {
            return Err(Error::Operation(Some(
                "The key handle is not representing an AES key".to_string(),
            )));
        },
    };

    // Step 4. Return plaintext.
    Ok(plaintext)
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-generate-key>
pub(crate) fn generate_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    aes_common::generate_key(
        AesAlgorithm::AesCtr,
        global,
        normalized_algorithm,
        extractable,
        usages,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    aes_common::import_key(
        AesAlgorithm::AesCtr,
        global,
        format,
        key_data,
        extractable,
        usages,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    aes_common::export_key(AesAlgorithm::AesCtr, format, key)
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-get-key-length>
pub(crate) fn get_key_length(
    normalized_derived_key_algorithm: &SubtleAesDerivedKeyParams,
) -> Result<Option<u32>, Error> {
    aes_common::get_key_length(normalized_derived_key_algorithm)
}
