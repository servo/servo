/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use digest::Digest;
use ecdsa::signature::hazmat::{PrehashVerifier, RandomizedPrehashSigner};
use ecdsa::{Signature, SigningKey, VerifyingKey};
use elliptic_curve::rand_core::OsRng;
use js::context::JSContext;
use p256::NistP256;
use p384::NistP384;
use p521::NistP521;
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

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
    CryptoAlgorithm, ExportedKey, KeyAlgorithmAndDerivatives, NAMED_CURVE_P256, NAMED_CURVE_P384,
    NAMED_CURVE_P521, NormalizedAlgorithm, SubtleEcKeyGenParams, SubtleEcKeyImportParams,
    SubtleEcdsaParams, ec_common,
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
        CryptoAlgorithm::Sha1 => Sha1::new_with_prefix(message).finalize().to_vec(),
        CryptoAlgorithm::Sha256 => Sha256::new_with_prefix(message).finalize().to_vec(),
        CryptoAlgorithm::Sha384 => Sha384::new_with_prefix(message).finalize().to_vec(),
        CryptoAlgorithm::Sha512 => Sha512::new_with_prefix(message).finalize().to_vec(),
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
        CryptoAlgorithm::Sha1 => Sha1::new_with_prefix(message).finalize().to_vec(),
        CryptoAlgorithm::Sha256 => Sha256::new_with_prefix(message).finalize().to_vec(),
        CryptoAlgorithm::Sha384 => Sha384::new_with_prefix(message).finalize().to_vec(),
        CryptoAlgorithm::Sha512 => Sha512::new_with_prefix(message).finalize().to_vec(),
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
