/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::KeyUsage;
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{JsonWebKey, KeyFormat};
use crate::dom::bindings::error::Error;
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{JsonWebKeyExt, JwkStringField};

// TODO: Add AES-CTR, AES-CBC, AES-GCM, AES-KW
pub(crate) enum AesAlgorithm {
    AesOcb,
}

/// Step 3 of <https://wicg.github.io/webcrypto-modern-algos/#aes-ocb-operations-import-key>
pub(crate) fn import_key_from_key_data(
    aes_algorithm: AesAlgorithm,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: &[KeyUsage],
) -> Result<Vec<u8>, Error> {
    let data;
    match format {
        // If format is "raw-secret":
        KeyFormat::Raw_secret => {
            // Step 1. Let data be keyData.
            data = key_data.to_vec();

            // Step 2. If the length in bits of data is not 128, 192 or 256 then throw a DataError.
            if !matches!(data.len(), 16 | 24 | 32) {
                return Err(Error::Data(Some(
                    "The length in bits of key is not 128, 192 or 256".to_string(),
                )));
            }
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 1.
            // If keyData is a JsonWebKey dictionary:
            //     Let jwk equal keyData.
            // Otherwise:
            //     Throw a DataError.
            let jwk = JsonWebKey::parse(GlobalScope::get_cx(), key_data)?;

            // Step 3.2. If the kty field of jwk is not "oct", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "oct") {
                return Err(Error::Data(Some(
                    "The kty field of jwk is not \"oct\"".to_string(),
                )));
            }

            // Step 3. If jwk does not meet the requirements of Section 6.4 of JSON Web Algorithms
            // [JWA], then throw a DataError.
            // Step 4. Let data be the byte sequence obtained by decoding the k field of jwk.
            data = jwk.decode_required_string_field(JwkStringField::K)?;

            match aes_algorithm {
                AesAlgorithm::AesOcb => {
                    // Step 5.
                    // If data has length 128 bits:
                    //     If the alg field of jwk is present, and is not "A128OCB", then throw a
                    //     DataError.
                    // If data has length 192 bits:
                    //     If the alg field of jwk is present, and is not "A192OCB", then throw a
                    //     DataError.
                    // If data has length 256 bits:
                    //     If the alg field of jwk is present, and is not "A256OCB", then throw a
                    //     DataError.
                    // Otherwise:
                    //     throw a DataError.
                    let expected_alg = match data.len() {
                        16 => "A128OCB",
                        24 => "A192OCB",
                        32 => "A256OCB",
                        _ => {
                            return Err(Error::Data(Some(
                                "The length in bits of key is not 128, 192 or 256".to_string(),
                            )));
                        },
                    };
                    if jwk.alg.as_ref().is_none_or(|alg| alg != expected_alg) {
                        return Err(Error::Data(Some(format!(
                            "The alg field of jwk is present, and is not {}",
                            expected_alg
                        ))));
                    }
                },
            }

            // Step 6. If usages is non-empty and the use field of jwk is present and is not "enc",
            // then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "enc") {
                return Err(Error::Data(Some(
                    "Usages is non-empty and the use field of jwk is present and is not \"enc\""
                        .to_string(),
                )));
            }

            // Step 7. If the key_ops field of jwk is present, and is invalid according to the
            // requirements of JSON Web Key [JWK] or does not contain all of the specified usages
            // values, then throw a DataError.
            jwk.check_key_ops(usages)?;

            // Step 8. If the ext field of jwk is present and has the value false and extractable
            // is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data(Some(
                    "The ext field of jwk is present and has the value false and \
                    extractable is true"
                        .to_string(),
                )));
            }
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported(Some(
                "Unupported import key format for AES key".to_string(),
            )));
        },
    }

    Ok(data)
}
