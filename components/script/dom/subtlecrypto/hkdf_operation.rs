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
use crate::dom::subtlecrypto::{ALG_HKDF, AlgorithmFromName};
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/webcrypto/#hkdf-operations-import-key>
#[allow(unsafe_code)]
pub(crate) fn import(
    global: &GlobalScope,
    format: KeyFormat,
    key_data: &[u8],
    extractable: bool,
    usages: Vec<KeyUsage>,
    can_gc: CanGc,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 1. Let keyData be the key data to be imported.

    // Step 2. If format is "raw":
    if format == KeyFormat::Raw {
        // Step 2.1. If usages contains a value that is not "deriveKey" or "deriveBits", then throw
        // a SyntaxError.
        if usages
            .iter()
            .any(|usage| !matches!(usage, KeyUsage::DeriveKey | KeyUsage::DeriveBits)) ||
            usages.is_empty()
        {
            return Err(Error::Syntax(None));
        }

        // Step 2.2. If extractable is not false, then throw a SyntaxError.
        if extractable {
            return Err(Error::Syntax(None));
        }

        // Step 2.3. Let key be a new CryptoKey representing the key data provided in keyData.
        // Step 2.4. Set the [[type]] internal slot of key to "secret".
        // Step 2.5. Let algorithm be a new KeyAlgorithm object.
        // Step 2.6. Set the name attribute of algorithm to "HKDF".
        // Step 2.7. Set the [[algorithm]] internal slot of key to algorithm.
        let name = DOMString::from(ALG_HKDF);
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
            Handle::Hkdf(key_data.to_vec()),
            can_gc,
        );

        // Step 2.8. Return key.
        Ok(key)
    }
    // Otherwise:
    else {
        // throw a NotSupportedError.
        Err(Error::NotSupported)
    }
}
