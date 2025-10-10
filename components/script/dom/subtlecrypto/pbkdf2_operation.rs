/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use js::jsapi::JS_NewObject;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{KeyType, KeyUsage};
use crate::dom::bindings::codegen::Bindings::SubtleCryptoBinding::{KeyAlgorithm, KeyFormat};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::{ALG_PBKDF2, AlgorithmFromName};
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/webcrypto/#pbkdf2-operations-import-key>
#[allow(unsafe_code)]
pub(crate) fn import(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. If format is not "raw", throw a NotSupportedError
    if format != KeyFormat::Raw {
        return Err(Error::NotSupported);
    }

    // Step 2. If usages contains a value that is not "deriveKey" or "deriveBits", then throw a SyntaxError.
    if usages
        .iter()
        .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits)) ||
        usages.is_empty()
    {
        return Err(Error::Syntax(None));
    }

    // Step 3. If extractable is not false, then throw a SyntaxError.
    if extractable {
        return Err(Error::Syntax(None));
    }

    // Step 4. Let key be a new CryptoKey representing keyData.
    // Step 5. Set the [[type]] internal slot of key to "secret".
    // Step 6. Let algorithm be a new KeyAlgorithm object.
    // Step 7. Set the name attribute of algorithm to "PBKDF2".
    // Step 8. Set the [[algorithm]] internal slot of key to algorithm.
    let name = DOMString::from(ALG_PBKDF2);
    let cx = GlobalScope::get_cx();
    rooted!(in(*cx) let mut algorithm_object = unsafe {JS_NewObject(*cx, ptr::null()) });
    assert!(!algorithm_object.is_null());
    KeyAlgorithm::from_name(name.clone(), algorithm_object.handle_mut(), cx);

    let key = CryptoKey::new(
        global,
        KeyType::Secret,
        extractable,
        name,
        algorithm_object.handle(),
        usages,
        Handle::Pbkdf2(key_data.to_vec()),
        can_gc,
    );

    // Step 9. Return key.
    Ok(key)
}
