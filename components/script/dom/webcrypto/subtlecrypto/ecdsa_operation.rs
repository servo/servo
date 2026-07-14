/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ecdsa::signature::hazmat::{PrehashVerifier, RandomizedPrehashSigner};
use ecdsa::{Signature, SigningKey, VerifyingKey};
use js::context::JSContext;
use p256::NistP256;
use p384::NistP384;
use p521::NistP521;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, CryptoKeyPair, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyFormat;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::ec_common::EcAlgorithm;
use crate::dom::subtlecrypto::{
    ExportedKey, KeyAlgorithmAndDerivatives, NAMED_CURVE_P256, NAMED_CURVE_P384, NAMED_CURVE_P521,
    SubtleEcKeyGenParams, SubtleEcKeyImportParams, SubtleEcdsaParams, ec_common,
};

/// <https://w3c.github.io/webcrypto/#ecdsa-operations-sign>
pub(crate) fn sign(
    normalized_algorithm: &SubtleEcdsaParams,
    key: &CryptoKey,
    message: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the [[type]] internal slot of key is not "private", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Private {
        return Err(Error::InvalidAccess(Some(
            "The key type is not private.".into(),
        )));
    }

    // Step 2. Let hashAlgorithm be the hash member of normalizedAlgorithm.
    let hash_algorithm = &normalized_algorithm.hash;

    // Step 3. Let M be the result of performing the digest operation specified by hashAlgorithm
    // using message.
    let m = hash_algorithm.digest(message)?;

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
        return Err(Error::Operation(Some(
            "Key algorithm is not a elliptic curve key algorithm.".into(),
        )));
    };
    let mut rng = rand::rng();
    let result = match algorithm.named_curve.as_str() {
        NAMED_CURVE_P256 => {
            let Handle::P256PrivateKey(d) = key.handle() else {
                return Err(Error::Operation(Some(
                    "Key handle is not a P256PrivateKey.".into(),
                )));
            };
            let signing_key = SigningKey::<NistP256>::from(d);
            let signature: Signature<NistP256> = signing_key
                .sign_prehash_with_rng(&mut rng, &m)
                .map_err(|_| Error::Operation(Some("ECDSA signing process failed.".into())))?9
            signature.to_vec()
        },
        NAMED_CURVE_P384 => {
            let Handle::P384PrivateKey(d) = key.handle() else {
                return Err(Error::Operation(Some(
                    "Key handle is not a P384PrivateKey.".into(),
                )));
            };
            let signing_key = SigningKey::<NistP384>::from(d);
            let signature: Signature<NistP384> = signing_key
                .sign_prehash_with_rng(&mut rng, &m)
                .map_err(|_| Error::Operation(Some("ECDSA signing process failed.".into())))?;
            signature.to_vec()
        },
        NAMED_CURVE_P521 => {
            let Handle::P521PrivateKey(d) = key.handle() else {
                return Err(Error::Operation(Some(
                    "Key handle is not a P521PrivateKey.".into(),
                )));
            };
            let signing_key = SigningKey::<NistP521>::from(d);
            let signature: Signature<NistP521> = signing_key
                .sign_prehash_with_rng(&mut rng, &m)
                .map_err(|_| Error::Operation(Some("ECDSA signing process failed.".into())))?;
            signature.to_vec()
        },
        _ => {
            return Err(Error::NotSupported(Some(
                "Algorithm's curve is not supported.".into(),
            )));
        },
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
        return Err(Error::InvalidAccess(Some("Key type is not public".into())));
    }

    // Step 2. Let hashAlgorithm be the hash member of normalizedAlgorithm.
    let hash_algorithm = &normalized_algorithm.hash;

    // Step 3. Let M be the result of performing the digest operation specified by hashAlgorithm
    // using message.
    let m = hash_algorithm.digest(message)?;

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
        return Err(Error::Operation(Some(
            "Key algorithm is not a elliptic curve key algorithm.".into(),
        )));
    };
    let result = match algorithm.named_curve.as_str() {
        NAMED_CURVE_P256 => {
            let Handle::P256PublicKey(q) = key.handle() else {
                return Err(Error::Operation(Some(
                    "Key handle is not a P256PublicKey.".into(),
                )));
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
            let Handle::P384PublicKey(q) = key.handle() else {
                return Err(Error::Operation(Some(
                    "Key handle is not a P384PublicKey.".into(),
                )));
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
            let Handle::P521PublicKey(q) = key.handle() else {
                return Err(Error::Operation(None));
            };
            match Signature::<NistP521>::from_slice(signature) {
                Ok(signature) => {
                    let verifying_key = VerifyingKey::<NistP521>::from(q);
                    verifying_key.verify_prehash(&m, &signature).is_ok()
                },
                Err(_) => false,
            }
        },
        _ => {
            return Err(Error::NotSupported(Some(
                "Algorithm's curve is not supported.".into(),
            )));
        },
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
    ec_common::generate_key(
        EcAlgorithm::Ecdsa,
        cx,
        global,
        normalized_algorithm,
        extractable,
        usages,
    )
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
    ec_common::import_key(
        EcAlgorithm::Ecdsa,
        cx,
        global,
        normalized_algorithm,
        format,
        key_data,
        extractable,
        usages,
    )
}

/// <https://w3c.github.io/webcrypto/#ecdsa-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    ec_common::export_key(format, key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-getPublicKey>
/// Step 9 - 15, for ECDSA
pub(crate) fn get_public_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    key: &CryptoKey,
    algorithm: &KeyAlgorithmAndDerivatives,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    ec_common::get_public_key(cx, global, key, algorithm, usages)
}
