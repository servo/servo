/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base64ct::{Base64UrlUnpadded, Encoding};
use chacha20poly1305::Key;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{JsonWebKey, KeyFormat};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_CHACHA20_POLY1305, ExportedKey, JsonWebKeyExt, KeyAlgorithmAndDerivatives,
    SubtleKeyAlgorithm,
};
use crate::script_runtime::CanGc;

/// <https://wicg.github.io/webcrypto-modern-algos/#chacha20-poly1305-operations-import-key>
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
        return Err(Error::Syntax(None));
    }

    // Step 3.
    let data;
    match format {
        // If format is "raw-secret":
        KeyFormat::Raw_secret => {
            // Step 3.1. Let data be keyData.
            data = key_data.to_vec();

            // Step 3.2. If the length in bits of data is not 256 then throw a DataError.
            if data.len() != 32 {
                return Err(Error::Data(None));
            }
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 3.1.
            // If keyData is a JsonWebKey dictionary:
            //     Let jwk equal keyData.
            // Otherwise:
            //     Throw a DataError.
            let jwk = JsonWebKey::parse(GlobalScope::get_cx(), key_data)?;

            // Step 3.2. If the kty field of jwk is not "oct", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "oct") {
                return Err(Error::Data(None));
            }

            // Step 3.3. If jwk does not meet the requirements of Section 6.4 of JSON Web
            // Algorithms [JWA], then throw a DataError.
            let Some(k) = jwk.k.as_ref() else {
                return Err(Error::Data(None));
            };

            // Step 3.4. Let data be the byte sequence obtained by decoding the k field of jwk.
            data = Base64UrlUnpadded::decode_vec(&k.str()).map_err(|_| Error::Data(None))?;

            // Step 3.5. If the alg field of jwk is present, and is not "C20P", then throw a
            // DataError.
            if jwk.alg.as_ref().is_some_and(|alg| alg != "C20P") {
                return Err(Error::Data(None));
            }

            // Step 3.6. If usages is non-empty and the use field of jwk is present and is not
            // "enc", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                return Err(Error::Data(None));
            }

            // Step 3.7. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK] or does not contain all of the specified usages
            // values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 3.8. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data(None));
            }
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(None));
        },
    }

    // Step 4. Let key be a new CryptoKey object representing a key with value data.
    // Step 5. Let algorithm be a new KeyAlgorithm.
    // Step 6. Set the name attribute of algorithm to "ChaCha20-Poly1305".
    // Step 7. Set the [[algorithm]] internal slot of key to algorithm.
    let algorithm = SubtleKeyAlgorithm {
        name: ALG_CHACHA20_POLY1305.to_string(),
    };
    let key = CryptoKey::new(
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::KeyAlgorithm(algorithm),
        usages,
        Handle::ChaCha20Poly1305Key(Key::from_exact_iter(data).ok_or(Error::Data(None))?),
        can_gc,
    );

    // Step 8. Return key.
    Ok(key)
}

/// <https://wicg.github.io/webcrypto-modern-algos/#chacha20-poly1305-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    // Step 1. If the underlying cryptographic key material represented by the [[handle]] internal
    // slot of key cannot be accessed, then throw an OperationError.
    let Handle::ChaCha20Poly1305Key(key_handle) = key.handle() else {
        return Err(Error::Operation(None));
    };

    // Step 2.
    let result = match format {
        // If format is "raw-secret":
        KeyFormat::Raw_secret => {
            // Step 2.1. Let data be a byte sequence containing the raw octets of the key
            // represented by [[handle]] internal slot of key.
            let data = key_handle.to_vec();

            // Step 2.2 Let result be data.
            ExportedKey::Bytes(data)
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1. Let jwk be a new JsonWebKey dictionary.
            // Step 2.2. Set the kty attribute of jwk to the string "oct".
            // Step 2.3. Set the k attribute of jwk to be a string containing the raw octets of the
            // key represented by [[handle]] internal slot of key, encoded according to Section 6.4
            // of JSON Web Algorithms [JWA].
            // Step 2.4. Set the alg attribute of jwk to the string "C20P".
            // Step 2.5. Set the key_ops attribute of jwk to equal the usages attribute of key.
            // Step 2.6. Set the ext attribute of jwk to equal the [[extractable]] internal slot of
            // key.
            let jwk = JsonWebKey {
                kty: Some(DOMString::from("oct")),
                k: Some(Base64UrlUnpadded::encode_string(key_handle.as_slice()).into()),
                alg: Some(DOMString::from("C20P")),
                key_ops: Some(
                    key.usages()
                        .iter()
                        .map(|usage| DOMString::from(usage.as_str()))
                        .collect::<Vec<DOMString>>(),
                ),
                ext: Some(key.Extractable()),
                ..Default::default()
            };

            // Step 2.7. Let result be the result of converting jwk to an ECMAScript Object, as
            // defined by [WebIDL].
            // NOTE: We convert it to JSObject in SubtleCrypto::ExportKey.
            ExportedKey::Jwk(Box::new(jwk))
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(None));
        },
    };

    // Step 3. Return result.
    Ok(result)
}
