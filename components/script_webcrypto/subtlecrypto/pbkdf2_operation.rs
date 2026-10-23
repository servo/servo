/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use pbkdf2::pbkdf2_hmac;
use script_bindings::codegen::GenericBindings::CryptoKeyBinding::{KeyType, KeyUsage};
use script_bindings::codegen::GenericBindings::SubtleCryptoBinding::KeyFormat;
use script_bindings::error::Error;
use script_bindings::root::DomRoot;
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

use crate::cryptokey::{CryptoKey, Handle, KeyUsageSliceHelper};
use crate::subtlecrypto::{
    CryptoAlgorithm, KeyAlgorithm, KeyAlgorithmAndDerivatives, NormalizedAlgorithm, Pbkdf2Params,
};
use crate::traits::Equivalence;

/// <https://w3c.github.io/webcrypto/#pbkdf2-operations-derive-bits>
pub(crate) fn derive_bits<D: Equivalence>(
    normalized_algorithm: &Pbkdf2Params,
    key: &CryptoKey<D>,
    length: Option<u32>,
) -> Result<Vec<u8>, Error> {
    // Step 1. If length is null or is not a multiple of 8, then throw an OperationError.
    let Some(length) = length else {
        return Err(Error::Operation(Some("Length is null".into())));
    };
    if length % 8 != 0 {
        return Err(Error::Operation(Some(
            "Length is not a multiple of 8".into(),
        )));
    };

    // Step 2. If the iterations member of normalizedAlgorithm is zero, then throw an
    // OperationError.
    if normalized_algorithm.iterations == 0 {
        return Err(Error::Operation(Some(
            "Normalized algorithm's iterations is zero".into(),
        )));
    }

    // Step 3. If length is zero, return an empty byte sequence.
    if length == 0 {
        return Ok(Vec::new());
    }

    // Step 4. Let prf be the MAC Generation function described in Section 4 of [FIPS-198-1] using
    // the hash function described by the hash member of normalizedAlgorithm.
    // Step 5. Let result be the result of performing the PBKDF2 operation defined in Section 5.2
    // of [RFC8018] using prf as the pseudo-random function, PRF, the password represented by the
    // [[handle]] internal slot of key as the password, P, the salt attribute of
    // normalizedAlgorithm as the salt, S, the value of the iterations attribute of
    // normalizedAlgorithm as the iteration count, c, and length divided by 8 as the intended key
    // length, dkLen.
    // Step 5. If the key derivation operation fails, then throw an OperationError.
    let Handle::Pbkdf2Password(password) = key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of key is not a PBKDF2 password".into(),
        )));
    };
    let mut result = vec![0; length as usize / 8];
    match normalized_algorithm.hash.name() {
        CryptoAlgorithm::Sha1 => {
            pbkdf2_hmac::<Sha1>(
                password,
                &normalized_algorithm.salt,
                normalized_algorithm.iterations,
                &mut result,
            );
        },
        CryptoAlgorithm::Sha256 => {
            pbkdf2_hmac::<Sha256>(
                password,
                &normalized_algorithm.salt,
                normalized_algorithm.iterations,
                &mut result,
            );
        },
        CryptoAlgorithm::Sha384 => {
            pbkdf2_hmac::<Sha384>(
                password,
                &normalized_algorithm.salt,
                normalized_algorithm.iterations,
                &mut result,
            );
        },
        CryptoAlgorithm::Sha512 => {
            pbkdf2_hmac::<Sha512>(
                password,
                &normalized_algorithm.salt,
                normalized_algorithm.iterations,
                &mut result,
            );
        },
        _ => {
            return Err(Error::NotSupported(Some(
                "Normalized algorithm's hash name is not supported".into(),
            )));
        },
    };

    // Step 6. Return result
    Ok(result)
}

/// <https://w3c.github.io/webcrypto/#pbkdf2-operations-import-key>
pub(crate) fn import_key<D: Equivalence>(
    cx: &mut JSContext,
    global: &D::GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey<D>>, Error> {
    // Step 1. If format is not "raw", throw a NotSupportedError
    if !matches!(format, KeyFormat::Raw | KeyFormat::Raw_secret) {
        return Err(Error::NotSupported(Some("Format is not raw".into())));
    }

    // Step 2. If usages contains a value that is not "deriveKey" or "deriveBits", then throw a
    // SyntaxError.
    usages.ensure_only_contain_entries_from(&[KeyUsage::DeriveKey, KeyUsage::DeriveBits])?;

    // Step 3. If extractable is not false, then throw a SyntaxError.
    if extractable {
        return Err(Error::Syntax(Some("Extractable is not false".into())));
    }

    // Step 4. Let key be a new CryptoKey representing keyData.
    // Step 5. Set the [[type]] internal slot of key to "secret".
    // Step 6. Let algorithm be a new KeyAlgorithm object.
    // Step 7. Set the name attribute of algorithm to "PBKDF2".
    // Step 8. Set the [[algorithm]] internal slot of key to algorithm.
    let algorithm = KeyAlgorithm {
        name: CryptoAlgorithm::Pbkdf2,
    };
    let key = CryptoKey::new(
        cx,
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
        usages.normalized_value(),
        Handle::Pbkdf2Password(key_data.to_vec().into()),
    );

    // Step 9. Return key.
    Ok(key)
}

/// <https://w3c.github.io/webcrypto/#pbkdf2-operations-get-key-length>
pub(crate) fn get_key_length() -> Result<Option<u32>, Error> {
    // Step 1. Return null.
    Ok(None)
}
