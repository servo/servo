/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aes::cipher::generic_array::GenericArray;
use aes_kw::{KekAes128, KekAes192, KekAes256};
use rand::TryRngCore;
use rand::rngs::OsRng;

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
    ALG_AES_KW, ExportedKey, JsonWebKeyExt, JwkStringField, KeyAlgorithmAndDerivatives,
    SubtleAesDerivedKeyParams, SubtleAesKeyAlgorithm, SubtleAesKeyGenParams,
};
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-wrap-key>
pub(crate) fn wrap_key_aes_kw(key: &CryptoKey, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
    // Step 1. If plaintext is not a multiple of 64 bits in length, then throw an OperationError.
    if !plaintext.len().is_multiple_of(8) {
        return Err(Error::Operation(Some(
            "The plaintext bit-length is not a multiple of 64".into(),
        )));
    }

    // Step 2. Let ciphertext be the result of performing the Key Wrap operation described in
    // Section 2.2.1 of [RFC3394] with plaintext as the plaintext to be wrapped and using the
    // default Initial Value defined in Section 2.2.3.1 of the same document.
    let key_data = key.handle().as_bytes();
    let ciphertext = match key_data.len() {
        16 => {
            let key_array = GenericArray::from_slice(key_data);
            let kek = KekAes128::new(key_array);
            match kek.wrap_vec(plaintext) {
                Ok(key) => key,
                Err(_) => {
                    return Err(Error::Operation(Some(
                        "Plaintext key wrapping failed".into(),
                    )));
                },
            }
        },
        24 => {
            let key_array = GenericArray::from_slice(key_data);
            let kek = KekAes192::new(key_array);
            match kek.wrap_vec(plaintext) {
                Ok(key) => key,
                Err(_) => {
                    return Err(Error::Operation(Some(
                        "Plaintext key wrapping failed".into(),
                    )));
                },
            }
        },
        32 => {
            let key_array = GenericArray::from_slice(key_data);
            let kek = KekAes256::new(key_array);
            match kek.wrap_vec(plaintext) {
                Ok(key) => key,
                Err(_) => {
                    return Err(Error::Operation(Some(
                        "Plaintext key wrapping failed".into(),
                    )));
                },
            }
        },
        _ => return Err(Error::Operation(Some("Key length is invalid".into()))),
    };

    // Step 3. Return ciphertext.
    Ok(ciphertext)
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-unwrap-key>
pub(crate) fn unwrap_key_aes_kw(key: &CryptoKey, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
    // Step 1. Let plaintext be the result of performing the Key Unwrap operation described in
    // Section 2.2.2 of [RFC3394] with ciphertext as the input ciphertext and using the default
    // Initial Value defined in Section 2.2.3.1 of the same document.
    // Step 2. If the Key Unwrap operation returns an error, then throw an OperationError.
    let key_data = key.handle().as_bytes();
    let plaintext = match key_data.len() {
        16 => {
            let key_array = GenericArray::from_slice(key_data);
            let kek = KekAes128::new(key_array);
            match kek.unwrap_vec(ciphertext) {
                Ok(key) => key,
                Err(_) => {
                    return Err(Error::Operation(Some(
                        "Ciphertext key unwrapping failed".into(),
                    )));
                },
            }
        },
        24 => {
            let key_array = GenericArray::from_slice(key_data);
            let kek = KekAes192::new(key_array);
            match kek.unwrap_vec(ciphertext) {
                Ok(key) => key,
                Err(_) => {
                    return Err(Error::Operation(Some(
                        "Ciphertext key unwrapping failed".into(),
                    )));
                },
            }
        },
        32 => {
            let key_array = GenericArray::from_slice(key_data);
            let kek = KekAes256::new(key_array);
            match kek.unwrap_vec(ciphertext) {
                Ok(key) => key,
                Err(_) => {
                    return Err(Error::Operation(Some(
                        "Ciphertext key unwrapping failed".into(),
                    )));
                },
            }
        },
        _ => {
            return Err(Error::Operation(Some(
                "Key length is not 128, 192, or 256 bits".into(),
            )));
        },
    };

    // Step 3. Return plaintext.
    Ok(plaintext)
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-generate-key>
pub(crate) fn generate_key_aes_kw(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    generate_key_aes(
        global,
        normalized_algorithm,
        extractable,
        usages,
        ALG_AES_KW,
        &[KeyUsage::WrapKey, KeyUsage::UnwrapKey],
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-import-key>
pub(crate) fn import_key_aes_kw(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    import_key_aes(
        global,
        format,
        key_data,
        extractable,
        usages,
        ALG_AES_KW,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-export-key>
pub(crate) fn export_key_aes_kw(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    export_key_aes(format, key)
}

/// <https://w3c.github.io/webcrypto/#aes-kw-operations-get-key-length>
pub(crate) fn get_key_length_aes_kw(
    normalized_derived_key_algorithm: &SubtleAesDerivedKeyParams,
) -> Result<Option<u32>, Error> {
    get_key_length_aes(normalized_derived_key_algorithm)
}

/// Helper function for
/// <https://w3c.github.io/webcrypto/#aes-kw-operations-generate-key>
#[allow(clippy::too_many_arguments)]
fn generate_key_aes(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    alg_name: &str,
    allowed_usages: &[KeyUsage],
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. If usages contains any entry which is not one of allowed_usages, then throw a SyntaxError.
    if usages.iter().any(|usage| !allowed_usages.contains(usage)) {
        return Err(Error::Syntax(Some(
            "Key generation usage is not allowed".into(),
        )));
    }

    // Step 2. If the length member of normalizedAlgorithm is not equal to one of 128, 192 or 256,
    // then throw an OperationError.
    if !matches!(normalized_algorithm.length, 128 | 192 | 256) {
        return Err(Error::Operation(Some(
            "Key length is not 128, 192, or 256 bits".into(),
        )));
    }

    // Step 3. Generate an AES key of length equal to the length member of normalizedAlgorithm.
    // Step 4. If the key generation step fails, then throw an OperationError.
    let mut rand = vec![0; normalized_algorithm.length as usize / 8];
    if OsRng.try_fill_bytes(&mut rand).is_err() {
        return Err(Error::JSFailed);
    }

    let handle = match normalized_algorithm.length {
        128 => Handle::Aes128(rand),
        192 => Handle::Aes192(rand),
        256 => Handle::Aes256(rand),
        _ => {
            return Err(Error::Operation(Some(
                "Key length is not 128, 192, or 256 bits".into(),
            )));
        },
    };

    // Step 6. Let algorithm be a new AesKeyAlgorithm.
    // Step 7. Set the name attribute of algorithm to alg_name.
    // Step 8. Set the length attribute of algorithm to equal the length member of normalizedAlgorithm.
    let algorithm = SubtleAesKeyAlgorithm {
        name: alg_name.to_string(),
        length: normalized_algorithm.length,
    };

    // Step 5. Let key be a new CryptoKey object representing the generated AES key.
    // Step 9. Set the [[type]] internal slot of key to "secret".
    // Step 10. Set the [[algorithm]] internal slot of key to algorithm.
    // Step 11. Set the [[extractable]] internal slot of key to be extractable.
    // Step 12. Set the [[usages]] internal slot of key to be usages.
    let crypto_key = CryptoKey::new(
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::AesKeyAlgorithm(algorithm),
        usages,
        handle,
        can_gc,
    );

    // Step 13. Return key.
    Ok(crypto_key)
}

/// Helper function for
/// <https://w3c.github.io/webcrypto/#aes-kw-operations-import-key>
fn import_key_aes(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    alg_name: &str,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. If usages contains an entry which is not one of "encrypt", "decrypt", "wrapKey" or
    // "unwrapKey", then throw a SyntaxError.
    if usages.iter().any(|usage| {
        !matches!(
            usage,
            KeyUsage::Encrypt | KeyUsage::Decrypt | KeyUsage::WrapKey | KeyUsage::UnwrapKey
        )
    }) || usages.is_empty()
    {
        return Err(Error::Syntax(Some(
            "Key generation usage is not allowed".into(),
        )));
    }

    // Step 2.
    let data;
    match format {
        // If format is "raw":
        KeyFormat::Raw | KeyFormat::Raw_secret => {
            // Step 2.1. Let data be keyData.
            data = key_data.to_vec();

            // Step 2.2. If the length in bits of data is not 128, 192 or 256 then throw a DataError.
            if !matches!(data.len() * 8, 128 | 192 | 256) {
                return Err(Error::Data(Some(
                    "Key length is not 128, 192, or 256 bits".into(),
                )));
            }
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1. If keyData is a JsonWebKey dictionary: Let jwk equal keyData.
            // Otherwise: Throw a DataError.
            // NOTE: Deserialize keyData to JsonWebKey dictionary by calling JsonWebKey::parse
            let jwk = JsonWebKey::parse(GlobalScope::get_cx(), key_data)?;

            // Step 2.2. If the kty field of jwk is not "oct", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "oct") {
                return Err(Error::Data(Some("JWK `kty` field is not \"oct\"".into())));
            }

            // Step 2.3. If jwk does not meet the requirements of Section 6.4 of JSON Web
            // Algorithms [JWA], then throw a DataError.
            // NOTE: Done by Step 2.4 and 2.5.

            // Step 2.4. Let data be the byte sequence obtained by decoding the k field of jwk.
            data = jwk.decode_required_string_field(JwkStringField::K)?;

            // Different static texts are used in different AES types, in the following step.
            let alg_matching = match alg_name {
                ALG_AES_KW => ["A128KW", "A192KW", "A256KW"],
                _ => unreachable!(),
            };

            // Step 2.5.
            match data.len() * 8 {
                // If the length in bits of data is 128:
                128 => {
                    // If the alg field of jwk is present, and is not "A128KW", then throw a DataError.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != alg_matching[0]) {
                        return Err(Error::Data(Some(
                            "JWK algorithm and key length do not match".into(),
                        )));
                    }
                },
                // If the length in bits of data is 192:
                192 => {
                    // If the alg field of jwk is present, and is not "A192KW", then throw a DataError.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != alg_matching[1]) {
                        return Err(Error::Data(Some(
                            "JWK algorithm and key length do not match".into(),
                        )));
                    }
                },
                // If the length in bits of data is 256:
                256 => {
                    // If the alg field of jwk is present, and is not "A256KW", then throw a DataError.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != alg_matching[2]) {
                        return Err(Error::Data(Some(
                            "JWK algorithm and key length do not match".into(),
                        )));
                    }
                },
                // Otherwise:
                _ => {
                    // throw a DataError.
                    return Err(Error::Data(Some(
                        "Key length is not 128, 192, or 256 bits".into(),
                    )));
                },
            }

            // Step 2.6. If usages is non-empty and the use field of jwk is present and is not
            // "enc", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                return Err(Error::Data(Some("JWK usage is not encryption".into())));
            }

            // Step 2.7. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK] or does not contain all of the specified usages
            // values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.8. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data(Some("JWK is not extractable".into())));
            }
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError
            return Err(Error::NotSupported(Some(
                "Key format is unsupported".into(),
            )));
        },
    };

    // Step 5. Let algorithm be a new AesKeyAlgorithm.
    // Step 6. Set the name attribute of algorithm to alg_name.
    // Step 7. Set the length attribute of algorithm to the length, in bits, of data.
    let algorithm = SubtleAesKeyAlgorithm {
        name: alg_name.to_string(),
        length: (data.len() * 8) as u16,
    };

    // Step 3. Let key be a new CryptoKey object representing an AES key with value data.
    // Step 4. Set the [[type]] internal slot of key to "secret".
    // Step 8. Set the [[algorithm]] internal slot of key to algorithm.
    let handle = match data.len() * 8 {
        128 => Handle::Aes128(data.to_vec()),
        192 => Handle::Aes192(data.to_vec()),
        256 => Handle::Aes256(data.to_vec()),
        _ => {
            return Err(Error::Data(Some(
                "Key length is not 128, 192, or 256 bits".into(),
            )));
        },
    };
    let key = CryptoKey::new(
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::AesKeyAlgorithm(algorithm),
        usages,
        handle,
        can_gc,
    );

    // Step 9. Return key.
    Ok(key)
}

/// Helper function for
/// <https://w3c.github.io/webcrypto/#aes-kw-operations-export-key>
fn export_key_aes(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    // Step 1. If the underlying cryptographic key material represented by the [[handle]]
    // internal slot of key cannot be accessed, then throw an OperationError.
    // NOTE: key.handle() guarantees access.

    // Step 2.
    let result;
    match format {
        // If format is "raw":
        KeyFormat::Raw | KeyFormat::Raw_secret => match key.handle() {
            // Step 2.1. Let data be a byte sequence containing the raw octets of the key
            // represented by the [[handle]] internal slot of key.
            // Step 2.2. Let result be data.
            Handle::Aes128(key_data) => {
                result = ExportedKey::Bytes(key_data.clone());
            },
            Handle::Aes192(key_data) => {
                result = ExportedKey::Bytes(key_data.clone());
            },
            Handle::Aes256(key_data) => {
                result = ExportedKey::Bytes(key_data.clone());
            },
            _ => unreachable!(),
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1. Let jwk be a new JsonWebKey dictionary.
            // Step 2.2. Set the kty attribute of jwk to the string "oct".
            let mut jwk = JsonWebKey {
                kty: Some(DOMString::from("oct")),
                ..Default::default()
            };

            // Step 2.3. Set the k attribute of jwk to be a string containing the raw octets of
            // the key represented by the [[handle]] internal slot of key, encoded according to
            // Section 6.4 of JSON Web Algorithms [JWA].
            match key.handle() {
                Handle::Aes128(key) => jwk.encode_string_field(JwkStringField::K, key),
                Handle::Aes192(key) => jwk.encode_string_field(JwkStringField::K, key),
                Handle::Aes256(key) => jwk.encode_string_field(JwkStringField::K, key),
                _ => unreachable!(),
            };

            // Step 2.4.
            // If the length attribute of key is 128: Set the alg attribute of jwk to the string "A128KW".
            // If the length attribute of key is 192: Set the alg attribute of jwk to the string "A192KW".
            // If the length attribute of key is 256: Set the alg attribute of jwk to the string "A256KW".
            //
            // NOTE: Check key length via key.handle()
            jwk.alg = Some(
                match (key.handle(), key.algorithm().name()) {
                    (Handle::Aes128(_), ALG_AES_KW) => "A128KW",
                    (Handle::Aes192(_), ALG_AES_KW) => "A192KW",
                    (Handle::Aes256(_), ALG_AES_KW) => "A256KW",
                    _ => unreachable!(),
                }
                .into(),
            );

            // Step 2.5. Set the key_ops attribute of jwk to equal the [[usages]] internal slot of key.
            jwk.set_key_ops(key.usages());

            // Step 2.6. Set the ext attribute of jwk to equal the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 2.7. Let result be jwk.
            result = ExportedKey::Jwk(Box::new(jwk));
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Key format is unsupported".into(),
            )));
        },
    };

    // Step 3. Return result.
    Ok(result)
}

/// Helper function for
/// <https://w3c.github.io/webcrypto/#aes-kw-operations-get-key-length>
pub(crate) fn get_key_length_aes(
    normalized_derived_key_algorithm: &SubtleAesDerivedKeyParams,
) -> Result<Option<u32>, Error> {
    // Step 1. If the length member of normalizedDerivedKeyAlgorithm is not 128, 192 or 256, then
    // throw a OperationError.
    if !matches!(normalized_derived_key_algorithm.length, 128 | 192 | 256) {
        return Err(Error::Operation(Some(
            "Key length is not 128, 192, or 256 bits".into(),
        )));
    }

    // Step 2. Return the length member of normalizedDerivedKeyAlgorithm.
    Ok(Some(normalized_derived_key_algorithm.length as u32))
}
