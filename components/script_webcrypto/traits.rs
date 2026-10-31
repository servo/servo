/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::DomTypes;
#[cfg(feature = "subtlecrypto")]
use script_bindings::reflector::DomObject;
#[cfg(feature = "subtlecrypto")]
use script_bindings::tasks::TaskOnce;

use crate::crypto::Crypto;
#[cfg(feature = "subtlecrypto")]
use crate::cryptokey::CryptoKey;
#[cfg(feature = "subtlecrypto")]
use crate::subtlecrypto::SubtleCrypto;

// This trait enforces the equivalence of all local types with the types in DomTypes.
#[cfg(feature = "subtlecrypto")]
trait_set::trait_set! {
    pub trait Equivalence = DomTypes<
        Crypto = Crypto<Self>,
        CryptoKey = CryptoKey<Self>,
        SubtleCrypto = SubtleCrypto<Self>,
        // General Bounds
        GlobalScope: WebCryptoGlobalTrait<Self>>;
}

#[cfg(not(feature = "subtlecrypto"))]
trait_set::trait_set! {
    pub trait Equivalence = DomTypes<Crypto = Crypto<Self>>;
}

#[cfg(feature = "subtlecrypto")]
pub trait WebCryptoGlobalTrait<D: DomTypes>: Sized + DomObject {
    fn queue_crypto_task_source(&self, task: impl TaskOnce + 'static);
    fn queue_dom_manipulation_task_source(&self, task: impl TaskOnce + 'static);
}
