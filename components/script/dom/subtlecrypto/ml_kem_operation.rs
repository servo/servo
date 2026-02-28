/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use der::asn1::{BitString, OctetString};
use der::{AnyRef, Choice, Decode, Encode, Sequence};
use js::context::JSContext;
use ml_kem::kem::{Decapsulate, Encapsulate, EncapsulationKey};
use ml_kem::{
    B32, Encoded, EncodedSizeUser, KemCore, MlKem512, MlKem512Params, MlKem768, MlKem768Params,
    MlKem1024, MlKem1024Params,
};
use pkcs8::rand_core::{OsRng, RngCore};
use pkcs8::spki::AlgorithmIdentifier;
use pkcs8::{ObjectIdentifier, PrivateKeyInfo, SubjectPublicKeyInfo};

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
    ALG_ML_KEM_512, ALG_ML_KEM_768, ALG_ML_KEM_1024, ExportedKey, JsonWebKeyExt, JwkStringField,
    KeyAlgorithmAndDerivatives, SubtleAlgorithm, SubtleEncapsulatedBits, SubtleKeyAlgorithm,
};

/// Object Identifier (OID) of MK-KEM-512
/// Section 3 of <https://datatracker.ietf.org/doc/draft-ietf-lamps-kyber-certificates/>
const ID_ALG_ML_KEM_512: &str = "2.16.840.1.101.3.4.4.1";

/// Object Identifier (OID) of MK-KEM-768
/// Section 3 of <https://datatracker.ietf.org/doc/draft-ietf-lamps-kyber-certificates/>
const ID_ALG_ML_KEM_768: &str = "2.16.840.1.101.3.4.4.2";

/// Object Identifier (OID) of MK-KEM-1024
/// Section 3 of <https://datatracker.ietf.org/doc/draft-ietf-lamps-kyber-certificates/>
const ID_ALG_ML_KEM_1024: &str = "2.16.840.1.101.3.4.4.3";

/// Structure in Rust representing the `both` SEQUENCE used in the following ASN.1 structures, as
/// defined in [draft-ietf-lamps-kyber-certificates-11 Section 6].
///
/// - ASN.1 ML-KEM-512-PrivateKey Structure
/// - ASN.1 ML-KEM-768-PrivateKey Structure
/// - ASN.1 ML-KEM-1024-PrivateKey Structure
///
/// <https://datatracker.ietf.org/doc/draft-ietf-lamps-kyber-certificates/>
///
/// ```text
/// both SEQUENCE {
///   seed OCTET STRING (SIZE (64)),
///   expandedKey OCTET STRING (SIZE (1632))
///   }
/// ```
///
/// ```text
/// both SEQUENCE {
///   seed OCTET STRING (SIZE (64)),
///   expandedKey OCTET STRING (SIZE (2400))
///   }
/// ```
///
/// ```text
/// both SEQUENCE {
///   seed OCTET STRING (SIZE (64)),
///   expandedKey OCTET STRING (SIZE (3168))
///   }
/// ```
#[derive(Sequence)]
struct Both {
    seed: OctetString,
    expanded_key: OctetString,
}

/// Structure in Rust representing all the following three structures as defined in
/// [draft-ietf-lamps-kyber-certificates-11 Section 6].
///
/// - ASN.1 ML-KEM-512-PrivateKey Structure
/// - ASN.1 ML-KEM-768-PrivateKey Structure
/// - ASN.1 ML-KEM-1024-PrivateKey Structure
///
/// <https://datatracker.ietf.org/doc/draft-ietf-lamps-kyber-certificates/>
///
/// ```text
/// ML-KEM-512-PrivateKey ::= CHOICE {
///   seed [0] OCTET STRING (SIZE (64)),
///   expandedKey OCTET STRING (SIZE (1632)),
///   both SEQUENCE {
///     seed OCTET STRING (SIZE (64)),
///     expandedKey OCTET STRING (SIZE (1632))
///     }
///   }
/// ```
///
/// ```text
/// ML-KEM-768-PrivateKey ::= CHOICE {
///   seed [0] OCTET STRING (SIZE (64)),
///   expandedKey OCTET STRING (SIZE (2400)),
///   both SEQUENCE {
///     seed OCTET STRING (SIZE (64)),
///     expandedKey OCTET STRING (SIZE (2400))
///     }
///   }
/// ```
///
/// ```text
/// ML-KEM-1024-PrivateKey ::= CHOICE {
///   seed [0] OCTET STRING (SIZE (64)),
///   expandedKey OCTET STRING (SIZE (3168)),
///   both SEQUENCE {
///     seed OCTET STRING (SIZE (64)),
///     expandedKey OCTET STRING (SIZE (3168))
///     }
///   }
/// ```
#[derive(Choice)]
enum MlKemPrivateKeyStructure {
    #[asn1(context_specific = "0", tag_mode = "IMPLICIT")]
    Seed(OctetString),
    ExpandedKey(OctetString),
    Both(Both),
}

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-kem-operations-encapsulate>
pub(crate) fn encapsulate(
    normalized_algorithm: &SubtleAlgorithm,
    key: &CryptoKey,
) -> Result<SubtleEncapsulatedBits, Error> {
    // Step 1. If the [[type]] internal slot of key is not "public", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Public {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"public\"".to_string(),
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
    let (shared_key, ciphertext) = match normalized_algorithm.name.as_str() {
        ALG_ML_KEM_512 => {
            let Handle::MlKem512PublicKey(encoded_ek) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-KEM-512 public key".to_string(),
                )));
            };
            let ek = EncapsulationKey::<MlKem512Params>::from_bytes(encoded_ek);
            let (encoded_ciphertext, shared_key) = ek.encapsulate(&mut OsRng).map_err(|_| {
                Error::Operation(Some("Failed to perform ML-KEM encapsulation".to_string()))
            })?;
            (shared_key.to_vec(), encoded_ciphertext.to_vec())
        },
        ALG_ML_KEM_768 => {
            let Handle::MlKem768PublicKey(encoded_ek) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-KEM-768 public key".to_string(),
                )));
            };
            let ek = EncapsulationKey::<MlKem768Params>::from_bytes(encoded_ek);
            let (encoded_ciphertext, shared_key) = ek.encapsulate(&mut OsRng).map_err(|_| {
                Error::Operation(Some("Failed to perform ML-KEM encapsulation".to_string()))
            })?;
            (shared_key.to_vec(), encoded_ciphertext.to_vec())
        },
        ALG_ML_KEM_1024 => {
            let Handle::MlKem1024PublicKey(encoded_ek) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-KEM-1024 public key".to_string(),
                )));
            };
            let ek = EncapsulationKey::<MlKem1024Params>::from_bytes(encoded_ek);
            let (encoded_ciphertext, shared_key) = ek.encapsulate(&mut OsRng).map_err(|_| {
                Error::Operation(Some("Failed to perform ML-KEM encapsulation".to_string()))
            })?;
            (shared_key.to_vec(), encoded_ciphertext.to_vec())
        },
        _ => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not an ML-KEM algorithm",
                normalized_algorithm.name.as_str()
            ))));
        },
    };

    // Step 6. Let result be a new EncapsulatedBits dictionary.
    // Step 7. Set the sharedKey attribute of result to the result of creating an ArrayBuffer
    // containing sharedKey.
    // Step 8. Set the ciphertext attribute of result to the result of creating an ArrayBuffer
    // containing ciphertext.
    let result = SubtleEncapsulatedBits {
        shared_key: Some(shared_key),
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
            "[[type]] internal slot of key is not \"private\"".to_string(),
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
    let shared_key = match normalized_algorithm.name.as_str() {
        ALG_ML_KEM_512 => {
            let Handle::MlKem512PrivateKey(seed) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-KEM-512 private key".to_string(),
                )));
            };
            let ciphertext = ciphertext
                .try_into()
                .map_err(|_| Error::Operation(Some("Failed to load the ciphertext".to_string())))?;
            let (dk, _) = MlKem512::generate_deterministic(&seed.0, &seed.1);
            dk.decapsulate(ciphertext)
                .map_err(|_| {
                    Error::Operation(Some("Failed to perform ML-KEM decapsulation".to_string()))
                })?
                .to_vec()
        },
        ALG_ML_KEM_768 => {
            let Handle::MlKem768PrivateKey(seed) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-KEM-768 private key".to_string(),
                )));
            };
            let ciphertext = ciphertext
                .try_into()
                .map_err(|_| Error::Operation(Some("Failed to load the ciphertext".to_string())))?;
            let (dk, _) = MlKem768::generate_deterministic(&seed.0, &seed.1);
            dk.decapsulate(ciphertext)
                .map_err(|_| {
                    Error::Operation(Some("Failed to perform ML-KEM decapsulation".to_string()))
                })?
                .to_vec()
        },
        ALG_ML_KEM_1024 => {
            let Handle::MlKem1024PrivateKey(seed) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-KEM-1024 private key".to_string(),
                )));
            };
            let ciphertext = ciphertext
                .try_into()
                .map_err(|_| Error::Operation(Some("Failed to load the ciphertext".to_string())))?;
            let (dk, _) = MlKem1024::generate_deterministic(&seed.0, &seed.1);
            dk.decapsulate(ciphertext)
                .map_err(|_| {
                    Error::Operation(Some("Failed to perform ML-KEM decapsulation".to_string()))
                })?
                .to_vec()
        },
        _ => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not an ML-KEM algorithm",
                normalized_algorithm.name.as_str()
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
                .to_string(),
        )));
    }

    // Step 2. Generate an ML-KEM key pair, as described in Section 7.1 of [FIPS-203], with the
    // parameter set indicated by the name member of normalizedAlgorithm.
    // Step 3. If the key generation step fails, then throw an OperationError.
    let mut seed_bytes = vec![0u8; 64];
    OsRng.fill_bytes(&mut seed_bytes);
    let (private_key_handle, public_key_handle) =
        convert_seed_to_handles(&normalized_algorithm.name, &seed_bytes, None, None)?;

    // Step 4. Let algorithm be a new KeyAlgorithm object.
    // Step 5. Set the name attribute of algorithm to the name attribute of normalizedAlgorithm.
    let algorithm = SubtleKeyAlgorithm {
        name: normalized_algorithm.name.clone(),
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
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm.clone()),
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
                        .to_string(),
                )));
            }

            // Step 2.2. Let spki be the result of running the parse a subjectPublicKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurred while parsing, then throw a DataError.
            let spki =
                SubjectPublicKeyInfo::<AnyRef, BitString>::from_der(key_data).map_err(|_| {
                    Error::Data(Some(
                        "Failed to parse SubjectPublicKeyInfo over keyData".to_string(),
                    ))
                })?;

            // Step 2.4.
            // If the name member of normalizedAlgorithm is "ML-KEM-512":
            //     Let expectedOid be id-alg-ml-kem-512 (2.16.840.1.101.3.4.4.1).
            // If the name member of normalizedAlgorithm is "ML-KEM-768":
            //     Let expectedOid be id-alg-ml-kem-768 (2.16.840.1.101.3.4.4.2).
            // If the name member of normalizedAlgorithm is "ML-KEM-1024":
            //     Let expectedOid be id-alg-ml-kem-1024 (2.16.840.1.101.3.4.4.3).
            // Otherwise:
            //     throw a NotSupportedError.
            let expected_oid = match normalized_algorithm.name.as_str() {
                ALG_ML_KEM_512 => ObjectIdentifier::new_unwrap(ID_ALG_ML_KEM_512),
                ALG_ML_KEM_768 => ObjectIdentifier::new_unwrap(ID_ALG_ML_KEM_768),
                ALG_ML_KEM_1024 => ObjectIdentifier::new_unwrap(ID_ALG_ML_KEM_1024),
                _ => {
                    return Err(Error::NotSupported(Some(format!(
                        "{} is not an ML-KEM algorithm",
                        normalized_algorithm.name.as_str()
                    ))));
                },
            };

            // Step 2.5. If the algorithm object identifier field of the algorithm
            // AlgorithmIdentifier field of spki is not equal to expectedOid, then throw a
            // DataError.
            if spki.algorithm.oid != expected_oid {
                return Err(Error::Data(Some(
                    "Algorithm object identifier of spki in not equal to expectedOid".to_string(),
                )));
            }

            // Step 2.6. If the parameters field of the algorithm AlgorithmIdentifier field of spki
            // is present, then throw a DataError.
            if spki.algorithm.parameters.is_some() {
                return Err(Error::Data(Some(
                    "Parameters field of spki is present".to_string(),
                )));
            }

            // Step 2.7. Let publicKey be the ML-KEM public key identified by the subjectPublicKey
            // field of spki.
            let key_bytes = spki.subject_public_key.as_bytes().ok_or(Error::Data(Some(
                "Fail to parse byte sequence over SubjectPublicKey field of spki".to_string(),
            )))?;
            let public_key = convert_public_key_to_handle(&normalized_algorithm.name, key_bytes)?;

            // Step 2.8. Let key be a new CryptoKey that represents publicKey.
            // Step 2.9. Set the [[type]] internal slot of key to "public"
            // Step 2.10. Let algorithm be a new KeyAlgorithm.
            // Step 2.11. Set the name attribute of algorithm to the name attribute of
            // normalizedAlgorithm.
            // Step 2.12. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name.clone(),
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
                        .to_string(),
                )));
            }

            // Step 2.2. Let privateKeyInfo be the result of running the parse a privateKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurs while parsing, then throw a DataError.
            let private_key_info = PrivateKeyInfo::from_der(key_data).map_err(|_| {
                Error::Data(Some(
                    "Fail to parse PrivateKeyInfo over keyData".to_string(),
                ))
            })?;

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
            let expected_oid = match normalized_algorithm.name.as_str() {
                ALG_ML_KEM_512 => ObjectIdentifier::new_unwrap(ID_ALG_ML_KEM_512),
                ALG_ML_KEM_768 => ObjectIdentifier::new_unwrap(ID_ALG_ML_KEM_768),
                ALG_ML_KEM_1024 => ObjectIdentifier::new_unwrap(ID_ALG_ML_KEM_1024),
                _ => {
                    return Err(Error::NotSupported(Some(format!(
                        "{} is not an ML-KEM algorithm",
                        normalized_algorithm.name.as_str()
                    ))));
                },
            };

            // Step 2.5. If the algorithm object identifier field of the privateKeyAlgorithm
            // PrivateKeyAlgorithm field of privateKeyInfo is not equal to expectedOid, then throw
            // a DataError.
            if private_key_info.algorithm.oid != expected_oid {
                return Err(Error::Data(Some(
                    "Algorithm object identifier of PrivateKeyInfo is not equal to expectedOid"
                        .to_string(),
                )));
            }

            // Step 2.6. If the parameters field of the privateKeyAlgorithm
            // PrivateKeyAlgorithmIdentifier field of privateKeyInfo is present, then throw a
            // DataError.
            if private_key_info.algorithm.parameters.is_some() {
                return Err(Error::Data(Some(
                    "Parameters field of PrivateKeyInfo is present".to_string(),
                )));
            }

            // Step 2.7. Let mlKemPrivateKey be the result of performing the parse an ASN.1
            // structure algorithm, with data as the privateKey field of privateKeyInfo, structure
            // as asn1Structure, and exactData set to true.
            // Step 2.8. If an error occurred while parsing, then throw a DataError.
            //
            // NOTE: There is an ongoing discussion on how to handle private keys in the
            // "expandedKey" and "both" formats.
            // - <https://github.com/WICG/webcrypto-modern-algos/issues/29>
            // - <https://github.com/WICG/webcrypto-modern-algos/pull/34>
            // For now, we accept the "seed" format, reject the "expandedKey" format with a
            // NotSupportedError, and accept the "both" format subject to a consistency check. This
            // behavior may change in the future once the discussion is settled.
            let private_key_structure =
                MlKemPrivateKeyStructure::from_der(private_key_info.private_key).map_err(|_| {
                    Error::Data(Some(
                        "Failed to parse privateKey field of PrivateKeyInfo".to_string(),
                    ))
                })?;
            let ml_kem_private_key = match private_key_structure {
                MlKemPrivateKeyStructure::Seed(seed) => {
                    let (private_key_handle, _) = convert_seed_to_handles(
                        &normalized_algorithm.name,
                        seed.as_bytes(),
                        None,
                        None,
                    )?;
                    private_key_handle
                },
                MlKemPrivateKeyStructure::ExpandedKey(_) => {
                    return Err(Error::NotSupported(Some(
                        "Not support \"expandedKey\" format of ASN.1 ML-KEM private key structures"
                            .to_string(),
                    )));
                },
                MlKemPrivateKeyStructure::Both(both) => {
                    let (private_key_handle, _) = convert_seed_to_handles(
                        &normalized_algorithm.name,
                        both.seed.as_bytes(),
                        Some(both.expanded_key.as_bytes()),
                        None,
                    )?;
                    private_key_handle
                },
            };

            // Step 2.9. Let key be a new CryptoKey that represents the ML-KEM private key
            // identified by mlKemPrivateKey.
            // Step 2.10. Set the [[type]] internal slot of key to "private"
            // Step 2.11. Let algorithm be a new KeyAlgorithm.
            // Step 2.12. Set the name attribute of algorithm to the name attribute of
            // normalizedAlgorithm.
            // Step 2.13. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name.clone(),
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
                        .to_string(),
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
            let public_key_handle =
                convert_public_key_to_handle(&normalized_algorithm.name, key_data)?;
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name.clone(),
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                public_key_handle,
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
                        .to_string(),
                )));
            }

            // Step 2.2. Let data be keyData.
            let data = key_data;

            // Step 2.3. If the length in bits of data is not 512 then throw a DataError.
            // Step 2.4. Let privateKey be the result of performing the ML-KEM.KeyGen_internal
            // function described in Section 6.1 of [FIPS-203] with the parameter set indicated by
            // the name member of normalizedAlgorithm, using the first 256 bits of data as d and
            // the last 256 bits of data as z.
            let (private_key_handle, _) =
                convert_seed_to_handles(&normalized_algorithm.name, data, None, None)?;

            // Step 2.5. Let key be a new CryptoKey that represents the ML-KEM private key
            // identified by privateKey.
            // Step 2.6. Set the [[type]] internal slot of key to "private"
            // Step 2.7. Let algorithm be a new KeyAlgorithm.
            // Step 2.8. Set the name attribute of algorithm to the name attribute of
            // normalizedAlgorithm.
            // Step 2.9. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name.clone(),
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Private,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                private_key_handle,
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
                        .to_string(),
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
                        .to_string(),
                )));
            }

            // Step 2.4. If the kty field of jwk is not "AKP", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "AKP") {
                return Err(Error::Data(Some(
                    "The kty field of jwk is not \"AKP\"".to_string(),
                )));
            }

            // Step 2.5. If the alg field of jwk is not one of the alg values corresponding to the
            // name member of normalizedAlgorithm indicated in Section 8 of
            // [draft-ietf-jose-pqc-kem-01] (Figure 1 or 2), then throw a DataError.
            match normalized_algorithm.name.as_str() {
                ALG_ML_KEM_512 => {
                    if jwk
                        .alg
                        .as_ref()
                        .is_none_or(|alg| alg != "MLKEM512" && alg != "MLKEM512-AES128KW")
                    {
                        return Err(Error::Data(Some(
                            "The alg field of jwk is not invalid.".to_string(),
                        )));
                    }
                },
                ALG_ML_KEM_768 => {
                    if jwk
                        .alg
                        .as_ref()
                        .is_none_or(|alg| alg != "MLKEM768" && alg != "MLKEM768-AES192KW")
                    {
                        return Err(Error::Data(Some(
                            "The alg field of jwk is not invalid.".to_string(),
                        )));
                    }
                },
                ALG_ML_KEM_1024 => {
                    if jwk
                        .alg
                        .as_ref()
                        .is_none_or(|alg| alg != "MLKEM1024" && alg != "MLKEM1024-AES256KW")
                    {
                        return Err(Error::Data(Some(
                            "The alg field of jwk is not invalid.".to_string(),
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
                        .to_string(),
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
                        .to_string(),
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
                // NOTE: Completed in Step 2.10 - 2.12.
                let pub_bytes = jwk.decode_required_string_field(JwkStringField::Pub)?;
                let (private_key_handle, _) = convert_seed_to_handles(
                    &normalized_algorithm.name,
                    &priv_bytes,
                    None,
                    Some(&pub_bytes),
                )?;

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
                // NOTE: Completed in Step 2.10 - 2.12.
                let public_key_handle =
                    convert_public_key_to_handle(&normalized_algorithm.name, &pub_bytes)?;

                (KeyType::Public, public_key_handle)
            };

            // Step 2.10. Let algorithm be a new instance of a KeyAlgorithm object.
            // Step 2.11. Set the name attribute of algorithm to the name member of
            // normalizedAlgorithm.
            // Step 2.12. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name.clone(),
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
                "Unsupported import key format for ML-KEM key".to_string(),
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
                    "[[type]] internal slot of key is not \"public\"".to_string(),
                )));
            }

            // Step 2.2. Let keyAlgorithm be the [[algorithm]] internal slot of key.
            let KeyAlgorithmAndDerivatives::KeyAlgorithm(key_algorithm) = key.algorithm() else {
                return Err(Error::Operation(Some(
                    "[[algorithm]] internal slot of key is not a KeyAlgorithm".to_string(),
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
            let oid = match key_algorithm.name.as_str() {
                ALG_ML_KEM_512 => ID_ALG_ML_KEM_512,
                ALG_ML_KEM_768 => ID_ALG_ML_KEM_768,
                ALG_ML_KEM_1024 => ID_ALG_ML_KEM_1024,
                _ => {
                    return Err(Error::Operation(Some(format!(
                        "{} is not an ML-KEM algorithm",
                        key_algorithm.name.as_str()
                    ))));
                },
            };
            let key_bytes = convert_handle_to_public_key(key.handle())?;
            let subject_public_key = BitString::from_bytes(&key_bytes).map_err(|_| {
                Error::Operation(Some(
                    "Failed to encode BitString for subjectPublicKey field of SubjectPublicKeyInfo"
                        .to_string(),
                ))
            })?;
            let data = SubjectPublicKeyInfo {
                algorithm: AlgorithmIdentifier::<AnyRef> {
                    oid: ObjectIdentifier::new_unwrap(oid),
                    parameters: None,
                },
                subject_public_key,
            };

            // Step 2.4. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(data.to_der().map_err(|_| {
                Error::Operation(Some(
                    "Failed to encode SubjectPublicKeyInfo in DER format".to_string(),
                ))
            })?)
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 2.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"private\"".to_string(),
                )));
            }

            // Step 2.2. Let keyAlgorithm be the [[algorithm]] internal slot of key.
            let KeyAlgorithmAndDerivatives::KeyAlgorithm(key_algorithm) = key.algorithm() else {
                return Err(Error::Operation(Some(
                    "[[algorithm]] internal slot of key is not a KeyAlgorithm".to_string(),
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
            let oid = match key_algorithm.name.as_str() {
                ALG_ML_KEM_512 => ID_ALG_ML_KEM_512,
                ALG_ML_KEM_768 => ID_ALG_ML_KEM_768,
                ALG_ML_KEM_1024 => ID_ALG_ML_KEM_1024,
                _ => {
                    return Err(Error::Operation(Some(format!(
                        "{} is not an ML-KEM algorithm",
                        key_algorithm.name.as_str()
                    ))));
                },
            };
            let (seed_bytes, _) = convert_handle_to_seed_and_public_key(key.handle())?;
            let private_key =
                MlKemPrivateKeyStructure::Seed(OctetString::new(seed_bytes).map_err(|_| {
                    Error::Operation(Some(
                        "Failed to encode OctetString for privateKey field of \
                ASN.1 ML-KEM private key structure"
                            .to_string(),
                    ))
                })?);
            let encoded_private_key = private_key.to_der().map_err(|_| {
                Error::Operation(Some(
                    "Failed to encode ASN.1 ML-KEM private key structure in DER format".to_string(),
                ))
            })?;
            let private_key_info = PrivateKeyInfo {
                algorithm: AlgorithmIdentifier {
                    oid: ObjectIdentifier::new_unwrap(oid),
                    parameters: None,
                },
                private_key: &encoded_private_key,
                public_key: None,
            };

            // Step 2.4. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(private_key_info.to_der().map_err(|_| {
                Error::Operation(Some(
                    "Failed to encode PrivateKeyInfo in DER format".to_string(),
                ))
            })?)
        },
        // If format is "raw-public":
        KeyFormat::Raw_public => {
            // Step 2.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"public\"".to_string(),
                )));
            }

            // Step 2.2. Let data be a byte sequence containing the raw octets of the key
            // represented by the [[handle]] internal slot of key.
            let data = convert_handle_to_public_key(key.handle())?;

            // Step 2.3. Let result be data.
            ExportedKey::Bytes(data)
        },
        // If format is "raw-seed":
        KeyFormat::Raw_seed => {
            // Step 2.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"private\"".to_string(),
                )));
            }

            // Step 2.2. Let data be a byte sequence containing the concatenation of the d and z
            // seed variables of the key represented by the [[handle]] internal slot of key.
            let (data, _) = convert_handle_to_seed_and_public_key(key.handle())?;

            // Step 2.3. Let result be data.
            ExportedKey::Bytes(data)
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // The JWK format for ML-KEM is not standardized yet and thus subject to change.

            // Step 2.1. Let jwk be a new JsonWebKey dictionary.
            let mut jwk = JsonWebKey::default();

            // Step 2.2. Let keyAlgorithm be the [[algorithm]] internal slot of key.
            let KeyAlgorithmAndDerivatives::KeyAlgorithm(key_algorithm) = key.algorithm() else {
                return Err(Error::Operation(Some(
                    "[[algorithm]] internal slot of key is not a KeyAlgorithm".to_string(),
                )));
            };

            // Step 2.3. Set the kty attribute of jwk to "AKP".
            jwk.kty = Some(DOMString::from("AKP"));

            // Step 2.4. Set the alg attribute of jwk to the alg value corresponding to the name
            // member of normalizedAlgorithm indicated in Section 8 of [draft-ietf-jose-pqc-kem-01]
            // (Figure 1).
            //
            // <https://www.ietf.org/archive/id/draft-ietf-jose-pqc-kem-01.html#direct-table>
            let alg = match key_algorithm.name.as_str() {
                ALG_ML_KEM_512 => "MLKEM512",
                ALG_ML_KEM_768 => "MLKEM768",
                ALG_ML_KEM_1024 => "MLKEM1024",
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
                let (seed_bytes, public_key_bytes) =
                    convert_handle_to_seed_and_public_key(key.handle())?;
                jwk.encode_string_field(JwkStringField::Pub, &public_key_bytes);
                jwk.encode_string_field(JwkStringField::Priv, &seed_bytes);
            } else {
                let public_key_bytes = convert_handle_to_public_key(key.handle())?;
                jwk.encode_string_field(JwkStringField::Pub, &public_key_bytes);
            }

            // Step 2.7. Set the key_ops attribute of jwk to the usages attribute of key.
            jwk.set_key_ops(key.usages());

            // Step 2.8. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 2.9. Let result be jwk.
            ExportedKey::Jwk(Box::new(jwk))
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported export key format for ML-KEM key".to_string(),
            )));
        },
    };

    // Step 3. Return result.
    Ok(result)
}

/// Convert seed bytes to an ML-KEM private key handle and an ML-KEM public key handle. If private
/// key bytes and/or public key bytes are provided, it runs a consistency check against the seed.
/// If the length in bits of seed bytes is not 512, the conversion fails, or the consistency check
/// fails, throw a DataError.
fn convert_seed_to_handles(
    algo_name: &str,
    seed_bytes: &[u8],
    private_key_bytes: Option<&[u8]>,
    public_key_bytes: Option<&[u8]>,
) -> Result<(Handle, Handle), Error> {
    if seed_bytes.len() != 64 {
        return Err(Error::Data(Some(
            "The length in bits of seed bytes is not 512".to_string(),
        )));
    }

    let d: B32 = (&seed_bytes[..32]).try_into().map_err(|_| {
        Error::Data(Some(
            "Failed to parse first 256 bits of seed bytes".to_string(),
        ))
    })?;
    let z: B32 = (&seed_bytes[32..64]).try_into().map_err(|_| {
        Error::Data(Some(
            "Failed to parse last 256 bits of seed bytes".to_string(),
        ))
    })?;
    let handles = match algo_name {
        ALG_ML_KEM_512 => {
            let (decapsulation_key, encapsulation_key) = MlKem512::generate_deterministic(&d, &z);
            if let Some(private_key_bytes) = private_key_bytes {
                if private_key_bytes != decapsulation_key.as_bytes().as_slice() {
                    return Err(Error::Data(Some(
                        "The expanded private key does not match the seed".to_string(),
                    )));
                }
            }
            if let Some(public_key_bytes) = public_key_bytes {
                if public_key_bytes != encapsulation_key.as_bytes().as_slice() {
                    return Err(Error::Data(Some(
                        "The public key does not match the seed".to_string(),
                    )));
                }
            }

            (
                Handle::MlKem512PrivateKey((d, z)),
                Handle::MlKem512PublicKey(Box::new(encapsulation_key.as_bytes())),
            )
        },
        ALG_ML_KEM_768 => {
            let (decapsulation_key, encapsulation_key) = MlKem768::generate_deterministic(&d, &z);
            if let Some(private_key_bytes) = private_key_bytes {
                if private_key_bytes != decapsulation_key.as_bytes().as_slice() {
                    return Err(Error::Data(Some(
                        "The expanded private key does not match the seed".to_string(),
                    )));
                }
            }
            if let Some(public_key_bytes) = public_key_bytes {
                if public_key_bytes != encapsulation_key.as_bytes().as_slice() {
                    return Err(Error::Data(Some(
                        "The public key does not match the seed".to_string(),
                    )));
                }
            }

            (
                Handle::MlKem768PrivateKey((d, z)),
                Handle::MlKem768PublicKey(Box::new(encapsulation_key.as_bytes())),
            )
        },
        ALG_ML_KEM_1024 => {
            let (decapsulation_key, encapsulation_key) = MlKem1024::generate_deterministic(&d, &z);
            if let Some(private_key_bytes) = private_key_bytes {
                if private_key_bytes != decapsulation_key.as_bytes().as_slice() {
                    return Err(Error::Data(Some(
                        "The expanded private key does not match the seed".to_string(),
                    )));
                }
            }
            if let Some(public_key_bytes) = public_key_bytes {
                if public_key_bytes != encapsulation_key.as_bytes().as_slice() {
                    return Err(Error::Data(Some(
                        "The public key does not match the seed".to_string(),
                    )));
                }
            }

            (
                Handle::MlKem1024PrivateKey((d, z)),
                Handle::MlKem1024PublicKey(Box::new(encapsulation_key.as_bytes())),
            )
        },
        _ => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not an ML-KEM algorithm",
                algo_name
            ))));
        },
    };

    Ok(handles)
}

/// Convert public key bytes to an ML-KEM public key handle. If the conversion fails, throw a
/// DataError.
fn convert_public_key_to_handle(algo_name: &str, public_key_bytes: &[u8]) -> Result<Handle, Error> {
    let public_key_handle = match algo_name {
        ALG_ML_KEM_512 => {
            let encoded_encapsulation_key = Encoded::<EncapsulationKey<MlKem512Params>>::try_from(
                public_key_bytes,
            )
            .map_err(|_| Error::Data(Some("Failed to parse ML-KEM public key".to_string())))?;
            Handle::MlKem512PublicKey(Box::new(encoded_encapsulation_key))
        },
        ALG_ML_KEM_768 => {
            let encoded_encapsulation_key = Encoded::<EncapsulationKey<MlKem768Params>>::try_from(
                public_key_bytes,
            )
            .map_err(|_| Error::Data(Some("Failed to parse ML-KEM public key".to_string())))?;
            Handle::MlKem768PublicKey(Box::new(encoded_encapsulation_key))
        },
        ALG_ML_KEM_1024 => {
            let encoded_encapsulation_key = Encoded::<EncapsulationKey<MlKem1024Params>>::try_from(
                public_key_bytes,
            )
            .map_err(|_| Error::Data(Some("Failed to parse ML-KEM public key".to_string())))?;
            Handle::MlKem1024PublicKey(Box::new(encoded_encapsulation_key))
        },
        _ => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not an ML-KEM algorithm",
                algo_name
            ))));
        },
    };

    Ok(public_key_handle)
}

/// Convert an ML-KEM private key handle to seed bytes and public key bytes. If the handle is not
/// representing a ML-KEM private key, throw an OperationError.
fn convert_handle_to_seed_and_public_key(handle: &Handle) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let result = match handle {
        Handle::MlKem512PrivateKey((d, z)) => {
            let mut seed = d.to_vec();
            seed.extend_from_slice(z);
            let (_private_key, public_key) = MlKem512::generate_deterministic(d, z);
            (seed, public_key.as_bytes().to_vec())
        },
        Handle::MlKem768PrivateKey((d, z)) => {
            let mut seed = d.to_vec();
            seed.extend_from_slice(z);
            let (_private_key, public_key) = MlKem768::generate_deterministic(d, z);
            (seed, public_key.as_bytes().to_vec())
        },
        Handle::MlKem1024PrivateKey((d, z)) => {
            let mut seed = d.to_vec();
            seed.extend_from_slice(z);
            let (_private_key, public_key) = MlKem1024::generate_deterministic(d, z);
            (seed, public_key.as_bytes().to_vec())
        },
        _ => {
            return Err(Error::Operation(Some(
                "The key handle is not representing an ML-KEM private key".to_string(),
            )));
        },
    };

    Ok(result)
}

/// Convert an ML-KEM public key handle to public key bytes. If the handle is not representing a
/// ML-KEM public key, throw an OperationError.
fn convert_handle_to_public_key(handle: &Handle) -> Result<Vec<u8>, Error> {
    let result = match handle {
        Handle::MlKem512PublicKey(public_key) => public_key.to_vec(),
        Handle::MlKem768PublicKey(public_key) => public_key.to_vec(),
        Handle::MlKem1024PublicKey(public_key) => public_key.to_vec(),
        _ => {
            return Err(Error::Operation(Some(
                "The key handle is not representing an ML-KEM public key".to_string(),
            )));
        },
    };

    Ok(result)
}
