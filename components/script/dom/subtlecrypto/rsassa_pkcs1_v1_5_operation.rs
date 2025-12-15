/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base64ct::{Base64UrlUnpadded, Encoding};
use pkcs8::der::asn1::BitString;
use pkcs8::der::{AnyRef, Decode};
use pkcs8::{PrivateKeyInfo, SubjectPublicKeyInfo};
use rsa::pkcs1::{self, DecodeRsaPrivateKey};
use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::signature::{SignatureEncoding, Signer, Verifier};
use rsa::traits::PublicKeyParts;
use rsa::{BigUint, RsaPrivateKey, RsaPublicKey};
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, CryptoKeyPair, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{
    AlgorithmIdentifier, JsonWebKey, KeyFormat,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::rsa_common::{self, RsaAlgorithm};
use crate::dom::subtlecrypto::{
    ALG_RSASSA_PKCS1_V1_5, ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, ExportedKey,
    JsonWebKeyExt, KeyAlgorithmAndDerivatives, Operation, SubtleRsaHashedImportParams,
    SubtleRsaHashedKeyAlgorithm, SubtleRsaHashedKeyGenParams, normalize_algorithm,
};
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/webcrypto/#rsassa-pkcs1-operations-sign>
pub(crate) fn sign(key: &CryptoKey, message: &[u8]) -> Result<Vec<u8>, Error> {
    // Step 1. If the [[type]] internal slot of key is not "private", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Private {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"private\"".to_string(),
        )));
    }

    // Step 2. Perform the signature generation operation defined in Section 8.2 of [RFC3447] with
    // the key represented by the [[handle]] internal slot of key as the signer's private key and
    // message as M and using the hash function specified in the hash attribute of the
    // [[algorithm]] internal slot of key as the Hash option for the EMSA-PKCS1-v1_5 encoding
    // method.
    // Step 3. If performing the operation results in an error, then throw an OperationError.
    // Step 4. Let signature be the value S that results from performing the operation.
    let Handle::RsaPrivateKey(private_key) = key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of key is not an RSA private key".to_string(),
        )));
    };
    let KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algorithm) = key.algorithm() else {
        return Err(Error::Operation(Some(
            "[[algorithm]] internal slot of key is not an RsaHashedKeyAlgorithm".to_string(),
        )));
    };
    let signature = match algorithm.hash.name() {
        ALG_SHA1 => {
            let signing_key = SigningKey::<Sha1>::new(private_key.clone());
            signing_key.try_sign(message)
        },
        ALG_SHA256 => {
            let signing_key = SigningKey::<Sha256>::new(private_key.clone());
            signing_key.try_sign(message)
        },
        ALG_SHA384 => {
            let signing_key = SigningKey::<Sha384>::new(private_key.clone());
            signing_key.try_sign(message)
        },
        ALG_SHA512 => {
            let signing_key = SigningKey::<Sha512>::new(private_key.clone());
            signing_key.try_sign(message)
        },
        _ => {
            return Err(Error::Operation(Some(format!(
                "Unsupported \"{}\" hash for RSASSA-PKCS1-v1_5",
                algorithm.hash.name()
            ))));
        },
    }
    .map_err(|_| Error::Operation(Some("RSASSA-PKCS1-v1_5 failed to sign message".to_string())))?;

    // Step 5. Return signature.
    Ok(signature.to_vec())
}

/// <https://w3c.github.io/webcrypto/#rsassa-pkcs1-operations-verify>
pub(crate) fn verify(key: &CryptoKey, message: &[u8], signature: &[u8]) -> Result<bool, Error> {
    // Step 1. If the [[type]] internal slot of key is not "public", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Public {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"public\"".to_string(),
        )));
    }

    // Step 2. Perform the signature verification operation defined in Section 8.2 of [RFC3447]
    // with the key represented by the [[handle]] internal slot of key as the signer's RSA public
    // key and message as M and signature as S and using the hash function specified in the hash
    // attribute of the [[algorithm]] internal slot of key as the Hash option for the
    // EMSA-PKCS1-v1_5 encoding method.
    // Step 3. Let result be a boolean with value true if the result of the operation was "valid
    // signature" and the value false otherwise.
    let Handle::RsaPublicKey(public_key) = key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of key is not an RSA public key".to_string(),
        )));
    };
    let KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algorithm) = key.algorithm() else {
        return Err(Error::Operation(Some(
            "[[algorithm]] internal slot of key is not an RsaHashedKeyAlgorithm".to_string(),
        )));
    };
    let signature = Signature::try_from(signature)
        .map_err(|_| Error::Operation(Some("Failed to parse RSA signature".to_string())))?;
    let result = match algorithm.hash.name() {
        ALG_SHA1 => {
            let verifying_key = VerifyingKey::<Sha1>::new(public_key.clone());
            verifying_key.verify(message, &signature)
        },
        ALG_SHA256 => {
            let verifying_key = VerifyingKey::<Sha256>::new(public_key.clone());
            verifying_key.verify(message, &signature)
        },
        ALG_SHA384 => {
            let verifying_key = VerifyingKey::<Sha384>::new(public_key.clone());
            verifying_key.verify(message, &signature)
        },
        ALG_SHA512 => {
            let verifying_key = VerifyingKey::<Sha512>::new(public_key.clone());
            verifying_key.verify(message, &signature)
        },
        _ => {
            return Err(Error::Operation(Some(format!(
                "Unsupported \"{}\" hash for RSASSA-PKCS1-v1_5",
                algorithm.hash.name()
            ))));
        },
    }
    .is_ok();

    // Step 4. Return result.
    Ok(result)
}

/// <https://w3c.github.io/webcrypto/#rsassa-pkcs1-operations-generate-key>
pub(crate) fn generate_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleRsaHashedKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<CryptoKeyPair, Error> {
    rsa_common::generate_key(
        RsaAlgorithm::RsaSsaPkcs1v15,
        global,
        normalized_algorithm,
        extractable,
        usages,
        can_gc,
    )
}

/// <https://w3c.github.io/webcrypto/#rsassa-pkcs1-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleRsaHashedImportParams,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2.
    let (key_handle, key_type) = match format {
        // If format is "spki":
        KeyFormat::Spki => {
            // Step 2.1. If usages contains an entry which is not "verify", then throw a
            // SyntaxError.
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
            // Step 2.1. If usages contains an entry which is not "sign" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"sign\"".to_string(),
                )));
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
            let cx = GlobalScope::get_cx();
            let jwk = JsonWebKey::parse(cx, key_data)?;

            // Step 2.2. If the d field of jwk is present and usages contains an entry which is not
            // "sign", or, if the d field of jwk is not present and usages contains an entry which
            // is not "verify" then throw a SyntaxError.
            if jwk.d.is_some() && usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                return Err(Error::Syntax(Some(
                    "The d field of jwk is present and usages contains an entry which is not \"sign\""
                        .to_string()
                )));
            }
            if jwk.d.is_none() && usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(Some(
                    "The d field of jwk is not present and usages contains an entry which is not \"verify\""
                        .to_string()
                )));
            }

            // Step 2.3. If the kty field of jwk is not a case-sensitive string match to "RSA",
            // then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "RSA") {
                return Err(Error::Data(Some(
                    "The kty field of jwk is not a case-sensitive string match to \"RSA\""
                        .to_string(),
                )));
            }

            // Step 2.4. If usages is non-empty and the use field of jwk is present and is not a
            // case-sensitive string match to "sig", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "sig") {
                return Err(Error::Data(Some(
                    "Usages is non-empty and the use field of jwk is present and \
                    is not a case-sensitive string match to \"sig\""
                        .to_string(),
                )));
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

            // Step 2.7. Let hash be a be a string whose initial value is undefined.
            // Step 2.8.
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
            //     Perform any key import steps defined by other applicable specifications, passing
            //     format, jwk and obtaining hash.
            //     If an error occurred or there are no applicable specifications, throw a DataError.
            let hash = match jwk.alg {
                None => None,
                Some(alg) => match &*alg.str() {
                    "RS1" => Some(ALG_SHA1),
                    "RS256" => Some(ALG_SHA256),
                    "RS384" => Some(ALG_SHA384),
                    "RS512" => Some(ALG_SHA512),
                    _ => None,
                },
            };

            // Step 2.9. If hash is not undefined:
            if let Some(hash) = hash {
                // Step 2.9.1. Let normalizedHash be the result of normalize an algorithm with alg
                // set to hash and op set to digest.
                let normalized_hash = normalize_algorithm(
                    cx,
                    &Operation::Digest,
                    &AlgorithmIdentifier::String(DOMString::from(hash)),
                    can_gc,
                )?;

                // Step 2.9.2. If normalizedHash is not equal to the hash member of
                // normalizedAlgorithm, throw a DataError.
                if normalized_hash.name() != normalized_algorithm.hash.name() {
                    return Err(Error::Data(Some(
                        "The normalizedHash is not equal to the hash member of normalizedAlgorithm"
                            .to_string(),
                    )));
                }
            }

            // Step 2.10.
            match jwk.d {
                // If the d field of jwk is present:
                Some(d) => {
                    // Step 2.10.1. If jwk does not meet the requirements of Section 6.3.2 of JSON
                    // Web Algorithms [JWA], then throw a DataError.
                    let n = Base64UrlUnpadded::decode_vec(
                        &jwk.n
                            .ok_or(Error::Data(Some(
                                "The n field does not exist in jwk".to_string(),
                            )))?
                            .str(),
                    )
                    .map_err(|_| Error::Data(Some("Fail to decode n field of jwk".to_string())))?;
                    let e = Base64UrlUnpadded::decode_vec(
                        &jwk.e
                            .ok_or(Error::Data(Some(
                                "The e field does not exist in jwk".to_string(),
                            )))?
                            .str(),
                    )
                    .map_err(|_| Error::Data(Some("Fail to decode e field of jwk".to_string())))?;
                    let d = Base64UrlUnpadded::decode_vec(&d.str()).map_err(|_| {
                        Error::Data(Some("Fail to decode d field of jwk".to_string()))
                    })?;
                    let p = jwk
                        .p
                        .map(|p| Base64UrlUnpadded::decode_vec(&p.str()))
                        .transpose()
                        .map_err(|_| {
                            Error::Data(Some("Fail to decode p field of jwk".to_string()))
                        })?;
                    let q = jwk
                        .q
                        .map(|q| Base64UrlUnpadded::decode_vec(&q.str()))
                        .transpose()
                        .map_err(|_| {
                            Error::Data(Some("Fail to decode q field of jwk".to_string()))
                        })?;
                    let dp = jwk
                        .dp
                        .map(|dp| Base64UrlUnpadded::decode_vec(&dp.str()))
                        .transpose()
                        .map_err(|_| {
                            Error::Data(Some("Fail to decode dp field of jwk".to_string()))
                        })?;
                    let dq = jwk
                        .dq
                        .map(|dq| Base64UrlUnpadded::decode_vec(&dq.str()))
                        .transpose()
                        .map_err(|_| {
                            Error::Data(Some("Fail to decode dq field of jwk".to_string()))
                        })?;
                    let qi = jwk
                        .qi
                        .map(|qi| Base64UrlUnpadded::decode_vec(&qi.str()))
                        .transpose()
                        .map_err(|_| {
                            Error::Data(Some("Fail to decode qi field of jwk".to_string()))
                        })?;
                    let mut primes = match (p, q, dp, dq, qi) {
                        (Some(p), Some(q), Some(_dp), Some(_dq), Some(_qi)) => vec![p, q],
                        (None, None, None, None, None) => Vec::new(),
                        _ => return Err(Error::Data(Some(
                            "The p, q, dp, dq, qi fields of jwk must be either all-present or all-absent"
                                .to_string()
                        ))),
                    };
                    if let Some(oth) = jwk.oth {
                        if primes.is_empty() {
                            return Err(Error::Data(Some(
                                "The oth field exists in jwk but one of p, q, dp, dq, qi is missing".to_string()
                            )));
                        }
                        for other_prime in oth {
                            let r = Base64UrlUnpadded::decode_vec(
                                &other_prime
                                    .r
                                    .ok_or(Error::Data(Some(
                                        "The e field does not exist in oth field of jwk"
                                            .to_string(),
                                    )))?
                                    .str(),
                            )
                            .map_err(|_| {
                                Error::Data(Some(
                                    "Fail to decode r field of oth field of jwk".to_string(),
                                ))
                            })?;
                            primes.push(r);
                        }
                    }

                    // Step 2.10.2. Let privateKey represents the RSA private key identified by
                    // interpreting jwk according to Section 6.3.2 of JSON Web Algorithms [JWA].
                    // Step 2.10.3. If privateKey is not a valid RSA private key according to
                    // [RFC3447], then throw a DataError.
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
                            "Fail to construct RSA private key from values in jwk".to_string(),
                        ))
                    })?;

                    // Step 2.10.4. Let key be a new CryptoKey object that represents privateKey.
                    // Step 2.10.5. Set the [[type]] internal slot of key to "private"
                    // NOTE: Done in Step 3-8.
                    let key_handle = Handle::RsaPrivateKey(private_key);
                    let key_type = KeyType::Private;
                    (key_handle, key_type)
                },
                // Otherwise:
                None => {
                    // Step 2.10.1. If jwk does not meet the requirements of Section 6.3.1 of JSON
                    // Web Algorithms [JWA], then throw a DataError.
                    let n = Base64UrlUnpadded::decode_vec(
                        &jwk.n
                            .ok_or(Error::Data(Some(
                                "The n field does not exist in jwk".to_string(),
                            )))?
                            .str(),
                    )
                    .map_err(|_| Error::Data(Some("Fail to decode n field of jwk".to_string())))?;
                    let e = Base64UrlUnpadded::decode_vec(
                        &jwk.e
                            .ok_or(Error::Data(Some(
                                "The e field does not exist in jwk".to_string(),
                            )))?
                            .str(),
                    )
                    .map_err(|_| Error::Data(Some("Fail to decode e field of jwk".to_string())))?;

                    // Step 2.10.2. Let publicKey represent the RSA public key identified by
                    // interpreting jwk according to Section 6.3.1 of JSON Web Algorithms [JWA].
                    // Step 2.10.3. If publicKey can be determined to not be a valid RSA public key
                    // according to [RFC3447], then throw a DataError.
                    let public_key =
                        RsaPublicKey::new(BigUint::from_bytes_be(&n), BigUint::from_bytes_be(&e))
                            .map_err(|_| {
                            Error::Data(Some(
                                "Fail to construct RSA public key from values in jwk".to_string(),
                            ))
                        })?;

                    // Step 2.10.4. Let key be a new CryptoKey representing publicKey.
                    // Step 2.10.5. Set the [[type]] internal slot of key to "public"
                    // NOTE: Done in Step 3-8.
                    let key_handle = Handle::RsaPublicKey(public_key);
                    let key_type = KeyType::Public;
                    (key_handle, key_type)
                },
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
    // Step 4. Set the name attribute of algorithm to "RSASSA-PKCS1-v1_5"
    // Step 5. Set the modulusLength attribute of algorithm to the length, in bits, of the RSA
    // public modulus.
    // Step 6. Set the publicExponent attribute of algorithm to the BigInteger representation of
    // the RSA public exponent.
    // Step 7. Set the hash attribute of algorithm to the hash member of normalizedAlgorithm.
    // Step 8. Set the [[algorithm]] internal slot of key to algorithm.
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
        name: ALG_RSASSA_PKCS1_V1_5.to_string(),
        modulus_length,
        public_exponent,
        hash: normalized_algorithm.hash.clone(),
    };
    let key = CryptoKey::new(
        global,
        key_type,
        extractable,
        KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algorithm),
        usages,
        key_handle,
        can_gc,
    );

    // Step 9. Return key.
    Ok(key)
}

/// <https://w3c.github.io/webcrypto/#rsassa-pkcs1-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    rsa_common::export_key(RsaAlgorithm::RsaSsaPkcs1v15, format, key)
}
