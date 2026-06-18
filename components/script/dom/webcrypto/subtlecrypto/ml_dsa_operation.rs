/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use ml_dsa::common::getrandom::SysRng;
use ml_dsa::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey};
use ml_dsa::{
    Generate, KeyExport, KeyInit, Keypair, MlDsa44, MlDsa65, MlDsa87, Signature, SignatureEncoding,
    SigningKey, VerifyingKey,
};

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
    CryptoAlgorithm, ExportedKey, JsonWebKeyExt, JwkStringField, KeyAlgorithmAndDerivatives,
    SubtleAlgorithm, SubtleContextParams, SubtleKeyAlgorithm,
};

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-dsa-operations-sign>
pub(crate) fn sign(
    normalized_algorithm: &SubtleContextParams,
    key: &CryptoKey,
    message: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the [[type]] internal slot of key is not "private", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Private {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"private\"".into(),
        )));
    }

    // Step 2. Let context be the context member of normalizedAlgorithm or the empty octet string
    // if the context member of normalizedAlgorithm is not present.
    let context = normalized_algorithm.context.as_deref().unwrap_or_default();

    // Step 3. Let result be the result of performing the ML-DSA.Sign signing algorithm, as
    // specified in Section 5.2 of [FIPS-204], with the parameter set indicated by the name member
    // of normalizedAlgorithm, using the ML-DSA private key associated with key as sk, message as M
    // and context as ctx.
    // Step 4. If the ML-DSA.Sign algorithm returned an error, return an OperationError.
    let result = match normalized_algorithm.name {
        CryptoAlgorithm::MlDsa44 => {
            let Handle::MlDsa44PrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-DSA-44 private key".into(),
                )));
            };
            private_key
                .expanded_key()
                .sign_randomized(message, context, &mut SysRng)
                .map_err(|_| Error::Operation(Some("ML-DSA-44 failed to sign the message".into())))?
                .to_vec()
        },
        CryptoAlgorithm::MlDsa65 => {
            let Handle::MlDsa65PrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-DSA-65 private key".into(),
                )));
            };
            private_key
                .expanded_key()
                .sign_randomized(message, context, &mut SysRng)
                .map_err(|_| Error::Operation(Some("ML-DSA-65 failed to sign the message".into())))?
                .to_vec()
        },
        CryptoAlgorithm::MlDsa87 => {
            let Handle::MlDsa87PrivateKey(private_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-DSA-87 private key".into(),
                )));
            };
            private_key
                .expanded_key()
                .sign_randomized(message, context, &mut SysRng)
                .map_err(|_| Error::Operation(Some("ML-DSA-87 failed to sign the message".into())))?
                .to_vec()
        },
        _ => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not an ML-DSA algorithm",
                normalized_algorithm.name.as_str()
            ))));
        },
    };

    // Step 5. Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-dsa-operations-verify>
pub(crate) fn verify(
    normalized_algorithm: &SubtleContextParams,
    key: &CryptoKey,
    message: &[u8],
    signature: &[u8],
) -> Result<bool, Error> {
    // Step 1. If the [[type]] internal slot of key is not "public", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Public {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"public\"".into(),
        )));
    }

    // Step 2. Let context be the context member of normalizedAlgorithm or the empty octet string
    // if the context member of normalizedAlgorithm is not present.
    let context = normalized_algorithm.context.as_deref().unwrap_or_default();

    // Step 3. Let result be the result of performing the ML-DSA.Verify verification algorithm, as
    // specified in Section 5.3 of [FIPS-204], with the parameter set indicated by the name member
    // of normalizedAlgorithm, using the ML-DSA public key associated with key as pk, message as M,
    // signature as σ and context as ctx.
    // Step 4. If the ML-DSA.Verify algorithm returned an error, return an OperationError.
    let result = match normalized_algorithm.name {
        CryptoAlgorithm::MlDsa44 => {
            let Handle::MlDsa44PublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-DSA-44 public key".into(),
                )));
            };
            match Signature::try_from(signature) {
                Ok(signature) => public_key.verify_with_context(message, context, &signature),
                Err(_) => false,
            }
        },
        CryptoAlgorithm::MlDsa65 => {
            let Handle::MlDsa65PublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-DSA-65 public key".into(),
                )));
            };
            match Signature::try_from(signature) {
                Ok(signature) => public_key.verify_with_context(message, context, &signature),
                Err(_) => false,
            }
        },
        CryptoAlgorithm::MlDsa87 => {
            let Handle::MlDsa87PublicKey(public_key) = key.handle() else {
                return Err(Error::Operation(Some(
                    "The key handle is not representing an ML-DSA-87 public key".into(),
                )));
            };
            match Signature::try_from(signature) {
                Ok(signature) => public_key.verify_with_context(message, context, &signature),
                Err(_) => false,
            }
        },
        _ => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not an ML-DSA algorithm",
                normalized_algorithm.name.as_str()
            ))));
        },
    };

    // Step 5. Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-dsa-operations-generate-key>
pub(crate) fn generate_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleAlgorithm,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<CryptoKeyPair, Error> {
    // Step 1. If usages contains a value which is not one of "sign" or "verify", then throw a
    // SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify))
    {
        return Err(Error::Syntax(Some(
            "Usages contains an entry which is not one of \"sign\" or \"verify\"".into(),
        )));
    }

    // Step 2. Generate an ML-DSA key pair, as described in Section 5.1 of [FIPS-204], with the
    // parameter set indicated by the name member of normalizedAlgorithm.
    // Step 3. If the key generation step fails, then throw an OperationError.
    let (private_key_handle, public_key_handle) = match normalized_algorithm.name {
        CryptoAlgorithm::MlDsa44 => {
            let signing_key = SigningKey::<MlDsa44>::generate();
            let verifying_key = signing_key.verifying_key();
            (
                Handle::MlDsa44PrivateKey(signing_key),
                Handle::MlDsa44PublicKey(verifying_key),
            )
        },
        CryptoAlgorithm::MlDsa65 => {
            let signing_key = SigningKey::<MlDsa65>::generate();
            let verifying_key = signing_key.verifying_key();
            (
                Handle::MlDsa65PrivateKey(signing_key),
                Handle::MlDsa65PublicKey(verifying_key),
            )
        },
        CryptoAlgorithm::MlDsa87 => {
            let signing_key = SigningKey::<MlDsa87>::generate();
            let verifying_key = signing_key.verifying_key();
            (
                Handle::MlDsa87PrivateKey(signing_key),
                Handle::MlDsa87PublicKey(verifying_key),
            )
        },
        name => {
            return Err(Error::NotSupported(Some(format!(
                "{} is not an ML-DSA algorithm",
                name.as_str()
            ))));
        },
    };

    // Step 4. Let algorithm be a new KeyAlgorithm object.
    // Step 5. Set the name attribute of algorithm to the name attribute of normalizedAlgorithm.
    let algorithm = SubtleKeyAlgorithm {
        name: normalized_algorithm.name,
    };

    // Step 6. Let publicKey be a new CryptoKey representing the public key of the generated key
    // pair.
    // Step 7. Set the [[type]] internal slot of publicKey to "public".
    // Step 8. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 9. Set the [[extractable]] internal slot of publicKey to true.
    // Step 10. Set the [[usages]] internal slot of publicKey to be the usage intersection of
    // usages and [ "verify" ].
    let public_key = CryptoKey::new(
        cx,
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
    );

    // Step 11. Let privateKey be a new CryptoKey representing the private key of the generated key
    // pair.
    // Step 12. Set the [[type]] internal slot of privateKey to "private".
    // Step 13. Set the [[algorithm]] internal slot of privateKey to algorithm.
    // Step 14. Set the [[extractable]] internal slot of privateKey to extractable.
    // Step 15. Set the [[usages]] internal slot of privateKey to be the usage intersection of
    // usages and [ "sign" ].
    let private_key = CryptoKey::new(
        cx,
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
            // Step 2.1. If usages contains a value which is not "verify" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"verify\"".into(),
                )));
            }

            // Step 2.2. Let spki be the result of running the parse a subjectPublicKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurred while parsing, then throw a DataError.
            // Step 2.4.
            // If the name member of normalizedAlgorithm is "ML-DSA-44":
            //     Let expectedOid be id-ml-dsa-44 (2.16.840.1.101.3.4.3.17).
            // If the name member of normalizedAlgorithm is "ML-DSA-65":
            //     Let expectedOid be id-ml-dsa-65 (2.16.840.1.101.3.4.3.18).
            // If the name member of normalizedAlgorithm is "ML-DSA-87":
            //     Let expectedOid be id-ml-dsa-87 (2.16.840.1.101.3.4.3.19).
            // Otherwise:
            //     throw a NotSupportedError.
            // Step 2.5. If the algorithm object identifier field of the algorithm
            // AlgorithmIdentifier field of spki is not equal to expectedOid, then throw a
            // DataError.
            // Step 2.6. If the parameters field of the algorithm AlgorithmIdentifier field of spki
            // is present, then throw a DataError.
            // Step 2.7. Let publicKey be the ML-DSA public key identified by the subjectPublicKey
            // field of spki.
            let public_key = match normalized_algorithm.name {
                CryptoAlgorithm::MlDsa44 => Handle::MlDsa44PublicKey(
                    VerifyingKey::from_public_key_der(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to decode the ML-DSA-44 public key from SPKI format".into(),
                        ))
                    })?,
                ),
                CryptoAlgorithm::MlDsa65 => Handle::MlDsa65PublicKey(
                    VerifyingKey::from_public_key_der(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to decode the ML-DSA-65 public key from SPKI format".into(),
                        ))
                    })?,
                ),
                CryptoAlgorithm::MlDsa87 => Handle::MlDsa87PublicKey(
                    VerifyingKey::from_public_key_der(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to decode the ML-DSA-87 pulic key from SPKI format".into(),
                        ))
                    })?,
                ),
                name => {
                    return Err(Error::NotSupported(Some(format!(
                        "{} is not an ML-DSA algorithm",
                        name.as_str()
                    ))));
                },
            };

            // Step 2.8. Let key be a new CryptoKey that represents publicKey.
            // Step 2.9. Set the [[type]] internal slot of key to "public"
            // Step 2.10. Let algorithm be a new KeyAlgorithm.
            // Step 2.11. Set the name attribute of algorithm to the name attribute of
            // normalizedAlgorithm.
            // Step 2.12. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name,
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
            // Step 2.1. If usages contains a value which is not "sign" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"sign\"".into(),
                )));
            }

            // Step 2.2. Let privateKeyInfo be the result of running the parse a privateKeyInfo
            // algorithm over keyData.
            // Step 2.3. If an error occurs while parsing, then throw a DataError.
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
            // Step 2.5. If the algorithm object identifier field of the privateKeyAlgorithm
            // PrivateKeyAlgorithm field of privateKeyInfo is not equal to expectedOid, then throw
            // a DataError.
            // Step 2.6. If the parameters field of the privateKeyAlgorithm
            // PrivateKeyAlgorithmIdentifier field of privateKeyInfo is present, then throw a
            // DataError.
            // Step 2.7. Let mlDsaPrivateKey be the result of performing the parse an ASN.1
            // structure algorithm, with data as the privateKey field of privateKeyInfo, structure
            // as asn1Structure, and exactData set to true.
            // Step 2.8. If an error occurred while parsing, then throw a DataError.
            // Step 2.9. If mlDsaPrivateKey represents an ML-DSA key in the expandedKey format, or
            // if mlDsaPrivateKey represents an ML-DSA key in the both format and the both format
            // is not supported, throw a NotSupportedError.
            // Step 2.10. If mlDsaPrivateKey represents an ML-DSA key in the both format, and the
            // seed field does not correspond to the expandedKey field, throw a DataError.
            //
            // NOTE: We do not support the `both` format.
            let ml_dsa_private_key = match normalized_algorithm.name {
                CryptoAlgorithm::MlDsa44 => Handle::MlDsa44PrivateKey(
                    SigningKey::from_pkcs8_der(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to decode the ML-DSA-44 private key from PKCS#8 format".into(),
                        ))
                    })?,
                ),
                CryptoAlgorithm::MlDsa65 => Handle::MlDsa65PrivateKey(
                    SigningKey::from_pkcs8_der(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to decode the ML-DSA-65 private key from PKCS#8 format".into(),
                        ))
                    })?,
                ),
                CryptoAlgorithm::MlDsa87 => Handle::MlDsa87PrivateKey(
                    SigningKey::from_pkcs8_der(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to decode the ML-DSA-87 private key from PKCS#8 format".into(),
                        ))
                    })?,
                ),
                name => {
                    return Err(Error::NotSupported(Some(format!(
                        "{} is not an ML-DSA algorithm",
                        name.as_str()
                    ))));
                },
            };

            // Step 2.11. Let key be a new CryptoKey that represents the ML-DSA private key
            // identified by mlDsaPrivateKey.
            // Step 2.12. Set the [[type]] internal slot of key to "private"
            // Step 2.13. Let algorithm be a new KeyAlgorithm.
            // Step 2.14. Set the name attribute of algorithm to the name attribute of
            // normalizedAlgorithm.
            // Step 2.15. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name,
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Private,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                ml_dsa_private_key,
            )
        },
        // If format is "raw-public":
        KeyFormat::Raw_public => {
            // Step 2.1. If usages contains a value which is not "verify" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"verify\"".into(),
                )));
            }

            // Step 2.2. Let algorithm be a new KeyAlgorithm object.
            // Step 2.3. Set the name attribute of algorithm to the name attribute of
            // normalizedAlgorithm.
            // Step 2.4. Let key be a new CryptoKey representing the key data provided in keyData.
            // Step 2.5. Set the [[type]] internal slot of key to "public"
            // Step 2.6. Set the [[algorithm]] internal slot of key to algorithm.
            let public_key = match normalized_algorithm.name {
                CryptoAlgorithm::MlDsa44 => {
                    let verifying_key = VerifyingKey::new_from_slice(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to parse the public ML-DSA-44 key in raw format".into(),
                        ))
                    })?;
                    Handle::MlDsa44PublicKey(verifying_key)
                },
                CryptoAlgorithm::MlDsa65 => {
                    let verifying_key = VerifyingKey::new_from_slice(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to parse the public ML-DSA-65 key in raw format".into(),
                        ))
                    })?;
                    Handle::MlDsa65PublicKey(verifying_key)
                },
                CryptoAlgorithm::MlDsa87 => {
                    let verifying_key = VerifyingKey::new_from_slice(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to parse the public ML-DSA-87 key in raw format".into(),
                        ))
                    })?;
                    Handle::MlDsa87PublicKey(verifying_key)
                },
                name => {
                    return Err(Error::NotSupported(Some(format!(
                        "{} is not an ML-DSA algorithm",
                        name.as_str()
                    ))));
                },
            };
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name,
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
        // If format is "raw-seed":
        KeyFormat::Raw_seed => {
            // Step 2.1. If usages contains an entry which is not "sign" then throw a SyntaxError.
            if usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"sign\"".into(),
                )));
            }

            // Step 2.2. Let data be keyData.
            // Step 2.3. If the length in bits of data is not 256 then throw a DataError.
            // Step 2.4. Let privateKey be the result of performing the ML-DSA.KeyGen_internal
            // function described in Section 6.1 of [FIPS-204] with the parameter set indicated by
            // the name member of normalizedAlgorithm, using data as ξ.
            let private_key = match normalized_algorithm.name {
                CryptoAlgorithm::MlDsa44 => {
                    let signing_key = SigningKey::new_from_slice(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to parse the private ML-DSA-44 key in raw format".into(),
                        ))
                    })?;
                    Handle::MlDsa44PrivateKey(signing_key)
                },
                CryptoAlgorithm::MlDsa65 => {
                    let signing_key = SigningKey::new_from_slice(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to parse the private ML-DSA-65 key in raw format".into(),
                        ))
                    })?;
                    Handle::MlDsa65PrivateKey(signing_key)
                },
                CryptoAlgorithm::MlDsa87 => {
                    let signing_key = SigningKey::new_from_slice(key_data).map_err(|_| {
                        Error::Data(Some(
                            "Failed to parse the private ML-DSA-87 key in raw format".into(),
                        ))
                    })?;
                    Handle::MlDsa87PrivateKey(signing_key)
                },
                name => {
                    return Err(Error::NotSupported(Some(format!(
                        "{} is not an ML-DSA algorithm",
                        name.as_str()
                    ))));
                },
            };

            // Step 2.5. Let key be a new CryptoKey that represents the ML-DSA private key
            // identified by privateKey.
            // Step 2.6. Set the [[type]] internal slot of key to "private"
            // Step 2.7. Let algorithm be a new KeyAlgorithm.
            // Step 2.8. Set the name attribute of algorithm to the name attribute of
            // normalizedAlgorithm.
            // Step 2.9. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name,
            };
            CryptoKey::new(
                cx,
                global,
                KeyType::Private,
                extractable,
                KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
                usages,
                private_key,
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

            // Step 2.2. If the priv field is present and usages contains a value which is not
            // "sign", or, if the priv field is not present and usages contains a value which is
            // not "verify" then throw a SyntaxError.
            if jwk.priv_.is_some() && usages.iter().any(|usage| *usage != KeyUsage::Sign) {
                return Err(Error::Syntax(Some(
                    "The priv field is present and usages contains a value which is not \"sign\""
                        .into(),
                )));
            }
            if jwk.priv_.is_none() && usages.iter().any(|usage| *usage != KeyUsage::Verify) {
                return Err(Error::Syntax(Some(
                    "The priv field is not present and usages contains a value which is not \
                        \"verify\""
                        .into(),
                )));
            }

            // Step 2.3. If the kty field of jwk is not "AKP", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "AKP") {
                return Err(Error::Data(Some(
                    "The kty field of jwk is not \"AKP\"".into(),
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
                        .into(),
                )));
            }

            // Step 2.5. If usages is non-empty and the use field of jwk is present and is not
            // equal to "sig", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                return Err(Error::Data(Some(
                    "Usages is non-empty and the use field of jwk is present and is not \
                    equal to \"sig\""
                        .into(),
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
                        .into(),
                )));
            }

            // Step 2.8.
            // If the priv field of jwk is present:
            let (key_type, key_handle) = if jwk.priv_.is_some() {
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
                // NOTE: The CryptoKey object is created in Step 2.9 - 2.11.
                let pub_bytes = jwk.decode_required_string_field(JwkStringField::Pub)?;
                let private_key_handle = match normalized_algorithm.name {
                    CryptoAlgorithm::MlDsa44 => {
                        let signing_key =
                            SigningKey::new_from_slice(&priv_bytes).map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the private ML-DSA-44 key in priv attribute"
                                        .into(),
                                ))
                            })?;
                        let verifying_key =
                            VerifyingKey::new_from_slice(&pub_bytes).map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the public ML-DSA-44 key in pub attribute"
                                        .into(),
                                ))
                            })?;
                        if signing_key.verifying_key() != verifying_key {
                            return Err(Error::Data(Some(
                                "The public key in pub attribute does not match \
                                    the private key in priv attribute"
                                    .into(),
                            )));
                        }
                        Handle::MlDsa44PrivateKey(signing_key)
                    },
                    CryptoAlgorithm::MlDsa65 => {
                        let signing_key =
                            SigningKey::new_from_slice(&priv_bytes).map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the private ML-DSA-65 key in priv attribute"
                                        .into(),
                                ))
                            })?;
                        let verifying_key =
                            VerifyingKey::new_from_slice(&pub_bytes).map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the public ML-DSA-65 key in pub attribute"
                                        .into(),
                                ))
                            })?;
                        if signing_key.verifying_key() != verifying_key {
                            return Err(Error::Data(Some(
                                "The public key in pub attribute does not match \
                                    the private key in priv attribute"
                                    .into(),
                            )));
                        }
                        Handle::MlDsa65PrivateKey(signing_key)
                    },
                    CryptoAlgorithm::MlDsa87 => {
                        let signing_key =
                            SigningKey::new_from_slice(&priv_bytes).map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the private ML-DSA-87 key in priv attribute"
                                        .into(),
                                ))
                            })?;
                        let verifying_key =
                            VerifyingKey::new_from_slice(&pub_bytes).map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the public ML-DSA-87 key in pub attribute"
                                        .into(),
                                ))
                            })?;
                        if signing_key.verifying_key() != verifying_key {
                            return Err(Error::Data(Some(
                                "The public key in pub attribute does not match \
                                    the private key in priv attribute"
                                    .into(),
                            )));
                        }
                        Handle::MlDsa87PrivateKey(signing_key)
                    },
                    name => {
                        return Err(Error::NotSupported(Some(format!(
                            "{} is not an ML-DSA algorithm",
                            name.as_str()
                        ))));
                    },
                };
                (KeyType::Private, private_key_handle)
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
                // NOTE: The CryptoKey object is created in Step 2.9 - 2.11.
                let public_key_handle = match normalized_algorithm.name {
                    CryptoAlgorithm::MlDsa44 => {
                        let verifying_key =
                            VerifyingKey::new_from_slice(&pub_bytes).map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the public ML-DSA-44 key in pub attribute"
                                        .into(),
                                ))
                            })?;
                        Handle::MlDsa44PublicKey(verifying_key)
                    },
                    CryptoAlgorithm::MlDsa65 => {
                        let verifying_key =
                            VerifyingKey::new_from_slice(&pub_bytes).map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the public ML-DSA-65 key in pub attribute"
                                        .into(),
                                ))
                            })?;
                        Handle::MlDsa65PublicKey(verifying_key)
                    },
                    CryptoAlgorithm::MlDsa87 => {
                        let verifying_key =
                            VerifyingKey::new_from_slice(&pub_bytes).map_err(|_| {
                                Error::Data(Some(
                                    "Failed to parse the public ML-DSA-87 key in pub attribute"
                                        .into(),
                                ))
                            })?;
                        Handle::MlDsa87PublicKey(verifying_key)
                    },
                    name => {
                        return Err(Error::NotSupported(Some(format!(
                            "{} is not an ML-DSA algorithm",
                            name.as_str()
                        ))));
                    },
                };
                (KeyType::Public, public_key_handle)
            };

            // Step 2.9. Let algorithm be a new instance of a KeyAlgorithm object.
            // Step 2.10. Set the name attribute of algorithm to the name member of
            // normalizedAlgorithm.
            // Step 2.11. Set the [[algorithm]] internal slot of key to algorithm.
            let algorithm = SubtleKeyAlgorithm {
                name: normalized_algorithm.name,
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
                "Unsupported import key format for ML-DSA key".into(),
            )));
        },
    };

    // Step 3. Return key.
    Ok(key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#ml-dsa-operations-export-key>
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
                    "[[type]] internal slot of key is not \"public\"".into(),
                )));
            }

            // Step 3.2. Let keyAlgorithm be the [[algorithm]] internal slot of key.
            let KeyAlgorithmAndDerivatives::KeyAlgorithm(key_algorithm) = key.algorithm() else {
                return Err(Error::Operation(Some(
                    "[[algorithm]] internal slot of key is not a KeyAlgorithm".into(),
                )));
            };

            // Step 3.3.
            // Let data be an instance of the SubjectPublicKeyInfo ASN.1 structure defined in
            // [RFC5280] with the following properties:
            //
            //     Set the algorithm field to an AlgorithmIdentifier ASN.1 type with the following
            //     properties:
            //
            //         If the name member of keyAlgorithm is "ML-DSA-44":
            //             Set the algorithm object identifier to the id-ml-dsa-44
            //             (2.16.840.1.101.3.4.3.17) OID.
            //
            //         If the name member of keyAlgorithm is "ML-DSA-65":
            //             Set the algorithm object identifier to the id-ml-dsa-65
            //             (2.16.840.1.101.3.4.3.18) OID.
            //
            //         If the name member of keyAlgorithm is "ML-DSA-87":
            //             Set the algorithm object identifier to the id-ml-dsa-87
            //             (2.16.840.1.101.3.4.3.19) OID.
            //
            //         Otherwise:
            //             throw a NotSupportedError.
            //
            //     Set the subjectPublicKey field to keyData.
            let data = match (key_algorithm.name, key.handle()) {
                (CryptoAlgorithm::MlDsa44, Handle::MlDsa44PublicKey(public_key)) => {
                    public_key.to_public_key_der().map_err(|_| {
                        Error::Operation(Some(
                            "Failed to encode the ML-DSA-44 public key into SPKI format".into(),
                        ))
                    })?
                },
                (CryptoAlgorithm::MlDsa65, Handle::MlDsa65PublicKey(public_key)) => {
                    public_key.to_public_key_der().map_err(|_| {
                        Error::Operation(Some(
                            "Failed to encode the ML-DSA-65 public key into SPKI format".into(),
                        ))
                    })?
                },
                (CryptoAlgorithm::MlDsa87, Handle::MlDsa87PublicKey(public_key)) => {
                    public_key.to_public_key_der().map_err(|_| {
                        Error::Operation(Some(
                            "Failed to encode the ML-DSA-87 public key into SPKI format".into(),
                        ))
                    })?
                },
                _ => {
                    return Err(Error::Operation(Some(
                        "The key handle is not representing an ML-DSA public key".into(),
                    )));
                },
            };

            // Step 3.4. Let result be the result of DER-encoding data.
            ExportedKey::new_bytes(data.into_vec())
        },
        KeyFormat::Pkcs8 => {
            // Step 3.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"private\"".into(),
                )));
            }

            // Step 3.2. Let keyAlgorithm be the [[algorithm]] internal slot of key.
            let KeyAlgorithmAndDerivatives::KeyAlgorithm(key_algorithm) = key.algorithm() else {
                return Err(Error::Operation(Some(
                    "[[algorithm]] internal slot of key is not a KeyAlgorithm".into(),
                )));
            };

            // Step 3.3.
            // Let data be an instance of the PrivateKeyInfo ASN.1 structure defined in [RFC5208]
            // with the following properties:
            //
            //     Set the version field to 0.
            //
            //     Set the privateKeyAlgorithm field to a PrivateKeyAlgorithmIdentifier ASN.1 type
            //     with the following properties:
            //
            //         If the name member of keyAlgorithm is "ML-DSA-44":
            //             Set the algorithm object identifier to the id-ml-dsa-44
            //             (2.16.840.1.101.3.4.3.17) OID.
            //
            //         If the name member of keyAlgorithm is "ML-DSA-65":
            //             Set the algorithm object identifier to the id-ml-dsa-65
            //             (2.16.840.1.101.3.4.3.18) OID.
            //
            //         If the name member of keyAlgorithm is "ML-DSA-87":
            //             Set the algorithm object identifier to the id-ml-dsa-87
            //             (2.16.840.1.101.3.4.3.19) OID.
            //
            //         Otherwise:
            //             throw a NotSupportedError.
            //
            //     Set the privateKey field as follows:
            //
            //         If the name member of keyAlgorithm is "ML-DSA-44":
            //             Set the privateKey field to the result of DER-encoding a
            //             ML-DSA-44-PrivateKey ASN.1 type that represents the ML-DSA private key
            //             seed represented by the [[handle]] internal slot of key using the
            //             seed-only format (using a context-specific [0] primitive tag with an
            //             implicit encoding of OCTET STRING).
            //
            //         If the name member of keyAlgorithm is "ML-DSA-65":
            //             Set the privateKey field to the result of DER-encoding a
            //             ML-DSA-65-PrivateKey ASN.1 type that represents the ML-DSA private key
            //             seed represented by the [[handle]] internal slot of key using the
            //             seed-only format (using a context-specific [0] primitive tag with an
            //             implicit encoding of OCTET STRING).
            //
            //         If the name member of keyAlgorithm is "ML-DSA-87":
            //             Set the privateKey field to the result of DER-encoding a
            //             ML-DSA-87-PrivateKey ASN.1 type that represents the ML-DSA private key
            //             seed represented by the [[handle]] internal slot of key using the
            //             seed-only format (using a context-specific [0] primitive tag with an
            //             implicit encoding of OCTET STRING).
            //
            //         Otherwise:
            //             throw a NotSupportedError.
            let private_key_info = match (key_algorithm.name, key.handle()) {
                (CryptoAlgorithm::MlDsa44, Handle::MlDsa44PrivateKey(private_key)) => {
                    private_key.to_pkcs8_der().map_err(|_| {
                        Error::Operation(Some(
                            "Failed to encode the ML-DSA-44 private key into PKCS#8 format".into(),
                        ))
                    })?
                },
                (CryptoAlgorithm::MlDsa65, Handle::MlDsa65PrivateKey(private_key)) => {
                    private_key.to_pkcs8_der().map_err(|_| {
                        Error::Operation(Some(
                            "Failed to encode the ML-DSA-65 private key into PKCS#8 format".into(),
                        ))
                    })?
                },
                (CryptoAlgorithm::MlDsa87, Handle::MlDsa87PrivateKey(private_key)) => {
                    private_key.to_pkcs8_der().map_err(|_| {
                        Error::Operation(Some(
                            "Failed to encode the ML-DSA-87 private key into PKCS#8 format".into(),
                        ))
                    })?
                },
                _ => {
                    return Err(Error::Operation(Some(
                        "The key handle is not representing an ML-DSA private key".into(),
                    )));
                },
            };

            // Step 3.4. Let result be the result of DER-encoding data.
            ExportedKey::Bytes(private_key_info.to_bytes())
        },
        // If format is "raw-public":
        KeyFormat::Raw_public => {
            // Step 3.1. If the [[type]] internal slot of key is not "public", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Public {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"public\"".into(),
                )));
            }

            // Step 3.2. Let data be a byte sequence containing the ML-DSA public key represented
            // by the [[handle]] internal slot of key.
            let data = match key.handle() {
                Handle::MlDsa44PublicKey(public_key) => public_key.to_bytes().as_slice().to_vec(),
                Handle::MlDsa65PublicKey(public_key) => public_key.to_bytes().as_slice().to_vec(),
                Handle::MlDsa87PublicKey(public_key) => public_key.to_bytes().as_slice().to_vec(),
                _ => {
                    return Err(Error::Operation(Some(
                        "The key handle is not representing an ML-DSA public key".into(),
                    )));
                },
            };

            // Step 3.2. Let result be data.
            ExportedKey::new_bytes(data)
        },
        // If format is "raw-seed":
        KeyFormat::Raw_seed => {
            // Step 3.1. If the [[type]] internal slot of key is not "private", then throw an
            // InvalidAccessError.
            if key.Type() != KeyType::Private {
                return Err(Error::InvalidAccess(Some(
                    "[[type]] internal slot of key is not \"private\"".into(),
                )));
            }

            // Step 3.2. Let data be a byte sequence containing the ξ seed variable of the key
            // represented by the [[handle]] internal slot of key.
            let data = match key.handle() {
                Handle::MlDsa44PrivateKey(private_key) => private_key.as_seed().as_slice().to_vec(),
                Handle::MlDsa65PrivateKey(private_key) => private_key.as_seed().as_slice().to_vec(),
                Handle::MlDsa87PrivateKey(private_key) => private_key.as_seed().as_slice().to_vec(),
                _ => {
                    return Err(Error::Operation(Some(
                        "The key handle is not representing an ML-DSA private key".into(),
                    )));
                },
            };

            // Step 3.3. Let result be data.
            ExportedKey::new_bytes(data)
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 3.1. Let jwk be a new JsonWebKey dictionary.
            let mut jwk = JsonWebKey::default();

            // Step 3.2.  Let keyAlgorithm be the [[algorithm]] internal slot of key.
            let KeyAlgorithmAndDerivatives::KeyAlgorithm(key_algorithm) = key.algorithm() else {
                return Err(Error::Operation(Some(
                    "[[algorithm]] internal slot of key is not a KeyAlgorithm".into(),
                )));
            };

            // Step 3.3. Set the kty attribute of jwk to "AKP".
            jwk.kty = Some(DOMString::from("AKP"));

            // Step 3.4. Set the alg attribute of jwk to the name member of normalizedAlgorithm.
            jwk.alg = Some(DOMString::from(key_algorithm.name.as_str()));

            // Step 3.5. Set the pub attribute of jwk to the base64url encoded public key
            // corresponding to the [[handle]] internal slot of key.
            // Step 3.6.
            // If the [[type]] internal slot of key is "private":
            //     Set the priv attribute of jwk to the base64url encoded seed represented by the
            //     [[handle]] internal slot of key.
            if key.Type() == KeyType::Private {
                match key.handle() {
                    Handle::MlDsa44PrivateKey(private_key) => {
                        jwk.encode_string_field(
                            JwkStringField::Priv,
                            private_key.as_seed().as_slice(),
                        );
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            private_key.as_ref().to_bytes().as_slice(),
                        );
                    },
                    Handle::MlDsa65PrivateKey(private_key) => {
                        jwk.encode_string_field(
                            JwkStringField::Priv,
                            private_key.as_seed().as_slice(),
                        );
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            private_key.as_ref().to_bytes().as_slice(),
                        );
                    },
                    Handle::MlDsa87PrivateKey(private_key) => {
                        jwk.encode_string_field(
                            JwkStringField::Priv,
                            private_key.as_seed().as_slice(),
                        );
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            private_key.as_ref().to_bytes().as_slice(),
                        );
                    },
                    _ => {
                        return Err(Error::Operation(Some(
                            "The key handle is not representing an ML-DSA private key".into(),
                        )));
                    },
                }
            } else {
                match key.handle() {
                    Handle::MlDsa44PublicKey(public_key) => {
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            public_key.to_bytes().as_slice(),
                        );
                    },
                    Handle::MlDsa65PublicKey(public_key) => {
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            public_key.to_bytes().as_slice(),
                        );
                    },
                    Handle::MlDsa87PublicKey(public_key) => {
                        jwk.encode_string_field(
                            JwkStringField::Pub,
                            public_key.to_bytes().as_slice(),
                        );
                    },
                    _ => {
                        return Err(Error::Operation(Some(
                            "The key handle is not representing an ML-DSA public key".into(),
                        )));
                    },
                };
            }

            // Step 3.7. Set the key_ops attribute of jwk to the usages attribute of key.
            jwk.set_key_ops(&key.usages());

            // Step 3.8. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 3.9. Let result be jwk.
            ExportedKey::new_jwk(jwk)
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unsupported export key format for ML-DSA key".into(),
            )));
        },
    };

    // Step 3. Return result.
    Ok(result)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-getPublicKey>
/// Step 9 - 15, for ML-DSA
pub(crate) fn get_public_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    key: &CryptoKey,
    algorithm: &KeyAlgorithmAndDerivatives,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 9. If usages contains an entry which is not supported for a public key by the algorithm
    // identified by algorithm, then throw a SyntaxError.
    //
    // NOTE: See "importKey" operation for supported usages
    if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
        return Err(Error::Syntax(Some(
            "Usages contains an entry which is not \"verify\"".into(),
        )));
    }

    // Step 10. Let publicKey be a new CryptoKey representing the public key corresponding to the
    // private key represented by the [[handle]] internal slot of key.
    // Step 11. If an error occurred, then throw a OperationError.
    // Step 12. Set the [[type]] internal slot of publicKey to "public".
    // Step 13. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 14. Set the [[extractable]] internal slot of publicKey to true.
    // Step 15. Set the [[usages]] internal slot of publicKey to usages.
    let public_key_handle = match key.handle() {
        Handle::MlDsa44PrivateKey(private_key) => {
            Handle::MlDsa44PublicKey(private_key.verifying_key())
        },
        Handle::MlDsa65PrivateKey(private_key) => {
            Handle::MlDsa65PublicKey(private_key.verifying_key())
        },
        Handle::MlDsa87PrivateKey(private_key) => {
            Handle::MlDsa87PublicKey(private_key.verifying_key())
        },
        _ => {
            return Err(Error::Operation(Some(
                "[[handle]] internal slot of key is not an ML-DSA private key".into(),
            )));
        },
    };
    let public_key = CryptoKey::new(
        cx,
        global,
        KeyType::Public,
        true,
        algorithm.clone(),
        usages,
        public_key_handle,
    );

    Ok(public_key)
}
