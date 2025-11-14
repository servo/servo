/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aws_lc_rs::hkdf;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{KeyType, KeyUsage};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyFormat;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_HKDF, ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, KeyAlgorithmAndDerivatives,
    SubtleHkdfParams, SubtleKeyAlgorithm,
};
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/webcrypto/#hkdf-operations-derive-bits>
pub(crate) fn derive_bits(
    normalized_algorithm: &SubtleHkdfParams,
    key: &CryptoKey,
    length: Option<u32>,
) -> Result<Vec<u8>, Error> {
    // Step 1. If length is null or is not a multiple of 8, then throw an OperationError.
    let Some(length) = length else {
        return Err(Error::Operation);
    };
    if length % 8 != 0 {
        return Err(Error::Operation);
    };

    // Step 2. Let keyDerivationKey be the secret represented by the [[handle]] internal slot of key.
    let key_derivation_key = key.handle().as_bytes();

    // NOTE: Since https://github.com/w3c/webcrypto/pull/380, WebCrypto allows zero length in HKDF
    // deriveBits operation. However, in Step 4 below, `output_key_material.fill(&mut result)`
    // gives an error when length is 0. Therefore, when length is zero, we immediately returns an
    // empty byte sequence beforehand.
    if length == 0 {
        return Ok(Vec::new());
    }

    // Step 3. Let result be the result of performing the HKDF extract and then the HKDF expand
    // step described in Section 2 of [RFC5869] using:
    // * the hash member of normalizedAlgorithm as Hash,
    // * keyDerivationKey as the input keying material, IKM,
    // * the salt member of normalizedAlgorithm as salt,
    // * the info member of normalizedAlgorithm as info,
    // * length divided by 8 as the value of L,
    // Step 4. If the key derivation operation fails, then throw an OperationError.
    let mut result = vec![0; length as usize / 8];
    let algorithm = match normalized_algorithm.hash.name.as_str() {
        ALG_SHA1 => hkdf::HKDF_SHA1_FOR_LEGACY_USE_ONLY,
        ALG_SHA256 => hkdf::HKDF_SHA256,
        ALG_SHA384 => hkdf::HKDF_SHA384,
        ALG_SHA512 => hkdf::HKDF_SHA512,
        _ => {
            return Err(Error::NotSupported);
        },
    };
    let salt = hkdf::Salt::new(algorithm, &normalized_algorithm.salt);
    let info = normalized_algorithm.info.as_slice();
    let pseudo_random_key = salt.extract(key_derivation_key);
    let Ok(output_key_material) = pseudo_random_key.expand(std::slice::from_ref(&info), algorithm)
    else {
        return Err(Error::Operation);
    };
    if output_key_material.fill(&mut result).is_err() {
        return Err(Error::Operation);
    };

    // Step 5. Return result.
    Ok(result)
}

/// <https://w3c.github.io/webcrypto/#hkdf-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2. If format is "raw":
    if format == KeyFormat::Raw {
        // Step 2.1. If usages contains a value that is not "deriveKey" or "deriveBits", then throw
        // a SyntaxError.
        if usages
            .iter()
            .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits)) ||
            usages.is_empty()
        {
            return Err(Error::Syntax(None));
        }

        // Step 2.2. If extractable is not false, then throw a SyntaxError.
        if extractable {
            return Err(Error::Syntax(None));
        }

        // Step 2.3. Let key be a new CryptoKey representing the key data provided in keyData.
        // Step 2.4. Set the [[type]] internal slot of key to "secret".
        // Step 2.5. Let algorithm be a new KeyAlgorithm object.
        // Step 2.6. Set the name attribute of algorithm to "HKDF".
        // Step 2.7. Set the [[algorithm]] internal slot of key to algorithm.
        let algorithm = SubtleKeyAlgorithm {
            name: ALG_HKDF.to_string(),
        };
        let key = CryptoKey::new(
            global,
            KeyType::Secret,
            extractable,
            KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
            usages,
            Handle::Hkdf(key_data.to_vec()),
            can_gc,
        );

        // Step 2.8. Return key.
        Ok(key)
    }
    // Otherwise:
    else {
        // throw a NotSupportedError.
        Err(Error::NotSupported)
    }
}

/// <https://w3c.github.io/webcrypto/#hkdf-operations-get-key-length>
pub(crate) fn get_key_length() -> Result<Option<u32>, Error> {
    // Step 1. Return null.
    Ok(None)
}
