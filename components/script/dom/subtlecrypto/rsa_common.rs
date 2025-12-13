/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::{Mul, Sub};

use base64ct::{Base64UrlUnpadded, Encoding};
use num_bigint_dig::{BigInt, ModInverse, Sign};
use num_traits::One;
use pkcs8::EncodePrivateKey;
use pkcs8::rand_core::OsRng;
use pkcs8::spki::EncodePublicKey;
use rsa::traits::{PrivateKeyParts, PublicKeyParts};
use rsa::{BigUint, RsaPrivateKey};
use sec1::der::Encode;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, CryptoKeyPair, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{
    JsonWebKey, KeyFormat, RsaOtherPrimesInfo,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_RSA_OAEP, ALG_RSA_PSS, ALG_RSASSA_PKCS1_V1_5, ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512,
    ExportedKey, KeyAlgorithmAndDerivatives, SubtleRsaHashedKeyAlgorithm,
    SubtleRsaHashedKeyGenParams,
};
use crate::script_runtime::CanGc;

pub(crate) enum RsaAlgorithm {
    RsaSsaPkcs1v15,
    RsaPss,
    RsaOaep,
}

/// <https://w3c.github.io/webcrypto/#rsassa-pkcs1-operations-generate-key>
/// <https://w3c.github.io/webcrypto/#rsa-pss-operations-generate-key>
/// <https://w3c.github.io/webcrypto/#rsa-oaep-operations-generate-key>
pub(crate) fn generate_key(
    rsa_algorithm: RsaAlgorithm,
    global: &GlobalScope,
    normalized_algorithm: &SubtleRsaHashedKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<CryptoKeyPair, Error> {
    match rsa_algorithm {
        RsaAlgorithm::RsaSsaPkcs1v15 | RsaAlgorithm::RsaPss => {
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
            RsaAlgorithm::RsaSsaPkcs1v15 => ALG_RSASSA_PKCS1_V1_5,
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
        RsaAlgorithm::RsaSsaPkcs1v15 | RsaAlgorithm::RsaPss => {
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
        global,
        KeyType::Public,
        true,
        KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algorithm.clone()),
        intersected_usages,
        Handle::RsaPublicKey(public_key),
        can_gc,
    );

    // Step 14. Let privateKey be a new CryptoKey representing the private key of the generated key
    // pair.
    // Step 15. Set the [[type]] internal slot of privateKey to "private"
    // Step 16. Set the [[algorithm]] internal slot of privateKey to algorithm.
    // Step 17. Set the [[extractable]] internal slot of privateKey to extractable.
    let intersected_usages = match rsa_algorithm {
        RsaAlgorithm::RsaSsaPkcs1v15 | RsaAlgorithm::RsaPss => {
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
        global,
        KeyType::Private,
        extractable,
        KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algorithm),
        intersected_usages,
        Handle::RsaPrivateKey(private_key),
        can_gc,
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
                RsaAlgorithm::RsaSsaPkcs1v15 => {
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
                jwk.d = Some(Base64UrlUnpadded::encode_string(&d.to_bytes_be()).into());
                jwk.p = Some(Base64UrlUnpadded::encode_string(&p.to_bytes_be()).into());
                jwk.q = Some(Base64UrlUnpadded::encode_string(&q.to_bytes_be()).into());
                jwk.dp = Some(Base64UrlUnpadded::encode_string(&dp.to_bytes_be()).into());
                jwk.dq = Some(Base64UrlUnpadded::encode_string(&dq.to_bytes_be()).into());
                jwk.qi = Some(Base64UrlUnpadded::encode_string(&qi.to_bytes_be()).into());

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
            jwk.key_ops = Some(
                key.usages()
                    .iter()
                    .map(|usage| DOMString::from(usage.as_str()))
                    .collect::<Vec<DOMString>>(),
            );

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
