/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aws_lc_rs::hmac;
use js::context::JSContext;
use rand::TryRngCore;
use rand::rngs::OsRng;
use script_bindings::codegen::GenericBindings::CryptoKeyBinding::CryptoKeyMethods;
use script_bindings::domstring::DOMString;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{KeyType, KeyUsage};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{JsonWebKey, KeyFormat};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_HMAC, ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, ExportedKey, JsonWebKeyExt,
    JwkStringField, KeyAlgorithmAndDerivatives, NormalizedAlgorithm, SubtleHmacImportParams,
    SubtleHmacKeyAlgorithm, SubtleHmacKeyGenParams, SubtleKeyAlgorithm,
};

/// <https://w3c.github.io/webcrypto/#hmac-operations-sign>
pub(crate) fn sign(key: &CryptoKey, message: &[u8]) -> Result<Vec<u8>, Error> {
    // Step 1. Let mac be the result of performing the MAC Generation operation described in
    // Section 4 of [FIPS-198-1] using the key represented by the [[handle]] internal slot of key,
    // the hash function identified by the hash attribute of the [[algorithm]] internal slot of key
    // and message as the input data text.
    let hash_function = match key.algorithm() {
        KeyAlgorithmAndDerivatives::HmacKeyAlgorithm(algo) => match algo.hash.name.as_str() {
            ALG_SHA1 => hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
            ALG_SHA256 => hmac::HMAC_SHA256,
            ALG_SHA384 => hmac::HMAC_SHA384,
            ALG_SHA512 => hmac::HMAC_SHA512,
            _ => return Err(Error::NotSupported(None)),
        },
        _ => return Err(Error::NotSupported(None)),
    };
    let sign_key = hmac::Key::new(hash_function, key.handle().as_bytes());
    let mac = hmac::sign(&sign_key, message);

    // Step 2. Return mac.
    Ok(mac.as_ref().to_vec())
}

/// <https://w3c.github.io/webcrypto/#hmac-operations-verify>
pub(crate) fn verify(key: &CryptoKey, message: &[u8], signature: &[u8]) -> Result<bool, Error> {
    // Step 1. Let mac be the result of performing the MAC Generation operation described in
    // Section 4 of [FIPS-198-1] using the key represented by the [[handle]] internal slot of key,
    // the hash function identified by the hash attribute of the [[algorithm]] internal slot of key
    // and message as the input data text.
    let hash_function = match key.algorithm() {
        KeyAlgorithmAndDerivatives::HmacKeyAlgorithm(algo) => match algo.hash.name.as_str() {
            ALG_SHA1 => hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
            ALG_SHA256 => hmac::HMAC_SHA256,
            ALG_SHA384 => hmac::HMAC_SHA384,
            ALG_SHA512 => hmac::HMAC_SHA512,
            _ => return Err(Error::NotSupported(None)),
        },
        _ => return Err(Error::NotSupported(None)),
    };
    let sign_key = hmac::Key::new(hash_function, key.handle().as_bytes());
    let mac = hmac::sign(&sign_key, message);

    // Step 2. Return true if mac is equal to signature and false otherwise.
    Ok(mac.as_ref() == signature)
}

/// <https://w3c.github.io/webcrypto/#hmac-operations-generate-key>
pub(crate) fn generate_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleHmacKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. If usages contains any entry which is not "sign" or "verify", then throw a SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify))
    {
        return Err(Error::Syntax(None));
    }

    // Step 2.
    let length = match normalized_algorithm.length {
        // If the length member of normalizedAlgorithm is not present:
        None => {
            // Let length be the block size in bits of the hash function identified by the
            // hash member of normalizedAlgorithm.
            hash_function_block_size_in_bits(normalized_algorithm.hash.name())?
        },
        // Otherwise, if the length member of normalizedAlgorithm is non-zero:
        Some(length) if length != 0 => {
            // Let length be equal to the length member of normalizedAlgorithm.
            length
        },
        // Otherwise:
        _ => {
            // throw an OperationError.
            return Err(Error::Operation(None));
        },
    };

    // Step 3. Generate a key of length length bits.
    // Step 4. If the key generation step fails, then throw an OperationError.
    let mut key_data = vec![0; length as usize];
    if OsRng.try_fill_bytes(&mut key_data).is_err() {
        return Err(Error::JSFailed);
    }

    // Step 6. Let algorithm be a new HmacKeyAlgorithm.
    // Step 7. Set the name attribute of algorithm to "HMAC".
    // Step 8. Set the length attribute of algorithm to length.
    // Step 9. Let hash be a new KeyAlgorithm.
    // Step 10. Set the name attribute of hash to equal the name member of the hash member of
    // normalizedAlgorithm.
    // Step 11. Set the hash attribute of algorithm to hash.
    let hash = SubtleKeyAlgorithm {
        name: normalized_algorithm.hash.name().to_string(),
    };
    let algorithm = SubtleHmacKeyAlgorithm {
        name: ALG_HMAC.to_string(),
        hash,
        length,
    };

    // Step 5. Let key be a new CryptoKey object representing the generated key.
    // Step 12. Set the [[type]] internal slot of key to "secret".
    // Step 13. Set the [[algorithm]] internal slot of key to algorithm.
    // Step 14. Set the [[extractable]] internal slot of key to be extractable.
    // Step 15. Set the [[usages]] internal slot of key to be usages.
    let key = CryptoKey::new(
        cx,
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::HmacKeyAlgorithm(algorithm),
        usages,
        Handle::Hmac(key_data),
    );

    // Step 16. Return key.
    Ok(key)
}

/// <https://w3c.github.io/webcrypto/#hmac-operations-import-key>
pub(crate) fn import_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    normalized_algorithm: &SubtleHmacImportParams,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2. If usages contains an entry which is not "sign" or "verify", then throw a SyntaxError.
    // Note: This is not explicitly spec'ed, but also throw a SyntaxError if usages is empty
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::Sign | KeyUsage::Verify)) ||
        usages.is_empty()
    {
        return Err(Error::Syntax(None));
    }

    // Step 3. Let hash be a new KeyAlgorithm.
    let hash;

    // Step 4.
    let data;
    match format {
        // If format is "raw":
        KeyFormat::Raw | KeyFormat::Raw_secret => {
            // Step 4.1. Let data be keyData.
            data = key_data.to_vec();

            // Step 4.2. Set hash to equal the hash member of normalizedAlgorithm.
            hash = &normalized_algorithm.hash;
        },
        // If format is "jwk":
        KeyFormat::Jwk => {
            // Step 2.1. If keyData is a JsonWebKey dictionary: Let jwk equal keyData.
            // Otherwise: Throw a DataError.
            // NOTE: Deserialize keyData to JsonWebKey dictionary by running JsonWebKey::parse
            let jwk = JsonWebKey::parse(cx, key_data)?;

            // Step 2.2. If the kty field of jwk is not "oct", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "oct") {
                return Err(Error::Data(None));
            }

            // Step 2.3. If jwk does not meet the requirements of Section 6.4 of JSON Web
            // Algorithms [JWA], then throw a DataError.
            // NOTE: Done by Step 2.4 and 2.6.

            // Step 2.4. Let data be the byte sequence obtained by decoding the k field of jwk.
            data = jwk.decode_required_string_field(JwkStringField::K)?;

            // Step 2.5. Set the hash to equal the hash member of normalizedAlgorithm.
            hash = &normalized_algorithm.hash;

            // Step 2.6.
            match hash.name() {
                // If the name attribute of hash is "SHA-1":
                ALG_SHA1 => {
                    // If the alg field of jwk is present and is not "HS1", then throw a DataError.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != "HS1") {
                        return Err(Error::Data(None));
                    }
                },
                // If the name attribute of hash is "SHA-256":
                ALG_SHA256 => {
                    // If the alg field of jwk is present and is not "HS256", then throw a DataError.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != "HS256") {
                        return Err(Error::Data(None));
                    }
                },
                // If the name attribute of hash is "SHA-384":
                ALG_SHA384 => {
                    // If the alg field of jwk is present and is not "HS384", then throw a DataError.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != "HS384") {
                        return Err(Error::Data(None));
                    }
                },
                // If the name attribute of hash is "SHA-512":
                ALG_SHA512 => {
                    // If the alg field of jwk is present and is not "HS512", then throw a DataError.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != "HS512") {
                        return Err(Error::Data(None));
                    }
                },
                // Otherwise,
                _name => {
                    // if the name attribute of hash is defined in another applicable specification:
                    // Perform any key import steps defined by other applicable specifications,
                    // passing format, jwk and hash and obtaining hash
                    // NOTE: Currently not support applicable specification.
                    return Err(Error::NotSupported(None));
                },
            }

            // Step 2.7. If usages is non-empty and the use field of jwk is present and is not
            // "sig", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "sig") {
                return Err(Error::Data(None));
            }

            // Step 2.8. If the key_ops field of jwk is present, and is invalid according to
            // the requirements of JSON Web Key [JWK] or does not contain all of the specified
            // usages values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.9. If the ext field of jwk is present and has the value false and
            // extractable is true, then throw a DataError.
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

    // Step 5. Let length be the length in bits of data.
    let mut length = data.len() as u32 * 8;

    // Step 6. If length is zero then throw a DataError.
    if length == 0 {
        return Err(Error::Data(None));
    }

    // Step 7. If the length member of normalizedAlgorithm is present:
    if let Some(given_length) = normalized_algorithm.length {
        //  If the length member of normalizedAlgorithm is greater than length:
        if given_length > length {
            // throw a DataError.
            return Err(Error::Data(None));
        }
        // Otherwise:
        else {
            // Set length equal to the length member of normalizedAlgorithm.
            length = given_length;
        }
    }

    // Step 10. Let algorithm be a new HmacKeyAlgorithm.
    // Step 11. Set the name attribute of algorithm to "HMAC".
    // Step 12. Set the length attribute of algorithm to length.
    // Step 13. Set the hash attribute of algorithm to hash.
    let algorithm = SubtleHmacKeyAlgorithm {
        name: ALG_HMAC.to_string(),
        hash: SubtleKeyAlgorithm {
            name: hash.name().to_string(),
        },
        length,
    };

    // Step 8. Let key be a new CryptoKey object representing an HMAC key with the first length
    // bits of data.
    // Step 9. Set the [[type]] internal slot of key to "secret".
    // Step 14. Set the [[algorithm]] internal slot of key to algorithm.
    let truncated_data = data[..length as usize / 8].to_vec();
    let key = CryptoKey::new(
        cx,
        global,
        KeyType::Secret,
        extractable,
        KeyAlgorithmAndDerivatives::HmacKeyAlgorithm(algorithm),
        usages,
        Handle::Hmac(truncated_data),
    );

    // Step 15. Return key.
    Ok(key)
}

/// <https://w3c.github.io/webcrypto/#hmac-operations-export-key>
pub(crate) fn export_key(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    match format {
        KeyFormat::Raw | KeyFormat::Raw_secret => match key.handle() {
            Handle::Hmac(key_data) => Ok(ExportedKey::Bytes(key_data.as_slice().to_vec())),
            _ => Err(Error::Operation(None)),
        },
        KeyFormat::Jwk => {
            // Step 4.1. Let jwk be a new JsonWebKey dictionary.
            // Step 4.2. Set the kty attribute of jwk to the string "oct".
            let mut jwk = JsonWebKey {
                kty: Some(DOMString::from("oct")),
                ..Default::default()
            };

            // Step 4.3. Set the k attribute of jwk to be a string containing data, encoded according
            // to Section 6.4 of JSON Web Algorithms [JWA].
            let key_data = key.handle().as_bytes();
            jwk.encode_string_field(JwkStringField::K, key_data);

            // Step 4.4. Let algorithm be the [[algorithm]] internal slot of key.
            // Step 4.5. Let hash be the hash attribute of algorithm.
            // Step 4.6.
            // If the name attribute of hash is "SHA-1":
            //     Set the alg attribute of jwk to the string "HS1".
            // If the name attribute of hash is "SHA-256":
            //     Set the alg attribute of jwk to the string "HS256".
            // If the name attribute of hash is "SHA-384":
            //     Set the alg attribute of jwk to the string "HS384".
            // If the name attribute of hash is "SHA-512":
            //     Set the alg attribute of jwk to the string "HS512".
            // Otherwise, the name attribute of hash is defined in another applicable
            // specification:
            //     Perform any key export steps defined by other applicable specifications, passing
            //     format and key and obtaining alg.
            //     Set the alg attribute of jwk to alg.
            let hash_algorithm = match key.algorithm() {
                KeyAlgorithmAndDerivatives::HmacKeyAlgorithm(alg) => match &*alg.hash.name {
                    ALG_SHA1 => "HS1",
                    ALG_SHA256 => "HS256",
                    ALG_SHA384 => "HS384",
                    ALG_SHA512 => "HS512",
                    _ => return Err(Error::NotSupported(None)),
                },
                _ => return Err(Error::NotSupported(None)),
            };
            jwk.alg = Some(DOMString::from(hash_algorithm));

            // Step 4.7. Set the key_ops attribute of jwk to the usages attribute of key.
            jwk.set_key_ops(key.usages());

            // Step 4.8. Set the ext attribute of jwk to the [[extractable]] internal slot of key.
            jwk.ext = Some(key.Extractable());

            // Step 4.9. Let result be jwk.
            Ok(ExportedKey::Jwk(Box::new(jwk)))
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            Err(Error::NotSupported(None))
        },
    }
}

/// <https://w3c.github.io/webcrypto/#hmac-operations-get-key-length>
pub(crate) fn get_key_length(
    normalized_derived_key_algorithm: &SubtleHmacImportParams,
) -> Result<Option<u32>, Error> {
    // Step 1.
    let length = match normalized_derived_key_algorithm.length {
        // If the length member of normalizedDerivedKeyAlgorithm is not present:
        None => {
            // Let length be the block size in bits of the hash function identified by the hash
            // member of normalizedDerivedKeyAlgorithm.
            hash_function_block_size_in_bits(normalized_derived_key_algorithm.hash.name())?
        },
        // Otherwise, if the length member of normalizedDerivedKeyAlgorithm is non-zero:
        Some(length) if length != 0 => {
            // Let length be equal to the length member of normalizedDerivedKeyAlgorithm.
            length
        },
        // Otherwise:
        _ => {
            // throw a TypeError.
            return Err(Error::Type(c"[[length]] must not be zero".to_owned()));
        },
    };

    // Step 2. Return length.
    Ok(Some(length))
}

/// Return the block size in bits of a hash function, according to Figure 1 of
/// <https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf>.
fn hash_function_block_size_in_bits(hash: &str) -> Result<u32, Error> {
    match hash {
        ALG_SHA1 => Ok(512),
        ALG_SHA256 => Ok(512),
        ALG_SHA384 => Ok(1024),
        ALG_SHA512 => Ok(1024),
        _ => Err(Error::NotSupported(Some(
            "Unidentified hash member".to_string(),
        ))),
    }
}
