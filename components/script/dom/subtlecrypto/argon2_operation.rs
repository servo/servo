/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use argon2::{Argon2, AssociatedData, ParamsBuilder, Version};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{KeyType, KeyUsage};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyFormat;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_ARGON2D, ALG_ARGON2I, ALG_ARGON2ID, KeyAlgorithmAndDerivatives, SubtleAlgorithm,
    SubtleArgon2Params, SubtleKeyAlgorithm,
};
use crate::script_runtime::CanGc;

/// <https://wicg.github.io/webcrypto-modern-algos/#argon2-operations-derive-bits>
pub(crate) fn derive_bits(
    normalized_algorithm: &SubtleArgon2Params,
    key: &CryptoKey,
    length: Option<u32>,
) -> Result<Vec<u8>, Error> {
    // Step 1. If length is null, or is less than 32 (4*8), then throw an OperationError.
    let length = length.ok_or(Error::Operation(Some(
        "Length for deriving bits is null".to_string(),
    )))?;
    if length < 32 {
        return Err(Error::Operation(Some(
            "Length for deriving bits is less than 32".to_string(),
        )));
    }

    // Step 2. If the version member of normalizedAlgorithm is present and is not 19 (0x13), then
    // throw an OperationError.
    if normalized_algorithm
        .version
        .is_some_and(|version| version != 19)
    {
        return Err(Error::Operation(Some(
            "Argon2 version is not 19 (0x13)".to_string(),
        )));
    }

    // Step 3. If the parallelism member of normalizedAlgorithm is zero, or greater than 16777215
    // (2^24-1), then throw an OperationError.
    if normalized_algorithm.parallelism == 0 || normalized_algorithm.parallelism > 16777215 {
        return Err(Error::Operation(Some(
            "Argon2 parallelism is zero, or greater than 16777215 (2^24-1)".to_string(),
        )));
    }

    // Step 4. If the memory member of normalizedAlgorithm is less than 8 times the parallelism
    // member of normalizedAlgorithm, then throw an OperationError.
    if normalized_algorithm.memory < 8 * normalized_algorithm.parallelism {
        return Err(Error::Operation(Some(
            "Argon2 memory is less than 8 times the parallelism".to_string(),
        )));
    }

    // Step 5. If the passes member of normalizedAlgorithm is zero, then throw an OperationError.
    if normalized_algorithm.passes == 0 {
        return Err(Error::Operation(Some("Argon2 passes is zero".to_string())));
    }

    // Step 6.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "Argon2d":
    //     Let type be 0.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "Argon2i":
    //     Let type be 1.
    // If the name member of normalizedAlgorithm is a case-sensitive string match for "Argon2id":
    //     Let type be 2.
    let type_ = match normalized_algorithm.name.as_str() {
        ALG_ARGON2D => argon2::Algorithm::Argon2d,
        ALG_ARGON2I => argon2::Algorithm::Argon2i,
        ALG_ARGON2ID => argon2::Algorithm::Argon2id,
        _ => {
            return Err(Error::NotSupported(Some(format!(
                "Unknown Argon2 algorithm name: {}",
                normalized_algorithm.name
            ))));
        },
    };

    // Step 7. Let secretValue be the secretValue member of normalizedAlgorithm, if present.
    // Step 8. Let associatedData be the associatedData member of normalizedAlgorithm, if present.
    // Step 9. Let result be the result of performing the Argon2 function defined in Section 3 of
    // [RFC9106] using the password represented by [[handle]] internal slot of key as the message,
    // P, the nonce attribute of normalizedAlgorithm as the nonce, S, the value of the parallelism
    // attribute of normalizedAlgorithm as the degree of parallelism, p, the value of the memory
    // attribute of normalizedAlgorithm as the memory size, m, the value of the passes attribute of
    // normalizedAlgorithm as the number of passes, t, 0x13 as the version number, v, secretValue
    // (if present) as the secret value, K, associatedData (if present) as the associated data, X,
    // type as the type, y, and length divided by 8 as the tag length, T.
    // Step 10. If the key derivation operation fails, then throw an OperationError.
    let Handle::Argon2Password(password) = key.handle() else {
        return Err(Error::Operation(Some(
            "Key handle is not an Argon2 password".to_string(),
        )));
    };
    let mut params_builder = ParamsBuilder::new();
    if let Some(associated_data) = &normalized_algorithm.associated_data {
        let _ = params_builder.data(AssociatedData::new(associated_data).map_err(|_| {
            Error::Operation(Some(
                "Argon2 fails to add associated data to parameter builder".to_string(),
            ))
        })?);
    }
    let params = params_builder
        .p_cost(normalized_algorithm.parallelism)
        .m_cost(normalized_algorithm.memory)
        .t_cost(normalized_algorithm.passes)
        .build()
        .map_err(|_| Error::Operation(Some("Argon2 fails to build parameters".to_string())))?;
    let argon2_context = match &normalized_algorithm.secret_value {
        Some(secret) => Argon2::new_with_secret(secret, type_, Version::V0x13, params)
            .map_err(|_| Error::Operation(Some("Argon2 fails to create context".to_string())))?,
        None => Argon2::new(type_, Version::V0x13, params),
    };
    let mut result = vec![0u8; length as usize / 8];
    argon2_context
        .hash_password_into(password, &normalized_algorithm.nonce, &mut result)
        .map_err(|_| Error::Operation(Some("Argon2 fails to hash the password".to_string())))?;

    // Step 11. Return result.
    Ok(result)
}

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
        return Err(Error::NotSupported(Some(
            "Import key format is not \"raw-secret\"".to_string(),
        )));
    }

    // Step 3. If usages contains a value that is not "deriveKey" or "deriveBits", then throw a
    // SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits))
    {
        return Err(Error::Syntax(Some(
            "Usages contains a value that is not \"deriveKey\" or \"deriveBits\"".to_string(),
        )));
    }

    // Step 4. If extractable is not false, then throw a SyntaxError.
    if extractable {
        return Err(Error::Syntax(Some("Extrabctable is not false".to_string())));
    }

    // Step 5. Let key be a new CryptoKey representing keyData.
    // Step 6. Set the [[type]] internal slot of key to "secret".
    // Step 7. Let algorithm be a new KeyAlgorithm object.
    // Step 8. Set the name attribute of algorithm to the name member of normalizedAlgorithm.
    // Step 9. Set the [[algorithm]] internal slot of key to algorithm.
    let algorithm = SubtleKeyAlgorithm {
        name: normalized_algorithm.name.clone(),
    };
    let key = CryptoKey::new(
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
        usages,
        Handle::Argon2Password(key_data.to_vec()),
        can_gc,
    );

    // Step 10. Return key.
    Ok(key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#argon2-operations-get-key-length>
pub(crate) fn get_key_length() -> Result<Option<u32>, Error> {
    // Step 1. Return null.
    Ok(None)
}
