/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aws_lc_rs::encoding::{AsBigEndian, AsDer};
use aws_lc_rs::signature::{ED25519, Ed25519KeyPair, KeyPair, ParsedPublicKey, UnparsedPublicKey};
use base64ct::{Base64UrlUnpadded, Encoding};
use rand::TryRngCore;
use rand::rngs::OsRng;

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
    ALG_ED25519, ExportedKey, JsonWebKeyExt, KeyAlgorithmAndDerivatives, SubtleKeyAlgorithm,
};
use crate::script_runtime::CanGc;

const ED25519_SEED_LENGTH: usize = 32;

/// <https://w3c.github.io/webcrypto/#ed25519-operations-sign>
pub(crate) fn sign(key: &CryptoKey, message: &[u8]) -> Result<Vec<u8>, Error> {
    // Step 1. If the [[type]] internal slot of key is not "private", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Private {
        return Err(Error::InvalidAccess(None));
    }

    // Step 2. Let result be the result of performing the Ed25519 signing process, as specified in
    // [RFC8032], Section 5.1.6, with message as M, using the Ed25519 private key associated with
    // key.
    let key_pair = Ed25519KeyPair::from_seed_unchecked(key.handle().as_bytes())
        .map_err(|_| Error::Operation(None))?;
    let result = key_pair.sign(message).as_ref().to_vec();

    // Step 3. Return result.
    Ok(result)
}

/// <https://w3c.github.io/webcrypto/#ed25519-operations-verify>
pub(crate) fn verify(key: &CryptoKey, message: &[u8], signature: &[u8]) -> Result<bool, Error> {
    // Step 1. If the [[type]] internal slot of key is not "public", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Public {
        return Err(Error::InvalidAccess(None));
    }

    // Step 2. If the key data of key represents an invalid point or a small-order element on the
    // Elliptic Curve of Ed25519, return false.
    // NOTE: Not all implementations perform this check. See WICG/webcrypto-secure-curves issue 27.

    // Step 3. If the point R, encoded in the first half of signature, represents an invalid point
    // or a small-order element on the Elliptic Curve of Ed25519, return false.
    // NOTE: Not all implementations perform this check. See WICG/webcrypto-secure-curves issue 27.

    // Step 4. Perform the Ed25519 verification steps, as specified in [RFC8032], Section
    // 5.1.7, using the cofactorless (unbatched) equation, [S]B = R + [k]A', on the signature, with
    // message as M, using the Ed25519 public key associated with key.
    // Step 5. Let result be a boolean with the value true if the signature is valid and the value
    // false otherwise.
    let public_key = UnparsedPublicKey::new(&ED25519, key.handle().as_bytes());
    let result = match public_key.verify(message, signature) {
        Ok(()) => true,
        Err(aws_lc_rs::error::Unspecified) => false,
    };

    // Step 6. Return result.
    Ok(result)
}

/// <https://w3c.github.io/webcrypto/#ed25519-operations-generate-key>
pub(crate) fn generate_key(
    global: &GlobalScope,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<CryptoKeyPair, Error> {
    // Step 1. If usages contains any entry which is not "sign" or "verify", then throw a
    // SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify))
    {
        return Err(Error::Syntax(None));
    }

    // Step 2. Generate an Ed25519 key pair, as defined in [RFC8032], section 5.1.5.
    let mut seed = vec![0u8; ED25519_SEED_LENGTH];
    if OsRng.try_fill_bytes(&mut seed).is_err() {
        return Err(Error::Operation(None));
    }
    let key_pair =
        Ed25519KeyPair::from_seed_unchecked(&seed).map_err(|_| Error::Operation(None))?;

    // Step 3. Let algorithm be a new KeyAlgorithm object.
    // Step 4. Set the name attribute of algorithm to "Ed25519".
    let algorithm = SubtleKeyAlgorithm {
        name: ALG_ED25519.to_string(),
    };

    // Step 5. Let publicKey be a new CryptoKey representing the public key of the generated key pair.
    // Step 6. Set the [[type]] internal slot of publicKey to "public"
    // Step 7. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 8. Set the [[extractable]] internal slot of publicKey to true.
    // Step 9. Set the [[usages]] internal slot of publicKey to be the usage intersection of usages
    // and [ "verify" ].
    let public_key = CryptoKey::new(
        global,
        KeyType::Public,
        true,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm.clone()),
        usages
            .iter()
            .filter(|&usage| *usage == KeyUsage::Verify)
            .cloned()
            .collect(),
        Handle::Ed25519(key_pair.public_key().as_ref().to_vec()),
        can_gc,
    );

    // Step 10. Let privateKey be a new CryptoKey representing the private key of the generated key pair.
    // Step 11. Set the [[type]] internal slot of privateKey to "private"
    // Step 12. Set the [[algorithm]] internal slot of privateKey to algorithm.
    // Step 13. Set the [[extractable]] internal slot of privateKey to extractable.
    // Step 14. Set the [[usages]] internal slot of privateKey to be the usage intersection of
    // usages and [ "sign" ].
    let private_key = CryptoKey::new(
        global,
        KeyType::Private,
        extractable,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
        usages
            .iter()
            .filter(|&usage| *usage == KeyUsage::Sign)
            .cloned()
            .collect(),
        Handle::Ed25519(seed),
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

/// <https://w3c.github.io/webcrypto/#ed25519-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.
    // NOTE: It is given as a method parameter.

    // Step 2.
    let key = match format {
        // If format is "spki":
        KeyFormat::Spki => {
            // Step 2.1. If usages contains a value which is not "verify" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. Let spki be the result of running the parse a subjectPublicKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurred while parsing, then throw a DataError.
            // Step 2.4. If the algorithm object identifier field of the algorithm
            // AlgorithmIdentifier field of spki is not equal to the id-Ed25519 object identifier
            // defined in [RFC8410], then throw a DataError.
            // Step 2.5. If the parameters field of the algorithm AlgorithmIdentifier field of spki
            // is present, then throw a DataError.
            // Step 2.6. Let publicKey be the Ed25519 public key identified by the subjectPublicKey
            // field of spki.
            let public_key =
                ParsedPublicKey::new(&ED25519, key_data).map_err(|_| Error::Data(None))?;

            // Step 2.9. Let algorithm be a new KeyAlgorithm.
            // Step 2.10. Set the name attribute of algorithm to "Ed25519".
            let algorithm = SubtleKeyAlgorithm {
                name: ALG_ED25519.to_string(),
            };

            // Step 2.7. Let key be a new CryptoKey that represents publicKey.
            // Step 2.8. Set the [[type]] internal slot of key to "public"
            // Step 2.11. Set the [[algorithm]] internal slot of key to algorithm.
            CryptoKey::new(
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                Handle::Ed25519(public_key.as_ref().to_vec()),
                can_gc,
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
            // Step 2.3. If an error occurs while parsing, then throw a DataError.
            // Step 2.4. If the algorithm object identifier field of the privateKeyAlgorithm
            // PrivateKeyAlgorithm field of privateKeyInfo is not equal to the id-Ed25519 object
            // identifier defined in [RFC8410], then throw a DataError.
            // Step 2.5. If the parameters field of the privateKeyAlgorithm
            // PrivateKeyAlgorithmIdentifier field of privateKeyInfo is present, then throw a
            // DataError.
            let private_key_info =
                Ed25519KeyPair::from_pkcs8(key_data).map_err(|_| Error::Data(None))?;

            // Step 2.6. Let curvePrivateKey be the result of performing the parse an ASN.1
            // structure algorithm, with data as the privateKey field of privateKeyInfo, structure
            // as the ASN.1 CurvePrivateKey structure specified in Section 7 of [RFC8410], and
            // exactData set to true.
            // Step 2.7. If an error occurred while parsing, then throw a DataError.
            let curve_private_key = private_key_info
                .seed()
                .map_err(|_| Error::Data(None))?
                .as_be_bytes()
                .map_err(|_| Error::Data(None))?
                .as_ref()
                .to_vec();

            // Step 2.10. Let algorithm be a new KeyAlgorithm.
            // Step 2.11. Set the name attribute of algorithm to "Ed25519".
            let algorithm = SubtleKeyAlgorithm {
                name: ALG_ED25519.to_string(),
            };

            // Step 2.8. Let key be a new CryptoKey that represents the Ed25519 private key
            // identified by curvePrivateKey.
            // Step 2.9. Set the [[type]] internal slot of key to "private"
            // Step 2.12. Set the [[algorithm]] internal slot of key to algorithm.
            CryptoKey::new(
                global,
                KeyType::Private,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                Handle::Ed25519(curve_private_key),
                can_gc,
            )
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1. If keyData is a JsonWebKey dictionary: Let jwk equal keyData.
            // Otherwise: Throw a DataError.
            let cx = GlobalScope::get_cx();
            let jwk = JsonWebKey::parse(cx, key_data)?;

            // Step 2.2 If the d field is present and usages contains a value which is not "sign",
            // or, if the d field is not present and usages contains a value which is not "verify"
            // then throw a SyntaxError.
            if (jwk.d.as_ref().is_some() && usages.iter().any(|usage| *usage != KeyUsage::Sign)) ||
                (jwk.d.as_ref().is_none() &&
                    usages.iter().any(|usage| *usage != KeyUsage::Verify))
            {
                return Err(Error::Syntax(None));
            }

            // Step 2.3 If the kty field of jwk is not "OKP", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "OKP") {
                return Err(Error::Data(None));
            }

            // Step 2.4 If the crv field of jwk is not "Ed25519", then throw a DataError.
            if jwk.crv.as_ref().is_none_or(|crv| crv != ALG_ED25519) {
                return Err(Error::Data(None));
            }

            // Step 2.5 If the alg field of jwk is present and is not "Ed25519" or "EdDSA", then
            // throw a DataError.
            if jwk
                .alg
                .as_ref()
                .is_some_and(|alg| !matches!(alg.str().as_ref(), ALG_ED25519 | "EdDSA"))
            {
                return Err(Error::Data(None));
            }

            // Step 2.6 If usages is non-empty and the use field of jwk is present and is not
            // "sig", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "sig") {
                return Err(Error::Data(None));
            }

            // Step 2.7 If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK], or it does not contain all of the specified
            // usages values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.8 If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.as_ref().is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data(None));
            }

            // Step 2.10. Let algorithm be a new instance of a KeyAlgorithm object.
            // Step 2.11. Set the name attribute of algorithm to "Ed25519".
            let algorithm = SubtleKeyAlgorithm {
                name: ALG_ED25519.to_string(),
            };

            // Step 2.9
            match jwk.d {
                // If the d field is present:
                Some(d) => {
                    // Step 2.9.1. If jwk does not meet the requirements of the JWK private key
                    // format described in Section 2 of [RFC8037], then throw a DataError.
                    let x = jwk.x.ok_or(Error::Data(None))?;

                    // Step 2.9.2. Let key be a new CryptoKey object that represents the Ed25519
                    // private key identified by interpreting jwk according to Section
                    // 2 of [RFC8037]
                    // Step 2.9.3. Set the [[type]] internal slot of Key to "private".
                    let public_key_bytes =
                        Base64UrlUnpadded::decode_vec(&x.str()).map_err(|_| Error::Data(None))?;
                    let private_key_bytes =
                        Base64UrlUnpadded::decode_vec(&d.str()).map_err(|_| Error::Data(None))?;
                    let _ = Ed25519KeyPair::from_seed_and_public_key(
                        &private_key_bytes,
                        &public_key_bytes,
                    )
                    .map_err(|_| Error::Data(None))?;
                    CryptoKey::new(
                        global,
                        KeyType::Private,
                        extractable,
                        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                        usages,
                        Handle::Ed25519(private_key_bytes),
                        can_gc,
                    )
                },
                // Otherwise:
                None => {
                    // Step 2.9.1. If jwk does not meet the requirements of the JWK public key
                    // format described in Section 2 of [RFC8037], then throw a DataError.
                    let x = jwk.x.ok_or(Error::Data(None))?;

                    // Step 2.9.2. Let key be a new CryptoKey object that represents the Ed25519
                    // public key identified by interpreting jwk according to Section 2 of
                    // [RFC8037].
                    // Step 2.9.3. Set the [[type]] internal slot of Key to "public".
                    let public_key_bytes =
                        Base64UrlUnpadded::decode_vec(&x.str()).map_err(|_| Error::Data(None))?;
                    CryptoKey::new(
                        global,
                        KeyType::Public,
                        extractable,
                        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                        usages,
                        Handle::Ed25519(public_key_bytes),
                        can_gc,
                    )
                },
            }

            // Step 2.12. Set the [[algorithm]] internal slot of key to algorithm.
            // NOTE: Done in Step 2.9
        },
        // If format is "raw":
        KeyFormat::Raw => {
            // Step 2.1. If usages contains a value which is not "verify" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(None));
            }

            // Step 2.2. If the length in bits of keyData is not 256 then throw a DataError.
            if key_data.len() * 8 != 256 {
                return Err(Error::Data(None));
            }

            // Step 2.3. Let algorithm be a new KeyAlgorithm object.
            // Step 2.4. Set the name attribute of algorithm to "Ed25519".
            let algorithm = SubtleKeyAlgorithm {
                name: ALG_ED25519.to_string(),
            };

            // Step 2.5. Let key be a new CryptoKey representing the key data provided in keyData.
            // Step 2.6. Set the [[type]] internal slot of key to "public"
            // Step 2.7. Set the [[algorithm]] internal slot of key to algorithm.
            CryptoKey::new(
                global,
                KeyType::Public,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                Handle::Ed25519(key_data.to_vec()),
                can_gc,
            )
        },
        // Otherwise: throw a NotSupportedError. (Unreachable)
    };

    // Step 3. Return key
    Ok(key)
}

/// <https://w3c.github.io/webcrypto/#ed25519-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    // Step 1. Let key be the CryptoKey to be exported.
    // NOTE: It is given as a method parameter.

    // Step 2. If the underlying cryptographic key material represented by the [[handle]] internal
    // slot of key cannot be accessed, then throw an OperationError.
    // NOTE: key.handle() guarantees access.
    let key_data = key.handle().as_bytes();

    // Step 3.
    let result = match format {
        // If format is "spki":
        KeyFormat::Spki => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(None));
            }

            // Step 3.2. Let data be an instance of the SubjectPublicKeyInfo ASN.1 structure
            // defined in [RFC5280] with the following properties:
            //     Set the algorithm field to an AlgorithmIdentifier ASN.1 type with the following
            //     properties:
            //         Set the algorithm object identifier to the id-Ed25519 OID defined in
            //         [RFC8410].
            //     Set the subjectPublicKey field to keyData.
            let data =
                ParsedPublicKey::new(&ED25519, key_data).map_err(|_| Error::Operation(None))?;

            // Step 3.3. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(
                data.as_der()
                    .map_err(|_| Error::Operation(None))?
                    .as_ref()
                    .to_vec(),
            )
        },
        // If format is "pkcs8":
        KeyFormat::Pkcs8 => {
            // Step 3.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(None));
            }

            // Step 3.2. Let data be an instance of the PrivateKeyInfo ASN.1 structure defined in
            // [RFC5208] with the following properties:
            //     Set the version field to 0.
            //     Set the privateKeyAlgorithm field to a PrivateKeyAlgorithmIdentifier ASN.1 type
            //     with the following properties:
            //         Set the algorithm object identifier to the id-Ed25519 OID defined in
            //         [RFC8410].
            //     Set the privateKey field to the result of DER-encoding a CurvePrivateKey ASN.1
            //     type, as defined in Section 7 of [RFC8410], that represents the Ed25519 private
            //     key represented by the [[handle]] internal slot of key
            let data = Ed25519KeyPair::from_seed_unchecked(key_data)
                .map_err(|_| Error::Operation(None))?
                .to_pkcs8v1()
                .map_err(|_| Error::Operation(None))?;

            // Step 3.3. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(data.as_ref().to_vec())
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 3.5. Set the x attribute of jwk according to the definition in Section 2 of [RFC8037].
            // Step 3.6.
            // If the [[type]] internal slot of key is "private"
            //     Set the d attribute of jwk according to the definition in Section 2 of [RFC8037].
            let (x, d) = match key.Type() {
                KeyType::Public => {
                    let public_key = Base64UrlUnpadded::encode_string(key_data);
                    (Some(DOMString::from(public_key)), None)
                },
                KeyType::Private => {
                    let key_pair = Ed25519KeyPair::from_seed_unchecked(key_data)
                        .map_err(|_| Error::Data(None))?;
                    let public_key =
                        Base64UrlUnpadded::encode_string(key_pair.public_key().as_ref());
                    let private_key = Base64UrlUnpadded::encode_string(key_data);
                    (
                        Some(DOMString::from(public_key)),
                        Some(DOMString::from(private_key)),
                    )
                },
                KeyType::Secret => {
                    return Err(Error::Data(None));
                },
            };

            // Step 3.7. Set the key_ops attribute of jwk to the usages attribute of key.
            let key_ops = Some(
                key.usages()
                    .iter()
                    .map(|usage| DOMString::from(usage.as_str()))
                    .collect::<Vec<DOMString>>(),
            );

            // Step 3.1. Let jwk be a new JsonWebKey dictionary.
            // Step 3.2. Set the kty attribute of jwk to "OKP".
            // Step 3.3. Set the alg attribute of jwk to "Ed25519".
            // Step 3.4. Set the crv attribute of jwk to "Ed25519".
            // Step 3.8. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            let jwk = JsonWebKey {
                kty: Some(DOMString::from("OKP")),
                alg: Some(DOMString::from(ALG_ED25519)),
                crv: Some(DOMString::from(ALG_ED25519)),
                x,
                d,
                key_ops,
                ext: Some(key.Extractable()),
                ..Default::default()
            };

            // Step 9. Let result be jwk.
            ExportedKey::Jwk(Box::new(jwk))
        },
        // If format is "raw":
        KeyFormat::Raw => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(None));
            }

            // Step 3.2. Let data be a byte sequence representing the Ed25519 public key
            // represented by the [[handle]] internal slot of key.
            // Step 3.3. Let result be data.
            ExportedKey::Bytes(key_data.to_vec())
        },
        // Otherwise: throw a NotSupportedError. (Unreachable)
    };

    // Step 4. Return result.
    Ok(result)
}
