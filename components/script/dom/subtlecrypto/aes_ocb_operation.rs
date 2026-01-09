/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aes::cipher::crypto_common::Key;
use aes::{Aes128, Aes192, Aes256};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{KeyType, KeyUsage};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyFormat;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::aes_common::AesAlgorithm;
use crate::dom::subtlecrypto::{
    ALG_AES_OCB, ExportedKey, KeyAlgorithmAndDerivatives, SubtleAesKeyAlgorithm,
    SubtleAesKeyGenParams, aes_common,
};
use crate::script_runtime::CanGc;

/// <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-generate-key>
pub(crate) fn generate_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleAesKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    aes_common::generate_key(
        AesAlgorithm::AesOcb,
        global,
        normalized_algorithm,
        extractable,
        usages,
        can_gc,
    )
}

/// <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-import-key>
pub(crate) fn import_key(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2. If usages contains an entry which is not one of "encrypt", "decrypt", "wrapKey" or
    // "unwrapKey", then throw a SyntaxError.
    if usages.iter().any(|usage| {
        !matches!(
            usage,
            KeyUsage::Encrypt | KeyUsage::Decrypt | KeyUsage::WrapKey | KeyUsage::UnwrapKey
        )
    }) {
        return Err(Error::Syntax(Some(
            "Usages contains an entry which is not one of \"encrypt\", \"decrypt\", \"wrapKey\" \
            or \"unwrapKey\""
                .to_string(),
        )));
    }

    // Step 3.
    let data = aes_common::import_key_from_key_data(
        AesAlgorithm::AesOcb,
        format,
        key_data,
        extractable,
        &usages,
    )?;

    // Step 4. Let key be a new CryptoKey object representing an AES key with value data.
    // Step 5. Let algorithm be a new AesKeyAlgorithm.
    // Step 6. Set the name attribute of algorithm to "AES-OCB".
    // Step 7. Set the length attribute of algorithm to the length, in bits, of data.
    // Step 8. Set the [[algorithm]] internal slot of key to algorithm.
    let handle = match data.len() {
        16 => Handle::Aes128Key(Key::<Aes128>::clone_from_slice(&data)),
        24 => Handle::Aes192Key(Key::<Aes192>::clone_from_slice(&data)),
        32 => Handle::Aes256Key(Key::<Aes256>::clone_from_slice(&data)),
        _ => {
            return Err(Error::Data(Some(
                "The length in bits of key is not 128, 192 or 256".to_string(),
            )));
        },
    };
    let algorithm = SubtleAesKeyAlgorithm {
        name: ALG_AES_OCB.to_string(),
        length: data.len() as u16 * 8,
    };
    let key = CryptoKey::new(
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::AesKeyAlgorithm(algorithm),
        usages,
        handle,
        can_gc,
    );

    // Step 9. Return key.
    Ok(key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    aes_common::export_key(AesAlgorithm::AesOcb, format, key)
}
