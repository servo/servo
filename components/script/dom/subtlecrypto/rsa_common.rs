/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::{Mul, Sub};

use base64ct::{Base64UrlUnpadded, Encoding};
use js::context::JSContext;
use num_bigint_dig::{BigInt, ModInverse, Sign};
use num_traits::One;
use pkcs8::der::asn1::BitString;
use pkcs8::der::{AnyRef, Decode};
use pkcs8::rand_core::OsRng;
use pkcs8::spki::EncodePublicKey;
use pkcs8::{EncodePrivateKey, PrivateKeyInfo, SubjectPublicKeyInfo};
use rsa::pkcs1::{self, DecodeRsaPrivateKey};
use rsa::traits::{PrivateKeyParts, PublicKeyParts};
use rsa::{BigUint, RsaPrivateKey, RsaPublicKey};
use sec1::der::Encode;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, CryptoKeyPair, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{
    AlgorithmIdentifier, JsonWebKey, KeyFormat, RsaOtherPrimesInfo,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_RSA_OAEP, ALG_RSA_PSS, ALG_RSASSA_PKCS1_V1_5, ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512,
    DigestOperation, ExportedKey, JsonWebKeyExt, JwkStringField, KeyAlgorithmAndDerivatives,
    NormalizedAlgorithm, SubtleRsaHashedImportParams, SubtleRsaHashedKeyAlgorithm,
    SubtleRsaHashedKeyGenParams, normalize_algorithm,
};

pub(crate) enum RsaAlgorithm {
    RsassaPkcs1v1_5,
    RsaPss,
    RsaOaep,
}

/// <https://w3c.github.io/webcrypto/#rsassa-pkcs1-operations-generate-key>
/// <https://w3c.github.io/webcrypto/#rsa-pss-operations-generate-key>
/// <https://w3c.github.io/webcrypto/#rsa-oaep-operations-generate-key>
pub(crate) fn generate_key(
    rsa_algorithm: RsaAlgorithm,
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleRsaHashedKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<CryptoKeyPair, Error> {
    match rsa_algorithm {
        RsaAlgorithm::RsassaPkcs1v1_5 | RsaAlgorithm::RsaPss => {
            // Step 1. If usages contains an entry which is not "sign" or "verify", then throw a
            // SyntaxError.
            if usages
                .iter()
                .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify))
            {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"sign\" or \"verify\"".to_string(),
                )));
            }
        },
        RsaAlgorithm::RsaOaep => {
            // Step 1. If usages contains an entry which is not "encrypt", "decrypt", "wrapKey" or
            // "unwrapKey", then throw a SyntaxError.
            if usages.iter().any(|usage| {
                !matches!(
                    usage,
                    KeyUsage::Encrypt | KeyUsage::Decrypt | KeyUsage::WrapKey | KeyUsage::UnwrapKey
                )
            }) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"encrypt\", \"decrypt\", \
                    \"wrapKey\" or \"unwrapKey\""
                        .to_string(),
                )));
            }
        },
    }

    // Step 2. Generate an RSA key pair, as defined in [RFC3447], with RSA modulus length equal to
    // the modulusLength attribute of normalizedAlgorithm and RSA public exponent equal to the
    // publicExponent attribute of normalizedAlgorithm.
    // Step 3. If generation of the key pair fails, then throw an OperationError.
    // NOTE: If the public exponent is even, it is invalid for RSA, and RsaPrivateKey::new_with_exp
    // should throw an error. However, RsaPrivateKey::new_with_exp would take a long period of time
    // to validate this case. So, we manually check it before running RsaPrivateKey::new_with_exp,
    // in order to throw error eariler.
    if normalized_algorithm
        .public_exponent
        .last()
        .is_none_or(|last_byte| last_byte % 2 == 0)
    {
        return Err(Error::Operation(Some(
            "The public expoenent is an even number".to_string(),
        )));
    }
    let mut rng = OsRng;
    let modulus_length = normalized_algorithm.modulus_length as usize;
    let public_exponent = BigUint::from_bytes_be(&normalized_algorithm.public_exponent);
    let private_key = RsaPrivateKey::new_with_exp(&mut rng, modulus_length, &public_exponent)
        .map_err(|_| Error::Operation(Some("RSA failed to generate private key".to_string())))?;
    let public_key = private_key.to_public_key();

    // Step 4. Let algorithm be a new RsaHashedKeyAlgorithm dictionary.
    // Step 6. Set the modulusLength attribute of algorithm to equal the modulusLength attribute of
    // normalizedAlgorithm.
    // Step 7. Set the publicExponent attribute of algorithm to equal the publicExponent attribute
    // of normalizedAlgorithm.
    // Step 8. Set the hash attribute of algorithm to equal the hash member of normalizedAlgorithm.
    let algorithm = SubtleRsaHashedKeyAlgorithm {
        name: match rsa_algorithm {
            // Step 5. Set the name attribute of algorithm to "RSASSA-PKCS1-v1_5".
            RsaAlgorithm::RsassaPkcs1v1_5 => ALG_RSASSA_PKCS1_V1_5,
            // Step 5. Set the name attribute of algorithm to "RSA-PSS".
            RsaAlgorithm::RsaPss => ALG_RSA_PSS,
            // Step 5. Set the name attribute of algorithm to "RSA-OAEP".
            RsaAlgorithm::RsaOaep => ALG_RSA_OAEP,
        }
        .to_string(),
        modulus_length: normalized_algorithm.modulus_length,
        public_exponent: normalized_algorithm.public_exponent.clone(),
        hash: normalized_algorithm.hash.clone(),
    };

    // Step 9. Let publicKey be a new CryptoKey representing the public key of the generated key
    // pair.
    // Step 10. Set the [[type]] internal slot of publicKey to "public"
    // Step 11. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 12. Set the [[extractable]] internal slot of publicKey to true.
    let intersected_usages = match rsa_algorithm {
        RsaAlgorithm::RsassaPkcs1v1_5 | RsaAlgorithm::RsaPss => {
            // Step 13. Set the [[usages]] internal slot of publicKey to be the usage intersection
            // of usages and [ "verify" ].
            usages
                .iter()
                .filter(|usage| **usage == KeyUsage::Verify)
                .cloned()
                .collect()
        },
        RsaAlgorithm::RsaOaep => {
            // Step 13. Set the [[usages]] internal slot of publicKey to be the usage intersection
            // of usages and [ "encrypt", "wrapKey" ].
            usages
                .iter()
                .filter(|usage| matches!(usage, KeyUsage::Encrypt | KeyUsage::WrapKey))
                .cloned()
                .collect()
        },
    };
    let public_key = CryptoKey::new(
        cx,
        global,
        KeyType::Public,
        true,
        KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algorithm.clone()),
        intersected_usages,
        Handle::RsaPublicKey(public_key),
    );

    // Step 14. Let privateKey be a new CryptoKey representing the private key of the generated key
    // pair.
    // Step 15. Set the [[type]] internal slot of privateKey to "private"
    // Step 16. Set the [[algorithm]] internal slot of privateKey to algorithm.
    // Step 17. Set the [[extractable]] internal slot of privateKey to extractable.
    let intersected_usages = match rsa_algorithm {
        RsaAlgorithm::RsassaPkcs1v1_5 | RsaAlgorithm::RsaPss => {
            // Step 18. Set the [[usages]] internal slot of privateKey to be the usage intersection
            // of usages and [ "sign" ].
            usages
                .iter()
                .filter(|usage| **usage == KeyUsage::Sign)
                .cloned()
                .collect()
        },
        RsaAlgorithm::RsaOaep => {
            // Step 18. Set the [[usages]] internal slot of privateKey to be the usage intersection
            // of usages and [ "decrypt", "unwrapKey" ].
            usages
                .iter()
                .filter(|usage| matches!(usage, KeyUsage::Decrypt | KeyUsage::UnwrapKey))
                .cloned()
                .collect()
        },
    };
    let private_key = CryptoKey::new(
        cx,
        global,
        KeyType::Private,
        extractable,
        KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algorithm),
        intersected_usages,
        Handle::RsaPrivateKey(private_key),
    );

    // Step 19. Let result be a new CryptoKeyPair dictionary.
    // Step 20. Set the publicKey attribute of result to be publicKey.
    // Step 21. Set the privateKey attribute of result to be privateKey.
    let result = CryptoKeyPair {
        publicKey: Some(public_key),
        privateKey: Some(private_key),
    };

    // Step 22. Return result.
    Ok(result)
}

/// <https://w3c.github.io/webcrypto/#rsassa-pkcs1-operations-import-key>
/// <https://w3c.github.io/webcrypto/#rsa-pss-operations-import-key>
/// <https://w3c.github.io/webcrypto/#rsa-oaep-operations-import-key>
///
/// This implementation is based on the specification for RSA-PSS.
/// When format is "jwk", Step 2.7 in the specification for RSASSA-PKCS1-v1_5 is skipped since it is redundent.
/// When format is "jwk", Step 2.2 and 2.3 in the specification of RSA-OAEP are combined into a single step.
#[allow(clippy::too_many_arguments)]
pub(crate) fn import_key(
    rsa_algorithm: RsaAlgorithm,
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleRsaHashedImportParams,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2.
    let (key_handle, key_type) = match format {
        // If format is "spki":
        KeyFormat::Spki => {
            match &rsa_algorithm {
                RsaAlgorithm::RsassaPkcs1v1_5 | RsaAlgorithm::RsaPss => {
                    // Step 2.1. If usages contains an entry which is not "verify" then throw a
                    // SyntaxError.
                    if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                        return Err(Error::Syntax(Some(
                            "Usages contains an entry which is not \"verify\"".to_string(),
                        )));
                    }
                },
                RsaAlgorithm::RsaOaep => {
                    // Step 2.1. If usages contains an entry which is not "encrypt" or "wrapKey",
                    // then throw a SyntaxError.
                    if usages
                        .iter()
                        .any(|usage| !matches!(usage, KeyUsage::Encrypt | KeyUsage::WrapKey))
                    {
                        return Err(Error::Syntax(Some(
                            "Usages contains an entry which is not \"encrypt\" or \"wrapKey\""
                                .to_string(),
                        )));
                    }
                },
            }

            // Step 2.2. Let spki be the result of running the parse a subjectPublicKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurred while parsing, then throw a DataError.
            let spki =
                SubjectPublicKeyInfo::<AnyRef, BitString>::from_der(key_data).map_err(|_| {
                    Error::Data(Some(
                        "Fail to parse SubjectPublicKeyInfo over keyData".to_string(),
                    ))
                })?;

            // Step 2.4. If the algorithm object identifier field of the algorithm
            // AlgorithmIdentifier field of spki is not equal to the rsaEncryption object
            // identifier defined in [RFC3447], then throw a DataError.
            if spki.algorithm.oid != pkcs1::ALGORITHM_OID {
                return Err(Error::Data(Some(
                    "Algorithm object identifier of spki is not an rsaEncryption".to_string(),
                )));
            }

            // Step 2.5. Let publicKey be the result of performing the parse an ASN.1 structure
            // algorithm, with data as the subjectPublicKeyInfo field of spki, structure as the
            // RSAPublicKey structure specified in Section A.1.1 of [RFC3447], and exactData set to
            // true.
            // Step 2.6. If an error occurred while parsing, or it can be determined that publicKey
            // is not a valid public key according to [RFC3447], then throw a DataError.
            let pkcs1_bytes = spki.subject_public_key.as_bytes().ok_or(Error::Data(Some(
                "Fail to parse byte sequence over SubjectPublicKey field of spki".to_string(),
            )))?;
            let rsa_public_key_structure =
                pkcs1::RsaPublicKey::try_from(pkcs1_bytes).map_err(|_| {
                    Error::Data(Some(
                        "SubjectPublicKey field of spki is not an RSAPublicKey structure"
                            .to_string(),
                    ))
                })?;
            let n = BigUint::from_bytes_be(rsa_public_key_structure.modulus.as_bytes());
            let e = BigUint::from_bytes_be(rsa_public_key_structure.public_exponent.as_bytes());
            let public_key = RsaPublicKey::new(n, e).map_err(|_| {
                Error::Data(Some(
                    "Fail to construct RSA public key from modulus and public exponent".to_string(),
                ))
            })?;

            // Step 2.7. Let key be a new CryptoKey that represents the RSA public key identified
            // by publicKey.
            // Step 2.8. Set the [[type]] internal slot of key to "public"
            // NOTE: Done in Step 3-8.
            let key_handle = Handle::RsaPublicKey(public_key);
            let key_type = KeyType::Public;
            (key_handle, key_type)
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            match &rsa_algorithm {
                RsaAlgorithm::RsassaPkcs1v1_5 | RsaAlgorithm::RsaPss => {
                    // Step 2.1. If usages contains an entry which is not "sign" then throw a
                    // SyntaxError.
                    if usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                        return Err(Error::Syntax(Some(
                            "Usages contains an entry which is not \"sign\"".to_string(),
                        )));
                    }
                },
                RsaAlgorithm::RsaOaep => {
                    // Step 2.1. If usages contains an entry which is not "decrypt" or "unwrapKey",
                    // then throw a SyntaxError.
                    if usages
                        .iter()
                        .any(|usage| !matches!(usage, KeyUsage::Decrypt | KeyUsage::UnwrapKey))
                    {
                        return Err(Error::Syntax(Some(
                            "Usages contains an entry which is not \"decrypt\" or \"unwrapKey\""
                                .to_string(),
                        )));
                    }
                },
            }

            // Step 2.2. Let privateKeyInfo be the result of running the parse a privateKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurred while parsing, then throw a DataError.
            let private_key_info = PrivateKeyInfo::from_der(key_data).map_err(|_| {
                Error::Data(Some(
                    "Fail to parse PrivateKeyInfo over keyData".to_string(),
                ))
            })?;

            // Step 2.4. If the algorithm object identifier field of the privateKeyAlgorithm
            // PrivateKeyAlgorithm field of privateKeyInfo is not equal to the rsaEncryption object
            // identifier defined in [RFC3447], then throw a DataError.
            if private_key_info.algorithm.oid != pkcs1::ALGORITHM_OID {
                return Err(Error::Data(Some(
                    "Algorithm object identifier of PrivateKeyInfo is not an rsaEncryption"
                        .to_string(),
                )));
            }

            // Step 2.5. Let rsaPrivateKey be the result of performing the parse an ASN.1 structure
            // algorithm, with data as the privateKey field of privateKeyInfo, structure as the
            // RSAPrivateKey structure specified in Section A.1.2 of [RFC3447], and exactData set
            // to true.
            // Step 2.6. If an error occurred while parsing, or if rsaPrivateKey is not a valid RSA
            // private key according to [RFC3447], then throw a DataError.
            let rsa_private_key = RsaPrivateKey::from_pkcs1_der(private_key_info.private_key)
                .map_err(|_| {
                    Error::Data(Some(
                        "PrivateKey field of PrivateKeyInfo is not an RSAPrivateKey structure"
                            .to_string(),
                    ))
                })?;

            // Step 2.7. Let key be a new CryptoKey that represents the RSA private key identified
            // by rsaPrivateKey.
            // Step 2.8. Set the [[type]] internal slot of key to "private"
            // NOTE: Done in Step 3-8.
            let key_handle = Handle::RsaPrivateKey(rsa_private_key);
            let key_type = KeyType::Private;
            (key_handle, key_type)
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1.
            // If keyData is a JsonWebKey dictionary:
            //     Let jwk equal keyData.
            // Otherwise:
            //     Throw a DataError.
            let jwk = JsonWebKey::parse(cx, key_data)?;

            match &rsa_algorithm {
                RsaAlgorithm::RsassaPkcs1v1_5 | RsaAlgorithm::RsaPss => {
                    // Step 2.2. If the d field of jwk is present and usages contains an entry
                    // which is not "sign", or, if the d field of jwk is not present and usages
                    // contains an entry which is not "verify" then throw a SyntaxError.
                    if jwk.d.is_some() && usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                        return Err(Error::Syntax(Some(
                            "The d field of jwk is present and usages contains an entry which is \
                            not \"sign\""
                                .to_string(),
                        )));
                    }
                    if jwk.d.is_none() && usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                        return Err(Error::Syntax(Some(
                            "The d field of jwk is not present and usages contains an entry which \
                            is not \"verify\""
                                .to_string(),
                        )));
                    }
                },
                RsaAlgorithm::RsaOaep => {
                    // Step 2.2.
                    // * If the d field of jwk is present and usages contains an entry which is not
                    // "decrypt" or "unwrapKey", then throw a SyntaxError.
                    // * If the d field of jwk is not present and usages contains an entry which is
                    // not "encrypt" or "wrapKey", then throw a SyntaxError.
                    if jwk.d.is_some() &&
                        usages.iter().any(|usage| {
                            !matches!(usage, KeyUsage::Decrypt | KeyUsage::UnwrapKey)
                        })
                    {
                        return Err(Error::Syntax(Some(
                            "The d field of jwk is present and usages contains an entry which is \
                            not \"decrypt\" or \"unwrapKey\""
                                .to_string(),
                        )));
                    }
                    if jwk.d.is_none() &&
                        usages
                            .iter()
                            .any(|usage| !matches!(usage, KeyUsage::Encrypt | KeyUsage::WrapKey))
                    {
                        return Err(Error::Syntax(Some(
                            "The d field of jwk is not present and usages contains an entry which \
                            is not \"encrypt\" or \"wrapKey\""
                                .to_string(),
                        )));
                    }
                },
            }

            // Step 2.3. If the kty field of jwk is not a case-sensitive string match to "RSA",
            // then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "RSA") {
                return Err(Error::Data(Some(
                    "The kty field of jwk is not a case-sensitive string match to \"RSA\""
                        .to_string(),
                )));
            }

            match &rsa_algorithm {
                RsaAlgorithm::RsassaPkcs1v1_5 | RsaAlgorithm::RsaPss => {
                    // Step 2.4. If usages is non-empty and the use field of jwk is present and is
                    // not a case-sensitive string match to "sig", then throw a DataError.
                    if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "sig") {
                        return Err(Error::Data(Some(
                            "Usages is non-empty and the use field of jwk is present and \
                            is not a case-sensitive string match to \"sig\""
                                .to_string(),
                        )));
                    }
                },
                RsaAlgorithm::RsaOaep => {
                    // Step 2.4. If usages is non-empty and the use field of jwk is present and is
                    // not a case-sensitive string match to "enc", then throw a DataError.
                    if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                        return Err(Error::Data(Some(
                            "Usages is non-empty and the use field of jwk is present and \
                            is not a case-sensitive string match to \"enc\""
                                .to_string(),
                        )));
                    }
                },
            }

            // Step 2.5. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK] or does not contain all of the specified usages
            // values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.6. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data(Some(
                    "The ext field of jwk is present and \
                    has the value false and extractable is true"
                        .to_string(),
                )));
            }

            let hash = match &rsa_algorithm {
                RsaAlgorithm::RsassaPkcs1v1_5 => {
                    // Step 2.7.
                    // If the alg field of jwk is not present:
                    //     Let hash be undefined.
                    // If the alg field is equal to the string "RS1":
                    //     Let hash be the string "SHA-1".
                    // If the alg field is equal to the string "RS256":
                    //     Let hash be the string "SHA-256".
                    // If the alg field is equal to the string "RS384":
                    //     Let hash be the string "SHA-384".
                    // If the alg field is equal to the string "RS512":
                    //     Let hash be the string "SHA-512".
                    // Otherwise:
                    //     Perform any key import steps defined by other applicable specifications,
                    //     passing format, jwk and obtaining hash.
                    //     If an error occurred or there are no applicable specifications, throw a
                    //     DataError.
                    match &jwk.alg {
                        None => None,
                        Some(alg) => match &*alg.str() {
                            "RS1" => Some(ALG_SHA1),
                            "RS256" => Some(ALG_SHA256),
                            "RS384" => Some(ALG_SHA384),
                            "RS512" => Some(ALG_SHA512),
                            _ => None,
                        },
                    }
                },
                RsaAlgorithm::RsaPss => {
                    // Step 2.7.
                    // If the alg field of jwk is not present:
                    //     Let hash be undefined.
                    // If the alg field is equal to the string "PS1":
                    //     Let hash be the string "SHA-1".
                    // If the alg field is equal to the string "PS256":
                    //     Let hash be the string "SHA-256".
                    // If the alg field is equal to the string "PS384":
                    //     Let hash be the string "SHA-384".
                    // If the alg field is equal to the string "PS512":
                    //     Let hash be the string "SHA-512".
                    // Otherwise:
                    //     Perform any key import steps defined by other applicable specifications,
                    //     passing format, jwk and obtaining hash.
                    //     If an error occurred or there are no applicable specifications, throw a
                    //     DataError.
                    match &jwk.alg {
                        None => None,
                        Some(alg) => match &*alg.str() {
                            "PS1" => Some(ALG_SHA1),
                            "PS256" => Some(ALG_SHA256),
                            "PS384" => Some(ALG_SHA384),
                            "PS512" => Some(ALG_SHA512),
                            _ => None,
                        },
                    }
                },
                RsaAlgorithm::RsaOaep => {
                    // Step 2.7.
                    // If the alg field of jwk is not present:
                    //     Let hash be undefined.
                    // If the alg field of jwk is equal to "RSA-OAEP":
                    //     Let hash be the string "SHA-1".
                    // If the alg field of jwk is equal to "RSA-OAEP-256":
                    //     Let hash be the string "SHA-256".
                    // If the alg field of jwk is equal to "RSA-OAEP-384":
                    //     Let hash be the string "SHA-384".
                    // If the alg field of jwk is equal to "RSA-OAEP-512":
                    //     Let hash be the string "SHA-512".
                    // Otherwise:
                    //     Perform any key import steps defined by other applicable specifications,
                    //     passing format, jwk and obtaining hash.
                    //     If an error occurred or there are no applicable specifications, throw a
                    //     DataError.
                    match &jwk.alg {
                        None => None,
                        Some(alg) => match &*alg.str() {
                            "RSA-OAEP" => Some(ALG_SHA1),
                            "RSA-OAEP-256" => Some(ALG_SHA256),
                            "RSA-OAEP-384" => Some(ALG_SHA384),
                            "RSA-OAEP-512" => Some(ALG_SHA512),
                            _ => None,
                        },
                    }
                },
            };

            // Step 2.8. If hash is not undefined:
            if let Some(hash) = hash {
                // Step 2.8.1. Let normalizedHash be the result of normalize an algorithm with alg
                // set to hash and op set to digest.
                let normalized_hash = normalize_algorithm::<DigestOperation>(
                    cx,
                    &AlgorithmIdentifier::String(DOMString::from(hash)),
                )?;

                // Step 2.8.2. If normalizedHash is not equal to the hash member of
                // normalizedAlgorithm, throw a DataError.
                if normalized_hash.name() != normalized_algorithm.hash.name() {
                    return Err(Error::Data(Some(
                        "The normalizedHash is not equal to the hash member of normalizedAlgorithm"
                            .to_string(),
                    )));
                }
            }

            // Step 2.9.
            // If the d field of jwk is present:
            if jwk.d.is_some() {
                // Step 2.9.1. If jwk does not meet the requirements of Section 6.3.2 of JSON Web
                // Algorithms [JWA], then throw a DataError.
                let n = jwk.decode_required_string_field(JwkStringField::N)?;
                let e = jwk.decode_required_string_field(JwkStringField::E)?;
                let d = jwk.decode_required_string_field(JwkStringField::D)?;
                let p = jwk.decode_optional_string_field(JwkStringField::P)?;
                let q = jwk.decode_optional_string_field(JwkStringField::Q)?;
                let dp = jwk.decode_optional_string_field(JwkStringField::DP)?;
                let dq = jwk.decode_optional_string_field(JwkStringField::DQ)?;
                let qi = jwk.decode_optional_string_field(JwkStringField::QI)?;
                let mut primes = match (p, q, dp, dq, qi) {
                    (Some(p), Some(q), Some(_dp), Some(_dq), Some(_qi)) => vec![p, q],
                    (None, None, None, None, None) => Vec::new(),
                    _ => return Err(Error::Data(Some(
                        "The p, q, dp, dq, qi fields of jwk must be either all-present or all-absent"
                            .to_string()
                    ))),
                };
                jwk.decode_primes_from_oth_field(&mut primes)?;

                // Step 2.9.2. Let privateKey represents the RSA private key identified by
                // interpreting jwk according to Section 6.3.2 of JSON Web Algorithms [JWA].
                // Step 2.9.3. If privateKey is not a valid RSA private key according to [RFC3447],
                // then throw a DataError.
                let private_key = RsaPrivateKey::from_components(
                    BigUint::from_bytes_be(&n),
                    BigUint::from_bytes_be(&e),
                    BigUint::from_bytes_be(&d),
                    primes
                        .into_iter()
                        .map(|prime| BigUint::from_bytes_be(&prime))
                        .collect(),
                )
                .map_err(|_| {
                    Error::Data(Some(
                        "Failed to construct RSA private key from values in jwk".to_string(),
                    ))
                })?;

                // Step 2.9.4. Let key be a new CryptoKey object that represents privateKey.
                // Step 2.9.5. Set the [[type]] internal slot of key to "private"
                // NOTE: Done in Step 3-8.
                let key_handle = Handle::RsaPrivateKey(private_key);
                let key_type = KeyType::Private;
                (key_handle, key_type)
            }
            // Otherwise:
            else {
                // Step 2.9.1. If jwk does not meet the requirements of Section 6.3.1 of JSON Web
                // Algorithms [JWA], then throw a DataError.
                let n = jwk.decode_required_string_field(JwkStringField::N)?;
                let e = jwk.decode_required_string_field(JwkStringField::E)?;

                // Step 2.9.2. Let publicKey represent the RSA public key identified by
                // interpreting jwk according to Section 6.3.1 of JSON Web Algorithms [JWA].
                // Step 2.9.3. If publicKey can be determined to not be a valid RSA public key
                // according to [RFC3447], then throw a DataError.
                let public_key =
                    RsaPublicKey::new(BigUint::from_bytes_be(&n), BigUint::from_bytes_be(&e))
                        .map_err(|_| {
                            Error::Data(Some(
                                "Failed to construct RSA public key from values in jwk".to_string(),
                            ))
                        })?;

                // Step 2.9.4. Let key be a new CryptoKey representing publicKey.
                // Step 2.9.5. Set the [[type]] internal slot of key to "public"
                // NOTE: Done in Step 3-8.
                let key_handle = Handle::RsaPublicKey(public_key);
                let key_type = KeyType::Public;
                (key_handle, key_type)
            }
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported import key format for RSA key".to_string(),
            )));
        },
    };

    // Step 3. Let algorithm be a new RsaHashedKeyAlgorithm dictionary.
    // Step 5. Set the modulusLength attribute of algorithm to the length, in bits, of the RSA
    // public modulus.
    // Step 6. Set the publicExponent attribute of algorithm to the BigInteger representation of
    // the RSA public exponent.
    // Step 7. Set the hash attribute of algorithm to the hash member of normalizedAlgorithm.
    // Step 8. Set the [[algorithm]] internal slot of key to algorithm
    let (modulus_length, public_exponent) = match &key_handle {
        Handle::RsaPrivateKey(private_key) => {
            (private_key.size() as u32 * 8, private_key.e().to_bytes_be())
        },
        Handle::RsaPublicKey(public_key) => {
            (public_key.size() as u32 * 8, public_key.e().to_bytes_be())
        },
        _ => unreachable!(),
    };
    let algorithm = SubtleRsaHashedKeyAlgorithm {
        name: match &rsa_algorithm {
            RsaAlgorithm::RsassaPkcs1v1_5 => {
                // Step 4. Set the name attribute of algorithm to "RSASSA-PKCS1-v1_5"
                ALG_RSASSA_PKCS1_V1_5.to_string()
            },
            RsaAlgorithm::RsaPss => {
                // Step 4. Set the name attribute of algorithm to "RSA-PSS"
                ALG_RSA_PSS.to_string()
            },
            RsaAlgorithm::RsaOaep => {
                // Step 4. Set the name attribute of algorithm to "RSA-OAEP"
                ALG_RSA_OAEP.to_string()
            },
        },
        modulus_length,
        public_exponent,
        hash: normalized_algorithm.hash.clone(),
    };
    let key = CryptoKey::new(
        cx,
        global,
        key_type,
        extractable,
        KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algorithm),
        usages,
        key_handle,
    );

    // Step 9. Return key.
    Ok(key)
}

/// <https://w3c.github.io/webcrypto/#rsassa-pkcs1-operations-export-key>
/// <https://w3c.github.io/webcrypto/#rsa-pss-operations-export-key>
/// <https://w3c.github.io/webcrypto/#rsa-oaep-operations-export-key>
pub(crate) fn export_key(
    rsa_algorithm: RsaAlgorithm,
    format: KeyFormat,
    key: &CryptoKey,
) -> Result<ExportedKey, Error> {
    // Step 1. Let key be the key to be exported.

    // Step 2. If the underlying cryptographic key material represented by the [[handle]] internal
    // slot of key cannot be accessed, then throw an OperationError.
    // NOTE: Done in Step 3.

    // Step 3.
    let result = match format {
        // If format is "spki"
        KeyFormat::Spki => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(Some(
                    "The [[type]] internal slot of key is not \"public\"".to_string(),
                )));
            }

            // Step 3.2.
            // Let data be an instance of the SubjectPublicKeyInfo ASN.1 structure defined in
            // [RFC5280] with the following properties:
            //
            //     Set the algorithm field to an AlgorithmIdentifier ASN.1 type with the following
            //     properties:
            //
            //         Set the algorithm field to the OID rsaEncryption defined in [RFC3447].
            //
            //         Set the params field to the ASN.1 type NULL.
            //
            //     Set the subjectPublicKey field to the result of DER-encoding an RSAPublicKey
            //     ASN.1 type, as defined in [RFC3447], Appendix A.1.1, that represents the RSA
            //     public key represented by the [[handle]] internal slot of key
            let Handle::RsaPublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The [[handle]] internal slot of key is not an RSA public key".to_string(),
                )));
            };
            let data = public_key.to_public_key_der().map_err(|_| {
                Error::Operation(Some(
                    "Failed to convert RSA public key to SubjectPublicKeyInfo".to_string(),
                ))
            })?;

            // Step 3.3. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(data.to_der().map_err(|_| {
                Error::Operation(Some(
                    "Failed to convert SubjectPublicKeyInfo to DER-encodeing data".to_string(),
                ))
            })?)
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 3.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(Some(
                    "The [[type]] internal slot of key is not \"private\"".to_string(),
                )));
            }

            // Step 3.2.
            // Let data be an instance of the PrivateKeyInfo ASN.1 structure defined in [RFC5208]
            // with the following properties:
            //
            //    Set the version field to 0.
            //
            //    Set the privateKeyAlgorithm field to a PrivateKeyAlgorithmIdentifier ASN.1 type
            //    with the following properties:
            //
            //        Set the algorithm field to the OID rsaEncryption defined in [RFC3447].
            //
            //        Set the params field to the ASN.1 type NULL.
            //
            //    Set the privateKey field to the result of DER-encoding an RSAPrivateKey ASN.1
            //    type, as defined in [RFC3447], Appendix A.1.2, that represents the RSA private
            //    key represented by the [[handle]] internal slot of key
            let Handle::RsaPrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The [[handle]] internal slot of key is not an RSA private key".to_string(),
                )));
            };
            let data = private_key.to_pkcs8_der().map_err(|_| {
                Error::Operation(Some(
                    "Failed to convert RSA private key to PrivateKeyInfo".to_string(),
                ))
            })?;

            // Step 3.3. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(data.to_bytes().to_vec())
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 3.1. Let jwk be a new JsonWebKey dictionary.
            // Step 3.2. Set the kty attribute of jwk to the string "RSA".
            let mut jwk = JsonWebKey {
                kty: Some(DOMString::from("RSA")),
                ..Default::default()
            };

            // Step 3.3. Let hash be the name attribute of the hash attribute of the [[algorithm]]
            // internal slot of key.
            let KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algorithm) = key.algorithm()
            else {
                return Err(Error::Operation(Some(
                    "The [[algorithm]] internal slot of key is not an RsaHashedKeyAlgorithm"
                        .to_string(),
                )));
            };
            let hash = algorithm.hash.name();

            match rsa_algorithm {
                RsaAlgorithm::RsassaPkcs1v1_5 => {
                    // Step 3.4.
                    // If hash is "SHA-1":
                    //     Set the alg attribute of jwk to the string "RS1".
                    // If hash is "SHA-256":
                    //     Set the alg attribute of jwk to the string "RS256".
                    // If hash is "SHA-384":
                    //     Set the alg attribute of jwk to the string "RS384".
                    // If hash is "SHA-512":
                    //     Set the alg attribute of jwk to the string "RS512".
                    // Otherwise:
                    //     Perform any key export steps defined by other applicable specifications,
                    //     passing format and the hash attribute of the [[algorithm]] internal slot
                    //     of key and obtaining alg.
                    //     Set the alg attribute of jwk to alg.
                    let alg = match hash {
                        ALG_SHA1 => "RS1",
                        ALG_SHA256 => "RS256",
                        ALG_SHA384 => "RS384",
                        ALG_SHA512 => "RS512",
                        _ => {
                            return Err(Error::NotSupported(Some(format!(
                                "Unsupported \"{}\" hash for RSASSA-PKCS1-v1_5",
                                hash
                            ))));
                        },
                    };
                    jwk.alg = Some(DOMString::from(alg));
                },
                RsaAlgorithm::RsaPss => {
                    // Step 3.4.
                    // If hash is "SHA-1":
                    //     Set the alg attribute of jwk to the string "PS1".
                    // If hash is "SHA-256":
                    //     Set the alg attribute of jwk to the string "PS256".
                    // If hash is "SHA-384":
                    //     Set the alg attribute of jwk to the string "PS384".
                    // If hash is "SHA-512":
                    //     Set the alg attribute of jwk to the string "PS512".
                    // Otherwise:
                    //     Perform any key export steps defined by other applicable specifications,
                    //     passing format and the hash attribute of the [[algorithm]] internal slot
                    //     of key and obtaining alg.
                    //     Set the alg attribute of jwk to alg.
                    let alg = match hash {
                        ALG_SHA1 => "PS1",
                        ALG_SHA256 => "PS256",
                        ALG_SHA384 => "PS384",
                        ALG_SHA512 => "PS512",
                        _ => {
                            return Err(Error::NotSupported(Some(format!(
                                "Unsupported \"{}\" hash for RSA-PSS",
                                hash
                            ))));
                        },
                    };
                    jwk.alg = Some(DOMString::from(alg));
                },
                RsaAlgorithm::RsaOaep => {
                    // Step 3.4.
                    // If hash is "SHA-1":
                    //     Set the alg attribute of jwk to the string "RSA-OAEP".
                    // If hash is "SHA-256":
                    //     Set the alg attribute of jwk to the string "RSA-OAEP-256".
                    // If hash is "SHA-384":
                    //     Set the alg attribute of jwk to the string "RSA-OAEP-384".
                    // If hash is "SHA-512":
                    //     Set the alg attribute of jwk to the string "RSA-OAEP-512".
                    // Otherwise:
                    //     Perform any key export steps defined by other applicable specifications,
                    //     passing format and the hash attribute of the [[algorithm]] internal slot
                    //     of key and obtaining alg.
                    //     Set the alg attribute of jwk to alg.
                    let alg = match hash {
                        ALG_SHA1 => "RSA-OAEP",
                        ALG_SHA256 => "RSA-OAEP-256",
                        ALG_SHA384 => "RSA-OAEP-384",
                        ALG_SHA512 => "RSA-OAEP-512",
                        _ => {
                            return Err(Error::NotSupported(Some(format!(
                                "Unsupported \"{}\" hash for RSA-OAEP",
                                hash
                            ))));
                        },
                    };
                    jwk.alg = Some(DOMString::from(alg));
                },
            }

            // Step 3.5. Set the attributes n and e of jwk according to the corresponding
            // definitions in JSON Web Algorithms [JWA], Section 6.3.1.
            let (n, e) = match key.handle() {
                Handle::RsaPrivateKey(private_key) => (private_key.n(), private_key.e()),
                Handle::RsaPublicKey(public_key) => (public_key.n(), public_key.e()),
                _ => {
                    return Err(Error::Operation(Some(
                        "Failed to extract modulus n and public exponent e from RSA key"
                            .to_string(),
                    )));
                },
            };
            jwk.n = Some(Base64UrlUnpadded::encode_string(&n.to_bytes_be()).into());
            jwk.e = Some(Base64UrlUnpadded::encode_string(&e.to_bytes_be()).into());

            // Step 3.6. If the [[type]] internal slot of key is "private":
            if key.Type() == KeyType::Private {
                // Step 3.6.1. Set the attributes named d, p, q, dp, dq, and qi of jwk according to
                // the corresponding definitions in JSON Web Algorithms [JWA], Section 6.3.2.
                let Handle::RsaPrivateKey(private_key) = key.handle() else {
                    return Err(Error::Operation(Some(
                        "The [[handle]] internal slot of key is not an RSA private key".to_string(),
                    )));
                };
                let mut private_key = private_key.clone();
                private_key.precompute().map_err(|_| {
                    Error::Operation(Some("Failed to perform RSA pre-computation".to_string()))
                })?;
                let primes = private_key.primes();
                let d = private_key.d();
                let p = primes.first().ok_or(Error::Operation(Some(
                    "Failed to extract first prime factor p from RSA private key".to_string(),
                )))?;
                let q = primes.get(1).ok_or(Error::Operation(Some(
                    "Failed to extract second prime factor q from RSA private key".to_string(),
                )))?;
                let dp = private_key.dp().ok_or(Error::Operation(Some(
                    "Failed to extract first factor CRT exponent dp from RSA private key"
                        .to_string(),
                )))?;
                let dq = private_key.dq().ok_or(Error::Operation(Some(
                    "Failed to extract second factor CRT exponent dq from RSA private key"
                        .to_string(),
                )))?;
                let qi = private_key
                    .qinv()
                    .ok_or(Error::Operation(Some(
                        "Failed to extract first CRT coefficient qi from RSA private key"
                            .to_string(),
                    )))?
                    .modpow(&BigInt::one(), &BigInt::from_biguint(Sign::Plus, p.clone()))
                    .to_biguint()
                    .ok_or(Error::Operation(Some(
                        "Failed to convert first CRT coefficient qi to BigUint".to_string(),
                    )))?;
                jwk.encode_string_field(JwkStringField::D, &d.to_bytes_be());
                jwk.encode_string_field(JwkStringField::P, &p.to_bytes_be());
                jwk.encode_string_field(JwkStringField::Q, &q.to_bytes_be());
                jwk.encode_string_field(JwkStringField::DP, &dp.to_bytes_be());
                jwk.encode_string_field(JwkStringField::DQ, &dq.to_bytes_be());
                jwk.encode_string_field(JwkStringField::QI, &qi.to_bytes_be());

                // Step 3.6.2. If the underlying RSA private key represented by the [[handle]]
                // internal slot of key is represented by more than two primes, set the attribute
                // named oth of jwk according to the corresponding definition in JSON Web
                // Algorithms [JWA], Section 6.3.2.7
                let mut oth = Vec::new();
                for (i, p_i) in primes.iter().enumerate().skip(2) {
                    // d_i = d mod (p_i - 1)
                    // t_i = (p_1 * p_2 * ... * p_(i-1)) ^ (-1) mod p_i
                    let d_i = private_key
                        .d()
                        .modpow(&BigUint::one(), &p_i.sub(&BigUint::one()));
                    let t_i = primes
                        .iter()
                        .take(i - 1)
                        .fold(BigUint::one(), |product, p_j| product.mul(p_j))
                        .mod_inverse(p_i)
                        .ok_or(Error::Operation(Some(
                            "Failed to compute factor CRT coefficient of other RSA primes"
                                .to_string(),
                        )))?
                        .modpow(
                            &BigInt::one(),
                            &BigInt::from_biguint(Sign::Plus, p_i.clone()),
                        )
                        .to_biguint()
                        .ok_or(Error::Operation(Some(
                            "Failed to convert factor CRT coefficient of other RSA primes to BigUint"
                                .to_string(),
                        )))?;
                    oth.push(RsaOtherPrimesInfo {
                        r: Some(Base64UrlUnpadded::encode_string(&p_i.to_bytes_be()).into()),
                        d: Some(Base64UrlUnpadded::encode_string(&d_i.to_bytes_be()).into()),
                        t: Some(Base64UrlUnpadded::encode_string(&t_i.to_bytes_be()).into()),
                    });
                }
                if !oth.is_empty() {
                    jwk.oth = Some(oth);
                }
            }

            // Step 3.7. Set the key_ops attribute of jwk to the usages attribute of key.
            jwk.set_key_ops(key.usages());

            // Step 3.8. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 3.9. Let result be jwk.
            ExportedKey::Jwk(Box::new(jwk))
        },
        // Otherwise
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported export key format for RSA key".to_string(),
            )));
        },
    };

    // Step 4. Return result.
    Ok(result)
}
