/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use aws_lc_rs::hmac;
use base64::prelude::*;
use js::jsapi::JS_NewObject;
use js::jsval::ObjectValue;
use servo_rand::{RngCore, ServoRng};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{
    HmacKeyAlgorithm, JsonWebKey, KeyFormat,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_HMAC, ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, AlgorithmFromLengthAndHash,
    ExportedKey, JsonWebKeyExt, SubtleHmacImportParams, SubtleHmacKeyGenParams,
    value_from_js_object,
};
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/webcrypto/#hmac-operations-sign>
pub(crate) fn sign(key: &CryptoKey, message: &[u8], can_gc: CanGc) -> Result<Vec<u8>, Error> {
    // Step 1. Let mac be the result of performing the MAC Generation operation described in
    // Section 4 of [FIPS-198-1] using the key represented by the [[handle]] internal slot of key,
    // the hash function identified by the hash attribute of the [[algorithm]] internal slot of key
    // and message as the input data text.
    let cx = GlobalScope::get_cx();
    rooted!(in(*cx) let mut algorithm_slot = ObjectValue(key.Algorithm(cx).as_ptr()));
    let params = value_from_js_object::<HmacKeyAlgorithm>(cx, algorithm_slot.handle(), can_gc)?;
    let hash_algorithm = match &*params.hash.name.str() {
        ALG_SHA1 => hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
        ALG_SHA256 => hmac::HMAC_SHA256,
        ALG_SHA384 => hmac::HMAC_SHA384,
        ALG_SHA512 => hmac::HMAC_SHA512,
        _ => return Err(Error::NotSupported),
    };
    let sign_key = hmac::Key::new(hash_algorithm, key.handle().as_bytes());
    let mac = hmac::sign(&sign_key, message);

    // Step 2. Return mac.
    Ok(mac.as_ref().to_vec())
}

/// <https://w3c.github.io/webcrypto/#hmac-operations-verify>
pub(crate) fn verify(
    key: &CryptoKey,
    message: &[u8],
    signature: &[u8],
    can_gc: CanGc,
) -> Result<bool, Error> {
    // Step 1. Let mac be the result of performing the MAC Generation operation described in
    // Section 4 of [FIPS-198-1] using the key represented by the [[handle]] internal slot of key,
    // the hash function identified by the hash attribute of the [[algorithm]] internal slot of key
    // and message as the input data text.
    let cx = GlobalScope::get_cx();
    rooted!(in(*cx) let mut algorithm_slot = ObjectValue(key.Algorithm(cx).as_ptr()));
    let params = value_from_js_object::<HmacKeyAlgorithm>(cx, algorithm_slot.handle(), can_gc)?;
    let hash_algorithm = match &*params.hash.name.str() {
        ALG_SHA1 => hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
        ALG_SHA256 => hmac::HMAC_SHA256,
        ALG_SHA384 => hmac::HMAC_SHA384,
        ALG_SHA512 => hmac::HMAC_SHA512,
        _ => return Err(Error::NotSupported),
    };
    let sign_key = hmac::Key::new(hash_algorithm, key.handle().as_bytes());
    let mac = hmac::sign(&sign_key, message);

    // Step 2. Return true if mac is equal to signature and false otherwise.
    Ok(mac.as_ref() == signature)
}

/// <https://w3c.github.io/webcrypto/#hmac-operations-generate-key>
#[allow(unsafe_code)]
pub(crate) fn generate_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleHmacKeyGenParams,
    extractable: bool,
    usages: Vec<KeyUsage>,
    rng: &DomRefCell<ServoRng>,
    can_gc: CanGc,
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
            normalized_algorithm.hash.block_size_in_bits()?
        },
        // Otherwise, if the length member of normalizedAlgorithm is non-zero:
        Some(length) if length != 0 => {
            // Let length be equal to the length member of normalizedAlgorithm.
            length
        },
        // Otherwise:
        _ => {
            // throw an OperationError.
            return Err(Error::Operation);
        },
    };

    // Step 3. Generate a key of length length bits.
    let mut key_data = vec![0; length as usize];
    rng.borrow_mut().fill_bytes(&mut key_data);

    // Step 4. If the key generation step fails, then throw an OperationError.
    // NOTE: Our key generation is infallible.

    // Step 6. Let algorithm be a new HmacKeyAlgorithm.
    // Step 7. Set the name attribute of algorithm to "HMAC".
    // Step 8. Set the length attribute of algorithm to length.
    // Step 9. Let hash be a new KeyAlgorithm.
    // Step 10. Set the name attribute of hash to equal the name member of the hash member of
    // normalizedAlgorithm.
    let name = DOMString::from(ALG_HMAC);
    let cx = GlobalScope::get_cx();
    rooted!(in(*cx) let mut algorithm_object = unsafe {JS_NewObject(*cx, ptr::null()) });
    HmacKeyAlgorithm::from_length_and_hash(
        length,
        &normalized_algorithm.hash.borrow_arc(),
        algorithm_object.handle_mut(),
        cx,
    );

    // Step 5. Let key be a new CryptoKey object representing the generated key.
    // Step 11. Set the hash attribute of algorithm to hash.
    // Step 12. Set the [[type]] internal slot of key to "secret".
    // Step 13. Set the [[algorithm]] internal slot of key to algorithm.
    // Step 14. Set the [[extractable]] internal slot of key to be extractable.
    // Step 15. Set the [[usages]] internal slot of key to be usages.
    let key = CryptoKey::new(
        global,
        KeyType::Secret,
        extractable,
        name,
        algorithm_object.handle(),
        usages,
        Handle::Hmac(key_data),
        can_gc,
    );

    // Step 16. Return key.
    Ok(key)
}

/// <https://w3c.github.io/webcrypto/#hmac-operations-import-key>
#[allow(unsafe_code)]
pub(crate) fn import_key(
    global: &GlobalScope,
    normalized_algorithm: &SubtleHmacImportParams,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
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
        KeyFormat::Raw => {
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
            let jwk = JsonWebKey::parse(GlobalScope::get_cx(), key_data)?;

            // Step 2.2. If the kty field of jwk is not "oct", then throw a DataError.
            if jwk.kty.as_ref().is_none_or(|kty| kty != "oct") {
                return Err(Error::Data);
            }

            // Step 2.3. If jwk does not meet the requirements of Section 6.4 of JSON Web
            // Algorithms [JWA], then throw a DataError.
            // NOTE: Done by Step 2.4 and 2.6.

            // Step 2.4. Let data be the byte sequence obtained by decoding the k field of jwk.
            data = base64::engine::general_purpose::STANDARD_NO_PAD
                .decode(&*jwk.k.as_ref().ok_or(Error::Data)?.as_bytes())
                .map_err(|_| Error::Data)?;

            // Step 2.5. Set the hash to equal the hash member of normalizedAlgorithm.
            hash = &normalized_algorithm.hash;

            // Step 2.6.
            match hash.name() {
                // If the name attribute of hash is "SHA-1":
                ALG_SHA1 => {
                    // If the alg field of jwk is present and is not "HS1", then throw a DataError.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != "HS1") {
                        return Err(Error::Data);
                    }
                },
                // If the name attribute of hash is "SHA-256":
                ALG_SHA256 => {
                    // If the alg field of jwk is present and is not "HS256", then throw a DataError.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != "HS256") {
                        return Err(Error::Data);
                    }
                },
                // If the name attribute of hash is "SHA-384":
                ALG_SHA384 => {
                    // If the alg field of jwk is present and is not "HS384", then throw a DataError.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != "HS384") {
                        return Err(Error::Data);
                    }
                },
                // If the name attribute of hash is "SHA-512":
                ALG_SHA512 => {
                    // If the alg field of jwk is present and is not "HS512", then throw a DataError.
                    if jwk.alg.as_ref().is_some_and(|alg| alg != "HS512") {
                        return Err(Error::Data);
                    }
                },
                // Otherwise,
                _name => {
                    // if the name attribute of hash is defined in another applicable specification:
                    // Perform any key import steps defined by other applicable specifications,
                    // passing format, jwk and hash and obtaining hash
                    // NOTE: Currently not support applicable specification.
                    return Err(Error::NotSupported);
                },
            }

            // Step 2.7. If usages is non-empty and the use field of jwk is present and is not
            // "sig", then throw a DataError.
            if !usages.is_empty() && jwk.use_.as_ref().is_some_and(|use_| use_ != "sig") {
                return Err(Error::Data);
            }

            // Step 2.8. If the key_ops field of jwk is present, and is invalid according to
            // the requirements of JSON Web Key [JWK] or does not contain all of the specified
            // usages values, then throw a DataError.
            jwk.check_key_ops(&usages)?;

            // Step 2.9. If the ext field of jwk is present and has the value false and
            // extractable is true, then throw a DataError.
            if jwk.ext.is_some_and(|ext| !ext) && extractable {
                return Err(Error::Data);
            }
        },
        // Otherwise:
        _ => {
            // throw a NotSupportedError.
            return Err(Error::NotSupported);
        },
    }

    // Step 5. Let length be the length in bits of data.
    let mut length = data.len() as u32 * 8;

    // Step 6. If length is zero then throw a DataError.
    if length == 0 {
        return Err(Error::Data);
    }

    // Step 7. If the length member of normalizedAlgorithm is present:
    if let Some(given_length) = normalized_algorithm.length {
        //  If the length member of normalizedAlgorithm is greater than length:
        if given_length > length {
            // throw a DataError.
            return Err(Error::Data);
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
    // Step 14. Set the [[algorithm]] internal slot of key to algorithm.
    let truncated_data = data[..length as usize / 8].to_vec();
    let name = DOMString::from(ALG_HMAC);
    let cx = GlobalScope::get_cx();
    rooted!(in(*cx) let mut algorithm_object = unsafe { JS_NewObject(*cx, ptr::null()) });
    assert!(!algorithm_object.is_null());
    HmacKeyAlgorithm::from_length_and_hash(length, hash, algorithm_object.handle_mut(), cx);
    let key = CryptoKey::new(
        global,
        KeyType::Secret,
        extractable,
        name,
        algorithm_object.handle(),
        usages,
        Handle::Hmac(truncated_data),
        can_gc,
    );

    // Step 15. Return key.
    Ok(key)
}

/// <https://w3c.github.io/webcrypto/#hmac-operations-export-key>
pub(crate) fn export(format: KeyFormat, key: &CryptoKey) -> Result<ExportedKey, Error> {
    match format {
        KeyFormat::Raw => match key.handle() {
            Handle::Hmac(key_data) => Ok(ExportedKey::Raw(key_data.as_slice().to_vec())),
            _ => Err(Error::Operation),
        },
        // FIXME: Implement JWK export for HMAC keys
        _ => Err(Error::NotSupported),
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
            match normalized_derived_key_algorithm.hash.name() {
                ALG_SHA1 => 160,
                ALG_SHA256 => 256,
                ALG_SHA384 => 384,
                ALG_SHA512 => 512,
                _ => {
                    return Err(Error::Type("Unidentified hash member".to_string()));
                },
            }
        },
        // Otherwise, if the length member of normalizedDerivedKeyAlgorithm is non-zero:
        Some(length) if length != 0 => {
            // Let length be equal to the length member of normalizedDerivedKeyAlgorithm.
            length
        },
        // Otherwise:
        _ => {
            // throw a TypeError.
            return Err(Error::Type("[[length]] must not be zero".to_string()));
        },
    };

    // Step 2. Return length.
    Ok(Some(length))
}
