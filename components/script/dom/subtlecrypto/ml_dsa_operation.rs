/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use der::asn1::{BitString, OctetString};
use der::{AnyRef, Choice, Decode, Encode, Sequence};
use ml_dsa::{B32, EncodedVerifyingKey, KeyGen, MlDsa44, MlDsa65, MlDsa87};
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
    ALG_ML_DSA_44, ALG_ML_DSA_65, ALG_ML_DSA_87, ExportedKey, JsonWebKeyExt, JwkStringField,
    KeyAlgorithmAndDerivatives, SubtleAlgorithm, SubtleKeyAlgorithm,
};
use crate::script_runtime::CanGc;

/// Object Identifier (OID) of ML-DSA-44
/// Section 2 of <https://datatracker.ietf.org/doc/html/rfc9881>
const ID_ALG_ML_DSA_44: &str = "2.16.840.1.101.3.4.3.17";

/// Object Identifier (OID) of ML-DSA-65
/// Section 2 of <https://datatracker.ietf.org/doc/html/rfc9881>
const ID_ALG_ML_DSA_65: &str = "2.16.840.1.101.3.4.3.18";

/// Object Identifier (OID) of ML-DSA-87
/// Section 2 of <https://datatracker.ietf.org/doc/html/rfc9881>
const ID_ALG_ML_DSA_87: &str = "2.16.840.1.101.3.4.3.19";

/// Structure in Rust representing the `both` SEQUENCE used in the following ASN.1 structures, as
/// defined in [RFC 9881 Section 6].
///
/// - ASN.1 ML-DSA-44-PrivateKey Structure
/// - ASN.1 ML-DSA-44-PrivateKey Structure
/// - ASN.1 ML-DSA-44-PrivateKey Structure
///
/// <https://datatracker.ietf.org/doc/html/rfc9881>
///
/// ```text
/// both SEQUENCE {
///   seed OCTET STRING (SIZE (32)),
///   expandedKey OCTET STRING (SIZE (2560))
///   }
/// ```
///
/// ```text
/// both SEQUENCE {
///   seed OCTET STRING (SIZE (32)),
///   expandedKey OCTET STRING (SIZE (4032))
///   }
/// ```
///
/// ```text
/// both SEQUENCE {
///   seed OCTET STRING (SIZE (32)),
///   expandedKey OCTET STRING (SIZE (4896))
///   }
/// ```
#[derive(Sequence)]
struct Both {
    seed: OctetString,
    expanded_key: OctetString,
}

/// Structure in Rust representing all the following three structures as defined in
/// [RFC 9881 Section 6].
///
/// - ASN.1 ML-DSA-44-PrivateKey Structure
/// - ASN.1 ML-DSA-44-PrivateKey Structure
/// - ASN.1 ML-DSA-44-PrivateKey Structure
///
/// <https://datatracker.ietf.org/doc/html/rfc9881>
///
/// ```text
/// ML-DSA-44-PrivateKey ::= CHOICE {
///   seed [0] OCTET STRING (SIZE (32)),
///   expandedKey OCTET STRING (SIZE (2560)),
///   both SEQUENCE {
///       seed OCTET STRING (SIZE (32)),
///       expandedKey OCTET STRING (SIZE (2560))
///       }
///   }
/// ```
///
/// ```text
/// ML-DSA-65-PrivateKey ::= CHOICE {
///   seed [0] OCTET STRING (SIZE (32)),
///   expandedKey OCTET STRING (SIZE (4032)),
///   both SEQUENCE {
///       seed OCTET STRING (SIZE (32)),
///       expandedKey OCTET STRING (SIZE (4032))
///       }
///   }
/// ```
///
/// ```text
/// ML-DSA-87-PrivateKey ::= CHOICE {
///   seed [0] OCTET STRING (SIZE (32)),
///   expandedKey OCTET STRING (SIZE (4896)),
///   both SEQUENCE {
///       seed OCTET STRING (SIZE (32)),
///       expandedKey OCTET STRING (SIZE (4896))
///       }
///   }
/// ```
#[derive(Choice)]
enum MlDsaPrivateKeyStructure {
    #[asn1(context_specific = "0", tag_mode = "IMPLICIT")]
    Seed(OctetString),
    ExpandedKey(OctetString),
    Both(Both),
}

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-dsa-operations-generate-key>
pub(crate) fn generate_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAlgorithm,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<CryptoKeyPair, Error> {
    // Step 1. If usages contains a value which is not one of "sign" or "verify", then throw a
    // SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify))
    {
        return Err(Error::Syntax(Some(
            "Usages contains an entry which is not one of \"sign\" or \"verify\"".to_string(),
        )));
    }

    // Step 2. Generate an ML-DSA key pair, as described in Section 5.1 of [FIPS-204], with the
    // parameter set indicated by the name member of normalizedAlgorithm.
    // Step 3. If the key generation step fails, then throw an OperationError.
    let mut seed_bytes = vec![0u8; 32];
    OsRng.fill_bytes(&mut seed_bytes);
    let (private_key_handle, public_key_handle) =
        convert_seed_to_handles(&normalized_algorithm.name, &seed_bytes, None, None)?;

    // Step 4. Let algorithm be a new KeyAlgorithm object.
    // Step 5. Set the name attribute of algorithm to the name attribute of normalizedAlgorithm.
    let algorithm = SubtleKeyAlgorithm {
        name: normalized_algorithm.name.clone(),
    };

    // Step 6. Let publicKey be a new CryptoKey representing the public key of the generated key
    // pair.
    // Step 7. Set the [[type]] internal slot of publicKey to "public".
    // Step 8. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 9. Set the [[extractable]] internal slot of publicKey to true.
    // Step 10. Set the [[usages]] internal slot of publicKey to be the usage intersection of
    // usages and [ "verify" ].
    let public_key = CryptoKey::new(
        global,
        KeyType::Public,
        true,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm.clone()),
        usages
            .iter()
            .filter(|usage| **usage == KeyUsage::Verify)
            .cloned()
            .collect(),
        public_key_handle,
        can_gc,
    );

    // Step 11. Let privateKey be a new CryptoKey representing the private key of the generated key
    // pair.
    // Step 12. Set the [[type]] internal slot of privateKey to "private".
    // Step 13. Set the [[algorithm]] internal slot of privateKey to algorithm.
    // Step 14. Set the [[extractable]] internal slot of privateKey to extractable.
    // Step 15. Set the [[usages]] internal slot of privateKey to be the usage intersection of
    // usages and [ "sign" ].
    let private_key = CryptoKey::new(
        global,
        KeyType::Private,
        extractable,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
        usages
            .iter()
            .filter(|usage| **usage == KeyUsage::Sign)
            .cloned()
            .collect(),
        private_key_handle,
        can_gc,
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

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-dsa-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAlgorithm,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2.
    let key = match format {
        // If format is "spki":
        KeyFormat::Spki => {
            // Step 2.1. If usages contains a value which is not "verify" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"verify\"".to_string(),
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
            // If the name member of normalizedAlgorithm is "ML-DSA-44":
            //     Let expectedOid be id-ml-dsa-44 (2.16.840.1.101.3.4.3.17).
            // If the name member of normalizedAlgorithm is "ML-DSA-65":
            //     Let expectedOid be id-ml-dsa-65 (2.16.840.1.101.3.4.3.18).
            // If the name member of normalizedAlgorithm is "ML-DSA-87":
            //     Let expectedOid be id-ml-dsa-87 (2.16.840.1.101.3.4.3.19).
            // Otherwise:
            //     throw a NotSupportedError.
            let expected_oid = match normalized_algorithm.name.as_str() {
                ALG_ML_DSA_44 => ObjectIdentifier::new_unwrap(ID_ALG_ML_DSA_44),
                ALG_ML_DSA_65 => ObjectIdentifier::new_unwrap(ID_ALG_ML_DSA_65),
                ALG_ML_DSA_87 => ObjectIdentifier::new_unwrap(ID_ALG_ML_DSA_87),
                _ => {
                    return Err(Error::NotSupported(Some(format!(
                        "{} is not an ML-DSA algorithm",
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

            // Step 2.7. Let publicKey be the ML-DSA public key identified by the subjectPublicKey
            // field of spki.
            let key_bytes = spki.subject_public_key.as_bytes().ok_or(Error::Data(Some(
                "Failed to parse byte sequence over SubjectPublicKey field of spki".to_string(),
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
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                public_key,
                can_gc,
            )
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 2.1. If usages contains a value which is not "sign" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"sign\"".to_string(),
                )));
            }

            // Step 2.2. Let privateKeyInfo be the result of running the parse a privateKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurs while parsing, then throw a DataError.
            let private_key_info = PrivateKeyInfo::from_der(key_data).map_err(|_| {
                Error::Data(Some(
                    "Failed to parse PrivateKeyInfo over keyData".to_string(),
                ))
            })?;

            // Step 2.4.
            // If the name member of normalizedAlgorithm is "ML-DSA-44":
            //     Let expectedOid be id-ml-dsa-44 (2.16.840.1.101.3.4.3.17).
            //     Let asn1Structure be the ASN.1 ML-DSA-44-PrivateKey structure.
            // If the name member of normalizedAlgorithm is "ML-DSA-65":
            //     Let expectedOid be id-ml-dsa-65 (2.16.840.1.101.3.4.3.18).
            //     Let asn1Structure be the ASN.1 ML-DSA-65-PrivateKey structure.
            // If the name member of normalizedAlgorithm is "ML-DSA-87":
            //     Let expectedOid be id-ml-dsa-87 (2.16.840.1.101.3.4.3.19).
            //     Let asn1Structure be the ASN.1 ML-DSA-87-PrivateKey structure.
            // Otherwise:
            //     throw a NotSupportedError.
            let expected_oid = match normalized_algorithm.name.as_str() {
                ALG_ML_DSA_44 => ObjectIdentifier::new_unwrap(ID_ALG_ML_DSA_44),
                ALG_ML_DSA_65 => ObjectIdentifier::new_unwrap(ID_ALG_ML_DSA_65),
                ALG_ML_DSA_87 => ObjectIdentifier::new_unwrap(ID_ALG_ML_DSA_87),
                _ => {
                    return Err(Error::NotSupported(Some(format!(
                        "{} is not an ML-DSA algorithm",
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

            // Step 2.7. Let mlDsaPrivateKey be the result of performing the parse an ASN.1
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
                MlDsaPrivateKeyStructure::from_der(private_key_info.private_key).map_err(|_| {
                    Error::Data(Some(
                        "Failed to parse privateKey field of PrivateKeyInfo".to_string(),
                    ))
                })?;
            let ml_dsa_private_key = match private_key_structure {
                MlDsaPrivateKeyStructure::Seed(seed) => {
                    let (private_key_handle, _public_key_handle) = convert_seed_to_handles(
                        &normalized_algorithm.name,
                        seed.as_bytes(),
                        None,
                        None,
                    )?;
                    private_key_handle
                },
                MlDsaPrivateKeyStructure::ExpandedKey(_) => {
                    return Err(Error::NotSupported(Some(
                        "Not support \"expandedKey\" format of ASN.1 ML-DSA private key structures"
                            .to_string(),
                    )));
                },
                MlDsaPrivateKeyStructure::Both(both) => {
                    let (private_key_handle, _public_key_handle) = convert_seed_to_handles(
                        &normalized_algorithm.name,
                        both.seed.as_bytes(),
                        Some(both.expanded_key.as_bytes()),
                        None,
                    )?;
                    private_key_handle
                },
            };

            // Step 2.9. Let key be a new CryptoKey that represents the ML-DSA private key
            // identified by mlDsaPrivateKey.
            // Step 2.10. Set the [[type]] internal slot of key to "private"
            // Step 2.11. Let algorithm be a new KeyAlgorithm.
            // Step 2.12. Set the name attribute of algorithm to the name attribute of
            // normalizedAlgorithm.
            // Step 2.13. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name.clone(),
            };
            CryptoKey::new(
                global,
                KeyType::Private,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                ml_dsa_private_key,
                can_gc,
            )
        },
        // If format is "raw-public":
        KeyFormat::Raw_public => {
            // Step 2.1. If usages contains a value which is not "verify" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"verify\"".to_string(),
                )));
            }

            // Step 2.2. Let algorithm be a new KeyAlgorithm object.
            // Step 2.3. Set the name attribute of algorithm to the name attribute of
            // normalizedAlgorithm.
            // Step 2.4. Let key be a new CryptoKey representing the key data provided in keyData.
            // Step 2.5. Set the [[type]] internal slot of key to "public"
            // Step 2.6. Set the [[algorithm]] internal slot of key to algorithm.
            let public_key_handle =
                convert_public_key_to_handle(&normalized_algorithm.name, key_data)?;
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name.clone(),
            };
            CryptoKey::new(
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                public_key_handle,
                can_gc,
            )
        },
        // If format is "raw-seed":
        KeyFormat::Raw_seed => {
            // Step 2.1. If usages contains an entry which is not "sign" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"sign\"".to_string(),
                )));
            }

            // Step 2.2. Let data be keyData.
            let data = key_data;

            // Step 2.3. If the length in bits of data is not 256 then throw a DataError.
            // Step 2.4. Let privateKey be the result of performing the ML-DSA.KeyGen_internal
            // function described in Section 6.1 of [FIPS-204] with the parameter set indicated by
            // the name member of normalizedAlgorithm, using data as ξ.
            let (private_key_handle, _public_key_handle) =
                convert_seed_to_handles(&normalized_algorithm.name, data, None, None)?;

            // Step 2.5. Let key be a new CryptoKey that represents the ML-DSA private key
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
                global,
                KeyType::Private,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                private_key_handle,
                can_gc,
            )
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1.
            // If keyData is a JsonWebKey dictionary:
            //     Let jwk equal keyData.
            // Otherwise:
            //     Throw a DataError.
            let jwk = JsonWebKey::parse(GlobalScope::get_cx(), key_data)?;

            // Step 2.2. If the priv field is present and usages contains a value which is not
            // "sign", or, if the priv field is not present and usages contains a value which is
            // not "verify" then throw a SyntaxError.
            if jwk.priv_.is_some() && usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                return Err(Error::Syntax(Some(
                    "The priv field is present and usages contains a value which is not \"sign\""
                        .to_string(),
                )));
            }
            if jwk.priv_.is_none() && usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(Some(
                    "The priv field is not present and usages contains a value which is not \
                        \"verify\""
                        .to_string(),
                )));
            }

            // Step 2.3. If the kty field of jwk is not "AKP", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "AKP") {
                return Err(Error::Data(Some(
                    "The kty field of jwk is not \"AKP\"".to_string(),
                )));
            }

            // Step 2.4. If the alg field of jwk is not equal to the name member of
            // normalizedAlgorithm, then throw a DataError.
            if jwk
                .alg
                .as_ref()
                .is_none_or(|alg| alg != normalized_algorithm.name.as_str())
            {
                return Err(Error::Data(Some(
                    "The alg field of jwk is not equal to the name member of normalizedAlgorithm"
                        .to_string(),
                )));
            }

            // Step 2.5. If usages is non-empty and the use field of jwk is present and is not
            // equal to "sig", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                return Err(Error::Data(Some(
                    "Usages is non-empty and the use field of jwk is present and is not \
                    equal to \"sig\""
                        .to_string(),
                )));
            }

            // Step 2.6. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK], or it does not contain all of the specified
            // usages values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.7. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data(Some(
                    "The ext field of jwk is present and has the value false and extractable \
                    is true"
                        .to_string(),
                )));
            }

            // Step 2.8.
            // If the priv field of jwk is present:
            if jwk.priv_.is_some() {
                // Step 2.8.1. If the priv attribute of jwk does not contain a valid base64url
                // encoded seed representing an ML-DSA private key, then throw a DataError.
                let priv_bytes = jwk.decode_required_string_field(JwkStringField::Priv)?;

                // Step 2.8.2. Let key be a new CryptoKey object that represents the ML-DSA private
                // key identified by interpreting the priv attribute of jwk as a base64url encoded
                // seed.
                // Step 2.8.3. Set the [[type]] internal slot of key to "private".
                // Step 2.8.4. If the pub attribute of jwk does not contain the base64url encoded
                // public key representing the ML-DSA public key corresponding to key, then throw a
                // DataError.
                let pub_bytes = jwk.decode_required_string_field(JwkStringField::Pub)?;
                let (private_key_handle, _public_key_handle) = convert_seed_to_handles(
                    &normalized_algorithm.name,
                    &priv_bytes,
                    None,
                    Some(&pub_bytes),
                )?;
                let algorithm = SubtleKeyAlgorithm {
                    name: normalized_algorithm.name.clone(),
                };
                CryptoKey::new(
                    global,
                    KeyType::Private,
                    extractable,
                    KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                    usages,
                    private_key_handle,
                    can_gc,
                )
            }
            // Otherwise:
            else {
                // Step 2.8.1. If the pub attribute of jwk does not contain a valid base64url
                // encoded public key representing an ML-DSA public key, then throw a DataError.
                let pub_bytes = jwk.decode_required_string_field(JwkStringField::Pub)?;

                // Step 2.8.2. Let key be a new CryptoKey object that represents the ML-DSA public
                // key identified by interpreting the pub attribute of jwk as a base64url encoded
                // public key.
                // Step 2.8.3. Set the [[type]] internal slot of key to "public".
                let public_key_handle =
                    convert_public_key_to_handle(&normalized_algorithm.name, &pub_bytes)?;
                let algorithm = SubtleKeyAlgorithm {
                    name: normalized_algorithm.name.clone(),
                };
                CryptoKey::new(
                    global,
                    KeyType::Public,
                    extractable,
                    KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                    usages,
                    public_key_handle,
                    can_gc,
                )
            }
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported import key format for ML-DSA key".to_string(),
            )));
        },
    };

    // Step 3. Return key.
    Ok(key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-dsa-operations-export-key>
///
/// The exportKey() method does not involve AlgorithmIdentifier and algorithm normalization, so
/// there should not be normalizedAlgorithm in the export key operation. It could be a mistake in
/// the specification (Related issue: <https://github.com/WICG/webcrypto-modern-algos/issues/47>).
///
/// In our implementation, we use the name attribute of the [[algorithhm]] internal slot of key to
/// determine the security category.
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    // Step 1. Let key be the CryptoKey to be exported.

    // Step 2. If the underlying cryptographic key material represented by the [[handle]] internal
    // slot of key cannot be accessed, then throw an OperationError.

    // Step 3.
    let result = match format {
        KeyFormat::Spki => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"public\"".to_string(),
                )));
            }

            // Step 3.2.
            // Let data be an instance of the SubjectPublicKeyInfo ASN.1 structure defined in
            // [RFC5280] with the following properties:
            //
            //     Set the algorithm field to an AlgorithmIdentifier ASN.1 type with the following
            //     properties:
            //
            //         If the name member of normalizedAlgorithm is "ML-DSA-44":
            //             Set the algorithm object identifier to the id-ml-dsa-44
            //             (2.16.840.1.101.3.4.3.17) OID.
            //
            //         If the name member of normalizedAlgorithm is "ML-DSA-65":
            //             Set the algorithm object identifier to the id-ml-dsa-65
            //             (2.16.840.1.101.3.4.3.18) OID.
            //
            //         If the name member of normalizedAlgorithm is "ML-DSA-87":
            //             Set the algorithm object identifier to the id-ml-dsa-87
            //             (2.16.840.1.101.3.4.3.19) OID.
            //
            //         Otherwise:
            //             throw a NotSupportedError.
            //
            //     Set the subjectPublicKey field to keyData.
            let KeyAlgorithmAndDerivatives::KeyAlgorithm(key_algorithm) = key.algorithm() else {
                return Err(Error::Operation(Some(
                    "[[algorithm]] internal slot of key is not a KeyAlgorithm".to_string(),
                )));
            };
            let oid = match key_algorithm.name.as_str() {
                ALG_ML_DSA_44 => ID_ALG_ML_DSA_44,
                ALG_ML_DSA_65 => ID_ALG_ML_DSA_65,
                ALG_ML_DSA_87 => ID_ALG_ML_DSA_87,
                _ => {
                    return Err(Error::Operation(Some(format!(
                        "{} is not an ML-DSA algorithm",
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

            // Step 3.3. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(data.to_der().map_err(|_| {
                Error::Operation(Some(
                    "Failed to encode SubjectPublicKeyInfo in DER format".to_string(),
                ))
            })?)
        },
        KeyFormat::Pkcs8 => {
            // Step 3.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"private\"".to_string(),
                )));
            }

            // Step 3.2.
            // Let data be an instance of the PrivateKeyInfo ASN.1 structure defined in [RFC5208]
            // with the following properties:
            //
            //     Set the version field to 0.
            //
            //     Set the privateKeyAlgorithm field to a PrivateKeyAlgorithmIdentifier ASN.1 type
            //     with the following properties:
            //
            //         If the name member of normalizedAlgorithm is "ML-DSA-44":
            //             Set the algorithm object identifier to the id-ml-dsa-44
            //             (2.16.840.1.101.3.4.3.17) OID.
            //
            //         If the name member of normalizedAlgorithm is "ML-DSA-65":
            //             Set the algorithm object identifier to the id-ml-dsa-65
            //             (2.16.840.1.101.3.4.3.18) OID.
            //
            //         If the name member of normalizedAlgorithm is "ML-DSA-87":
            //             Set the algorithm object identifier to the id-ml-dsa-87
            //             (2.16.840.1.101.3.4.3.19) OID.
            //
            //         Otherwise:
            //             throw a NotSupportedError.
            //
            //     Set the privateKey field as follows:
            //
            //         If the name member of normalizedAlgorithm is "ML-DSA-44":
            //             Set the privateKey field to the result of DER-encoding a
            //             ML-DSA-44-PrivateKey ASN.1 type that represents the ML-DSA private key
            //             seed represented by the [[handle]] internal slot of key using the
            //             seed-only format (using a context-specific [0] primitive tag with an
            //             implicit encoding of OCTET STRING).
            //
            //         If the name member of normalizedAlgorithm is "ML-DSA-65":
            //             Set the privateKey field to the result of DER-encoding a
            //             ML-DSA-65-PrivateKey ASN.1 type that represents the ML-DSA private key
            //             seed represented by the [[handle]] internal slot of key using the
            //             seed-only format (using a context-specific [0] primitive tag with an
            //             implicit encoding of OCTET STRING).
            //
            //         If the name member of normalizedAlgorithm is "ML-DSA-87":
            //             Set the privateKey field to the result of DER-encoding a
            //             ML-DSA-87-PrivateKey ASN.1 type that represents the ML-DSA private key
            //             seed represented by the [[handle]] internal slot of key using the
            //             seed-only format (using a context-specific [0] primitive tag with an
            //             implicit encoding of OCTET STRING).
            //
            //         Otherwise:
            //             throw a NotSupportedError.
            let KeyAlgorithmAndDerivatives::KeyAlgorithm(key_algorithm) = key.algorithm() else {
                return Err(Error::Operation(Some(
                    "[[algorithm]] internal slot of key is not a KeyAlgorithm".to_string(),
                )));
            };
            let oid = match key_algorithm.name.as_str() {
                ALG_ML_DSA_44 => ID_ALG_ML_DSA_44,
                ALG_ML_DSA_65 => ID_ALG_ML_DSA_65,
                ALG_ML_DSA_87 => ID_ALG_ML_DSA_87,
                _ => {
                    return Err(Error::Operation(Some(format!(
                        "{} is not an ML-DSA algorithm",
                        key_algorithm.name.as_str()
                    ))));
                },
            };
            let (seed_bytes, _public_key_bytes) =
                convert_handle_to_seed_and_public_key(key.handle())?;
            let private_key =
                MlDsaPrivateKeyStructure::Seed(OctetString::new(seed_bytes).map_err(|_| {
                    Error::Operation(Some(
                        "Failed to encode OctetString for privateKey field of \
                        ASN.1 ML-DSA private key structure"
                            .to_string(),
                    ))
                })?);
            let encoded_private_key = private_key.to_der().map_err(|_| {
                Error::Operation(Some(
                    "Failed to encode ASN.1 ML-DSA private key structure in DER format".to_string(),
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

            // Step 3.3. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(private_key_info.to_der().map_err(|_| {
                Error::Operation(Some(
                    "Failed to encode PrivateKeyInfo in DER format".to_string(),
                ))
            })?)
        },
        // If format is "raw-public":
        KeyFormat::Raw_public => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"public\"".to_string(),
                )));
            }

            // Step 3.2. Let data be a byte sequence containing the ML-DSA public key represented
            // by the [[handle]] internal slot of key.
            let data = convert_handle_to_public_key(key.handle())?;

            // Step 3.2. Let result be data.
            ExportedKey::Bytes(data)
        },
        // If format is "raw-seed":
        KeyFormat::Raw_seed => {
            // Step 3.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"private\"".to_string(),
                )));
            }

            // Step 3.2. Let data be a byte sequence containing the ξ seed variable of the key
            // represented by the [[handle]] internal slot of key.
            let (data, _public_key_bytes) = convert_handle_to_seed_and_public_key(key.handle())?;

            // Step 3.3. Let result be data.
            ExportedKey::Bytes(data)
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 3.1. Let jwk be a new JsonWebKey dictionary.
            // Step 3.2. Set the kty attribute of jwk to "AKP".
            // Step 3.3. Set the alg attribute of jwk to the name member of normalizedAlgorithm.
            let KeyAlgorithmAndDerivatives::KeyAlgorithm(key_algorithm) = key.algorithm() else {
                return Err(Error::Operation(Some(
                    "[[algorithm]] internal slot of key is not a KeyAlgorithm".to_string(),
                )));
            };
            let mut jwk = JsonWebKey {
                kty: Some(DOMString::from("AKP")),
                alg: Some(DOMString::from(key_algorithm.name.as_str())),
                ..Default::default()
            };

            // Step 3.4. Set the pub attribute of jwk to the base64url encoded public key
            // corresponding to the [[handle]] internal slot of key.
            // Step 3.5
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

            // Step 3.6. Set the key_ops attribute of jwk to the usages attribute of key.
            jwk.set_key_ops(key.usages());

            // Step 3.7. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 3.8. Let result be jwk.
            ExportedKey::Jwk(Box::new(jwk))
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported export key format for ML-DSA key".to_string(),
            )));
        },
    };

    // Step 3. Return result.
    Ok(result)
}

/// Convert seed bytes to an ML-DSA private key handle and an ML-DSA public key handle. If private
/// key bytes and/or public key bytes are provided, it runs a consistency check against the seed.
/// If the length in bits of seed bytes is not 256, the conversion fails, or the consistency check
/// fails, throw a DataError.
fn convert_seed_to_handles(
    algo_name: &str,
    seed_bytes: &[u8],
    private_key_bytes: Option<&[u8]>,
    public_key_bytes: Option<&[u8]>,
) -> Result<(Handle, Handle), Error> {
    if seed_bytes.len() != 32 {
        return Err(Error::Data(Some(
            "The length in bits of seed bytes is not 256".to_string(),
        )));
    }

    let seed: B32 = seed_bytes
        .try_into()
        .map_err(|_| Error::Data(Some("Failed to parse the seed bytes".to_string())))?;
    let handles = match algo_name {
        ALG_ML_DSA_44 => {
            let key_pair = MlDsa44::key_gen_internal(&seed);
            if let Some(private_key_bytes) = private_key_bytes {
                if private_key_bytes != key_pair.signing_key().encode().as_slice() {
                    return Err(Error::Data(Some(
                        "The expanded private key does not match the seed".to_string(),
                    )));
                }
            }
            if let Some(public_key_bytes) = public_key_bytes {
                if public_key_bytes != key_pair.verifying_key().encode().as_slice() {
                    return Err(Error::Data(Some(
                        "The public key does not match the seed".to_string(),
                    )));
                }
            }

            (
                Handle::MlDsa44PrivateKey(seed),
                Handle::MlDsa44PublicKey(Box::new(key_pair.verifying_key().encode())),
            )
        },
        ALG_ML_DSA_65 => {
            let key_pair = MlDsa65::key_gen_internal(&seed);
            if let Some(private_key_bytes) = private_key_bytes {
                if private_key_bytes != key_pair.signing_key().encode().as_slice() {
                    return Err(Error::Data(Some(
                        "The expanded private key does not match the seed".to_string(),
                    )));
                }
            }
            if let Some(public_key_bytes) = public_key_bytes {
                if public_key_bytes != key_pair.verifying_key().encode().as_slice() {
                    return Err(Error::Data(Some(
                        "The public key does not match the seed".to_string(),
                    )));
                }
            }

            (
                Handle::MlDsa65PrivateKey(seed),
                Handle::MlDsa65PublicKey(Box::new(key_pair.verifying_key().encode())),
            )
        },
        ALG_ML_DSA_87 => {
            let key_pair = MlDsa87::key_gen_internal(&seed);
            if let Some(private_key_bytes) = private_key_bytes {
                if private_key_bytes != key_pair.signing_key().encode().as_slice() {
                    return Err(Error::Data(Some(
                        "The expanded private key does not match the seed".to_string(),
                    )));
                }
            }
            if let Some(public_key_bytes) = public_key_bytes {
                if public_key_bytes != key_pair.verifying_key().encode().as_slice() {
                    return Err(Error::Data(Some(
                        "The public key does not match the seed".to_string(),
                    )));
                }
            }

            (
                Handle::MlDsa87PrivateKey(seed),
                Handle::MlDsa87PublicKey(Box::new(key_pair.verifying_key().encode())),
            )
        },
        _ => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not an ML-DSA algorithm",
                algo_name
            ))));
        },
    };

    Ok(handles)
}

/// Convert public key bytes to an ML-DSA public key handle. If the conversion fails, throw a
/// DataError.
fn convert_public_key_to_handle(algo_name: &str, public_key_bytes: &[u8]) -> Result<Handle, Error> {
    let public_key_handle = match algo_name {
        ALG_ML_DSA_44 => {
            let encoded_verifying_key = EncodedVerifyingKey::<MlDsa44>::try_from(public_key_bytes)
                .map_err(|_| Error::Data(Some("Failed to parse ML-DSA public key".to_string())))?;
            Handle::MlDsa44PublicKey(Box::new(encoded_verifying_key))
        },
        ALG_ML_DSA_65 => {
            let encoded_verifying_key = EncodedVerifyingKey::<MlDsa65>::try_from(public_key_bytes)
                .map_err(|_| Error::Data(Some("Failed to parse ML-DSA public key".to_string())))?;
            Handle::MlDsa65PublicKey(Box::new(encoded_verifying_key))
        },
        ALG_ML_DSA_87 => {
            let encoded_verifying_key = EncodedVerifyingKey::<MlDsa87>::try_from(public_key_bytes)
                .map_err(|_| Error::Data(Some("Failed to parse ML-DSA public key".to_string())))?;
            Handle::MlDsa87PublicKey(Box::new(encoded_verifying_key))
        },
        _ => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not an ML-DSA algorithm",
                algo_name
            ))));
        },
    };

    Ok(public_key_handle)
}

/// Convert an ML-DSA private key handle to seed bytes and public key bytes. If the handle is not
/// representing a ML-DSA private key, throw an OperationError.
fn convert_handle_to_seed_and_public_key(handle: &Handle) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let result = match handle {
        Handle::MlDsa44PrivateKey(seed) => {
            let key_pair = MlDsa44::key_gen_internal(seed);
            (
                seed.to_vec(),
                key_pair.verifying_key().encode().as_slice().to_vec(),
            )
        },
        Handle::MlDsa65PrivateKey(seed) => {
            let key_pair = MlDsa65::key_gen_internal(seed);
            (
                seed.to_vec(),
                key_pair.verifying_key().encode().as_slice().to_vec(),
            )
        },
        Handle::MlDsa87PrivateKey(seed) => {
            let key_pair = MlDsa87::key_gen_internal(seed);
            (
                seed.to_vec(),
                key_pair.verifying_key().encode().as_slice().to_vec(),
            )
        },
        _ => {
            return Err(Error::Operation(Some(
                "The key handle is not representing an ML-DSA private key".to_string(),
            )));
        },
    };

    Ok(result)
}

/// Convert an ML-DSA public key handle to public key bytes. If the handle is not representing a
/// ML-DSA public key, throw an OperationError.
fn convert_handle_to_public_key(handle: &Handle) -> Result<Vec<u8>, Error> {
    let result = match handle {
        Handle::MlDsa44PublicKey(public_key) => public_key.to_vec(),
        Handle::MlDsa65PublicKey(public_key) => public_key.as_slice().to_vec(),
        Handle::MlDsa87PublicKey(public_key) => public_key.to_vec(),
        _ => {
            return Err(Error::Operation(Some(
                "The key handle is not representing an ML-DSA public key".to_string(),
            )));
        },
    };

    Ok(result)
}
