use aws_lc_rs::hmac;
use script_bindings::codegen::GenericBindings::CryptoKeyBinding::{CryptoKeyMethods, KeyType};
use script_bindings::error::Error;

use crate::dom::cryptokey::CryptoKey;
use crate::dom::subtlecrypto::{ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, KeyAlgorithmAndDerivatives, SubtleAlgorithm, SubtleEcdsaParams, SubtleKeyAlgorithm};

/// <https://www.w3.org/TR/webcrypto-2/#ecdsa-operations-sign>
pub(crate) fn sign(algo: &SubtleEcdsaParams, key: &CryptoKey, message: &[u8]) -> Result<Vec<u8>, Error> {
    // Step 1. If the [[type]] internal slot of key is not "private", then throw an InvalidAccessError
    if key.Type() != KeyType::Private {
        return Err(Error::InvalidAccess);
    }
    // Step 2. Let hashAlgorithm be the hash member of normalizedAlgorithm.
    // hash_algorithm = algo.hash;
    // Step 3. Let M be the result of performing the digest operation specified by hashAlgorithm using message.
    let hash_function = match algo.hash.name.as_str() {
        ALG_SHA1 => hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
        ALG_SHA256 => hmac::HMAC_SHA256,
        ALG_SHA384 => hmac::HMAC_SHA384,
        ALG_SHA512 => hmac::HMAC_SHA512,
        _ => return Err(Error::NotSupported),
    };
    let sign_key = hmac::Key::new(hash_function, key.handle().as_bytes());
    let mac = hmac::sign(&sign_key, message);
    // Step 4. Let d be the ECDSA private key associated with key.
    // Step 5. Let params be the EC domain parameters associated with key.
    // TODO: finish
    return Err(Error::NotSupported);
}

/// <https://www.w3.org/TR/webcrypto-2/#ecdsa-operations-verify>
pub(crate) fn verify() -> Result<bool, Error> {
    Err(Error::NotSupported)
}
