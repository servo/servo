/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use elliptic_curve::Curve;
use elliptic_curve::generic_array::typenum::Unsigned;
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
    SubtleEcKeyGenParams, SubtleEcKeyImportParams, SubtleEcdhKeyDeriveParams, ec_common,
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
    ec_common::export_key(format, key)
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
