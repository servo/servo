/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aes::{Aes128, Aes192, Aes256};
use aes_kw::Kek;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::KeyUsage;
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyFormat;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::aes_common::AesAlgorithm;
use crate::dom::subtlecrypto::{
    ExportedKey, SubtleAesDerivedKeyParams, SubtleAesKeyGenParams, aes_common,
};
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-wrap-key>
pub(crate) fn wrap_key(key: &CryptoKey, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
    // Step 1. If plaintext is not a multiple of 64 bits in length, then throw an OperationError.
    if plaintext.len() % 8 != 0 {
        return Err(Error::Operation(Some(
            "The plaintext bit-length is not a multiple of 64".into(),
        )));
    }

    // Step 2. Let ciphertext be the result of performing the Key Wrap operation described in
    // Section 2.2.1 of [RFC3394] with plaintext as the plaintext to be wrapped and using the
    // default Initial Value defined in Section 2.2.3.1 of the same document.
    let ciphertext = match key.handle() {
        Handle::Aes128Key(key) => {
            let kek = Kek::<Aes128>::new(key);
            kek.wrap_vec(plaintext).map_err(|_| {
                Error::Operation(Some(
                    "AES-KW failed to perform the Key Wrap operation".to_string(),
                ))
            })?
        },
        Handle::Aes192Key(key) => {
            let kek = Kek::<Aes192>::new(key);
            kek.wrap_vec(plaintext).map_err(|_| {
                Error::Operation(Some(
                    "AES-KW failed to perform the Key Wrap operation".to_string(),
                ))
            })?
        },
        Handle::Aes256Key(key) => {
            let kek = Kek::<Aes256>::new(key);
            kek.wrap_vec(plaintext).map_err(|_| {
                Error::Operation(Some(
                    "AES-KW failed to perform the Key Wrap operation".to_string(),
                ))
            })?
        },
        _ => {
            return Err(Error::Operation(Some(
                "The key handle is not representing an AES key".to_string(),
            )));
        },
    };

    // Step 3. Return ciphertext.
    Ok(ciphertext)
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-unwrap-key>
pub(crate) fn unwrap_key(key: &CryptoKey, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
    // Step 1. Let plaintext be the result of performing the Key Unwrap operation described in
    // Section 2.2.2 of [RFC3394] with ciphertext as the input ciphertext and using the default
    // Initial Value defined in Section 2.2.3.1 of the same document.
    // Step 2. If the Key Unwrap operation returns an error, then throw an OperationError.
    let plaintext = match key.handle() {
        Handle::Aes128Key(key) => {
            let kek = Kek::<Aes128>::new(key);
            kek.unwrap_vec(ciphertext).map_err(|_| {
                Error::Operation(Some(
                    "AES-KW failed to perform the Key Unwrap operation".to_string(),
                ))
            })?
        },
        Handle::Aes192Key(key) => {
            let kek = Kek::<Aes192>::new(key);
            kek.unwrap_vec(ciphertext).map_err(|_| {
                Error::Operation(Some(
                    "AES-KW failed to perform the Key Unwrap operation".to_string(),
                ))
            })?
        },
        Handle::Aes256Key(key) => {
            let kek = Kek::<Aes256>::new(key);
            kek.unwrap_vec(ciphertext).map_err(|_| {
                Error::Operation(Some(
                    "AES-KW failed to perform the Key Unwrap operation".to_string(),
                ))
            })?
        },
        _ => {
            return Err(Error::Operation(Some(
                "The key handle is not representing an AES key".to_string(),
            )));
        },
    };

    // Step 3. Return plaintext.
    Ok(plaintext)
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-generate-key>
pub(crate) fn generate_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    aes_common::generate_key(
        AesAlgorithm::AesKw,
        global,
        normalized_algorithm,
        extractable,
        usages,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    aes_common::import_key(
        AesAlgorithm::AesKw,
        global,
        format,
        key_data,
        extractable,
        usages,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    aes_common::export_key(AesAlgorithm::AesKw, format, key)
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-get-key-length>
pub(crate) fn get_key_length(
    normalized_derived_key_algorithm: &SubtleAesDerivedKeyParams,
) -> Result<Option<u32>, Error> {
    aes_common::get_key_length(normalized_derived_key_algorithm)
}
