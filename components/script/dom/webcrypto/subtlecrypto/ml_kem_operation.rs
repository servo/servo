/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use ml_dsa::KeyExport;
use ml_kem::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey};
use ml_kem::{
    Decapsulate, DecapsulationKey, Encapsulate, EncapsulationKey, Generate, KeyInit, MlKem512,
    MlKem768, MlKem1024, TryKeyInit,
};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, CryptoKeyPair, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{JsonWebKey, KeyFormat};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    CryptoAlgorithm, ExportedKey, JsonWebKeyExt, JwkStringField, KeyAlgorithmAndDerivatives,
    SubtleAlgorithm, SubtleEncapsulatedBits, SubtleKeyAlgorithm,
};

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-kem-operations-encapsulate>
pub(crate) fn encapsulate(
    normalized_algorithm: &SubtleAlgorithm,
    key: &CryptoKey,
) -> Result<SubtleEncapsulatedBits, Error> {
    // Step 1. If the [[type]] internal slot of key is not "public", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Public {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"public\"".into(),
        )));
    }

    // Step 2. Perform the encapsulation key check described in Section 7.2 of [FIPS-203] with the
    // parameter set indicated by the name member of algorithm, using the key represented by the
    // [[handle]] internal slot of key as the ek input parameter.
    // Step 3. If the encapsulation key check failed, return an OperationError.
    // Step 4. Let sharedKey and ciphertext be the outputs that result from performing the
    // ML-KEM.Encaps function described in Section 7.2 of [FIPS-203] with the parameter set
    // indicated by the name member of algorithm, using the key represented by the [[handle]]
    // internal slot of key as the ek input parameter.
    // Step 5. If the ML-KEM.Encaps function returned an error, return an OperationError.
    let (shared_key, ciphertext) = match normalized_algorithm.name {
        CryptoAlgorithm::MlKem512 => {
            let Handle::MlKem512PublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-KEM-512 public key".into(),
                )));
            };
            let (ciphertext, shared_key) = public_key.encapsulate();
            (shared_key.to_vec(), ciphertext.to_vec())
        },
        CryptoAlgorithm::MlKem768 => {
            let Handle::MlKem768PublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-KEM-768 public key".into(),
                )));
            };
            let (ciphertext, shared_key) = public_key.encapsulate();
            (shared_key.to_vec(), ciphertext.to_vec())
        },
        CryptoAlgorithm::MlKem1024 => {
            let Handle::MlKem1024PublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-KEM-1024 public key".into(),
                )));
            };
            let (ciphertext, shared_key) = public_key.encapsulate();
            (shared_key.to_vec(), ciphertext.to_vec())
        },
        name => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not an ML-KEM algorithm",
                name.as_str()
            ))));
        },
    };

    // Step 6. Let result be a new EncapsulatedBits dictionary.
    // Step 7. Set the sharedKey attribute of result to the result of creating an ArrayBuffer
    // containing sharedKey.
    // Step 8. Set the ciphertext attribute of result to the result of creating an ArrayBuffer
    // containing ciphertext.
    let result = SubtleEncapsulatedBits {
        shared_key: Some(shared_key.into()),
        ciphertext: Some(ciphertext),
    };

    // Step 9. Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-kem-operations-decapsulate>
pub(crate) fn decapsulate(
    normalized_algorithm: &SubtleAlgorithm,
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

    // Step 2. Perform the decapsulation input check described in Section 7.3 of [FIPS-203] with
    // the parameter set indicated by the name member of algorithm, using the key represented by
    // the [[handle]] internal slot of key as the dk input parameter, and ciphertext as the c input
    // parameter.
    // Step 3. If the decapsulation key check failed, return an OperationError.
    // Step 4. Let sharedKey be the output that results from performing the ML-KEM.Decaps function
    // described in Section 7.3 of [FIPS-203] with the parameter set indicated by the name member
    // of algorithm, using the key represented by the [[handle]] internal slot of key as the dk
    // input parameter, and ciphertext as the c input parameter.
    let shared_key = match normalized_algorithm.name {
        CryptoAlgorithm::MlKem512 => {
            let Handle::MlKem512PrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-KEM-512 private key".into(),
                )));
            };
            private_key
                .decapsulate_slice(ciphertext)
                .map_err(|_| {
                    Error::Operation(Some("Failed to perform ML-KEM decapsulation".into()))
                })?
                .to_vec()
        },
        CryptoAlgorithm::MlKem768 => {
            let Handle::MlKem768PrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-KEM-768 private key".into(),
                )));
            };
            private_key
                .decapsulate_slice(ciphertext)
                .map_err(|_| {
                    Error::Operation(Some("Failed to perform ML-KEM decapsulation".into()))
                })?
                .to_vec()
        },
        CryptoAlgorithm::MlKem1024 => {
            let Handle::MlKem1024PrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-KEM-1024 private key".into(),
                )));
            };
            private_key
                .decapsulate_slice(ciphertext)
                .map_err(|_| {
                    Error::Operation(Some("Failed to perform ML-KEM decapsulation".into()))
                })?
                .to_vec()
        },
        name => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not an ML-KEM algorithm",
                name.as_str()
            ))));
        },
    };

    // Step 5. Return sharedKey.
    Ok(shared_key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-kem-operations-generate-key>
pub(crate) fn generate_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleAlgorithm,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<CryptoKeyPair, Error> {
    // Step 1. If usages contains any entry which is not one of "encapsulateKey",
    // "encapsulateBits", "decapsulateKey" or "decapsulateBits", then throw a SyntaxError.
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
        CryptoAlgorithm::MlKem512 => {
            let decapsulation_key = DecapsulationKey::<MlKem512>::generate();
            let encapsulation_key = decapsulation_key.encapsulation_key().clone();
            (
                Handle::MlKem512PrivateKey(decapsulation_key),
                Handle::MlKem512PublicKey(encapsulation_key),
            )
        },
        CryptoAlgorithm::MlKem768 => {
            let decapsulation_key = DecapsulationKey::<MlKem768>::generate();
            let encapsulation_key = decapsulation_key.encapsulation_key().clone();
            (
                Handle::MlKem768PrivateKey(decapsulation_key),
                Handle::MlKem768PublicKey(encapsulation_key),
            )
        },
        CryptoAlgorithm::MlKem1024 => {
            let decapsulation_key = DecapsulationKey::<MlKem1024>::generate();
            let encapsulation_key = decapsulation_key.encapsulation_key().clone();
            (
                Handle::MlKem1024PrivateKey(decapsulation_key),
                Handle::MlKem1024PublicKey(encapsulation_key),
            )
        },
        name => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not an ML-KEM algorithm",
                name.as_str()
            ))));
        },
    };

    // Step 4. Let algorithm be a new KeyAlgorithm object.
    // Step 5. Set the name attribute of algorithm to the name attribute of normalizedAlgorithm.
    let algorithm = SubtleKeyAlgorithm {
        name: normalized_algorithm.name,
    };

    // Step 6. Let publicKey be a new CryptoKey representing the encapsulation key of the generated
    // key pair.
    // Step 7. Set the [[type]] internal slot of publicKey to "public".
    // Step 8. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 9. Set the [[extractable]] internal slot of publicKey to true.
    // Step 10. Set the [[usages]] internal slot of publicKey to be the usage intersection of
    // usages and [ "encapsulateKey", "encapsulateBits" ].
    let public_key = CryptoKey::new(
        cx,
        global,
        KeyType::Public,
        true,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm.clone()),
        usages
            .iter()
            .filter(|usage| matches!(usage, KeyUsage::EncapsulateKey | KeyUsage::EncapsulateBits))
            .cloned()
            .collect(),
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
        usages
            .iter()
            .filter(|usage| matches!(usage, KeyUsage::DecapsulateKey | KeyUsage::DecapsulateBits))
            .cloned()
            .collect(),
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

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-kem-operations-import-key>
pub(crate) fn import_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleAlgorithm,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2.
    let key = match format {
        // If format is "spki":
        KeyFormat::Spki => {
            // Step 2.1. If usages contains an entry which is not "encapsulateKey" or
            // "encapsulateBits" then throw a SyntaxError.
            if usages
                .iter()
                .any(|usage| !matches!(usage, KeyUsage::EncapsulateKey | KeyUsage::EncapsulateBits))
            {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"encapsulateKey\" or \
                    \"encapsulateBits\""
                        .into(),
                )));
            }

            // Step 2.2. Let spki be the result of running the parse a subjectPublicKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurred while parsing, then throw a DataError.
            // Step 2.4.
            // If the name member of normalizedAlgorithm is "ML-KEM-512":
            //     Let expectedOid be id-alg-ml-kem-512 (2.16.840.1.101.3.4.4.1).
            // If the name member of normalizedAlgorithm is "ML-KEM-768":
            //     Let expectedOid be id-alg-ml-kem-768 (2.16.840.1.101.3.4.4.2).
            // If the name member of normalizedAlgorithm is "ML-KEM-1024":
            //     Let expectedOid be id-alg-ml-kem-1024 (2.16.840.1.101.3.4.4.3).
            // Otherwise:
            //     throw a NotSupportedError.
            // Step 2.5. If the algorithm object identifier field of the algorithm
            // AlgorithmIdentifier field of spki is not equal to expectedOid, then throw a
            // DataError.
            // Step 2.6. If the parameters field of the algorithm AlgorithmIdentifier field of spki
            // is present, then throw a DataError.
            // Step 2.7. Let publicKey be the ML-KEM public key identified by the subjectPublicKey
            // field of spki.
            let public_key = match normalized_algorithm.name {
                CryptoAlgorithm::MlKem512 => Handle::MlKem512PublicKey(
                    EncapsulationKey::from_public_key_der(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to decode the ML-KEM-512 public key from SPKI format".into(),
                        ))
                    })?,
                ),
                CryptoAlgorithm::MlKem768 => Handle::MlKem768PublicKey(
                    EncapsulationKey::from_public_key_der(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to decode the ML-KEM-768 public key from SPKI format".into(),
                        ))
                    })?,
                ),
                CryptoAlgorithm::MlKem1024 => Handle::MlKem1024PublicKey(
                    EncapsulationKey::from_public_key_der(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to decode the ML-KEM-1024 public key from SPKI format".into(),
                        ))
                    })?,
                ),
                name => {
                    return Err(Error::NotSupported(Some(format!(
                        "{} is not an ML-KEM algorithm",
                        name.as_str()
                    ))));
                },
            };

            // Step 2.8. Let key be a new CryptoKey that represents publicKey.
            // Step 2.9. Set the [[type]] internal slot of key to "public"
            // Step 2.10. Let algorithm be a new KeyAlgorithm.
            // Step 2.11. Set the name attribute of algorithm to the name attribute of
            // normalizedAlgorithm.
            // Step 2.12. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name,
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                public_key,
            )
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 2.1. If usages contains an entry which is not "decapsulateKey" or
            // "decapsulateBits" then throw a SyntaxError.
            if usages
                .iter()
                .any(|usage| !matches!(usage, KeyUsage::DecapsulateKey | KeyUsage::DecapsulateBits))
            {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"decapsulateKey\" or \
                    \"decapsulateBits\""
                        .into(),
                )));
            }

            // Step 2.2. Let privateKeyInfo be the result of running the parse a privateKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurs while parsing, then throw a DataError.
            // Step 2.4.
            // If the name member of normalizedAlgorithm is "ML-KEM-512":
            //     Let expectedOid be id-alg-ml-kem-512 (2.16.840.1.101.3.4.4.1).
            //     Let asn1Structure be the ASN.1 ML-KEM-512-PrivateKey structure.
            // If the name member of normalizedAlgorithm is "ML-KEM-768":
            //     Let expectedOid be id-alg-ml-kem-768 (2.16.840.1.101.3.4.4.2).
            //     Let asn1Structure be the ASN.1 ML-KEM-768-PrivateKey structure.
            // If the name member of normalizedAlgorithm is "ML-KEM-1024":
            //     Let expectedOid be id-alg-ml-kem-1024 (2.16.840.1.101.3.4.4.3).
            //     Let asn1Structure be the ASN.1 ML-KEM-1024-PrivateKey structure.
            // Otherwise:
            //     throw a NotSupportedError.
            // Step 2.5. If the algorithm object identifier field of the privateKeyAlgorithm
            // PrivateKeyAlgorithm field of privateKeyInfo is not equal to expectedOid, then throw
            // a DataError.
            // Step 2.6. If the parameters field of the privateKeyAlgorithm
            // PrivateKeyAlgorithmIdentifier field of privateKeyInfo is present, then throw a
            // DataError.
            // Step 2.7. Let mlKemPrivateKey be the result of performing the parse an ASN.1
            // structure algorithm, with data as the privateKey field of privateKeyInfo, structure
            // as asn1Structure, and exactData set to true.
            // Step 2.8. If an error occurred while parsing, then throw a DataError.
            // Step 2.9. If mlKemPrivateKey represents an ML-KEM key in the expandedKey format, or
            // if mlKemPrivateKey represents an ML-KEM key in the both format and the both format
            // is not supported, throw a NotSupportedError.
            // Step 2.10. If mlKemPrivateKey represents an ML-KEM key in the both format, and the
            // seed field does not correspond to the expandedKey field, throw a DataError.
            //
            // NOTE: We do not support the `both` format.
            let ml_kem_private_key = match normalized_algorithm.name {
                CryptoAlgorithm::MlKem512 => Handle::MlKem512PrivateKey(
                    DecapsulationKey::from_pkcs8_der(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to decode the ML-KEM-512 private key from PKCS#8 format".into(),
                        ))
                    })?,
                ),
                CryptoAlgorithm::MlKem768 => Handle::MlKem768PrivateKey(
                    DecapsulationKey::from_pkcs8_der(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to decode the ML-KEM-768 private key from PKCS#8 format".into(),
                        ))
                    })?,
                ),
                CryptoAlgorithm::MlKem1024 => Handle::MlKem1024PrivateKey(
                    DecapsulationKey::from_pkcs8_der(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to decode the ML-KEM-1024 private key from PKCS#8 format"
                                .into(),
                        ))
                    })?,
                ),
                name => {
                    return Err(Error::NotSupported(Some(format!(
                        "{} is not an ML-KEM algorithm",
                        name.as_str()
                    ))));
                },
            };

            // Step 2.11. Let key be a new CryptoKey that represents the ML-KEM private key
            // identified by mlKemPrivateKey.
            // Step 2.12. Set the [[type]] internal slot of key to "private"
            // Step 2.13. Let algorithm be a new KeyAlgorithm.
            // Step 2.14. Set the name attribute of algorithm to the name attribute of
            // normalizedAlgorithm.
            // Step 2.15. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name,
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Private,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                ml_kem_private_key,
            )
        },
        // If format is "raw-public":
        KeyFormat::Raw_public => {
            // Step 2.1. If usages contains an entry which is not "encapsulateKey" or
            // "encapsulateBits" then throw a SyntaxError.
            if usages
                .iter()
                .any(|usage| !matches!(usage, KeyUsage::EncapsulateKey | KeyUsage::EncapsulateBits))
            {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"encapsulateKey\" or \
                    \"encapsulateBits\""
                        .into(),
                )));
            }

            // Step 2.2. Let data be keyData.
            // Step 2.3. Let key be a new CryptoKey that represents the ML-KEM public key data in
            // data.
            // Step 2.4. Set the [[type]] internal slot of key to "public"
            // Step 2.5. Let algorithm be a new KeyAlgorithm.
            // Step 2.6. Set the name attribute of algorithm to the name attribute of
            // normalizedAlgorithm.
            // Step 2.7. Set the [[algorithm]] internal slot of key to algorithm.
            let public_key = match normalized_algorithm.name {
                CryptoAlgorithm::MlKem512 => {
                    let encapsulation_key =
                        EncapsulationKey::new_from_slice(key_data).map_err(|_| {
                            Error::Data(Some(
                                "Failed to parse the public ML-KEM-512 key in raw format".into(),
                            ))
                        })?;
                    Handle::MlKem512PublicKey(encapsulation_key)
                },
                CryptoAlgorithm::MlKem768 => {
                    let encapsulation_key =
                        EncapsulationKey::new_from_slice(key_data).map_err(|_| {
                            Error::Data(Some(
                                "Failed to parse the public ML-KEM-768 key in raw format".into(),
                            ))
                        })?;
                    Handle::MlKem768PublicKey(encapsulation_key)
                },
                CryptoAlgorithm::MlKem1024 => {
                    let encapsulation_key =
                        EncapsulationKey::new_from_slice(key_data).map_err(|_| {
                            Error::Data(Some(
                                "Failed to parse the public ML-KEM-1024 key in raw format".into(),
                            ))
                        })?;
                    Handle::MlKem1024PublicKey(encapsulation_key)
                },
                name => {
                    return Err(Error::NotSupported(Some(format!(
                        "{} is not an ML-KEM algorithm",
                        name.as_str()
                    ))));
                },
            };
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name,
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                public_key,
            )
        },
        // If format is "raw-seed":
        KeyFormat::Raw_seed => {
            // Step 2.1. If usages contains an entry which is not "decapsulateKey" or
            // "decapsulateBits" then throw a SyntaxError.
            if usages
                .iter()
                .any(|usage| !matches!(usage, KeyUsage::DecapsulateKey | KeyUsage::DecapsulateBits))
            {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"decapsulateKey\" or \
                    \"decapsulateBits\""
                        .into(),
                )));
            }

            // Step 2.2. Let data be keyData.
            let data = key_data;

            // Step 2.3. If the length in bits of data is not 512 then throw a DataError.
            if data.len() != 64 {
                return Err(Error::Data(Some(
                    "The length in bits of data is not 512".into(),
                )));
            }

            // Step 2.4. Let privateKey be the result of performing the ML-KEM.KeyGen_internal
            // function described in Section 6.1 of [FIPS-203] with the parameter set indicated by
            // the name member of normalizedAlgorithm, using the first 256 bits of data as d and
            // the last 256 bits of data as z.
            let private_key = match normalized_algorithm.name {
                CryptoAlgorithm::MlKem512 => {
                    let decapsulation_key =
                        DecapsulationKey::new_from_slice(key_data).map_err(|_| {
                            Error::Data(Some(
                                "Failed to parse the private ML-KEM-512 key in raw format".into(),
                            ))
                        })?;
                    Handle::MlKem512PrivateKey(decapsulation_key)
                },
                CryptoAlgorithm::MlKem768 => {
                    let decapsulation_key =
                        DecapsulationKey::new_from_slice(key_data).map_err(|_| {
                            Error::Data(Some(
                                "Failed to parse the private ML-KEM-768 key in raw format".into(),
                            ))
                        })?;
                    Handle::MlKem768PrivateKey(decapsulation_key)
                },
                CryptoAlgorithm::MlKem1024 => {
                    let decapsulation_key =
                        DecapsulationKey::new_from_slice(key_data).map_err(|_| {
                            Error::Data(Some(
                                "Failed to parse the private ML-KEM-1024 key in raw format".into(),
                            ))
                        })?;
                    Handle::MlKem1024PrivateKey(decapsulation_key)
                },
                name => {
                    return Err(Error::NotSupported(Some(format!(
                        "{} is not an ML-KEM algorithm",
                        name.as_str()
                    ))));
                },
            };

            // Step 2.5. Let key be a new CryptoKey that represents the ML-KEM private key
            // identified by privateKey.
            // Step 2.6. Set the [[type]] internal slot of key to "private"
            // Step 2.7. Let algorithm be a new KeyAlgorithm.
            // Step 2.8. Set the name attribute of algorithm to the name attribute of
            // normalizedAlgorithm.
            // Step 2.9. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name,
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Private,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
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
                    "The priv field of jwk is not present and usages contains an entry which is \
                    not \"encapsulateKey\" or \"encapsulateBits\""
                        .into(),
                )));
            }

            // Step 2.4. If the kty field of jwk is not "AKP", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "AKP") {
                return Err(Error::Data(Some(
                    "The kty field of jwk is not \"AKP\"".into(),
                )));
            }

            // Step 2.5. If the alg field of jwk is not one of the alg values corresponding to the
            // name member of normalizedAlgorithm indicated in Section 8 of
            // [draft-ietf-jose-pqc-kem-05] (Figure 1 or 2), then throw a DataError.
            match normalized_algorithm.name {
                CryptoAlgorithm::MlKem512 => {
                    if jwk
                        .alg
                        .as_ref()
                        .is_none_or(|alg| alg != "ML-KEM-512" && alg != "ML-KEM-512-AES128KW")
                    {
                        return Err(Error::Data(Some(
                            "The alg field of jwk is not invalid.".into(),
                        )));
                    }
                },
                CryptoAlgorithm::MlKem768 => {
                    if jwk
                        .alg
                        .as_ref()
                        .is_none_or(|alg| alg != "ML-KEM-768" && alg != "ML-KEM-768-AES192KW")
                    {
                        return Err(Error::Data(Some(
                            "The alg field of jwk is not invalid.".into(),
                        )));
                    }
                },
                CryptoAlgorithm::MlKem1024 => {
                    if jwk
                        .alg
                        .as_ref()
                        .is_none_or(|alg| alg != "ML-KEM-1024" && alg != "ML-KEM-1024-AES256KW")
                    {
                        return Err(Error::Data(Some(
                            "The alg field of jwk is not invalid.".into(),
                        )));
                    }
                },
                _ => {
                    return Err(Error::NotSupported(Some(format!(
                        "{} is not an ML-KEM algorithm",
                        normalized_algorithm.name.as_str()
                    ))));
                },
            };

            // Step 2.6. If usages is non-empty and the use field of jwk is present and is not
            // equal to "enc", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                return Err(Error::Data(Some(
                    "usages is non-empty and the use field of jwk is present and is not \
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
                // encoded seed representing an ML-KEM private key, then throw a DataError.
                let priv_bytes = jwk.decode_required_string_field(JwkStringField::Priv)?;

                // Step 2.9.2. Let key be a new CryptoKey object that represents the ML-KEM private
                // key identified by interpreting the priv attribute of jwk as a base64url encoded
                // seed.
                // Step 2.9.3. Set the [[type]] internal slot of Key to "private".
                // Step 2.9.4. If the pub attribute of jwk does not contain the base64url encoded
                // public key representing the ML-KEM public key corresponding to key, then throw a
                // DataError.
                // NOTE: The CryptoKey object is created in Step 2.10 - 2.12.
                let pub_bytes = jwk.decode_required_string_field(JwkStringField::Pub)?;
                let private_key_handle = match normalized_algorithm.name {
                    CryptoAlgorithm::MlKem512 => {
                        let decapsulation_key = DecapsulationKey::new_from_slice(&priv_bytes)
                            .map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the private ML-KEM-512 key in priv attribute"
                                        .into(),
                                ))
                            })?;
                        let encapsulation_key = EncapsulationKey::new_from_slice(&pub_bytes)
                            .map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the public ML-KEM-512 key in pub attribute"
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
                        Handle::MlKem512PrivateKey(decapsulation_key)
                    },
                    CryptoAlgorithm::MlKem768 => {
                        let decapsulation_key = DecapsulationKey::new_from_slice(&priv_bytes)
                            .map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the private ML-KEM-768 key in priv attribute"
                                        .into(),
                                ))
                            })?;
                        let encapsulation_key = EncapsulationKey::new_from_slice(&pub_bytes)
                            .map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the public ML-KEM-768 key in pub attribute"
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
                        Handle::MlKem768PrivateKey(decapsulation_key)
                    },
                    CryptoAlgorithm::MlKem1024 => {
                        let decapsulation_key = DecapsulationKey::new_from_slice(&priv_bytes)
                            .map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the private ML-KEM-1024 key in priv attribute"
                                        .into(),
                                ))
                            })?;
                        let encapsulation_key = EncapsulationKey::new_from_slice(&pub_bytes)
                            .map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the public ML-KEM-1024 key in pub attribute"
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
                        Handle::MlKem1024PrivateKey(decapsulation_key)
                    },
                    name => {
                        return Err(Error::NotSupported(Some(format!(
                            "{} is not an ML-KEM algorithm",
                            name.as_str()
                        ))));
                    },
                };
                (KeyType::Private, private_key_handle)
            }
            // Otherwise:
            else {
                // Step 2.9.1. If the pub attribute of jwk does not contain a valid base64url
                // encoded public key representing an ML-KEM public key, then throw a DataError.
                let pub_bytes = jwk.decode_required_string_field(JwkStringField::Pub)?;

                // Step 2.9.2. Let key be a new CryptoKey object that represents the ML-KEM public
                // key identified by interpreting the pub attribute of jwk as a base64url encoded
                // public key.
                // Step 2.9.3. Set the [[type]] internal slot of Key to "public".
                // NOTE: The CryptoKey object is created in Step 2.10 - 2.12.
                let public_key_handle = match normalized_algorithm.name {
                    CryptoAlgorithm::MlKem512 => {
                        let encapsulation_key = EncapsulationKey::new_from_slice(&pub_bytes)
                            .map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the public ML-KEM-512 key in pub attribute"
                                        .into(),
                                ))
                            })?;
                        Handle::MlKem512PublicKey(encapsulation_key)
                    },
                    CryptoAlgorithm::MlKem768 => {
                        let encapsulation_key = EncapsulationKey::new_from_slice(&pub_bytes)
                            .map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the public ML-KEM-768 key in pub attribute"
                                        .into(),
                                ))
                            })?;
                        Handle::MlKem768PublicKey(encapsulation_key)
                    },
                    CryptoAlgorithm::MlKem1024 => {
                        let encapsulation_key = EncapsulationKey::new_from_slice(&pub_bytes)
                            .map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the public ML-KEM-1024 key in pub attribute"
                                        .into(),
                                ))
                            })?;
                        Handle::MlKem1024PublicKey(encapsulation_key)
                    },
                    name => {
                        return Err(Error::NotSupported(Some(format!(
                            "{} is not an ML-KEM algorithm",
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
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name,
            };
            CryptoKey::new(
                cx,
                global,
                key_type,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
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

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-kem-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    // Step 1. If the underlying cryptographic key material represented by the [[handle]] internal
    // slot of key cannot be accessed, then throw an OperationError.

    // Step 2.
    let result = match format {
        // If format is "spki":
        KeyFormat::Spki => {
            // Step 2.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"public\"".into(),
                )));
            }

            // Step 2.2. Let keyAlgorithm be the [[algorithm]] internal slot of key.
            let KeyAlgorithmAndDerivatives::KeyAlgorithm(key_algorithm) = key.algorithm() else {
                return Err(Error::Operation(Some(
                    "[[algorithm]] internal slot of key is not a KeyAlgorithm".into(),
                )));
            };

            // Step 2.3.
            // Let data be an instance of the SubjectPublicKeyInfo ASN.1 structure defined in
            // [RFC5280] with the following properties:
            //
            //     Set the algorithm field to an AlgorithmIdentifier ASN.1 type with the following
            //     properties:
            //
            //         If the name member of keyAlgorithm is "ML-KEM-512":
            //             Set the algorithm object identifier to the id-alg-ml-kem-512
            //             (2.16.840.1.101.3.4.4.1) OID.
            //
            //         If the name member of keyAlgorithm is "ML-KEM-768":
            //             Set the algorithm object identifier to the id-alg-ml-kem-768
            //             (2.16.840.1.101.3.4.4.2) OID.
            //
            //         If the name member of keyAlgorithm is "ML-KEM-1024":
            //             Set the algorithm object identifier to the id-alg-ml-kem-1024
            //             (2.16.840.1.101.3.4.4.3) OID.
            //
            //         Otherwise:
            //             throw a NotSupportedError.
            //
            //     Set the subjectPublicKey field to keyData.
            let data = match (key_algorithm.name, key.handle()) {
                (CryptoAlgorithm::MlKem512, Handle::MlKem512PublicKey(public_key)) => {
                    public_key.to_public_key_der().map_err(|_| {
                        Error::Operation(Some(
                            "Failed to encode the ML-KEM-512 public key into SPKI format".into(),
                        ))
                    })?
                },
                (CryptoAlgorithm::MlKem768, Handle::MlKem768PublicKey(public_key)) => {
                    public_key.to_public_key_der().map_err(|_| {
                        Error::Operation(Some(
                            "Failed to encode the ML-KEM-768 public key into SPKI format".into(),
                        ))
                    })?
                },
                (CryptoAlgorithm::MlKem1024, Handle::MlKem1024PublicKey(public_key)) => {
                    public_key.to_public_key_der().map_err(|_| {
                        Error::Operation(Some(
                            "Failed to encode the ML-KEM-1024 public key into SPKI format".into(),
                        ))
                    })?
                },
                _ => {
                    return Err(Error::Operation(Some(
                        "The key handle is not representing an ML-KEM public key".into(),
                    )));
                },
            };

            // Step 2.4. Let result be the result of DER-encoding data.
            ExportedKey::new_bytes(data.into_vec())
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 2.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"private\"".into(),
                )));
            }

            // Step 2.2. Let keyAlgorithm be the [[algorithm]] internal slot of key.
            let KeyAlgorithmAndDerivatives::KeyAlgorithm(key_algorithm) = key.algorithm() else {
                return Err(Error::Operation(Some(
                    "[[algorithm]] internal slot of key is not a KeyAlgorithm".into(),
                )));
            };

            // Step 2.3.
            // Let data be an instance of the PrivateKeyInfo ASN.1 structure defined in [RFC5208]
            // with the following properties:
            //
            //     Set the version field to 0.
            //
            //     Set the privateKeyAlgorithm field to a PrivateKeyAlgorithmIdentifier ASN.1 type
            //     with the following properties:
            //
            //         If the name member of keyAlgorithm is "ML-KEM-512":
            //             Set the algorithm object identifier to the id-alg-ml-kem-512
            //             (2.16.840.1.101.3.4.4.1) OID.
            //
            //         If the name member of keyAlgorithm is "ML-KEM-768":
            //             Set the algorithm object identifier to the id-alg-ml-kem-768
            //             (2.16.840.1.101.3.4.4.2) OID.
            //
            //         If the name member of keyAlgorithm is "ML-KEM-1024":
            //             Set the algorithm object identifier to the id-alg-ml-kem-1024
            //             (2.16.840.1.101.3.4.4.3) OID.
            //
            //         Otherwise:
            //             throw a NotSupportedError.
            //
            //     Set the privateKey field as follows:
            //
            //         If the name member of keyAlgorithm is "ML-KEM-512":
            //             Set the privateKey field to the result of DER-encoding a
            //             ML-KEM-512-PrivateKey ASN.1 type that represents the ML-KEM private key
            //             seed represented by the [[handle]] internal slot of key using the
            //             seed-only format (using a context-specific [0] primitive tag with an
            //             implicit encoding of OCTET STRING).
            //
            //         If the name member of keyAlgorithm is "ML-KEM-768":
            //             Set the privateKey field to the result of DER-encoding a
            //             ML-KEM-768-PrivateKey ASN.1 type that represents the ML-KEM private key
            //             seed represented by the [[handle]] internal slot of key using the
            //             seed-only format (using a context-specific [0] primitive tag with an
            //             implicit encoding of OCTET STRING).
            //
            //         If the name member of keyAlgorithm is "ML-KEM-1024":
            //             Set the privateKey field to the result of DER-encoding a
            //             ML-KEM-1024-PrivateKey ASN.1 type that represents the ML-KEM private key
            //             seed represented by the [[handle]] internal slot of key using the
            //             seed-only format (using a context-specific [0] primitive tag with an
            //             implicit encoding of OCTET STRING).
            //
            //         Otherwise:
            //             throw a NotSupportedError.
            let private_key_info = match (key_algorithm.name, key.handle()) {
                (CryptoAlgorithm::MlKem512, Handle::MlKem512PrivateKey(private_key)) => {
                    private_key.to_pkcs8_der().map_err(|_| {
                        Error::Operation(Some(
                            "Failed to encode the ML-KEM-512 private key into PKCS#8 format".into(),
                        ))
                    })?
                },
                (CryptoAlgorithm::MlKem768, Handle::MlKem768PrivateKey(private_key)) => {
                    private_key.to_pkcs8_der().map_err(|_| {
                        Error::Operation(Some(
                            "Failed to encode the ML-KEM-768 private key into PKCS#8 format".into(),
                        ))
                    })?
                },
                (CryptoAlgorithm::MlKem1024, Handle::MlKem1024PrivateKey(private_key)) => {
                    private_key.to_pkcs8_der().map_err(|_| {
                        Error::Operation(Some(
                            "Failed to encode the ML-KEM-1024 private key into PKCS#8 format"
                                .into(),
                        ))
                    })?
                },
                _ => {
                    return Err(Error::Operation(Some(
                        "The key handle is not representing an ML-KEM private key".into(),
                    )));
                },
            };

            // Step 2.4. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(private_key_info.to_bytes())
        },
        // If format is "raw-public":
        KeyFormat::Raw_public => {
            // Step 2.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"public\"".into(),
                )));
            }

            // Step 2.2. Let data be a byte sequence containing the raw octets of the key
            // represented by the [[handle]] internal slot of key.
            let data = match key.handle() {
                Handle::MlKem512PublicKey(public_key) => public_key.to_bytes().as_slice().to_vec(),
                Handle::MlKem768PublicKey(public_key) => public_key.to_bytes().as_slice().to_vec(),
                Handle::MlKem1024PublicKey(public_key) => public_key.to_bytes().as_slice().to_vec(),
                _ => {
                    return Err(Error::Operation(Some(
                        "The key handle is not representing an ML-KEM public key".into(),
                    )));
                },
            };

            // Step 2.3. Let result be data.
            ExportedKey::new_bytes(data)
        },
        // If format is "raw-seed":
        KeyFormat::Raw_seed => {
            // Step 2.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"private\"".into(),
                )));
            }

            // Step 2.2. Let data be a byte sequence containing the concatenation of the d and z
            // seed variables of the key represented by the [[handle]] internal slot of key.
            let data = match key.handle() {
                Handle::MlKem512PrivateKey(private_key) => private_key
                    .to_seed()
                    .expect("This decapsulation key should contain seed value")
                    .as_slice()
                    .to_vec(),
                Handle::MlKem768PrivateKey(private_key) => private_key
                    .to_seed()
                    .expect("This decapsulation key should contain seed value")
                    .as_slice()
                    .to_vec(),
                Handle::MlKem1024PrivateKey(private_key) => private_key
                    .to_seed()
                    .expect("This decapsulation key should contain seed value")
                    .as_slice()
                    .to_vec(),
                _ => {
                    return Err(Error::Operation(Some(
                        "The key handle is not representing an ML-KEM private key".into(),
                    )));
                },
            };

            // Step 2.3. Let result be data.
            ExportedKey::new_bytes(data)
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // The JWK format for ML-KEM is not standardized yet and thus subject to change.

            // Step 2.1. Let jwk be a new JsonWebKey dictionary.
            let mut jwk = JsonWebKey::default();

            // Step 2.2. Let keyAlgorithm be the [[algorithm]] internal slot of key.
            let KeyAlgorithmAndDerivatives::KeyAlgorithm(key_algorithm) = key.algorithm() else {
                return Err(Error::Operation(Some(
                    "[[algorithm]] internal slot of key is not a KeyAlgorithm".into(),
                )));
            };

            // Step 2.3. Set the kty attribute of jwk to "AKP".
            jwk.kty = Some(DOMString::from("AKP"));

            // Step 2.4. Set the alg attribute of jwk to the alg value corresponding to the name
            // member of normalizedAlgorithm indicated in Section 8 of [draft-ietf-jose-pqc-kem-05]
            // (Figure 1).
            //
            // <https://www.ietf.org/archive/id/draft-ietf-jose-pqc-kem-01.html#direct-table>
            let alg = match key_algorithm.name {
                CryptoAlgorithm::MlKem512 => "ML-KEM-512",
                CryptoAlgorithm::MlKem768 => "ML-KEM-768",
                CryptoAlgorithm::MlKem1024 => "ML-KEM-1024",
                _ => {
                    return Err(Error::Operation(Some(format!(
                        "{} is not an ML-KEM algorithm",
                        key_algorithm.name.as_str()
                    ))));
                },
            };
            jwk.alg = Some(DOMString::from(alg));

            // Step 2.5. Set the pub attribute of jwk to the base64url encoded public key
            // corresponding to the [[handle]] internal slot of key.
            // Step 2.6.
            // If the [[type]] internal slot of key is "private":
            //     Set the priv attribute of jwk to the base64url encoded seed represented by the
            //     [[handle]] internal slot of key.
            if key.Type() == KeyType::Private {
                match key.handle() {
                    Handle::MlKem512PrivateKey(private_key) => {
                        jwk.encode_string_field(
                            JwkStringField::Priv,
                            private_key
                                .to_seed()
                                .expect("This decapsulation key should contain seed value")
                                .as_slice(),
                        );
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            private_key.encapsulation_key().to_bytes().as_slice(),
                        );
                    },
                    Handle::MlKem768PrivateKey(private_key) => {
                        jwk.encode_string_field(
                            JwkStringField::Priv,
                            private_key
                                .to_seed()
                                .expect("This decapsulation key should contain seed value")
                                .as_slice(),
                        );
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            private_key.encapsulation_key().to_bytes().as_slice(),
                        );
                    },
                    Handle::MlKem1024PrivateKey(private_key) => {
                        jwk.encode_string_field(
                            JwkStringField::Priv,
                            private_key
                                .to_seed()
                                .expect("This decapsulation key should contain seed value")
                                .as_slice(),
                        );
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            private_key.encapsulation_key().to_bytes().as_slice(),
                        );
                    },
                    _ => {
                        return Err(Error::Operation(Some(
                            "The key handle is not representing an ML-KEM private key".into(),
                        )));
                    },
                }
            } else {
                match key.handle() {
                    Handle::MlKem512PublicKey(public_key) => {
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            public_key.to_bytes().as_slice(),
                        );
                    },
                    Handle::MlKem768PublicKey(public_key) => {
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            public_key.to_bytes().as_slice(),
                        );
                    },
                    Handle::MlKem1024PublicKey(public_key) => {
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            public_key.to_bytes().as_slice(),
                        );
                    },
                    _ => {
                        return Err(Error::Operation(Some(
                            "The key handle is not representing an ML-KEM public key".into(),
                        )));
                    },
                };
            }

            // Step 2.7. Set the key_ops attribute of jwk to the usages attribute of key.
            jwk.set_key_ops(&key.usages());

            // Step 2.8. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 2.9. Let result be jwk.
            ExportedKey::new_jwk(jwk)
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported export key format for ML-KEM key".into(),
            )));
        },
    };

    // Step 3. Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-getPublicKey>
/// Step 9 - 15, for ML-KEM
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
        Handle::MlKem512PrivateKey(decapsulation_key) => {
            Handle::MlKem512PublicKey(decapsulation_key.encapsulation_key().clone())
        },
        Handle::MlKem768PrivateKey(decapsulation_key) => {
            Handle::MlKem768PublicKey(decapsulation_key.encapsulation_key().clone())
        },
        Handle::MlKem1024PrivateKey(decapsulation_key) => {
            Handle::MlKem1024PublicKey(decapsulation_key.encapsulation_key().clone())
        },
        _ => {
            return Err(Error::Operation(Some(
                "[[handle]] internal slot of key is not an ML-KEM private key".into(),
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
