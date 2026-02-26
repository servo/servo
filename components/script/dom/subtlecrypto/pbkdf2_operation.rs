/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::num::NonZero;

use aws_lc_rs::pbkdf2;
use js::context::JSContext;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{KeyType, KeyUsage};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyFormat;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_PBKDF2, ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, KeyAlgorithmAndDerivatives,
    NormalizedAlgorithm, SubtleKeyAlgorithm, SubtlePbkdf2Params,
};

/// <https://w3c.github.io/webcrypto/#pbkdf2-operations-derive-bits>
pub(crate) fn derive_bits(
    normalized_algorithm: &SubtlePbkdf2Params,
    key: &CryptoKey,
    length: Option<u32>,
) -> Result<Vec<u8>, Error> {
    // Step 1. If length is null or is not a multiple of 8, then throw an OperationError.
    let Some(length) = length else {
        return Err(Error::Operation(None));
    };
    if length % 8 != 0 {
        return Err(Error::Operation(None));
    };

    // Step 2. If the iterations member of normalizedAlgorithm is zero, then throw an OperationError.
    let Ok(iterations) = NonZero::<u32>::try_from(normalized_algorithm.iterations) else {
        return Err(Error::Operation(None));
    };

    // Step 3. If length is zero, return an empty byte sequence.
    if length == 0 {
        return Ok(Vec::new());
    }

    // Step 4. Let prf be the MAC Generation function described in Section 4 of [FIPS-198-1] using
    // the hash function described by the hash member of normalizedAlgorithm.
    let prf = match normalized_algorithm.hash.name() {
        ALG_SHA1 => pbkdf2::PBKDF2_HMAC_SHA1,
        ALG_SHA256 => pbkdf2::PBKDF2_HMAC_SHA256,
        ALG_SHA384 => pbkdf2::PBKDF2_HMAC_SHA384,
        ALG_SHA512 => pbkdf2::PBKDF2_HMAC_SHA512,
        _ => {
            return Err(Error::NotSupported(None));
        },
    };

    // Step 5. Let result be the result of performing the PBKDF2 operation defined in Section 5.2
    // of [RFC8018] using prf as the pseudo-random function, PRF, the password represented by the
    // [[handle]] internal slot of key as the password, P, the salt attribute of
    // normalizedAlgorithm as the salt, S, the value of the iterations attribute of
    // normalizedAlgorithm as the iteration count, c, and length divided by 8 as the intended key
    // length, dkLen.
    let mut result = vec![0; length as usize / 8];
    pbkdf2::derive(
        prf,
        iterations,
        &normalized_algorithm.salt,
        key.handle().as_bytes(),
        &mut result,
    );

    // Step 5. If the key derivation operation fails, then throw an OperationError.
    // TODO: Investigate when key derivation can fail and how ring handles that case
    // (pbkdf2::derive does not return a Result type)

    // Step 6. Return result
    Ok(result)
}

/// <https://w3c.github.io/webcrypto/#pbkdf2-operations-import-key>
pub(crate) fn import_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. If format is not "raw", throw a NotSupportedError
    if !matches!(format, KeyFormat::Raw | KeyFormat::Raw_secret) {
        return Err(Error::NotSupported(None));
    }

    // Step 2. If usages contains a value that is not "deriveKey" or "deriveBits", then throw a SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits)) ||
        usages.is_empty()
    {
        return Err(Error::Syntax(None));
    }

    // Step 3. If extractable is not false, then throw a SyntaxError.
    if extractable {
        return Err(Error::Syntax(None));
    }

    // Step 4. Let key be a new CryptoKey representing keyData.
    // Step 5. Set the [[type]] internal slot of key to "secret".
    // Step 6. Let algorithm be a new KeyAlgorithm object.
    // Step 7. Set the name attribute of algorithm to "PBKDF2".
    // Step 8. Set the [[algorithm]] internal slot of key to algorithm.
    let algorithm = SubtleKeyAlgorithm {
        name: ALG_PBKDF2.to_string(),
    };
    let key = CryptoKey::new(
        cx,
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
        usages,
        Handle::Pbkdf2(key_data.to_vec()),
    );

    // Step 9. Return key.
    Ok(key)
}

/// <https://w3c.github.io/webcrypto/#pbkdf2-operations-get-key-length>
pub(crate) fn get_key_length() -> Result<Option<u32>, Error> {
    // Step 1. Return null.
    Ok(None)
}
