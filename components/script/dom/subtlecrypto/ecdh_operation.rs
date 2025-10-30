/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base64::prelude::*;
use elliptic_curve::sec1::{FromEncodedPoint, ValidatePublicKey};
use p256::NistP256;
use p384::NistP384;
use p521::NistP521;
use pkcs8::der::Decode;
use pkcs8::{AssociatedOid, ObjectIdentifier, PrivateKeyInfo, SubjectPublicKeyInfo};
use sec1::der::asn1::BitString;
use sec1::{EcPrivateKey, EncodedPoint};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{KeyType, KeyUsage};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{JsonWebKey, KeyFormat};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_ECDH, JsonWebKeyExt, KeyAlgorithmAndDerivatives, NAMED_CURVE_P256, NAMED_CURVE_P384,
    NAMED_CURVE_P521, SUPPORTED_CURVES, SubtleEcKeyAlgorithm, SubtleEcKeyImportParams,
};
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/webcrypto/#ecdh-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleEcKeyImportParams,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2.
    let key = match format {
        KeyFormat::Spki => {
            // Step 2.1. If usages is not empty then throw a SyntaxError.
            if !usages.is_empty() {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. Let spki be the result of running the parse a subjectPublicKeyInfo
            // algorithm over keyData
            // Step 2.3. If an error occurred while parsing, then throw a DataError.
            let spki = SubjectPublicKeyInfo::<_, BitString>::from_der(key_data)
                .map_err(|_| Error::Data)?;

            // Step 2.4. If the algorithm object identifier field of the algorithm
            // AlgorithmIdentifier field of spki is not equal to the id-ecPublicKey object
            // identifier defined in [RFC5480], then throw a DataError.
            if spki.algorithm.oid != elliptic_curve::ALGORITHM_OID {
                return Err(Error::Data);
            }

            // Step 2.5. If the parameters field of the algorithm AlgorithmIdentifier field of spki
            // is absent, then throw a DataError.
            // Step 2.6. Let params be the parameters field of the algorithm AlgorithmIdentifier
            // field of spki.
            // Step 2.7. If params is not an instance of the ECParameters ASN.1 type defined in
            // [RFC5480] that specifies a namedCurve, then throw a DataError.
            let Some(params): Option<ObjectIdentifier> = spki.algorithm.parameters else {
                return Err(Error::Data);
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
                NistP256::OID => Some(NAMED_CURVE_P256),
                NistP384::OID => Some(NAMED_CURVE_P384),
                NistP521::OID => Some(NAMED_CURVE_P521),
                _ => None,
            };

            // Step 2.10.
            let handle = match named_curve {
                // If namedCurve is not undefined:
                Some(curve) => {
                    // Step 2.10.1. Let publicKey be the Elliptic Curve public key identified by
                    // performing the conversion steps defined in Section 2.3.4 of [SEC1] to the
                    // subjectPublicKey field of spki.
                    // Step 2.10.2. The uncompressed point format MUST be supported.
                    // Step 2.10.3. If the implementation does not support the compressed point
                    // format and a compressed point is provided, throw a DataError.
                    // Step 2.10.4. If a decode error occurs or an identity point is found, throw a
                    // DataError.
                    let sec1_bytes = spki.subject_public_key.as_bytes().ok_or(Error::Data)?;
                    match curve {
                        NAMED_CURVE_P256 => {
                            let public_key = p256::PublicKey::from_sec1_bytes(sec1_bytes)
                                .map_err(|_| Error::Data)?;
                            Handle::P256PublicKey(public_key)
                        },
                        NAMED_CURVE_P384 => {
                            let public_key = p384::PublicKey::from_sec1_bytes(sec1_bytes)
                                .map_err(|_| Error::Data)?;
                            Handle::P384PublicKey(public_key)
                        },
                        NAMED_CURVE_P521 => {
                            let public_key = p521::PublicKey::from_sec1_bytes(sec1_bytes)
                                .map_err(|_| Error::Data)?;
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
                    // NOTE: We currently do not support applicable specifications.
                    return Err(Error::NotSupported);
                },
            };

            // Step 2.11. If namedCurve is defined, and not equal to the namedCurve member of
            // normalizedAlgorithm, throw a DataError.
            if named_curve.is_some_and(|curve| curve != normalized_algorithm.named_curve) {
                return Err(Error::Data);
            }

            // Step 2.12. If the key value is not a valid point on the Elliptic Curve identified by
            // the namedCurve member of normalizedAlgorithm throw a DataError.
            // NOTE: Done in Step 2.10.

            // Step 2.13. Set the [[type]] internal slot of key to "public"
            // Step 2.14. Let algorithm be a new EcKeyAlgorithm.
            // Step 2.15. Set the name attribute of algorithm to "ECDH".
            // Step 2.16. Set the namedCurve attribute of algorithm to namedCurve.
            // Step 2.17. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleEcKeyAlgorithm {
                name: ALG_ECDH.to_string(),
                named_curve: named_curve
                    .expect("named_curve must exist here")
                    .to_string(),
            };
            CryptoKey::new(
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm),
                usages,
                handle,
                can_gc,
            )
        },
        KeyFormat::Pkcs8 => {
            // Step 2.1. If usages contains an entry which is not "deriveKey" or "deriveBits" then
            // throw a SyntaxError.
            if usages
                .iter()
                .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
            {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. Let privateKeyInfo be the result of running the parse a privateKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurs while parsing, throw a DataError.
            let private_key_info = PrivateKeyInfo::from_der(key_data).map_err(|_| Error::Data)?;

            // Step 2.4. If the algorithm object identifier field of the privateKeyAlgorithm
            // PrivateKeyAlgorithm field of privateKeyInfo is not equal to the id-ecPublicKey
            // object identifier defined in [RFC5480], throw a DataError.
            if private_key_info.algorithm.oid != elliptic_curve::ALGORITHM_OID {
                return Err(Error::Data);
            }

            // Step 2.5. If the parameters field of the privateKeyAlgorithm
            // PrivateKeyAlgorithmIdentifier field of privateKeyInfo is not present, throw a
            // DataError.
            // Step 2.6. Let params be the parameters field of the privateKeyAlgorithm
            // PrivateKeyAlgorithmIdentifier field of privateKeyInfo.
            // Step 2.7. If params is not an instance of the ECParameters ASN.1 type defined in
            // [RFC5480] that specifies a namedCurve, then throw a DataError.
            let params: ObjectIdentifier =
                if let Some(params) = private_key_info.algorithm.parameters {
                    params.decode_as().map_err(|_| Error::Data)?
                } else {
                    return Err(Error::Data);
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
                NistP256::OID => Some(NAMED_CURVE_P256),
                NistP384::OID => Some(NAMED_CURVE_P384),
                NistP521::OID => Some(NAMED_CURVE_P521),
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
                        .map_err(|_| Error::Data)?;

                    // Step 2.10.3. If the parameters field of ecPrivateKey is present, and is not
                    // an instance of the namedCurve ASN.1 type defined in [RFC5480], or does not
                    // contain the same object identifier as the parameters field of the
                    // privateKeyAlgorithm PrivateKeyAlgorithmIdentifier field of privateKeyInfo,
                    // throw a DataError.
                    if ec_private_key.parameters.is_some_and(|parameters| {
                        parameters
                            .named_curve()
                            .is_none_or(|parameters| parameters != params)
                    }) {
                        return Err(Error::Data);
                    }

                    // Step 2.10.4. Let key be a new CryptoKey that represents the Elliptic Curve
                    // private key identified by performing the conversion steps defined in Section
                    // 3 of [RFC5915] using ecPrivateKey.
                    // NOTE: CryptoKey is created in Step 2.13 - 2.17.
                    match curve {
                        NAMED_CURVE_P256 => {
                            let private_key = p256::SecretKey::try_from(ec_private_key)
                                .map_err(|_| Error::Data)?;
                            Handle::P256PrivateKey(private_key)
                        },
                        NAMED_CURVE_P384 => {
                            let private_key = p384::SecretKey::try_from(ec_private_key)
                                .map_err(|_| Error::Data)?;
                            Handle::P384PrivateKey(private_key)
                        },
                        NAMED_CURVE_P521 => {
                            let private_key = p521::SecretKey::try_from(ec_private_key)
                                .map_err(|_| Error::Data)?;
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
                    // NOTE: We currently do not support applicable specifications.
                    return Err(Error::NotSupported);
                },
            };

            // Step 2.11. If namedCurve is defined, and not equal to the namedCurve member of
            // normalizedAlgorithm, throw a DataError.
            if named_curve.is_some_and(|curve| curve != normalized_algorithm.named_curve) {
                return Err(Error::Data);
            }

            // Step 2.12. If the key value is not a valid point on the Elliptic Curve identified by
            // the namedCurve member of normalizedAlgorithm throw a DataError.
            // NOTE: Done in Step 2.10.

            // Step 2.13. Set the [[type]] internal slot of key to "private".
            // Step 2.14. Let algorithm be a new EcKeyAlgorithm.
            // Step 2.15. Set the name attribute of algorithm to "ECDH".
            // Step 2.16. Set the namedCurve attribute of algorithm to namedCurve.
            // Step 2.17. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleEcKeyAlgorithm {
                name: ALG_ECDH.to_string(),
                named_curve: named_curve
                    .expect("named_curve must exist here")
                    .to_string(),
            };
            CryptoKey::new(
                global,
                KeyType::Private,
                extractable,
                KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm),
                usages,
                handle,
                can_gc,
            )
        },
        KeyFormat::Jwk => {
            // Step 2.1.
            // If keyData is a JsonWebKey dictionary:
            //     Let jwk equal keyData.
            // Otherwise:
            //     Throw a DataError.
            let jwk = JsonWebKey::parse(GlobalScope::get_cx(), key_data)?;

            // Step 2.2. If the d field is present and if usages contains an entry which is not
            // "deriveKey" or "deriveBits" then throw a SyntaxError.
            if jwk.d.as_ref().is_some() &&
                usages
                    .iter()
                    .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
            {
                return Err(Error::Syntax(None));
            }

            // Step 2.3. If the d field is not present and if usages is not empty then throw a
            // SyntaxError.
            if jwk.d.as_ref().is_none() && !usages.is_empty() {
                return Err(Error::Syntax(None));
            }

            // Step 2.4. If the kty field of jwk is not "EC", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "EC") {
                return Err(Error::Data);
            }

            // Step 2.5. If usages is non-empty and the use field of jwk is present and is not
            // equal to "enc" then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                return Err(Error::Data);
            }

            // Step 2.6. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK], or it does not contain all of the specified
            // usages values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.7. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data);
            }

            // Step 2.8. Let namedCurve be a string whose value is equal to the crv field of jwk.
            // Step 2.9. If namedCurve is not equal to the namedCurve member of
            // normalizedAlgorithm, throw a DataError.
            let named_curve = jwk
                .crv
                .filter(|crv| *crv == normalized_algorithm.named_curve)
                .map(|crv| crv.to_string())
                .ok_or(Error::Data)?;

            // Step 2.10.
            // If namedCurve is "P-256", "P-384" or "P-521":
            let (handle, key_type) = if matches!(
                named_curve.as_str(),
                NAMED_CURVE_P256 | NAMED_CURVE_P384 | NAMED_CURVE_P521
            ) {
                match jwk.d {
                    // If the d field is present:
                    Some(d) => {
                        // Step 2.10.1. If jwk does not meet the requirements of Section 6.2.2 of
                        // JSON Web Algorithms [JWA], then throw a DataError.
                        let x = match jwk.x {
                            Some(x) => base64::engine::general_purpose::URL_SAFE_NO_PAD
                                .decode(&*x.as_bytes())
                                .map_err(|_| Error::Data)?,
                            None => return Err(Error::Data),
                        };
                        let y = match jwk.y {
                            Some(y) => base64::engine::general_purpose::URL_SAFE_NO_PAD
                                .decode(&*y.as_bytes())
                                .map_err(|_| Error::Data)?,
                            None => return Err(Error::Data),
                        };
                        let d = base64::engine::general_purpose::URL_SAFE_NO_PAD
                            .decode(&*d.as_bytes())
                            .map_err(|_| Error::Data)?;

                        // Step 2.10.2. Let key be a new CryptoKey object that represents the
                        // Elliptic Curve private key identified by interpreting jwk according to
                        // Section 6.2.2 of JSON Web Algorithms [JWA].
                        // NOTE: CryptoKey is created in Step 2.12 - 2.15.
                        let handle = match named_curve.as_str() {
                            NAMED_CURVE_P256 => {
                                let private_key =
                                    p256::SecretKey::from_slice(&d).map_err(|_| Error::Data)?;
                                let mut sec1_bytes = vec![4u8];
                                sec1_bytes.extend_from_slice(&x);
                                sec1_bytes.extend_from_slice(&y);
                                let encoded_point = EncodedPoint::from_bytes(&sec1_bytes)
                                    .map_err(|_| Error::Data)?;
                                NistP256::validate_public_key(&private_key, &encoded_point)
                                    .map_err(|_| Error::Data)?;
                                Handle::P256PrivateKey(private_key)
                            },
                            NAMED_CURVE_P384 => {
                                let private_key =
                                    p384::SecretKey::from_slice(&d).map_err(|_| Error::Data)?;
                                let mut sec1_bytes = vec![4u8];
                                sec1_bytes.extend_from_slice(&x);
                                sec1_bytes.extend_from_slice(&y);
                                let encoded_point = EncodedPoint::from_bytes(&sec1_bytes)
                                    .map_err(|_| Error::Data)?;
                                NistP384::validate_public_key(&private_key, &encoded_point)
                                    .map_err(|_| Error::Data)?;
                                Handle::P384PrivateKey(private_key)
                            },
                            NAMED_CURVE_P521 => {
                                let private_key =
                                    p521::SecretKey::from_slice(&d).map_err(|_| Error::Data)?;
                                let mut sec1_bytes = vec![4u8];
                                sec1_bytes.extend_from_slice(&x);
                                sec1_bytes.extend_from_slice(&y);
                                let encoded_point = EncodedPoint::from_bytes(&sec1_bytes)
                                    .map_err(|_| Error::Data)?;
                                NistP521::validate_public_key(&private_key, &encoded_point)
                                    .map_err(|_| Error::Data)?;
                                Handle::P521PrivateKey(private_key)
                            },
                            _ => unreachable!(),
                        };

                        // Step 2.10.3. Set the [[type]] internal slot of Key to "private".
                        let key_type = KeyType::Private;

                        (handle, key_type)
                    },
                    // Otherwise:
                    None => {
                        // Step 2.10.1. If jwk does not meet the requirements of Section 6.2.1 of
                        // JSON Web Algorithms [JWA], then throw a DataError.
                        let x = match jwk.x {
                            Some(x) => base64::engine::general_purpose::URL_SAFE_NO_PAD
                                .decode(&*x.as_bytes())
                                .map_err(|_| Error::Data)?,
                            None => return Err(Error::Data),
                        };
                        let y = match jwk.y {
                            Some(y) => base64::engine::general_purpose::URL_SAFE_NO_PAD
                                .decode(&*y.as_bytes())
                                .map_err(|_| Error::Data)?,
                            None => return Err(Error::Data),
                        };

                        // Step 2.10.2. Let key be a new CryptoKey object that represents the
                        // Elliptic Curve public key identified by interpreting jwk according to
                        // Section 6.2.1 of JSON Web Algorithms [JWA].
                        // NOTE: CryptoKey is created in Step 2.12 - 2.15.
                        let handle = match named_curve.as_str() {
                            NAMED_CURVE_P256 => {
                                let mut sec1_bytes = vec![4u8];
                                sec1_bytes.extend_from_slice(&x);
                                sec1_bytes.extend_from_slice(&y);
                                let encoded_point = EncodedPoint::from_bytes(&sec1_bytes)
                                    .map_err(|_| Error::Data)?;
                                let public_key =
                                    p256::PublicKey::from_encoded_point(&encoded_point)
                                        .into_option()
                                        .ok_or(Error::Data)?;
                                Handle::P256PublicKey(public_key)
                            },
                            NAMED_CURVE_P384 => {
                                let mut sec1_bytes = vec![4u8];
                                sec1_bytes.extend_from_slice(&x);
                                sec1_bytes.extend_from_slice(&y);
                                let encoded_point = EncodedPoint::from_bytes(&sec1_bytes)
                                    .map_err(|_| Error::Data)?;
                                let public_key =
                                    p384::PublicKey::from_encoded_point(&encoded_point)
                                        .into_option()
                                        .ok_or(Error::Data)?;
                                Handle::P384PublicKey(public_key)
                            },
                            NAMED_CURVE_P521 => {
                                let mut sec1_bytes = vec![4u8];
                                sec1_bytes.extend_from_slice(&x);
                                sec1_bytes.extend_from_slice(&y);
                                let encoded_point = EncodedPoint::from_bytes(&sec1_bytes)
                                    .map_err(|_| Error::Data)?;
                                let public_key =
                                    p521::PublicKey::from_encoded_point(&encoded_point)
                                        .into_option()
                                        .ok_or(Error::Data)?;
                                Handle::P521PublicKey(public_key)
                            },
                            _ => unreachable!(),
                        };

                        // Step 2.10.3. Set the [[type]] internal slot of Key to "public".
                        let key_type = KeyType::Public;

                        (handle, key_type)
                    },
                }
            }
            // Otherwise
            else {
                // Step 2.10.1. Perform any key import steps defined by other applicable
                // specifications, passing format, jwk and obtaining key.
                // Step 2.10.2. If an error occurred or there are no applicable specifications,
                // throw a DataError.
                // NOTE: We currently do not support applicable specifications.
                return Err(Error::NotSupported);
            };

            // Step 2.11. If the key value is not a valid point on the Elliptic Curve identified by
            // the namedCurve member of normalizedAlgorithm throw a DataError.
            // NOTE: Done in Step 2.10.

            // Step 2.12. Let algorithm be a new instance of an EcKeyAlgorithm object.
            // Step 2.13. Set the name attribute of algorithm to "ECDH".
            // Step 2.14. Set the namedCurve attribute of algorithm to namedCurve.
            // Step 2.15. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleEcKeyAlgorithm {
                name: ALG_ECDH.to_string(),
                named_curve,
            };
            CryptoKey::new(
                global,
                key_type,
                extractable,
                KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm),
                usages,
                handle,
                can_gc,
            )
        },
        KeyFormat::Raw => {
            // Step 2.1. If the namedCurve member of normalizedAlgorithm is not a named curve, then
            // throw a DataError.
            if !SUPPORTED_CURVES
                .iter()
                .any(|&supported_curve| supported_curve == normalized_algorithm.named_curve)
            {
                return Err(Error::Data);
            }

            // Step 2.2. If usages is not the empty list, then throw a SyntaxError.
            if !usages.is_empty() {
                return Err(Error::Syntax(None));
            }

            // Step 2.3.
            // If namedCurve is "P-256", "P-384" or "P-521":
            let handle = if matches!(
                normalized_algorithm.named_curve.as_str(),
                NAMED_CURVE_P256 | NAMED_CURVE_P384 | NAMED_CURVE_P521
            ) {
                // Step 2.3.1. Let Q be the Elliptic Curve public key on the curve identified by
                // the namedCurve member of normalizedAlgorithm identified by performing the
                // conversion steps defined in Section 2.3.4 of [SEC1] to keyData.
                // Step 2.3.1. The uncompressed point format MUST be supported.
                // Step 2.3.1. If the implementation does not support the compressed point format
                // and a compressed point is provided, throw a DataError.
                // Step 2.3.1. If a decode error occurs or an identity point is found, throw a
                // DataError.
                match normalized_algorithm.named_curve.as_str() {
                    NAMED_CURVE_P256 => {
                        let q =
                            p256::PublicKey::from_sec1_bytes(key_data).map_err(|_| Error::Data)?;
                        Handle::P256PublicKey(q)
                    },
                    NAMED_CURVE_P384 => {
                        let q =
                            p384::PublicKey::from_sec1_bytes(key_data).map_err(|_| Error::Data)?;
                        Handle::P384PublicKey(q)
                    },
                    NAMED_CURVE_P521 => {
                        let q =
                            p521::PublicKey::from_sec1_bytes(key_data).map_err(|_| Error::Data)?;
                        Handle::P521PublicKey(q)
                    },
                    _ => unreachable!(),
                }

                // Step 2.3.1. Let key be a new CryptoKey that represents Q.
                // NOTE: CryptoKey is created in Step 2.7 - 2.8.
            }
            // Otherwise:
            else {
                // Step. 2.3.1. Perform any key import steps defined by other applicable
                // specifications, passing format, keyData and obtaining key.
                // Step. 2.3.2. If an error occured or there are no applicable specifications,
                // throw a DataError.
                // NOTE: We currently do not support applicable specifications.
                return Err(Error::NotSupported);
            };

            // Step 2.4. Let algorithm be a new EcKeyAlgorithm object.
            // Step 2.5. Set the name attribute of algorithm to "ECDH".
            // Step 2.6. Set the namedCurve attribute of algorithm to equal the namedCurve member
            // of normalizedAlgorithm.
            let algorithm = SubtleEcKeyAlgorithm {
                name: ALG_ECDH.to_string(),
                named_curve: normalized_algorithm.named_curve.clone(),
            };

            // Step 2.7. Set the [[type]] internal slot of key to "public"
            // Step 2.8. Set the [[algorithm]] internal slot of key to algorithm.
            CryptoKey::new(
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm),
                usages,
                handle,
                can_gc,
            )
        },
    };

    // Step 3. Return key.
    Ok(key)
}
