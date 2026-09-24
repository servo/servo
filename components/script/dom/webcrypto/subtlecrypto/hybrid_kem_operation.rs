/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use x_wing::{
    Decapsulate, DecapsulationKey, Decapsulator, Encapsulate, EncapsulationKey, Generate,
    KeyExport, KeyInit, TryKeyInit,
};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, CryptoKeyPair, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{JsonWebKey, KeyFormat};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle, KeyUsageVecHelper};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    Algorithm, CryptoAlgorithm, EncapsulatedBits, ExportedKey, JsonWebKeyExt, JwkStringField,
    KeyAlgorithm, KeyAlgorithmAndDerivatives,
};

/// <https://wicg.github.io/webcrypto-modern-algos/#hybrid-kems-operations-encapsulate>
pub(crate) fn encapsulate(
    normalized_algorithm: &Algorithm,
    key: &CryptoKey,
) -> Result<EncapsulatedBits, Error> {
    // Step 1. If the [[type]] internal slot of key is not "public", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Public {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"public\"".into(),
        )));
    }

    // Step 2. Let sharedKey and ciphertext be the outputs that result from performing the Encaps
    // function for the hybrid KEM instance indicated by the name member of algorithm in Section 4
    // of [draft-irtf-cfrg-concrete-hybrid-kems-04], using the key represented by the [[handle]]
    // internal slot of key as the ek input parameter.
    // Step 3. If the Encaps function returned an error, return an OperationError.
    let (shared_key, ciphertext) = match normalized_algorithm.name {
        CryptoAlgorithm::MlKem768X25519 => {
            let Handle::MlKem768X25519PublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing a MLKEM768-X25519 public key".into(),
                )));
            };
            let (ciphertext, shared_key) = public_key.encapsulate();
            (shared_key.to_vec(), ciphertext.to_vec())
        },
        name => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not a hybrid KEM algorithm",
                name.as_str()
            ))));
        },
    };

    // Step 4. Let result be a new EncapsulatedBits dictionary.
    // Step 5. Set the sharedKey attribute of result to the result of creating an ArrayBuffer
    // containing sharedKey.
    // Step 6. Set the ciphertext attribute of result to the result of creating an ArrayBuffer
    // containing ciphertext.
    let result = EncapsulatedBits {
        shared_key: Some(shared_key.into()),
        ciphertext: Some(ciphertext),
    };

    // Step 7. Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#hybrid-kems-operations-decapsulate>
pub(crate) fn decapsulate(
    normalized_algorithm: &Algorithm,
    key: &CryptoKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the [[type]] internal slot of key is not "private", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Private {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"private\"".into(),
        )));
    }

    // Step 2. Let sharedKey be the output that results from performing the Decaps function for the
    // hybrid KEM instance indicated by the name member of algorithm in Section 4 of
    // [draft-irtf-cfrg-concrete-hybrid-kems-04], using the key represented by the [[handle]]
    // internal slot of key as the dk input parameter, and ciphertext as the ct input parameter.
    // Step 3. If the Decaps function returned an error, return an OperationError.
    let shared_key = match normalized_algorithm.name {
        CryptoAlgorithm::MlKem768X25519 => {
            let Handle::MlKem768X25519PrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an MLKEM768-X25519 private key".into(),
                )));
            };
            private_key
                .decapsulate_slice(ciphertext)
                .map_err(|_| {
                    Error::Operation(Some(
                        "Failed to perform MLKEM768-X25519 decapsulation".into(),
                    ))
                })?
                .to_vec()
        },
        name => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not a hybrid KEM algorithm",
                name.as_str()
            ))));
        },
    };

    // Step 4. Return sharedKey.
    Ok(shared_key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#hybrid-kems-operations-get-shared-key-length>
pub(crate) fn get_shared_key_length() -> u32 {
    // Step 1. Return 256.
    256
}

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-kem-operations-generate-key>
pub(crate) fn generate_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &Algorithm,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<CryptoKeyPair, Error> {
    // Step 1. If usages contains an entry which is not one of "encapsulateKey", "encapsulateBits",
    // "decapsulateKey" or "decapsulateBits", then throw a SyntaxError.
    if usages.iter().any(|usage| {
        !matches!(
            usage,
            KeyUsage::EncapsulateKey |
                KeyUsage::EncapsulateBits |
                KeyUsage::DecapsulateKey |
                KeyUsage::DecapsulateBits
        )
    }) {
        return Err(Error::Syntax(Some(
            "Usages contains any entry which is not one of \"encapsulateKey\", \
            \"encapsulateBits\", \"decapsulateKey\" or \"decapsulateBits\""
                .into(),
        )));
    }

    // Step 2. Generate an ML-KEM key pair, as described in Section 7.1 of [FIPS-203], with the
    // parameter set indicated by the name member of normalizedAlgorithm.
    // Step 3. If the key generation step fails, then throw an OperationError.
    let (private_key_handle, public_key_handle) = match normalized_algorithm.name {
        CryptoAlgorithm::MlKem768X25519 => {
            let decapsulation_key = DecapsulationKey::generate();
            let encapsulation_key = decapsulation_key.encapsulation_key().clone();
            (
                Handle::MlKem768X25519PrivateKey(decapsulation_key),
                Handle::MlKem768X25519PublicKey(encapsulation_key),
            )
        },
        name => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not a hybrid KEM algorithm",
                name.as_str()
            ))));
        },
    };

    // Step 4. Let algorithm be a new KeyAlgorithm object.
    // Step 5. Set the name attribute of algorithm to the name attribute of normalizedAlgorithm.
    let algorithm = KeyAlgorithm {
        name: normalized_algorithm.name,
    };

    // Step 6. Let publicKey be a new CryptoKey representing the encapsulation key of the generated
    // key pair.
    // Step 7. Set the [[type]] internal slot of publicKey to "public".
    // Step 8. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 9. Set the [[extractable]] internal slot of publicKey to true.
    // Step 10. Set the [[usages]] internal slot of publicKey to be the usage intersection of usages
    // and [ "encapsulateKey", "encapsulateBits" ].
    let public_key = CryptoKey::new(
        cx,
        global,
        KeyType::Public,
        true,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm.clone()),
        usages.usage_intersection(&[KeyUsage::EncapsulateKey, KeyUsage::EncapsulateBits]),
        public_key_handle,
    );

    // Step 11. Let privateKey be a new CryptoKey representing the decapsulation key of the
    // generated key pair.
    // Step 12. Set the [[type]] internal slot of privateKey to "private".
    // Step 13. Set the [[algorithm]] internal slot of privateKey to algorithm.
    // Step 14. Set the [[extractable]] internal slot of privateKey to extractable.
    // Step 15. Set the [[usages]] internal slot of privateKey to be the usage intersection of
    // usages and [ "decapsulateKey", "decapsulateBits" ].
    let private_key = CryptoKey::new(
        cx,
        global,
        KeyType::Private,
        extractable,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
        usages.usage_intersection(&[KeyUsage::DecapsulateKey, KeyUsage::DecapsulateBits]),
        private_key_handle,
    );

    // Step 16. Let result be a new CryptoKeyPair dictionary.
    // Step 17. Set the publicKey attribute of result to be publicKey.
    // Step 18. Set the privateKey attribute of result to be privateKey.
    let result = CryptoKeyPair {
        publicKey: Some(public_key),
        privateKey: Some(private_key),
    };

    // Step 19. Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#hybrid-kems-operations-import-key>
pub(crate) fn import_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &Algorithm,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2.
    let key =
        match format {
            // If format is "raw-public":
            KeyFormat::Raw_public => {
                // Step 2.1. If usages contains an entry which is not "encapsulateKey" or
                // "encapsulateBits" then throw a SyntaxError.
                if usages.iter().any(|usage| {
                    !matches!(usage, KeyUsage::EncapsulateKey | KeyUsage::EncapsulateBits)
                }) {
                    return Err(Error::Syntax(Some(
                        "Usages contains an entry which is not \"encapsulateKey\" or \
                        \"encapsulateBits\""
                            .into(),
                    )));
                }

                // Step 2.2. Let data be keyData.
                let data = key_data;

                // Step 2.3. If the length in bytes of data is not the raw public key length, Nek, for
                // the hybrid KEM instance indicated by the name member of normalizedAlgorithm in
                // Section 4 of [draft-irtf-cfrg-concrete-hybrid-kems-04], then throw a DataError.
                // Step 2.4. Let key be a new CryptoKey that represents the hybrid KEM public key data
                // in data.
                // Step 2.5. Set the [[type]] internal slot of key to "public"
                // Step 2.6. Let algorithm be a new KeyAlgorithm.
                // Step 2.7. Set the name attribute of algorithm to the name attribute of
                // normalizedAlgorithm.
                // Step 2.8. Set the [[algorithm]] internal slot of key to algorithm.
                let public_key = match normalized_algorithm.name {
                    CryptoAlgorithm::MlKem768X25519 => {
                        if key_data.len() != 1216 {
                            return Err(Error::Data(Some(
                                "Invalid key length for MLKEM768-X25519 public key".into(),
                            )));
                        }
                        let encapsulation_key =
                            EncapsulationKey::new_from_slice(data).map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the public MLKEM768-X25519 key in raw format"
                                        .into(),
                                ))
                            })?;
                        Handle::MlKem768X25519PublicKey(encapsulation_key)
                    },
                    name => {
                        return Err(Error::NotSupported(Some(format!(
                            "{} is not a hybrid KEM algorithm",
                            name.as_str()
                        ))));
                    },
                };
                let algorithm = KeyAlgorithm {
                    name: normalized_algorithm.name,
                };
                CryptoKey::new(
                    cx,
                    global,
                    KeyType::Public,
                    extractable,
                    KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                    usages.normalized_value(),
                    public_key,
                )
            },
            // If format is "raw-seed":
            KeyFormat::Raw_seed => {
                // Step 2.1. If usages contains an entry which is not "decapsulateKey" or
                // "decapsulateBits" then throw a SyntaxError.
                if usages.iter().any(|usage| {
                    !matches!(usage, KeyUsage::DecapsulateKey | KeyUsage::DecapsulateBits)
                }) {
                    return Err(Error::Syntax(Some(
                        "Usages contains an entry which is not \"decapsulateKey\" or \
                        \"decapsulateBits\""
                            .into(),
                    )));
                }

                // Step 2.2. Let data be keyData.
                let data = key_data;

                // Step 2.3. If the length in bits of data is not 256 then throw a DataError.
                if data.len() != 32 {
                    return Err(Error::Data(Some(
                        "The length in bits of data is not 256".into(),
                    )));
                }

                // Step 2.4. Let keyPair be the result of performing the DeriveKeyPair function
                // described in Section 5.5 of [draft-irtf-cfrg-hybrid-kems-12] with the hybrid KEM
                // instance indicated by the name member of normalizedAlgorithm, using data as the seed
                // input parameter.
                // Step 2.5. If the DeriveKeyPair function returned an error, then throw an
                // OperationError.
                let private_key = match normalized_algorithm.name {
                    CryptoAlgorithm::MlKem768X25519 => {
                        let decapsulation_key = DecapsulationKey::new_from_slice(key_data)
                            .map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the private MLKEM768-X25519 key in raw format"
                                        .into(),
                                ))
                            })?;
                        Handle::MlKem768X25519PrivateKey(decapsulation_key)
                    },
                    name => {
                        return Err(Error::NotSupported(Some(format!(
                            "{} is not a hybrid KEM algorithm",
                            name.as_str()
                        ))));
                    },
                };

                // Step 2.6. Let key be a new CryptoKey that represents the hybrid KEM private key
                // identified by the decapsulation key of keyPair.
                // Step 2.7. Set the [[type]] internal slot of key to "private"
                // Step 2.8. Let algorithm be a new KeyAlgorithm.
                // Step 2.9. Set the name attribute of algorithm to the name attribute of
                // normalizedAlgorithm.
                // Step 2.10. Set the [[algorithm]] internal slot of key to algorithm.
                let algorithm = KeyAlgorithm {
                    name: normalized_algorithm.name,
                };
                CryptoKey::new(
                    cx,
                    global,
                    KeyType::Private,
                    extractable,
                    KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                    usages.normalized_value(),
                    private_key,
                )
            },
            // If format is "jwk":
            KeyFormat::Jwk => {
                // Step 2.1.
                // If keyData is a JsonWebKey dictionary:
                //     Let jwk equal keyData.
                // Otherwise:
                //     Throw a DataError.
                let jwk = JsonWebKey::parse(cx, key_data)?;

                // Step 2.2. If the priv field of jwk is present and if usages contains an entry which
                // is not "decapsulateKey" or "decapsulateBits" then throw a SyntaxError.
                if jwk.priv_.is_some() &&
                    usages.iter().any(|usage| {
                        !matches!(usage, KeyUsage::DecapsulateKey | KeyUsage::DecapsulateBits)
                    })
                {
                    return Err(Error::Syntax(Some(
                        "The priv field of jwk is present and usages contains an entry which is \
                        not \"decapsulateKey\" or \"decapsulateBits\""
                            .into(),
                    )));
                }

                // Step 2.3. If the priv field of jwk is not present and if usages contains an entry
                // which is not "encapsulateKey" or "encapsulateBits" then throw a SyntaxError.
                if jwk.priv_.is_none() &&
                    usages.iter().any(|usage| {
                        !matches!(usage, KeyUsage::EncapsulateKey | KeyUsage::EncapsulateBits)
                    })
                {
                    return Err(Error::Syntax(Some(
                        "The priv field of jwk is not present and usages contains an entry which \
                        is not \"encapsulateKey\" or \"encapsulateBits\""
                            .into(),
                    )));
                }

                // Step 2.4. If the kty field of jwk is not "AKP", then throw a DataError.
                if jwk.kty.as_ref().is_none_or(|kty| kty != "AKP") {
                    return Err(Error::Data(Some(
                        "The kty field of jwk is not \"AKP\"".into(),
                    )));
                }

                // Step 2.5. If the alg field of jwk is not present, or its value does not identify the
                // hybrid KEM instance indicated by the name member of normalizedAlgorithm, then throw a
                // DataError.
                match normalized_algorithm.name {
                    CryptoAlgorithm::MlKem768X25519 => {
                        if jwk.alg.as_ref().is_none_or(|alg| alg != "MLKEM768-X25519") {
                            return Err(Error::Data(Some(
                                "The alg field of jwk is not invalid.".into(),
                            )));
                        }
                    },
                    name => {
                        return Err(Error::NotSupported(Some(format!(
                            "{} is not a hybrid KEM algorithm",
                            name.as_str()
                        ))));
                    },
                }

                // Step 2.6. If usages is non-empty and the use field of jwk is present and is not equal
                // to "enc", then throw a DataError.
                if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                    return Err(Error::Data(Some(
                        "Usages is non-empty and the use field of jwk is present and is not \
                        equal to \"enc\""
                            .into(),
                    )));
                }

                // Step 2.7. If the key_ops field of jwk is present, and is invalid according to the
                // requirements of JSON Web Key [JWK], or it does not contain all of the specified
                // usages values, then throw a DataError.
                jwk.check_key_ops(&usages)?;

                // Step 2.8. If the ext field of jwk is present and has the value false and extractable
                // is true, then throw a DataError.
                if jwk.ext.is_some_and(|ext| !ext) && extractable {
                    return Err(Error::Data(Some(
                        "The ext field of jwk is present and has the value false and extractable \
                        is true"
                            .into(),
                    )));
                }

                // Step 2.9.
                // If the priv field of jwk is present:
                let (key_type, key_handle) = if jwk.priv_.is_some() {
                    // Step 2.9.1. If the priv attribute of jwk does not contain a valid base64url
                    // encoded 32-byte seed representing a hybrid KEM private key, then throw a
                    // DataError.
                    let priv_bytes = jwk.decode_required_string_field(JwkStringField::Priv)?;
                    if priv_bytes.len() != 32 {
                        return Err(Error::Data(Some(
                            "The priv attribute of jwk does not contain a valid base64url \
                            encoded 32-byte seed"
                                .into(),
                        )));
                    }

                    // Step 2.9.2. Let key be a new CryptoKey object that represents the hybrid KEM
                    // private key identified by interpreting the priv attribute of jwk as a base64url
                    // encoded seed.
                    // Step 2.9.3. Set the [[type]] internal slot of key to "private".
                    // Step 2.9.4. If the pub attribute of jwk does not contain the base64url encoded
                    // public key representing the hybrid KEM public key corresponding to key, then
                    // throw a DataError.
                    // NOTE: The CryptoKey object is created in Step 2.10 - 2.12.
                    let pub_bytes = jwk.decode_required_string_field(JwkStringField::Pub)?;
                    let private_key_handle = match normalized_algorithm.name {
                        CryptoAlgorithm::MlKem768X25519 => {
                            let decapsulation_key = DecapsulationKey::new_from_slice(&priv_bytes)
                                .map_err(|_| {
                                Error::Data(Some(
                                "Failed to parse the private MLKEM768-X25519 key in priv attribute"
                                    .into(),
                            ))
                            })?;
                            let encapsulation_key = EncapsulationKey::new_from_slice(&pub_bytes)
                                .map_err(|_| {
                                    Error::Data(Some(
                                "Failed to parse the public MLKEM768-X25519 key in pub attribute"
                                    .into(),
                            ))
                                })?;
                            if *decapsulation_key.encapsulation_key() != encapsulation_key {
                                return Err(Error::Data(Some(
                                    "The public key in pub attribute does not match \
                                    the private key in priv attribute"
                                        .into(),
                                )));
                            }
                            Handle::MlKem768X25519PrivateKey(decapsulation_key)
                        },
                        name => {
                            return Err(Error::NotSupported(Some(format!(
                                "{} is not a hybrid KEM algorithm",
                                name.as_str()
                            ))));
                        },
                    };
                    (KeyType::Private, private_key_handle)
                }
                // Otherwise:
                else {
                    // Step 2.9.1. If the pub attribute of jwk does not contain a valid base64url
                    // encoded raw public key whose length is Nek for the hybrid KEM instance indicated
                    // by the name member of normalizedAlgorithm in Section 4 of
                    // [draft-irtf-cfrg-concrete-hybrid-kems-04], then throw a DataError.
                    // Step 2.9.2. Let key be a new CryptoKey object that represents the hybrid KEM
                    // public key identified by interpreting the pub attribute of jwk as a base64url
                    // encoded public key.
                    // Step 2.9.3. Set the [[type]] internal slot of key to "public".
                    // NOTE: The CryptoKey object is created in Step 2.10 - 2.12.
                    let pub_bytes = jwk.decode_required_string_field(JwkStringField::Pub)?;
                    let public_key_handle = match normalized_algorithm.name {
                        CryptoAlgorithm::MlKem768X25519 => {
                            if pub_bytes.len() != 1216 {
                                return Err(Error::Data(Some(
                                    "The pub attribute of jwk does not contain a valid base64url \
                                    encoded raw public key with valid length"
                                        .into(),
                                )));
                            }
                            let encapsulation_key = EncapsulationKey::new_from_slice(&pub_bytes)
                                .map_err(|_| {
                                    Error::Data(Some(
                                        "Failed to parse the public MLKEM768-X25519 key in pub \
                                        attribute"
                                            .into(),
                                    ))
                                })?;
                            Handle::MlKem768X25519PublicKey(encapsulation_key)
                        },
                        name => {
                            return Err(Error::NotSupported(Some(format!(
                                "{} is not a hybrid KEM algorithm",
                                name.as_str()
                            ))));
                        },
                    };
                    (KeyType::Public, public_key_handle)
                };

                // Step 2.10. Let algorithm be a new instance of a KeyAlgorithm object.
                // Step 2.11. Set the name attribute of algorithm to the name member of
                // normalizedAlgorithm.
                // Step 2.12. Set the [[algorithm]] internal slot of key to algorithm.
                let algorithm = KeyAlgorithm {
                    name: normalized_algorithm.name,
                };
                CryptoKey::new(
                    cx,
                    global,
                    key_type,
                    extractable,
                    KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                    usages.normalized_value(),
                    key_handle,
                )
            },
            // Otherwise:
            _ => {
                // throw a NotSupportedError.
                return Err(Error::NotSupported(Some(
                    "Unsupported import key format for ML-KEM key".into(),
                )));
            },
        };

    // Step 3. Return key.
    Ok(key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#hybrid-kems-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    // Step 1. Let key be the CryptoKey to be exported.

    // Step 2. If the underlying cryptographic key material represented by the [[handle]] internal
    // slot of key cannot be accessed, then throw an OperationError.
    // NOTE: Done in Step 3.

    // Step 3.
    let result = match format {
        // If format is "raw-public":
        KeyFormat::Raw_public => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"public\"".into(),
                )));
            }

            // Step 3.2. Let data be a byte sequence containing the raw octets of the key
            // represented by the [[handle]] internal slot of key.
            let data = match key.handle() {
                Handle::MlKem768X25519PublicKey(public_key) => public_key.to_bytes().to_vec(),
                _ => {
                    return Err(Error::Operation(Some(
                        "The key handle is not representing a hybrid KEM public key".into(),
                    )));
                },
            };

            // Step 3.3. Let result be data.
            ExportedKey::new_bytes(data)
        },
        // If format is "raw-seed":
        KeyFormat::Raw_seed => {
            // Step 3.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"private\"".into(),
                )));
            }

            // Step 3.2. Let data be a byte sequence containing the 32-byte seed represented by the
            // [[handle]] internal slot of key.
            let data = match key.handle() {
                Handle::MlKem768X25519PrivateKey(private_key) => private_key.as_bytes().to_vec(),
                _ => {
                    return Err(Error::Operation(Some(
                        "The key handle is not representing a hybrid KEM private key".into(),
                    )));
                },
            };

            // Step 3.3. Let result be data.
            ExportedKey::new_bytes(data)
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 3.1. Let jwk be a new JsonWebKey dictionary.
            let mut jwk = JsonWebKey::default();

            // Step 3.2. Let keyAlgorithm be the [[algorithm]] internal slot of key.
            let KeyAlgorithmAndDerivatives::KeyAlgorithm(key_algorithm) = key.algorithm() else {
                return Err(Error::Operation(Some(
                    "[[algorithm]] internal slot of key is not a KeyAlgorithm".into(),
                )));
            };

            // Step 3.3. Set the kty attribute of jwk to "AKP".
            jwk.kty = Some(DOMString::from_static("AKP"));

            // Step 3.4. Set the alg attribute of jwk to the name member of keyAlgorithm.
            jwk.alg = Some(DOMString::from(key_algorithm.name.as_str()));

            // Step 3.5. Set the pub attribute of jwk to the base64url encoded public key
            // corresponding to the [[handle]] internal slot of key.
            // Step 3.6.
            // If the [[type]] internal slot of key is "private":
            //     Set the priv attribute of jwk to the base64url encoded 32-byte seed represented
            //     by the [[handle]] internal slot of key.
            if key.Type() == KeyType::Private {
                match key.handle() {
                    Handle::MlKem768X25519PrivateKey(private_key) => {
                        jwk.encode_string_field(JwkStringField::Priv, private_key.as_bytes());
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            private_key.encapsulation_key().to_bytes().as_slice(),
                        );
                    },
                    _ => {
                        return Err(Error::Operation(Some(
                            "The key handle is not representing a hybrid KEM private key".into(),
                        )));
                    },
                }
            } else {
                match key.handle() {
                    Handle::MlKem768X25519PublicKey(public_key) => {
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            public_key.to_bytes().as_slice(),
                        );
                    },
                    _ => {
                        return Err(Error::Operation(Some(
                            "The key handle is not representing a hybrid KEM public key".into(),
                        )));
                    },
                };
            }

            // Step 3.7. Set the key_ops attribute of jwk to the usages attribute of key.
            jwk.set_key_ops(key.usages());

            // Step 3.8. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 3.9. Let result be jwk.
            ExportedKey::new_jwk(jwk)
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported export key format for hybrid KEM key".into(),
            )));
        },
    };

    // Step 4.  Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-getPublicKey>
/// Step 9 - 15, for hybrid KEM
pub(crate) fn get_public_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    key: &CryptoKey,
    algorithm: &KeyAlgorithmAndDerivatives,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 9. If usages contains an entry which is not supported for a public key by the algorithm
    // identified by algorithm, then throw a SyntaxError.
    //
    // NOTE: See "importKey" operation for supported usages
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::EncapsulateKey | KeyUsage::EncapsulateBits))
    {
        return Err(Error::Syntax(Some(
            "Usages contains an entry which is not \"encapsulateKey\" or \"encapsulateBits\""
                .into(),
        )));
    }

    // Step 10. Let publicKey be a new CryptoKey representing the public key corresponding to the
    // private key represented by the [[handle]] internal slot of key.
    // Step 11. If an error occurred, then throw a OperationError.
    // Step 12. Set the [[type]] internal slot of publicKey to "public".
    // Step 13. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 14. Set the [[extractable]] internal slot of publicKey to true.
    // Step 15. Set the [[usages]] internal slot of publicKey to usages.
    let public_key_handle = match key.handle() {
        Handle::MlKem768X25519PrivateKey(decapsulation_key) => {
            Handle::MlKem768X25519PublicKey(decapsulation_key.encapsulation_key().clone())
        },
        _ => {
            return Err(Error::Operation(Some(
                "[[handle]] internal slot of key is not a hybrid KEM private key".into(),
            )));
        },
    };
    let public_key = CryptoKey::new(
        cx,
        global,
        KeyType::Public,
        true,
        algorithm.clone(),
        usages,
        public_key_handle,
    );

    Ok(public_key)
}
