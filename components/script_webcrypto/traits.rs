/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::DomTypes;
use script_bindings::reflector::DomObject;
use script_bindings::tasks::TaskOnce;

use crate::crypto::Crypto;
use crate::cryptokey::CryptoKey;
use crate::subtlecrypto::SubtleCrypto;

// This trait enforces the equivalence of all local types with the types in DomTypes.
trait_set::trait_set! {
    pub trait Equivalence = DomTypes<
        Crypto = Crypto<Self>,
        CryptoKey = CryptoKey<Self>,
        SubtleCrypto = SubtleCrypto<Self>,
        // General Bounds
        GlobalScope: WebCryptoGlobalTrait<Self>>;
}

pub trait WebCryptoGlobalTrait<D: DomTypes>: Sized + DomObject {
    fn queue_crypto_task_source(&self, task: impl TaskOnce + 'static);
    fn queue_dom_manipulation_task_source(&self, task: impl TaskOnce + 'static);
}
