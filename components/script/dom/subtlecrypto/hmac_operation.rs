/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use aws_lc_rs::hmac;
use js::jsapi::JS_NewObject;
use js::jsval::ObjectValue;
use servo_rand::{RngCore, ServoRng};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{
    CryptoKeyMethods, KeyType, KeyUsage,
};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::HmacKeyAlgorithm;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_HMAC, ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, AlgorithmFromLengthAndHash,
    SubtleHmacKeyGenParams, value_from_js_object,
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
    let hash_algorithm = match params.hash.name.str() {
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
    let hash_algorithm = match params.hash.name.str() {
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
