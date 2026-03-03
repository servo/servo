/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use hkdf::Hkdf;
use js::context::JSContext;
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{KeyType, KeyUsage};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyFormat;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_HKDF, ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, KeyAlgorithmAndDerivatives,
    NormalizedAlgorithm, SubtleHkdfParams, SubtleKeyAlgorithm,
};

/// <https://w3c.github.io/webcrypto/#hkdf-operations-derive-bits>
pub(crate) fn derive_bits(
    normalized_algorithm: &SubtleHkdfParams,
    key: &CryptoKey,
    length: Option<u32>,
) -> Result<Vec<u8>, Error> {
    // Step 1. If length is null or is not a multiple of 8, then throw an OperationError.
    let Some(length) = length else {
        return Err(Error::Operation(Some("length is null".into())));
    };
    if length % 8 != 0 {
        return Err(Error::Operation(Some(
            "length is not a multiple of 8".into(),
        )));
    };

    // Step 2. Let keyDerivationKey be the secret represented by the [[handle]] internal slot of key.
    let Handle::HkdfSecret(key_derivation_key) = key.handle() else {
        return Err(Error::Operation(Some(
            "The [[handle]] internal slot is not from an HKDF key".into(),
        )));
    };

    // Step 3. Let result be the result of performing the HKDF extract and then the HKDF expand
    // step described in Section 2 of [RFC5869] using:
    //     * the hash member of normalizedAlgorithm as Hash,
    //     * keyDerivationKey as the input keying material, IKM,
    //     * the salt member of normalizedAlgorithm as salt,
    //     * the info member of normalizedAlgorithm as info,
    //     * length divided by 8 as the value of L,
    // Step 4. If the key derivation operation fails, then throw an OperationError.
    let mut result = vec![0u8; length as usize / 8];
    match normalized_algorithm.hash.name() {
        ALG_SHA1 => Hkdf::<Sha1>::new(Some(&normalized_algorithm.salt), key_derivation_key)
            .expand(&normalized_algorithm.info, &mut result)
            .map_err(|error| Error::Operation(Some(error.to_string())))?,
        ALG_SHA256 => Hkdf::<Sha256>::new(Some(&normalized_algorithm.salt), key_derivation_key)
            .expand(&normalized_algorithm.info, &mut result)
            .map_err(|error| Error::Operation(Some(error.to_string())))?,
        ALG_SHA384 => Hkdf::<Sha384>::new(Some(&normalized_algorithm.salt), key_derivation_key)
            .expand(&normalized_algorithm.info, &mut result)
            .map_err(|error| Error::Operation(Some(error.to_string())))?,
        ALG_SHA512 => Hkdf::<Sha512>::new(Some(&normalized_algorithm.salt), key_derivation_key)
            .expand(&normalized_algorithm.info, &mut result)
            .map_err(|error| Error::Operation(Some(error.to_string())))?,
        algorithm_name => {
            return Err(Error::Operation(Some(format!(
                "Invalid hash algorithm: {algorithm_name}"
            ))));
        },
    }

    // Step 5. Return result.
    Ok(result)
}

/// <https://w3c.github.io/webcrypto/#hkdf-operations-import-key>
pub(crate) fn import_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2. If format is "raw":
    if matches!(format, KeyFormat::Raw | KeyFormat::Raw_secret) {
        // Step 2.1. If usages contains a value that is not "deriveKey" or "deriveBits", then throw
        // a SyntaxError.
        if usages
            .iter()
            .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits)) ||
            usages.is_empty()
        {
            return Err(Error::Syntax(Some(
                "Usages contains an entry which is not \"deriveKey\" or \"deriveBits\"".into(),
            )));
        }

        // Step 2.2. If extractable is not false, then throw a SyntaxError.
        if extractable {
            return Err(Error::Syntax(Some("'extractable' is not false".into())));
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
            cx,
            global,
            KeyType::Secret,
            extractable,
            KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
            usages,
            Handle::HkdfSecret(key_data.to_vec()),
        );

        // Step 2.8. Return key.
        Ok(key)
    }
    // Otherwise:
    else {
        // throw a NotSupportedError.
        Err(Error::NotSupported(Some(
            "Formats different than \"raw\" are unsupported".into(),
        )))
    }
}

/// <https://w3c.github.io/webcrypto/#hkdf-operations-get-key-length>
pub(crate) fn get_key_length() -> Result<Option<u32>, Error> {
    // Step 1. Return null.
    Ok(None)
}
