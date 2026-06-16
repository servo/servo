/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aes_kw::{KeyInit, KwAes128, KwAes192, KwAes256};
use js::context::JSContext;
use zeroize::Zeroizing;

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

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-wrap-key>
pub(crate) fn wrap_key(key: &CryptoKey, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
    // Step 1. If plaintext is not a multiple of 64 bits in length, then throw an OperationError.
    if !plaintext.len().is_multiple_of(8) {
        return Err(Error::Operation(Some(
            "The plaintext bit-length is not a multiple of 64".into(),
        )));
    }

    // Step 2. Let ciphertext be the result of performing the Key Wrap operation described in
    // Section 2.2.1 of [RFC3394] with plaintext as the plaintext to be wrapped and using the
    // default Initial Value defined in Section 2.2.3.1 of the same document.
    // NOTE: The length of buffer must be greater or equal to length of plaintext plus 8 bytes.
    let mut buffer = vec![0u8; plaintext.len() + 8];
    let ciphertext = match key.handle() {
        Handle::Aes128Key(key) => {
            let key_wrapper = KwAes128::new(key);
            key_wrapper.wrap_key(plaintext, &mut buffer)
        },
        Handle::Aes192Key(key) => {
            let key_wrapper = KwAes192::new(key);
            key_wrapper.wrap_key(plaintext, &mut buffer)
        },
        Handle::Aes256Key(key) => {
            let key_wrapper = KwAes256::new(key);
            key_wrapper.wrap_key(plaintext, &mut buffer)
        },
        _ => {
            return Err(Error::Operation(Some(
                "The key handle is not representing an AES key".to_string(),
            )));
        },
    }
    .map_err(|_| {
        Error::Operation(Some(
            "AES-KW failed to perform the Key Wrap operation".into(),
        ))
    })?;

    // Step 3. Return ciphertext.
    Ok(ciphertext.to_vec())
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-unwrap-key>
pub(crate) fn unwrap_key(key: &CryptoKey, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
    // Step 1. Let plaintext be the result of performing the Key Unwrap operation described in
    // Section 2.2.2 of [RFC3394] with ciphertext as the input ciphertext and using the default
    // Initial Value defined in Section 2.2.3.1 of the same document.
    // Step 2. If the Key Unwrap operation returns an error, then throw an OperationError.
    let mut buffer = Zeroizing::new(vec![0u8; ciphertext.len()]);
    let plaintext = match key.handle() {
        Handle::Aes128Key(key) => {
            let key_unwrapper = KwAes128::new(key);
            key_unwrapper.unwrap_key(ciphertext, &mut buffer)
        },
        Handle::Aes192Key(key) => {
            let key_unwrapper = KwAes192::new(key);
            key_unwrapper.unwrap_key(ciphertext, &mut buffer)
        },
        Handle::Aes256Key(key) => {
            let key_unwrapper = KwAes256::new(key);
            key_unwrapper.unwrap_key(ciphertext, &mut buffer)
        },
        _ => {
            return Err(Error::Operation(Some(
                "The key handle is not representing an AES key".to_string(),
            )));
        },
    }
    .map_err(|_| {
        Error::Operation(Some(
            "AES-KW failed to perform the Key Unwrap operation".into(),
        ))
    })?;

    // Step 3. Return plaintext.
    Ok(plaintext.to_vec())
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-generate-key>
pub(crate) fn generate_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    aes_common::generate_key(
        AesAlgorithm::AesKw,
        cx,
        global,
        normalized_algorithm,
        extractable,
        usages,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-import-key>
pub(crate) fn import_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    aes_common::import_key(
        AesAlgorithm::AesKw,
        cx,
        global,
        format,
        key_data,
        extractable,
        usages,
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
