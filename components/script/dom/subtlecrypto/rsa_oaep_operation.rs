/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use pkcs8::rand_core::OsRng;
use rsa::Oaep;
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, CryptoKeyPair, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::KeyFormat;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::rsa_common::{self, RsaAlgorithm};
use crate::dom::subtlecrypto::{
    ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, ExportedKey, KeyAlgorithmAndDerivatives,
    NormalizedAlgorithm, SubtleRsaHashedImportParams, SubtleRsaHashedKeyGenParams,
    SubtleRsaOaepParams,
};

/// <https://w3c.github.io/webcrypto/#rsa-oaep-operations-encrypt>
pub(crate) fn encrypt(
    normalized_algorithm: &SubtleRsaOaepParams,
    key: &CryptoKey,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the [[type]] internal slot of key is not "public", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Public {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"public\"".to_string(),
        )));
    }

    // Step 2. Let label be the label member of normalizedAlgorithm or the empty byte sequence if
    // the label member of normalizedAlgorithm is not present.
    let label = normalized_algorithm
        .label
        .as_ref()
        .map(|label| String::from_utf8_lossy(label))
        .unwrap_or_default();

    // Step 3. Perform the encryption operation defined in Section 7.1 of [RFC3447] with the key
    // represented by key as the recipient's RSA public key, plaintext as the message to be
    // encrypted, M and label as the label, L, and with the hash function specified by the hash
    // attribute of the [[algorithm]] internal slot of key as the Hash option and MGF1 (defined in
    // Section B.2.1 of [RFC3447]) as the MGF option.
    // Step 4. If performing the operation results in an error, then throw an OperationError.
    // Step 5. Let ciphertext be the value C that results from performing the operation.
    let Handle::RsaPublicKey(public_key) = key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of key is not an RSA public key".to_string(),
        )));
    };
    let KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algorithm) = key.algorithm() else {
        return Err(Error::Operation(Some(
            "[[algorithm]] internal slot of key is not an RsaHashedKeyAlgorithm".to_string(),
        )));
    };
    let padding = match algorithm.hash.name() {
        ALG_SHA1 => Oaep::new_with_label::<Sha1, _>(label),
        ALG_SHA256 => Oaep::new_with_label::<Sha256, _>(label),
        ALG_SHA384 => Oaep::new_with_label::<Sha384, _>(label),
        ALG_SHA512 => Oaep::new_with_label::<Sha512, _>(label),
        _ => {
            return Err(Error::Operation(Some(format!(
                "Unsupported \"{}\" hash for RSASSA-PKCS1-v1_5",
                algorithm.hash.name()
            ))));
        },
    };
    let ciphertext = public_key
        .encrypt(&mut OsRng, padding, plaintext)
        .map_err(|_| Error::Operation(Some("RSA-OAEP failed to encrypt plaintext".to_string())))?;

    // Step 6. Return ciphertext.
    Ok(ciphertext)
}

/// <https://w3c.github.io/webcrypto/#rsa-oaep-operations-decrypt>
pub(crate) fn decrypt(
    normalized_algorithm: &SubtleRsaOaepParams,
    key: &CryptoKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    // Step 1. If the [[type]] internal slot of key is not "private", then throw an
    // InvalidAccessError.
    if key.Type() != KeyType::Private {
        return Err(Error::InvalidAccess(Some(
            "[[type]] internal slot of key is not \"private\"".to_string(),
        )));
    }

    // Step 2. Let label be the label member of normalizedAlgorithm or the empty byte sequence if
    // the label member of normalizedAlgorithm is not present.
    let label = normalized_algorithm
        .label
        .as_ref()
        .map(|label| String::from_utf8_lossy(label))
        .unwrap_or_default();

    // Step 3. Perform the decryption operation defined in Section 7.1 of [RFC3447] with the key
    // represented by key as the recipient's RSA private key, ciphertext as the ciphertext to be
    // decrypted, C, and label as the label, L, and with the hash function specified by the hash
    // attribute of the [[algorithm]] internal slot of key as the Hash option and MGF1 (defined in
    // Section B.2.1 of [RFC3447]) as the MGF option.
    // Step 4. If performing the operation results in an error, then throw an OperationError.
    // Step 5. Let plaintext the value M that results from performing the operation.
    let Handle::RsaPrivateKey(private_key) = key.handle() else {
        return Err(Error::Operation(Some(
            "[[handle]] internal slot of key is not an RSA private key".to_string(),
        )));
    };
    let KeyAlgorithmAndDerivatives::RsaHashedKeyAlgorithm(algorithm) = key.algorithm() else {
        return Err(Error::Operation(Some(
            "[[algorithm]] internal slot of key is not an RsaHashedKeyAlgorithm".to_string(),
        )));
    };
    let padding = match algorithm.hash.name() {
        ALG_SHA1 => Oaep::new_with_label::<Sha1, _>(label),
        ALG_SHA256 => Oaep::new_with_label::<Sha256, _>(label),
        ALG_SHA384 => Oaep::new_with_label::<Sha384, _>(label),
        ALG_SHA512 => Oaep::new_with_label::<Sha512, _>(label),
        _ => {
            return Err(Error::Operation(Some(format!(
                "Unsupported \"{}\" hash for RSA-OAEP",
                algorithm.hash.name()
            ))));
        },
    };
    let plaintext = private_key
        .decrypt(padding, ciphertext)
        .map_err(|_| Error::Operation(Some("RSA-OAEP failed to decrypt ciphertext".to_string())))?;

    // Step 6. Return plaintext.
    Ok(plaintext)
}

/// <https://w3c.github.io/webcrypto/#rsa-oaep-operations-generate-key>
pub(crate) fn generate_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleRsaHashedKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<CryptoKeyPair, Error> {
    rsa_common::generate_key(
        RsaAlgorithm::RsaOaep,
        cx,
        global,
        normalized_algorithm,
        extractable,
        usages,
    )
}

/// <https://w3c.github.io/webcrypto/#rsa-oaep-operations-import-key>
pub(crate) fn import_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleRsaHashedImportParams,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    rsa_common::import_key(
        RsaAlgorithm::RsaOaep,
        cx,
        global,
        normalized_algorithm,
        format,
        key_data,
        extractable,
        usages,
    )
}

/// <https://w3c.github.io/webcrypto/#rsa-oaep-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    rsa_common::export_key(RsaAlgorithm::RsaOaep, format, key)
}
