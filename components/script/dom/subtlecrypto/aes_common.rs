/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aes::cipher::crypto_common::Key;
use aes::{Aes128, Aes192, Aes256};
use pkcs8::rand_core::{OsRng, RngCore};

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
    ALG_AES_CBC, ALG_AES_CTR, ALG_AES_GCM, ALG_AES_KW, ALG_AES_OCB, ExportedKey, JsonWebKeyExt,
    JwkStringField, KeyAlgorithmAndDerivatives, SubtleAesDerivedKeyParams, SubtleAesKeyAlgorithm,
    SubtleAesKeyGenParams,
};
use crate::script_runtime::CanGc;

#[expect(clippy::enum_variant_names)]
pub(crate) enum AesAlgorithm {
    AesCtr,
    AesCbc,
    AesGcm,
    AesKw,
    AesOcb,
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-generate-key>
/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-generate-key>
/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-generate-key>
/// <https://w3c.github.io/webcrypto/#aes-kw-operations-generate-key>
/// <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-generate-key>
///
/// The step order in the specification of AES-OCB is slightly different, but it is equivalent to
/// this implementation.
pub(crate) fn generate_key(
    aes_algorithm: AesAlgorithm,
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    match aes_algorithm {
        AesAlgorithm::AesCtr |
        AesAlgorithm::AesCbc |
        AesAlgorithm::AesGcm |
        AesAlgorithm::AesOcb => {
            // Step 1. If usages contains any entry which is not one of "encrypt", "decrypt",
            // "wrapKey" or "unwrapKey", then throw a SyntaxError.
            if usages.iter().any(|usage| {
                !matches!(
                    usage,
                    KeyUsage::Encrypt | KeyUsage::Decrypt | KeyUsage::WrapKey | KeyUsage::UnwrapKey
                )
            }) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not one of \"encrypt\", \"decrypt\", \
                    \"wrapKey\" or \"unwrapKey\""
                        .to_string(),
                )));
            }
        },
        AesAlgorithm::AesKw => {
            // Step 1. If usages contains any entry which is not one of "wrapKey" or "unwrapKey",
            // then throw a SyntaxError.
            if usages
                .iter()
                .any(|usage| !matches!(usage, KeyUsage::WrapKey | KeyUsage::UnwrapKey))
            {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not one of \"wrapKey\" or \"unwrapKey\""
                        .to_string(),
                )));
            }
        },
    }

    // Step 2. If the length member of normalizedAlgorithm is not equal to one of 128, 192 or 256,
    // then throw an OperationError.
    // Step 3. Generate an AES key of length equal to the length member of normalizedAlgorithm.
    // Step 4. If the key generation step fails, then throw an OperationError.
    let handle =
        match normalized_algorithm.length {
            128 => {
                let mut key_bytes = vec![0; 16];
                OsRng.fill_bytes(&mut key_bytes);
                Handle::Aes128Key(Key::<Aes128>::clone_from_slice(&key_bytes))
            },
            192 => {
                let mut key_bytes = vec![0; 24];
                OsRng.fill_bytes(&mut key_bytes);
                Handle::Aes192Key(Key::<Aes192>::clone_from_slice(&key_bytes))
            },
            256 => {
                let mut key_bytes = vec![0; 32];
                OsRng.fill_bytes(&mut key_bytes);
                Handle::Aes256Key(Key::<Aes256>::clone_from_slice(&key_bytes))
            },
            _ => return Err(Error::Operation(Some(
                "The length member of normalizedAlgorithm is not equal to one of 128, 192 or 256"
                    .to_string(),
            ))),
        };

    // Step 5. Let key be a new CryptoKey object representing the generated AES key.
    // Step 6. Let algorithm be a new AesKeyAlgorithm.
    // Step 8. Set the length attribute of algorithm to equal the length member of
    // normalizedAlgorithm.
    // Step 9. Set the [[type]] internal slot of key to "secret".
    // Step 10. Set the [[algorithm]] internal slot of key to algorithm.
    // Step 11. Set the [[extractable]] internal slot of key to be extractable.
    // Step 12. Set the [[usages]] internal slot of key to be usages.
    let algorithm_name = match aes_algorithm {
        AesAlgorithm::AesCtr => {
            // Step 7. Set the name attribute of algorithm to "AES-CTR".
            "AES-CTR"
        },
        AesAlgorithm::AesCbc => {
            // Step 7. Set the name attribute of algorithm to "AES-CBC".
            "AES-CBC"
        },
        AesAlgorithm::AesGcm => {
            // Step 7. Set the name attribute of algorithm to "AES-GCM".
            "AES-GCM"
        },
        AesAlgorithm::AesKw => {
            // Step 7. Set the name attribute of algorithm to "AES-KW".
            "AES-KW"
        },
        AesAlgorithm::AesOcb => {
            // Step 7. Set the name attribute of algorithm to "AES-OCB".
            "AES-OCB"
        },
    };
    let algorithm = SubtleAesKeyAlgorithm {
        name: algorithm_name.to_string(),
        length: normalized_algorithm.length,
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

    // Step 13. Return key.
    Ok(key)
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-import-key>
/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-import-key>
/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-import-key>
/// <https://w3c.github.io/webcrypto/#aes-kw-operations-import-key>
/// <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-import-key>
///
/// The specification of AES-OCB has one more step at the beginning of the operation:
///
/// > Let keyData be the key data to be imported.
///
/// As it is simply used to name the variable, it is safe to omit it in the implementation below to
/// align with the specification of other AES algorithms.
pub(crate) fn import_key(
    aes_algorithm: AesAlgorithm,
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    match &aes_algorithm {
        AesAlgorithm::AesCtr |
        AesAlgorithm::AesCbc |
        AesAlgorithm::AesGcm |
        AesAlgorithm::AesOcb => {
            // Step 1. If usages contains an entry which is not one of "encrypt", "decrypt",
            // "wrapKey" or "unwrapKey", then throw a SyntaxError.
            if usages.iter().any(|usage| {
                !matches!(
                    usage,
                    KeyUsage::Encrypt | KeyUsage::Decrypt | KeyUsage::WrapKey | KeyUsage::UnwrapKey
                )
            }) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not one of \"encrypt\", \"decrypt\", \
                    \"wrapKey\"  or \"unwrapKey\""
                        .to_string(),
                )));
            }
        },
        AesAlgorithm::AesKw => {
            // Step 1. If usages contains an entry which is not one of "wrapKey" or "unwrapKey",
            // then throw a SyntaxError.
            if usages
                .iter()
                .any(|usage| !matches!(usage, KeyUsage::WrapKey | KeyUsage::UnwrapKey))
            {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not one of \"wrapKey\"  or \"unwrapKey\""
                        .to_string(),
                )));
            }
        },
    }

    // Step 2.
    let data;
    match format {
        // If format is "raw": (Only applied to AES-CTR, AES-CBC, AES-GCM, AES-KW)
        KeyFormat::Raw
            if matches!(
                aes_algorithm,
                AesAlgorithm::AesCtr |
                    AesAlgorithm::AesCbc |
                    AesAlgorithm::AesGcm |
                    AesAlgorithm::AesKw
            ) =>
        {
            // Step 2.1. Let data be keyData.
            data = key_data.to_vec();

            // Step 2.2. If the length in bits of data is not 128, 192 or 256 then throw a
            // DataError.
            if !matches!(data.len(), 16 | 24 | 32) {
                return Err(Error::Data(Some(
                    "The length in bits of data is not 128, 192 or 256".to_string(),
                )));
            }
        },
        // If format is "raw-secret":
        KeyFormat::Raw_secret => {
            // Step 2.1. Let data be keyData.
            data = key_data.to_vec();

            // Step 2.2. If the length in bits of data is not 128, 192 or 256 then throw a
            // DataError.
            if !matches!(data.len(), 16 | 24 | 32) {
                return Err(Error::Data(Some(
                    "The length in bits of key is not 128, 192 or 256".to_string(),
                )));
            }
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1.
            // If keyData is a JsonWebKey dictionary:
            //     Let jwk equal keyData.
            // Otherwise:
            //     Throw a DataError.
            let jwk = JsonWebKey::parse(GlobalScope::get_cx(), key_data)?;

            // Step 2.2. If the kty field of jwk is not "oct", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "oct") {
                return Err(Error::Data(Some(
                    "The kty field of jwk is not \"oct\"".to_string(),
                )));
            }

            // Step 2.3. If jwk does not meet the requirements of Section 6.4 of JSON Web
            // Algorithms [JWA], then throw a DataError.
            // Step 2.4. Let data be the byte sequence obtained by decoding the k field of jwk.
            data = jwk.decode_required_string_field(JwkStringField::K)?;

            match aes_algorithm {
                AesAlgorithm::AesCtr => {
                    // Step 2.5.
                    // If data has length 128 bits:
                    //     If the alg field of jwk is present, and is not "A128CTR", then throw a
                    //     DataError.
                    // If data has length 192 bits:
                    //     If the alg field of jwk is present, and is not "A192CTR", then throw a
                    //     DataError.
                    // If data has length 256 bits:
                    //     If the alg field of jwk is present, and is not "A256CTR", then throw a
                    //     DataError.
                    // Otherwise:
                    //     throw a DataError.
                    let expected_alg = match data.len() {
                        16 => "A128CTR",
                        24 => "A192CTR",
                        32 => "A256CTR",
                        _ => {
                            return Err(Error::Data(Some(
                                "The length in bits of data is not 128, 192 or 256".to_string(),
                            )));
                        },
                    };
                    if jwk.alg.as_ref().is_none_or(|alg| alg != expected_alg) {
                        return Err(Error::Data(Some(format!(
                            "The alg field of jwk is present, and is not {}",
                            expected_alg
                        ))));
                    }
                },
                AesAlgorithm::AesCbc => {
                    // Step 2.5.
                    // If data has length 128 bits:
                    //     If the alg field of jwk is present, and is not "A128CBC", then throw a
                    //     DataError.
                    // If data has length 192 bits:
                    //     If the alg field of jwk is present, and is not "A192CBC", then throw a
                    //     DataError.
                    // If data has length 256 bits:
                    //     If the alg field of jwk is present, and is not "A256CBC", then throw a
                    //     DataError.
                    // Otherwise:
                    //     throw a DataError.
                    let expected_alg = match data.len() {
                        16 => "A128CBC",
                        24 => "A192CBC",
                        32 => "A256CBC",
                        _ => {
                            return Err(Error::Data(Some(
                                "The length in bits of data is not 128, 192 or 256".to_string(),
                            )));
                        },
                    };
                    if jwk.alg.as_ref().is_none_or(|alg| alg != expected_alg) {
                        return Err(Error::Data(Some(format!(
                            "The alg field of jwk is present, and is not {}",
                            expected_alg
                        ))));
                    }
                },
                AesAlgorithm::AesGcm => {
                    // Step 2.5.
                    // If data has length 128 bits:
                    //     If the alg field of jwk is present, and is not "A128GCM", then throw a
                    //     DataError.
                    // If data has length 192 bits:
                    //     If the alg field of jwk is present, and is not "A192GCM", then throw a
                    //     DataError.
                    // If data has length 256 bits:
                    //     If the alg field of jwk is present, and is not "A256GCM", then throw a
                    //     DataError.
                    // Otherwise:
                    //     throw a DataError.
                    let expected_alg = match data.len() {
                        16 => "A128GCM",
                        24 => "A192GCM",
                        32 => "A256GCM",
                        _ => {
                            return Err(Error::Data(Some(
                                "The length in bits of data is not 128, 192 or 256".to_string(),
                            )));
                        },
                    };
                    if jwk.alg.as_ref().is_none_or(|alg| alg != expected_alg) {
                        return Err(Error::Data(Some(format!(
                            "The alg field of jwk is present, and is not {}",
                            expected_alg
                        ))));
                    }
                },
                AesAlgorithm::AesKw => {
                    // Step 2.5.
                    // If data has length 128 bits:
                    //     If the alg field of jwk is present, and is not "A128KW", then throw a
                    //     DataError.
                    // If data has length 192 bits:
                    //     If the alg field of jwk is present, and is not "A192KW", then throw a
                    //     DataError.
                    // If data has length 256 bits:
                    //     If the alg field of jwk is present, and is not "A256KW", then throw a
                    //     DataError.
                    // Otherwise:
                    //     throw a DataError.
                    let expected_alg = match data.len() {
                        16 => "A128KW",
                        24 => "A192KW",
                        32 => "A256KW",
                        _ => {
                            return Err(Error::Data(Some(
                                "The length in bits of data is not 128, 192 or 256".to_string(),
                            )));
                        },
                    };
                    if jwk.alg.as_ref().is_none_or(|alg| alg != expected_alg) {
                        return Err(Error::Data(Some(format!(
                            "The alg field of jwk is present, and is not {}",
                            expected_alg
                        ))));
                    }
                },
                AesAlgorithm::AesOcb => {
                    // Step 2.5.
                    // If data has length 128 bits:
                    //     If the alg field of jwk is present, and is not "A128OCB", then throw a
                    //     DataError.
                    // If data has length 192 bits:
                    //     If the alg field of jwk is present, and is not "A192OCB", then throw a
                    //     DataError.
                    // If data has length 256 bits:
                    //     If the alg field of jwk is present, and is not "A256OCB", then throw a
                    //     DataError.
                    // Otherwise:
                    //     throw a DataError.
                    let expected_alg = match data.len() {
                        16 => "A128OCB",
                        24 => "A192OCB",
                        32 => "A256OCB",
                        _ => {
                            return Err(Error::Data(Some(
                                "The length in bits of key is not 128, 192 or 256".to_string(),
                            )));
                        },
                    };
                    if jwk.alg.as_ref().is_none_or(|alg| alg != expected_alg) {
                        return Err(Error::Data(Some(format!(
                            "The alg field of jwk is present, and is not {}",
                            expected_alg
                        ))));
                    }
                },
            }

            // Step 2.6. If usages is non-empty and the use field of jwk is present and is not
            // "enc", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                return Err(Error::Data(Some(
                    "Usages is non-empty and the use field of jwk is present and is not \"enc\""
                        .to_string(),
                )));
            }

            // Step 2.7. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK] or does not contain all of the specified usages
            // values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.8. If the ext field of jwk is present and has the value false and extractable
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
                "Unupported import key format for AES key".to_string(),
            )));
        },
    }

    // Step 3. Let key be a new CryptoKey object representing an AES key with value data.
    // Step 4. Set the [[type]] internal slot of key to "secret".
    // Step 5. Let algorithm be a new AesKeyAlgorithm.
    // Step 7. Set the length attribute of algorithm to the length, in bits, of data.
    // Step 8. Set the [[algorithm]] internal slot of key to algorithm.
    let handle = match data.len() {
        16 => Handle::Aes128Key(Key::<Aes128>::clone_from_slice(&data)),
        24 => Handle::Aes192Key(Key::<Aes192>::clone_from_slice(&data)),
        32 => Handle::Aes256Key(Key::<Aes256>::clone_from_slice(&data)),
        _ => {
            return Err(Error::Data(Some(
                "The length in bits of data is not 128, 192 or 256".to_string(),
            )));
        },
    };
    let algorithm = SubtleAesKeyAlgorithm {
        name: match &aes_algorithm {
            AesAlgorithm::AesCtr => {
                // Step 6. Set the name attribute of algorithm to "AES-CTR".
                ALG_AES_CTR.to_string()
            },
            AesAlgorithm::AesCbc => {
                // Step 6. Set the name attribute of algorithm to "AES-CBC".
                ALG_AES_CBC.to_string()
            },
            AesAlgorithm::AesGcm => {
                // Step 6. Set the name attribute of algorithm to "AES-GCM".
                ALG_AES_GCM.to_string()
            },
            AesAlgorithm::AesKw => {
                // Step 6. Set the name attribute of algorithm to "AES-KW".
                ALG_AES_KW.to_string()
            },
            AesAlgorithm::AesOcb => {
                // Step 6. Set the name attribute of algorithm to "AES-OCB".
                ALG_AES_OCB.to_string()
            },
        },
        length: data.len() as u16 * 8,
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

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-export-key>
/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-export-key>
/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-export-key>
/// <https://w3c.github.io/webcrypto/#aes-kw-operations-export-key>
/// <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-export-key>
pub(crate) fn export_key(
    aes_algorithm: AesAlgorithm,
    format: KeyFormat,
    key: &CryptoKey,
) -> Result<ExportedKey, Error> {
    // Step 1. If the underlying cryptographic key material represented by the [[handle]] internal
    // slot of key cannot be accessed, then throw an OperationError.

    // Step 2.
    let result = match format {
        // If format is "raw": (Only applied to AES-CTR, AES-CBC, AES-GCM, AES-KW)
        KeyFormat::Raw
            if matches!(
                aes_algorithm,
                AesAlgorithm::AesCtr |
                    AesAlgorithm::AesCbc |
                    AesAlgorithm::AesGcm |
                    AesAlgorithm::AesKw
            ) =>
        {
            // Step 2.1. Let data be a byte sequence containing the raw octets of the key
            // represented by [[handle]] internal slot of key.
            let data = match key.handle() {
                Handle::Aes128Key(key) => key.to_vec(),
                Handle::Aes192Key(key) => key.to_vec(),
                Handle::Aes256Key(key) => key.to_vec(),
                _ => {
                    return Err(Error::Operation(Some(
                        "The key handle is not representing an AES key".to_string(),
                    )));
                },
            };

            // Step 2.2. Let result be data.
            ExportedKey::Bytes(data)
        },
        // If format is "raw-secret":
        KeyFormat::Raw_secret => {
            // Step 2.1. Let data be a byte sequence containing the raw octets of the key
            // represented by [[handle]] internal slot of key.
            let data = match key.handle() {
                Handle::Aes128Key(key) => key.to_vec(),
                Handle::Aes192Key(key) => key.to_vec(),
                Handle::Aes256Key(key) => key.to_vec(),
                _ => {
                    return Err(Error::Operation(Some(
                        "The key handle is not representing an AES key".to_string(),
                    )));
                },
            };

            // Step 2.2. Let result be data.
            ExportedKey::Bytes(data)
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1. Let jwk be a new JsonWebKey dictionary.
            // Step 2.2. Set the kty attribute of jwk to the string "oct".
            let mut jwk = JsonWebKey {
                kty: Some(DOMString::from("oct")),
                ..Default::default()
            };

            // Step 2.3. Set the k attribute of jwk to be a string containing the raw octets of the
            // key represented by [[handle]] internal slot of key, encoded according to Section 6.4
            // of JSON Web Algorithms [JWA].
            let key_bytes = match key.handle() {
                Handle::Aes128Key(key) => key.as_slice(),
                Handle::Aes192Key(key) => key.as_slice(),
                Handle::Aes256Key(key) => key.as_slice(),
                _ => {
                    return Err(Error::Operation(Some(
                        "The key handle is not representing an AES key".to_string(),
                    )));
                },
            };
            jwk.encode_string_field(JwkStringField::K, key_bytes);

            match aes_algorithm {
                AesAlgorithm::AesCtr => {
                    // Step 2.4.
                    // If the length attribute of key is 128:
                    //     Set the alg attribute of jwk to the string "A128CTR".
                    // If the length attribute of key is 192:
                    //     Set the alg attribute of jwk to the string "A192CTR".
                    // If the length attribute of key is 256:
                    //     Set the alg attribute of jwk to the string "A256CTR".
                    let KeyAlgorithmAndDerivatives::AesKeyAlgorithm(algorithm) = key.algorithm()
                    else {
                        return Err(Error::Operation(None));
                    };
                    let alg = match algorithm.length {
                        128 => "A128CTR",
                        192 => "A192CTR",
                        256 => "A256CTR",
                        _ => return Err(Error::Operation(Some(
                            "The length attribute of the [[algorithm]] internal slot of key is not \
                            128, 192 or 256".to_string(),
                        )))
                    };
                    jwk.alg = Some(DOMString::from(alg));
                },
                AesAlgorithm::AesCbc => {
                    // Step 2.4.
                    // If the length attribute of key is 128:
                    //     Set the alg attribute of jwk to the string "A128CBC".
                    // If the length attribute of key is 192:
                    //     Set the alg attribute of jwk to the string "A192CBC".
                    // If the length attribute of key is 256:
                    //     Set the alg attribute of jwk to the string "A256CBC".
                    let KeyAlgorithmAndDerivatives::AesKeyAlgorithm(algorithm) = key.algorithm()
                    else {
                        return Err(Error::Operation(None));
                    };
                    let alg = match algorithm.length {
                        128 => "A128CBC",
                        192 => "A192CBC",
                        256 => "A256CBC",
                        _ => return Err(Error::Operation(Some(
                            "The length attribute of the [[algorithm]] internal slot of key is not \
                            128, 192 or 256".to_string(),
                        )))
                    };
                    jwk.alg = Some(DOMString::from(alg));
                },
                AesAlgorithm::AesGcm => {
                    // Step 2.4.
                    // If the length attribute of key is 128:
                    //     Set the alg attribute of jwk to the string "A128GCM".
                    // If the length attribute of key is 192:
                    //     Set the alg attribute of jwk to the string "A192GCM".
                    // If the length attribute of key is 256:
                    //     Set the alg attribute of jwk to the string "A256GCM".
                    let KeyAlgorithmAndDerivatives::AesKeyAlgorithm(algorithm) = key.algorithm()
                    else {
                        return Err(Error::Operation(None));
                    };
                    let alg = match algorithm.length {
                        128 => "A128GCM",
                        192 => "A192GCM",
                        256 => "A256GCM",
                        _ => return Err(Error::Operation(Some(
                            "The length attribute of the [[algorithm]] internal slot of key is not \
                            128, 192 or 256".to_string(),
                        )))
                    };
                    jwk.alg = Some(DOMString::from(alg));
                },
                AesAlgorithm::AesKw => {
                    // Step 2.4.
                    // If the length attribute of key is 128:
                    //     Set the alg attribute of jwk to the string "A128KW".
                    // If the length attribute of key is 192:
                    //     Set the alg attribute of jwk to the string "A192KW".
                    // If the length attribute of key is 256:
                    //     Set the alg attribute of jwk to the string "A256KW".
                    let KeyAlgorithmAndDerivatives::AesKeyAlgorithm(algorithm) = key.algorithm()
                    else {
                        return Err(Error::Operation(None));
                    };
                    let alg = match algorithm.length {
                        128 => "A128KW",
                        192 => "A192KW",
                        256 => "A256KW",
                        _ => return Err(Error::Operation(Some(
                            "The length attribute of the [[algorithm]] internal slot of key is not \
                            128, 192 or 256".to_string(),
                        )))
                    };
                    jwk.alg = Some(DOMString::from(alg));
                },
                AesAlgorithm::AesOcb => {
                    // Step 2.4.
                    // If the length attribute of key is 128:
                    //     Set the alg attribute of jwk to the string "A128OCB".
                    // If the length attribute of key is 192:
                    //     Set the alg attribute of jwk to the string "A192OCB".
                    // If the length attribute of key is 256:
                    //     Set the alg attribute of jwk to the string "A256OCB".
                    let KeyAlgorithmAndDerivatives::AesKeyAlgorithm(algorithm) = key.algorithm()
                    else {
                        return Err(Error::Operation(None));
                    };
                    let alg = match algorithm.length {
                        128 => "A128OCB",
                        192 => "A192OCB",
                        256 => "A256OCB",
                        _ => return Err(Error::Operation(Some(
                            "The length attribute of the [[algorithm]] internal slot of key is not \
                            128, 192 or 256".to_string(),
                        )))
                    };
                    jwk.alg = Some(DOMString::from(alg));
                },
            }

            // Step 2.5. Set the key_ops attribute of jwk to equal the usages attribute of key.
            jwk.set_key_ops(key.usages());

            // Step 2.6. Set the ext attribute of jwk to equal the [[extractable]] internal slot of
            // key.
            jwk.ext = Some(key.Extractable());

            // Step 2.7. Let result be jwk.
            ExportedKey::Jwk(Box::new(jwk))
        },
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported import key format for AES key".to_string(),
            )));
        },
    };

    // Step 3. Return result.
    Ok(result)
}

/// <https://w3c.github.io/webcrypto/#aes-ctr-operations-get-key-length>
/// <https://w3c.github.io/webcrypto/#aes-cbc-operations-get-key-length>
/// <https://w3c.github.io/webcrypto/#aes-gcm-operations-get-key-length>
/// <https://w3c.github.io/webcrypto/#aes-kw-operations-get-key-length>
/// <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-get-key-length>
pub(crate) fn get_key_length(
    normalized_derived_key_algorithm: &SubtleAesDerivedKeyParams,
) -> Result<Option<u32>, Error> {
    // Step 1. If the length member of normalizedDerivedKeyAlgorithm is not 128, 192 or 256, then
    // throw an OperationError.
    if !matches!(normalized_derived_key_algorithm.length, 128 | 192 | 256) {
        return Err(Error::Operation(Some(
            "The length member of normalizedDerivedKeyAlgorithm is not 128, 192 or 256".to_string(),
        )));
    }

    // Step 2. Return the length member of normalizedDerivedKeyAlgorithm.
    Ok(Some(normalized_derived_key_algorithm.length as u32))
}
