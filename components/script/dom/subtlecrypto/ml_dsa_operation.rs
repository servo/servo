/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use der::asn1::{BitString, OctetString};
use der::{AnyRef, Choice, Decode, Sequence};
use ml_dsa::{B32, EncodedVerifyingKey, KeyGen, MlDsa44, MlDsa65, MlDsa87};
use pkcs8::{ObjectIdentifier, PrivateKeyInfo, SubjectPublicKeyInfo};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{KeyType, KeyUsage};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{JsonWebKey, KeyFormat};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_ML_DSA_44, ALG_ML_DSA_65, ALG_ML_DSA_87, JsonWebKeyExt, JwkStringField,
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
