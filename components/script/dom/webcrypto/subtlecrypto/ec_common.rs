/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;

use crate::dom::bindings::codegen::Bindings::CryptoKeyBinding::{KeyType, KeyUsage};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::cryptokey::{CryptoKey, Handle};
use crate::dom::globalscope::GlobalScope;
use crate::dom::subtlecrypto::KeyAlgorithmAndDerivatives;

/// <https://wicg.github.io/webcrypto-modern-algos/#SubtleCrypto-method-getPublicKey>
/// Step 9 - 15, for elliptic curve cryptography
pub(crate) fn get_public_key(
    cx: &mut JSContext,
    global: &GlobalScope,
    key: &CryptoKey,
    algorithm: &KeyAlgorithmAndDerivatives,
    usages: Vec<KeyUsage>,
) -> Result<DomRoot<CryptoKey>, Error> {
    // Step 9. If usages contains an entry which is not supported for a public key by the algorithm
    // identified by algorithm, then throw a SyntaxError.
    //
    // NOTE: See "importKey" operation for supported usages
    if usages.iter().any(|usage| *usage != KeyUsage::Verify) {
        return Err(Error::Syntax(Some(
            "Usages contains an entry which is not \"verify\"".to_string(),
        )));
    }

    // Step 10. Let publicKey be a new CryptoKey representing the public key corresponding to the
    // private key represented by the [[handle]] internal slot of key.
    // Step 11. If an error occurred, then throw a OperationError.
    // Step 12. Set the [[type]] internal slot of publicKey to "public".
    // Step 13. Set the [[algorithm]] internal slot of publicKey to algorithm.
    // Step 14. Set the [[extractable]] internal slot of publicKey to true.
    // Step 15. Set the [[usages]] internal slot of publicKey to usages.
    let public_key_handle = match key.handle() {
        Handle::P256PrivateKey(private_key) => Handle::P256PublicKey(private_key.public_key()),
        Handle::P384PrivateKey(private_key) => Handle::P384PublicKey(private_key.public_key()),
        Handle::P521PrivateKey(private_key) => Handle::P521PublicKey(private_key.public_key()),
        _ => {
            return Err(Error::Operation(Some(
                "[[handle]] internal slot of key is not an elliptic curve private key".to_string(),
            )));
        },
    };
    let public_key = CryptoKey::new(
        cx,
        global,
        KeyType::Public,
        true,
        algorithm.clone(),
        usages,
        public_key_handle,
    );

    Ok(public_key)
}
