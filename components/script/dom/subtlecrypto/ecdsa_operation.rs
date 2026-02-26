/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use digest::Digest;
use ecdsa::signature::hazmat::{PrehashVerifier, RandomizedPrehashSigner};
use ecdsa::{Signature, SigningKey, VerifyingKey};
use elliptic_curve::SecretKey;
use elliptic_curve::rand_core::OsRng;
use elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint, ValidatePublicKey};
use js::context::JSContext;
use p256::NistP256;
use p384::NistP384;
use p521::NistP521;
use pkcs8::der::Decode;
use pkcs8::spki::EncodePublicKey;
use pkcs8::{AssociatedOid, EncodePrivateKey, PrivateKeyInfo, SubjectPublicKeyInfo};
use sec1::der::asn1::BitString;
use sec1::{EcParameters, EcPrivateKey, EncodedPoint};
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

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
    ALG_ECDSA, ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, ExportedKey, JsonWebKeyExt,
    JwkStringField, KeyAlgorithmAndDerivatives, NAMED_CURVE_P256, NAMED_CURVE_P384,
    NAMED_CURVE_P521, NormalizedAlgorithm, SUPPORTED_CURVES, SubtleEcKeyAlgorithm,
    SubtleEcKeyGenParams, SubtleEcKeyImportParams, SubtleEcdsaParams,
};

const P256_PREHASH_LENGTH: usize = 32;
const P384_PREHASH_LENGTH: usize = 48;
const P521_PREHASH_LENGTH: usize = 66;

/// <https://w3c.github.io/webcrypto/#ecdsa-operations-sign>
pub(crate) fn sign(
    normalized_algorithm: &SubtleEcdsaParams,
    key: &CryptoKey,
    message: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the [[type]] internal slot of key is not "private", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Private {
        return Err(Error::InvalidAccess(None));
    }

    // Step 2. Let hashAlgorithm be the hash member of normalizedAlgorithm.
    let hash_algorithm = &normalized_algorithm.hash;

    // Step 3. Let M be the result of performing the digest operation specified by hashAlgorithm
    // using message.
    let m = match hash_algorithm.name() {
        ALG_SHA1 => Sha1::new_with_prefix(message).finalize().to_vec(),
        ALG_SHA256 => Sha256::new_with_prefix(message).finalize().to_vec(),
        ALG_SHA384 => Sha384::new_with_prefix(message).finalize().to_vec(),
        ALG_SHA512 => Sha512::new_with_prefix(message).finalize().to_vec(),
        _ => return Err(Error::NotSupported(None)),
    };

    // Step 4. Let d be the ECDSA private key associated with key.
    // Step 5. Let params be the EC domain parameters associated with key.
    // Step 6.
    // If the namedCurve attribute of the [[algorithm]] internal slot of key is "P-256", "P-384" or
    // "P-521":
    //     1. Perform the ECDSA signing process, as specified in [RFC6090], Section 5.4.2, with M
    //        as the message, using params as the EC domain parameters, and with d as the private
    //        key.
    //     2. Let r and s be the pair of integers resulting from performing the ECDSA signing
    //        process.
    //     3. Let result be an empty byte sequence.
    //     4. Let n be the smallest integer such that n * 8 is greater than the logarithm to base 2
    //        of the order of the base point of the elliptic curve identified by params.
    //     5. Convert r to a byte sequence of length n and append it to result.
    //     6. Convert s to a byte sequence of length n and append it to result.
    // Otherwise, the namedCurve attribute of the [[algorithm]] internal slot of key is a value
    // specified in an applicable specification:
    //     Perform the ECDSA signature steps specified in that specification, passing in M, params
    //     and d and resulting in result.
    // NOTE: We currently do not support other applicable specifications.
    let KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm) = key.algorithm() else {
        return Err(Error::Operation(None));
    };
    let result = match algorithm.named_curve.as_str() {
        NAMED_CURVE_P256 => {
            // P-256 expects prehash with length at least 32 bytes. If M is shorter than 32 bytes,
            // expand it by left padding with zeros.
            let m = expand_prehash(m, P256_PREHASH_LENGTH);

            let Handle::P256PrivateKey(d) = key.handle() else {
                return Err(Error::Operation(None));
            };
            let signing_key = SigningKey::<NistP256>::from(d);
            let signature: Signature<NistP256> = signing_key
                .sign_prehash_with_rng(&mut OsRng, m.as_slice())
                .map_err(|_| Error::Operation(None))?;
            signature.to_vec()
        },
        NAMED_CURVE_P384 => {
            // P-384 expects prehash with length at least 48 bytes. If M is shorter than 48 bytes,
            // expand it by left padding with zeros.
            let m = expand_prehash(m, P384_PREHASH_LENGTH);

            let Handle::P384PrivateKey(d) = key.handle() else {
                return Err(Error::Operation(None));
            };
            let signing_key = SigningKey::<NistP384>::from(d);
            let signature: Signature<NistP384> = signing_key
                .sign_prehash_with_rng(&mut OsRng, m.as_slice())
                .map_err(|_| Error::Abort(None))?;
            signature.to_vec()
        },
        NAMED_CURVE_P521 => {
            // P-521 expects prehash with length at least 66 bytes. If M is shorter than 66 bytes,
            // expand it by left padding with zeros.
            let m = expand_prehash(m, P521_PREHASH_LENGTH);

            let Handle::P521PrivateKey(d) = key.handle() else {
                return Err(Error::Operation(None));
            };
            let signing_key = p521::ecdsa::SigningKey::from_slice(d.to_bytes().as_slice())
                .map_err(|_| Error::Operation(None))?;
            let signature: Signature<NistP521> = signing_key
                .sign_prehash_with_rng(&mut OsRng, m.as_slice())
                .map_err(|_| Error::Operation(None))?;
            signature.to_vec()
        },
        _ => return Err(Error::NotSupported(None)),
    };

    // Step 7. Return result.
    Ok(result)
}

/// <https://w3c.github.io/webcrypto/#ecdsa-operations-verify>
pub(crate) fn verify(
    normalized_algorithm: &SubtleEcdsaParams,
    key: &CryptoKey,
    message: &[u8],
    signature: &[u8],
) -> Result<bool, Error> {
    // Step 1. If the [[type]] internal slot of key is not "public", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Public {
        return Err(Error::InvalidAccess(None));
    }

    // Step 2. Let hashAlgorithm be the hash member of normalizedAlgorithm.
    let hash_algorithm = &normalized_algorithm.hash;

    // Step 3. Let M be the result of performing the digest operation specified by hashAlgorithm
    // using message.
    let m = match hash_algorithm.name() {
        ALG_SHA1 => Sha1::new_with_prefix(message).finalize().to_vec(),
        ALG_SHA256 => Sha256::new_with_prefix(message).finalize().to_vec(),
        ALG_SHA384 => Sha384::new_with_prefix(message).finalize().to_vec(),
        ALG_SHA512 => Sha512::new_with_prefix(message).finalize().to_vec(),
        _ => return Err(Error::NotSupported(None)),
    };

    // Step 4. Let Q be the ECDSA public key associated with key.
    // Step 5. Let params be the EC domain parameters associated with key.
    // Step 6.
    // If the namedCurve attribute of the [[algorithm]] internal slot of key is "P-256", "P-384" or
    // "P-521":
    //     1. Let n be the smallest integer such that n * 8 is greater than the logarithm to base 2
    //        of the order of the base point of the elliptic curve identified by params.
    //     2. If signature does not have a length of n * 2 bytes, then return false.
    //     3. Let r be the result of converting the first n bytes of signature to an integer.
    //     4. Let s be the result of converting the last n bytes of signature to an integer.
    //     5. Perform the ECDSA verifying process, as specified in [RFC6090], Section 5.4.3, with M
    //        as the received message, (r, s) as the signature and using params as the EC domain
    //        parameters, and Q as the public key.
    // Otherwise, the namedCurve attribute of the [[algorithm]] internal slot of key is a value
    // specified in an applicable specification:
    //     Perform the ECDSA verification steps specified in that specification passing in M,
    //     signature, params and Q and resulting in an indication of whether or not the purported
    //     signature is valid.
    // Step 7. Let result be a boolean with the value true if the signature is valid and the value
    // false otherwise.
    // NOTE: We currently do not support other applicable specifications.
    let KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm) = key.algorithm() else {
        return Err(Error::Operation(None));
    };
    let result = match algorithm.named_curve.as_str() {
        NAMED_CURVE_P256 => {
            // P-256 expects prehash with length at least 32 bytes. If M is shorter than 32 bytes,
            // expand it by left padding with zeros.
            let m = expand_prehash(m, P256_PREHASH_LENGTH);

            let Handle::P256PublicKey(q) = key.handle() else {
                return Err(Error::Operation(None));
            };
            match Signature::<NistP256>::from_slice(signature) {
                Ok(signature) => {
                    let verifying_key = VerifyingKey::<NistP256>::from(q);
                    verifying_key.verify_prehash(&m, &signature).is_ok()
                },
                Err(_) => false,
            }
        },
        NAMED_CURVE_P384 => {
            // P-384 expects prehash with length at least 48 bytes. If M is shorter than 48 bytes,
            // expand it by left padding with zeros.
            let m = expand_prehash(m, P384_PREHASH_LENGTH);

            let Handle::P384PublicKey(q) = key.handle() else {
                return Err(Error::Operation(None));
            };
            match Signature::<NistP384>::from_slice(signature) {
                Ok(signature) => {
                    let verifying_key = VerifyingKey::<NistP384>::from(q);
                    verifying_key.verify_prehash(&m, &signature).is_ok()
                },
                Err(_) => false,
            }
        },
        NAMED_CURVE_P521 => {
            // P-521 expects prehash with length at least 66 bytes. If M is shorter than 66 bytes,
            // expand it by left padding with zeros.
            let m = expand_prehash(m, P521_PREHASH_LENGTH);

            let Handle::P521PublicKey(q) = key.handle() else {
                return Err(Error::Operation(None));
            };
            match (
                Signature::<NistP521>::from_slice(signature),
                p521::ecdsa::VerifyingKey::from_sec1_bytes(q.to_sec1_bytes().to_vec().as_slice()),
            ) {
                (Ok(signature), Ok(verifying_key)) => {
                    verifying_key.verify_prehash(&m, &signature).is_ok()
                },
                _ => false,
            }
        },
        _ => return Err(Error::NotSupported(None)),
    };

    // Step 8. Return result.
    Ok(result)
}

/// <https://w3c.github.io/webcrypto/#ecdsa-operations-generate-key>
pub(crate) fn generate_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleEcKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<CryptoKeyPair, Error> {
    // Step 1. If usages contains a value which is not one of "sign" or "verify", then throw a
    // SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify))
    {
        return Err(Error::Syntax(None));
    }

    // Step 2.
    // If the namedCurve member of normalizedAlgorithm is "P-256", "P-384" or "P-521":
    //     Generate an Elliptic Curve key pair, as defined in [RFC6090] with domain parameters for
    //     the curve identified by the namedCurve member of normalizedAlgorithm.
    // If the namedCurve member of normalizedAlgorithm is a value specified in an applicable
    // specification:
    //     Perform the ECDSA generation steps specified in that specification, passing in
    //     normalizedAlgorithm and resulting in an elliptic curve key pair.
    // Otherwise:
    //     throw a NotSupportedError
    // Step 3. If performing the key generation operation results in an error, then throw an
    // OperationError.
    // NOTE: We currently do not support other applicable specifications.
    let (private_key_handle, public_key_handle) = match normalized_algorithm.named_curve.as_str() {
        NAMED_CURVE_P256 => {
            let private_key = SecretKey::<NistP256>::random(&mut OsRng);
            let public_key = private_key.public_key();
            (
                Handle::P256PrivateKey(private_key),
                Handle::P256PublicKey(public_key),
            )
        },
        NAMED_CURVE_P384 => {
            let private_key = SecretKey::<NistP384>::random(&mut OsRng);
            let public_key = private_key.public_key();
            (
                Handle::P384PrivateKey(private_key),
                Handle::P384PublicKey(public_key),
            )
        },
        NAMED_CURVE_P521 => {
            let private_key = SecretKey::<NistP521>::random(&mut OsRng);
            let public_key = private_key.public_key();
            (
                Handle::P521PrivateKey(private_key),
                Handle::P521PublicKey(public_key),
            )
        },
        _ => return Err(Error::NotSupported(None)),
    };

    // Step 4. Let algorithm be a new EcKeyAlgorithm object.
    // Step 5. Set the name attribute of algorithm to "ECDSA".
    // Step 6. Set the namedCurve attribute of algorithm to equal the namedCurve member of
    // normalizedAlgorithm.
    let algorithm = SubtleEcKeyAlgorithm {
        name: ALG_ECDSA.to_string(),
        named_curve: normalized_algorithm.named_curve.clone(),
    };

    // Step 7. Let publicKey be a new CryptoKey representing the public key of the generated key pair.
    // Step 8. Set the [[type]] internal slot of publicKey to "public"
    // Step 9. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 10. Set the [[extractable]] internal slot of publicKey to true.
    // Step 11. Set the [[usages]] internal slot of publicKey to be the usage intersection of
    // usages and [ "verify" ].
    let public_key = CryptoKey::new(
        cx,
        global,
        KeyType::Public,
        true,
        KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm.clone()),
        usages
            .iter()
            .filter(|usage| **usage == KeyUsage::Verify)
            .cloned()
            .collect(),
        public_key_handle,
    );

    // Step 12. Let privateKey be a new CryptoKey representing the private key of the generated key pair.
    // Step 13. Set the [[type]] internal slot of privateKey to "private"
    // Step 14. Set the [[algorithm]] internal slot of privateKey to algorithm.
    // Step 15. Set the [[extractable]] internal slot of privateKey to extractable.
    // Step 16. Set the [[usages]] internal slot of privateKey to be the usage intersection of
    // usages and [ "sign" ].
    let private_key = CryptoKey::new(
        cx,
        global,
        KeyType::Private,
        extractable,
        KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm),
        usages
            .iter()
            .filter(|usage| **usage == KeyUsage::Sign)
            .cloned()
            .collect(),
        private_key_handle,
    );

    // Step 17. Let result be a new CryptoKeyPair dictionary.
    // Step 18. Set the publicKey attribute of result to be publicKey.
    // Step 19. Set the privateKey attribute of result to be privateKey.
    let result = CryptoKeyPair {
        publicKey: Some(public_key),
        privateKey: Some(private_key),
    };

    // Step 20. Return result.
    Ok(result)
}

/// <https://w3c.github.io/webcrypto/#ecdsa-operations-import-key>
pub(crate) fn import_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleEcKeyImportParams,
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
            // Step 2.1. If usages contains a value which is not "verify" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. Let spki be the result of running the parse a subjectPublicKeyInfo
            // algorithm over keyData
            // Step 2.3. If an error occurred while parsing, then throw a DataError.
            let spki = SubjectPublicKeyInfo::<_, BitString>::from_der(key_data)
                .map_err(|_| Error::Data(None))?;

            // Step 2.4. If the algorithm object identifier field of the algorithm
            // AlgorithmIdentifier field of spki is not equal to the id-ecPublicKey object
            // identifier defined in [RFC5480], then throw a DataError.
            if spki.algorithm.oid != elliptic_curve::ALGORITHM_OID {
                return Err(Error::Data(None));
            }

            // Step 2.5. If the parameters field of the algorithm AlgorithmIdentifier field of spki
            // is absent, then throw a DataError.
            // Step 2.6. Let params be the parameters field of the algorithm AlgorithmIdentifier
            // field of spki.
            // Step 2.7. If params is not an instance of the ECParameters ASN.1 type defined in
            // [RFC5480] that specifies a namedCurve, then throw a DataError.
            let Some(params): Option<EcParameters> = spki.algorithm.parameters else {
                return Err(Error::Data(None));
            };

            // Step 2.8. Let namedCurve be a string whose initial value is undefined.
            // Step 2.9.
            // If params is equivalent to the secp256r1 object identifier defined in [RFC5480]:
            //     Set namedCurve "P-256".
            // If params is equivalent to the secp384r1 object identifier defined in [RFC5480]:
            //     Set namedCurve "P-384".
            // If params is equivalent to the secp521r1 object identifier defined in [RFC5480]:
            //     Set namedCurve "P-521".
            let named_curve = match params {
                EcParameters::NamedCurve(NistP256::OID) => Some(NAMED_CURVE_P256),
                EcParameters::NamedCurve(NistP384::OID) => Some(NAMED_CURVE_P384),
                EcParameters::NamedCurve(NistP521::OID) => Some(NAMED_CURVE_P521),
                _ => None,
            };

            // Step 2.10.
            let handle = match named_curve {
                // If namedCurve is not undefined:
                Some(curve) => {
                    // Step 2.10.1. Let publicKey be the Elliptic Curve public key identified by
                    // performing the conversion steps defined in Section 2.3.4 of [SEC1] using the
                    // subjectPublicKey field of spki.
                    // Step 2.10.2. The uncompressed point format MUST be supported.
                    // Step 2.10.3. If the implementation does not support the compressed point
                    // format and a compressed point is provided, throw a DataError.
                    // Step 2.10.4. If a decode error occurs or an identity point is found, throw a
                    // DataError.
                    let sec1_bytes = spki
                        .subject_public_key
                        .as_bytes()
                        .ok_or(Error::Data(None))?;
                    match curve {
                        NAMED_CURVE_P256 => {
                            let public_key = p256::PublicKey::from_sec1_bytes(sec1_bytes)
                                .map_err(|_| Error::Data(None))?;
                            Handle::P256PublicKey(public_key)
                        },
                        NAMED_CURVE_P384 => {
                            let public_key = p384::PublicKey::from_sec1_bytes(sec1_bytes)
                                .map_err(|_| Error::Data(None))?;
                            Handle::P384PublicKey(public_key)
                        },
                        NAMED_CURVE_P521 => {
                            let public_key = p521::PublicKey::from_sec1_bytes(sec1_bytes)
                                .map_err(|_| Error::Data(None))?;
                            Handle::P521PublicKey(public_key)
                        },
                        _ => unreachable!(),
                    }

                    // Step 2.10.5. Let key be a new CryptoKey that represents publicKey.
                    // NOTE: CryptoKey is created in Step 2.13 - 2.17.
                },
                // Otherwise:
                None => {
                    // Step 2.10.1. Perform any key import steps defined by other applicable
                    // specifications, passing format, spki and obtaining namedCurve and key.
                    // Step 2.10.2. If an error occurred or there are no applicable specifications,
                    // throw a DataError.
                    // NOTE: We currently do not support other applicable specifications.
                    return Err(Error::Data(None));
                },
            };

            // Step 2.11. If namedCurve is defined, and not equal to the namedCurve member of
            // normalizedAlgorithm, throw a DataError.
            if named_curve.is_some_and(|curve| curve != normalized_algorithm.named_curve) {
                return Err(Error::Data(None));
            }

            // Step 2.12. If the key value is not a valid point on the Elliptic Curve identified by
            // the namedCurve member of normalizedAlgorithm throw a DataError.
            // NOTE: Done in Step 2.10.

            // Step 2.13. Set the [[type]] internal slot of key to "public"
            // Step 2.14. Let algorithm be a new EcKeyAlgorithm.
            // Step 2.15. Set the name attribute of algorithm to "ECDSA".
            // Step 2.16. Set the namedCurve attribute of algorithm to namedCurve.
            // Step 2.17. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleEcKeyAlgorithm {
                name: ALG_ECDSA.to_string(),
                named_curve: named_curve
                    .expect("named_curve must exist here")
                    .to_string(),
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm),
                usages,
                handle,
            )
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 2.1. If usages contains a value which is not "sign" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. Let privateKeyInfo be the result of running the parse a privateKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurs while parsing, throw a DataError.
            let private_key_info =
                PrivateKeyInfo::from_der(key_data).map_err(|_| Error::Data(None))?;

            // Step 2.4. If the algorithm object identifier field of the privateKeyAlgorithm
            // PrivateKeyAlgorithm field of privateKeyInfo is not equal to the id-ecPublicKey
            // object identifier defined in [RFC5480], throw a DataError.
            if private_key_info.algorithm.oid != elliptic_curve::ALGORITHM_OID {
                return Err(Error::Data(None));
            }

            // Step 2.5. If the parameters field of the privateKeyAlgorithm
            // PrivateKeyAlgorithmIdentifier field of privateKeyInfo is not present, then throw a
            // DataError.
            // Step 2.6. Let params be the parameters field of the privateKeyAlgorithm
            // PrivateKeyAlgorithmIdentifier field of privateKeyInfo.
            // Step 2.7. If params is not an instance of the ECParameters ASN.1 type defined in
            // [RFC5480] that specifies a namedCurve, then throw a DataError.
            let params: EcParameters = if let Some(params) = private_key_info.algorithm.parameters {
                params.decode_as().map_err(|_| Error::Data(None))?
            } else {
                return Err(Error::Data(None));
            };

            // Step 2.8. Let namedCurve be a string whose initial value is undefined.
            // Step 2.9.
            // If params is equivalent to the secp256r1 object identifier defined in [RFC5480]:
            //     Set namedCurve to "P-256".
            // If params is equivalent to the secp384r1 object identifier defined in [RFC5480]:
            //     Set namedCurve to "P-384".
            // If params is equivalent to the secp521r1 object identifier defined in [RFC5480]:
            //     Set namedCurve to "P-521".
            let named_curve = match params {
                EcParameters::NamedCurve(NistP256::OID) => Some(NAMED_CURVE_P256),
                EcParameters::NamedCurve(NistP384::OID) => Some(NAMED_CURVE_P384),
                EcParameters::NamedCurve(NistP521::OID) => Some(NAMED_CURVE_P521),
                _ => None,
            };

            // Step 2.10.
            let handle = match named_curve {
                // If namedCurve is not undefined:
                Some(curve) => {
                    // Step 2.10.1. Let ecPrivateKey be the result of performing the parse an ASN.1
                    // structure algorithm, with data as the privateKey field of privateKeyInfo,
                    // structure as the ASN.1 ECPrivateKey structure specified in Section 3 of
                    // [RFC5915], and exactData set to true.
                    // Step 2.10.2. If an error occurred while parsing, then throw a DataError.
                    let ec_private_key = EcPrivateKey::try_from(private_key_info.private_key)
                        .map_err(|_| Error::Data(None))?;

                    // Step 2.10.3. If the parameters field of ecPrivateKey is present, and is not
                    // an instance of the namedCurve ASN.1 type defined in [RFC5480], or does not
                    // contain the same object identifier as the parameters field of the
                    // privateKeyAlgorithm PrivateKeyAlgorithmIdentifier field of privateKeyInfo,
                    // then throw a DataError.
                    if ec_private_key
                        .parameters
                        .is_some_and(|parameters| parameters != params)
                    {
                        return Err(Error::Data(None));
                    }

                    // Step 2.10.4. Let key be a new CryptoKey that represents the Elliptic Curve
                    // private key identified by performing the conversion steps defined in Section
                    // 3 of [RFC5915] using ecPrivateKey.
                    // NOTE: CryptoKey is created in Step 2.13 - 2.17.
                    match curve {
                        NAMED_CURVE_P256 => {
                            let private_key = p256::SecretKey::try_from(ec_private_key)
                                .map_err(|_| Error::Data(None))?;
                            Handle::P256PrivateKey(private_key)
                        },
                        NAMED_CURVE_P384 => {
                            let private_key = p384::SecretKey::try_from(ec_private_key)
                                .map_err(|_| Error::Data(None))?;
                            Handle::P384PrivateKey(private_key)
                        },
                        NAMED_CURVE_P521 => {
                            let private_key = p521::SecretKey::try_from(ec_private_key)
                                .map_err(|_| Error::Data(None))?;
                            Handle::P521PrivateKey(private_key)
                        },
                        _ => unreachable!(),
                    }
                },
                // Otherwise:
                None => {
                    // Step 2.10.1. Perform any key import steps defined by other applicable
                    // specifications, passing format, privateKeyInfo and obtaining namedCurve and
                    // key.
                    // Step 2.10.2. If an error occurred or there are no applicable specifications,
                    // throw a DataError.
                    // NOTE: We currently do not support other applicable specifications.
                    return Err(Error::Data(None));
                },
            };

            // Step 2.11. If namedCurve is defined, and not equal to the namedCurve member of
            // normalizedAlgorithm, throw a DataError.
            if named_curve.is_some_and(|curve| curve != normalized_algorithm.named_curve) {
                return Err(Error::Data(None));
            }

            // Step 2.12. If the private key value is not a valid point on the Elliptic Curve
            // identified by the namedCurve member of normalizedAlgorithm throw a DataError.
            // NOTE: Done in Step 2.10.

            // Step 2.13. Set the [[type]] internal slot of key to "private"
            // Step 2.14. Let algorithm be a new EcKeyAlgorithm.
            // Step 2.15. Set the name attribute of algorithm to "ECDSA".
            // Step 2.16. Set the namedCurve attribute of algorithm to namedCurve.
            // Step 2.17. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleEcKeyAlgorithm {
                name: ALG_ECDSA.to_string(),
                named_curve: named_curve
                    .expect("named_curve must exist here")
                    .to_string(),
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Private,
                extractable,
                KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm),
                usages,
                handle,
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

            // Step 2.2. If the d field is present and usages contains a value which is not "sign",
            // or, if the d field is not present and usages contains a value which is not "verify"
            // then throw a SyntaxError.
            if (jwk.d.is_some() && usages.iter().any(|usage| *usage != KeyUsage::Sign)) ||
                (jwk.d.is_none() && usages.iter().any(|usage| *usage != KeyUsage::Verify))
            {
                return Err(Error::Syntax(None));
            }

            // Step 2.3. If the kty field of jwk is not "EC", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "EC") {
                return Err(Error::Data(None));
            }

            // Step 2.4. If usages is non-empty and the use field of jwk is present and is not
            // "sig", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "sig") {
                return Err(Error::Data(None));
            }

            // Step 2.5. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK], or it does not contain all of the specified
            // usages values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.6. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data(None));
            }

            // Step 2.7. Let namedCurve be a string whose value is equal to the crv field of jwk.
            // Step 2.8. If namedCurve is not equal to the namedCurve member of
            // normalizedAlgorithm, throw a DataError.
            let named_curve = jwk
                .crv
                .as_ref()
                .filter(|crv| **crv == normalized_algorithm.named_curve)
                .map(|crv| crv.to_string())
                .ok_or(Error::Data(None))?;

            // Step 2.9.
            let (handle, key_type) =
                // If namedCurve is "P-256", "P-384" or "P-521":
                if matches!(
                    named_curve.as_str(),
                    NAMED_CURVE_P256 | NAMED_CURVE_P384 | NAMED_CURVE_P521
                ) {
                    // Step 2.9.1. Let algNamedCurve be a string whose initial value is undefined.
                    // Step 2.9.2.
                    // If the alg field is not present:
                    //     Let algNamedCurve be undefined.
                    // If the alg field is equal to the string "ES256":
                    //     Let algNamedCurve be the string "P-256".
                    // If the alg field is equal to the string "ES384":
                    //     Let algNamedCurve be the string "P-384".
                    // If the alg field is equal to the string "ES512":
                    //     Let algNamedCurve be the string "P-521".
                    // otherwise:
                    //     throw a DataError.
                    let alg = jwk.alg.as_ref().map(|alg| alg.to_string());
                    let alg_named_curve = match alg.as_deref() {
                        None => None,
                        Some("ES256") => Some(NAMED_CURVE_P256),
                        Some("ES384") => Some(NAMED_CURVE_P384),
                        Some("ES521") => Some(NAMED_CURVE_P521),
                        _ => return Err(Error::Data(None)),
                    };

                    // Step 2.9.3. If algNamedCurve is defined, and is not equal to namedCurve,
                    // throw a DataError.
                    if alg_named_curve.is_some_and(|alg_named_curve| alg_named_curve != named_curve) {
                        return Err(Error::Data(None));
                    }

                    // Step 2.9.4.
                    // If the d field is present:
                    if jwk.d.is_some() {
                        // Step 2.9.4.1. If jwk does not meet the requirements of Section 6.2.2 of
                        // JSON Web Algorithms [JWA], then throw a DataError.
                        let x = jwk.decode_required_string_field(JwkStringField::X)?;
                        let y = jwk.decode_required_string_field(JwkStringField::Y)?;
                        let d = jwk.decode_required_string_field(JwkStringField::D)?;

                        // Step 2.9.4.2. Let key be a new CryptoKey object that represents the
                        // Elliptic Curve private key identified by interpreting jwk according to
                        // Section 6.2.2 of JSON Web Algorithms [JWA].
                        // NOTE: CryptoKey is created in Step 2.11 - 2.14.
                        let handle = match named_curve.as_str() {
                            NAMED_CURVE_P256 => {
                                let private_key =
                                    p256::SecretKey::from_slice(&d).map_err(|_| Error::Data(None))?;
                                let mut sec1_bytes = vec![4u8];
                                sec1_bytes.extend_from_slice(&x);
                                sec1_bytes.extend_from_slice(&y);
                                let encoded_point = EncodedPoint::from_bytes(&sec1_bytes)
                                    .map_err(|_| Error::Data(None))?;
                                NistP256::validate_public_key(&private_key, &encoded_point)
                                    .map_err(|_| Error::Data(None))?;
                                Handle::P256PrivateKey(private_key)
                            },
                            NAMED_CURVE_P384 => {
                                let private_key =
                                    p384::SecretKey::from_slice(&d).map_err(|_| Error::Data(None))?;
                                let mut sec1_bytes = vec![4u8];
                                sec1_bytes.extend_from_slice(&x);
                                sec1_bytes.extend_from_slice(&y);
                                let encoded_point = EncodedPoint::from_bytes(&sec1_bytes)
                                    .map_err(|_| Error::Data(None))?;
                                NistP384::validate_public_key(&private_key, &encoded_point)
                                    .map_err(|_| Error::Data(None))?;
                                Handle::P384PrivateKey(private_key)
                            },
                            NAMED_CURVE_P521 => {
                                let private_key =
                                    p521::SecretKey::from_slice(&d).map_err(|_| Error::Data(None))?;
                                let mut sec1_bytes = vec![4u8];
                                sec1_bytes.extend_from_slice(&x);
                                sec1_bytes.extend_from_slice(&y);
                                let encoded_point = EncodedPoint::from_bytes(&sec1_bytes)
                                    .map_err(|_| Error::Data(None))?;
                                NistP521::validate_public_key(&private_key, &encoded_point)
                                    .map_err(|_| Error::Data(None))?;
                                Handle::P521PrivateKey(private_key)
                            },
                            _ => unreachable!(),
                        };

                        // Step 2.9.4.3. Set the [[type]] internal slot of Key to "private".
                        // NOTE: CryptoKey is created in Step 2.11 - 2.14.
                        let key_type = KeyType::Private;

                        (handle, key_type)
                    }
                    // Otherwise:
                    else {
                        // Step 2.9.4.1. If jwk does not meet the requirements of Section 6.2.1 of
                        // JSON Web Algorithms [JWA], then throw a DataError.
                        let x = jwk.decode_required_string_field(JwkStringField::X)?;
                        let y = jwk.decode_required_string_field(JwkStringField::Y)?;

                        // Step 2.9.4.2. Let key be a new CryptoKey object that represents the
                        // Elliptic Curve public key identified by interpreting jwk according to
                        // Section 6.2.1 of JSON Web Algorithms [JWA].
                        // NOTE: CryptoKey is created in Step 2.11 - 2.14.
                        let handle = match named_curve.as_str() {
                            NAMED_CURVE_P256 => {
                                let mut sec1_bytes = vec![4u8];
                                sec1_bytes.extend_from_slice(&x);
                                sec1_bytes.extend_from_slice(&y);
                                let encoded_point = EncodedPoint::from_bytes(&sec1_bytes)
                                    .map_err(|_| Error::Data(None))?;
                                let public_key =
                                    p256::PublicKey::from_encoded_point(&encoded_point)
                                        .into_option()
                                        .ok_or(Error::Data(None))?;
                                Handle::P256PublicKey(public_key)
                            },
                            NAMED_CURVE_P384 => {
                                let mut sec1_bytes = vec![4u8];
                                sec1_bytes.extend_from_slice(&x);
                                sec1_bytes.extend_from_slice(&y);
                                let encoded_point = EncodedPoint::from_bytes(&sec1_bytes)
                                    .map_err(|_| Error::Data(None))?;
                                let public_key =
                                    p384::PublicKey::from_encoded_point(&encoded_point)
                                        .into_option()
                                        .ok_or(Error::Data(None))?;
                                Handle::P384PublicKey(public_key)
                            },
                            NAMED_CURVE_P521 => {
                                let mut sec1_bytes = vec![4u8];
                                sec1_bytes.extend_from_slice(&x);
                                sec1_bytes.extend_from_slice(&y);
                                let encoded_point = EncodedPoint::from_bytes(&sec1_bytes)
                                    .map_err(|_| Error::Data(None))?;
                                let public_key =
                                    p521::PublicKey::from_encoded_point(&encoded_point)
                                        .into_option()
                                        .ok_or(Error::Data(None))?;
                                Handle::P521PublicKey(public_key)
                            },
                            _ => unreachable!(),
                        };

                        // Step 2.9.4.3. Set the [[type]] internal slot of Key to "public".
                        // NOTE: CryptoKey is created in Step 2.11 - 2.14.
                        let key_type = KeyType::Public;

                        (handle, key_type)
                    }
                }
                // Otherwise
                else {
                    // Step 2.9.1. Perform any key import steps defined by other applicable
                    // specifications, passing format, jwk and obtaining key.
                    // Step 2.9.2. If an error occurred or there are no applicable specifications,
                    // throw a DataError.
                    // NOTE: We currently do not support other applicable specifications.
                    return Err(Error::Data(None));
                };

            // Step 2.10. If the key value is not a valid point on the Elliptic Curve identified by
            // the namedCurve member of normalizedAlgorithm throw a DataError.
            // NOTE: Done in Step 2.9.

            // Step 2.11. Let algorithm be a new instance of an EcKeyAlgorithm object.
            // Step 2.12. Set the name attribute of algorithm to "ECDSA".
            // Step 2.13. Set the namedCurve attribute of algorithm to namedCurve.
            // Step 2.14. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleEcKeyAlgorithm {
                name: ALG_ECDSA.to_string(),
                named_curve,
            };
            CryptoKey::new(
                cx,
                global,
                key_type,
                extractable,
                KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm),
                usages,
                handle,
            )
        },
        // If format is "raw":
        KeyFormat::Raw | KeyFormat::Raw_public => {
            // Step 2.1. If the namedCurve member of normalizedAlgorithm is not a named curve, then
            // throw a DataError.
            if !SUPPORTED_CURVES
                .iter()
                .any(|&supported_curve| supported_curve == normalized_algorithm.named_curve)
            {
                return Err(Error::Data(None));
            }

            // Step 2.2. If usages contains a value which is not "verify" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(None));
            }

            // Step 2.3.
            // If namedCurve is "P-256", "P-384" or "P-521":
            let handle = if matches!(
                normalized_algorithm.named_curve.as_str(),
                NAMED_CURVE_P256 | NAMED_CURVE_P384 | NAMED_CURVE_P521
            ) {
                // Step 2.3.1. Let Q be the elliptic curve point on the curve identified by the
                // namedCurve member of normalizedAlgorithm identified by performing the conversion
                // steps defined in Section 2.3.4 of [SEC1] on keyData.
                // Step 2.3.2. The uncompressed point format MUST be supported.
                // Step 2.3.3. If the implementation does not support the compressed point format
                // and a compressed point is provided, throw a DataError.
                // Step 2.3.4. If a decode error occurs or an identity point is found, throw a
                // DataError.
                match normalized_algorithm.named_curve.as_str() {
                    NAMED_CURVE_P256 => {
                        let q = p256::PublicKey::from_sec1_bytes(key_data)
                            .map_err(|_| Error::Data(None))?;
                        Handle::P256PublicKey(q)
                    },
                    NAMED_CURVE_P384 => {
                        let q = p384::PublicKey::from_sec1_bytes(key_data)
                            .map_err(|_| Error::Data(None))?;
                        Handle::P384PublicKey(q)
                    },
                    NAMED_CURVE_P521 => {
                        let q = p521::PublicKey::from_sec1_bytes(key_data)
                            .map_err(|_| Error::Data(None))?;
                        Handle::P521PublicKey(q)
                    },
                    _ => unreachable!(),
                }

                // Step 2.3.5. Let key be a new CryptoKey that represents Q.
                // NOTE: CryptoKey is created in Step 2.7 - 2.8.
            }
            // Otherwise:
            else {
                // Step. 2.3.1. Perform any key import steps defined by other applicable
                // specifications, passing format, keyData and obtaining key.
                // Step. 2.3.2. If an error occurred or there are no applicable specifications,
                // throw a DataError.
                // NOTE: We currently do not support other applicable specifications.
                return Err(Error::Data(None));
            };

            // Step 2.4. Let algorithm be a new EcKeyAlgorithm object.
            // Step 2.5. Set the name attribute of algorithm to "ECDSA".
            // Step 2.6. Set the namedCurve attribute of algorithm to equal the namedCurve member
            // of normalizedAlgorithm.
            let algorithm = SubtleEcKeyAlgorithm {
                name: ALG_ECDSA.to_string(),
                named_curve: normalized_algorithm.named_curve.clone(),
            };

            // Step 2.7. Set the [[type]] internal slot of key to "public"
            // Step 2.8. Set the [[algorithm]] internal slot of key to algorithm.
            CryptoKey::new(
                cx,
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm),
                usages,
                handle,
            )
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(None));
        },
    };

    // Step 3. Return key.
    Ok(key)
}

/// <https://w3c.github.io/webcrypto/#ecdsa-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    // Step 1. Let key be the CryptoKey to be exported.

    // Step 2. If the underlying cryptographic key material represented by the [[handle]] internal
    // slot of key cannot be accessed, then throw an OperationError.
    // NOTE: Done in Step 3.

    // Step 3.
    let result = match format {
        // If format is "spki":
        KeyFormat::Spki => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(None));
            }

            // Step 3.2.
            // Let data be an instance of the SubjectPublicKeyInfo ASN.1 structure defined in
            // [RFC5280] with the following properties:
            //     * Set the algorithm field to an AlgorithmIdentifier ASN.1 type with the
            //       following properties:
            //         * Set the algorithm field to the OID id-ecPublicKey defined in [RFC5480].
            //         * Set the parameters field to an instance of the ECParameters ASN.1 type
            //           defined in [RFC5480] as follows:
            //             If the namedCurve attribute of the [[algorithm]] internal slot of key is
            //             "P-256", "P-384" or "P-521":
            //                 Let keyData be the byte sequence that represents the Elliptic Curve
            //                 public key represented by the [[handle]] internal slot of key
            //                 according to the encoding rules specified in Section 2.2 of
            //                 [RFC5480] and using the uncompressed form. and keyData.
            //                     If the namedCurve attribute of the [[algorithm]] internal slot
            //                     of key is "P-256":
            //                         Set parameters to the namedCurve choice with value equal to
            //                         the object identifier secp256r1 defined in [RFC5480]
            //                     If the namedCurve attribute of the [[algorithm]] internal slot
            //                     of key is "P-384":
            //                         Set parameters to the namedCurve choice with value equal to
            //                         the object identifier secp384r1 defined in [RFC5480]
            //                     If the namedCurve attribute of the [[algorithm]] internal slot
            //                     of key is "P-521":
            //                         Set parameters to the namedCurve choice with value equal to
            //                         the object identifier secp521r1 defined in [RFC5480]
            //             Otherwise:
            //                 1. Perform any key export steps defined by other applicable
            //                    specifications, passing format and the namedCurve attribute of
            //                    the [[algorithm]] internal slot of key and obtaining
            //                    namedCurveOid and keyData.
            //                 2. Set parameters to the namedCurve choice with value equal to the
            //                    object identifier namedCurveOid.
            //     * Set the subjectPublicKey field to keyData.
            // NOTE: We currently do not support other applicable specifications.
            let data = match key.handle() {
                Handle::P256PublicKey(public_key) => public_key.to_public_key_der(),
                Handle::P384PublicKey(public_key) => public_key.to_public_key_der(),
                Handle::P521PublicKey(public_key) => public_key.to_public_key_der(),
                _ => return Err(Error::Operation(None)),
            }
            .map_err(|_| Error::Operation(None))?;

            // Step 3.3. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(data.to_vec())
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 3.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(None));
            }

            // Step 3.2.
            // Let data be an instance of the PrivateKeyInfo ASN.1 structure defined in [RFC5208]
            // with the following properties:
            //     * Set the version field to 0.
            //     * Set the privateKeyAlgorithm field to a PrivateKeyAlgorithmIdentifier ASN.1
            //       type with the following properties:
            //         * Set the algorithm field to the OID id-ecPublicKey defined in [RFC5480].
            //         * Set the parameters field to an instance of the ECParameters ASN.1 type
            //           defined in [RFC5480] as follows:
            //             If the namedCurve attribute of the [[algorithm]] internal slot of key is
            //             "P-256", "P-384" or "P-521":
            //                 Let keyData be the result of DER-encoding an instance of the
            //                 ECPrivateKey structure defined in Section 3 of [RFC5915] for the
            //                 Elliptic Curve private key represented by the [[handle]] internal
            //                 slot of key and that conforms to the following:
            //                     * The parameters field is present, and is equivalent to the
            //                       parameters field of the privateKeyAlgorithm field of this
            //                       PrivateKeyInfo ASN.1 structure.
            //                     * The publicKey field is present and represents the Elliptic
            //                       Curve public key associated with the Elliptic Curve private key
            //                       represented by the [[handle]] internal slot of key.
            //                     * If the namedCurve attribute of the [[algorithm]] internal slot
            //                       of key is "P-256":
            //                         Set parameters to the namedCurve choice with value equal to
            //                         the object identifier secp256r1 defined in [RFC5480]
            //                     * If the namedCurve attribute of the [[algorithm]] internal slot
            //                       of key is "P-384":
            //                         Set parameters to the namedCurve choice with value equal to
            //                         the object identifier secp384r1 defined in [RFC5480]
            //                     * If the namedCurve attribute of the [[algorithm]] internal slot
            //                       of key is "P-521":
            //                         Set parameters to the namedCurve choice with value equal to
            //                         the object identifier secp521r1 defined in [RFC5480]
            //             Otherwise:
            //                 1. Perform any key export steps defined by other applicable
            //                    specifications, passing format and the namedCurve attribute of
            //                    the [[algorithm]] internal slot of key and obtaining
            //                    namedCurveOid and keyData.
            //                 2. Set parameters to the namedCurve choice with value equal to the
            //                    object identifier namedCurveOid.
            //     * Set the privateKey field to keyData.
            // NOTE: We currently do not support other applicable specifications.
            let data = match key.handle() {
                Handle::P256PrivateKey(private_key) => private_key.to_pkcs8_der(),
                Handle::P384PrivateKey(private_key) => private_key.to_pkcs8_der(),
                Handle::P521PrivateKey(private_key) => private_key.to_pkcs8_der(),
                _ => return Err(Error::Operation(None)),
            }
            .map_err(|_| Error::Operation(None))?;

            // Step 3.3. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(data.as_bytes().to_vec())
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 3.1. Let jwk be a new JsonWebKey dictionary.
            // Step 3.2. Set the kty attribute of jwk to "EC".
            let mut jwk = JsonWebKey {
                kty: Some(DOMString::from("EC")),
                ..Default::default()
            };

            // Step 3.3.
            let KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm) = key.algorithm() else {
                return Err(Error::Operation(None));
            };
            // If the namedCurve attribute of the [[algorithm]] internal slot of key is "P-256",
            // "P-384" or "P-521":
            if matches!(
                algorithm.named_curve.as_str(),
                NAMED_CURVE_P256 | NAMED_CURVE_P384 | NAMED_CURVE_P521
            ) {
                // Step 3.3.1.
                // If the namedCurve attribute of the [[algorithm]] internal slot of key is
                // "P-256":
                //     Set the crv attribute of jwk to "P-256"
                // If the namedCurve attribute of the [[algorithm]] internal slot of key is
                // "P-384":
                //     Set the crv attribute of jwk to "P-384"
                // If the namedCurve attribute of the [[algorithm]] internal slot of key is
                // "P-521":
                //     Set the crv attribute of jwk to "P-521"
                jwk.crv = Some(DOMString::from(algorithm.named_curve.as_str()));

                // Step 3.3.2. Set the x attribute of jwk according to the definition in Section
                // 6.2.1.2 of JSON Web Algorithms [JWA].
                // Step 3.3.3. Set the y attribute of jwk according to the definition in Section
                // 6.2.1.3 of JSON Web Algorithms [JWA].
                let (x, y) = match key.handle() {
                    Handle::P256PublicKey(public_key) => {
                        let encoded_point = public_key.to_encoded_point(false);
                        (
                            encoded_point.x().ok_or(Error::Operation(None))?.to_vec(),
                            encoded_point.y().ok_or(Error::Operation(None))?.to_vec(),
                        )
                    },
                    Handle::P384PublicKey(public_key) => {
                        let encoded_point = public_key.to_encoded_point(false);
                        (
                            encoded_point.x().ok_or(Error::Operation(None))?.to_vec(),
                            encoded_point.y().ok_or(Error::Operation(None))?.to_vec(),
                        )
                    },
                    Handle::P521PublicKey(public_key) => {
                        let encoded_point = public_key.to_encoded_point(false);
                        (
                            encoded_point.x().ok_or(Error::Operation(None))?.to_vec(),
                            encoded_point.y().ok_or(Error::Operation(None))?.to_vec(),
                        )
                    },
                    Handle::P256PrivateKey(private_key) => {
                        let public_key = private_key.public_key();
                        let encoded_point = public_key.to_encoded_point(false);
                        (
                            encoded_point.x().ok_or(Error::Operation(None))?.to_vec(),
                            encoded_point.y().ok_or(Error::Operation(None))?.to_vec(),
                        )
                    },
                    Handle::P384PrivateKey(private_key) => {
                        let public_key = private_key.public_key();
                        let encoded_point = public_key.to_encoded_point(false);
                        (
                            encoded_point.x().ok_or(Error::Operation(None))?.to_vec(),
                            encoded_point.y().ok_or(Error::Operation(None))?.to_vec(),
                        )
                    },
                    Handle::P521PrivateKey(private_key) => {
                        let public_key = private_key.public_key();
                        let encoded_point = public_key.to_encoded_point(false);
                        (
                            encoded_point.x().ok_or(Error::Operation(None))?.to_vec(),
                            encoded_point.y().ok_or(Error::Operation(None))?.to_vec(),
                        )
                    },
                    _ => return Err(Error::Operation(None)),
                };
                jwk.encode_string_field(JwkStringField::X, &x);
                jwk.encode_string_field(JwkStringField::Y, &y);

                // Step 3.3.4.
                // If the [[type]] internal slot of key is "private"
                //     Set the d attribute of jwk according to the definition in Section 6.2.2.1 of
                //     JSON Web Algorithms [JWA].
                if key.Type() == KeyType::Private {
                    let d = match key.handle() {
                        Handle::P256PrivateKey(private_key) => private_key.to_bytes().to_vec(),
                        Handle::P384PrivateKey(private_key) => private_key.to_bytes().to_vec(),
                        Handle::P521PrivateKey(private_key) => private_key.to_bytes().to_vec(),
                        _ => return Err(Error::NotSupported(None)),
                    };
                    jwk.encode_string_field(JwkStringField::D, &d);
                }
            }
            // Otherwise:
            else {
                // Step 3.3.1. Perform any key export steps defined by other applicable
                // specifications, passing format and the namedCurve attribute of the [[algorithm]]
                // internal slot of key and obtaining namedCurve and a new value of jwk.
                // Step 3.3.2. Set the crv attribute of jwk to namedCurve.
                // NOTE: We currently do not support other applicable specifications.
                unreachable!()
            }

            // Step 3.4. Set the key_ops attribute of jwk to the usages attribute of key.
            jwk.set_key_ops(key.usages());

            // Step 3.4. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 3.4. Let result be jwk.
            ExportedKey::Jwk(Box::new(jwk))
        },
        // If format is "raw":
        KeyFormat::Raw | KeyFormat::Raw_public => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(None));
            }

            // Step 3.2.
            // If the namedCurve attribute of the [[algorithm]] internal slot of key is "P-256",
            // "P-384" or "P-521":
            //     Let data be a byte sequence representing the Elliptic Curve point Q represented
            //     by the [[handle]] internal slot of key according to [SEC1] 2.3.3 using the
            //     uncompressed format.
            // Otherwise:
            //     Perform any key export steps defined by other applicable specifications, passing
            //     format and the namedCurve attribute of the [[algorithm]] internal slot of key
            //     and obtaining namedCurve and data.
            //     NOTE: We currently do not support other applicable specifications.
            let named_curve =
                if let KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm) = key.algorithm() {
                    algorithm.named_curve.as_str()
                } else {
                    return Err(Error::Operation(None));
                };
            let data = if matches!(
                named_curve,
                NAMED_CURVE_P256 | NAMED_CURVE_P384 | NAMED_CURVE_P521
            ) {
                match key.handle() {
                    Handle::P256PublicKey(public_key) => public_key.to_sec1_bytes().to_vec(),
                    Handle::P384PublicKey(public_key) => public_key.to_sec1_bytes().to_vec(),
                    Handle::P521PublicKey(public_key) => public_key.to_sec1_bytes().to_vec(),
                    _ => return Err(Error::Operation(None)),
                }
            } else {
                return Err(Error::NotSupported(None));
            };

            // Step 3.3. Let result be data.
            ExportedKey::Bytes(data)
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(None));
        },
    };

    // Step 4. Return result.
    Ok(result)
}

/// A helper function that expand a prehash to a specified length if the prehash is shorter than
/// the specified length.
///
/// If the length of `prehash` is less than `length`, return a prehash with length `length`
/// constructed by left-padding `prehash` with zeros. Otherwire, return the unmodified `prehash`.
fn expand_prehash(prehash: Vec<u8>, length: usize) -> Vec<u8> {
    if prehash.len() < length {
        let mut left_padded_prehash = vec![0u8; length];
        left_padded_prehash[length - prehash.len()..].copy_from_slice(&prehash);
        left_padded_prehash
    } else {
        prehash
    }
}
