/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base64ct::{Base64UrlUnpadded, Encoding};
use chacha20poly1305::aead::{AeadMutInPlace, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Key};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{JsonWebKey, KeyFormat};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_CHACHA20_POLY1305, ExportedKey, JsonWebKeyExt, JwkStringField, KeyAlgorithmAndDerivatives,
    SubtleAeadParams, SubtleKeyAlgorithm,
};
use crate::script_runtime::CanGc;

/// <https://wicg.github.io/webcrypto-modern-algos/#chacha20-poly1305-operations-encrypt>
pub(crate) fn encrypt(
    normalized_algorithm: &SubtleAeadParams,
    key: &CryptoKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the iv member of normalizedAlgorithm does not have a length of 12 bytes, then
    // throw an OperationError.
    if normalized_algorithm.iv.len() != 12 {
        return Err(Error::Operation(Some(
            "The iv member of normalizedAlgorithm does not have a length of 12 bytes".to_string(),
        )));
    }

    // Step 2. If the tagLength member of normalizedAlgorithm is present and is not 128, then throw
    // an OperationError.
    if normalized_algorithm
        .tag_length
        .is_some_and(|tag_length| tag_length != 128)
    {
        return Err(Error::Operation(Some(
            "The tagLength member of normalizedAlgorithm is present and is not 128".to_string(),
        )));
    }

    // Step 3. Let additionalData be the additionalData member of normalizedAlgorithm if present or
    // the empty octet string otherwise.
    let additional_data = normalized_algorithm
        .additional_data
        .as_deref()
        .unwrap_or_default();

    // Step 4. Let ciphertext be the output that results from performing the AEAD_CHACHA20_POLY1305
    // encryption algorithm described in Section 2.8 of [RFC8439], using the key represented by
    // [[handle]] internal slot of key as the key input parameter, the iv member of
    // normalizedAlgorithm as the nonce input parameter, plaintext as the plaintext input
    // parameter, and additionalData as the additional authenticated data (AAD) input parameter.
    let Handle::ChaCha20Poly1305Key(handle) = key.handle() else {
        return Err(Error::Operation(Some(
            "Unable to access key represented by [[handle]] internal slot".to_string(),
        )));
    };
    let mut cipher = ChaCha20Poly1305::new(handle);
    let nonce = normalized_algorithm.iv.as_slice();
    let mut ciphertext = plaintext.to_vec();
    cipher
        .encrypt_in_place(nonce.into(), additional_data, &mut ciphertext)
        .map_err(|_| {
            Error::Operation(Some(
                "ChaCha20-Poly1305 fails to encrypt plaintext".to_string(),
            ))
        })?;

    // Step 5. Return ciphertext.
    Ok(ciphertext)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#chacha20-poly1305-operations-decrypt>
pub(crate) fn decrypt(
    normalized_algorithm: &SubtleAeadParams,
    key: &CryptoKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the iv member of normalizedAlgorithm does not have a length of 12 bytes, then
    // throw an OperationError.
    if normalized_algorithm.iv.len() != 12 {
        return Err(Error::Operation(Some(
            "The iv member of normalizedAlgorithm does not have a length of 12 bytes".to_string(),
        )));
    }

    // Step 2. If the tagLength member of normalizedAlgorithm is present and is not 128, then throw
    // an OperationError.
    if normalized_algorithm
        .tag_length
        .is_some_and(|tag_length| tag_length != 128)
    {
        return Err(Error::Operation(Some(
            "The tagLength member of normalizedAlgorithm is present and is not 128".to_string(),
        )));
    }

    // Step 3. If ciphertext has a length less than 128 bits, then throw an OperationError.
    if ciphertext.len() < 16 {
        return Err(Error::Operation(Some(
            "Ciphertext has a length less than 128 bits".to_string(),
        )));
    }

    // Step 4. Let additionalData be the additionalData member of normalizedAlgorithm if present or
    // the empty octet string otherwise.
    let additional_data = normalized_algorithm
        .additional_data
        .as_deref()
        .unwrap_or_default();

    // Step 5. Perform the AEAD_CHACHA20_POLY1305 decryption algorithm described in Section 2.8 of
    // [RFC8439], using the key represented by [[handle]] internal slot of key as the key input
    // parameter, the iv member of normalizedAlgorithm as the nonce input parameter, ciphertext as
    // the ciphertext input parameter, and additionalData as the additional authenticated data
    // (AAD) input parameter.
    //
    // If the result of the algorithm is the indication of authentication failure:
    //     throw an OperationError
    // Otherwise:
    //     Let plaintext be the resulting plaintext.
    let Handle::ChaCha20Poly1305Key(handle) = key.handle() else {
        return Err(Error::Operation(Some(
            "Unable to access key represented by [[handle]] internal slot".to_string(),
        )));
    };
    let mut cipher = ChaCha20Poly1305::new(handle);
    let nonce = normalized_algorithm.iv.as_slice();
    let mut plaintext = ciphertext.to_vec();
    cipher
        .decrypt_in_place(nonce.into(), additional_data, &mut plaintext)
        .map_err(|_| {
            Error::Operation(Some(
                "ChaCha20-Poly1305 fails to decrypt ciphertext".to_string(),
            ))
        })?;

    // Step 6. Return plaintext.
    Ok(plaintext)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#chacha20-poly1305-operations-generate-key>
pub(crate) fn generate_key(
    global: &GlobalScope,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. If usages contains any entry which is not one of "encrypt", "decrypt", "wrapKey" or
    // "unwrapKey", then throw a SyntaxError.
    if usages.iter().any(|usage| {
        !matches!(
            usage,
            KeyUsage::Encrypt | KeyUsage::Decrypt | KeyUsage::WrapKey | KeyUsage::UnwrapKey
        )
    }) {
        return Err(Error::Syntax(Some(
            "Usages contains an entry which is not one of \"encrypt\", \"decrypt\", \"wrapKey\" \
            or \"unwrapKey\""
                .to_string(),
        )));
    }

    // Step 2. Generate a 256-bit key.
    // Step 3. If the key generation step fails, then throw an OperationError.
    let generated_key = ChaCha20Poly1305::generate_key(&mut OsRng);

    // Step 4. Let key be a new CryptoKey object representing the generated key.
    // Step 5. Set the [[type]] internal slot of key to "secret".
    // Step 6. Let algorithm be a new KeyAlgorithm.
    // Step 7. Set the name attribute of algorithm to "ChaCha20-Poly1305".
    // Step 8. Set the [[algorithm]] internal slot of key to algorithm.
    // Step 9. Set the [[extractable]] internal slot of key to be extractable.
    // Step 10. Set the [[usages]] internal slot of key to be usages.
    let algorithm = SubtleKeyAlgorithm {
        name: ALG_CHACHA20_POLY1305.to_string(),
    };
    let key = CryptoKey::new(
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
        usages,
        Handle::ChaCha20Poly1305Key(generated_key),
        can_gc,
    );

    // Step 11. Return key.
    Ok(key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#chacha20-poly1305-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2. If usages contains an entry which is not one of "encrypt", "decrypt", "wrapKey" or
    // "unwrapKey", then throw a SyntaxError.
    if usages.iter().any(|usage| {
        !matches!(
            usage,
            KeyUsage::Encrypt | KeyUsage::Decrypt | KeyUsage::WrapKey | KeyUsage::UnwrapKey
        )
    }) {
        return Err(Error::Syntax(Some(
            "Usages contains an entry which is not one of \"encrypt\", \"decrypt\", \"wrapKey\" \
            or \"unwrapKey\""
                .to_string(),
        )));
    }

    // Step 3.
    let data;
    match format {
        // If format is "raw-secret":
        KeyFormat::Raw_secret => {
            // Step 3.1. Let data be keyData.
            data = key_data.to_vec();

            // Step 3.2. If the length in bits of data is not 256 then throw a DataError.
            if data.len() != 32 {
                return Err(Error::Data(Some(
                    "The length in bits of data is not 256".to_string(),
                )));
            }
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 3.1.
            // If keyData is a JsonWebKey dictionary:
            //     Let jwk equal keyData.
            // Otherwise:
            //     Throw a DataError.
            let jwk = JsonWebKey::parse(GlobalScope::get_cx(), key_data)?;

            // Step 3.2. If the kty field of jwk is not "oct", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "oct") {
                return Err(Error::Data(Some(
                    "The kty field of jwk is not \"oct\"".to_string(),
                )));
            }

            // Step 3.3. If jwk does not meet the requirements of Section 6.4 of JSON Web
            // Algorithms [JWA], then throw a DataError.
            // Step 3.4. Let data be the byte sequence obtained by decoding the k field of jwk.
            data = jwk.decode_required_string_field(JwkStringField::K)?;

            // Step 3.5. If the alg field of jwk is present, and is not "C20P", then throw a
            // DataError.
            if jwk.alg.as_ref().is_some_and(|alg| alg != "C20P") {
                return Err(Error::Data(Some(
                    "The alg field of jwk is present, and is not \"C20P\"".to_string(),
                )));
            }

            // Step 3.6. If usages is non-empty and the use field of jwk is present and is not
            // "enc", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                return Err(Error::Data(Some(
                    "Usages is non-empty and the use field of jwk is present and is not \"enc\""
                        .to_string(),
                )));
            }

            // Step 3.7. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK] or does not contain all of the specified usages
            // values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 3.8. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data(Some(
                    "The ext field of jwk is present and has the value false and \
                    extractable is true"
                        .to_string(),
                )));
            }
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "ChaCha20-Poly1305 does not support this import key format".to_string(),
            )));
        },
    }

    // Step 4. Let key be a new CryptoKey object representing a key with value data.
    // Step 5. Set the [[type]] internal slot of key to "secret".
    // Step 6. Let algorithm be a new KeyAlgorithm.
    // Step 7. Set the name attribute of algorithm to "ChaCha20-Poly1305".
    // Step 8. Set the [[algorithm]] internal slot of key to algorithm.
    let handle = Handle::ChaCha20Poly1305Key(Key::from_exact_iter(data).ok_or(Error::Data(
        Some("ChaCha20-Poly1305 fails to create key from data".to_string()),
    ))?);
    let algorithm = SubtleKeyAlgorithm {
        name: ALG_CHACHA20_POLY1305.to_string(),
    };
    let key = CryptoKey::new(
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
        usages,
        handle,
        can_gc,
    );

    // Step 9. Return key.
    Ok(key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#chacha20-poly1305-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    // Step 1. If the underlying cryptographic key material represented by the [[handle]] internal
    // slot of key cannot be accessed, then throw an OperationError.
    let Handle::ChaCha20Poly1305Key(key_handle) = key.handle() else {
        return Err(Error::Operation(Some(
            "The underlying cryptographic key material represented by the [[handle]] internal \
            slot of key cannot be accessed"
                .to_string(),
        )));
    };

    // Step 2.
    let result = match format {
        // If format is "raw-secret":
        KeyFormat::Raw_secret => {
            // Step 2.1. Let data be a byte sequence containing the raw octets of the key
            // represented by [[handle]] internal slot of key.
            let data = key_handle.to_vec();

            // Step 2.2 Let result be data.
            ExportedKey::Bytes(data)
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1. Let jwk be a new JsonWebKey dictionary.
            // Step 2.2. Set the kty attribute of jwk to the string "oct".
            // Step 2.3. Set the k attribute of jwk to be a string containing the raw octets of the
            // key represented by [[handle]] internal slot of key, encoded according to Section 6.4
            // of JSON Web Algorithms [JWA].
            // Step 2.4. Set the alg attribute of jwk to the string "C20P".
            // Step 2.5. Set the key_ops attribute of jwk to equal the usages attribute of key.
            // Step 2.6. Set the ext attribute of jwk to equal the [[extractable]] internal slot of
            // key.
            let jwk = JsonWebKey {
                kty: Some(DOMString::from("oct")),
                k: Some(Base64UrlUnpadded::encode_string(key_handle.as_slice()).into()),
                alg: Some(DOMString::from("C20P")),
                key_ops: Some(
                    key.usages()
                        .iter()
                        .map(|usage| DOMString::from(usage.as_str()))
                        .collect::<Vec<DOMString>>(),
                ),
                ext: Some(key.Extractable()),
                ..Default::default()
            };

            // Step 2.7. Let result be jwk.
            ExportedKey::Jwk(Box::new(jwk))
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "ChaCha20-Poly1305 does not support this import key format".to_string(),
            )));
        },
    };

    // Step 3. Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#chacha20-poly1305-operations-get-key-length>
pub(crate) fn get_key_length() -> Result<Option<u32>, Error> {
    // Step 1. Return 256.
    Ok(Some(256))
}
