/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{KeyType, KeyUsage};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyFormat;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{KeyAlgorithmAndDerivatives, SubtleAlgorithm, SubtleKeyAlgorithm};
use crate::script_runtime::CanGc;

/// <https://wicg.github.io/webcrypto-modern-algos/#argon2-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAlgorithm,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2. If format is not "raw-secret", throw a NotSupportedError
    if format != KeyFormat::Raw_secret {
        return Err(Error::NotSupported(None));
    }

    // Step 3. If usages contains a value that is not "deriveKey" or "deriveBits", then throw a
    // SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
    {
        return Err(Error::Syntax(None));
    }

    // Step 4. If extractable is not false, then throw a SyntaxError.
    if extractable {
        return Err(Error::Syntax(None));
    }

    // Step 5. Let key be a new CryptoKey representing keyData.
    // Step 6. Set the [[type]] internal slot of key to "secret".
    // Step 7. Set the [[extractable]] internal slot of key to false.
    // Step 8. Let algorithm be a new KeyAlgorithm object.
    // Step 9. Set the name attribute of algorithm to the name member of normalizedAlgorithm.
    // Step 10. Set the [[algorithm]] internal slot of key to algorithm.
    let algorithm = SubtleKeyAlgorithm {
        name: normalized_algorithm.name.clone(),
    };
    let key = CryptoKey::new(
        global,
        KeyType::Secret,
        false,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
        usages,
        Handle::Argon2Password(key_data.to_vec()),
        can_gc,
    );

    // Step 11. Return key.
    Ok(key)
}
