/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use aws_lc_rs::hmac;
use js::jsval::ObjectValue;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::CryptoKeyMethods;
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::HmacKeyAlgorithm;
use crate::dom::bindings::error::Error;
use crate::dom::cryptokey::CryptoKey;
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{
    ALG_SHA1, ALG_SHA256, ALG_SHA384, ALG_SHA512, value_from_js_object,
};
use crate::script_runtime::CanGc;

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
