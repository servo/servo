/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use elliptic_curve::SecretKey;
use elliptic_curve::rand_core::OsRng;
use js::context::JSContext;
use p256::NistP256;
use p384::NistP384;
use p521::NistP521;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{CryptoKeyPair, KeyType, KeyUsage};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    CryptoAlgorithm, KeyAlgorithmAndDerivatives, NAMED_CURVE_P256, NAMED_CURVE_P384,
    NAMED_CURVE_P521, SubtleEcKeyAlgorithm, SubtleEcKeyGenParams,
};

pub(crate) enum EcAlgorithm {
    Ecdsa,
    Ecdh,
}

/// <https://w3c.github.io/webcrypto/#ecdsa-operations-generate-key>
/// <https://w3c.github.io/webcrypto/#ecdh-operations-generate-key>
pub(crate) fn generate_key(
    ec_algorithm: EcAlgorithm,
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleEcKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<CryptoKeyPair, Error> {
    match ec_algorithm {
        EcAlgorithm::Ecdsa => {
            // Step 1. If usages contains a value which is not one of "sign" or "verify", then throw
            // a SyntaxError.
            if usages
                .iter()
                .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify))
            {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"sign\" or \"verify\"".into(),
                )));
            }
        },
        EcAlgorithm::Ecdh => {
            // Step 1. If usages contains an entry which is not "deriveKey" or "deriveBits" then
            // throw a SyntaxError.
            if usages
                .iter()
                .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
            {
                return Err(Error::Syntax(Some(
                    "Usages contains an entry which is not \"deriveKey\" or \"deriveBits\"".into(),
                )));
            }
        },
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
        named_curve => {
            return Err(Error::NotSupported(Some(format!(
                "Unsupported named curve: {}",
                named_curve
            ))));
        },
    };

    // Step 4. Let algorithm be a new EcKeyAlgorithm object.
    // Step 6. Set the namedCurve attribute of algorithm to equal the namedCurve member of
    // normalizedAlgorithm.
    let algorithm = SubtleEcKeyAlgorithm {
        name: match ec_algorithm {
            EcAlgorithm::Ecdsa => {
                // Step 5. Set the name attribute of algorithm to "ECDSA".
                CryptoAlgorithm::Ecdsa
            },
            EcAlgorithm::Ecdh => {
                // Step 5. Set the name member of algorithm to "ECDH".
                CryptoAlgorithm::Ecdh
            },
        },
        named_curve: normalized_algorithm.named_curve.clone(),
    };

    // Step 7. Let publicKey be a new CryptoKey representing the public key of the generated key pair.
    // Step 8. Set the [[type]] internal slot of publicKey to "public"
    // Step 9. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 10. Set the [[extractable]] internal slot of publicKey to true.
    let public_key_usage = match ec_algorithm {
        EcAlgorithm::Ecdsa => {
            // Step 11. Set the [[usages]] internal slot of publicKey to be the usage intersection
            // of usages and [ "verify" ].
            usages
                .iter()
                .filter(|usage| **usage == KeyUsage::Verify)
                .cloned()
                .collect()
        },
        EcAlgorithm::Ecdh => {
            // Step 11. Set the [[usages]] internal slot of publicKey to be the empty list.
            Vec::new()
        },
    };
    let public_key = CryptoKey::new(
        cx,
        global,
        KeyType::Public,
        true,
        KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm.clone()),
        public_key_usage,
        public_key_handle,
    );

    // Step 12. Let privateKey be a new CryptoKey representing the private key of the generated key pair.
    // Step 13. Set the [[type]] internal slot of privateKey to "private"
    // Step 14. Set the [[algorithm]] internal slot of privateKey to algorithm.
    // Step 15. Set the [[extractable]] internal slot of privateKey to extractable.
    let private_key_usage = match ec_algorithm {
        EcAlgorithm::Ecdsa => {
            // Step 16. Set the [[usages]] internal slot of privateKey to be the usage intersection
            // of usages and [ "sign" ].
            usages
                .iter()
                .filter(|usage| **usage == KeyUsage::Sign)
                .cloned()
                .collect()
        },
        EcAlgorithm::Ecdh => {
            // Step 16. Set the [[usages]] internal slot of privateKey to be the usage intersection
            // of usages and [ "deriveKey", "deriveBits" ].
            usages
                .iter()
                .filter(|usage| matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
                .cloned()
                .collect()
        },
    };
    let private_key = CryptoKey::new(
        cx,
        global,
        KeyType::Private,
        extractable,
        KeyAlgorithmAndDerivatives::EcKeyAlgorithm(algorithm),
        private_key_usage,
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

/// <https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-getPublicKey>
/// Step 9 - 15, for elliptic curve cryptography
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
            "Usages contains an entry which is not \"verify\"".to_string(),
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
        Handle::P256PrivateKey(private_key) => Handle::P256PublicKey(private_key.public_key()),
        Handle::P384PrivateKey(private_key) => Handle::P384PublicKey(private_key.public_key()),
        Handle::P521PrivateKey(private_key) => Handle::P521PublicKey(private_key.public_key()),
        _ => {
            return Err(Error::Operation(Some(
                "[[handle]] internal slot of key is not an elliptic curve private key".to_string(),
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
