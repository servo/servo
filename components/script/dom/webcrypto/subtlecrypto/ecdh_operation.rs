/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use elliptic_curve::Curve;
use elliptic_curve::generic_array::typenum::Unsigned;
use elliptic_curve::sec1::ToEncodedPoint;
use js::context::JSContext;
use p256::NistP256;
use p384::NistP384;
use p521::NistP521;
use pkcs8::EncodePrivateKey;
use pkcs8::spki::EncodePublicKey;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, CryptoKeyPair, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{JsonWebKey, KeyFormat};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::ec_common::EcAlgorithm;
use crate::dom::subtlecrypto::{
    ExportedKey, JsonWebKeyExt, JwkStringField, KeyAlgorithmAndDerivatives, NAMED_CURVE_P256,
    NAMED_CURVE_P384, NAMED_CURVE_P521, SubtleEcKeyGenParams, SubtleEcKeyImportParams,
    SubtleEcdhKeyDeriveParams, ec_common,
};

/// <https://w3c.github.io/webcrypto/#ecdh-operations-generate-key>
pub(crate) fn generate_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleEcKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<CryptoKeyPair, Error> {
    ec_common::generate_key(
        EcAlgorithm::Ecdh,
        cx,
        global,
        normalized_algorithm,
        extractable,
        usages,
    )
}

/// <https://w3c.github.io/webcrypto/#ecdh-operations-derive-bits>
pub(crate) fn derive_bits(
    normalized_algorithm: &SubtleEcdhKeyDeriveParams,
    key: &CryptoKey,
    length: Option<u32>,
) -> Result<Vec<u8>, Error> {
    // Step 1. If the [[type]] internal slot of key is not "private", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Private {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"private\"".to_string(),
        )));
    }

    // Step 2. Let publicKey be the public member of normalizedAlgorithm.
    let public_key = normalized_algorithm.public.root();

    // Step 3. If the [[type]] internal slot of publicKey is not "public", then throw an
    // InvalidAccessError.
    if public_key.Type() != KeyType::Public {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"public\"".to_string(),
        )));
    }

    // Step 4. If the name attribute of the [[algorithm]] internal slot of publicKey is not equal
    // to the name property of the [[algorithm]] internal slot of key, then throw an
    // InvalidAccessError.
    if public_key.algorithm().name() != key.algorithm().name() {
        return Err(Error::InvalidAccess(Some(
            "public key [[algorithm]] internal slot name does not match that of private key"
                .to_string(),
        )));
    }

    // Step 5. If the namedCurve attribute of the [[algorithm]] internal slot of publicKey is not
    // equal to the namedCurve property of the [[algorithm]] internal slot of key, then throw an
    // InvalidAccessError.
    let (
        KeyAlgorithmAndDerivatives::EcKeyAlgorithm(public_key_algorithm),
        KeyAlgorithmAndDerivatives::EcKeyAlgorithm(key_algorithm),
    ) = (public_key.algorithm(), key.algorithm())
    else {
        return Err(Error::Operation(Some("Public or private key's [[algorithm]] internal slot is not an elliptic curve algorithm".to_string())));
    };
    if public_key_algorithm.named_curve != key_algorithm.named_curve {
        return Err(Error::InvalidAccess(Some(
            "Public and private keys' [[algorithm]] internal slots namedCurves do not match"
                .to_string(),
        )));
    }

    // Step 6.
    // If the namedCurve property of the [[algorithm]] internal slot of key is "P-256", "P-384" or "P-521":
    //     Step 6.1. Perform the ECDH primitive specified in [RFC6090] Section 4 with key as the EC
    //     private key d and the EC public key represented by the [[handle]] internal slot of
    //     publicKey as the EC public key.
    //
    //     Step 6.2. Let secret be a byte sequence containing the result of applying the field
    //     element to octet string conversion defined in Section 6.2 of [RFC6090] to the output of
    //     the ECDH primitive.
    //
    // If the namedCurve property of the [[algorithm]] internal slot of key is a value specified in
    // an applicable specification that specifies the use of that value with ECDH:
    //     Perform the ECDH derivation steps specified in that specification, passing in key and
    //     publicKey and resulting in secret.
    //
    // Otherwise:
    //     throw a NotSupportedError
    //
    // Step 7. If performing the operation results in an error, then throw a OperationError.
    let secret = match key_algorithm.named_curve.as_str() {
        NAMED_CURVE_P256 => {
            let Handle::P256PrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "Private key is not a P-256 private key".to_string(),
                )));
            };
            let Handle::P256PublicKey(public_key) = public_key.handle() else {
                return Err(Error::Operation(Some(
                    "Public key is not a P-256 public key".to_string(),
                )));
            };
            p256::ecdh::diffie_hellman(private_key.to_nonzero_scalar(), public_key.as_affine())
                .raw_secret_bytes()
                .to_vec()
        },
        NAMED_CURVE_P384 => {
            let Handle::P384PrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "Private key is not a P-384 private key".to_string(),
                )));
            };
            let Handle::P384PublicKey(public_key) = public_key.handle() else {
                return Err(Error::Operation(Some(
                    "Public key is not a P384 public key".to_string(),
                )));
            };
            p384::ecdh::diffie_hellman(private_key.to_nonzero_scalar(), public_key.as_affine())
                .raw_secret_bytes()
                .to_vec()
        },
        NAMED_CURVE_P521 => {
            let Handle::P521PrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "Private key is not a P-521 private key".to_string(),
                )));
            };
            let Handle::P521PublicKey(public_key) = public_key.handle() else {
                return Err(Error::Operation(Some(
                    "Public key is not a P-521 public key".to_string(),
                )));
            };
            p521::ecdh::diffie_hellman(private_key.to_nonzero_scalar(), public_key.as_affine())
                .raw_secret_bytes()
                .to_vec()
        },
        _ => {
            return Err(Error::NotSupported(Some(format!(
                "Unsupported namedCurve: {}",
                key_algorithm.named_curve
            ))));
        },
    };

    // Step 8.
    // If length is null:
    //     Return secret
    // Otherwise:
    //     If the length in bits of secret is less than length:
    //         throw an OperationError.
    //     Otherwise:
    //         Return a byte sequence containing the first length bits of secret.
    match length {
        None => Ok(secret),
        Some(length) => {
            if secret.len() * 8 < length as usize {
                Err(Error::Operation(Some(
                    "Derived secret is too short".to_string(),
                )))
            } else {
                let mut secret = secret[..length.div_ceil(8) as usize].to_vec();
                if length % 8 != 0 {
                    // Clean excess bits in last byte of secret.
                    let mask = u8::MAX << (8 - length % 8);
                    if let Some(last_byte) = secret.last_mut() {
                        *last_byte &= mask;
                    }
                }
                Ok(secret)
            }
        },
    }
}

/// <https://w3c.github.io/webcrypto/#ecdh-operations-import-key>
pub(crate) fn import_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleEcKeyImportParams,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    ec_common::import_key(
        EcAlgorithm::Ecdh,
        cx,
        global,
        normalized_algorithm,
        format,
        key_data,
        extractable,
        usages,
    )
}

/// <https://w3c.github.io/webcrypto/#ecdh-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    // Step 1. Let key be the CryptoKey to be exported.

    // Step 2. If the underlying cryptographic key material represented by the [[handle]] internal
    // slot of key cannot be accessed, then throw an OperationError.
    // NOTE: Done in Step 3.

    // Step 3.
    let create_public_key_export_error =
        || Error::Operation(Some("Failed to export public key".to_string()));
    let create_private_key_export_error =
        || Error::Operation(Some("Failed to export private key".to_string()));
    let result = match format {
        KeyFormat::Spki => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not public".to_string(),
                )));
            }

            // Step 3.2.
            // Let data be an instance of the SubjectPublicKeyInfo ASN.1 structure defined in
            // [RFC5280] with the following properties:
            //     * Set the algorithm field to an AlgorithmIdentifier ASN.1 type with the
            //     following properties:
            //         * Set the algorithm field to the OID id-ecPublicKey defined in [RFC5480].
            //         * Set the parameters field to an instance of the ECParameters ASN.1 type
            //         defined in [RFC5480] as follows:
            //             If the namedCurve attribute of the [[algorithm]] internal slot of key is
            //             "P-256", "P-384" or "P-521":
            //                 Let keyData be the byte sequence that represents the Elliptic Curve
            //                 public key represented by the [[handle]] internal slot of key
            //                 according to the encoding rules specified in Section 2.3.3 of [SEC1]
            //                 and using the uncompressed form.
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
            //     * Set the subjectPublicKey field to keyData
            let data = match key.handle() {
                Handle::P256PublicKey(public_key) => public_key.to_public_key_der(),
                Handle::P384PublicKey(public_key) => public_key.to_public_key_der(),
                Handle::P521PublicKey(public_key) => public_key.to_public_key_der(),
                _ => return Err(Error::Operation(None)),
            }
            .map_err(|_| create_public_key_export_error())?;

            ExportedKey::new_bytes(data.to_vec())
        },
        KeyFormat::Pkcs8 => {
            // Step 3.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not private".to_string(),
                )));
            }

            // Step 3.2.
            // Let data be an instance of the PrivateKeyInfo ASN.1 structure defined in [RFC5208]
            // with the following properties:
            //     * Set the version field to 0.
            //     * Set the privateKeyAlgorithm field to a PrivateKeyAlgorithmIdentifier ASN.1
            //     type with the following properties:
            //         * Set the algorithm field to the OID id-ecPublicKey defined in [RFC5480].
            //         * Set the parameters field to an instance of the ECParameters ASN.1 type
            //         defined in [RFC5480] as follows:
            //             If the namedCurve attribute of the [[algorithm]] internal slot of key is
            //             "P-256", "P-384" or "P-521":
            //                 Let keyData be the result of DER-encoding an instance of the
            //                 ECPrivateKey structure defined in Section 3 of [RFC5915] for the
            //                 Elliptic Curve private key represented by the [[handle]] internal
            //                 slot of key and that conforms to the following:
            //                     * The parameters field is present, and is equivalent to the
            //                     parameters field of the privateKeyAlgorithm field of this
            //                     PrivateKeyInfo ASN.1 structure.
            //                     * The publicKey field is present and represents the Elliptic
            //                     Curve public key associated with the Elliptic Curve private key
            //                     represented by the [[handle]] internal slot of key.
            //                     * If the namedCurve attribute of the [[algorithm]] internal slot
            //                     of key is "P-256":
            //                         Set parameters to the namedCurve choice with value equal to
            //                         the object identifier secp256r1 defined in [RFC5480]
            //                     * If the namedCurve attribute of the [[algorithm]] internal slot
            //                     of key is "P-384":
            //                         Set parameters to the namedCurve choice with value equal to
            //                         the object identifier secp384r1 defined in [RFC5480]
            //                     * If the namedCurve attribute of the [[algorithm]] internal slot
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
            //     * Set the privateKey field to keyData.
            let data = match key.handle() {
                Handle::P256PrivateKey(private_key) => private_key.to_pkcs8_der(),
                Handle::P384PrivateKey(private_key) => private_key.to_pkcs8_der(),
                Handle::P521PrivateKey(private_key) => private_key.to_pkcs8_der(),
                _ => return Err(Error::Operation(None)),
            }
            .map_err(|_| create_private_key_export_error())?;

            ExportedKey::new_bytes(data.as_bytes().to_vec())
        },
        KeyFormat::Jwk => {
            // Step 3.1. Let jwk be a new JsonWebKey dictionary.
            let mut jwk = JsonWebKey::default();

            // Step 3.2. Set the kty attribute of jwk to "EC".
            jwk.kty = Some(DOMString::from("EC"));

            // Step 3.3.
            let named_curve =
                if let KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm) = key.algorithm() {
                    algorithm.named_curve.as_str()
                } else {
                    return Err(Error::Operation(Some(
                        "key is not an elliptic curve algorithm key".to_string(),
                    )));
                };
            // If the namedCurve attribute of the [[algorithm]] internal slot of key is "P-256",
            // "P-384" or "P-521":
            if matches!(
                named_curve,
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
                jwk.crv = Some(DOMString::from(named_curve));

                // Step 3.3.2. Set the x attribute of jwk according to the definition in Section
                // 6.2.1.2 of JSON Web Algorithms [JWA].
                // Step 3.3.3. Set the y attribute of jwk according to the definition in Section
                // 6.2.1.3 of JSON Web Algorithms [JWA].
                let (x, y) = match key.handle() {
                    Handle::P256PublicKey(public_key) => {
                        let encoded_point = public_key.to_encoded_point(false);
                        (
                            encoded_point
                                .x()
                                .ok_or(create_public_key_export_error())?
                                .to_vec(),
                            encoded_point
                                .y()
                                .ok_or(create_public_key_export_error())?
                                .to_vec(),
                        )
                    },
                    Handle::P384PublicKey(public_key) => {
                        let encoded_point = public_key.to_encoded_point(false);
                        (
                            encoded_point
                                .x()
                                .ok_or(create_public_key_export_error())?
                                .to_vec(),
                            encoded_point
                                .y()
                                .ok_or(create_public_key_export_error())?
                                .to_vec(),
                        )
                    },
                    Handle::P521PublicKey(public_key) => {
                        let encoded_point = public_key.to_encoded_point(false);
                        (
                            encoded_point
                                .x()
                                .ok_or(create_public_key_export_error())?
                                .to_vec(),
                            encoded_point
                                .y()
                                .ok_or(create_public_key_export_error())?
                                .to_vec(),
                        )
                    },
                    Handle::P256PrivateKey(private_key) => {
                        let public_key = private_key.public_key();
                        let encoded_point = public_key.to_encoded_point(false);
                        (
                            encoded_point
                                .x()
                                .ok_or(create_private_key_export_error())?
                                .to_vec(),
                            encoded_point
                                .y()
                                .ok_or(create_private_key_export_error())?
                                .to_vec(),
                        )
                    },
                    Handle::P384PrivateKey(private_key) => {
                        let public_key = private_key.public_key();
                        let encoded_point = public_key.to_encoded_point(false);
                        (
                            encoded_point
                                .x()
                                .ok_or(create_private_key_export_error())?
                                .to_vec(),
                            encoded_point
                                .y()
                                .ok_or(create_private_key_export_error())?
                                .to_vec(),
                        )
                    },
                    Handle::P521PrivateKey(private_key) => {
                        let public_key = private_key.public_key();
                        let encoded_point = public_key.to_encoded_point(false);
                        (
                            encoded_point
                                .x()
                                .ok_or(create_private_key_export_error())?
                                .to_vec(),
                            encoded_point
                                .y()
                                .ok_or(create_private_key_export_error())?
                                .to_vec(),
                        )
                    },
                    _ => {
                        return Err(Error::NotSupported(Some(
                            "Unsupported key type".to_string(),
                        )));
                    },
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
                        _ => {
                            return Err(Error::NotSupported(Some(
                                "Unsupported key type".to_string(),
                            )));
                        },
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
                // NOTE: We currently do not support applicable specifications.
            }

            // Step 3.4. Set the key_ops attribute of jwk to the usages attribute of key.
            jwk.set_key_ops(key.usages());

            // Step 3.4. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 3.4. Let result be jwk.
            ExportedKey::new_jwk(jwk)
        },
        KeyFormat::Raw | KeyFormat::Raw_public => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not public".to_string(),
                )));
            }

            // Step 3.2.
            // If the namedCurve attribute of the [[algorithm]] internal slot of key is "P-256",
            // "P-384" or "P-521":
            //     Let data be the byte sequence that represents the Elliptic Curve public key
            //     represented by the [[handle]] internal slot of key according to the encoding
            //     rules specified in Section 2.3.3 of [SEC1] and using the uncompressed form.
            // Otherwise:
            //     Perform any key export steps defined by other applicable specifications, passing
            //     format and the namedCurve attribute of the [[algorithm]] internal slot of key
            //     and obtaining namedCurve and data.
            //     NOTE: We currently do not support applicable specifications.
            let named_curve =
                if let KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm) = key.algorithm() {
                    algorithm.named_curve.as_str()
                } else {
                    return Err(Error::Operation(Some(
                        "key is not an elliptic curve algorithm key".to_string(),
                    )));
                };
            let data = if matches!(
                named_curve,
                NAMED_CURVE_P256 | NAMED_CURVE_P384 | NAMED_CURVE_P521
            ) {
                match key.handle() {
                    Handle::P256PublicKey(public_key) => public_key.to_sec1_bytes().to_vec(),
                    Handle::P384PublicKey(public_key) => public_key.to_sec1_bytes().to_vec(),
                    Handle::P521PublicKey(public_key) => public_key.to_sec1_bytes().to_vec(),
                    _ => {
                        return Err(Error::Operation(Some(
                            "Failed to export public key".to_string(),
                        )));
                    },
                }
            } else {
                return Err(Error::NotSupported(Some(
                    "Unsupported namedCurve".to_string(),
                )));
            };

            // Step 3.3. Let result be data.
            ExportedKey::new_bytes(data)
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported key format".to_string(),
            )));
        },
    };

    // Step 4. Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-getPublicKey>
/// Step 9 - 15, for ECDH
pub(crate) fn get_public_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    key: &CryptoKey,
    algorithm: &KeyAlgorithmAndDerivatives,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    ec_common::get_public_key(cx, global, key, algorithm, usages)
}

/// Given a normalizedAlgorithm (an EcdhKeyDeriveParams dictionary), return the length of the secret
/// derived by the named curve specified by the `named_curve` member of the `[[algorithm]]` slot of
/// the `public` member of normalizedAlgorithm.
pub(crate) fn secret_length(
    normalized_algorithm: &SubtleEcdhKeyDeriveParams,
) -> Result<u32, Error> {
    let public_key = normalized_algorithm.public.root();
    let KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm) = public_key.algorithm() else {
        return Err(Error::Operation(Some(
            "The key is not an elliptic curve algorithm key".to_string(),
        )));
    };

    let secret_length_in_bits = match algorithm.named_curve.as_str() {
        NAMED_CURVE_P256 => <NistP256 as Curve>::FieldBytesSize::to_u32(),
        NAMED_CURVE_P384 => <NistP384 as Curve>::FieldBytesSize::to_u32(),
        NAMED_CURVE_P521 => <NistP521 as Curve>::FieldBytesSize::to_u32(),
        named_curve => {
            return Err(Error::NotSupported(Some(format!(
                "Unsupported namedCurve: {}",
                named_curve
            ))));
        },
    };

    Ok(secret_length_in_bits)
}
